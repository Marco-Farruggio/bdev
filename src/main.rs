//! # bdev
//! 
//! bdev is a block device manipulation CLI TUI written in Rust for Linux

mod parts;
mod commands;
mod disks;
mod filesys;
mod tui;
mod mbr;

fn main() -> std::io::Result<()> {
    let mut terminal = ratatui::init();
    let result = tui::run(&mut terminal);
    ratatui::restore();
    result
}