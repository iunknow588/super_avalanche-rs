# 二元 Snowball 算法实现

本目录包含 Avalanche 共识协议中二元 Snowball 算法的 Rust 实现，用于处理两个选择之间的共识决策。

## 目录

1. [概述](#概述)
2. [文件结构](#文件结构)
3. [核心组件](#核心组件)
4. [设计模式](#设计模式)
5. [实现细节](#实现细节)
6. [使用示例](#使用示例)

## 概述

二元 Snowball 算法是 Snowball 共识协议的一个变体，专门用于处理两个选择之间的决策。在二元 Snowball 中，节点在特定位置测试两个选择的差异，并根据投票结果更新其偏好。这种方法允许系统在有冲突的选择之间高效地达成共识。

二元 Snowball 节点通常作为 Snowball 树的一部分使用，当在特定位置检测到冲突时，一元节点会转变为二元节点。

## 文件结构

- `mod.rs`：模块入口，定义 `Snowball` 结构体和相关方法
- `node.rs`：定义 `Node` 结构体和节点级操作

## 核心组件

### Snowball 结构体

`mod.rs` 中定义的 `Snowball` 结构体实现了二元 Snowball 算法的核心逻辑：

```rust
pub struct Snowball {
    /// 偏好为 0 的成功投票次数
    pub preference0_successful_polls: Cell<i64>,
    
    /// 偏好为 1 的成功投票次数
    pub preference1_successful_polls: Cell<i64>,
    
    /// 雪花算法实例
    pub snowflake: snowflake::Snowflake,
}
```

`Snowball` 提供了以下核心方法：

- `new()`：创建新的 Snowball 实例
- `preference()`：返回当前的偏好选择（0 或 1）
- `record_successful_poll()`：记录成功的投票
- `record_unsuccessful_poll()`：记录不成功的投票

### Node 结构体

`node.rs` 中定义的 `Node` 结构体实现了二元 Snowball 节点：

```rust
pub struct Node {
    /// 共识参数
    pub parameters: crate::Parameters,
    
    /// Snowball 实例
    pub snowball: Snowball,
    
    /// 当前测试的位索引
    pub bit: usize,
    
    /// 当前偏好的选择
    pub preference: Cell<Id>,
    
    /// 已决定的前缀位数
    pub decided_prefix: Cell<i64>,
    
    /// 偏好为 0 的子节点
    pub child0: Option<Box<snowball::Node>>,
    
    /// 偏好为 1 的子节点
    pub child1: Option<Box<snowball::Node>>,
    
    /// 是否需要重置
    pub should_reset: Cell<bool>,
}
```

`Node` 提供了以下核心方法：

- `new()`：创建新的二元节点
- `preference()`：返回当前的偏好选择
- `decided_prefix()`：返回已决定的前缀位数
- `finalized()`：检查是否已达成共识
- `add()`：添加新的选择
- `record_poll()`：记录投票结果

## 设计模式

### 1. 组合模式

二元节点使用组合模式构建树形结构，每个节点可以包含两个子节点：

```rust
pub struct Node {
    // ...
    pub child0: Option<Box<snowball::Node>>,
    pub child1: Option<Box<snowball::Node>>,
    // ...
}
```

这种设计允许递归处理复杂的决策树。

### 2. 状态模式

二元 Snowball 算法本质上是一个状态机，通过 `record_poll()` 方法更新状态：

```rust
pub fn record_poll(&mut self, votes: &Bag, reset: bool) -> (snowball::Node, bool) {
    // ...
    if reset {
        self.snowball.record_unsuccessful_poll();
    }
    
    // ...
    
    if preference_votes >= alpha {
        self.snowball.record_successful_poll(preference_bit);
        // ...
    } else {
        self.snowball.record_unsuccessful_poll();
        // ...
    }
    // ...
}
```

### 3. 策略模式

通过 `snowball::Node` 枚举，系统可以在运行时选择不同的节点实现：

```rust
pub enum Node {
    Unary(unary::node::Node),
    Binary(binary::node::Node),
}
```

这种设计允许根据具体情况选择最合适的算法变体。

## 实现细节

### 位测试

二元节点的核心操作是在特定位置测试选择的差异：

```rust
pub fn new(
    parameters: crate::Parameters,
    bit: usize,
    preference: Id,
    decided_prefix: i64,
) -> Self {
    // ...
    Self {
        parameters,
        snowball: Snowball::new(beta_rogue),
        bit,
        preference: Cell::new(preference),
        decided_prefix: Cell::new(decided_prefix),
        child0: None,
        child1: None,
        should_reset: Cell::new(false),
    }
}
```

`bit` 字段指定了要测试的位索引，用于确定选择应该走哪条路径。

### 投票过程

投票过程是二元 Snowball 算法的核心，通过以下步骤实现：

1. 如果需要重置，调用 `record_unsuccessful_poll()`
2. 计算当前偏好位的值
3. 统计支持每个选择的票数
4. 如果达到 alpha 阈值，记录成功的投票
5. 否则，记录不成功的投票
6. 更新偏好和子节点状态

```rust
pub fn record_poll(&mut self, votes: &Bag, reset: bool) -> (snowball::Node, bool) {
    if reset {
        self.snowball.record_unsuccessful_poll();
    }
    
    let preference = self.preference.get();
    let preference_bit = bits::get(&preference, self.bit);
    
    let alpha = self.parameters.alpha;
    let preference_votes = votes.count(&preference);
    
    // ...
    
    if preference_votes >= alpha {
        self.snowball.record_successful_poll(preference_bit);
        // ...
    } else {
        self.snowball.record_unsuccessful_poll();
        // ...
    }
    
    // ...
}
```

### 子节点管理

二元节点可以有两个子节点，分别对应位值为 0 和 1 的情况：

```rust
pub fn add(&mut self, new_choice: &Id) -> snowball::Node {
    // ...
    
    let new_bit = bits::get(new_choice, self.bit);
    if new_bit == 0 {
        if let Some(child0) = &mut self.child0 {
            let added_node = child0.add(new_choice);
            *child0 = Box::new(added_node);
        } else {
            self.child0 = Some(Box::new(snowball::Node::Unary(unary::node::Node::new(
                self.parameters.clone(),
                *new_choice,
                decided_prefix,
            ))));
        }
    } else {
        if let Some(child1) = &mut self.child1 {
            let added_node = child1.add(new_choice);
            *child1 = Box::new(added_node);
        } else {
            self.child1 = Some(Box::new(snowball::Node::Unary(unary::node::Node::new(
                self.parameters.clone(),
                *new_choice,
                decided_prefix,
            ))));
        }
    }
    
    // ...
}
```

## 使用示例

以下是使用二元 Snowball 节点的基本示例：

```rust
// 创建二元节点
let mut node = Node::new(
    crate::Parameters {
        k: 20,                // 采样大小
        alpha: 15,            // 法定人数大小
        beta_virtuous: 15,    // 良性决策阈值
        beta_rogue: 20,       // 恶意决策阈值
        ..Default::default()
    },
    0,                        // 测试位索引
    initial_preference,       // 初始偏好
    0,                        // 已决定前缀
);

// 添加新的选择
let snowball_node = node.add(&new_choice);

// 记录投票结果
let votes = Bag::new();
votes.add_count(&choice1, count1);
votes.add_count(&choice2, count2);
let (updated_node, successful) = node.record_poll(&votes, false);

// 检查是否已达成共识
if updated_node.finalized() {
    println!("共识已达成，最终选择: {}", updated_node.preference());
}
```

更多详细示例请参考 `node.rs` 文件中的测试用例。
