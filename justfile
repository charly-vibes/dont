set shell := ["bash", "-cu"]

default:
  @just --list

build:
  cargo build

install:
  cargo install --path . --locked

test:
  cargo test

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
  cargo clippy --all-targets --all-features -- -D warnings
  prek run --all-files
  typos
  vale README.md AGENTS.md CLAUDE.md llm.txt

docs-build:
  mdbook build

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
