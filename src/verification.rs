use chrono::prelude::*;
use crate::tax_id::TaxId;
use crate::errors::VerificationError;
use std::collections::HashMap;

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
    data: HashMap<String, String>,
}

impl Verification {
    pub fn new(status: VerificationStatus, data: HashMap<String, String>) -> Verification {
        Verification {
            performed_at: Local::now(),
            status,
            data,
        }
    }

    pub fn status(&self) -> &VerificationStatus { &self.status }
    pub fn data(&self) -> &HashMap<String, String> { &self.data }
}

pub trait Verifier {
    fn verify(&self, tax_id: &TaxId) -> Result<Verification, VerificationError> {
        let request = self.make_request(tax_id)?;
        let verification = self.parse_response(request)?;
        Ok(verification)
    }
    fn make_request(&self, tax_id: &TaxId) -> Result<String, VerificationError>;

    fn parse_response(&self, _response: String) -> Result<Verification, VerificationError>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_verification() {
        let verification = Verification::new(
            VerificationStatus::Verified,
            HashMap::new(),
        );
        assert_eq!(*verification.status(), VerificationStatus::Verified);
        assert_eq!(verification.performed_at.date_naive(), Local::now().date_naive());
    }

    struct TestVerifier;

    impl Verifier for TestVerifier {
        fn make_request(&self, _tax_id: &TaxId) -> Result<String, VerificationError> {
            Ok("test".to_string())
        }

        fn parse_response(&self, response: String) -> Result<Verification, VerificationError> {
            let mut hash = HashMap::new();
            hash.insert("key".to_string(), "value".to_string());

            if response == "test" {
                Ok(Verification::new(
                    VerificationStatus::Verified,
                    hash
                ))
            } else { panic!("Unexpected response") }
        }
    }

    #[test]
    fn test_verify() {
        let tax_id = TaxId::new("SE123456789101").unwrap();
        let verifier = TestVerifier;
        let verification = verifier.verify(&tax_id).unwrap();
        assert_eq!(*verification.status(), VerificationStatus::Verified);
        assert_eq!(verification.performed_at.date_naive(), Local::now().date_naive());
        assert_eq!(verification.data().get("key").unwrap(), "value");
    }
}