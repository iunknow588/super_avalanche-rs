# Avalanche Protocol Buffers 定义

本目录包含 Avalanche 网络协议的 Protocol Buffers (protobuf) 定义文件，这些文件定义了 Avalanche 网络中各组件之间通信的消息格式和服务接口。

## 目录

1. [概述](#概述)
2. [目录结构与设计模式](#目录结构与设计模式)
3. [子目录及文件说明](#子目录及文件说明)
4. [使用方式](#使用方式)
5. [版本管理](#版本管理)
6. [与 AvalancheGo 的关系](#与-avalanchego-的关系)
7. [贡献指南](#贡献指南)

## 概述

Protocol Buffers (protobuf) 是 Google 开发的一种语言中立、平台中立、可扩展的序列化数据结构的方法，用于通信协议、数据存储等。在 Avalanche 网络中，protobuf 用于定义：

- 网络节点间的通信消息
- 虚拟机与主节点之间的 gRPC 接口
- 区块链状态和交易的序列化格式
- 跨链通信协议

这些定义确保了不同语言实现（如 Go、Rust）之间的互操作性，并提供了清晰的接口契约。

## 目录结构与设计模式

Avalanche 的 protobuf 定义遵循以下设计模式：

1. **模块化设计**：每个功能模块都有自己的目录和 proto 文件，便于维护和理解
2. **服务导向架构**：使用 gRPC 服务定义组件间的交互，支持跨语言、跨进程通信
3. **版本兼容性**：使用保留字段和版本标记确保向后兼容性，支持网络升级
4. **消息复用**：通过导入共享消息类型减少重复定义，提高代码复用性
5. **清晰的文档**：每个消息和服务都有详细的注释说明其用途和行为
6. **字段编号管理**：谨慎管理字段编号，避免冲突和混淆
7. **枚举类型**：使用枚举类型表示有限的选项集合，提高代码可读性

## 子目录及文件说明

### p2p/

`p2p/` 目录包含节点间网络通信的消息定义：

- `p2p.proto`: 定义了节点间通信的所有消息类型，包括：
  - 网络握手消息 (Handshake, PeerList)
  - 共识消息 (Get, Put, PushQuery, PullQuery, Chits)
  - 引导消息 (GetAcceptedFrontier, AcceptedFrontier)
  - 状态同步消息 (GetStateSummaryFrontier, StateSummaryFrontier)
  - 应用层消息 (AppRequest, AppResponse, AppGossip)

这些消息构成了 Avalanche 网络的通信基础，使节点能够交换信息、达成共识和同步状态。

### vm/

`vm/` 目录包含虚拟机 (VM) 接口的定义，用于主节点与虚拟机之间的通信：

- `vm.proto`: 定义了虚拟机的 gRPC 服务接口，包括：
  - 区块处理方法 (BuildBlock, ParseBlock, GetBlock)
  - 状态管理方法 (Initialize, Shutdown)
  - 共识相关方法 (SetPreference, BlockVerify)
  - 健康检查和版本信息 (Health, Version)

这些接口允许 Avalanche 节点与不同的虚拟机实现进行交互，支持可插拔的区块链架构。

### vm/runtime/

`vm/runtime/` 目录包含虚拟机运行时的接口定义：

- `runtime.proto`: 定义了虚拟机运行时的 gRPC 服务，包括：
  - 区块链操作 (GetBlock, SetState)
  - 共识操作 (VerifyBlock, AcceptBlock)
  - 网络通信 (Gather, Connected)

这些接口提供了虚拟机与主节点之间的标准化通信方式。

### aliasreader/

`aliasreader/` 目录包含别名读取器的接口定义：

- `aliasreader.proto`: 定义了别名读取器的 gRPC 服务，用于解析和管理区块链别名。

### auth/

`auth/` 目录包含身份验证和授权相关的接口定义：

- `auth.proto`: 定义了身份验证服务的 gRPC 接口，用于节点间的身份验证和权限控制。

### health/

`health/` 目录包含健康检查相关的接口定义：

- `health.proto`: 定义了健康检查服务的 gRPC 接口，用于监控节点和服务的健康状态。

### keystore/

`keystore/` 目录包含密钥存储相关的接口定义：

- `keystore.proto`: 定义了密钥存储服务的 gRPC 接口，用于管理和操作加密密钥。

### messenger/

`messenger/` 目录包含消息传递相关的接口定义：

- `messenger.proto`: 定义了消息传递服务的 gRPC 接口，用于节点间的消息交换。

### platformvm/

`platformvm/` 目录包含平台虚拟机 (P-Chain) 相关的接口定义：

- `platformvm.proto`: 定义了平台虚拟机的特定消息和服务，用于管理验证者、子网和链创建。

### rpcdb/

`rpcdb/` 目录包含远程数据库访问的接口定义：

- `rpcdb.proto`: 定义了远程数据库服务的 gRPC 接口，允许通过 RPC 访问和操作数据库。

### sharedmemory/

`sharedmemory/` 目录包含共享内存相关的接口定义：

- `sharedmemory.proto`: 定义了共享内存服务的 gRPC 接口，用于不同链之间的数据共享。

### subnet/

`subnet/` 目录包含子网相关的接口定义：

- `subnet.proto`: 定义了子网服务的 gRPC 接口，用于子网管理和操作。

### warp/

`warp/` 目录包含 Warp 消息传递相关的接口定义：

- `warp.proto`: 定义了 Warp 消息传递服务的 gRPC 接口，用于跨链通信。

## 使用方式

### 在 Rust 项目中使用

1. **添加依赖**

   在 `Cargo.toml` 中添加 `prost` 和 `tonic` 依赖：

   ```toml
   [dependencies]
   prost = "0.11.0"
   tonic = "0.8.0"

   [build-dependencies]
   tonic-build = "0.8.0"
   ```

2. **创建构建脚本**

   在 `build.rs` 中编译 protobuf 文件：

   ```rust
   fn main() -> Result<(), Box<dyn std::error::Error>> {
       tonic_build::compile_protos("path/to/vm.proto")?;
       tonic_build::compile_protos("path/to/p2p.proto")?;
       Ok(())
   }
   ```

3. **在代码中使用生成的类型**

   ```rust
   // 导入生成的模块
   pub mod vm {
       include!(concat!(env!("OUT_DIR"), "/vm.rs"));
   }

   // 使用生成的类型
   use vm::{InitializeRequest, InitializeResponse};
   ```

### 在 Go 项目中使用

1. **安装工具**

   ```bash
   go install google.golang.org/protobuf/cmd/protoc-gen-go@latest
   go install google.golang.org/grpc/cmd/protoc-gen-go-grpc@latest
   ```

2. **编译 protobuf 文件**

   ```bash
   protoc --go_out=. --go_opt=paths=source_relative \
       --go-grpc_out=. --go-grpc_opt=paths=source_relative \
       path/to/vm.proto
   ```

3. **在代码中使用生成的类型**

   ```go
   import (
       pb "path/to/generated/vm"
   )

   func main() {
       req := &pb.InitializeRequest{
           NetworkId: 1,
           // ...
       }
   }
   ```

## 版本管理

Avalanche 的 protobuf 定义遵循以下版本管理原则：

1. **向后兼容性**：新版本必须与旧版本兼容，不得删除或修改现有字段的类型
2. **保留字段**：使用 `reserved` 关键字标记已删除的字段，防止字段编号重用
3. **可选字段**：新增字段应标记为可选 (`optional`)，确保旧客户端可以处理新消息
4. **枚举扩展**：枚举类型可以添加新值，但不能删除或修改现有值
5. **版本标记**：重要的协议变更通过版本字段或注释明确标记

## 与 AvalancheGo 的关系

这些 protobuf 定义与 AvalancheGo（Avalanche 的官方 Go 实现）保持同步，确保 Rust 实现与 Go 实现之间的互操作性。当 AvalancheGo 更新其 protobuf 定义时，这些文件也应相应更新。

主要对应关系：

- 本项目的 protobuf 定义基于 AvalancheGo 的 `proto/pb` 目录
- 字段名称和编号与 AvalancheGo 保持一致
- 服务接口与 AvalancheGo 定义的接口兼容

## 贡献指南

在修改或添加 protobuf 定义时，请遵循以下准则：

1. **保持兼容性**：确保修改不会破坏现有代码
2. **添加注释**：为每个消息、字段和服务添加清晰的注释
3. **遵循命名约定**：使用 CamelCase 命名消息和服务，使用 snake_case 命名字段
4. **管理字段编号**：为新字段分配唯一的编号，不要重用已删除字段的编号
5. **更新文档**：修改后更新相关文档，包括本 README 文件
6. **运行测试**：确保生成的代码可以正确编译和使用
