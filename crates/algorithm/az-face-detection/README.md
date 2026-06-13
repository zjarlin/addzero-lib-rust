# 人脸检测算法

独立人脸检测算法 crate，基于本地 SCRFD ONNX 模型执行真实推理，并输出结构化人脸框与画框图片。

## 输入

- 绝对图片路径：`detect_faces_from_path_with_options(...)`
- 图片二进制：`detect_faces_from_bytes_with_options(...)`
- base64 图片字符串：`detect_faces_from_base64_with_options(...)`
- 实时视频帧：先创建 `FaceDetectionRunner`，再对每帧调用 `detect_rgb_image_with_output_dir(...)`

## 实时视频用法

实时视频管线中不要每帧调用 path/bytes/base64 入口重新加载模型。应在算法启动时创建一次：

```rust,no_run
use az_face_detection::logic_face_detection::assist::FaceDetectionRunner;
use az_face_detection::logic_face_detection::model::FaceDetectionOptions;

# fn main() -> anyhow::Result<()> {
let mut runner = FaceDetectionRunner::new(FaceDetectionOptions {
    model_path: "/absolute/path/to/face_detection_scrfd_500m.onnx".into(),
    output_dir: "/absolute/path/to/default-output".into(),
    score_threshold: 0.5,
    nms_threshold: 0.4,
})?;

let frame_rgb = image::RgbImage::new(640, 480);
let result = runner.detect_rgb_image_with_output_dir(
    frame_rgb,
    "/absolute/path/to/output/frame_00001",
)?;
println!("{}", result.files.detected_faces_json.display());
# Ok(())
# }
```

## 输出

测试默认输出到 workspace 下的 `target/az-algorithm-results/face_detection_*`：

- `source_input.jpg`：原始输入图副本
- `model_input_preview.png`：模型实际看到的 resize 输入
- `raw_outputs.json`：ONNX 原始输出摘要
- `detected_faces.json`：真实模型后处理得到的人脸框
- `detected_faces.png`：画出人脸框的图片

模型文件放在源码资源目录：

`crates/algorithm/az-face-detection/resources/models/face_detection_scrfd_500m.onnx`

## 测试

```shell
cargo test -p az-face-detection --test face_detection -- --nocapture
```
