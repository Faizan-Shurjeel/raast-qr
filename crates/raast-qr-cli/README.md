# raast-qr

[![Crates.io](https://img.shields.io/crates/v/raast-qr.svg)](https://crates.io/crates/raast-qr)
[![Documentation](https://docs.rs/raast-qr/badge.svg)](https://docs.rs/raast-qr)
[![License](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue.svg)](LICENSE)
[![No-Std](https://img.shields.io/badge/no--std-supported-success.svg)](#)

A high-performance, `#![no_std]`-compatible, fail-closed EMVCo Merchant-Presented Mode (MPM) QR engine tailored for the **State Bank of Pakistan (SBP) Raast P2M & P2P** payment rails.

Written in pure Rust with zero heap allocations on parsing, deterministic fixed-point financial math, and strict CRC16-CCITT validation.

---

## Features

- **Strictly Conforming:** Fully implements EMVCo MPM v1.0 and SBP Raast QR guidelines.
- **Fail-Closed Security:** Corrupt lengths, missing checksums, or malformed tags fail immediately.
- **Zero-Alloc Parsing:** Parse slices directly (`&str` / `&[u8]`) without heap overhead.
- **Safe Currency & Amounts:** Zero floating-point drift using `rust_decimal`.
- **Standalone CLI:** Inspect, generate, and verify Raast payloads directly from the terminal.

---

## Installation

Add `raast-qr` to your `Cargo.toml`:

```toml
[dependencies]
raast-qr = "0.1.0"
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
use raast_qr::{RaastQr, InitiationMethod, Currency};
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
        .build()?;

    let emv_payload = qr.to_emv_string();
    println!("Raast QR String:\n{}", emv_payload);

    // Optional: render to SVG/PNG with `image` feature enabled
    // qr.to_svg("output.svg")?;

    Ok(())
}
```

### 2. Parsing & Validating an Incoming QR Payload

```rust
use raast_qr::RaastQr;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let raw = "00020101021226320008pk.raast0114+92336786582352045411530358654071250.505802PK5915Faizan Shurjeel6006Lahore62160112INV-2026-0016304E1D2";

    // Validates Tag 63 CRC16, length boundaries, and Raast profile constraints
    let parsed = RaastQr::parse(raw)?;

    println!("Merchant: {}", parsed.merchant_name());
    println!("Amount: {:?} PKR", parsed.amount());
    println!("Dynamic Checkout: {}", parsed.is_dynamic());

    Ok(())
}
```

---

## CLI Usage

Verify a payload:
```bash
raast-qr verify "000201010212..."
```

Decode payload to structured JSON:
```bash
raast-qr decode "000201010212..." --format json
```

Generate a dynamic payload:
```bash
raast-qr generate \
  --dynamic \
  --alias "+923367865823" \
  --name "Al-Madina Mart" \
  --city "Lahore" \
  --amount 850.00 \
  --ref "ORDER-9912"
```

---

## License

Licensed under either of:
- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE))
- MIT License ([LICENSE-MIT](LICENSE-MIT))
