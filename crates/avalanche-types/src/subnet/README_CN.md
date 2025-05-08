# Avalanche 子网 SDK

## 概述

`subnet` 模块是 Avalanche 的 Rust SDK，提供了一系列抽象和工具，用于构建纯 Rust 实现的 Avalanche 虚拟机（VM）。该模块包含共识、网络通信和所有 JSON-RPC 端点的实现，使开发者能够轻松创建自定义的区块链虚拟机。

使用该 SDK 已经构建了以下虚拟机：
* 简单 Rust VM：[TimestampVM](https://github.com/ava-labs/timestampvm-rs)
* 复杂 Rust VM：[SpacesVM](https://github.com/ava-labs/spacesvm-rs)

## 目录结构

```
subnet/
├── config/             # 子网配置相关
│   ├── consensus.rs    # 共识参数配置
│   ├── gossip.rs       # 网络通信配置
│   └── mod.rs          # 配置模块入口
└── rpc/                # RPC 相关实现
    ├── consensus/      # 共识接口
    ├── context.rs      # 上下文管理
    ├── database/       # 数据库实现
    ├── errors.rs       # 错误处理
    ├── health.rs       # 健康检查
    ├── http/           # HTTP 服务
    ├── metrics.rs      # 指标收集
    ├── mod.rs          # RPC 模块入口
    ├── plugin.rs       # 插件系统
    ├── runtime/        # 运行时
    ├── snow/           # Snow 共识引擎
    ├── snowman/        # Snowman 区块链
    ├── utils/          # 工具函数
    └── vm/             # 虚拟机实现
```

## 主要组件

### 1. 配置模块 (`config/`)

#### `config/mod.rs`

提供子网配置的核心功能，包括配置的序列化、反序列化和持久化。

主要类型：
- `Config`：子网配置结构体，包含共识参数和网络通信设置

主要方法：
- `encode_json()`：将配置序列化为 JSON 字符串
- `sync()`：将配置保存到磁盘

#### `config/consensus.rs`

定义共识协议的参数配置。

主要类型：
- `SnowballParameters`：雪球共识算法参数
- `Parameters`：扩展的共识参数

关键参数：
- `k`：采样大小
- `alpha`：法定人数阈值
- `beta_virtuous`：良性交易的决策阈值
- `beta_rogue`：恶意交易的决策阈值

#### `config/gossip.rs`

定义网络通信和消息传播的配置。

主要类型：
- `SenderConfig`：消息发送配置

### 2. RPC 模块 (`rpc/`)

#### `rpc/vm/`

虚拟机的 RPC 服务器实现，处理与 Avalanche 节点的通信。

主要文件：
- `server.rs`：实现 gRPC 服务器，处理虚拟机的各种 RPC 请求
- `mod.rs`：提供服务启动和初始化功能

主要类型：
- `Server<V>`：泛型 VM 服务器，封装了底层 VM 实现

主要方法：
- `initialize()`：初始化虚拟机
- `build_block()`：构建新区块
- `set_preference()`：设置首选区块
- `set_state()`：设置虚拟机状态

#### `rpc/database/`

提供数据库抽象和多种数据库实现。

主要子模块：
- `memdb/`：内存数据库实现，适用于测试
- `rpcdb/`：远程数据库客户端和服务器
- `versiondb/`：支持版本控制的数据库
- `corruptabledb/`：可模拟故障的数据库

主要接口：
- `Database`：数据库通用接口
- `KeyValueReaderWriterDeleter`：键值操作接口
- `Iteratee`：迭代器接口
- `Batcher`：批处理接口

#### `rpc/snowman/`

Snowman 共识协议的区块链实现。

主要文件：
- `block.rs`：定义区块接口和链虚拟机

主要接口：
- `Block`：区块接口，定义区块的基本操作
- `ChainVm`：链虚拟机接口，定义区块链操作
- `Getter`：区块获取接口
- `Parser`：区块解析接口

#### `rpc/snow/`

Snow 共识引擎的实现。

主要子模块：
- `engine/`：共识引擎实现
- `validators/`：验证者管理

主要类型：
- `State`：共识状态枚举
- `Context`：共识上下文

## 设计模式

### 1. 接口抽象模式

SDK 大量使用 Rust 的 trait 系统来定义接口，实现组件之间的解耦。例如：

```rust
#[tonic::async_trait]
pub trait Block: Decidable + Send + Sync {
    async fn bytes(&self) -> &[u8];
    async fn height(&self) -> u64;
    async fn timestamp(&self) -> u64;
    async fn parent(&self) -> Id;
    async fn verify(&mut self) -> Result<()>;
}
```

### 2. 组合模式

通过组合多个小型接口来构建复杂功能，例如数据库接口：

```rust
pub trait Database:
    batch::Batcher + CloneBox + KeyValueReaderWriterDeleter + Closer + Checkable + iterator::Iteratee
{
}
```

### 3. 工厂模式

用于创建各种组件的实例，例如数据库和迭代器：

```rust
async fn new_iterator_with_start_and_prefix(
    &self,
    start: &[u8],
    prefix: &[u8],
) -> io::Result<BoxedIterator>
```

### 4. 适配器模式

将不同接口转换为兼容接口，例如 RPC 客户端和服务器之间的适配：

```rust
pub struct Server<V> {
    pub vm: Arc<RwLock<V>>,
    pub stop_ch: broadcast::Sender<()>,
}
```

### 5. 策略模式

允许在运行时选择不同的算法实现，例如不同的数据库实现。

## 使用示例

创建一个简单的子网配置：

```rust
use avalanche_types::subnet::config;

// 创建默认配置
let mut subnet_config = config::Config::default();

// 自定义配置参数
subnet_config.validator_only = true;
subnet_config.consensus_parameters.snowball_parameters.k = 25;
subnet_config.consensus_parameters.snowball_parameters.alpha = 18;

// 保存配置到文件
subnet_config.sync("path/to/config.json").unwrap();
```

## 实现自定义虚拟机

要实现自定义虚拟机，需要实现以下核心接口：

1. `snowman::Block`：定义区块的基本操作
2. `snowman::ChainVm`：定义虚拟机的区块链操作
3. `snow::engine::common::vm::Vm`：定义虚拟机的通用操作

详细的实现步骤请参考 [TimestampVM](https://github.com/ava-labs/timestampvm-rs) 或 [SpacesVM](https://github.com/ava-labs/spacesvm-rs) 的源代码。

## 参考资源

- [如何构建简单的 Rust VM](https://docs.avax.network/build/vm/create/rust-vm)：详细的 SDK 使用教程
- [TimestampVM 模板](https://github.com/ava-labs/timestampvm-rs-template)：快速生成基于 TimestampVM 的项目
