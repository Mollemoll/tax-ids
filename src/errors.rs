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

impl std::error::Error for ValidationError {}