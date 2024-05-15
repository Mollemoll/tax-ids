use std::collections::HashMap;
use lazy_static::lazy_static;
use reqwest::header::{HeaderMap, HeaderName, HeaderValue, ACCEPT, CONTENT_TYPE};
use roxmltree;
use serde_json::json;
use crate::verification::{Verifier, Verification, VerificationStatus, VerificationResponse};
use crate::errors::VerificationError;
use crate::TaxId;

// INFO(2024-05-07 mollemoll):
// https://www.bfs.admin.ch/bfs/en/home/registers/enterprise-register/enterprise-identification/uid-register/uid-interfaces.html#-125185306
// https://www.bfs.admin.ch/bfs/fr/home/registres/registre-entreprises/numero-identification-entreprises/registre-ide/interfaces-ide.assetdetail.11007266.html
// BFS Accepted format: 'CHE123456789' or 'CHE-123.456.789' with optional space and
// MWST/TVA/IVA extension: 'CHE123456789 MWST' or 'CHE-123.456.789 MWST'

static URI: &'static str = "https://www.uid-wse-a.admin.ch/V5.0/PublicServices.svc";

static ENVELOPE: &'static str = "
    <soapenv:Envelope xmlns:soapenv=\"http://schemas.xmlsoap.org/soap/envelope/\" xmlns:uid=\"http://www.uid.admin.ch/xmlns/uid-wse\">
        <soapenv:Header/>
        <soapenv:Body>
            <uid:ValidateVatNumber>
                <uid:vatNumber>{value}</uid:vatNumber>
            </uid:ValidateVatNumber>
        </soapenv:Body>
    </soapenv:Envelope>
";

lazy_static! {
    #[derive(Debug)]
    pub static ref HEADERS: HeaderMap = {
        let mut headers = HeaderMap::new();
        headers.insert(ACCEPT, HeaderValue::from_static("text/xml;charset=UTF-8"));
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("text/xml;charset=UTF-8"));
        headers.insert(
            HeaderName::from_static("soapaction"),
            HeaderValue::from_static("http://www.uid.admin.ch/xmlns/uid-wse/IPublicServices/ValidateVatNumber")
        );
        headers
    };
}

#[derive(Debug)]
pub struct BFS;

impl BFS {
    fn xml_to_hash(xml: &roxmltree::Document) -> HashMap<String, Option<String>> {
        let mut hash = HashMap::new();
        let tags_to_exclude = [
            "Body",
            "Envelope",
            "Fault",
            "businessFault",
            "detail",
            "ValidateVatNumberResponse",
        ];

        for node in xml.descendants() {
            let tag_name = node.tag_name().name();
            if tag_name.trim().is_empty() || tags_to_exclude.contains(&tag_name) {
                continue;
            }

            if let Some(text) = node.text() {
                hash.insert(tag_name.to_string(), Some(text.to_string()));
            }
        }

        hash
    }
}

impl Verifier for BFS {
    fn make_request(&self, tax_id: &TaxId) -> Result<VerificationResponse, VerificationError> {
        let client = reqwest::blocking::Client::new();
        let body = ENVELOPE
            .replace("{value}", tax_id.value());
        let res = client
            .post(URI)
            .headers(HEADERS.clone())
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
        let hash = BFS::xml_to_hash(&doc);
        let fault_string = hash.get("faultstring")
            .and_then(|x| x.as_deref());

        let status = match fault_string {
            Some("Data_validation_failed") => VerificationStatus::Unverified,
            Some("Request_limit_exceeded") => VerificationStatus::Unavailable,
            Some(_) => return Err(VerificationError::UnexpectedResponse(
                format!("Unexpected faultstring: {}", fault_string.unwrap().to_string())
            )),
            None => {
                let result = hash.get("ValidateVatNumberResult").and_then(|x| x.as_deref());
                match result {
                    Some("true") => VerificationStatus::Verified,
                    Some("false") => VerificationStatus::Unverified,
                    None | Some(_) => return Err(VerificationError::UnexpectedResponse(
                        "ValidateVatNumberResult should be 'true' or 'false'".to_string()
                    )),
                }
            },
        };

        Ok(Verification::new(status, json!(hash)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bfs_xml_to_hash() {
        let xml = r#"
            <s:Envelope xmlns:s="http://schemas.xmlsoap.org/soap/envelope/">
                <s:Body xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance" xmlns:xsd="http://www.w3.org/2001/XMLSchema">
                    <ValidateVatNumberResponse xmlns="http://www.uid.admin.ch/xmlns/uid-wse">
                        <ValidateVatNumberResult>true</ValidateVatNumberResult>
                    </ValidateVatNumberResponse>
                </s:Body>
            </s:Envelope>
        "#;
        let doc = roxmltree::Document::parse(xml).unwrap();
        let hash = BFS::xml_to_hash(&doc);

        assert_eq!(hash.get("ValidateVatNumberResult"), Some(&Some("true".to_string())));
    }

    #[test]
    fn test_parse_response_verified() {
        let response = VerificationResponse::new(
            200,
            r#"
                <s:Envelope xmlns:s="http://schemas.xmlsoap.org/soap/envelope/">
                    <s:Body xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance" xmlns:xsd="http://www.w3.org/2001/XMLSchema">
                        <ValidateVatNumberResponse xmlns="http://www.uid.admin.ch/xmlns/uid-wse">
                            <ValidateVatNumberResult>true</ValidateVatNumberResult>
                        </ValidateVatNumberResponse>
                    </s:Body>
                </s:Envelope>
            "#.to_string()
        );

        let verifier = BFS;
        let verification = verifier.parse_response(response).unwrap();

        assert_eq!(verification.status(), &VerificationStatus::Verified);
        assert_eq!(verification.data(), &json!({
            "ValidateVatNumberResult": "true"
        }));
    }

    #[test]
    fn test_parse_response_unverified() {
        let response = VerificationResponse::new(
            200,
            r#"
                <s:Envelope xmlns:s="http://schemas.xmlsoap.org/soap/envelope/">
                    <s:Body xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance" xmlns:xsd="http://www.w3.org/2001/XMLSchema">
                        <ValidateVatNumberResponse xmlns="http://www.uid.admin.ch/xmlns/uid-wse">
                            <ValidateVatNumberResult>false</ValidateVatNumberResult>
                        </ValidateVatNumberResponse>
                    </s:Body>
                </s:Envelope>
            "#.to_string()
        );

        let verifier = BFS;
        let verification = verifier.parse_response(response).unwrap();

        assert_eq!(verification.status(), &VerificationStatus::Unverified);
        assert_eq!(verification.data(), &json!({
            "ValidateVatNumberResult": "false"
        }));
    }

    #[test]
    fn test_parse_response_unavailable() {
        let response = VerificationResponse::new(
            500,
            r#"
                <s:Envelope xmlns:s="http://schemas.xmlsoap.org/soap/envelope/">
                    <s:Body>
                        <s:Fault>
                            <faultcode>s:Client</faultcode>
                            <faultstring xml:lang="de-CH">Request_limit_exceeded</faultstring>
                            <detail>
                                <businessFault xmlns="http://www.uid.admin.ch/xmlns/uid-wse" xmlns:xsd="http://www.w3.org/2001/XMLSchema" xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance">
                                    <operation xmlns="http://www.uid.admin.ch/xmlns/uid-wse-shared/2">ValidateVatNumber</operation>
                                    <error xmlns="http://www.uid.admin.ch/xmlns/uid-wse-shared/2">Request_limit_exceeded</error>
                                    <errorDetail xmlns="http://www.uid.admin.ch/xmlns/uid-wse-shared/2">Maximum number of 20 requests per 1 minute(s) exceeded</errorDetail>
                                </businessFault>
                            </detail>
                        </s:Fault>
                    </s:Body>
                </s:Envelope>
            "#.to_string()
        );

        let verifier = BFS;
        let verification = verifier.parse_response(response).unwrap();

        assert_eq!(verification.status(), &VerificationStatus::Unavailable);
        assert_eq!(verification.data(), &json!({
            "error": "Request_limit_exceeded",
            "errorDetail": "Maximum number of 20 requests per 1 minute(s) exceeded",
            "operation": "ValidateVatNumber",
            "faultcode": "s:Client",
            "faultstring": "Request_limit_exceeded"
        }));
    }

    #[test]
    fn test_parse_response_unavailable_unexpected_faultstring() {
        let response = VerificationResponse::new(
            500,
            r#"
                <s:Envelope xmlns:s="http://schemas.xmlsoap.org/soap/envelope/">
                    <s:Body>
                        <s:Fault>
                            <faultcode>s:Client</faultcode>
                            <faultstring xml:lang="de-CH">Unexpected_fault_string</faultstring>
                        </s:Fault>
                    </s:Body>
                </s:Envelope>
            "#.to_string()
        );

        let verifier = BFS;
        let verification = verifier.parse_response(response);

        match verification {
            Err(VerificationError::UnexpectedResponse(msg)) => {
                assert_eq!(msg, "Unexpected faultstring: Unexpected_fault_string");
            }
            _ => panic!("Expected UnexpectedResponse error"),
        }
    }

    #[test]
    fn test_parse_response_unexpected_response_value() {
        let response = VerificationResponse::new(
            200,
            r#"
                <s:Envelope xmlns:s="http://schemas.xmlsoap.org/soap/envelope/">
                    <s:Body xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance" xmlns:xsd="http://www.w3.org/2001/XMLSchema">
                        <ValidateVatNumberResponse xmlns="http://www.uid.admin.ch/xmlns/uid-wse">
                            <ValidateVatNumberResult>unexpected value</ValidateVatNumberResult>
                        </ValidateVatNumberResponse>
                    </s:Body>
                </s:Envelope>
            "#.to_string()
        );

        let verifier = BFS;
        let verification = verifier.parse_response(response);

        match verification {
            Err(VerificationError::UnexpectedResponse(msg)) => {
                assert_eq!(msg, "ValidateVatNumberResult should be 'true' or 'false'");
            }
            _ => panic!("Expected UnexpectedResponse error"),
        }
    }

    #[test]
    fn test_parse_response_empty_response_value() {
        let response = VerificationResponse::new(
            200,
            r#"
                <s:Envelope xmlns:s="http://schemas.xmlsoap.org/soap/envelope/">
                    <s:Body xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance" xmlns:xsd="http://www.w3.org/2001/XMLSchema">
                        <ValidateVatNumberResponse xmlns="http://www.uid.admin.ch/xmlns/uid-wse">
                            <ValidateVatNumberResult></ValidateVatNumberResult>
                        </ValidateVatNumberResponse>
                    </s:Body>
                </s:Envelope>
            "#.to_string()
        );

        let verifier = BFS;
        let verification = verifier.parse_response(response);

        match verification {
            Err(VerificationError::UnexpectedResponse(msg)) => {
                assert_eq!(msg, "ValidateVatNumberResult should be 'true' or 'false'");
            }
            _ => panic!("Expected UnexpectedResponse error"),
        }
    }
}
