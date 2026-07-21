# az-math

数学工具库，提供矩阵操作和线性规划求解能力。

## 功能

### 矩阵工具
- 创建、转置、访问和批量更新矩阵元素
- 类型安全的矩阵校验（矩形性检查、边界检查）
- 向量生成与索引分组工具
- 支持任意类型矩阵的泛型操作

### 线性规划
- 基于活动集枚举法的线性规划求解器
- 支持最小化/最大化目标函数
- 支持等式、不等式约束条件
- 面向运输/配置问题的便捷求解接口（带可选价格矩阵）

### 数值精度
- 默认浮点容差 `1e-8`
- 默认最大枚举组合数 500,000

## 安装

在 `Cargo.toml` 中添加：

```toml
[dependencies]
az-math = { path = "../math" }       # workspace 内部引用
# 或发布后：
# az-math = "0.1"                       # crates.io 引用
```

## 用法

```rust
use az_math::optimization::{
    ConstraintRelation, GoalType, LinearConstraint, LinearObjective,
    LinearProgrammingProblem, create_matrix, transpose_matrix,
};

fn main() -> anyhow::Result<()> {
// 矩阵操作
let matrix = create_matrix::<f64>(3, 4, 0.0);
let transposed = transpose_matrix(&matrix)?;

// 线性规划
let objective = LinearObjective::new(vec![1.0, 2.0], 0.0);
let constraints = vec![
    LinearConstraint::new(vec![1.0, 0.0], ConstraintRelation::LessOrEqual, 4.0),
    LinearConstraint::new(vec![0.0, 1.0], ConstraintRelation::LessOrEqual, 6.0),
];
let problem = LinearProgrammingProblem::new(GoalType::Minimize, objective, constraints);
let solution = problem.solve()?;
Ok(())
}
```

## 依赖的 crates

- `anyhow` - 统一错误返回与上下文补充
