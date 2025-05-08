# Avalanche 共识模块 (avalanche-consensus)

本模块实现了 Avalanche 共识协议的核心逻辑，提供了高性能、确定性的共识机制。

## 目录

1. [概述](#概述)
2. [目录结构](#目录结构)
3. [核心功能](#核心功能)
4. [设计模式](#设计模式)
5. [使用示例](#使用示例)
6. [参数配置](#参数配置)
7. [API 文档](#api-文档)

## 概述

Avalanche 共识协议是一种高性能、可扩展的共识机制，结合了经典共识和 Nakamoto 共识的优点。本模块实现了 Avalanche 共识协议的 Rust 版本，特别是 Snowman 协议，用于线性区块链的共识。

主要特点包括：

- **高吞吐量**：每秒可处理数千笔交易
- **快速终结**：交易在几秒内达成最终确认
- **能源效率**：不需要大量计算资源
- **去中心化**：支持大规模节点参与
- **安全性**：抵抗各种攻击，包括 Sybil 攻击

## 目录结构

- `src/`: 共识核心实现
  - `lib.rs`: 主模块文件，包含共识引擎实现和参数定义
  - `context.rs`: 共识上下文管理
  - `snowman/`: Snowman 共识协议实现
    - `block/`: 区块相关类型和操作
    - `bootstrap/`: 引导过程实现
    - `consensus/`: 共识算法核心
    - `topological/`: 拓扑排序实现
    - `mod.rs`: Snowman 模块入口
- `Cargo.toml`: 项目配置文件
- `LICENSE`: 许可证文件
- `README.md`: 英文文档
- `tests.unit.sh`: 单元测试脚本

## 核心功能

### 1. 共识参数管理

- **参数验证**：验证共识参数的有效性
- **参数优化**：根据网络情况优化参数
- **默认配置**：提供合理的默认参数值

### 2. Snowman 共识协议

- **区块处理**：处理新区块的提议和验证
- **投票机制**：实现基于采样的投票系统
- **分叉选择**：选择最优的区块链分支
- **最终确认**：确定区块的最终状态

### 3. 引导过程

- **初始同步**：新节点加入网络时的状态同步
- **检查点处理**：使用检查点加速同步
- **状态验证**：验证同步状态的正确性

### 4. 拓扑排序

- **DAG 管理**：管理有向无环图结构
- **排序算法**：实现高效的拓扑排序
- **冲突解决**：处理排序中的冲突

## 设计模式

### 1. 状态模式 (State Pattern)

用于管理共识过程中的不同状态：

```rust
enum ConsensusState {
    Bootstrapping,
    NormalOperation,
    Syncing,
}

struct ConsensusEngine {
    state: ConsensusState,
    // ...
}

impl ConsensusEngine {
    fn process_block(&mut self, block: Block) {
        match self.state {
            ConsensusState::Bootstrapping => { /* ... */ },
            ConsensusState::NormalOperation => { /* ... */ },
            ConsensusState::Syncing => { /* ... */ },
        }
    }
}
```

### 2. 发布-订阅模式 (Observer Pattern)

用于处理共识事件通知：

```rust
trait ConsensusObserver {
    fn on_block_accepted(&self, block_id: &Id);
    fn on_block_rejected(&self, block_id: &Id);
}

struct ConsensusEngine {
    observers: Vec<Box<dyn ConsensusObserver>>,
    // ...
}

impl ConsensusEngine {
    fn notify_block_accepted(&self, block_id: &Id) {
        for observer in &self.observers {
            observer.on_block_accepted(block_id);
        }
    }
}
```

### 3. 模板方法模式 (Template Method Pattern)

定义共识算法的骨架，允许子类重写特定步骤：

```rust
trait Consensus {
    fn initialize(&mut self);
    fn add_block(&mut self, block: Block) -> Result<(), Error>;
    fn finalize_block(&mut self, block_id: &Id) -> Result<(), Error>;

    // 模板方法
    fn process_block(&mut self, block: Block) -> Result<(), Error> {
        self.add_block(block)?;
        // 共同的处理逻辑
        Ok(())
    }
}
```

### 4. 策略模式 (Strategy Pattern)

允许在运行时选择不同的共识算法实现：

```rust
trait ConsensusStrategy {
    fn select_preferred(&self, blocks: &[Block]) -> Option<Block>;
}

struct SnowmanStrategy;
struct AvalancheStrategy;

impl ConsensusStrategy for SnowmanStrategy {
    fn select_preferred(&self, blocks: &[Block]) -> Option<Block> {
        // Snowman 实现
    }
}
```

## 使用示例

### 创建共识引擎

```rust
use avalanche_consensus::{Parameters, snowman::consensus::Consensus};

// 创建共识参数
let params = Parameters {
    k: 20,                // 采样大小
    alpha: 15,            // 法定人数大小
    beta_virtuous: 15,    // 良性决策阈值
    beta_rogue: 20,       // 恶意决策阈值
    concurrency: 4,       // 并发度
    // 其他参数...
};

// 验证参数
params.verify().expect("Invalid parameters");

// 创建共识引擎
let mut consensus = Consensus::new(params);

// 初始化共识引擎
consensus.initialize();
```

### 处理区块

```rust
use avalanche_consensus::snowman::block::Block;

// 处理新区块
let block = /* 从网络接收的区块 */;
match consensus.process_block(block) {
    Ok(_) => println!("Block processed successfully"),
    Err(e) => println!("Failed to process block: {}", e),
}

// 查询区块状态
let block_id = /* 区块 ID */;
let status = consensus.block_status(&block_id);
println!("Block status: {:?}", status);
```

### 处理投票

```rust
// 处理来自其他节点的投票
let vote = /* 从网络接收的投票 */;
consensus.record_poll(vote);

// 获取首选区块
let preferred = consensus.preference();
println!("Preferred block: {:?}", preferred);
```

## 参数配置

Avalanche 共识协议的性能和安全性很大程度上取决于其参数配置。以下是主要参数及其推荐值：

| 参数 | 描述 | 推荐值 | 影响 |
|------|------|--------|------|
| `k` | 采样大小 | 20 | 更大的值提高安全性，但增加网络负载 |
| `alpha` | 法定人数大小 | 15 | 必须 ≤ k，影响共识达成速度和安全性 |
| `beta_virtuous` | 良性决策阈值 | 15 | 影响无冲突交易的确认速度 |
| `beta_rogue` | 恶意决策阈值 | 20 | 影响有冲突交易的确认速度 |
| `concurrency` | 并发度 | 4 | 影响处理效率，应根据硬件调整 |

## API 文档

完整的 API 文档请参考代码文档或使用以下命令生成：

```bash
cargo doc --open
```
