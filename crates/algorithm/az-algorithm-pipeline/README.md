# 多算法叠加编排

多算法叠加编排 crate。

单算法实现仍然保持一个算法一个 crate；本 crate 只负责编排：

- 接收同一张图片绝对路径
- 接收启用算法列表
- 调用对应算法 crate
- 将每个算法的输出放在同一次任务目录的子目录下
- 写出总汇总 `pipeline_results.json`

例如同时启用人脸检测和安全帽检测：

```rust,no_run
use az_algorithm_pipeline::logic_algorithm_pipeline::assist::run_image_pipeline_from_path;
use az_algorithm_pipeline::logic_algorithm_pipeline::model::{
    ImageAlgorithmKind, ImagePipelineOptions,
};

# fn main() -> az_algorithm_pipeline::error::AlgorithmPipelineResult<()> {
let result = run_image_pipeline_from_path(
    "/absolute/path/to/input.jpg",
    &ImagePipelineOptions {
        algorithms: vec![
            ImageAlgorithmKind::FaceDetection,
            ImageAlgorithmKind::SafetyHelmetDetection,
        ],
        output_dir: "/absolute/path/to/output".into(),
    },
)?;
println!("{}", result.summary_file.display());
# Ok(())
# }
```

抽烟检测这类新增算法应先作为 `az-smoking-detection` 独立 crate 接入真实模型和测试，再在 pipeline 中增加一个分支。
