#!/usr/bin/env bash
set -xue

# fmt check
rustup default stable
find . -name "*.rs" ! -path "./target/*" ! -path "**/pb/*" ! -path "**/proto/pb/*" ! -path "**/proto/tmp/*" ! -path "**/generated/*" ! -path "**/proto/generated/*" -print0 | \
xargs -0 cargo fmt -- --config-path .rustfmt.toml --verbose --check

# clippy check with specific lints instead of restriction group
# Removed -D clippy::cargo and added individual cargo lints except multiple_crate_versions
cargo clippy --all --all-features --tests --benches --examples -- \
-D clippy::suspicious \
-D clippy::style \
-D clippy::complexity \
-D clippy::perf \
-D clippy::pedantic \
-D clippy::nursery \
-D clippy::missing_docs_in_private_items \
-D clippy::missing_errors_doc \
-D clippy::missing_panics_doc \
-D warnings \
-D clippy::large_stack_arrays \
-D clippy::missing_safety_doc \
-D clippy::redundant_pub_crate \
-D clippy::unwrap_in_result \
-D clippy::wildcard_dependencies \
-A clippy::multiple_crate_versions

cargo clean

echo "ALL SUCCESS!"
