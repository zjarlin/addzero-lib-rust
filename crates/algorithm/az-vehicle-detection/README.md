# 车辆检测算法

独立车辆检测算法 crate，基于本地 COCO SSD MobileNet v1 ONNX 模型执行真实图片推理。

## 测试

```shell
cargo test -p az-vehicle-detection --test vehicle_detection -- --nocapture
```
