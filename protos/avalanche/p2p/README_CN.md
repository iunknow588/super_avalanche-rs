# P2P 网络通信协议

本目录包含 Avalanche 网络中节点间点对点 (P2P) 通信的 Protocol Buffers 定义。

## 目录

1. [概述](#概述)
2. [文件说明](#文件说明)
3. [消息类型](#消息类型)
4. [设计模式](#设计模式)
5. [使用示例](#使用示例)
6. [版本兼容性](#版本兼容性)

## 概述

P2P 通信协议是 Avalanche 网络的基础，定义了节点之间如何交换信息、达成共识和同步状态。这些 Protocol Buffers 定义了所有网络消息的格式和结构，确保不同实现（如 Go 和 Rust）之间的互操作性。

## 文件说明

### p2p.proto

`p2p.proto` 是主要文件，定义了所有 P2P 网络消息类型。该文件包含：

- 消息容器 (`Message`)：封装所有可能的消息类型
- 网络握手消息：用于节点发现和连接建立
- 共识消息：用于节点间达成共识
- 引导消息：用于新节点加入网络时的状态同步
- 应用层消息：用于虚拟机间的自定义通信

## 消息类型

### 网络消息

1. **Ping/Pong**
   - `Ping`：报告节点的在线状态和正常运行时间
   - `Pong`：对 Ping 消息的响应

2. **握手消息**
   - `Handshake`：建立连接时发送的第一条消息，包含节点信息
   - `GetPeerList`：请求已知节点列表
   - `PeerList`：提供已知节点的列表

### 状态同步消息

1. **状态摘要消息**
   - `GetStateSummaryFrontier`：请求最新的状态摘要
   - `StateSummaryFrontier`：提供最新的状态摘要
   - `GetAcceptedStateSummary`：请求特定高度的状态摘要
   - `AcceptedStateSummary`：提供特定高度的状态摘要

### 引导消息

1. **边界和祖先消息**
   - `GetAcceptedFrontier`：请求接受的边界
   - `AcceptedFrontier`：提供接受的边界
   - `GetAccepted`：请求接受的容器
   - `Accepted`：提供接受的容器
   - `GetAncestors`：请求容器的祖先
   - `Ancestors`：提供容器的祖先

### 共识消息

1. **查询和投票消息**
   - `Get`：请求特定容器
   - `Put`：提供请求的容器
   - `PushQuery`：推送容器并请求偏好
   - `PullQuery`：请求特定容器的偏好
   - `Chits`：提供对查询的响应，表示节点的偏好

### 应用层消息

1. **虚拟机通信**
   - `AppRequest`：虚拟机定义的请求
   - `AppResponse`：对 AppRequest 的响应
   - `AppGossip`：虚拟机定义的广播消息
   - `AppError`：虚拟机定义的错误响应

## 设计模式

### 1. 消息封装

所有消息类型都封装在一个 `Message` 容器中，使用 `oneof` 字段确保只有一种消息类型被设置：

```protobuf
message Message {
  oneof message {
    bytes compressed_zstd = 2;
    
    Ping ping = 11;
    Pong pong = 12;
    Handshake handshake = 13;
    // ...其他消息类型
  }
}
```

这种设计允许：
- 统一的消息处理流程
- 消息压缩支持
- 未来扩展新消息类型

### 2. 请求-响应模式

许多消息遵循请求-响应模式，其中请求消息包含：
- 链 ID：标识目标链
- 请求 ID：唯一标识请求
- 截止时间：请求超时时间
- 请求特定数据

对应的响应消息包含：
- 链 ID：标识源链
- 请求 ID：对应原始请求
- 响应数据

例如：
```protobuf
message GetBlock {
  bytes chain_id = 1;
  uint32 request_id = 2;
  uint64 deadline = 3;
  bytes block_id = 4;
}

message Block {
  bytes chain_id = 1;
  uint32 request_id = 2;
  bytes block_data = 3;
}
```

### 3. 版本兼容性

协议使用以下技术确保版本兼容性：

1. **保留字段**：使用 `reserved` 关键字标记已删除的字段
   ```protobuf
   message Ping {
     uint32 uptime = 1;
     reserved 2; // 直到 Etna 升级激活
   }
   ```

2. **可选字段**：新增字段标记为 `optional`，确保旧客户端可以处理新消息

3. **字段编号管理**：谨慎分配和管理字段编号，避免冲突

### 4. 消息压缩

支持消息压缩以减少网络带宽使用：

```protobuf
message Message {
  oneof message {
    // zstd 压缩的 Message 字节
    bytes compressed_zstd = 2;
    
    // 未压缩的消息类型
    Ping ping = 11;
    // ...
  }
}
```

## 使用示例

### 在 Rust 中处理 P2P 消息

```rust
use avalanche_types::proto::pb::p2p::{Message, message};

// 创建 Ping 消息
let ping_msg = message::Ping {
    uptime: 95,
};

// 封装到 Message 容器
let message = Message {
    message: Some(message::Message::Ping(ping_msg)),
};

// 序列化消息
let bytes = message.encode_to_vec();

// 反序列化消息
let decoded_message = Message::decode(bytes.as_slice()).unwrap();

// 处理消息
match decoded_message.message {
    Some(message::Message::Ping(ping)) => {
        println!("收到 Ping 消息，正常运行时间: {}", ping.uptime);
    },
    Some(message::Message::Pong(_)) => {
        println!("收到 Pong 消息");
    },
    // 处理其他消息类型...
    _ => println!("未知消息类型"),
}
```

## 版本兼容性

P2P 协议随着网络升级而演进。重要的兼容性注意事项：

1. **字段保留**：已删除的字段使用 `reserved` 标记，防止字段编号重用
2. **新消息类型**：新增消息类型不会影响旧版本节点处理现有消息
3. **可选字段**：新增字段应标记为 `optional`，确保向后兼容性
4. **升级标记**：某些字段带有注释，指示它们在特定网络升级后才启用
