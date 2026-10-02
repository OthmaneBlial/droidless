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
if [ "$(uname -s)" = Darwin ]; then
    font_check=$(mktemp -t droidless-native-font)
    trap 'rm -f "$font_check"' EXIT HUP INT TERM
    xcrun clang -fobjc-arc -Wall -Wextra -Werror -framework AppKit -framework QuartzCore tools/native-font-check.m -o "$font_check"
    "$font_check"
fi
