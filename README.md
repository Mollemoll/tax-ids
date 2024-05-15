# Tax Ids

This crate provides a solution for verifying tax ids (VAT/GST) for businesses operating
within the European Union, the United Kingdom, Switzerland, and Norway.

So far, the library provides the following functionality:

- Validate the syntax of a tax id against its type specific regex pattern.
- Verify the tax id in the appropriate government database (based on the tax id type).

The library has been inspired by the [valvat](https://github.com/yolk/valvat) library for Ruby.

### Available features

| Feature | Description | Default
| --- | --- | --- |
| `eu_vat` | European Union VAT numbers | ✓ |
| `gb_vat` | United Kingdom VAT numbers |
| `ch_vat` | Switzerland VAT numbers |
| `no_vat` | Norway VAT numbers |

More info at [Tax Id Types](#tax-id-types).

### Installation

With default feature `eu_vat`:
```toml
[dependencies]
tax_ids = "0.1.0"
```

With `eu_vat` and `gb_vat` features enabled:
```toml
[dependencies]
tax_ids = { version = "0.1.0", features = ["eu_vat", "gb_vat"] }
```

## Usage

```rust
use tax_ids::TaxId;

fn main () {
  // Instantiate a new TaxId object. Can raise a ValidationError.
  let tax_id = match TaxId::new("SE556703748501") {
    Ok(tax_id) => tax_id,
    Err(e) => {
      println!("ValidationError: {}", e);
      return;
    }
  };

  println!("Tax Id: {}", tax_id.value());
  println!("Country code: {}", tax_id.country_code());
  println!("Tax country code: {}", tax_id.tax_country_code());
  println!("Local value: {}", tax_id.local_value());
  println!("Id type: {}", tax_id.tax_id_type());

  // Tax Id: SE556703748501
  // Country code: SE
  // Tax country code: SE
  // Local value: 556703748501
  // Id type: eu_vat

  // Country code is the 2-char iso code of the country
  // Often the same as the tax country code, but not always.
  // Example country code GR for Greece, but EL for the Greek VAT number.

  // The United Kingdom, has country code GB and tax country code GB.
  // However, as a consequence of Brexit, businesses in Northern Ireland
  // have country code GB but use VAT number/tax country code XI when trading
  // with EU.

  // Verification

  // Perform a verification request against the country's tax id database.
  // Can raise a VerificationError
  let verification = match tax_id.verify() {
    Ok(verification) => verification,
    Err(e) => {
      println!("VerificationError: {}", e);
      return;
    },
  };

  println!("Verification status: {:?}", verification.status());

  // Verification can come back with one out of three different statuses:
  // - Verified - The tax id is legitimate.
  // - Unverified - The tax id is not legitimate.
  // - Unavailable - The verification couldn't be performed (rate limit, database unavailable etc).

  // These statuses are what you want to act upon.
  match verification.status() {
    tax_ids::VerificationStatus::Verified => {
      // Proceed payment
    },
    tax_ids::VerificationStatus::Unverified => {
      // Ask customer to provide a valid tax id
    },
    tax_ids::VerificationStatus::Unavailable => {
      // Process payment and verify the tax id later?
    },
  }
  
  // The full verification object:
  
  println!("{:?}", verification);

  // The data field is experimental and subject to change or removal.
  // It will contain different data depending on what tax id type is being verified.
  // And what response the verification service provides.

  // Verification status: Verified
  // Verification {
  //    performed_at: 2024-05-15T14:38:31.388914+02:00,
  //    status: Verified,
  //    data: Object {
  //        "address": String("REGERINGSGATAN 19 \n111 53 STOCKHOLM"),
  //        "countryCode": String("SE"),
  //        "name": String("Spotify AB"),
  //        "requestDate": String("2024-05-15+02:00"),
  //        "valid": String("true"),
  //        "vatNumber": String("556703748501"
  //    )}
  // }
```

### Tax Id Types

| Tax Id Type | Authority           | Lookup | Documentation                                                                                                                                                      |
| --- | --- | --- | --- |
| `eu_vat` | VIES                | [VIES](https://ec.europa.eu/taxation_customs/vies/) | [VIES](https://ec.europa.eu/taxation_customs/vies/faqvies.do)                                                                                                      |
| `gb_vat` | HMRC                | [HMRC](https://www.tax.service.gov.uk/check-vat-number/enter-vat-details) | -                                                                                                                                                                  |
| `ch_vat` | BFS                 | [BFS](https://www.uid.admin.ch/Search.aspx?lang=en) | [BFS](https://www.bfs.admin.ch/bfs/fr/home/registres/registre-entreprises/numero-identification-entreprises/registre-ide/interfaces-ide.assetdetail.11007266.html) |
| `no_vat` | Brønnøysundregistrene | [Brønnøysundregistrene](https://data.brreg.no/enhetsregisteret/oppslag/enheter) | [Brønnøysundregistrene](https://data.brreg.no/enhetsregisteret/api/dokumentasjon/no/index.html#tag/Enheter/operation/hentEnhet)                                                  |


## License

Licensed under either of

* Apache License, Version 2.0
  ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
* MIT license
  ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.
