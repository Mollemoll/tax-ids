use std::env;

use tax_ids::TaxId;

// Usage:
//
// cargo run --example async_verify -- <VAT number to verify>
//
#[tokio::main]
async fn main() {
    let args: Vec<String> = env::args().collect();

    let lookup = args[1].as_ref();

    let tax_id = TaxId::new(lookup).expect("invalid tax id format");
    let verification = tax_id.verify().expect("unverified tax id");

    println!("Status: {:?}", verification.status());
    println!("Data: {}", verification.data());
}
