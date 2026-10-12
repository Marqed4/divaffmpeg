use ratatui::widgets::Paragraph;
use ansi_to_tui::IntoText as _;
use crate::styles;

const PINK: &str = "\x1b[38;5;205m\x1b[1m";
const RESET: &str = "\x1b[22m\x1b[39m";

pub fn explain_intro_line_1(width: u16, height: u16) -> Paragraph<'static> {
    Paragraph::new(
        styles::left_directions(width, height)
            .render("Gain pushes the whole track up or down by a fixed number of decibels.")
            .as_bytes().into_text().unwrap()
    )
}

pub fn explain_intro_line_2(width: u16, height: u16) -> Paragraph<'static> {
    Paragraph::new(
        styles::left_directions(width, height)
            .render("Decibels are a ratio, not a percentage, so the numbers move faster than they look like they should:")
            .as_bytes().into_text().unwrap()
    )
}

pub fn explain_outro_line(width: u16, height: u16) -> Paragraph<'static> {
    Paragraph::new(
        styles::left_directions(width, height)
            .render("This is a flat offset, not loudness matching: a track that swings between whispers and shouting still does.")
            .as_bytes().into_text().unwrap()
    )
}

pub fn explain_format_list(width: u16, height: u16) -> Paragraph<'static> {
    Paragraph::new(
        styles::left_directions_transparent(width, height)
            .render(
                format!(
                    "{PINK}\"volume=XdB\"{RESET}  : scales every sample by a fixed amount, the filter this screen drives\n\
                     +6 dB         : roughly twice as loud\n\
                     0 dB          : untouched\n\
                     -6 dB         : roughly half as loud\n\
                     \n\
                     Most music you'll feed this is already mastered close to the ceiling, so pushing much past\n\
                     +12 dB tends to clip: the peaks run out of headroom and flatten into distortion. Going down\n\
                     never clips, which makes negative gain the safer direction when you're unsure."
                ).as_str()
            )
            .as_bytes()
            .into_text()
            .unwrap()
    )
}
