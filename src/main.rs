use std::{fs::File, io::{self, Stdout}, rc::Rc};
use ansi_to_tui::IntoText as _;
use std::time::Duration;
use crossterm::{
    cursor::MoveTo,
    event::{self, Event, KeyCode},
    execute,
    terminal::{EnterAlternateScreen,
        LeaveAlternateScreen,
        disable_raw_mode, enable_raw_mode
    },
};

use ratatui::prelude::*;
use ratatui::widgets::*;

use std::fs::OpenOptions;
use std::io::Write;
use chrono::Utc;

mod home;
mod video;
mod audio;
mod image;
mod styles;
mod component;
mod background;
mod explanations;
mod implementations;

use home::{HomeState, HomeMenuState};
use video::{VideoProcessingState, DirectionsMenuState, complete_textarea};
use audio::{AudioProcessingState, AudioDirectionsMenuState};
use image::{ImageProcessingState, ImageDirectionsMenuState};
use implementations::{
    VideoConvertState, VideoConvertField, VideoCompressState, VideoCompressField,
    VideoTrimState, VideoTrimField, VideoMergeState, VideoMergeField,
    VideoExtractionState, VideoExtractionField,
    VideoResizeState, VideoResizeField,
    AudioConvertState, AudioConvertField,
    AudioExtractState, AudioExtractField,
    AudioTrimState, AudioTrimField,
    AudioSpeedState, AudioSpeedField,
    AudioFadeState, AudioFadeField,
    AudioVolumeState, AudioVolumeField,
    ImageConvertState, ImageConvertField,
    ImageResizeState, ImageResizeField,
    ImageCompressState, ImageCompressField,
    ImageCropState, ImageCropField,
    ImageRotateState, ImageRotateField,
    ImageFilterState, ImageFilterField,
};

//                      <-- SCENE STATE -->
// Tracks exactly which screen is live at any given time. Using a single enum
// instead of several independent bools makes it structurally impossible for
// two scenes to render (or receive key input) in the same frame.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Scene {
    Start,
    Home,
    ImageProcessing,
    VideoProcessing, 
    AudioProcessing,
    Exit,
}

/// Logs live in %LOCALAPPDATA%\DivaFFMPEG\log so the app works when installed under
/// Program Files (not writable) and regardless of the launch directory.
fn log_dir() -> std::path::PathBuf {
    let base = std::env::var_os("LOCALAPPDATA")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|| std::path::PathBuf::from("."));
    base.join("DivaFFMPEG").join("log")
}

fn main() -> Result<(), io::Error> {

    //                      <-- COMMAND LINE -->
    // `divaffmpeg` and `divaffmpeg diva` both launch the editor.
    match std::env::args().nth(1).as_deref() {
        None => {}
        Some(a) if a.eq_ignore_ascii_case("diva") => {}
        Some("-V") | Some("--version") => {
            println!("DivaFFMPEG {}", env!("CARGO_PKG_VERSION"));
            return Ok(());
        }
        Some(_) => {
            println!("Usage: divaffmpeg [diva]
  diva         launch the editor (default)
  -V, --version  print version");
            return Ok(());
        }
    }

    //                      <-- ENABLE TERMINAL -->
    enable_raw_mode()?;
    let mut stdout: Stdout = io::stdout();

    // The way 'execute!(stdout, EnterAlternateScreen)?;' works is by pushing invisible ascii
    // characters on the screen clearing chatter that was released from previous
    // use including compiling the .exe or any other related executions and. E.g., \x1b[?1049h
    execute!(stdout, EnterAlternateScreen)?;

    //                      <-- LOGGING APPLICATION STATE -->
    // This uses the standard write and file opening crates, as well as the chrono package to
    // accurately create a .log file that can be written to during a session of this terminal application.
    std::fs::create_dir_all(log_dir())?;
    let filename: String = log_dir()
        .join(format!("DivaFFMPEG-{}.log", Utc::now().format("%Y-%m-%d_%H-%M-%S")))
        .to_string_lossy()
        .into_owned();
    let mut _file: File = OpenOptions::new()
        .append(true)
        .create(true)
        .open(&filename)?;

    // Logging Var
    let mut last_logged_state = String::new();

    let backend: CrosstermBackend<Stdout> = CrosstermBackend::new(stdout);
    let mut terminal: Terminal<CrosstermBackend<Stdout>> = Terminal::new(backend)?;

    //                      <-- APPLICATION STATE -->
    let mut current_scene: Scene = Scene::Start;
    // Remembers which scene to return to when the user backs out of the exit prompt.
    let mut previous_scene: Scene = Scene::Start;

    // Home Menu States
    let mut home_state: HomeState = HomeState::Default;
    let mut home_menu: HomeMenuState = HomeMenuState::new();

    // Video Processing State
    let mut video_state: VideoProcessingState = VideoProcessingState::Default;
    let mut video_menu: DirectionsMenuState = DirectionsMenuState::new();
    let mut convert_state: VideoConvertState = VideoConvertState::new();
    let mut compress_state: VideoCompressState = VideoCompressState::new();
    let mut trim_state: VideoTrimState = VideoTrimState::new();
    let mut merge_state: VideoMergeState = VideoMergeState::new();
    let mut extraction_state: VideoExtractionState = VideoExtractionState::new();
    let mut resize_state: VideoResizeState = VideoResizeState::new();

    // Audio Processing State
    let mut audio_state: AudioProcessingState = AudioProcessingState::Default;
    let mut audio_menu: AudioDirectionsMenuState = AudioDirectionsMenuState::new();
    let mut audio_convert_state: AudioConvertState = AudioConvertState::new();
    let mut audio_extract_state: AudioExtractState = AudioExtractState::new();
    let mut audio_trim_state: AudioTrimState = AudioTrimState::new();
    let mut audio_speed_state: AudioSpeedState = AudioSpeedState::new();
    let mut audio_fade_state: AudioFadeState = AudioFadeState::new();
    let mut audio_volume_state: AudioVolumeState = AudioVolumeState::new();

    // Image Processing State
    let mut image_state: ImageProcessingState = ImageProcessingState::Default;
    let mut image_menu: ImageDirectionsMenuState = ImageDirectionsMenuState::new();
    let mut image_convert_state: ImageConvertState = ImageConvertState::new();
    let mut image_resize_state: ImageResizeState = ImageResizeState::new();
    let mut image_compress_state: ImageCompressState = ImageCompressState::new();
    let mut image_crop_state: ImageCropState = ImageCropState::new();
    let mut image_rotate_state: ImageRotateState = ImageRotateState::new();
    let mut image_filter_state: ImageFilterState = ImageFilterState::new();


    // Drain any stray input events left over from launching the process
    // (e.g. the Enter keystroke used to run the binary) so they don't
    // get misinterpreted as real user input once the app starts.
    while event::poll(Duration::from_millis(0))? {
        let _ = event::read()?;
    }

    // Tracks where the clickable footer (if any) landed in the last frame, so a real
    // OSC-8 hyperlink can be stamped over it after ratatui finishes drawing.
    let mut footer_rect: Option<Rect> = None;

    loop
    {
        terminal.draw(|frame| {
            // Creates our terminal frame.
            let size: Rect = frame.area();
            footer_rect = None;

            match current_scene {
                //                      <-- EXIT SCENE -->
                // The goal of this block is to render a splash screen for a user who is trying to exit the TUI.
                // Directions on how to restore the state previous to pressing q outside of a text input situation
                // are displayed. External links to app support and developer socials are also provided.
                Scene::Exit => {
                    let outer: Rc<[Rect]> = Layout::default()
                        .direction(Direction::Vertical)
                        .constraints([
                            Constraint::Fill(1),
                        ])
                        .split(size);

                    let intro_block = Block::default()
                        .borders(Borders::ALL)
                        .border_type(BorderType::Rounded)
                        .title("✦  Diva FFMPEG: Exit  ✦")
                        .title_alignment(Alignment::Center)
                        .bold();

                    let intro_inner = intro_block.inner(outer[0]);

                    frame.render_widget(intro_block, outer[0]);

                    let inner: Rc<[Rect]> = Layout::default()
                        .direction(Direction::Vertical)
                        .constraints([
                            Constraint::Fill(1),
                            Constraint::Length(1),
                            Constraint::Min(1),
                            Constraint::Length(1),
                        ])
                        .split(intro_inner);

                    let exit_page: Paragraph<'_> = Paragraph::new(styles::title(inner[0].width, inner[0].height).render(
                        "Press '\x1b[38;5;205m\x1b[1mESC\x1b[22m\x1b[39m' to return to the PREVIOUS SCREEN.\nPress '\x1b[38;5;205m\x1b[1mQ\x1b[22m\x1b[39m' again to EXIT! 💋").as_bytes().into_text().unwrap());
                    
                    frame.render_widget(exit_page, inner[0]);
                    frame.render_widget(component::social_footer_hyperlinks(inner[3].width, inner[3].height), inner[3]);
                    footer_rect = Some(inner[3]);

                    if last_logged_state != "Exit Screen" {
                        let _ = writeln!(_file, "Entered the exit screen at {}.", Utc::now().format("%H-%M-%S"));
                        last_logged_state = "Exit Screen".to_string();
                    }
                },

                //                      <-- START SCENE -->
                // The goal of this block is to render a splash screen for those who open the app.
                // A welcoming text, input directions, and external links to app support and
                // developer socials should be shown.
                Scene::Start => {
                    let outer: Rc<[Rect]> = Layout::default()
                        .direction(Direction::Vertical)
                        .constraints([Constraint::Fill(1)])
                        .split(size);

                    let intro_block = Block::default()
                        .borders(Borders::ALL)
                        .border_type(BorderType::Rounded)
                        .title("✦  Diva FFMPEG  ✦")
                        .title_alignment(Alignment::Center)
                        .bold();

                    let intro_inner = intro_block.inner(outer[0]);
                    frame.render_widget(intro_block, outer[0]);

                    let inner: Rc<[Rect]> = Layout::default()
                        .direction(Direction::Vertical)
                        .constraints([
                            Constraint::Length(8),    // inner[0]: project info blurb
                            Constraint::Length(1),    // inner[1]: directions bar, right up top
                            Constraint::Length(16),   // inner[2]: art (native 16 rows) + info box
                            Constraint::Length(16),   // inner[3]: art (native 16 rows)
                            Constraint::Length(1),    // inner[4]: footer links
                        ])
                        .split(intro_inner);

                    // Astolfo's native art is only ~42 cols wide, so giving it the full row leaves a
                    // wide dead-space column beside it on anything but a narrow terminal. Splitting
                    // each art row into an art column + a text column fills that space with copy
                    // instead of blank cells, and keeps the art itself centered within its own column.
                    const ART_COL_WIDTH: u16 = 46;

                    let art_row_top: Rc<[Rect]> = Layout::default()
                        .direction(Direction::Horizontal)
                        .constraints([Constraint::Length(ART_COL_WIDTH), Constraint::Fill(1)])
                        .split(inner[2]);

                    let art_row_bottom: Rc<[Rect]> = Layout::default()
                        .direction(Direction::Horizontal)
                        .constraints([Constraint::Length(ART_COL_WIDTH), Constraint::Fill(1)])
                        .split(inner[3]);

                    background::render_diva_top(frame, art_row_top[0]);
                    background::render_diva_bottom(frame, art_row_bottom[0]);

                    // One shared box for both blurbs, sized to its content (not the full
                    // art-column height) and pinned to the top of the column, so there's no
                    // dead space stretching it down and the eye lands on the directions first.
                    const INFO_BOX_HEIGHT: u16 = 11;
                    let info_area = Rect {
                        x: art_row_top[1].x,
                        y: art_row_top[1].y,
                        width: art_row_top[1].width,
                        height: INFO_BOX_HEIGHT,
                    };

                    let info_block = Block::default()
                        .borders(Borders::ALL)
                        .border_type(BorderType::Rounded)
                        .style(Style::default().fg(Color::Rgb(200, 160, 255)));
                    let info_inner = info_block.inner(info_area);
                    frame.render_widget(info_block, info_area);

                    let info_rows: Rc<[Rect]> = Layout::default()
                        .direction(Direction::Vertical)
                        .constraints([
                            Constraint::Length(1), // "What is FFmpeg?" heading
                            Constraint::Length(3), // FFmpeg blurb
                            Constraint::Length(1), // gap
                            Constraint::Length(1), // "What is Diva FFMPEG?" heading
                            Constraint::Length(3), // Diva FFMPEG blurb
                        ])
                        .split(info_inner);

                    let ffmpeg_heading = Paragraph::new("♦ What is FFmpeg? ♦")
                        .style(Style::default().fg(Color::Rgb(200, 160, 255)).add_modifier(Modifier::BOLD))
                        .alignment(Alignment::Center);

                    let ffmpeg_info = Paragraph::new(
                        "FFmpeg is the multimedia Swiss-army knife: it converts, transcodes, filters, and streams almost any audio or video format that exists, all through a dense wall of command-line flags most people never learn.",
                    )
                    .style(Style::default().fg(Color::Rgb(200, 160, 255)).add_modifier(Modifier::BOLD))
                    .alignment(Alignment::Center)
                    .wrap(Wrap { trim: true });

                    let project_heading = Paragraph::new("♦ What is Diva FFMPEG? ♦")
                        .style(Style::default().fg(Color::Rgb(255, 133, 200)).add_modifier(Modifier::BOLD))
                        .alignment(Alignment::Center);

                    let project_info = Paragraph::new(
                        "Diva FFMPEG puts that power behind a keyboard-driven menu: pick your inputs, watch a real progress bar, and skip the manpage archaeology. Convert, compress, trim, and merge, no flags required.",
                    )
                    .style(Style::default().fg(Color::Rgb(255, 133, 200)).add_modifier(Modifier::BOLD))
                    .alignment(Alignment::Center)
                    .wrap(Wrap { trim: true });

                    frame.render_widget(ffmpeg_heading, info_rows[0]);
                    frame.render_widget(ffmpeg_info, info_rows[1]);
                    frame.render_widget(project_heading, info_rows[3]);
                    frame.render_widget(project_info, info_rows[4]);

                    let introduction: Paragraph<'_> = Paragraph::new(styles::title(inner[0].width, inner[0].height).render(
                        "Diva FFMPEG wraps ffmpeg in a terminal UI, because typing flags is beneath her. \nConvert, compress, trim, and merge video: point, select, done. \nSlay your multimedia pipeline 💅 \nBy Marqed").as_bytes().into_text().unwrap());

                    let directions: Paragraph<'_> = Paragraph::new(styles::center_directions_transparent(inner[1].width, inner[1].height).blink().render(
                        "Press '\x1b[38;5;218m\x1b[1mENTER\x1b[22m\x1b[39m' to get started OR '\x1b[38;5;205m\x1b[1mQ\x1b[22m\x1b[39m' to EXIT! 💋").as_bytes().into_text().unwrap());

                    frame.render_widget(introduction, inner[0]);
                    frame.render_widget(directions, inner[1]);
                    frame.render_widget(component::social_footer_hyperlinks(inner[4].width, inner[4].height), inner[4]);
                    footer_rect = Some(inner[4]);

                    if last_logged_state != "Start Screen" {
                        let _ = writeln!(_file, "Entered the start screen at {}.", Utc::now().format("%H-%M-%S"));
                        last_logged_state = "Start Screen".to_string();
                    }
                },

                //                      <-- HOME SCENE -->
                // The goal of this block is to render the portion of this TUI that displays information
                // on how to reach and enter abstracted scenes involving image_processing, video_processing, audio_processing, etc.
                Scene::Home => {
                    home::render(frame, home_state, &home_menu);
                },

                //                      <-- VIDEO PROCESSING SCENE -->
                // The goal of this block is to render the portion of this TUI that displays information
                // and control used for video processing.
                Scene::VideoProcessing => {
                    video::render(
                        frame,
                        video_state,
                        &video_menu,
                        &mut convert_state,
                        &mut compress_state,
                        &mut trim_state,
                        &mut merge_state,
                        &mut extraction_state,
                        &mut resize_state,
                    );
                },

                //                      <-- IMAGE PROCESSING SCENE -->
                // The goal of this block is to render the portion of this TUI that displays information
                // and control used for image processing.
                Scene::ImageProcessing => {
                    image::render(
                        frame,
                        image_state,
                        &image_menu,
                        &mut image_convert_state,
                        &mut image_resize_state,
                        &mut image_compress_state,
                        &mut image_crop_state,
                        &mut image_rotate_state,
                        &mut image_filter_state,
                    );
                },

                //                      <-- AUDIO PROCESSING SCENE -->
                // The goal of this block is to render the portion of this TUI that displays information
                // and control used for audio processing.
                Scene::AudioProcessing => {
                    audio::render(
                        frame,
                        audio_state,
                        &audio_menu,
                        &mut audio_convert_state,
                        &mut audio_extract_state,
                        &mut audio_trim_state,
                        &mut audio_speed_state,
                        &mut audio_fade_state,
                        &mut audio_volume_state,
                    );
                },
            }
        })?;

        // Ratatui's diffed buffer has no notion of OSC-8, so the footer's real, clickable
        // hyperlink is stamped on directly over the already-drawn text right here.
        if let Some(rect) = footer_rect {
            let overlay: String = component::social_footer_overlay(rect.width, rect.height);
            let backend = terminal.backend_mut();
            execute!(backend, MoveTo(rect.x, rect.y))?;
            write!(backend, "{}", overlay)?;
            std::io::Write::flush(backend)?;
        }

        // The purpose of this block is to catch key strokes that trigger events and change code state.
        if event::poll(Duration::from_millis(10))? {
            if let Event::Key(k) = event::read()? {
                if k.kind == event::KeyEventKind::Press {
                    match (current_scene, k.code) {
                        //                      <-- FIELD TEXT-EDIT KEY EVENTS -->
                        // While a text field is being edited, every key (including 'q') must be
                        // forwarded to the textarea instead of being caught by a global shortcut.
                        (Scene::VideoProcessing, _)
                            if video_state == VideoProcessingState::Resize && resize_state.menu.editing =>
                        {
                            match k.code {
                                KeyCode::Enter | KeyCode::Esc => {
                                    resize_state.menu.editing = false;
                                    if resize_state.menu.focus_field() == VideoResizeField::InputPath {
                                        resize_state.sync_dimensions();
                                    }
                                },
                                KeyCode::Tab => {
                                    match resize_state.menu.focus_field() {
                                        VideoResizeField::InputPath => {
                                            complete_textarea(&mut resize_state.input_file_path);
                                            resize_state.sync_dimensions();
                                        },
                                        VideoResizeField::OutputPath => complete_textarea(&mut resize_state.output_file_path),
                                        _ => {},
                                    }
                                },
                                _ => {
                                    match resize_state.menu.focus_field() {
                                        VideoResizeField::InputPath => { resize_state.input_file_path.input(Event::Key(k)); },
                                        VideoResizeField::OutputPath => { resize_state.output_file_path.input(Event::Key(k)); },
                                        VideoResizeField::Width => { resize_state.width.input(Event::Key(k)); },
                                        VideoResizeField::Height => { resize_state.height.input(Event::Key(k)); },
                                        _ => {},
                                    }
                                },
                            }
                        },

                        (Scene::VideoProcessing, _)
                            if video_state == VideoProcessingState::Compress && compress_state.menu.editing =>
                        {
                            match k.code {
                                KeyCode::Enter | KeyCode::Esc => compress_state.menu.editing = false,
                                KeyCode::Tab => {
                                    match compress_state.menu.focus_field() {
                                        VideoCompressField::InputPath => complete_textarea(&mut compress_state.input_file_path),
                                        VideoCompressField::OutputPath => complete_textarea(&mut compress_state.output_file_path),
                                        _ => {},
                                    }
                                },
                                _ => {
                                    match compress_state.menu.focus_field() {
                                        VideoCompressField::InputPath => { compress_state.input_file_path.input(Event::Key(k)); },
                                        VideoCompressField::OutputPath => { compress_state.output_file_path.input(Event::Key(k)); },
                                        _ => {},
                                    }
                                },
                            }
                        },

                        (Scene::VideoProcessing, _)
                            if video_state == VideoProcessingState::Trim && trim_state.menu.editing =>
                        {
                            match k.code {
                                KeyCode::Enter | KeyCode::Esc => {
                                    trim_state.menu.editing = false;
                                    if trim_state.menu.focus_field() == VideoTrimField::InputPath {
                                        trim_state.sync_duration();
                                    }
                                },
                                KeyCode::Tab => {
                                    match trim_state.menu.focus_field() {
                                        VideoTrimField::InputPath => complete_textarea(&mut trim_state.input_file_path),
                                        VideoTrimField::OutputPath => complete_textarea(&mut trim_state.output_file_path),
                                        _ => {},
                                    }
                                },
                                _ => {
                                    match trim_state.menu.focus_field() {
                                        VideoTrimField::InputPath => { trim_state.input_file_path.input(Event::Key(k)); },
                                        VideoTrimField::OutputPath => { trim_state.output_file_path.input(Event::Key(k)); },
                                        VideoTrimField::Start => { trim_state.start_time.input(Event::Key(k)); },
                                        VideoTrimField::End => { trim_state.end_time.input(Event::Key(k)); },
                                        _ => {},
                                    }
                                },
                            }
                        },

                        (Scene::VideoProcessing, _)
                            if video_state == VideoProcessingState::Merge && merge_state.menu.editing =>
                        {
                            match k.code {
                                KeyCode::Enter | KeyCode::Esc => merge_state.menu.editing = false,
                                KeyCode::Tab => {
                                    match merge_state.menu.focus_field() {
                                        VideoMergeField::InputA => complete_textarea(&mut merge_state.input_a_path),
                                        VideoMergeField::InputB => complete_textarea(&mut merge_state.input_b_path),
                                        VideoMergeField::OutputPath => complete_textarea(&mut merge_state.output_file_path),
                                        _ => {},
                                    }
                                },
                                _ => {
                                    match merge_state.menu.focus_field() {
                                        VideoMergeField::InputA => { merge_state.input_a_path.input(Event::Key(k)); },
                                        VideoMergeField::InputB => { merge_state.input_b_path.input(Event::Key(k)); },
                                        VideoMergeField::OutputPath => { merge_state.output_file_path.input(Event::Key(k)); },
                                        _ => {},
                                    }
                                },
                            }
                        },

                        (Scene::VideoProcessing, _)
                            if video_state == VideoProcessingState::Extraction && extraction_state.menu.editing =>
                        {
                            match k.code {
                                KeyCode::Enter | KeyCode::Esc => {
                                    extraction_state.menu.editing = false;
                                    if extraction_state.menu.focus_field() == VideoExtractionField::Input {
                                        extraction_state.sync_duration();
                                    }
                                },
                                KeyCode::Tab => {
                                    match extraction_state.menu.focus_field() {
                                        VideoExtractionField::Input => complete_textarea(&mut extraction_state.input_path),
                                        VideoExtractionField::OutputPath => complete_textarea(&mut extraction_state.output_file_path),
                                        _ => {},
                                    }
                                },
                                _ => {
                                    match extraction_state.menu.focus_field() {
                                        VideoExtractionField::Input => { extraction_state.input_path.input(Event::Key(k)); },
                                        VideoExtractionField::OutputPath => { extraction_state.output_file_path.input(Event::Key(k)); },
                                        VideoExtractionField::Start => { extraction_state.start_time.input(Event::Key(k)); },
                                        VideoExtractionField::End => { extraction_state.end_time.input(Event::Key(k)); },
                                        _ => {},
                                    }
                                },
                            }
                        },

                        //                      <-- AUDIO FIELD TEXT-EDIT KEY EVENTS -->
                        // Same contract as the video blocks above: while a textarea is open,
                        // every key belongs to it, including 'q', or the user would get thrown
                        // to the exit screen halfway through typing a filename.
                        (Scene::AudioProcessing, _)
                            if audio_state == AudioProcessingState::Convert && audio_convert_state.menu.editing =>
                        {
                            match k.code {
                                KeyCode::Enter | KeyCode::Esc => audio_convert_state.menu.editing = false,
                                KeyCode::Tab => {
                                    match audio_convert_state.menu.focus_field() {
                                        AudioConvertField::InputPath => complete_textarea(&mut audio_convert_state.input_file_path),
                                        AudioConvertField::OutputPath => complete_textarea(&mut audio_convert_state.output_file_path),
                                        _ => {},
                                    }
                                },
                                _ => {
                                    match audio_convert_state.menu.focus_field() {
                                        AudioConvertField::InputPath => { audio_convert_state.input_file_path.input(Event::Key(k)); },
                                        AudioConvertField::OutputPath => { audio_convert_state.output_file_path.input(Event::Key(k)); },
                                        _ => {},
                                    }
                                },
                            }
                        },

                        (Scene::AudioProcessing, _)
                            if audio_state == AudioProcessingState::Extract && audio_extract_state.menu.editing =>
                        {
                            match k.code {
                                KeyCode::Enter | KeyCode::Esc => audio_extract_state.menu.editing = false,
                                KeyCode::Tab => {
                                    match audio_extract_state.menu.focus_field() {
                                        AudioExtractField::InputPath => complete_textarea(&mut audio_extract_state.input_file_path),
                                        AudioExtractField::OutputPath => complete_textarea(&mut audio_extract_state.output_file_path),
                                        _ => {},
                                    }
                                },
                                _ => {
                                    match audio_extract_state.menu.focus_field() {
                                        AudioExtractField::InputPath => { audio_extract_state.input_file_path.input(Event::Key(k)); },
                                        AudioExtractField::OutputPath => { audio_extract_state.output_file_path.input(Event::Key(k)); },
                                        _ => {},
                                    }
                                },
                            }
                        },

                        (Scene::AudioProcessing, _)
                            if audio_state == AudioProcessingState::Trim && audio_trim_state.menu.editing =>
                        {
                            match k.code {
                                KeyCode::Enter | KeyCode::Esc => {
                                    audio_trim_state.menu.editing = false;
                                    if audio_trim_state.menu.focus_field() == AudioTrimField::InputPath {
                                        audio_trim_state.sync_duration();
                                    }
                                },
                                KeyCode::Tab => {
                                    match audio_trim_state.menu.focus_field() {
                                        AudioTrimField::InputPath => complete_textarea(&mut audio_trim_state.input_file_path),
                                        AudioTrimField::OutputPath => complete_textarea(&mut audio_trim_state.output_file_path),
                                        _ => {},
                                    }
                                },
                                _ => {
                                    match audio_trim_state.menu.focus_field() {
                                        AudioTrimField::InputPath => { audio_trim_state.input_file_path.input(Event::Key(k)); },
                                        AudioTrimField::OutputPath => { audio_trim_state.output_file_path.input(Event::Key(k)); },
                                        AudioTrimField::Start => { audio_trim_state.start_time.input(Event::Key(k)); },
                                        AudioTrimField::End => { audio_trim_state.end_time.input(Event::Key(k)); },
                                        _ => {},
                                    }
                                },
                            }
                        },

                        (Scene::AudioProcessing, _)
                            if audio_state == AudioProcessingState::Speed && audio_speed_state.menu.editing =>
                        {
                            match k.code {
                                KeyCode::Enter | KeyCode::Esc => audio_speed_state.menu.editing = false,
                                KeyCode::Tab => {
                                    match audio_speed_state.menu.focus_field() {
                                        AudioSpeedField::InputPath => complete_textarea(&mut audio_speed_state.input_file_path),
                                        AudioSpeedField::OutputPath => complete_textarea(&mut audio_speed_state.output_file_path),
                                        _ => {},
                                    }
                                },
                                _ => {
                                    match audio_speed_state.menu.focus_field() {
                                        AudioSpeedField::InputPath => { audio_speed_state.input_file_path.input(Event::Key(k)); },
                                        AudioSpeedField::OutputPath => { audio_speed_state.output_file_path.input(Event::Key(k)); },
                                        _ => {},
                                    }
                                },
                            }
                        },

                        (Scene::AudioProcessing, _)
                            if audio_state == AudioProcessingState::Fade && audio_fade_state.menu.editing =>
                        {
                            match k.code {
                                KeyCode::Enter | KeyCode::Esc => {
                                    audio_fade_state.menu.editing = false;
                                    // The fade-out is measured back from the end of the track, so
                                    // the length has to be read the moment the path is settled.
                                    if audio_fade_state.menu.focus_field() == AudioFadeField::InputPath {
                                        audio_fade_state.sync_duration();
                                    }
                                },
                                KeyCode::Tab => {
                                    match audio_fade_state.menu.focus_field() {
                                        AudioFadeField::InputPath => {
                                            complete_textarea(&mut audio_fade_state.input_file_path);
                                            audio_fade_state.sync_duration();
                                        },
                                        AudioFadeField::OutputPath => complete_textarea(&mut audio_fade_state.output_file_path),
                                        _ => {},
                                    }
                                },
                                _ => {
                                    match audio_fade_state.menu.focus_field() {
                                        AudioFadeField::InputPath => { audio_fade_state.input_file_path.input(Event::Key(k)); },
                                        AudioFadeField::OutputPath => { audio_fade_state.output_file_path.input(Event::Key(k)); },
                                        AudioFadeField::FadeIn => { audio_fade_state.fade_in.input(Event::Key(k)); },
                                        AudioFadeField::FadeOut => { audio_fade_state.fade_out.input(Event::Key(k)); },
                                        _ => {},
                                    }
                                },
                            }
                        },

                        (Scene::AudioProcessing, _)
                            if audio_state == AudioProcessingState::Volume && audio_volume_state.menu.editing =>
                        {
                            match k.code {
                                KeyCode::Enter | KeyCode::Esc => audio_volume_state.menu.editing = false,
                                KeyCode::Tab => {
                                    match audio_volume_state.menu.focus_field() {
                                        AudioVolumeField::InputPath => complete_textarea(&mut audio_volume_state.input_file_path),
                                        AudioVolumeField::OutputPath => complete_textarea(&mut audio_volume_state.output_file_path),
                                        _ => {},
                                    }
                                },
                                _ => {
                                    match audio_volume_state.menu.focus_field() {
                                        AudioVolumeField::InputPath => { audio_volume_state.input_file_path.input(Event::Key(k)); },
                                        AudioVolumeField::OutputPath => { audio_volume_state.output_file_path.input(Event::Key(k)); },
                                        _ => {},
                                    }
                                },
                            }
                        },

                        //                      <-- IMAGE FIELD TEXT-EDIT KEY EVENTS -->
                        // Same contract as the video and audio blocks: while a textarea is open,
                        // every key belongs to it, including 'q'.
                        (Scene::ImageProcessing, _)
                            if image_state == ImageProcessingState::Convert && image_convert_state.menu.editing =>
                        {
                            match k.code {
                                KeyCode::Enter | KeyCode::Esc => image_convert_state.menu.editing = false,
                                KeyCode::Tab => {
                                    match image_convert_state.menu.focus_field() {
                                        ImageConvertField::InputPath => complete_textarea(&mut image_convert_state.input_file_path),
                                        ImageConvertField::OutputPath => complete_textarea(&mut image_convert_state.output_file_path),
                                        _ => {},
                                    }
                                },
                                _ => {
                                    match image_convert_state.menu.focus_field() {
                                        ImageConvertField::InputPath => { image_convert_state.input_file_path.input(Event::Key(k)); },
                                        ImageConvertField::OutputPath => { image_convert_state.output_file_path.input(Event::Key(k)); },
                                        _ => {},
                                    }
                                },
                            }
                        },

                        (Scene::ImageProcessing, _)
                            if image_state == ImageProcessingState::Resize && image_resize_state.menu.editing =>
                        {
                            match k.code {
                                KeyCode::Enter | KeyCode::Esc => image_resize_state.menu.editing = false,
                                KeyCode::Tab => {
                                    match image_resize_state.menu.focus_field() {
                                        ImageResizeField::InputPath => complete_textarea(&mut image_resize_state.input_file_path),
                                        ImageResizeField::OutputPath => complete_textarea(&mut image_resize_state.output_file_path),
                                        _ => {},
                                    }
                                },
                                _ => {
                                    match image_resize_state.menu.focus_field() {
                                        ImageResizeField::InputPath => { image_resize_state.input_file_path.input(Event::Key(k)); },
                                        ImageResizeField::OutputPath => { image_resize_state.output_file_path.input(Event::Key(k)); },
                                        ImageResizeField::Width => { image_resize_state.width.input(Event::Key(k)); },
                                        ImageResizeField::Height => { image_resize_state.height.input(Event::Key(k)); },
                                        _ => {},
                                    }
                                },
                            }
                        },

                        (Scene::ImageProcessing, _)
                            if image_state == ImageProcessingState::Compress && image_compress_state.menu.editing =>
                        {
                            match k.code {
                                KeyCode::Enter | KeyCode::Esc => image_compress_state.menu.editing = false,
                                KeyCode::Tab => {
                                    match image_compress_state.menu.focus_field() {
                                        ImageCompressField::InputPath => complete_textarea(&mut image_compress_state.input_file_path),
                                        ImageCompressField::OutputPath => complete_textarea(&mut image_compress_state.output_file_path),
                                        _ => {},
                                    }
                                },
                                _ => {
                                    match image_compress_state.menu.focus_field() {
                                        ImageCompressField::InputPath => { image_compress_state.input_file_path.input(Event::Key(k)); },
                                        ImageCompressField::OutputPath => { image_compress_state.output_file_path.input(Event::Key(k)); },
                                        _ => {},
                                    }
                                },
                            }
                        },

                        (Scene::ImageProcessing, _)
                            if image_state == ImageProcessingState::Crop && image_crop_state.menu.editing =>
                        {
                            match k.code {
                                KeyCode::Enter | KeyCode::Esc => image_crop_state.menu.editing = false,
                                KeyCode::Tab => {
                                    match image_crop_state.menu.focus_field() {
                                        ImageCropField::InputPath => complete_textarea(&mut image_crop_state.input_file_path),
                                        ImageCropField::OutputPath => complete_textarea(&mut image_crop_state.output_file_path),
                                        _ => {},
                                    }
                                },
                                _ => {
                                    match image_crop_state.menu.focus_field() {
                                        ImageCropField::InputPath => { image_crop_state.input_file_path.input(Event::Key(k)); },
                                        ImageCropField::OutputPath => { image_crop_state.output_file_path.input(Event::Key(k)); },
                                        ImageCropField::X => { image_crop_state.x.input(Event::Key(k)); },
                                        ImageCropField::Y => { image_crop_state.y.input(Event::Key(k)); },
                                        ImageCropField::Width => { image_crop_state.width.input(Event::Key(k)); },
                                        ImageCropField::Height => { image_crop_state.height.input(Event::Key(k)); },
                                        _ => {},
                                    }
                                },
                            }
                        },

                        (Scene::ImageProcessing, _)
                            if image_state == ImageProcessingState::Rotate && image_rotate_state.menu.editing =>
                        {
                            match k.code {
                                KeyCode::Enter | KeyCode::Esc => image_rotate_state.menu.editing = false,
                                KeyCode::Tab => {
                                    match image_rotate_state.menu.focus_field() {
                                        ImageRotateField::InputPath => complete_textarea(&mut image_rotate_state.input_file_path),
                                        ImageRotateField::OutputPath => complete_textarea(&mut image_rotate_state.output_file_path),
                                        _ => {},
                                    }
                                },
                                _ => {
                                    match image_rotate_state.menu.focus_field() {
                                        ImageRotateField::InputPath => { image_rotate_state.input_file_path.input(Event::Key(k)); },
                                        ImageRotateField::OutputPath => { image_rotate_state.output_file_path.input(Event::Key(k)); },
                                        _ => {},
                                    }
                                },
                            }
                        },

                        (Scene::ImageProcessing, _)
                            if image_state == ImageProcessingState::Filter && image_filter_state.menu.editing =>
                        {
                            match k.code {
                                KeyCode::Enter | KeyCode::Esc => image_filter_state.menu.editing = false,
                                KeyCode::Tab => {
                                    match image_filter_state.menu.focus_field() {
                                        ImageFilterField::InputPath => complete_textarea(&mut image_filter_state.input_file_path),
                                        ImageFilterField::OutputPath => complete_textarea(&mut image_filter_state.output_file_path),
                                        _ => {},
                                    }
                                },
                                _ => {
                                    match image_filter_state.menu.focus_field() {
                                        ImageFilterField::InputPath => { image_filter_state.input_file_path.input(Event::Key(k)); },
                                        ImageFilterField::OutputPath => { image_filter_state.output_file_path.input(Event::Key(k)); },
                                        _ => {},
                                    }
                                },
                            }
                        },

                        //                      <-- GLOBAL EXIT KEY EVENTS -->
                        (scene, KeyCode::Char('q')) if scene != Scene::Exit => {
                            previous_scene = scene;
                            current_scene = Scene::Exit;

                            writeln!(_file, "State: '{:?}' at {} because '{:?}' was pressed.",
                                current_scene, Utc::now().format("%H-%M-%S"), k.code)?;
                        },
                        (Scene::Exit, KeyCode::Char('q')) => {
                            writeln!(_file, "State: '{:?}' at {} because '{:?}' was pressed.",
                                current_scene, Utc::now().format("%H-%M-%S"), k.code)?;

                            // Stop the application.
                            break;
                        },
                        (Scene::Exit, KeyCode::Esc) => {
                            current_scene = previous_scene;
                            writeln!(_file, "State: '{:?}' at {} because '{:?}' was pressed.",
                                current_scene, Utc::now().format("%H-%M-%S"), k.code)?;
                        },

                        //                      <-- START SCENE KEY EVENTS -->
                        // If the app has just started, entering advances us to the home menu.
                        (Scene::Start, KeyCode::Enter) => {
                            current_scene = Scene::Home;
                            writeln!(_file, "State: '{:?}' at {} because '{:?}' was pressed.",
                                current_scene, Utc::now().format("%H-%M-%S"), k.code)?;
                        },

                        //                      <-- HOME SCENE KEY EVENTS -->
                        (Scene::Home, KeyCode::Esc) => {
                            current_scene = Scene::Start;
                            writeln!(_file, "State: '{:?}' at {} because '{:?}' was pressed.",
                                current_scene, Utc::now().format("%H-%M-%S"), k.code)?;
                        },
                        (Scene::Home, KeyCode::Left) | (Scene::Home, KeyCode::Char('a')) => home_menu.previous(),
                        (Scene::Home, KeyCode::Right) | (Scene::Home, KeyCode::Char('d')) => home_menu.next(),
                        (Scene::Home, KeyCode::Enter) => {
                            home_state = home_menu.selected_state();

                                                        current_scene = match home_state {
                                HomeState::Image => Scene::ImageProcessing,
                                HomeState::Video => Scene::VideoProcessing,
                                HomeState::Audio => Scene::AudioProcessing,
                                HomeState::Default => Scene::Home,
                            };

                            writeln!(_file, "State: '{:?}' at {} because '{:?}' was pressed.",
                                current_scene, Utc::now().format("%H-%M-%S"), k.code)?;
                        },

                        //                      <-- AUDIO CONVERT FIELD NAVIGATION KEY EVENTS -->
                        // Mirrors the video Convert arms below: A/D walk focus across the row,
                        // W/S cycle the focused field's value (or step off a path field, which
                        // has nothing to cycle), ENTER opens a path or runs ffmpeg.
                        (Scene::AudioProcessing, KeyCode::Left) | (Scene::AudioProcessing, KeyCode::Char('a'))
                            if audio_state == AudioProcessingState::Convert => audio_convert_state.menu.previous(),
                        (Scene::AudioProcessing, KeyCode::Right) | (Scene::AudioProcessing, KeyCode::Char('d'))
                            if audio_state == AudioProcessingState::Convert => audio_convert_state.menu.next(),
                        (Scene::AudioProcessing, KeyCode::Up) | (Scene::AudioProcessing, KeyCode::Char('w'))
                            if audio_state == AudioProcessingState::Convert => {
                                match audio_convert_state.menu.focus_field() {
                                    AudioConvertField::InputPath | AudioConvertField::OutputPath => audio_convert_state.menu.previous(),
                                    _ => audio_convert_state.cycle_value(true),
                                }
                            },
                        (Scene::AudioProcessing, KeyCode::Down) | (Scene::AudioProcessing, KeyCode::Char('s'))
                            if audio_state == AudioProcessingState::Convert => {
                                match audio_convert_state.menu.focus_field() {
                                    AudioConvertField::InputPath | AudioConvertField::OutputPath => audio_convert_state.menu.next(),
                                    _ => audio_convert_state.cycle_value(false),
                                }
                            },
                        (Scene::AudioProcessing, KeyCode::Enter) if audio_state == AudioProcessingState::Convert => {
                            match audio_convert_state.menu.focus_field() {
                                AudioConvertField::InputPath | AudioConvertField::OutputPath => audio_convert_state.menu.editing = true,
                                AudioConvertField::Run => audio_convert_state.start_convert(&filename),
                                _ => {},
                            }
                        },

                        //                      <-- AUDIO EXTRACT FIELD NAVIGATION KEY EVENTS -->
                        (Scene::AudioProcessing, KeyCode::Left) | (Scene::AudioProcessing, KeyCode::Char('a'))
                            if audio_state == AudioProcessingState::Extract => audio_extract_state.menu.previous(),
                        (Scene::AudioProcessing, KeyCode::Right) | (Scene::AudioProcessing, KeyCode::Char('d'))
                            if audio_state == AudioProcessingState::Extract => audio_extract_state.menu.next(),
                        (Scene::AudioProcessing, KeyCode::Up) | (Scene::AudioProcessing, KeyCode::Char('w'))
                            if audio_state == AudioProcessingState::Extract => {
                                match audio_extract_state.menu.focus_field() {
                                    AudioExtractField::InputPath | AudioExtractField::OutputPath => audio_extract_state.menu.previous(),
                                    _ => audio_extract_state.cycle_value(true),
                                }
                            },
                        (Scene::AudioProcessing, KeyCode::Down) | (Scene::AudioProcessing, KeyCode::Char('s'))
                            if audio_state == AudioProcessingState::Extract => {
                                match audio_extract_state.menu.focus_field() {
                                    AudioExtractField::InputPath | AudioExtractField::OutputPath => audio_extract_state.menu.next(),
                                    _ => audio_extract_state.cycle_value(false),
                                }
                            },
                        (Scene::AudioProcessing, KeyCode::Enter) if audio_state == AudioProcessingState::Extract => {
                            match audio_extract_state.menu.focus_field() {
                                AudioExtractField::InputPath | AudioExtractField::OutputPath => audio_extract_state.menu.editing = true,
                                AudioExtractField::Run => audio_extract_state.start_extraction(&filename),
                                _ => {},
                            }
                        },

                        //                      <-- AUDIO TRIM FIELD NAVIGATION KEY EVENTS -->
                        // Every non-Run field here is typed rather than cycled, so W/S just
                        // walk the row the same way A/D do, exactly like video Trim.
                        (Scene::AudioProcessing, KeyCode::Left) | (Scene::AudioProcessing, KeyCode::Char('a'))
                            if audio_state == AudioProcessingState::Trim => audio_trim_state.menu.previous(),
                        (Scene::AudioProcessing, KeyCode::Right) | (Scene::AudioProcessing, KeyCode::Char('d'))
                            if audio_state == AudioProcessingState::Trim => audio_trim_state.menu.next(),
                        (Scene::AudioProcessing, KeyCode::Up) | (Scene::AudioProcessing, KeyCode::Char('w'))
                            if audio_state == AudioProcessingState::Trim => audio_trim_state.menu.previous(),
                        (Scene::AudioProcessing, KeyCode::Down) | (Scene::AudioProcessing, KeyCode::Char('s'))
                            if audio_state == AudioProcessingState::Trim => audio_trim_state.menu.next(),
                        (Scene::AudioProcessing, KeyCode::Enter) if audio_state == AudioProcessingState::Trim => {
                            match audio_trim_state.menu.focus_field() {
                                AudioTrimField::Run => audio_trim_state.start_trim(&filename),
                                _ => audio_trim_state.menu.editing = true,
                            }
                        },

                        //                      <-- AUDIO SPEED FIELD NAVIGATION KEY EVENTS -->
                        (Scene::AudioProcessing, KeyCode::Left) | (Scene::AudioProcessing, KeyCode::Char('a'))
                            if audio_state == AudioProcessingState::Speed => audio_speed_state.menu.previous(),
                        (Scene::AudioProcessing, KeyCode::Right) | (Scene::AudioProcessing, KeyCode::Char('d'))
                            if audio_state == AudioProcessingState::Speed => audio_speed_state.menu.next(),
                        (Scene::AudioProcessing, KeyCode::Up) | (Scene::AudioProcessing, KeyCode::Char('w'))
                            if audio_state == AudioProcessingState::Speed => {
                                match audio_speed_state.menu.focus_field() {
                                    AudioSpeedField::InputPath | AudioSpeedField::OutputPath => audio_speed_state.menu.previous(),
                                    _ => audio_speed_state.cycle_value(true),
                                }
                            },
                        (Scene::AudioProcessing, KeyCode::Down) | (Scene::AudioProcessing, KeyCode::Char('s'))
                            if audio_state == AudioProcessingState::Speed => {
                                match audio_speed_state.menu.focus_field() {
                                    AudioSpeedField::InputPath | AudioSpeedField::OutputPath => audio_speed_state.menu.next(),
                                    _ => audio_speed_state.cycle_value(false),
                                }
                            },
                        (Scene::AudioProcessing, KeyCode::Enter) if audio_state == AudioProcessingState::Speed => {
                            match audio_speed_state.menu.focus_field() {
                                AudioSpeedField::InputPath | AudioSpeedField::OutputPath => audio_speed_state.menu.editing = true,
                                AudioSpeedField::Run => audio_speed_state.start_respeed(&filename),
                                _ => {},
                            }
                        },

                        //                      <-- AUDIO FADE FIELD NAVIGATION KEY EVENTS -->
                        (Scene::AudioProcessing, KeyCode::Left) | (Scene::AudioProcessing, KeyCode::Char('a'))
                            if audio_state == AudioProcessingState::Fade => audio_fade_state.menu.previous(),
                        (Scene::AudioProcessing, KeyCode::Right) | (Scene::AudioProcessing, KeyCode::Char('d'))
                            if audio_state == AudioProcessingState::Fade => audio_fade_state.menu.next(),
                        (Scene::AudioProcessing, KeyCode::Up) | (Scene::AudioProcessing, KeyCode::Char('w'))
                            if audio_state == AudioProcessingState::Fade => {
                                match audio_fade_state.menu.focus_field() {
                                    AudioFadeField::Format => audio_fade_state.cycle_value(true),
                                    _ => audio_fade_state.menu.previous(),
                                }
                            },
                        (Scene::AudioProcessing, KeyCode::Down) | (Scene::AudioProcessing, KeyCode::Char('s'))
                            if audio_state == AudioProcessingState::Fade => {
                                match audio_fade_state.menu.focus_field() {
                                    AudioFadeField::Format => audio_fade_state.cycle_value(false),
                                    _ => audio_fade_state.menu.next(),
                                }
                            },
                        (Scene::AudioProcessing, KeyCode::Enter) if audio_state == AudioProcessingState::Fade => {
                            match audio_fade_state.menu.focus_field() {
                                AudioFadeField::Run => audio_fade_state.start_fade(&filename),
                                AudioFadeField::Format => {},
                                _ => audio_fade_state.menu.editing = true,
                            }
                        },

                        //                      <-- AUDIO VOLUME FIELD NAVIGATION KEY EVENTS -->
                        (Scene::AudioProcessing, KeyCode::Left) | (Scene::AudioProcessing, KeyCode::Char('a'))
                            if audio_state == AudioProcessingState::Volume => audio_volume_state.menu.previous(),
                        (Scene::AudioProcessing, KeyCode::Right) | (Scene::AudioProcessing, KeyCode::Char('d'))
                            if audio_state == AudioProcessingState::Volume => audio_volume_state.menu.next(),
                        (Scene::AudioProcessing, KeyCode::Up) | (Scene::AudioProcessing, KeyCode::Char('w'))
                            if audio_state == AudioProcessingState::Volume => {
                                match audio_volume_state.menu.focus_field() {
                                    AudioVolumeField::InputPath | AudioVolumeField::OutputPath => audio_volume_state.menu.previous(),
                                    _ => audio_volume_state.cycle_value(true),
                                }
                            },
                        (Scene::AudioProcessing, KeyCode::Down) | (Scene::AudioProcessing, KeyCode::Char('s'))
                            if audio_state == AudioProcessingState::Volume => {
                                match audio_volume_state.menu.focus_field() {
                                    AudioVolumeField::InputPath | AudioVolumeField::OutputPath => audio_volume_state.menu.next(),
                                    _ => audio_volume_state.cycle_value(false),
                                }
                            },
                        (Scene::AudioProcessing, KeyCode::Enter) if audio_state == AudioProcessingState::Volume => {
                            match audio_volume_state.menu.focus_field() {
                                AudioVolumeField::InputPath | AudioVolumeField::OutputPath => audio_volume_state.menu.editing = true,
                                AudioVolumeField::Run => audio_volume_state.start_volume(&filename),
                                _ => {},
                            }
                        },

                        //                      <-- AUDIO PROCESSING SCENE KEY EVENTS -->
                        // The catch-all arms: these only get reached while no individual audio
                        // screen is open, i.e. while the user is still picking one off the tabs.
                        (Scene::AudioProcessing, KeyCode::Left) | (Scene::AudioProcessing, KeyCode::Char('a')) => audio_menu.previous(),
                        (Scene::AudioProcessing, KeyCode::Right) | (Scene::AudioProcessing, KeyCode::Char('d')) => audio_menu.next(),
                        (Scene::AudioProcessing, KeyCode::Up) | (Scene::AudioProcessing, KeyCode::Char('w')) => audio_menu.above(),
                        (Scene::AudioProcessing, KeyCode::Down) | (Scene::AudioProcessing, KeyCode::Char('s')) => audio_menu.below(),
                        (Scene::AudioProcessing, KeyCode::Enter) => {
                            audio_state = audio_menu.selected_state();
                            previous_scene = Scene::AudioProcessing;

                            writeln!(_file, "State: '{:?}' at {} showing {:?} because '{:?}' was pressed.",
                                current_scene, Utc::now().format("%H-%M-%S"), audio_menu, k.code)?;
                        },

                        (Scene::AudioProcessing, KeyCode::Esc) => {
                            // First ESC backs out of whichever audio screen is open, a second
                            // one leaves the scene entirely, same as Video.
                            if audio_state != AudioProcessingState::Default {
                                audio_state = AudioProcessingState::Default;
                            } else {
                                current_scene = Scene::Home;
                            }

                            writeln!(_file, "State: '{:?}' at {} showing {:?} because '{:?}' was pressed.",
                                current_scene, Utc::now().format("%H-%M-%S"), audio_menu, k.code)?;
                        },

                        //                      <-- CONVERT FIELD NAVIGATION KEY EVENTS -->
                        // A/D walk focus across the Convert row's fields, W/S cycle the
                        // focused field's value, ENTER opens a path for editing or runs ffmpeg.
                        (Scene::VideoProcessing, KeyCode::Left) | (Scene::VideoProcessing, KeyCode::Char('a'))
                            if video_state == VideoProcessingState::Convert => convert_state.menu.previous(),
                        (Scene::VideoProcessing, KeyCode::Right) | (Scene::VideoProcessing, KeyCode::Char('d'))
                            if video_state == VideoProcessingState::Convert => convert_state.menu.next(),
                        (Scene::VideoProcessing, KeyCode::Up) | (Scene::VideoProcessing, KeyCode::Char('w'))
                            if video_state == VideoProcessingState::Convert => {
                                match convert_state.menu.focus_field() {
                                    VideoConvertField::InputPath | VideoConvertField::OutputPath => convert_state.menu.previous(),
                                    _ => convert_state.cycle_value(true),
                                }
                            },
                        (Scene::VideoProcessing, KeyCode::Down) | (Scene::VideoProcessing, KeyCode::Char('s'))
                            if video_state == VideoProcessingState::Convert => {
                                match convert_state.menu.focus_field() {
                                    VideoConvertField::InputPath | VideoConvertField::OutputPath => convert_state.menu.next(),
                                    _ => convert_state.cycle_value(false),
                                }
                            },
                        (Scene::VideoProcessing, KeyCode::Enter) if video_state == VideoProcessingState::Convert => {
                            match convert_state.menu.focus_field() {
                                VideoConvertField::InputPath | VideoConvertField::OutputPath => convert_state.menu.editing = true,
                                VideoConvertField::Run => convert_state.start_convert(&filename),
                                _ => {},
                            }
                        },

                        //                      <-- COMPRESS FIELD NAVIGATION KEY EVENTS -->
                        (Scene::VideoProcessing, KeyCode::Left) | (Scene::VideoProcessing, KeyCode::Char('a'))
                            if video_state == VideoProcessingState::Compress => compress_state.menu.previous(),
                        (Scene::VideoProcessing, KeyCode::Right) | (Scene::VideoProcessing, KeyCode::Char('d'))
                            if video_state == VideoProcessingState::Compress => compress_state.menu.next(),
                        (Scene::VideoProcessing, KeyCode::Up) | (Scene::VideoProcessing, KeyCode::Char('w'))
                            if video_state == VideoProcessingState::Compress => {
                                match compress_state.menu.focus_field() {
                                    VideoCompressField::InputPath | VideoCompressField::OutputPath => compress_state.menu.previous(),
                                    _ => compress_state.cycle_value(true),
                                }
                            },
                        (Scene::VideoProcessing, KeyCode::Down) | (Scene::VideoProcessing, KeyCode::Char('s'))
                            if video_state == VideoProcessingState::Compress => {
                                match compress_state.menu.focus_field() {
                                    VideoCompressField::InputPath | VideoCompressField::OutputPath => compress_state.menu.next(),
                                    _ => compress_state.cycle_value(false),
                                }
                            },
                        (Scene::VideoProcessing, KeyCode::Enter) if video_state == VideoProcessingState::Compress => {
                            match compress_state.menu.focus_field() {
                                VideoCompressField::InputPath | VideoCompressField::OutputPath => compress_state.menu.editing = true,
                                VideoCompressField::Run => compress_state.start_compress(&filename),
                                _ => {},
                            }
                        },

                        //                      <-- TRIM FIELD NAVIGATION KEY EVENTS -->
                        (Scene::VideoProcessing, KeyCode::Left) | (Scene::VideoProcessing, KeyCode::Char('a'))
                            if video_state == VideoProcessingState::Trim => trim_state.menu.previous(),
                        (Scene::VideoProcessing, KeyCode::Right) | (Scene::VideoProcessing, KeyCode::Char('d'))
                            if video_state == VideoProcessingState::Trim => trim_state.menu.next(),
                        (Scene::VideoProcessing, KeyCode::Up) | (Scene::VideoProcessing, KeyCode::Char('w'))
                            if video_state == VideoProcessingState::Trim => trim_state.menu.previous(),
                        (Scene::VideoProcessing, KeyCode::Down) | (Scene::VideoProcessing, KeyCode::Char('s'))
                            if video_state == VideoProcessingState::Trim => trim_state.menu.next(),
                        (Scene::VideoProcessing, KeyCode::Enter) if video_state == VideoProcessingState::Trim => {
                            match trim_state.menu.focus_field() {
                                VideoTrimField::Run => trim_state.start_trim(&filename),
                                _ => trim_state.menu.editing = true,
                            }
                        },

                        //                      <-- MERGE FIELD NAVIGATION KEY EVENTS -->
                        (Scene::VideoProcessing, KeyCode::Left) | (Scene::VideoProcessing, KeyCode::Char('a'))
                            if video_state == VideoProcessingState::Merge => merge_state.menu.previous(),
                        (Scene::VideoProcessing, KeyCode::Right) | (Scene::VideoProcessing, KeyCode::Char('d'))
                            if video_state == VideoProcessingState::Merge => merge_state.menu.next(),
                        (Scene::VideoProcessing, KeyCode::Up) | (Scene::VideoProcessing, KeyCode::Char('w'))
                            if video_state == VideoProcessingState::Merge => merge_state.menu.previous(),
                        (Scene::VideoProcessing, KeyCode::Down) | (Scene::VideoProcessing, KeyCode::Char('s'))
                            if video_state == VideoProcessingState::Merge => merge_state.menu.next(),
                        (Scene::VideoProcessing, KeyCode::Enter) if video_state == VideoProcessingState::Merge => {
                            match merge_state.menu.focus_field() {
                                VideoMergeField::Run => merge_state.start_merge(&filename),
                                _ => merge_state.menu.editing = true,
                            }
                        },

                        //                      <-- EXTRACTION FIELD NAVIGATION KEY EVENTS -->
                        (Scene::VideoProcessing, KeyCode::Left) | (Scene::VideoProcessing, KeyCode::Char('a'))
                            if video_state == VideoProcessingState::Extraction => extraction_state.menu.previous(),
                        (Scene::VideoProcessing, KeyCode::Right) | (Scene::VideoProcessing, KeyCode::Char('d'))
                            if video_state == VideoProcessingState::Extraction => extraction_state.menu.next(),
                        (Scene::VideoProcessing, KeyCode::Up) | (Scene::VideoProcessing, KeyCode::Char('w'))
                            if video_state == VideoProcessingState::Extraction => {
                                match extraction_state.menu.focus_field() {
                                    VideoExtractionField::Input | VideoExtractionField::OutputPath
                                        | VideoExtractionField::Start | VideoExtractionField::End => extraction_state.menu.previous(),
                                    _ => extraction_state.cycle_value(true),
                                }
                            },
                        (Scene::VideoProcessing, KeyCode::Down) | (Scene::VideoProcessing, KeyCode::Char('s'))
                            if video_state == VideoProcessingState::Extraction => {
                                match extraction_state.menu.focus_field() {
                                    VideoExtractionField::Input | VideoExtractionField::OutputPath
                                        | VideoExtractionField::Start | VideoExtractionField::End => extraction_state.menu.next(),
                                    _ => extraction_state.cycle_value(false),
                                }
                            },
                        (Scene::VideoProcessing, KeyCode::Enter) if video_state == VideoProcessingState::Extraction => {
                            match extraction_state.menu.focus_field() {
                                VideoExtractionField::Input | VideoExtractionField::OutputPath => extraction_state.menu.editing = true,
                                VideoExtractionField::Run => extraction_state.start_extraction(&filename),
                                _ => {},
                            }
                        },

                        //                      <-- RESIZE FIELD NAVIGATION KEY EVENTS -->
                        (Scene::VideoProcessing, KeyCode::Left) | (Scene::VideoProcessing, KeyCode::Char('a'))
                            if video_state == VideoProcessingState::Resize => resize_state.menu.previous(),
                        (Scene::VideoProcessing, KeyCode::Right) | (Scene::VideoProcessing, KeyCode::Char('d'))
                            if video_state == VideoProcessingState::Resize => resize_state.menu.next(),
                        (Scene::VideoProcessing, KeyCode::Up) | (Scene::VideoProcessing, KeyCode::Char('w'))
                            if video_state == VideoProcessingState::Resize => resize_state.menu.previous(),
                        (Scene::VideoProcessing, KeyCode::Down) | (Scene::VideoProcessing, KeyCode::Char('s'))
                            if video_state == VideoProcessingState::Resize => resize_state.menu.next(),
                        (Scene::VideoProcessing, KeyCode::Enter) if video_state == VideoProcessingState::Resize => {
                            match resize_state.menu.focus_field() {
                                VideoResizeField::Run => resize_state.start_resize(&filename),
                                VideoResizeField::Lock => resize_state.toggle_lock(),
                                _ => resize_state.menu.editing = true,
                            }
                        },

                        //                      <-- VIDEO PROCESSING SCENE KEY EVENTS -->
                        (Scene::VideoProcessing, KeyCode::Left) | (Scene::VideoProcessing, KeyCode::Char('a')) => video_menu.previous(),
                        (Scene::VideoProcessing, KeyCode::Right) | (Scene::VideoProcessing, KeyCode::Char('d')) => video_menu.next(),
                        (Scene::VideoProcessing, KeyCode::Up) | (Scene::VideoProcessing, KeyCode::Char('w')) => video_menu.above(),
                        (Scene::VideoProcessing, KeyCode::Down) | (Scene::VideoProcessing, KeyCode::Char('s')) => video_menu.below(),
                        (Scene::VideoProcessing, KeyCode::Enter) => {
                            video_state = video_menu.selected_state();
                            previous_scene = Scene::VideoProcessing;

                            writeln!(_file, "State: '{:?}' at {} showing {:?} because '{:?}' was pressed.",
                                current_scene, Utc::now().format("%H-%M-%S"), video_menu, k.code)?;
                        },

                        (Scene::VideoProcessing, KeyCode::Esc) => {
                            if video_state != VideoProcessingState::Default {
                                video_state = VideoProcessingState::Default;
                                current_scene = previous_scene;
                            } else {
                                current_scene = Scene::Home;
                            }
                            
                            writeln!(_file, "State: '{:?}' at {} showing {:?} because '{:?}' was pressed.",
                                current_scene, Utc::now().format("%H-%M-%S"), video_menu, k.code)?;
                        },

                        //                      <-- IMAGE FIELD NAVIGATION KEY EVENTS -->
                        // Mirrors the audio arms: A/D walk focus across the row, W/S cycle the
                        // focused field's value (or just walk the row where every field is typed),
                        // ENTER opens a field for editing or runs ffmpeg.
                        (Scene::ImageProcessing, KeyCode::Left) | (Scene::ImageProcessing, KeyCode::Char('a'))
                            if image_state == ImageProcessingState::Convert => image_convert_state.menu.previous(),
                        (Scene::ImageProcessing, KeyCode::Right) | (Scene::ImageProcessing, KeyCode::Char('d'))
                            if image_state == ImageProcessingState::Convert => image_convert_state.menu.next(),
                        (Scene::ImageProcessing, KeyCode::Up) | (Scene::ImageProcessing, KeyCode::Char('w'))
                            if image_state == ImageProcessingState::Convert => {
                                match image_convert_state.menu.focus_field() {
                                    ImageConvertField::InputPath | ImageConvertField::OutputPath => image_convert_state.menu.previous(),
                                    _ => image_convert_state.cycle_value(true),
                                }
                            },
                        (Scene::ImageProcessing, KeyCode::Down) | (Scene::ImageProcessing, KeyCode::Char('s'))
                            if image_state == ImageProcessingState::Convert => {
                                match image_convert_state.menu.focus_field() {
                                    ImageConvertField::InputPath | ImageConvertField::OutputPath => image_convert_state.menu.next(),
                                    _ => image_convert_state.cycle_value(false),
                                }
                            },
                        (Scene::ImageProcessing, KeyCode::Enter) if image_state == ImageProcessingState::Convert => {
                            match image_convert_state.menu.focus_field() {
                                ImageConvertField::InputPath | ImageConvertField::OutputPath => image_convert_state.menu.editing = true,
                                ImageConvertField::Run => image_convert_state.start_convert(&filename),
                                _ => {},
                            }
                        },

                        (Scene::ImageProcessing, KeyCode::Left) | (Scene::ImageProcessing, KeyCode::Char('a'))
                            if image_state == ImageProcessingState::Resize => image_resize_state.menu.previous(),
                        (Scene::ImageProcessing, KeyCode::Right) | (Scene::ImageProcessing, KeyCode::Char('d'))
                            if image_state == ImageProcessingState::Resize => image_resize_state.menu.next(),
                        (Scene::ImageProcessing, KeyCode::Up) | (Scene::ImageProcessing, KeyCode::Char('w'))
                            if image_state == ImageProcessingState::Resize => image_resize_state.menu.previous(),
                        (Scene::ImageProcessing, KeyCode::Down) | (Scene::ImageProcessing, KeyCode::Char('s'))
                            if image_state == ImageProcessingState::Resize => image_resize_state.menu.next(),
                        (Scene::ImageProcessing, KeyCode::Enter) if image_state == ImageProcessingState::Resize => {
                            match image_resize_state.menu.focus_field() {
                                ImageResizeField::Run => image_resize_state.start_resize(&filename),
                                _ => image_resize_state.menu.editing = true,
                            }
                        },

                        (Scene::ImageProcessing, KeyCode::Left) | (Scene::ImageProcessing, KeyCode::Char('a'))
                            if image_state == ImageProcessingState::Compress => image_compress_state.menu.previous(),
                        (Scene::ImageProcessing, KeyCode::Right) | (Scene::ImageProcessing, KeyCode::Char('d'))
                            if image_state == ImageProcessingState::Compress => image_compress_state.menu.next(),
                        (Scene::ImageProcessing, KeyCode::Up) | (Scene::ImageProcessing, KeyCode::Char('w'))
                            if image_state == ImageProcessingState::Compress => {
                                match image_compress_state.menu.focus_field() {
                                    ImageCompressField::InputPath | ImageCompressField::OutputPath => image_compress_state.menu.previous(),
                                    _ => image_compress_state.cycle_value(true),
                                }
                            },
                        (Scene::ImageProcessing, KeyCode::Down) | (Scene::ImageProcessing, KeyCode::Char('s'))
                            if image_state == ImageProcessingState::Compress => {
                                match image_compress_state.menu.focus_field() {
                                    ImageCompressField::InputPath | ImageCompressField::OutputPath => image_compress_state.menu.next(),
                                    _ => image_compress_state.cycle_value(false),
                                }
                            },
                        (Scene::ImageProcessing, KeyCode::Enter) if image_state == ImageProcessingState::Compress => {
                            match image_compress_state.menu.focus_field() {
                                ImageCompressField::InputPath | ImageCompressField::OutputPath => image_compress_state.menu.editing = true,
                                ImageCompressField::Run => image_compress_state.start_compress(&filename),
                                _ => {},
                            }
                        },

                        (Scene::ImageProcessing, KeyCode::Left) | (Scene::ImageProcessing, KeyCode::Char('a'))
                            if image_state == ImageProcessingState::Crop => image_crop_state.menu.previous(),
                        (Scene::ImageProcessing, KeyCode::Right) | (Scene::ImageProcessing, KeyCode::Char('d'))
                            if image_state == ImageProcessingState::Crop => image_crop_state.menu.next(),
                        (Scene::ImageProcessing, KeyCode::Up) | (Scene::ImageProcessing, KeyCode::Char('w'))
                            if image_state == ImageProcessingState::Crop => image_crop_state.menu.previous(),
                        (Scene::ImageProcessing, KeyCode::Down) | (Scene::ImageProcessing, KeyCode::Char('s'))
                            if image_state == ImageProcessingState::Crop => image_crop_state.menu.next(),
                        (Scene::ImageProcessing, KeyCode::Enter) if image_state == ImageProcessingState::Crop => {
                            match image_crop_state.menu.focus_field() {
                                ImageCropField::Run => image_crop_state.start_crop(&filename),
                                _ => image_crop_state.menu.editing = true,
                            }
                        },

                        (Scene::ImageProcessing, KeyCode::Left) | (Scene::ImageProcessing, KeyCode::Char('a'))
                            if image_state == ImageProcessingState::Rotate => image_rotate_state.menu.previous(),
                        (Scene::ImageProcessing, KeyCode::Right) | (Scene::ImageProcessing, KeyCode::Char('d'))
                            if image_state == ImageProcessingState::Rotate => image_rotate_state.menu.next(),
                        (Scene::ImageProcessing, KeyCode::Up) | (Scene::ImageProcessing, KeyCode::Char('w'))
                            if image_state == ImageProcessingState::Rotate => {
                                match image_rotate_state.menu.focus_field() {
                                    ImageRotateField::InputPath | ImageRotateField::OutputPath => image_rotate_state.menu.previous(),
                                    _ => image_rotate_state.cycle_value(true),
                                }
                            },
                        (Scene::ImageProcessing, KeyCode::Down) | (Scene::ImageProcessing, KeyCode::Char('s'))
                            if image_state == ImageProcessingState::Rotate => {
                                match image_rotate_state.menu.focus_field() {
                                    ImageRotateField::InputPath | ImageRotateField::OutputPath => image_rotate_state.menu.next(),
                                    _ => image_rotate_state.cycle_value(false),
                                }
                            },
                        (Scene::ImageProcessing, KeyCode::Enter) if image_state == ImageProcessingState::Rotate => {
                            match image_rotate_state.menu.focus_field() {
                                ImageRotateField::InputPath | ImageRotateField::OutputPath => image_rotate_state.menu.editing = true,
                                ImageRotateField::Run => image_rotate_state.start_rotate(&filename),
                                _ => {},
                            }
                        },

                        (Scene::ImageProcessing, KeyCode::Left) | (Scene::ImageProcessing, KeyCode::Char('a'))
                            if image_state == ImageProcessingState::Filter => image_filter_state.menu.previous(),
                        (Scene::ImageProcessing, KeyCode::Right) | (Scene::ImageProcessing, KeyCode::Char('d'))
                            if image_state == ImageProcessingState::Filter => image_filter_state.menu.next(),
                        (Scene::ImageProcessing, KeyCode::Up) | (Scene::ImageProcessing, KeyCode::Char('w'))
                            if image_state == ImageProcessingState::Filter => {
                                match image_filter_state.menu.focus_field() {
                                    ImageFilterField::InputPath | ImageFilterField::OutputPath => image_filter_state.menu.previous(),
                                    _ => image_filter_state.cycle_value(true),
                                }
                            },
                        (Scene::ImageProcessing, KeyCode::Down) | (Scene::ImageProcessing, KeyCode::Char('s'))
                            if image_state == ImageProcessingState::Filter => {
                                match image_filter_state.menu.focus_field() {
                                    ImageFilterField::InputPath | ImageFilterField::OutputPath => image_filter_state.menu.next(),
                                    _ => image_filter_state.cycle_value(false),
                                }
                            },
                        (Scene::ImageProcessing, KeyCode::Enter) if image_state == ImageProcessingState::Filter => {
                            match image_filter_state.menu.focus_field() {
                                ImageFilterField::InputPath | ImageFilterField::OutputPath => image_filter_state.menu.editing = true,
                                ImageFilterField::Run => image_filter_state.start_filter(&filename),
                                _ => {},
                            }
                        },

                        //                      <-- IMAGE PROCESSING SCENE KEY EVENTS -->
                        // The catch-all arms: these only get reached while no individual image
                        // screen is open, i.e. while the user is still picking one off the tabs.
                        (Scene::ImageProcessing, KeyCode::Left) | (Scene::ImageProcessing, KeyCode::Char('a')) => image_menu.previous(),
                        (Scene::ImageProcessing, KeyCode::Right) | (Scene::ImageProcessing, KeyCode::Char('d')) => image_menu.next(),
                        (Scene::ImageProcessing, KeyCode::Up) | (Scene::ImageProcessing, KeyCode::Char('w')) => image_menu.above(),
                        (Scene::ImageProcessing, KeyCode::Down) | (Scene::ImageProcessing, KeyCode::Char('s')) => image_menu.below(),
                        (Scene::ImageProcessing, KeyCode::Enter) => {
                            image_state = image_menu.selected_state();
                            previous_scene = Scene::ImageProcessing;

                            writeln!(_file, "State: '{:?}' at {} showing {:?} because '{:?}' was pressed.",
                                current_scene, Utc::now().format("%H-%M-%S"), image_menu, k.code)?;
                        },

                        (Scene::ImageProcessing, KeyCode::Esc) => {
                            // First ESC backs out of whichever image screen is open, a second
                            // one leaves the scene entirely, same as Video and Audio.
                            if image_state != ImageProcessingState::Default {
                                image_state = ImageProcessingState::Default;
                            } else {
                                current_scene = Scene::Home;
                            }

                            writeln!(_file, "State: '{:?}' at {} showing {:?} because '{:?}' was pressed.",
                                current_scene, Utc::now().format("%H-%M-%S"), image_menu, k.code)?;
                        },

                        _ => {}
                    }
                }
            }
        }
    }

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    Ok(())
}