#![warn(clippy::nursery)]
#![warn(clippy::pedantic)]
#![allow(clippy::missing_panics_doc)]
#![allow(clippy::missing_errors_doc)]

/*!
# Slidy: presentations, for developers.

Let's be real: presentation's software is made for marketers. You can add
effects, slide's transitions, and funny jokes.

But sometimes you're interested in just writing some text in a simple text file
and version control it! Or maybe, you just want to share your slides and the
program to run it - together!


Introducing `slidy`: a plain text slide format, and a viewer, that allow you to

*/

pub mod backends;
pub mod parser;
pub mod slideshow;
