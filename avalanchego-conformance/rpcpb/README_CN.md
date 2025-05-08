# RPC 协议定义

本目录包含 AvalancheGo 一致性测试框架的 RPC 协议定义，使用 Protocol Buffers 和 gRPC 实现跨语言通信。

## 目录

1. [文件结构](#文件结构)
2. [设计模式](#设计模式)
3. [核心功能](#核心功能)
4. [使用示例](#使用示例)
5. [扩展指南](#扩展指南)

## 文件结构

- `*.proto`: Protocol Buffers 定义文件
  - `rpcpb.proto`: 主要服务和消息定义
- `*.pb.go`: 自动生成的 Go 代码
- `*_grpc.pb.go`: 自动生成的 gRPC 服务代码

## 设计模式

### 接口定义模式

使用 Protocol Buffers 定义服务接口和消息结构：

```protobuf
service ConformanceService {
    // 服务健康检查
    rpc PingService(PingServiceRequest) returns (PingServiceResponse) {}
    
    // 测试消息序列化
    rpc TestMessage(TestMessageRequest) returns (TestMessageResponse) {}
    
    // 测试密钥操作
    rpc TestKey(TestKeyRequest) returns (TestKeyResponse) {}
}
```

### 版本控制模式

使用 Protocol Buffers 的版本控制机制管理接口变更：

```protobuf
syntax = "proto3";

package rpcpb;

option go_package = "github.com/ava-labs/avalanche-rs/avalanchego-conformance/rpcpb";
```

### 数据模型模式

使用消息定义数据模型：

```protobuf
message TestMessageRequest {
    string message_type = 1;
    bytes message_bytes = 2;
}

message TestMessageResponse {
    bool success = 1;
    string error_message = 2;
    bytes expected_bytes = 3;
}
```

## 核心功能

### 服务定义

定义了一致性测试服务接口：

- `PingService`: 服务健康检查
- `TestMessage`: 测试消息序列化
- `TestKey`: 测试密钥操作
- `TestPacker`: 测试数据打包

### 消息定义

定义了各种请求和响应消息：

- `PingServiceRequest`/`PingServiceResponse`: 健康检查
- `TestMessageRequest`/`TestMessageResponse`: 消息测试
- `TestKeyRequest`/`TestKeyResponse`: 密钥测试
- `TestPackerRequest`/`TestPackerResponse`: 打包测试

### 错误处理

定义了统一的错误处理机制：

- 使用 `success` 字段表示操作是否成功
- 使用 `error_message` 字段提供错误详情
- 使用特定的错误代码表示不同类型的错误

## 使用示例

### 定义新服务

```protobuf
// 在 rpcpb.proto 中添加新服务
service ConformanceService {
    // 现有方法
    rpc PingService(PingServiceRequest) returns (PingServiceResponse) {}
    
    // 新方法
    rpc TestNewFeature(TestNewFeatureRequest) returns (TestNewFeatureResponse) {}
}

// 定义新请求消息
message TestNewFeatureRequest {
    string feature_name = 1;
    bytes input_data = 2;
}

// 定义新响应消息
message TestNewFeatureResponse {
    bool success = 1;
    string error_message = 2;
    bytes result_data = 3;
}
```

### 生成代码

使用 `buf` 工具生成代码：

```bash
buf generate
```

### 使用生成的代码

在 Go 中使用生成的代码：

```go
import (
    "context"
    "github.com/ava-labs/avalanche-rs/avalanchego-conformance/rpcpb"
    "google.golang.org/grpc"
)

func main() {
    // 创建 gRPC 连接
    conn, err := grpc.Dial("localhost:8080", grpc.WithInsecure())
    if err != nil {
        panic(err)
    }
    defer conn.Close()
    
    // 创建客户端
    client := rpcpb.NewConformanceServiceClient(conn)
    
    // 调用服务
    resp, err := client.PingService(context.Background(), &rpcpb.PingServiceRequest{})
    if err != nil {
        panic(err)
    }
    
    fmt.Printf("服务响应: %+v\n", resp)
}
```

## 扩展指南

### 添加新服务

要添加新服务，请按照以下步骤操作：

1. 在 `.proto` 文件中定义新服务：

```protobuf
service NewService {
    rpc Method1(Method1Request) returns (Method1Response) {}
    rpc Method2(Method2Request) returns (Method2Response) {}
}
```

2. 定义请求和响应消息：

```protobuf
message Method1Request {
    // 字段定义
}

message Method1Response {
    // 字段定义
}
```

3. 生成代码：

```bash
buf generate
```

4. 实现服务接口。

### 更新现有服务

要更新现有服务，请按照以下步骤操作：

1. 在 `.proto` 文件中添加新方法：

```protobuf
service ConformanceService {
    // 现有方法
    rpc PingService(PingServiceRequest) returns (PingServiceResponse) {}
    
    // 新方法
    rpc NewMethod(NewMethodRequest) returns (NewMethodResponse) {}
}
```

2. 定义新的请求和响应消息：

```protobuf
message NewMethodRequest {
    // 字段定义
}

message NewMethodResponse {
    // 字段定义
}
```

3. 生成代码：

```bash
buf generate
```

4. 实现新方法。

### 版本兼容性

在更新 Protocol Buffers 定义时，请遵循以下规则以保持向后兼容性：

1. 不要更改现有字段的标签号
2. 不要删除必填字段
3. 不要重新使用已删除字段的标签号
4. 添加新字段时使用新的标签号
5. 将新字段设为可选字段
