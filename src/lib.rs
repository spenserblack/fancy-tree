use std::error::Error;
pub use tree::Tree;

pub mod cli;
pub mod color;
pub mod colors;
pub mod config;
mod git;
pub mod icons;
pub mod lua;
pub mod sorting;
pub mod tree;

/// The standard result type.
pub type Result<T = (), E = Box<dyn Error>> = core::result::Result<T, E>;
