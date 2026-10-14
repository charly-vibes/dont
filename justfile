set shell := ["bash", "-cu"]

default:
  @just --list

build:
  cargo build

install:
  cargo install --path . --locked

test:
	#!/usr/bin/env bash
	# dont-a4lk: prefer nextest (parallel, per-test timing); fall back to cargo test
	if command -v cargo-nextest >/dev/null 2>&1; then
		cargo nextest run
	else
		cargo test
	fi
	# nextest does not run doctests; keep them covered explicitly
	cargo test --doc --quiet

# Validate spec-test correspondence (requires espectacular)
validate:
  ah check

run *args:
  cargo run -- {{args}}

status:
  wai status

doctor:
  wai doctor

way:
  wai way

sync:
  wai sync --yes

prime:
  wai prime

show:
  wai show

ready:
  bd ready

bd-status:
  bd status

fmt-check:
  cargo fmt --all --check

lint:
  cargo fmt --all --check
  cargo clippy --all-targets --all-features -- -D warnings
  typos
  vale README.md AGENTS.md CLAUDE.md llm.txt

docs-build:
  mdbook build

docs: docs-build

coverage:
  cargo llvm-cov --workspace --lcov --output-path target/coverage/lcov.info --fail-under-lines 60

check-claims:
  cargo build --quiet
  ./target/debug/dont prime

ci:
  just test
  just lint
  just docs-build
  just check-claims
  wai doctor

reflect:
  wai reflect
