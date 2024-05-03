use std::error::Error;
use std::fmt;

#[derive(Debug)]
pub struct ValidationError {
    pub message: String,
}

impl ValidationError {
    pub fn new(message: &str) -> ValidationError {
        ValidationError {
            message: message.to_string(),
        }
    }
}

impl fmt::Display for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl Error for ValidationError {}

#[derive(Debug)]
pub enum VerificationError {
    HttpError(reqwest::Error),
    ParsingError(roxmltree::Error),
}

impl fmt::Display for VerificationError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            VerificationError::HttpError(err) => write!(f, "HTTP client error: {}", err),
            VerificationError::ParsingError(err) => write!(f, "Parsing error: {}", err),
        }
    }
}

impl Error for VerificationError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            VerificationError::HttpError(err) => Some(err),
            VerificationError::ParsingError(err) => Some(err),
        }
    }
}
