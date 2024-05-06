mod hmrc;

use regex::Regex;
use crate::tax_id::TaxIdType;
use crate::verification::Verifier;

pub struct GBVat;

impl TaxIdType for GBVat {
    fn name(&self) -> &'static str {
        "gb_vat"
    }

    fn ensure_valid_syntax(&self, value: &str) -> bool {
        let regex = Regex::new(r"^GB([0-9]{9}|[0-9]{12}|(HA|GD)[0-9]{3})$").unwrap();
        regex.is_match(value)
    }

    fn country_code_from(&self, tax_country_code: &str) -> String {
        tax_country_code.to_string()
    }

    fn verifier(&self) -> Box<dyn Verifier> {
        Box::new(hmrc::HMRC)
    }
}

#[cfg(test)]
mod tests {
    use crate::gb_vat::GBVat;
    use super::*;

    #[test]
    fn test_gb_vat() {
        let valid_vat_numbers = vec![
            "GB123456789",
            "GB123456789101",
            "GBHA123",
            "GBGD123"
        ];
        let invalid_vat_numbers = vec![
            "GB12345678",
            "GB1234567891011",
            "GBHA1234",
            "GBGD1234",
            "SE123456789101"
        ];

        for vat_number in valid_vat_numbers {
            let valid_syntax = GBVat::ensure_valid_syntax(&GBVat, vat_number);
            assert_eq!(
                valid_syntax,
                true,
                "Expected valid VAT number, got invalid: {}",
                vat_number
            );
        }

        for vat_number in invalid_vat_numbers {
            let valid_syntax = GBVat::ensure_valid_syntax(&GBVat, vat_number);
            assert_eq!(
                valid_syntax,
                false,
                "Expected invalid VAT number, got valid: {}",
                vat_number
            );
        }
    }
}