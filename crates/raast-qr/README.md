# raast-qr

[![Crates.io](https://img.shields.io/crates/v/raast-qr.svg)](https://crates.io/crates/raast-qr)
[![Documentation](https://docs.rs/raast-qr/badge.svg)](https://docs.rs/raast-qr)
[![License](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue.svg)](LICENSE-MIT)
[![No-Std](https://img.shields.io/badge/no--std-supported-success.svg)](#)

A `#![no_std]`-capable EMVCo Merchant-Presented Mode (MPM) QR parser and generator with a provisional Raast-style account template for Pakistan. Live SBP/acquirer profile interoperability has not yet been verified; parsing a QR does not authenticate its payee.

Written in pure Rust with zero heap allocations on parsing, deterministic fixed-point financial math, and table-driven CRC16-CCITT validation.

---

## Features

- **Interoperable MAI Architecture:** Supports Merchant Account Information across tags `26..=51` and customizable scheme GUIDs.
- **Checked Fields:** Validates CRC, TLV byte lengths, mandatory profile fields, and supported fee-tag combinations; unknown tags may be ignored.
- **Zero-Alloc Parsing:** Borrows from an input `&str` without allocating; generation requires `std` or `alloc`.
- **Safe Currency & Amounts:** Zero floating-point drift using `rust_decimal`.
- **Standalone CLI:** Inspect, generate, verify, and output structured JSON from the terminal.

---

## Installation

Add `raast-qr` to your `Cargo.toml`:

```toml
[dependencies]
raast-qr = "0.3.0"
rust_decimal = "1.33"
rust_decimal_macros = "1.33"
```

For `no_std` parsing without an allocator, use `raast-qr = { version = "0.3.0", default-features = false }`. For `no_std` generation, enable `features = ["alloc"]` too. The default configuration enables `std`.

Or install the CLI tool:

```bash
cargo install raast-qr-cli
```

---

## Quick Start

### 1. Generating a Dynamic Raast P2M QR Code

```rust
use raast_qr::{RaastQr, InitiationMethod};
use rust_decimal_macros::dec;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let qr = RaastQr::builder()
        .initiation_method(InitiationMethod::Dynamic)
        .raast_alias("+923367865823")
        .merchant_name("Faizan Shurjeel")
        .merchant_city("Lahore")
        .mcc("5411")
        .amount(dec!(1250.50))
        .bill_reference("INV-2026-001")
        .build_emv_string()?;

    println!("Raast QR String:\n{}", qr);
    Ok(())
}
```

### 2. Parsing and Checking an Incoming QR Payload

```rust
use raast_qr::RaastQr;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Example dynamic QR string (Raast P2M profile) (CRC: 4A1D)
    let raw = "00020101021226290008pk.raast0113+92336786582352045411530358654071250.505802PK5915Faizan Shurjeel6006Lahore62160112INV-2026-00163044A1D";

    let parsed = RaastQr::parse(raw)?;

    assert_eq!(parsed.merchant_name, "Faizan Shurjeel");
    assert_eq!(parsed.raast_id, "+923367865823");
    println!("Merchant field: {}", parsed.merchant_name);
    println!("Base amount: {:?} PKR", parsed.amount);

    Ok(())
}
```

---

## Tip and convenience fee tags

EMVCo MPM v1.1 (sections 4.7.6–4.7.8) defines Tag `55=01` to request a payer-entered tip, `55=02` plus Tag `56` for a fixed fee, and `55=03` plus Tag `57` for a percentage fee. `RaastQr::parse` validates these combinations and exposes `fee: Option<Fee>` separately from the **base** `amount` (Tag 54). For generation, use `.fee(Fee::Fixed(dec!(10.00)))` or `.fee(Fee::Percentage(dec!(2.50)))` on the builder. Without a base amount, or for a prompted tip, a total cannot be determined from the QR alone.

These are _merchant-defined QR fields_, not bank transfer charges or merchant discount rates. The [SBP P2P circular](https://www.sbp.org.pk/disd/2022/C1.htm) requires free Raast P2P transactions; the [QR standardization circular](https://www.sbp.org.pk/disd/2022/CL1.htm) discusses acquirer charges separately. Confirm with your Raast acquirer before **issuing** fee-bearing QRs; support for them in the Raast QR profile has not been established here. Parsing the fields does not authorize charging them.

## CLI Usage

`verify` checks supported fields and CRC, not the payee's identity or transaction authenticity. These examples use project-generated payloads, not verified live Raast QRs.

```bash
# Check QR format and checksum
raast-qr-cli verify "00020101021226290008pk.raast0113+92336786582352045411530358654071250.505802PK5915Faizan Shurjeel6006Lahore62160112INV-2026-00163044A1D"

# Decode to machine-readable JSON
raast-qr-cli decode "00020101021226290008pk.raast0113+92336786582352045411530358654071250.505802PK5915Faizan Shurjeel6006Lahore62160112INV-2026-00163044A1D" --format json

# Select a different MAI template from a multi-scheme QR
raast-qr-cli decode "00020101021126210005other01081234567828290008pk.raast0113+9233678658235204541153035865406100.005802PK5906Faizan6006Lahore6304C12C" --scheme-guid other --format json

# Generate a Raast-format payload
raast-qr-cli generate \
  --dynamic \
  --alias "+923367865823" \
  --bank-code 1234 \
  --name "Faizan Shurjeel" \
  --city "Lahore" \
  --amount 1250.50 \
  --reference "INV-2026-001"

# Only with acquirer approval: choose ONE of --prompt-tip,
# --fixed-fee 10.00, or --percentage-fee 2.50 when generating.
```

---

## License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE))
- MIT License ([LICENSE-MIT](LICENSE-MIT))
