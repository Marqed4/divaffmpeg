use ratatui::widgets::Paragraph;
use ansi_to_tui::IntoText as _;
use crate::styles;

const PINK: &str = "\x1b[38;5;205m\x1b[1m";
const RESET: &str = "\x1b[22m\x1b[39m";

pub fn explain_intro_line_1(width: u16, height: u16) -> Paragraph<'static> {
    Paragraph::new(
        styles::left_directions(width, height)
            .render("Converting re-saves a picture in a different file format, optionally trading detail for a smaller file.")
            .as_bytes().into_text().unwrap()
    )
}

pub fn explain_intro_line_2(width: u16, height: u16) -> Paragraph<'static> {
    Paragraph::new(
        styles::left_directions(width, height)
            .render("The format decides the extension and the encoder, so the output name is always corrected to match:")
            .as_bytes().into_text().unwrap()
    )
}

pub fn explain_outro_line(width: u16, height: u16) -> Paragraph<'static> {
    Paragraph::new(
        styles::left_directions(width, height)
            .render("Going from a lossy format (JPG) to a lossless one (PNG) won't bring detail back, it only stops further loss.")
            .as_bytes().into_text().unwrap()
    )
}

pub fn explain_format_list(width: u16, height: u16) -> Paragraph<'static> {
    Paragraph::new(
        styles::left_directions_transparent(width, height)
            .render(
                format!(
                    "{PINK}.png{RESET}   : lossless, supports transparency, best for screenshots and graphics\n\
                     {PINK}.jpg{RESET}   : lossy, no transparency, best for photos and the smallest everyday files\n\
                     {PINK}.webp{RESET}  : lossy, modern and smaller than JPG at the same look, not opened by older apps\n\
                     {PINK}.bmp{RESET}   : uncompressed, huge, only for tools that demand it\n\
                     {PINK}.tiff{RESET}  : lossless, common in print and scanning workflows\n\
                     {PINK}.gif{RESET}   : 256 colors only, fine for flat art, poor for photos\n\
                     \n\
                     {PINK}quality{RESET} : 1-100, only applies to JPG and WebP. Higher is cleaner and bigger.\n\
                     The other formats never lose detail to a quality dial, so it shows n/a for them."
                ).as_str()
            )
            .as_bytes()
            .into_text()
            .unwrap()
    )
}
