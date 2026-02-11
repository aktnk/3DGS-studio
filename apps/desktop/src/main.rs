use anyhow::Result;
use core::Project;
use std::path::PathBuf;
use eframe::egui;

fn main() -> Result<()> {
    // とりあえず既存のプロジェクトを読み込む (パスは環境に合わせて)
    let yaml_path = PathBuf::from("workspaces/shibuya_run/project.yaml");
    let project = Project::load(yaml_path)?;

    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1200.0, 800.0]),
        ..Default::default()
    };

    eframe::run_native(
        "3DGS Studio - GUI Picker",
        native_options,
        Box::new(|cc| Ok(Box::new(gui::App::new(cc, project)))),
    ).map_err(|e| anyhow::anyhow!("eframe error: {}", e))?;

    Ok(())
}