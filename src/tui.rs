use ratatui::{
    style::{Style, Modifier},
    text::{Line, Span},
};

pub fn spans_from_hotkey_word<'a>(letter: &'a str, rest: &'a str) -> Vec<Span<'a>> {
    vec![
        Span::styled(letter, Style::default().add_modifier(Modifier::UNDERLINED | Modifier::BOLD)),
        Span::raw(rest)
    ]
}