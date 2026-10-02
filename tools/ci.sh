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
    native_check=$(mktemp -t droidless-native-check)
    trap 'rm -f "$native_check"' EXIT HUP INT TERM
    for source in tools/native-font-check.m tools/native-focus-check.m tools/native-dialog-check.m tools/native-foreground-check.m; do
        xcrun clang -fobjc-arc -Wall -Wextra -Werror -framework AppKit -framework QuartzCore "$source" -o "$native_check"
        "$native_check"
    done
fi
