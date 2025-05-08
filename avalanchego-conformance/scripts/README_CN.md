# 构建和工具脚本

本目录包含 AvalancheGo 一致性测试框架的构建和工具脚本，用于自动化各种开发和维护任务。

## 目录

1. [文件结构](#文件结构)
2. [设计模式](#设计模式)
3. [核心功能](#核心功能)
4. [使用说明](#使用说明)
5. [扩展指南](#扩展指南)

## 文件结构

- `genproto.sh`: 生成 Protocol Buffers 代码
- `updatedep.sh`: 更新依赖

## 设计模式

### 自动化脚本模式

使用 shell 脚本自动化重复任务：

```bash
#!/usr/bin/env bash

set -e

# 脚本逻辑
```

### 错误处理模式

使用 `set -e` 确保一旦出错立即退出：

```bash
#!/usr/bin/env bash

set -e

# 如果任何命令失败，脚本将立即退出
```

### 参数化模式

使用环境变量和命令行参数配置脚本行为：

```bash
#!/usr/bin/env bash

# 默认值
OUTPUT_DIR=${OUTPUT_DIR:-"./output"}
VERBOSE=${VERBOSE:-false}

# 使用参数
if [ "$VERBOSE" = true ]; then
    echo "输出目录: $OUTPUT_DIR"
fi
```

## 核心功能

### Protocol Buffers 代码生成

`genproto.sh` 脚本用于生成 Protocol Buffers 代码：

- 使用 `buf` 工具生成 Go 代码
- 支持多种生成选项
- 处理生成后的文件

### 依赖更新

`updatedep.sh` 脚本用于更新项目依赖：

- 更新 Go 模块依赖
- 清理未使用的依赖
- 验证依赖版本

## 使用说明

### 生成 Protocol Buffers 代码

```bash
# 使用默认配置生成代码
./scripts/genproto.sh

# 指定输出目录
OUTPUT_DIR=./custom-output ./scripts/genproto.sh

# 启用详细输出
VERBOSE=true ./scripts/genproto.sh
```

### 更新依赖

```bash
# 更新所有依赖
./scripts/updatedep.sh

# 更新特定依赖
DEP_NAME="github.com/example/package" ./scripts/updatedep.sh
```

## 扩展指南

### 添加新脚本

要添加新脚本，请按照以下步骤操作：

1. 创建新的 shell 脚本文件：

```bash
#!/usr/bin/env bash

set -e

# 脚本说明
# 用法: ./scripts/new-script.sh [参数]

# 默认配置
CONFIG_VALUE=${CONFIG_VALUE:-"默认值"}

# 脚本逻辑
echo "执行操作..."

# 退出
echo "完成"
exit 0
```

2. 添加执行权限：

```bash
chmod +x ./scripts/new-script.sh
```

3. 更新文档。

### 修改现有脚本

要修改现有脚本，请按照以下步骤操作：

1. 更新脚本逻辑
2. 测试脚本功能
3. 更新文档

### 脚本最佳实践

编写脚本时，请遵循以下最佳实践：

1. 始终包含 shebang 行：`#!/usr/bin/env bash`
2. 使用 `set -e` 确保错误时退出
3. 使用环境变量配置脚本行为
4. 提供有用的错误消息和帮助信息
5. 使用函数组织复杂逻辑
6. 添加注释说明脚本用途和用法
