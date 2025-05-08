# AvalancheGo 一致性测试 SDK

本目录包含 AvalancheGo 一致性测试的 Rust SDK，用于与 AvalancheGo 一致性测试服务器通信，验证 Rust 实现与 Go 实现的兼容性。

## 目录

1. [概述](#概述)
2. [目录结构](#目录结构)
3. [核心功能](#核心功能)
4. [设计模式](#设计模式)
5. [使用示例](#使用示例)
6. [构建和测试](#构建和测试)
7. [API 文档](#api-文档)

## 概述

AvalancheGo 一致性测试 SDK 是一个 Rust 库，提供了与 AvalancheGo 一致性测试服务器通信的客户端实现。该 SDK 允许开发者验证 Rust 实现的 Avalanche 组件与官方 Go 实现 (AvalancheGo) 的行为一致性，确保不同语言实现之间的互操作性。

主要功能包括：

- 连接到一致性测试服务器
- 执行各种一致性测试
- 验证消息序列化/反序列化
- 测试密钥操作和签名
- 验证数据打包格式

## 目录结构

- `src/`: SDK 核心实现
  - `lib.rs`: 主入口文件，定义公共 API 和客户端实现
- `examples/`: 使用示例
  - `ping_service.rs`: 演示如何连接服务器并发送 ping 请求
- `scripts/`: 构建和测试脚本
  - `build.release.sh`: 构建发布版本的脚本
- `build.rs`: 构建脚本，用于生成 gRPC 代码
- `Cargo.toml`: Rust 项目配置文件
- `LICENSE`: 项目许可证文件

## 核心功能

### 客户端实现

SDK 提供了一个 gRPC 客户端，用于与一致性测试服务器通信：

- **连接管理**：建立和维护与服务器的连接
- **请求处理**：发送请求并处理响应
- **错误处理**：处理网络错误和服务器错误
- **超时控制**：支持请求超时设置

### 测试功能

SDK 支持以下测试功能：

1. **服务健康检查**
   - 验证服务器是否正常运行
   - 检查服务器版本兼容性

2. **消息测试**
   - 测试网络消息的序列化和反序列化
   - 验证消息格式的兼容性
   - 支持各种消息类型（Ping、Pong、Version 等）

3. **密钥测试**
   - 测试密钥生成和导入
   - 验证签名和验证操作
   - 测试地址生成

4. **数据打包测试**
   - 测试数据打包和解包
   - 验证二进制格式兼容性

## 设计模式

### 1. 客户端-服务器模式

SDK 使用客户端-服务器架构与一致性测试服务器通信：

```rust
pub struct Client<T> {
    pub rpc_endpoint: String,
    pub grpc_client: Arc<GrpcClient<T>>,
}

impl Client<Channel> {
    pub async fn ping_service(&self) -> io::Result<PingServiceResponse> {
        let mut ping_client = self.grpc_client.ping_service_client.lock().await;
        let req = tonic::Request::new(PingServiceRequest {});
        let resp = ping_client.ping_service(req).await?;
        Ok(resp.into_inner())
    }
}
```

### 2. 工厂模式

使用工厂模式创建客户端实例：

```rust
impl Client<Channel> {
    pub async fn new(rpc_endpoint: &str) -> Self {
        let ep = String::from(rpc_endpoint);
        let ping_client = PingServiceClient::connect(ep.clone()).await.unwrap();
        let key_client = KeyServiceClient::connect(ep.clone()).await.unwrap();
        // ...
        Self {
            rpc_endpoint: String::from(rpc_endpoint),
            grpc_client: Arc::new(grpc_client),
        }
    }
}
```

### 3. 适配器模式

将 Rust 类型转换为 gRPC 消息，反之亦然：

```rust
// 将 Rust 类型转换为 gRPC 请求
let req = tonic::Request::new(PingServiceRequest {});

// 将 gRPC 响应转换回 Rust 类型
let resp = ping_client.ping_service(req).await?;
let ping_response = resp.into_inner();
```

### 4. 互斥锁模式

使用互斥锁保护共享资源：

```rust
pub struct GrpcClient<T> {
    pub ping_service_client: Mutex<PingServiceClient<T>>,
    pub key_service_client: Mutex<KeyServiceClient<T>>,
    // ...
}

// 使用锁访问客户端
let mut ping_client = self.grpc_client.ping_service_client.lock().await;
```

## 使用示例

### 基本用法

```rust
use avalanchego_conformance_sdk::Client;
use tokio::runtime::Runtime;

fn main() {
    // 创建运行时
    let rt = Runtime::new().unwrap();

    // 创建客户端
    let client = rt.block_on(Client::new("http://localhost:8080"));

    // 发送 ping 请求
    let response = rt.block_on(client.ping_service()).expect("failed ping_service");
    println!("Ping response: {:?}", response);
}
```

### 测试密钥操作

```rust
// 测试证书到节点 ID 的转换
let cert_bytes = std::fs::read("cert.pem").unwrap();
let req = CertificateToNodeIdRequest {
    certificate: cert_bytes,
};
let resp = client.certificate_to_node_id(req).await?;
println!("Node ID: {:?}", resp.node_id);

// 测试 secp256k1 密钥信息
let req = Secp256k1InfoRequest {
    private_key: "private_key_here".to_string(),
};
let resp = client.secp256k1_info(req).await?;
println!("Public key: {:?}", resp.public_key);
```

### 测试消息处理

```rust
// 测试版本消息
let req = VersionRequest {
    network_id: 5,
    my_time: 123456789,
    ip_addr: vec![192, 168, 1, 1],
    my_version: "avalanche/1.0.0".to_string(),
    my_version_time: 123456700,
    sig: vec![],
    tracked_subnets: vec![],
};
let resp = client.version(req).await?;
println!("Version response: {:?}", resp);
```

## 构建和测试

### 构建 SDK

```bash
# 使用提供的脚本构建发布版本
./scripts/build.release.sh

# 或者直接使用 cargo
cargo build --release
```

### 运行示例

```bash
# 运行 ping 服务示例
cargo run --example ping_service -- http://localhost:8080
```

### 运行测试

```bash
# 运行单元测试
cargo test

# 运行文档测试
cargo test --doc
```

## API 文档

生成并查看 API 文档：

```bash
cargo doc --open
```

详细的 API 文档包含了所有公共接口的说明和使用示例。
