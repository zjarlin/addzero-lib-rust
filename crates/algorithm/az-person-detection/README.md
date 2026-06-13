# 人员检测算法

独立人员检测算法 crate，基于本地 COCO SSD MobileNet v1 ONNX 模型执行真实图片推理，并输出结构化人员框与画框图片。

## 输入

- 绝对图片路径：`detect_persons_from_path_with_options(...)`
- 图片二进制：`detect_persons_from_bytes_with_options(...)`
- base64 图片字符串：`detect_persons_from_base64_with_options(...)`
- 绝对视频路径：`detect_persons_in_video_from_path(...)`
- 实时视频帧：先创建 `PersonDetectionRunner`，再对每帧调用 `detect_rgb_image_with_output_dir(...)`

## 输出

测试默认输出到 workspace 下的 `target/az-algorithm-results/person_detection_*`：

- `source_input.jpg`：原始输入图副本
- `model_input_preview.png`：模型实际看到的 resize 输入
- `raw_outputs.json`：ONNX 原始输出摘要
- `detected_persons.json`：真实模型后处理得到的人员框
- `detected_persons.png`：画出人员框的图片

视频检测默认输出到调用方配置的目录：

- `source_input.mp4`：原始输入视频副本
- `extracted_frames/`：ffmpeg 抽帧
- `per_frame_detection/`：逐帧图片检测输出
- `annotated_frames/`：逐帧画框结果
- `frame_detections.json`：每帧人员框
- `annotated_persons.mp4`：标注后视频

实时视频管线中不要每帧调用 path/bytes/base64 入口重新加载模型。应在算法启动时创建一次：

```rust,no_run
use az_person_detection::logic_person_detection::assist::PersonDetectionRunner;
use az_person_detection::logic_person_detection::model::{
    PersonDetectionModelKind, PersonDetectionOptions,
};

# fn main() -> anyhow::Result<()> {
let mut runner = PersonDetectionRunner::new(PersonDetectionOptions {
    model_path: "/absolute/path/to/coco_ssd_mobilenet_v1_10.onnx".into(),
    model_kind: PersonDetectionModelKind::CocoSsdMobileNetV1,
    output_dir: "/absolute/path/to/default-output".into(),
    score_threshold: 0.5,
})?;

let frame_rgb = image::RgbImage::new(640, 480);
let result = runner.detect_rgb_image_with_output_dir(
    frame_rgb,
    "/absolute/path/to/output/frame_00001",
)?;
println!("{}", result.files.detected_persons_json.display());
# Ok(())
# }
```

说明：当前保留两个真实模型：

- `CocoSsdMobileNetV1`：默认图片检测模型，对常规直立人员图片较稳定。
- `Yolo11nCoco`：视频检测可选模型，适合继续做人员框验证；工业俯视、横向旋转、遮挡画面可能需要降低阈值或换成业务定制模型。

当前用户视频测试使用 YOLO11n 和 `0.01` 阈值，是因为该视频为旋转/俯视工位画面，默认 `0.25` 阈值会真实输出 0 个人员框。低阈值会带来误检，后续做敲击计数时应接入姿态/动作/ROI 定制模型，不应直接把低阈值人框当成敲击识别结果。

模型文件放在源码资源目录：

`crates/algorithm/az-person-detection/resources/models/coco_ssd_mobilenet_v1_10.onnx`

`crates/algorithm/az-person-detection/resources/models/yolo11n_coco.onnx`

## 测试

```shell
cargo test -p az-person-detection --test person_detection -- --nocapture
```
