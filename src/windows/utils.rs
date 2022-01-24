use std::cell::RefCell;

use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::Canvas;
use sdl2::video::Window;

pub struct GenericWindow {
    /// All the canvases where we need to draw.
    pub canvases: Vec<RefCell<Canvas<Window>>>,
}

pub trait GetWinId {
    fn get_win_ids(&self) -> Vec<u32>;
}

pub trait CanvasPresent {
    fn canvases_present(&mut self) {}
}

impl GetWinId for GenericWindow {
    fn get_win_ids(&self) -> Vec<u32> {
        self.canvases
            .iter()
            .map(|c| c.borrow().window().id())
            .collect::<Vec<u32>>()
    }
}

impl CanvasPresent for GenericWindow {
    fn canvases_present(&mut self) {
        self.canvases.iter().for_each(|c| c.borrow_mut().present());
    }
}

pub fn convert_point(win: &Window, x: f32, y: f32) -> (u32, u32) {
    let (sx, sy) = win.size();
    let px = (sx as f32 * x).floor() as u32;
    let py = (sy as f32 * y).floor() as u32;
    (px, py)
}

pub fn get_scaled_rect(win: &Window, x: f32, y: f32, w: f32, h: f32) -> Rect {
    let (nx, ny) = convert_point(win, x, y);
    let (nw, nh) = convert_point(win, w, h);
    let (sx, sy) = win.size();
    let rect = Rect::new(nx as i32, ny as i32, nw as u32, nh as u32);

    if (nx + nw) > sx || (ny + nh) > sy {
        // Something will not fit in the image, show an log,
        // but still display things.
        warn!("Building rect outside of the visible area: {:?}", rect);
    }
    rect
}

/// Get a canvas out of an sdl context.
/// Note that to extract the canvas, we build the window (if accelerated is not
/// possible, we build the non-accelerated one) and then we get the canvas out
/// of it.
/// @todo <dp> instead of passing the canvas around, it should and could be
/// possible to give the window itself to the "window" manager and let it do
/// what it wants.
pub fn get_canvas(
    context: &sdl2::Sdl,
    resizable: bool,
    height: u32,
    width: u32,
    name: &str,
) -> sdl2::render::Canvas<sdl2::video::Window> {
    let video_subsystem = context.video().unwrap();
    video_subsystem
        .gl_load_library_default()
        .expect("unable to initialize opengl");

    // Create window, canvas
    let mut windowbuilder = video_subsystem.window(name, height, width);
    if resizable {
        windowbuilder.resizable();
    }
    let window = windowbuilder.build().unwrap();

    match window.into_canvas().target_texture().accelerated().build() {
        Ok(res) => return res,
        Err(_) => warn!(
            "Unable to build an accelerated context, trying the plain one."
        ),
    }

    // If accelerated is does not work, try not accelerated one.
    windowbuilder = video_subsystem.window(name, height, width);
    if resizable {
        windowbuilder.resizable();
    }
    let window = windowbuilder.build().unwrap();

    window
        .into_canvas()
        .target_texture()
        .build()
        .expect("Unable to build even the non-accelerated window...")
}

/// Change the color of a canvas.
pub fn canvas_change_color(
    canvas: &mut sdl2::render::Canvas<sdl2::video::Window>,
    color: Color,
) {
    canvas.set_draw_color(color);
    canvas.clear();
}
