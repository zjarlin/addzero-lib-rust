use std::fs;
use std::path::Path;
use std::sync::OnceLock;

use anyhow::{Context, Result};

use crate::pipeline::PipelineConfig;
use crate::pipeline::ffmpeg::{filter_exists, run_ffmpeg};
use crate::pipeline::planner::Scene;

pub fn render_scene_video(
    config: &PipelineConfig,
    scene: &Scene,
    audio_path: &Path,
    output_video: &Path,
) -> Result<()> {
    let text_path = output_video.with_extension("txt");
    fs::write(&text_path, &scene.text)
        .with_context(|| format!("write scene caption text: {}", text_path.display()))?;

    let mut args = vec![
        "-i".to_string(),
        audio_path.display().to_string(),
        "-f".to_string(),
        "lavfi".to_string(),
        "-i".to_string(),
        format!(
            "color=c=#111827:s={}x{}:r={}",
            config.width, config.height, config.fps
        ),
        "-shortest".to_string(),
        "-c:v".to_string(),
        "libx264".to_string(),
        "-pix_fmt".to_string(),
        "yuv420p".to_string(),
        "-c:a".to_string(),
        "aac".to_string(),
        output_video.display().to_string(),
    ];

    if filter_exists("drawtext") {
        let filter = format!(
            "drawtext=textfile={}:fontcolor=white:fontsize=42:line_spacing=12:x=(w-text_w)/2:y=(h-text_h)/2",
            escape_filter_path(&text_path.display().to_string())
        );
        args.splice(7..7, ["-vf".to_string(), filter]);
    } else {
        warn_missing_drawtext_once();
    }

    run_ffmpeg(&args)
}

pub fn stitch_scenes(
    scene_videos: &[impl AsRef<Path>],
    bgm: Option<&Path>,
    output: &Path,
) -> Result<()> {
    let parent = output
        .parent()
        .context("output path must have parent directory")?;
    let list_file = parent.join("concat_list.txt");

    let mut body = String::new();
    for video in scene_videos {
        body.push_str(&format!("file '{}'\n", video.as_ref().display()));
    }
    fs::write(&list_file, body)
        .with_context(|| format!("write concat list: {}", list_file.display()))?;

    let merged_video = parent.join("_merged.mp4");
    let concat_args = vec![
        "-f".to_string(),
        "concat".to_string(),
        "-safe".to_string(),
        "0".to_string(),
        "-i".to_string(),
        list_file.display().to_string(),
        "-c".to_string(),
        "copy".to_string(),
        merged_video.display().to_string(),
    ];
    run_ffmpeg(&concat_args)?;

    if let Some(bgm_path) = bgm {
        let mix_args = vec![
            "-i".to_string(),
            merged_video.display().to_string(),
            "-i".to_string(),
            bgm_path.display().to_string(),
            "-filter_complex".to_string(),
            "[1:a]volume=0.2[bgm];[0:a][bgm]amix=inputs=2:duration=first:dropout_transition=2[aout]".to_string(),
            "-map".to_string(),
            "0:v".to_string(),
            "-map".to_string(),
            "[aout]".to_string(),
            "-c:v".to_string(),
            "copy".to_string(),
            "-c:a".to_string(),
            "aac".to_string(),
            output.display().to_string(),
        ];
        run_ffmpeg(&mix_args)?;
    } else {
        fs::copy(&merged_video, output).with_context(|| {
            format!(
                "copy merged video {} -> {}",
                merged_video.display(),
                output.display()
            )
        })?;
    }

    Ok(())
}

fn escape_filter_path(path: &str) -> String {
    path.replace('\\', "\\\\")
        .replace(':', "\\:")
        .replace('"', "\\\"")
}

fn warn_missing_drawtext_once() {
    static WARNED: OnceLock<()> = OnceLock::new();

    WARNED.get_or_init(|| {
        eprintln!(
            "warning: ffmpeg drawtext filter is unavailable; rendering scene without captions"
        );
    });
}
