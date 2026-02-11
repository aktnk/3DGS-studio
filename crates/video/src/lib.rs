use std::process::Command;
use std::path::Path;
use std::fs; // <--- これが足りませんでした
use anyhow::{anyhow, Result, Context}; // <--- Context を追加
use serde_json::Value;

pub struct VideoMetadata {
    pub width: u32,
    pub height: u32,
    pub fps: f32,
}

/// 動画の情報を取得
pub fn probe_video<P: AsRef<Path>>(path: P) -> Result<VideoMetadata> {
    let output = Command::new("ffprobe")
        .args([
            "-v", "error",
            "-select_streams", "v:0",
            "-show_entries", "stream=width,height,avg_frame_rate",
            "-of", "json",
        ])
        .arg(path.as_ref())
        .output()
        .context("Failed to execute ffprobe. Is it installed?")?; // contextが使えるようになります

    if !output.status.success() {
        return Err(anyhow!("ffprobe failed: {}", String::from_utf8_lossy(&output.stderr)));
    }

    let json: Value = serde_json::from_slice(&output.stdout)?;
    let stream = &json["streams"][0];

    let width = stream["width"].as_u64().ok_or(anyhow!("Width not found"))? as u32;
    let height = stream["height"].as_u64().ok_or(anyhow!("Height not found"))? as u32;
    
    let fps_str = stream["avg_frame_rate"].as_str().ok_or(anyhow!("FPS not found"))?;
    let fps = parse_fps(fps_str)?;

    Ok(VideoMetadata { width, height, fps })
}

/// 画像を切り出す
pub fn extract_frames<P1, P2>(
    input: P1,
    output_dir: P2,
    fps: f32,
    resolution: (u32, u32),
) -> Result<()> 
where
    P1: AsRef<Path>,
    P2: AsRef<Path>,
{
    let output_path = output_dir.as_ref().join("frame_%04d.png");
    
    if !output_dir.as_ref().exists() {
        fs::create_dir_all(&output_dir)?;
    }

    let filter = format!("fps={},scale={}:{}", fps, resolution.0, resolution.1);

    let status = Command::new("ffmpeg")
        .args([
            "-i", input.as_ref().to_str().unwrap(),
            "-vf", &filter,
            "-vsync", "vfr",
            "-q:v", "2",
            output_path.to_str().unwrap(),
        ])
        .status()
        .context("Failed to execute ffmpeg. Is it installed and in your PATH?")?;

    if !status.success() {
        return Err(anyhow!("FFmpeg exited with error status"));
    }

    Ok(())
}

/// 生成された画像の枚数を数える
pub fn count_frames<P: AsRef<Path>>(dir: P) -> Result<u32> {
    if !dir.as_ref().exists() { return Ok(0); }
    
    // 型推論を助けるため、entry に型明示を追加しました
    let count = fs::read_dir(dir)?
        .filter_map(|entry| entry.ok())
        .filter(|entry: &fs::DirEntry| {
            entry.path().extension().map_or(false, |ext| ext == "png")
        })
        .count();
        
    Ok(count as u32)
}

fn parse_fps(fps_str: &str) -> Result<f32> {
    let parts: Vec<&str> = fps_str.split('/').collect();
    if parts.len() == 2 {
        let num: f32 = parts[0].parse()?;
        let den: f32 = parts[1].parse()?;
        if den > 0.0 { return Ok(num / den); }
    }
    fps_str.parse().map_err(|e| anyhow!("Failed to parse FPS: {}", e))
}