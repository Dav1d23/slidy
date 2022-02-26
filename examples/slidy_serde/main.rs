use std::thread::sleep;
use std::time::Duration;

use slidy::backends::sdl;
use slidy::backends::SlidyBackend;

#[doc(hidden)]
fn main() {
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
