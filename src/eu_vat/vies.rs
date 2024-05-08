use std::collections::HashMap;
use anyhow::{bail, Result};
use roxmltree;
use serde_json::json;
use crate::verification::{Verifier, Verification, VerificationStatus, VerificationResponse};
use crate::tax_id::TaxId;

// INFO(2024-05-08 mollemoll):
// Data from VIES
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

pub struct VIES;

impl VIES {
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

impl Verifier for VIES {
    fn make_request(&self, tax_id: &TaxId) -> Result<VerificationResponse> {
        let client = reqwest::blocking::Client::new();
        let body = ENVELOPE
            .replace("{country}", tax_id.tax_country_code())
            .replace("{number}", tax_id.local_value());
        let res = client
            .post(URI)
            .header("Content-Type", "text/xml")
            .body(body)
            .send()?;

        Ok(
            VerificationResponse::new(
                res.status().as_u16(),
                res.text()?
            )
        )
    }

    fn parse_response(&self, response: VerificationResponse) -> Result<Verification> {
        let doc = roxmltree::Document::parse(response.body())?;
        let hash = VIES::xml_to_hash(&doc);
        let fault_string = hash.get("faultstring")
            .and_then(|x| x.as_deref());

        let verification_status = match fault_string {
            Some(_) => VerificationStatus::Unavailable,
            None => {
                let validity_value = hash.get("valid")
                    .and_then(|x| x.as_deref());

                match validity_value {
                    Some("true") => VerificationStatus::Verified,
                    Some("false") => VerificationStatus::Unverified,
                    None => bail!("Missing valid field in VIES response"),
                    Some(_) => bail!("Invalid value for valid field in VIES response"),
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
        let hash = VIES::xml_to_hash(&doc);

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
        let verifier = VIES;
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
        let verifier = VIES;
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
        let verifier = VIES;
        let verification = verifier.parse_response(response).unwrap();

        assert_eq!(verification.status(), &VerificationStatus::Unavailable);
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
        let verifier = VIES;
        let verification = verifier.parse_response(response);

        assert!(verification.is_err());
        assert_eq!(verification.unwrap_err().to_string(), "Missing valid field in VIES response")
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
        let verifier = VIES;
        let verification = verifier.parse_response(response);

        assert!(verification.is_err());
        assert_eq!(verification.unwrap_err().to_string(), "Invalid value for valid field in VIES response")
    }
}
