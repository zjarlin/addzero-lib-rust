# 火焰与烟雾检测算法

独立火焰检测算法 crate，基于本地 YOLOv8n fire/smoke ONNX 模型执行真实图片检测，并输出检测框 JSON 与标注图。

## 模型

- 来源：`fiacecson20/cctv-ai-fire-smoke`
- 文件：`best.onnx`
- 本地文件：`resources/models/fire_smoke_yolov8n.onnx`
- 输入：`(1, 3, 320, 320)` float32 RGB，归一化到 `[0, 1]`
- 类别：`fire`, `smoke`
- 许可边界：模型卡标注权重 MIT；其 base model 为 Ultralytics YOLOv8，模型卡提示 hosted service 场景可能需要 AGPL 合规。

## 输出

- `source_input.jpg`：原始输入图副本
- `model_input_preview.png`：模型输入尺寸预览图，带检测框
- `raw_outputs.json`：ONNX 原始输出摘要
- `detected_flames.json`：后处理后的 fire/smoke 检测框
- `detected_flames.png`：原图尺寸标注图

## 测试

```shell
cargo test -p az-flame-detection --test flame_detection -- --nocapture
```
