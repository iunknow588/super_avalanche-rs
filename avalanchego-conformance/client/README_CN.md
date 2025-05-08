# AvalancheGo 一致性测试客户端

本目录包含 AvalancheGo 一致性测试框架的客户端实现，用于与一致性测试服务器通信并执行测试。

## 目录

1. [文件结构](#文件结构)
2. [设计模式](#设计模式)
3. [核心功能](#核心功能)
4. [使用示例](#使用示例)
5. [扩展指南](#扩展指南)

## 文件结构

- `client.go`: 客户端接口和实现，提供与一致性测试服务器通信的功能

## 设计模式

### 接口抽象

客户端代码使用接口抽象设计模式，通过 `Client` 接口定义客户端行为：

```go
type Client interface {
    PingService(ctx context.Context) (*rpcpb.PingServiceResponse, error)
    Close() error
}
```

这种设计允许多种客户端实现，同时保持一致的 API。

### 工厂模式

使用工厂模式创建客户端实例：

```go
func New(cfg Config) (Client, error) {
    // 创建和配置客户端实例
    return &client{...}, nil
}
```

这种方式封装了客户端创建的复杂性，提供了简洁的 API。

### 资源管理

客户端实现了资源管理模式，确保连接正确关闭：

```go
func (c *client) Close() error {
    c.closeOnce.Do(func() {
        close(c.closed)
    })
    return c.conn.Close()
}
```

使用 `sync.Once` 确保资源只被释放一次。

## 核心功能

### 配置管理

客户端支持通过 `Config` 结构体进行配置：

```go
type Config struct {
    LogLevel    string
    Endpoint    string
    DialTimeout time.Duration
}
```

### 服务连接

客户端使用 gRPC 与服务器建立连接：

```go
conn, err := grpc.DialContext(
    ctx,
    cfg.Endpoint,
    grpc.WithBlock(),
    grpc.WithTransportCredentials(insecure.NewCredentials()),
)
```

### 服务调用

客户端提供方法调用服务器端 API：

```go
func (c *client) PingService(ctx context.Context) (*rpcpb.PingServiceResponse, error) {
    return c.pingc.PingService(ctx, &rpcpb.PingServiceRequest{})
}
```

## 使用示例

### 创建客户端

```go
client, err := client.New(client.Config{
    LogLevel:    "info",
    Endpoint:    "localhost:8080",
    DialTimeout: 5 * time.Second,
})
if err != nil {
    log.Fatalf("Failed to create client: %v", err)
}
defer client.Close()
```

### 调用服务

```go
ctx, cancel := context.WithTimeout(context.Background(), 3*time.Second)
defer cancel()

resp, err := client.PingService(ctx)
if err != nil {
    log.Fatalf("Failed to ping service: %v", err)
}
fmt.Printf("Ping response: %+v\n", resp)
```

## 扩展指南

### 添加新服务

1. 在 Protocol Buffers 定义中添加新服务
2. 生成 gRPC 代码
3. 在 `client` 结构体中添加新的服务客户端字段
4. 在 `New` 函数中初始化新的服务客户端
5. 在 `Client` 接口中添加新方法
6. 实现新方法

### 添加新功能

要添加新的客户端功能，请按照以下步骤操作：

1. 在 `Client` 接口中定义新方法
2. 在 `client` 结构体中实现该方法
3. 更新文档和测试

### 自定义配置

可以扩展 `Config` 结构体以支持更多配置选项：

```go
type Config struct {
    LogLevel    string
    Endpoint    string
    DialTimeout time.Duration
    // 添加新的配置选项
    TLS         bool
    CertFile    string
    KeyFile     string
}
```
