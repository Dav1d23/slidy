//! Get out the logic from a stream of tokens.

use log::{debug, trace};
use std::error::Error;
use std::path::Path;

use super::tokenizer::{Structure, Token};
use super::utils;

use crate::slideshow;

/// Helper to understand in which section we're in.
/// It is based upon the tag we encountered while parsing.
#[derive(Debug, PartialEq)]
pub(super) enum CurrentState {
    General,
    Slide,
    Figure,
    Text,
    Import,
    /// We are in no section (useful to init the slides).
    None,
}

impl Default for CurrentState {
    fn default() -> CurrentState {
        CurrentState::None
    }
}

/// The internals of the TextParser.
#[derive(Debug, Default)]
pub(super) struct LexerInternal {
    /// In which section were we?
    pub state: CurrentState,
    pub slide: Option<slideshow::Slide>,
}

/// The text parser structure.
#[derive(Debug, Default)]
pub(super) struct Lexer<'a> {
    /// The slideshows created up to now.
    pub slideshow: slideshow::Slideshow,
    /// The parser's internal status.
    pub internals: LexerInternal,
    pub base_folder: Option<&'a Path>,
}

/// Check for the existence of a slide, and apply a closure on that.
impl<'a> Lexer<'a> {
    pub(super) fn new(base_folder: &'a Path) -> Lexer {
        Lexer {
            base_folder: Some(base_folder),
            ..Default::default()
        }
    }

    /// Consume the lexer and extract the slideshow.
    pub(super) fn take(self) -> slideshow::Slideshow {
        let s = self.internals.slide;
        let mut slideshow = self.slideshow;
        if let Some(s) = s {
            debug!("Pushing slide: {:?}", &s);
            slideshow.slides.push(s);
        }
        slideshow
    }

    /// Read the input tokens and build the related slideshow.
    ///
    /// Note that this function may be called multiple times in case multiple
    /// streams are given, but a prerequisite is that each "group" of token
    /// must be present when we read it. As an example, it is perfectly ok to
    /// build the slides by passing each complete slide to this function, but
    /// is it _not_ ok to give a color token not followed by the color itself.
    pub(super) fn read_tokens(
        &mut self,
        tokens: &[Token],
    ) -> Result<(), Box<dyn Error + 'static>> {
        let base_folder = match self.base_folder {
            Some(b) => b,
            None => todo!("base_folder must be set for now."),
        };
        let mut tokens = tokens;
        while let Some((t, rem)) = tokens.split_first() {
            // t is the token we are checking, rem is the remaining tokens.
            // We need to update rem in case we peek some elements.
            trace!("token: {:?}", t);
            let skip = match t.symbol {
                Structure::Generic => {
                    self.internals.state = CurrentState::General;
                    Ok(0)
                }
                Structure::Figure => {
                    utils::manage_figure(self, rem, base_folder)
                }
                Structure::Import => {
                    utils::manage_import(self, rem, base_folder)
                }
                Structure::Slide => utils::manage_slide(self, rem),
                Structure::TextLine(el) => {
                    utils::manage_textline(self, el, rem, base_folder)
                }
                Structure::TextBuffer => utils::manage_textbuffer(self, rem),
                Structure::Position => utils::manage_position(self, rem),
                Structure::Size => utils::manage_size(self, rem),
                Structure::Rotation => utils::manage_rotation(self, rem),
                Structure::Fontcolor => utils::manage_fontcolor(self, rem),
                Structure::BackGroundColor => utils::manage_bg_color(self, rem),
                Structure::Comment(_) => {
                    // Ignore comments.
                    Ok(0)
                }
                Structure::String(_) | Structure::Number(_) => {
                    // If I see a floating string or number, something went wrong.
                    return Err("I should not been able to see strings or numbers, as I should already have put this in the appropriate sections. Getting here means that for instance I did not read a number for a color, or a string for an import, or something similar.".into());
                }
            };
            let skip: Result<usize, Box<dyn Error>> =
                skip.map_err(|e| format!("token {:?}: {}", &t, e).into());
            let skip = skip?;
            tokens = &rem[skip..];
        }
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use std::path::PathBuf;

    use crate::parser::tokenizer::TokenSpan;

    use super::*;
    use crate::slideshow::*;
    use Structure::*;

    fn resources_path() -> PathBuf {
        let mut base_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        base_path.push("resources");
        assert!(base_path.is_dir());
        base_path
    }

    #[test]
    fn test_import_wrong_tokens() {
        let tokens = [
            Token::new(Import, TokenSpan::new(0, 1, 2)),
            Token::new(Figure, TokenSpan::new(0, 1, 2)),
        ];
        let base_path = resources_path();
        let mut lex = Lexer::new(base_path.as_path());
        assert!(lex.read_tokens(&tokens).is_err());
    }

    #[test]
    fn test_color_string() {
        let tokens = [
            Token::new(Slide, TokenSpan::new(0, 1, 2)),
            Token::new(TextBuffer, TokenSpan::new(1, 1, 2)),
            Token::new(Fontcolor, TokenSpan::new(2, 1, 2)),
            Token::new(String("#80808012"), TokenSpan::new(3, 1, 2)),
            Token::new(TextBuffer, TokenSpan::new(4, 1, 2)),
            Token::new(Fontcolor, TokenSpan::new(5, 1, 2)),
            Token::new(String("red"), TokenSpan::new(6, 1, 2)),
        ];
        let base_path = resources_path();
        let mut lex = Lexer::new(base_path.as_path());
        assert!(lex.read_tokens(&tokens).is_ok());
        let slideshow = lex.take();
        assert_eq!(slideshow.slides.len(), 1);
        let result = slideshow
            .slides
            .get(0)
            .expect("I expect the slide to be built.");

        let slide = slideshow::Slide {
            bg_color: None,
            sections: vec![
                Section {
                    size: None,
                    position: None,
                    sec_main: Some(SectionMain::Text(SectionText {
                        text: std::string::String::from(""),
                        color: Some(Color {
                            r: 128,
                            g: 128,
                            b: 128,
                            a: 18,
                        }),
                        font: None,
                    })),
                },
                Section {
                    size: None,
                    position: None,
                    sec_main: Some(SectionMain::Text(SectionText {
                        text: std::string::String::from(""),
                        color: Some(Color {
                            r: 255,
                            g: 0,
                            b: 0,
                            a: 255,
                        }),
                        font: None,
                    })),
                },
            ],
        };
        assert_eq!(result, &slide);
    }
}
