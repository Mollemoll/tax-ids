use chrono::prelude::*;
use crate::errors::VerificationError;
use crate::TaxId;

#[derive(Debug, PartialEq)]
pub struct VerificationResponse {
    status: u16,
    body: String,
}

impl VerificationResponse {
    pub fn new(status: u16, body: String) -> VerificationResponse {
        VerificationResponse {
            status,
            body,
        }
    }

    pub fn status(&self) -> u16 { self.status }
    pub fn body(&self) -> &str { &self.body }
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum VerificationStatus {
    /// Represents a successful verification where the government database confirmed the ID as legitimate.
    Verified,
    /// Represents an unsuccessful verification where the government database identified the ID as illegitimate.
    Unverified,
    /// Represents a case where verification was not possible due to certain reasons (e.g., government database was unavailable).
    Unavailable(UnavailableReason),
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum UnavailableReason {
    ServiceUnavailable,
    Timeout,
    Block,
    RateLimit,
}

#[derive(Debug, PartialEq)]
pub struct Verification {
    performed_at: DateTime<Local>,
    status: VerificationStatus,
    data: serde_json::Value,
}

impl Verification {
    #[doc(hidden)]
    pub fn new(status: VerificationStatus, data: serde_json::Value) -> Verification {
        Verification {
            performed_at: Local::now(),
            status,
            data,
        }
    }

    /// This VerificationStatus is what the crate user should use to determine how to proceed.
    ///
    /// A checkout example:
    /// - Enable/process the transaction upon `VerificationStatus::Verified`.
    /// - Block transaction/provide a validation msg upon `VerificationStatus::Unverified`.
    /// - Enable/process the transaction upon `VerificationStatus::Unavailable` but perform a
    ///     re-verification at a later stage.
    pub fn status(&self) -> &VerificationStatus { &self.status }
    /// Additional data selected by the crate owner from the government database response.
    /// This data can be used to provide more context about the verification.
    /// The data is in JSON format.
    ///
    /// Includes error details in case of an unsuccessful verification.
    ///
    /// Subject to change in future versions.
    pub fn data(&self) -> &serde_json::Value { &self.data }
}

pub trait Verifier {
    fn verify(&self, tax_id: &TaxId) -> Result<Verification, VerificationError> {
        let response = self.make_request(tax_id)?;
        let verification = self.parse_response(response)?;
        Ok(verification)
    }
    fn make_request(&self, tax_id: &TaxId) -> Result<VerificationResponse, VerificationError>;

    fn parse_response(&self, response: VerificationResponse) -> Result<Verification, VerificationError>;
}

#[cfg(test)]
mod tests {
    use serde_json::json;
    use super::*;

    #[test]
    fn test_new_verification() {
        let verification = Verification::new(
            VerificationStatus::Verified,
            json!({})
        );
        assert_eq!(verification.status(), &VerificationStatus::Verified);
        assert_eq!(verification.performed_at.date_naive(), Local::now().date_naive());
    }

    struct TestVerifier;

    impl Verifier for TestVerifier {
        fn make_request(&self, _tax_id: &TaxId) -> Result<VerificationResponse, VerificationError> {
            Ok(VerificationResponse::new(
                200,
                "test".to_string()
            ))
        }

        fn parse_response(&self, response: VerificationResponse) -> Result<Verification, VerificationError> {
            let data = json!({
                "key": "value"
            });

            if response.status() == 200 && response.body() == "test" {
                Ok(Verification::new(
                    VerificationStatus::Verified,
                    data
                ))
            } else { panic!("Unexpected response") }
        }
    }

    #[test]
    fn test_verify() {
        #[cfg(feature="eu_vat")]
        let value = "SE123456789101";
        #[cfg(feature="gb_vat")]
        let value = "GB123456789";
        #[cfg(feature="ch_vat")]
        let value = "CHE123456789";
        #[cfg(feature = "no_vat")]
        let value = "NO123456789";
        
        let tax_id = TaxId::new(value).unwrap();
        let verifier = TestVerifier;
        let verification = verifier.verify(&tax_id).unwrap();
        assert_eq!(verification.status(), &VerificationStatus::Verified);
        assert_eq!(verification.performed_at.date_naive(), Local::now().date_naive());
        assert_eq!(verification.data().get("key").unwrap(), "value");
    }
}