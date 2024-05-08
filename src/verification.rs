use chrono::prelude::*;
use crate::tax_id::TaxId;
use crate::errors::VerificationError;

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

#[derive(Debug, PartialEq)]
pub enum VerificationStatus {
    Verified,
    Unverified,
    Unavailable,
}

#[derive(Debug)]
pub struct Verification {
    performed_at: DateTime<Local>,
    status: VerificationStatus,
    data: serde_json::Value,
}

impl Verification {
    pub fn new(status: VerificationStatus, data: serde_json::Value) -> Verification {
        Verification {
            performed_at: Local::now(),
            status,
            data,
        }
    }

    pub fn status(&self) -> &VerificationStatus { &self.status }
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
        let tax_id = TaxId::new("SE123456789101").unwrap();
        let verifier = TestVerifier;
        let verification = verifier.verify(&tax_id).unwrap();
        assert_eq!(verification.status(), &VerificationStatus::Verified);
        assert_eq!(verification.performed_at.date_naive(), Local::now().date_naive());
        assert_eq!(verification.data().get("key").unwrap(), "value");
    }
}