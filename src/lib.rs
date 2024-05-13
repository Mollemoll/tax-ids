pub mod tax_id;
mod errors;
mod verification;
mod syntax;

#[cfg(feature = "eu_vat")]
mod eu_vat;
#[cfg(feature = "gb_vat")]
mod gb_vat;
#[cfg(feature = "ch_vat")]
mod ch_vat;
#[cfg(feature = "no_vat")]
mod no_vat;
