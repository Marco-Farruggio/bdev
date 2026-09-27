# Version 0.1.0:

- added deps: clap, ratatui
- wrote ratatui gui loop
- read from /dev
- added r for refresh
- fix: read only block devices from /sys/class/block
- added partition scheme detection (requires sudo/disk group)
- added size detection for disks
- added size detection for partitions
- made a specific partition type
- added curves to the bottom of each branched list of partitions
- the whole block is now rounded corners too
- removed the top bdev bar
- added bdev version to the gui
- made hotkeys much more compact
- made a tui.rs file to handle that
- added an enum for the MBR partition type byte
- removed protective gpt in favour of just gpt as its part of the spec

# To-Do:

- rename some reads to detect
- extract selection state from ratatui and handle myself
- heirarchy for NVMEs by namespace also
- human readable mode
- help menu
- formatting
- purpose settings
- partitioning
- drop mouse events
- highlight whole device, then whole namespace, then partition
- move device logic into a specific file
- move all tui into tui.rs
- make read devices return an option/error, shouldnt ever, but we'll see
- inverse for highlighting
- disk/partition enum with an impl enum pub fn name()
- trait for partition schemes