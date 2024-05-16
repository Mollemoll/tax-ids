mod brreg;
mod translator;

use std::collections::HashMap;
use lazy_static::lazy_static;
use regex::Regex;
use crate::{TaxId, TaxIdType};
use crate::verification::Verifier;

lazy_static! {
    #[derive(Debug)]
    pub static ref NO_VAT_PATTERN: HashMap<String, Regex> = {
        let mut m = HashMap::new();
        m.insert("NO".to_string(), Regex::new(r"^NO[0-9]{9}(MVA)?$").unwrap());
        m
    };
}

#[derive(Debug)]
pub struct NoVat;

impl NoVat {
    pub fn extract_org_number(&self, tax_id: &TaxId) -> String {
        tax_id.local_value().replace("MVA", "")
    }
}

impl TaxIdType for NoVat {
    fn name(&self) -> &'static str {
        "no_vat"
    }
    fn syntax_map(&self) -> &HashMap<String, Regex> {
        &NO_VAT_PATTERN
    }

    fn country_code_from_tax_country(&self, tax_country_code: &str) -> String {
        tax_country_code.to_string()
    }

    fn verifier(&self) -> Box<dyn Verifier> {
        Box::new(brreg::BrReg)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(feature = "no_vat")]
    #[test]
    fn test_extract_org_number() {
        let tax_id = TaxId::new("NO123456789MVA").unwrap();

        assert_eq!(NoVat::extract_org_number(&NoVat, &tax_id), "123456789");
    }

    #[cfg(feature = "no_vat")]
    #[test]
    fn test_no_vats() {
        let valid_vat_numbers = vec![
            "NO123456789MVA",
            "NO123456789",
        ];
        let invalid_vat_numbers = vec![
            "NO123456789 MVA",
            "NO12345678MVA",
            "NO1234567891MVA",
            "NO123456789XXX",
            "NO123456789MVA1",
            "NO12345678",
            "NO1234567890",
        ];

        for valid in valid_vat_numbers {
            assert!(NoVat::validate_syntax(&NoVat, valid).is_ok());
        }

        for invalid in invalid_vat_numbers {
            assert!(NoVat::validate_syntax(&NoVat, invalid).is_err());
        }
    }
}
