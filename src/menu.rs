use crate::config::{self, Config, GameMode, LayoutTheme};
use crossterm::{
    cursor,
    event::{self, Event, KeyCode},
    style::{Print, Stylize},
    terminal::{self, Clear, ClearType, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use std::io::{self, Stdout, Write};

struct MenuState {
    config: Config,
    selected_item: usize,
    status_message: String,
}

const MENU_ITEMS: [&str; 5] = [
    "Game Mode",
    "Test Length (Words)",
    "Time Limit (Seconds)",
    "Layout Theme",
    "Language",
];

pub fn run() -> io::Result<()> {
    let mut stdout = io::stdout();
    stdout.execute(EnterAlternateScreen)?;
    terminal::enable_raw_mode()?;

    let result = show_menu(&mut stdout);

    terminal::disable_raw_mode()?;
    stdout.execute(LeaveAlternateScreen)?;
    result
}

pub fn show_menu(stdout: &mut io::Stdout) -> io::Result<()> {
    let mut state = MenuState {
        config: config::load_config(),
        selected_item: 0,
        status_message: "".to_string(),
    };

    loop {
        draw_menu(stdout, &state)?;

        if let Event::Key(key_event) = event::read()? {
            match key_event.code {
                KeyCode::Char('q') => break,
                KeyCode::Up => {
                    state.selected_item = state.selected_item.saturating_sub(1);
                }
                KeyCode::Down => {
                    state.selected_item = (state.selected_item + 1).min(MENU_ITEMS.len() - 1);
                }
                KeyCode::Left => change_value(&mut state, -1),
                KeyCode::Right => change_value(&mut state, 1),
                KeyCode::Enter => {
                    match config::save_config(&state.config) {
                        Ok(_) => state.status_message = "Config saved successfully!".to_string(),
                        Err(e) => state.status_message = format!("Error saving config: {}", e),
                    }
                }
                _ => {}
            }
        }
    }
    Ok(())
}

fn change_value(state: &mut MenuState, direction: i32) {
    match state.selected_item {
        0 => { // Game Mode
            state.config.game_mode = match state.config.game_mode {
                GameMode::Words => GameMode::Time,
                GameMode::Time => GameMode::Words,
            };
        }
        1 => { // Test Length
            let current = state.config.default_test_length as i32;
            state.config.default_test_length = (current + direction * 5).max(5) as usize;
        }
        2 => { // Time Limit
            let current = state.config.default_time_limit as i32;
            state.config.default_time_limit = (current + direction * 5).max(10) as u64;
        }
        3 => { // Layout Theme
            state.config.layout_theme = match state.config.layout_theme {
                LayoutTheme::Default => LayoutTheme::Boxes,
                LayoutTheme::Boxes => LayoutTheme::Default,
            };
        }
        4 => { // Language
            let current_language_index = state.config.language_packs.iter().position(|p| p.name == state.config.selected_language).unwrap_or(0);
            let next_index = (current_language_index as i32 + direction).rem_euclid(state.config.language_packs.len() as i32) as usize;
            state.config.selected_language = state.config.language_packs[next_index].name.clone();
        }
        _ => {},
    }
}

fn draw_menu(stdout: &mut Stdout, state: &MenuState) -> io::Result<()> {
    let (width, height) = terminal::size()?;
    stdout.execute(Clear(ClearType::All))?;

    let title = "Settings Menu";
    let title_x = (width - title.len() as u16) / 2;
    stdout
        .execute(cursor::MoveTo(title_x, 2))?
        .execute(Print(title.bold()))?;

    for (i, item) in MENU_ITEMS.iter().enumerate() {
        let y = 5 + i as u16 * 2;
        let value_str = get_value_string(&state.config, i);

        let line = format!("{: <25}: {}", item, value_str);
        
        if i == state.selected_item {
            stdout
                .execute(cursor::MoveTo(5, y))?
                .execute(Print(line.negative()))?;
        } else {
            stdout.execute(cursor::MoveTo(5, y))?.execute(Print(line))?;
        }
    }

    let instructions = "Use ↑/↓ to navigate, ←/→ to change values, 'enter' to save, 'q' to quit.";
    let status_x = (width - state.status_message.len() as u16) / 2;
    let inst_x = (width - instructions.len() as u16) / 2;

    stdout
        .execute(cursor::MoveTo(status_x, height - 4))?
        .execute(Print(&state.status_message))?;
    stdout
        .execute(cursor::MoveTo(inst_x, height - 2))?
        .execute(Print(instructions.dark_grey()))?;

    stdout.flush()
}

fn get_value_string(config: &Config, item_index: usize) -> String {
    match item_index {
        0 => format!("{:?}", config.game_mode),
        1 => format!("{} words", config.default_test_length),
        2 => format!("{} seconds", config.default_time_limit),
        3 => format!("{:?}", config.layout_theme),
        4 => config.selected_language.clone(),
        _ => "".to_string(),
    }
}
