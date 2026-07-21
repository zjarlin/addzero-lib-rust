use az_algorithm::components::worker_hit_counting::assist::annotate_worker_hits_video_to_path;
use std::path::{Path, PathBuf};

const INPUT_VIDEO_URL: &str = "/Users/zjarlin/Library/Containers/com.tencent.xinWeChat/Data/Documents/xwechat_files/wxid_ofjmj34otla312_0d1f/msg/video/2026-06/2fc2accdebf728f085c2ba75a6eff64a_raw.mp4";
const OUTPUT_VIDEO_URL: &str =
    "target/az-algorithm-results/worker-hit-counting/annotated_worker_hits.mp4";

#[test]
#[ignore = "需要真实输入视频、ffmpeg 和 ONNX 推理；默认测试只编译该两参入口"]
fn worker_hit_counting_should_accept_input_video_url_and_output_video_url() -> anyhow::Result<()> {
    let input_video_url = PathBuf::from(INPUT_VIDEO_URL);
    if !input_video_url.is_file() {
        eprintln!(
            "跳过工人敲击计数测试，输入视频不存在：{}",
            input_video_url.display()
        );
        return Ok(());
    }

    let output_video_url = workspace_root().join(OUTPUT_VIDEO_URL);
    let output = annotate_worker_hits_video_to_path(&input_video_url, &output_video_url)?;

    assert_eq!(output, output_video_url);
    assert_existing_file(&output);
    Ok(())
}

fn workspace_root() -> PathBuf {
    std::fs::canonicalize(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../.."))
        .expect("workspace 根目录必须存在")
}

fn assert_existing_file(path: &Path) {
    assert!(path.is_file(), "输出视频必须存在：{}", path.display());
}
