use ratatui::widgets::Paragraph;
use ansi_to_tui::IntoText as _;
use crate::styles;

const PINK: &str = "\x1b[38;5;205m\x1b[1m";
const RESET: &str = "\x1b[22m\x1b[39m";

pub fn explain_intro_line_1(width: u16, height: u16) -> Paragraph<'static> {
    Paragraph::new(
        styles::left_directions(width, height)
            .render("Compression re-encodes your video into a smaller file, either by lowering quality, lowering bitrate, or both.")
            .as_bytes().into_text().unwrap()
    )
}

pub fn explain_intro_line_2(width: u16, height: u16) -> Paragraph<'static> {
    Paragraph::new(
        styles::left_directions(width, height)
            .render("Diva FFMPEG gives you a few flags to control that trade-off, listed roughly most to least commonly reached for:")
            .as_bytes().into_text().unwrap()
    )
}

pub fn explain_outro_line(width: u16, height: u16) -> Paragraph<'static> {
    Paragraph::new(
        styles::left_directions(width, height)
            .render("Stick with a CRF-based encode unless you have a hard file size limit to hit, that's what Diva FFMPEG defaults to.")
            .as_bytes().into_text().unwrap()
    )
}

pub fn explain_format_list(width: u16, height: u16) -> Paragraph<'static> {
    Paragraph::new(
        styles::left_directions_transparent(width, height)
            .render(
                format!(
                    "{PINK}\"-crf\"{RESET}     : Constant Rate Factor, quality-based, lower is higher quality (18 to 28 is sane)\n\
                     \"-b:v\"     : target bitrate, size-based, hits a strict file size cap\n\
                     \"-preset\"  : encoding speed, \"ultrafast\" through \"veryslow\"\n\
                     \"-c:v\"     : codec choice, libx264 for compatibility, libx265/libaom-av1 for smaller files"
                ).as_str()
            )
            .as_bytes()
            .into_text()
            .unwrap()
    )
}
