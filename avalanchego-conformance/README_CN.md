# AvalancheGo 一致性测试框架

本目录包含 AvalancheGo 的一致性测试框架，用于验证不同 Avalanche 实现之间的互操作性。

## 目录结构说明

### 核心配置文件
- `buf.gen.yaml`, `buf.lock`, `buf.yaml`: Buf 构建工具的配置文件，用于 gRPC 代码生成
- `go.mod`, `go.sum`: Go 模块依赖管理文件
- `LICENSE`: 项目许可证文件
- `README.md`: 英文版说明文档

### 主要代码目录

#### `client/`
客户端实现，包含与服务器交互的代码

#### `cmd/`
命令行工具入口:
- `conformance-client`: 一致性测试客户端
- `conformance-server`: 一致性测试服务器

#### `pkg/`
核心功能包:
- `conformance`: 一致性测试核心逻辑
- `utils`: 工具函数

#### `rpcpb/`
gRPC 协议定义:
- 包含所有 Protobuf 定义文件
- 自动生成的 gRPC 服务代码

#### `scripts/`
构建和测试脚本

#### `server/`
服务器端实现:
- 核心服务逻辑
- 测试用例实现
- 结果验证

## 设计模式

1. **gRPC 服务架构**: 使用 Protobuf 定义接口，确保跨语言兼容性
2. **模块化设计**: 功能按目录清晰划分
3. **一致性测试框架**: 通过定义标准接口验证不同实现的兼容性
4. **客户端-服务器模式**: 分离测试执行和验证逻辑

## 使用说明

1. 运行 `conformance-server` 启动测试服务器
2. 使用 `conformance-client` 连接并执行测试
3. 查看测试结果验证实现一致性

如需更详细信息，请参考各子目录中的文档。
