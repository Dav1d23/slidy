use std::error::Error;
use std::path::Path;

use super::lexer::{CurrentState, Lexer};
use super::tokenizer::{Structure, Token};

use crate::windows::slideshow::{
    Color, Section, SectionFigure, SectionMain, SectionText, Slide, Vec2,
};

fn apply_slide<T, U>(
    slide: &mut Option<Slide>,
    mut f: T,
) -> Result<U, Box<dyn Error + 'static>>
where
    T: FnMut(&mut Slide) -> Result<U, Box<dyn Error + 'static>>,
{
    match slide {
        Some(slide) => f(slide),
        None => Err("Please create a slide first.".into()),
    }
}

pub(super) fn manage_import(
    lexer: &mut Lexer,
    tokens: &[Token],
    base_folder: &Path,
) -> Result<usize, Box<dyn Error + 'static>> {
    lexer.internals.state = CurrentState::Import;
    // For the import to work, the next token must be a string.
    let el = if let Some(el) = tokens.get(0).and_then(|t| match t.symbol {
        Structure::String(el) => Some(el),
        _ => None,
    }) {
        el
    } else {
        return Err("In an import, we must have a path.".into());
    };
    // If we have a slide to import, we need to import it
    // after the current one. To do so, we store the
    // current slide and then we append the new ones.
    if lexer.internals.slide.is_some() {
        let cs = lexer.internals.slide.take().unwrap();
        lexer.slideshow.slides.push(cs);
    }
    let mut path = std::path::PathBuf::new();
    path.push(format!("{}/{}", base_folder.display(), el).as_str());
    let mut imported_slides = super::parse_file(&path)?;
    lexer.slideshow.slides.append(&mut imported_slides.slides);
    // If everything went ok, we can ignore the next token.
    Ok(1)
}

pub(super) fn manage_slide(
    lexer: &mut Lexer,
    _tokens: &[Token],
) -> Result<usize, Box<dyn Error + 'static>> {
    match &mut lexer.internals.slide {
        None => lexer.internals.slide = Some(Slide::default()),
        Some(s) => {
            let slide = std::mem::replace(s, Slide::default());
            debug!("Pushing slide: {:?}", &slide);
            lexer.slideshow.slides.push(slide);
        }
    }
    lexer.internals.state = CurrentState::Slide;
    Ok(0)
}

pub(super) fn manage_textline(
    lexer: &mut Lexer,
    el: &str,
    _tokens: &[Token],
    _base_folder: &Path,
) -> Result<usize, Box<dyn Error + 'static>> {
    use CurrentState::*;
    match lexer.internals.state {
        Import | Figure | Slide | General | None => {
            if el.is_empty() {
                Ok(0)
            } else {
                Err("A textline does make sense only in a text section.".into())
            }
        }
        Text => {
            apply_slide(&mut lexer.internals.slide, |slide| {
                let last_section = slide.sections.len() - 1;

                if let Some(ref mut sec_main) =
                    slide.sections[last_section].sec_main
                {
                    if let SectionMain::Text(ref mut text) = sec_main {
                        text.text.push_str(&el.replace("\\:", ":"));
                        text.text.push('\n');
                        Ok(())
                    } else {
                        Err("In a Text section but the last section is not a figure... How?".into())
                    }
                } else {
                    Err("No section is built yet.".into())
                }
            })?;
            Ok(0)
        }
    }
}

pub(super) fn manage_textbuffer(
    lexer: &mut Lexer,
    _tokens: &[Token],
) -> Result<usize, Box<dyn Error + 'static>> {
    lexer.internals.state = CurrentState::Text;
    apply_slide(&mut lexer.internals.slide, |slide| {
        let text_sec = Section {
            sec_main: Some(SectionMain::Text(SectionText::default())),
            ..Default::default()
        };
        slide.sections.push(text_sec);
        Ok(())
    })?;

    Ok(0)
}

pub(super) fn manage_figure(
    lexer: &mut Lexer,
    tokens: &[Token],
    base_folder: &Path,
) -> Result<usize, Box<dyn Error + 'static>> {
    lexer.internals.state = CurrentState::Figure;

    let el = if let Some(el) = tokens.get(0).and_then(|t| match t.symbol {
        Structure::String(el) => Some(el),
        _ => None,
    }) {
        el
    } else {
        return Err("In an figure, we must have a path.".into());
    };

    let figure_path = String::from(
        base_folder
            .join(el)
            .canonicalize()
            .unwrap()
            .to_str()
            .unwrap(),
    );

    apply_slide(&mut lexer.internals.slide, |slide| {
        let figure_sec = Section {
            sec_main: Some(SectionMain::Figure(SectionFigure {
                path: figure_path.clone(),
                ..Default::default()
            })),
            ..Default::default()
        };
        slide.sections.push(figure_sec);
        Ok(())
    })?;

    Ok(1)
}

pub(super) fn manage_position(
    lexer: &mut Lexer,
    tokens: &[Token],
) -> Result<usize, Box<dyn Error + 'static>> {
    use CurrentState::*;
    match lexer.internals.state {
        Import | Slide | General | None => {
            Err("Position does make sense only for text and figures.".into())
        }
        Text | Figure => {
            apply_slide(&mut lexer.internals.slide, |slide| {
                // Get 2 numbers
                let v = if let Some([t1, t2]) = tokens.get(0..2) {
                    let v1 = match t1.symbol {
                        Structure::Number(v) => v,
                        _ => {
                            return Err(format!(
                                "Expect a float, found {:?}",
                                t1
                            )
                            .into())
                        }
                    };
                    let v2 = match t2.symbol {
                        Structure::Number(v) => v,
                        _ => {
                            return Err(format!(
                                "Expect a float, found {:?}",
                                t2
                            )
                            .into())
                        }
                    };
                    Vec2 { x: v1, y: v2 }
                } else {
                    return Err("Position must have 2 tokens after it".into());
                };

                let last_section = slide.sections.len() - 1;
                slide.sections[last_section].position = Some(v);
                Ok(())
            })?;
            Ok(2)
        }
    }
}

/// As a size, we both accept a single integer or 2 floats.
/// In case we find a single float, we re-interpret that as a "single size" and
/// we change both x and y value based on that.
fn get_size(
    tokens: &[Token],
) -> Result<(Vec2, usize), Box<dyn Error + 'static>> {
    if let Some([t1, t2]) = tokens.get(0..2) {
        let skip;
        let mut v1 = match t1.symbol {
            Structure::Number(v) => v,
            _ => return Err(format!("Expect a float, found {:?}", t1).into()),
        };
        let v2 = if let Structure::Number(v) = t2.symbol {
            // We have a second number, so we take that for the size
            skip = 2;
            v
        } else {
            // We did not have a number, so we take
            skip = 1;
            let v2 = v1 / 10.0 * 0.06;
            v1 = v1 / 10.0 * 0.012;
            v2
        };
        Ok((Vec2 { x: v1, y: v2 }, skip))
    } else if let Some(t) = tokens.get(0) {
        // Single value
        let (v1, v2) = if let Structure::Number(v) = t.symbol {
            (v / 10.0 * 0.012, v / 10.0 * 0.06)
        } else {
            return Err(format!("Expect a float, found {:?}", t).into());
        };
        Ok((Vec2 { x: v1, y: v2 }, 1))
    } else {
        Err("Size must have 1/2 tokens after it".into())
    }
}

pub(super) fn manage_size(
    lexer: &mut Lexer,
    tokens: &[Token],
) -> Result<usize, Box<dyn Error + 'static>> {
    use CurrentState::*;
    match lexer.internals.state {
        Import | Slide | None => Err(
            "Size does make sense only in general, text and figure sections."
                .into(),
        ),
        General => {
            let r = get_size(tokens)?;
            lexer.slideshow.font_size = Some(r.0);
            Ok(r.1)
        }
        Text | Figure => {
            let skip = apply_slide(&mut lexer.internals.slide, |slide| {
                let last_section = slide.sections.len() - 1;
                let r = get_size(tokens)?;
                slide.sections[last_section].size = Some(r.0);
                Ok(r.1)
            })?;
            Ok(skip)
        }
    }
}

fn extract_f32(t: &Token) -> Result<f32, Box<dyn Error + 'static>> {
    let v = match t.symbol {
        Structure::Number(v) => v,
        _ => return Err(format!("Expect a float, found {:?}", t).into()),
    };
    Ok(v)
}

fn extract_u8(t: &Token) -> Result<u8, Box<dyn Error + 'static>> {
    let v = extract_f32(t)?;
    if v.ceil() == v.floor() {
        // Ceil is the same as floor, so we can assume the value to be an int.
        let v = v.floor();
        if (0.0..=255.0).contains(&v) {
            // We are in the good range!
            return Ok(v as u8);
        }
    }
    Err(format!("Expect a float, found {:?}", t).into())
}

pub(super) fn manage_fontcolor(
    lexer: &mut Lexer,
    tokens: &[Token],
) -> Result<usize, Box<dyn Error + 'static>> {
    use CurrentState::*;
    match lexer.internals.state {
        Import | Slide | Figure | None => Err(
            "FontColor color does make sense only in general and slide sections."
                .into(),
        ),
        General => {
                        let (c, skip) = get_color(tokens)?;

            lexer.slideshow.font_col = Some(c);
            Ok(skip)
        }
        Text => {
                        let (c, skip) = get_color(tokens)?;

            apply_slide(&mut lexer.internals.slide, |slide| {
                let last_section = slide.sections.len() - 1;
                if let Some(ref mut sec_main) = slide.sections[last_section].sec_main {
                    match sec_main {
                        SectionMain::Text(ref mut text) => {
                            text.color = Some(c);
                        }
                        _ => {
                            return Err("In a text section, but SectionMain is not a text.".into());
                        }
                    }
                } else {
                    return Err("The last section is not ready.".into());
                };
                Ok(())
            })?;
            Ok(skip)
        }
    }
}

/// Color's names are taken from https://encycolorpedia.com/websafe
fn match_string_color(
    color_str: &str,
) -> Result<Color, Box<dyn Error + 'static>> {
    // Try to match the exa values
    if let Some(color_str) = color_str.strip_prefix('#') {
        // Hex mode
        for c in color_str.chars() {
            if !(('0'..='9').contains(&c)
                || ('a'..='f').contains(&c)
                || ('A'..='F').contains(&c))
            {
                return Err("Only exadecimal characters are allowed.".into());
            }
        }
        if color_str.len() == 8 {
            let red = u8::from_str_radix(&color_str[0..2], 16)
                .expect("This cannot fail");
            let green = u8::from_str_radix(&color_str[2..4], 16)
                .expect("This cannot fail");
            let blue = u8::from_str_radix(&color_str[4..6], 16)
                .expect("This cannot fail");
            let alpha = u8::from_str_radix(&color_str[6..8], 16)
                .expect("This cannot fail");
            return Ok((red, green, blue, alpha).into());
        } else {
            return Err("Exa format must be 0xrrggbbaa".into());
        }
    }
    // Try to match the string names
    match color_str.to_lowercase().as_str() {
        "acqua" => return Ok((0x00, 0xff, 0xff, 0xff).into()),
        "black" => return Ok((0x00, 0x00, 0x00, 0xff).into()),
        "blue" => return Ok((0x00, 0x00, 0xff, 0xff).into()),
        "fuchsia" => return Ok((0xff, 0x00, 0xff, 0xff).into()),
        "gray" => return Ok((0x80, 0x80, 0x80, 0xff).into()),
        "green" => return Ok((0x00, 0x80, 0x00, 0xff).into()),
        "lime" => return Ok((0x00, 0xff, 0x00, 0xff).into()),
        "maroon" => return Ok((0x80, 0x00, 0x00, 0xff).into()),
        "navy" => return Ok((0x00, 0x00, 0x80, 0xff).into()),
        "olive" => return Ok((0x80, 0x80, 0x00, 0xff).into()),
        "purple" => return Ok((0x80, 0x00, 0x80, 0xff).into()),
        "red" => return Ok((0xff, 0x00, 0x00, 0xff).into()),
        "silver" => return Ok((0xc0, 0xc0, 0xc0, 0xff).into()),
        "teal" => return Ok((0x00, 0x80, 0x80, 0xff).into()),
        "white" => return Ok((0xff, 0xff, 0xff, 0xff).into()),
        "yellow" => return Ok((0xff, 0xff, 0x00, 0xff).into()),
        _ => {}
    }
    Err(format!("Unable to parse {} into a known color.", color_str).into())
}

fn get_color(
    tokens: &[Token],
) -> Result<(Color, usize), Box<dyn Error + 'static>> {
    // Get 4 numbers
    let mut res = None;
    let mut err_msg = String::with_capacity(1024);
    trace!("get_color: check if there are 4 tokens.");
    if let Some([t1, t2, t3, t4]) = tokens.get(0..4) {
        let v1 = extract_u8(t1);
        let v2 = extract_u8(t2);
        let v3 = extract_u8(t3);
        let v4 = extract_u8(t4);
        if let (Ok(v1), Ok(v2), Ok(v3), Ok(v4)) = (v1, v2, v3, v4) {
            res = Some((v1, v2, v3, v4));
        } else {
            err_msg.push_str(&format!("found 4 invalid tokens, but some are invalid: {:?} {:?} {:?} {:?}", t1, t2, t3, t4));
        }
    }
    if let Some(res) = res {
        return Ok((res.into(), 4));
    }
    trace!("get_color: check if a single token works.");
    // Res was not ok, try to take a single string.
    if let Some(t) = tokens.get(0) {
        if let Structure::String(el) = t.symbol {
            match match_string_color(el) {
                Ok(c) => return Ok((c, 1)),
                Err(e) => err_msg.push_str(&format!(
                    "unable to get the color out of a string: {}",
                    e
                )),
            }
        }
        err_msg.push_str("token is not a string, unable to get the color out");
    } else {
        err_msg.push_str("not enough tokens to take the color out");
    }
    Err(err_msg.into())
}

pub(super) fn manage_bg_color(
    lexer: &mut Lexer,
    tokens: &[Token],
) -> Result<usize, Box<dyn Error + 'static>> {
    use CurrentState::*;
    match lexer.internals.state {
        Import  | Text | Figure | None => Err(
            "Background color does make sense only in general and slide sections."
                .into(),
        ),
        General => {
            let (c, skip) = get_color(tokens)?;
            lexer.slideshow.bg_col = Some(c);
            Ok(skip)
        }
        Slide => {
            let (c, skip) = get_color(tokens)?;
            apply_slide(&mut lexer.internals.slide, |slide| {
                slide.bg_color = Some(c);
                Ok(())
            })?;
            Ok(skip)
        }}
}

pub(super) fn manage_rotation(
    lexer: &mut Lexer,
    tokens: &[Token],
) -> Result<usize, Box<dyn Error + 'static>> {
    use CurrentState::*;
    match lexer.internals.state {
        Import | Slide | Text | General | None => {
            Err("Rotation does make sense only in a figure section.".into())
        }
        Figure => {
            apply_slide(&mut lexer.internals.slide, |slide| {
                let v = if let Some(t) = tokens.get(0) {
                    match t.symbol {
                        Structure::Number(v) => v,
                        _ => {
                            return Err(format!(
                                "Expect a float, found {:?}",
                                t
                            )
                            .into())
                        }
                    }
                } else {
                    return Err("Position must have 1 tokens after it".into());
                };
                let last_section = slide.sections.len() - 1;
                if let Some(SectionMain::Figure(figure)) =
                    &mut slide.sections[last_section].sec_main
                {
                    figure.rotation = v;
                    Ok(())
                } else {
                    Err("In a Figure section but the last section is not a figure... How?".into())
                }
            })?;
            Ok(1)
        }
    }
}

#[cfg(test)]
mod test {
    use crate::parser::tokenizer::tokenizer;

    use super::*;

    #[test]
    fn get_color_ok() {
        let tokens = tokenizer(":cl 0 23 2 42");
        let c = get_color(&tokens[1..]);
        assert!(c.is_ok(), "{:#?}", c);
        let c = c.unwrap().0;
        assert_eq!(c.r, 0, "{:?}", c);
        assert_eq!(c.g, 23, "{:?}", c);
        assert_eq!(c.b, 2, "{:?}", c);
        assert_eq!(c.a, 42, "{:?}", c);
    }

    #[test]
    fn get_color_ok_2() {
        let tokens = tokenizer(":cl #0305a0c1");
        let c = get_color(&tokens[1..]);
        assert!(c.is_ok(), "{:?}", c);
        let c = c.unwrap().0;
        assert_eq!(c.r, 3, "{:?}", c);
        assert_eq!(c.g, 5, "{:?}", c);
        assert_eq!(c.b, 160, "{:?}", c);
        assert_eq!(c.a, 193, "{:?}", c);
    }

    #[test]
    fn get_color_ok_3() {
        let tokens = tokenizer(":cl silver");
        let c = get_color(&tokens[1..]);
        assert!(c.is_ok(), "{:?}", c);
        let c = c.unwrap().0;
        assert_eq!(c.r, 192, "{:?}", c);
        assert_eq!(c.g, 192, "{:?}", c);
        assert_eq!(c.b, 192, "{:?}", c);
        assert_eq!(c.a, 255, "{:?}", c);
    }

    #[test]
    fn get_color_ko() {
        let tokens = tokenizer(":cl pinka");
        let c = get_color(&tokens[1..]);
        assert!(c.is_err(), "{:?}", c);
    }

    #[test]
    fn get_color_ko2() {
        let tokens = tokenizer(":cl 300 200 100 100");
        let c = get_color(&tokens[1..]);
        assert!(c.is_err(), "{:?}", c);
    }
    #[test]
    fn get_color_ko3() {
        let tokens = tokenizer(":cl #q2222222");
        let c = get_color(&tokens[1..]);
        assert!(c.is_err(), "{:?}", c);
    }
}
