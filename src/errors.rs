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
    XmlParsingError(roxmltree::Error),
    JSONParsingError(serde_json::Error),
    UnexpectedResponse(String),
}

impl fmt::Display for VerificationError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            VerificationError::HttpError(err) => write!(f, "HTTP client error: {}", err),
            VerificationError::XmlParsingError(err) => write!(f, "XML parsing error: {}", err),
            VerificationError::JSONParsingError(err) => write!(f, "JSON parsing error: {}", err),
            VerificationError::UnexpectedResponse(msg) => write!(f, "Unexpected response: {}", msg),
        }
    }
}

impl Error for VerificationError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            VerificationError::HttpError(err) => Some(err),
            VerificationError::XmlParsingError(err) => Some(err),
            VerificationError::JSONParsingError(err) => Some(err),
            VerificationError::UnexpectedResponse(_) => None,
        }
    }
}
