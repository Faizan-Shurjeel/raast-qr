# Roadmap: `raast-qr` & The Pakistan Rust Fintech Ecosystem

## Phase 1: `raast-qr` Foundation (Sprint 1 — Completed)
- [x] Establish workspace (`raast-qr` core + `raast-qr-cli`).
- [x] Table-Driven CRC16-CCITT implementation with EMVCo test vectors.
- [x] Zero-copy, panic-free TLV scanner and parser (`tlv.rs`).
- [x] SBP Raast profile builder and validation rules (`raast.rs` / `builder.rs`).
- [x] Symmetric parse-path and build-path EMVCo byte-length checks.
- [x] Interoperable multi-scheme MAI support (`26..=51`) with dynamic disambiguation (`parse_with_guid`).
- [x] Standalone CLI tool (`crates/raast-qr-cli`) with `--format json`.
- [x] Automated regression tests ensuring README examples never drift from parser logic.
- [x] Property-based mutation fuzzing tests (`proptest`).
- [x] Tracked `Cargo.lock` and MSRV verified at Rust 1.85+.
- [x] Published `v0.1.5` to crates.io.

## Phase 2: Production Payload Calibration & Interop (Current)
- [ ] Collect and calibrate live scanned QR payloads from Pakistani acquirers (Meezan, HBL, 1LINK, EasyPaisa).
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
