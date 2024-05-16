mod bfs;

use std::collections::HashMap;
use lazy_static::lazy_static;
use regex::Regex;
use crate::TaxIdType;
use crate::verification::Verifier;

lazy_static! {
    #[derive(Debug)]
    pub static ref CH_VAT_PATTERN: HashMap<String, Regex> = {
        let mut m = HashMap::new();
        m.insert(
            "CH".to_string(),
            Regex::new(r"^CHE([0-9]{9}|-[0-9]{3}(\.[0-9]{3}){2})(?:\s(MWST|TVA|IVA))?$").unwrap()
        );
        m
    };
}

#[derive(Debug)]
pub struct ChVat;

impl TaxIdType for ChVat {
    fn name(&self) -> &'static str {
        "ch_vat"
    }

    fn syntax_map(&self) -> &HashMap<String, Regex> {
        &CH_VAT_PATTERN
    }

    fn country_code_from_tax_country(&self, tax_country_code: &str) -> String {
        tax_country_code.to_string()
    }

    fn verifier(&self) -> Box<dyn Verifier> {
        Box::new(bfs::Bfs)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(feature = "ch_vat")]
    #[test]
    fn test_ch_vats() {
        let valid_vat_numbers = vec![
            "CHE-778.887.921",
            "CHE-778.887.921 MWST",
            "CHE778887921",
            "CHE778887921 MWST",
            "CHE-778.887.921",
            "CHE-778.887.921 TVA",
            "CHE778887921",
            "CHE778887921 TVA",
            "CHE-778.887.921",
            "CHE-778.887.921 IVA",
            "CHE778887921",
            "CHE778887921 IVA"
        ];
        let invalid_vat_numbers = vec![
            "CHE-778.887.921MWST",
            "CHE778887921MWST",
            "CHE-778.887.921TVA",
            "CHE778887921TVA",
            "CHE-778.887.921IVA",
            "CHE778887921IVA",
            "CHE-778.887.9211",
            "CHE-778.887.9211MWST",
            "CHE-778.887.9211 MWST",
            "CHE-34.887.921",
            "CHE-34.887.921MWST",
            "CHE-34.887.921 MWST",
            "CHE-778.887.9211",
            "CHE-778.887.9211TVA",
            "CHE-778.887.9211 TVA",
            "CHE-34.887.921",
            "CHE-34.887.921TVA",
            "CHE-34.887.921 TVA",
            "CHE-778.887.9211",
            "CHE-778.887.9211IVA",
            "CHE-778.887.9211 IVA",
            "CHE-34.887.921",
            "CHE-34.887.921IVA",
            "CHE-34.887.921 IVA"
        ];

        for valid in valid_vat_numbers {
            assert!(ChVat::validate_syntax(&ChVat, valid).is_ok());
        }

        for invalid in invalid_vat_numbers {
            assert!(ChVat::validate_syntax(&ChVat, invalid).is_err());
        }
    }
}
