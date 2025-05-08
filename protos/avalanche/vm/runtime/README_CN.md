# 虚拟机运行时接口定义

本目录包含 Avalanche 虚拟机运行时 (VM Runtime) 的 Protocol Buffers 定义，用于管理虚拟机进程的生命周期和通信。

## 目录

1. [概述](#概述)
2. [文件说明](#文件说明)
3. [核心接口](#核心接口)
4. [设计模式](#设计模式)
5. [使用示例](#使用示例)
6. [版本兼容性](#版本兼容性)

## 概述

虚拟机运行时 (VM Runtime) 是 Avalanche 节点与虚拟机进程之间的桥梁，负责虚拟机的启动、初始化和通信。这种设计允许虚拟机作为独立进程运行，提高了系统的模块化、安全性和可扩展性。

运行时接口定义了 Avalanche 节点如何与虚拟机进程通信，包括初始化参数传递、服务发现和生命周期管理。

## 文件说明

### runtime.proto

`runtime.proto` 是主要文件，定义了虚拟机运行时的 gRPC 服务接口。该文件包含：

- `Runtime` 服务：定义了运行时的 RPC 方法
- 初始化请求和响应消息：定义了虚拟机初始化所需的参数
- 错误类型：定义了运行时可能返回的错误

## 核心接口

### Runtime 服务

`Runtime` 服务定义了以下核心方法：

1. **初始化**
   - `Initialize`：初始化虚拟机运行时，提供协议版本和通信参数

```protobuf
service Runtime {
  rpc Initialize(InitializeRequest) returns (InitializeResponse);
}
```

### 初始化请求

`InitializeRequest` 消息包含初始化虚拟机运行时所需的参数：

```protobuf
message InitializeRequest {
  // 协议版本
  uint32 protocol_version = 1;
  
  // 虚拟机的 gRPC 服务器地址
  string server_addr = 2;
}
```

### 初始化响应

`InitializeResponse` 消息包含初始化结果：

```protobuf
message InitializeResponse {
  // 可选的错误信息
  Error error = 1;
}
```

### 错误类型

```protobuf
enum Error {
  // 未指定错误，表示成功
  ERROR_UNSPECIFIED = 0;
  
  // 协议版本不兼容
  ERROR_INCOMPATIBLE_PROTOCOL_VERSION = 1;
}
```

## 设计模式

### 1. 进程间通信

运行时接口采用 gRPC 实现进程间通信，允许虚拟机作为独立进程运行：

```
Avalanche 节点 <--gRPC--> 虚拟机进程
```

这种设计提供了以下优势：
- 进程隔离：虚拟机崩溃不会影响主节点
- 语言无关：虚拟机可以用任何支持 gRPC 的语言实现
- 资源控制：可以限制虚拟机的资源使用

### 2. 服务发现

运行时接口使用服务发现模式，通过初始化请求传递服务地址：

```protobuf
message InitializeRequest {
  string server_addr = 2;
}
```

这允许虚拟机知道如何连接回 Avalanche 节点提供的服务。

### 3. 版本协商

运行时接口使用版本协商模式，确保节点和虚拟机使用兼容的协议版本：

```protobuf
message InitializeRequest {
  uint32 protocol_version = 1;
}

message InitializeResponse {
  Error error = 1;
}

enum Error {
  ERROR_INCOMPATIBLE_PROTOCOL_VERSION = 1;
}
```

如果协议版本不兼容，虚拟机可以返回错误，防止使用不兼容的接口。

## 使用示例

### 在 Rust 中实现运行时服务

```rust
use tonic::{Request, Response, Status};
use avalanche_types::proto::pb::vm::runtime::{
    runtime_server::Runtime, InitializeRequest, InitializeResponse, Error,
};

pub struct RuntimeService {
    // 运行时状态
}

#[tonic::async_trait]
impl Runtime for RuntimeService {
    async fn initialize(
        &self,
        request: Request<InitializeRequest>,
    ) -> Result<Response<InitializeResponse>, Status> {
        let req = request.into_inner();
        
        // 检查协议版本兼容性
        if req.protocol_version != SUPPORTED_PROTOCOL_VERSION {
            return Ok(Response::new(InitializeResponse {
                error: Error::ErrorIncompatibleProtocolVersion as i32,
            }));
        }
        
        // 存储服务器地址，用于后续通信
        println!("初始化运行时，服务器地址: {}", req.server_addr);
        
        // 返回成功响应
        Ok(Response::new(InitializeResponse {
            error: Error::ErrorUnspecified as i32, // 0 表示无错误
        }))
    }
}
```

### 在 Rust 中调用运行时服务

```rust
use avalanche_types::proto::pb::vm::runtime::{
    runtime_client::RuntimeClient, InitializeRequest,
};

async fn initialize_runtime() -> Result<(), Box<dyn std::error::Error>> {
    // 连接到运行时服务
    let mut client = RuntimeClient::connect("http://[::1]:50051").await?;
    
    // 创建初始化请求
    let request = InitializeRequest {
        protocol_version: 33, // 当前协议版本
        server_addr: "http://[::1]:50052".to_string(), // 节点服务地址
    };
    
    // 发送请求
    let response = client.initialize(request).await?;
    let result = response.into_inner();
    
    // 检查是否有错误
    if result.error != 0 {
        println!("运行时初始化失败，错误码: {}", result.error);
        return Err("初始化失败".into());
    }
    
    println!("运行时初始化成功");
    
    Ok(())
}
```

## 版本兼容性

运行时接口的版本兼容性考虑：

1. **协议版本**：`protocol_version` 字段用于确保节点和虚拟机使用兼容的协议
2. **向后兼容性**：新版本应保持向后兼容，不删除或修改现有字段
3. **错误处理**：通过 `Error` 枚举提供标准化的错误处理
4. **服务扩展**：可以通过添加新的 RPC 方法扩展服务，但不应修改现有方法的语义
