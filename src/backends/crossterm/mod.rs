//! The provided SDL2 backend.
use crate::slideshow::{Position, SectionMain, Slideshow};
use crossterm::{
    cursor,
    event::{poll, read, Event, KeyCode, KeyEvent},
    style::{Color, PrintStyledContent, Stylize},
    terminal, QueueableCommand,
};

use std::io::{stdout, Stdout, Write};
use std::{marker::PhantomData, time::Duration};
use tracing::{debug, error, trace, warn};

/// The backend.
pub struct Backend {}

impl super::SlidyBackend for Backend {
    fn get_context(&mut self) -> Box<dyn super::SlidyContext + '_> {
        let ctx = self.internal_get_context();
        Box::new(ctx)
    }
}

impl Backend {
    /// Create a new backend.
    pub fn new() -> Backend {
        debug!("Enable raw-mode.");
        terminal::enable_raw_mode().unwrap();
        Backend {}
    }

    /// Get the runnable context.
    fn internal_get_context(&self) -> Context {
        let mut stdout = stdout();
        stdout.queue(cursor::Hide).unwrap();
        stdout.flush().unwrap();

        Context {
            slide_id: 0,
            slides: Slideshow::default(),
            _lifetime: PhantomData,
            stdout,
        }
    }
}

impl Drop for Backend {
    fn drop(&mut self) {
        match terminal::disable_raw_mode() {
            Ok(_) => debug!("Raw-mode disabled."),
            Err(e) => error!("Unable to switch to raw-mode: {:?}", e),
        }
    }
}

impl Default for Backend {
    fn default() -> Self {
        Self::new()
    }
}

/// The context, which contains the live data.
/// This structure has to be used to update the slides in the event loop, or
/// manage keypresses, and so on.
pub struct Context<'backend> {
    slide_id: usize,
    slides: Slideshow,
    _lifetime: PhantomData<&'backend ()>,
    stdout: Stdout,
}

impl<'b> super::SlidyContext for Context<'b> {
    fn set_slides(&mut self, slides: crate::slideshow::Slideshow) {
        self.slides = slides;
    }

    /// Manage the incoming events.
    fn manage_inputs(&mut self) -> super::ShouldQuit {
        while let Ok(true) = poll(Duration::ZERO) {
            let evt = read().expect("Poll told us this should work.");
            trace!("{:#?}", evt);
            match evt {
                Event::Key(KeyEvent {
                    code: KeyCode::Char('q'),
                    ..
                }) => return true,
                Event::Key(KeyEvent {
                    code: KeyCode::Char('n'),
                    ..
                }) => {
                    self.slide_id =
                        (self.slide_id + 1).min(self.slides.slides.len() - 1)
                }
                Event::Key(KeyEvent {
                    code: KeyCode::Char('p'),
                    ..
                }) => {
                    self.slide_id = if self.slide_id > 0 {
                        self.slide_id - 1
                    } else {
                        0
                    }
                }
                _ => {}
            }
        }
        false
    }

    /// Render the windows.
    fn render(&mut self) {
        trace!("Rendering phase");
        self.clear_all();
        let term_size = terminal::size().unwrap();
        debug!("Considering slide {}", self.slide_id);

        if let Some(slide) = self.slides.slides.get(self.slide_id) {
            for sec in slide.sections.iter() {
                // @TODO why is position 0. 0. if it is not there?
                let pos =
                    sec.position.as_ref().unwrap_or(&Position { x: 0., y: 0. });
                let x: u16 = (term_size.0 as f32 * pos.x).ceil() as u16;
                let y: u16 = (term_size.1 as f32 * pos.y).ceil() as u16;
                if let Some(SectionMain::Text(sec_text)) = &sec.sec_main {
                    self.stdout.queue(cursor::MoveTo(x, y)).unwrap();
                    // I should use the "style" defined in the slides instead of this one.
                    let styled = sec_text.text.as_str().with(Color::White);
                    self.stdout.queue(PrintStyledContent(styled)).unwrap();
                }
            }
        } else {
            warn!("There are no slides to show!");
        }
        self.flush();
    }
}

impl Context<'_> {
    fn clear_all(&mut self) {
        self.stdout
            .queue(terminal::Clear(terminal::ClearType::All))
            .unwrap();
    }

    fn flush(&mut self) {
        self.stdout.flush().unwrap();
    }
}

impl Drop for Context<'_> {
    fn drop(&mut self) {
        self.stdout.queue(cursor::Show).unwrap();
        self.flush();
    }
}
