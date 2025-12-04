use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use directories::ProjectDirs;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct LanguagePack {
    pub name: String,
    pub words: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum LayoutTheme {
    Default,
    Boxes,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ColorTheme {
    pub correct: (u8, u8, u8),
    pub incorrect: (u8, u8, u8),
    pub default: (u8, u8, u8),
}

impl Default for ColorTheme {
    fn default() -> Self {
        Self {
            correct: (0, 255, 0),   // Green
            incorrect: (255, 0, 0), // Red
            default: (255, 255, 255), // White
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum GameMode {
    Words,
    Time,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TestResult {
    pub wpm: f64,
    pub accuracy: f64,
    pub timestamp: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Config {
    pub default_test_length: usize,
    pub default_time_limit: u64,
    pub game_mode: GameMode,
    pub restart_button: bool,
    pub color_theme: ColorTheme,
    pub layout_theme: LayoutTheme,
    pub results: HashMap<String, Vec<TestResult>>,
    pub language_packs: Vec<LanguagePack>,
    pub selected_language: String,
}

impl Default for Config {
    fn default() -> Self {
        let language_packs = load_language_packs().unwrap_or_default();
        let selected_language = if language_packs.is_empty() {
            "english".to_string()
        } else {
            language_packs[0].name.clone()
        };
        Self {
            default_test_length: 20,
            default_time_limit: 60,
            game_mode: GameMode::Words,
            restart_button: true,
            color_theme: ColorTheme::default(),
            layout_theme: LayoutTheme::Default,
            results: HashMap::new(),
            language_packs,
            selected_language,
        }
    }
}

fn get_config_path() -> Option<PathBuf> {
    if let Some(proj_dirs) = ProjectDirs::from("com", "gemini", "typing_test") {
        let config_dir = proj_dirs.config_dir();
        if !config_dir.exists() {
            fs::create_dir_all(config_dir).ok()?;
        }
        Some(config_dir.join("config.json"))
    } else {
        None
    }
}

pub fn load_language_packs() -> std::io::Result<Vec<LanguagePack>> {
    let mut packs = Vec::new();
    let paths = fs::read_dir("./languages")?;
    for path in paths {
        let path = path?.path();
        if path.is_file() {
            if let Some(ext) = path.extension() {
                if ext == "json" {
                    let file_content = fs::read_to_string(&path)?;
                    let pack: LanguagePack = serde_json::from_str(&file_content)?;
                    packs.push(pack);
                }
            }
        }
    }
    Ok(packs)
}

pub fn load_config() -> Config {
    if let Some(config_path) = get_config_path() {
        if let Ok(config_str) = fs::read_to_string(&config_path) {
            let mut config: Config = match serde_json::from_str(&config_str) {
                Ok(config) => config,
                Err(_) => {
                    // If the file is invalid, create a default one
                    let config = Config::default();
                    if let Ok(config_str) = serde_json::to_string_pretty(&config) {
                        fs::write(config_path, config_str).ok();
                    }
                    return config;
                }
            };
            config.language_packs = load_language_packs().unwrap_or_default();
            return config;
        }
    }
    // If the file doesn't exist, create a default one
    let config = Config::default();
    if let Some(config_path) = get_config_path() {
        if let Ok(config_str) = serde_json::to_string_pretty(&config) {
            fs::write(config_path, config_str).ok();
        }
    }
    config
}

pub fn save_config(config: &Config) -> std::io::Result<()> {
    if let Some(config_path) = get_config_path() {
        let config_str = serde_json::to_string_pretty(config)?;
        fs::write(config_path, config_str)
    } else {
        Err(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "Could not find config directory",
        ))
    }
}