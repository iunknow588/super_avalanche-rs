# 终端彩色输出工具

本目录包含终端彩色输出工具，用于在命令行界面中显示彩色文本，提高用户体验和可读性。

## 目录

1. [文件结构](#文件结构)
2. [设计模式](#设计模式)
3. [核心功能](#核心功能)
4. [使用示例](#使用示例)
5. [扩展指南](#扩展指南)

## 文件结构

- `color.go`: 定义颜色常量和格式化函数

## 设计模式

### 常量模式

使用常量定义各种颜色代码：

```go
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
```

### 工具函数模式

提供简单的工具函数，将文本包装在颜色代码中：

```go
func Red(s string) string {
    return Red + s + ResetColor
}

func Green(s string) string {
    return Green + s + ResetColor
}
```

### 全局配置模式

支持全局启用或禁用彩色输出：

```go
var (
    // 是否启用彩色输出
    colorEnabled = true
)

// 设置是否启用彩色输出
func SetColorEnabled(enabled bool) {
    colorEnabled = enabled
}
```

## 核心功能

### 颜色常量

定义了多种颜色的 ANSI 转义序列：

- `Red`: 红色
- `Green`: 绿色
- `Yellow`: 黄色
- `Blue`: 蓝色
- `Purple`: 紫色
- `Cyan`: 青色
- `Gray`: 灰色
- `White`: 白色

### 格式化函数

提供将文本包装在颜色代码中的函数：

- `Red(s string)`: 返回红色文本
- `Green(s string)`: 返回绿色文本
- `Yellow(s string)`: 返回黄色文本
- `Blue(s string)`: 返回蓝色文本
- `Purple(s string)`: 返回紫色文本
- `Cyan(s string)`: 返回青色文本
- `Gray(s string)`: 返回灰色文本
- `White(s string)`: 返回白色文本

### 颜色控制

提供启用或禁用彩色输出的功能：

- `SetColorEnabled(enabled bool)`: 设置是否启用彩色输出
- `IsColorEnabled() bool`: 检查是否启用彩色输出

## 使用示例

### 基本用法

```go
import (
    "fmt"
    "github.com/ava-labs/avalanche-rs/avalanchego-conformance/pkg/color"
)

func main() {
    // 输出彩色文本
    fmt.Println(color.Red("错误信息"))
    fmt.Println(color.Green("成功信息"))
    fmt.Println(color.Yellow("警告信息"))
    
    // 组合使用
    fmt.Printf("%s 处理完成，%s 项成功，%s 项失败\n",
        color.Blue("任务"),
        color.Green("10"),
        color.Red("2"),
    )
}
```

### 禁用彩色输出

```go
import (
    "fmt"
    "github.com/ava-labs/avalanche-rs/avalanchego-conformance/pkg/color"
)

func main() {
    // 检查是否启用彩色输出
    if color.IsColorEnabled() {
        fmt.Println("彩色输出已启用")
    }
    
    // 禁用彩色输出
    color.SetColorEnabled(false)
    
    // 现在文本不会有颜色
    fmt.Println(color.Red("这不会是红色的"))
}
```

## 扩展指南

### 添加新颜色

要添加新颜色，请按照以下步骤操作：

1. 在常量定义中添加新颜色代码：

```go
const (
    // 现有颜色
    Red    = "\033[31m"
    Green  = "\033[32m"
    
    // 添加新颜色
    Orange = "\033[38;5;208m"
    Pink   = "\033[38;5;219m"
)
```

2. 添加相应的格式化函数：

```go
func Orange(s string) string {
    if !colorEnabled {
        return s
    }
    return Orange + s + ResetColor
}

func Pink(s string) string {
    if !colorEnabled {
        return s
    }
    return Pink + s + ResetColor
}
```

### 添加文本样式

要添加文本样式（如粗体、下划线），请按照以下步骤操作：

1. 在常量定义中添加样式代码：

```go
const (
    // 颜色代码
    // ...
    
    // 样式代码
    Bold      = "\033[1m"
    Underline = "\033[4m"
    Blink     = "\033[5m"
)
```

2. 添加相应的格式化函数：

```go
func Bold(s string) string {
    if !colorEnabled {
        return s
    }
    return Bold + s + ResetColor
}

func Underline(s string) string {
    if !colorEnabled {
        return s
    }
    return Underline + s + ResetColor
}
```

3. 添加组合函数：

```go
func RedBold(s string) string {
    if !colorEnabled {
        return s
    }
    return Red + Bold + s + ResetColor
}
```
