use std::fmt;
use crate::ch_vat::CHVat;
use crate::eu_vat::EUVat;
use crate::gb_vat::GBVat;
use crate::errors::{ValidationError, VerificationError};
use crate::eu_vat;
use crate::no_vat::NOVat;
use crate::verification::{Verification, Verifier};

pub trait TaxIdType {
    fn name(&self) -> &'static str;
    fn ensure_valid_syntax(&self, value: &str) -> bool;
    fn country_code_from(&self, tax_country_code: &str) -> String;
    fn verifier(&self) -> Box<dyn Verifier>;
    fn verify(&self, tax_id: &TaxId) -> Result<Verification, VerificationError> {
        self.verifier().verify(tax_id)
    }
}

pub struct TaxId {
    value: String,
    country_code: String,
    tax_country_code: String,
    local_value: String,
    id_type: Box<dyn TaxIdType>,
}

impl fmt::Debug for TaxId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "TaxId {{ value: {}, country_code: {}, tax_country_code: {}, local_value: {}, id_type: {}}}",
               self.value, self.country_code, self.tax_country_code, self.local_value, self.id_type.name())
    }
}

impl TaxId {
    pub fn new(value: &str) -> Result<TaxId, ValidationError> {
        let tax_country_code = &value[0..2];
        let local_value = &value[2..];

        let id_type: Box<dyn TaxIdType> = match tax_country_code {
            "GB" => Box::new(GBVat),
            "CH" => Box::new(CHVat),
            "NO" => Box::new(NOVat),
            _ if eu_vat::COUNTRIES.contains(&tax_country_code) => Box::new(EUVat),
            _ => return Err(ValidationError::UnknownCountryCode(tax_country_code.to_string()))
        };

        match id_type.ensure_valid_syntax(value) {
            false => Err(ValidationError::InvalidSyntax),
            true => Ok(TaxId {
                country_code: id_type.country_code_from(tax_country_code),
                value: value.to_string(),
                tax_country_code: tax_country_code.to_string(),
                local_value: local_value.to_string(),
                id_type,
            })
        }
    }

    pub fn verify(&self) -> Result<Verification, VerificationError> {
        self.id_type.verifier().verify(self)
    }

    pub fn value(&self) -> &str { &self.value }
    pub fn country_code(&self) -> &str { &self.country_code }
    pub fn tax_country_code(&self) -> &str { &self.tax_country_code }
    pub fn local_value(&self) -> &str { &self.local_value }
    pub fn id_type(&self) -> &Box<dyn TaxIdType> { &self.id_type }
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
        assert_eq!(tax_id.id_type().name(), "eu_vat");
    }

    #[test]
    fn test_new_gr_vat() {
        let tax_id = TaxId::new("EL123456789").unwrap();
        assert_eq!(tax_id.value(), "EL123456789");
        assert_eq!(tax_id.country_code(), "GR");
        assert_eq!(tax_id.local_value(), "123456789");
        assert_eq!(tax_id.id_type().name(), "eu_vat");
    }

    #[test]
    fn test_new_gb_vat() {
        let tax_id = TaxId::new("GB591819014").unwrap();
        assert_eq!(tax_id.value(), "GB591819014");
        assert_eq!(tax_id.country_code(), "GB");
        assert_eq!(tax_id.local_value(), "591819014");
        assert_eq!(tax_id.id_type().name(), "gb_vat");
    }

    #[test]
    fn test_new_xi_vat() {
        let tax_id = TaxId::new("XI591819014").unwrap();
        assert_eq!(tax_id.value(), "XI591819014");
        assert_eq!(tax_id.country_code(), "GB");
        assert_eq!(tax_id.local_value(), "591819014");
        assert_eq!(tax_id.id_type().name(), "eu_vat");
    }

    #[test]
    fn test_new_ch_vat() {
        let tax_id = TaxId::new("CHE123456789").unwrap();
        assert_eq!(tax_id.value(), "CHE123456789");
        assert_eq!(tax_id.country_code(), "CH");
        assert_eq!(tax_id.local_value(), "E123456789");
        assert_eq!(tax_id.id_type().name(), "ch_vat");
    }

    #[test]
    fn test_new_no_vat() {
        let tax_id = TaxId::new("NO123456789MVA").unwrap();
        assert_eq!(tax_id.value(), "NO123456789MVA");
        assert_eq!(tax_id.country_code(), "NO");
        assert_eq!(tax_id.local_value(), "123456789MVA");
        assert_eq!(tax_id.id_type().name(), "no_vat");
    }

    #[test]
    fn test_new_unknown_country_code_err() {
        let tax_id = TaxId::new("XX123456789");
        assert!(tax_id.is_err());
        assert_eq!(tax_id.unwrap_err(), ValidationError::UnknownCountryCode("XX".to_string()));
    }

    #[test]
    fn test_failed_validation() {
        let tax_id = TaxId::new("SE12");
        assert!(tax_id.is_err());
        assert_eq!(tax_id.unwrap_err(), ValidationError::InvalidSyntax);
    }
}
