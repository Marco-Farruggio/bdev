use std::time::Duration;

use ratatui::{
    DefaultTerminal, Frame,
    crossterm::event::{self, Event, KeyCode},
    widgets::{Block, Borders, BorderType, Paragraph},
    layout::{Constraint, Layout},
    style::{Modifier, Style},
    text::{Line, Span},
};

use crate::{
    disks::{Disk, read_block_devices},
};

pub fn spans_from_hotkey_word<'a>(letter: &'a str, rest: &'a str) -> Vec<Span<'a>> {
    vec![
        Span::styled(letter, Style::default().add_modifier(Modifier::UNDERLINED | Modifier::BOLD)),
        Span::raw(rest)
    ]
}

pub enum SelectedMenu {
    Devices,
    Settings,
    Help
}

struct App {
    view: SelectedMenu,
    disks: Vec<Disk>,
    selected_name: Option<String>,
    highlighted_name: Option<String>,
}

impl App {
    fn new() -> Self {
        let disks = read_block_devices();
        let highlighted_name = flatten(&disks).into_iter().next();
        Self {
            view: SelectedMenu::Devices,
            disks: read_block_devices(),
            selected_name: None,
            highlighted_name,
        }
    }

    fn refresh(&mut self) {
        self.disks = read_block_devices();
    }

    /// flatten the list of devices, find the currently selected one,
    /// and select the next one (wrapping)
    /// 
    /// if we cant find the currently selected one, or there is none selected
    /// (impossible, for now) then we simply (try to) select the first one (idx 0)
    fn highlight_next(&mut self) {
        let rows = flatten(&self.disks);
        if rows.is_empty() {
            return;
        }
        if let Some(highlighted_name) = &self.highlighted_name {
            match rows.iter().position(|r| r == highlighted_name) {
                Some(i) => {
                    if i == rows.len() - 1 {
                        // we're currently highlighting the last row,
                        // so loop back round to the start
                        //
                        // safety: rows is guarded to be of len >= 1
                        self.highlighted_name = Some(rows[0].clone());
                    } else {
                        // safety: rows is guarded to be atleast 1 less than rows.len()
                        self.highlighted_name = Some(rows[i + 1].clone());
                    }
                }
                None => {
                    // The highlighted device was removed (after a refresh),
                    // so we go back to selecting device 0, this is panic-safe
                    // as we assured above that rows is not empty
                    //
                    // safety: rows is guarded to be of len >= 1
                    self.highlighted_name = Some(rows[0].clone());
                }
            }
        } else {
            // safety: rows is guarded to be of len >= 1
            self.highlighted_name = Some(rows[0].clone());
        }
    }

    fn highlight_previous(&mut self) {
        let rows = flatten(&self.disks);
        if rows.is_empty() {
            return;
        }
        if let Some(highlighted_name) = &self.highlighted_name {
            match rows.iter().position(|r| r == highlighted_name) {
                Some(i) => {
                    if i == 0 {
                        // we're currently highlighting the last row,
                        // so loop back round to the start
                        //
                        // safety: rows is guarded to be of len >= 1
                        self.highlighted_name = Some(rows[rows.len() - 1].clone());
                    } else {
                        // safety: rows is guarded to be atleast 1 less than rows.len()
                        self.highlighted_name = Some(rows[i - 1].clone());
                    }
                }
                None => {
                    // The highlighted device was removed (after a refresh),
                    // so we go back to selecting device 0, this is panic-safe
                    // as we assured above that rows is not empty
                    //
                    // safety: rows is guarded to be of len >= 1
                    self.highlighted_name = Some(rows[0].clone());
                }
            }
        } else {
            // safety: rows is guarded to be of len >= 1
            self.highlighted_name = Some(rows[0].clone());
        }
    }
}

pub fn run(terminal: &mut DefaultTerminal) -> std::io::Result<()> {
    let mut app = App::new();

    loop {
        terminal.draw(|frame| draw(frame, &mut app))?;

        if event::poll(Duration::from_millis(250))? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('q') | KeyCode::Esc => return Ok(()),
                    KeyCode::Char('r') => app.refresh(),
                    KeyCode::Char('s') => app.view = SelectedMenu::Settings,
                    KeyCode::Char('h') => app.view = SelectedMenu::Help,
                    KeyCode::Char('d') => app.view = SelectedMenu::Devices,
                    KeyCode::Down => app.highlight_next(),
                    KeyCode::Up => app.highlight_previous(),
                    KeyCode::Enter => app.selected_name = app.highlighted_name.clone(),
                    _ => {}
                }
            }
        }
    }
}

/// flatts the disk/partition tree into the same order which `draw()`
/// would render, so that both the selection indici always lines up
/// with whats being disdplayed
fn flatten(disks: &[Disk]) -> Vec<String> {
    let mut rows = Vec::new();
    for disk in disks {
        rows.push(disk.name.clone());
        for part in &disk.partitions {
            rows.push(part.name.clone());
        }
    }
    rows
}

fn draw(frame: &mut Frame, app: &mut App) {
    let area = frame.area();

    let [body, footer] = Layout::vertical([
        Constraint::Min(0),
        Constraint::Length(1),
    ])
    .areas(area);


    let mut hotkey_bar_spans = Vec::new();

    hotkey_bar_spans.extend(spans_from_hotkey_word("Q", "uit"));
    hotkey_bar_spans.push(Span::raw(" | "));
    hotkey_bar_spans.extend(spans_from_hotkey_word("W", "rite"));
    hotkey_bar_spans.push(Span::raw(" | "));
    hotkey_bar_spans.extend(spans_from_hotkey_word("R", "efresh"));
    hotkey_bar_spans.push(Span::raw(" | ↑/↓: Select | Enter: Confirm | Esc: Back"));

    frame.render_widget(
        Line::from(hotkey_bar_spans).centered(),
        footer,
    );

    match app.view {
        SelectedMenu::Devices => {
            let mut items: Vec<Line> = Vec::new();
            for disk in &app.disks {
                let size_display = if let Some(size) = disk.size {
                    crate::disks::format_nbytes(size)
                } else {
                    "UNK".to_string()   
                };

                // maybe highlighted doesnt have to be an option?

                let sel = if let Some(selected) = &app.selected_name {
                    &disk.name == selected
                } else {
                    false
                };

                let hov = if let Some(highlighted) = &app.highlighted_name {
                    &disk.name == highlighted
                } else {
                    false
                };

                let drive_display_name = format!("{} {size_display} [{}]", disk.name.as_str(), crate::partitions::maybe_scheme_to_str(&disk.partition_scheme));
                let mut drive_style = Style::new().bold();
                
                if hov {
                    drive_style = drive_style.add_modifier(Modifier::REVERSED);
                }

                items.push(Line::from(drive_display_name).style(drive_style));
                for (i, part) in disk.partitions.iter().enumerate() {
                    let sel = if let Some(selected) = &app.selected_name {
                        &part.name == selected
                    } else {
                        false
                    };

                    let hov = if let Some(highlighted) = &app.highlighted_name {
                        &part.name == highlighted
                    } else {
                        false
                    };

                    let mut part_style = Style::new();

                    if hov {
                        part_style = part_style.add_modifier(Modifier::REVERSED);
                    }

                    let branch = if i + 1 == disk.partitions.len() {
                        "╰─"
                    } else {
                        "├─"
                    };
                    let part_size = if let Some(size) = part.size {
                        crate::disks::format_nbytes(size)
                    } else {
                        "UNK".to_string()
                    };
                    items.push(Line::from(format!("  {branch} {} {part_size}", part.name)).style(part_style));
                }
            }

            let list = Paragraph::new(items)
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .border_type(BorderType::Rounded)
                        .title(Line::from(spans_from_hotkey_word("D", "evices")).centered())
                        .title(Line::from(spans_from_hotkey_word("S", "ettings")).centered())
                        .title(Line::from(spans_from_hotkey_word("H", "elp")).centered())
                        .title_bottom(Line::from("bdev v0.1.0").right_aligned())
                );

            frame.render_widget(list, body);
        }
        SelectedMenu::Settings => {

        }
        SelectedMenu::Help => {

        }
    }
}