#![warn(clippy::nursery)]
#![warn(clippy::pedantic)]
#![warn(missing_docs)]
#![allow(clippy::missing_panics_doc)]
#![allow(clippy::missing_errors_doc)]

/*!
# Slidy: presentations, for developers

Let's be real: presentation's software is made for marketers. You can add
effects, slide's transitions, and funny jokes.

But sometimes you're interested in just writing some text in a simple text file
and version control it! Or maybe, you just want to share your slides and the
program to run it - together!

## Slidy, as a library

### The slidy language

Slidy comes with a simple language to define slides.

```
use slidy::parser::parse_text;
use std::path::Path;

let text = r#"
:sl
:tb :sz 40 :fc red
Big, red title
:tb
And a line
"#;

let p = Path::new("./");

let slides = parse_text(text, p).unwrap();

println!("{:?}", slides);
```

Since the [Slideshow](`crate::slideshow::Slideshow`) struct implements
`serde`'s `Serialize` and `Deserialize`, slides can also be defined in other
formats. A json example is provided.

### The available backends

Slides are meaningless without a way to see them. `Slidy` comes with 2 provided
backends: a richer and full-features viewer based on SDL2, and a barebone
terminal one based on Crossterm.

An easy way to use them is described below.

```no_run
# use slidy::parser::parse_text;
# use std::path::Path;
#
# let text = r#"
# :sl
# :tb :sz 40 :fc red
# Big, red title
# :tb
# And a line
# "#;
#
# let p = Path::new("./");
#
# let slides = parse_text(text, p).unwrap();

// let slides = ...

let mut backend = slidy::backends::get_backend(&slidy::backends::Backends::Crossterm);
let mut context = backend.get_context();

// Here, an event loop should be used, but we skip that in this example.
context.set_slides(slides)

// We would then manage the inputs ...
// ... context.manage_inputs() ...

// ... And finally render everything!
// context.render();
```

Just check the provided executable or the examples for more details.

# Slidy, as an executable

This crate also comes with an executable, which provides an easy way to read
the slides written with the slidy language.

*/

/// The available backends.
pub mod backends;
/// The parser for `slidy`'s language.
pub mod parser;
/// The slideshow structure.
pub mod slideshow;
