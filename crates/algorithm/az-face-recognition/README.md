# 人脸识别算法

独立人脸识别算法 crate，基于本地 ArcFace ResNet100 int8 ONNX 模型执行真实图片推理。

## 输入

- `logic_face_recognition::run_face_recognition_from_path(...)`：传图片绝对路径

## 输出

默认输出目录：

`target/az-algorithm-results/face_recognition`

每次运行会输出：

- `source_input.jpg`
- `model_input_preview.png`
- `raw_outputs.json`

## 测试

```shell
cargo test -p az-face-recognition --test face_recognition -- --nocapture
```
