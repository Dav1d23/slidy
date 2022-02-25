use std::thread::sleep;
use std::time::Duration;

use env_logger::{Builder, WriteStyle};
use log::LevelFilter;

use slidy::backends::sdl;
use slidy::backends::SlidyBackend;

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

    // Set the slides.
    let file_content = include_str!("./resources/input_file.json");
    let slides = serde_json::from_str(file_content).unwrap();
    context.set_slides(slides);

    // Event loop
    loop {
        if context.manage_inputs() {
            break;
        };
        context.render();
        sleep(Duration::from_secs(1));
    }
}
