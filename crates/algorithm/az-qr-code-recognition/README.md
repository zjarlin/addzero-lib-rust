# 二维码识别算法

独立二维码识别算法 crate，基于纯 Rust `rqrr` 解码器识别图片中的二维码。

## 输入

- `logic_qr_code_recognition::assist::decode_qr_codes_from_path(...)`：传图片绝对路径

## 输出

测试会写出：

`target/az-algorithm-results/qr_code_recognition/decoded_qr.png`

## 测试

```shell
cargo test -p az-qr-code-recognition --test qr_code_recognition -- --nocapture
```
