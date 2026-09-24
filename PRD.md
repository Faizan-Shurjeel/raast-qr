# Product Requirements Document (PRD): `raast-qr`

- **Project:** `raast-qr` (Workspace: `raast-qr` core + `raast-qr-cli`)
- **Author:** Muhammad Faizan Shurjeel
- **Target Ecosystem:** crates.io, Pakistani Fintech, Banking Microservices, Embedded POS
- **License:** MIT or Apache-2.0

---

## 1. Objectives & Principles

1. **Fail-Closed Within Supported Fields:** Reject invalid framing, truncation, CRC mismatches, and inconsistent supported fields; parsing does not authenticate a payee or establish full SBP profile compliance.
2. **`#![no_std]` Parsing:** The TLV scanner and borrowed parse result work without `std` or `alloc`; generation uses `String` and requires `std` or the `alloc` feature. The default feature is `std`.
3. **Deterministic Financial Math:** Amounts must use fixed-point arithmetic (`rust_decimal` or integer paisas), never binary floating point (`f32`/`f64`).
4. **Profile Goal:** Implement the supported EMV® merchant-presented QR fields while working toward verified SBP Raast profile compliance. The current Raast-style MAI defaults are provisional pending live acquirer payload calibration.

---

## 2. Specification & Tag Dictionary

The payload is a continuous string composed of Tag-Length-Value (TLV) elements:
`Tag (2-character identifier) | Length (2 numeric digits: 01-99) | Value (L bytes)`

### 2.1 Supported Standard Tags

| Tag       | Field Name                      | Status      | Format            | Constraints / Expected Values                                                                                                        |
| :-------- | :------------------------------ | :---------- | :---------------- | :----------------------------------------------------------------------------------------------------------------------------------- |
| `00`      | Payload Format Indicator        | Mandatory   | Numeric, 2 digits | Must be `"01"`.                                                                                                                      |
| `01`      | Point of Initiation Method      | Optional    | Numeric, 2 digits | `"11"` (static: amount optional) or `"12"` (dynamic: amount required; single-use is not enforced by this library).                   |
| `26`–`51` | Merchant Account Info           | Mandatory   | Nested Sub-TLV    | Select an MAI template by GUID; builder defaults to Tag `26` and GUID `"pk.raast"` (not verified as an official SBP assignment).     |
| `52`      | Merchant Category Code (MCC)    | Mandatory   | Numeric, 4 digits | ISO 18245 MCC code (e.g., `5411` for Grocery).                                                                                       |
| `53`      | Transaction Currency            | Mandatory   | Numeric, 3 digits | Supported profile requires `"586"` (PKR).                                                                                            |
| `54`      | Base Transaction Amount         | Conditional | String Decimal    | Required for dynamic QR; optional for static. Max 13 bytes, up to 2 decimal places; builder emits exactly 2. Excludes fees and tips. |
| `55`      | Tip / Convenience Fee Indicator | Optional    | Numeric, 2 digits | `"01"` payer-entered tip, `"02"` fixed fee (requires Tag `56`), or `"03"` percentage fee (requires Tag `57`).                        |
| `56`      | Fixed Convenience Fee           | Conditional | String Decimal    | Positive amount, max 13 bytes and up to 2 decimal places; only with `55=02`. Not a bank transfer fee.                                |
| `57`      | Percentage Convenience Fee      | Conditional | String Decimal    | 0.01–99.99, max 5 bytes and up to 2 decimal places; only with `55=03`.                                                               |
| `58`      | Country Code                    | Mandatory   | Alpha, 2 chars    | Supported profile requires `"PK"`.                                                                                                   |
| `59`      | Merchant Name                   | Mandatory   | UTF-8             | Max 25 UTF-8 bytes; alphanumeric-only content is not enforced.                                                                       |
| `60`      | Merchant City                   | Mandatory   | UTF-8             | Max 15 UTF-8 bytes; alphanumeric-only content is not enforced.                                                                       |
| `62`      | Additional Data Field           | Optional    | Nested Sub-TLV    | Exposes Bill/Invoice reference (`62.01`); other sub-tags are not exposed.                                                            |
| `63`      | CRC Checksum                    | Mandatory   | Hex, 4 chars      | CRC16-CCITT over entire string up to `6304`.                                                                                         |

### 2.2 Selected Merchant Account Sub-Tags (Tag 26–51)

| Sub-Tag | Field Name              | Description                                                                                               |
| :------ | :---------------------- | :-------------------------------------------------------------------------------------------------------- |
| `00`    | Scheme GUID             | Configurable; defaults to `"pk.raast"`. An official SBP GUID assignment has not been verified here.       |
| `01`    | Account Identifier      | Accepts an alias or IBAN as an opaque value; format is not verified against a live scheme.                |
| `02`    | Bank / Participant Code | Optional, 1–20 UTF-8 bytes in the current implementation; no verified four-character SBP/1LINK code rule. |

---

## 3. Functional Requirements

### 3.1 Parser Engine

- Must scan an EMV string and yield a borrowed, validated representation of supported fields (`RaastQr`).
- Must verify that Tag `00` is first and Tag `63` is the terminal tag.
- Must verify Tag `63` CRC16-CCITT before parsing payload details to reject corrupt strings immediately.
- Must reject any payload exceeding the maximum EMVCo payload length (512 bytes).

### 3.2 Builder / Serializer Engine

- Builder checks mandatory fields at `build_emv_string()` and returns structured runtime validation errors; it requires `std` or `alloc`.
- Automatic computation and injection of Tag `63` CRC.
- Formatting of base amount (`rust_decimal::Decimal`) into exactly two-decimal-place strings without scientific notation; fee values are encoded separately in Tags `55`–`57`, not as a computed payable total.

### 3.3 CRC16-CCITT Specification

- **Polynomial:** `0x1021` ($x^{16} + x^{12} + x^5 + 1$)
- **Initial Value:** `0xFFFF`
- **Reflect In / Out:** `false`
- **Xor Out:** `0x0000`
- **Calculation Input:** String bytes up to and including `"6304"`.
- **Output:** 4-character uppercase hexadecimal (e.g., `"E2D1"`).

---

## 4. Workspace Architecture

```text
raast-qr/
├── Cargo.toml                # Workspace manifest
├── crates/
│   ├── raast-qr/             # Core crate (std default; no_std parsing supported)
│   │   ├── Cargo.toml
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── error.rs      # Fail-closed Error enum
│   │   │   ├── crc.rs        # Table-driven CRC16-CCITT
│   │   │   ├── tlv.rs        # Zero-alloc TLV scanner
│   │   │   ├── raast.rs      # Provisional Raast-style model & validation
│   │   │   └── builder.rs    # Allocating builder (std or alloc feature)
│   │   └── tests/
│   │       ├── vectors.rs    # Locally authored Raast-style regression payload
│   │       └── fee.rs        # Fee parsing and builder tests
│   └── raast-qr-cli/         # Standalone CLI binary
│       ├── Cargo.toml
│       ├── src/
│       │   └── main.rs       # decode, generate, verify subcommands
│       └── tests/
│           └── cli.rs        # CLI integration tests
```

---

## 5. Non-Functional Requirements

- **MSRV:** Rust 1.85+ (Edition 2021).
- **Zero Panics:** No `unwrap()` or `expect()` in production code paths.
- **Performance Targets (not yet benchmarked):** Parsing & CRC verification under 5 microseconds on modern x86_64 and under 25 microseconds on aarch64 (Raspberry Pi 4).
