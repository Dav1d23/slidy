//! The provided SDL2 backend.

use self::{slideshow::SlideShowWindow, timer::TimerWindow};

use sdl2::event::Event;
use sdl2::keyboard::Keycode;

pub mod slideshow;
pub mod timer;
pub mod utils;

/// Get the default, included font. It is the FreeMono one, and it is included
/// in the binary, so no need to provide any other file.
pub fn get_default_font<'ttf>(
    context: &'ttf sdl2::ttf::Sdl2TtfContext,
) -> sdl2::ttf::Font<'ttf, '_> {
    // TODO The font should be read from the slide directly
    //      and _then_ if nothing is provided use the default one.
    let fontbytes = include_bytes!("../../../assets/FreeMono.ttf");
    let mut points = 100;
    loop {
        let rwfont = sdl2::rwops::RWops::from_bytes(fontbytes)
            .expect("Font file has been moved");
        if let Ok(font) = context.load_font_from_rwops(rwfont, points) {
            return font;
        }
        points -= 10;
        if points < 10 {
            panic!("This is not enough to show the font...");
        }
    }
}

/// Helper: init the SDL context.
pub fn get_sdl_context() -> sdl2::Sdl {
    // Init stuffs.
    let sdl_context = sdl2::init().expect("Unable to init sdl.");
    // This is unused, but needs to stay in scope to be able to use the SDL_image.
    let _image_context = sdl2::image::init(sdl2::image::InitFlag::JPG)
        .expect("Unable to init image.");
    sdl_context
}

/// Helper: init the TTF context.
pub fn get_ttf_context() -> sdl2::ttf::Sdl2TtfContext {
    sdl2::ttf::init().expect("Unable to init ttf.")
}

/// Define the window options.
pub struct WindowOptions {
    pub h: u32,
    pub w: u32,
    pub resizable: bool,
    pub fullscreen: bool,
}

impl Default for WindowOptions {
    fn default() -> Self {
        WindowOptions {
            h: 800,
            w: 600,
            resizable: true,
            fullscreen: false,
        }
    }
}

/// The backend. Stores all the SDL internals.
/// This structure needs to created only once, and is used to get the live
/// context.
pub struct Backend {
    pub sdl_context: sdl2::Sdl,
    pub ttf_context: sdl2::ttf::Sdl2TtfContext,
}

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
    pub slideshow_win: SlideShowWindow<'backend>,
    pub timer_win: TimerWindow<'backend>,

    pub active_win_id: u32,
    pub main_slide_id: u32,
    pub side_slide_id: u32,
    pub timer_id: u32,

    pub event_pump: sdl2::EventPump,
}

impl Backend {
    /// Create a new backend.
    pub fn new() -> Backend {
        let sdl_context = get_sdl_context();
        let ttf_context = get_ttf_context();

        Backend {
            sdl_context,
            ttf_context,
        }
    }

    /// Get the runnable context.
    /// @TODO manage windows options.
    fn internal_get_context(&self) -> Context {
        let screen_options = WindowOptions::default();

        // 1. The slideshow window
        let slideshow_win = SlideShowWindow::new(
            &self.sdl_context,
            get_default_font(&self.ttf_context),
            screen_options.resizable,
            screen_options.h,
            screen_options.w,
        );

        // 2. The timer window
        // @todo <dp> create options for the size of this window as well?
        let mut timer_win = TimerWindow::new(
            &self.sdl_context,
            get_default_font(&self.ttf_context),
            screen_options.resizable,
            screen_options.h / 5,
            screen_options.w / 5,
        );
        timer_win.visibility_toggle();

        // Get the windows ids.
        let main_slide_id = slideshow_win.main_win.id;
        let side_slide_id = slideshow_win.side_win.id;
        let timer_id = timer_win.generic_win.id;

        // Create the event pump.
        let event_pump = self
            .sdl_context
            .event_pump()
            .expect("Unable to get the event pump, another one is alive?");

        Context {
            slideshow_win,
            timer_win,
            active_win_id: 0,
            main_slide_id,
            side_slide_id,
            timer_id,
            event_pump,
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
        self.slideshow_win.set_slides(slides);
    }

    /// Manage the incoming events.
    fn manage_inputs(&mut self) -> super::ShouldQuit {
        for event in self.event_pump.poll_iter() {
            match self.active_win_id {
                x if x == self.main_slide_id => {
                    self.slideshow_win.manage_keypress(&event)
                }
                x if x == self.side_slide_id => {
                    self.slideshow_win.manage_keypress(&event)
                }
                x if x == self.timer_id => {
                    self.timer_win.manage_keypress(&event)
                }
                _ => {}
            }
            // Then, match events that should always occur, whatever window is
            // highlighted.
            match event {
                // If we click on "close" on the window itself.
                Event::Window {
                    window_id,
                    win_event: sdl2::event::WindowEvent::Close,
                    ..
                } => match window_id {
                    x if x == self.main_slide_id => return true,
                    x if x == self.side_slide_id => {
                        self.slideshow_win.toggle_sideslide()
                    }
                    x if x == self.timer_id => {
                        self.timer_win.visibility_toggle()
                    }
                    _ => {}
                },
                // Quit event, QUIT (I guess F4, C-c) or Q or ESC
                Event::Quit { .. }
                | Event::KeyUp {
                    keycode: Some(Keycode::Escape),
                    ..
                }
                | Event::KeyUp {
                    keycode: Some(Keycode::Q),
                    ..
                } => return true,
                // KeyUp: T
                Event::KeyUp {
                    keycode: Some(Keycode::T),
                    ..
                } => self.timer_win.visibility_toggle(),
                // KeyUp: S
                Event::KeyUp {
                    keycode: Some(Keycode::S),
                    ..
                } => self.slideshow_win.toggle_sideslide(),
                // Window Event: set the id of the window when focus is gained.
                Event::Window {
                    window_id,
                    win_event: sdl2::event::WindowEvent::FocusGained,
                    ..
                }
                | Event::MouseMotion { window_id, .. } => {
                    // Store window that last gained focus.
                    self.active_win_id = window_id;
                }
                _ => self.slideshow_win.is_changed = true,
            }
        }
        false
    }

    /// Render the windows.
    fn render(&mut self) {
        // Update slideshow window
        if self.slideshow_win.is_changed {
            self.slideshow_win.present_slide();
            self.slideshow_win.is_changed = false;
        }

        // Update timer window
        // self.timer_win.update_pseudo_random_position();
        let (slide_idx, slide_len) = self.slideshow_win.get_slides_counters();
        self.timer_win.update(slide_len, slide_idx + 1);

        self.slideshow_win.main_win.canvas.present();
        self.slideshow_win.side_win.canvas.present();
        self.timer_win.generic_win.canvas.present();
    }
}
