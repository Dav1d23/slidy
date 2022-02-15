// Import external references
extern crate env_logger;
#[macro_use]
extern crate log;

// Std imports.

use std::fs::canonicalize;
use std::path::Path;
use std::str::FromStr;
use std::sync::mpsc::channel;
use std::thread;
use std::thread::sleep;
use std::time::Duration;

// Imports.

use env_logger::{Builder, WriteStyle};
use log::LevelFilter;
use notify::{raw_watcher, RecursiveMode, Watcher};
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use structopt::StructOpt;

// Slidy imports.

use slidy::backend_sdl::slideshow::SlideShowWindow;
use slidy::backend_sdl::timer::TimerWindow;

// Local modules.

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
    #[structopt(required = true)]
    /// The path to the slides to be shown.
    slide_path: String,
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

    let path = canonicalize(Path::new(&args.slide_path)).unwrap_or_else(|e| {
        panic!("`{}` is not a valid path: {}", &args.slide_path, e)
    });
    info!("Using file {}", &path.display());

    // Prepare the 3 channels to be used.
    // 1. Send slides from parser to graphical loop.
    let (send_slides_tx, send_slides_rx) = channel();
    // 2. Ask the parser to create new slides.
    let (request_update_tx, request_update_rx) = channel();
    // 3. Notify a change in the input file.
    let (watcher_tx, watcher_rx) = channel();
    let mut watcher =
        raw_watcher(watcher_tx).expect("Unable to create the raw watcher");
    watcher
        .watch(&path, RecursiveMode::NonRecursive)
        .unwrap_or_else(|e| panic!("Unable to watch {:?}: {}", &path, e));

    // Let's start the threads now.
    // The first one is related to the slider. Whenever a request to request_update is sent,
    // the parser will parse the file and respond with the prepared array in the send_slides
    // channel.
    // This thread does something only when there is a new request to update
    // the slides (mind that the only component that can send the requests is
    // the inotify component described below.
    // The recv call is blocking, which means that this thread sleeps until
    // inotify wakes it up.
    thread::spawn(move || {
        loop {
            if request_update_rx.recv().is_ok() {
                // If we can't parse or send the slides, just print the reason,
                // and then loop again waiting for a new request.

                let slides = slidy::parser::parse_file(&path);
                match slides {
                    Err(e) => error!("Error when parsing {:?}: {}", &path, e),
                    Ok(slides) => {
                        if let Err(e) = send_slides_tx.send(slides) {
                            error!("Error when sending the slides: {}", e)
                        }
                    }
                };
            }
        }
    });

    // This second thread is way less useful: we just use it to listed for the watcher thread
    // and transfer the message to  the parser using the single request_update channel.
    // This thread will basically sleep all the time (the recv() call is blocking) until there
    // is a change in the slide file.
    let request_update_tx_watcher = request_update_tx.clone();
    thread::spawn(move || loop {
        if watcher_rx.recv().is_ok() {
            request_update_tx_watcher
                .send(())
                .expect("Unable request a slide's update")
        }
    });

    // ... And finally the main graphical loop. This will receive new slides in the send_slides
    // channel and show them whenever they are ready. Moreover, it can send update requests to
    // to the request_update channel - even if this is only useful on initialization to tell
    // the slider "hey give me the slides you have since I'm pretty new here ;)
    // It could also have been done with a reference to the slider, maybe, but this looks nicer
    // since I want the slider to live on another thread.

    // Init stuffs
    let sdl_context = slidy::backend_sdl::get_sdl_context();
    let ttf_context = slidy::backend_sdl::get_ttf_context();
    let free_mono = slidy::backend_sdl::get_default_font(&ttf_context);

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

    // Get the windows ids.
    let main_slide_id = slideshow_win.main_win.id;
    let side_slide_id = slideshow_win.side_win.id;
    let timer_id = timer_win.generic_win.id;

    // Request slides
    request_update_tx
        .send(())
        .expect("Unable to request slide update");
    let mut win_id: u32 = 0;

    let fixed_fps = Duration::from_nanos(1_000_000_000 / 10);

    // Event loop
    let mut event_pump = sdl_context
        .event_pump()
        .expect("Unable to get the event pump");

    'running: loop {
        let timer = std::time::SystemTime::now();
        // Check if we have new slides
        if let Ok(slides) = send_slides_rx.try_recv() {
            slideshow_win.set_slides(slides)
        };
        for event in event_pump.poll_iter() {
            match win_id {
                x if x == main_slide_id => {
                    slideshow_win.manage_keypress(&event)
                }
                x if x == side_slide_id => {
                    slideshow_win.manage_keypress(&event)
                }
                x if x == timer_id => timer_win.manage_keypress(&event),
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
                    x if x == main_slide_id => break 'running,
                    x if x == side_slide_id => slideshow_win.toggle_sideslide(),
                    x if x == timer_id => timer_win.visibility_toggle(),
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
                _ => slideshow_win.is_changed = true,
            }
        }

        // Update slideshow window
        if slideshow_win.is_changed {
            slideshow_win.present_slide();
            slideshow_win.is_changed = false;
        }

        // Update timer window
        // timer_win.update_pseudo_random_position();
        let (slide_idx, slide_len) = slideshow_win.get_slides_counters();
        timer_win.update(slide_len, slide_idx + 1);

        slideshow_win.main_win.canvas.present();
        slideshow_win.side_win.canvas.present();
        timer_win.generic_win.canvas.present();

        match timer.elapsed() {
            Ok(elapsed) => {
                if elapsed < fixed_fps {
                    let sleeptime = fixed_fps - elapsed;
                    // Fix framerate to 10 fps
                    sleep(sleeptime);
                } else {
                    warn!(
                        "Unable to have 10 fps, needed {:?} to show the slide",
                        elapsed
                    );
                }
            }
            Err(e) => {
                error!("Previous measured time is later than actual one: {}", e)
            }
        }
    }
}
