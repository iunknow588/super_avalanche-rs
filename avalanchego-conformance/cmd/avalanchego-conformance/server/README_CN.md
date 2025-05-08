# AvalancheGo 一致性测试服务器命令

本目录包含 AvalancheGo 一致性测试框架的服务器命令实现，用于启动和配置测试服务器。

## 目录

1. [文件结构](#文件结构)
2. [设计模式](#设计模式)
3. [核心功能](#核心功能)
4. [使用说明](#使用说明)

## 文件结构

- `server.go`: 服务器命令定义和实现，包含命令行参数解析和服务器启动逻辑

## 设计模式

### 命令模式

服务器命令使用命令模式设计，通过 Cobra 库实现：

```go
func NewCommand() *cobra.Command {
    cmd := &cobra.Command{
        Use:   "server",
        Short: "Start an avalanchego-conformance server",
        RunE:  serverFunc,
    }
    // ...
    return cmd
}
```

### 工厂模式

使用工厂方法创建服务器命令，封装命令的创建和配置：

```go
func NewCommand() *cobra.Command {
    // 创建和配置命令
    return cmd
}
```

### 选项模式

使用命令行标志作为选项，配置服务器行为：

```go
func NewCommand() *cobra.Command {
    cmd := &cobra.Command{
        // ...
    }
    
    cmd.PersistentFlags().StringVar(&logLevel, "log-level", "info", "设置日志级别")
    cmd.PersistentFlags().StringVar(&port, "port", "8080", "设置服务器端口")
    
    return cmd
}
```

## 核心功能

### 命令行参数解析

解析命令行参数，配置服务器：

- `--log-level`: 设置日志级别（默认为 "info"）
- `--port`: 设置服务器端口（默认为 "8080"）

### 服务器启动

启动 gRPC 服务器，处理客户端请求：

```go
func serverFunc(cmd *cobra.Command, args []string) error {
    // 配置日志
    // 创建服务器
    // 启动服务器
    // 处理信号
    return nil
}
```

### 信号处理

处理操作系统信号，实现优雅关闭：

```go
sigc := make(chan os.Signal, 1)
signal.Notify(sigc, syscall.SIGINT, syscall.SIGTERM)
<-sigc
```

## 使用说明

### 基本用法

```bash
# 使用默认配置启动服务器
go run ../main.go server

# 指定端口
go run ../main.go server --port=9090

# 设置日志级别
go run ../main.go server --log-level=debug
```

### 查看帮助

```bash
go run ../main.go server --help
```

## 扩展指南

### 添加新参数

要添加新的命令行参数，请在 `NewCommand()` 函数中定义：

```go
func NewCommand() *cobra.Command {
    cmd := &cobra.Command{
        // ...
    }
    
    // 添加现有参数
    cmd.PersistentFlags().StringVar(&logLevel, "log-level", "info", "设置日志级别")
    cmd.PersistentFlags().StringVar(&port, "port", "8080", "设置服务器端口")
    
    // 添加新参数
    cmd.PersistentFlags().BoolVar(&enableTLS, "tls", false, "启用 TLS")
    cmd.PersistentFlags().StringVar(&certFile, "cert-file", "", "TLS 证书文件路径")
    cmd.PersistentFlags().StringVar(&keyFile, "key-file", "", "TLS 密钥文件路径")
    
    return cmd
}
```

### 修改服务器行为

要修改服务器行为，请更新 `serverFunc` 函数：

```go
func serverFunc(cmd *cobra.Command, args []string) error {
    // 配置日志
    
    // 创建服务器配置
    cfg := server.Config{
        LogLevel: logLevel,
        Port:     port,
        // 添加新配置
        EnableTLS: enableTLS,
        CertFile:  certFile,
        KeyFile:   keyFile,
    }
    
    // 创建并启动服务器
    srv, err := server.New(cfg)
    if err != nil {
        return err
    }
    
    // 处理信号和关闭
    return nil
}
```
