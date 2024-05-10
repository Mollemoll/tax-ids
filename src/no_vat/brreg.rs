use std::collections::HashMap;
use lazy_static::lazy_static;
use reqwest::header::{HeaderMap, HeaderValue, ACCEPT};
use serde_json::json;
use crate::verification::{Verifier, Verification, VerificationStatus, VerificationResponse};
use crate::tax_id::TaxId;
use crate::errors::VerificationError;
use crate::no_vat::NOVat;

// INFO(2024-05-08 mollemoll):
// Data from Brønnøysund Register Centre
// https://data.brreg.no/enhetsregisteret/oppslag/enheter
// https://data.brreg.no/enhetsregisteret/api/dokumentasjon/no/index.html#tag/Enheter/operation/hentEnhet

static BASE_URI: &'static str = "https://data.brreg.no/enhetsregisteret/api/enheter";

lazy_static! {
    pub static ref HEADERS: HeaderMap = {
        let mut headers = HeaderMap::new();
        headers.insert(ACCEPT, HeaderValue::from_static("application/vnd.brreg.enhetsregisteret.enhet.v2+json"));
        headers
    };

    pub static ref REQUIREMENTS_TO_BE_VALID : HashMap<&'static str, bool> = {
        let mut map = HashMap::new();
        map.insert("registrertIMvaregisteret", true); // Registered for VAT
        map.insert("konkurs", false); // In default?
        map.insert("underAvvikling", false); // In liquidation?
        map.insert("underTvangsavviklingEllerTvangsopplosning", false); // Forced liquidation?
        map
    };
}

pub struct BRReg;

impl BRReg {
    fn qualify(&self, hash: &serde_json::Map<String, serde_json::Value>) -> VerificationStatus {
        let mut valid = true;
        for (key, value) in REQUIREMENTS_TO_BE_VALID.iter() {
            if hash.contains_key(*key) && hash.get(*key).unwrap().as_bool().unwrap() == *value {
                continue;
            }
            valid = false;
            break;
        }

        if valid {
            VerificationStatus::Verified
        } else {
            VerificationStatus::Unverified
        }
    }
}

impl Verifier for BRReg {
    fn make_request(&self, tax_id: &TaxId) -> Result<VerificationResponse, VerificationError> {
        let client = reqwest::blocking::Client::new();
        let res = client
            .get(format!("{}/{}", BASE_URI, NOVat::extract_org_number(&NOVat, tax_id)))
            .headers(HEADERS.clone())
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
        match response.status() {
            404 | 410 => return Ok(
                Verification::new(
                    VerificationStatus::Unverified, json!({})
                )
            ),
            200 | 500 => {
                let v: serde_json::Value = serde_json::from_str(response.body())
                    .map_err(VerificationError::JSONParsingError)?;
                let hash = v.as_object().unwrap();

                if response.status() == 500 {
                    return Ok(Verification::new(VerificationStatus::Unavailable, json!(hash)));
                }

                Ok(
                    Verification::new(
                        self.qualify(hash),
                        json!(hash)
                    )
                )
            },
            _ => return Err(VerificationError::UnexpectedStatusCode(response.status())),
        }
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
                "organisasjonsnummer": "123456789",
                "navn": "Test Company AS",
                "registrertIMvaregisteret": true,
                "konkurs": false,
                "underAvvikling": false,
                "underTvangsavviklingEllerTvangsopplosning": false
            }"#.to_string()
        );

        let verifier = BRReg;
        let verification = verifier.parse_response(response).unwrap();
        assert_eq!(verification.status(), &VerificationStatus::Verified);
        assert_eq!(verification.data(), &json!({
            "organisasjonsnummer": "123456789",
            "navn": "Test Company AS",
            "registrertIMvaregisteret": true,
            "konkurs": false,
            "underAvvikling": false,
            "underTvangsavviklingEllerTvangsopplosning": false
        }));
    }

    #[test]
    fn test_parse_response_unverified_due_to_not_found() {
        let response = VerificationResponse::new(
            404,
            r#"{}"#.to_string()
        );

        let verifier = BRReg;
        let verification = verifier.parse_response(response).unwrap();
        assert_eq!(verification.status(), &VerificationStatus::Unverified);
        assert_eq!(verification.data(), &json!({}));
    }

    #[test]
    fn test_parse_response_unverified_due_to_qualification() {
        let response = VerificationResponse::new(
            200,
            r#"{
                "organisasjonsnummer": "123456789",
                "navn": "Test Company AS",
                "registrertIMvaregisteret": false,
                "konkurs": false,
                "underAvvikling": false,
                "underTvangsavviklingEllerTvangsopplosning": false
            }"#.to_string()
        );

        let verifier = BRReg;
        let verification = verifier.parse_response(response).unwrap();
        assert_eq!(verification.status(), &VerificationStatus::Unverified);
        assert_eq!(verification.data(), &json!({
            "organisasjonsnummer": "123456789",
            "navn": "Test Company AS",
            "registrertIMvaregisteret": false,
            "konkurs": false,
            "underAvvikling": false,
            "underTvangsavviklingEllerTvangsopplosning": false
        }));
    }

    #[test]
    fn test_parse_response_unverified_due_to_deleted() {
        let response = VerificationResponse::new(
            410,
            r#"
                {
                    "organisasjonsnummer": "123456789",
                    "slettedato": "2024-03-09",
                    "_links": {
                        "self": {}
                    }
                }
            "#.to_string()
        );

        let verifier = BRReg;
        let verification = verifier.parse_response(response).unwrap();
        assert_eq!(verification.status(), &VerificationStatus::Unverified);
        assert_eq!(verification.data(), &json!({}));
    }

    #[test]
    fn test_parse_response_unavailable() {
        let response = VerificationResponse::new(
            500,
            r#"
                {
                    "timestamp": "2024-01-05T07:36:21.523+0000",
                    "status": 500,
                    "error": "Internal Server Error",
                    "message": "Internal Server Error",
                    "path": "/enhetsregisteret/api/enheter",
                    "trace": "b94669c0-425a-4b6c-ab30-504de8d9c127"
                }
            "#.to_string()
        );

        let verifier = BRReg;
        let verification = verifier.parse_response(response).unwrap();
        assert_eq!(verification.status(), &VerificationStatus::Unavailable);
        assert_eq!(verification.data(), &json!({
            "timestamp": "2024-01-05T07:36:21.523+0000",
            "status": 500,
            "error": "Internal Server Error",
            "message": "Internal Server Error",
            "path": "/enhetsregisteret/api/enheter",
            "trace": "b94669c0-425a-4b6c-ab30-504de8d9c127"
        }));
    }

    #[test]
    fn test_parse_response_unexpected_status_code() {
        let response = VerificationResponse::new(
            204,
            "".to_string()
        );

        let verifier = BRReg;
        let verification = verifier.parse_response(response);
        assert_eq!(verification.is_err(), true);
        match verification {
            Err(VerificationError::UnexpectedStatusCode(code)) => {
                assert_eq!(code, 204);
            }
            _ => panic!("Expected UnexpectedStatusCode error"),
        }
    }
}