use ratatui::widgets::Paragraph;
use ansi_to_tui::IntoText as _;
use crate::styles;

const PINK: &str = "\x1b[38;5;205m\x1b[1m";
const RESET: &str = "\x1b[22m\x1b[39m";

pub fn explain_intro_line_1(width: u16, height: u16) -> Paragraph<'static> {
    Paragraph::new(
        styles::left_directions(width, height)
            .render("Cropping cuts a rectangle out of a picture and throws the rest away.")
            .as_bytes().into_text().unwrap()
    )
}

pub fn explain_intro_line_2(width: u16, height: u16) -> Paragraph<'static> {
    Paragraph::new(
        styles::left_directions(width, height)
            .render("You describe the rectangle by where its top-left corner sits and how big it is, all in pixels:")
            .as_bytes().into_text().unwrap()
    )
}

pub fn explain_outro_line(width: u16, height: u16) -> Paragraph<'static> {
    Paragraph::new(
        styles::left_directions(width, height)
            .render("Cropping never resamples, so what's left is pixel-for-pixel identical to the original.")
            .as_bytes().into_text().unwrap()
    )
}

pub fn explain_format_list(width: u16, height: u16) -> Paragraph<'static> {
    Paragraph::new(
        styles::left_directions_transparent(width, height)
            .render(
                format!(
                    "{PINK}x{RESET}       : pixels in from the LEFT edge to the rectangle's left side (blank = 0)\n\
                     {PINK}y{RESET}       : pixels down from the TOP edge to the rectangle's top side (blank = 0)\n\
                     {PINK}width{RESET}   : how wide the kept rectangle is\n\
                     {PINK}height{RESET}  : how tall the kept rectangle is\n\
                     \n\
                     Example: x=100 y=50 width=640 height=480 keeps a 640x480 window that starts\n\
                     100 pixels in and 50 pixels down. If the rectangle reaches past the picture's\n\
                     edge, ffmpeg refuses and the status line shows its complaint."
                ).as_str()
            )
            .as_bytes()
            .into_text()
            .unwrap()
    )
}
