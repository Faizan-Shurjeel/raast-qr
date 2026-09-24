# Roadmap: `raast-qr` & The Pakistan Rust Fintech Ecosystem

## Phase 1: `raast-qr` Foundation (Sprint 1 — Completed)

- [x] Establish workspace (`raast-qr` core + `raast-qr-cli`).
- [x] Table-driven CRC16-CCITT implementation with a standard check vector and an EMVCo-style CRC example; no sourced official SBP full-payload vectors yet.
- [x] Zero-copy TLV scanner and borrowed parser (`tlv.rs` / `raast.rs`); builder requires `std` or `alloc`.
- [x] Provisional Raast-style builder and validation rules (`raast.rs` / `builder.rs`); SBP profile compliance awaits live payload calibration.
- [x] Symmetric parse-path and build-path EMVCo byte-length checks.
- [x] MAI tag support (`26..=51`) with GUID selection (`parse_with_guid`); live multi-network interoperability is not yet verified.
- [x] Standalone CLI (`crates/raast-qr-cli`) with `--format json`, `generate --bank-code`, and `verify` / `decode --scheme-guid`.
- [x] Regression tests for a copied example payload and builder/parser round trips; README examples are not automatically synchronized.
- [x] Property-based mutation tests (`proptest`) and parser/fee tests compatible with no-default-features; CI checks `no_std` and `alloc` builds but does not run no-default-features tests.
- [x] Tracked `Cargo.lock` and CI MSRV check at Rust 1.85 (Edition 2021).
- [x] Published both `raast-qr` and `raast-qr-cli` at `0.3.0` (earlier `0.1.5` milestone completed).
- [x] Added Tags `55`–`57` tip/convenience-fee parsing and generation, separate from Tag `54` base amount.

## Phase 2: Production Payload Calibration & Interop (Current)

- [ ] Collect and calibrate live scanned QR payloads from Pakistani acquirers (e.g., Meezan, HBL, 1LINK, EasyPaisa); do not claim official Raast profile compliance until verified.
- [ ] Feature flag `image`/`qrcode` for instant terminal/file SVG/PNG generation in CLI.
- [ ] Create `raast-qr-wasm` for in-browser client generation.

## Phase 3: `payfast-rs` (Industrial Gateway SDK)

- [ ] Build `payfast-core` (pure protocol, serde models, payload signing).
- [ ] Build `payfast-client` (async checkout client for Cards, Raast RTP, and Direct Bank Debit).
- [ ] Implement `PayFastSentinel`: constant-time HMAC-SHA256 IPN/Webhook validation for Axum.

## Phase 4: `veraslip` Standard (Anti-Slip-Scam Protocol)

- [ ] Define canonical CBOR receipt schema for point-of-sale payments.
- [ ] Implement Ed25519 signing by payment gateways / merchant terminals.
- [ ] Integrate C2PA provenance verification logic adapted from `Verascope` for tamper-evident digital receipts.
