///////////////////////////////////////
/// MODS
///////////////////////////////////////
mod colors;
mod glyphs;
mod text_renderer;

///////////////////////////////////////
/// USINGS
///////////////////////////////////////
use clap::Parser;
use pixels::{Pixels, SurfaceTexture};
use winit:: {
    application::ApplicationHandler,
    event::WindowEvent,
    event_loop::{ActiveEventLoop, EventLoop},
    window::{Window, WindowAttributes, WindowId}
};
use winit::dpi::LogicalSize;
use colors::{Colors, get_color};

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
    window: Option<Window>,
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
fn main() -> Result<(), winit::error::EventLoopError> {
    let args = Args::parse();
    let event_loop = EventLoop::new()?;
    let mut app = App::default();
    event_loop.run_app(&mut app)

    //println!("Parsing file: {}", args.file)
}

///////////////////////////////////////
/// IMPLEMENTATIONS
///////////////////////////////////////
impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let attrs = WindowAttributes::default()
            .with_title("Crayon")
            .with_inner_size(LogicalSize::new((WIDTH * 2) as f64, (HEIGHT * 2) as f64));

        let window = event_loop.create_window(attrs).expect("create window");
        self.window = Some(window);
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _window_id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::RedrawRequested => {
                if let Some(window) = self.window.as_ref() {
                    let size = window.inner_size();
                    let surface_texture = SurfaceTexture::new(size.width, size.height, window);
                    let mut pixels = Pixels::new(WIDTH, HEIGHT, surface_texture).expect("create pixel buffer");

                    let frame = pixels.frame_mut();
                    let fb_width = WIDTH as usize;
                    let fb_height = HEIGHT as usize;

                    let frame = pixels.frame_mut();
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
                        [255, 255, 255, 255],  // fg
                        None,                  // transparent background
                        1,                     // spacing
                    );


                    if pixels.render().is_err() {
                        event_loop.exit();
                    }
                }
            },
            WindowEvent::Resized(size) => {
                if let Some(window) = self.window.as_ref() {
                    window.request_redraw();
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
