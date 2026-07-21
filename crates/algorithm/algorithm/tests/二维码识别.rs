use std::path::{Path, PathBuf};

use anyhow::Context;
use az_algorithm::components::qr_code_recognition::assist::decode_qr_codes_from_path;
use az_algorithm::components::qr_code_recognition::model::ImagePoint;
use image::{Rgb, RgbImage};
use imageproc::drawing::draw_line_segment_mut;

fn workspace_root() -> PathBuf {
    std::fs::canonicalize(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../.."))
        .expect("workspace 根目录必须存在")
}

fn fixture_path(file_name: &str) -> PathBuf {
    std::fs::canonicalize(
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures/qr_code_recognition/input")
            .join(file_name),
    )
    .expect("测试输入图片必须存在")
}

fn output_dir() -> anyhow::Result<PathBuf> {
    let dir = workspace_root()
        .join("target/az-algorithm-results")
        .join("qr_code_recognition");
    std::fs::create_dir_all(&dir)
        .with_context(|| format!("failed to create output dir `{}`", dir.display()))?;
    Ok(dir)
}

#[expect(clippy::dbg_macro, reason = "用户要求测试输出输入、输出的绝对路径")]
fn save_qr_output_image(image_path: &Path, bounds: &[[ImagePoint; 4]]) -> anyhow::Result<PathBuf> {
    let output_path = output_dir()?.join("decoded_qr.png");
    dbg!(&image_path);
    dbg!(&output_path);

    let mut image = image::open(image_path)?.to_rgb8();
    for corners in bounds {
        draw_polygon(&mut image, corners, Rgb([255, 0, 0]));
    }
    image.save(&output_path)?;
    Ok(output_path)
}

fn draw_polygon(image: &mut RgbImage, corners: &[ImagePoint; 4], color: Rgb<u8>) {
    for index in 0..corners.len() {
        let from = &corners[index];
        let to = &corners[(index + 1) % corners.len()];
        draw_line_segment_mut(
            image,
            (from.x as f32, from.y as f32),
            (to.x as f32, to.y as f32),
            color,
        );
    }
}

#[test]
fn qr_code_recognition_should_decode_real_image_and_write_output_image() -> anyhow::Result<()> {
    // 输入图片：crates/algorithm/algorithm/tests/fixtures/qr_code_recognition/input/qr_code.png
    // 输出图片：target/az-algorithm-results/qr_code_recognition/decoded_qr.png
    let input_path = fixture_path("qr_code.png");
    let payload = "az-algorithm://真实二维码测试";

    let results = decode_qr_codes_from_path(&input_path)?;
    let bounds = results
        .iter()
        .map(|result| result.bounds.clone())
        .collect::<Vec<_>>();
    let output_path = save_qr_output_image(&input_path, &bounds)?;

    // 关键断言：验证真实解码内容，而不是只验证函数返回 Ok。
    assert_eq!(results[0].payload, payload);
    assert!(
        output_path.is_file(),
        "二维码输出图必须写入 {}",
        output_path.display()
    );
    Ok(())
}
