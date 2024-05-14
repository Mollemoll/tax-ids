use std::collections::HashMap;
use lazy_static::lazy_static;
use regex::Regex;
use crate::tax_id::TaxIdType;

#[cfg(feature = "ch_vat")]
use crate::ch_vat::ChVat;
#[cfg(feature = "eu_vat")]
use crate::eu_vat::EuVat;
#[cfg(feature = "gb_vat")]
use crate::gb_vat::GbVat;
#[cfg(feature = "no_vat")]
use crate::no_vat::NoVat;

lazy_static! {
    #[derive(Debug)]
    pub static ref SYNTAX: HashMap<String, Regex> = {
        let mut m = HashMap::new();

        let types: Vec<Box<dyn TaxIdType>> = vec![
            #[cfg(feature = "gb_vat")]
            Box::new(GbVat),
            #[cfg(feature = "ch_vat")]
            Box::new(ChVat),
            #[cfg(feature = "no_vat")]
            Box::new(NoVat),
            #[cfg(feature = "eu_vat")]
            Box::new(EuVat),
        ];

        for t in types {
            let syntax_map = t.syntax_map();
            for (code, pattern) in syntax_map {
                m.insert(code.to_string(), pattern.clone());
            }
        }

        m
    };
}
