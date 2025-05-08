# Avalanche Protocol Buffers 模块

本目录包含 Avalanche 区块链网络中使用的 Protocol Buffers (protobuf) 定义和生成的代码。Protocol Buffers 是一种与语言无关、平台无关的可扩展机制，用于序列化结构化数据，在 Avalanche 网络中用于节点间通信和数据交换。

## 目录

1. [模块概述](#模块概述)
2. [设计模式](#设计模式)
3. [目录结构](#目录结构)
4. [主要组件](#主要组件)
   - [网络通信](#网络通信)
   - [虚拟机接口](#虚拟机接口)
   - [数据库接口](#数据库接口)
   - [共享内存](#共享内存)
   - [其他服务](#其他服务)
5. [代码生成](#代码生成)
6. [使用示例](#使用示例)

## 模块概述

Avalanche Protocol Buffers 模块是 Avalanche 网络中跨语言通信的基础。它定义了：

- 节点间通信协议
- 虚拟机 (VM) 接口
- 数据库接口
- 共享内存接口
- 各种服务的 RPC 接口

这些定义以 `.proto` 文件的形式存储，并通过 `buf` 工具生成对应的 Rust 代码。生成的代码提供了类型安全的 API，用于序列化和反序列化网络消息，以及通过 gRPC 进行远程过程调用。

当前支持的协议版本为 **33**，与 AvalancheGo v1.11.0 兼容。

## 设计模式

### 接口定义模式

使用 Protocol Buffers 定义接口，实现语言无关的通信协议：

- 消息结构定义：使用 `.proto` 文件定义消息结构
- 服务接口定义：使用 gRPC 定义远程服务接口
- 版本控制：通过协议版本号管理兼容性

### 代码生成模式

使用自动化工具从接口定义生成代码：

- 使用 `buf` 工具管理 protobuf 文件
- 使用 `prost` 和 `tonic` 生成 Rust 代码
- 生成的代码包含序列化/反序列化和 RPC 客户端/服务器实现

### 适配器模式

提供与 Avalanche 其他组件的兼容性：

- 将 protobuf 消息转换为 Rust 原生类型
- 将 Rust 原生类型转换为 protobuf 消息

## 目录结构

- `pb/`: 生成的 Rust 代码目录
  - 包含从 `.proto` 文件生成的所有 Rust 代码
  - 每个 `.proto` 文件对应一个或多个 `.rs` 文件
  - `.tonic.rs` 文件包含 gRPC 服务定义

- `protos/`: 原始 Protocol Buffers 定义目录
  - `avalanche/`: Avalanche 特定的 protobuf 定义
  - `static/`: 静态依赖的 protobuf 定义

- 配置文件：
  - `buf.yaml`: buf 工具的主配置文件
  - `buf.gen.yaml`: 代码生成配置
  - `buf.work.yaml`: 工作空间配置
  - `buf.lock`: 依赖锁定文件

## 主要组件

### 网络通信

#### p2p.rs

定义了节点间通信的消息格式，包括：

- 握手消息：`Handshake`、`PeerList`
- 网络管理消息：`Ping`、`Pong`、`GetPeerList`
- 共识消息：`PushQuery`、`PullQuery`、`Chits`
- 数据同步消息：`Get`、`Put`、`GetAncestors`、`Ancestors`
- 引导消息：`GetAcceptedFrontier`、`AcceptedFrontier`
- 状态同步消息：`GetStateSummaryFrontier`、`StateSummaryFrontier`
- 应用层消息：`AppRequest`、`AppResponse`、`AppGossip`

这些消息是 Avalanche 网络协议的核心，用于节点间的数据交换和共识达成。

### 虚拟机接口

#### vm.rs 和 vm.runtime.rs

定义了虚拟机 (VM) 接口，允许自定义区块链逻辑：

- `vm.rs`：定义了 VM 的核心接口，包括区块处理、交易验证和状态管理
- `vm.runtime.rs`：定义了 VM 运行时环境接口

这些接口使开发者能够创建自定义虚拟机，扩展 Avalanche 的功能。

### 数据库接口

#### rpcdb.rs

定义了远程数据库接口，允许通过 RPC 访问数据库：

- 基本操作：`Get`、`Put`、`Delete`
- 批量操作：`Batch`、`WriteBatch`
- 迭代器操作：`NewIterator`、`NewIteratorWithStart`、`NewIteratorWithPrefix`

这个接口使虚拟机能够通过网络访问远程数据库，而不需要直接访问底层存储。

### 共享内存

#### sharedmemory.rs

定义了共享内存接口，允许不同区块链之间共享数据：

- 原子操作：`Atomic`、`Apply`
- 批量获取：`Get`、`BatchGet`
- 索引操作：`Index`、`RemoveIndex`

共享内存是 Avalanche 多链架构的关键组件，允许 X-Chain、P-Chain 和 C-Chain 之间安全地共享数据。

### 其他服务

#### keystore.rs

定义了密钥存储服务接口，用于管理用户密钥：

- 创建用户：`CreateUser`
- 导入/导出用户：`ImportUser`、`ExportUser`
- 列出用户：`ListUsers`

#### messenger.rs

定义了消息传递服务接口，用于在不同组件之间传递消息：

- 通知：`Notify`、`NotifyAll`

#### appsender.rs

定义了应用消息发送接口，用于向网络中的其他节点发送应用层消息：

- 发送消息：`SendAppRequest`、`SendAppResponse`、`SendAppGossip`、`SendAppError`

## 代码生成

代码生成使用 `buf` 工具和以下插件：

- `buf.build/community/neoeinstein-prost`：生成 Rust 结构体和序列化/反序列化代码
- `buf.build/community/neoeinstein-tonic`：生成 gRPC 客户端和服务器代码
- `prost-crate`：生成 Rust crate 结构

生成的代码位于 `pb/` 目录，包括：

- 消息定义：如 `p2p.rs`、`vm.rs` 等
- gRPC 服务定义：如 `vm.tonic.rs`、`rpcdb.tonic.rs` 等
- 模块组织：`mod.rs`

要更新协议版本，需要修改 `mod.rs` 中的 `PROTOCOL_VERSION` 常量和 `scripts/protobuf_codegen.sh` 脚本中的环境变量，然后运行脚本重新生成代码。

## 使用示例

### 序列化和反序列化消息

```rust
use avalanche_types::proto::pb::p2p;
use prost::Message;

// 创建一个 Ping 消息
let ping = p2p::Ping { uptime: 100 };

// 将消息包装在 Message 枚举中
let message = p2p::Message {
    message: Some(p2p::message::Message::Ping(ping)),
};

// 序列化消息
let encoded = message.encode_to_vec();

// 反序列化消息
let decoded = p2p::Message::decode(encoded.as_slice()).unwrap();

// 处理消息
match decoded.message {
    Some(p2p::message::Message::Ping(ping)) => {
        println!("收到 Ping 消息，运行时间: {}", ping.uptime);
    },
    Some(p2p::message::Message::Pong(_)) => {
        println!("收到 Pong 消息");
    },
    // 处理其他消息类型...
    _ => println!("未知消息类型"),
}
```

### 使用 gRPC 服务

```rust
use avalanche_types::proto::pb::vm::tonic::vm_client::VmClient;
use avalanche_types::proto::pb::vm::InitializeRequest;
use tonic::transport::Channel;

async fn initialize_vm() -> Result<(), Box<dyn std::error::Error>> {
    // 连接到 VM 服务
    let mut client = VmClient::connect("http://localhost:9090").await?;
    
    // 创建初始化请求
    let request = tonic::Request::new(InitializeRequest {
        network_id: 1,
        subnet_id: vec![0; 32],
        chain_id: vec![0; 32],
        node_id: vec![0; 20],
        x_chain_id: vec![0; 32],
        avax_asset_id: vec![0; 32],
        genesis_bytes: vec![],
        upgrade_bytes: vec![],
        config_bytes: vec![],
        db_server_addr: "".to_string(),
    });
    
    // 发送请求
    let response = client.initialize(request).await?;
    println!("VM 初始化响应: {:?}", response);
    
    Ok(())
}
```
