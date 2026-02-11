use anyhow::Result;
use core::Project;
use video::{probe_video, extract_frames, count_frames};
use std::path::PathBuf;

fn main() -> Result<()> {
    let project_name = "shibuya_run";
    let video_file = PathBuf::from("input/sample.mp4");
    let project_dir = PathBuf::from("workspaces").join(project_name);
    let yaml_path = project_dir.join("project.yaml");

    // 1. プロジェクトのロード（既存なら読み込み、新規なら作成）
    let mut project = if yaml_path.exists() {
        println!("Loading existing project...");
        Project::load(&yaml_path)?
    } else {
        println!("Creating new project...");
        Project::new(project_name, video_file)
    };

    // 2. メタデータの補完（未設定の場合のみ）
    if project.config.extraction.resolution.is_none() || project.config.extraction.fps.is_none() {
        if let Ok(meta) = probe_video(&project.config.video_path) {
            project.config.extraction.resolution.get_or_insert((meta.width, meta.height));
            project.config.extraction.fps.get_or_insert(meta.fps);
        }
    }

    // 3. 画像切り出しの実行判断
    let frames_dir = project.config.output_dir.join("frames");
    
    if project.state.is_extraction_completed {
        println!("Extraction already completed. Skipping.");
    } else {
        println!("Starting frame extraction...");
        let fps = project.config.extraction.fps.unwrap_or(30.0);
        let res = project.config.extraction.resolution.unwrap_or((1920, 1080));

        extract_frames(&project.config.video_path, &frames_dir, fps, res)?;

        // 状態を更新
        project.state.extracted_frame_count = count_frames(&frames_dir)?;
        project.state.is_extraction_completed = true;
        
        // 状態をファイルに保存
        project.save()?;
        println!("Extraction finished. Found {} frames.", project.state.extracted_frame_count);
    }

    Ok(())
}