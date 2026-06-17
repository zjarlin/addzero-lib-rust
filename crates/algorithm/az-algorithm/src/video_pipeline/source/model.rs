//! 视频源解码配置模型。

use std::path::PathBuf;

/// `ffmpeg` 可读取的视频源。
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum FfmpegVideoSource {
    /// 本地视频文件。
    File(PathBuf),
    /// RTSP、HTTP-FLV、HLS 等 URL。
    Url(String),
    /// 摄像头或采集设备输入，例如 macOS 的 `0` 或 Linux 的 `/dev/video0`。
    CameraDevice(String),
}

impl FfmpegVideoSource {
    /// 返回传给 `ffmpeg -i` 的输入值。
    #[must_use]
    pub fn input_arg(&self) -> String {
        match self {
            Self::File(path) => path.display().to_string(),
            Self::Url(url) | Self::CameraDevice(url) => url.clone(),
        }
    }
}

/// `ffmpeg` 解码为 RGB 帧的配置。
#[derive(Clone, Debug, PartialEq)]
pub struct FfmpegFrameDecodeOptions {
    /// `ffmpeg` 可执行文件路径或命令名。
    pub ffmpeg_path: PathBuf,
    /// 输入视频源。
    pub source: FfmpegVideoSource,
    /// 源帧率，写入 `VideoFrame.timestamp_ms` 时使用。
    pub source_fps: f32,
    /// 抽样输出帧率。为 `None` 时保留源帧率。
    pub sample_fps: Option<f32>,
    /// 最多解码多少帧。生产长流应设置上限或由外层流式调度。
    pub max_frames: Option<usize>,
    /// 输入宽度。当前实现不做 ffprobe 自动探测，调用方需要明确提供。
    pub width: u32,
    /// 输入高度。
    pub height: u32,
}
