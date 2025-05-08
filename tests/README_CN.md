# Avalanche 测试套件

本目录包含 Avalanche 区块链的各种测试套件，用于验证系统的正确性、一致性和抗拜占庭故障能力。这些测试套件是确保 Avalanche 网络稳定性和安全性的关键组件。

## 目录

1. [测试套件概述](#测试套件概述)
2. [设计模式](#设计模式)
3. [测试类型](#测试类型)
   - [端到端测试 (E2E)](#端到端测试-e2e)
   - [拜占庭测试](#拜占庭测试)
   - [一致性测试](#一致性测试)
4. [测试工具](#测试工具)
5. [运行测试](#运行测试)
6. [编写新测试](#编写新测试)

## 测试套件概述

Avalanche 测试套件由三个主要组件组成：

1. **端到端测试 (E2E)**：验证整个系统的功能正确性，模拟真实用户场景
2. **拜占庭测试**：验证系统在存在恶意节点的情况下的安全性和鲁棒性
3. **一致性测试**：验证 Rust 实现与 Go 实现 (AvalancheGo) 的行为一致性

这些测试套件共同确保 Avalanche 网络的可靠性、安全性和互操作性。

## 设计模式

### 测试驱动开发 (TDD)

测试套件采用测试驱动开发方法，先编写测试用例，再实现功能：

- 定义预期行为
- 编写测试用例
- 实现功能代码
- 验证测试通过

### 模拟与存根

使用模拟和存根技术隔离测试环境：

- 网络模拟：使用 `network-runner` 创建本地测试网络
- 节点模拟：模拟不同类型的节点行为
- API 存根：提供可预测的 API 响应

### 参数化测试

通过配置文件和命令行参数实现测试参数化：

- 可配置的网络拓扑
- 可调整的节点数量
- 可定制的测试场景

### 断言与验证

使用多层次断言验证系统行为：

- 状态断言：验证系统状态
- 行为断言：验证系统行为
- 性能断言：验证系统性能

## 测试类型

### 端到端测试 (E2E)

位于 `avalanche-e2e` 目录，实现了完整的端到端测试框架，用于验证整个系统的功能正确性。

#### 主要组件

- `main.rs`：测试入口点，处理命令行参数和测试配置
- `command.rs`：实现测试命令执行逻辑
- `default_spec.rs`：提供默认测试规范
- `spec.rs`：定义测试规范结构
- `c.rs`、`p.rs`、`x.rs`：分别实现 C-Chain、P-Chain 和 X-Chain 的测试用例

#### 功能特点

- 支持自定义测试规范
- 支持并行测试执行
- 支持随机化测试顺序
- 支持错误忽略和继续测试

#### 使用方式

```bash
# 生成默认测试规范
./target/release/avalanche-e2e \
  --spec-path /tmp/tests.avalanchego-e2e.yaml \
  default-spec \
  --keys-to-generate 5 \
  --network-runner-grpc-endpoint http://127.0.0.1:12342

# 执行测试
./target/release/avalanche-e2e \
  --skip-prompt \
  --spec-path /tmp/tests.avalanchego-e2e.yaml
```

### 拜占庭测试

位于 `avalanchego-byzantine` 目录，实现了拜占庭容错测试，用于验证系统在存在恶意节点的情况下的安全性和鲁棒性。

#### 主要组件

- `src/tests/mod.rs`：测试入口点
- `src/tests/byzantine.rs`：实现拜占庭测试用例

#### 测试场景

- **冲突交易**：测试系统处理冲突交易的能力
- **恶意消息**：测试系统处理恶意格式化消息的能力
- **网络分区**：测试系统在网络分区情况下的行为

#### 实现细节

- 使用 `network-runner` 创建本地测试网络
- 创建并发送冲突交易
- 绕过共识引擎直接向节点发送消息
- 验证系统正确处理恶意行为

### 一致性测试

位于 `avalanchego-conformance` 目录，实现了一致性测试，用于验证 Rust 实现与 Go 实现 (AvalancheGo) 的行为一致性。

#### 主要组件

- `src/tests/mod.rs`：测试入口点
- `src/tests/version.rs`：实现版本消息一致性测试

#### 测试场景

- **消息序列化**：测试消息序列化的一致性
- **消息处理**：测试消息处理的一致性
- **API 行为**：测试 API 行为的一致性

#### 实现细节

- 使用 `avalanchego-conformance-sdk` 与 AvalancheGo 通信
- 创建相同的消息并比较处理结果
- 验证两种实现的行为一致性

## 测试工具

### network-runner

`network-runner` 是一个用于创建和管理本地 Avalanche 测试网络的工具，提供以下功能：

- 创建自定义网络拓扑
- 启动和停止节点
- 监控节点状态
- 注入网络故障

### avalanchego-conformance-sdk

`avalanchego-conformance-sdk` 是一个用于与 AvalancheGo 通信的 SDK，提供以下功能：

- 发送和接收消息
- 验证消息处理
- 比较行为一致性

## 运行测试

### 环境准备

1. 安装 Rust 工具链
2. 安装 Go 1.19+ (用于 AvalancheGo)
3. 安装 network-runner

### 运行端到端测试

```bash
# 构建测试
./scripts/build.release.sh

# 运行端到端测试
./target/release/avalanche-e2e \
  --spec-path /tmp/tests.avalanchego-e2e.yaml \
  default-spec \
  --keys-to-generate 5 \
  --network-runner-grpc-endpoint http://127.0.0.1:12342

./target/release/avalanche-e2e \
  --skip-prompt \
  --spec-path /tmp/tests.avalanchego-e2e.yaml
```

### 运行拜占庭测试

```bash
# 启动 network-runner
network-runner server \
  --port=12342 \
  --grpc-gateway-port=12343

# 运行拜占庭测试
NETWORK_RUNNER_GRPC_ENDPOINT=http://127.0.0.1:12342 \
  cargo test --package avalanchego-byzantine -- --nocapture
```

### 运行一致性测试

```bash
# 启动 AvalancheGo 一致性测试服务器
avalanchego-conformance server --port=12345

# 运行一致性测试
AVALANCHEGO_CONFORMANCE_ENDPOINT=http://127.0.0.1:12345 \
  cargo test --package avalanchego-conformance -- --nocapture
```

## 编写新测试

### 端到端测试

1. 在 `avalanche-e2e/src` 目录中创建新的测试模块
2. 在 `spec.rs` 中定义测试规范
3. 在 `command.rs` 中添加测试执行逻辑

### 拜占庭测试

1. 在 `avalanchego-byzantine/src/tests` 目录中创建新的测试文件
2. 实现测试用例，模拟恶意行为
3. 在 `mod.rs` 中注册测试用例

### 一致性测试

1. 在 `avalanchego-conformance/src/tests` 目录中创建新的测试文件
2. 实现测试用例，比较 Rust 和 Go 实现的行为
3. 在 `mod.rs` 中注册测试用例
