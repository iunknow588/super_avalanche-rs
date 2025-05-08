# AvalancheGo 一致性测试工具包

本目录包含 AvalancheGo 一致性测试框架的工具包，提供了各种辅助功能和实用工具。

## 目录

1. [目录结构](#目录结构)
2. [设计模式](#设计模式)
3. [核心功能](#核心功能)
4. [使用示例](#使用示例)

## 目录结构

- `color/`: 终端彩色输出工具
  - `color.go`: 定义颜色常量和格式化函数
- `logutil/`: 日志工具函数
  - `logutil.go`: 提供日志配置和格式化功能

## 设计模式

### 工具类模式

工具包使用工具类模式，提供静态方法和常量：

```go
// color 包
const (
    Red    = "\033[31m"
    Green  = "\033[32m"
    Yellow = "\033[33m"
    Blue   = "\033[34m"
    Purple = "\033[35m"
    Cyan   = "\033[36m"
    Gray   = "\033[37m"
    White  = "\033[97m"
)

func Red(s string) string {
    return Red + s + ResetColor
}
```

### 单例模式

日志工具使用单例模式，确保全局日志配置一致：

```go
var (
    globalLogger *zap.Logger
    globalMu     sync.Mutex
)

func GetLogger() *zap.Logger {
    globalMu.Lock()
    defer globalMu.Unlock()
    
    if globalLogger == nil {
        // 初始化日志
    }
    
    return globalLogger
}
```

### 工厂模式

日志工具使用工厂方法创建日志实例：

```go
func NewLogger(logLevel string) (*zap.Logger, error) {
    // 创建和配置日志实例
    return logger, nil
}
```

## 核心功能

### 终端彩色输出

`color` 包提供终端彩色输出功能：

- 定义颜色常量（红色、绿色、黄色等）
- 提供格式化函数（`Red()`、`Green()`、`Yellow()` 等）
- 支持彩色输出的启用和禁用

### 日志工具

`logutil` 包提供日志配置和格式化功能：

- 配置日志级别（debug、info、warn、error）
- 配置日志格式（JSON、控制台）
- 提供全局日志实例
- 支持结构化日志

## 使用示例

### 终端彩色输出

```go
import "github.com/ava-labs/avalanche-rs/avalanchego-conformance/pkg/color"

func main() {
    fmt.Println(color.Red("错误信息"))
    fmt.Println(color.Green("成功信息"))
    fmt.Println(color.Yellow("警告信息"))
}
```

### 日志工具

```go
import "github.com/ava-labs/avalanche-rs/avalanchego-conformance/pkg/logutil"

func main() {
    // 创建日志实例
    logger, err := logutil.NewLogger("info")
    if err != nil {
        panic(err)
    }
    
    // 使用日志
    logger.Info("信息日志", zap.String("key", "value"))
    logger.Error("错误日志", zap.Error(err))
    
    // 使用全局日志
    logutil.GetLogger().Info("使用全局日志")
}
```

## 扩展指南

### 添加新工具包

要添加新的工具包，请按照以下步骤操作：

1. 创建新的子目录（如 `pkg/timeutil/`）
2. 实现工具函数和类型
3. 编写测试和文档

### 添加新功能

要向现有工具包添加新功能，请按照以下步骤操作：

1. 在相应的文件中添加新函数或常量
2. 更新文档和测试
3. 确保向后兼容性
