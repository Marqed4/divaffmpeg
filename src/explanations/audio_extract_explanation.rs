use ratatui::widgets::Paragraph;
use ansi_to_tui::IntoText as _;
use crate::styles;

const PINK: &str = "\x1b[38;5;205m\x1b[1m";
const RESET: &str = "\x1b[22m\x1b[39m";

pub fn explain_intro_line_1(width: u16, height: u16) -> Paragraph<'static> {
    Paragraph::new(
        styles::left_directions(width, height)
            .render("Extraction lifts the audio track out of a video and leaves the picture behind.")
            .as_bytes().into_text().unwrap()
    )
}

pub fn explain_intro_line_2(width: u16, height: u16) -> Paragraph<'static> {
    Paragraph::new(
        styles::left_directions(width, height)
            .render("There are two ways to do it, and the first one is almost always the one you want:")
            .as_bytes().into_text().unwrap()
    )
}

pub fn explain_outro_line(width: u16, height: u16) -> Paragraph<'static> {
    Paragraph::new(
        styles::left_directions(width, height)
            .render("Copy is instant and loses nothing, so only reach for re-encode when you need a container copy can't fill.")
            .as_bytes().into_text().unwrap()
    )
}

pub fn explain_format_list(width: u16, height: u16) -> Paragraph<'static> {
    Paragraph::new(
        styles::left_directions_transparent(width, height)
            .render(
                format!(
                    "{PINK}\"-vn\"{RESET}         : no video, the flag that makes this an extraction at all\n\
                     {PINK}\"-c:a copy\"{RESET}   : lift the existing track out byte for byte, instant and lossless\n\
                     re-encode     : run it through a new encoder, slower and lossy, but lands in any container\n\
                     \n\
                     Most .mp4 files carry AAC audio, which drops straight into .m4a with a copy. A .webm carries\n\
                     Opus or Vorbis, which copies into .opus or .ogg. Copying into a container that can't hold the\n\
                     track's codec fails outright, and the log says so, so flip to re-encode when that happens."
                ).as_str()
            )
            .as_bytes()
            .into_text()
            .unwrap()
    )
}
