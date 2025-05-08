# 一元 Snowball 算法实现

本目录包含 Avalanche 共识协议中一元 Snowball 算法的 Rust 实现，用于处理单一选择的共识决策。

## 目录

1. [概述](#概述)
2. [文件结构](#文件结构)
3. [核心组件](#核心组件)
4. [设计模式](#设计模式)
5. [实现细节](#实现细节)
6. [使用示例](#使用示例)

## 概述

一元 Snowball 算法是 Snowball 共识协议的基本变体，用于处理单一选择的情况。在一元 Snowball 中，节点维护一个单一的偏好，并通过投票过程逐渐增强对该偏好的信心，直到达到终结阈值。

一元 Snowball 节点通常作为 Snowball 树的叶子节点使用，当检测到冲突时，它可以转变为二元节点。这种设计允许系统高效地处理大多数无冲突的情况，同时在需要时能够处理冲突。

## 文件结构

- `mod.rs`：模块入口，定义 `Snowball` 结构体和相关方法
- `node.rs`：定义 `Node` 结构体和节点级操作

## 核心组件

### Snowball 结构体

`mod.rs` 中定义的 `Snowball` 结构体实现了一元 Snowball 算法的核心逻辑：

```rust
pub struct Snowball {
    /// 成功投票的次数
    pub successful_polls: Cell<i64>,
    
    /// 雪花算法实例
    pub snowflake: snowflake::Snowflake,
}
```

`Snowball` 提供了以下核心方法：

- `new()`：创建新的 Snowball 实例
- `record_successful_poll()`：记录成功的投票
- `record_unsuccessful_poll()`：记录不成功的投票
- `finalized()`：检查是否已达成共识

### Node 结构体

`node.rs` 中定义的 `Node` 结构体实现了一元 Snowball 节点：

```rust
pub struct Node {
    /// 共识参数
    pub parameters: crate::Parameters,
    
    /// Snowball 实例
    pub snowball: Snowball,
    
    /// 当前偏好的选择
    pub preference: Cell<Id>,
    
    /// 已决定的前缀位数
    pub decided_prefix: Cell<i64>,
    
    /// 与其他选择的共同前缀位数
    pub common_prefix: Cell<i64>,
    
    /// 是否需要重置
    pub should_reset: Cell<bool>,
    
    /// 子节点
    pub child: Option<Box<snowball::Node>>,
}
```

`Node` 提供了以下核心方法：

- `new()`：创建新的一元节点
- `preference()`：返回当前的偏好选择
- `decided_prefix()`：返回已决定的前缀位数
- `finalized()`：检查是否已达成共识
- `add()`：添加新的选择
- `record_poll()`：记录投票结果

## 设计模式

### 1. 组合模式

一元节点使用组合模式构建树形结构，每个节点可以包含一个子节点：

```rust
pub struct Node {
    // ...
    pub child: Option<Box<snowball::Node>>,
    // ...
}
```

这种设计允许递归处理复杂的决策树。

### 2. 状态模式

一元 Snowball 算法本质上是一个状态机，通过 `record_poll()` 方法更新状态：

```rust
pub fn record_poll(&mut self, votes: &Bag, reset: bool) -> (snowball::Node, bool) {
    // ...
    if reset {
        self.snowball.record_unsuccessful_poll();
    }
    
    // ...
    
    if preference_votes >= alpha {
        self.snowball.record_successful_poll();
        // ...
    } else {
        self.snowball.record_unsuccessful_poll();
        // ...
    }
    // ...
}
```

### 3. 转换模式

当检测到冲突时，一元节点可以转换为二元节点：

```rust
pub fn add(&mut self, new_choice: &Id) -> snowball::Node {
    // ...
    
    // 找到第一个不同的位
    let different_bit = bits::get_first_different_bit(&self.preference.get(), new_choice, 0);
    
    // 创建二元节点
    let binary_node = binary::node::Node::new(
        self.parameters.clone(),
        different_bit,
        self.preference.get(),
        self.decided_prefix.get(),
    );
    
    // ...
    
    snowball::Node::Binary(binary_node)
}
```

## 实现细节

### 共同前缀

一元节点跟踪与其他选择的共同前缀位数，这对于优化性能很重要：

```rust
pub struct Node {
    // ...
    pub common_prefix: Cell<i64>,
    // ...
}
```

`common_prefix` 字段允许节点跳过已知相同的位，直接检查可能有差异的位。

### 投票过程

投票过程是一元 Snowball 算法的核心，通过以下步骤实现：

1. 如果需要重置，调用 `record_unsuccessful_poll()`
2. 统计支持当前偏好的票数
3. 如果达到 alpha 阈值，记录成功的投票
4. 否则，记录不成功的投票
5. 更新子节点状态

```rust
pub fn record_poll(&mut self, votes: &Bag, reset: bool) -> (snowball::Node, bool) {
    if reset {
        self.snowball.record_unsuccessful_poll();
    }
    
    let preference = self.preference.get();
    let alpha = self.parameters.alpha;
    let preference_votes = votes.count(&preference);
    
    // ...
    
    if preference_votes >= alpha {
        self.snowball.record_successful_poll();
        // ...
    } else {
        self.snowball.record_unsuccessful_poll();
        // ...
    }
    
    // ...
}
```

### 子节点管理

当一元节点有子节点时，它会将投票传递给子节点：

```rust
pub fn record_poll(&mut self, votes: &Bag, reset: bool) -> (snowball::Node, bool) {
    // ...
    
    let mut successful = preference_votes >= alpha;
    
    if let Some(child) = &mut self.child {
        let common_prefix_usize = usize::try_from(self.common_prefix.get())
            .expect("common prefix should be non-negative");
        let filtered_votes = votes.filter(0, common_prefix_usize, &preference);
        
        let (polled_child, child_successful) = child.record_poll(&filtered_votes, reset);
        *child = Box::new(polled_child);
        
        successful = successful && child_successful;
    }
    
    // ...
}
```

## 使用示例

以下是使用一元 Snowball 节点的基本示例：

```rust
// 创建一元节点
let mut node = Node::new(
    crate::Parameters {
        k: 20,                // 采样大小
        alpha: 15,            // 法定人数大小
        beta_virtuous: 15,    // 良性决策阈值
        beta_rogue: 20,       // 恶意决策阈值
        ..Default::default()
    },
    initial_preference,       // 初始偏好
    0,                        // 已决定前缀
);

// 添加新的选择（如果有冲突，会转换为二元节点）
let snowball_node = node.add(&new_choice);

// 记录投票结果
let votes = Bag::new();
votes.add_count(&choice, count);
let (updated_node, successful) = node.record_poll(&votes, false);

// 检查是否已达成共识
if updated_node.finalized() {
    println!("共识已达成，最终选择: {}", updated_node.preference());
}
```

更多详细示例请参考 `node.rs` 文件中的测试用例。
