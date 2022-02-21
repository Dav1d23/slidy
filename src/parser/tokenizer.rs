/*! The tokenizer.

Every line starting with # is a comment.

Every line that does not contain "tokens" is considered a Text line.

Whitespaces are ignored, except in the text buffers, where these are
maintained.

One can escape tokens by using \ in front of a token (like \\:ge).

Check the module's tests for more details.

*/
use log::error;

#[derive(Debug, PartialEq)]
pub(super) struct TokenSpan {
    line: usize,
    beg: usize,
    end: usize,
}

impl TokenSpan {
    pub(super) fn new(line: usize, beg: usize, end: usize) -> TokenSpan {
        TokenSpan { line, beg, end }
    }
}

/// The list of symbols the parser will recognize.
/// Note that this is not great. Instead of parsing like
/// letters, symbols, numbers and stuffs, I just try to
/// recognize the symbol as a whole.
#[derive(Debug, PartialEq)]
pub(super) enum Structure<'a> {
    Generic,
    Fontcolor,
    BackGroundColor,
    Slide,
    Size,
    TextBuffer,
    Position,
    Figure,
    Rotation,
    Import,
    TextLine(&'a str),
    Comment(&'a str),
    // Generic stuffs, like string, numbers (everything is a f32 internally)
    String(&'a str),
    Number(f32),
}

#[derive(Debug, PartialEq)]
/// A token is built without knowing about the structure of the thing to be
/// parsed.
pub(super) struct Token<'a> {
    pub symbol: Structure<'a>,
    span: TokenSpan,
}

impl<'a> Token<'a> {
    pub(super) fn new(symbol: Structure<'a>, span: TokenSpan) -> Token {
        Token { symbol, span }
    }
}

fn build_token(val: &str, linenum: usize, beg: usize, end: usize) -> Token {
    use Structure::*;
    let structure = match val {
        ":ge" => Generic,
        ":fc" => Fontcolor,
        ":bc" => BackGroundColor,
        ":sl" => Slide,
        ":sz" => Size,
        ":tb" => TextBuffer,
        ":ps" => Position,
        ":fg" => Figure,
        ":rt" => Rotation,
        ":im" => Import,
        _ => {
            if let Ok(num) = val.parse::<f32>() {
                Number(num)
            } else {
                String(val)
            }
        }
    };

    Token {
        symbol: structure,
        span: TokenSpan::new(linenum, beg, end),
    }
}

/// Parse the line, knowing that we surely don't have TextLine and Comments here.
fn parse_single_tokens<'a>(
    tokens: &mut Vec<Token<'a>>,
    line: &'a str,
    linenum: usize,
) {
    if line.len() >= isize::MAX as usize {
        error!(
            "We don't support lines that are longer than {}: found {}",
            isize::MAX,
            line.len()
        );
        return;
    }
    let mut last_whitespace = -1_isize;
    let mut whitespace_mode = false;
    for (pos, ch) in line.chars().enumerate() {
        if ch.is_whitespace() {
            if pos == 0 {
                whitespace_mode = true;
            }
            if !whitespace_mode {
                // This whitespace comes after "something".
                let elem = line
                    .get((last_whitespace + 1) as usize..pos)
                    .expect("Pos is past the end of the slice.");
                if !elem.is_empty() {
                    let tk = build_token(
                        elem,
                        linenum,
                        (last_whitespace + 1) as usize,
                        pos,
                    );
                    tokens.push(tk);
                }

                whitespace_mode = true;
            }
            last_whitespace = pos as isize;
        } else {
            // Not whitespace anymore: advance until the end of the line or a whitespace.
            whitespace_mode = false;
        }
    }
    if !whitespace_mode {
        // The last char was not a whitespace, so it has to be considered.
        let elem = line
            .get((last_whitespace + 1) as usize..)
            .expect("last_whitespace is out of the array");
        let tk = build_token(
            elem,
            linenum,
            (last_whitespace + 1) as usize,
            line.len(),
        );
        tokens.push(tk);
    }
}

/// Parse a line, and detect all the TextLine and Comments that are there.
fn parse_line<'a>(tokens: &mut Vec<Token<'a>>, line: &'a str, linenum: usize) {
    // Find the position of columns.
    let mut found_token = false;
    let mut idx = 0;
    let mut modline = line;
    loop {
        if idx >= modline.len() {
            break;
        }
        modline = &modline[idx..];
        let col = modline.find(':');
        if let Some(col) = col {
            if col == 0 {
                // This must be a token, so the line cannot be a text line.
                found_token = true;
                break;
            } else if modline.get(col - 1..col) == Some("\\") {
                // This token is escaped, this can still be a line.
                idx = col + 1;
            } else {
                // The token is not escaped.
                found_token = true;
                break;
            }
        } else {
            break;
        }
    }
    if !found_token {
        let tok = Token::new(
            if !line.starts_with('#') {
                Structure::TextLine(line)
            } else {
                Structure::Comment(line)
            },
            TokenSpan::new(linenum, 0, line.len()),
        );
        tokens.push(tok);
    } else {
        // There is a token, so we must build the tokens and add them.
        parse_single_tokens(tokens, line, linenum);
    }
}

pub(super) fn tokenizer(inp: &str) -> Vec<Token> {
    let mut tokens: Vec<Token> = vec![];
    for (linenum, line) in inp.lines().enumerate() {
        parse_line(&mut tokens, line, linenum);
    }
    tokens
}

#[cfg(test)]
mod test {
    use super::*;
    use Structure::*;

    #[test]
    fn test_single_token() {
        let inp = ":sl";
        let tokens = tokenizer(inp);
        let res = vec![Token {
            symbol: Slide,
            span: TokenSpan {
                line: 0,
                beg: 0,
                end: 3,
            },
        }];
        for (e1, e2) in tokens.iter().zip(res.iter()) {
            assert_eq!(e1, e2, "{:?} vs {:?}", tokens, res);
        }
    }

    #[test]
    fn test_multiple_tokens() {
        let inp = ":tb :ps 0.1 0.2 :fc 255 0 0 255";
        let tokens = tokenizer(inp);
        let res = vec![
            Token {
                symbol: TextBuffer,
                span: TokenSpan {
                    line: 0,
                    beg: 0,
                    end: 3,
                },
            },
            Token {
                symbol: Position,
                span: TokenSpan {
                    line: 0,
                    beg: 4,
                    end: 7,
                },
            },
            Token {
                symbol: Number(0.1),
                span: TokenSpan {
                    line: 0,
                    beg: 8,
                    end: 11,
                },
            },
            Token {
                symbol: Number(0.2),
                span: TokenSpan {
                    line: 0,
                    beg: 12,
                    end: 15,
                },
            },
            Token {
                symbol: Fontcolor,
                span: TokenSpan {
                    line: 0,
                    beg: 16,
                    end: 19,
                },
            },
            Token {
                symbol: Number(255.0),
                span: TokenSpan {
                    line: 0,
                    beg: 20,
                    end: 23,
                },
            },
            Token {
                symbol: Number(0.0),
                span: TokenSpan {
                    line: 0,
                    beg: 24,
                    end: 25,
                },
            },
            Token {
                symbol: Number(0.0),
                span: TokenSpan {
                    line: 0,
                    beg: 26,
                    end: 27,
                },
            },
            Token {
                symbol: Number(255.0),
                span: TokenSpan {
                    line: 0,
                    beg: 28,
                    end: 31,
                },
            },
        ];
        assert_eq!(tokens.len(), res.len(), "{:?} vs {:?}", tokens, res);
        for (e1, e2) in tokens.iter().zip(res.iter()) {
            assert_eq!(e1, e2, "{:?} vs {:?}", tokens, res);
        }
    }

    #[test]
    fn test_parse_single_line() {
        let inp = " line no :ge escaped ";
        let mut tokens = vec![];
        parse_single_tokens(&mut tokens, inp, 0);
        let res = [
            Token {
                symbol: String("line"),
                span: TokenSpan {
                    line: 0,
                    beg: 1,
                    end: 5,
                },
            },
            Token {
                symbol: String("no"),
                span: TokenSpan {
                    line: 0,
                    beg: 6,
                    end: 8,
                },
            },
            Token {
                symbol: Generic,
                span: TokenSpan {
                    line: 0,
                    beg: 9,
                    end: 12,
                },
            },
            Token {
                symbol: String("escaped"),
                span: TokenSpan {
                    line: 0,
                    beg: 13,
                    end: 20,
                },
            },
        ];
        for (e1, e2) in tokens.iter().zip(res.iter()) {
            assert_eq!(e1, e2, "{:?} vs {:?}", tokens, res);
        }
    }

    #[test]
    fn test_gettokens() {
        let inp = r#":ge 1 2 :ps 1 
# a comment!
:sz something

   a text line with \:ge escaped  
 and another line below
        "#;
        let tokens = tokenizer(inp);
        let res = [
            Token {
                symbol: Generic,
                span: TokenSpan {
                    line: 0,
                    beg: 0,
                    end: 3,
                },
            },
            Token {
                symbol: Number(1.0),
                span: TokenSpan {
                    line: 0,
                    beg: 4,
                    end: 5,
                },
            },
            Token {
                symbol: Number(2.0),
                span: TokenSpan {
                    line: 0,
                    beg: 6,
                    end: 7,
                },
            },
            Token {
                symbol: Position,
                span: TokenSpan {
                    line: 0,
                    beg: 8,
                    end: 11,
                },
            },
            Token {
                symbol: Number(1.0),
                span: TokenSpan {
                    line: 0,
                    beg: 12,
                    end: 13,
                },
            },
            Token {
                symbol: Comment("# a comment!"),
                span: TokenSpan {
                    line: 1,
                    beg: 0,
                    end: 12,
                },
            },
            Token {
                symbol: Size,
                span: TokenSpan {
                    line: 2,
                    beg: 0,
                    end: 3,
                },
            },
            Token {
                symbol: String("something"),
                span: TokenSpan {
                    line: 2,
                    beg: 4,
                    end: 13,
                },
            },
            Token {
                symbol: TextLine(""),
                span: TokenSpan {
                    line: 3,
                    beg: 0,
                    end: 0,
                },
            },
            Token {
                symbol: TextLine("   a text line with \\:ge escaped  "),
                span: TokenSpan {
                    line: 4,
                    beg: 0,
                    end: 34,
                },
            },
            Token {
                symbol: TextLine(" and another line below"),
                span: TokenSpan {
                    line: 5,
                    beg: 0,
                    end: 23,
                },
            },
            Token {
                symbol: TextLine("        "),
                span: TokenSpan {
                    line: 6,
                    beg: 0,
                    end: 8,
                },
            },
        ];
        for (e1, e2) in tokens.iter().zip(res.iter()) {
            assert_eq!(e1, e2, "{:?} vs {:?}", tokens, res);
        }
    }
}
