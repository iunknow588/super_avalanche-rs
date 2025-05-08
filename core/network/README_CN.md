# 网络模块 (Network)

本模块实现了 Avalanche 共识协议的网络层功能，负责节点间的通信、连接管理和消息传输。

## 目录

1. [概述](#概述)
2. [目录结构](#目录结构)
3. [核心功能](#核心功能)
4. [设计模式](#设计模式)
5. [使用示例](#使用示例)
6. [配置选项](#配置选项)
7. [API 文档](#api-文档)

## 概述

网络模块是 Avalanche 节点的核心组件之一，负责以下功能：

- 建立和维护与其他节点的 P2P 连接
- 处理入站和出站网络消息
- 实现节点发现和网络拓扑管理
- 提供可靠的消息传递机制
- 确保网络通信的安全性和效率

该模块使用异步 I/O 和 TLS 加密，确保高性能和安全的网络通信。

## 目录结构

- `src/`: 网络层核心实现代码
  - `lib.rs`: 模块入口文件，定义公共 API
  - `peer/`: 对等节点管理实现
    - `mod.rs`: 对等节点模块入口，定义通用类型和接口
    - `inbound.rs`: 入站连接处理
    - `outbound.rs`: 出站连接管理
- `examples/`: 使用示例
  - `simple_server.rs`: 简单服务器示例
  - `simple_client.rs`: 简单客户端示例
  - `tls_communication.rs`: TLS 通信示例
- `Cargo.toml`: 项目配置文件，定义依赖和版本
- `LICENSE`: 许可证文件
- `README.md`: 英文说明文档

## 核心功能

### 1. 连接管理

- **入站连接处理**：接受并处理来自其他节点的连接请求
- **出站连接管理**：主动连接到其他节点
- **连接池**：维护活跃连接的池，支持连接复用
- **连接监控**：监控连接状态，处理断开和重连

### 2. 消息处理

- **消息序列化/反序列化**：支持多种消息格式
- **消息路由**：将消息路由到正确的处理程序
- **消息优先级**：支持消息优先级和队列管理
- **流量控制**：防止网络拥塞和资源耗尽

### 3. 节点发现

- **引导节点**：通过预配置的引导节点加入网络
- **节点交换**：与其他节点交换已知节点信息
- **网络拓扑优化**：维护高效的网络拓扑结构
- **NAT 穿透**：支持在 NAT 环境中的节点发现

### 4. 安全通信

- **TLS 加密**：使用 TLS 1.3 加密所有通信
- **证书验证**：验证节点证书的有效性
- **防重放攻击**：防止消息重放攻击
- **DDoS 防护**：基本的 DDoS 防护机制

## 设计模式

### 1. 观察者模式

用于处理网络事件通知：

```rust
// 网络事件监听器接口
pub trait NetworkEventListener {
    fn on_peer_connected(&self, peer_id: &PeerId);
    fn on_peer_disconnected(&self, peer_id: &PeerId);
    fn on_message_received(&self, peer_id: &PeerId, message: &[u8]);
}

// 注册监听器
pub fn register_listener(&mut self, listener: Box<dyn NetworkEventListener>) {
    self.listeners.push(listener);
}
```

### 2. 工厂模式

用于创建不同类型的网络连接：

```rust
// 连接工厂接口
pub trait ConnectionFactory {
    fn create_connection(&self, address: &SocketAddr) -> Result<Box<dyn Connection>, Error>;
}

// TCP 连接工厂
pub struct TcpConnectionFactory;

// TLS 连接工厂
pub struct TlsConnectionFactory {
    cert_manager: CertManager,
}
```

### 3. 策略模式

实现可插拔的网络传输协议：

```rust
// 传输策略接口
pub trait TransportStrategy {
    fn send(&self, data: &[u8]) -> Result<(), Error>;
    fn receive(&self) -> Result<Vec<u8>, Error>;
}

// TCP 传输策略
pub struct TcpTransport;

// WebSocket 传输策略
pub struct WebSocketTransport;
```

### 4. 命令模式

封装网络操作为命令对象：

```rust
// 网络命令接口
pub trait NetworkCommand {
    fn execute(&self, network: &mut Network) -> Result<(), Error>;
}

// 连接命令
pub struct ConnectCommand {
    address: SocketAddr,
}

// 发送消息命令
pub struct SendMessageCommand {
    peer_id: PeerId,
    message: Vec<u8>,
}
```

## 使用示例

### 创建网络管理器

```rust
use network::{NetworkManager, Config};

// 创建配置
let config = Config {
    listen_address: "0.0.0.0:9651".parse()?,
    max_connections: 100,
    connection_timeout: Duration::from_secs(30),
    // 其他配置...
};

// 创建网络管理器
let mut network = NetworkManager::new(config)?;

// 启动网络服务
network.start()?;
```

### 连接到远程节点

```rust
use network::peer::outbound::Connector;
use std::time::Duration;

// 创建连接器
let connector = Connector::new_from_pem(&client_key_path, &client_cert_path)?;

// 连接到远程节点
let peer = connector.connect("example.com:9651", Duration::from_secs(5))?;

// 发送消息
peer.send(message).await?;

// 接收消息
let response = peer.receive().await?;
```

### 处理入站连接

```rust
use network::peer::inbound::Listener;

// 创建监听器
let mut listener = Listener::new("0.0.0.0:9651", tls_config)?;

// 处理入站连接
while let Some(peer) = listener.accept().await? {
    // 处理新连接
    tokio::spawn(async move {
        while let Some(message) = peer.receive().await? {
            // 处理消息
            let response = process_message(message);
            peer.send(response).await?;
        }
    });
}
```

## 配置选项

网络模块支持以下配置选项：

| 选项 | 描述 | 默认值 |
|------|------|--------|
| `listen_address` | 监听地址和端口 | "0.0.0.0:9651" |
| `max_connections` | 最大连接数 | 100 |
| `connection_timeout` | 连接超时（秒） | 30 |
| `read_buffer_size` | 读取缓冲区大小（字节） | 65536 |
| `write_buffer_size` | 写入缓冲区大小（字节） | 65536 |
| `max_message_size` | 最大消息大小（字节） | 1048576 |
| `idle_timeout` | 空闲连接超时（秒） | 300 |
| `bootstrap_nodes` | 引导节点列表 | [] |
| `enable_upnp` | 是否启用 UPnP | false |
| `enable_nat_pmp` | 是否启用 NAT-PMP | false |

## API 文档

完整的 API 文档请参考代码文档或使用以下命令生成：

```bash
cargo doc --open
```

## 注意事项

- 该模块正在积极开发中，API 可能会发生变化
- 某些高级功能（如 NAT 穿透）可能尚未完全实现
- 性能优化仍在进行中
