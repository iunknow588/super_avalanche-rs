# Avalanche Protocol Buffers 定义

本目录包含 Avalanche 网络协议的 Protocol Buffers (protobuf) 定义文件，这些文件定义了 Avalanche 网络中各组件之间通信的消息格式和服务接口。

## 目录结构与设计模式

Avalanche 的 protobuf 定义遵循以下设计模式：

1. **模块化设计**：每个功能模块都有自己的目录和 proto 文件
2. **服务导向架构**：使用 gRPC 服务定义组件间的交互
3. **版本兼容性**：保留字段和版本标记确保向后兼容性
4. **消息复用**：通过导入共享消息类型减少重复定义
5. **清晰的文档**：每个消息和服务都有详细的注释说明其用途和行为

## 子目录及文件说明

### aliasreader

定义区块链别名读取器接口，用于将区块链 ID 映射到人类可读的别名。

### appsender

定义应用消息发送器接口，允许虚拟机向网络中的其他节点发送消息。

### http

定义 HTTP 服务接口，用于处理 HTTP 请求和响应。

- `http.proto`: 定义基本的 HTTP 服务接口
- `responsewriter/`: 包含 HTTP 响应写入器的定义

### io

包含输入/输出相关的接口定义。

- `reader/`: 定义数据读取接口
- `writer/`: 定义数据写入接口
- `prometheus/`: 包含 Prometheus 监控指标相关定义

### keystore

定义密钥存储服务接口，用于管理用户的密钥和证书。

### messenger

定义节点间消息传递接口，用于在不同的虚拟机和组件之间传递消息。

### net

网络相关接口定义。

- `conn/`: 定义网络连接接口

### p2p

定义点对点网络通信协议，是 Avalanche 网络通信的核心。

- `p2p.proto`: 定义了所有 P2P 消息类型，包括：
  - 网络消息（Ping, Pong, Handshake, PeerList）
  - 状态同步消息（GetStateSummaryFrontier, StateSummaryFrontier）
  - 引导消息（GetAcceptedFrontier, AcceptedFrontier）
  - 共识消息（Get, Put, PushQuery, PullQuery, Chits）
  - 应用消息（AppRequest, AppResponse, AppGossip, AppError）

### platformvm

定义平台虚拟机（P-Chain）的接口和消息类型，用于管理验证者、子网和区块链。

### rpcdb

定义远程数据库访问接口，允许通过 RPC 访问和操作数据库。

### sdk

包含软件开发工具包相关的接口定义。

### sharedmemory

定义共享内存接口，允许不同的区块链之间共享状态。

### sync

定义同步服务接口，用于节点间的数据同步。

### validatorstate

定义验证者状态接口，用于查询验证者信息。

### vm

定义虚拟机接口，是自定义区块链实现的核心。

- `vm.proto`: 定义了虚拟机的核心接口，包括：
  - 初始化和生命周期管理（Initialize, Shutdown）
  - 区块处理（BuildBlock, ParseBlock, GetBlock）
  - 共识操作（BlockVerify, BlockAccept, BlockReject）
  - 状态同步（StateSyncEnabled, GetStateSummary）
  - 网络通信（AppRequest, AppResponse, AppGossip）

- `runtime/`: 包含虚拟机运行时相关定义

### warp

定义 Warp 消息接口，用于跨链通信。

## 使用方式

这些 protobuf 定义文件通过以下流程使用：

1. 使用 `buf` 工具和相关插件生成目标语言的代码
2. 在 Rust 代码中导入生成的类型和服务
3. 使用生成的客户端和服务器代码实现组件间的通信

## 版本管理

当前协议版本为 33，与 AvalancheGo v1.11.0 兼容。版本更新时需要同步更新 `crates/avalanche-types/src/proto/mod.rs` 中的 `PROTOCOL_VERSION` 常量。

## 与 AvalancheGo 的关系

这些 protobuf 定义与 AvalancheGo（Avalanche 的 Go 实现）保持一致，确保 Rust 实现可以与 Go 实现互操作。这对于开发自定义虚拟机和子网至关重要，因为它们需要与主网节点通信。

## 贡献指南

修改 protobuf 定义时，请遵循以下原则：

1. 保持向后兼容性，使用保留字段而不是删除字段
2. 为所有消息和字段添加清晰的注释
3. 遵循现有的命名约定和结构
4. 更新后运行代码生成脚本 `scripts/protobuf_codegen.sh` 更新生成的代码
