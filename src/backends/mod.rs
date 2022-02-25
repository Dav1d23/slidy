pub mod crossterm;
pub mod sdl;

use crate::slideshow::Slideshow;

type ShouldQuit = bool;

pub trait SlidyBackend {
    fn get_context(&mut self) -> Box<dyn SlidyContext + '_>;
}

/// The definition of a backend.
/// It has to be able to update the slides, manage inputs, and render.
/// Note that these 3 steps are really vague, but this is what I need at the end :)
pub trait SlidyContext {
    /// Read the slide's format and use it.
    fn set_slides(&mut self, slideshow: Slideshow);
    /// React to user's input.
    fn manage_inputs(&mut self) -> ShouldQuit;
    /// Render to screen.
    fn render(&mut self);
}

pub enum AvailableBackends {
    Sdl,
    Crossterm,
}

pub fn get_backend(which: AvailableBackends) -> Box<dyn SlidyBackend> {
    use AvailableBackends::*;
    match which {
        Sdl => Box::new(sdl::Backend::new()),
        _ => unimplemented!("Backend not implemented."),
    }
}
