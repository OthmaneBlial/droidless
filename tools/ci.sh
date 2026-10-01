#!/bin/sh
# Local CI only. This project intentionally has no GitHub Actions workflows.
set -eu
cd "$(dirname "$0")/.."
cargo fmt --all -- --check
cargo check --workspace --locked
cargo test --workspace --locked
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo build --workspace --release --locked
cargo build --release --locked -p droidless-runtime --example document-replay
cargo run --release --locked -p droidless-formats --example fuzz-smoke
