# 日志工具

本目录包含日志工具函数，用于配置和管理应用程序的日志系统，提供统一的日志接口和格式。

## 目录

1. [文件结构](#文件结构)
2. [设计模式](#设计模式)
3. [核心功能](#核心功能)
4. [使用示例](#使用示例)
5. [扩展指南](#扩展指南)

## 文件结构

- `logutil.go`: 提供日志配置和格式化功能

## 设计模式

### 单例模式

使用单例模式管理全局日志实例：

```go
var (
    globalLogger *zap.Logger
    globalMu     sync.Mutex
)

func GetLogger() *zap.Logger {
    globalMu.Lock()
    defer globalMu.Unlock()
    
    if globalLogger == nil {
        // 初始化默认日志
        globalLogger, _ = NewLogger("info")
    }
    
    return globalLogger
}
```

### 工厂模式

使用工厂方法创建日志实例：

```go
func NewLogger(logLevel string) (*zap.Logger, error) {
    // 解析日志级别
    level, err := zapcore.ParseLevel(logLevel)
    if err != nil {
        return nil, err
    }
    
    // 创建日志配置
    config := zap.Config{
        // ...
    }
    
    // 创建日志实例
    return config.Build()
}
```

### 装饰器模式

使用装饰器模式添加额外的日志功能：

```go
func WithFields(logger *zap.Logger, fields ...zap.Field) *zap.Logger {
    return logger.With(fields...)
}
```

## 核心功能

### 日志级别管理

支持多种日志级别：

- `debug`: 调试信息
- `info`: 一般信息
- `warn`: 警告信息
- `error`: 错误信息
- `dpanic`: 开发环境中的严重错误
- `panic`: 导致 panic 的严重错误
- `fatal`: 导致程序终止的致命错误

### 日志格式配置

支持多种日志格式：

- JSON 格式：适合机器处理
- 控制台格式：适合人类阅读

### 全局日志管理

提供全局日志实例和管理函数：

- `GetLogger()`: 获取全局日志实例
- `SetLogger(logger *zap.Logger)`: 设置全局日志实例

### 日志工具函数

提供各种日志工具函数：

- `ParseLevel(level string)`: 解析日志级别
- `WithFields(logger *zap.Logger, fields ...zap.Field)`: 添加字段到日志
- `NewFileLogger(filename string, level string)`: 创建文件日志

## 使用示例

### 基本用法

```go
import (
    "github.com/ava-labs/avalanche-rs/avalanchego-conformance/pkg/logutil"
    "go.uber.org/zap"
)

func main() {
    // 创建日志实例
    logger, err := logutil.NewLogger("info")
    if err != nil {
        panic(err)
    }
    
    // 记录日志
    logger.Info("服务启动", zap.String("port", "8080"))
    logger.Debug("调试信息", zap.Int("connections", 10))
    logger.Error("发生错误", zap.Error(err))
}
```

### 使用全局日志

```go
import (
    "github.com/ava-labs/avalanche-rs/avalanchego-conformance/pkg/logutil"
    "go.uber.org/zap"
)

func init() {
    // 初始化全局日志
    logger, _ := logutil.NewLogger("debug")
    logutil.SetLogger(logger)
}

func main() {
    // 使用全局日志
    logutil.GetLogger().Info("使用全局日志")
    
    // 在函数中使用全局日志
    doSomething()
}

func doSomething() {
    logutil.GetLogger().Debug("执行操作")
}
```

### 添加结构化字段

```go
import (
    "github.com/ava-labs/avalanche-rs/avalanchego-conformance/pkg/logutil"
    "go.uber.org/zap"
)

func main() {
    logger, _ := logutil.NewLogger("info")
    
    // 添加固定字段
    requestLogger := logutil.WithFields(logger,
        zap.String("request_id", "123456"),
        zap.String("user_id", "user-001"),
    )
    
    // 使用带有固定字段的日志
    requestLogger.Info("处理请求")
    requestLogger.Info("请求完成", zap.Int("status", 200))
}
```

## 扩展指南

### 添加新的日志格式

要添加新的日志格式，请按照以下步骤操作：

1. 创建新的编码器配置：

```go
func NewCustomEncoderConfig() zapcore.EncoderConfig {
    return zapcore.EncoderConfig{
        // 自定义编码器配置
    }
}
```

2. 在 `NewLogger` 函数中添加格式选项：

```go
func NewLogger(logLevel string, format string) (*zap.Logger, error) {
    // ...
    
    var encoder zapcore.Encoder
    switch format {
    case "json":
        encoder = zapcore.NewJSONEncoder(encoderConfig)
    case "console":
        encoder = zapcore.NewConsoleEncoder(encoderConfig)
    case "custom":
        encoder = zapcore.NewConsoleEncoder(NewCustomEncoderConfig())
    default:
        encoder = zapcore.NewConsoleEncoder(encoderConfig)
    }
    
    // ...
}
```

### 添加日志轮转功能

要添加日志轮转功能，请按照以下步骤操作：

1. 添加 lumberjack 依赖：

```go
import (
    "gopkg.in/natefinch/lumberjack.v2"
)
```

2. 创建支持轮转的日志函数：

```go
func NewRotatingFileLogger(filename string, level string, maxSize int, maxBackups int, maxAge int) (*zap.Logger, error) {
    // 解析日志级别
    zapLevel, err := zapcore.ParseLevel(level)
    if err != nil {
        return nil, err
    }
    
    // 创建轮转日志写入器
    rotator := &lumberjack.Logger{
        Filename:   filename,
        MaxSize:    maxSize,    // 单位：MB
        MaxBackups: maxBackups, // 最大备份数量
        MaxAge:     maxAge,     // 最大保留天数
        Compress:   true,       // 压缩备份
    }
    
    // 创建核心
    core := zapcore.NewCore(
        zapcore.NewJSONEncoder(zap.NewProductionEncoderConfig()),
        zapcore.AddSync(rotator),
        zapLevel,
    )
    
    // 创建日志实例
    return zap.New(core), nil
}
```
