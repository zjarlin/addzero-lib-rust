enum PipelineFile {
    Manifest,
    FinalVideo,
}

use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use serde::Serialize;

use super::ffmpeg;
use super::planner::build_plan;
use super::render::{render_scene_video, stitch_scenes};
use super::tts::synthesize_scene_audio;

#[derive(Debug, Clone)]
pub enum TtsMode {
    None,
    SystemSay,
    Command(String),
}

#[derive(Debug, Clone)]
pub struct PipelineConfig {
    pub input: PathBuf,
    pub output: PathBuf,
    pub title: String,
    pub scene_chars: usize,
    pub width: u32,
    pub height: u32,
    pub fps: u32,
    pub tts_mode: TtsMode,
    pub bgm: Option<PathBuf>,
}

#[derive(Debug, Serialize)]
struct RunManifest {
    title: String,
    scene_count: usize,
    output_video: String,
}

pub fn run_pipeline(config: PipelineConfig) -> Result<()> {
    ensure_tooling()?;
    fs::create_dir_all(&config.output)
        .with_context(|| format!("create output dir: {}", config.output.display()))?;

    let source = fs::read_to_string(&config.input)
        .with_context(|| format!("read input: {}", config.input.display()))?;
    let plan = build_plan(&source, config.scene_chars);

    if plan.scenes.is_empty() {
        anyhow::bail!("no scenes generated from input text");
    }

    let scenes_dir = config.output.join("scenes");
    fs::create_dir_all(&scenes_dir)
        .with_context(|| format!("create scenes dir: {}", scenes_dir.display()))?;

    let mut scene_videos = Vec::with_capacity(plan.scenes.len());

    for scene in &plan.scenes {
        let scene_dir = scenes_dir.join(format!("scene_{:04}", scene.index));
        fs::create_dir_all(&scene_dir)
            .with_context(|| format!("create scene dir: {}", scene_dir.display()))?;

        let audio_path = scene_dir.join("voice.m4a");
        synthesize_scene_audio(&config.tts_mode, scene, &audio_path)?;

        let scene_video_path = scene_dir.join("video.mp4");
        render_scene_video(&config, scene, &audio_path, &scene_video_path)?;

        scene_videos.push(scene_video_path);
    }

    let final_video = config.output.join(file_name(PipelineFile::FinalVideo));
    stitch_scenes(&scene_videos, config.bgm.as_deref(), &final_video)?;

    let manifest = RunManifest {
        title: config.title,
        scene_count: plan.scenes.len(),
        output_video: final_video.display().to_string(),
    };

    let manifest_path = config.output.join(file_name(PipelineFile::Manifest));
    fs::write(&manifest_path, serde_json::to_vec_pretty(&manifest)?)
        .with_context(|| format!("write manifest: {}", manifest_path.display()))?;

    println!("done: {}", final_video.display());
    Ok(())
}

fn ensure_tooling() -> Result<()> {
    ffmpeg::assert_ffmpeg_exists()
}

fn file_name(file: PipelineFile) -> &'static str {
    match file {
        PipelineFile::Manifest => "manifest.json",
        PipelineFile::FinalVideo => "final.mp4",
    }
}

fn _must_exist(path: &Path) -> Result<()> {
    if !path.exists() {
        anyhow::bail!("path does not exist: {}", path.display());
    }
    Ok(())
}
