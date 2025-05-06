#!/usr/bin/env bash
set -xue

if ! [[ "$0" =~ scripts/tests.fuzz.sh ]]; then
  echo "must be run from repository root"
  exit 255
fi

# ref. https://github.com/rust-fuzz/cargo-fuzz
# ref. https://rust-fuzz.github.io/book/cargo-fuzz/setup.html
rustup default nightly

# Check if cargo-fuzz is installed, if not install it
if ! cargo fuzz --help &> /dev/null; then
  echo "cargo-fuzz not found, installing..."
  cargo install cargo-fuzz
fi

pushd crates/avalanche-types
cargo fuzz run ids
popd

rustup default stable

cargo clean

echo "ALL SUCCESS!"
