mod config;
mod menu;
mod stats; 
use config::{EASY_WORDS, MEDIUM_WORDS, HARD_WORDS};
use crossterm::{
    cursor,
    event::{self, Event, KeyCode},
    style::{Color, Print, ResetColor, SetForegroundColor},
    terminal::{self, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use rand::seq::SliceRandom;
use std::env;
use std::io;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Instant;
use chrono::Local;

fn main() -> io::Result<()> {
    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();

    ctrlc::set_handler(move || {
        r.store(false, Ordering::SeqCst);
    })
    .expect("Error setting Ctrl-C handler");

    let mut config = config::load_config();
    let args: Vec<String> = env::args().collect();

    let mut stdout = io::stdout();
    let mut rng = rand::thread_rng();

    if args.contains(&"-h".to_string()) || args.contains(&"--help".to_string()) {
        println!("Typing Test CLI");
        println!("A terminal-based typing test application.");
        println!("USAGE:");
        println!("    typing_test [OPTIONS]");
        println!("OPTIONS:");
        println!("    -m, --menu              Opens the interactive settings menu.");
        println!("    -s, --stats             Shows your saved stats.");
        println!("    -h, --help              Prints this help message.");
        println!("EXAMPLES:");
        println!("    cargo run --             # Starts the typing test with current settings.");
        println!("    cargo run -- -m          # Opens the settings menu.");
        return Ok(());
    }

    if args.contains(&"-m".to_string()) || args.contains(&"--menu".to_string()) {
        return menu::run();
    }

    if args.contains(&"-s".to_string()) || args.contains(&"--stats".to_string()) {
        return stats::show_stats();
    }

    stdout.execute(EnterAlternateScreen)?;
    terminal::enable_raw_mode()?;

    while running.load(Ordering::SeqCst) {
        match (|| -> io::Result<()> {
            let game_mode = config.game_mode.clone();
            let num_words = config.default_test_length;
            let time_limit = config.default_time_limit;
            let layout_theme = config.layout_theme.clone();

            let current_word_list = match config.word_list_difficulty {
                config::WordListDifficulty::Easy => EASY_WORDS,
                config::WordListDifficulty::Medium => MEDIUM_WORDS,
                config::WordListDifficulty::Hard => HARD_WORDS,
            };

            let (mut words_to_type, mut user_typed_words) = match game_mode {
                config::GameMode::Words => {
                    let w: Vec<&str> = current_word_list.choose_multiple(&mut rng, num_words).cloned().collect();
                    let u = vec![String::new(); w.len()];
                    (w, u)
                }
                config::GameMode::Time => {
                    let mut word_pool: Vec<&str> = Vec::new();
                    for _ in 0..10 {
                        word_pool.extend(current_word_list.choose_multiple(&mut rng, current_word_list.len()).cloned());
                    }
                    let u = vec![String::new(); word_pool.len()];
                    (word_pool, u)
                }
            };

            let mut current_word_index = 0;
            let mut start_time: Option<Instant> = None;
            let mut last_wpm_update: Option<Instant> = None;
            let mut wpm = 0.0;

            loop {
                if !running.load(Ordering::SeqCst) {
                    break;
                }

                let mut game_over = false;
                match game_mode {
                    config::GameMode::Time => {
                        if let Some(start) = start_time {
                            if start.elapsed().as_secs() >= time_limit {
                                game_over = true;
                            }
                        }
                    }
                    config::GameMode::Words => {
                        if current_word_index >= num_words {
                            game_over = true;
                        }
                    }
                }
                if game_over {
                    break;
                }

                let (width, height) = terminal::size()?;

                if last_wpm_update.is_none() || last_wpm_update.unwrap().elapsed().as_secs() >= 1 {
                    let correct_chars_total: usize = user_typed_words
                        .iter()
                        .zip(words_to_type.iter())
                        .map(|(typed, original)| {
                            typed
                                .chars()
                                .zip(original.chars())
                                .filter(|(a, b)| a == b)
                                .count()
                        })
                        .sum();

                    let elapsed_seconds = if let Some(start) = start_time {
                        start.elapsed().as_secs_f64()
                    } else {
                        0.0
                    };

                    let cpm = if elapsed_seconds > 0.0 {
                        (correct_chars_total as f64 / elapsed_seconds) * 60.0
                    } else {
                        0.0
                    };
                    wpm = cpm / 5.0;
                    last_wpm_update = Some(Instant::now());
                }

                stdout
                    .execute(cursor::MoveTo(0, 2))?
                    .execute(terminal::Clear(terminal::ClearType::FromCursorDown))?;
                match layout_theme {
                    config::LayoutTheme::Default => {
                        let text_block = words_to_type.join(" ");
                        let text_width = text_block.len() as u16;
                        let start_x = (width.saturating_sub(text_width)) / 2;
                        let start_y = height / 2;

                        let top_bar_text = match game_mode {
                            config::GameMode::Time => {
                                let elapsed = start_time.map_or(0, |s| s.elapsed().as_secs());
                                let remaining = time_limit.saturating_sub(elapsed);
                                format!("WPM: {:.2} | Time: {}", wpm, remaining)
                            }
                            config::GameMode::Words => format!("WPM: {:.2}", wpm),
                        };

                        stdout
                            .execute(cursor::MoveTo(start_x, start_y - 2))?
                            .execute(Print(top_bar_text))?;

                        let mut x = start_x;
                        let mut y = start_y;

                        for (i, word) in words_to_type.iter().enumerate() {
                            let word_len = word.len() as u16;
                            if x + word_len > width {
                                y += 2;
                                x = start_x;
                            }

                            if i == current_word_index {
                                let typed_text = &user_typed_words[i];
                                for (char_i, char) in word.chars().enumerate() {
                                    if char_i < typed_text.len() {
                                        if typed_text.chars().nth(char_i).unwrap() == char {
                                            stdout.execute(SetForegroundColor(Color::from(
                                                config.color_theme.correct,
                                            )))?;
                                        } else {
                                            stdout.execute(SetForegroundColor(Color::from(
                                                config.color_theme.incorrect,
                                            )))?;
                                        }
                                    } else {
                                        stdout.execute(SetForegroundColor(Color::from(
                                            config.color_theme.default,
                                        )))?;
                                    }
                                    stdout
                                        .execute(cursor::MoveTo(x + char_i as u16, y))?
                                        .execute(Print(char))?;
                                }
                                if typed_text.len() > word.len() {
                                    stdout.execute(SetForegroundColor(Color::from(
                                        config.color_theme.incorrect,
                                    )))?;
                                    for (char_i, char) in
                                        typed_text.chars().skip(word.len()).enumerate()
                                    {
                                        stdout
                                            .execute(cursor::MoveTo(
                                                x + word.len() as u16 + char_i as u16,
                                                y,
                                            ))?
                                            .execute(Print(char))?;
                                    }
                                }
                            } else {
                                let typed_word = &user_typed_words[i];
                                for (char_i, original_char) in word.chars().enumerate() {
                                    let color = if char_i < typed_word.len() {
                                        if typed_word.chars().nth(char_i).unwrap() == original_char {
                                            Color::from(config.color_theme.correct)
                                        } else {
                                            Color::from(config.color_theme.incorrect)
                                        }
                                    } else {
                                        Color::DarkGrey
                                    };
                                    stdout
                                        .execute(SetForegroundColor(color))?
                                        .execute(cursor::MoveTo(x + char_i as u16, y))?
                                        .execute(Print(original_char))?;
                                }
                            }
                            x += word_len + 1;
                        }
                    }
                    config::LayoutTheme::Boxes => {
                        let box_width = (width as f32 * 0.8).max(40.0) as u16;
                        let box_start_x = (width - box_width) / 2;

                        // --- WPM/Timer Box ---
                        let top_bar_text = match game_mode {
                            config::GameMode::Time => {
                                let elapsed = start_time.map_or(0, |s| s.elapsed().as_secs());
                                let remaining = time_limit.saturating_sub(elapsed);
                                format!("WPM: {:.2} | Time: {}", wpm, remaining)
                            }
                            config::GameMode::Words => format!("WPM: {:.2}", wpm),
                        };
                        let wpm_box_start_y: u16 = 2;
                        let wpm_box_content_x = box_start_x + 2;
                        let wpm_box_content_y = wpm_box_start_y + 1;

                        stdout
                            .execute(cursor::MoveTo(box_start_x, wpm_box_start_y))?
                            .execute(Print("┌".to_string() + &"─".repeat((box_width - 2) as usize) + "┐"))?;
                        stdout
                            .execute(cursor::MoveTo(box_start_x, wpm_box_start_y + 1))?
                            .execute(Print("│".to_string() + &" ".repeat((box_width - 2) as usize) + "│"))?;
                        stdout
                            .execute(cursor::MoveTo(box_start_x, wpm_box_start_y + 2))?
                            .execute(Print("└".to_string() + &"─".repeat((box_width - 2) as usize) + "┘"))?;
                        stdout
                            .execute(cursor::MoveTo(wpm_box_content_x, wpm_box_content_y))?
                            .execute(Print(top_bar_text))?;

                        // --- Main Text Box ---
                        let main_box_start_y: u16 = wpm_box_start_y + 4;
                        let text_area_start_x = box_start_x + 2;
                        let text_area_width = box_width - 4;
                        
                        let mut temp_x = 0;
                        let mut num_lines = 1;
                        for word in words_to_type.iter() {
                            let word_len = word.len() as u16;
                            if temp_x + word_len > text_area_width {
                                num_lines += 1;
                                temp_x = 0;
                            }
                            temp_x += word_len + 1;
                        }

                        let main_box_height = num_lines + 1;

                        stdout
                            .execute(cursor::MoveTo(box_start_x, main_box_start_y))?
                            .execute(Print("┌".to_string() + &"─".repeat((box_width - 2) as usize) + "┐"))?;
                        for i in 0..main_box_height {
                            stdout
                                .execute(cursor::MoveTo(box_start_x, main_box_start_y + 1 + i))?
                                .execute(Print("│".to_string() + &" ".repeat((box_width - 2) as usize) + "│"))?;
                        }
                        stdout
                            .execute(cursor::MoveTo(box_start_x, main_box_start_y + main_box_height + 1))?
                            .execute(Print("└".to_string() + &"─".repeat((box_width - 2) as usize) + "┘"))?;

                        // --- Render Text Inside Box ---
                        let mut x = text_area_start_x;
                        let mut y = main_box_start_y + 1;

                        for (i, word) in words_to_type.iter().enumerate() {
                            let word_len = word.len() as u16;
                            if x + word_len > text_area_start_x + text_area_width {
                                y += 1;
                                x = text_area_start_x;
                            }

                            if i == current_word_index {
                                let typed_text = &user_typed_words[i];
                                for (char_i, char) in word.chars().enumerate() {
                                    if char_i < typed_text.len() {
                                        if typed_text.chars().nth(char_i).unwrap() == char {
                                            stdout.execute(SetForegroundColor(Color::from(config.color_theme.correct)))?;
                                        } else {
                                            stdout.execute(SetForegroundColor(Color::from(config.color_theme.incorrect)))?;
                                        }
                                    } else {
                                        stdout.execute(SetForegroundColor(Color::from(config.color_theme.default)))?;
                                    }
                                    stdout.execute(cursor::MoveTo(x + char_i as u16, y))?.execute(Print(char))?;
                                }
                                if typed_text.len() > word.len() {
                                    stdout.execute(SetForegroundColor(Color::from(config.color_theme.incorrect)))?;
                                    for (char_i, char) in typed_text.chars().skip(word.len()).enumerate() {
                                        stdout.execute(cursor::MoveTo(x + word.len() as u16 + char_i as u16, y))?.execute(Print(char))?;
                                    }
                                }
                            } else {
                                let typed_word = &user_typed_words[i];
                                for (char_i, original_char) in word.chars().enumerate() {
                                    let color = if char_i < typed_word.len() {
                                        if typed_word.chars().nth(char_i).unwrap() == original_char {
                                            Color::from(config.color_theme.correct)
                                        } else {
                                            Color::from(config.color_theme.incorrect)
                                        }
                                    } else {
                                        Color::DarkGrey
                                    };
                                    stdout.execute(SetForegroundColor(color))?.execute(cursor::MoveTo(x + char_i as u16, y))?.execute(Print(original_char))?;
                                }
                            }
                            x += word_len + 1;
                        }
                    }
                }

                stdout.execute(ResetColor)?;

                let cursor_x;
                let cursor_y;

                match layout_theme {
                    config::LayoutTheme::Default => {
                        let text_block = words_to_type.join(" ");
                        let text_width = text_block.len() as u16;
                        let start_x = (width.saturating_sub(text_width)) / 2;
                        let start_y = height / 2;

                        let mut x = start_x;
                        let mut y = start_y;

                        // Recalculate position considering wrapping
                        for word in words_to_type.iter().take(current_word_index) {
                            let word_len = word.len() as u16;
                            if x + word_len > width {
                                y += 2; // The original code did this
                                x = start_x;
                            }
                            x += word_len + 1;
                        }
                        cursor_x = x + user_typed_words[current_word_index].len() as u16;
                        cursor_y = y;
                    }
                    config::LayoutTheme::Boxes => {
                        let box_width = (width as f32 * 0.8).max(40.0) as u16;
                        let box_start_x = (width - box_width) / 2;
                        let wpm_box_start_y: u16 = 2;
                        let main_box_start_y: u16 = wpm_box_start_y + 4;
                        let text_area_start_x = box_start_x + 2;
                        let text_area_width = box_width - 4;

                        let mut x = text_area_start_x;
                        let mut y = main_box_start_y + 1;

                        for word in words_to_type.iter().take(current_word_index) {
                            let word_len = word.len() as u16;
                            if x + word_len > text_area_start_x + text_area_width {
                                y += 1;
                                x = text_area_start_x;
                            }
                            x += word_len + 1;
                        }
                        cursor_x = x + user_typed_words[current_word_index].len() as u16;
                        cursor_y = y;
                    }
                };

                stdout
                    .execute(cursor::MoveTo(cursor_x, cursor_y))?
                    .execute(cursor::Show)?;

                if event::poll(std::time::Duration::from_millis(50))? {
                    if let Event::Key(key_event) = event::read()? {
                        match key_event.code {
                            KeyCode::Char(' ') => {
                                if current_word_index < words_to_type.len() - 1 {
                                    current_word_index += 1;

                                    if let config::GameMode::Time = game_mode {
                                        if words_to_type.len() - current_word_index < 10 {
                                            let mut new_words: Vec<&str> = current_word_list.choose_multiple(&mut rng, 20).cloned().collect();
                                            words_to_type.append(&mut new_words);
                                            user_typed_words.resize(words_to_type.len(), String::new());
                                        }
                                    }
                                }
                            }
                            KeyCode::Char(c) => {
                                if start_time.is_none() {
                                    start_time = Some(Instant::now());
                                }
                                user_typed_words[current_word_index].push(c);
                                if let config::GameMode::Words = game_mode {
                                    if current_word_index == num_words - 1
                                        && user_typed_words[current_word_index]
                                            == words_to_type[current_word_index]
                                    {
                                        break;
                                    }
                                }
                            }
                            KeyCode::Backspace => {
                                user_typed_words[current_word_index].pop();
                            }
                            KeyCode::Tab => {
                                if config.restart_button {
                                    // Restart the test
                                    words_to_type = current_word_list
                                        .choose_multiple(&mut rng, num_words)
                                        .cloned()
                                        .collect();
                                    user_typed_words = vec![String::new(); words_to_type.len()];
                                    current_word_index = 0;
                                    start_time = None;
                                    last_wpm_update = None;
                                    wpm = 0.0;
                                }
                            }
                            KeyCode::Esc => {
                                break; // Exit test and go to results screen
                            },
                            _ => {}
                        }
                    }
                }

                if current_word_index >= words_to_type.len() {
                    break;
                }
            }

            let duration = match game_mode {
                config::GameMode::Time => time_limit as f64,
                config::GameMode::Words => start_time.map_or(0.0, |s| s.elapsed().as_secs_f64()),
            };

            let (correct_chars_total, incorrect_chars_total) = user_typed_words
                .iter()
                .zip(words_to_type.iter())
                .take(current_word_index + 1)
                .fold((0, 0), |(mut c, mut i), (typed, original)| {
                    for (tc, oc) in typed.chars().zip(original.chars()) {
                        if tc == oc {
                            c += 1;
                        } else {
                            i += 1;
                        }
                    }
                    if typed.len() > original.len() {
                        i += typed.len() - original.len();
                    }
                    (c, i)
                });

            let final_wpm = if duration > 0.0 {
                (correct_chars_total as f64 / 5.0) / (duration / 60.0)
            } else {
                0.0
            };

            let accuracy = if (correct_chars_total + incorrect_chars_total) == 0 {
                100.0
            } else {
                (correct_chars_total as f64 / (correct_chars_total + incorrect_chars_total) as f64)
                    * 100.0
            };

            let test_result = config::TestResult {
                wpm: final_wpm,
                accuracy,
                timestamp: Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
            };

            let key = match config.game_mode {
                config::GameMode::Words => format!("words_{}_{:?}", config.default_test_length, config.word_list_difficulty),
                config::GameMode::Time => format!("time_{}_{:?}", config.default_time_limit, config.word_list_difficulty),
            };
            config.results.entry(key).or_insert_with(Vec::new).push(test_result);
            config::save_config(&config)?;

            stdout.execute(terminal::Clear(terminal::ClearType::All))?;
            let results = vec![
                "Typing test complete!".to_string(),
                format!("WPM: {:.2}", final_wpm),
                format!("Accuracy: {:.2}%", accuracy),
                "".to_string(),
                "Press 'Tab' to restart or 'Esc' to exit.".to_string(),
            ];

            let (width, height) = terminal::size()?;
            for (i, line) in results.iter().enumerate() {
                let x = (width.saturating_sub(line.len() as u16)) / 2;
                let y = (height / 2) + i as u16;
                stdout.execute(cursor::MoveTo(x, y))?.execute(Print(line))?;
            }

            loop {
                if let Event::Key(key_event) = event::read()? {
                    match key_event.code {
                        KeyCode::Tab => {
                            break;
                        }
                        KeyCode::Esc => {
                            running.store(false, Ordering::SeqCst);
                            break;
                        }
                        _ => {}
                    }
                }
            }
            Ok(())
        })() {
            Ok(_) => {},
            Err(e) => return Err(e),
        }
    }
    terminal::disable_raw_mode()?;
    stdout.execute(LeaveAlternateScreen)?;
    Ok(())
}