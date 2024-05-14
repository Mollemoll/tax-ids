mod syntax;
mod vies;

use std::collections::HashMap;
use lazy_static::lazy_static;
use regex::Regex;
use syntax::EU_VAT_PATTERNS;
use crate::tax_id::TaxIdType;
use crate::verification::{Verifier};

pub struct EuVat;

lazy_static! {
    #[derive(Debug)]
    pub static ref COUNTRIES: Vec<&'static str> = vec![
        "AT", "BE", "BG", "CY", "CZ", "DE", "DK", "EE", "EL", "ES", "FI", "FR", "HR", "HU",
        "IE", "IT", "LT", "LU", "LV", "MT", "NL", "PL", "PT", "RO", "SE", "SI", "SK", "XI",
    ];
}

impl TaxIdType for EuVat {
    fn name(&self) -> &'static str {
        "eu_vat"
    }

    fn syntax_map(&self) -> &HashMap<String, Regex> {
        &EU_VAT_PATTERNS
    }

    fn country_code_from(&self, tax_country_code: &str) -> String {
        let country_code = match tax_country_code {
            "XI" => "GB",
            "EL" => "GR",
            _ => tax_country_code,
        };

        country_code.to_string()
    }

    fn verifier(&self) -> Box<dyn Verifier> {
        Box::new(vies::VIES)
    }
}

#[cfg(test)]
mod tests {
    use crate::errors::ValidationError;
    use super::*;

    fn assert_validations(valid_vat_numbers: Vec<&str>, invalid_vat_numbers: Vec<&str>) {
        for vat_number in valid_vat_numbers {
            let valid_syntax = EuVat::validate_syntax(&EuVat, vat_number);
            assert_eq!(
                valid_syntax,
                Ok(()),
                "Expected valid VAT number, got invalid: {}",
                vat_number
            );
        }

        for vat_number in invalid_vat_numbers {
            let valid_syntax = EuVat::validate_syntax(&EuVat, vat_number);
            assert_eq!(valid_syntax, Err(ValidationError::InvalidSyntax));
        }
    }

    #[test]
    fn test_at_vat() {
        let valid_vat_numbers = vec!["ATU12345678", "ATU87654321"];
        let invalid_vat_numbers = vec!["AT12345678", "ATU1234567", "ATU123456789", "ATU1234567A"];

        assert_validations(valid_vat_numbers, invalid_vat_numbers);
    }

    #[test]
    fn test_be_vat() {
        let valid_vat_numbers = vec!["BE0123456789", "BE0987654321"];
        let invalid_vat_numbers = vec![
            "BE123456789",
            "BE012345678",
            "BE01234567890",
            "BE012345678A",
        ];

        assert_validations(valid_vat_numbers, invalid_vat_numbers);
    }

    #[test]
    fn test_bg_vat() {
        let valid_vat_numbers = vec!["BG123456789", "BG1234567890"];
        let invalid_vat_numbers = vec!["BG12345678", "BG12345678901", "BG12345678A"];

        assert_validations(valid_vat_numbers, invalid_vat_numbers);
    }

    #[test]
    fn test_cy_vat() {
        let valid_vat_numbers = vec!["CY12345678A", "CY98765432Z"];
        let invalid_vat_numbers = vec!["CY12345678", "CY1234567A", "CY123456789A", "CY12345678AA"];

        assert_validations(valid_vat_numbers, invalid_vat_numbers);
    }

    #[test]
    fn test_cz_vat() {
        let valid_vat_numbers = vec!["CZ12345678", "CZ123456789", "CZ1234567890"];
        let invalid_vat_numbers = vec!["CZ1234567", "CZ12345678901", "CZ12345678A"];

        assert_validations(valid_vat_numbers, invalid_vat_numbers);
    }

    #[test]
    fn test_de_vat() {
        let valid_vat_numbers = vec!["DE123456789", "DE987654321"];
        let invalid_vat_numbers = vec!["DE12345678", "DE1234567890", "DE12345678A"];

        assert_validations(valid_vat_numbers, invalid_vat_numbers);
    }

    #[test]
    fn test_dk_vat() {
        let valid_vat_numbers = vec!["DK12345678"];
        let invalid_vat_numbers = vec!["DK1234567", "DK123456789", "DK1234567A"];

        assert_validations(valid_vat_numbers, invalid_vat_numbers);
    }

    #[test]
    fn test_ee_vat() {
        let valid_vat_numbers = vec!["EE101234567"];
        let invalid_vat_numbers = vec!["EE10123456", "EE1012345678", "EE10123456A"];

        assert_validations(valid_vat_numbers, invalid_vat_numbers);
    }

    #[test]
    fn test_el_vat() {
        let valid_vat_numbers = vec!["EL123456789"];
        let invalid_vat_numbers = vec!["EL12345678", "EL1234567890", "EL12345678A"];

        assert_validations(valid_vat_numbers, invalid_vat_numbers);
    }

    #[test]
    fn test_es_vat() {
        let valid_vat_numbers = vec!["ESX12345678", "ES12345678Z", "ESX1234567Z"];
        let invalid_vat_numbers = vec!["ES12345678", "ESX123456789", "ES12345678ZZ"];

        assert_validations(valid_vat_numbers, invalid_vat_numbers);
    }

    #[test]
    fn test_fi_vat() {
        let valid_vat_numbers = vec!["FI12345678"];
        let invalid_vat_numbers = vec!["FI1234567", "FI123456789", "FI1234567A"];

        assert_validations(valid_vat_numbers, invalid_vat_numbers);
    }

    #[test]
    fn test_fr_vat() {
        let valid_vat_numbers = vec!["FR12345678901", "FRX1234567890"];
        let invalid_vat_numbers = vec!["FR1234567890", "FR123456789012", "FR1234567890A"];

        assert_validations(valid_vat_numbers, invalid_vat_numbers);
    }

    #[test]
    fn test_hr_vat() {
        let valid_vat_numbers = vec!["HR12345678901"];
        let invalid_vat_numbers = vec!["HR1234567890", "HR123456789012", "HR1234567890A"];

        assert_validations(valid_vat_numbers, invalid_vat_numbers);
    }

    #[test]
    fn test_hu_vat() {
        let valid_vat_numbers = vec!["HU12345678"];
        let invalid_vat_numbers = vec!["HU1234567", "HU123456789", "HU1234567A"];

        assert_validations(valid_vat_numbers, invalid_vat_numbers);
    }

    #[test]
    fn test_ie_vat() {
        let valid_vat_numbers = vec!["IE1234567A", "IE1A23456A", "IE1234567AA"];
        let invalid_vat_numbers = vec!["IE1234567", "IE12345678A", "IE1234567AAA"];

        assert_validations(valid_vat_numbers, invalid_vat_numbers);
    }

    #[test]
    fn test_it_vat() {
        let valid_vat_numbers = vec!["IT12345678901"];
        let invalid_vat_numbers = vec!["IT1234567890", "IT123456789012", "IT1234567890A"];

        assert_validations(valid_vat_numbers, invalid_vat_numbers);
    }

    #[test]
    fn test_lt_vat() {
        let valid_vat_numbers = vec!["LT999999919", "LT999999919"];
        let invalid_vat_numbers = vec!["LT12345678", "LT12345678901", "LT12345678A"];

        assert_validations(valid_vat_numbers, invalid_vat_numbers);
    }

    #[test]
    fn test_lu_vat() {
        let valid_vat_numbers = vec!["LU12345678"];
        let invalid_vat_numbers = vec!["LU1234567", "LU123456789", "LU1234567A"];

        assert_validations(valid_vat_numbers, invalid_vat_numbers);
    }

    #[test]
    fn test_lv_vat() {
        let valid_vat_numbers = vec!["LV12345678901"];
        let invalid_vat_numbers = vec!["LV1234567890", "LV123456789012", "LV1234567890A"];

        assert_validations(valid_vat_numbers, invalid_vat_numbers);
    }

    #[test]
    fn test_mt_vat() {
        let valid_vat_numbers = vec!["MT12345678"];
        let invalid_vat_numbers = vec!["MT1234567", "MT123456789", "MT1234567A"];

        assert_validations(valid_vat_numbers, invalid_vat_numbers);
    }

    #[test]
    fn test_nl_vat() {
        let valid_vat_numbers = vec!["NL123456789B01"];
        let invalid_vat_numbers = vec!["NL123456789B0", "NL123456789B012", "NL123456789B0A"];

        assert_validations(valid_vat_numbers, invalid_vat_numbers);
    }

    #[test]
    fn test_pl_vat() {
        let valid_vat_numbers = vec!["PL1234567890"];
        let invalid_vat_numbers = vec!["PL123456789", "PL12345678901", "PL123456789A"];

        assert_validations(valid_vat_numbers, invalid_vat_numbers);
    }

    #[test]
    fn test_pt_vat() {
        let valid_vat_numbers = vec!["PT123456789"];
        let invalid_vat_numbers = vec!["PT12345678", "PT1234567890", "PT12345678A"];

        assert_validations(valid_vat_numbers, invalid_vat_numbers);
    }

    #[test]
    fn test_ro_vat() {
        let valid_vat_numbers = vec!["RO99999999", "RO999999999"];
        let invalid_vat_numbers = vec!["RO12345678910", "RO12345678901", "RO12345678A"];

        assert_validations(valid_vat_numbers, invalid_vat_numbers);
    }

    #[test]
    fn test_se_vat() {
        let valid_vat_numbers = vec!["SE123456789101"];
        let invalid_vat_numbers = vec!["SE12345678900", "SE123456789002", "SE12345678900A"];

        assert_validations(valid_vat_numbers, invalid_vat_numbers);
    }

    #[test]
    fn test_si_vat() {
        let valid_vat_numbers = vec!["SI12345678"];
        let invalid_vat_numbers = vec!["SI1234567", "SI123456789", "SI1234567A"];

        assert_validations(valid_vat_numbers, invalid_vat_numbers);
    }

    #[test]
    fn test_sk_vat() {
        let valid_vat_numbers = vec!["SK1234567890"];
        let invalid_vat_numbers = vec!["SK123456789", "SK12345678901", "SK123456789A"];

        assert_validations(valid_vat_numbers, invalid_vat_numbers);
    }

    #[test]
    fn test_xi_vat() {
        let valid_vat_numbers = vec!["XI123456789", "XI987654321", "XIHA123", "XIGD123"];
        let invalid_vat_numbers = vec!["XI12345678", "XI1234567890", "XI12345678A"];

        assert_validations(valid_vat_numbers, invalid_vat_numbers);
    }
}
