//! Calibre-Web Rust Rewrite

pub mod config;
pub mod error;

pub use error::{AppError, AppResult};

pub mod infrastructure;
pub mod domain;
pub mod web;
