# Avalanche-rs Crates

本目录包含 Avalanche-rs 项目的核心 Rust crates，这些 crates 实现了 Avalanche 网络的各种功能和组件。

## 目录结构

```
crates/
├── avalanche-types/     # Avalanche 基础类型和工具
└── avalanche-consensus/ # Avalanche 共识算法实现
```

## 设计理念

Avalanche-rs 项目采用模块化设计，将功能分解为独立的 crates，每个 crate 专注于特定的功能领域。这种设计有以下优势：

1. **关注点分离**：每个 crate 专注于特定的功能领域
2. **灵活的依赖管理**：用户可以只引入所需的功能
3. **独立的版本控制**：各个 crate 可以独立发布和版本控制
4. **清晰的 API 边界**：明确定义的公共接口促进了代码的可维护性
5. **可选功能**：通过 feature flags 提供可选功能

## Crates 详细说明

### avalanche-types

**版本**：0.1.5  
**描述**：Avalanche 基础类型和工具库  
**路径**：`/crates/avalanche-types`

这个 crate 提供了 Avalanche 网络中使用的所有基础类型和工具，是整个项目的基础。它包含了从网络通信到密码学操作的各种功能。

#### 主要模块

1. **avm**：Avalanche 虚拟机相关类型和功能
2. **choices**：共识决策相关类型
3. **codec**：数据编解码工具
4. **errors**：错误类型和处理
5. **formatting**：数据格式化工具
6. **hash**：哈希函数和工具
7. **ids**：ID 类型和操作
8. **jsonrpc**：JSON-RPC 客户端和工具
9. **key**：密钥管理和操作
10. **message**：网络消息类型和处理
11. **packer**：数据打包和序列化
12. **platformvm**：平台虚拟机（P-Chain）相关类型
13. **proto**：Protocol Buffers 生成的代码
14. **subnet**：子网相关功能和 SDK
15. **txs**：交易相关类型和操作
16. **wallet**：钱包功能

#### 可选功能（Feature Flags）

avalanche-types 提供了多种可选功能，可以根据需要启用：

- **avalanchego**：AvalancheGo 兼容性功能
- **codec_base64**：Base64 编解码支持
- **evm**：以太坊虚拟机支持
- **jsonrpc_client**：JSON-RPC 客户端
- **kms_aws**：AWS KMS 集成
- **message**：网络消息处理
- **mnemonic**：助记词支持
- **proto**：Protocol Buffers 支持
- **subnet**：子网支持
- **subnet_evm**：子网 EVM 支持
- **wallet**：钱包功能
- **wallet_evm**：EVM 钱包功能

#### 使用示例

```rust
use avalanche_types::{ids, key};

// 创建一个新的 ID
let id = ids::Id::from_slice(&[0; 32]);
println!("ID: {}", id);

// 生成一个新的密钥对
let key_pair = key::secp256k1::KeyPair::generate().unwrap();
println!("Public Key: {}", key_pair.public_key());
```

### avalanche-consensus

**版本**：0.1.1  
**描述**：Avalanche 共识算法实现  
**路径**：`/crates/avalanche-consensus`

这个 crate 实现了 Avalanche 共识协议，包括 Snowball、Slush 和区块共识。它提供了构建高性能共识系统所需的数据结构和算法。

#### 主要模块

1. **context**：共识上下文和环境
2. **snowman**：Snowman 共识协议实现
   - **block**：区块相关类型和操作
   - **bootstrap**：引导过程
   - **consensus**：共识算法核心
   - **topological**：拓扑排序

#### 核心组件

1. **Parameters**：共识参数配置，包括：
   - `k`：样本大小
   - `alpha`：法定人数阈值
   - `beta_virtuous`：良性交易的决策阈值
   - `beta_rogue`：恶意交易的决策阈值
   - 其他配置参数

2. **Snowball**：Snowball 共识算法实现
3. **Snowman**：基于 Snowball 的区块链共识实现

#### 使用示例

```rust
use avalanche_consensus::Parameters;

// 创建默认共识参数
let params = Parameters::default();

// 验证参数有效性
match params.verify() {
    Ok(_) => println!("Parameters are valid"),
    Err(e) => println!("Invalid parameters: {}", e),
}
```

## 依赖关系

- **avalanche-consensus** 依赖于 **avalanche-types**，使用其中的基础类型和工具

## 构建和测试

每个 crate 都可以独立构建和测试：

```bash
# 构建 avalanche-types
cd crates/avalanche-types
cargo build

# 测试 avalanche-types
cargo test

# 构建 avalanche-consensus
cd ../avalanche-consensus
cargo build

# 测试 avalanche-consensus
cargo test
```

## 特性和功能

1. **高性能**：优化的数据结构和算法
2. **类型安全**：利用 Rust 的类型系统确保安全性
3. **可扩展**：模块化设计便于扩展
4. **互操作性**：与 AvalancheGo 兼容
5. **文档完善**：详细的文档和示例

## 贡献指南

贡献新功能或修复时，请遵循以下原则：

1. 保持向后兼容性
2. 添加适当的测试
3. 更新文档
4. 遵循现有的代码风格和命名约定

## 许可证

这些 crates 使用与 Avalanche-rs 项目相同的许可证。
