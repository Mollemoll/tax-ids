use std::error::Error;
use std::fmt::Debug;

#[derive(thiserror::Error, Debug, PartialEq)]
pub enum ValidationError {
    #[error("Country code {0} is not supported")]
    /// The country code is not supported
    UnsupportedCountryCode(String),

    #[error("Invalid syntax")]
    /// The syntax of the tax id is invalid for the given country
    InvalidSyntax,
}

#[derive(thiserror::Error)]
pub enum VerificationError {
    #[error("HTTP client error: {0}")]
    HttpError(#[from] reqwest::Error),

    #[error("JSON parsing error: {0}")]
    JsonParsingError(#[source] serde_json::Error),

    #[error("Unexpected response: {0}")]
    UnexpectedResponse(String),

    #[error("Unexpected status code: {0}")]
    UnexpectedStatusCode(u16),

    #[cfg(any(feature = "eu_vat", feature = "ch_vat"))]
    #[error("XML parsing error: {0}")]
    XmlParsingError(#[from] roxmltree::Error),
}

impl Debug for VerificationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "{}", self)?;
        if let Some(source) = self.source() {
            writeln!(f, "Caused by:\n\t{}", source)?;
        }
        Ok(())
    }
}

