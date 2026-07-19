//! 二维码识别辅助函数。

use std::path::Path;

use anyhow::Context;

use crate::components::qr_code_recognition::model::{ImagePoint, QrCodeRecognition};

/// 解码图片文件中全部二维码。
///
/// # Errors
/// 当图片读取失败时返回错误。
pub fn decode_qr_codes_from_path(
    path: impl AsRef<Path>,
) -> anyhow::Result<Vec<QrCodeRecognition>> {
    let path = path.as_ref();
    let image = image::open(path)
        .with_context(|| format!("failed to open QR code image at `{}`", path.display()))?
        .to_luma8();
    Ok(decode_qr_codes_from_luma8(image))
}

/// 解码灰度图中全部二维码。
#[must_use]
pub fn decode_qr_codes_from_luma8(image: image::GrayImage) -> Vec<QrCodeRecognition> {
    let mut prepared = rqrr::PreparedImage::prepare(image);
    let grids = prepared.detect_grids();

    grids
        .into_iter()
        .filter_map(|grid| {
            let (metadata, payload) = grid.decode().ok()?;
            Some(QrCodeRecognition {
                payload,
                version: metadata.version.0,
                ecc_level: metadata.ecc_level,
                mask: metadata.mask,
                bounds: grid.bounds.map(|point| ImagePoint {
                    x: point.x,
                    y: point.y,
                }),
            })
        })
        .collect()
}
