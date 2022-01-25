use std::error::Error;
use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;
use std::path::Path;

use crate::windows::slideshow::{
    Color, Section, SectionFigure, SectionMain, SectionText, Slide, Slideshow,
    Vec2,
};

/// The token structure.
#[derive(Debug)]
struct Token<'a> {
    symbol: Symbol<'a>,
}

/// The list of symbols the parser will recognize.
/// Note that this is not great. Instead of parsing like
/// letters, symbols, numbers and stuffs, I just try to
/// recognize the symbol as a whole.
#[derive(Debug, PartialEq)]
enum Symbol<'a> {
    Generic,
    Slide,
    Size,
    Text,
    BackgroundColor,
    FontColor,
    Position,
    Figure,
    Rotation,
    Import,
    Newline,
    // Generic tokens (numbers are always 32 bit, could probably do something better here.
    String(&'a str),
}

/// Decode a string of text into the appropriate Token.
fn get_token(text: &str) -> Token {
    let symbol = match text {
        ":ge" => Symbol::Generic,
        ":fc" => Symbol::FontColor,
        ":bc" => Symbol::BackgroundColor,
        ":sl" => Symbol::Slide,
        ":sz" => Symbol::Size,
        ":tb" => Symbol::Text,
        ":ps" => Symbol::Position,
        ":fg" => Symbol::Figure,
        ":rt" => Symbol::Rotation,
        ":im" => Symbol::Import,
        "\\" => Symbol::Newline,
        _ => Symbol::String(text),
    };
    Token { symbol }
}

/// Small macro to create simple tokens -> object functions.
macro_rules! create_parse {
    ($func_name:ident, $how_many:expr, $type:ident) => {
        fn $func_name(tokens: &[Token]) -> Vec<$type> {
            let vals_: Vec<Option<$type>> = tokens
                .iter()
                .take($how_many)
                .map(|e| match e.symbol {
                    Symbol::String(val) => match val.parse::<$type>() {
                        Ok(val) => Some(val),
                        Err(err) => {
                            warn!("`{}` cannot be parsed: {}", val, err);
                            None
                        }
                    },
                    _ => None,
                })
                .collect();
            if vals_.len() != $how_many {
                return vec![];
            }
            for elem in vals_.iter() {
                if elem.is_none() {
                    return vec![];
                }
            }
            vals_.iter().map(|e| e.unwrap()).collect()
        }
    };
}

create_parse!(get_2_f32_from_float, 2, f32);
create_parse!(get_1_f32_from_float, 1, f32);
create_parse!(get_4_i32_from_int, 4, i32);

/// Get the color out of a sequence of tokens.
fn get_color(tokens: &[Token]) -> Option<Color> {
    let vals_ = get_4_i32_from_int(tokens);
    match vals_[..] {
        [r, g, b, a] => Some(Color {
            r: r as u8,
            g: g as u8,
            b: b as u8,
            a: a as u8,
        }),
        _ => None,
    }
}

/// Get the size out of a sequence of tokens.
fn get_size(tokens: &[Token]) -> (Option<Vec2>, u8) {
    let val_single = get_1_f32_from_float(tokens);
    let val_double = get_2_f32_from_float(tokens);
    if let [x, y] = val_double[..] {
        // If we have 2 values, w and h are set.
        return (Some(Vec2 { x, y }), 2);
    } else if let [s] = val_single[..] {
        // ... Otherwise, we have a "font size" that we can interprete based on
        // the assumption that 10 is 0.012 x 0.06
        let x = s / 10.0 * 0.012;
        let y = s / 10.0 * 0.06;
        return (Some(Vec2 { x, y }), 1);
    }
    (None, 0)
}

/// Get the rotation out of a sequence of tokens.
fn get_rotation(tokens: &[Token]) -> Option<f32> {
    let vals_ = get_1_f32_from_float(tokens);
    match vals_[..] {
        [val] => Some(val),
        _ => None,
    }
}

/// Get the position out of a sequence of tokens.
fn get_position(tokens: &[Token]) -> Option<Vec2> {
    let vals_ = get_2_f32_from_float(tokens);
    match vals_[..] {
        [x, y] => Some(Vec2 { x, y }),
        _ => None,
    }
}

/// Helper to understand in which section we're in.
/// It is based upon the tag we encountered while parsing.
#[derive(Debug, PartialEq)]
enum InSection {
    General,
    Slide,
    Figure,
    Text,
    Import,
    /// We are in no section (useful to init the slides).
    None,
}

/// Create an error printing the token being read.
fn create_err(what: &str, token_num: usize) -> Box<dyn Error + 'static> {
    format!("token: {} => {}", token_num, what).into()
}

/// The internals of the TextParser.
#[derive(Debug)]
struct TextParserInternal {
    /// In which section were we?
    which_section: InSection,
}

/// The text parser structure.
#[derive(Debug)]
struct TextParser {
    /// The slideshows created up to now.
    slideshow: Slideshow,
    /// The parser's internal status.
    internals: TextParserInternal,
}

impl TextParser {
    /// Create a new TextParser.
    fn new() -> TextParser {
        TextParser {
            slideshow: Slideshow::default(),
            internals: TextParserInternal {
                which_section: InSection::None,
            },
        }
    }

    /// Extract the slideshow from the TextParser.
    fn take(self) -> Slideshow {
        self.slideshow
    }

    /// Parse a single line of text.
    /// The partial result is stored in the slideshow structure
    /// inside the TextParser structure.
    /// * `inp`: the input string
    /// * `line_num`: to give some context, which line was the error found.
    /// * `base_folder`: needed to resolve the (relative) paths
    fn parse_line(
        &mut self,
        inp: &str,
        base_folder: &Path,
    ) -> Result<(), Box<dyn Error + 'static>> {
        // "tokenize" the input str
        let tokens: Vec<Token> = inp.split(' ').map(get_token).collect();

        let mut idx = 0;
        // Consume tokens one by one and update the slides and the internals of
        // the parser.
        while idx < tokens.len() {
            let token = &tokens[idx];
            let next_tokens = &tokens[idx + 1..];
            match token.symbol {
                Symbol::Import => {
                    trace!("Symbol: Import.");
                    self.internals.which_section = InSection::Import;
                    // Extend slides with the ones we're reading after the
                    // string is read, nothing more to do in here.
                }
                Symbol::Text => {
                    trace!("Symbol: Text.");
                    self.internals.which_section = InSection::Text;
                    if self.slideshow.slides.is_empty() {
                        return Err(create_err(
                            "Please create a slide first.",
                            idx,
                        ));
                    }
                    let last_idx = self.slideshow.slides.len() - 1;
                    let last_slide =
                        self.slideshow.slides.get_mut(last_idx).unwrap();
                    let text_sec = Section {
                        sec_main: Some(SectionMain::Text(
                            SectionText::default(),
                        )),
                        ..Default::default()
                    };
                    last_slide.sections.push(text_sec);
                }
                Symbol::Newline => {
                    trace!("Symbol: Newline");
                    if self.slideshow.slides.is_empty() {
                        return Err(create_err(
                            "Please create a slide first.",
                            idx,
                        ));
                    }
                    let last_idx = self.slideshow.slides.len() - 1;
                    let last_slide =
                        self.slideshow.slides.get_mut(last_idx).unwrap();
                    let last_section = last_slide.sections.len() - 1;
                    if let Some(SectionMain::Text(ref mut text)) =
                        last_slide.sections[last_section].sec_main
                    {
                        text.text.push('\n');
                    } else {
                        warn!(
                            "Newline token is found, but we're not in a text section. Ignore."
                        );
                    }
                }
                Symbol::String(el) => {
                    trace!("Symbol: String -> {}", &el);
                    if self.slideshow.slides.is_empty() {
                        return Err("Please create a slide first.".into());
                    }
                    let last_idx = self.slideshow.slides.len() - 1;
                    if self.internals.which_section == InSection::Import {
                        let mut path = std::path::PathBuf::new();
                        path.push(
                            format!("{}/{}", base_folder.display(), el)
                                .as_str(),
                        );
                        let mut imported_slides = parse_file(&path)?;
                        self.slideshow
                            .slides
                            .append(&mut imported_slides.slides);
                    } else {
                        let last_slide =
                            self.slideshow.slides.get_mut(last_idx).unwrap();
                        let last_section = last_slide.sections.len() - 1;

                        if let Some(ref mut sec_main) =
                            last_slide.sections[last_section].sec_main
                        {
                            match sec_main {
                                SectionMain::Text(ref mut text) => {
                                    text.text.push_str(el);
                                    text.text.push(' ');
                                }
                                SectionMain::Figure(ref mut figure) => {
                                    figure.path = String::from(
                                        base_folder
                                            .join(el)
                                            .canonicalize()
                                            .unwrap()
                                            .to_str()
                                            .unwrap(),
                                    );
                                }
                            }
                        }
                    }
                }
                Symbol::Slide => {
                    trace!("Symbol: Slide.");
                    self.slideshow.slides.push(Slide::default());
                    self.internals.which_section = InSection::Slide;
                }
                Symbol::Figure => {
                    trace!("Symbol: Figure.");
                    self.internals.which_section = InSection::Figure;
                    if self.slideshow.slides.is_empty() {
                        return Err("Please create a slide first.".into());
                    }
                    let last_idx = self.slideshow.slides.len() - 1;
                    let last_slide =
                        self.slideshow.slides.get_mut(last_idx).unwrap();
                    let figure_sec = Section {
                        sec_main: Some(SectionMain::Figure(
                            SectionFigure::default(),
                        )),
                        ..Default::default()
                    };

                    last_slide.sections.push(figure_sec);
                }
                Symbol::Generic => {
                    trace!("Symbol: Generic.");
                    self.internals.which_section = InSection::General;
                }
                Symbol::Size => {
                    trace!(
                        "Symbol: Size in {:?}",
                        self.internals.which_section
                    );
                    // Why is this complaining? Is it because I may quit the
                    // statement without using this variable? And then, what?
                    #[allow(unused_assignments)]
                    let mut how_many_skips = 0;
                    match self.internals.which_section {
                        InSection::Import => {
                            return Err(create_err(
                                "Unable to use the size token when importing slides.",
                                idx,
                            ))
                        }

                        InSection::General => {
                            self.slideshow.font_size =
                                match get_size(next_tokens) {
                                    (Some(s), skip) => {
                                        how_many_skips = skip; Some(s)
                                    },
                                    (None, _) => {
                                        return Err(create_err(
                                            "Unable to parse the size token.",
                                            idx,
                                        ))
                                    }
                                }
                        }
                        InSection::Figure | InSection::Text => {
                            let last_idx = self.slideshow.slides.len() - 1;
                            let last_slide = self
                                .slideshow
                                .slides
                                .get_mut(last_idx)
                                .unwrap();
                            let last_section = last_slide.sections.len() - 1;
                            last_slide.sections[last_section].size =
                                match get_size(next_tokens) {
                                    (Some(s), skip) => {
                                        how_many_skips = skip; Some(s)
                                    },
                                    (None, _) => {
                                        return Err(create_err(
                                            "Unable to parse the size token.",
                                            idx,
                                        ))
                                    }
                                }
                        }
                        InSection::Slide | InSection::None => {
                            return Err(create_err(
                                "We don't manage size in a slide section.",
                                idx,
                            ))
                        }
                    }
                    idx += how_many_skips as usize;
                }
                Symbol::BackgroundColor => {
                    trace!(
                        "Symbol: BackgroundColor in {:?}",
                        self.internals.which_section
                    );
                    match self.internals.which_section {
                        InSection::Import => {
                            return Err(create_err(
                                "Unable to use the BackgroundColor token when importing slides.",
                                idx,
                            ))
                        }
                        InSection::Slide => {
                            let last_idx = self.slideshow.slides.len() - 1;
                            let mut last_slide = self
                                .slideshow
                                .slides
                                .get_mut(last_idx)
                                .unwrap();

                            last_slide.bg_color = match get_color(next_tokens) {
                                Some(c) => Some(c),
                                None => {
                                    return Err(create_err(
                                        "Unable to read the color token.",
                                        idx,
                                    ))
                                }
                            };
                        }
                        InSection::Text => {
                            return Err(create_err(
                                "You can't set the background color of a text.",
                                idx,
                            ))
                        }
                        InSection::General => {
                            self.slideshow.bg_col = match get_color(next_tokens)
                            {
                                Some(c) => Some(c),
                                None => return Err(create_err(
                                    "Unable to read the color token.",
                                    idx,
                                )),
                            };
                        }
                        InSection::Figure => {
                            return Err(
                                "Color is not managed in a figure tag.".into()
                            )
                        }
                        InSection::None => {
                            return Err(
                                "You must enter a section to set the color."
                                    .into(),
                            )
                        }
                    }
                    idx += 4;
                }
                Symbol::FontColor => {
                    trace!(
                        "Symbol: FontColor in {:?}",
                        self.internals.which_section
                    );
                    match self.internals.which_section {
                        InSection::Import => {
                            return Err(create_err(
                                "We can't set the FontColor when importing slides.",
                                idx,
                            ))
                        }
                        InSection::Slide => {
                            return Err(create_err(
                                "FontColor is invalid when defining a slide.",
                                idx,
                            ))
                        }
                        InSection::Text => {
                            let last_idx = self.slideshow.slides.len() - 1;
                            let last_slide = self
                                .slideshow
                                .slides
                                .get_mut(last_idx)
                                .unwrap();
                            let last_section = last_slide.sections.len() - 1;
                            if let Some(ref mut sec_main) =
                                last_slide.sections[last_section].sec_main
                            {
                                match sec_main {
                                    SectionMain::Text(ref mut text) => {
                                        text.color =
                                            match get_color(next_tokens) {
                                                Some(c) => Some(c),
                                                None => return Err(create_err(
                                                    "Unable to read the color token.",
                                                    idx,
                                                ))
                                            }
                                    }
                                    _ => panic!("In a text section, but SectionMain is not a text."),
                                }
                            }
                        }
                        InSection::General => {
                            self.slideshow.font_col = match get_color(next_tokens)
                            {
                                Some(c) => Some(c),
                                None => return Err(create_err(
                                    "Unable to read the color token.",
                                    idx,
                                )),
                            };
                        }
                        InSection::Figure => {
                            return Err(
                                "FontColor is not managed in a figure tag.".into()
                            )
                        }
                        InSection::None => {
                            return Err(
                                "You must enter a section to set the color."
                                    .into(),
                            )
                        }
                    }
                    idx += 4;
                }
                Symbol::Position => {
                    trace!(
                        "Symbol: Position in {:?}",
                        self.internals.which_section
                    );
                    let last_idx = self.slideshow.slides.len() - 1;
                    let last_slide =
                        self.slideshow.slides.get_mut(last_idx).unwrap();
                    let last_section = last_slide.sections.len() - 1;
                    last_slide.sections[last_section].position =
                        match get_position(next_tokens) {
                            Some(s) => Some(s),
                            None => {
                                return Err(create_err(
                                    "Unable to parse the position token.",
                                    idx,
                                ))
                            }
                        };
                    idx += 2;
                }
                Symbol::Rotation => {
                    trace!("Symbol: Rotation.");
                    let last_idx = self.slideshow.slides.len() - 1;
                    let last_slide =
                        self.slideshow.slides.get_mut(last_idx).unwrap();
                    let last_section = last_slide.sections.len() - 1;
                    if let Some(SectionMain::Figure(ref mut fig)) =
                        last_slide.sections[last_section].sec_main
                    {
                        fig.rotation = match get_rotation(next_tokens) {
                            Some(s) => s,
                            None => {
                                return Err(create_err(
                                    "Unable to parse the rotation token.",
                                    idx,
                                ))
                            }
                        };
                    } else {
                        return Err(
                            "We can apply rotations to figure only.".into()
                        );
                    }
                    idx += 1;
                }
            }
            idx += 1;
        }
        Ok(())
    }
}

/// Create the slides.
fn create_slides(
    reader: BufReader<File>,
    base_folder: &Path,
) -> Result<Slideshow, Box<dyn Error + 'static>> {
    let mut tp = TextParser::new();

    for (line_num, line) in reader.lines().enumerate() {
        let line: String = line?;
        // Ignore empty lines, and lines starting with # (comments!)
        if line.is_empty() || line.as_str().starts_with('#') {
            continue;
        }
        if let Err(e) = tp.parse_line(&line, base_folder) {
            // Report the line_num in "normal way"
            return Err(format!("Line {}: {}", line_num + 1, e).into());
        };
    }

    let slideshow = tp.take();
    Ok(slideshow)
}

/// Parse the file, and return the slides as a result.
pub fn parse_file(
    path: &std::path::Path,
) -> Result<Slideshow, Box<dyn Error + 'static>> {
    let file = File::open(path)?;
    if !path.is_file() {
        return Err("`{}` is not a file, please provide one.".into());
    }
    let reader = BufReader::new(file);
    let base_folder = path
        .parent()
        .ok_or("Unable to find the parent: is this root already?")?;
    let slides = create_slides(reader, base_folder)?;
    Ok(slides)
}
