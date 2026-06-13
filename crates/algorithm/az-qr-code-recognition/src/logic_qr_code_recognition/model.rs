//! 二维码识别公开模型。

/// 二维码识别稳定算法 code。
pub const ALGORITHM_CODE: &str = "qr_code_recognition";

/// 图片像素坐标点。
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
pub struct ImagePoint {
    /// 横向坐标。
    pub x: i32,
    /// 纵向坐标。
    pub y: i32,
}

/// 单个二维码解码结果。
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
pub struct QrCodeRecognition {
    /// 解码出的载荷文本。
    pub payload: String,
    /// 解码器报告的二维码版本。
    pub version: usize,
    /// 解码器报告的纠错等级。
    pub ecc_level: u16,
    /// 解码器报告的二维码 mask。
    pub mask: u16,
    /// 四个角点，顺序为左上、右上、右下、左下。
    pub bounds: [ImagePoint; 4],
}
