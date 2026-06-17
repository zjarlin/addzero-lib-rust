use std::path::PathBuf;
use std::process::Command;

use az_algorithm::video_pipeline::alarm::{AlarmOutputTarget, AlarmRule, plan_alarm_actions};
use az_algorithm::video_pipeline::model::{VideoAlgorithmEvent, VideoAlgorithmFrameResult};
use az_algorithm::video_pipeline::source::ffmpeg::decode_video_frames_with_ffmpeg;
use az_algorithm::video_pipeline::source::model::{FfmpegFrameDecodeOptions, FfmpegVideoSource};
use serde_json::json;

fn workspace_root() -> PathBuf {
    std::fs::canonicalize(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../.."))
        .expect("workspace 根目录必须存在")
}

fn fixture_image_path() -> PathBuf {
    std::fs::canonicalize(
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures/safety_helmet_detection/input/safety_helmet.jpg"),
    )
    .expect("安全帽测试图片必须存在")
}

fn output_dir(name: &str) -> PathBuf {
    workspace_root()
        .join("target/az-algorithm-results")
        .join(name)
}

fn ffmpeg_path() -> Option<PathBuf> {
    [
        "/opt/homebrew/bin/ffmpeg",
        "/usr/local/bin/ffmpeg",
        "ffmpeg",
    ]
    .into_iter()
    .map(PathBuf::from)
    .find(|path| {
        if path.components().count() > 1 {
            path.is_file()
        } else {
            Command::new(path)
                .arg("-version")
                .output()
                .is_ok_and(|output| output.status.success())
        }
    })
}

#[test]
fn ffmpeg_decoder_should_decode_short_video_to_rgb_frame() -> anyhow::Result<()> {
    let Some(ffmpeg) = ffmpeg_path() else {
        eprintln!("跳过 ffmpeg 成功路径测试：当前环境未找到 ffmpeg");
        return Ok(());
    };
    let output_dir = output_dir("video_source_decode");
    if output_dir.exists() {
        std::fs::remove_dir_all(&output_dir)?;
    }
    std::fs::create_dir_all(&output_dir)?;
    let video_path = output_dir.join("one_frame.mp4");
    let output = Command::new(&ffmpeg)
        .args([
            "-y",
            "-loop",
            "1",
            "-i",
            &fixture_image_path().display().to_string(),
            "-vf",
            "scale=320:240",
            "-frames:v",
            "1",
            "-pix_fmt",
            "yuv420p",
            &video_path.display().to_string(),
        ])
        .output()?;
    if !output.status.success() {
        anyhow::bail!(
            "ffmpeg 生成测试视频失败：{}",
            String::from_utf8_lossy(&output.stderr)
        );
    }

    let frames = decode_video_frames_with_ffmpeg(&FfmpegFrameDecodeOptions {
        ffmpeg_path: ffmpeg,
        source: FfmpegVideoSource::File(video_path),
        source_fps: 25.0,
        sample_fps: None,
        max_frames: Some(1),
        width: 320,
        height: 240,
    })?;

    assert_eq!(frames.len(), 1);
    assert_eq!((frames[0].width, frames[0].height), (320, 240));
    assert_eq!(frames[0].rgb.len(), 320 * 240 * 3);
    Ok(())
}

#[test]
fn ffmpeg_decoder_should_reject_missing_video_file() {
    let error = decode_video_frames_with_ffmpeg(&FfmpegFrameDecodeOptions {
        ffmpeg_path: PathBuf::from("ffmpeg"),
        source: FfmpegVideoSource::File(workspace_root().join("target/missing-video.mp4")),
        source_fps: 25.0,
        sample_fps: Some(1.0),
        max_frames: Some(1),
        width: 640,
        height: 480,
    })
    .expect_err("缺失视频文件必须被拒绝");

    assert!(
        error.to_string().contains("missing"),
        "错误信息应说明视频文件缺失：{error:#}"
    );
}

#[test]
fn alarm_planner_should_map_events_to_all_output_targets() -> anyhow::Result<()> {
    let frame_results = vec![VideoAlgorithmFrameResult {
        algorithm_code: "safety_helmet_detection".to_owned(),
        frame_index: 7,
        timestamp_ms: 280,
        detections: Vec::new(),
        events: vec![VideoAlgorithmEvent {
            event_code: "safety_helmet_missing".to_owned(),
            score: 0.86,
            message: "检测到未佩戴安全帽".to_owned(),
            extra: json!({"class_index": 1}),
        }],
        raw_json: serde_json::Value::Null,
    }];
    let rules = vec![AlarmRule {
        code: "ppe_alarm".to_owned(),
        event_code: "safety_helmet_missing".to_owned(),
        min_score: 0.5,
        targets: vec![
            AlarmOutputTarget::Mqtt {
                topic: "site/alarm/ppe".to_owned(),
                qos: 1,
            },
            AlarmOutputTarget::HttpPost {
                url: "http://127.0.0.1:8080/alarm".to_owned(),
            },
            AlarmOutputTarget::RelayPulse {
                channel: 1,
                duration_ms: 500,
            },
            AlarmOutputTarget::Rs485Write {
                port: "/dev/ttyUSB0".to_owned(),
                frame_hex: "01 05 00 00 FF 00".to_owned(),
            },
        ],
    }];

    let actions = plan_alarm_actions(&frame_results, &rules)?;

    assert_eq!(actions.len(), 4);
    assert!(
        actions
            .iter()
            .all(|action| action.event_code == "safety_helmet_missing" && action.score == 0.86),
        "每个输出目标都必须保留原始事件信息"
    );
    Ok(())
}

#[test]
fn alarm_planner_should_reject_invalid_rs485_frame() {
    let error = plan_alarm_actions(
        &[],
        &[AlarmRule {
            code: "bad_485".to_owned(),
            event_code: "fire_detected".to_owned(),
            min_score: 0.5,
            targets: vec![AlarmOutputTarget::Rs485Write {
                port: "/dev/ttyUSB0".to_owned(),
                frame_hex: "not-hex".to_owned(),
            }],
        }],
    )
    .expect_err("非法 485 帧必须被拒绝");

    assert!(
        error.to_string().contains("frame_hex"),
        "错误信息应指向非法 frame_hex：{error:#}"
    );
}
