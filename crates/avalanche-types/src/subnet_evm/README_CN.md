# Subnet-EVM 模块说明

## 概述

`subnet_evm` 模块提供了与 Avalanche 子网 EVM (Subnet-EVM) 相关的配置和工具。Subnet-EVM 是 Avalanche 网络上的一个特殊虚拟机，它允许开发者创建兼容以太坊的自定义区块链（子网），同时享受 Avalanche 网络的高性能和可扩展性。

该模块主要用于：
1. 生成和管理 Subnet-EVM 的创世配置
2. 配置 Subnet-EVM 的链参数
3. 为 Avalanche 子网部署提供必要的工具

## 目录结构

```
subnet_evm/
├── chain_config.rs    # 链配置定义
├── genesis.rs         # 创世状态配置
└── mod.rs             # 模块入口
```

## 主要组件

### 1. 链配置 (`chain_config.rs`)

定义了 Subnet-EVM 链的运行时配置参数。

#### 主要类型

- `Config`：Subnet-EVM 链配置结构体，包含大量可配置参数

#### 主要字段

- `eth_apis`：启用的以太坊 API 列表
- `coreth_admin_api`：管理 API 配置
- `pruning_enabled`：是否启用状态修剪
- `snapshot_async`：是否异步生成快照
- `tx_pool_*`：交易池配置参数
- `log_level`：日志级别
- `state_sync_enabled`：是否启用状态同步

#### 主要方法

- `encode_json()`：将配置序列化为 JSON 字符串
- `sync()`：将配置保存到磁盘

#### 使用示例

```rust
use avalanche_types::subnet_evm::chain_config;

// 创建默认配置
let mut config = chain_config::Config::default();

// 自定义配置
config.log_level = Some(String::from("debug"));
config.pruning_enabled = Some(true);

// 保存配置到文件
config.sync("/path/to/config.json").unwrap();
```

### 2. 创世配置 (`genesis.rs`)

定义了 Subnet-EVM 链的初始状态，包括账户余额、链参数和共识规则。

#### 主要类型

- `Genesis`：创世块配置
- `ChainConfig`：链配置参数
- `FeeConfig`：费用配置
- `AllocAccount`：预分配账户

#### 主要字段

- `config`：链配置参数
- `alloc`：初始账户分配
- `gas_limit`：区块 gas 限制
- `timestamp`：创世时间戳

#### 主要方法

- `new()`：创建新的创世配置
- `encode_json()`：将创世配置序列化为 JSON
- `sync()`：将创世配置保存到磁盘

#### 使用示例

```rust
use avalanche_types::subnet_evm::genesis;
use std::collections::BTreeMap;

// 创建默认创世配置
let mut gen = genesis::Genesis::default();

// 自定义链 ID
if let Some(chain_config) = &mut gen.config {
    chain_config.chain_id = Some(54321);
}

// 添加预分配账户
let seed_addresses = vec![
    "0x8db97C7cEcE249c2b98bDC0226Cc4C2A57BF52FC".to_string()
];
let custom_gen = genesis::Genesis::new(&seed_addresses).unwrap();

// 保存到文件
custom_gen.sync("/path/to/genesis.json").unwrap();
```

## 设计模式

### 1. 构建者模式 (Builder Pattern)

通过提供默认配置和可选参数，允许用户逐步构建复杂的配置对象。

```rust
let mut config = Config::default();
config.pruning_enabled = Some(true);
config.log_level = Some(String::from("debug"));
```

### 2. 工厂模式 (Factory Pattern)

`Genesis::new()` 方法作为工厂函数，根据输入参数创建适当配置的 Genesis 对象。

```rust
let genesis = Genesis::new(&seed_addresses).unwrap();
```

### 3. 序列化/反序列化模式

使用 Serde 框架实现配置的序列化和反序列化，支持与 JSON 格式的互操作。

```rust
let json_str = config.encode_json().unwrap();
```

## 与 EVM 的关系

Subnet-EVM 是 Avalanche 网络上的以太坊虚拟机实现，它：

1. 兼容以太坊的智能合约和交易格式
2. 支持以太坊的 JSON-RPC API
3. 可以使用以太坊工具（如 MetaMask、Remix、Hardhat）进行开发
4. 但运行在 Avalanche 子网上，享受更高的性能和更低的费用

`subnet_evm` 模块提供了配置和管理这些子网所需的工具。

## 与其他模块的关系

- **subnet/**：提供子网 SDK，而 subnet_evm 是特定于 EVM 的子网实现
- **evm/**：提供通用的 EVM 工具，如 ABI 编码、交易构建
- **wallet/evm/**：提供与 EVM 兼容的钱包功能
- **coreth/**：提供 C-Chain（Avalanche 主网的 EVM 链）的配置

## 使用场景

### 1. 创建自定义 EVM 子网

```rust
// 创建子网创世配置
let seed_addresses = vec![
    "0x8db97C7cEcE249c2b98bDC0226Cc4C2A57BF52FC".to_string()
];
let genesis = Genesis::new(&seed_addresses).unwrap();

// 自定义链参数
if let Some(chain_config) = &mut genesis.config {
    chain_config.chain_id = Some(54321);
    
    if let Some(fee_config) = &mut chain_config.fee_config {
        fee_config.gas_limit = 15_000_000;
    }
}

// 保存创世配置
genesis.sync("path/to/genesis.json").unwrap();

// 创建链配置
let chain_config = Config::default();
chain_config.sync("path/to/config.json").unwrap();
```

### 2. 与现有子网交互

使用 `avalanche-types` 中的其他模块（如 `jsonrpc_client` 和 `wallet_evm`）与子网 EVM 交互：

```rust
// 示例代码 - 需要启用相应的特性标志
use avalanche_types::{
    key,
    wallet,
    jsonrpc::client::evm as json_client_evm,
};

// 连接到子网 EVM
let chain_rpc_url = "http://node:9650/ext/bc/mySubnetID/rpc";
let chain_id = json_client_evm::chain_id(chain_rpc_url).await.unwrap();

// 创建钱包
let private_key = key::secp256k1::private_key::Key::generate().unwrap();
let signer = private_key.to_ethers_core_signing_key().into();
let wallet = wallet::Builder::new(&private_key)
    .base_http_url(chain_rpc_url)
    .build()
    .await
    .unwrap();
let evm_wallet = wallet.evm(&signer, chain_rpc_url, chain_id).unwrap();

// 发送交易
let tx_id = evm_wallet
    .eip1559()
    .recipient(recipient_address)
    .value(transfer_amount)
    .submit()
    .await
    .unwrap();
```

## 参考资源

- [Avalanche 子网文档](https://docs.avax.network/subnets)
- [Subnet-EVM GitHub 仓库](https://github.com/ava-labs/subnet-evm)
- [创建 EVM 子网教程](https://docs.avax.network/subnets/create-evm-subnet)
