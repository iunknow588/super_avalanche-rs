# AvalancheGo 一致性测试 SDK

本目录包含 AvalancheGo 一致性测试的 Rust SDK 实现，用于构建与 AvalancheGo 节点交互的客户端。

## 目录结构说明

### 核心文件
- `Cargo.toml`: Rust 项目配置文件，定义依赖和构建参数
- `build.rs`: 自定义构建脚本，处理 Protobuf 代码生成
- `LICENSE`: 项目许可证文件

### 主要代码目录

#### `src/`
SDK 核心实现:
- 包含与 AvalancheGo 节点通信的核心逻辑
- gRPC 客户端封装
- 一致性测试工具函数

#### `examples/`
使用示例:
- 演示如何集成和使用该 SDK

#### `scripts/`
构建和测试脚本

## 设计模式

1. **Rust 原生实现**: 提供高性能的 Rust 语言 SDK
2. **gRPC 客户端封装**: 简化与 AvalancheGo 节点的交互
3. **模块化设计**: 功能按目录清晰划分
4. **示例驱动**: 提供完整的使用示例

## 使用说明

1. 添加依赖到您的 `Cargo.toml`
2. 参考 `examples/` 中的代码集成 SDK
3. 使用提供的 gRPC 客户端与节点交互

如需更详细信息，请参考各子目录中的文档。
