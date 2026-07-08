# Development targets for skill-compress
# Run `make help` to list everything

BLUE  := \033[34m
GREEN := \033[32m
YELLOW:= \033[33m
RED   := \033[31m
RESET := \033[0m

# Where to relocate ./target/ when `make setup-target` runs.
CARGO_PKG_NAME := $(shell awk '/^\[package\]/ {in_package=1; next} /^\[/ {in_package=0} in_package && $$1=="name" {gsub(/"/, "", $$3); print $$3; exit}' Cargo.toml)
TARGET_CACHE_DIR ?= $(HOME)/.cache/cargo-targets/$(CARGO_PKG_NAME)

# Project constants for sample skill minification and analysis
SAMPLE_NAME := sample-skill
SAMPLE_INPUT := examples/$(SAMPLE_NAME).md
SAMPLE_OUTPUT := output/$(SAMPLE_NAME).min.md
SAMPLE_REPORT := output/$(SAMPLE_NAME).report.json
SAMPLE_DIFF := output/$(SAMPLE_NAME).diff
SAMPLE_RUNTIME_OUTPUT := output/$(SAMPLE_NAME).runtime.md
SAMPLE_RUNTIME_DIFF := output/$(SAMPLE_NAME).runtime.diff
# Candidate checked by sample-verify (override to audit any compressed candidate)
SAMPLE_VERIFY_CANDIDATE ?= $(SAMPLE_OUTPUT)
# OpenAI model for the advisory --verify-llm judge (sample-verify-llm target)
SAMPLE_LLM_MODEL ?= gpt-4o-mini

.PHONY: help fmt fmt-check lint lint-fix audit check build build-release run \
        run-release test verify clean setup-target teardown-target \
        sample sample-json sample-diff sample-runtime sample-runtime-diff sample-verify \
        sample-verify-llm sample-all benchmark

help: ## Show this help.
	@echo "Usage:"
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | sort | awk 'BEGIN {FS = ":.*?## "}; {printf "  $(YELLOW)%-14s$(RESET) %s\n", $$1, $$2}'

#########################
# Code hygiene
#########################

fmt: ## Format Rust code with rustfmt
	@echo "$(BLUE)Formatting Rust...$(RESET)"
	cargo fmt --all

fmt-check: ## Check Rust formatting without writing
	@echo "$(BLUE)Checking Rust formatting...$(RESET)"
	cargo fmt --all -- --check

lint: ## Run clippy across all targets and fail on warnings
	@echo "$(BLUE)Running clippy...$(RESET)"
	cargo clippy --all-targets --no-deps -- -D warnings

lint-fix: ## Apply clippy's auto-fixable suggestions
	@echo "$(BLUE)Applying clippy fixes...$(RESET)"
	cargo clippy --fix --all-targets --allow-dirty --allow-staged

audit: ## Run cargo-audit against the dependency tree. Auto-installs if missing
	@command -v cargo-audit >/dev/null || cargo install --locked cargo-audit
	@echo "$(BLUE)Running cargo audit...$(RESET)"
	cargo audit

#########################
# Build and run
#########################

check: ## Type-check the crate without producing an executable
	@echo "$(BLUE)Checking crate...$(RESET)"
	cargo check --all-targets

build: ## Build the crate in debug mode
	@echo "$(BLUE)Building debug binary...$(RESET)"
	cargo build

build-release: ## Build the optimized release binary
	@echo "$(BLUE)Building release binary...$(RESET)"
	cargo build --release

run: ## Run the CLI help locally
	@echo "$(BLUE)Running debug binary...$(RESET)"
	cargo run -- --help

run-release: ## Run the optimized CLI help locally
	@echo "$(BLUE)Running release binary...$(RESET)"
	cargo run --release -- --help

#########################
# Tests and verification
#########################

test: ## Run the Rust test suite
	@echo "$(BLUE)Running Rust tests...$(RESET)"
	cargo test

verify: ## Full local verification: check + clippy + fmt-check + tests + build
	@echo "$(BLUE)[1/5] Crate check...$(RESET)"
	cargo check --all-targets
	@echo ""
	@echo "$(BLUE)[2/5] Clippy...$(RESET)"
	cargo clippy --all-targets --no-deps -- -D warnings
	@echo ""
	@echo "$(BLUE)[3/5] Format check...$(RESET)"
	cargo fmt --all -- --check
	@echo ""
	@echo "$(BLUE)[4/5] Tests...$(RESET)"
	cargo test
	@echo ""
	@echo "$(BLUE)[5/5] Build...$(RESET)"
	cargo build
	@echo ""
	@echo "$(GREEN)✓ Verify passed.$(RESET)"

clean: ## Remove Cargo build artifacts
	@echo "$(BLUE)Cleaning Cargo artifacts...$(RESET)"
	@if [ -L target ]; then \
		dest=$$(readlink target); \
		if [ -n "$$dest" ] && [ -d "$$dest" ]; then \
			echo "$(BLUE)Preserving target symlink → $$dest$(RESET)"; \
			find "$$dest" -mindepth 1 -maxdepth 1 -exec rm -rf {} +; \
		else \
			echo "$(YELLOW)target symlink destination does not exist: $$dest$(RESET)"; \
		fi; \
	else \
		cargo clean; \
	fi

#########################
# Build cache relocation
#########################

setup-target: ## Move ./target/ to the configured cache directory and symlink it back
	@if [ -L target ] && [ "$$(readlink target)" = "$(TARGET_CACHE_DIR)" ]; then \
		echo "$(GREEN)✓ target → $(TARGET_CACHE_DIR) (already set up)$(RESET)"; \
	else \
		if [ -L target ]; then \
			echo "$(YELLOW)Replacing existing symlink ($$(readlink target))$(RESET)"; \
			rm target; \
		elif [ -d target ]; then \
			size=$$(du -sh target 2>/dev/null | cut -f1); \
			echo "$(BLUE)Moving existing target/ ($$size) to $(TARGET_CACHE_DIR)...$(RESET)"; \
			mkdir -p "$$(dirname "$(TARGET_CACHE_DIR)")"; \
			rm -rf "$(TARGET_CACHE_DIR)"; \
			mv target "$(TARGET_CACHE_DIR)"; \
		fi; \
		mkdir -p "$(TARGET_CACHE_DIR)"; \
		ln -s "$(TARGET_CACHE_DIR)" target; \
		echo "$(GREEN)✓ target → $(TARGET_CACHE_DIR)$(RESET)"; \
	fi

teardown-target: ## Remove the target/ symlink without deleting the cache
	@if [ -L target ]; then \
		dest=$$(readlink target); \
		echo "$(BLUE)Removing symlink: target → $$dest$(RESET)"; \
		rm target; \
		echo "$(YELLOW)Cache at $$dest preserved. Delete manually with: rm -rf $$dest$(RESET)"; \
	else \
		echo "target is not a symlink — nothing to do"; \
	fi

#########################
# Sample workflows
#########################

sample: ## Minify the sample skill into output/
	@mkdir -p output
	@echo "$(BLUE)Writing $(SAMPLE_OUTPUT) from $(SAMPLE_INPUT)...$(RESET)"
	cp "$(SAMPLE_INPUT)" "$(SAMPLE_OUTPUT)"
	cargo run -- --write "$(SAMPLE_OUTPUT)"

sample-json: ## Analyze the sample skill and save JSON report into output/
	@mkdir -p output
	@echo "$(BLUE)Writing $(SAMPLE_REPORT) from $(SAMPLE_INPUT)...$(RESET)"
	cargo run -- --report json "$(SAMPLE_INPUT)" > "$(SAMPLE_REPORT)"

sample-diff: ## Save deterministic cleanup diff for the sample skill into output/
	@mkdir -p output
	@echo "$(BLUE)Writing $(SAMPLE_DIFF) from $(SAMPLE_INPUT)...$(RESET)"
	cargo run -- --diff "$(SAMPLE_INPUT)" > "$(SAMPLE_DIFF)"

sample-runtime: ## Generate runtime-only compressed sample into output/
	@mkdir -p output
	@echo "$(BLUE)Writing $(SAMPLE_RUNTIME_OUTPUT) from $(SAMPLE_INPUT)...$(RESET)"
	cp "$(SAMPLE_INPUT)" "$(SAMPLE_RUNTIME_OUTPUT)"
	cargo run -- --runtime-only --write "$(SAMPLE_RUNTIME_OUTPUT)"

sample-runtime-diff: ## Save runtime-only compression diff for the sample skill into output/
	@mkdir -p output
	@echo "$(BLUE)Writing $(SAMPLE_RUNTIME_DIFF) from $(SAMPLE_INPUT)...$(RESET)"
	cargo run -- --runtime-only --diff "$(SAMPLE_INPUT)" > "$(SAMPLE_RUNTIME_DIFF)"

sample-verify: sample ## Verify a candidate preserves the sample's must-preserve atoms (default: the deterministic min).
	@echo "$(BLUE)Verifying $(SAMPLE_VERIFY_CANDIDATE) against $(SAMPLE_INPUT)...$(RESET)"
	cargo run -- "$(SAMPLE_INPUT)" --verify "$(SAMPLE_VERIFY_CANDIDATE)"

sample-verify-llm: sample ## Run the advisory OpenAI --verify-llm judge over the sample candidate (needs OPENAI_API_KEY).
	@if [ -z "$$OPENAI_API_KEY" ]; then \
		echo "$(RED)OPENAI_API_KEY is not set; export it before running make sample-verify-llm.$(RESET)"; \
		exit 4; \
	fi
	@echo "$(BLUE)Verifying $(SAMPLE_VERIFY_CANDIDATE) against $(SAMPLE_INPUT) with OpenAI judge ($(SAMPLE_LLM_MODEL))...$(RESET)"
	cargo run -- "$(SAMPLE_INPUT)" --verify "$(SAMPLE_VERIFY_CANDIDATE)" --verify-llm --provider openai --model "$(SAMPLE_LLM_MODEL)"

sample-all: sample sample-json sample-diff sample-runtime sample-runtime-diff ## Generate all deterministic sample outputs.

#########################
# Benchmark
#########################

benchmark: ## Render the compression-accuracy benchmark (fidelity vs size) to output/benchmark.html
	@mkdir -p output
	@echo "$(BLUE)Running compression-accuracy benchmark...$(RESET)"
	python3 benchmark/run_benchmark.py
	@echo "$(GREEN)✓ Open output/benchmark.html$(RESET)"
