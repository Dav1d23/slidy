use std::fs::canonicalize;
use std::path::Path;
use std::str::FromStr;
use std::sync::mpsc::channel;
use std::thread;
use std::thread::sleep;
use std::time::Duration;

use notify::{RecursiveMode, Watcher};
use structopt::StructOpt;
use tracing::{error, info, level_filters, warn};

#[derive(Debug, structopt::StructOpt)]
/// My Amazing Personal Slideshow command line options.
struct Args {
    #[structopt(required = true)]
    /// The path to the slides to be shown.
    slide_path: String,
    #[structopt(short = "l", long = "log-level", default_value = "INFO")]
    /// The log level to be used.
    log_level: String,
    #[structopt(short = "b", long = "backend")]
    /// The log level to be used.
    backend: Option<String>,
}

#[doc(hidden)]
fn main() {
    let args = Args::from_args();

    let filter = level_filters::LevelFilter::from_str(&args.log_level)
        .expect("Please provide a valid log level.");

    // Init logger.
    let file_appender = tracing_appender::rolling::hourly("/tmp/", "slidy.log");
    let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);
    tracing_subscriber::fmt()
        .with_max_level(filter)
        .with_writer(non_blocking)
        .init();

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
    let mut watcher = notify::recommended_watcher(watcher_tx)
        .expect("Unable to create the watcher");
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

    // Request slides
    request_update_tx
        .send(())
        .expect("Unable to request slide update");

    // ... And finally the main graphical loop. This will receive new slides in the send_slides
    // channel and show them whenever they are ready. Moreover, it can send update requests to
    // to the request_update channel - even if this is only useful on initialization to tell
    // the slider "hey give me the slides you have since I'm pretty new here ;)
    // It could also have been done with a reference to the slider, maybe, but this looks nicer
    // since I want the slider to live on another thread.

    // @TODO change this based on the feature? Add a method in backend?
    let preferred_backend = "sdl";
    // Init backend and context.
    let backend: slidy::backends::Backends = match args.backend {
        Some(v) => v.try_into().unwrap(),
        None => preferred_backend.try_into().unwrap(),
    };

    let mut backend = slidy::backends::get_backend(&backend);
    let mut context = backend.get_context();

    // Fix the max fps.
    let fixed_fps = Duration::from_nanos(1_000_000_000 / 10);

    // The event loop.
    'running: loop {
        let timer = std::time::SystemTime::now();
        // Check if we have new slides
        if let Ok(slides) = send_slides_rx.try_recv() {
            context.set_slides(slides)
        };

        if context.manage_inputs() {
            break 'running;
        }
        context.render();

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
