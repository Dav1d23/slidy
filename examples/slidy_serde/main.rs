use std::thread::sleep;
use std::time::Duration;

use env_logger::{Builder, WriteStyle};
use log::LevelFilter;

use slidy::backends::sdl;

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
    // Init stuffs
    let backend = sdl::Backend::new();
    let mut context = backend.get_default_context();

    // Set the slides.
    let file_content = include_str!("./resources/input_file.json");
    let slides = serde_json::from_str(file_content).unwrap();
    context.slideshow_win.set_slides(slides);

    // Event loop
    loop {
        if context.manage_events() {
            break;
        };
        context.update_internals();
        context.render();
        sleep(Duration::from_secs(1));
    }
}
