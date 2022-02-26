#[cfg(feature = "cterm")]
pub mod crossterm;
#[cfg(feature = "sdl")]
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
    #[cfg(feature = "sdl")]
    Sdl,
    #[cfg(feature = "cterm")]
    Crossterm,
}

fn match_try(value: &str) -> Result<AvailableBackends, String> {
    match value.to_lowercase().as_str() {
        #[cfg(feature = "sdl")]
        "sdl" => Ok(AvailableBackends::Sdl),
        #[cfg(feature = "cterm")]
        "crossterm" => Ok(AvailableBackends::Crossterm),
        _ => Err(format!("{} backend is not supported.", value)),
    }
}

pub fn get_backend(which: AvailableBackends) -> Box<dyn SlidyBackend> {
    use AvailableBackends::*;
    match which {
        #[cfg(feature = "sdl")]
        Sdl => Box::new(sdl::Backend::new()),
        #[cfg(feature = "cterm")]
        Crossterm => Box::new(crossterm::Backend::new()),
    }
}

impl TryFrom<String> for AvailableBackends {
    type Error = String;
    fn try_from(value: String) -> Result<Self, String> {
        match_try(value.as_str())
    }
}

impl TryFrom<&str> for AvailableBackends {
    type Error = String;
    fn try_from(value: &str) -> Result<Self, String> {
        match_try(value)
    }
}
