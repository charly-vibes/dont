#!/usr/bin/env bash
# Run the full test suite; exit non-zero if any test fails
cargo test --quiet 2>&1 || exit 1
