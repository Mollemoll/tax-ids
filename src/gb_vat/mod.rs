mod hmrc;

use std::collections::HashMap;
use lazy_static::lazy_static;
use regex::Regex;
use crate::TaxIdType;
use crate::verification::Verifier;

lazy_static! {
    #[derive(Debug)]
    pub static ref GB_VAT_PATTERN: HashMap<String, Regex> = {
        let mut m = HashMap::new();
        m.insert(
            "GB".to_string(),
            Regex::new(r"^GB([0-9]{9}|[0-9]{12}|(HA|GD)[0-9]{3})$").unwrap()
        );
        m
    };

}

#[derive(Debug)]
pub struct GbVat;

impl TaxIdType for GbVat {
    fn name(&self) -> &'static str {
        "gb_vat"
    }

    fn syntax_map(&self) -> &HashMap<String, Regex> {
        &GB_VAT_PATTERN
    }

    fn country_code_from_tax_country(&self, tax_country_code: &str) -> String {
        tax_country_code.to_string()
    }

    fn verifier(&self) -> Box<dyn Verifier> {
        Box::new(hmrc::Hmrc)
    }
}

#[cfg(test)]
mod tests {
    use crate::gb_vat::GbVat;
    use crate::TaxIdType;

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

        for valid in valid_vat_numbers {
            assert!(GbVat::validate_syntax(&GbVat, valid).is_ok());
        }

        for invalid in invalid_vat_numbers {
            assert!(GbVat::validate_syntax(&GbVat, invalid).is_err());
        }

    }
}
