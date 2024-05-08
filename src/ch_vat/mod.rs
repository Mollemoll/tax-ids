mod bfs;

use regex::Regex;
use crate::tax_id::TaxIdType;
use crate::verification::Verifier;

pub struct CHVat;

impl TaxIdType for CHVat {
    fn name(&self) -> &'static str {
        "ch_vat"
    }

    fn ensure_valid_syntax(&self, value: &str) -> bool {
        let regex = Regex::new(r"^CHE([0-9]{9}|-[0-9]{3}(\.[0-9]{3}){2})(?:\s(MWST|TVA|IVA))?$").unwrap();
        regex.is_match(value)
    }

    fn country_code_from(&self, tax_country_code: &str) -> String {
        tax_country_code.to_string()
    }

    fn verifier(&self) -> Box<dyn Verifier> {
        Box::new(bfs::BFS)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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

        for vat_number in valid_vat_numbers {
            let valid_syntax = CHVat::ensure_valid_syntax(&CHVat, vat_number);
            assert_eq!(
                valid_syntax,
                true,
                "Expected valid VAT number, got invalid: {}",
                vat_number
            );
        }

        for vat_number in invalid_vat_numbers {
            let valid_syntax = CHVat::ensure_valid_syntax(&CHVat, vat_number);
            assert_eq!(
                valid_syntax,
                false,
                "Expected invalid VAT number, got valid: {}",
                vat_number
            );
        }
    }
}
