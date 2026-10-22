use ratatui::widgets::Paragraph;
use ansi_to_tui::IntoText as _;
use crate::styles;

const PINK: &str = "\x1b[38;5;205m\x1b[1m";
const RESET: &str = "\x1b[22m\x1b[39m";

pub fn explain_intro_line_1(width: u16, height: u16) -> Paragraph<'static> {
    Paragraph::new(
        styles::left_directions(width, height)
            .render("Trimming cuts your video down to a start and end timestamp, no format conversion required.")
            .as_bytes().into_text().unwrap()
    )
}

pub fn explain_intro_line_2(width: u16, height: u16) -> Paragraph<'static> {
    Paragraph::new(
        styles::left_directions(width, height)
            .render("Diva FFMPEG can do this two ways, listed roughly most to least commonly reached for:")
            .as_bytes().into_text().unwrap()
    )
}

pub fn explain_outro_line(width: u16, height: u16) -> Paragraph<'static> {
    Paragraph::new(
        styles::left_directions(width, height)
            .render("Diva FFMPEG defaults to a stream copy for speed, flip to re-encode mode if you need a frame-perfect cut instead.")
            .as_bytes().into_text().unwrap()
    )
}

pub fn explain_format_list(width: u16, height: u16) -> Paragraph<'static> {
    Paragraph::new(
        styles::left_directions_transparent(width, height)
            .render(
                format!(
                    "{PINK}\"-c copy\"{RESET}      : stream copy, lossless and near-instant, but only cuts on keyframes\n\
                     \"-ss\"/\"-to\"    : start and end timestamps, before -i for fast seek, after -i for frame-accurate\n\
                     re-encode      : drop \"-c copy\" for an exact, frame-accurate cut at any point\n\
                     \"-t\"           : clip duration, an alternative to an end timestamp"
                ).as_str()
            )
            .as_bytes()
            .into_text()
            .unwrap()
    )
}
