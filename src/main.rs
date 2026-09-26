use std::fs;
use std::time::Duration;

use ratatui::{
    crossterm::event::{self, Event, KeyCode},
    layout::{Alignment, Constraint, Layout},
    style::{Color, Style, Stylize},
    text::Line,
    widgets::{Block, Borders, List, ListItem, Paragraph},
    DefaultTerminal, Frame,
};

struct App {
    devices: Vec<String>,
}

impl App {
    fn new() -> Self {
        Self {
            devices: read_block_devices(),
        }
    }

    fn refresh(&mut self) {
        self.devices = read_block_devices();
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
        terminal.draw(|frame| draw(frame, &app))?;

        if event::poll(Duration::from_millis(250))? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('q') | KeyCode::Esc => return Ok(()),
                    KeyCode::Char('r') => app.refresh(),
                    _ => {}
                }
            }
        }
    }
}

fn read_block_devices() -> Vec<String> {
    let mut entries: Vec<String> = match fs::read_dir("/sys/class/block") {
        Ok(read_dir) => read_dir
            .filter_map(|entry| entry.ok())
            .map(|entry| entry.file_name().to_string_lossy().into_owned())
            .collect(),
        Err(_) => Vec::new(),
    };
    entries.sort();
    entries
}

fn draw(frame: &mut Frame, app: &App) {
    let area = frame.area();
    //
    // if area.width < 40 || area.height < 10 {
    //     frame.render_widget(
    //         Paragraph::new("terminal too small").alignment(Alignment::Center),
    //         area,
    //     );
    //     return;
    // }

    let [header, body, footer] = Layout::vertical([
        Constraint::Length(3),
        Constraint::Min(0),
        Constraint::Length(1),
    ])
    .areas(area);

    frame.render_widget(
        Paragraph::new("bdev")
            .style(Style::new().fg(Color::Green).bold())
            .alignment(Alignment::Center)
            .block(Block::default().borders(Borders::ALL)),
        header,
    );

    let title = format!(" Devices ({}) ", app.devices.len());
    let items: Vec<ListItem> = app
        .devices
        .iter()
        .map(|name| ListItem::new(name.as_str()))
        .collect();

    frame.render_widget(
        List::new(items).block(Block::default().borders(Borders::ALL).title(title)),
        body,
    );

    frame.render_widget(
        Line::from("Q/Esc: Quit | W/F: Write | R: Refresh | H: Help").centered(),
        footer,
    );
}