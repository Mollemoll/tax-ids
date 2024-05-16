use std::collections::HashMap;
use lazy_static::lazy_static;

use roxmltree;
use serde_json::json;

use crate::errors::VerificationError;
use crate::TaxId;
use crate::verification::{Verification, VerificationResponse, VerificationStatus, UnavailableReason, Verifier};
use crate::verification::UnavailableReason::{*};

// INFO(2024-05-08 mollemoll):
// Data from Vies
// https://ec.europa.eu/taxation_customs/vies/checkVatService.wsdl

static URI: &'static str = "http://ec.europa.eu/taxation_customs/vies/services/checkVatService";
static ENVELOPE: &'static str = "
<soapenv:Envelope xmlns:soapenv=\"http://schemas.xmlsoap.org/soap/envelope/\" xmlns:v1=\"http://schemas.conversesolutions.com/xsd/dmticta/v1\">
    <soapenv:Header/>
    <soapenv:Body>
        <checkVat xmlns=\"urn:ec.europa.eu:taxud:vies:services:checkVat:types\">
            <countryCode>{country}</countryCode>
            <vatNumber>{number}</vatNumber>
        </checkVat>
    </soapenv:Body>
</soapenv:Envelope>
";

lazy_static! {
    pub static ref FAULT_MAP: HashMap<&'static str, UnavailableReason> = {
        let mut m = HashMap::new();
        m.insert("SERVICE_UNAVAILABLE", ServiceUnavailable);
        m.insert("MS_UNAVAILABLE", ServiceUnavailable);
        // Not implemented: 'INVALID_REQUESTER_INFO'
        m.insert("TIMEOUT", Timeout);
        m.insert("VAT_BLOCKED", Block);
        m.insert("IP_BLOCKED", Block);
        m.insert("GLOBAL_MAX_CONCURRENT_REQ", RateLimit);
        m.insert("GLOBAL_MAX_CONCURRENT_REQ_TIME", RateLimit);
        m.insert("MS_MAX_CONCURRENT_REQ", RateLimit);
        m.insert("MS_MAX_CONCURRENT_REQ_TIME", RateLimit);
        m
    };
}

#[derive(Debug)]
pub struct Vies;

impl Vies {
    fn xml_to_hash(xml: &roxmltree::Document) -> HashMap<String, Option<String>> {
        let mut hash = HashMap::new();
        let tags_to_exclude = ["Body", "Envelope", "Fault"];

        for node in xml.descendants() {
            let tag_name = node.tag_name().name();
            if tag_name.trim().is_empty() || tags_to_exclude.contains(&tag_name) {
                continue;
            }

            if let Some(text) = node.text() {
                // Absence of data is represented by "---" in VIES
                if text == "---" {
                    hash.insert(tag_name.to_string(), None);
                } else {
                    hash.insert(tag_name.to_string(), Some(text.to_string()));
                }
            }
        }

        hash
    }
}

impl Verifier for Vies {
    fn make_request(&self, tax_id: &TaxId) -> Result<VerificationResponse, VerificationError> {
        let client = reqwest::blocking::Client::new();
        let body = ENVELOPE
            .replace("{country}", tax_id.tax_country_code())
            .replace("{number}", tax_id.local_value());
        let res = client
            .post(URI)
            .header("Content-Type", "text/xml")
            .body(body)
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
        let doc = roxmltree::Document::parse(response.body()).map_err(VerificationError::XmlParsingError)?;
        let hash = Vies::xml_to_hash(&doc);
        let fault_string = hash.get("faultstring")
            .and_then(|x| x.as_deref());

        let verification_status = match fault_string {
            Some(fault) => {
                match FAULT_MAP.get(fault){
                    Some(reason) => VerificationStatus::Unavailable(*reason),
                    None => {
                        return Err(VerificationError::UnexpectedResponse(
                            format!("Unknown fault code: {}", fault)
                        ));
                    }
                }
            }
            None => {
                let validity_value = hash.get("valid")
                    .and_then(|x| x.as_deref());

                match validity_value {
                    Some("true") => VerificationStatus::Verified,
                    Some("false") => VerificationStatus::Unverified,
                    None => return Err(
                        VerificationError::UnexpectedResponse(
                            "Missing valid field in VIES response".to_string()
                        )
                    ),
                    Some(_) => return Err(
                        VerificationError::UnexpectedResponse(
                            "Invalid value for valid field in VIES response".to_string()
                        )
                    )
                }
            }
        };

        Ok(
            Verification::new(
                verification_status,
                json!(hash)
            )
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_xml_to_hash() {
        let xml = r#"
            <soapenv:Envelope xmlns:soapenv="http://schemas.xmlsoap.org/soap/envelope/" xmlns:v1="http://schemas.conversesolutions.com/xsd/dmticta/v1">
                <soapenv:Header/>
                <soapenv:Body>
                    <checkVat xmlns="urn:ec.europa.eu:taxud:vies:services:checkVat:types">
                        <countryCode>SE</countryCode>
                        <vatNumber>123456789101</vatNumber>
                        <requestDate>2021-01-01+01:00</requestDate>
                        <valid>true</valid>
                        <name>Test Company</name>
                        <address>---</address>
                    </checkVat>
                </soapenv:Body>
            </soapenv:Envelope>
        "#;
        let doc = roxmltree::Document::parse(xml).unwrap();
        let hash = Vies::xml_to_hash(&doc);

        assert_eq!(hash.get("countryCode"), Some(&Some("SE".to_string())));
        assert_eq!(hash.get("vatNumber"), Some(&Some("123456789101".to_string())));
        assert_eq!(hash.get("requestDate"), Some(&Some("2021-01-01+01:00".to_string())));
        assert_eq!(hash.get("valid"), Some(&Some("true".to_string())));
        assert_eq!(hash.get("name"), Some(&Some("Test Company".to_string())));
        assert_eq!(hash.get("address"), Some(&None));
    }

    #[test]
    fn test_parse_response_verified() {
        let response = VerificationResponse::new(
            200,
            r#"
                    <soapenv:Envelope xmlns:soapenv="http://schemas.xmlsoap.org/soap/envelope/" xmlns:v1="http://schemas.conversesolutions.com/xsd/dmticta/v1">
                        <soapenv:Header/>
                        <soapenv:Body>
                            <checkVat xmlns="urn:ec.europa.eu:taxud:vies:services:checkVat:types">
                                <countryCode>SE</countryCode>
                                <vatNumber>123456789101</vatNumber>
                                <requestDate>2021-01-01+01:00</requestDate>
                                <valid>true</valid>
                                <name>Test Company</name>
                                <address>Test Address</address>
                            </checkVat>
                        </soapenv:Body>
                    </soapenv:Envelope>
                "#.to_string()
        );
        let verifier = Vies;
        let verification = verifier.parse_response(response).unwrap();

        assert_eq!(verification.status(), &VerificationStatus::Verified);
    }

    #[test]
    fn test_parse_response_unverified() {
        let response = VerificationResponse::new(
            200,
            r#"
                <soapenv:Envelope xmlns:soapenv="http://schemas.xmlsoap.org/soap/envelope/" xmlns:v1="http://schemas.conversesolutions.com/xsd/dmticta/v1">
                    <soapenv:Header/>
                    <soapenv:Body>
                        <checkVat xmlns="urn:ec.europa.eu:taxud:vies:services:checkVat:types">
                            <countryCode>SE</countryCode>
                            <vatNumber>123456789101</vatNumber>
                            <requestDate>2021-01-01+01:00</requestDate>
                            <valid>false</valid>
                            <name>Test Company</name>
                            <address>Test Address</address>
                        </checkVat>
                    </soapenv:Body>
                </soapenv:Envelope>
            "#.to_string()
        );
        let verifier = Vies;
        let verification = verifier.parse_response(response).unwrap();

        assert_eq!(verification.status(), &VerificationStatus::Unverified);
    }

    #[test]
    fn test_parse_response_unavailable() {
        let response = VerificationResponse::new(
            200,
            r#"
                <env:Envelope xmlns:env="http://schemas.xmlsoap.org/soap/envelope/">
                    <env:Header/>
                    <env:Body>
                        <env:Fault>
                            <faultcode>env:Server</faultcode>
                            <faultstring>MS_MAX_CONCURRENT_REQ</faultstring>
                        </env:Fault>
                    </env:Body>
                </env:Envelope>
            "#.to_string()
        );
        let verifier = Vies;
        let verification = verifier.parse_response(response).unwrap();

        assert_eq!(verification.status(), &VerificationStatus::Unavailable(UnavailableReason::RateLimit));
        assert_eq!(verification.data(), &json!({
            "faultcode": "env:Server",
            "faultstring": "MS_MAX_CONCURRENT_REQ"
        }));
    }

    #[test]
    fn test_parse_response_missing_valid_field() {
        let response = VerificationResponse::new(
            200,
            r#"
                <soapenv:Envelope xmlns:soapenv="http://schemas.xmlsoap.org/soap/envelope/" xmlns:v1="http://schemas.conversesolutions.com/xsd/dmticta/v1">
                    <soapenv:Header/>
                    <soapenv:Body>
                        <checkVat xmlns="urn:ec.europa.eu:taxud:vies:services:checkVat:types">
                            <countryCode>SE</countryCode>
                        </checkVat>
                    </soapenv:Body>
                </soapenv:Envelope>
            "#.to_string()
        );
        let verifier = Vies;
        let verification = verifier.parse_response(response);

        match verification {
            Err(VerificationError::UnexpectedResponse(msg)) => {
                assert_eq!(msg, "Missing valid field in VIES response");
            }
            _ => panic!("Expected UnexpectedResponse error"),
        }
    }

    #[test]
    fn test_parse_response_invalid_validity_value() {
        let response = VerificationResponse::new(
            200,
            r#"
                <soapenv:Envelope xmlns:soapenv="http://schemas.xmlsoap.org/soap/envelope/" xmlns:v1="http://schemas.conversesolutions.com/xsd/dmticta/v1">
                    <soapenv:Header/>
                    <soapenv:Body>
                        <checkVat xmlns="urn:ec.europa.eu:taxud:vies:services:checkVat:types">
                            <valid>invalid value</valid>
                        </checkVat>
                    </soapenv:Body>
                </soapenv:Envelope>
            "#.to_string()
        );
        let verifier = Vies;
        let verification = verifier.parse_response(response);

        match verification {
            Err(VerificationError::UnexpectedResponse(msg)) => {
                assert_eq!(msg, "Invalid value for valid field in VIES response");
            }
            _ => panic!("Expected UnexpectedResponse error"),
        }
    }
}
