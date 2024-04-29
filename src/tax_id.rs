use crate::eu_vat::EUVat;
use crate::gb_vat::GBVat;
use crate::errors::ValidationError;

pub trait TaxIdType {
    fn name(&self) -> &'static str;
}

struct TaxId {
    value: String,
    country_code: String,
    local_value: String,
    id_type: &'static str,
}

impl TaxId {
    pub fn new(value: &str) -> Result<TaxId, ValidationError> {
        let country_code = &value[0..2];
        let local_value = &value[2..];
        let id_type: Box<dyn TaxIdType> = match country_code {
            "SE" => Box::new(EUVat),
            "GB" => Box::new(GBVat),
            _ => return Err(ValidationError::new("Unknown country code"))
        };

        Ok(TaxId {
            id_type: id_type.name(),
            value: value.to_string(),
            country_code: country_code.to_string(),
            local_value: local_value.to_string(),
        })
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
        let tax_id= TaxId::new("SE1234567890").unwrap();
        assert_eq!(tax_id.value(), "SE1234567890");
        assert_eq!(tax_id.country_code(), "SE");
        assert_eq!(tax_id.local_value(), "1234567890");
        assert_eq!(tax_id.id_type(), "eu_vat");
    }

    #[test]
    fn test_new_gb_vat() {
        let tax_id = TaxId::new("GB123456789").unwrap();
        assert_eq!(tax_id.value(), "GB123456789");
        assert_eq!(tax_id.country_code(), "GB");
        assert_eq!(tax_id.local_value(), "123456789");
        assert_eq!(tax_id.id_type(), "gb_vat");
    }

    #[test]
    #[should_panic(expected = "Unknown country code")]
    fn test_new_unknown_country_code() {
        let _ = TaxId::new("XX123456789").unwrap();
    }
}