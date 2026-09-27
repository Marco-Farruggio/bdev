use std::collections::HashMap;
use std::fs;
use std::time::Duration;

use ratatui::{
    crossterm::event::{self, Event, KeyCode},
    layout::{Alignment, Constraint, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, BorderType, List, ListItem, ListState, Paragraph},
    DefaultTerminal, Frame,
};

mod partitions;
mod disks;
mod tui;
mod mbr;

use partitions::{PartitionScheme, read_partition_scheme};
use tui::spans_from_hotkey_word;

struct Disk {
    name: String,
    size: Option<u64>,
    partition_scheme: Result<Option<PartitionScheme>, std::io::Error>,
    partitions: Vec<Partition>,
}

struct Partition {
    name: String,
    // index: Result<u8, std::io::Error>,
    size: Option<u64>,
    // bootable: Result<bool, std::io::Error>,
    // part_type: crate::mbr::MbrPartitionType,
}

struct Row {
    name: String,
    is_partition: bool,
}

struct App {
    disks: Vec<Disk>,
    selected_name: Option<String>,
    highlighted_name: Option<String>,
    list_state: ListState,
}

impl App {
    fn new() -> Self {
        let disks = read_block_devices();
        let highlighted_name = flatten(&disks).into_iter().next().map(|r| r.name);
        let mut list_state = ListState::default();
        list_state.select(Some(0));
        Self {
            disks,
            selected_name: None,
            highlighted_name,
            list_state,
        }
    }

    fn refresh(&mut self) {
        self.disks = read_block_devices();
        self.sync_selection();
    }

    fn sync_selection(&mut self) {
        let rows = flatten(&self.disks);
        if rows.is_empty() {
            self.highlighted_name = None;
            self.list_state.select(None);
            return;
        }

        let found = self
            .highlighted_name
            .as_ref()
            .and_then(|name| rows.iter().position(|r| &r.name == name));

        let index = found.unwrap_or_else(|| {
            let prev = self.list_state.selected().unwrap_or(0);
            prev.min(rows.len() - 1)
        });

        self.highlighted_name = Some(rows[index].name.clone());
        self.list_state.select(Some(index));
    }

    fn select_next(&mut self) {
        let rows = flatten(&self.disks);
        if rows.is_empty() {
            return;
        }
        let i = match self.list_state.selected() {
            Some(i) if i + 1 < rows.len() => i + 1,
            Some(_) => 0, // wrap to top
            None => 0,
        };
        self.highlighted_name = Some(rows[i].name.clone());
        self.list_state.select(Some(i));
    }

    fn select_previous(&mut self) {
        let rows = flatten(&self.disks);
        if rows.is_empty() {
            return;
        }
        let i = match self.list_state.selected() {
            Some(0) => rows.len() - 1, // wrap to bottom
            Some(i) => i - 1,
            None => 0,
        };
        self.highlighted_name = Some(rows[i].name.clone());
        self.list_state.select(Some(i));
    }
}

fn main() -> std::io::Result<()> {
    let mut terminal = ratatui::init();
    let result = run(&mut terminal);
    ratatui::restore();
    result
}

fn run(terminal: &mut DefaultTerminal) -> std::io::Result<()> {
    let mut app = App::new();

    loop {
        terminal.draw(|frame| draw(frame, &mut app))?;

        if event::poll(Duration::from_millis(250))? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('q') | KeyCode::Esc => return Ok(()),
                    KeyCode::Char('r') => app.refresh(),
                    KeyCode::Down => app.select_next(),
                    KeyCode::Up => app.select_previous(),
                    _ => {}
                }
            }
        }
    }
}

fn is_partition(name: &str) -> bool {
    fs::metadata(format!("/sys/class/block/{name}/partition")).is_ok()
}

fn read_block_devices() -> Vec<Disk> {
    let mut names: Vec<String> = match fs::read_dir("/sys/class/block") {
        Ok(read_dir) => read_dir
            .filter_map(|entry| entry.ok())
            .map(|entry| entry.file_name().to_string_lossy().into_owned())
            .collect(),
        Err(_) => return Vec::new(),
    };
    names.sort();

    let mut disks: Vec<Disk> = Vec::new();
    let mut partitions_of: HashMap<String, Vec<Partition>> = HashMap::new();

    for name in &names {
        if is_partition(name) {
            let link_path = format!("/sys/class/block/{name}");
            if let Ok(real_path) = fs::canonicalize(&link_path) {
                if let Some(parent) = real_path
                    .parent()
                    .and_then(|p| p.file_name())
                    .map(|f| f.to_string_lossy().into_owned())
                {
                    partitions_of.entry(parent).or_default().push(
                        Partition {
                            name: name.clone(),
                            size: crate::disks::read_size_bytes(&name)
                        }
                    );
                }
            }
        } else {
            disks.push(Disk {
                name: name.clone(),
                size: crate::disks::read_size_bytes(name),
                partition_scheme: read_partition_scheme(name),
                partitions: Vec::new(),
            });
        }
    }

    for disk in &mut disks {
        if let Some(parts) = partitions_of.remove(&disk.name) {
            disk.partitions = parts;
        }
    }

    disks
}

/// flatts the disk/partition tree into the same order which `draw()`
/// would render, so that both the selection indici always lines up
/// with whats being disdplayed
fn flatten(disks: &[Disk]) -> Vec<Row> {
    let mut rows = Vec::new();
    for disk in disks {
        rows.push(Row {
            name: disk.name.clone(),
            is_partition: false,
        });
        for part in &disk.partitions {
            rows.push(Row {
                name: part.name.clone(),
                is_partition: true,
            });
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

    let mut items: Vec<ListItem> = Vec::new();
    for disk in &app.disks {
        let size_display = if let Some(size) = disk.size {
            crate::disks::format_nbytes(size)
        } else {
            "UNK".to_string()   
        };

        let drive_display_name = format!("{} {size_display} [{}]", disk.name.as_str(), crate::partitions::maybe_scheme_to_str(&disk.partition_scheme));
        items.push(ListItem::new(drive_display_name).style(Style::new().bold()));
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
            items.push(ListItem::new(format!("  {branch} {} {part_size}", part.name)));
        }
    }

    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .title(format!("Devices ({})", app.disks.len());)
                .title(Line::from("bdev v0.1.0").right_aligned()),
        )
        .highlight_style(Style::new().add_modifier(Modifier::REVERSED));

    frame.render_stateful_widget(list, body, &mut app.list_state);

    let mut hotkey_bar_spans = Vec::new();

    hotkey_bar_spans.extend(spans_from_hotkey_word("Q", "uit"));
    hotkey_bar_spans.push(Span::raw(" | "));
    hotkey_bar_spans.extend(spans_from_hotkey_word("W", "rite"));
    hotkey_bar_spans.push(Span::raw(" | "));
    hotkey_bar_spans.extend(spans_from_hotkey_word("R", "efresh"));
    hotkey_bar_spans.push(Span::raw(" | "));
    hotkey_bar_spans.extend(spans_from_hotkey_word("H", "elp"));
    hotkey_bar_spans.push(Span::raw(" | ↑/↓: Select | Enter: Confirm | Esc: Back"));

    frame.render_widget(
        Line::from(hotkey_bar_spans).centered(),
        footer,
    );
}