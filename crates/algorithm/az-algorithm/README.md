# 算法组件目录

算法组件目录与公共契约 crate。

## 功能

- 注册算法组件：人脸检测、人脸识别、人员检测、OCR 文字识别、火焰检测、安全帽检测、车辆检测、二维码识别、工人敲击计数
- 为组件提供稳定 code、中文名称、任务类型、目标对象、输入契约和输出契约
- 支持按 code、中文名称、任务类型和目标对象查询组件
- 提供可序列化 DTO，便于 API、CLI、admin UI 和推理 runtime 复用

具体算法实现按“一个算法一个 crate”拆分：

- `crates/algorithm/az-face-detection`
- `crates/algorithm/az-face-recognition`
- `crates/algorithm/az-person-detection`
- `crates/algorithm/az-ocr-text-recognition`
- `crates/algorithm/az-flame-detection`
- `crates/algorithm/az-safety-helmet-detection`
- `crates/algorithm/az-vehicle-detection`
- `crates/algorithm/az-qr-code-recognition`
- `crates/algorithm/az-worker-hit-counting`

多个图片算法叠加运行使用：

- `crates/algorithm/az-algorithm-pipeline`

`az-worker-hit-counting` 是纯视觉后处理算法：上游需要提供人员轨迹、接触点、目标类型和悬挂金属板响应评分。只有命中悬挂金属板且目标有足够响应才计入有效敲击；敲流水线台体边缘、支架或无响应目标会记录为无效候选。

## 用法

```rust
use az_algorithm::catalog::{AlgorithmTaskKind, algorithm_components_by_task};

let recognition_components: Vec<_> =
    algorithm_components_by_task(AlgorithmTaskKind::Recognition).collect();

assert_eq!(recognition_components.len(), 3);
assert_eq!(recognition_components[0].label, "人脸识别");
```

## 运行测试

目录契约测试：

```shell
cargo test -p az-algorithm
```

真实模型测试在各算法 crate 内运行，例如：

```shell
cargo test -p az-face-detection --test face_detection -- --nocapture
cargo test -p az-safety-helmet-detection --test safety_helmet_detection -- --nocapture
cargo test -p az-algorithm-pipeline --test image_pipeline -- --nocapture
```
