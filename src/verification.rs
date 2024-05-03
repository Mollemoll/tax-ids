use chrono::prelude::*;
use crate::tax_id::TaxId;
use crate::errors::VerificationError;

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
}

impl Verification {
    pub fn new(status: VerificationStatus) -> Verification {
        Verification {
            performed_at: Local::now(),
            status,
        }
    }

    pub fn status(&self) -> &VerificationStatus { &self.status }
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
        let verification = Verification::new(VerificationStatus::Verified);
        assert_eq!(*verification.status(), VerificationStatus::Verified);
        assert_eq!(verification.performed_at.date_naive(), Local::now().date_naive());
    }

    struct TestVerifier;

    impl Verifier for TestVerifier {
        fn make_request(&self, _tax_id: &TaxId) -> Result<String, VerificationError> {
            Ok("test".to_string())
        }

        fn parse_response(&self, response: String) -> Result<Verification, VerificationError> {
            if response == "test" {
                Ok(Verification::new(VerificationStatus::Verified))
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
    }
}