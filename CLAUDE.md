# ratatui-cheese

## Verification

Always use justfile commands for verification, never raw cargo commands:

- `just check` — compilation check on all targets
- `just lint` — fmt check + clippy with `-D warnings`
- `just test` — run all tests via cargo-nextest
- `just all` — check + test + lint + dead-code (full quality gate)
- `just example <name>` — run a specific example
- `just showcase` — run the showcase demo app
