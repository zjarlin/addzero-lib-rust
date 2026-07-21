# 算法组件目录

算法组件目录与公共契约 crate。

## 功能

- 注册算法组件：人脸检测、人脸识别、人员检测、OCR 文字识别、火焰检测、安全帽检测、车辆检测、二维码识别、工人敲击计数
- 为组件提供稳定 code、中文名称、任务类型、目标对象、输入契约和输出契约
- 支持按 code、中文名称、任务类型和目标对象查询组件
- 提供可序列化 DTO，便于 API、CLI、admin UI 和推理 runtime 复用

具体算法实现按“一个算法一个 crate”拆分：

- `crates/algorithm/face-detection`
- `crates/algorithm/face-recognition`
- `crates/algorithm/person-detection`
- `crates/algorithm/ocr-text-recognition`
- `crates/algorithm/flame-detection`
- `crates/algorithm/safety-helmet-detection`
- `crates/algorithm/vehicle-detection`
- `crates/algorithm/qr-code-recognition`
- `crates/algorithm/worker-hit-counting`

多个图片算法叠加运行使用：

- `crates/algorithm/algorithm-pipeline`

`az-worker-hit-counting` 是纯视觉后处理算法：上游需要提供人员轨迹、接触点、目标类型和悬挂金属板响应评分。只有命中悬挂金属板且目标有足够响应才计入有效敲击；敲流水线台体边缘、支架或无响应目标会记录为无效候选。

## 用法

```rust
use az_algorithm::catalog::model::AlgorithmTaskKind;
use az_algorithm::catalog::query::algorithm_components_by_task;

let mut recognition_components: Vec<_> =
    algorithm_components_by_task(AlgorithmTaskKind::Recognition).collect();
recognition_components.sort_by_key(|c| c.label);

assert_eq!(recognition_components.len(), 3);
assert_eq!(recognition_components[0].label, "OCR文字识别");
```

应用运行时通过 Rudi 注入统一服务：

```rust
use az_algorithm::di::{create_algorithm_context, resolve_algorithm_catalog};

let mut context = create_algorithm_context();
let catalog = resolve_algorithm_catalog(&mut context)?;
let components = catalog.components();

assert_eq!(components.len(), 9);
# Ok::<(), anyhow::Error>(())
```

`AlgorithmCatalogServiceRef`、`ImagePipelineServiceRef` 和
`VideoPipelineServiceRef` 都是 singleton。每次任务的输入、输出目录、阈值和可变算法实例
仍通过方法参数传入，避免把请求状态放进全局容器。

## 运行测试

目录契约测试：

```shell
cargo test -p az-algorithm
```

真实模型测试在 `az-algorithm` 内运行，例如：

```shell
cargo test -p az-algorithm --test 人脸检测 -- --nocapture
cargo test -p az-algorithm --test 安全帽检测 -- --nocapture
cargo test -p az-algorithm --test 图片流水线 -- --nocapture
```
