use crate::tax_id::TaxIdType;

pub struct GBVat;

impl TaxIdType for GBVat {
    fn name(&self) -> &'static str {
        "gb_vat"
    }
}