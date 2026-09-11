# Roadmap: `raast-qr` & The Pakistan Rust Fintech Ecosystem

## Phase 1: `raast-qr` Foundation (Sprint 1 — 3 Days)
- [ ] Establish workspace (`raast-qr` core + `raast-qr-cli`).
- [ ] Implement Table-Driven CRC16-CCITT with EMVCo test vectors.
- [ ] Implement zero-copy TLV scanner and parser (`winnow`/`nom` or hand-rolled).
- [ ] Build SBP Raast profile builder and validation rules (PKR `586`, Tag 26 Sub-tags).
- [ ] Add property-based testing (`proptest`) checking roundtrip invariant: `parse(build(x)) == x`.
- [ ] Publish `v0.1.0` to crates.io.

## Phase 2: Interop & Ecosystem Growth (Sprint 2)
- [ ] Feature flag `image`/`qrcode` for instant SVG/PNG generation in CLI.
- [ ] Create `raast-qr-wasm` for in-browser client generation.
- [ ] Write integration tutorials for Axum/Actix-web checkout microservices.

## Phase 3: `payfast-rs` (Industrial Gateway SDK)
- [ ] Build `payfast-core` (pure protocol, serde models, payload signing).
- [ ] Build `payfast-client` (async checkout client for Cards, Raast RTP, and Direct Bank Debit).
- [ ] Implement `PayFastSentinel`: constant-time HMAC-SHA256 IPN/Webhook validation for Axum.

## Phase 4: `veraslip` Standard (Anti-Slip-Scam Protocol)
- [ ] Define canonical CBOR receipt schema for point-of-sale payments.
- [ ] Implement Ed25519 signing by payment gateways / merchant terminals.
- [ ] Integrate C2PA provenance verification logic adapted from `Verascope` for tamper-evident digital receipts.
