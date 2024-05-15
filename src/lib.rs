mod errors;
mod verification;
mod syntax;

#[cfg(feature = "eu_vat")]
mod eu_vat;
#[cfg(feature = "eu_vat")]
use eu_vat::EuVat;
#[cfg(feature = "gb_vat")]
mod gb_vat;
#[cfg(feature = "gb_vat")]
use gb_vat::GbVat;
#[cfg(feature = "ch_vat")]
mod ch_vat;
#[cfg(feature = "ch_vat")]
use ch_vat::ChVat;
#[cfg(feature = "no_vat")]
mod no_vat;
#[cfg(feature = "no_vat")]
use no_vat::NoVat;

use std::collections::HashMap;
use std::fmt;
use regex::Regex;
use syntax::SYNTAX;
use verification::{Verifier};
pub use verification::{Verification, VerificationStatus};
pub use errors::{ValidationError, VerificationError};


trait TaxIdType {
    fn name(&self) -> &'static str;
    fn syntax_map(&self) -> &HashMap<String, Regex>;
    fn validate_syntax(&self, value: &str) -> Result<(), ValidationError> {
        let tax_country_code = &value[0..2];
        let pattern = self.syntax_map()
            .get(tax_country_code)
            .ok_or(ValidationError::UnsupportedCountryCode(tax_country_code.to_string()));

        if pattern?.is_match(value) {
            Ok(())
        } else {
            Err(ValidationError::InvalidSyntax)
        }
    }
    fn country_code_from_tax_country(&self, tax_country_code: &str) -> String;
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
    pub fn validate_syntax(value: &str) -> Result<(), ValidationError> {
        let tax_country_code = &value[0..2];
        SYNTAX.get(tax_country_code)
            .ok_or(ValidationError::UnsupportedCountryCode(tax_country_code.to_string()))
            .and_then(|syntax| {
                if syntax.is_match(value) {
                    Ok(())
                } else {
                    Err(ValidationError::InvalidSyntax)
                }
            })
    }

    pub fn new(value: &str) -> Result<TaxId, ValidationError> {
        let tax_country_code = &value[0..2];
        let local_value = &value[2..];

        let id_type: Box<dyn TaxIdType> = match tax_country_code {
            #[cfg(feature = "gb_vat")]
            "GB" => Box::new(GbVat),
            #[cfg(feature = "ch_vat")]
            "CH" => Box::new(ChVat),
            #[cfg(feature = "no_vat")]
            "NO" => Box::new(NoVat),
            #[cfg(feature = "eu_vat")]
            _ if eu_vat::COUNTRIES.contains(&tax_country_code) => Box::new(EuVat),
            _ => return Err(ValidationError::UnsupportedCountryCode(tax_country_code.to_string()))
        };

        id_type.validate_syntax(value)?;

        Ok(TaxId {
            country_code: id_type.country_code_from_tax_country(tax_country_code),
            value: value.to_string(),
            tax_country_code: tax_country_code.to_string(),
            local_value: local_value.to_string(),
            id_type,
        })
    }

    pub fn verify(&self) -> Result<Verification, VerificationError> {
        self.id_type.verifier().verify(self)
    }

    pub fn value(&self) -> &str { &self.value }
    pub fn country_code(&self) -> &str { &self.country_code }
    pub fn tax_country_code(&self) -> &str { &self.tax_country_code }
    pub fn local_value(&self) -> &str { &self.local_value }

    pub fn tax_id_type(&self) -> &str { self.id_type.name() }
    fn id_type(&self) -> &Box<dyn TaxIdType> { &self.id_type }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_syntax() {
        let mut valid_vat_numbers: Vec<&str> = Vec::new();
        #[cfg(feature = "eu_vat")]
        {
            valid_vat_numbers.push("SE123456789101");
            valid_vat_numbers.push("EL123456789");
            valid_vat_numbers.push("XI591819014");
        }
        #[cfg(feature = "gb_vat")]
        valid_vat_numbers.push("GB591819014");
        #[cfg(feature = "ch_vat")]
        valid_vat_numbers.push("CHE123456789");
        #[cfg(feature = "no_vat")]
        valid_vat_numbers.push("NO123456789MVA");

        for vat_number in valid_vat_numbers {
            let valid_syntax = TaxId::validate_syntax(vat_number);
            assert_eq!(
                valid_syntax,
                Ok(()),
                "Expected {} to be valid",
                vat_number
            );
        }
    }

    #[test]
    fn test_validate_syntax_unsupported_country() {
        let validation = TaxId::validate_syntax("XX123456789");
        assert!(validation.is_err());
        assert_eq!(validation.unwrap_err(), ValidationError::UnsupportedCountryCode("XX".to_string()));
    }

    #[test]
    fn test_new_unsupported_country() {
        let tax_id = TaxId::new("XX123456789");
        assert!(tax_id.is_err());
        assert_eq!(tax_id.unwrap_err(), ValidationError::UnsupportedCountryCode("XX".to_string()));
    }


    #[cfg(feature = "eu_vat")]
    #[test]
    fn test_validate_eu_syntax_fail() {
        let validation = TaxId::validate_syntax("SE12");
        assert!(validation.is_err());
        assert_eq!(validation.unwrap_err(), ValidationError::InvalidSyntax);
    }

    #[cfg(feature = "gb_vat")]
    #[test]
    fn test_validate_gb_syntax_fail() {
        let validation = TaxId::validate_syntax("GB12");
        assert!(validation.is_err());
        assert_eq!(validation.unwrap_err(), ValidationError::InvalidSyntax);
    }

    #[cfg(feature = "ch_vat")]
    #[test]
    fn test_validate_ch_syntax_fail() {
        let validation = TaxId::validate_syntax("CHE12");
        assert!(validation.is_err());
        assert_eq!(validation.unwrap_err(), ValidationError::InvalidSyntax);
    }

    #[cfg(feature = "no_vat")]
    #[test]
    fn test_validate_no_syntax_fail() {
        let validation = TaxId::validate_syntax("NO12");
        assert!(validation.is_err());
        assert_eq!(validation.unwrap_err(), ValidationError::InvalidSyntax);
    }

    #[cfg(feature = "eu_vat")]
    #[test]
    fn test_eu_new_unsupported_country_code_err() {
        let tax_id = TaxId::new("SE12");
        assert!(tax_id.is_err());
        assert_eq!(tax_id.unwrap_err(), ValidationError::InvalidSyntax);
    }

    #[cfg(feature = "gb_vat")]
    #[test]
    fn test_new_gb_unsupported_country_code_err() {
        let tax_id = TaxId::new("GB12");
        assert!(tax_id.is_err());
        assert_eq!(tax_id.unwrap_err(), ValidationError::InvalidSyntax);
    }

    #[cfg(feature = "ch_vat")]
    #[test]
    fn test_new_ch_unsupported_country_code_err() {
        let tax_id = TaxId::new("CHE12");
        assert!(tax_id.is_err());
        assert_eq!(tax_id.unwrap_err(), ValidationError::InvalidSyntax);
    }

    #[cfg(feature = "no_vat")]
    #[test]
    fn test_new_no_unsupported_country_code_err() {
        let tax_id = TaxId::new("NO12");
        assert!(tax_id.is_err());
        assert_eq!(tax_id.unwrap_err(), ValidationError::InvalidSyntax);
    }

    #[cfg(feature = "eu_vat")]
    #[test]
    fn test_new_eu_vat() {
        let tax_id= TaxId::new("SE123456789101").unwrap();
        assert_eq!(tax_id.value(), "SE123456789101");
        assert_eq!(tax_id.country_code(), "SE");
        assert_eq!(tax_id.local_value(), "123456789101");
        assert_eq!(tax_id.tax_id_type(), "eu_vat");
    }

    #[cfg(feature = "eu_vat")]
    #[test]
    fn test_new_gr_vat() {
        let tax_id = TaxId::new("EL123456789").unwrap();
        assert_eq!(tax_id.value(), "EL123456789");
        assert_eq!(tax_id.country_code(), "GR");
        assert_eq!(tax_id.local_value(), "123456789");
        assert_eq!(tax_id.tax_id_type(), "eu_vat");
    }

    #[cfg(feature = "eu_vat")]
    #[test]
    fn test_new_xi_vat() {
        let tax_id = TaxId::new("XI591819014").unwrap();
        assert_eq!(tax_id.value(), "XI591819014");
        assert_eq!(tax_id.country_code(), "GB");
        assert_eq!(tax_id.local_value(), "591819014");
        assert_eq!(tax_id.tax_id_type(), "eu_vat");
    }

    #[cfg(feature = "gb_vat")]
    #[test]
    fn test_new_gb_vat() {
        let tax_id = TaxId::new("GB591819014").unwrap();
        assert_eq!(tax_id.value(), "GB591819014");
        assert_eq!(tax_id.country_code(), "GB");
        assert_eq!(tax_id.local_value(), "591819014");
        assert_eq!(tax_id.tax_id_type(), "gb_vat");
    }

    #[cfg(feature = "ch_vat")]
    #[test]
    fn test_new_ch_vat() {
        let tax_id = TaxId::new("CHE123456789").unwrap();
        assert_eq!(tax_id.value(), "CHE123456789");
        assert_eq!(tax_id.country_code(), "CH");
        assert_eq!(tax_id.local_value(), "E123456789");
        assert_eq!(tax_id.tax_id_type(), "ch_vat");
    }

    #[cfg(feature = "no_vat")]
    #[test]
    fn test_new_no_vat() {
        let tax_id = TaxId::new("NO123456789MVA").unwrap();
        assert_eq!(tax_id.value(), "NO123456789MVA");
        assert_eq!(tax_id.country_code(), "NO");
        assert_eq!(tax_id.local_value(), "123456789MVA");
        assert_eq!(tax_id.tax_id_type(), "no_vat");
    }
}

