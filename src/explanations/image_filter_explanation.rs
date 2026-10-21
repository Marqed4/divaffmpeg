use ratatui::widgets::Paragraph;
use ansi_to_tui::IntoText as _;
use crate::styles;

const PINK: &str = "\x1b[38;5;205m\x1b[1m";
const RESET: &str = "\x1b[22m\x1b[39m";

pub fn explain_intro_line_1(width: u16, height: u16) -> Paragraph<'static> {
    Paragraph::new(
        styles::left_directions(width, height)
            .render("Filters give a picture a one-pick look: drain its color, soften it, sharpen it, and more.")
            .as_bytes().into_text().unwrap()
    )
}

pub fn explain_intro_line_2(width: u16, height: u16) -> Paragraph<'static> {
    Paragraph::new(
        styles::left_directions(width, height)
            .render("Each effect is a single ffmpeg filter, and the strength slider scales how hard it hits:")
            .as_bytes().into_text().unwrap()
    )
}

pub fn explain_outro_line(width: u16, height: u16) -> Paragraph<'static> {
    Paragraph::new(
        styles::left_directions(width, height)
            .render("The result is a new file, so you can apply an effect, look at it, and run another on top.")
            .as_bytes().into_text().unwrap()
    )
}

pub fn explain_format_list(width: u16, height: u16) -> Paragraph<'static> {
    Paragraph::new(
        styles::left_directions_transparent(width, height)
            .render(
                format!(
                    "{PINK}grayscale{RESET} : drains color, 100% is fully gray, 0% is untouched\n\
                     {PINK}sepia{RESET}     : blends toward a warm brown tint, 100% is the full old-photo look\n\
                     {PINK}blur{RESET}      : gaussian softening, higher spreads further\n\
                     {PINK}sharpen{RESET}   : boosts edge contrast, too much amplifies noise\n\
                     {PINK}vignette{RESET}  : darkens the corners, higher reaches further toward the middle\n\
                     {PINK}invert{RESET}    : swaps every color for its opposite, no strength (shows n/a)\n\
                     \n\
                     The picture keeps its original format, with JPG and WebP re-saved at high quality."
                ).as_str()
            )
            .as_bytes()
            .into_text()
            .unwrap()
    )
}
