use ratatui::widgets::Paragraph;
use ansi_to_tui::IntoText as _;
use crate::styles;

const PINK: &str = "\x1b[38;5;205m\x1b[1m";
const RESET: &str = "\x1b[22m\x1b[39m";

pub fn explain_intro_line_1(width: u16, height: u16) -> Paragraph<'static> {
    Paragraph::new(
        styles::left_directions(width, height)
            .render("Respeeding stretches or squashes the timeline, making a track run longer or shorter than it did.")
            .as_bytes().into_text().unwrap()
    )
}

pub fn explain_intro_line_2(width: u16, height: u16) -> Paragraph<'static> {
    Paragraph::new(
        styles::left_directions(width, height)
            .render("The pitch stays put while the length moves, so slowed audio sounds slower instead of deeper:")
            .as_bytes().into_text().unwrap()
    )
}

pub fn explain_outro_line(width: u16, height: u16) -> Paragraph<'static> {
    Paragraph::new(
        styles::left_directions(width, height)
            .render("Past about 2x in either direction the artefacts start showing, so treat the far ends of the slider as a stunt.")
            .as_bytes().into_text().unwrap()
    )
}

pub fn explain_format_list(width: u16, height: u16) -> Paragraph<'static> {
    Paragraph::new(
        styles::left_directions_transparent(width, height)
            .render(
                format!(
                    "{PINK}\"atempo\"{RESET}      : changes tempo while holding pitch, the filter this whole screen drives\n\
                     0.50x         : twice as long, the slowest a single atempo can go\n\
                     1.00x         : untouched\n\
                     2.00x         : half as long, the fastest a single atempo can go\n\
                     \n\
                     One atempo only accepts 0.5 through 2.0, so anything beyond gets chained: 4.00x becomes\n\
                     \"atempo=2.0,atempo=2.0\" and 0.25x becomes \"atempo=0.5,atempo=0.5\". Diva FFMPEG builds that\n\
                     chain for you, which is why the slider happily runs out to 0.25x and 4.00x.\n\
                     \n\
                     Changing tempo touches every sample, so this always re-encodes. There's no copy shortcut."
                ).as_str()
            )
            .as_bytes()
            .into_text()
            .unwrap()
    )
}
