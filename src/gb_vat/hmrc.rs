use serde_json::json;
use crate::errors::VerificationError;
use crate::TaxId;
use crate::verification::{Verification, VerificationResponse, VerificationStatus, Verifier};

// INFO(2024-05-08 mollemoll):
// Data from HMRC
// https://www.tax.service.gov.uk/check-vat-number/enter-vat-details
// https://developer.service.hmrc.gov.uk/api-documentation/docs/api/service/vat-api/1.0

static BASE_URI: &'static str = "https://api.service.hmrc.gov.uk/organisations/vat/check-vat-number/lookup";

#[derive(Debug)]
pub struct Hmrc;

impl Verifier for Hmrc {
    fn make_request(&self, tax_id: &TaxId) -> Result<VerificationResponse, VerificationError> {
        let client = reqwest::blocking::Client::new();
        let res = client
            .get(format!("{}/{}", BASE_URI, tax_id.local_value()))
            .header("Accept", "application/vnd.hmrc.1.0+json")
            .send()
            .map_err(VerificationError::HttpError)?;

        Ok(
            VerificationResponse::new(
                res.status().as_u16(),
                res.text().map_err(VerificationError::HttpError)?
            )
        )
    }

    fn parse_response(&self, response: VerificationResponse) -> Result<Verification, VerificationError> {
        let v: serde_json::Value = serde_json::from_str(response.body())
            .map_err(VerificationError::JsonParsingError)?;
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
        let response = VerificationResponse::new(
            200,
            r#"{
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
            }"#.to_string()
        );

        let verifier = Hmrc;
        let verification = verifier.parse_response(response).unwrap();

        assert_eq!(verification.status(), &VerificationStatus::Verified);
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
        let response = VerificationResponse::new(
            404,
            r#"{
                "code": "NOT_FOUND",
                "reason": "targetVrn does not match a registered company"
            }"#.to_string()
        );

        let verifier = Hmrc;
        let verification = verifier.parse_response(response).unwrap();

        assert_eq!(verification.status(), &VerificationStatus::Unverified);
        assert_eq!(verification.data(), &json!({
            "code": "NOT_FOUND",
            "reason": "targetVrn does not match a registered company"
        }));
    }

    #[test]
    fn test_parse_response_unavailable() {
        let response = VerificationResponse::new(
            500,
            r#"{
            "code": "SERVER_ERROR"
            }"#.to_string()
        );

        let verifier = Hmrc;
        let verification = verifier.parse_response(response).unwrap();

        assert_eq!(verification.status(), &VerificationStatus::Unavailable);
        assert_eq!(verification.data().get("code").unwrap(), "SERVER_ERROR");
    }
}
