use crate::tax_id::TaxIdType;

pub struct EUVat;

impl TaxIdType for EUVat {
    fn name(&self) -> &'static str {
        "eu_vat"
    }

    fn ensure_valid_syntax(&self, value: &str) -> bool {
        // Placeholder
        value.len() > 5
    }
}