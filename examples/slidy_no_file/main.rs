use std::thread::sleep;
use std::time::Duration;

use env_logger::{Builder, WriteStyle};
use log::LevelFilter;

use slidy::backends::sdl;
use slidy::backends::SlidyBackend;

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
    let mut backend = sdl::Backend::new();
    let mut context = backend.get_context();

    // Selected slide
    let mut rotations = 0.0;
    let text = "Starting soon... ";
    let mut display_text = text.to_owned();
    let mut text_counter = 0;
    let mut c1 = 120;
    let mut c2 = 50;

    // Event loop
    loop {
        let slides = slides::prepare_slide(rotations, display_text, c1, c2);
        context.set_slides(slides);

        if context.manage_inputs() {
            break;
        };
        context.render();

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
