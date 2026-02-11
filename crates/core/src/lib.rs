use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::fs;
use anyhow::{Context, Result};

// --- 設定 ---
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ProjectConfig {
    pub project_name: String,
    pub video_path: PathBuf,
    pub output_dir: PathBuf,
    pub extraction: ExtractionSettings,
    #[serde(default)]
    pub targets: Vec<MaskTarget>, // 追加：複数のマスク対象を管理
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ExtractionSettings {
    pub fps: Option<f32>,
    pub resolution: Option<(u32, u32)>,
}

// --- 追加：ターゲットの定義 ---
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MaskTarget {
    pub id: String,
    pub target_type: TargetType,
    pub auto_track: bool,
    pub points: Vec<(u32, u32)>, // [(x, y)]
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "snake_case")]
pub enum TargetType {
    Photographer,
    Vehicle,
    Pedestrian,
}

// --- 状態 ---
#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct ProjectState {
    pub is_extraction_completed: bool,
    pub extracted_frame_count: u32,
}

// --- プロジェクト全体 ---
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Project {
    pub config: ProjectConfig,
    pub state: ProjectState,
}

impl Project {
    pub fn new(name: &str, video: PathBuf) -> Self {
        Self {
            config: ProjectConfig {
                project_name: name.to_string(),
                video_path: video,
                output_dir: PathBuf::from("workspaces").join(name),
                extraction: ExtractionSettings {
                    fps: None,
                    resolution: None,
                },
                targets: Vec::new(), // 空で初期化
            },
            state: ProjectState::default(),
        }
    }

    // --- 追加：座標を更新するメソッド ---
    pub fn update_target_point(&mut self, target_id: &str, x: u32, y: u32) {
        // すでに同じIDのターゲットがあれば更新、なければ新規作成
        if let Some(target) = self.config.targets.iter_mut().find(|t| t.id == target_id) {
            target.points = vec![(x, y)]; // 現在は単一クリックのみ想定
        } else {
            self.config.targets.push(MaskTarget {
                id: target_id.to_string(),
                target_type: TargetType::Photographer,
                auto_track: true,
                points: vec![(x, y)],
            });
        }
    }

    pub fn save(&self) -> Result<()> {
        let yaml_path = self.config.output_dir.join("project.yaml");
        let yaml = serde_yaml::to_string(self)?;
        fs::write(yaml_path, yaml).context("Failed to save project.yaml")
    }

    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self> {
        let content = fs::read_to_string(path)?;
        let project: Self = serde_yaml::from_str(&content)?;
        Ok(project)
    }
}