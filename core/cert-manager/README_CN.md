# 证书管理模块 (Cert Manager)

本模块负责 Avalanche 节点的证书生命周期管理，包括生成、验证和续期。它基于 X.509 标准实现，确保网络通信的安全性和身份验证。

## 目录

1. [概述](#概述)
2. [目录结构](#目录结构)
3. [核心功能](#核心功能)
4. [设计模式](#设计模式)
5. [使用示例](#使用示例)
6. [配置选项](#配置选项)
7. [API 文档](#api-文档)

## 概述

X.509 是一种公钥基础设施 (PKI) 标准，用于数字证书的管理与验证，确保通信双方的身份真实性和数据的安全性。它主要包括：

- **身份验证**：通过数字证书确认通信对方的身份
- **数据加密**：支持数据的加密传输，保护信息不被窃取
- **数字签名**：验证数据的完整性和真实性，防篡改
- **证书管理**：包括证书的颁发、吊销、更新等过程

在 Avalanche 网络中，证书管理模块负责：

- 为节点生成唯一的身份证书
- 验证其他节点的证书有效性
- 管理证书的生命周期，包括续期和吊销
- 为 TLS 连接提供安全配置

## 目录结构

- `Cargo.toml`: 模块依赖配置，定义了版本和外部依赖
- `src/`: 实现代码目录
  - `lib.rs`: 模块入口和主要功能，定义公共 API
  - `x509.rs`: X.509 证书处理的具体实现

## 核心功能

### 1. 证书生成

- **支持多种密钥类型**：RSA (2048/4096 位) 和 ECC (P-256/P-384/P-521)
- **自签名证书**：为节点创建自签名证书
- **证书属性配置**：支持配置主题名称、有效期、密钥用途等
- **PEM/DER 格式**：支持多种证书格式

### 2. 证书验证

- **有效期检查**：验证证书是否在有效期内
- **签名验证**：验证证书签名的有效性
- **信任链验证**：验证证书链的完整性和可信度
- **吊销检查**：支持证书吊销列表 (CRL) 检查

### 3. 证书管理

- **证书存储**：安全存储证书和私钥
- **证书续期**：自动检测即将过期的证书并续期
- **证书吊销**：支持吊销不再使用的证书
- **事件通知**：证书状态变更时发送通知

### 4. TLS 配置

- **服务器配置**：为服务器提供 TLS 配置
- **客户端配置**：为客户端提供 TLS 配置
- **双向认证**：支持客户端和服务器双向认证
- **密码套件选择**：配置安全的密码套件

## 设计模式

### 1. 工厂模式

用于创建不同类型的证书和密钥：

```rust
// 工厂方法创建证书
pub fn generate_self_signed_cert(
    subject_alt_names: Option<&str>,
    key_usages: &[KeyUsagePurpose],
    not_before: Option<SystemTime>,
    not_after: Option<SystemTime>,
) -> Result<(Vec<u8>, Vec<u8>), Error> {
    // 实现细节
}
```

### 2. 状态模式

管理证书的不同生命周期状态：

- 有效状态
- 即将过期状态
- 已过期状态
- 已吊销状态

### 3. 观察者模式

通知系统中的其他组件证书状态变更：

```rust
// 注册证书状态变更监听器
pub fn register_listener(&mut self, listener: Box<dyn CertificateListener>) {
    self.listeners.push(listener);
}

// 通知所有监听器
fn notify_listeners(&self, event: CertificateEvent) {
    for listener in &self.listeners {
        listener.on_certificate_event(event);
    }
}
```

### 4. 策略模式

支持不同的证书验证策略：

```rust
// 验证策略接口
pub trait ValidationStrategy {
    fn validate(&self, cert: &Certificate) -> Result<(), ValidationError>;
}

// 具体策略实现
pub struct StandardValidation;
pub struct RelaxedValidation;
```

## 使用示例

### 基本使用

```rust
use cert_manager::{CertificateManager, Config};

// 创建默认配置
let config = Config::default();

// 初始化证书管理器
let mut manager = CertificateManager::new(config);

// 生成新证书
let cert = manager.generate_certificate("node1");

// 验证证书
let is_valid = manager.verify_certificate(&cert);
```

### 自定义配置

```rust
use cert_manager::{CertificateManager, Config, KeyAlgorithm};
use std::time::Duration;

// 创建自定义配置
let config = Config {
    expiration_days: 365,
    key_algorithm: KeyAlgorithm::EcdsaP256,
    auto_renew: true,
    renewal_threshold: Duration::from_days(30),
    cert_path: "/path/to/certs".into(),
};

// 初始化证书管理器
let manager = CertificateManager::new(config);
```

### TLS 配置

```rust
use cert_manager::{CertificateManager, TlsConfig};
use std::path::Path;

// 加载证书和密钥
let (key, cert) = manager.load_certificate(Path::new("node1.key"), Path::new("node1.crt"))?;

// 创建 TLS 配置
let tls_config = manager.create_tls_config(key, cert)?;

// 使用 TLS 配置创建服务器
let server = Server::new(tls_config);
```

## 配置选项

证书管理器支持以下配置选项：

| 选项 | 描述 | 默认值 |
|------|------|--------|
| `expiration_days` | 证书有效期（天） | 365 |
| `key_algorithm` | 密钥算法 (RSA/ECC) | RSA-2048 |
| `auto_renew` | 是否自动续期 | true |
| `renewal_threshold` | 续期阈值（到期前多久续期） | 30 天 |
| `cert_path` | 证书存储路径 | "./certs" |
| `crl_check` | 是否检查证书吊销列表 | false |
| `strict_validation` | 是否使用严格验证 | true |

## API 文档

完整的 API 文档请参考代码文档或使用以下命令生成：

```bash
cargo doc --open
```
