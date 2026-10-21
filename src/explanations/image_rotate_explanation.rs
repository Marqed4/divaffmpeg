use ratatui::widgets::Paragraph;
use ansi_to_tui::IntoText as _;
use crate::styles;

const PINK: &str = "\x1b[38;5;205m\x1b[1m";
const RESET: &str = "\x1b[22m\x1b[39m";

pub fn explain_intro_line_1(width: u16, height: u16) -> Paragraph<'static> {
    Paragraph::new(
        styles::left_directions(width, height)
            .render("Rotating turns a picture on its side or upside down, and flipping mirrors it.")
            .as_bytes().into_text().unwrap()
    )
}

pub fn explain_intro_line_2(width: u16, height: u16) -> Paragraph<'static> {
    Paragraph::new(
        styles::left_directions(width, height)
            .render("Pick one transform with W/S while the turn field is focused:")
            .as_bytes().into_text().unwrap()
    )
}

pub fn explain_outro_line(width: u16, height: u16) -> Paragraph<'static> {
    Paragraph::new(
        styles::left_directions(width, height)
            .render("Every option just reshuffles pixels, so nothing gets blurry the way a free-angle rotation would.")
            .as_bytes().into_text().unwrap()
    )
}

pub fn explain_format_list(width: u16, height: u16) -> Paragraph<'static> {
    Paragraph::new(
        styles::left_directions_transparent(width, height)
            .render(
                format!(
                    "{PINK}90 right{RESET}  : quarter turn clockwise, a landscape picture becomes portrait\n\
                     {PINK}90 left{RESET}   : quarter turn counter-clockwise\n\
                     {PINK}180{RESET}       : upside down, same shape as before\n\
                     {PINK}mirror{RESET}    : left and right swapped, like looking in a mirror\n\
                     {PINK}flip{RESET}      : top and bottom swapped\n\
                     \n\
                     The picture keeps its original format. JPG and WebP are re-saved at high quality\n\
                     so the extra pass costs very little."
                ).as_str()
            )
            .as_bytes()
            .into_text()
            .unwrap()
    )
}
