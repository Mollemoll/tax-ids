use std::process::id;
use crate::eu_vat::EUVat;
use crate::gb_vat::GBVat;
use crate::errors::ValidationError;
use crate::eu_vat;

pub trait TaxIdType {
    fn name(&self) -> &'static str;
    fn ensure_valid_syntax(&self, value: &str) -> bool;
    fn country_code_from(&self, tax_country_code: &str) -> String;
}

struct TaxId {
    value: String,
    country_code: String,
    tax_country_code: String,
    local_value: String,
    id_type: &'static str,
}

impl TaxId {
    pub fn new(value: &str) -> Result<TaxId, ValidationError> {
        let tax_country_code = &value[0..2];
        let local_value = &value[2..];

        let id_type: Box<dyn TaxIdType> = match tax_country_code {
            "GB" => Box::new(GBVat),
            _ if eu_vat::COUNTRIES.contains(&tax_country_code) => Box::new(EUVat),
            _ => return Err(ValidationError::new("Unknown country code")),
        };

        match id_type.ensure_valid_syntax(value) {
            false => Err(ValidationError::new("Invalid syntax")),
            true => Ok(TaxId {
                id_type: id_type.name(),
                value: value.to_string(),
                tax_country_code: tax_country_code.to_string(),
                country_code: id_type.country_code_from(tax_country_code),
                local_value: local_value.to_string(),
            })
        }
    }

    pub fn value(&self) -> &str { &self.value }
    pub fn country_code(&self) -> &str { &self.country_code }
    pub fn local_value(&self) -> &str { &self.local_value }
    pub fn id_type(&self) -> &str { &self.id_type }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_eu_vat() {
        let tax_id= TaxId::new("SE123456789101").unwrap();
        assert_eq!(tax_id.value(), "SE123456789101");
        assert_eq!(tax_id.country_code(), "SE");
        assert_eq!(tax_id.local_value(), "123456789101");
        assert_eq!(tax_id.id_type(), "eu_vat");
    }

    #[test]
    fn test_new_gr_vat() {
        let tax_id = TaxId::new("EL123456789").unwrap();
        assert_eq!(tax_id.value(), "EL123456789");
        assert_eq!(tax_id.country_code(), "GR");
        assert_eq!(tax_id.local_value(), "123456789");
        assert_eq!(tax_id.id_type(), "eu_vat");
    }

    #[test]
    fn test_new_gb_vat() {
        let tax_id = TaxId::new("GB591819014").unwrap();
        assert_eq!(tax_id.value(), "GB591819014");
        assert_eq!(tax_id.country_code(), "GB");
        assert_eq!(tax_id.local_value(), "591819014");
        assert_eq!(tax_id.id_type(), "gb_vat");
    }

    #[test]
    fn test_new_xi_vat() {
        let tax_id = TaxId::new("XI591819014").unwrap();
        assert_eq!(tax_id.value(), "XI591819014");
        assert_eq!(tax_id.country_code(), "GB");
        assert_eq!(tax_id.local_value(), "591819014");
        assert_eq!(tax_id.id_type(), "eu_vat");
    }

    #[test]
    #[should_panic(expected = "Unknown country code")]
    fn test_new_unknown_country_code() {
        let _ = TaxId::new("XX123456789").unwrap();
    }

    #[test]
    #[should_panic(expected = "Invalid syntax")]
    fn test_failed_validation() {
        let _ = TaxId::new("SE12").unwrap();
    }
}