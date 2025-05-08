# 对等节点管理模块

本目录包含对等节点 (Peer) 管理的实现代码，负责 Avalanche 网络中节点间连接的建立、维护和通信。

## 文件结构

- `mod.rs`: 模块入口点，定义通用类型和接口
- `inbound.rs`: 入站连接处理实现
- `outbound.rs`: 出站连接管理实现

## 详细说明

### mod.rs

对等节点模块的入口文件，定义了以下内容：

1. **Peer 结构体**
   - 表示网络中的一个对等节点
   - 包含连接状态、地址信息、TLS 会话等
   - 提供消息发送和接收的方法

2. **PeerState 枚举**
   - 表示节点的不同状态
   - 例如：连接中、已连接、断开连接等

3. **PeerConfig 结构体**
   - 配置节点连接参数
   - 包括超时设置、缓冲区大小等

4. **通用接口**
   - `PeerManager` trait: 定义节点管理接口
   - `MessageHandler` trait: 定义消息处理接口

### inbound.rs

处理入站连接的实现，包含以下功能：

1. **Listener 结构体**
   - 监听网络端口
   - 接受新连接请求

2. **AcceptedConn 结构体**
   - 表示已接受的连接
   - 处理 TLS 握手

3. **连接处理流程**
   - 接受 TCP 连接
   - 执行 TLS 握手
   - 验证客户端证书
   - 创建 Peer 对象

4. **连接限制**
   - 最大连接数控制
   - 连接速率限制

### outbound.rs

管理出站连接的实现，包含以下功能：

1. **Connector 结构体**
   - 创建到远程节点的连接
   - 配置 TLS 客户端

2. **Dialer 结构体**
   - 管理连接尝试
   - 处理连接超时和重试

3. **连接池管理**
   - `PeerPool` 结构体: 管理活跃连接
   - 连接复用策略
   - 负载均衡

4. **连接建立流程**
   - 解析目标地址
   - 建立 TCP 连接
   - 执行 TLS 握手
   - 验证服务器证书
   - 创建 Peer 对象

## 设计模式

1. **工厂模式**
   - `Peer::new()`: 创建 Peer 对象
   - `Connector::new_from_pem()`: 创建连接器

2. **状态模式**
   - `PeerState` 枚举: 管理节点状态转换
   - 不同状态下的行为差异

3. **观察者模式**
   - 连接状态变化通知
   - 网络事件处理

4. **策略模式**
   - 可配置的连接策略
   - 不同类型连接的处理策略

## 使用示例

### 入站连接处理

```rust
// 创建监听器
let listener = Listener::new("0.0.0.0:9651", tls_config)?;

// 接受连接
while let Some(conn) = listener.accept().await? {
    // 处理新连接
    let peer = Peer::new(conn);
    
    // 启动消息处理
    tokio::spawn(async move {
        while let Some(msg) = peer.receive().await? {
            // 处理消息
        }
    });
}
```

### 出站连接建立

```rust
// 创建连接器
let connector = Connector::new_from_pem(&client_key_path, &client_cert_path)?;

// 连接到远程节点
let stream = connector.connect("example.com:9651", Duration::from_secs(5))?;

// 创建 Peer 对象
let peer = Peer::new(stream);

// 发送消息
peer.send(message).await?;
```

## 错误处理

模块定义了专门的错误类型 `PeerError`，处理各种连接和通信错误：

- 连接错误: 无法建立连接
- TLS 错误: 证书验证失败
- 超时错误: 连接或操作超时
- I/O 错误: 网络读写错误

## 依赖关系

主要依赖以下组件：

- `tokio`: 异步 I/O 和任务管理
- `rustls`: TLS 实现
- `cert-manager`: 证书管理
- `bytes`: 高效字节处理

## 未来改进

1. 实现更高效的连接池管理
2. 添加连接质量监控
3. 实现智能路由选择
4. 优化重连机制
