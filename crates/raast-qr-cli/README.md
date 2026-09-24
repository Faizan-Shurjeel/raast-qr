# raast-qr-cli

[![Crates.io](https://img.shields.io/crates/v/raast-qr-cli.svg)](https://crates.io/crates/raast-qr-cli)
[![License](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue.svg)](LICENSE-MIT)

A command-line tool to generate, inspect, and check EMVCo merchant-presented QR payloads with a provisional Raast-style account template. It uses the [`raast-qr`](https://crates.io/crates/raast-qr) library and requires `std`.

```bash
cargo install raast-qr-cli --version 0.3.0
```

## Usage

```bash
# Generate a Raast-style QR with a participant code
raast-qr-cli generate \
  --dynamic \
  --alias "+923367865823" \
  --bank-code 1234 \
  --name "Faizan Shurjeel" \
  --city "Lahore" \
  --amount 1250.50 \
  --reference "INV-2026-001"

# Check supported fields and CRC in a complete example payload
raast-qr-cli verify "00020101021226290008pk.raast0113+92336786582352045411530358654071250.505802PK5915Faizan Shurjeel6006Lahore62160112INV-2026-00163044A1D"

# Decode that same payload to JSON
raast-qr-cli decode "00020101021226290008pk.raast0113+92336786582352045411530358654071250.505802PK5915Faizan Shurjeel6006Lahore62160112INV-2026-00163044A1D" --format json

# Select the "other" MAI template in a multi-scheme example
raast-qr-cli decode "00020101021126210005other01081234567828290008pk.raast0113+9233678658235204541153035865406100.005802PK5906Faizan6006Lahore6304C12C" --scheme-guid other --format json
```

`verify` and `decode` select `pk.raast` by default; use `--scheme-guid` for another MAI template. The QR strings above are project examples, **not** verified live Raast/acquirer payloads.

## Tip and convenience fees

The CLI also supports `generate --prompt-tip`, `--fixed-fee <PKR>`, or `--percentage-fee <PERCENT>` (choose one). These produce EMVCo Merchant-Presented Mode Tags 55–57. The `decode` output exposes a tip or fee separately from Tag 54, which is the _base_ amount; `verify` does not calculate a payable total.

These fields are not bank transfer charges or merchant discount rates. Confirm with your acquirer **before issuing** a fee-bearing QR: support in the live Raast profile has not been established. See the [library documentation](https://docs.rs/raast-qr/) for API and `no_std` usage.

**Security:** CRC detects accidental changes; it does not authenticate the payee, approve charges, or guarantee that a payment succeeded. Confirm payment details via a trusted channel.

## License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE))
- MIT License ([LICENSE-MIT](LICENSE-MIT))
