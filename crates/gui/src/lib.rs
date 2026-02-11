use eframe::egui;
use core::Project;
use std::path::PathBuf;

pub struct App {
    project: Project,
    current_frame_idx: u32,
    _texture: Option<egui::TextureHandle>,
    pixel_coord: Option<(u32, u32)>, 
}

impl App {
    pub fn new(cc: &eframe::CreationContext<'_>, project: Project) -> Self {
        egui_extras::install_image_loaders(&cc.egui_ctx);
        Self {
            project,
            current_frame_idx: 1,
            _texture: None,
            pixel_coord: None,
        }
    }

    fn current_image_path(&self) -> PathBuf {
        self.project.config.output_dir
            .join("frames")
            .join(format!("frame_{:04}.png", self.current_frame_idx))
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::SidePanel::left("controls").show(ctx, |ui| {
            ui.heading("3DGS Dataset Tool");
            ui.label(format!("Project: {}", self.project.config.project_name));
            ui.separator();

            let max_frames = self.project.state.extracted_frame_count;
            ui.add(egui::Slider::new(&mut self.current_frame_idx, 1..=max_frames).text("Frame"));
            
            // ... SidePanel内のボタン処理 ...
            if ui.button("Save Coordinates").clicked() {
                if let Some((x, y)) = self.pixel_coord {
                    // 1. メモリ上のプロジェクトデータを更新
                    self.project.update_target_point("photographer_01", x, y);
                    
                    // 2. ファイルに保存
                    match self.project.save() {
                        Ok(_) => {
                            println!("Successfully saved: Photographer at ({}, {})", x, y);
                        }
                        Err(e) => {
                            eprintln!("Error saving coordinates: {}", e);
                        }
                    }
                }
            }

            if let Some((x, y)) = self.pixel_coord {
                ui.label(format!("Clicked Pixel: X={}, Y={}", x, y));
            } else {
                ui.label("Click on image to select photographer");
            }
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            let img_path = self.current_image_path();
            let uri = format!("file://{}", img_path.to_str().unwrap_or("").replace("\\", "/"));
            
            // 重要： .sense(egui::Sense::click()) を追加してクリックを有効化します
            let image = egui::Image::new(uri)
                .maintain_aspect_ratio(true)
                .shrink_to_fit()
                .sense(egui::Sense::click()); 

            let response = ui.add(image);
            let rect = response.rect; 

            // --- クリック判定 ---
            if response.clicked() {
                if let Some(pointer_pos) = response.interact_pointer_pos() {
                    let (w_orig, h_orig) = self.project.config.extraction.resolution.unwrap_or((1, 1));
                    
                    let x_pct = (pointer_pos.x - rect.min.x) / rect.width();
                    let y_pct = (pointer_pos.y - rect.min.y) / rect.height();
                    
                    let px = (x_pct * w_orig as f32) as u32;
                    let py = (y_pct * h_orig as f32) as u32;
                    
                    self.pixel_coord = Some((px, py));
                    
                    // これでコンソールに表示されるはずです
                    println!("SUCCESS: Pixel({}, {})", px, py);
                }
            }

            // --- 描画 ---
            if let Some((px, py)) = self.pixel_coord {
                let (w_orig, h_orig) = self.project.config.extraction.resolution.unwrap_or((1, 1));
                
                let screen_x = rect.min.x + (px as f32 / w_orig as f32) * rect.width();
                let screen_y = rect.min.y + (py as f32 / h_orig as f32) * rect.height();
                let pos = egui::pos2(screen_x, screen_y);

                let painter = ui.painter();
                let stroke = egui::Stroke::new(3.0, egui::Color32::RED);
                
                painter.line_segment([pos - egui::vec2(20.0, 0.0), pos + egui::vec2(20.0, 0.0)], stroke);
                painter.line_segment([pos - egui::vec2(0.0, 20.0), pos + egui::vec2(0.0, 20.0)], stroke);
                painter.circle_filled(pos, 4.0, egui::Color32::YELLOW);
            }
        });
    }
}