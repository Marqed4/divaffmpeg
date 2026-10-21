use ratatui::widgets::Paragraph;
use ansi_to_tui::IntoText as _;
use crate::styles;

const PINK: &str = "\x1b[38;5;205m\x1b[1m";
const RESET: &str = "\x1b[22m\x1b[39m";

pub fn explain_intro_line_1(width: u16, height: u16) -> Paragraph<'static> {
    Paragraph::new(
        styles::left_directions(width, height)
            .render("Resizing changes a picture's pixel dimensions, keeping its format and re-saving it at high quality.")
            .as_bytes().into_text().unwrap()
    )
}

pub fn explain_intro_line_2(width: u16, height: u16) -> Paragraph<'static> {
    Paragraph::new(
        styles::left_directions(width, height)
            .render("Fill in one side and leave the other blank to keep the original proportions:")
            .as_bytes().into_text().unwrap()
    )
}

pub fn explain_outro_line(width: u16, height: u16) -> Paragraph<'static> {
    Paragraph::new(
        styles::left_directions(width, height)
            .render("Shrinking looks clean, enlarging can't invent detail that was never there and will look soft.")
            .as_bytes().into_text().unwrap()
    )
}

pub fn explain_format_list(width: u16, height: u16) -> Paragraph<'static> {
    Paragraph::new(
        styles::left_directions_transparent(width, height)
            .render(
                format!(
                    "{PINK}width only{RESET}   : 1280 wide, height worked out from the original ratio\n\
                     {PINK}height only{RESET}  : 720 tall, width worked out from the original ratio\n\
                     {PINK}both{RESET}         : exactly that size, so a mismatched ratio will stretch the picture\n\
                     {PINK}neither{RESET}      : nothing to do, so the run button does nothing\n\
                     \n\
                     Values are whole pixels. Anything that isn't a positive whole number is refused\n\
                     rather than guessed at, so check the width and height boxes if nothing happens."
                ).as_str()
            )
            .as_bytes()
            .into_text()
            .unwrap()
    )
}
