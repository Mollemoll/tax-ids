use std::collections::HashMap;
use lazy_static::lazy_static;
use regex::Regex;
use crate::tax_id::TaxIdType;
use crate::ch_vat::CHVat;
use crate::eu_vat::EUVat;
use crate::gb_vat::GBVat;
use crate::no_vat::NOVat;

lazy_static! {
    pub static ref SYNTAX: HashMap<String, Regex> = {
        let mut m = HashMap::new();

        let types: Vec<Box<dyn TaxIdType>> = vec![
            Box::new(GBVat),
            Box::new(CHVat),
            Box::new(NOVat),
            Box::new(EUVat),
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
