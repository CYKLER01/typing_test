use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use directories::ProjectDirs;

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
pub enum WordListDifficulty {
    Easy,
    Medium,
    Hard,
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
    pub word_list_difficulty: WordListDifficulty,
    pub restart_button: bool,
    pub color_theme: ColorTheme,
    pub layout_theme: LayoutTheme,
    pub results: HashMap<String, Vec<TestResult>>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            default_test_length: 20,
            default_time_limit: 60,
            game_mode: GameMode::Words,
            word_list_difficulty: WordListDifficulty::Easy,
            restart_button: true,
            color_theme: ColorTheme::default(),
            layout_theme: LayoutTheme::Default,
            results: HashMap::new(),
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

pub fn load_config() -> Config {
    if let Some(config_path) = get_config_path() {
        if let Ok(config_str) = fs::read_to_string(&config_path) {
            if let Ok(config) = serde_json::from_str(&config_str) {
                return config;
            }
        }
        // If the file doesn't exist or is invalid, create a default one
        let config = Config::default();
        if let Ok(config_str) = serde_json::to_string_pretty(&config) {
            fs::write(config_path, config_str).ok();
        }
        config
    } else {
        Config::default()
    }
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

pub const EASY_WORDS: &[&str] = &[
    "the", "and", "for", "are", "but", "not", "you", "all", "any", "can",
    "her", "was", "one", "our", "out", "day", "get", "has", "him", "his",
];

pub const MEDIUM_WORDS: &[&str] = &[
    "about", "above", "added", "after", "again", "along", "always", "among", "apple", "baker",
    "basic", "begin", "being", "below", "bring", "build", "carry", "catch", "cause", "chair",
];

pub const HARD_WORDS: &[&str] = &[
    "abundant", "accurate", "achieve", "acquire", "address", "advance", "adverse", "advocate", "ageless", "agitate",
    "airplane", "allocate", "although", "ambition", "analysis", "ancestor", "announce", "anxious", "apology", "apparent",
];