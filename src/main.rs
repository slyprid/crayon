///////////////////////////////////////
/// MODS
///////////////////////////////////////
mod colors;
mod glyphs;
mod text_renderer;
mod interpreter;
mod runtime;
mod engine;
mod audio;

///////////////////////////////////////
/// USINGS
///////////////////////////////////////
use anyhow::{Context, Error, Result};
use std::fs;
use clap::Parser;
use pixels::{Pixels, SurfaceTexture};
use winit:: {
    application::ApplicationHandler,
    event::WindowEvent,
    event_loop::{ActiveEventLoop, EventLoop},
    keyboard::{Key, NamedKey},
    window::{Window, WindowAttributes, WindowId},
    dpi::{LogicalSize, Size},
};
use colors::{Colors, get_color, get_rgb};

///////////////////////////////////////
/// CONSTANTS
///////////////////////////////////////
const WIDTH: u32 = 320;
const HEIGHT: u32 = 240;
const STEPS_PER_FRAME: usize = 500;


///////////////////////////////////////
/// STRUCTS
///////////////////////////////////////
struct App {
    window: Option<&'static Window>,
    pixels: Option<Pixels<'static>>,
    runtime: runtime::Runtime,
    program: engine::Program,
    last_error: Option<String>,
    audio: Option<audio::Audio>,
    input_state: Option<InputState>,
}

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    file: String
}

#[derive(Debug, Clone)]
struct InputState {
    var: String,
    buffer: String,
    line_index: usize,
    cursor_phase: usize,
    cursor_visible: bool,
    frame_counter: usize,
}

///////////////////////////////////////
/// MAIN
///////////////////////////////////////
fn main() -> Result<()> {
    eprintln!("=========================================================");
    eprintln!(">> CRAYON Interpreter ");
    eprintln!("=========================================================");
    eprintln!();

    let args = Args::parse();

    let source = fs::read_to_string(&args.file)
        .with_context(|| format!("Failed to read '{}'", args.file))?;

    let program = engine::Program::from_source(&source)
        .map_err(anyhow::Error::msg)
        .with_context(|| format!("Failed to parse '{}'", args.file))?;

    let runtime = runtime::Runtime::default();

    let event_loop = EventLoop::new().context("Failed to create event loop")?;
    let app: &'static mut App = Box::leak(Box::new(App::new(runtime, program)));

    event_loop
        .run_app(app)
        .context("Event loop terminated with error")?;

    Ok(())
}

///////////////////////////////////////
/// IMPLEMENTATIONS
///////////////////////////////////////
impl App {
    fn new(runtime: runtime::Runtime, program: engine::Program) -> Self {
        Self {
            window: None,
            pixels: None,
            runtime,
            program,
            last_error: None,
            audio: audio::Audio::new().ok(),
            input_state: None,
        }
    }

    /////////////////////////////////////////
    /// Application TICK
    /////////////////////////////////////////
    fn tick_interpreter(&mut self) {
        if self.program.halted || self.last_error.is_some() {
            return;
        }

        if self.input_state.is_some() {
            return;
        }

        for _ in 0..STEPS_PER_FRAME {
            if self.program.halted || self.input_state.is_some() {
                break;
            }

            match self.program.step(&mut self.runtime) {
                Ok(Some(engine::VmEffect::PlayTone { hz, ms })) => {
                    if let Some(audio) = self.audio.as_ref() {
                        let _ = audio.play_tone_hz(hz, ms, 0.20);
                    }
                }
                Ok(Some(engine::VmEffect::BeginInput { prompt, var })) => {
                    self.begin_input(prompt, var);
                }
                Ok(None) => {}
                Err(e) => {
                    let user_msg = match e.kind {
                        interpreter::RuntimeErrorKind::Syntax => format!("?SN ERROR: {}", e.message),
                        interpreter::RuntimeErrorKind::DivideByZero => format!("?/0 ERROR: {}", e.message),
                        interpreter::RuntimeErrorKind::TypeMismatch => format!("?TM ERROR: {}", e.message),
                    };

                    eprintln!("{}", user_msg);
                    self.last_error = Some(user_msg);
                    self.program.halted = true;
                    break;
                }
            }
        }
    }

    ////////////////////////////////////
    /// Application RENDER
    ////////////////////////////////////
    fn render(&mut self) {
        let Some(pixels) = self.pixels.as_mut() else { return };

        let frame = pixels.frame_mut();
        let fb_width = WIDTH as usize;
        let fb_height = HEIGHT as usize;

        // Background
        let bg = self.runtime.current_bg_rgba();
        for px in frame.chunks_exact_mut(4) {
            px.copy_from_slice(&bg);
        }

        // Draw interpreter output lines
        let fg = get_rgb(Colors::Black);
        let mut y = 8i32;
        let line_step = (glyphs::GLYPH_HEIGHT as i32) + 2;

        for (idx, line) in self.runtime.lines.iter().enumerate() {
            text_renderer::draw_text(frame, fb_width, fb_height, 8, y, line, fg, None, 1);

            // overlay live INPUT edit + cursor on the active prompt line
            if let Some(state) = self.input_state.as_ref() {
                if idx == state.line_index {
                    let input_x = 8 + glyphs::GLYPH_WIDTH as i32; // after '?'
                    text_renderer::draw_text(
                        frame,
                        fb_width,
                        fb_height,
                        input_x,
                        y,
                        &state.buffer,
                        fg,
                        None,
                        1,
                    );

                    if state.cursor_visible {
                        let colors = [
                            Colors::BrightGreen,
                            Colors::BrightYellow,
                            Colors::BrightBlue,
                            Colors::BrightRed,
                            Colors::White,
                            Colors::LightGreen,
                            Colors::Magenta,
                            Colors::BrightOrange,
                        ];
                        let c = get_rgb(colors[state.cursor_phase % colors.len()]);
                        let cursor_x = input_x
                            + (state.buffer.chars().count() as i32 * glyphs::GLYPH_WIDTH as i32);

                        // Draw cursor as GLYPH002
                        text_renderer::draw_glyph(
                            frame,
                            fb_width,
                            fb_height,
                            cursor_x,
                            y,
                            6, // GLYPH006
                            c,
                            None,
                            6,
                            8
                        );
                    }
                }
            }

            y += line_step;
            if y + glyphs::GLYPH_HEIGHT as i32 >= HEIGHT as i32 {
                break;
            }
        }

        if let Some(err) = &self.last_error {
            let err_fg = get_rgb(Colors::BrightRed);
            let y_err = HEIGHT as i32 - (glyphs::GLYPH_HEIGHT as i32 + 4);
            text_renderer::draw_text(
                frame,
                fb_width,
                fb_height,
                8,
                y_err.max(0),
                err,
                err_fg,
                None,
                1,
            );
        }

        if pixels.render().is_err() {
            // no event_loop here; caller handles close on next event
        }
    }

    ////////////////////////////////////
    /// Begin Input
    ////////////////////////////////////
    fn begin_input(&mut self, prompt: Option<String>, var: String) {
        if let Some(p) = prompt {
            self.runtime.print(p);
        }

        // show ? on its own line, and user input will visually follow it
        self.runtime.print("?".to_string());
        let line_index = self.runtime.lines.len().saturating_sub(1);

        self.input_state = Some(InputState {
            var,
            buffer: String::new(),
            line_index,
            cursor_phase: 0,
            cursor_visible: true,
            frame_counter: 0,
        });
    }

    ////////////////////////////////////
    /// Commit Input
    ////////////////////////////////////
    fn commit_input(&mut self) {
        let Some(state) = self.input_state.clone() else { return };
        let raw = state.buffer;
        let var = state.var;

        if var.ends_with('$') {
            self.runtime
                .vars
                .insert(var, runtime::Value::Str(raw.clone()));
        } else {
            let n: f64 = match raw.trim().parse() {
                Ok(v) => v,
                Err(_) => {
                    self.last_error = Some("TYPE MISMATCH: numeric INPUT expected".to_string());
                    self.program.halted = true;
                    self.input_state = None;
                    return;
                }
            };
            self.runtime.vars.insert(var, runtime::Value::Num(n));
        }

        // finalize prompt line as "?<typed>"
        if let Some(line) = self.runtime.lines.get_mut(state.line_index) {
            *line = format!("?{}", raw);
        } else {
            self.runtime.print(raw);
        }

        self.input_state = None;
    }

    fn animate_input_cursor(&mut self) {
        let Some(state) = self.input_state.as_mut() else { return };

        state.frame_counter = state.frame_counter.wrapping_add(1);

        // blink every ~20 frames
        if state.frame_counter % 20 == 0 {
            state.cursor_visible = !state.cursor_visible;
        }

        // color cycle every ~8 frames
        if state.frame_counter % 8 == 0 {
            state.cursor_phase = (state.cursor_phase + 1) % 8;
        }
    }

    fn handle_input_key(&mut self, event: &winit::event::KeyEvent) -> bool {
        // returns true when consumed by INPUT mode
        let Some(state) = self.input_state.as_mut() else { return false };

        use winit::event::ElementState;
        if event.state != ElementState::Pressed {
            return true;
        }

        match &event.logical_key {
            Key::Named(NamedKey::Enter) => {
                self.commit_input();
            }
            Key::Named(NamedKey::Backspace) => {
                state.buffer.pop();
            }
            Key::Character(text) => {
                for ch in text.chars() {
                    if !ch.is_control() {
                        state.buffer.push(ch);
                    }
                }
            }
            _ => {}
        }

        true
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_some() {
            return;
        }

        let attrs = WindowAttributes::default()
            .with_title("Crayon - Extended Color Basic 2.1 Interpreter")
            .with_maximized(true)
            .with_inner_size(Size::Logical(LogicalSize::new(
                (WIDTH * 2) as f64,
                (HEIGHT * 2) as f64,
            )));

        let window = event_loop.create_window(attrs).expect("create window");
        window.set_maximized(true);

        let window_ref: &'static Window = Box::leak(Box::new(window));

        let size = window_ref.inner_size();
        let surface = SurfaceTexture::new(size.width, size.height, window_ref);
        let pixels = Pixels::new(WIDTH, HEIGHT, surface).expect("create pixel buffer");

        self.pixels = Some(pixels);
        self.window = Some(window_ref);

        window_ref.request_redraw();
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _window_id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::KeyboardInput { event, .. } => {
                if self.handle_input_key(&event) {
                    if let Some(window) = self.window.as_ref() {
                        window.request_redraw();
                    }
                    return;
                }
            }
            WindowEvent::RedrawRequested => {
                self.tick_interpreter();
                self.animate_input_cursor();
                self.render();
            },
            WindowEvent::Resized(size) => {
                if let Some(p) = self.pixels.as_mut() {
                    let _ = p.resize_surface(size.width, size.height);
                }
                if let Some(w) = self.window.as_ref() {
                    w.request_redraw();
                }
            }
            _ => { }
        }
    }

    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        if let Some(window) = self.window.as_ref() {
            window.request_redraw();
        }
    }
}
