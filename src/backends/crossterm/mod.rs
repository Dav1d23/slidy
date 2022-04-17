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
    #[must_use]
    /// Create a new backend.
    pub fn new() -> Self {
        debug!("Enable raw-mode.");
        terminal::enable_raw_mode()
            .expect("Raw mode is needed for input management.");
        Self {}
    }

    /// Get the runnable context.
    fn internal_get_context(&self) -> Context {
        let _ = self;
        let mut stdout = stdout();
        stdout
            .queue(cursor::Hide)
            .expect("Unable to hide the cursor?");
        stdout.flush().expect("Unable to flush?");

        Context {
            slide_id: 0,
            slides: Slideshow::default(),
            _lifetime: PhantomData,
            stdout,
            slides_changed: true,
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
    slides_changed: bool,
}

impl<'b> super::SlidyContext for Context<'b> {
    fn set_slides(&mut self, slides: crate::slideshow::Slideshow) {
        self.slides = slides;
        self.slides_changed = true;
    }

    /// Manage the incoming events.
    fn manage_inputs(&mut self) -> super::ShouldQuit {
        while let Ok(true) = poll(Duration::ZERO) {
            let evt = read().expect("Poll told us this should work.");
            trace!("{:#?}", evt);
            match evt {
                Event::Resize(..) => {
                    self.slides_changed = true;
                }
                Event::Key(KeyEvent {
                    code: KeyCode::Char('q'),
                    ..
                }) => return true,
                Event::Key(KeyEvent {
                    code: KeyCode::Char('n'),
                    ..
                }) => {
                    self.slide_id =
                        (self.slide_id + 1).min(self.slides.slides.len() - 1);
                    self.slides_changed = true;
                }
                Event::Key(KeyEvent {
                    code: KeyCode::Char('p'),
                    ..
                }) => {
                    self.slide_id = if self.slide_id > 0 {
                        self.slide_id - 1
                    } else {
                        0
                    };
                    self.slides_changed = true;
                }
                _ => {}
            }
        }
        false
    }

    /// Render the windows.
    fn render(&mut self) {
        if self.slides_changed {
            trace!("Rendering phase");
            self.clear_all();
            let term_size = match terminal::size() {
                Ok(v) => v,
                Err(e) => {
                    error!("Unable to get the terminal size, using a default one: {}", e);
                    (30, 20)
                }
            };
            debug!("Considering slide {}", self.slide_id);

            if let Some(slide) = self.slides.slides.get(self.slide_id) {
                for sec in &slide.sections {
                    // @TODO why is position 0. 0. if it is not there?
                    let pos = sec
                        .position
                        .as_ref()
                        .unwrap_or(&Position { x: 0.01, y: 0.01 });
                    let x = (f32::from(term_size.0) * pos.x).ceil();
                    assert!(0.0 <= x && x <= u16::MAX.into());

                    #[allow(clippy::cast_possible_truncation)]
                    #[allow(clippy::cast_sign_loss)]
                    let x: u16 = x as u16;
                    let y = (f32::from(term_size.1) * pos.y).ceil();
                    assert!(0.0 <= y && y <= u16::MAX.into());

                    #[allow(clippy::cast_possible_truncation)]
                    #[allow(clippy::cast_sign_loss)]
                    let mut y: u16 = y as u16;
                    if let Some(SectionMain::Text(sec_text)) = &sec.sec_main {
                        for chunk in sec_text.text.as_str().split('\n') {
                            debug!("Writing {chunk} to [{x}, {y}]");
                            self.stdout
                                .queue(cursor::MoveTo(x, y))
                                .expect("Unable to move the cursor?");
                            // I should use the "style" defined in the slides instead of this one.
                            let styled = chunk.with(Color::White);
                            self.stdout
                                .queue(PrintStyledContent(styled))
                                .expect("Unable to write on the terminal?");
                            y += 1;
                        }
                    }
                }
            } else {
                warn!("There are no slides to show!");
            }
            self.flush();
        }
        self.slides_changed = false;
    }
}

impl Context<'_> {
    fn clear_all(&mut self) {
        self.stdout
            .queue(terminal::Clear(terminal::ClearType::All))
            .expect("Unable to clear the screen?");
    }

    fn flush(&mut self) {
        self.stdout.flush().expect("Unable to flush?");
    }
}

impl Drop for Context<'_> {
    fn drop(&mut self) {
        self.stdout
            .queue(cursor::Show)
            .expect("Unable to show the cursor back?");
        self.flush();
    }
}
