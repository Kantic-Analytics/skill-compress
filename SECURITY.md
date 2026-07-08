# Security Policy

## Supported Versions

Security fixes target the current `main` branch until the project publishes versioned releases.

## Reporting a Vulnerability

Please do not open a public issue for vulnerabilities.

Report security concerns by contacting Kantic Analytics privately. If no dedicated security contact is available for your deployment, open a minimal public issue asking for a private disclosure channel without including exploit details.

## Sensitive Data

`skill-compress` sends content to external LLM providers only through the experimental `--verify-llm` judge, and only when that flag is explicitly used (there is no LLM rewrite mode). Do not use it on files containing secrets until redaction is implemented and verified for your use case.

Never commit:

- API keys;
- bearer tokens;
- private keys;
- provider responses containing private document content;
- private `SKILL.md` samples.
