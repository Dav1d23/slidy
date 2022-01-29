use std::collections::VecDeque;
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
        fn $func_name(tokens: &VecDeque<Token>) -> Vec<$type> {
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
fn get_color(tokens: &mut VecDeque<Token>) -> Option<Color> {
    let vals_ = get_4_i32_from_int(tokens);
    match vals_[..] {
        [r, g, b, a] => {
            tokens.pop_front();
            tokens.pop_front();
            tokens.pop_front();
            tokens.pop_front();
            Some(Color {
                r: r as u8,
                g: g as u8,
                b: b as u8,
                a: a as u8,
            })
        }
        _ => None,
    }
}

/// Get the size out of a sequence of tokens.
fn get_size(tokens: &mut VecDeque<Token>) -> Option<Vec2> {
    let val_single = get_1_f32_from_float(tokens);
    let val_double = get_2_f32_from_float(tokens);
    if let [x, y] = val_double[..] {
        // If we have 2 values, w and h are set.
        tokens.pop_front();
        tokens.pop_front();
        return Some(Vec2 { x, y });
    } else if let [s] = val_single[..] {
        // ... Otherwise, we have a "font size" that we can interprete based on
        // the assumption that 10 is 0.012 x 0.06
        let x = s / 10.0 * 0.012;
        let y = s / 10.0 * 0.06;
        tokens.pop_front();
        return Some(Vec2 { x, y });
    }
    None
}

/// Get the rotation out of a sequence of tokens.
fn get_rotation(tokens: &mut VecDeque<Token>) -> Option<f32> {
    let vals_ = get_1_f32_from_float(tokens);
    match vals_[..] {
        [val] => {
            tokens.pop_front();
            Some(val)
        }
        _ => None,
    }
}

/// Get the position out of a sequence of tokens.
fn get_position(tokens: &mut VecDeque<Token>) -> Option<Vec2> {
    let vals_ = get_2_f32_from_float(tokens);
    match vals_[..] {
        [x, y] => {
            tokens.pop_front();
            tokens.pop_front();
            Some(Vec2 { x, y })
        }
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

/// The internals of the TextParser.
#[derive(Debug)]
struct TextParserInternal {
    /// In which section were we?
    which_section: InSection,
    current_slide: Option<Slide>,
}

/// The text parser structure.
#[derive(Debug)]
struct TextParser {
    /// The slideshows created up to now.
    slideshow: Slideshow,
    /// The parser's internal status.
    internals: TextParserInternal,
}

/// Check for the existence of a slide, and apply a closure on that.
fn apply_current_slide<T, U>(
    curr_slide: &mut Option<Slide>,
    mut f: T,
) -> Result<U, Box<dyn Error + 'static>>
where
    T: FnMut(&mut Slide) -> Result<U, Box<dyn Error + 'static>>,
{
    match curr_slide {
        Some(slide) => f(slide),
        None => Err("Please create a slide first.".into()),
    }
}

impl TextParser {
    /// Create a new TextParser.
    fn new() -> TextParser {
        TextParser {
            slideshow: Slideshow::default(),
            internals: TextParserInternal {
                which_section: InSection::None,
                current_slide: None,
            },
        }
    }

    /// Extract the slideshow from the TextParser.
    fn take(self) -> Slideshow {
        let s = self.internals.current_slide;
        let mut slideshow = self.slideshow;
        if let Some(s) = s {
            debug!("Pushing slide: {:?}", &s);
            slideshow.slides.push(s);
        }
        slideshow
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
        if inp.is_empty() || inp.starts_with('#') {
            return Ok(());
        }

        // "tokenize" the input str
        let mut tokens: std::collections::VecDeque<Token> =
            inp.split_ascii_whitespace().map(get_token).collect();

        // Consume tokens one by one and update the slides and the internals of
        // the parser.
        while !tokens.is_empty() {
            let token = tokens.pop_front().unwrap();
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
                    apply_current_slide(
                        &mut self.internals.current_slide,
                        |slide| {
                            let text_sec = Section {
                                sec_main: Some(SectionMain::Text(
                                    SectionText::default(),
                                )),
                                ..Default::default()
                            };
                            slide.sections.push(text_sec);
                            Ok(())
                        },
                    )?;
                }
                Symbol::Newline => {
                    trace!("Symbol: Newline");
                    let res = apply_current_slide(
                        &mut self.internals.current_slide,
                        |slide| {
                            let last_section = slide.sections.len() - 1;
                            if let Some(SectionMain::Text(ref mut text)) =
                                slide.sections[last_section].sec_main
                            {
                                text.text.push('\n');
                            } else {
                                warn!(
                            "Newline token is found, but we're not in a text section. Ignore."
                        );
                            };
                            Ok(())
                        },
                    );

                    if res.is_err() {
                        trace!(
                            "Ignoring the newline, since we have no slide yet."
                        );
                    };
                }
                Symbol::String(el) => {
                    trace!("Symbol: String -> {}", &el);
                    if self.internals.which_section == InSection::Import {
                        // If we have a slide to import, we need to import it
                        // after the current one. To do so, we store the
                        // current slide and then we append the new ones.
                        if self.internals.current_slide.is_some() {
                            let cs = self.internals.current_slide.take().unwrap();
                            self.slideshow.slides.push(cs);
                        }
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
                        apply_current_slide(
                            &mut self.internals.current_slide,
                            |slide| {
                                let last_section = slide.sections.len() - 1;

                                if let Some(ref mut sec_main) =
                                    slide.sections[last_section].sec_main
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
                                };
                                Ok(())
                            },
                        )?;
                    }
                }
                Symbol::Slide => {
                    trace!("Symbol: Slide.");
                    match &mut self.internals.current_slide {
                        None => {
                            self.internals.current_slide =
                                Some(Slide::default())
                        }
                        Some(s) => {
                            let slide = std::mem::replace(s, Slide::default());
                            debug!("Pushing slide: {:?}", &slide);
                            self.slideshow.slides.push(slide);
                        }
                    }
                    self.internals.which_section = InSection::Slide;
                }
                Symbol::Figure => {
                    trace!("Symbol: Figure.");
                    self.internals.which_section = InSection::Figure;
                    apply_current_slide(
                        &mut self.internals.current_slide,
                        |slide| {
                            let figure_sec = Section {
                                sec_main: Some(SectionMain::Figure(
                                    SectionFigure::default(),
                                )),
                                ..Default::default()
                            };

                            slide.sections.push(figure_sec);
                            Ok(())
                        },
                    )?;
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
                    match self.internals.which_section {
                        InSection::Import => {
                            return Err(
                                "Unable to use the size token when importing slides.".into()
                            )
                        }

                        InSection::General => {
                            self.slideshow.font_size =
                                match get_size(&mut tokens) {
                                    Some(s) => {
                                        Some(s)
                                    },
                                    None => {
                                        return Err(
                                            "Unable to parse the size token."
                                        .into())
                                    }
                                }
                        }
                        InSection::Figure | InSection::Text => {

                            apply_current_slide(
                        &mut self.internals.current_slide,
                        |slide| {
                            let last_section = slide.sections.len() - 1;
                            slide.sections[last_section].size =
                                match get_size(&mut tokens) {
                                    Some(s) => {
                                        Some(s)
                                    },
                                    None => {
                                        return Err(
                                            "Unable to parse the size token."
                                                .into())
                                    }
                                };
                            Ok(())
                        })?;
                        }
                        InSection::Slide | InSection::None => {
                            return Err(
                                "We don't manage size in a slide section.".
                            into())
                        }
                    }
                }
                Symbol::BackgroundColor => {
                    trace!(
                        "Symbol: BackgroundColor in {:?}",
                        self.internals.which_section
                    );
                    match self.internals.which_section {
                        InSection::Import => {
                            return Err(
                                "Unable to use the BackgroundColor token when importing slides."
                            .into())
                        }
                        InSection::Slide => {
                        apply_current_slide(
                        &mut self.internals.current_slide,
                        |slide| {
                            slide.bg_color = match get_color(&mut tokens) {
                                Some(c) => Some(c),
                                None => {
                                    return Err(
                                        "Unable to read the color token."
                                    .into())
                                }
                            };
                            Ok(())
                        })?;
                        }
                        InSection::Text => {
                            return Err(
                                "You can't set the background color of a text."
                            .into())
                        }
                        InSection::General => {
                            self.slideshow.bg_col = match get_color(&mut tokens)
                            {
                                Some(c) => Some(c),
                                None => return Err(
                                    "Unable to read the color token."
                                .into()),
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
                }
                Symbol::FontColor => {
                    trace!(
                        "Symbol: FontColor in {:?}",
                        self.internals.which_section
                    );
                    match self.internals.which_section {
                        InSection::Import => return Err(
                            "We can't set the FontColor when importing slides."
                                .into(),
                        ),
                        InSection::Slide => {
                            return Err(
                                "FontColor is invalid when defining a slide."
                                    .into(),
                            )
                        }
                        InSection::Text => {
                            apply_current_slide(
                                &mut self.internals.current_slide,
                                |slide| {
                                    let last_section = slide.sections.len() - 1;
                                    if let Some(ref mut sec_main) =
                                        slide.sections[last_section].sec_main
                                    {
                                        match sec_main {
                                    SectionMain::Text(ref mut text) => {
                                        text.color =
                                            match get_color(&mut tokens) {
                                                Some(c) => Some(c),
                                                None => return Err(
                                                    "Unable to read the color token."
                                                .into())
                                            }
                                    }
                                    _ => panic!("In a text section, but SectionMain is not a text."),
                                }
                                    };
                                    Ok(())
                                },
                            )?;
                        }
                        InSection::General => {
                            self.slideshow.font_col =
                                match get_color(&mut tokens) {
                                    Some(c) => Some(c),
                                    None => {
                                        return Err(
                                            "Unable to read the color token."
                                                .into(),
                                        )
                                    }
                                };
                        }
                        InSection::Figure => {
                            return Err(
                                "FontColor is not managed in a figure tag."
                                    .into(),
                            )
                        }
                        InSection::None => {
                            return Err(
                                "You must enter a section to set the color."
                                    .into(),
                            )
                        }
                    }
                }
                Symbol::Position => {
                    trace!(
                        "Symbol: Position in {:?}",
                        self.internals.which_section
                    );
                    apply_current_slide(
                        &mut self.internals.current_slide,
                        |slide| {
                            let last_section = slide.sections.len() - 1;
                            slide.sections[last_section].position =
                                match get_position(&mut tokens) {
                                    Some(s) => Some(s),
                                    None => return Err(
                                        "Unable to parse the position token."
                                            .into(),
                                    ),
                                };
                            Ok(())
                        },
                    )?;
                }
                Symbol::Rotation => {
                    trace!("Symbol: Rotation.");
                    apply_current_slide(
                        &mut self.internals.current_slide,
                        |slide| {
                            let last_section = slide.sections.len() - 1;
                            if let Some(SectionMain::Figure(ref mut fig)) =
                                slide.sections[last_section].sec_main
                            {
                                fig.rotation = match get_rotation(&mut tokens) {
                                    Some(s) => s,
                                    None => return Err(
                                        "Unable to parse the rotation token."
                                            .into(),
                                    ),
                                };
                            } else {
                                return Err(
                                    "We can apply rotations to figure only."
                                        .into(),
                                );
                            };
                            Ok(())
                        },
                    )?;
                }
            }
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

#[cfg(test)]
mod test {
    use super::*;
    use serde_json;
    use std::fs::File;
    use std::io::BufReader;

    /// Load and a file and check its existence.
    macro_rules! load_exists {
        ($f:expr) => {{
            use std::path::PathBuf;

            let mut d = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
            d.push($f);
            assert!(d.exists());
            d
        }};
    }

    #[test]
    /// Verify the simple test in resources works fine.
    fn test_resource_simple_slide() {
        let d = load_exists!("resources/simple_slide.txt");

        let slideshow = parse_file(&d)
            .map_err(|e| panic!("Unable to read the slides: {}", e))
            .unwrap();

        assert_eq!(slideshow.slides.len(), 3);
    }

    #[test]
    /// Verify the input json file is valid.
    fn test_load_json() {
        let d = load_exists!("examples/slidy_serde/resources/input_file.json");

        let f = File::open(&d).expect(&format!("File {:?} not found.", &d));
        let reader = BufReader::new(f);
        let slideshow: Slideshow = serde_json::from_reader(reader)
            .map_err(|e| panic!("Unable to read the slides: {}", e))
            .unwrap();

        assert_eq!(slideshow.slides.len(), 2);
    }

    #[test]
    /// Verify the example in the README works.
    /// Note that if this test fails, we need to change the README as well!
    fn test_readme_example() {
        let example = r#"
# Comments are ignored

:ge :bc 20 40 40 250 :fc 250 250 250 180

:sl
:tb :sz 20 :fc 250 0 0 180
This is title 1
:tb :ps 0.1 0.3 :sz 16
A line \
Another line \
And the last one

:sl
:tb :sz 20 :fc 250 250 0 180
And title 2
:tb :ps 0.1 0.3 :sz 16
Some other content

"#;

        let mut tp = TextParser::new();
        for line in example.split("\n") {
            tp.parse_line(&line, Path::new(""))
                .expect(&format!("Unable to read `{}`.", &line));
        }
    }
}
