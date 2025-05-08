# AvalancheGo 一致性测试命令行工具

本目录包含 AvalancheGo 一致性测试框架的主命令行工具实现，是测试框架的入口点。

## 目录

1. [文件结构](#文件结构)
2. [设计模式](#设计模式)
3. [核心功能](#核心功能)
4. [使用说明](#使用说明)

## 文件结构

- `main.go`: 程序入口点，定义根命令和初始化逻辑
- `server/`: 服务器命令实现
  - `server.go`: 服务器命令定义和实现

## 设计模式

### 命令模式

主程序使用命令模式设计，通过 Cobra 库实现命令的定义和执行：

```go
var rootCmd = &cobra.Command{
    Use:        "avalanchego-conformance",
    Short:      "avalanchego-conformance commands",
    SuggestFor: []string{"avalanche-conformance"},
}
```

### 前缀匹配

启用命令前缀匹配，提高用户体验：

```go
func init() {
    cobra.EnablePrefixMatching = true
}
```

这允许用户输入命令的部分前缀而不是完整命令。

### 组合模式

使用组合模式组织命令结构，根命令包含多个子命令：

```go
func init() {
    rootCmd.AddCommand(
        server.NewCommand(),
    )
}
```

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

### 错误处理

主程序实现了统一的错误处理机制：

- 捕获命令执行过程中的错误
- 将错误信息输出到标准错误
- 设置适当的退出码

## 使用说明

### 基本用法

```bash
# 查看帮助
go run main.go --help

# 运行服务器命令
go run main.go server
```

### 构建和运行

```bash
# 构建可执行文件
go build -o avalanchego-conformance

# 运行可执行文件
./avalanchego-conformance server
```

## 扩展指南

### 添加新命令

要添加新命令，请按照以下步骤操作：

1. 创建新的命令包（如 `client/`）
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

### 添加全局标志

要添加全局标志，请在根命令上定义：

```go
func init() {
    rootCmd.PersistentFlags().StringVar(&globalLogLevel, "log-level", "info", "设置日志级别")
}
```
