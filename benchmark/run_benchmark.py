#!/usr/bin/env python3
"""Compression-accuracy benchmark for skill-compress.

Runs the deterministic minifier across its compression modes on a corpus of
SKILL.md files, measures fidelity (must-preserve atoms retained via --verify)
against size/token reduction, and renders a self-contained HTML report.

Reproducible and offline: it only shells out to `cargo run` (no network, no LLM
judge). Regenerate with `make benchmark`.

stdlib only — no third-party deps.
"""

from __future__ import annotations

import html
import json
import shutil
import subprocess
import sys
from dataclasses import dataclass, field
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
OUT_DIR = ROOT / "output"
REPORT = OUT_DIR / "benchmark.html"

# (label, description, extra CLI flags) for each compression mode under test.
MODES: list[tuple[str, str, list[str]]] = [
    (
        "Deterministic min",
        "Default conservative cleanup: strips decorative blanks, references "
        "duplicate fenced blocks and business rules. Lossless by design.",
        [],
    ),
    (
        "Runtime-only",
        "Aggressive bundle (--runtime-only): additionally drops changelog, "
        "examples, and meta prose — optional sections not needed at runtime.",
        ["--runtime-only"],
    ),
]

# Corpus: every *.md under examples/ and input/ is treated as a benchmark input.
# examples/ holds the bundled sample; input/ holds real-world skills (git-ignored).
CORPUS = sorted((ROOT / "examples").glob("*.md")) + sorted((ROOT / "input").glob("*.md"))


@dataclass
class ModeResult:
    label: str
    description: str
    before: dict
    after: dict
    preserved: int
    total: int
    missing_by_kind: dict[str, int] = field(default_factory=dict)

    @property
    def fidelity(self) -> float:
        return 100.0 * self.preserved / self.total if self.total else 100.0

    @property
    def missing(self) -> int:
        return self.total - self.preserved

    def reduction(self, key: str) -> float:
        b, a = self.before[key], self.after[key]
        return 100.0 * (b - a) / b if b else 0.0


@dataclass
class FileResult:
    path: str
    before: dict
    modes: list[ModeResult]


def cargo(args: list[str]) -> str:
    """Run `cargo run -q -- <args>` and return stdout (verify exits 1 on drift)."""
    proc = subprocess.run(
        ["cargo", "run", "-q", "--", *args],
        cwd=ROOT,
        capture_output=True,
        text=True,
    )
    return proc.stdout


def analyze(source: Path, flags: list[str]) -> dict:
    return json.loads(cargo([*flags, "--report", "json", str(source)]))


def verify(source: Path, candidate: Path) -> dict:
    return json.loads(
        cargo([str(source), "--verify", str(candidate), "--report", "json"])
    )


def bench_file(source: Path) -> FileResult:
    before = analyze(source, [])["metrics"]["before"]
    results: list[ModeResult] = []
    for label, desc, flags in MODES:
        candidate = OUT_DIR / f"bench-{source.stem}-{label.lower().replace(' ', '-')}.md"
        shutil.copyfile(source, candidate)
        cargo([*flags, "--write", str(candidate)])
        after = analyze(source, flags)["metrics"]["after"]
        v = verify(source, candidate)
        by_kind: dict[str, int] = {}
        for atom in v["missing"]:
            by_kind[atom["kind"]] = by_kind.get(atom["kind"], 0) + 1
        results.append(
            ModeResult(label, desc, before, after, v["preserved"], v["total"], by_kind)
        )
    return FileResult(str(source.relative_to(ROOT)), before, results)


# --------------------------------------------------------------------------- #
# HTML rendering (Carbon IBM palette, theme-aware, self-contained)
# --------------------------------------------------------------------------- #

CSS = """
:root {
  --primary:#0F62FE; --primary-bg:#EDF4FF; --secondary-bg:#DAE8FC;
  --success:#24A148; --success-bg:#C8E6C9; --info-bg:#E1F5FE;
  --highlight:#D9A100; --highlight-bg:#FFF3E0; --accent:#B85450; --accent-bg:#F8CECC;
  --neutral:#F5F5F5; --neutral-border:#DDE1E6; --text:#161616; --muted:#525252;
  --card:#ffffff; --bg:#f4f4f4; --track:#E0E7F3;
}
@media (prefers-color-scheme: dark) {
  :root:not([data-theme="light"]) {
    --primary:#78A9FF; --primary-bg:#1C2A44; --secondary-bg:#22314F;
    --success:#42BE65; --success-bg:#213524; --info-bg:#1B2A38;
    --highlight:#F1C21B; --highlight-bg:#3A2F14; --accent:#FA8775; --accent-bg:#3A2321;
    --neutral:#262626; --neutral-border:#393939; --text:#F4F4F4; --muted:#A8A8A8;
    --card:#1C1C1C; --bg:#121212; --track:#2A3247;
  }
}
:root[data-theme="dark"] {
  --primary:#78A9FF; --primary-bg:#1C2A44; --secondary-bg:#22314F;
  --success:#42BE65; --success-bg:#213524; --info-bg:#1B2A38;
  --highlight:#F1C21B; --highlight-bg:#3A2F14; --accent:#FA8775; --accent-bg:#3A2321;
  --neutral:#262626; --neutral-border:#393939; --text:#F4F4F4; --muted:#A8A8A8;
  --card:#1C1C1C; --bg:#121212; --track:#2A3247;
}
* { box-sizing:border-box; }
body {
  margin:0; background:var(--bg); color:var(--text);
  font:15px/1.55 -apple-system,BlinkMacSystemFont,"Segoe UI",Roboto,Helvetica,Arial,sans-serif;
}
.wrap { max-width:1040px; margin:0 auto; padding:40px 24px 64px; }
header h1 { font-size:1.8rem; margin:0 0 6px; letter-spacing:-.02em; }
header .sub { color:var(--muted); margin:0 0 4px; }
.badge { display:inline-block; font-size:.72rem; font-weight:600; padding:2px 8px;
  border-radius:99px; background:var(--primary-bg); color:var(--primary);
  border:1px solid var(--primary); }
h2 { font-size:1.15rem; margin:40px 0 14px; padding-bottom:6px;
  border-bottom:2px solid var(--primary); }
.cards { display:grid; grid-template-columns:repeat(auto-fit,minmax(220px,1fr)); gap:14px; }
.card { background:var(--card); border:1px solid var(--neutral-border);
  border-radius:10px; padding:16px 18px; }
.card .k { font-size:.75rem; text-transform:uppercase; letter-spacing:.04em; color:var(--muted); }
.card .v { font-size:1.9rem; font-weight:700; margin-top:2px; letter-spacing:-.02em; }
.card .note { font-size:.8rem; color:var(--muted); margin-top:2px; }
.scroll { overflow-x:auto; }
table { border-collapse:collapse; width:100%; min-width:560px; background:var(--card);
  border:1px solid var(--neutral-border); border-radius:10px; overflow:hidden; }
th,td { padding:11px 14px; text-align:left; border-bottom:1px solid var(--neutral-border); }
th { background:var(--primary-bg); color:var(--primary); font-size:.78rem;
  text-transform:uppercase; letter-spacing:.03em; }
td.num, th.num { text-align:right; font-variant-numeric:tabular-nums; }
tr:last-child td { border-bottom:none; }
.pill { display:inline-block; padding:1px 9px; border-radius:99px; font-size:.78rem; font-weight:600; }
.pill.ok { background:var(--success-bg); color:var(--success); }
.pill.warn { background:var(--highlight-bg); color:var(--highlight); }
.bar-row { display:grid; grid-template-columns:150px 1fr 64px; align-items:center; gap:12px; margin:9px 0; }
.bar-label { font-size:.85rem; color:var(--muted); }
.track { background:var(--track); border-radius:99px; height:16px; overflow:hidden; }
.fill { height:100%; border-radius:99px; }
.fill.fid { background:var(--success); }
.fill.comp { background:var(--primary); }
.bar-val { font-size:.82rem; font-variant-numeric:tabular-nums; text-align:right; color:var(--muted); }
.mode-card { background:var(--card); border:1px solid var(--neutral-border);
  border-radius:10px; padding:18px 20px; margin:14px 0; }
.mode-card h3 { margin:0 0 4px; font-size:1.05rem; }
.mode-card p.desc { color:var(--muted); font-size:.88rem; margin:0 0 12px; }
.kinds { display:flex; flex-wrap:wrap; gap:8px; margin-top:10px; }
.kind { font-size:.78rem; background:var(--accent-bg); color:var(--accent);
  border-radius:6px; padding:3px 9px; }
.callout { background:var(--highlight-bg); border-left:4px solid var(--highlight);
  border-radius:0 8px 8px 0; padding:12px 16px; margin:18px 0; font-size:.9rem; }
footer { margin-top:44px; color:var(--muted); font-size:.8rem;
  border-top:1px solid var(--neutral-border); padding-top:16px; }
code { background:var(--neutral); padding:1px 6px; border-radius:5px; font-size:.85em; }
"""


def fmt(n: float, digits: int = 1) -> str:
    return f"{n:,.{digits}f}"


def bar(label: str, pct: float, kind: str, value_txt: str) -> str:
    width = max(0.0, min(100.0, pct))
    return (
        f'<div class="bar-row"><div class="bar-label">{html.escape(label)}</div>'
        f'<div class="track"><div class="fill {kind}" style="width:{width:.1f}%"></div></div>'
        f'<div class="bar-val">{html.escape(value_txt)}</div></div>'
    )


def render(files: list[FileResult]) -> str:
    # Aggregate hero stats over the primary (first) corpus file.
    primary = files[0]
    lossless = next((m for m in primary.modes if m.missing == 0), primary.modes[0])
    aggressive = max(primary.modes, key=lambda m: m.reduction("estimated_tokens"))

    cards = "".join(
        f'<div class="card"><div class="k">{k}</div><div class="v">{v}</div>'
        f'<div class="note">{note}</div></div>'
        for k, v, note in [
            ("Corpus files", str(len(files)), "under examples/"),
            ("Must-preserve atoms", f"{primary.modes[0].total}", "headings · rules · code"),
            (
                "Best lossless",
                f"{fmt(lossless.reduction('estimated_tokens'))}%",
                f"tokens saved @ {fmt(lossless.fidelity, 0)}% fidelity ({lossless.label})",
            ),
            (
                "Max compression",
                f"{fmt(aggressive.reduction('estimated_tokens'))}%",
                f"tokens saved @ {fmt(aggressive.fidelity)}% fidelity ({aggressive.label})",
            ),
        ]
    )

    file_sections = []
    for fr in files:
        rows = []
        mode_cards = []
        for m in fr.modes:
            fid_cls = "ok" if m.missing == 0 else "warn"
            rows.append(
                f"<tr><td>{html.escape(m.label)}</td>"
                f'<td class="num"><span class="pill {fid_cls}">{m.preserved}/{m.total}'
                f" · {fmt(m.fidelity)}%</span></td>"
                f'<td class="num">{m.after["lines"]:,}</td>'
                f'<td class="num">{m.after["chars"]:,}</td>'
                f'<td class="num">{m.after["estimated_tokens"]:,}</td>'
                f'<td class="num">{fmt(m.reduction("chars"))}%</td>'
                f'<td class="num">{fmt(m.reduction("estimated_tokens"))}%</td></tr>'
            )
            kinds = "".join(
                f'<span class="kind">{html.escape(k)}: {n}</span>'
                for k, n in sorted(m.missing_by_kind.items())
            )
            kinds_block = f'<div class="kinds">{kinds}</div>' if kinds else (
                '<div class="kinds"><span class="kind" '
                'style="background:var(--success-bg);color:var(--success)">no atoms dropped</span></div>'
            )
            mode_cards.append(
                f'<div class="mode-card"><h3>{html.escape(m.label)} '
                f'<span class="pill {fid_cls}">{fmt(m.fidelity)}% fidelity</span></h3>'
                f'<p class="desc">{html.escape(m.description)}</p>'
                f"<div>{''.join(bars_for(m))}</div>{kinds_block}</div>"
            )

        b = fr.before
        file_sections.append(
            f'<h2>{html.escape(fr.path)}</h2>'
            f'<p class="sub" style="color:var(--muted);margin-top:-6px">'
            f'Original: {b["lines"]:,} lines · {b["chars"]:,} chars · '
            f'~{b["estimated_tokens"]:,} est. tokens</p>'
            f'<div class="scroll"><table><thead><tr>'
            f"<th>Mode</th><th class=num>Fidelity</th><th class=num>Lines</th>"
            f"<th class=num>Chars</th><th class=num>Est. tokens</th>"
            f"<th class=num>Char ↓</th><th class=num>Token ↓</th></tr></thead>"
            f"<tbody>{''.join(rows)}</tbody></table></div>"
            f'{"".join(mode_cards)}'
        )

    callout = (
        '<div class="callout"><strong>Reading the numbers.</strong> '
        "Fidelity is measured by the deterministic gate (<code>--verify</code>): the share of "
        "the original's must-preserve atoms — frontmatter keys, headings, rule/acceptance "
        "bullets, and fenced code — that survive verbatim. The <em>min</em> mode is lossless "
        "by construction; <em>runtime-only</em>'s dropped atoms are optional sections it removes "
        "on purpose (changelog entries, examples, meta prose), not fidelity regressions.</div>"
    )

    return (
        "<div class='wrap'><header>"
        "<span class='badge'>skill-compress</span>"
        "<h1>Compression Accuracy Benchmark</h1>"
        "<p class='sub'>Fidelity vs. size reduction across deterministic compression modes. "
        "Offline &amp; reproducible — <code>make benchmark</code>.</p>"
        "</header>"
        f"<h2>Summary</h2><div class='cards'>{cards}</div>"
        f"{callout}"
        f"{''.join(file_sections)}"
        "<footer>Generated by <code>benchmark/run_benchmark.py</code> from live "
        "<code>cargo run</code> output. No network or LLM judge involved. "
        "Palette: Carbon IBM.</footer>"
        "</div>"
    )


def bars_for(m: ModeResult) -> list[str]:
    return [
        bar("fidelity", m.fidelity, "fid", f"{fmt(m.fidelity)}%"),
        bar("chars saved", m.reduction("chars"), "comp", f"{fmt(m.reduction('chars'))}%"),
        bar("tokens saved", m.reduction("estimated_tokens"), "comp",
            f"{fmt(m.reduction('estimated_tokens'))}%"),
        bar("lines saved", m.reduction("lines"), "comp", f"{fmt(m.reduction('lines'))}%"),
    ]


PAGE = """<!doctype html>
<html lang="en"><head><meta charset="utf-8">
<meta name="viewport" content="width=device-width,initial-scale=1">
<title>skill-compress — Compression Accuracy Benchmark</title>
<style>{css}</style></head><body>{body}</body></html>
"""


def main() -> int:
    if not CORPUS:
        print("no corpus files under examples/*.md", file=sys.stderr)
        return 1
    OUT_DIR.mkdir(exist_ok=True)
    print(f"Building crate & benchmarking {len(CORPUS)} file(s)...", file=sys.stderr)
    files = [bench_file(p) for p in CORPUS]
    REPORT.write_text(PAGE.format(css=CSS, body=render(files)), encoding="utf-8")
    for fr in files:
        for m in fr.modes:
            print(
                f"  {fr.path:28} {m.label:20} "
                f"{m.preserved}/{m.total} atoms  "
                f"tokens -{m.reduction('estimated_tokens'):.1f}%",
                file=sys.stderr,
            )
    print(f"\nWrote {REPORT.relative_to(ROOT)}", file=sys.stderr)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
