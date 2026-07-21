//! 基于 `ffmpeg` 的视频源解码。

use std::io::Read;
use std::process::{Command, Stdio};

use anyhow::bail;
use image::RgbImage;

use crate::video_pipeline::model::VideoFrame;
use crate::video_pipeline::source::model::{FfmpegFrameDecodeOptions, FfmpegVideoSource};

/// 使用 `ffmpeg` 将视频源解码为 RGB 帧。
///
/// 该函数面向短片段、抽样验证和网关侧有限窗口处理。实时常驻 RTSP/摄像头任务应在外层
/// 控制窗口大小或改用流式消费，避免无界读取。
///
/// # Errors
/// 当配置无效、`ffmpeg` 启动失败、输出帧尺寸不匹配或进程失败时返回错误。
pub fn decode_video_frames_with_ffmpeg(
    options: &FfmpegFrameDecodeOptions,
) -> anyhow::Result<Vec<VideoFrame>> {
    validate_options(options)?;
    let mut command = Command::new(&options.ffmpeg_path);
    configure_input(&mut command, &options.source);
    command.args(["-i", &options.source.input_arg()]);
    if let Some(sample_fps) = options.sample_fps {
        command.args(["-vf", &format!("fps={sample_fps}")]);
    }
    command.args(["-f", "rawvideo", "-pix_fmt", "rgb24", "-"]);
    command.stdout(Stdio::piped());
    command.stderr(Stdio::piped());

    let mut child = command
        .spawn()
        .map_err(|source| path_error(options.ffmpeg_path.display().to_string(), source))?;
    let mut stdout = child
        .stdout
        .take()
        .ok_or_else(|| anyhow::anyhow!("ffmpeg stdout pipe was not available"))?;
    let mut frames = Vec::new();
    let frame_size = options.width as usize * options.height as usize * 3;
    let mut buffer = vec![0_u8; frame_size];
    loop {
        match stdout.read_exact(&mut buffer) {
            Ok(()) => {
                let rgb = RgbImage::from_raw(options.width, options.height, buffer.clone())
                    .ok_or_else(|| anyhow::anyhow!("ffmpeg produced an invalid RGB frame"))?;
                let frame_index = frames.len() as u64;
                frames.push(VideoFrame {
                    frame_index,
                    timestamp_ms: frame_timestamp_ms(frame_index, effective_fps(options)),
                    width: options.width,
                    height: options.height,
                    rgb,
                });
                if options
                    .max_frames
                    .is_some_and(|max_frames| frames.len() >= max_frames)
                {
                    break;
                }
            }
            Err(error) if error.kind() == std::io::ErrorKind::UnexpectedEof => {
                break;
            }
            Err(error) => {
                return Err(path_error(options.source.input_arg(), error));
            }
        }
    }
    drop(stdout);

    let output = child
        .wait_with_output()
        .map_err(|source| path_error(options.ffmpeg_path.display().to_string(), source))?;
    if !output.status.success() && frames.is_empty() {
        bail!(
            "ffmpeg failed with status {}\nstderr:\n{}",
            output.status,
            String::from_utf8_lossy(&output.stderr)
        );
    }
    if frames.is_empty() {
        bail!("ffmpeg decoded zero frames from `{}`", options.source.input_arg());
    }

    Ok(frames)
}

fn configure_input(command: &mut Command, source: &FfmpegVideoSource) {
    match source {
        FfmpegVideoSource::File(_) | FfmpegVideoSource::Url(_) => {}
        FfmpegVideoSource::CameraDevice(_) => {
            if cfg!(target_os = "macos") {
                command.args(["-f", "avfoundation"]);
            } else if cfg!(target_os = "linux") {
                command.args(["-f", "v4l2"]);
            } else if cfg!(target_os = "windows") {
                command.args(["-f", "dshow"]);
            }
        }
    }
}

fn validate_options(options: &FfmpegFrameDecodeOptions) -> anyhow::Result<()> {
    if options.width == 0 || options.height == 0 {
        bail!("ffmpeg frame decode width and height must be greater than zero");
    }
    validate_positive_fps("source_fps", options.source_fps)?;
    if let Some(sample_fps) = options.sample_fps {
        validate_positive_fps("sample_fps", sample_fps)?;
    }
    if options.max_frames == Some(0) {
        bail!("ffmpeg frame decode max_frames must be greater than zero when set");
    }
    if let FfmpegVideoSource::Url(url) | FfmpegVideoSource::CameraDevice(url) = &options.source
        && url.trim().is_empty()
    {
        bail!("ffmpeg video source cannot be blank");
    }
    if let FfmpegVideoSource::File(path) = &options.source
        && !path.is_file()
    {
        bail!("ffmpeg video file `{}` is missing", path.display());
    }
    Ok(())
}

fn validate_positive_fps(name: &str, fps: f32) -> anyhow::Result<()> {
    if !fps.is_finite() || fps <= 0.0 {
        bail!("{name} must be a positive finite number, got {fps}");
    }
    Ok(())
}

fn effective_fps(options: &FfmpegFrameDecodeOptions) -> f32 {
    options.sample_fps.unwrap_or(options.source_fps)
}

fn frame_timestamp_ms(frame_index: u64, fps: f32) -> u64 {
    ((frame_index as f32 * 1_000.0) / fps).round() as u64
}

fn path_error(path: String, source: std::io::Error) -> anyhow::Error {
    anyhow::anyhow!("ffmpeg I/O error at `{path}`: {source}")
}
