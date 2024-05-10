mod brreg;
use regex::Regex;
use crate::tax_id::{TaxId, TaxIdType};
use crate::verification::Verifier;

pub struct NOVat;

impl NOVat {
    pub fn extract_org_number(&self, tax_id: &TaxId) -> String {
        tax_id.local_value().replace("MVA", "")
    }
}

impl TaxIdType for NOVat {
    fn name(&self) -> &'static str {
        "no_vat"
    }

    fn ensure_valid_syntax(&self, value: &str) -> bool {
        let regex = Regex::new(r"^NO[0-9]{9}(MVA)?$").unwrap();
        regex.is_match(value)
    }

    fn country_code_from(&self, tax_country_code: &str) -> String {
        tax_country_code.to_string()
    }

    fn verifier(&self) -> Box<dyn Verifier> {
        Box::new(brreg::BRReg)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_org_number() {
        let tax_id = TaxId::new("NO123456789MVA").unwrap();

        assert_eq!(NOVat::extract_org_number(&NOVat, &tax_id), "123456789");
    }

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

        for vat_number in valid_vat_numbers {
            let valid_syntax = NOVat::ensure_valid_syntax(&NOVat, vat_number);
            assert_eq!(
                valid_syntax,
                true,
                "Expected valid VAT number, got invalid: {}",
                vat_number
            );
        }

        for vat_number in invalid_vat_numbers {
            let valid_syntax = NOVat::ensure_valid_syntax(&NOVat, vat_number);
            assert_eq!(
                valid_syntax,
                false,
                "Expected invalid VAT number, got valid: {}",
                vat_number
            );
        }
    }
}
