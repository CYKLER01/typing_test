use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::{self, OpenOptions};
use std::io::{self, Write};
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
        Self {
            default_test_length: 20,
            default_time_limit: 60,
            game_mode: GameMode::Words,
            restart_button: true,
            color_theme: ColorTheme::default(),
            layout_theme: LayoutTheme::Default,
            results: HashMap::new(),
            language_packs: Vec::new(), // Will be populated by load_config
            selected_language: "english".to_string(), // Will be validated by load_config
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

fn log_debug(message: &str) {
    if let Ok(mut file) = OpenOptions::new().create(true).append(true).open("debug_log.txt") {
        writeln!(file, "{}", message).ok();
    }
}

pub fn load_language_packs() -> std::io::Result<Vec<LanguagePack>> {
    let mut packs = Vec::new();
    let current_dir = std::env::current_dir()?;
    log_debug(&format!("Current working directory: {:?}", current_dir));

    let language_dir = current_dir.join("languages");
    log_debug(&format!("Attempting to load language packs from: {:?}", language_dir));

    if !language_dir.exists() {
        log_debug(&format!("Language directory {:?} does not exist.", language_dir));
        return Ok(packs); // Return empty if directory not found
    }

    let paths = fs::read_dir(&language_dir)?;
    for path in paths {
        let path = path?.path();
        if path.is_file() {
            if let Some(ext) = path.extension() {
                if ext == "json" {
                    log_debug(&format!("Found language file: {:?}", path));
                    if let Ok(file_content) = fs::read_to_string(&path) {
                        match serde_json::from_str::<LanguagePack>(&file_content) {
                            Ok(pack) => {
                                log_debug(&format!("Successfully parsed language pack: {}", pack.name));
                                packs.push(pack);
                            }
                            Err(e) => {
                                log_debug(&format!("Failed to parse {:?}: {}", path, e));
                            }
                        }
                    } else {
                        log_debug(&format!("Failed to read file: {:?}", path));
                    }
                }
            }
        }
    }
    log_debug(&format!("Loaded {} language packs.", packs.len()));
    Ok(packs)
}

pub fn load_config() -> Config {
    let current_language_packs = load_language_packs().unwrap_or_default();
    let default_selected_language = if current_language_packs.is_empty() {
        "english".to_string()
    } else {
        current_language_packs[0].name.clone()
    };

    let mut config = if let Some(config_path) = get_config_path() {
        if let Ok(config_str) = fs::read_to_string(&config_path) {
            match serde_json::from_str::<Config>(&config_str) {
                Ok(mut c) => {
                    c.language_packs = current_language_packs;
                    if !c.language_packs.iter().any(|p| p.name == c.selected_language) {
                        c.selected_language = default_selected_language.clone();
                    }
                    c
                },
                Err(_) => {
                    // If the file is invalid, create a default one
                    let mut new_config = Config::default();
                    new_config.language_packs = current_language_packs;
                    new_config.selected_language = default_selected_language.clone();
                    if let Ok(config_str) = serde_json::to_string_pretty(&new_config) {
                        fs::write(config_path, config_str).ok();
                    }
                    new_config
                }
            }
        } else {
            // If the file doesn't exist, create a default one
            let mut new_config = Config::default();
            new_config.language_packs = current_language_packs;
            new_config.selected_language = default_selected_language.clone();
            if let Ok(config_str) = serde_json::to_string_pretty(&new_config) {
                fs::write(config_path, config_str).ok();
            }
            new_config
        }
    } else {
        // If config path cannot be determined, return a default config
        let mut new_config = Config::default();
        new_config.language_packs = current_language_packs;
        new_config.selected_language = default_selected_language.clone();
        new_config
    };

    // Ensure language_packs are always up-to-date in the returned config
    config.language_packs = load_language_packs().unwrap_or_default();
    if !config.language_packs.iter().any(|p| p.name == config.selected_language) {
        config.selected_language = default_selected_language;
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