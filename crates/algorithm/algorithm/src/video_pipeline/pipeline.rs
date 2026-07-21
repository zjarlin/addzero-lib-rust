//! 视频算法 pipeline 执行辅助函数。

use std::collections::HashSet;
use std::fs::{self, File};
use std::io::{BufWriter, Write};

use anyhow::anyhow;

use crate::video_pipeline::model::{
    VideoAlgorithmBinding, VideoAlgorithmFrameResult, VideoAlgorithmRunSummary,
    VideoAlgorithmSchedule, VideoFrame, VideoPipelineOptions, VideoPipelineOutputFiles,
    VideoPipelineRun,
};

/// 在同一条视频帧流上叠加运行多个常驻算法实例。
///
/// 本函数不负责视频解码。调用方应先把视频、摄像头或 RTSP 流解码为 `VideoFrame`，
/// 再把帧迭代器交给 pipeline。这样同一帧只解码一次，多个算法可以共享帧数据。
///
/// # Errors
/// 配置无效、帧尺寸不一致、算法执行失败或结果文件写入失败时返回错误。
pub fn run_video_frame_pipeline(
    frames: impl IntoIterator<Item = VideoFrame>,
    algorithms: &mut [VideoAlgorithmBinding<'_>],
    options: &VideoPipelineOptions,
) -> anyhow::Result<VideoPipelineRun> {
    validate_options(algorithms, options)?;
    fs::create_dir_all(&options.output_dir)
        .map_err(|source| path_error(options.output_dir.clone(), source))?;
    let output_dir = fs::canonicalize(&options.output_dir)
        .map_err(|source| path_error(options.output_dir.clone(), source))?;

    let files = VideoPipelineOutputFiles {
        frame_results_jsonl: output_dir.join("pipeline_frame_results.jsonl"),
        summary_json: output_dir.join("pipeline_summary.json"),
    };
    let jsonl_file = File::create(&files.frame_results_jsonl)
        .map_err(|source| path_error(files.frame_results_jsonl.clone(), source))?;
    let mut jsonl_writer = BufWriter::new(jsonl_file);

    let mut frame_results = Vec::new();
    let mut total_input_frames = 0usize;
    let mut processed_frame_indices = algorithms
        .iter()
        .map(|_| Vec::<u64>::new())
        .collect::<Vec<_>>();

    for frame in frames {
        validate_frame(&frame)?;
        total_input_frames += 1;

        for (algorithm_index, binding) in algorithms.iter_mut().enumerate() {
            if !should_run_frame(binding.schedule, frame.frame_index, options.source_fps)? {
                continue;
            }

            let algorithm_code = binding.algorithm.code();
            let mut result = binding.algorithm.process_frame(&frame)?;
            normalize_frame_result(&mut result, algorithm_code, &frame)?;
            serde_json::to_writer(&mut jsonl_writer, &result)?;
            jsonl_writer
                .write_all(b"\n")
                .map_err(|source| path_error(files.frame_results_jsonl.clone(), source))?;
            processed_frame_indices[algorithm_index].push(frame.frame_index);
            frame_results.push(result);
        }
    }
    jsonl_writer
        .flush()
        .map_err(|source| path_error(files.frame_results_jsonl.clone(), source))?;

    let algorithm_runs = algorithms
        .iter()
        .enumerate()
        .map(|(index, binding)| VideoAlgorithmRunSummary {
            algorithm_code: binding.algorithm.code().to_owned(),
            schedule: binding.schedule,
            processed_frame_indices: processed_frame_indices[index].clone(),
            processed_frame_count: processed_frame_indices[index].len(),
        })
        .collect::<Vec<_>>();

    let run = VideoPipelineRun {
        source_fps: options.source_fps,
        output_dir,
        files: files.clone(),
        total_input_frames,
        frame_results,
        algorithm_runs,
    };
    let summary_json = serde_json::to_string_pretty(&run)?;
    fs::write(&files.summary_json, summary_json)
        .map_err(|source| path_error(files.summary_json.clone(), source))?;

    Ok(run)
}

/// 判断某个调度策略在当前帧是否应该执行。
///
/// # Errors
/// 调度配置或源帧率无效时返回错误。
pub fn should_run_frame(
    schedule: VideoAlgorithmSchedule,
    frame_index: u64,
    source_fps: f32,
) -> anyhow::Result<bool> {
    let interval = frame_interval_for_schedule(schedule, source_fps)?;
    Ok(frame_index.is_multiple_of(interval))
}

/// 把 fps 调度策略折算成帧间隔。
///
/// # Errors
/// 调度配置或源帧率无效时返回错误。
pub fn frame_interval_for_schedule(
    schedule: VideoAlgorithmSchedule,
    source_fps: f32,
) -> anyhow::Result<u64> {
    match schedule {
        VideoAlgorithmSchedule::EveryFrame => Ok(1),
        VideoAlgorithmSchedule::EveryNFrames { n } => {
            if n == 0 {
                anyhow::bail!("EveryNFrames.n 必须大于 0");
            }
            Ok(n)
        }
        VideoAlgorithmSchedule::TargetFps { fps } => {
            validate_positive_fps("source_fps", source_fps)?;
            validate_positive_fps("target_fps", fps)?;
            if fps >= source_fps {
                return Ok(1);
            }
            Ok((source_fps / fps).round().max(1.0) as u64)
        }
    }
}

fn validate_options(
    algorithms: &[VideoAlgorithmBinding<'_>],
    options: &VideoPipelineOptions,
) -> anyhow::Result<()> {
    validate_positive_fps("source_fps", options.source_fps)?;
    if algorithms.is_empty() {
        anyhow::bail!("至少需要传入一个算法实例");
    }

    let mut codes = HashSet::new();
    for binding in algorithms {
        let code = binding.algorithm.code();
        if code.trim().is_empty() {
            anyhow::bail!("算法 code 不能为空");
        }
        if !codes.insert(code) {
            let message = format!("重复算法 code：{code}");
            anyhow::bail!(message);
        }
        let _ = frame_interval_for_schedule(binding.schedule, options.source_fps)?;
    }
    Ok(())
}

fn validate_positive_fps(name: &str, fps: f32) -> anyhow::Result<()> {
    if !fps.is_finite() || fps <= 0.0 {
        let message = format!("{name} 必须是大于 0 的有限数字");
        anyhow::bail!(message);
    }
    Ok(())
}

fn validate_frame(frame: &VideoFrame) -> anyhow::Result<()> {
    let actual_width = frame.rgb.width();
    let actual_height = frame.rgb.height();
    if frame.width != actual_width || frame.height != actual_height {
        let message = format!(
            "帧 {} 声明尺寸 {}x{} 与 RGB 数据尺寸 {}x{} 不一致",
            frame.frame_index, frame.width, frame.height, actual_width, actual_height
        );
        anyhow::bail!(message);
    }
    Ok(())
}

fn normalize_frame_result(
    result: &mut VideoAlgorithmFrameResult,
    algorithm_code: &str,
    frame: &VideoFrame,
) -> anyhow::Result<()> {
    if result.algorithm_code.is_empty() {
        result.algorithm_code = algorithm_code.to_owned();
    }
    if result.algorithm_code != algorithm_code {
        let message = format!(
            "算法 {algorithm_code} 返回了不匹配的结果 code：{}",
            result.algorithm_code
        );
        anyhow::bail!(message);
    }
    if result.frame_index != frame.frame_index {
        let message = format!(
            "算法 {algorithm_code} 返回的帧序号 {} 与输入帧序号 {} 不一致",
            result.frame_index, frame.frame_index
        );
        anyhow::bail!(message);
    }
    if result.timestamp_ms != frame.timestamp_ms {
        let message = format!(
            "算法 {algorithm_code} 返回的时间戳 {} 与输入时间戳 {} 不一致",
            result.timestamp_ms, frame.timestamp_ms
        );
        anyhow::bail!(message);
    }
    Ok(())
}

fn path_error(path: std::path::PathBuf, source: std::io::Error) -> anyhow::Error {
    anyhow!("filesystem error at `{}`: {source}", path.display())
}
