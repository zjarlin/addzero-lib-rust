use std::fs;
use std::path::Path;
use std::process::Command;

use anyhow::{Context, Result};

use crate::pipeline::TtsMode;
use crate::pipeline::ffmpeg::run_ffmpeg;
use crate::pipeline::planner::Scene;

pub fn synthesize_scene_audio(mode: &TtsMode, scene: &Scene, output_audio: &Path) -> Result<()> {
    match mode {
        TtsMode::None => synthesize_silence(scene, output_audio),
        TtsMode::SystemSay => synthesize_with_say(scene, output_audio),
        TtsMode::Command(template) => synthesize_with_command(template, scene, output_audio),
    }
}

fn synthesize_silence(scene: &Scene, output_audio: &Path) -> Result<()> {
    let seconds = estimate_seconds(&scene.text).max(2.0);
    let args = vec![
        "-f".to_string(),
        "lavfi".to_string(),
        "-i".to_string(),
        format!("anullsrc=r=44100:cl=stereo"),
        "-t".to_string(),
        format!("{seconds:.2}"),
        "-c:a".to_string(),
        "aac".to_string(),
        output_audio.display().to_string(),
    ];
    run_ffmpeg(&args)
}

fn synthesize_with_say(scene: &Scene, output_audio: &Path) -> Result<()> {
    let txt = output_audio.with_extension("txt");
    let aiff = output_audio.with_extension("aiff");
    fs::write(&txt, &scene.text).with_context(|| format!("write scene text: {}", txt.display()))?;

    let status = Command::new("say")
        .arg("-f")
        .arg(&txt)
        .arg("-o")
        .arg(&aiff)
        .status()
        .context("run say")?;
    if !status.success() {
        anyhow::bail!("say failed with status: {status}");
    }

    let args = vec![
        "-i".to_string(),
        aiff.display().to_string(),
        "-c:a".to_string(),
        "aac".to_string(),
        output_audio.display().to_string(),
    ];
    run_ffmpeg(&args)
}

fn synthesize_with_command(template: &str, scene: &Scene, output_audio: &Path) -> Result<()> {
    let text_path = output_audio.with_extension("txt");
    fs::write(&text_path, &scene.text)
        .with_context(|| format!("write tts text file: {}", text_path.display()))?;

    let command_line = template
        .replace("{input}", &text_path.display().to_string())
        .replace("{output}", &output_audio.display().to_string());

    #[cfg(target_os = "windows")]
    let status = Command::new("cmd")
        .arg("/C")
        .arg(&command_line)
        .status()
        .with_context(|| format!("run tts command: {command_line}"))?;

    #[cfg(not(target_os = "windows"))]
    let status = Command::new("sh")
        .arg("-c")
        .arg(&command_line)
        .status()
        .with_context(|| format!("run tts command: {command_line}"))?;

    if !status.success() {
        anyhow::bail!("tts command failed with status: {status}");
    }

    Ok(())
}

fn estimate_seconds(text: &str) -> f32 {
    let chars = text.chars().count() as f32;
    // Approximation for Mandarin narration pace.
    chars / 4.2
}
