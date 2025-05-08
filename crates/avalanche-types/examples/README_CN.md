# Avalanche 类型示例代码

本目录包含 `avalanche-types` 库的使用示例，展示如何使用该库的各种功能和组件。

## 目录

1. [概述](#概述)
2. [示例列表](#示例列表)
3. [运行示例](#运行示例)
4. [创建新示例](#创建新示例)

## 概述

这些示例代码旨在帮助开发者理解和使用 `avalanche-types` 库的各种功能。每个示例都专注于特定的功能领域，并提供了详细的注释和说明。

示例代码涵盖了从基本类型操作到复杂交易构建的各个方面，是学习和参考的宝贵资源。

## 示例列表

### 基础类型

- **ids_example.rs**: 演示 ID 类型的创建和操作
- **hash_example.rs**: 展示哈希函数的使用
- **codec_example.rs**: 演示数据编解码功能

### 密钥和地址

- **key_secp256k1_example.rs**: 演示 secp256k1 密钥的生成和使用
- **key_bls_example.rs**: 展示 BLS 密钥和签名
- **address_example.rs**: 演示不同链的地址格式和生成

### 交易构建

- **tx_builder_example.rs**: 展示如何构建和签名交易
- **utxo_example.rs**: 演示 UTXO 模型的使用
- **memo_example.rs**: 展示如何在交易中添加备注

### 网络通信

- **message_example.rs**: 演示网络消息的创建和处理
- **packer_example.rs**: 展示数据打包和序列化

### 钱包功能

- **wallet_x_example.rs**: 演示 X-Chain 钱包功能
- **wallet_p_example.rs**: 演示 P-Chain 钱包功能
- **wallet_c_example.rs**: 演示 C-Chain (EVM) 钱包功能

### 高级功能

- **subnet_example.rs**: 演示子网功能
- **jsonrpc_example.rs**: 展示 JSON-RPC 客户端的使用
- **kms_aws_example.rs**: 演示 AWS KMS 集成

## 运行示例

### 基本用法

使用 `cargo run --example` 命令运行特定示例：

```bash
# 运行 ID 示例
cargo run --example ids_example

# 运行 secp256k1 密钥示例
cargo run --example key_secp256k1_example
```

### 启用特性

某些示例需要启用特定的特性标志：

```bash
# 运行需要 jsonrpc_client 特性的示例
cargo run --example jsonrpc_example --features jsonrpc_client

# 运行需要 wallet 特性的示例
cargo run --example wallet_x_example --features wallet
```

### 带参数的示例

某些示例接受命令行参数：

```bash
# 运行带参数的示例
cargo run --example tx_builder_example -- --amount 100 --recipient X-avax1...
```

## 创建新示例

### 基本步骤

1. 在 `examples` 目录中创建新的 `.rs` 文件
2. 实现 `main` 函数作为入口点
3. 添加详细的注释和说明
4. 更新本 README 文件，添加新示例的信息

### 示例模板

```rust
//! 示例标题
//!
//! 详细描述示例的功能和用途。

// 导入必要的依赖
use avalanche_types::ids::Id;
use std::str::FromStr;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 示例代码
    println!("示例开始");
    
    // 实现示例逻辑
    let id = Id::from_str("2oYMBNsz5qctDi5mY8545fRJYP5SQ3TyP1nbt4gE4AUVza5JRy")?;
    println!("ID: {}", id);
    
    println!("示例结束");
    Ok(())
}
```

### 特性依赖

如果示例依赖于可选特性，请在文件顶部添加注释说明：

```rust
//! 示例标题
//!
//! 详细描述示例的功能和用途。
//!
//! 运行此示例需要启用 `jsonrpc_client` 特性：
//! ```bash
//! cargo run --example this_example --features jsonrpc_client
//! ```

// 导入依赖
// ...
```

### 最佳实践

- 保持示例简单明了，专注于单一功能
- 添加详细的注释，解释代码的目的和工作原理
- 处理错误，使用 `Result` 类型返回
- 提供有意义的输出，帮助用户理解结果
- 如果示例较复杂，考虑拆分为多个函数
