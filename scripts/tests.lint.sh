#!/usr/bin/env bash
set -xue

# 运行格式化检查脚本
echo "Running format check..."
bash ./scripts/format.sh

# 运行质量检查脚本
echo "Running clippy check..."
bash ./scripts/clippy.sh

# 清理构建产物
cargo clean

echo "ALL SUCCESS!"
