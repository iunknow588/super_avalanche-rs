# Avalanche 密钥管理模块

本目录包含 Avalanche 区块链生态系统中密钥管理相关的实现，支持多种密码学算法，主要包括 BLS 和 secp256k1 两种密钥类型。

## 目录

1. [模块概述](#模块概述)
2. [设计模式](#设计模式)
3. [BLS 密钥模块](#bls-密钥模块)
4. [Secp256k1 密钥模块](#secp256k1-密钥模块)
5. [使用示例](#使用示例)

## 模块概述

密钥管理模块是 Avalanche 区块链的核心组件之一，负责处理以下功能：

- 密钥生成、存储和加载
- 数字签名的创建和验证
- 地址生成和格式化
- 与外部密钥管理系统（如 AWS KMS）的集成
- 支持交易签名和验证

该模块支持两种主要的密码学算法：

1. **BLS (Boneh-Lynn-Shacham)**: 用于聚合签名和验证，主要用于共识机制
2. **secp256k1**: 兼容以太坊的椭圆曲线算法，用于常规交易签名和地址生成

## 设计模式

### 接口抽象

模块采用 Rust 的 trait 系统实现接口抽象，主要包括：

- `SignOnly`: 限制只能执行签名操作的接口
- `ReadOnly`: 限制只能执行读取操作的接口

这种设计允许安全地集成外部密钥管理服务，同时保持 API 的一致性。

### 工厂模式

通过静态方法和构造函数提供密钥的创建方式：

- 从文件加载
- 从字节数组创建
- 随机生成
- 从助记词恢复

### 组合模式

将复杂的密钥操作分解为更小的组件：

- 私钥/公钥分离
- 签名与验证分离
- 地址生成与格式化分离

### 适配器模式

提供与不同格式和标准的兼容性：

- CB58 编码支持（Avalanche 原生格式）
- 十六进制编码支持（以太坊兼容）
- 助记词支持（BIP39 标准）

## BLS 密钥模块

BLS 密钥模块实现了 Boneh-Lynn-Shacham 签名算法，主要用于 Avalanche 共识机制中的聚合签名。

### 文件结构

- `bls/mod.rs`: 模块入口，定义了 `ProofOfPossession` 结构体
- `bls/private_key.rs`: BLS 私钥实现
- `bls/public_key.rs`: BLS 公钥实现
- `bls/signature.rs`: BLS 签名实现

### 主要功能

#### ProofOfPossession

`ProofOfPossession` 结构体用于验证节点身份，确保节点确实拥有对应的私钥。这是 Avalanche 网络中节点身份验证的关键组件。

#### 私钥 (private_key.rs)

- 生成新的 BLS 私钥
- 从字节数组加载私钥
- 签名消息
- 生成证明签名（Proof of Possession）
- 转换为公钥

#### 公钥 (public_key.rs)

- 从字节数组加载公钥
- 验证签名
- 验证证明签名
- 序列化和反序列化

#### 签名 (signature.rs)

- 从字节数组加载签名
- 验证签名
- 聚合多个签名
- 序列化和反序列化

## Secp256k1 密钥模块

Secp256k1 模块实现了与以太坊兼容的椭圆曲线数字签名算法，用于常规交易签名和地址生成。

### 文件结构

- `secp256k1/mod.rs`: 模块入口，定义了核心接口和类型
- `secp256k1/private_key.rs`: 私钥实现
- `secp256k1/public_key.rs`: 公钥实现
- `secp256k1/signature.rs`: 签名实现
- `secp256k1/address.rs`: 地址格式化和验证
- `secp256k1/keychain.rs`: 密钥链管理
- `secp256k1/mnemonic.rs`: 助记词支持（BIP39）
- `secp256k1/libsecp256k1.rs`: libsecp256k1 库的集成
- `secp256k1/kms/`: 外部密钥管理系统集成
- `secp256k1/txs/`: 交易签名支持

### 主要功能

#### 核心接口 (mod.rs)

- `SignOnly`: 限制只能执行签名操作的接口
- `ReadOnly`: 限制只能执行读取操作的接口
- `KeyType`: 定义密钥类型（热钱包、AWS KMS 等）
- `Info`: 密钥信息结构体，包含地址、类型等元数据

#### 私钥 (private_key.rs)

- 生成新的 secp256k1 私钥
- 从 CB58/十六进制字符串加载私钥
- 签名消息和交易
- 转换为公钥
- 导出为不同格式（CB58、十六进制）

#### 公钥 (public_key.rs)

- 从字节数组加载公钥
- 验证签名
- 生成 Avalanche 地址（X-Chain、P-Chain）
- 生成以太坊地址（C-Chain）
- 生成短地址（Short ID）

#### 地址 (address.rs)

- 地址格式化和解析
- 地址验证
- 不同链（X、P、C）的地址转换

#### 密钥链 (keychain.rs)

- 管理多个密钥
- 批量签名
- 密钥派生

#### KMS 集成 (kms/)

- AWS KMS 集成
- 远程签名支持
- 以太坊交易签名

#### 交易支持 (txs/)

- 转账交易签名
- 交易构建辅助函数

## 使用示例

### BLS 密钥生成和签名

```rust
// 生成 BLS 私钥
let private_key = bls::private_key::Key::generate().unwrap();

// 获取对应的公钥
let public_key = private_key.to_public_key();

// 签名消息
let message = b"Hello, Avalanche!";
let signature = private_key.sign(message);

// 验证签名
assert!(signature.verify(message, &public_key));

// 生成证明签名
let proof = private_key.to_proof_of_possession();
assert!(proof.verify().unwrap());
```

### Secp256k1 密钥生成和地址

```rust
// 生成 secp256k1 私钥
let private_key = secp256k1::private_key::Key::generate().unwrap();

// 获取对应的公钥
let public_key = private_key.to_public_key();

// 获取 X-Chain 地址
let x_address = public_key.hrp_address(1, "X").unwrap();

// 获取以太坊地址
let eth_address = public_key.eth_address();

// 签名消息
let message = b"Hello, Avalanche!";
let digest = sha2::Sha256::digest(message);
let signature = private_key.sign_digest(&digest).await.unwrap();

// 验证签名
let is_valid = public_key.verify_signature(&digest, &signature).unwrap();
assert!(is_valid);
```
