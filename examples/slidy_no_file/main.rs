use std::thread::sleep;
use std::time::Duration;

use env_logger::{Builder, WriteStyle};
use log::LevelFilter;

use slidy::backends::sdl;

mod slides;

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
    let backend = sdl::Backend::new();
    let mut context = backend.get_default_context();

    // Selected slide
    let mut slide_counter = 0;
    let mut rotations = 0.0;
    let text = "Starting soon... ";
    let mut display_text = text.to_owned();
    let mut text_counter = 0;
    let mut c1 = 120;
    let mut c2 = 50;

    // Event loop
    loop {
        let slides = slides::prepare_slides(rotations, display_text, c1, c2);
        context.slideshow_win.set_slides(slides);

        if context.manage_events() {
            break;
        };
        context.update_internals();
        context.render();

        // Update the "mutable" slide :)
        let (_slide_idx, slide_len) =
            context.slideshow_win.get_slides_counters();
        slide_counter = (slide_counter + 1) % slide_len;
        rotations = (rotations + 1.) % 360.0;
        text_counter = (text_counter + 1) % text.len();
        let (t1, t2) = text.split_at(text_counter);
        display_text = format!("{}{}", t2, t1);
        c1 = ((c1 as u16 + 29_u16) % 255_u16) as u8;
        c2 = ((c2 as u16 + 17_u16) % 255_u16) as u8;

        // Sleep for some time.
        sleep(Duration::from_millis(200));
    }
}
