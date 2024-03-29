use std::collections::HashMap;

use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::{Canvas, Texture};
use sdl2::video::Window;
use tracing::{debug, error, warn};

/// A Generic SDL window.
pub struct GenericWindow {
    /// All the canvases where we need to draw.
    pub canvas: Canvas<Window>,
    /// The textures related to the canvas.
    pub textures: HashMap<String, Texture>,
    /// The window id.
    pub id: u32,
}

impl GenericWindow {
    #[must_use]
    pub fn new(
        context: &sdl2::Sdl,
        resizable: bool,
        height: u32,
        width: u32,
        name: &str,
    ) -> Self {
        let video_subsystem = context
            .video()
            .expect("Unable to build the video subsystem?");
        video_subsystem
            .gl_load_library_default()
            .expect("unable to initialize opengl");

        // Create window, canvas
        let mut windowbuilder = video_subsystem.window(name, height, width);
        if resizable {
            windowbuilder.resizable();
        }
        let window = windowbuilder.build().expect("Unable to build the window");

        let canvas = window
            .into_canvas()
            .target_texture()
            .accelerated()
            .build().map_or({
                warn!(
                    "Unable to build an accelerated context, trying the plain one."
                );
                // If accelerated is does not work, try not accelerated one.
                windowbuilder = video_subsystem.window(name, height, width);
                if resizable {
                    windowbuilder.resizable();
                }
                let window =
                    windowbuilder.build().expect("Unable to build the window");

                window.into_canvas().target_texture().build().expect(
                    "Unable to build even the non-accelerated window...",
                )
            }, |c| c)
        ;

        let id = &canvas.window().id();
        Self {
            canvas,
            textures: HashMap::new(),
            id: *id,
        }
    }

    /// Clean the textures hashmap, by destroying them.
    pub fn remove_textures(&mut self) {
        // Remove the old textures
        for (_name, texture) in self.textures.drain() {
            // Safety: we are cleaning the map at the same time, so we won't be
            // able to find the textures anyway after this.
            unsafe { texture.destroy() };
        }
        self.textures.clear();
    }

    /// Add the texture that can be found at `texture_path`, and use that path
    /// as a key to retrieve it.
    pub fn add_texture<T>(&mut self, texture_path: &T)
    where
        T: AsRef<str>,
    {
        use sdl2::image::LoadTexture;

        // Put the textures in the map.
        let texture_creator = self.canvas.texture_creator();

        if !self.textures.contains_key(texture_path.as_ref()) {
            let res = texture_creator.load_texture(texture_path.as_ref());
            if let Ok(texture) = res {
                debug!("Loading {} into the hashmap.", texture_path.as_ref());
                self.textures
                    .insert(String::from(texture_path.as_ref()), texture);
            } else {
                error!(
                    "Error while loading to show: {}",
                    texture_path.as_ref()
                );
            }
        }
    }
}

#[must_use]
pub fn convert_point(win: &Window, x: f32, y: f32) -> (u32, u32) {
    let (sx, sy) = win.size();

    #[allow(clippy::cast_possible_truncation)]
    #[allow(clippy::cast_sign_loss)]
    let f32_max = f32::MAX.ceil() as u32;

    assert!(sx <= f32_max);
    assert!(sy <= f32_max);

    #[allow(clippy::cast_possible_truncation)]
    #[allow(clippy::cast_sign_loss)]
    #[allow(clippy::cast_precision_loss)]
    let px = (sx as f32 * x).floor() as u32;
    #[allow(clippy::cast_possible_truncation)]
    #[allow(clippy::cast_sign_loss)]
    #[allow(clippy::cast_precision_loss)]
    let py = (sy as f32 * y).floor() as u32;
    (px, py)
}

#[must_use]
pub fn get_scaled_rect(win: &Window, x: f32, y: f32, w: f32, h: f32) -> Rect {
    let (nx, ny) = convert_point(win, x, y);
    let (nw, nh) = convert_point(win, w, h);
    let (sx, sy) = win.size();

    assert!(nx < i32::MAX as u32);
    assert!(ny < i32::MAX as u32);

    #[allow(clippy::cast_possible_wrap)]
    let nxx = nx as i32;
    #[allow(clippy::cast_possible_wrap)]
    let nyy = ny as i32;

    let rect = Rect::new(nxx, nyy, nw, nh);

    if (nx + nw) > sx || (ny + nh) > sy {
        // Something will not fit in the image, show a log,
        // but still display the thing that lives on the screen area.
        warn!("Building rect outside of the visible area: {:?}", rect);
    }
    rect
}

/// Change the color of a canvas.
pub fn canvas_change_color(
    canvas: &mut sdl2::render::Canvas<sdl2::video::Window>,
    color: Color,
) {
    canvas.set_draw_color(color);
    canvas.clear();
}

impl From<crate::slideshow::Color> for Color {
    fn from(c: crate::slideshow::Color) -> Self {
        Self::from((c.r, c.g, c.b, c.a))
    }
}

#[allow(clippy::many_single_char_names)]
impl From<Color> for crate::slideshow::Color {
    fn from(c: Color) -> Self {
        let (r, g, b, a) = c.rgba();
        Self { r, g, b, a }
    }
}
