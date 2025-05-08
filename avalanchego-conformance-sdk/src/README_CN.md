# AvalancheGo 一致性测试 SDK 源代码

本目录包含 AvalancheGo 一致性测试 SDK 的核心实现代码，提供与 AvalancheGo 一致性测试服务器通信的客户端功能。

## 目录

1. [文件结构](#文件结构)
2. [核心组件](#核心组件)
3. [设计模式](#设计模式)
4. [实现细节](#实现细节)
5. [使用指南](#使用指南)

## 文件结构

- `lib.rs`: 模块入口点，定义公共 API 和客户端实现

## 核心组件

### lib.rs

模块的主入口文件，定义了以下内容：

1. **gRPC 协议定义**
   - 导入 Protocol Buffers 生成的代码
   - 导出 gRPC 消息类型和服务客户端

2. **客户端结构体**
   - `Client<T>`: 主客户端结构体，封装与服务器的通信
   - `GrpcClient<T>`: 内部 gRPC 客户端结构体，管理服务连接

3. **服务接口**
   - `PingServiceClient`: 服务健康检查
   - `KeyServiceClient`: 密钥操作服务
   - `PackerServiceClient`: 数据打包服务
   - `MessageServiceClient`: 网络消息服务

4. **辅助类型**
   - `CertificateToNodeIdArgs`: 证书到节点 ID 转换参数
   - `Secp256k1InfoArgs`: Secp256k1 密钥信息参数

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
        // 实现细节
    }
}
```

### 2. 工厂模式

使用工厂模式创建客户端实例：

```rust
impl Client<Channel> {
    pub async fn new(rpc_endpoint: &str) -> Self {
        // 创建和初始化客户端
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

### 5. 错误处理模式

统一的错误处理和转换：

```rust
.map_err(|e| Error::new(ErrorKind::Other, format!("failed ping_service '{e}'")))
```

## 实现细节

### 1. 客户端初始化

```rust
pub async fn new(rpc_endpoint: &str) -> Self {
    log::info!("creating a new client with {}", rpc_endpoint);
    let ep = String::from(rpc_endpoint);
    let ping_client = PingServiceClient::connect(ep.clone()).await.unwrap();
    let key_client = KeyServiceClient::connect(ep.clone()).await.unwrap();
    let packer_client = PackerServiceClient::connect(ep.clone()).await.unwrap();
    let message_client = MessageServiceClient::connect(ep.clone()).await.unwrap();
    let grpc_client = GrpcClient {
        ping_service_client: Mutex::new(ping_client),
        key_service_client: Mutex::new(key_client),
        packer_service_client: Mutex::new(packer_client),
        message_service_client: Mutex::new(message_client),
    };
    Self {
        rpc_endpoint: String::from(rpc_endpoint),
        grpc_client: Arc::new(grpc_client),
    }
}
```

### 2. 服务健康检查

```rust
pub async fn ping_service(&self) -> io::Result<PingServiceResponse> {
    let mut ping_client = self.grpc_client.ping_service_client.lock().await;
    let req = tonic::Request::new(PingServiceRequest {});
    let resp = ping_client
        .ping_service(req)
        .await
        .map_err(|e| Error::new(ErrorKind::Other, format!("failed ping_service '{e}'")))?;
    Ok(resp.into_inner())
}
```

### 3. 密钥操作

```rust
pub async fn certificate_to_node_id(
    &self,
    req: CertificateToNodeIdRequest,
) -> io::Result<CertificateToNodeIdResponse> {
    let mut cli = self.grpc_client.key_service_client.lock().await;
    let req = tonic::Request::new(req);
    let resp = cli.certificate_to_node_id(req).await.map_err(|e| {
        Error::new(
            ErrorKind::Other,
            format!("failed certificate_to_node_id '{e}'"),
        )
    })?;
    Ok(resp.into_inner())
}

pub async fn secp256k1_info(
    &self,
    req: Secp256k1InfoRequest,
) -> io::Result<Secp256k1InfoResponse> {
    let mut cli = self.grpc_client.key_service_client.lock().await;
    let req = tonic::Request::new(req);
    let resp = cli
        .secp256k1_info(req)
        .await
        .map_err(|e| Error::new(ErrorKind::Other, format!("failed secp256k1_info '{e}'")))?;
    Ok(resp.into_inner())
}
```

### 4. 数据打包

```rust
pub async fn build_vertex(&self, req: BuildVertexRequest) -> io::Result<BuildVertexResponse> {
    let mut cli = self.grpc_client.packer_service_client.lock().await;
    let req = tonic::Request::new(req);
    let resp = cli
        .build_vertex(req)
        .await
        .map_err(|e| Error::new(ErrorKind::Other, format!("failed build_vertex '{e}'")))?;
    Ok(resp.into_inner())
}
```

### 5. 消息处理

```rust
pub async fn version(&self, req: VersionRequest) -> io::Result<VersionResponse> {
    let mut cli = self.grpc_client.message_service_client.lock().await;
    let req = tonic::Request::new(req);
    let resp = cli
        .version(req)
        .await
        .map_err(|e| Error::new(ErrorKind::Other, format!("failed version '{e}'")))?;
    Ok(resp.into_inner())
}
```

## 使用指南

### 1. 创建客户端

```rust
use avalanchego_conformance_sdk::Client;
use tokio::runtime::Runtime;

// 创建运行时
let rt = Runtime::new().unwrap();

// 创建客户端
let client = rt.block_on(Client::new("http://localhost:8080"));
```

### 2. 服务健康检查

```rust
// 发送 ping 请求
let response = rt.block_on(client.ping_service()).expect("failed ping_service");
println!("Ping response: {:?}", response);
```

### 3. 测试密钥操作

```rust
// 测试证书到节点 ID 的转换
let cert_bytes = std::fs::read("cert.pem").unwrap();
let req = CertificateToNodeIdRequest {
    certificate: cert_bytes,
};
let resp = rt.block_on(client.certificate_to_node_id(req)).unwrap();
println!("Node ID: {:?}", resp.node_id);
```

### 4. 测试消息处理

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
let resp = rt.block_on(client.version(req)).unwrap();
println!("Version response: {:?}", resp);
```
