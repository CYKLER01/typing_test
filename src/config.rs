use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
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
    /// Not persisted: language packs are always re-read from the language
    /// files at startup, so no need to duplicate thousands of words here.
    #[serde(skip)]
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

/// Small built-in word list used when no language pack files can be found,
/// so the test still works from a bare installed binary.
fn fallback_pack() -> LanguagePack {
    LanguagePack {
        name: "english".to_string(),
        words: vec![
            "the", "be", "to", "of", "and", "a", "in", "that", "have", "i", "it", "for", "not",
            "on", "with", "he", "as", "you", "do", "at", "this", "but", "his", "by", "from",
            "they", "we", "say", "her", "she", "or", "an", "will", "my", "one", "all", "would",
            "there", "their", "what", "so", "up", "out", "if", "about", "who", "get", "which",
            "go", "me", "when", "make", "can", "like", "time", "no", "just", "him", "know",
            "take", "people", "into", "year", "your", "good", "some", "could", "them", "see",
            "other", "than", "then", "now", "look", "only", "come", "its", "over", "think",
            "also", "back", "after", "use", "two", "how", "our", "work", "first", "well", "way",
            "even", "new", "want", "because", "any", "these", "give", "day", "most", "us",
            "is", "are", "was", "were", "has", "had", "word", "being", "set", "hand",
        ]
        .iter()
        .map(|s| s.to_string())
        .collect(),
    }
}

/// Directories where language packs may live, in priority order: next to the
/// executable (so an installed binary works from any directory), then the
/// current working directory (so a source checkout just works).
fn language_pack_dirs() -> Vec<PathBuf> {
    let mut dirs = Vec::new();
    if let Ok(exe) = std::env::current_exe() {
        if let Some(dir) = exe.parent() {
            dirs.push(dir.join("languages"));
        }
    }
    if let Ok(current_dir) = std::env::current_dir() {
        dirs.push(current_dir.join("languages"));
    }
    dirs
}

fn load_packs_from(dir: &Path) -> std::io::Result<Vec<LanguagePack>> {
    let mut packs = Vec::new();
    for entry in fs::read_dir(dir)? {
        let path = entry?.path();
        if path.is_file() && path.extension().map(|ext| ext == "json").unwrap_or(false) {
            if let Ok(file_content) = fs::read_to_string(&path) {
                if let Ok(pack) = serde_json::from_str::<LanguagePack>(&file_content) {
                    packs.push(pack);
                }
            }
        }
    }
    Ok(packs)
}

pub fn load_language_packs() -> std::io::Result<Vec<LanguagePack>> {
    for dir in language_pack_dirs() {
        if !dir.exists() {
            continue;
        }
        let packs = load_packs_from(&dir)?;
        if !packs.is_empty() {
            return Ok(packs);
        }
    }
    Ok(vec![fallback_pack()])
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