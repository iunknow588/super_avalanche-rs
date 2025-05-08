# AvalancheGo 一致性测试命令行工具

本目录包含 AvalancheGo 一致性测试框架的命令行工具实现，提供了启动测试服务器和执行测试的功能。

## 目录

1. [目录结构](#目录结构)
2. [设计模式](#设计模式)
3. [核心功能](#核心功能)
4. [使用说明](#使用说明)

## 目录结构

- `avalanchego-conformance/`: 主命令行工具目录
  - `main.go`: 程序入口点，定义命令行界面
  - `server/`: 服务器命令实现
    - `server.go`: 服务器命令定义和实现

## 设计模式

### 命令模式

命令行工具使用命令模式设计，通过 Cobra 库实现命令的定义和执行：

```go
var rootCmd = &cobra.Command{
    Use:        "avalanchego-conformance",
    Short:      "avalanchego-conformance commands",
    SuggestFor: []string{"avalanche-conformance"},
}
```

每个子命令（如 `server`）都是一个独立的命令对象，可以单独配置和执行。

### 组合模式

命令行工具使用组合模式组织命令结构，根命令包含多个子命令：

```go
func init() {
    rootCmd.AddCommand(
        server.NewCommand(),
    )
}
```

这种设计允许灵活地添加、移除和组织命令。

### 工厂模式

子命令使用工厂方法创建：

```go
func NewCommand() *cobra.Command {
    // 创建和配置命令
    return cmd
}
```

这种方式封装了命令创建的复杂性，提供了清晰的 API。

## 核心功能

### 命令行解析

使用 Cobra 库解析命令行参数和选项：

```go
func main() {
    if err := rootCmd.Execute(); err != nil {
        fmt.Fprintf(os.Stderr, "avalanchego-conformance failed %v\n", err)
        os.Exit(1)
    }
    os.Exit(0)
}
```

### 服务器命令

`server` 命令用于启动一致性测试服务器：

- 解析命令行参数（如端口、日志级别等）
- 配置和启动 gRPC 服务器
- 处理信号和优雅关闭

## 使用说明

### 启动服务器

```bash
# 使用默认配置启动服务器
go run cmd/avalanchego-conformance/main.go server

# 指定端口
go run cmd/avalanchego-conformance/main.go server --port=8080

# 设置日志级别
go run cmd/avalanchego-conformance/main.go server --log-level=debug
```

### 查看帮助

```bash
# 查看根命令帮助
go run cmd/avalanchego-conformance/main.go --help

# 查看服务器命令帮助
go run cmd/avalanchego-conformance/main.go server --help
```

## 扩展指南

### 添加新命令

要添加新命令，请按照以下步骤操作：

1. 创建新的命令包（如 `cmd/avalanchego-conformance/client/`）
2. 实现 `NewCommand()` 函数，返回配置好的 `cobra.Command`
3. 在 `main.go` 中导入新包并添加到根命令

```go
import (
    "github.com/ava-labs/avalanche-rs/avalanchego-conformance/cmd/avalanchego-conformance/client"
)

func init() {
    rootCmd.AddCommand(
        server.NewCommand(),
        client.NewCommand(),
    )
}
```
