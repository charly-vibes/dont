set shell := ["bash", "-cu"]

default:
  @just --list

build:
  cargo build

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

ci:
  just lint
  just docs-build
  wai doctor

reflect:
  wai reflect
