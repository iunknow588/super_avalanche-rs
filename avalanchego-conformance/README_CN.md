# AvalancheGo 一致性测试框架

本目录包含 AvalancheGo 的一致性测试框架，用于验证不同 Avalanche 实现（如 Rust 和 Go）之间的互操作性和行为一致性。该框架通过定义标准化的测试接口和验证流程，确保不同语言实现的 Avalanche 组件能够无缝协作。

## 目录

1. [目录结构](#目录结构)
2. [设计模式](#设计模式)
3. [核心功能](#核心功能)
4. [使用说明](#使用说明)
5. [开发指南](#开发指南)

## 目录结构

### 核心配置文件
- `buf.gen.yaml`, `buf.lock`, `buf.yaml`: Buf 构建工具的配置文件，用于 Protocol Buffers 和 gRPC 代码生成
- `go.mod`, `go.sum`: Go 模块依赖管理文件
- `LICENSE`: 项目许可证文件
- `README.md`: 英文版说明文档

### 主要代码目录

#### `client/`
客户端实现，包含与服务器交互的代码：
- `client.go`: 定义客户端接口和实现，提供与一致性测试服务器通信的功能

#### `cmd/`
命令行工具入口：
- `avalanchego-conformance/`: 主命令行工具
  - `main.go`: 程序入口点
  - `server/`: 服务器命令实现

#### `pkg/`
核心功能包：
- `color/`: 终端彩色输出工具
- `logutil/`: 日志工具函数

#### `rpcpb/`
gRPC 协议定义：
- `*.proto`: Protocol Buffers 定义文件
- `*.pb.go`: 自动生成的 Go 代码
- `*_grpc.pb.go`: 自动生成的 gRPC 服务代码

#### `scripts/`
构建和测试脚本：
- `genproto.sh`: 生成 Protocol Buffers 代码
- `updatedep.sh`: 更新依赖

#### `server/`
服务器端实现：
- `server.go`: 服务器核心实现
- `key.go`: 密钥相关测试
- `message.go`: 消息处理相关测试
- `packer.go`: 数据打包相关测试

## 设计模式

### gRPC 服务架构
- 使用 Protocol Buffers 定义接口，确保跨语言兼容性
- 服务器和客户端通过 gRPC 进行通信
- 支持双向流和异步通信

### 模块化设计
- 功能按目录清晰划分
- 每个组件负责特定功能
- 松耦合设计便于扩展和维护

### 一致性测试框架
- 定义标准化的测试接口
- 提供参考实现作为基准
- 验证不同实现的行为一致性
- 支持多种测试场景和边缘情况

### 客户端-服务器模式
- 分离测试执行和验证逻辑
- 服务器提供测试环境和验证功能
- 客户端实现被测试的功能
- 支持多种语言实现的客户端

## 核心功能

### 消息序列化一致性测试
验证不同实现对 Avalanche 网络消息的序列化和反序列化是否一致。

### 密钥和签名一致性测试
验证不同实现的密钥生成、签名和验证功能是否兼容。

### 数据打包一致性测试
验证不同实现的数据打包和解包功能是否一致。

### 网络协议一致性测试
验证不同实现对 Avalanche 网络协议的处理是否兼容。

## 使用说明

### 启动服务器
```bash
go run cmd/avalanchego-conformance/main.go server --port=8080
```

### 运行测试
使用 Rust 客户端运行测试：
```bash
cargo test --package avalanchego-conformance -- --nocapture
```

### 查看结果
测试结果将显示在控制台输出中，包括通过和失败的测试用例。

## 开发指南

### 添加新测试
1. 在 `rpcpb/` 目录中定义新的 Protocol Buffers 消息和服务
2. 运行 `scripts/genproto.sh` 生成代码
3. 在 `server/` 目录中实现服务器端测试逻辑
4. 在客户端实现中添加相应的测试用例

### 支持新语言
1. 使用目标语言的 Protocol Buffers 和 gRPC 工具生成客户端代码
2. 实现客户端接口和测试用例
3. 与服务器进行通信并验证结果

如需更详细信息，请参考各子目录中的文档。
