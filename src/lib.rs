#![allow(clippy::pedantic)]

// Import external references
extern crate env_logger;
#[macro_use]
extern crate log;

// Re-export modules.
pub mod parser;
pub mod windows;

/// Get the default, included font. It is the FreeMono one, and it is included
/// in the binary, so no need to provide any other file.
pub fn get_default_font(
    context: &sdl2::ttf::Sdl2TtfContext,
) -> sdl2::ttf::Font {
    // TODO The font should be read from the slide directly
    //      and _then_ if nothing is provided use the default one.
    let fontbytes = include_bytes!("../assets/FreeMono.ttf");
    let mut points = 100;
    loop {
        let rwfont = sdl2::rwops::RWops::from_bytes(fontbytes).unwrap();
        if let Ok(font) = context.load_font_from_rwops(rwfont, points) {
            return font;
        }
        points -= 10;
        if points < 10 {
            panic!("This is not enough to show the font...");
        }
    }
}

/// Init the SDL context.
pub fn get_sdl_context() -> sdl2::Sdl {
    // Init stuffs.
    let sdl_context = sdl2::init().expect("Unable to init sdl.");
    // This is unused, but needs to stay in scope to be able to use the SDL_image.
    let _image_context = sdl2::image::init(sdl2::image::InitFlag::JPG)
        .expect("Unable to init image.");
    sdl_context
}

/// Init the TTF context.
pub fn get_ttf_context() -> sdl2::ttf::Sdl2TtfContext {
    sdl2::ttf::init().expect("Unable to init ttf.")
}
