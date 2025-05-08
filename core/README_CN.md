# Avalanche-rs Core 组件

本目录包含 Avalanche-rs 项目的核心基础组件，这些组件为 Avalanche 网络节点提供底层功能支持。

## 目录

1. [目录结构](#目录结构)
2. [设计理念](#设计理念)
3. [组件详细说明](#组件详细说明)
4. [组件间的关系](#组件间的关系)
5. [构建和测试](#构建和测试)
6. [特性和功能](#特性和功能)
7. [贡献指南](#贡献指南)

## 目录结构

```
core/
├── cert-manager/     # 证书管理工具
│   ├── src/          # 源代码目录
│   │   ├── lib.rs    # 模块入口
│   │   └── x509.rs   # X.509 证书实现
│   └── Cargo.toml    # 项目配置
├── network/          # 网络通信组件
│   ├── examples/     # 使用示例
│   ├── src/          # 源代码目录
│   │   ├── lib.rs    # 模块入口
│   │   └── peer/     # 对等节点管理
│   │       ├── mod.rs     # 模块入口
│   │       ├── inbound.rs # 入站连接处理
│   │       └── outbound.rs # 出站连接管理
│   └── Cargo.toml    # 项目配置
└── server/           # HTTP 服务器组件
    ├── src/          # 源代码目录
    │   ├── lib.rs    # 模块入口
    │   └── handler.rs # 请求处理器
    └── Cargo.toml    # 项目配置
```

## 设计理念

Avalanche-rs Core 组件采用模块化设计，将底层基础设施功能分解为独立的、可重用的组件。这种设计有以下优势：

1. **关注点分离**：每个组件专注于特定的功能领域
2. **可重用性**：组件可以被其他 Avalanche 生态系统项目重用
3. **可测试性**：独立组件更容易进行单元测试和集成测试
4. **可维护性**：模块化设计使代码更易于理解和维护
5. **灵活性**：组件可以独立升级和替换
6. **并行开发**：不同团队可以并行开发不同组件

## 组件详细说明

### cert-manager

**版本**：0.0.11
**描述**：证书管理工具
**路径**：`/core/cert-manager`

这个组件提供了 X.509 证书的生成、管理和验证功能，用于 Avalanche 网络中的 TLS 通信。

#### 主要功能

1. **证书生成**：创建自签名 X.509 证书
2. **密钥管理**：生成和管理 RSA/EC 密钥对
3. **证书验证**：验证证书的有效性和信任链
4. **PEM 格式支持**：支持 PEM 格式的证书和密钥文件
5. **TLS 配置**：为 Rustls 提供 TLS 配置

#### 主要模块

- **x509.rs**：X.509 证书操作的核心功能
- **lib.rs**：提供 CertManager 和 CertConfig 等主要类型

#### 使用示例

```rust
use cert_manager::x509;
use std::path::Path;

// 生成证书和密钥
let key_path = Path::new("key.pem");
let cert_path = Path::new("cert.pem");
x509::generate_and_write_pem(None, key_path, cert_path)?;

// 加载证书和密钥
let (private_key, certificate) = x509::load_pem_key_cert_to_der(key_path, cert_path)?;
```

### network

**版本**：0.0.1
**描述**：网络通信组件
**路径**：`/core/network`

这个组件提供了 Avalanche 网络中节点间通信的功能，包括 P2P 连接的建立和管理。

#### 主要功能

1. **P2P 连接**：建立和管理点对点连接
2. **TLS 通信**：使用 TLS 加密的安全通信
3. **入站连接处理**：处理来自其他节点的连接请求
4. **出站连接管理**：主动连接到其他节点
5. **消息传递**：在节点之间传递消息

#### 主要模块

- **peer/mod.rs**：定义 Peer 类型和通用功能
- **peer/inbound.rs**：处理入站连接
- **peer/outbound.rs**：管理出站连接

#### 使用示例

```rust
use network::peer::outbound;
use std::time::Duration;

// 创建 TLS 连接器
let connector = outbound::Connector::new_from_pem(&client_key_path, &client_cert_path)?;

// 连接到远程节点
let stream = connector.connect(remote_addr, Duration::from_secs(5))?;

// 创建 Peer 对象
let peer = network::peer::Peer::new(stream);
```

### server

**版本**：0.0.1
**描述**：HTTP 服务器组件
**路径**：`/core/server`

这个组件提供了 HTTP 服务器功能，用于处理 Avalanche 节点的 API 请求。

#### 主要功能

1. **HTTP 服务器**：提供 HTTP API 端点
2. **请求处理**：处理和路由 HTTP 请求
3. **健康检查**：提供节点健康状态检查
4. **API 端点**：实现 Avalanche 标准 API 端点
5. **优雅关闭**：支持优雅关闭服务器

#### 主要模块

- **handler.rs**：HTTP 请求处理器
- **lib.rs**：服务器组件的主要入口点

#### 使用示例

```rust
use server::handler::Handler;

// 创建 HTTP 处理器
let handler = Handler::new("0.0.0.0", 9650, std::time::Duration::from_secs(10));

// 启动服务器
handler.start().await?;
```

## 组件间的关系

- **cert-manager** 被 **network** 使用，为 P2P 连接提供 TLS 证书
- **network** 和 **server** 可以协同工作，分别处理 P2P 通信和 HTTP API 请求

## 构建和测试

每个组件都可以独立构建和测试：

```bash
# 构建 cert-manager
cd core/cert-manager
cargo build

# 测试 cert-manager
cargo test

# 构建 network
cd ../network
cargo build

# 测试 network
cargo test

# 构建 server
cd ../server
cargo build

# 测试 server
cargo test
```

## 特性和功能

1. **安全通信**：使用 TLS 加密的安全通信
2. **高性能**：异步 I/O 和高效的资源管理
3. **可靠性**：错误处理和优雅降级
4. **可扩展性**：模块化设计便于扩展
5. **标准兼容**：遵循 Avalanche 网络协议标准

## 贡献指南

贡献新功能或修复时，请遵循以下原则：

1. 保持向后兼容性
2. 添加适当的测试
3. 更新文档
4. 遵循现有的代码风格和命名约定

## 许可证

这些组件使用与 Avalanche-rs 项目相同的许可证。
