use std::collections::HashMap;

/// The position data.
/// Note that this contains float between 0 and 1, and our coordinates are
/// relative to the window.
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
#[derive(serde::Serialize, serde::Deserialize, Debug, PartialEq)]
pub struct Position {
    /// The `x` coordinate.
    pub x: f32,
    /// The `y` coordinate.
    pub y: f32,
}

/// The size of the object to be represented.
#[derive(serde::Serialize, serde::Deserialize, Debug, PartialEq)]
pub struct Size {
    /// The `width`.
    pub w: f32,
    /// The `height`.
    pub h: f32,
}

#[derive(
    serde::Serialize, serde::Deserialize, Debug, Copy, Clone, PartialEq,
)]
/// A color, represented as rgb + alpha.
pub struct Color {
    /// Red
    pub r: u8,
    /// Green
    pub g: u8,
    /// Blue
    pub b: u8,
    /// Alpha
    pub a: u8,
}

impl From<(u8, u8, u8, u8)> for Color {
    fn from(c: (u8, u8, u8, u8)) -> Self {
        Self {
            r: c.0,
            g: c.1,
            b: c.2,
            a: c.3,
        }
    }
}

#[derive(serde::Serialize, serde::Deserialize, Debug, PartialEq)]
/// Define a section that contains a text.
pub struct SectionText {
    /// The text that should be rendered
    pub text: String,
    /// The color of the text
    pub color: Option<Color>,
    // The font name, must be aligned with the global one in the Slide struct
    /// Unused at the moment
    pub font: Option<String>,
}

impl Default for SectionText {
    fn default() -> Self {
        Self {
            text: "".to_owned(),
            color: None,
            font: None,
        }
    }
}

#[derive(serde::Serialize, serde::Deserialize, Debug, PartialEq)]
/// Define a section that contains a figure.
pub struct SectionFigure {
    /// Path to the actual figure's location on disk
    pub path: String,
    /// The rotation, in degrees
    pub rotation: f32,
}

impl Default for SectionFigure {
    #[must_use]
    fn default() -> Self {
        Self {
            path: "".to_owned(),
            rotation: 0.0,
        }
    }
}

/// The main entry in each section.
#[derive(serde::Serialize, serde::Deserialize, Debug, PartialEq)]
pub enum SectionMain {
    /// The variant that represents a picture.
    Figure(SectionFigure),
    /// The variant that represents a text chunk.
    Text(SectionText),
}

/// The internal representation for a `section`.
/// The section can contain text, has a size, a position,
/// and so on and so forth.
#[derive(serde::Serialize, serde::Deserialize, Debug, Default, PartialEq)]
pub struct Section {
    /// The size of the section.
    pub size: Option<Size>,
    /// The position of the section in the slide.
    pub position: Option<Position>,
    /// The specific section.
    pub sec_main: Option<SectionMain>,
}

/// The representation of a single slide.
/// It has a background color and one or more sections.
/// Each section contains either text, or an image, or both.
#[derive(serde::Serialize, serde::Deserialize, Debug, PartialEq)]
pub struct Slide {
    /// The default backgound color.
    pub bg_color: Option<Color>,
    /// The list of sections in the single slide.
    pub sections: Vec<Section>,
}

impl Slide {
    #[must_use]
    /// Create an empty Slide object.
    pub const fn default() -> Self {
        let sections = vec![];
        let bg_color = None;
        Self { bg_color, sections }
    }
}

/// The whole slideshow we have to render.
///
/// Note that not all the information are used by all the backends. But since
/// we have a single parser and multiple backends, it is what it is.
#[derive(serde::Serialize, serde::Deserialize, Debug, Default)]
pub struct Slideshow {
    /// The slides to be shown.
    pub slides: Vec<Slide>,
    /// The hashmap containing the association between the
    /// font names and their path.
    ///
    /// Unused at the moment, as there is only a single font available for SDL.
    pub fonts: HashMap<String, String>,
    /// The default background color.
    pub bg_col: Option<Color>,
    /// The default font color.
    pub font_col: Option<Color>,
    /// The default font size.
    pub font_size: Option<Size>,
}
