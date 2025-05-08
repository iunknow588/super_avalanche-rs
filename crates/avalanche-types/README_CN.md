# Avalanche 类型模块 (avalanche-types)

本模块定义了 Avalanche 生态系统的核心数据类型和工具，是整个项目的基础库。

## 目录

1. [概述](#概述)
2. [目录结构](#目录结构)
3. [核心功能](#核心功能)
4. [设计模式](#设计模式)
5. [使用示例](#使用示例)
6. [特性标志](#特性标志)
7. [API 文档](#api-文档)

## 概述

`avalanche-types` 是 Avalanche 区块链生态系统的 Rust 实现，提供了所有基础类型和工具。它包含了从网络通信到密码学操作的各种功能，是整个 Avalanche-rs 项目的基础。

该库不仅提供了与 AvalancheGo（Go 语言实现）兼容的类型定义，还包含了用于开发自定义子网和虚拟机的 SDK。

## 目录结构

- `src/`: 核心类型定义
  - `lib.rs`: 模块入口
  - `avm/`: X-Chain 虚拟机相关类型
  - `choices/`: 共识决策相关类型
  - `codec/`: 数据编解码工具
  - `errors/`: 错误类型和处理
  - `evm/`: 以太坊虚拟机相关类型
  - `formatting/`: 数据格式化工具
  - `hash/`: 哈希函数和工具
  - `ids/`: ID 类型和操作
  - `jsonrpc/`: JSON-RPC 客户端和工具
  - `key/`: 密钥管理和操作
  - `message/`: 网络消息类型和处理
  - `packer/`: 数据打包和序列化
  - `platformvm/`: P-Chain 相关类型
  - `proto/`: Protocol Buffers 生成的代码
  - `subnet/`: 子网相关功能和 SDK
  - `txs/`: 交易相关类型和操作
  - `wallet/`: 钱包功能
  - 其他辅助模块
- `examples/`: 使用示例
  - 包含各种功能的示例代码
- `tests/`: 集成测试
  - 验证库功能的测试代码
- `scripts/`: 工具脚本
  - 包含代码生成和维护脚本
- `fuzz/`: 模糊测试
  - 用于发现潜在问题的模糊测试
- `artifacts/`: 构建产物
  - 包含生成的文件和资源

## 核心功能

### 1. 区块链基础类型

- **ID 和哈希**: 提供区块链中使用的各种 ID 和哈希类型
- **序列化/反序列化**: 支持多种数据编码格式
- **错误处理**: 统一的错误类型和处理机制

### 2. 虚拟机相关类型

- **X-Chain (AVM)**: 资产虚拟机相关类型
- **P-Chain (PlatformVM)**: 平台虚拟机相关类型
- **C-Chain (EVM)**: 以太坊虚拟机相关类型
- **子网 VM**: 自定义虚拟机支持

### 3. 网络通信

- **消息类型**: 定义节点间通信的消息格式
- **消息处理**: 消息的序列化、压缩和处理
- **协议支持**: 实现 Avalanche 网络协议

### 4. 密钥和安全

- **密钥管理**: 支持多种密钥类型和操作
- **签名验证**: 交易签名和验证
- **地址生成**: 不同链的地址格式和生成

### 5. 钱包功能

- **交易构建**: 创建和签名交易
- **UTXO 管理**: 管理未花费交易输出
- **多链支持**: X-Chain、P-Chain 和 C-Chain 钱包

## 设计模式

### 1. 建造者模式 (Builder Pattern)

用于复杂对象的构造，如交易构建：

```rust
// 使用建造者模式构建交易
let tx_builder = TransactionBuilder::new()
    .add_input(input)
    .add_output(output)
    .set_memo(memo);
let tx = tx_builder.build();
```

### 2. 适配器模式 (Adapter Pattern)

用于不同类型之间的转换：

```rust
// 将 Rust 类型转换为 Protocol Buffers 消息
let proto_msg = message.into_pb();

// 将 Protocol Buffers 消息转换回 Rust 类型
let rust_msg = Message::from_pb(proto_msg);
```

### 3. 组合模式 (Composite Pattern)

用于构建复杂的嵌套类型：

```rust
// 交易包含多个输入和输出
struct Transaction {
    inputs: Vec<Input>,
    outputs: Vec<Output>,
    memo: Vec<u8>,
}
```

### 4. 工厂模式 (Factory Pattern)

用于创建不同类型的对象：

```rust
// 使用工厂方法创建密钥
let key = KeyFactory::create(KeyType::Secp256k1);
```

### 5. 单例模式 (Singleton Pattern)

用于全局状态管理：

```rust
// 全局编解码器注册表
let codec = Codec::instance();
codec.register_type(type_id, encoder, decoder);
```

## 使用示例

### 创建和使用 ID

```rust
use avalanche_types::ids;

// 从字节创建 ID
let id = ids::Id::from_slice(&[0; 32]);
println!("ID: {}", id);

// 从字符串创建 ID
let id_from_str = ids::Id::from_str("2oYMBNsz5qctDi5mY8545fRJYP5SQ3TyP1nbt4gE4AUVza5JRy").unwrap();
println!("ID from string: {}", id_from_str);
```

### 密钥管理

```rust
use avalanche_types::key::secp256k1;

// 生成新密钥对
let key_pair = secp256k1::KeyPair::generate().unwrap();
println!("Private Key: {}", key_pair.private_key());
println!("Public Key: {}", key_pair.public_key());

// 从私钥创建密钥对
let private_key_bytes = [/* ... */];
let key_pair = secp256k1::KeyPair::from_private_key(&private_key_bytes).unwrap();
```

### 交易构建

```rust
use avalanche_types::wallet::p;

// 创建 P-Chain 交易
let tx = p::builder::Builder::new()
    .add_validator(
        &key_pair,
        node_id,
        start_time,
        end_time,
        stake_amount,
        reward_address,
    )
    .build()
    .unwrap();

// 签名交易
let signed_tx = tx.sign(&[&key_pair]).unwrap();
```

## 特性标志

该库提供了多种可选功能，可以根据需要启用：

- `avalanchego`: AvalancheGo 兼容性功能
- `codec_base64`: Base64 编解码支持
- `evm`: 以太坊虚拟机支持
- `jsonrpc_client`: JSON-RPC 客户端
- `kms_aws`: AWS KMS 集成
- `message`: 网络消息处理
- `mnemonic`: 助记词支持
- `proto`: Protocol Buffers 支持
- `subnet`: 子网支持
- `subnet_evm`: 子网 EVM 支持
- `wallet`: 钱包功能
- `wallet_evm`: EVM 钱包功能

## API 文档

完整的 API 文档请参考代码文档或使用以下命令生成：

```bash
cargo doc --open
```
