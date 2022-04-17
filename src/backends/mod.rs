/*!
Ultimately, we need to show some slides to the user.
This module provides (some) backends that allow us to do this.

The provided backends are behind a feature flag, so that you don't pay if you
don't want to use one or the other (but please note that at least one must be
available).

### SDL2

This is the main backend, available by default. The slides are shown in a
window, and the end user can interact by using `n`, `p`, and other facilities.

### Crossterm

Sometimes, we don't have the luxury of SDL, or simply we're only interested in
showing some text in a terminal. Crossterm backend does not support all the
features of SDL2 (such as images, colors, ...) but can be useful anyway.
*/

#[cfg(feature = "cterm")]
pub mod crossterm;
#[cfg(feature = "sdl")]
pub mod sdl;

use crate::slideshow::Slideshow;

type ShouldQuit = bool;

/// A (vague) backend definition.
/// There are no strict requirements to become a backend - infact, we need to
/// have something that reacts to user inputs and present to screen. That being
/// said, we organized the code to have a `backend` - that is, the "canvas" -
/// and a `context` - which is what we will perform the operations on.
pub trait SlidyBackend {
    /// The only thing the backend really needs to do is to provide the
    /// Context.
    fn get_context(&mut self) -> Box<dyn SlidyContext + '_>;
}

/// The internal definition of a a context for a backend.
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

/// The available backends - once feature flags have been resolved.
pub enum Backends {
    #[cfg(feature = "sdl")]
    /// The SDL2 variant.
    Sdl,
    #[cfg(feature = "cterm")]
    /// The Crossterm variant.
    Crossterm,
}

fn match_try(value: &str) -> Result<Backends, String> {
    match value.to_lowercase().as_str() {
        #[cfg(feature = "sdl")]
        "sdl" => Ok(Backends::Sdl),
        #[cfg(feature = "cterm")]
        "crossterm" => Ok(Backends::Crossterm),
        _ => Err(format!("{} backend is not supported.", value)),
    }
}

#[must_use]
/// Get the actual backend implementation.
pub fn get_backend(which: &Backends) -> Box<dyn SlidyBackend> {
    use Backends::{Crossterm, Sdl};
    match which {
        #[cfg(feature = "sdl")]
        Sdl => Box::new(sdl::Backend::new()),
        #[cfg(feature = "cterm")]
        Crossterm => Box::new(crossterm::Backend::new()),
    }
}

impl TryFrom<String> for Backends {
    type Error = String;
    fn try_from(value: String) -> Result<Self, String> {
        match_try(value.as_str())
    }
}

impl TryFrom<&str> for Backends {
    type Error = String;
    fn try_from(value: &str) -> Result<Self, String> {
        match_try(value)
    }
}
