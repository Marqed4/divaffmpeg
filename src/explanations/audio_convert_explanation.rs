use ratatui::widgets::Paragraph;
use ansi_to_tui::IntoText as _;
use crate::styles;

const PINK: &str = "\x1b[38;5;205m\x1b[1m";
const RESET: &str = "\x1b[22m\x1b[39m";

pub fn explain_intro_line_1(width: u16, height: u16) -> Paragraph<'static> {
    Paragraph::new(
        styles::left_directions(width, height)
            .render("Audio conversion re-encodes a sound file into a different container, at whatever quality you ask for.")
            .as_bytes().into_text().unwrap()
    )
}

pub fn explain_intro_line_2(width: u16, height: u16) -> Paragraph<'static> {
    Paragraph::new(
        styles::left_directions(width, height)
            .render("Diva FFMPEG offers these containers, listed roughly most to least commonly reached for:")
            .as_bytes().into_text().unwrap()
    )
}

pub fn explain_outro_line(width: u16, height: u16) -> Paragraph<'static> {
    Paragraph::new(
        styles::left_directions(width, height)
            .render("Picking the container picks the encoder too, so there's no separate codec field to get wrong here.")
            .as_bytes().into_text().unwrap()
    )
}

pub fn explain_format_list(width: u16, height: u16) -> Paragraph<'static> {
    Paragraph::new(
        styles::left_directions_transparent(width, height)
            .render(
                format!(
                    "{PINK}\".mp3\"{RESET}        : lossy, plays absolutely everywhere, the safe default\n\
                     {PINK}\".m4a\"{RESET}        : lossy AAC, better than mp3 at the same bitrate, Apple's house format\n\
                     \".opus\"       : lossy, the best of the bunch under 128k, newer so less universal\n\
                     \".ogg\"        : lossy Vorbis, open and widely supported, largely superseded by opus\n\
                     \".flac\"       : lossless and compressed, perfect quality at about half the size of wav\n\
                     \".wav\"        : lossless and uncompressed, enormous, what editors want to be handed"
                ).as_str()
            )
            .as_bytes()
            .into_text()
            .unwrap()
    )
}

pub fn explain_bitrate_list(width: u16, height: u16) -> Paragraph<'static> {
    Paragraph::new(
        styles::left_directions_transparent(width, height)
            .render(
                format!(
                    "{PINK}\"-b:a\"{RESET}        : bits per second of sound, higher is cleaner and bigger (320k is transparent for most ears, 128k is the old web default, 64k is speech territory)\n\
                     \"-ar\"         : sample rate, 44.1 kHz is CD, 48 kHz is video, dropping lower trims the top end off the sound\n\
                     \"-ac\"         : channel count, 2 keeps stereo, 1 folds it to mono and roughly halves the size"
                ).as_str()
            )
            .as_bytes()
            .into_text()
            .unwrap()
    )
}
