#![allow(clippy::pedantic)]

// Import external references
extern crate env_logger;
#[macro_use]
extern crate log;

// Re-export modules.
pub mod parser;
pub mod slideshow;
pub mod backend_sdl;
