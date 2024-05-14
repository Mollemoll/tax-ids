# tax-ids

TaxIds is a library for validating the syntax and verifying the legitimacy of business tax ids (VAT, GST etc) for various countries.

## Usage

```rust
use tax_ids::TaxId;

// Instantiate a new TaxId object
// Raises an ValidationError if the tax id does not follow country syntax
let tax_id = match TaxId::new("SE556703748501") {
    Ok(tax_id) => tax_id,
    Err(e) => println!("ValidationError: {}", e),
};

// Verify the tax id against the country's tax id database.
// Raises a VerificationError if the tax id is not legitimate
match tax_id.verify() {
    Ok(verified) => println!("Tax id is legitimate: {}", verified),
    Err(e) => println!("VerificationError: {}", e),
};
```

## Installation

With default feature `eu_vat`:
```toml
[dependencies]
tax_ids = "0.1.0"
```

### Available features

| Feature | Description | Default
| --- | --- | --- |
| `eu_vat` | European Union VAT numbers | ✓ |
| `gb_vat` | United Kingdom VAT numbers |
| `ch_vat` | Switzerland VAT numbers |
| `no_vat` | Norway VAT numbers |

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
