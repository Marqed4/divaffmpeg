use ratatui::widgets::Paragraph;
use ansi_to_tui::IntoText as _;
use crate::styles;

const PINK: &str = "\x1b[38;5;205m\x1b[1m";
const RESET: &str = "\x1b[22m\x1b[39m";

pub fn explain_intro_line_1(width: u16, height: u16) -> Paragraph<'static> {
    Paragraph::new(
        styles::left_directions(width, height)
            .render("Trimming cuts a track down to a start and end timestamp, without re-encoding a thing.")
            .as_bytes().into_text().unwrap()
    )
}

pub fn explain_intro_line_2(width: u16, height: u16) -> Paragraph<'static> {
    Paragraph::new(
        styles::left_directions(width, height)
            .render("The end field fills itself in with the track's real length as soon as you confirm the input path:")
            .as_bytes().into_text().unwrap()
    )
}

pub fn explain_outro_line(width: u16, height: u16) -> Paragraph<'static> {
    Paragraph::new(
        styles::left_directions(width, height)
            .render("Audio keyframes sit close together, so a copied cut lands near enough to the timestamp that you won't hear the difference.")
            .as_bytes().into_text().unwrap()
    )
}

pub fn explain_format_list(width: u16, height: u16) -> Paragraph<'static> {
    Paragraph::new(
        styles::left_directions_transparent(width, height)
            .render(
                format!(
                    "{PINK}\"-c:a copy\"{RESET}   : stream copy, lossless and near-instant, no quality lost to a second encode\n\
                     \"-ss\"/\"-to\"   : start and end timestamps in HH:MM:SS, placed before -i so ffmpeg seeks fast\n\
                     {PINK}\"-vn\"{RESET}         : drops embedded cover art, which ffmpeg otherwise treats as a video stream\n\
                     \n\
                     Video trims land only on keyframes, which can sit seconds apart. Audio keyframes are packed far\n\
                     tighter, which is why this screen doesn't bother offering a re-encode mode the way video Trim does."
                ).as_str()
            )
            .as_bytes()
            .into_text()
            .unwrap()
    )
}
