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
- removed number of devices indicator
- added a top toolbar to show which menu, devices, settings, or help
- removed clap dependency, i dont plan to have any cli for a while
- handle selection myself instead of letting ratatui lists do it
- moved all tui into tui.rs
- detect partition index from /sys/class/block, rather than parsing each name

# To-Do:

- show which menu is currently selected clearly
- rename some reads to detect
- heirarchy for NVMEs by namespace also
- human readable mode
- formatting
- partitioning
- drop mouse events
- highlight whole device, then whole namespace, then partition
- move device logic into a specific file
- make read devices return an option/error, shouldnt ever, but we'll see
- disk/partition enum with an impl enum pub fn name()
- trait for partition schemes
- make a nicer hotkey/menu generator func/macro
- check is partition and partition index in one call