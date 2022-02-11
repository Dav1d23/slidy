//! The slideshow definition.

use sdl2::pixels::Color as sdl_color;
use std::collections::HashMap;

/// A 2-D vector. Can also be used to store heights.
/// Note that this contains float, since we are in "coordinates relative to
/// the screen space" like (more or less)
/// ```text
/// (0,0)-----------------(1,0)
///   |                     |
///   |      (0,8,0.2) -> * |
///   |                     |
///   | (0.6,0.6) -> *      |
///   |                     |
///   |                     |
/// (0,1)-----------------(1,1)
/// ```
/// TODO This "coordinates" can also be interpreted as sizes, depending on
/// where they are used. This should be changed to make the code cleaner.
#[derive(serde::Serialize, serde::Deserialize, Debug, PartialEq)]
pub struct Vec2 {
    pub x: f32,
    pub y: f32,
}

#[derive(
    serde::Serialize, serde::Deserialize, Debug, Copy, Clone, PartialEq,
)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl From<Color> for sdl_color {
    fn from(c: Color) -> Self {
        sdl_color::from((c.r, c.g, c.b, c.a))
    }
}

#[allow(clippy::many_single_char_names)]
impl From<sdl_color> for Color {
    fn from(c: sdl_color) -> Self {
        let (r, g, b, a) = c.rgba();
        Color { r, g, b, a }
    }
}

impl From<(u8, u8, u8, u8)> for Color {
    fn from(c: (u8, u8, u8, u8)) -> Self {
        Color {
            r: c.0,
            g: c.1,
            b: c.2,
            a: c.3,
        }
    }
}

/// How a text section should looks like.
#[derive(serde::Serialize, serde::Deserialize, Debug, PartialEq)]
pub struct SectionText {
    /// The text that should be rendered
    pub text: String,
    /// The color of the text
    pub color: Option<Color>,
    // The font name, must be aligned with the global one in the Slide struct
    pub font: Option<String>,
}

impl Default for SectionText {
    /// Get a default, new SectionText.
    fn default() -> SectionText {
        SectionText {
            text: "".to_owned(),
            color: None,
            font: None,
        }
    }
}

/// How a figure section should looks like.
#[derive(serde::Serialize, serde::Deserialize, Debug, PartialEq)]
pub struct SectionFigure {
    pub path: String,
    pub rotation: f32,
}

impl Default for SectionFigure {
    /// Get a default, new SectionFigure.
    fn default() -> SectionFigure {
        SectionFigure {
            path: "".to_owned(),
            rotation: 0.0,
        }
    }
}

/// The main entry in each section.
#[derive(serde::Serialize, serde::Deserialize, Debug, PartialEq)]
pub enum SectionMain {
    // A figure
    Figure(SectionFigure),
    // A text section
    Text(SectionText),
}

/// The internal representation for a `section`.
/// The section can contain text, has a size, a position,
/// and so on and so forth.
#[derive(serde::Serialize, serde::Deserialize, Debug, Default, PartialEq)]
pub struct Section {
    pub size: Option<Vec2>,
    pub position: Option<Vec2>,
    pub sec_main: Option<SectionMain>,
}

/// The representation of a single slide.
/// It has a background color and one or more sections.
/// Each section contains either text, or an image, or both.
#[derive(serde::Serialize, serde::Deserialize, Debug, PartialEq)]
pub struct Slide {
    pub bg_color: Option<Color>,
    pub sections: Vec<Section>,
}

impl Slide {
    pub fn default() -> Slide {
        let sections = vec![];
        let bg_color = None;
        Slide { bg_color, sections }
    }
}

/// The whole slideshow we have to render.
/// Note that some information might not be useful in case we would
/// implement different back-ends, but this is not a problem now.
#[derive(serde::Serialize, serde::Deserialize, Debug, Default)]
pub struct Slideshow {
    /// The slides to be shown.
    pub slides: Vec<Slide>,
    /// The hashmap containing the association between the
    /// font names and their path.
    pub fonts: HashMap<String, String>,
    /// The default background color.
    pub bg_col: Option<Color>,
    /// The default font color.
    pub font_col: Option<Color>,
    /// The default font size.
    pub font_size: Option<Vec2>,
}
