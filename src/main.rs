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
}

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    file: String
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
        }
    }

    fn tick_interpreter(&mut self) {
        if self.program.halted || self.last_error.is_some() {
            return;
        }

        for _ in 0..STEPS_PER_FRAME {
            if self.program.halted {
                break;
            }

            match self.program.step(&mut self.runtime) {
                Ok(Some(engine::VmEffect::PlayTone { hz, ms })) => {
                    if let Some(audio) = self.audio.as_ref() {
                        let _ = audio.play_tone_hz(hz, ms, 0.20);
                    }
                }
                Ok(None) => {}
                Err(e) => {
                    eprintln!("Interpreter error: {}", e);
                    self.last_error = Some(e);
                    self.program.halted = true;
                    break;
                }
            }
        }
    }

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

        for line in &self.runtime.lines {
            text_renderer::draw_text(
                frame,
                fb_width,
                fb_height,
                8,
                y,
                line,
                fg,
                None,
                1,
            );
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
            WindowEvent::RedrawRequested => {
                self.tick_interpreter();
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
