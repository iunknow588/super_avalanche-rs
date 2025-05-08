# Snowman 共识协议实现

本目录包含 Snowman 共识协议的实现代码，Snowman 是 Avalanche 共识协议的一个变体，专门用于线性区块链。

## 目录

1. [概述](#概述)
2. [目录结构](#目录结构)
3. [核心组件](#核心组件)
4. [工作原理](#工作原理)
5. [使用示例](#使用示例)

## 概述

Snowman 协议是 Avalanche 共识协议的一个变体，专门设计用于线性区块链（每个区块只有一个父区块）。它保留了 Avalanche 的核心优势，如高吞吐量、快速终结和能源效率，同时适应了区块链的线性结构。

Snowman 协议的主要特点：

- **线性结构**：每个区块只有一个父区块，形成链式结构
- **基于采样的投票**：通过随机采样其他节点的偏好来达成共识
- **概率终结**：随着投票轮数增加，区块被接受的概率接近 1
- **分叉选择规则**：在多个竞争链中选择最优的一条

## 目录结构

- `mod.rs`: 模块入口，导出公共 API
- `block/`: 区块相关类型和接口
  - `mod.rs`: 区块模块入口
  - `state.rs`: 区块状态管理
  - `test_block.rs`: 测试用区块实现
- `bootstrap/`: 引导过程实现
  - `mod.rs`: 引导模块入口
  - `bootstrapper.rs`: 引导器实现
  - `config.rs`: 引导配置
- `consensus/`: 共识算法核心
  - `mod.rs`: 共识模块入口
  - `consensus.rs`: 共识引擎实现
  - `metrics.rs`: 性能指标收集
- `topological/`: 拓扑排序实现
  - `mod.rs`: 拓扑排序模块入口
  - `sort.rs`: 排序算法实现

## 核心组件

### block/

定义了区块接口和状态管理：

1. **Block Trait**
   - 定义区块的基本接口
   - 包括 ID、父区块、高度、时间戳等属性
   - 与 `Decidable` trait 结合，支持共识决策

2. **BlockState**
   - 管理区块的状态（待处理、已接受、已拒绝）
   - 跟踪区块的依赖关系
   - 维护区块的元数据

3. **TestBlock**
   - 用于测试的区块实现
   - 提供简单的区块创建和操作功能

### bootstrap/

实现节点引导过程：

1. **Bootstrapper**
   - 管理新节点加入网络时的状态同步
   - 处理区块下载和验证
   - 支持从检查点恢复

2. **Config**
   - 引导过程的配置参数
   - 包括超时设置、重试策略等

### consensus/

实现共识算法核心：

1. **Consensus**
   - 实现 Snowman 共识算法
   - 管理区块的投票和决策过程
   - 实现分叉选择规则

2. **Metrics**
   - 收集共识过程的性能指标
   - 跟踪处理的区块数、投票轮数等

### topological/

实现拓扑排序算法：

1. **Sort**
   - 对有向无环图 (DAG) 进行拓扑排序
   - 处理区块之间的依赖关系
   - 支持增量排序

## 工作原理

### 1. 区块处理流程

```
接收新区块
    │
    ▼
验证区块合法性
    │
    ▼
添加到待处理集合
    │
    ▼
更新区块依赖关系
    │
    ▼
执行投票过程
    │
    ▼
根据投票结果更新状态
    │
    ▼
如果达到决策阈值，接受或拒绝区块
```

### 2. 投票机制

Snowman 使用基于采样的投票机制：

1. 随机选择 `k` 个节点进行采样
2. 向这些节点查询它们对特定区块的偏好
3. 如果至少有 `alpha` 个节点赞成，则增加该区块的置信度
4. 如果置信度达到阈值 `beta`，则接受该区块
5. 否则，继续进行更多轮次的投票

### 3. 分叉选择

当存在多个竞争的区块链分支时，Snowman 使用以下规则选择最优分支：

1. 首先考虑已经达到最高置信度的分支
2. 如果多个分支具有相同的置信度，选择最长的分支
3. 如果长度也相同，选择时间戳最早的分支
4. 如果时间戳也相同，选择 ID 最小的分支

## 使用示例

### 创建和配置共识引擎

```rust
use avalanche_consensus::{Parameters, snowman::consensus::Consensus};
use avalanche_consensus::snowman::block::TestBlock;

// 创建共识参数
let params = Parameters {
    k: 20,                // 采样大小
    alpha: 15,            // 法定人数大小
    beta_virtuous: 15,    // 良性决策阈值
    beta_rogue: 20,       // 恶意决策阈值
    concurrency: 4,       // 并发度
};

// 创建共识引擎
let mut consensus = Consensus::<TestBlock>::new(params);

// 初始化
consensus.initialize();
```

### 处理区块

```rust
use avalanche_consensus::snowman::block::TestBlock;
use avalanche_types::ids::Id;

// 创建创世区块
let genesis = TestBlock::new(
    None,           // 没有父区块
    0,              // 高度为 0
    0,              // 时间戳为 0
    vec![0u8; 32],  // 数据
);

// 处理创世区块
consensus.add_block(genesis.clone()).unwrap();

// 创建子区块
let block1 = TestBlock::new(
    Some(genesis.id()),  // 父区块是创世区块
    1,                   // 高度为 1
    1000,                // 时间戳
    vec![1u8; 32],       // 数据
);

// 处理子区块
consensus.add_block(block1.clone()).unwrap();

// 查询区块状态
let status = consensus.block_status(&block1.id());
println!("Block status: {:?}", status);
```

### 处理投票和查询结果

```rust
// 模拟投票过程
consensus.record_poll(/* 投票结果 */);

// 获取首选区块
let preferred = consensus.preference();
println!("Preferred block: {:?}", preferred);

// 获取已接受的区块
let accepted = consensus.accepted();
println!("Accepted blocks: {:?}", accepted);
```
