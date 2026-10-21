use ratatui::widgets::Paragraph;
use ansi_to_tui::IntoText as _;
use crate::styles;

const PINK: &str = "\x1b[38;5;205m\x1b[1m";
const RESET: &str = "\x1b[22m\x1b[39m";

pub fn explain_intro_line_1(width: u16, height: u16) -> Paragraph<'static> {
    Paragraph::new(
        styles::left_directions(width, height)
            .render("Compressing shrinks a picture's file size by re-saving it as a lossy JPG or WebP.")
            .as_bytes().into_text().unwrap()
    )
}

pub fn explain_intro_line_2(width: u16, height: u16) -> Paragraph<'static> {
    Paragraph::new(
        styles::left_directions(width, height)
            .render("The quality slider decides how much fine detail gets thrown away to save space:")
            .as_bytes().into_text().unwrap()
    )
}

pub fn explain_outro_line(width: u16, height: u16) -> Paragraph<'static> {
    Paragraph::new(
        styles::left_directions(width, height)
            .render("Compressing an already-compressed JPG again stacks the loss, so keep the original around.")
            .as_bytes().into_text().unwrap()
    )
}

pub fn explain_format_list(width: u16, height: u16) -> Paragraph<'static> {
    Paragraph::new(
        styles::left_directions_transparent(width, height)
            .render(
                format!(
                    "{PINK}90-100%{RESET}  : near-lossless, files stay big\n\
                     {PINK}60-85%{RESET}   : the sweet spot, hard to tell apart from the original at normal size\n\
                     {PINK}30-55%{RESET}   : clearly smaller, softness and blocky edges start to show\n\
                     {PINK}1-25%{RESET}    : tiny, visibly damaged\n\
                     \n\
                     {PINK}.jpg{RESET}     : works everywhere, no transparency\n\
                     {PINK}.webp{RESET}    : smaller than JPG at the same look, keeps transparency, newer apps only\n\
                     \n\
                     Transparent areas become black when saving as JPG."
                ).as_str()
            )
            .as_bytes()
            .into_text()
            .unwrap()
    )
}
