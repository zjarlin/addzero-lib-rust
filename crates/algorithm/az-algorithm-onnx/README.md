# 算法 ONNX 图片推理

算法 crate 共享的本地 ONNX 图片推理基础库。

这个 crate 不代表一个具体算法，只负责可复用的运行时能力：

- 加载本地 ONNX 模型
- 按模型声明的输入形状 resize 图片
- 生成 `f32` 或 `u8` 输入张量
- 执行真实 ONNX Runtime 推理
- 写出 `source_input.jpg`、`model_input_preview.png`、`raw_outputs.json`

具体算法仍然放在独立 crate 中，例如 `az-face-recognition`、`az-person-detection`、`az-ocr-text-recognition`。
