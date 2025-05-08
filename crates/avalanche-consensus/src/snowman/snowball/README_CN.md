# Snowball 共识算法实现

本目录包含 Avalanche 共识协议中 Snowball 算法的 Rust 实现，这是 Avalanche 共识机制的核心组件。

## 目录

1. [概述](#概述)
2. [目录结构](#目录结构)
3. [核心组件](#核心组件)
4. [设计模式](#设计模式)
5. [实现细节](#实现细节)
6. [使用示例](#使用示例)

## 概述

Snowball 是 Avalanche 共识协议的核心算法，它是一种基于采样的概率共识协议，具有高吞吐量、快速终结和能源效率的特点。Snowball 算法通过随机采样其他节点的偏好来达成共识，随着投票轮数的增加，系统逐渐收敛到一个共同的决策。

本目录实现了 Snowball 算法的几个变体，包括：

- **Unary Snowball**：处理单一选择的基本 Snowball 实现
- **Binary Snowball**：处理二元选择的 Snowball 实现
- **Tree Snowball**：使用修改后的 Patricia 字典树处理多个选择的 Snowball 实现

## 目录结构

- `mod.rs`：模块入口，定义公共接口和 `Node` 枚举
- `tree.rs`：实现 Snowball 树，用于处理多个选择
- `binary/`：二元 Snowball 实现
  - `mod.rs`：二元 Snowball 模块入口
  - `node.rs`：二元 Snowball 节点实现
- `unary/`：一元 Snowball 实现
  - `mod.rs`：一元 Snowball 模块入口
  - `node.rs`：一元 Snowball 节点实现

## 核心组件

### Node 枚举

`mod.rs` 中定义的 `Node` 枚举是 Snowball 实现的核心，它统一了一元和二元节点的接口：

```rust
pub enum Node {
    Unary(unary::node::Node),
    Binary(binary::node::Node),
}
```

`Node` 提供了以下核心方法：

- `preference()`：返回当前节点的偏好选择
- `decided_prefix()`：返回已决定的前缀位数
- `finalized()`：检查是否已达成共识
- `add()`：添加新的选择
- `record_poll()`：记录投票结果

### Tree 结构体

`tree.rs` 中的 `Tree` 结构体实现了基于修改后的 Patricia 字典树的 Snowball 算法：

```rust
pub struct Tree {
    pub parameters: crate::Parameters,
    pub node: Box<snowball::Node>,
    pub should_reset: Cell<bool>,
}
```

`Tree` 提供了以下核心方法：

- `new()`：创建新的 Snowball 树
- `preference()`：返回当前的偏好选择
- `decided_prefix()`：返回已决定的前缀位数
- `finalized()`：检查是否已达成共识
- `add()`：添加新的选择
- `record_poll()`：记录投票结果
- `record_unsuccessful_poll()`：记录不成功的投票

### Binary 节点

`binary/node.rs` 中的 `Node` 结构体实现了二元 Snowball 节点：

```rust
pub struct Node {
    pub parameters: crate::Parameters,
    pub snowball: Snowball,
    pub bit: usize,
    pub preference: Cell<Id>,
    pub decided_prefix: Cell<i64>,
    pub child0: Option<Box<snowball::Node>>,
    pub child1: Option<Box<snowball::Node>>,
    pub should_reset: Cell<bool>,
}
```

### Unary 节点

`unary/node.rs` 中的 `Node` 结构体实现了一元 Snowball 节点：

```rust
pub struct Node {
    pub parameters: crate::Parameters,
    pub snowball: Snowball,
    pub preference: Cell<Id>,
    pub decided_prefix: Cell<i64>,
    pub common_prefix: Cell<i64>,
    pub should_reset: Cell<bool>,
    pub child: Option<Box<snowball::Node>>,
}
```

## 设计模式

### 1. 组合模式

Snowball 实现使用组合模式构建树形结构，每个节点可以包含子节点：

```rust
pub struct Node {
    // ...
    pub child0: Option<Box<snowball::Node>>,
    pub child1: Option<Box<snowball::Node>>,
    // ...
}
```

这种设计允许递归处理复杂的决策树。

### 2. 策略模式

通过 `Node` 枚举，系统可以在运行时选择不同的 Snowball 实现：

```rust
pub enum Node {
    Unary(unary::node::Node),
    Binary(binary::node::Node),
}
```

这种设计允许根据具体情况选择最合适的算法变体。

### 3. 状态模式

Snowball 算法本质上是一个状态机，通过 `record_poll()` 方法更新状态：

```rust
pub fn record_poll(&mut self, votes: &Bag) -> bool {
    // ...
    let (polled_node, successful) = self.node.record_poll(&filtered_votes, self.should_reset.get());
    self.node = Box::new(polled_node);
    self.should_reset.set(false);
    successful
}
```

### 4. 访问者模式

通过 `Display` trait 的实现，可以以树形结构可视化 Snowball 树：

```rust
impl std::fmt::Display for Tree {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // ...
    }
}
```

## 实现细节

### Patricia 字典树

Snowball 树使用修改后的 Patricia 字典树来组织选择。Patricia 字典树是一种压缩前缀树，可以高效地存储和查找具有共同前缀的字符串。在 Snowball 中，它用于组织和比较选择的二进制表示。

关键特性：

- **决定前缀**：跟踪已经达成共识的位数
- **位测试**：根据特定位的值决定走哪条路径
- **压缩路径**：跳过没有分支的位，提高效率

### 投票过程

投票过程是 Snowball 算法的核心，通过以下步骤实现：

1. 过滤无效投票（与已决定前缀不符的投票）
2. 将有效投票传递给根节点
3. 根节点根据投票结果更新状态
4. 如果达到阈值，节点被标记为已终结

```rust
pub fn record_poll(&mut self, votes: &Bag) -> bool {
    let decided_prefix = self.decided_prefix();
    let decided_prefix_usize = usize::try_from(decided_prefix).expect("decided prefix should be non-negative");
    let filtered_votes = votes.filter(0, decided_prefix_usize, &self.preference());
    
    let (polled_node, successful) = self.node.record_poll(&filtered_votes, self.should_reset.get());
    self.node = Box::new(polled_node);
    self.should_reset.set(false);
    
    successful
}
```

### 重置机制

为了优化性能，Snowball 实现使用 `should_reset` 标志来避免不必要的树遍历：

```rust
pub fn record_unsuccessful_poll(&self) {
    self.should_reset.set(true);
}
```

当一个节点没有获得足够的票数时，它及其所有子节点都需要重置。通过 `should_reset` 标志，可以延迟重置操作，直到下一次访问该子树。

## 使用示例

以下是使用 Snowball 树的基本示例：

```rust
// 创建 Snowball 树
let mut tree = Tree::new(
    crate::Parameters {
        k: 20,                // 采样大小
        alpha: 15,            // 法定人数大小
        beta_virtuous: 15,    // 良性决策阈值
        beta_rogue: 20,       // 恶意决策阈值
        ..Default::default()
    },
    initial_preference,
);

// 添加新的选择
tree.add(&new_choice);

// 记录投票结果
let votes = Bag::new();
votes.add_count(&choice1, count1);
votes.add_count(&choice2, count2);
let successful = tree.record_poll(&votes);

// 检查是否已达成共识
if tree.finalized() {
    println!("共识已达成，最终选择: {}", tree.preference());
}
```

更多详细示例请参考 `tree.rs` 文件中的测试用例。
