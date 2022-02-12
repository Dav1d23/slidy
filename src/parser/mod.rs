pub mod lexer;
pub mod tokenizer;
pub mod utils;

use std::error::Error;
use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;
use std::path::Path;

use crate::slideshow::Slideshow;

/// Create the slides.
fn parse_text(
    inp: &str,
    base_folder: &Path,
) -> Result<Slideshow, Box<dyn Error + 'static>> {
    // Build the tokens.
    let tokens = tokenizer::tokenizer(inp);
    // Feed the lexer with the tokens.
    let mut tp = lexer::Lexer::new(base_folder);
    tp.read_tokens(&tokens)?;
    // Take the slideshow out of the lexer.
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
    let mut reader = BufReader::new(file);
    let base_folder = path
        .parent()
        .ok_or("Unable to find the parent: is this root already?")?;
    // Read the whole file to a String.
    let mut file_to_string = String::new();
    reader.read_to_string(&mut file_to_string)?;
    let slides = parse_text(file_to_string.as_str(), base_folder)?;
    Ok(slides)
}

#[cfg(test)]
mod test {
    use super::*;
    use serde_json;
    use std::fs::File;
    use std::io::BufReader;
    use std::path::PathBuf;

    use crate::slideshow::SectionMain;

    /// Load and a file and check its existence.
    macro_rules! load_exists {
        ($f:expr) => {{
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
    fn test_import_ko_file_not_there() {
        let example = ":im ./non_existing_file.txt";
        let base_path = PathBuf::from("");
        assert!(parse_text(example, &base_path).is_err());
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

        assert_eq!(slideshow.slides.len(), 1);
    }

    #[test]
    /// Verify the example in the README works.
    /// Note that if this test fails, we need to change the README as well!
    fn test_readme_example() {
        let example = r#"
# Comments are ignored
:ge :bc green :fc yellow :sz 16

:sl
:tb :sz 40 :fc red
BIG TITLE
:tb
A line
  Note that it starts just below the title!

:sl
:tb :sz 10 :fc blue
Small title now
:tb
But again, the line is just below the title

:sl
:tb :ps 0.3 0.3 :fc fuchsia
 We can also
center the text
 manually!
"#;

        let p = Path::new("");
        let slides = parse_text(example, &p)
            .expect("should be able to create the slides.");
        assert_eq!(slides.slides.len(), 3);
    }

    #[test]
    fn test_maintain_whitespace() {
        let example = r#"
:sl :tb
    4 whitespaces before
"#;

        let p = Path::new("");
        let slides = parse_text(example, &p)
            .expect("should be able to create the slides.");

        let text = slides.slides.get(0).and_then(|slide| {
            slide.sections.get(0).and_then(|section| {
                if let Some(SectionMain::Text(sec_text)) = &section.sec_main {
                    Some(&sec_text.text)
                } else {
                    None
                }
            })
        });
        assert_eq!(
            &text.expect("text must be filled in").as_str(),
            &"    4 whitespaces before\n"
        );
    }
}
