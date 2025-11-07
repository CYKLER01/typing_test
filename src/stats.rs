use crate::config::{self, Config, TestResult};
use crossterm::{
    cursor,
    event::{self, Event, KeyCode},
    style::{Print, Stylize},
    terminal::{self, Clear, ClearType, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use std::io::{self, Stdout, Write};

struct StatsState {
    config: Config,
    selected_mode: usize,
    view_mode: ViewMode,
}

enum ViewMode {
    Table,
    Graph,
}

pub fn show_stats() -> io::Result<()> {
    let mut stdout = io::stdout();
    stdout.execute(EnterAlternateScreen)?;
    terminal::enable_raw_mode()?;

    let mut state = StatsState {
        config: config::load_config(),
        selected_mode: 0,
        view_mode: ViewMode::Table,
    };

    loop {
        draw_stats(&mut stdout, &state)?;

        if let Event::Key(key_event) = event::read()? {
            match key_event.code {
                KeyCode::Char('q') => break,
                KeyCode::Up => {
                    state.selected_mode = state.selected_mode.saturating_sub(1);
                }
                KeyCode::Down => {
                    let num_modes = state.config.results.len();
                    if num_modes > 0 {
                        state.selected_mode = (state.selected_mode + 1).min(num_modes - 1);
                    }
                }
                KeyCode::Char('t') => state.view_mode = ViewMode::Table,
                KeyCode::Char('g') => state.view_mode = ViewMode::Graph,
                _ => {}
            }
        }
    }

    terminal::disable_raw_mode()?;
    stdout.execute(LeaveAlternateScreen)?;
    Ok(())
}

fn draw_stats(stdout: &mut Stdout, state: &StatsState) -> io::Result<()> {
    stdout.execute(Clear(ClearType::All))?;
    let (width, height) = terminal::size()?;

    let title = "Saved Stats";
    let title_x = (width - title.len() as u16) / 2;
    stdout
        .execute(cursor::MoveTo(title_x, 1))?
        .execute(Print(title.bold()))?;

    let instructions = "Use ↑/↓ to select mode, 't' for table, 'g' for graph, 'q' to quit.";
    let inst_x = (width - instructions.len() as u16) / 2;
    stdout
        .execute(cursor::MoveTo(inst_x, height - 2))?
        .execute(Print(instructions.dark_grey()))?;

    if state.config.results.is_empty() {
        let no_stats = "No stats saved yet.";
        let no_stats_x = (width - no_stats.len() as u16) / 2;
        stdout
            .execute(cursor::MoveTo(no_stats_x, height / 2))?
            .execute(Print(no_stats))?;
        return stdout.flush();
    }

    let mut y = 4;
    let mut mode_keys: Vec<_> = state.config.results.keys().collect();
    mode_keys.sort();

    for (i, key) in mode_keys.iter().enumerate() {
        let display_key = key.replace("_", " ").to_uppercase();
        if i == state.selected_mode {
            stdout
                .execute(cursor::MoveTo(5, y))?
                .execute(Print(display_key.negative()))?;
            y += 2;
            match state.view_mode {
                ViewMode::Table => {
                    y = draw_table(stdout, state.config.results.get(*key).unwrap(), y)?;
                }
                ViewMode::Graph => {
                    y = draw_graph(stdout, state.config.results.get(*key).unwrap(), y, width - 10)?;
                }
            }
        } else {
            stdout
                .execute(cursor::MoveTo(5, y))?
                .execute(Print(display_key))?;
        }
        y += 2;
    }

    stdout.flush()
}

fn draw_table(stdout: &mut Stdout, results: &[TestResult], start_y: u16) -> io::Result<u16> {
    let mut y = start_y;
    let header = format!(
        "{: <25} | {: <10} | {: <10}",
        "Timestamp", "WPM", "Accuracy"
    );
    stdout
        .execute(cursor::MoveTo(7, y))?
        .execute(Print(header.bold()))?;
    y += 1;

    for result in results.iter().rev().take(5) {
        let line = format!(
            "{: <25} | {: <10.2} | {: <9.2}%",
            result.timestamp, result.wpm, result.accuracy
        );
        stdout.execute(cursor::MoveTo(7, y))?.execute(Print(line))?;
        y += 1;
    }
    Ok(y)
}

fn draw_graph(stdout: &mut Stdout, results: &[TestResult], start_y: u16, width: u16) -> io::Result<u16> {
    let y = start_y;
    if results.is_empty() {
        return Ok(y);
    }

    let max_wpm = results.iter().map(|r| r.wpm).fold(0.0, f64::max);
    let graph_height = 10;
    let graph_width = width.min(results.len() as u16);

    let mut points: Vec<(u16, u16)> = Vec::new();
    if !results.is_empty() {
        for (i, result) in results.iter().enumerate().take(graph_width as usize) {
            let x = i as u16;
            let y_pos = if max_wpm > 0.0 {
                (result.wpm / max_wpm * (graph_height as f64)) as u16
            } else {
                0
            };
            points.push((x, graph_height - y_pos));
        }
    }

    for gy in 0..=graph_height {
        stdout.execute(cursor::MoveTo(7, y + gy))?;
        for gx in 0..graph_width {
            let mut printed = false;
            for i in 0..points.len() {
                if i + 1 < points.len() {
                    let p1 = points[i];
                    let p2 = points[i+1];
                    if (p1.0..=p2.0).contains(&gx) || (p2.0..=p1.0).contains(&gx) {
                        let y1 = p1.1 as f32;
                        let y2 = p2.1 as f32;
                        let x1 = p1.0 as f32;
                        let x2 = p2.0 as f32;

                        let slope = (y2 - y1) / (x2 - x1);
                        let expected_y = y1 + slope * (gx as f32 - x1);

                        if (expected_y.round() as u16) == gy {
                            stdout.execute(Print("*".red()))?;
                            printed = true;
                            break;
                        }
                    }
                }
            }
            if !printed {
                 if points.contains(&(gx, gy)) {
                    stdout.execute(Print("*".red()))?;
                } else {
                    stdout.execute(Print(" "))?;
                }
            }
        }
    }
    
    // Draw Y-axis labels
    stdout.execute(cursor::MoveTo(2, y))?.execute(Print(format!("{:.0}", max_wpm)))?;
    stdout.execute(cursor::MoveTo(2, y + graph_height))?.execute(Print("0".to_string()))?;


    Ok(y + graph_height + 2)
}