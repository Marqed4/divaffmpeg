use ratatui::widgets::Paragraph;
use ansi_to_tui::IntoText as _;
use crate::styles;

// https://ansi.gabebanks.net/ hot pink used to flag the formats you'll actually run into day to day.
const PINK: &str = "\x1b[38;5;205m\x1b[1m";
const GREY: &str = "\x1b[38;5;245m";
const WHITE: &str = "\x1b[38;5;255m\x1b[1m";
const RESET: &str = "\x1b[22m\x1b[39m";

pub fn explain_intro_line_1(width: u16, height: u16) -> Paragraph<'static> {
    Paragraph::new(
        styles::left_directions(width, height)
            .render("Converting to a new video format requires the provision of a source file and an output path.")
            .as_bytes().into_text().unwrap()
    )
}

pub fn explain_intro_line_2(width: u16, height: u16) -> Paragraph<'static> {
    Paragraph::new(
        styles::left_directions(width, height)
            .render("The available options include to and from any of these multimedia formats, listed roughly most to least common:")
            .as_bytes().into_text().unwrap()
    )
}

pub fn explain_outro_line(width: u16, height: u16) -> Paragraph<'static> {
    Paragraph::new(
        styles::left_directions(width, height)
            .render("Pick a video codec below \u{1F3AC} libx264 plays almost anywhere, libx265 shrinks the file at the cost of a slower encode.")
            .as_bytes().into_text().unwrap()
    )
}

pub fn explain_codec_list(width: u16, height: u16) -> Paragraph<'static> {
    Paragraph::new(
        styles::left_directions_transparent(width, height)
            .render(
                format!(
                    "{PINK}\u{1F3C3} H.264 / libx264{RESET} {WHITE}(fast){RESET} : {GREY}near-universal playback, larger files, quick encode.{RESET}\n\
                     {PINK}\u{1F40C} H.265 / libx265{RESET} {WHITE}(small){RESET}  : {GREY}~50% smaller at same quality, slower encode, less universal support.{RESET}"
                ).as_str()
            )
            .as_bytes()
            .into_text()
            .unwrap()
    )
}

pub fn explain_format_list(width: u16, height: u16) -> Paragraph<'static> {
    Paragraph::new(
        styles::left_directions_transparent(width, height)
            .render(
                format!(
                    "{PINK}\".mp4\"{RESET}       : MPEG-4 Part 14\n\
                     {PINK}\".mkv\"{RESET}       : Matroska Video\n\
                     {PINK}\".webm\"{RESET}      : WebM\n\
                     {PINK}\".mov\"{RESET}       : QuickTime Movie\n\
                     {PINK}\".avi\"{RESET}       : Audio Video Interleave\n\
                     \".ts/.m2ts\"  : MPEG/Blu-ray Transport Stream\n\
                     \".mpd\"       : Media Presentation Description (DASH)\n\
                     \".mxf\"       : Material Exchange Format\n\
                     \".flv\"       : Flash Video\n\
                     \".3gp/.3g2\"  : 3GPP Multimedia\n\
                     \".mpeg/.mpg\" : Moving Picture Experts Group\n\
                     \".wmv\"       : Windows Media Video\n\
                     {PINK}\".gif\"{RESET}       : Graphics Interchange Format"
                ).as_str()
            )
            .as_bytes()
            .into_text()
            .unwrap()
    )
}
