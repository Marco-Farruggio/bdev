mod partitions;
mod disks;
mod tui;
mod mbr;


fn main() -> std::io::Result<()> {
    let mut terminal = ratatui::init();
    let result = tui::run(&mut terminal);
    ratatui::restore();
    result
}