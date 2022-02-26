//! The provided SDL2 backend.
use crate::slideshow::Slideshow;
use crossterm::{
    event::{poll, read, Event},
    terminal,
};
use log::{error, trace};
use std::{marker::PhantomData, time::Duration};

/// The backend.
pub struct Backend {}

impl super::SlidyBackend for Backend {
    fn get_context(&mut self) -> Box<dyn super::SlidyContext + '_> {
        let ctx = self.internal_get_context();
        Box::new(ctx)
    }
}

/// The context, which contains the live data.
/// This structure has to be used to update the slides in the event loop, or
/// manage keypresses, and so on.
pub struct Context<'backend> {
    slide_id: usize,
    slides: Slideshow,
    _lifetime: PhantomData<&'backend ()>,
}

impl Backend {
    /// Create a new backend.
    pub fn new() -> Backend {
        terminal::enable_raw_mode().unwrap();
        Backend {}
    }

    /// Get the runnable context.
    fn internal_get_context(&self) -> Context {
        Context {
            slide_id: 0,
            slides: Slideshow::default(),
            _lifetime: PhantomData,
        }
    }
}

impl Drop for Backend {
    fn drop(&mut self) {
        match terminal::disable_raw_mode() {
            Ok(_) => trace!("raw mode disabled."),
            Err(e) => error!("Unable to switch to raw mode: {:?}", e),
        }
    }
}

impl Default for Backend {
    fn default() -> Self {
        Self::new()
    }
}

impl<'b> super::SlidyContext for Context<'b> {
    fn set_slides(&mut self, slides: crate::slideshow::Slideshow) {
        self.slides = slides;
    }

    /// Manage the incoming events.
    fn manage_inputs(&mut self) -> super::ShouldQuit {
        while let Ok(true) = poll(Duration::from_secs(5)) {
            let evt = read().expect("Poll told us this should work.");
            trace!("{:#?}", evt);
        }
        false
    }

    /// Render the windows.
    fn render(&mut self) {
        unimplemented!("render the slide!")
    }
}
