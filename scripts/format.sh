#!/usr/bin/env bash
set -xue

# 格式化检查
rustup default stable
find . -name "*.rs" ! -path "./target/*" ! -path "**/fuzz/target/*" ! -path "**/pb/*" ! -path "**/proto/pb/*" ! -path "**/proto/tmp/*" ! -path "**/generated/*" ! -path "**/proto/generated/*" -print0 | \
xargs -0 cargo fmt -- --config-path .rustfmt.toml --verbose --check

echo "Format check completed successfully!"
