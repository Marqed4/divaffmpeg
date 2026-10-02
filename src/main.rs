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
mod styles;
mod component;
mod background;
mod explanations;
mod implementations;

use home::{HomeState, HomeMenuState};
use video::{VideoProcessingState, DirectionsMenuState, complete_textarea};
use implementations::{
    ConvertState, ConvertField, CompressState, CompressField,
    TrimState, TrimField, MergeState, MergeField,
    ExtractionState, ExtractionField,
    ResizeState, ResizeField,
};

//                      <-- SCENE STATE -->
// Tracks exactly which screen is live at any given time. Using a single enum
// instead of several independent bools makes it structurally impossible for
// two scenes to render (or receive key input) in the same frame.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Scene {
    Start,
    Home,
    ImageProcessing, // W.I.P.
    VideoProcessing, 
    AudioProcessing, // W.I.P.
    Exit,
}

fn main() -> Result<(), io::Error> {

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
    let filename: String = format!("log/DivaFFMPEG-{}.log", Utc::now().format("%Y-%m-%d_%H-%M-%S"));
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
    let mut convert_state: ConvertState = ConvertState::new();
    let mut compress_state: CompressState = CompressState::new();
    let mut trim_state: TrimState = TrimState::new();
    let mut merge_state: MergeState = MergeState::new();
    let mut extraction_state: ExtractionState = ExtractionState::new();
    let mut resize_state: ResizeState = ResizeState::new();
    

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

                //                      <-- IMAGE PROCESSING SCENE (W.I.P.) -->
                Scene::ImageProcessing => {
                    let outer: Rc<[Rect]> = Layout::default()
                        .direction(Direction::Vertical)
                        .constraints([
                            Constraint::Fill(1),
                        ])
                        .split(size);

                    let wip_block = Block::default()
                        .borders(Borders::ALL)
                        .border_type(BorderType::Rounded)
                        .title("✦  Diva FFMPEG: Image Processing  ✦")
                        .title_alignment(Alignment::Center)
                        .bold();

                    let wip_inner = wip_block.inner(outer[0]);
                    frame.render_widget(wip_block, outer[0]);

                    let wip_text = Paragraph::new(
                        styles::title(wip_inner.width, wip_inner.height).render("Work in progress...").as_bytes().into_text().unwrap()
                    ).alignment(Alignment::Center);

                    frame.render_widget(wip_text, wip_inner);
                },

                //                      <-- AUDIO PROCESSING SCENE (W.I.P.) -->
                Scene::AudioProcessing => {
                    let outer: Rc<[Rect]> = Layout::default()
                        .direction(Direction::Vertical)
                        .constraints([
                            Constraint::Fill(1),
                        ])
                        .split(size);

                    let wip_block = Block::default()
                        .borders(Borders::ALL)
                        .border_type(BorderType::Rounded)
                        .title("✦  Diva FFMPEG: Audio Processing  ✦")
                        .title_alignment(Alignment::Center)
                        .bold();

                    let wip_inner = wip_block.inner(outer[0]);
                    frame.render_widget(wip_block, outer[0]);

                    let wip_text = Paragraph::new(
                        styles::title(wip_inner.width, wip_inner.height).render("Work in progress...").as_bytes().into_text().unwrap()
                    ).alignment(Alignment::Center);

                    frame.render_widget(wip_text, wip_inner);
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
                                    if resize_state.menu.focus_field() == ResizeField::InputPath {
                                        resize_state.sync_dimensions();
                                    }
                                },
                                KeyCode::Tab => {
                                    match resize_state.menu.focus_field() {
                                        ResizeField::InputPath => {
                                            complete_textarea(&mut resize_state.input_file_path);
                                            resize_state.sync_dimensions();
                                        },
                                        ResizeField::OutputPath => complete_textarea(&mut resize_state.output_file_path),
                                        _ => {},
                                    }
                                },
                                _ => {
                                    match resize_state.menu.focus_field() {
                                        ResizeField::InputPath => { resize_state.input_file_path.input(Event::Key(k)); },
                                        ResizeField::OutputPath => { resize_state.output_file_path.input(Event::Key(k)); },
                                        ResizeField::Width => { resize_state.width.input(Event::Key(k)); },
                                        ResizeField::Height => { resize_state.height.input(Event::Key(k)); },
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
                                        CompressField::InputPath => complete_textarea(&mut compress_state.input_file_path),
                                        CompressField::OutputPath => complete_textarea(&mut compress_state.output_file_path),
                                        _ => {},
                                    }
                                },
                                _ => {
                                    match compress_state.menu.focus_field() {
                                        CompressField::InputPath => { compress_state.input_file_path.input(Event::Key(k)); },
                                        CompressField::OutputPath => { compress_state.output_file_path.input(Event::Key(k)); },
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
                                    if trim_state.menu.focus_field() == TrimField::InputPath {
                                        trim_state.sync_duration();
                                    }
                                },
                                KeyCode::Tab => {
                                    match trim_state.menu.focus_field() {
                                        TrimField::InputPath => complete_textarea(&mut trim_state.input_file_path),
                                        TrimField::OutputPath => complete_textarea(&mut trim_state.output_file_path),
                                        _ => {},
                                    }
                                },
                                _ => {
                                    match trim_state.menu.focus_field() {
                                        TrimField::InputPath => { trim_state.input_file_path.input(Event::Key(k)); },
                                        TrimField::OutputPath => { trim_state.output_file_path.input(Event::Key(k)); },
                                        TrimField::Start => { trim_state.start_time.input(Event::Key(k)); },
                                        TrimField::End => { trim_state.end_time.input(Event::Key(k)); },
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
                                        MergeField::InputA => complete_textarea(&mut merge_state.input_a_path),
                                        MergeField::InputB => complete_textarea(&mut merge_state.input_b_path),
                                        MergeField::OutputPath => complete_textarea(&mut merge_state.output_file_path),
                                        _ => {},
                                    }
                                },
                                _ => {
                                    match merge_state.menu.focus_field() {
                                        MergeField::InputA => { merge_state.input_a_path.input(Event::Key(k)); },
                                        MergeField::InputB => { merge_state.input_b_path.input(Event::Key(k)); },
                                        MergeField::OutputPath => { merge_state.output_file_path.input(Event::Key(k)); },
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
                                    if extraction_state.menu.focus_field() == ExtractionField::Input {
                                        extraction_state.sync_duration();
                                    }
                                },
                                KeyCode::Tab => {
                                    match extraction_state.menu.focus_field() {
                                        ExtractionField::Input => complete_textarea(&mut extraction_state.input_path),
                                        ExtractionField::OutputPath => complete_textarea(&mut extraction_state.output_file_path),
                                        _ => {},
                                    }
                                },
                                _ => {
                                    match extraction_state.menu.focus_field() {
                                        ExtractionField::Input => { extraction_state.input_path.input(Event::Key(k)); },
                                        ExtractionField::OutputPath => { extraction_state.output_file_path.input(Event::Key(k)); },
                                        ExtractionField::Start => { extraction_state.start_time.input(Event::Key(k)); },
                                        ExtractionField::End => { extraction_state.end_time.input(Event::Key(k)); },
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

                            // Image and audio scenes are still W.I.P. placeholders, but they're
                            // still reachable so their in-progress layouts can be seen and iterated on.
                            current_scene = match home_state {
                                HomeState::Image => Scene::ImageProcessing,
                                HomeState::Video => Scene::VideoProcessing,
                                HomeState::Audio => Scene::AudioProcessing,
                                HomeState::Default => Scene::Home,
                            };

                            writeln!(_file, "State: '{:?}' at {} because '{:?}' was pressed.",
                                current_scene, Utc::now().format("%H-%M-%S"), k.code)?;
                        },

                        //                      <-- AUDIO PROCESSING SCENE KEY EVENTS -->
                        (Scene::AudioProcessing, KeyCode::Esc) => {
                            // PLACE THE AUDIO_PROCESSING STATE HERE
                            current_scene = Scene::Home;
                            
                            writeln!(_file, "State: '{:?}' at {} showing {:?} because '{:?}' was pressed.",
                                current_scene, Utc::now().format("%H-%M-%S"), video_menu, k.code)?;
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
                                    ConvertField::InputPath | ConvertField::OutputPath => convert_state.menu.previous(),
                                    _ => convert_state.cycle_value(true),
                                }
                            },
                        (Scene::VideoProcessing, KeyCode::Down) | (Scene::VideoProcessing, KeyCode::Char('s'))
                            if video_state == VideoProcessingState::Convert => {
                                match convert_state.menu.focus_field() {
                                    ConvertField::InputPath | ConvertField::OutputPath => convert_state.menu.next(),
                                    _ => convert_state.cycle_value(false),
                                }
                            },
                        (Scene::VideoProcessing, KeyCode::Enter) if video_state == VideoProcessingState::Convert => {
                            match convert_state.menu.focus_field() {
                                ConvertField::InputPath | ConvertField::OutputPath => convert_state.menu.editing = true,
                                ConvertField::Run => convert_state.start_convert(&filename),
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
                                    CompressField::InputPath | CompressField::OutputPath => compress_state.menu.previous(),
                                    _ => compress_state.cycle_value(true),
                                }
                            },
                        (Scene::VideoProcessing, KeyCode::Down) | (Scene::VideoProcessing, KeyCode::Char('s'))
                            if video_state == VideoProcessingState::Compress => {
                                match compress_state.menu.focus_field() {
                                    CompressField::InputPath | CompressField::OutputPath => compress_state.menu.next(),
                                    _ => compress_state.cycle_value(false),
                                }
                            },
                        (Scene::VideoProcessing, KeyCode::Enter) if video_state == VideoProcessingState::Compress => {
                            match compress_state.menu.focus_field() {
                                CompressField::InputPath | CompressField::OutputPath => compress_state.menu.editing = true,
                                CompressField::Run => compress_state.start_compress(&filename),
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
                                TrimField::Run => trim_state.start_trim(&filename),
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
                                MergeField::Run => merge_state.start_merge(&filename),
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
                                    ExtractionField::Input | ExtractionField::OutputPath
                                        | ExtractionField::Start | ExtractionField::End => extraction_state.menu.previous(),
                                    _ => extraction_state.cycle_value(true),
                                }
                            },
                        (Scene::VideoProcessing, KeyCode::Down) | (Scene::VideoProcessing, KeyCode::Char('s'))
                            if video_state == VideoProcessingState::Extraction => {
                                match extraction_state.menu.focus_field() {
                                    ExtractionField::Input | ExtractionField::OutputPath
                                        | ExtractionField::Start | ExtractionField::End => extraction_state.menu.next(),
                                    _ => extraction_state.cycle_value(false),
                                }
                            },
                        (Scene::VideoProcessing, KeyCode::Enter) if video_state == VideoProcessingState::Extraction => {
                            match extraction_state.menu.focus_field() {
                                ExtractionField::Input | ExtractionField::OutputPath => extraction_state.menu.editing = true,
                                ExtractionField::Run => extraction_state.start_extraction(&filename),
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
                                ResizeField::Run => resize_state.start_resize(&filename),
                                ResizeField::Lock => resize_state.toggle_lock(),
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

                        //                      <-- IMAGE PROCESSING SCENE KEY EVENTS -->
                        (Scene::ImageProcessing, KeyCode::Esc) => {
                            // PLACE THE IMAGE_PROCESSING STATE HERE
                            current_scene = Scene::Home;
                            
                            writeln!(_file, "State: '{:?}' at {} showing {:?} because '{:?}' was pressed.",
                                current_scene, Utc::now().format("%H-%M-%S"), video_menu, k.code)?;
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