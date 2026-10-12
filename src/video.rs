use std::{rc::Rc};

use ansi_to_tui::IntoText as _;

use ratatui_textarea::{CursorMove, TextArea};
use ratatui::prelude::*;
use ratatui::widgets::*;

use crate::explanations;
use crate::background;
use crate::component;
use crate::styles;
use crate::implementations::{
    FieldSet, FfmpegJob,
    ConvertField, ConvertState,
    CompressField, CompressState, crf_quality_label, crf_ratio,
    TrimField, TrimState,
    MergeField, MergeState,
    ExtractionField, ExtractionState,
    ResizeField, ResizeState,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VideoProcessingState {
    Default,
    Convert,
    ExplainConvert,
    Compress,
    ExplainCompress,
    Trim,
    ExplainTrim,
    Merge,
    ExplainMerge,
    Extraction,
    ExplainExtraction,
    Resize,
    ExplainResize,
}

//                      <-- SELECTABLES -->
impl VideoProcessingState {
    const SELECTABLES: [VideoProcessingState; 7] = [
        VideoProcessingState::Default,
        VideoProcessingState::Convert,
        VideoProcessingState::Compress,
        VideoProcessingState::Trim,
        VideoProcessingState::Merge,
        VideoProcessingState::Extraction,
        VideoProcessingState::Resize,
    ];

    const EXPLANATIONS: [VideoProcessingState; 6] = [
        VideoProcessingState::ExplainConvert,
        VideoProcessingState::ExplainCompress,
        VideoProcessingState::ExplainTrim,
        VideoProcessingState::ExplainMerge,
        VideoProcessingState::ExplainExtraction,
        VideoProcessingState::ExplainResize,
    ];

    fn label(&self) -> &'static str {
        match self {
            Self::Default => "default",
            Self::Convert => "   Convert   ",
            Self::ExplainConvert => "Convert Guide",
            Self::Compress => "   Compress   ",
            Self::ExplainCompress => "Compress Guide",
            Self::Trim => "   Trim   ",
            Self::ExplainTrim => "Trim Guide",
            Self::Merge => "   Merge   ",
            Self::ExplainMerge => "Merge Guide",
            Self::Extraction => "Frame Extraction",
            Self::ExplainExtraction => "Frame Extraction Guide",
            Self::Resize => "   Resize   ",
            Self::ExplainResize => "Resize Guide",
        }
    }
}

#[derive(Debug)]
pub struct DirectionsMenuState {
    selected_row: usize,
    selected_column: usize,
}

impl DirectionsMenuState {
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

    pub fn selected_state(&self) -> VideoProcessingState {
        if self.selected_row == 0 {
            VideoProcessingState::SELECTABLES[self.selected_column + 1]
        } else {
            VideoProcessingState::EXPLANATIONS[self.selected_column]
        }
    }
}

//                      <-- PATH TAB-AUTOCOMPLETE -->
fn split_dir_prefix(current: &str) -> (String, String) {
    match current.rfind(['/', '\\']) {
        Some(i) => (current[..=i].to_string(), current[i + 1..].to_string()),
        None => (String::new(), current.to_string()),
    }
}

fn common_prefix(items: &[String]) -> String {
    let mut prefix = match items.first() {
        Some(first) => first.clone(),
        None => return String::new(),
    };
    for item in &items[1..] {
        while !item.to_lowercase().starts_with(&prefix.to_lowercase()) {
            prefix.pop();
            if prefix.is_empty() { return prefix; }
        }
    }
    prefix
}

/// Shell-style tab completion for a path typed into the input/output fields.
/// Completes to the single match, or the longest common prefix across matches.
pub fn autocomplete_path(current: &str) -> String {
    let (dir_str, prefix) = split_dir_prefix(current);
    let scan_dir = if dir_str.is_empty() { ".".to_string() } else { dir_str.clone() };

    let Ok(entries) = std::fs::read_dir(&scan_dir) else { return current.to_string() };

    let mut matches: Vec<String> = entries
        .filter_map(|e| e.ok())
        .filter_map(|e| {
            let name = e.file_name().to_string_lossy().to_string();
            name.to_lowercase().starts_with(&prefix.to_lowercase()).then_some(name)
        })
        .collect();

    if matches.is_empty() { return current.to_string(); }
    matches.sort();

    let completion = if matches.len() == 1 { matches[0].clone() } else { common_prefix(&matches) };
    if completion.is_empty() { return current.to_string(); }

    let mut result = format!("{dir_str}{completion}");
    if matches.len() == 1 && std::path::Path::new(&scan_dir).join(&completion).is_dir() {
        result.push('/');
    }
    result
}

/// Replaces a field's typed path with its tab-completed form, cursor moved to the end.
pub fn complete_textarea(text_area: &mut TextArea<'static>) {
    let current = text_area.lines().join("");
    let completed = autocomplete_path(&current);
    *text_area = TextArea::new(vec![completed]);
    text_area.move_cursor(CursorMove::End);
}

//                      <-- SHARED RENDER HELPERS -->
fn field_block<F: FieldSet>(field: F, focused: F, editing: bool) -> Block<'static> {
    let focused_here = focused == field;
    let style = if focused_here && editing {
        Style::default().fg(Color::Rgb(255, 133, 200)).add_modifier(Modifier::BOLD | Modifier::REVERSED)
    } else if focused_here {
        Style::default().fg(Color::Rgb(255, 133, 200)).add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::Rgb(150, 150, 150))
    };

    Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(style)
        .title(field.label())
}

/// Widest tab row ("Convert Guide | Compress Guide | ...") plus a little slack.
/// Above this width the art can climb past the tab rows without colliding.
/// (Six equal-width columns: 82 label cols + 12 tab padding + 15 dividers = 109.)
const TABS_RESERVED_COLS: u16 = 110;

/// The art is taller than `inner[6]`, so it loses its head when it only gets
/// that slot. Hand it the blank rows above too: always the explanation/UI band
/// (`inner[5]`), and on wide terminals the gap + tab rows as well, since the
/// tabs never reach that far right. The rect the art actually draws into stays
/// art-width and right-aligned, so nothing on the left is overwritten.
fn art_area(inner: &[Rect]) -> Rect {
    let top: Rect = if inner[6].width >= TABS_RESERVED_COLS + 34 { inner[2] } else { inner[5] };
    let bottom: Rect = inner[6];

    Rect {
        x: bottom.x,
        y: top.y,
        width: bottom.width,
        height: bottom.y + bottom.height - top.y,
    }
}

fn render_path_field<F: FieldSet>(frame: &mut Frame<'_>, field: F, focused: F, editing: bool, text_area: &TextArea<'static>, area: Rect) {
    let block = field_block(field, focused, editing);
    let inner = block.inner(area);
    frame.render_widget(block, area);
    frame.render_widget(text_area, inner);
}

fn render_value_field<F: FieldSet>(frame: &mut Frame<'_>, field: F, focused: F, editing: bool, value: &str, area: Rect) {
    let block = field_block(field, focused, editing);
    let inner = block.inner(area);
    frame.render_widget(block, area);
    frame.render_widget(
        Paragraph::new(value).alignment(Alignment::Center).style(Style::default().fg(Color::White)),
        inner,
    );
}

/// Draws a field as a filled bar instead of a static label, for values picked
/// off a continuous range (compress's CRF slider) rather than a fixed list.
fn render_slider_field<F: FieldSet>(frame: &mut Frame<'_>, field: F, focused: F, editing: bool, ratio: f64, label: String, area: Rect) {
    let block = field_block(field, focused, editing);
    let inner = block.inner(area);
    frame.render_widget(block, area);
    let gauge = Gauge::default()
        .gauge_style(Style::default().fg(Color::Rgb(255, 133, 200)))
        .ratio(ratio.clamp(0.0, 1.0))
        .label(label);
    frame.render_widget(gauge, inner);
}

fn render_intro_and_status(frame: &mut Frame<'_>, rows: &[Rect], job: Option<&FfmpegJob>, verb: &str, idle_msg: &str) {
    let intro_text = "'\x1b[38;5;218m\x1b[1mWASD\x1b[22m\x1b[39m'/arrow-key equivalents to move between fields below \u{2022} '\x1b[38;5;218m\x1b[1mA\x1b[22m\x1b[39m'/'\x1b[38;5;218m\x1b[1mleft arrow-key\x1b[22m\x1b[39m' changes focus, W/S cycles a value \u{2022} '\x1b[38;5;205m\x1b[1mENTER\x1b[22m\x1b[39m' EDITS PATH or runs the job.";
    frame.render_widget(
        Paragraph::new(styles::left_directions(rows[0].width, rows[0].height).render(intro_text).as_bytes().into_text().unwrap()),
        rows[0],
    );

    let status_text: String = match job {
        Some(job) => job.status_text(verb),
        None => idle_msg.to_string(),
    };
    frame.render_widget(
        Paragraph::new(styles::left_directions(rows[1].width, rows[1].height).render(&status_text).as_bytes().into_text().unwrap()),
        rows[1],
    );
}

fn render_progress_gauge(frame: &mut Frame<'_>, area: Rect, job: Option<&FfmpegJob>) {
    let ratio = job.map(|j| j.ratio()).unwrap_or(0.0);
    let gauge = Gauge::default()
        .block(Block::default().borders(Borders::ALL).border_type(BorderType::Rounded).title("♡ progress ♡"))
        .gauge_style(Style::default().fg(Color::Rgb(255, 133, 200)))
        .ratio(ratio);
    frame.render_widget(gauge, area);
}

pub fn render(
    frame: &mut Frame<'_>,
    state: VideoProcessingState,
    menu: &DirectionsMenuState,
    convert_state: &mut ConvertState,
    compress_state: &mut CompressState,
    trim_state: &mut TrimState,
    merge_state: &mut MergeState,
    extraction_state: &mut ExtractionState,
    resize_state: &mut ResizeState,
) {
    let size: Rect = frame.area();

    let outer: Rc<[Rect]> = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Fill(1),
        ])
        .split(size);

    let video_block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .title("✦  Diva FFMPEG: Video Processing  ✦")
        .title_alignment(Alignment::Center)
        .bold();

    let video_inner: Rect = video_block.inner(outer[0]);
    frame.render_widget(video_block, outer[0]);

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
        .split(video_inner);

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
    let col_widths: Vec<usize> = VideoProcessingState::EXPLANATIONS
        .iter()
        .enumerate()
        .map(|(i, explain)| {
            let selectable = VideoProcessingState::SELECTABLES[i + 1].label().trim().chars().count();
            selectable.max(explain.label().trim().chars().count())
        })
        .collect();

    let selectable_titles: Vec<Line> = VideoProcessingState::SELECTABLES
        .iter()
        .skip(1) // skip Default
        .zip(&col_widths)
        .map(|(s, w)| Line::from(format!("{:^w$}", s.label().trim(), w = *w)))
        .collect();

    let explanation_titles: Vec<Line> = VideoProcessingState::EXPLANATIONS
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
        VideoProcessingState::Default => {},
        VideoProcessingState::Convert => {
            convert_state.poll_job();

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
                frame, &rows, convert_state.job(), "converting",
                "Fill in the fields, then focus ▶ convert and press '\x1b[38;5;205m\x1b[1mENTER\x1b[22m\x1b[39m'.",
            );

            let field_area: Rc<[Rect]> = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),  // field_area[0]: input path (own row)
                Constraint::Length(3),  // field_area[1]: output path (own row)
                Constraint::Length(3),  // field_area[2]: format/codec/res/fps/run
                Constraint::Length(3),  // field_area[3]: progress gauge
            ])
            .split(cols[0]);

            let option_cols: Rc<[Rect]> = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Fill(1), // [0:] format
                Constraint::Fill(1), // [1:] codec
                Constraint::Fill(1), // [2:] resolution
                Constraint::Fill(1), // [3:] fps
                Constraint::Fill(1), // [4:] run
            ])
            .split(field_area[2]);

            let focused = convert_state.menu.focus_field();
            let editing = convert_state.menu.editing;

            render_path_field(frame, ConvertField::InputPath, focused, editing, &convert_state.input_file_path, field_area[0]);
            render_path_field(frame, ConvertField::OutputPath, focused, editing, &convert_state.output_file_path, field_area[1]);

            render_value_field(frame, ConvertField::Format, focused, editing, convert_state.format.label(), option_cols[0]);
            render_value_field(frame, ConvertField::Codec, focused, editing, convert_state.codec.label(), option_cols[1]);
            render_value_field(frame, ConvertField::Resolution, focused, editing, convert_state.resolution.label(), option_cols[2]);
            render_value_field(frame, ConvertField::Fps, focused, editing, convert_state.fps.label(), option_cols[3]);
            render_value_field(frame, ConvertField::Run, focused, editing, ConvertField::Run.label(), option_cols[4]);

            render_progress_gauge(frame, field_area[3], convert_state.job());
        },
        VideoProcessingState::ExplainConvert => {
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
                Constraint::Length(14), // explain_rows[0]: format list
                Constraint::Length(3),  // explain_rows[1]: codec list
                Constraint::Fill(1),    // explain_rows[2]: unused space
            ])
            .split(cols[0]);

            frame.render_widget(explanations::explain_convert_intro_line_1(rows[0].width, rows[0].height), rows[0]);
            frame.render_widget(explanations::explain_convert_intro_line_2(rows[1].width, rows[1].height), rows[1]);
            frame.render_widget(explanations::explain_convert_outro_line(rows[2].width, rows[2].height), rows[2]);
            frame.render_widget(explanations::explain_convert_format_list(explain_rows[0].width, explain_rows[0].height), explain_rows[0]);
            frame.render_widget(explanations::explain_convert_codec_list(explain_rows[1].width, explain_rows[1].height), explain_rows[1]);
        },
        VideoProcessingState::Compress => {
            compress_state.poll_job();

            let rows: Rc<[Rect]> = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1),  // rows[0]: intro line
                Constraint::Length(1),  // rows[1]: status line
                Constraint::Length(1),  // rows[2]: CRF quality legend
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
                frame, &rows, compress_state.job(), "compressing",
                "Fill in the fields, then focus ▶ compress and press '\x1b[38;5;205m\x1b[1mENTER\x1b[22m\x1b[39m'.",
            );

            // The compression slider is a raw CRF number, so this legend is what
            // actually tells the user what tradeoff their current pick means.
            let legend_text = format!(
                "quality scale \u{2022} 0-17 near-lossless \u{2022} 18-22 excellent \u{2022} 23-27 great balance \u{2022} 28-32 good/smaller \u{2022} 33-39 visible loss \u{2022} 40-51 heavy loss   [now: {}]",
                crf_quality_label(compress_state.crf),
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
                Constraint::Length(3),  // field_area[2]: codec/fps/quality slider/run
                Constraint::Length(3),  // field_area[3]: progress gauge
            ])
            .split(cols[0]);

            let option_cols: Rc<[Rect]> = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Fill(1), // codec
                Constraint::Fill(1), // fps
                Constraint::Fill(2), // quality slider (wider: it carries a value + label)
                Constraint::Fill(1), // run
            ])
            .split(field_area[2]);

            let focused = compress_state.menu.focus_field();
            let editing = compress_state.menu.editing;

            render_path_field(frame, CompressField::InputPath, focused, editing, &compress_state.input_file_path, field_area[0]);
            render_path_field(frame, CompressField::OutputPath, focused, editing, &compress_state.output_file_path, field_area[1]);

            render_value_field(frame, CompressField::Codec, focused, editing, compress_state.codec.label(), option_cols[0]);
            render_value_field(frame, CompressField::Fps, focused, editing, compress_state.fps.label(), option_cols[1]);
            render_slider_field(
                frame, CompressField::Crf, focused, editing,
                crf_ratio(compress_state.crf), format!("CRF {}", compress_state.crf), option_cols[2],
            );
            render_value_field(frame, CompressField::Run, focused, editing, CompressField::Run.label(), option_cols[3]);

            render_progress_gauge(frame, field_area[3], compress_state.job());
        },
        VideoProcessingState::ExplainCompress => {
            let rows: Rc<[Rect]> = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1),  // rows[0]: intro line 1
                Constraint::Length(1),  // rows[1]: intro line 2
                Constraint::Length(1),  // rows[2]: outro line (black bg)
            ])
            .split(inner[5]);

            let cols: Rc<[Rect]> = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(60), // cols[0]: format list
                Constraint::Percentage(40), // cols[1]: art
            ])
            .split(inner[6]);

            frame.render_widget(explanations::explain_compress_intro_line_1(rows[0].width, rows[0].height), rows[0]);
            frame.render_widget(explanations::explain_compress_intro_line_2(rows[1].width, rows[1].height), rows[1]);
            frame.render_widget(explanations::explain_compress_outro_line(rows[2].width, rows[2].height), rows[2]);
            frame.render_widget(explanations::explain_compress_format_list(cols[0].width, cols[0].height), cols[0]);
        },
        VideoProcessingState::Trim => {
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

            render_path_field(frame, TrimField::InputPath, focused, editing, &trim_state.input_file_path, field_area[0]);
            render_path_field(frame, TrimField::OutputPath, focused, editing, &trim_state.output_file_path, field_area[1]);

            render_path_field(frame, TrimField::Start, focused, editing, &trim_state.start_time, option_cols[0]);
            render_path_field(frame, TrimField::End, focused, editing, &trim_state.end_time, option_cols[1]);
            render_value_field(frame, TrimField::Run, focused, editing, TrimField::Run.label(), option_cols[2]);

            render_progress_gauge(frame, field_area[3], trim_state.job());
        },
        VideoProcessingState::ExplainTrim => {
            let rows: Rc<[Rect]> = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1),  // rows[0]: intro line 1
                Constraint::Length(1),  // rows[1]: intro line 2
                Constraint::Length(1),  // rows[2]: outro line (black bg)
            ])
            .split(inner[5]);

            let cols: Rc<[Rect]> = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(60), // cols[0]: format list
                Constraint::Percentage(40), // cols[1]: art
            ])
            .split(inner[6]);

            frame.render_widget(explanations::explain_trim_intro_line_1(rows[0].width, rows[0].height), rows[0]);
            frame.render_widget(explanations::explain_trim_intro_line_2(rows[1].width, rows[1].height), rows[1]);
            frame.render_widget(explanations::explain_trim_outro_line(rows[2].width, rows[2].height), rows[2]);
            frame.render_widget(explanations::explain_trim_format_list(cols[0].width, cols[0].height), cols[0]);
        },
        VideoProcessingState::Merge => {
            merge_state.poll_job();

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
                frame, &rows, merge_state.job(), "merging",
                "Fill in clip A, clip B, and output, then focus ▶ merge and press '\x1b[38;5;205m\x1b[1mENTER\x1b[22m\x1b[39m'.",
            );

            let field_area: Rc<[Rect]> = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),  // field_area[0]: clip A (own row)
                Constraint::Length(3),  // field_area[1]: clip B (own row)
                Constraint::Length(3),  // field_area[2]: output/run
                Constraint::Length(3),  // field_area[3]: progress gauge
            ])
            .split(cols[0]);

            let option_cols: Rc<[Rect]> = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Fill(2), // output
                Constraint::Fill(1), // run
            ])
            .split(field_area[2]);

            let focused = merge_state.menu.focus_field();
            let editing = merge_state.menu.editing;

            render_path_field(frame, MergeField::InputA, focused, editing, &merge_state.input_a_path, field_area[0]);
            render_path_field(frame, MergeField::InputB, focused, editing, &merge_state.input_b_path, field_area[1]);
            render_path_field(frame, MergeField::OutputPath, focused, editing, &merge_state.output_file_path, option_cols[0]);
            render_value_field(frame, MergeField::Run, focused, editing, MergeField::Run.label(), option_cols[1]);

            render_progress_gauge(frame, field_area[3], merge_state.job());
        },
        VideoProcessingState::ExplainMerge => {
            let rows: Rc<[Rect]> = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1),  // rows[0]: intro line 1
                Constraint::Length(1),  // rows[1]: intro line 2
                Constraint::Length(1),  // rows[2]: outro line (black bg)
            ])
            .split(inner[5]);

            let cols: Rc<[Rect]> = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(60), // cols[0]: format list
                Constraint::Percentage(40), // cols[1]: art
            ])
            .split(inner[6]);

            frame.render_widget(explanations::explain_merge_intro_line_1(rows[0].width, rows[0].height), rows[0]);
            frame.render_widget(explanations::explain_merge_intro_line_2(rows[1].width, rows[1].height), rows[1]);
            frame.render_widget(explanations::explain_merge_outro_line(rows[2].width, rows[2].height), rows[2]);
            frame.render_widget(explanations::explain_merge_format_list(cols[0].width, cols[0].height), cols[0]);
        },
        VideoProcessingState::Extraction => {
            extraction_state.poll_job();

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
                frame, &rows, extraction_state.job(), "extracting",
                "Fill in the fields, then focus ▶ extract and press '\x1b[38;5;205m\x1b[1mENTER\x1b[22m\x1b[39m'.",
            );

            let field_area: Rc<[Rect]> = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),  // field_area[0]: input path (own row)
                Constraint::Length(3),  // field_area[1]: output path (own row)
                Constraint::Length(3),  // field_area[2]: start/end/format/fps/run
                Constraint::Length(3),  // field_area[3]: progress gauge
            ])
            .split(cols[0]);

            let option_cols: Rc<[Rect]> = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Fill(1), // start
                Constraint::Fill(1), // end
                Constraint::Fill(1), // format
                Constraint::Fill(1), // fps
                Constraint::Fill(1), // run
            ])
            .split(field_area[2]);

            let focused = extraction_state.menu.focus_field();
            let editing = extraction_state.menu.editing;

            render_path_field(frame, ExtractionField::Input, focused, editing, &extraction_state.input_path, field_area[0]);
            render_path_field(frame, ExtractionField::OutputPath, focused, editing, &extraction_state.output_file_path, field_area[1]);

            render_path_field(frame, ExtractionField::Start, focused, editing, &extraction_state.start_time, option_cols[0]);
            render_path_field(frame, ExtractionField::End, focused, editing, &extraction_state.end_time, option_cols[1]);
            render_value_field(frame, ExtractionField::Format, focused, editing, extraction_state.format.label(), option_cols[2]);
            render_value_field(frame, ExtractionField::Fps, focused, editing, extraction_state.fps.label(), option_cols[3]);
            render_value_field(frame, ExtractionField::Run, focused, editing, ExtractionField::Run.label(), option_cols[4]);

            render_progress_gauge(frame, field_area[3], extraction_state.job());
        },
        VideoProcessingState::ExplainExtraction => {
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
                Constraint::Percentage(60), // cols[0]: format/fps lists
                Constraint::Percentage(40), // cols[1]: art
            ])
            .split(inner[6]);

            let explain_rows: Rc<[Rect]> = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(7), // explain_rows[0]: format list
                Constraint::Length(3), // explain_rows[1]: fps list
                Constraint::Fill(1),   // explain_rows[2]: unused space
            ])
            .split(cols[0]);

            frame.render_widget(explanations::explain_extraction_intro_line_1(rows[0].width, rows[0].height), rows[0]);
            frame.render_widget(explanations::explain_extraction_intro_line_2(rows[1].width, rows[1].height), rows[1]);
            frame.render_widget(explanations::explain_extraction_outro_line(rows[2].width, rows[2].height), rows[2]);
            frame.render_widget(explanations::explain_extraction_format_list(explain_rows[0].width, explain_rows[0].height), explain_rows[0]);
            frame.render_widget(explanations::explain_extraction_fps_list(explain_rows[1].width, explain_rows[1].height), explain_rows[1]);
        },
        VideoProcessingState::Resize => {
            resize_state.poll_job();

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
                frame, &rows, resize_state.job(), "resizing",
                "Fill in the fields, then focus ▶ resize and press '\x1b[38;5;205m\x1b[1mENTER\x1b[22m\x1b[39m'.",
            );

            let field_area: Rc<[Rect]> = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),  // field_area[0]: input path (own row)
                Constraint::Length(3),  // field_area[1]: output path (own row)
                Constraint::Length(3),  // field_area[2]: width/height/ratio lock/run
                Constraint::Length(3),  // field_area[3]: progress gauge
            ])
            .split(cols[0]);

            let option_cols: Rc<[Rect]> = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Fill(1), // width
                Constraint::Fill(1), // height
                Constraint::Fill(1), // ratio lock
                Constraint::Fill(1), // run
            ])
            .split(field_area[2]);

            let focused = resize_state.menu.focus_field();
            let editing = resize_state.menu.editing;

            render_path_field(frame, ResizeField::InputPath, focused, editing, &resize_state.input_file_path, field_area[0]);
            render_path_field(frame, ResizeField::OutputPath, focused, editing, &resize_state.output_file_path, field_area[1]);

            render_path_field(frame, ResizeField::Width, focused, editing, &resize_state.width, option_cols[0]);
            render_path_field(frame, ResizeField::Height, focused, editing, &resize_state.height, option_cols[1]);
            render_value_field(frame, ResizeField::Lock, focused, editing, resize_state.lock_label(), option_cols[2]);
            render_value_field(frame, ResizeField::Run, focused, editing, ResizeField::Run.label(), option_cols[3]);

            render_progress_gauge(frame, field_area[3], resize_state.job());
        },
        VideoProcessingState::ExplainResize => {
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
                Constraint::Percentage(60), // cols[0]: resolution list
                Constraint::Percentage(40), // cols[1]: art
            ])
            .split(inner[6]);

            frame.render_widget(explanations::explain_resize_intro_line_1(rows[0].width, rows[0].height), rows[0]);
            frame.render_widget(explanations::explain_resize_intro_line_2(rows[1].width, rows[1].height), rows[1]);
            frame.render_widget(explanations::explain_resize_outro_line(rows[2].width, rows[2].height), rows[2]);
            frame.render_widget(explanations::explain_resize_format_list(cols[0].width, cols[0].height), cols[0]);
        },
    }

    frame.render_widget(selection_guide, inner[1]);
    frame.render_widget(selectable_tabs, inner[3]);
    frame.render_widget(explanation_tabs, inner[4]);
    background::render_bottom_right(frame, art_area(&inner));
    frame.render_widget(component::social_footer_hyperlinks(inner[7].width, inner[7].height), inner[7]);
}