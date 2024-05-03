use std::collections::HashMap;
use roxmltree;
use crate::verification::{Verifier, Verification, VerificationStatus};
use crate::tax_id::TaxId;
use crate::errors::VerificationError;

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
    fn xml_to_hash(xml: &roxmltree::Document) -> HashMap<String, String> {
        let mut hash = HashMap::new();

        for node in xml.descendants() {
            let tag_name = node.tag_name().name();
            if let Some(text) = node.text() {
                hash.insert(tag_name.to_string(), text.to_string());
            }
        }

        hash
    }
}

impl Verifier for VIES {
    async fn make_request(&self, tax_id: TaxId) -> Result<String, VerificationError> {
        let client = reqwest::Client::new();
        let body = ENVELOPE
            .replace("{country}", tax_id.tax_country_code())
            .replace("{number}", tax_id.local_value());
        let res = client
            .post(URI)
            .header("Content-Type", "text/xml")
            .body(body)
            .send()
            .await
            .map_err(VerificationError::HttpError)?
            .text()
            .await
            .map_err(VerificationError::HttpError)?;

        Ok(res)
    }

    fn parse_response(&self, response: String) -> Result<Verification, VerificationError> {
        let doc = roxmltree::Document::parse(&response).map_err(VerificationError::ParsingError)?;
        let hash = VIES::xml_to_hash(&doc);
        let fault = hash.get("faultcode");

        if fault.is_some() {
            return Ok(
                Verification::new(
                    VerificationStatus::Unavailable
                )
            );
        } else {
            Ok(
                Verification::new(
                    if hash.get("valid").unwrap() == "true" {
                        VerificationStatus::Verified
                    } else if hash.get("valid").unwrap() == "false" {
                        VerificationStatus::Unverified
                    } else {
                        panic!("Unexpected response")
                    }
                )
            )
        }
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
                        <address>Test Address</address>
                    </checkVat>
                </soapenv:Body>
            </soapenv:Envelope>
        "#;
        let doc = roxmltree::Document::parse(xml).unwrap();
        let hash = VIES::xml_to_hash(&doc);

        assert_eq!(hash.get("countryCode"), Some(&"SE".to_string()));
        assert_eq!(hash.get("vatNumber"), Some(&"123456789101".to_string()));
        assert_eq!(hash.get("requestDate"), Some(&"2021-01-01+01:00".to_string()));
        assert_eq!(hash.get("valid"), Some(&"true".to_string()));
        assert_eq!(hash.get("name"), Some(&"Test Company".to_string()));
        assert_eq!(hash.get("address"), Some(&"Test Address".to_string()));
    }

    #[test]
    fn test_parse_response_verified() {
        let response = r#"
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
        "#;
        let verifier = VIES;
        let verification = verifier.parse_response(response.to_string()).unwrap();

        assert_eq!(*verification.status(), VerificationStatus::Verified);
    }

    #[test]
    fn test_parse_response_unverified() {
        let response = r#"
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
        "#;
        let verifier = VIES;
        let verification = verifier.parse_response(response.to_string()).unwrap();

        assert_eq!(*verification.status(), VerificationStatus::Unverified);
    }

    #[test]
    fn test_parse_response_unavailable() {
        let response = r#"
            <env:Envelope xmlns:env="http://schemas.xmlsoap.org/soap/envelope/">
                <env:Header/>
                <env:Body>
                    <env:Fault>
                        <faultcode>env:Server</faultcode>
                        <faultstring>MS_MAX_CONCURRENT_REQ</faultstring>
                    </env:Fault>
                </env:Body>
            </env:Envelope>
        "#;
        let verifier = VIES;
        let verification = verifier.parse_response(response.to_string()).unwrap();

        assert_eq!(*verification.status(), VerificationStatus::Unavailable);
    }
}
