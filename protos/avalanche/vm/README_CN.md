# Avalanche 虚拟机 (VM) 协议定义

本目录包含 Avalanche 区块链中虚拟机 (VM) 接口的 Protocol Buffers 定义。这些定义是 Avalanche 虚拟机架构的核心，允许开发者创建自定义区块链和应用程序。

## 目录

1. [概述](#概述)
2. [设计模式](#设计模式)
3. [文件结构](#文件结构)
4. [VM 接口](#vm-接口)
   - [核心接口](#核心接口)
   - [区块操作](#区块操作)
   - [状态同步](#状态同步)
   - [网络通信](#网络通信)
5. [运行时接口](#运行时接口)
6. [使用示例](#使用示例)

## 概述

Avalanche 虚拟机 (VM) 是 Avalanche 网络中的核心组件，负责处理区块链的业务逻辑。每个区块链都由一个虚拟机实例管理，该实例负责：

- 区块的创建、验证和处理
- 交易的执行和状态管理
- 与其他节点的通信和共识
- 区块链状态的同步

本目录中的 Protocol Buffers 定义了虚拟机的 gRPC 接口，使得虚拟机可以用任何支持 gRPC 的语言实现，并与 Avalanche 节点进行通信。

## 设计模式

### 接口分离原则

VM 接口设计遵循接口分离原则，将不同功能的接口分开定义：

- `ChainVM`：基本的区块链虚拟机接口
- `BatchedChainVM`：支持批量操作的虚拟机接口
- `HeightIndexedChainVM`：支持按高度索引区块的虚拟机接口
- `StateSyncableVM`：支持状态同步的虚拟机接口

### 插件架构

VM 设计采用插件架构，允许虚拟机作为独立进程运行：

- 虚拟机作为插件通过 gRPC 与 Avalanche 节点通信
- 运行时接口管理虚拟机进程的生命周期
- 支持动态加载和卸载虚拟机

### 状态机模式

VM 实现了状态机模式，通过明确定义的状态转换管理区块链：

- 区块验证、接受和拒绝的状态转换
- 引导、同步和正常操作的节点状态
- 状态同步和区块同步的协调

## 文件结构

本目录包含以下文件：

- `vm.proto`：定义了虚拟机的主要 gRPC 接口和消息类型
- `runtime/runtime.proto`：定义了虚拟机运行时的 gRPC 接口

## VM 接口

### 核心接口

`vm.proto` 文件定义了 `VM` 服务，包含以下核心方法：

#### 初始化和生命周期管理

- `Initialize`：初始化虚拟机，提供网络 ID、链 ID 等配置信息
- `SetState`：设置虚拟机的状态（同步中、引导中、正常操作）
- `Shutdown`：关闭虚拟机
- `CreateHandlers`：创建 HTTP 处理程序，用于自定义 API
- `Health`：检查虚拟机的健康状态
- `Version`：获取虚拟机的版本信息

#### 区块操作

- `BuildBlock`：从虚拟机中的数据创建新区块
- `ParseBlock`：从字节流解析区块
- `GetBlock`：获取指定 ID 的区块
- `SetPreference`：通知虚拟机当前首选的区块
- `BlockVerify`：验证区块
- `BlockAccept`：接受区块
- `BlockReject`：拒绝区块

#### 批量操作

- `GetAncestors`：获取指定区块的祖先区块
- `BatchedParseBlock`：批量解析多个区块

#### 高度索引

- `GetBlockIDAtHeight`：获取指定高度的区块 ID

#### 状态同步

- `StateSyncEnabled`：检查状态同步是否启用
- `GetOngoingSyncStateSummary`：获取正在进行的状态同步摘要
- `GetLastStateSummary`：获取最新的状态摘要
- `ParseStateSummary`：解析状态摘要
- `GetStateSummary`：获取指定高度的状态摘要
- `StateSummaryAccept`：接受状态摘要

#### 网络通信

- `Connected`：通知虚拟机节点已连接
- `Disconnected`：通知虚拟机节点已断开连接
- `AppRequest`：处理来自其他节点的应用请求
- `AppRequestFailed`：处理应用请求失败的情况
- `AppResponse`：处理来自其他节点的应用响应
- `AppGossip`：处理来自其他节点的八卦消息

### 消息类型

`vm.proto` 文件还定义了多种消息类型，包括：

- `State`：虚拟机的状态（同步中、引导中、正常操作）
- `Error`：错误类型（未指定、已关闭、未找到、未实现）
- `InitializeRequest`/`InitializeResponse`：初始化请求和响应
- `SetStateRequest`/`SetStateResponse`：设置状态请求和响应
- `BuildBlockRequest`/`BuildBlockResponse`：构建区块请求和响应
- `ParseBlockRequest`/`ParseBlockResponse`：解析区块请求和响应
- `GetBlockRequest`/`GetBlockResponse`：获取区块请求和响应
- 以及其他与区块操作、状态同步和网络通信相关的消息类型

## 运行时接口

`runtime/runtime.proto` 文件定义了 `Runtime` 服务，负责管理虚拟机进程的生命周期：

- `Initialize`：初始化虚拟机运行时，提供协议版本和 gRPC 服务器地址

这个接口允许 Avalanche 节点启动和管理虚拟机进程，建立与虚拟机的通信通道。

## 使用示例

### 实现自定义虚拟机

要实现自定义虚拟机，开发者需要：

1. 实现 `VM` gRPC 服务接口
2. 处理区块的创建、验证和执行
3. 管理区块链状态
4. 与其他节点通信

```go
// Go 语言示例
type MyVM struct {
    // 实现 VM 接口
}

func (vm *MyVM) Initialize(ctx context.Context, req *pb.InitializeRequest) (*pb.InitializeResponse, error) {
    // 初始化虚拟机
    return &pb.InitializeResponse{
        LastAcceptedId: lastAcceptedID,
        Height: currentHeight,
        // ...
    }, nil
}

func (vm *MyVM) BuildBlock(ctx context.Context, req *pb.BuildBlockRequest) (*pb.BuildBlockResponse, error) {
    // 创建新区块
    return &pb.BuildBlockResponse{
        Id: newBlockID,
        ParentId: parentID,
        Bytes: encodedBlock,
        Height: blockHeight,
        // ...
    }, nil
}

// 实现其他方法...
```

### 使用虚拟机 API

Avalanche 节点通过 gRPC 与虚拟机通信：

```go
// Go 语言示例
client := pb.NewVMClient(conn)

// 初始化虚拟机
initResp, err := client.Initialize(ctx, &pb.InitializeRequest{
    NetworkId: networkID,
    SubnetId: subnetID,
    ChainId: chainID,
    // ...
})

// 构建区块
buildResp, err := client.BuildBlock(ctx, &pb.BuildBlockRequest{})

// 验证区块
_, err = client.BlockVerify(ctx, &pb.BlockVerifyRequest{
    Bytes: blockBytes,
})

// 接受区块
_, err = client.BlockAccept(ctx, &pb.BlockAcceptRequest{
    Id: blockID,
})
```

### 在 Rust 中实现虚拟机

使用 Rust 实现虚拟机时，可以利用生成的 gRPC 代码：

```rust
// Rust 语言示例
use avalanche_types::proto::pb::vm::*;

struct MyVM {
    // 虚拟机状态
}

#[tonic::async_trait]
impl vm_server::Vm for MyVM {
    async fn initialize(
        &self,
        request: tonic::Request<InitializeRequest>,
    ) -> Result<tonic::Response<InitializeResponse>, tonic::Status> {
        // 初始化虚拟机
        Ok(tonic::Response::new(InitializeResponse {
            last_accepted_id: last_accepted_id,
            height: current_height,
            // ...
        }))
    }

    async fn build_block(
        &self,
        request: tonic::Request<BuildBlockRequest>,
    ) -> Result<tonic::Response<BuildBlockResponse>, tonic::Status> {
        // 创建新区块
        Ok(tonic::Response::new(BuildBlockResponse {
            id: new_block_id,
            parent_id: parent_id,
            bytes: encoded_block,
            height: block_height,
            // ...
        }))
    }

    // 实现其他方法...
}
```
