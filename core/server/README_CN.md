# 服务器模块 (Server)

本模块实现了 Avalanche 节点的 HTTP 服务器功能，提供 API 端点和请求处理能力。

## 目录

1. [概述](#概述)
2. [目录结构](#目录结构)
3. [核心功能](#核心功能)
4. [设计模式](#设计模式)
5. [使用示例](#使用示例)
6. [配置选项](#配置选项)
7. [API 文档](#api-文档)

## 概述

服务器模块是 Avalanche 节点的重要组件，负责以下功能：

- 提供 HTTP/HTTPS API 端点
- 处理来自客户端的请求
- 路由请求到适当的处理程序
- 返回响应和错误信息
- 提供节点状态和健康检查

该模块基于 Tokio 和 Hyper 构建，支持异步请求处理和高并发。

## 目录结构

- `src/`: 服务器核心实现代码
  - `lib.rs`: 模块入口文件，定义公共 API
  - `handler.rs`: 请求处理器实现，包含路由和中间件
- `.gitignore`: Git 忽略文件
- `Cargo.toml`: 项目配置文件，定义依赖和版本
- `LICENSE`: 许可证文件
- `README.md`: 英文说明文档

## 核心功能

### 1. HTTP 服务器

- **多端口监听**：支持在多个端口上监听请求
- **TLS 支持**：可选的 HTTPS 支持
- **优雅关闭**：支持优雅关闭服务器
- **请求超时**：自动处理超时请求

### 2. 请求路由

- **路径路由**：基于 URL 路径的请求路由
- **方法路由**：基于 HTTP 方法的请求路由
- **版本路由**：支持 API 版本控制
- **中间件支持**：请求处理前后的中间件

### 3. 请求处理

- **JSON 处理**：解析和生成 JSON 请求/响应
- **参数验证**：验证请求参数
- **错误处理**：统一的错误处理机制
- **日志记录**：请求和响应日志

### 4. API 端点

- **信息端点**：提供节点信息
- **健康检查**：节点健康状态检查
- **指标端点**：性能和状态指标
- **管理端点**：节点管理功能

## 设计模式

### 1. 责任链模式

用于处理不同类型的网络请求：

```rust
// 处理器接口
pub trait Handler {
    fn handle(&self, request: Request) -> Result<Response, Error>;
}

// 责任链
pub struct HandlerChain {
    handlers: Vec<Box<dyn Handler>>,
}

impl HandlerChain {
    pub fn add_handler(&mut self, handler: Box<dyn Handler>) {
        self.handlers.push(handler);
    }

    pub fn handle(&self, request: Request) -> Result<Response, Error> {
        for handler in &self.handlers {
            if let Some(response) = handler.handle(request.clone()).ok() {
                return Ok(response);
            }
        }
        Err(Error::NotFound)
    }
}
```

### 2. 单例模式

确保关键服务的唯一实例：

```rust
// 服务器单例
pub struct Server {
    // 服务器状态
}

impl Server {
    // 获取单例实例
    pub fn instance() -> &'static Server {
        static INSTANCE: OnceCell<Server> = OnceCell::new();
        INSTANCE.get_or_init(|| {
            Server::new()
        })
    }
}
```

### 3. 命令模式

封装请求处理操作：

```rust
// 命令接口
pub trait Command {
    fn execute(&self) -> Result<Response, Error>;
}

// 具体命令
pub struct GetInfoCommand;
pub struct SubmitTxCommand {
    tx_data: Vec<u8>,
}

// 命令执行器
pub struct CommandExecutor;

impl CommandExecutor {
    pub fn execute(&self, command: Box<dyn Command>) -> Result<Response, Error> {
        command.execute()
    }
}
```

### 4. 装饰器模式

添加中间件功能：

```rust
// 处理器接口
pub trait Handler {
    fn handle(&self, request: Request) -> Result<Response, Error>;
}

// 基本处理器
pub struct BaseHandler;

// 日志中间件
pub struct LoggingMiddleware<H> {
    inner: H,
}

// 认证中间件
pub struct AuthMiddleware<H> {
    inner: H,
}
```

## 使用示例

### 创建和启动服务器

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

### 添加自定义路由

```rust
use server::handler::{Handler, Router};
use hyper::{Body, Request, Response, StatusCode};

// 创建处理器
let mut handler = Handler::new("0.0.0.0", 9650, Duration::from_secs(30));

// 添加自定义路由
handler.router_mut().add(
    "/custom",
    |_req: Request<Body>| async {
        Ok(Response::builder()
            .status(StatusCode::OK)
            .body(Body::from("Custom endpoint"))
            .unwrap())
    },
);

// 启动服务器
handler.start().await?;
```

### 使用中间件

```rust
use server::handler::{Handler, Middleware};
use hyper::{Body, Request, Response};
use std::future::Future;
use std::pin::Pin;

// 定义中间件
struct LoggingMiddleware;

impl Middleware for LoggingMiddleware {
    fn call<'a>(
        &'a self,
        req: Request<Body>,
        next: Pin<Box<dyn Future<Output = Response<Body>> + Send + 'a>>,
    ) -> Pin<Box<dyn Future<Output = Response<Body>> + Send + 'a>> {
        Box::pin(async move {
            println!("Request: {} {}", req.method(), req.uri());
            let response = next.await;
            println!("Response: {}", response.status());
            response
        })
    }
}

// 创建处理器
let mut handler = Handler::new("0.0.0.0", 9650, Duration::from_secs(30));

// 添加中间件
handler.add_middleware(Box::new(LoggingMiddleware));

// 启动服务器
handler.start().await?;
```

## 配置选项

服务器模块支持以下配置选项：

| 选项 | 描述 | 默认值 |
|------|------|--------|
| `address` | 监听地址 | "0.0.0.0" |
| `port` | 监听端口 | 9650 |
| `request_timeout` | 请求超时（秒） | 30 |
| `max_connections` | 最大连接数 | 1024 |
| `enable_tls` | 是否启用 TLS | false |
| `cert_file` | TLS 证书文件 | "" |
| `key_file` | TLS 密钥文件 | "" |
| `enable_cors` | 是否启用 CORS | true |
| `log_requests` | 是否记录请求日志 | true |

## API 文档

完整的 API 文档请参考代码文档或使用以下命令生成：

```bash
cargo doc --open
```

## 注意事项

- 该模块正在积极开发中，API 可能会发生变化
- 某些高级功能可能尚未完全实现
- 性能优化仍在进行中
