# AvalancheGo 一致性测试服务器

本目录包含 AvalancheGo 一致性测试框架的服务器实现，提供了测试环境和验证功能。

## 目录

1. [文件结构](#文件结构)
2. [设计模式](#设计模式)
3. [核心功能](#核心功能)
4. [使用说明](#使用说明)
5. [扩展指南](#扩展指南)

## 文件结构

- `server.go`: 服务器核心实现，包含服务器配置和启动逻辑
- `key.go`: 密钥相关测试实现
- `message.go`: 消息处理相关测试实现
- `packer.go`: 数据打包相关测试实现

## 设计模式

### 服务器模式

使用 gRPC 服务器模式实现测试服务：

```go
type server struct {
    rpcpb.UnimplementedConformanceServiceServer
    
    cfg Config
    log *zap.Logger
    
    // 其他字段
}

func (s *server) PingService(ctx context.Context, req *rpcpb.PingServiceRequest) (*rpcpb.PingServiceResponse, error) {
    // 实现服务方法
    return &rpcpb.PingServiceResponse{}, nil
}
```

### 配置模式

使用结构体配置服务器行为：

```go
type Config struct {
    LogLevel string
    Port     string
    
    // 其他配置选项
}
```

### 工厂模式

使用工厂方法创建服务器实例：

```go
func New(cfg Config) (*server, error) {
    // 创建和配置服务器
    return &server{
        cfg: cfg,
        log: logger,
    }, nil
}
```

### 生命周期管理模式

管理服务器的生命周期：

```go
func (s *server) Start() error {
    // 启动服务器
}

func (s *server) Stop() {
    // 停止服务器
}
```

## 核心功能

### 服务器管理

提供服务器的创建、启动和停止功能：

- `New(cfg Config)`: 创建服务器实例
- `Start()`: 启动服务器
- `Stop()`: 停止服务器

### 健康检查

实现健康检查服务：

- `PingService`: 验证服务器是否正常运行

### 消息测试

实现消息序列化和反序列化测试：

- `TestMessage`: 验证消息序列化的一致性
- 支持各种 Avalanche 网络消息类型

### 密钥测试

实现密钥生成和签名测试：

- `TestKey`: 验证密钥操作的一致性
- 支持不同类型的密钥和签名算法

### 数据打包测试

实现数据打包和解包测试：

- `TestPacker`: 验证数据打包的一致性
- 支持不同的打包格式和选项

## 使用说明

### 创建和启动服务器

```go
import (
    "github.com/ava-labs/avalanche-rs/avalanchego-conformance/server"
)

func main() {
    // 创建服务器配置
    cfg := server.Config{
        LogLevel: "info",
        Port:     "8080",
    }
    
    // 创建服务器
    srv, err := server.New(cfg)
    if err != nil {
        panic(err)
    }
    
    // 启动服务器
    if err := srv.Start(); err != nil {
        panic(err)
    }
    
    // 等待信号
    sigc := make(chan os.Signal, 1)
    signal.Notify(sigc, syscall.SIGINT, syscall.SIGTERM)
    <-sigc
    
    // 停止服务器
    srv.Stop()
}
```

### 命令行启动

使用命令行工具启动服务器：

```bash
go run cmd/avalanchego-conformance/main.go server --port=8080 --log-level=info
```

## 扩展指南

### 添加新测试

要添加新测试，请按照以下步骤操作：

1. 在 Protocol Buffers 定义中添加新服务方法
2. 生成 gRPC 代码
3. 在服务器实现中添加新方法：

```go
func (s *server) TestNewFeature(ctx context.Context, req *rpcpb.TestNewFeatureRequest) (*rpcpb.TestNewFeatureResponse, error) {
    s.log.Info("收到 TestNewFeature 请求", zap.String("feature", req.FeatureName))
    
    // 实现测试逻辑
    
    return &rpcpb.TestNewFeatureResponse{
        Success: true,
        ResultData: resultBytes,
    }, nil
}
```

4. 添加测试用例和验证逻辑

### 添加新配置选项

要添加新配置选项，请按照以下步骤操作：

1. 更新 `Config` 结构体：

```go
type Config struct {
    LogLevel string
    Port     string
    
    // 添加新选项
    EnableTLS bool
    CertFile  string
    KeyFile   string
}
```

2. 在 `New` 函数中使用新选项：

```go
func New(cfg Config) (*server, error) {
    // ...
    
    // 使用新选项
    if cfg.EnableTLS {
        // 配置 TLS
    }
    
    // ...
}
```

3. 在命令行工具中添加新选项：

```go
cmd.PersistentFlags().BoolVar(&enableTLS, "tls", false, "启用 TLS")
cmd.PersistentFlags().StringVar(&certFile, "cert-file", "", "TLS 证书文件路径")
cmd.PersistentFlags().StringVar(&keyFile, "key-file", "", "TLS 密钥文件路径")
```

### 添加新依赖

要添加新依赖，请按照以下步骤操作：

1. 更新 `go.mod` 文件：

```bash
go get github.com/example/package
```

2. 在代码中导入新包：

```go
import (
    "github.com/example/package"
)
```

3. 使用新包的功能：

```go
result := package.Function()
```
