use std::{rc::Rc};

use ansi_to_tui::IntoText as _;

use ratatui::prelude::*;
use ratatui::widgets::*;

use crate::explanations;
use crate::background;
use crate::component;
use crate::styles;
// Audio draws the same furniture Video does, so the helpers live over there and
// get borrowed here rather than being copied into a second, drifting set.
use crate::video::{art_area, render_intro_and_status, render_path_field,
    render_progress_gauge, render_slider_field, render_value_field};
use crate::implementations::{
    FieldSet,
    AudioConvertField, AudioConvertState,
    AudioExtractField, AudioExtractState,
    AudioTrimField, AudioTrimState,
    AudioSpeedField, AudioSpeedState, speed_label, speed_ratio,
    AudioFadeField, AudioFadeState,
    AudioVolumeField, AudioVolumeState, gain_label, gain_ratio,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AudioProcessingState {
    Default,
    Convert,
    ExplainConvert,
    Extract,
    ExplainExtract,
    Trim,
    ExplainTrim,
    Speed,
    ExplainSpeed,
    Fade,
    ExplainFade,
    Volume,
    ExplainVolume,
}

//                      <-- SELECTABLES -->
impl AudioProcessingState {
    const SELECTABLES: [AudioProcessingState; 7] = [
        AudioProcessingState::Default,
        AudioProcessingState::Convert,
        AudioProcessingState::Extract,
        AudioProcessingState::Trim,
        AudioProcessingState::Speed,
        AudioProcessingState::Fade,
        AudioProcessingState::Volume,
    ];

    const EXPLANATIONS: [AudioProcessingState; 6] = [
        AudioProcessingState::ExplainConvert,
        AudioProcessingState::ExplainExtract,
        AudioProcessingState::ExplainTrim,
        AudioProcessingState::ExplainSpeed,
        AudioProcessingState::ExplainFade,
        AudioProcessingState::ExplainVolume,
    ];

    fn label(&self) -> &'static str {
        match self {
            Self::Default => "default",
            Self::Convert => "   Convert   ",
            Self::ExplainConvert => "Convert Guide",
            Self::Extract => "   Extract   ",
            Self::ExplainExtract => "Extract Guide",
            Self::Trim => "   Trim   ",
            Self::ExplainTrim => "Trim Guide",
            Self::Speed => "   Speed   ",
            Self::ExplainSpeed => "Speed Guide",
            Self::Fade => "   Fade   ",
            Self::ExplainFade => "Fade Guide",
            Self::Volume => "   Volume   ",
            Self::ExplainVolume => "Volume Guide",
        }
    }
}

// Deliberately a twin of video.rs's `DirectionsMenuState` rather than a shared
// one: that struct returns a `VideoProcessingState`, and making it generic over
// the scene's enum would cost more machinery than the twenty lines it saves.
#[derive(Debug)]
pub struct AudioDirectionsMenuState {
    selected_row: usize,
    selected_column: usize,
}

impl AudioDirectionsMenuState {
    pub fn new() -> Self {
        Self { selected_row: 0, selected_column: 0 }
    }

    pub fn next(&mut self) {
        self.selected_column = (self.selected_column + 1) % 6;
    }

    pub fn previous(&mut self) {
        self.selected_column = if self.selected_column == 0 { 5 } else { self.selected_column - 1 };
    }

    pub fn above(&mut self) {
        self.selected_row = if self.selected_row == 0 { 1 } else { self.selected_row - 1 };
    }

    pub fn below(&mut self) {
        self.selected_row = (self.selected_row + 1) % 2;
    }

    pub fn selected_state(&self) -> AudioProcessingState {
        if self.selected_row == 0 {
            AudioProcessingState::SELECTABLES[self.selected_column + 1]
        } else {
            AudioProcessingState::EXPLANATIONS[self.selected_column]
        }
    }
}

/// What a value field should show when the current container makes it moot, e.g.
/// a bitrate picker pointed at FLAC. Showing the stale number instead would imply
/// it still reaches ffmpeg, which it doesn't.
const IGNORED_VALUE: &str = "n/a";

pub fn render(
    frame: &mut Frame<'_>,
    state: AudioProcessingState,
    menu: &AudioDirectionsMenuState,
    convert_state: &mut AudioConvertState,
    extract_state: &mut AudioExtractState,
    trim_state: &mut AudioTrimState,
    speed_state: &mut AudioSpeedState,
    fade_state: &mut AudioFadeState,
    volume_state: &mut AudioVolumeState,
) {
    let size: Rect = frame.area();

    let outer: Rc<[Rect]> = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Fill(1),
        ])
        .split(size);

    let audio_block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .title("✦  Diva FFMPEG: Audio Processing  ✦")
        .title_alignment(Alignment::Center)
        .bold();

    let audio_inner: Rect = audio_block.inner(outer[0]);
    frame.render_widget(audio_block, outer[0]);

    let inner: Rc<[Rect]> = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),  // inner[0]: empty/unused space
            Constraint::Length(3),  // inner[1]: selection guide
            Constraint::Length(1),  // inner[2]: new line gap
            Constraint::Length(1),  // inner[3]: SELECTABLES tabs
            Constraint::Length(1),  // inner[4]: EXPLANATIONS tabs
            Constraint::Length(3),  // inner[5]: EXPLANATIONS/USER INTERFACE
            Constraint::Fill(1),    // inner[6]: art & UI
            Constraint::Length(1),  // inner[7]: footer links
        ])
        .split(audio_inner);

    let selection_guide: Paragraph<'_> = Paragraph::new(
        styles::center_directions(inner[1].width, inner[1].height)
            .render(
                "Press '\x1b[38;5;218m\x1b[1mA\x1b[22m\x1b[39m' or '\x1b[38;5;218m\x1b[1mleft arrow-key\x1b[22m\x1b[39m' OR '\x1b[38;5;218m\x1b[1mD\x1b[22m\x1b[39m' or '\x1b[38;5;218m\x1b[1mright arrow-key\x1b[22m\x1b[39m' \n \
                Press '\x1b[38;5;218m\x1b[1mW\x1b[22m\x1b[39m' or '\x1b[38;5;218m\x1b[1mup arrow-key\x1b[22m\x1b[39m' OR '\x1b[38;5;218m\x1b[1mS\x1b[22m\x1b[39m' or '\x1b[38;5;218m\x1b[1mdown arrow-key\x1b[22m\x1b[39m' to MOVE THE SELECTION HIGHLIGHT. \n \
                Press '\x1b[38;5;205m\x1b[1mESC\x1b[22m\x1b[39m' to return to the PREVIOUS SCREEN or '\x1b[38;5;205m\x1b[1mQ\x1b[22m\x1b[39m' to EXIT! 💋"
            )
            .as_bytes()
            .into_text()
            .unwrap()
    );

    //                      <-- TAB TITLES -->
    // Each column is as wide as the wider of its two labels, with both labels
    // centered in it, so the dividers of the two tab rows line up.
    let col_widths: Vec<usize> = AudioProcessingState::EXPLANATIONS
        .iter()
        .enumerate()
        .map(|(i, explain)| {
            let selectable = AudioProcessingState::SELECTABLES[i + 1].label().trim().chars().count();
            selectable.max(explain.label().trim().chars().count())
        })
        .collect();

    let selectable_titles: Vec<Line> = AudioProcessingState::SELECTABLES
        .iter()
        .skip(1) // skip Default
        .zip(&col_widths)
        .map(|(s, w)| Line::from(format!("{:^w$}", s.label().trim(), w = *w)))
        .collect();

    let explanation_titles: Vec<Line> = AudioProcessingState::EXPLANATIONS
        .iter()
        .zip(&col_widths)
        .map(|(s, w)| Line::from(format!("{:^w$}", s.label().trim(), w = *w)))
        .collect();

    //                      <-- TAB ELEMENTS -->
    let selectable_tabs = Tabs::new(selectable_titles)
        .select(if menu.selected_row == 0 { Some(menu.selected_column) } else { None })
        .highlight_style(Style::default().add_modifier(Modifier::REVERSED))
        .divider(" | ");

    let explanation_tabs = Tabs::new(explanation_titles)
        .select(if menu.selected_row == 1 { Some(menu.selected_column) } else { None })
        .highlight_style(Style::default().add_modifier(Modifier::REVERSED))
        .divider(" | ");

    //                      <-- DIRECTIONS -->
    // Explanations run down the LEFT side of inner[6], sharing rows with the
    // art on the right, art's own rect/rendering stays untouched.
    match state {
        AudioProcessingState::Default => {},
        AudioProcessingState::Convert => {
            convert_state.poll_job();

            let rows: Rc<[Rect]> = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1),  // rows[0]: intro line
                Constraint::Length(1),  // rows[1]: status line
                Constraint::Length(1),  // rows[2]: lossless note
            ])
            .split(inner[5]);

            let cols: Rc<[Rect]> = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(60), // cols[0]: fields + progress
                Constraint::Percentage(40), // cols[1]: art
            ])
            .split(inner[6]);

            render_intro_and_status(
                frame, &rows, convert_state.job(), "converting",
                "Fill in the fields, then focus ▶ convert and press '\x1b[38;5;205m\x1b[1mENTER\x1b[22m\x1b[39m'.",
            );

            // FLAC and WAV keep every sample, so the bitrate picker has nothing to
            // aim at. Saying so here is what keeps a greyed-out "n/a" from reading
            // like a bug the user caused.
            let note_text = if convert_state.bitrate_ignored() {
                format!("{} stores every sample as-is, so bitrate doesn't apply. Sample rate and channels still do.", convert_state.format.label())
            } else {
                "Bitrate sets how many bits a second of sound gets: higher is cleaner and bigger.".to_string()
            };
            frame.render_widget(
                Paragraph::new(styles::left_directions(rows[2].width, rows[2].height).render(&note_text).as_bytes().into_text().unwrap()),
                rows[2],
            );

            let field_area: Rc<[Rect]> = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),  // field_area[0]: input path (own row)
                Constraint::Length(3),  // field_area[1]: output path (own row)
                Constraint::Length(3),  // field_area[2]: format/bitrate/rate/channels/run
                Constraint::Length(3),  // field_area[3]: progress gauge
            ])
            .split(cols[0]);

            let option_cols: Rc<[Rect]> = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Fill(1), // [0:] format
                Constraint::Fill(1), // [1:] bitrate
                Constraint::Fill(1), // [2:] sample rate
                Constraint::Fill(1), // [3:] channels
                Constraint::Fill(1), // [4:] run
            ])
            .split(field_area[2]);

            let focused = convert_state.menu.focus_field();
            let editing = convert_state.menu.editing;

            render_path_field(frame, AudioConvertField::InputPath, focused, editing, &convert_state.input_file_path, field_area[0]);
            render_path_field(frame, AudioConvertField::OutputPath, focused, editing, &convert_state.output_file_path, field_area[1]);

            let bitrate_text = if convert_state.bitrate_ignored() { IGNORED_VALUE } else { convert_state.bitrate.label() };

            render_value_field(frame, AudioConvertField::Format, focused, editing, convert_state.format.label(), option_cols[0]);
            render_value_field(frame, AudioConvertField::Bitrate, focused, editing, bitrate_text, option_cols[1]);
            render_value_field(frame, AudioConvertField::SampleRate, focused, editing, convert_state.sample_rate.label(), option_cols[2]);
            render_value_field(frame, AudioConvertField::Channels, focused, editing, convert_state.channels.label(), option_cols[3]);
            render_value_field(frame, AudioConvertField::Run, focused, editing, AudioConvertField::Run.label(), option_cols[4]);

            render_progress_gauge(frame, field_area[3], convert_state.job());
        },
        AudioProcessingState::ExplainConvert => {
            let rows: Rc<[Rect]> = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1),  // rows[0]: intro line 1
                Constraint::Length(1),  // rows[1]: intro line 2
                Constraint::Length(1),  // rows[2]: outro line
            ])
            .split(inner[5]);

            let cols: Rc<[Rect]> = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(60), // cols[0]: format list
                Constraint::Percentage(40), // cols[1]: art
            ])
            .split(inner[6]);

            let explain_rows: Rc<[Rect]> = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(7), // explain_rows[0]: format list
                Constraint::Length(6), // explain_rows[1]: bitrate/rate/channel list
                Constraint::Fill(1),   // explain_rows[2]: unused space
            ])
            .split(cols[0]);

            frame.render_widget(explanations::explain_audio_convert_intro_line_1(rows[0].width, rows[0].height), rows[0]);
            frame.render_widget(explanations::explain_audio_convert_intro_line_2(rows[1].width, rows[1].height), rows[1]);
            frame.render_widget(explanations::explain_audio_convert_outro_line(rows[2].width, rows[2].height), rows[2]);
            frame.render_widget(explanations::explain_audio_convert_format_list(explain_rows[0].width, explain_rows[0].height), explain_rows[0]);
            frame.render_widget(explanations::explain_audio_convert_bitrate_list(explain_rows[1].width, explain_rows[1].height), explain_rows[1]);
        },
        AudioProcessingState::Extract => {
            extract_state.poll_job();

            let rows: Rc<[Rect]> = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1),  // rows[0]: intro line
                Constraint::Length(1),  // rows[1]: status line
                Constraint::Length(1),  // rows[2]: mode note
            ])
            .split(inner[5]);

            let cols: Rc<[Rect]> = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(60), // cols[0]: fields + progress
                Constraint::Percentage(40), // cols[1]: art
            ])
            .split(inner[6]);

            render_intro_and_status(
                frame, &rows, extract_state.job(), "extracting",
                "Point at a video, then focus ▶ extract and press '\x1b[38;5;205m\x1b[1mENTER\x1b[22m\x1b[39m'.",
            );

            // Copy mode runs no encoder at all, so format and bitrate stop meaning
            // anything. Spelling that out beats leaving the user to wonder why
            // cycling them changes nothing about the result.
            let note_text = if extract_state.encode_options_ignored() {
                "Copy lifts the existing track out untouched, so format and bitrate don't apply. Switch to re-encode to use them."
            } else {
                "Re-encode runs the track through a new encoder: slower and lossy, but it lands in any container."
            };
            frame.render_widget(
                Paragraph::new(styles::left_directions(rows[2].width, rows[2].height).render(note_text).as_bytes().into_text().unwrap()),
                rows[2],
            );

            let field_area: Rc<[Rect]> = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),  // field_area[0]: input path (own row)
                Constraint::Length(3),  // field_area[1]: output path (own row)
                Constraint::Length(3),  // field_area[2]: mode/format/bitrate/run
                Constraint::Length(3),  // field_area[3]: progress gauge
            ])
            .split(cols[0]);

            let option_cols: Rc<[Rect]> = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Fill(1), // mode
                Constraint::Fill(1), // format
                Constraint::Fill(1), // bitrate
                Constraint::Fill(1), // run
            ])
            .split(field_area[2]);

            let focused = extract_state.menu.focus_field();
            let editing = extract_state.menu.editing;

            render_path_field(frame, AudioExtractField::InputPath, focused, editing, &extract_state.input_file_path, field_area[0]);
            render_path_field(frame, AudioExtractField::OutputPath, focused, editing, &extract_state.output_file_path, field_area[1]);

            let ignored = extract_state.encode_options_ignored();
            let format_text = if ignored { IGNORED_VALUE } else { extract_state.format.label() };
            let bitrate_text = if ignored || extract_state.format.is_lossless() { IGNORED_VALUE } else { extract_state.bitrate.label() };

            render_value_field(frame, AudioExtractField::Mode, focused, editing, extract_state.mode.label(), option_cols[0]);
            render_value_field(frame, AudioExtractField::Format, focused, editing, format_text, option_cols[1]);
            render_value_field(frame, AudioExtractField::Bitrate, focused, editing, bitrate_text, option_cols[2]);
            render_value_field(frame, AudioExtractField::Run, focused, editing, AudioExtractField::Run.label(), option_cols[3]);

            render_progress_gauge(frame, field_area[3], extract_state.job());
        },
        AudioProcessingState::ExplainExtract => {
            let rows: Rc<[Rect]> = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1),  // rows[0]: intro line 1
                Constraint::Length(1),  // rows[1]: intro line 2
                Constraint::Length(1),  // rows[2]: outro line
            ])
            .split(inner[5]);

            let cols: Rc<[Rect]> = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(60), // cols[0]: format list
                Constraint::Percentage(40), // cols[1]: art
            ])
            .split(inner[6]);

            frame.render_widget(explanations::explain_audio_extract_intro_line_1(rows[0].width, rows[0].height), rows[0]);
            frame.render_widget(explanations::explain_audio_extract_intro_line_2(rows[1].width, rows[1].height), rows[1]);
            frame.render_widget(explanations::explain_audio_extract_outro_line(rows[2].width, rows[2].height), rows[2]);
            frame.render_widget(explanations::explain_audio_extract_format_list(cols[0].width, cols[0].height), cols[0]);
        },
        AudioProcessingState::Trim => {
            trim_state.poll_job();

            let rows: Rc<[Rect]> = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1),  // rows[0]: intro line
                Constraint::Length(1),  // rows[1]: status line
                Constraint::Length(1),  // rows[2]: empty/unused space
            ])
            .split(inner[5]);

            let cols: Rc<[Rect]> = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(60), // cols[0]: fields + progress
                Constraint::Percentage(40), // cols[1]: art
            ])
            .split(inner[6]);

            render_intro_and_status(
                frame, &rows, trim_state.job(), "trimming",
                "Fill in the fields, then focus ▶ trim and press '\x1b[38;5;205m\x1b[1mENTER\x1b[22m\x1b[39m'.",
            );

            let field_area: Rc<[Rect]> = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),  // field_area[0]: input path (own row)
                Constraint::Length(3),  // field_area[1]: output path (own row)
                Constraint::Length(3),  // field_area[2]: start/end/run
                Constraint::Length(3),  // field_area[3]: progress gauge
            ])
            .split(cols[0]);

            let option_cols: Rc<[Rect]> = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Fill(1), // start
                Constraint::Fill(1), // end
                Constraint::Fill(1), // run
            ])
            .split(field_area[2]);

            let focused = trim_state.menu.focus_field();
            let editing = trim_state.menu.editing;

            render_path_field(frame, AudioTrimField::InputPath, focused, editing, &trim_state.input_file_path, field_area[0]);
            render_path_field(frame, AudioTrimField::OutputPath, focused, editing, &trim_state.output_file_path, field_area[1]);

            render_path_field(frame, AudioTrimField::Start, focused, editing, &trim_state.start_time, option_cols[0]);
            render_path_field(frame, AudioTrimField::End, focused, editing, &trim_state.end_time, option_cols[1]);
            render_value_field(frame, AudioTrimField::Run, focused, editing, AudioTrimField::Run.label(), option_cols[2]);

            render_progress_gauge(frame, field_area[3], trim_state.job());
        },
        AudioProcessingState::ExplainTrim => {
            let rows: Rc<[Rect]> = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1),  // rows[0]: intro line 1
                Constraint::Length(1),  // rows[1]: intro line 2
                Constraint::Length(1),  // rows[2]: outro line
            ])
            .split(inner[5]);

            let cols: Rc<[Rect]> = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(60), // cols[0]: format list
                Constraint::Percentage(40), // cols[1]: art
            ])
            .split(inner[6]);

            frame.render_widget(explanations::explain_audio_trim_intro_line_1(rows[0].width, rows[0].height), rows[0]);
            frame.render_widget(explanations::explain_audio_trim_intro_line_2(rows[1].width, rows[1].height), rows[1]);
            frame.render_widget(explanations::explain_audio_trim_outro_line(rows[2].width, rows[2].height), rows[2]);
            frame.render_widget(explanations::explain_audio_trim_format_list(cols[0].width, cols[0].height), cols[0]);
        },
        AudioProcessingState::Speed => {
            speed_state.poll_job();

            let rows: Rc<[Rect]> = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1),  // rows[0]: intro line
                Constraint::Length(1),  // rows[1]: status line
                Constraint::Length(1),  // rows[2]: speed legend
            ])
            .split(inner[5]);

            let cols: Rc<[Rect]> = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(60), // cols[0]: fields + progress
                Constraint::Percentage(40), // cols[1]: art
            ])
            .split(inner[6]);

            render_intro_and_status(
                frame, &rows, speed_state.job(), "respeeding",
                "Fill in the fields, then focus ▶ respeed and press '\x1b[38;5;205m\x1b[1mENTER\x1b[22m\x1b[39m'.",
            );

            // Same idea as the compress screen's CRF legend: the slider shows a bare
            // multiplier, and this is what tells the user what that multiplier does.
            let legend_text = format!(
                "speed scale \u{2022} 0.25x quarter speed \u{2022} 0.50x half \u{2022} 1.00x untouched \u{2022} 2.00x double \u{2022} 4.00x quadruple \u{2022} pitch stays put throughout   [now: {}]",
                speed_label(speed_state.speed),
            );
            frame.render_widget(
                Paragraph::new(styles::left_directions(rows[2].width, rows[2].height).render(&legend_text).as_bytes().into_text().unwrap()),
                rows[2],
            );

            let field_area: Rc<[Rect]> = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),  // field_area[0]: input path (own row)
                Constraint::Length(3),  // field_area[1]: output path (own row)
                Constraint::Length(3),  // field_area[2]: speed slider/format/bitrate/run
                Constraint::Length(3),  // field_area[3]: progress gauge
            ])
            .split(cols[0]);

            let option_cols: Rc<[Rect]> = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Fill(2), // speed slider (wider: it carries a value + label)
                Constraint::Fill(1), // format
                Constraint::Fill(1), // bitrate
                Constraint::Fill(1), // run
            ])
            .split(field_area[2]);

            let focused = speed_state.menu.focus_field();
            let editing = speed_state.menu.editing;

            render_path_field(frame, AudioSpeedField::InputPath, focused, editing, &speed_state.input_file_path, field_area[0]);
            render_path_field(frame, AudioSpeedField::OutputPath, focused, editing, &speed_state.output_file_path, field_area[1]);

            let bitrate_text = if speed_state.bitrate_ignored() { IGNORED_VALUE } else { speed_state.bitrate.label() };

            render_slider_field(
                frame, AudioSpeedField::Speed, focused, editing,
                speed_ratio(speed_state.speed), speed_state.speed_text(), option_cols[0],
            );
            render_value_field(frame, AudioSpeedField::Format, focused, editing, speed_state.format.label(), option_cols[1]);
            render_value_field(frame, AudioSpeedField::Bitrate, focused, editing, bitrate_text, option_cols[2]);
            render_value_field(frame, AudioSpeedField::Run, focused, editing, AudioSpeedField::Run.label(), option_cols[3]);

            render_progress_gauge(frame, field_area[3], speed_state.job());
        },
        AudioProcessingState::ExplainSpeed => {
            let rows: Rc<[Rect]> = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1),  // rows[0]: intro line 1
                Constraint::Length(1),  // rows[1]: intro line 2
                Constraint::Length(1),  // rows[2]: outro line
            ])
            .split(inner[5]);

            let cols: Rc<[Rect]> = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(60), // cols[0]: filter list
                Constraint::Percentage(40), // cols[1]: art
            ])
            .split(inner[6]);

            frame.render_widget(explanations::explain_audio_speed_intro_line_1(rows[0].width, rows[0].height), rows[0]);
            frame.render_widget(explanations::explain_audio_speed_intro_line_2(rows[1].width, rows[1].height), rows[1]);
            frame.render_widget(explanations::explain_audio_speed_outro_line(rows[2].width, rows[2].height), rows[2]);
            frame.render_widget(explanations::explain_audio_speed_format_list(cols[0].width, cols[0].height), cols[0]);
        },
        AudioProcessingState::Fade => {
            fade_state.poll_job();

            let rows: Rc<[Rect]> = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1),  // rows[0]: intro line
                Constraint::Length(1),  // rows[1]: status line
                Constraint::Length(1),  // rows[2]: duration note
            ])
            .split(inner[5]);

            let cols: Rc<[Rect]> = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(60), // cols[0]: fields + progress
                Constraint::Percentage(40), // cols[1]: art
            ])
            .split(inner[6]);

            render_intro_and_status(
                frame, &rows, fade_state.job(), "fading",
                "Fill in the fields, then focus ▶ fade and press '\x1b[38;5;205m\x1b[1mENTER\x1b[22m\x1b[39m'.",
            );

            // A fade-out is measured back from the end, so until the length has been
            // probed it silently can't be applied. This is the warning that keeps
            // that from looking like the fade-out field is simply broken.
            let note_text = if fade_state.duration_known() {
                "Lengths are plain seconds: \"3\" is a three second fade, \"0\" skips that end entirely."
            } else {
                "⚠ Track length not read yet: confirm the input path with '\x1b[38;5;205m\x1b[1mENTER\x1b[22m\x1b[39m' so the fade-out knows where the end is."
            };
            frame.render_widget(
                Paragraph::new(styles::left_directions(rows[2].width, rows[2].height).render(note_text).as_bytes().into_text().unwrap()),
                rows[2],
            );

            let field_area: Rc<[Rect]> = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),  // field_area[0]: input path (own row)
                Constraint::Length(3),  // field_area[1]: output path (own row)
                Constraint::Length(3),  // field_area[2]: fade in/fade out/format/run
                Constraint::Length(3),  // field_area[3]: progress gauge
            ])
            .split(cols[0]);

            let option_cols: Rc<[Rect]> = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Fill(1), // fade in
                Constraint::Fill(1), // fade out
                Constraint::Fill(1), // format
                Constraint::Fill(1), // run
            ])
            .split(field_area[2]);

            let focused = fade_state.menu.focus_field();
            let editing = fade_state.menu.editing;

            render_path_field(frame, AudioFadeField::InputPath, focused, editing, &fade_state.input_file_path, field_area[0]);
            render_path_field(frame, AudioFadeField::OutputPath, focused, editing, &fade_state.output_file_path, field_area[1]);

            render_path_field(frame, AudioFadeField::FadeIn, focused, editing, &fade_state.fade_in, option_cols[0]);
            render_path_field(frame, AudioFadeField::FadeOut, focused, editing, &fade_state.fade_out, option_cols[1]);
            render_value_field(frame, AudioFadeField::Format, focused, editing, fade_state.format.label(), option_cols[2]);
            render_value_field(frame, AudioFadeField::Run, focused, editing, AudioFadeField::Run.label(), option_cols[3]);

            render_progress_gauge(frame, field_area[3], fade_state.job());
        },
        AudioProcessingState::ExplainFade => {
            let rows: Rc<[Rect]> = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1),  // rows[0]: intro line 1
                Constraint::Length(1),  // rows[1]: intro line 2
                Constraint::Length(1),  // rows[2]: outro line
            ])
            .split(inner[5]);

            let cols: Rc<[Rect]> = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(60), // cols[0]: filter list
                Constraint::Percentage(40), // cols[1]: art
            ])
            .split(inner[6]);

            frame.render_widget(explanations::explain_audio_fade_intro_line_1(rows[0].width, rows[0].height), rows[0]);
            frame.render_widget(explanations::explain_audio_fade_intro_line_2(rows[1].width, rows[1].height), rows[1]);
            frame.render_widget(explanations::explain_audio_fade_outro_line(rows[2].width, rows[2].height), rows[2]);
            frame.render_widget(explanations::explain_audio_fade_format_list(cols[0].width, cols[0].height), cols[0]);
        },
        AudioProcessingState::Volume => {
            volume_state.poll_job();

            let rows: Rc<[Rect]> = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1),  // rows[0]: intro line
                Constraint::Length(1),  // rows[1]: status line
                Constraint::Length(1),  // rows[2]: gain legend
            ])
            .split(inner[5]);

            let cols: Rc<[Rect]> = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(60), // cols[0]: fields + progress
                Constraint::Percentage(40), // cols[1]: art
            ])
            .split(inner[6]);

            render_intro_and_status(
                frame, &rows, volume_state.job(), "applying gain",
                "Fill in the fields, then focus ▶ apply and press '\x1b[38;5;205m\x1b[1mENTER\x1b[22m\x1b[39m'.",
            );

            let legend_text = format!(
                "gain scale \u{2022} -30 dB nearly silent \u{2022} -6 dB about half \u{2022} 0 dB untouched \u{2022} +6 dB about double \u{2022} +12 dB and up risks clipping   [now: {}]",
                gain_label(volume_state.gain),
            );
            frame.render_widget(
                Paragraph::new(styles::left_directions(rows[2].width, rows[2].height).render(&legend_text).as_bytes().into_text().unwrap()),
                rows[2],
            );

            let field_area: Rc<[Rect]> = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),  // field_area[0]: input path (own row)
                Constraint::Length(3),  // field_area[1]: output path (own row)
                Constraint::Length(3),  // field_area[2]: gain slider/format/bitrate/run
                Constraint::Length(3),  // field_area[3]: progress gauge
            ])
            .split(cols[0]);

            let option_cols: Rc<[Rect]> = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Fill(2), // gain slider (wider: it carries a value + label)
                Constraint::Fill(1), // format
                Constraint::Fill(1), // bitrate
                Constraint::Fill(1), // run
            ])
            .split(field_area[2]);

            let focused = volume_state.menu.focus_field();
            let editing = volume_state.menu.editing;

            render_path_field(frame, AudioVolumeField::InputPath, focused, editing, &volume_state.input_file_path, field_area[0]);
            render_path_field(frame, AudioVolumeField::OutputPath, focused, editing, &volume_state.output_file_path, field_area[1]);

            let bitrate_text = if volume_state.bitrate_ignored() { IGNORED_VALUE } else { volume_state.bitrate.label() };

            render_slider_field(
                frame, AudioVolumeField::Gain, focused, editing,
                gain_ratio(volume_state.gain), volume_state.gain_text(), option_cols[0],
            );
            render_value_field(frame, AudioVolumeField::Format, focused, editing, volume_state.format.label(), option_cols[1]);
            render_value_field(frame, AudioVolumeField::Bitrate, focused, editing, bitrate_text, option_cols[2]);
            render_value_field(frame, AudioVolumeField::Run, focused, editing, AudioVolumeField::Run.label(), option_cols[3]);

            render_progress_gauge(frame, field_area[3], volume_state.job());
        },
        AudioProcessingState::ExplainVolume => {
            let rows: Rc<[Rect]> = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1),  // rows[0]: intro line 1
                Constraint::Length(1),  // rows[1]: intro line 2
                Constraint::Length(1),  // rows[2]: outro line
            ])
            .split(inner[5]);

            let cols: Rc<[Rect]> = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(60), // cols[0]: filter list
                Constraint::Percentage(40), // cols[1]: art
            ])
            .split(inner[6]);

            frame.render_widget(explanations::explain_audio_volume_intro_line_1(rows[0].width, rows[0].height), rows[0]);
            frame.render_widget(explanations::explain_audio_volume_intro_line_2(rows[1].width, rows[1].height), rows[1]);
            frame.render_widget(explanations::explain_audio_volume_outro_line(rows[2].width, rows[2].height), rows[2]);
            frame.render_widget(explanations::explain_audio_volume_format_list(cols[0].width, cols[0].height), cols[0]);
        },
    }

    frame.render_widget(selection_guide, inner[1]);
    frame.render_widget(selectable_tabs, inner[3]);
    frame.render_widget(explanation_tabs, inner[4]);
    // EMO_POP instead of the Video scene's mascot, so a glance at the art is
    // enough to tell which half of the app you're standing in.
    background::render_emo_pop_bottom_right(frame, art_area(&inner));
    frame.render_widget(component::social_footer_hyperlinks(inner[7].width, inner[7].height), inner[7]);
}
