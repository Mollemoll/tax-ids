use crate::tax_id::TaxIdType;

pub struct GBVat;

impl TaxIdType for GBVat {
    fn name(&self) -> &'static str {
        "gb_vat"
    }

    fn ensure_valid_syntax(&self, _value: &str) -> bool {
        // Placeholder
        true
    }
}