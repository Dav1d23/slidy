// Import external references
extern crate env_logger;

use env_logger::{Builder, WriteStyle};

use log::LevelFilter;

// Sdl2 imports.
// @todo this can go in the lib so that this crate does not have dependencies
// to the SDL2 part.  This can be useful in case of a feature where we use the
// terminal to show the slides.

use sdl2::event::Event;
use sdl2::keyboard::Keycode;

// Std imports

use std::thread::sleep;
use std::time::Duration;

// Slidy imports.

use slidy::windows::slideshow::SlideShowWindow;


#[doc(hidden)]
fn main() {

    let level = LevelFilter::Info;

    // Init logger.
    let mut log_builder = Builder::new();
    log_builder
        .filter_level(level)
        .write_style(WriteStyle::Always)
        .init();

    // Init stuffs
    let sdl_context = slidy::get_sdl_context();
    let ttf_context = slidy::get_ttf_context();
    let free_mono = slidy::get_default_font(&ttf_context);

    // 1. The slideshow window
    let mut slideshow_win = SlideShowWindow::new(
        &sdl_context,
        &free_mono,
        true,
        800,
        600,
    );

    // Set the slides.
    let file_content = include_str!("./resources/input_file.json");
    let slides = serde_json::from_str(&file_content).unwrap();
    slideshow_win.set_slides(slides);

    // Event loop
    let mut event_pump = sdl_context.event_pump().unwrap();
    'running: loop {
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
                _ => {},
            }
        }

        slideshow_win.present_slide();
        slideshow_win.main_win.canvas.present();
        sleep(Duration::from_secs(1));
    }
}
