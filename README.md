# Tax Ids

This crate offers a solution for validating tax IDs (VAT/GST) for businesses operating within the European Union,
the United Kingdom, Switzerland, and Norway.

Currently, the library provides the following functionalities:  
- Validates the syntax of a tax ID against its type-specific regex pattern.
- Verifies the tax ID in the relevant government database (based on the tax ID type).

The library has been inspired by the [valvat](https://github.com/yolk/valvat) library for Ruby.

### Available features / tax id types

| Feature  | Description        | Default |
|----------|--------------------|---------|
| `eu_vat` | European Union VAT | ✓       |
| `gb_vat` | United Kingdom VAT |         |
| `ch_vat` | Switzerland VAT    |         |
| `no_vat` | Norway VAT         |         |

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
  // Instantiate a new TaxId object. This can raise a ValidationError.
  let tax_id = match TaxId::new("SE556703748501") {
    Ok(tax_id) => tax_id,
    Err(e) => {
      println!("ValidationError: {}", e);
      return;
    }
  };

  assert_eq!(tax_id.value(), "SE556703748501");
  assert_eq!(tax_id.country_code(), "SE");
  assert_eq!(tax_id.tax_country_code(), "SE");
  assert_eq!(tax_id.local_value(), "556703748501");
  assert_eq!(tax_id.tax_id_type(), "eu_vat");

  // The country code is the 2-char ISO code of the country.
  // It's often the same as the tax country code, but not always.
  // For example, the country code for Greece is GR, but EL for the Greek VAT number.

  // The United Kingdom has a country code GB and tax country code GB.
  // However, due to Brexit, businesses in Northern Ireland
  // have a country code GB but use VAT number/tax country code XI when trading
  // with the EU.

  // Verification

  // Perform a verification request against the country's tax ID database.
  // This can raise a VerificationError.
  let verification = match tax_id.verify() {
    Ok(verification) => verification,
    Err(e) => {
      println!("VerificationError: {}", e);
      return;
    },
  };
  
  assert_eq!(verification.status(), &tax_ids::VerificationStatus::Verified);

  // VerificationStatus can take one out of three different statuses:
  // - Verified - The tax ID is legitimate.
  // - Unverified - The tax ID is not legitimate.
  // - Unavailable - The verification couldn't be performed (due to rate limit, database unavailability, etc.).

  // These statuses are what you want to act upon.
  match verification.status() {
    tax_ids::VerificationStatus::Verified => {
      // Proceed with payment
    },
    tax_ids::VerificationStatus::Unverified => {
      // Ask the customer to provide a proper tax ID
    },
    tax_ids::VerificationStatus::Unavailable => {
      // Process payment and verify the tax ID later?
    },
  }

  // The full verification object:

  println!("{:?}", verification);

  // The data field is experimental and subject to change or removal.
  // It will contain different data depending on what tax ID type is being verified.
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
}
```

### Tax Id Types

| Tax Id Type | Authority                                                                                                   | Manual lookup                                                           | Documentation                                                                                                                                                     |
|-------------|-------------------------------------------------------------------------------------------------------------|-------------------------------------------------------------------------|-------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| `eu_vat`    | [VIES](https://ec.europa.eu/taxation_customs/vies/#/faq)                                                    | [🔍](https://ec.europa.eu/taxation_customs/vies/)                       | [📖](https://ec.europa.eu/taxation_customs/vies/#/technical-information) + [Availability](https://ec.europa.eu/taxation_customs/vies/#/help)                      |
| `gb_vat`    | [HMRC](https://www.gov.uk/government/organisations/hm-revenue-customs)                                      | [🔍](https://www.tax.service.gov.uk/check-vat-number/enter-vat-details) | [📖](https://developer.service.hmrc.gov.uk/api-documentation/docs/api/service/vat-registered-companies-api/1.0/oas/page)                                          |
| `ch_vat`    | [BFS](https://www.bfs.admin.ch/bfs/en/home/registers/enterprise-register/business-enterprise-register.html) | [🔍](https://www.uid.admin.ch/Search.aspx?lang=en)                      | [📖](https://www.bfs.admin.ch/bfs/fr/home/registres/registre-entreprises/numero-identification-entreprises/registre-ide/interfaces-ide.assetdetail.11007266.html) |
| `no_vat`    | [Brønnøysundregistrene](https://www.brreg.no/)                                                              | [🔍](https://data.brreg.no/enhetsregisteret/oppslag/enheter)            | [📖](https://data.brreg.no/enhetsregisteret/api/dokumentasjon/no/index.html#tag/Enheter/operation/hentEnhet)                                                      |

### License

Licensed under either of

- Apache License, Version 2.0
  ([LICENSE-APACHE](https://github.com/Mollemoll/tax-ids?tab=Apache-2.0-1-ov-file) or <http://www.apache.org/licenses/LICENSE-2.0>)
- MIT license
  ([LICENSE-MIT](https://github.com/Mollemoll/tax-ids?tab=MIT-2-ov-file) or <http://opensource.org/licenses/MIT>)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.
