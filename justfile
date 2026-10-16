set shell := ["bash", "-cu"]

default:
  @just --list

build:
  cargo build

install:
  cargo install --path . --locked


# TDD loop (most obvious entry, DDL test-runner standard — see DDL-85p in
# the dulce-de-leche repo): testaruda exec — select the tests the working
# diff affects, run only those, ingest, calibrate. Tolerance contract:
# exit 0/10/20 green; NOT CALIBRATED with 0 actual failures is a vacuous
# pass (tambor-cpd policy), with failures it blocks. Local store
# (.testaruda/, gitignored) initializes on first use; a stale store
# self-heals (wild-z56).
test:
	#!/usr/bin/env bash
	set -uo pipefail
	if [ ! -d .testaruda ]; then
	  testaruda init || { echo "❌ testaruda init failed" >&2; exit 1; }
	fi
	code=0
	testaruda exec || code=$?
	case "$code" in
	  0|10|20) exit 0 ;;
	  4)
	    echo "⚠️ runner exit 4 — pruning stale local calibration store and retrying once (wild-z56)" >&2
	    rm -rf .testaruda
	    testaruda init || { echo "❌ testaruda re-init failed" >&2; exit 1; }
	    testaruda exec ;;
	  *) exit "$code" ;;
	esac

# Blast-radius preview: which tests a change WOULD affect — runs nothing.
# Useful before a refactor; optionally name files: just test-blast src/foo.rs
test-blast *files:
	#!/usr/bin/env bash
	if [ -z "{{files}}" ]; then
	  testaruda select --pre-edit --human
	else
	  testaruda select --pre-edit --human --files "{{files}}"
	fi

test-full:
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
