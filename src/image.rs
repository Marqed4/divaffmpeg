use std::{rc::Rc};

use ansi_to_tui::IntoText as _;

use ratatui::prelude::*;
use ratatui::widgets::*;

use crate::explanations;
use crate::background;
use crate::component;
use crate::styles;
// Image draws the same furniture Video and Audio do, so the helpers live over
// in video.rs and get borrowed here rather than copied into a third set.
use crate::video::{art_area, render_intro_and_status, render_path_field,
    render_progress_gauge, render_slider_field, render_value_field};
use crate::implementations::{
    FieldSet,
    ImageConvertField, ImageConvertState, quality_label, quality_ratio,
    ImageResizeField, ImageResizeState,
    ImageCompressField, ImageCompressState,
    ImageCropField, ImageCropState,
    ImageRotateField, ImageRotateState,
    ImageFilterField, ImageFilterState, strength_ratio,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ImageProcessingState {
    Default,
    Convert,
    ExplainConvert,
    Resize,
    ExplainResize,
    Compress,
    ExplainCompress,
    Crop,
    ExplainCrop,
    Rotate,
    ExplainRotate,
    Filter,
    ExplainFilter,
}

//                      <-- SELECTABLES -->
impl ImageProcessingState {
    const SELECTABLES: [ImageProcessingState; 7] = [
        ImageProcessingState::Default,
        ImageProcessingState::Convert,
        ImageProcessingState::Resize,
        ImageProcessingState::Compress,
        ImageProcessingState::Crop,
        ImageProcessingState::Rotate,
        ImageProcessingState::Filter,
    ];

    const EXPLANATIONS: [ImageProcessingState; 6] = [
        ImageProcessingState::ExplainConvert,
        ImageProcessingState::ExplainResize,
        ImageProcessingState::ExplainCompress,
        ImageProcessingState::ExplainCrop,
        ImageProcessingState::ExplainRotate,
        ImageProcessingState::ExplainFilter,
    ];

    fn label(&self) -> &'static str {
        match self {
            Self::Default => "default",
            Self::Convert => "   Convert   ",
            Self::ExplainConvert => "Convert Guide",
            Self::Resize => "   Resize   ",
            Self::ExplainResize => "Resize Guide",
            Self::Compress => "   Compress   ",
            Self::ExplainCompress => "Compress Guide",
            Self::Crop => "   Crop   ",
            Self::ExplainCrop => "Crop Guide",
            Self::Rotate => "   Rotate   ",
            Self::ExplainRotate => "Rotate Guide",
            Self::Filter => "   Filter   ",
            Self::ExplainFilter => "Filter Guide",
        }
    }
}

// A twin of audio.rs's `AudioDirectionsMenuState` for the same reason that one is
// a twin of video.rs's: the struct hands back its own scene's enum.
#[derive(Debug)]
pub struct ImageDirectionsMenuState {
    selected_row: usize,
    selected_column: usize,
}

impl ImageDirectionsMenuState {
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

    pub fn selected_state(&self) -> ImageProcessingState {
        if self.selected_row == 0 {
            ImageProcessingState::SELECTABLES[self.selected_column + 1]
        } else {
            ImageProcessingState::EXPLANATIONS[self.selected_column]
        }
    }
}

/// What a value field should show when the current format makes it moot, e.g.
/// a quality slider pointed at PNG. Showing the stale number instead would imply
/// it still reaches ffmpeg, which it doesn't.
const IGNORED_VALUE: &str = "n/a";

/// Draws one note line under the status line, the spot every screen uses for its
/// legend or warning.
fn render_note(frame: &mut Frame<'_>, area: Rect, text: &str) {
    frame.render_widget(
        Paragraph::new(styles::left_directions(area.width, area.height).render(text).as_bytes().into_text().unwrap()),
        area,
    );
}

/// The three rows every editable screen starts with (intro, status, note) beside
/// the 60/40 split between fields and art, returned as (rows, fields column).
fn screen_layout(inner: &[Rect]) -> (Rc<[Rect]>, Rect) {
    let rows: Rc<[Rect]> = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),  // rows[0]: intro line
            Constraint::Length(1),  // rows[1]: status line
            Constraint::Length(1),  // rows[2]: note line
        ])
        .split(inner[5]);

    let cols: Rc<[Rect]> = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(60), // cols[0]: fields + progress
            Constraint::Percentage(40), // cols[1]: art
        ])
        .split(inner[6]);

    (rows, cols[0])
}

/// Splits the fields column into the four stacked rows every editable screen
/// uses: input path, output path, option row, progress gauge.
fn field_rows(area: Rect) -> Rc<[Rect]> {
    Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),  // [0]: input path (own row)
            Constraint::Length(3),  // [1]: output path (own row)
            Constraint::Length(3),  // [2]: option row
            Constraint::Length(3),  // [3]: progress gauge
        ])
        .split(area)
}

fn option_cols(area: Rect, weights: &[u32]) -> Rc<[Rect]> {
    Layout::default()
        .direction(Direction::Horizontal)
        .constraints(weights.iter().map(|w| Constraint::Fill(*w as u16)).collect::<Vec<_>>())
        .split(area)
}

/// Guide screens all share one shape: three one-line paragraphs on top and a
/// list down the left of the body, art on the right.
fn render_guide(
    frame: &mut Frame<'_>,
    inner: &[Rect],
    intro_1: fn(u16, u16) -> Paragraph<'static>,
    intro_2: fn(u16, u16) -> Paragraph<'static>,
    outro: fn(u16, u16) -> Paragraph<'static>,
    list: fn(u16, u16) -> Paragraph<'static>,
) {
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
            Constraint::Percentage(60), // cols[0]: list
            Constraint::Percentage(40), // cols[1]: art
        ])
        .split(inner[6]);

    frame.render_widget(intro_1(rows[0].width, rows[0].height), rows[0]);
    frame.render_widget(intro_2(rows[1].width, rows[1].height), rows[1]);
    frame.render_widget(outro(rows[2].width, rows[2].height), rows[2]);
    frame.render_widget(list(cols[0].width, cols[0].height), cols[0]);
}

pub fn render(
    frame: &mut Frame<'_>,
    state: ImageProcessingState,
    menu: &ImageDirectionsMenuState,
    convert_state: &mut ImageConvertState,
    resize_state: &mut ImageResizeState,
    compress_state: &mut ImageCompressState,
    crop_state: &mut ImageCropState,
    rotate_state: &mut ImageRotateState,
    filter_state: &mut ImageFilterState,
) {
    let size: Rect = frame.area();

    let outer: Rc<[Rect]> = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Fill(1),
        ])
        .split(size);

    let image_block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .title("✦  Diva FFMPEG: Image Processing  ✦")
        .title_alignment(Alignment::Center)
        .bold();

    let image_inner: Rect = image_block.inner(outer[0]);
    frame.render_widget(image_block, outer[0]);

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
        .split(image_inner);

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
    let col_widths: Vec<usize> = ImageProcessingState::EXPLANATIONS
        .iter()
        .enumerate()
        .map(|(i, explain)| {
            let selectable = ImageProcessingState::SELECTABLES[i + 1].label().trim().chars().count();
            selectable.max(explain.label().trim().chars().count())
        })
        .collect();

    let selectable_titles: Vec<Line> = ImageProcessingState::SELECTABLES
        .iter()
        .skip(1) // skip Default
        .zip(&col_widths)
        .map(|(s, w)| Line::from(format!("{:^w$}", s.label().trim(), w = *w)))
        .collect();

    let explanation_titles: Vec<Line> = ImageProcessingState::EXPLANATIONS
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
    match state {
        ImageProcessingState::Default => {},
        ImageProcessingState::Convert => {
            convert_state.poll_job();
            let (rows, fields) = screen_layout(&inner);

            render_intro_and_status(
                frame, &rows, convert_state.job(), "converting",
                "Fill in the fields, then focus ▶ convert and press '\x1b[38;5;205m\x1b[1mENTER\x1b[22m\x1b[39m'.",
            );

            // Same reasoning as the audio bitrate note: spell out why a field reads
            // "n/a" so it doesn't look like a bug the user caused.
            let note_text = if convert_state.quality_ignored() {
                format!("{} never trades detail for size, so quality doesn't apply. Pick .jpg or .webp to use it.", convert_state.format.label())
            } else {
                "Quality is how much detail survives: higher is cleaner and bigger.".to_string()
            };
            render_note(frame, rows[2], &note_text);

            let field_area = field_rows(fields);
            let option_cols = option_cols(field_area[2], &[1, 2, 1]);

            let focused = convert_state.menu.focus_field();
            let editing = convert_state.menu.editing;

            render_path_field(frame, ImageConvertField::InputPath, focused, editing, &convert_state.input_file_path, field_area[0]);
            render_path_field(frame, ImageConvertField::OutputPath, focused, editing, &convert_state.output_file_path, field_area[1]);

            render_value_field(frame, ImageConvertField::Format, focused, editing, convert_state.format.label(), option_cols[0]);
            if convert_state.quality_ignored() {
                render_value_field(frame, ImageConvertField::Quality, focused, editing, IGNORED_VALUE, option_cols[1]);
            } else {
                render_slider_field(
                    frame, ImageConvertField::Quality, focused, editing,
                    quality_ratio(convert_state.quality), convert_state.quality_text(), option_cols[1],
                );
            }
            render_value_field(frame, ImageConvertField::Run, focused, editing, ImageConvertField::Run.label(), option_cols[2]);

            render_progress_gauge(frame, field_area[3], convert_state.job());
        },
        ImageProcessingState::ExplainConvert => render_guide(
            frame, &inner,
            explanations::explain_image_convert_intro_line_1,
            explanations::explain_image_convert_intro_line_2,
            explanations::explain_image_convert_outro_line,
            explanations::explain_image_convert_format_list,
        ),
        ImageProcessingState::Resize => {
            resize_state.poll_job();
            let (rows, fields) = screen_layout(&inner);

            render_intro_and_status(
                frame, &rows, resize_state.job(), "resizing",
                "Fill in the fields, then focus ▶ resize and press '\x1b[38;5;205m\x1b[1mENTER\x1b[22m\x1b[39m'.",
            );
            render_note(frame, rows[2], "Leave width or height blank to keep the picture's proportions. Fill both to force an exact size.");

            let field_area = field_rows(fields);
            let option_cols = option_cols(field_area[2], &[1, 1, 1]);

            let focused = resize_state.menu.focus_field();
            let editing = resize_state.menu.editing;

            render_path_field(frame, ImageResizeField::InputPath, focused, editing, &resize_state.input_file_path, field_area[0]);
            render_path_field(frame, ImageResizeField::OutputPath, focused, editing, &resize_state.output_file_path, field_area[1]);

            render_path_field(frame, ImageResizeField::Width, focused, editing, &resize_state.width, option_cols[0]);
            render_path_field(frame, ImageResizeField::Height, focused, editing, &resize_state.height, option_cols[1]);
            render_value_field(frame, ImageResizeField::Run, focused, editing, ImageResizeField::Run.label(), option_cols[2]);

            render_progress_gauge(frame, field_area[3], resize_state.job());
        },
        ImageProcessingState::ExplainResize => render_guide(
            frame, &inner,
            explanations::explain_image_resize_intro_line_1,
            explanations::explain_image_resize_intro_line_2,
            explanations::explain_image_resize_outro_line,
            explanations::explain_image_resize_format_list,
        ),
        ImageProcessingState::Compress => {
            compress_state.poll_job();
            let (rows, fields) = screen_layout(&inner);

            render_intro_and_status(
                frame, &rows, compress_state.job(), "compressing",
                "Fill in the fields, then focus ▶ compress and press '\x1b[38;5;205m\x1b[1mENTER\x1b[22m\x1b[39m'.",
            );

            // Same idea as the video compress screen's CRF legend: the slider shows
            // a bare percentage, and this says what that percentage costs.
            let legend_text = format!(
                "quality scale \u{2022} 90+ near-lossless \u{2022} 60-85 sweet spot \u{2022} 30-55 visibly softer \u{2022} under 30 rough   [now: {}]",
                quality_label(compress_state.quality),
            );
            render_note(frame, rows[2], &legend_text);

            let field_area = field_rows(fields);
            let option_cols = option_cols(field_area[2], &[2, 1, 1]);

            let focused = compress_state.menu.focus_field();
            let editing = compress_state.menu.editing;

            render_path_field(frame, ImageCompressField::InputPath, focused, editing, &compress_state.input_file_path, field_area[0]);
            render_path_field(frame, ImageCompressField::OutputPath, focused, editing, &compress_state.output_file_path, field_area[1]);

            render_slider_field(
                frame, ImageCompressField::Quality, focused, editing,
                quality_ratio(compress_state.quality), compress_state.quality_text(), option_cols[0],
            );
            render_value_field(frame, ImageCompressField::Format, focused, editing, compress_state.format.label(), option_cols[1]);
            render_value_field(frame, ImageCompressField::Run, focused, editing, ImageCompressField::Run.label(), option_cols[2]);

            render_progress_gauge(frame, field_area[3], compress_state.job());
        },
        ImageProcessingState::ExplainCompress => render_guide(
            frame, &inner,
            explanations::explain_image_compress_intro_line_1,
            explanations::explain_image_compress_intro_line_2,
            explanations::explain_image_compress_outro_line,
            explanations::explain_image_compress_format_list,
        ),
        ImageProcessingState::Crop => {
            crop_state.poll_job();
            let (rows, fields) = screen_layout(&inner);

            render_intro_and_status(
                frame, &rows, crop_state.job(), "cropping",
                "Fill in the fields, then focus ▶ crop and press '\x1b[38;5;205m\x1b[1mENTER\x1b[22m\x1b[39m'.",
            );
            render_note(frame, rows[2], "x and y are the rectangle's top-left corner in pixels from the picture's top-left. Width and height are its size.");

            let field_area = field_rows(fields);
            let option_cols = option_cols(field_area[2], &[1, 1, 1, 1, 1]);

            let focused = crop_state.menu.focus_field();
            let editing = crop_state.menu.editing;

            render_path_field(frame, ImageCropField::InputPath, focused, editing, &crop_state.input_file_path, field_area[0]);
            render_path_field(frame, ImageCropField::OutputPath, focused, editing, &crop_state.output_file_path, field_area[1]);

            render_path_field(frame, ImageCropField::X, focused, editing, &crop_state.x, option_cols[0]);
            render_path_field(frame, ImageCropField::Y, focused, editing, &crop_state.y, option_cols[1]);
            render_path_field(frame, ImageCropField::Width, focused, editing, &crop_state.width, option_cols[2]);
            render_path_field(frame, ImageCropField::Height, focused, editing, &crop_state.height, option_cols[3]);
            render_value_field(frame, ImageCropField::Run, focused, editing, ImageCropField::Run.label(), option_cols[4]);

            render_progress_gauge(frame, field_area[3], crop_state.job());
        },
        ImageProcessingState::ExplainCrop => render_guide(
            frame, &inner,
            explanations::explain_image_crop_intro_line_1,
            explanations::explain_image_crop_intro_line_2,
            explanations::explain_image_crop_outro_line,
            explanations::explain_image_crop_format_list,
        ),
        ImageProcessingState::Rotate => {
            rotate_state.poll_job();
            let (rows, fields) = screen_layout(&inner);

            render_intro_and_status(
                frame, &rows, rotate_state.job(), "rotating",
                "Fill in the fields, then focus ▶ rotate and press '\x1b[38;5;205m\x1b[1mENTER\x1b[22m\x1b[39m'.",
            );
            render_note(frame, rows[2], "Quarter turns swap width and height. Mirror and flip keep the size and just reverse one axis.");

            let field_area = field_rows(fields);
            let option_cols = option_cols(field_area[2], &[2, 1]);

            let focused = rotate_state.menu.focus_field();
            let editing = rotate_state.menu.editing;

            render_path_field(frame, ImageRotateField::InputPath, focused, editing, &rotate_state.input_file_path, field_area[0]);
            render_path_field(frame, ImageRotateField::OutputPath, focused, editing, &rotate_state.output_file_path, field_area[1]);

            render_value_field(frame, ImageRotateField::Transform, focused, editing, rotate_state.transform.label(), option_cols[0]);
            render_value_field(frame, ImageRotateField::Run, focused, editing, ImageRotateField::Run.label(), option_cols[1]);

            render_progress_gauge(frame, field_area[3], rotate_state.job());
        },
        ImageProcessingState::ExplainRotate => render_guide(
            frame, &inner,
            explanations::explain_image_rotate_intro_line_1,
            explanations::explain_image_rotate_intro_line_2,
            explanations::explain_image_rotate_outro_line,
            explanations::explain_image_rotate_format_list,
        ),
        ImageProcessingState::Filter => {
            filter_state.poll_job();
            let (rows, fields) = screen_layout(&inner);

            render_intro_and_status(
                frame, &rows, filter_state.job(), "filtering",
                "Fill in the fields, then focus ▶ apply and press '\x1b[38;5;205m\x1b[1mENTER\x1b[22m\x1b[39m'.",
            );
            let note_text = if filter_state.strength_ignored() {
                "Invert swaps every color outright, so strength doesn't apply. Pick another effect to use it."
            } else {
                "Strength scales the effect: low is subtle, high is heavy. Apply it to the output again to stack a second pass."
            };
            render_note(frame, rows[2], note_text);

            let field_area = field_rows(fields);
            let option_cols = option_cols(field_area[2], &[2, 2, 1]);

            let focused = filter_state.menu.focus_field();
            let editing = filter_state.menu.editing;

            render_path_field(frame, ImageFilterField::InputPath, focused, editing, &filter_state.input_file_path, field_area[0]);
            render_path_field(frame, ImageFilterField::OutputPath, focused, editing, &filter_state.output_file_path, field_area[1]);

            render_value_field(frame, ImageFilterField::Effect, focused, editing, filter_state.effect.label(), option_cols[0]);
            if filter_state.strength_ignored() {
                render_value_field(frame, ImageFilterField::Strength, focused, editing, IGNORED_VALUE, option_cols[1]);
            } else {
                render_slider_field(
                    frame, ImageFilterField::Strength, focused, editing,
                    strength_ratio(filter_state.strength), filter_state.strength_text(), option_cols[1],
                );
            }
            render_value_field(frame, ImageFilterField::Run, focused, editing, ImageFilterField::Run.label(), option_cols[2]);

            render_progress_gauge(frame, field_area[3], filter_state.job());
        },
        ImageProcessingState::ExplainFilter => render_guide(
            frame, &inner,
            explanations::explain_image_filter_intro_line_1,
            explanations::explain_image_filter_intro_line_2,
            explanations::explain_image_filter_outro_line,
            explanations::explain_image_filter_format_list,
        ),
    }

    frame.render_widget(selection_guide, inner[1]);
    frame.render_widget(selectable_tabs, inner[3]);
    frame.render_widget(explanation_tabs, inner[4]);
    // Felix instead of Video's mascot or Audio's EMO_POP, so a glance at the art is
    // enough to tell which of the three scenes you're standing in.
    background::render_felix_bottom_right(frame, art_area(&inner));
    frame.render_widget(component::social_footer_hyperlinks(inner[7].width, inner[7].height), inner[7]);
}
