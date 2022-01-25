// Import external references
extern crate env_logger;
#[macro_use]
extern crate log;

// Std imports.

use std::str::FromStr;
use std::thread::sleep;
use std::time::Duration;

// Non std imports.

use env_logger::{Builder, WriteStyle};
use log::LevelFilter;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use structopt::StructOpt;

// Slidylib imports.

use slidy::windows::slideshow::SlideShowWindow;
use slidy::windows::utils::CanvasPresent;

// Local modules.

mod slides;

/// Define the window options.
pub struct ScreenOptions {
    pub h: u32,
    pub w: u32,
    pub resizable: bool,
    pub fullscreen: bool,
}

#[derive(Debug, structopt::StructOpt)]
/// My Amazing Personal Slideshow command line options.
struct Args {
    #[structopt(short = "l", long = "log-level", default_value = "INFO")]
    /// The log level to be used.
    log_level: String,
    #[structopt(short = "w", long = "window-size", default_value = "800x600")]
    /// Window size, expressed as <h>x<w>.
    winsize: String,
    #[structopt(long = "fixed-size")]
    /// If set, the user can't resize the window, which will be stuck to window-size
    /// Note: looks like "down-resizing" is always possible...
    fixed_size: bool,
}

#[doc(hidden)]
fn main() {
    let args = Args::from_args();

    let level = LevelFilter::from_str(&args.log_level)
        .expect("Please provide a valid log level.");

    // Init logger.
    let mut log_builder = Builder::new();
    log_builder
        .filter(None, level)
        .write_style(WriteStyle::Always)
        .init();

    info!("Using log level: {}", level);

    // Parse the window size value.
    let (h, w) = match args
        .winsize
        .split('x')
        .map(|e| {
            e.parse().unwrap_or_else(|_| {
                panic!("Unable to parse `{}` into u32", e)
            })
        })
        .collect::<Vec<u32>>().as_slice() {
            [a, b] => (*a, *b),
            _ => panic!(
                "Must provide 2 parameters for the winsize, found more than that :)",
            ),
        };

    let screen_options = ScreenOptions {
        h,
        w,
        fullscreen: false,
        resizable: !args.fixed_size,
    };

    // Init stuffs
    let sdl_context = slidy::get_sdl_context();
    let ttf_context = slidy::get_ttf_context();
    let free_mono = slidy::get_default_font(&ttf_context);

    // 1. The slideshow window
    let mut slideshow_win = SlideShowWindow::new(
        &sdl_context,
        &free_mono,
        screen_options.resizable,
        screen_options.h,
        screen_options.w,
    );

    // Selected slide
    let mut slide_counter = 0;
    let mut rotations = 0.0;
    let text = "Starting soon... ";
    let mut text_counter = 0;
    let mut c1 = 120;
    let mut c2 = 50;

    // Set the slides.
    let slides = slides::prepare_slides(rotations, text.to_string(), c1, c2);
    slideshow_win.set_slides(slides);

    // Event loop
    let mut event_pump = sdl_context.event_pump().unwrap();
    'running: loop {
        // Check if we have new slides
        for event in event_pump.poll_iter() {
            // Then, match events that should always occur, whatever window is
            // highlighted.
            match event {
                // Quit event, QUIT (I guess F4, C-c) or Q or ESC
                Event::Quit { .. }
                | Event::KeyUp {
                    keycode: Some(Keycode::Escape),
                    ..
                }
                | Event::KeyUp {
                    keycode: Some(Keycode::Q),
                    ..
                } => break 'running,
                _ => {}
            }
        }

        slideshow_win.present_slide();
        slideshow_win.generic_win.canvases_present();

        let (_slide_idx, slide_len) = slideshow_win.get_slides_counters();

        sleep(Duration::from_millis(500));
        slide_counter = (slide_counter + 1) % slide_len;
        rotations = (rotations + 1.) % 360.0;
        text_counter = (text_counter + 1) % text.len();
        let (t1, t2) = text.split_at(text_counter);
        let new_text = format!("{}{}", t2, t1);
        c1 = ((c1 as u16 + 29_u16) % 255_u16) as u8;
        c2 = ((c2 as u16 + 17_u16) % 255_u16) as u8;

        let slides = slides::prepare_slides(rotations, new_text, c1, c2);

        debug!("Selected slide {}", slide_counter);
        slideshow_win.set_slides(slides);
        slideshow_win.set_slide(slide_counter);
    }
}
