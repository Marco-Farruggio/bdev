//! # bdev
//! 
//! bdev is a block device manipulation CLI TUI written in Rust for Linux

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