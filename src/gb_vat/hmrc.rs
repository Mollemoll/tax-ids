use serde_json::json;
use crate::errors::VerificationError;
use crate::tax_id::TaxId;
use crate::verification::{Verification, VerificationStatus, Verifier};

static BASE_URI: &'static str = "https://api.service.hmrc.gov.uk/organisations/vat/check-vat-number/lookup";

pub struct HMRC;

impl Verifier for crate::gb_vat::hmrc::HMRC {
    fn make_request(&self, tax_id: &TaxId) -> Result<String, VerificationError> {
        let client = reqwest::blocking::Client::new();
        let res = client
            .get(format!("{}/{}", BASE_URI, tax_id.local_value()))
            .header("Accept", "application/vnd.hmrc.1.0+json")
            .send()
            .map_err(VerificationError::HttpError)?
            .text()
            .map_err(VerificationError::HttpError)?;

        Ok(res)
    }

    fn parse_response(&self, response: String) -> Result<Verification, VerificationError> {
        let v: serde_json::Value = serde_json::from_str(&response)
            .map_err(VerificationError::JSONParsingError)?;
        let hash = v.as_object().unwrap();

        let fault = hash.get("code").and_then(|v| v.as_str());

        let verification_result = match fault {
            None => {
                Verification::new(
                    VerificationStatus::Verified,
                    json!(hash.get("target"))
                )
            },
            Some(fault_code) if fault_code == "NOT_FOUND" => {
                Verification::new(
                    VerificationStatus::Unverified,
                    json!(hash)
                )
            },
            Some(_) => {
                Verification::new(
                    VerificationStatus::Unavailable,
                    json!(hash)
                )
            },
        };

        Ok(verification_result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_response_verified() {
        let response = r#"{
            "target": {
                "name": "VIRGIN ATLANTIC AIRWAYS LTD",
                "vatNumber": "425216184",
                "address": {
                    "line1": "THE VHQ",
                    "line2": "FLEMING WAY",
                    "line3": "CRAWLEY",
                    "line4": "WEST SUSSEX",
                    "postcode": "RH10 9DF",
                    "countryCode": "GB"
                }
            },
            "processingDate": "2024-05-06T09:18:58+01:00"
        }"#;

        let verifier = HMRC;
        let verification = verifier.parse_response(response.to_string()).unwrap();

        assert_eq!(*verification.status(), VerificationStatus::Verified);
        assert_eq!(verification.data(), &json!({
            "name": "VIRGIN ATLANTIC AIRWAYS LTD",
            "vatNumber": "425216184",
            "address": {
                "line1": "THE VHQ",
                "line2": "FLEMING WAY",
                "line3": "CRAWLEY",
                "line4": "WEST SUSSEX",
                "postcode": "RH10 9DF",
                "countryCode": "GB"
            }
        }));
    }

    #[test]
    fn test_parse_response_unverified() {
        let response = r#"{
            "code": "NOT_FOUND",
            "reason": "targetVrn does not match a registered company"
        }"#;

        let verifier = HMRC;
        let verification = verifier.parse_response(response.to_string()).unwrap();

        assert_eq!(*verification.status(), VerificationStatus::Unverified);
        assert_eq!(verification.data().get("code").unwrap(), "NOT_FOUND");
        assert_eq!(verification.data().get("reason").unwrap(), "targetVrn does not match a registered company");
    }

    #[test]
    fn test_parse_response_unavailable() {
        let response = r#"{
            "code": "SERVER_ERROR"
        }"#;

        let verifier = HMRC;
        let verification = verifier.parse_response(response.to_string()).unwrap();

        assert_eq!(*verification.status(), VerificationStatus::Unavailable);
        assert_eq!(verification.data().get("code").unwrap(), "SERVER_ERROR");
    }
}