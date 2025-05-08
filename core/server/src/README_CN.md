# 服务器模块源代码

本目录包含服务器模块的核心实现代码，提供 HTTP API 服务和请求处理功能。

## 文件结构

- `lib.rs`: 模块入口点，定义公共 API 和类型
- `handler.rs`: HTTP 请求处理器实现

## 详细说明

### lib.rs

模块的主入口文件，定义了以下内容：

- 公共 API 和类型导出
- 模块级文档
- 错误类型定义

这个文件保持简洁，主要功能实现在 `handler.rs` 中。

### handler.rs

HTTP 请求处理器的实现，包含以下功能：

1. **服务器配置**
   - 监听地址和端口
   - 超时设置
   - TLS 配置

2. **请求路由**
   - API 端点定义
   - 路由规则
   - 中间件配置

3. **请求处理**
   - 解析请求参数
   - 执行业务逻辑
   - 生成响应

4. **健康检查**
   - 服务器状态监控
   - 健康检查端点

5. **优雅关闭**
   - 处理关闭信号
   - 等待活跃连接完成
   - 资源清理

## 设计模式

1. **责任链模式**
   - 用于请求处理流程
   - 中间件链式处理

2. **单例模式**
   - 用于服务器实例
   - 确保资源的统一管理

3. **命令模式**
   - 封装请求处理操作
   - 将请求参数和处理逻辑分离

4. **工厂模式**
   - 创建不同类型的处理器
   - 根据请求类型选择处理器

## 使用示例

```rust
use server::handler::Handler;
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 创建处理器
    let handler = Handler::new("0.0.0.0", 9650, Duration::from_secs(30));
    
    // 启动服务器
    handler.start().await?;
    
    // 等待关闭信号
    tokio::signal::ctrl_c().await?;
    
    // 优雅关闭
    handler.shutdown().await?;
    
    Ok(())
}
```

## API 端点

服务器提供以下主要 API 端点：

1. **健康检查**
   - `GET /health`: 返回服务器健康状态

2. **节点信息**
   - `GET /node/info`: 返回节点信息
   - `GET /node/version`: 返回节点版本

3. **网络操作**
   - `GET /network/peers`: 返回连接的对等节点列表
   - `POST /network/connect`: 连接到指定节点

4. **区块链操作**
   - `GET /blockchain/status`: 返回区块链状态
   - `POST /blockchain/tx`: 提交交易

## 错误处理

模块使用自定义错误类型处理各种服务器错误：

- 配置错误
- 绑定错误
- 请求处理错误
- 超时错误

所有公共函数都返回 `Result` 类型，便于错误传播和处理。

## 依赖关系

主要依赖以下外部库：

- `tokio`: 异步运行时
- `hyper`: HTTP 服务器和客户端
- `tower`: HTTP 中间件
- `serde`: 序列化和反序列化

## 未来改进

1. 添加更多 API 端点
2. 实现请求限流
3. 添加认证和授权
4. 优化请求处理性能
5. 添加指标收集和监控
