# Product Requirements Document (PRD): `raast-qr`

- **Project:** `raast-qr` (Workspace: `raast-qr` core + `raast-qr-cli`)
- **Author:** Muhammad Faizan Shurjeel
- **Target Ecosystem:** crates.io, Pakistani Fintech, Banking Microservices, Embedded POS
- **License:** MIT or Apache-2.0

---

## 1. Objectives & Principles
1. **Fail-Closed:** An invalid, truncated, tampered, or non-compliant string must never parse as valid.
2. **Zero-Alloc / `#![no_std]` First:** Core parsing and generation must function in embedded systems, POS terminals, and WASM targets without requiring an allocator where possible.
3. **Deterministic Financial Math:** Amounts must use fixed-point arithmetic (`rust_decimal` or integer paisas), never binary floating point (`f32`/`f64`).
4. **Complete Spec Compliance:** Fully adhere to the EMV® QR Code Specification for Payment Systems (Merchant-Presented Mode) v1.0, targeting the State Bank of Pakistan (SBP) Raast profile (provisional pending live merchant scheme calibration).

---

## 2. Specification & Tag Dictionary

The payload is a continuous string composed of Tag-Length-Value (TLV) elements:
`Tag (2 alphanumeric) | Length (2 numeric digits: 01-99) | Value (L bytes)`

### 2.1 Supported Standard Tags
| Tag | Field Name | Status | Format | Constraints / Expected Values |
| :--- | :--- | :--- | :--- | :--- |
| `00` | Payload Format Indicator | Mandatory | Numeric, 2 digits | Must be `"01"`. |
| `01` | Point of Initiation Method | Optional | Numeric, 2 digits | `"11"` (Static: reusable, no amount) or `"12"` (Dynamic: single-use, strict amount). |
| `26`–`51` | Merchant Account Info | Mandatory | Nested Sub-TLV | Reserved for Payment Networks. Raast uses Tag `26` or assigned ID. |
| `52` | Merchant Category Code (MCC) | Mandatory | Numeric, 4 digits | ISO 18245 MCC code (e.g., `5411` for Grocery). |
| `53` | Transaction Currency | Mandatory | Numeric, 3 digits | ISO 4217 numeric. For Pakistan, must be `"586"` (PKR). |
| `54` | Transaction Amount | Conditional | String Decimal | Mandatory if Tag 01 == `"12"`. Max 13 chars, 2 decimal places. |
| `58` | Country Code | Mandatory | Alpha, 2 chars | ISO 3166-1 alpha-2. Must be `"PK"`. |
| `59` | Merchant Name | Mandatory | Alphanumeric | Max 25 characters. |
| `60` | Merchant City | Mandatory | Alphanumeric | Max 15 characters (e.g., `"Lahore"`, `"Karachi"`). |
| `62` | Additional Data Field | Optional | Nested Sub-TLV | Bill/Invoice number, Mobile, Reference ID. |
| `63` | CRC Checksum | Mandatory | Hex, 4 chars | CRC16-CCITT over entire string up to `6304`. |

### 2.2 Raast Merchant Account Sub-Tags (Tag 26)
| Sub-Tag | Field Name | Description |
| :--- | :--- | :--- |
| `00` | Reverse Domain / GUID | Identifies Raast scheme: `"pk.raast"` or SBP standard identifier. |
| `01` | Raast Identifier | Merchant IBAN (`PK..`) or Registered Phone Alias (`+92..`). |
| `02` | Bank / Participant Code | 4-character 1LINK / SBP participant bank code (optional if IBAN used). |

---

## 3. Functional Requirements

### 3.1 Parser Engine
- Must scan an EMV string and yield an immutable, verified AST (`RaastQr`).
- Must verify that Tag `00` is first and Tag `63` is the terminal tag.
- Must verify Tag `63` CRC16-CCITT before parsing payload details to reject corrupt strings immediately.
- Must reject any payload exceeding the maximum EMVCo payload length (512 bytes).

### 3.2 Builder / Serializer Engine
- Type-safe builder enforcing all mandatory fields at compile-time or via structured runtime validation errors.
- Automatic computation and injection of Tag `63` CRC.
- Formatting of `rust_decimal::Decimal` into exactly two-decimal-place strings without scientific notation.

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
│   ├── raast-qr/             # Core crate (#![no_std] by default)
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── error.rs      # Fail-closed Error enum
│   │       ├── crc.rs        # Table-driven CRC16-CCITT
│   │       ├── tlv.rs        # Zero-alloc TLV scanner
│   │       ├── raast.rs      # SBP Raast profiles & validation
│   │       └── builder.rs    # Ergonomic Builder
│   └── raast-qr-cli/         # Standalone CLI binary
│       ├── Cargo.toml
│       └── src/
│           └── main.rs       # decode, generate, verify subcommands
└── tests/
    └── vectors.rs            # EMVCo & SBP official test vectors
```

---

## 5. Non-Functional Requirements
- **MSRV:** Rust 1.85+ (Edition 2024 ecosystem baseline)
- **Zero Panics:** No `unwrap()` or `expect()` in production code paths.
- **Performance:** Parsing & CRC verification must complete in under 5 microseconds on modern x86_64 and under 25 microseconds on aarch64 (Raspberry Pi 4).

