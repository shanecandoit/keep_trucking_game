
set -e

cargo test

cargo fmt --all -- --check

cargo clippy --all -- -D warnings

cargo build
