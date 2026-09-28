use std::{f32::consts::E, time::Duration};

use ratatui::{
    DefaultTerminal, Frame, crossterm::{event::{self, Event, KeyCode}, style}, layout::{Constraint, Layout}, style::{Modifier, Style}, text::{Line, Span}, widgets::{Block, BorderType, Borders, Paragraph},
};

use crate::{
    disks::{Disk, read_block_devices},
    commands::Command,
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
    commands: Vec<Command>,
    disks: Vec<Disk>,
    hovered_dev: Option<String>,
}

impl App {
    fn new() -> Self {
        let disks = read_block_devices();
        let highlighted_name = flatten(&disks).into_iter().next();
        Self {
            view: SelectedMenu::Devices,
            commands: Vec::new(),
            disks,
            hovered_dev: highlighted_name,
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
        if let Some(highlighted_name) = &self.hovered_dev {
            match rows.iter().position(|r| r == highlighted_name) {
                Some(i) => {
                    if i == rows.len() - 1 {
                        // we're currently highlighting the last row,
                        // so loop back round to the start
                        //
                        // safety: rows is guarded to be of len >= 1
                        self.hovered_dev = Some(rows[0].clone());
                    } else {
                        // safety: rows is guarded to be atleast 1 less than rows.len()
                        self.hovered_dev = Some(rows[i + 1].clone());
                    }
                }
                None => {
                    // The highlighted device was removed (after a refresh),
                    // so we go back to selecting device 0, this is panic-safe
                    // as we assured above that rows is not empty
                    //
                    // safety: rows is guarded to be of len >= 1
                    self.hovered_dev = Some(rows[0].clone());
                }
            }
        } else {
            // safety: rows is guarded to be of len >= 1
            self.hovered_dev = Some(rows[0].clone());
        }
    }

    fn highlight_previous(&mut self) {
        let rows = flatten(&self.disks);
        if rows.is_empty() {
            return;
        }
        if let Some(highlighted_name) = &self.hovered_dev {
            match rows.iter().position(|r| r == highlighted_name) {
                Some(i) => {
                    if i == 0 {
                        // we're currently highlighting the last row,
                        // so loop back round to the start
                        //
                        // safety: rows is guarded to be of len >= 1
                        self.hovered_dev = Some(rows[rows.len() - 1].clone());
                    } else {
                        // safety: rows is guarded to be atleast 1 less than rows.len()
                        self.hovered_dev = Some(rows[i - 1].clone());
                    }
                }
                None => {
                    // The highlighted device was removed (after a refresh),
                    // so we go back to selecting device 0, this is panic-safe
                    // as we assured above that rows is not empty
                    //
                    // safety: rows is guarded to be of len >= 1
                    self.hovered_dev = Some(rows[0].clone());
                }
            }
        } else {
            // safety: rows is guarded to be of len >= 1
            self.hovered_dev = Some(rows[0].clone());
        }
    }
}

use std::collections::HashSet;
use std::hash::Hash;

/// Removes duplicate elements from `items`, preserving the order of
/// first occurrence. Duplicates don't need to be consecutive (unlike
/// `Vec::dedup`, which only removes adjacent duplicates).
fn dedup_preserve_order<T: Eq + Hash + Clone>(items: Vec<T>) -> Vec<T> {
    let mut seen = HashSet::new();
    items
        .into_iter()
        .filter(|item| seen.insert(item.clone()))
        .collect()
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
                    KeyCode::Backspace => {
                        if let Some(hovered) = &app.hovered_dev {
                            if crate::parts::is_partition(&hovered) {
                                app.commands.push(Command::DeletePartition { name: hovered.clone() })
                            } else {
                                app.commands.push(Command::DeletePartitionTable { name: hovered.clone() })
                            }

                            app.commands = dedup_preserve_order(app.commands);
                        }
                    }
                    KeyCode::Down => app.highlight_next(),
                    KeyCode::Up => app.highlight_previous(),
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

    let [body, footer, debug] = Layout::vertical([
        Constraint::Min(0),
        Constraint::Length(1),
        Constraint::Length(3),
    ])
    .areas(area);

    frame.render_widget(Line::from(format!("{:?}", app.commands)), debug);


    let mut hotkey_bar_spans = Vec::new();

    hotkey_bar_spans.extend(spans_from_hotkey_word("Q", "uit"));
    hotkey_bar_spans.push(Span::raw(" | "));
    hotkey_bar_spans.extend(spans_from_hotkey_word("W", "rite"));
    hotkey_bar_spans.push(Span::raw(" | "));
    hotkey_bar_spans.extend(spans_from_hotkey_word("R", "efresh"));
    hotkey_bar_spans.push(Span::raw(" | "));
    hotkey_bar_spans.extend(spans_from_hotkey_word("F", "ormat"));
    hotkey_bar_spans.push(Span::raw(" | ↑/↓: Select | ⌫: Delete"));

    frame.render_widget(
        Line::from(hotkey_bar_spans).centered(),
        footer,
    );

    match app.view {
        SelectedMenu::Devices => {
            let mut rows: Vec<Line> = Vec::new();
            for disk in &app.disks {
                let size_display = if let Some(size) = disk.size {
                    crate::disks::format_nbytes(size)
                } else {
                    "UNK".to_string()   
                };

                // maybe highlighted doesnt have to be an option?
                let hov = app.hovered_dev.as_ref().map(|h| h == &disk.name).unwrap_or(false);

                let drive_display_name = format!("{} {size_display} [{}]", disk.name.as_str(), crate::parts::maybe_scheme_to_str(&disk.partition_scheme));
                let mut drive_style = Style::new().bold();
                
                if hov {
                    drive_style = drive_style.add_modifier(Modifier::REVERSED);
                }

                rows.push(Line::from(drive_display_name).style(drive_style));
                for (i, part) in disk.partitions.iter().enumerate() {
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
                    
                    let hov = app.hovered_dev.as_ref().map(|h| h == &part.name).unwrap_or(false);

                    let mut part_style = Style::new();
                    let mut old_style = part_style
                        .fg(ratatui::style::Color::Red);

                    let mut new_style = part_style
                        .fg(ratatui::style::Color::Green);
                    // with the fg/bg inversion ^ for now
                    
                    if hov {
                        part_style = part_style.add_modifier(Modifier::REVERSED);
                        old_style = old_style.add_modifier(Modifier::REVERSED);
                        new_style = new_style.add_modifier(Modifier::REVERSED);
                    }

                    let modified = app.commands.iter().any(|c| c.relates_to(&part.name));
                    
                    if !modified {
                        rows.push(Line::from(format!("  {branch} {} {part_size}", part.name)).style(part_style));
                    } else {
                        for command in &app.commands {
                            if command.relates_to(&part.name) {
                                match command {
                                    Command::DeletePartition { .. } => {
                                        rows.push(Line::from(format!("  {branch} {} {part_size}", part.name)).style(old_style));
                                    }
                                    _ => {}
                                }
                            }
                        }
                    }

                }
            }

            let list = Paragraph::new(rows)
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