use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("URL parser error: {0}")]
    UrlParseError(#[from] url::ParseError),

    #[error("GLib Error: {0}")]
    GLibError(#[from] glib::error::Error),

    #[error("Input/Output error: {0}")]
    IOError(#[from] std::io::Error),

    #[error("Network error: {0}")]
    NetworkError(#[from] isahc::Error),
}
