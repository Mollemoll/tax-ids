use std::collections::HashMap;
use lazy_static::lazy_static;
use reqwest::header::{HeaderMap, HeaderValue, ACCEPT};
use serde_json::{json, Value};
use crate::verification::{Verifier, Verification, VerificationStatus, VerificationResponse};
use crate::verification::VerificationStatus::{*};
use crate::errors::VerificationError;
use crate::no_vat::NoVat;
use crate::no_vat::translator::translate_keys;
use crate::TaxId;
use crate::verification::UnavailableReason::{ServiceUnavailable};

// INFO(2024-05-08 mollemoll):
// Data from Brønnøysund Register Centre
// https://data.brreg.no/enhetsregisteret/oppslag/enheter
// https://data.brreg.no/enhetsregisteret/api/dokumentasjon/no/index.html#tag/Enheter/operation/hentEnhet

static BASE_URI: &'static str = "https://data.brreg.no/enhetsregisteret/api/enheter";

lazy_static! {
    #[derive(Debug)]
    pub static ref HEADERS: HeaderMap = {
        let mut headers = HeaderMap::new();
        headers.insert(ACCEPT, HeaderValue::from_static("application/vnd.brreg.enhetsregisteret.enhet.v2+json"));
        headers
    };

    #[derive(Debug)]
    pub static ref REQUIREMENTS_TO_BE_VALID : HashMap<&'static str, bool> = {
        let mut map = HashMap::new();
        map.insert("registeredInVatRegister", true); // Registered for VAT
        map.insert("bankruptcy", false); // In default?
        map.insert("underLiquidation", false);
        map.insert("underForcedLiquidation", false); // Forced liquidation?
        map
    };
}

#[derive(Debug)]
pub struct BrReg;

impl BrReg {
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
            Verified
        } else {
            Unverified
        }
    }
}

impl Verifier for BrReg {
    fn make_request(&self, tax_id: &TaxId) -> Result<VerificationResponse, VerificationError> {
        let client = reqwest::blocking::Client::new();
        let res = client
            .get(format!("{}/{}", BASE_URI, NoVat::extract_org_number(&NoVat, tax_id)))
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
                    Unverified, json!({})
                )
            ),
            200 | 500 => {
                let mut v: Value = serde_json::from_str(response.body())
                    .map_err(VerificationError::JsonParsingError)?;
                translate_keys(&mut v);
                let hash = v.as_object().unwrap();

                if response.status() == 500 {
                    return Ok(
                        Verification::new(
                            Unavailable(ServiceUnavailable),
                            json!(hash)
                        )
                    );
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

    #[cfg(feature = "no_vat")]
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
                "underTvangsavviklingEllerTvangsopplosning": false,
                "forretningsadresse": {
                    "land": "Norge",
                    "landkode": "NO",
                    "postnummer": "0151",
                    "poststed": "OSLO",
                    "adresse": [
                        "Grev Wedels plass 9"
                    ],
                    "kommune": "OSLO",
                    "kommunenummer": "0301"
                }
            }"#.to_string()
        );

        let verifier = BrReg;
        let verification = verifier.parse_response(response).unwrap();
        assert_eq!(verification.status(), &Verified);
        assert_eq!(verification.data(), &json!({
            "organizationNumber": "123456789",
            "name": "Test Company AS",
            "registeredInVatRegister": true,
            "bankruptcy": false,
            "underLiquidation": false,
            "underForcedLiquidation": false,
            "businessAddress": {
                "country": "Norge",
                "countryCode": "NO",
                "postalCode": "0151",
                "city": "OSLO",
                "street": [
                    "Grev Wedels plass 9"
                ],
                "municipality": "OSLO",
                "municipalityCode": "0301"
            },
        }));
    }

    #[cfg(feature = "no_vat")]
    #[test]
    fn test_parse_response_unverified_due_to_not_found() {
        let response = VerificationResponse::new(
            404,
            r#"{}"#.to_string()
        );

        let verifier = BrReg;
        let verification = verifier.parse_response(response).unwrap();
        assert_eq!(verification.status(), &Unverified);
        assert_eq!(verification.data(), &json!({}));
    }

    #[cfg(feature = "no_vat")]
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

        let verifier = BrReg;
        let verification = verifier.parse_response(response).unwrap();
        assert_eq!(verification.status(), &Unverified);
        assert_eq!(verification.data(), &json!({
            "organizationNumber": "123456789",
            "name": "Test Company AS",
            "registeredInVatRegister": false,
            "bankruptcy": false,
            "underLiquidation": false,
            "underForcedLiquidation": false
        }));
    }

    #[cfg(feature = "no_vat")]
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

        let verifier = BrReg;
        let verification = verifier.parse_response(response).unwrap();
        assert_eq!(verification.status(), &Unverified);
        assert_eq!(verification.data(), &json!({}));
    }

    #[cfg(feature = "no_vat")]
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

        let verifier = BrReg;
        let verification = verifier.parse_response(response).unwrap();
        assert_eq!(verification.status(), &Unavailable(ServiceUnavailable));
        assert_eq!(verification.data(), &json!({
            "timestamp": "2024-01-05T07:36:21.523+0000",
            "status": 500,
            "error": "Internal Server Error",
            "message": "Internal Server Error",
            "path": "/enhetsregisteret/api/enheter",
            "trace": "b94669c0-425a-4b6c-ab30-504de8d9c127"
        }));
    }

    #[cfg(feature = "no_vat")]
    #[test]
    fn test_parse_response_unexpected_status_code() {
        let response = VerificationResponse::new(
            204,
            "".to_string()
        );

        let verifier = BrReg;
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
