//! # bdev
//! 
//! bdev is a block device manipulation CLI TUI written in Rust for Linux

mod commands;
mod filesys;
mod parts;
mod disks;
mod mount;
mod tui;
mod mbr;

fn main() -> std::io::Result<()> {
    let mut terminal = ratatui::init();
    let result = tui::run(&mut terminal);
    ratatui::restore();
    result
}