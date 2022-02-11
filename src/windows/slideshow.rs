use std::collections::HashMap;

use sdl2::event::Event;
use sdl2::image::LoadTexture;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color as sdl_color;
use sdl2::render::Canvas;
use sdl2::render::Texture;
use sdl2::video::Window;

use super::{utils, utils::GenericWindow};
use crate::slideshow;

pub struct SlideShowWindow<'a> {
    /// Contains the generic information for a window
    pub generic_win: GenericWindow,
    /// The actual slide being shown.
    idx: usize,
    /// If the slide has to be drawn again.
    is_changed: bool,
    /// All the slides in the slideshow.
    slides: slideshow::Slideshow,
    // If the side slideshow should be visible.
    is_visible: bool,
    // Internal structure to hold the textures in order not to load them over
    // and over.
    textures: Vec<HashMap<String, Texture>>,
    /// The default font to be used.
    default_font: &'a sdl2::ttf::Font<'a, 'a>,
}

impl<'a> SlideShowWindow<'a> {
    pub fn new(
        context: &sdl2::Sdl,
        font: &'a sdl2::ttf::Font,
        resizable: bool,
        h: u32,
        w: u32,
    ) -> Self {
        // The main canvas for the main slides
        let canvas = utils::get_canvas(context, resizable, h, w, "SlideShow");
        // The next slide
        let mut canvas_next = utils::get_canvas(
            context,
            resizable,
            h,
            w,
            "SlideShow: next slide",
        );
        canvas_next.window_mut().hide();

        let slides = slideshow::Slideshow::default();
        SlideShowWindow {
            generic_win: GenericWindow {
                canvases: vec![canvas, canvas_next],
            },
            idx: 0,
            is_changed: true,
            slides,
            textures: vec![HashMap::new(), HashMap::new()],
            default_font: font,
            is_visible: false,
        }
    }

    pub fn is_changed(&self) -> bool {
        self.is_changed
    }

    pub fn set_changed(&mut self, how: bool) {
        self.is_changed = how;
    }

    /// Toggle visibility
    pub fn toggle_sideslide(&mut self) {
        // The side slide is with index 1.
        let c = self.generic_win.canvases.get_mut(1).unwrap();
        if self.is_visible {
            c.window_mut().hide();
        } else {
            c.window_mut().show();
        }
        self.is_visible = !self.is_visible;
    }

    pub fn get_slides_counters(&self) -> (usize, usize) {
        (self.idx, self.slides.slides.len())
    }

    pub fn set_slide(&mut self, idx: usize) {
        if self.idx >= self.slides.slides.len() {
            // Panic, this idx is not correct.
            panic!("Can't set slide {}/{}", idx, self.slides.slides.len());
        }
        self.idx = idx;
        self.is_changed = true;
    }

    pub fn next_slide(&mut self) {
        if self.idx < self.slides.slides.len() - 1 {
            self.idx += 1;
            self.is_changed = true;
        }
    }

    pub fn prev_slide(&mut self) {
        if self.idx > 0 {
            self.idx -= 1;
            self.is_changed = true;
        }
    }

    /// Manage the keypresses, or any other even related to this very
    /// window. We don't want other elements to manage our keys!
    pub fn manage_keypress(&mut self, event: &Event) {
        match event {
            // KeyUp: N
            Event::KeyUp {
                keycode: Some(Keycode::N),
                ..
            } => self.next_slide(),
            // KeyUp: P
            Event::KeyUp {
                keycode: Some(Keycode::P),
                ..
            } => self.prev_slide(),
            _ => {}
        }
    }

    /// If we remove some slide, we might have the index pointing in a location
    /// that does not exists anymore. This would be bad, and thus we simply
    /// loop until we find the first good index.  Note that if we add a slide,
    /// we can't really know if we add a slide before of after. Imagine the
    /// case where we add the slide in position 3 and we are showing slide in
    /// position 3 already: we will just show the new slide.  @TODO is there a
    /// better way to do it?
    fn set_first_good_slide(&mut self) {
        while self.idx > self.slides.slides.len() - 1 {
            self.prev_slide();
        }
    }

    /// This function sets the slides for the slideshow. Also, it preload the
    /// textures being used so there is no need to load them multiple
    /// times. This means that this function may take some time.
    /// @TODO I can side-load the slides and the texture and then atomically
    /// switch, it is not probably worth the effort... But what does it here?
    pub fn set_slides(&mut self, slides: slideshow::Slideshow) {
        self.slides = slides;
        self.preload_textures();
        self.set_first_good_slide();
        self.is_changed = true;
    }

    fn preload_textures(&mut self) {
        let keys: Vec<String> = self
            .textures
            .get(0)
            .unwrap()
            .iter()
            .map(|(k, _)| String::from(k))
            .collect();

        // Remove the old textures from the 2 old maps.
        for ref mut texture_holder in self.textures.iter_mut() {
            for el in &keys {
                let elem = texture_holder.remove(el).unwrap();
                // @safety This is ok. These textures will never be used
                // again, we can safely remove them.
                // @todo why is this not used again?
                unsafe {
                    elem.destroy();
                }
            }
            texture_holder.clear();
        }

        // Add all the new textures to all the canvases.
        for (canvas, texture_holder) in self
            .generic_win
            .canvases
            .iter_mut()
            .zip(self.textures.iter_mut())
        {
            let texture_creator = canvas.texture_creator();
            for elem in self.slides.slides.iter() {
                for sec in elem.sections.iter() {
                    if let Some(slideshow::SectionMain::Figure(fig)) =
                        &sec.sec_main
                    {
                        if !texture_holder.contains_key(&fig.path) {
                            let res = texture_creator.load_texture(&fig.path);
                            if let Ok(texture) = res {
                                debug!(
                                    "Loading {} into the hashmap.",
                                    &fig.path
                                );
                                texture_holder
                                    .insert(String::from(&fig.path), texture);
                            } else {
                                error!(
                                    "Error while loading to show: {}",
                                    fig.path
                                );
                            }
                        }
                    }
                }
            }
        }
    }

    /// Main method to show a slide on the screen.
    pub fn present_slide(&mut self) {
        if self.slides.slides.is_empty() {
            // Nothing is given, get some "default" slide to show.
            self.slides.slides.push(slideshow::Slide::default())
        }
        self.set_first_good_slide();
        // prepare the rects where to write the text
        // this is a loop over all the "sections" of a slide.
        // We technically "could" store the positions in order not to
        // recompute everything each time, but... Is it worth it? :)
        let bg_col = match self.slides.bg_col {
            Some(c) => c,
            None => sdl_color::WHITE.into(),
        };
        let font_col = match self.slides.font_col {
            Some(c) => c,
            None => sdl_color::BLACK.into(),
        };
        let font_size = match &self.slides.font_size {
            Some(f) => (f.x, f.y),
            None => (0.018, 0.08),
        };

        // First slide window.
        let mut base_height: f32 = 0.01;
        let col = match self.slides.slides[self.idx].bg_color {
            Some(c) => c.into(),
            None => bg_col.into(),
        };
        {
            utils::canvas_change_color(
                self.generic_win.canvases.get_mut(0).unwrap(),
                col,
            );

            let canvas = self.generic_win.canvases.get_mut(0).unwrap();

            let textures = self.textures.get(0).unwrap();

            for elem in self.slides.slides[self.idx].sections.iter() {
                draw_single_section(
                    canvas,
                    textures,
                    elem,
                    &mut base_height,
                    self.default_font,
                    font_size,
                    font_col,
                );
            }
        }

        // Second slide window.
        let mut base_height: f32 = 0.01;
        let next_idx = if self.idx < self.slides.slides.len() - 1 {
            self.idx + 1
        } else {
            self.idx
        };
        let col = match self.slides.slides[next_idx].bg_color {
            Some(c) => c,
            None => bg_col,
        };
        {
            let canvas = self.generic_win.canvases.get_mut(1).unwrap();

            utils::canvas_change_color(canvas, col.into());

            let textures = self.textures.get(0).unwrap();

            for elem in self.slides.slides[next_idx].sections.iter() {
                draw_single_section(
                    canvas,
                    textures,
                    elem,
                    &mut base_height,
                    self.default_font,
                    font_size,
                    font_col,
                );
            }
        }
    }
}

fn draw_single_section<'a>(
    canvas: &mut Canvas<Window>,
    textures: &HashMap<String, Texture>,
    elem: &slideshow::Section,
    base_height: &mut f32,
    default_font: &sdl2::ttf::Font<'a, 'a>,
    font_size: (f32, f32),
    font_col: slideshow::Color,
) {
    if let Some(sec_main) = &elem.sec_main {
        match sec_main {
            // Manage pictures
            slideshow::SectionMain::Figure(fig) => {
                {
                    let res = textures.get(&fig.path);
                    if let Some(texture) = res {
                        // if we have a path, the section cannot contain anything else
                        let (x_start, y_start) = match &elem.position {
                            Some(p) => (p.x, p.y),
                            None => (0.01, 0.01),
                        };
                        let (x_size, y_size) = match &elem.size {
                            Some(p) => (p.x, p.y),
                            None => (0.1, 0.1),
                        };
                        let rect = utils::get_scaled_rect(
                            canvas.window(),
                            x_start,
                            y_start,
                            x_size,
                            y_size,
                        );
                        canvas
                            .copy_ex(
                                texture,
                                None,
                                rect,
                                fig.rotation.into(),
                                None,
                                false,
                                false,
                            )
                            .unwrap();
                    } else {
                        error!("Texture at {} was not ready", fig.path);
                    }
                }
            }
            // Manage text
            slideshow::SectionMain::Text(slideshow::SectionText {
                text,
                color,
                font: _new_font,
            }) => {
                let text_slice = text.as_str();
                for (idx, chunk) in text_slice.split('\n').enumerate() {
                    if chunk.is_empty() {
                        continue;
                    }
                    // Get the default size for each letter.
                    let (x_size, y_size) = match &elem.size {
                        Some(p) => (p.x, p.y),
                        None => font_size,
                    };
                    let (x_start, y_start) = match &elem.position {
                        // Each line starts 0.1 lower than the size
                        Some(p) => (p.x, p.y + y_size * idx as f32),
                        // If we don't have any default, starts from base_height
                        // and 0.01
                        None => (0.01, *base_height),
                    };
                    // Update base_height so what next run we already are
                    // down this much and we won't overwrite new text.
                    *base_height += y_size;
                    // The chunk size is the whole line.
                    // We build a single rect that contains the whole line.
                    let chunk_size: f32 = chunk.len() as f32 * x_size;
                    let rect = utils::get_scaled_rect(
                        canvas.window(),
                        x_start,
                        y_start,
                        chunk_size,
                        y_size,
                    );
                    //let rect = Rect::new(x_start, y_start, chunk_size, 0.01);
                    let surface_text = default_font
                        .render(chunk)
                        .solid(match color {
                            Some(c) => *c,
                            None => font_col,
                        })
                        .unwrap();
                    let texture_creator = canvas.texture_creator();
                    let texture =
                        surface_text.as_texture(&texture_creator).unwrap();
                    canvas.copy(&texture, None, rect).unwrap();
                    // @safety This is ok, since the texture has been copied to the canvas and we can
                    // safely remove the one in here.
                    unsafe {
                        texture.destroy();
                    }
                }
            }
        }
    }
}
