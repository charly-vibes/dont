set shell := ["bash", "-cu"]

default:
  @just --list

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
  prek run --all-files
  typos
  vale README.md AGENTS.md CLAUDE.md llm.txt

ci:
  just lint
  wai doctor

reflect:
  wai reflect
