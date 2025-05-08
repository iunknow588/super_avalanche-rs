# AvalancheGo 一致性测试 SDK 示例

本目录包含 AvalancheGo 一致性测试 SDK 的使用示例，展示如何使用该 SDK 与 AvalancheGo 一致性测试服务器通信。

## 目录

1. [概述](#概述)
2. [示例列表](#示例列表)
3. [运行示例](#运行示例)
4. [创建新示例](#创建新示例)

## 概述

这些示例代码旨在帮助开发者理解和使用 AvalancheGo 一致性测试 SDK。每个示例都专注于特定的功能，并提供了详细的注释和说明。

示例代码涵盖了从基本连接到复杂测试的各个方面，是学习和参考的宝贵资源。

## 示例列表

### ping_service.rs

演示如何连接到一致性测试服务器并发送 ping 请求：

- 创建客户端连接
- 发送 ping 请求
- 处理响应

这是最基本的示例，展示了 SDK 的核心功能。

## 运行示例

### 基本用法

使用 `cargo run --example` 命令运行特定示例：

```bash
# 运行 ping 服务示例
cargo run --example ping_service -- http://localhost:8080
```

### 环境设置

在运行示例之前，确保：

1. AvalancheGo 一致性测试服务器已启动
2. 服务器地址正确（默认为 http://localhost:8080）
3. 网络连接正常

### 日志级别

示例使用 `env_logger` 进行日志记录。可以通过环境变量控制日志级别：

```bash
# 设置日志级别为 debug
RUST_LOG=debug cargo run --example ping_service -- http://localhost:8080

# 设置日志级别为 trace
RUST_LOG=trace cargo run --example ping_service -- http://localhost:8080
```

## 创建新示例

### 基本步骤

1. 在 `examples` 目录中创建新的 `.rs` 文件
2. 实现 `main` 函数作为入口点
3. 添加详细的注释和说明
4. 更新本 README 文件，添加新示例的信息

### 示例模板

```rust
use std::env::args;

use avalanchego_conformance_sdk::Client;
use tokio::runtime::Runtime;

/// 示例描述
/// 
/// 运行方式：
/// cargo run --example example_name -- [HTTP RPC ENDPOINT]
/// cargo run --example example_name -- http://127.0.0.1:8080
fn main() {
    // 初始化日志
    env_logger::init_from_env(
        env_logger::Env::default().filter_or(env_logger::DEFAULT_FILTER_ENV, "info"),
    );

    // 获取命令行参数
    let url = args().nth(1).expect("no url given");
    let rt = Runtime::new().unwrap();

    // 创建客户端
    log::info!("creating client");
    let client = rt.block_on(Client::new(&url));

    // 实现示例逻辑
    // ...

    log::info!("example completed");
}
```

### 最佳实践

- 保持示例简单明了，专注于单一功能
- 添加详细的注释，解释代码的目的和工作原理
- 处理错误，提供有意义的错误消息
- 提供命令行参数的说明
- 使用日志记录关键信息
