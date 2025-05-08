# AvalancheGo 一致性测试 SDK 脚本

本目录包含 AvalancheGo 一致性测试 SDK 的构建和测试脚本，用于自动化常见任务。

## 目录

1. [概述](#概述)
2. [脚本列表](#脚本列表)
3. [使用说明](#使用说明)
4. [创建新脚本](#创建新脚本)

## 概述

这些脚本旨在简化 AvalancheGo 一致性测试 SDK 的构建、测试和维护过程。它们提供了一种标准化的方式来执行常见任务，确保一致性和可重复性。

## 脚本列表

### build.release.sh

构建 SDK 的发布版本：

- 使用 `cargo build --release` 命令
- 生成优化的二进制文件
- 适用于生产环境

```bash
#!/usr/bin/env bash
set -xue

if ! [[ "$0" =~ scripts/build.release.sh ]]; then
  echo "must be run from repository root"
  exit 255
fi

cargo build --release
```

## 使用说明

### 运行脚本

从项目根目录运行脚本：

```bash
# 构建发布版本
./scripts/build.release.sh
```

### 权限设置

确保脚本具有执行权限：

```bash
# 添加执行权限
chmod +x scripts/build.release.sh
```

### 错误处理

脚本使用以下设置确保错误被正确处理：

- `set -e`: 如果任何命令失败，立即退出脚本
- `set -u`: 如果引用未定义的变量，立即退出脚本
- `set -x`: 在执行命令前打印命令，便于调试

## 创建新脚本

### 基本步骤

1. 在 `scripts` 目录中创建新的 `.sh` 文件
2. 添加 shebang 行 (`#!/usr/bin/env bash`)
3. 设置适当的错误处理选项
4. 添加脚本逻辑
5. 添加注释和说明
6. 设置执行权限
7. 更新本 README 文件，添加新脚本的信息

### 脚本模板

```bash
#!/usr/bin/env bash
set -euo pipefail

# 脚本描述: 这个脚本的用途

# 确保从项目根目录运行
if ! [[ "$0" =~ scripts/script_name.sh ]]; then
  echo "必须从项目根目录运行"
  exit 255
fi

# 脚本逻辑
echo "执行任务..."

# 示例命令
cargo build

echo "任务完成"
```

### 最佳实践

- 保持脚本简单明了，专注于单一任务
- 添加详细的注释，解释脚本的目的和工作原理
- 使用适当的错误处理机制
- 验证运行环境和前提条件
- 提供有意义的错误消息和退出代码
