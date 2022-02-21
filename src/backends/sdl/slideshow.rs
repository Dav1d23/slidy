//! Window used to show the slides.
use log::error;

use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color as sdl_color;

use super::{utils, utils::GenericWindow};
use crate::slideshow;

/// The window holding the slideshow.
pub struct SlideShowWindow<'a> {
    /// Contains the generic information for a window
    pub main_win: GenericWindow,
    pub side_win: GenericWindow,
    /// The actual slide being shown.
    idx: usize,
    /// If the slide has to be drawn again.
    pub is_changed: bool,
    /// All the slides in the slideshow.
    slides: slideshow::Slideshow,
    // If the side slideshow should be visible.
    pub side_win_is_visible: bool,
    // Internal structure to hold the textures in order not to load them over
    // and over.
    /// The default font to be used.
    default_font: sdl2::ttf::Font<'a, 'a>,
}

impl<'a> SlideShowWindow<'a> {
    pub fn new(
        context: &sdl2::Sdl,
        font: sdl2::ttf::Font<'a, 'a>,
        resizable: bool,
        h: u32,
        w: u32,
    ) -> Self {
        let main_win =
            GenericWindow::new(context, resizable, h, w, "Slideshow");
        let mut side_win = GenericWindow::new(
            context,
            resizable,
            h,
            w,
            "Slideshow: next slide",
        );
        side_win.canvas.window_mut().hide();

        let slides = slideshow::Slideshow::default();
        SlideShowWindow {
            main_win,
            side_win,
            idx: 0,
            is_changed: true,
            slides,
            default_font: font,
            side_win_is_visible: false,
        }
    }

    /// Toggle visibility
    pub fn toggle_sideslide(&mut self) {
        let c = &mut self.side_win.canvas;
        if self.side_win_is_visible {
            c.window_mut().hide();
        } else {
            c.window_mut().show();
        }
        self.side_win_is_visible = !self.side_win_is_visible;
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
        self.main_win.remove_textures();
        self.side_win.remove_textures();

        for elem in self.slides.slides.iter() {
            for sec in elem.sections.iter() {
                if let Some(slideshow::SectionMain::Figure(fig)) = &sec.sec_main
                {
                    self.main_win.add_texture(&fig.path);
                    self.side_win.add_texture(&fig.path);
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
        let bg_col = self
            .slides
            .bg_col
            .unwrap_or_else(|| sdl_color::WHITE.into());
        let font_col = self
            .slides
            .font_col
            .unwrap_or_else(|| sdl_color::BLACK.into());
        let font_size = self
            .slides
            .font_size
            .as_ref()
            .map(|r| (r.w, r.h))
            .unwrap_or((0.018, 0.08));

        // First slide window.
        draw_sections(
            self.idx,
            &mut self.slides.slides,
            bg_col,
            &mut self.main_win,
            font_size,
            font_col,
            &self.default_font,
        );

        // Second slide window.
        let next_idx = if self.idx < self.slides.slides.len() - 1 {
            self.idx + 1
        } else {
            self.idx
        };
        draw_sections(
            next_idx,
            &mut self.slides.slides,
            bg_col,
            &mut self.side_win,
            font_size,
            font_col,
            &self.default_font,
        );
    }
}

fn draw_single_section<'a>(
    window: &mut GenericWindow,
    elem: &slideshow::Section,
    base_height: &mut f32,
    default_font: &sdl2::ttf::Font<'a, 'a>,
    font_size: (f32, f32),
    font_col: slideshow::Color,
) {
    let canvas = &mut window.canvas;
    let textures = &mut window.textures;

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
                            Some(p) => (p.w, p.h),
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
                        Some(p) => (p.w, p.h),
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

fn draw_sections(
    idx: usize,
    slides: &mut Vec<slideshow::Slide>,
    bg_col: slideshow::Color,
    window: &mut GenericWindow,
    font_size: (f32, f32),
    font_col: slideshow::Color,
    default_font: &sdl2::ttf::Font<'_, '_>,
) {
    let mut base_height: f32 = 0.01;
    let col = slides[idx].bg_color.unwrap_or(bg_col).into();
    {
        utils::canvas_change_color(&mut window.canvas, col);

        for section in slides[idx].sections.iter() {
            draw_single_section(
                window,
                section,
                &mut base_height,
                default_font,
                font_size,
                font_col,
            );
        }
    }
}
