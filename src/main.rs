///////////////////////////////////////
/// MODS
///////////////////////////////////////
mod colors;
mod glyphs;
mod text_renderer;
mod interpreter;
mod runtime;
mod engine;

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
    window::{Window, WindowAttributes, WindowId}
};
use winit::dpi::LogicalSize;
use colors::{Colors, get_color, get_rgb};

///////////////////////////////////////
/// CONSTANTS
///////////////////////////////////////
const WIDTH: u32 = 320;
const HEIGHT: u32 = 240;


///////////////////////////////////////
/// STRUCTS
///////////////////////////////////////
#[derive(Default)]
struct App {
    window: Option<Box<dyn Window>>,
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
    let args = Args::parse();

    let source = fs::read_to_string(&args.file)
        .with_context(|| format!("Failed to read '{}'", args.file))?;

    // If you have interpreter execution:
    let mut runtime = runtime::Runtime::default();
    engine::run_program(&source, &mut runtime)
         .map_err(Error::msg)
         .with_context(|| format!("Failed to interpret '{}'", args.file))?;

    let event_loop = EventLoop::new()
        .context("Failed to create winit event loop")?;

    let app: &'static mut App = Box::leak(Box::new(App::default()));

    event_loop
        .run_app(app)
        .context("Event loop terminated with error")?;

    let _ = source;

    Ok(())
}

///////////////////////////////////////
/// IMPLEMENTATIONS
///////////////////////////////////////
impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &dyn ActiveEventLoop) {
        let attrs = WindowAttributes::default()
            .with_title("Crayon")
            .with_min_surface_size(LogicalSize::new((WIDTH * 2) as f64, (HEIGHT * 2) as f64));

        let window = event_loop.create_window(attrs).expect("create window");
        self.window = Some(window);
    }

    fn can_create_surfaces(&mut self, event_loop: &dyn ActiveEventLoop) {

    }

    fn window_event(&mut self, event_loop: &dyn ActiveEventLoop, _window_id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::RedrawRequested => {
                if let Some(window) = self.window.as_ref() {
                    let size = window.outer_size();
                    let surface_texture = SurfaceTexture::new(size.width, size.height, window);
                    let mut pixels = Pixels::new(WIDTH, HEIGHT, surface_texture).expect("create pixel buffer");

                    let fb_width = WIDTH as usize;
                    let fb_height = HEIGHT as usize;
                    let frame = pixels.frame_mut();

                    let foreground_color = get_rgb(Colors::Black);

                    for (i, px) in frame.chunks_exact_mut(4).enumerate() {
                        let x = (i as u32) % WIDTH;
                        let y = (i as u32) / WIDTH;

                        // let r = (x * 255 / WIDTH) as u8;
                        // let g = (y * 255 / HEIGHT) as u8;
                        // let b = 128u8;
                        // let a = 255u8;
                        let c = get_color(Colors::BrightGreen);
                        let r = c.0 as u8;
                        let g = c.1 as u8;
                        let b = c.2 as u8;
                        let a = c.3 as u8;

                        px.copy_from_slice(&[r, g, b, a]);
                    }

                    text_renderer::draw_text(
                        frame,
                        fb_width,
                        fb_height,
                        20,                    // x
                        40,                    // y
                        "ABCDEFGHIJKLMNOPQRSTUVWXYZ",
                        foreground_color,  // fg
                        None,                  // transparent background
                        1,                     // spacing
                    );

                    text_renderer::draw_text(
                        frame,
                        fb_width,
                        fb_height,
                        20,                    // x
                        60,                    // y
                        "abcdefghijklmnopqrstuvwxyz",
                        foreground_color,  // fg
                        None,                  // transparent background
                        1,                     // spacing
                    );

                    text_renderer::draw_text(
                        frame,
                        fb_width,
                        fb_height,
                        20,                    // x
                        80,                    // y
                        "0123456789",
                        foreground_color,  // fg
                        None,                  // transparent background
                        1,                     // spacing
                    );

                    text_renderer::draw_text(
                        frame,
                        fb_width,
                        fb_height,
                        20,                    // x
                        100,                    // y
                        "!@#$%^&*()",
                        foreground_color,  // fg
                        None,                  // transparent background
                        1,                     // spacing
                    );

                    text_renderer::draw_text(
                        frame,
                        fb_width,
                        fb_height,
                        20,                    // x
                        120,                    // y
                        "`-=_+[]{};':,./<>?~\"\\",
                        foreground_color,  // fg
                        None,                  // transparent background
                        1,                     // spacing
                    );

                    text_renderer::draw_text(
                        frame,
                        fb_width,
                        fb_height,
                        20,                    // x
                        140,                    // y
                        "Hello, world!",
                        foreground_color,  // fg
                        None,                  // transparent background
                        1,                     // spacing
                    );


                    if pixels.render().is_err() {
                        event_loop.exit();
                    }
                }
            },
            WindowEvent::SurfaceResized(size) => {
                if let Some(window) = self.window.as_ref() {
                    window.request_redraw();
                }
            }
            _ => { }
        }
    }

    fn about_to_wait(&mut self, event_loop: &dyn ActiveEventLoop) {
        if let Some(window) = self.window.as_ref() {
            window.request_redraw();
        }
    }
}
