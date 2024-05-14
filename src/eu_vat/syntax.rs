use lazy_static::lazy_static;
use std::collections::HashMap;
use regex::Regex;

lazy_static! {
    #[derive(Debug)]
    pub static ref EU_VAT_PATTERNS: HashMap<String, Regex> = {
        let mut m = HashMap::new();
        m.insert("AT".to_string(), Regex::new(r"^ATU[0-9]{8}$").unwrap());
        m.insert("BE".to_string(), Regex::new(r"^BE[0-1][0-9]{9}$").unwrap());
        m.insert("BG".to_string(), Regex::new(r"^BG[0-9]{9,10}$").unwrap());
        m.insert("CY".to_string(), Regex::new(r"^CY[0-69][0-9]{7}[A-Z]$").unwrap());
        m.insert("CZ".to_string(), Regex::new(r"^CZ[0-9]{8,10}$").unwrap());
        m.insert("DE".to_string(), Regex::new(r"^DE[0-9]{9}$").unwrap());
        m.insert("DK".to_string(), Regex::new(r"^DK[0-9]{8}$").unwrap());
        m.insert("EE".to_string(), Regex::new(r"^EE10[0-9]{7}$").unwrap());
        m.insert("EL".to_string(), Regex::new(r"^EL[0-9]{9}$").unwrap());
        m.insert("ES".to_string(), Regex::new(r"^ES([A-Z][0-9]{8}|[0-9]{8}[A-Z]|[A-Z][0-9]{7}[A-Z])$").unwrap());
        m.insert("FI".to_string(), Regex::new(r"^FI[0-9]{8}$").unwrap());
        m.insert("FR".to_string(), Regex::new(r"^FR[A-HJ-NP-Z0-9]{2}[0-9]{9}$").unwrap());
        m.insert("HR".to_string(), Regex::new(r"^HR[0-9]{11}$").unwrap());
        m.insert("HU".to_string(), Regex::new(r"^HU[0-9]{8}$").unwrap());
        m.insert("IE".to_string(), Regex::new(r"^IE([0-9][A-Z][0-9]{5}|[0-9]{7}[A-Z]?)[A-Z]$").unwrap());
        m.insert("IT".to_string(), Regex::new(r"^IT[0-9]{11}$").unwrap());
        m.insert("LT".to_string(), Regex::new(r"^LT([0-9]{7}1[0-9]|[0-9]{10}1[0-9])$").unwrap());
        m.insert("LU".to_string(), Regex::new(r"^LU[0-9]{8}$").unwrap());
        m.insert("LV".to_string(), Regex::new(r"^LV[0-9]{11}$").unwrap());
        m.insert("MT".to_string(), Regex::new(r"^MT[0-9]{8}$").unwrap());
        m.insert("NL".to_string(), Regex::new(r"^NL[0-9]{9}B[0-9]{2}$").unwrap());
        m.insert("PL".to_string(), Regex::new(r"^PL[0-9]{10}$").unwrap());
        m.insert("PT".to_string(), Regex::new(r"^PT[0-9]{9}$").unwrap());
        m.insert("RO".to_string(), Regex::new(r"^RO[1-9][0-9]{1,9}$").unwrap());
        m.insert("SE".to_string(), Regex::new(r"^SE[0-9]{10}01$").unwrap());
        m.insert("SI".to_string(), Regex::new(r"^SI[0-9]{8}$").unwrap());
        m.insert("SK".to_string(), Regex::new(r"^SK[0-9]{10}$").unwrap());
        m.insert("XI".to_string(), Regex::new(r"^XI([0-9]{9}|[0-9]{12}|(HA|GD)[0-9]{3})$").unwrap());
        m
    };
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::eu_vat::COUNTRIES;
    #[test]
    fn test_each_eu_country_has_a_regex() {
        let mut eu_regex_countries = EU_VAT_PATTERNS.keys().collect::<Vec<&String>>();
        eu_regex_countries.sort();
        assert_eq!(eu_regex_countries, *COUNTRIES);
    }
}