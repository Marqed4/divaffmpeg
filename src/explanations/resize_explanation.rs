use ratatui::widgets::Paragraph;
use ansi_to_tui::IntoText as _;
use crate::styles;

const PINK: &str = "\x1b[38;5;205m\x1b[1m";
const RESET: &str = "\x1b[22m\x1b[39m";

pub fn explain_intro_line_1(width: u16, height: u16) -> Paragraph<'static> {
    Paragraph::new(
        styles::left_directions(width, height)
            .render("Resize scales a video to a new width and height. Unlike trim, the clip is re-encoded, so it takes longer and default codecs are used.")
            .as_bytes().into_text().unwrap()
    )
}

pub fn explain_intro_line_2(width: u16, height: u16) -> Paragraph<'static> {
    Paragraph::new(
        styles::left_directions(width, height)
            .render("Type a width and a height in pixels (a trailing \"px\" is fine), or pick one of the common sizes below:")
            .as_bytes().into_text().unwrap()
    )
}

pub fn explain_outro_line(width: u16, height: u16) -> Paragraph<'static> {
    Paragraph::new(
        styles::left_directions(width, height)
            .render("Diva FFMPEG stretches to exactly what you type, so set one side to -2 and it will match the other side's aspect ratio for you.")
            .as_bytes().into_text().unwrap()
    )
}

pub fn explain_format_list(width: u16, height: u16) -> Paragraph<'static> {
    Paragraph::new(
        styles::left_directions_transparent(width, height)
            .render(
                format!(
                    "{PINK}3840 x 2160{RESET}   : 4K UHD, sharpest detail, biggest files\n\
                     {PINK}1920 x 1080{RESET}   : 1080p Full HD, the usual default for sharing\n\
                     {PINK}1280 x 720{RESET}    : 720p HD, a good balance of size and clarity\n\
                     854 x 480      : 480p, small files for quick previews\n\
                     640 x 360      : 360p, very small, visibly soft\n\
                     1280 x -2      : fixed width, height follows the aspect ratio\n\
                     -2 x 720       : fixed height, width follows the aspect ratio"
                ).as_str()
            )
            .as_bytes()
            .into_text()
            .unwrap() // Be careful when you change this.
    )
}