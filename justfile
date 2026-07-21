set shell := ["bash", "-cu"]

default:
  @just --list

build:
  cargo build

install:
  cargo install --path . --locked

test:
  cargo test

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

lint:
  cargo fmt --all --check
  cargo clippy --all-targets --all-features -- -D warnings
  typos
  vale README.md AGENTS.md CLAUDE.md llm.txt

docs-build:
  mdbook build

docs: docs-build

coverage:
  cargo tarpaulin

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
