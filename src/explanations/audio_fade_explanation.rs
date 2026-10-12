use ratatui::widgets::Paragraph;
use ansi_to_tui::IntoText as _;
use crate::styles;

const PINK: &str = "\x1b[38;5;205m\x1b[1m";
const RESET: &str = "\x1b[22m\x1b[39m";

pub fn explain_intro_line_1(width: u16, height: u16) -> Paragraph<'static> {
    Paragraph::new(
        styles::left_directions(width, height)
            .render("Fading eases a track up from silence at the start and back down to it at the end.")
            .as_bytes().into_text().unwrap()
    )
}

pub fn explain_intro_line_2(width: u16, height: u16) -> Paragraph<'static> {
    Paragraph::new(
        styles::left_directions(width, height)
            .render("Both fields are plain seconds, so \"3\" means a three second fade. Leave one at 0 to skip that end:")
            .as_bytes().into_text().unwrap()
    )
}

pub fn explain_outro_line(width: u16, height: u16) -> Paragraph<'static> {
    Paragraph::new(
        styles::left_directions(width, height)
            .render("Confirm the input path first: the fade-out is measured back from the end, so it needs the length to exist yet.")
            .as_bytes().into_text().unwrap()
    )
}

pub fn explain_format_list(width: u16, height: u16) -> Paragraph<'static> {
    Paragraph::new(
        styles::left_directions_transparent(width, height)
            .render(
                format!(
                    "{PINK}\"afade=t=in\"{RESET}  : climbs from silence to full volume, starting at 0 seconds\n\
                     {PINK}\"afade=t=out\"{RESET} : falls from full volume back to silence, ending with the track\n\
                     \"st=\"         : the second the fade starts at\n\
                     \"d=\"          : how many seconds the fade takes\n\
                     \n\
                     A fade-out has to be told where to start, not how long is left, so the track's length gets\n\
                     probed first and the start worked back from it. Diva FFMPEG also clamps the two fades so they\n\
                     can't overlap or overrun the track, which would hand ffmpeg a negative start time."
                ).as_str()
            )
            .as_bytes()
            .into_text()
            .unwrap()
    )
}
