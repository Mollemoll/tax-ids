use std::{env, error::Error};

use tax_ids::TaxId;

// Usage:
//
// cargo run --example async_verify -- <VAT number to verify>
//
#[tokio::main]
async fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
    let args: Vec<String> = env::args().collect();

    let lookup = args[1].as_ref();

    let tax_id = TaxId::new(lookup)?;
    let verification = tax_id.verify()?;

    println!("Status: {:?}", verification.status());
    println!("Data: {}", verification.data());

    return Ok(());
}
