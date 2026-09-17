# raast-qr

[![Crates.io](https://img.shields.io/crates/v/raast-qr.svg)](https://crates.io/crates/raast-qr)
[![Documentation](https://docs.rs/raast-qr/badge.svg)](https://docs.rs/raast-qr)
[![License](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue.svg)](LICENSE-MIT)
[![No-Std](https://img.shields.io/badge/no--std-supported-success.svg)](#)

A high-performance, `#![no_std]`-compatible, fail-closed EMVCo Merchant-Presented Mode (MPM) QR engine tailored for Pakistan's payment rails and the **State Bank of Pakistan (SBP) Raast P2M & P2P** specifications.

Written in pure Rust with zero heap allocations on parsing, deterministic fixed-point financial math, and table-driven CRC16-CCITT validation.

---

## Features

- **Interoperable MAI Architecture:** Supports Merchant Account Information across tags `26..=51` and customizable scheme GUIDs.
- **Fail-Closed Security:** Hard bounds on EMVCo byte lengths, non-ASCII boundary checks to prevent panic vectors, and duplicate-tag smuggling guards.
- **Zero-Alloc Parsing:** Borrows directly from byte slices (`&str` / `&[u8]`) without heap overhead.
- **Safe Currency & Amounts:** Zero floating-point drift using `rust_decimal`.
- **Standalone CLI:** Inspect, generate, verify, and output structured JSON from the terminal.

---

## Installation

Add `raast-qr` to your `Cargo.toml`:

```toml
[dependencies]
raast-qr = "0.1.11"
rust_decimal = "1.33"
```

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

### 2. Parsing & Validating an Incoming QR Payload

```rust
use raast_qr::RaastQr;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Reference EMVCo dynamic QR string (Raast P2M profile) (CRC: 4A1D)
    let raw = "00020101021226290008pk.raast0113+92336786582352045411530358654071250.505802PK5915Faizan Shurjeel6006Lahore62160112INV-2026-00163044A1D";

    let parsed = RaastQr::parse(raw)?;

    assert_eq!(parsed.merchant_name, "Faizan Shurjeel");
    assert_eq!(parsed.raast_id, "+923367865823");
    println!("Verified Merchant: {}", parsed.merchant_name);
    println!("Amount: {:?} PKR", parsed.amount);

    Ok(())
}
```

---

## CLI Usage

```bash
# Verify payload integrity
raast-qr verify "000201010212..."

# Decode to machine-readable JSON
raast-qr decode "000201010212..." --format json

# Generate a compliant payload
raast-qr generate \
  --dynamic \
  --alias "+923367865823" \
  --name "Faizan Shurjeel" \
  --city "Lahore" \
  --amount 1250.50 \
  --reference "INV-2026-001"
```

---

## License

Licensed under either of:
- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE))
- MIT License ([LICENSE-MIT](LICENSE-MIT))

