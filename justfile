# Check https://github.com/casey/just/blob/master/examples/kitchen-sink.just for examples

alias t := test
alias c := check
alias l := lint
alias f := fix
alias w := watch
alias dc := dead-code
alias cov := coverage

# List all recipes
default:
	@just --list --unsorted

##############################################
################ Setup #######################
##############################################

# Setup the entire project (idempotent)
[group('setup')]
setup:
	#!/usr/bin/env bash
	echo "🚀 Setting up ratatui-cheese project..."

	echo "🐚 Installing nu shell..."
	just install-nu

	echo "📦 Installing cargo tools..."
	command -v bacon >/dev/null || cargo install bacon
	command -v cargo-nextest >/dev/null || cargo install --locked cargo-nextest
	command -v cargo-outdated >/dev/null || cargo install cargo-outdated
	command -v cargo-llvm-cov >/dev/null || cargo install cargo-llvm-cov

	echo "🎬 Installing vhs..."
	command -v vhs >/dev/null || brew install charmbracelet/tap/vhs

	echo "📦 Syncing reference repos..."
	just update-reference-repos

	echo "✅ Project setup complete!"

##############################################
################ Dev #########################
##############################################

# Run all quality gates: check, test, lint, dead-code
[group('dev')]
all: check test lint dead-code

# Check all targets for compilation errors
[group('dev')]
check:
	cargo check --all-targets

# Start bacon watch tool for live feedback
[group('dev')]
watch:
	bacon

# Auto-fix formatting and clippy warnings
[group('dev')]
fix:
	cargo fmt
	cargo clippy --fix -- -D warnings # Note that --fix implies --all-targets

# Run all tests using cargo-nextest
[group('dev')]
test $RUST_BACKTRACE="1":
	cargo nextest run

# Run clippy on all targets
[group('dev')]
lint:
	cargo fmt --check
	cargo clippy --all-targets -- -D warnings

# Generate test coverage summary
[group('dev')]
coverage:
	cargo llvm-cov --lib -p ratatui-cheese

# Search for #[allow(dead_code)] occurrences
[group('dev')]
dead-code:
	@echo "Searching for #[allow(dead_code)] occurrences..."
	@rg "#\[allow\(dead_code\)\]" --glob '!.target/**' --glob '!JUSTFILE' || echo "None found."

##############################################
################ Examples ####################
##############################################

# Run the showcase demo app
[group('examples')]
showcase:
	cargo run -p showcase

# Run an example (e.g. just example spinners)
[group('examples')]
example name:
	cargo run -p ratatui-cheese --example {{name}}

##############################################
################ Recording ###################
##############################################

# Generate a GIF recording from a VHS tape file (e.g. just record spinners)
[group('recording')]
record name:
	vhs tools/vhs/{{name}}.tape

##############################################
################ Publishing ##################
##############################################

# Run all CI checks: check, lint, and test (no nextest dependency)
[group('publishing')]
ci: check lint
	cargo test --workspace

# Publish ratatui-cheese to crates.io (runs CI checks first)
[group('publishing')]
publish: ci
	cargo publish -p ratatui-cheese

##############################################
################ Dependencies ################
##############################################

# Show outdated dependencies
[group('deps')]
outdated:
	cargo outdated -wR

##############################################
################ References ##################
##############################################

# Clone/update reference repos from project-manifest.yaml
[group('references')]
update-reference-repos:
	@nu tools/scripts/update-reference-repos.nu

# Show status of reference repos
[group('references')]
reference-status:
	@nu tools/scripts/update-reference-repos.nu status

##############################################
################ Private #####################
##############################################

# Install nu shell
[macos, private]
install-nu:
	command -v nu >/dev/null 2>&1 || brew install nushell

# Install nu shell
[linux, private]
install-nu:
	command -v nu >/dev/null 2>&1 || cargo install nu
