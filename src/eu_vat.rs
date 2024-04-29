use crate::tax_id::TaxIdType;

pub struct EUVat;

impl TaxIdType for EUVat {
    fn name(&self) -> &'static str {
        "eu_vat"
    }
}