use calibre_web_rust::error::{AppError, AppResult};
use std::io;

#[test]
fn test_io_error_conversion() {
    let io_err = io::Error::new(io::ErrorKind::NotFound, "file not found");
    let app_err: AppError = io_err.into();
    assert!(matches!(app_err, AppError::Io(_)));
}

#[test]
fn test_error_display() {
    let err = AppError::NotFound("Book".to_string());
    let display = format!("{}", err);
    assert!(display.contains("Book not found"));
}
