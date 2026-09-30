use std::time::Duration;

use ratatui::{
    DefaultTerminal,
    Frame,
    crossterm::{
        event::{
            self,
            Event,
            KeyCode,
        },
    },
    layout::{
        Constraint,
        Layout,
    },
    style::{
        Modifier,
        Style,
    },
    text::{
        Line,
        Span,
    },
    widgets::{
        Block,
        BorderType,
        Borders,
        Paragraph,
    }
};

use crate::{
    disks::{Disk, read_block_devices},
    commands::{Command, Change},
    parts::PartitionScheme,
    filesys::FileSystem,
};

pub fn spans_from_hotkey_word<'a>(letter: &'a str, rest: &'a str) -> Vec<Span<'a>> {
    vec![
        Span::styled(letter, Style::default().add_modifier(Modifier::UNDERLINED | Modifier::BOLD)),
        Span::raw(rest)
    ]
}

pub enum SelectedMenu {
    Changes,
    Settings,
    Help
}

struct App {
    menu: SelectedMenu,
    hovered_dev: Option<String>,
    commands: Vec<Command>,
    old_disks: Vec<Disk>,
    new_disks: Vec<Disk>,
}

impl App {
    /// Selection logic:
    /// 
    /// References by name, but if it can't be found, it falls back
    /// to the index into the list of rows, if the index is greater
    /// than the list of devices, it shoots back to the highest index
    /// row, and if there are no devices (impossible?)
    /// then it simply sets it to None
    fn new() -> Self {
        let disks = read_block_devices();
        let highlighted_name = flatten(&disks).into_iter().next();
        Self {
            menu: SelectedMenu::Changes,
            commands: Vec::new(),
            old_disks: disks.clone(),
            new_disks: disks,
            hovered_dev: highlighted_name,
        }
    }

    fn refresh(&mut self) {
        self.old_disks = read_block_devices();
        self.new_disks = self.old_disks.clone();
        for command in self.commands.iter_mut() {
            command.try_mem_apply(&mut self.new_disks);
        }
    }

    fn apply_change(&mut self, mut command: Command) {
        command.try_mem_apply(&mut self.new_disks);
        self.commands.push(command);
    }

    /// flatten the list of devices, find the currently selected one,
    /// and select the next one (wrapping)
    /// 
    /// if we cant find the currently selected one, or there is none selected
    /// (impossible, for now) then we simply (try to) select the first one (idx 0)
    fn highlight_next(&mut self) {
        let rows = flatten(&self.new_disks);
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
        let rows = flatten(&self.new_disks);
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
                    KeyCode::Char('s') => app.menu = SelectedMenu::Settings,
                    KeyCode::Char('h') => app.menu = SelectedMenu::Help,
                    KeyCode::Char('c') => app.menu = SelectedMenu::Changes,
                    KeyCode::Char('d') => {
                        if let Some(hovered) = &app.hovered_dev {
                            let command = if crate::parts::is_partition(&hovered) {
                                Command::new(Change::DeletePartition { name: hovered.clone()})
                            } else {
                                Command::new(Change::DeletePartitionTable { name: hovered.clone()})
                            };

                            if app.commands.iter().find(|c| c.change == command.change).is_none() {
                                app.apply_change(command);
                            }
                        }
                    }
                    KeyCode::Char('f') => {
                        if let Some(hovered) = &app.hovered_dev {
                            let command = if crate::parts::is_partition(&hovered) {
                                Command::new(Change::ReformatPartition { partition: hovered.clone(), file_sys: FileSystem::Fat32 })
                            } else {
                                Command::new(Change::ReformatPartitionTable { name: hovered.clone(), scheme: PartitionScheme::Gpt })
                            };

                            if app.commands.iter().find(|c| c.change == command.change).is_none() {
                                app.apply_change(command);
                            }
                        }
                    }
                    KeyCode::Char('w') => {
                        for command in app.commands.iter_mut() {
                            command.perform();
                        }
                    }
                    // KeyCode::Char('c') => {
                    //     app.commands.clear();
                    //     app.new_disks = app.old_disks.clone();
                    // }
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

    let [body, footer] = Layout::vertical([
        Constraint::Min(0),
        Constraint::Length(1),
    ])
    .areas(area);

    let [left, right] = Layout::horizontal([
        Constraint::Min(20),
        Constraint::Min(0),
    ]).areas(body);

    let right_paragraph = match app.menu {
        SelectedMenu::Changes => {
            let mut changes = Vec::new();
            for command in app.commands.iter() {
                if command.error.is_none() {
                    changes.push(Line::raw(format!("• {}", command.change)));
                } else {
                    let red_style = Style::new().fg(ratatui::style::Color::Red);

                    changes.push(Line::raw(format!("• {}", command.change)));
                    changes.push(Line::raw(format!("  ╰─ {}", command.error.clone().unwrap())).style(red_style));
                }
            }

            Paragraph::new(changes)
        }
        SelectedMenu::Settings => {
            Paragraph::new(Line::raw("You're in settings"))
        }
        SelectedMenu::Help => {
            Paragraph::new(Line::raw("w -> Write"))
        }
    };

    let right_block = right_paragraph
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .title(Line::from(spans_from_hotkey_word("C", "hanges")).centered())
                .title(Line::from(spans_from_hotkey_word("S", "ettings")).centered())
                .title(Line::from(spans_from_hotkey_word("H", "elp")).centered())
                .title_bottom(Line::from("bdev v0.1.0").right_aligned())
        );

    frame.render_widget(right_block, right);


    // left side (and hotkey bar) below. TODO dynamic hotbar from current page maybe? idk kinda outdated now the menu is on the right
    let mut hotkey_bar_spans = Vec::new();

    hotkey_bar_spans.extend(spans_from_hotkey_word("Q", "uit"));
    hotkey_bar_spans.push(Span::raw(" | "));
    hotkey_bar_spans.extend(spans_from_hotkey_word("W", "rite"));
    hotkey_bar_spans.push(Span::raw(" | "));
    hotkey_bar_spans.extend(spans_from_hotkey_word("R", "efresh"));
    hotkey_bar_spans.push(Span::raw(" | "));
    hotkey_bar_spans.extend(spans_from_hotkey_word("F", "ormat"));
    hotkey_bar_spans.push(Span::raw(" | "));
    hotkey_bar_spans.extend(spans_from_hotkey_word("D", "elete"));
    // arrow keys to select up/down are ommitted here because I consider them obvious (again, opinionated)

    frame.render_widget(
        Line::from(hotkey_bar_spans).centered(),
        footer,
    );
    
    let mut rows: Vec<Line> = Vec::new();
    for disk in &app.new_disks {
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

            rows.push(Line::from(format!("  {branch} {} {part_size}", part.name)).style(part_style));
            // let modified = app.commands.iter().any(|c| c.change.relates_to(&part.name));
            
            // if !modified {
            // } else {
            //     for command in &app.commands {
            //         if command.change.relates_to(&part.name) {
            //             match command.change {
            //                 Change::DeletePartition { .. } => {
            //                     rows.push(Line::from(format!("  {branch} {} {part_size}", part.name)).style(old_style));
            //                 }
            //                 Change::ReformatPartition { ref partition, file_sys } => {
            //                     rows.push(Line::from(vec![
            //                         Span::raw(format!("  {branch} {} {part_size} ", part.name)).style(part_style),
            //                         Span::raw(format!("MBR")).style(old_style),
            //                         Span::raw(format!("{file_sys}")).style(new_style),
            //                     ]));
            //                 }
            //                 _ => {} // THIS IS WHY IT DISAPPEARED
            //             }
            //         }
            //     }
            // }

        }
    }

    let list = Paragraph::new(rows)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .title(Line::raw("Devices").centered())
        );

    frame.render_widget(list, left);
}