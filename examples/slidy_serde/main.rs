// Import external references
extern crate env_logger;
#[macro_use]
extern crate log;

use structopt::StructOpt;

use env_logger::{Builder, WriteStyle};

use log::LevelFilter;

// Sdl2 imports.
// @todo this can go in the lib so that this crate does not have dependencies
// to the SDL2 part.  This can be useful in case of a feature where we use the
// terminal to show the slides.

use sdl2::event::Event;
use sdl2::keyboard::Keycode;

// Std imports

use std::fs::File;
use std::str::FromStr;
use std::thread::sleep;
use std::time::Duration;

// Slidy imports.

use slidy::slideshow::Slideshow;
use slidy::windows::slideshow::SlideShowWindow;
use slidy::windows::timer::TimerWindow;
use slidy::windows::utils::{CanvasPresent, GetWinId};

/// Define the window options.
pub struct ScreenOptions {
    pub h: u32,
    pub w: u32,
    pub resizable: bool,
    pub fullscreen: bool,
}

#[derive(Debug, structopt::StructOpt)]
/// My Amazing Personal Slideshow command line options.
struct Args {
    #[structopt(short = "l", long = "log-level", default_value = "INFO")]
    /// The log level to be used.
    log_level: String,
    #[structopt(short = "w", long = "window-size", default_value = "800x600")]
    /// Window size, expressed as <h>x<w>.
    winsize: String,
    #[structopt(long = "fixed-size")]
    /// If set, the user can't resize the window, which will be stuck to window-size
    /// Note: looks like "down-resizing" is always possible...
    fixed_size: bool,
    #[structopt()]
    /// If set, the user can't resize the window, which will be stuck to window-size
    /// Note: looks like "down-resizing" is always possible...
    path_to_file: String,
}

pub fn read_slides(path: &str) -> Slideshow {
    let mut file = File::open(&path).unwrap();

    use std::io::Read;
    let mut input_str = String::new();
    file.read_to_string(&mut input_str).unwrap();
    serde_json::from_str(&input_str).unwrap()
}

#[doc(hidden)]
fn main() {
    let args = Args::from_args();

    let level = LevelFilter::from_str(&args.log_level)
        .expect("Please provide a valid log level.");

    // Init logger.
    let mut log_builder = Builder::new();
    log_builder
        .filter(None, level)
        .write_style(WriteStyle::Always)
        .init();

    info!("Using log level: {}", level);

    // Parse the window size value.
    let (h, w) = match args
        .winsize
        .split('x')
        .map(|e| {
            e.parse().unwrap_or_else(|_| {
                panic!("Unable to parse `{}` into u32", e)
            })
        })
        .collect::<Vec<u32>>().as_slice() {
            [a, b] => (*a, *b),
            _ => panic!(
                "Must provide 2 parameters for the winsize, found more than that :)",
            ),
        };

    let screen_options = ScreenOptions {
        h,
        w,
        fullscreen: false,
        resizable: !args.fixed_size,
    };

    // Init stuffs
    let sdl_context = slidy::get_sdl_context();
    let ttf_context = slidy::get_ttf_context();
    let free_mono = slidy::get_default_font(&ttf_context);

    // 1. The slideshow window
    let mut slideshow_win = SlideShowWindow::new(
        &sdl_context,
        &free_mono,
        screen_options.resizable,
        screen_options.h,
        screen_options.w,
    );

    // 2. The timer window
    // @todo <dp> create options for the size of this window as well?
    let mut timer_win = TimerWindow::new(
        &sdl_context,
        &free_mono,
        screen_options.resizable,
        screen_options.h / 5,
        screen_options.w / 5,
    );
    timer_win.visibility_toggle();

    // Set the slides.
    let slides = read_slides(&args.path_to_file);
    slideshow_win.set_slides(slides);

    let mut win_id: u32 = 0;

    let fixed_fps = Duration::from_nanos(1_000_000_000 / 10);
    let sec_as_nanos = Duration::from_secs(1).as_nanos();

    // Event loop
    let mut event_pump = sdl_context.event_pump().unwrap();
    'running: loop {
        let timer = std::time::SystemTime::now();
        // Check if we have new slides
        for event in event_pump.poll_iter() {
            // Check if one of the 2 windows is highlighted.
            if win_id
                == *slideshow_win.generic_win.get_win_ids().get(0).unwrap()
            {
                slideshow_win.manage_keypress(&event);
            } else if win_id
                == *timer_win.generic_win.get_win_ids().get(0).unwrap()
            {
                timer_win.manage_keypress(&event);
            }
            // Then, match events that should always occur, whatever window is
            // highlighted.
            match event {
                // Quit event, QUIT (I guess F4, C-c) or Q or ESC
                Event::Quit { .. }
                | Event::KeyUp {
                    keycode: Some(Keycode::Escape),
                    ..
                }
                | Event::KeyUp {
                    keycode: Some(Keycode::Q),
                    ..
                } => break 'running,
                // KeyUp: T
                Event::KeyUp {
                    keycode: Some(Keycode::T),
                    ..
                } => timer_win.visibility_toggle(),
                // KeyUp: S
                Event::KeyUp {
                    keycode: Some(Keycode::S),
                    ..
                } => slideshow_win.toggle_sideslide(),
                // Window Event: set the id of the window when focus is gained.
                Event::Window {
                    window_id,
                    win_event: sdl2::event::WindowEvent::FocusGained,
                    ..
                }
                | Event::MouseMotion { window_id, .. } => win_id = window_id,
                _ => slideshow_win.set_changed(true),
            }
        }

        // Update slideshow window
        if slideshow_win.is_changed() {
            slideshow_win.present_slide();
            slideshow_win.set_changed(false);
        }
        slideshow_win.generic_win.canvases_present();

        // Update timer window
        // timer_win.update_pseudo_random_position();
        let (slide_idx, slide_len) = slideshow_win.get_slides_counters();
        timer_win.update(slide_len, slide_idx + 1);
        timer_win.generic_win.canvases_present();

        let elapsed = timer.elapsed().unwrap();
        trace!("max fps: {:?}", sec_as_nanos / elapsed.as_nanos());
        if elapsed < fixed_fps {
            let sleeptime = fixed_fps - elapsed;
            trace!("Sleeping for {:?}", sleeptime);
            // Fix framerate to 10 fps
            sleep(sleeptime);
        } else {
            warn!(
                "Unable to have 10 fps, needed {:?} to show the slide",
                elapsed
            );
        }
    }
}
