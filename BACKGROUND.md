# Background & Genesis: The `raast-qr` Initiative

## 1. Executive Origin

This initiative was conceptualized by **Muhammad Faizan Shurjeel** (Systems & Backend Engineer; Author of `okf-attestor` on crates.io and `Verascope` offline C2PA provenance verifier).

The catalyst was an industry discourse on LinkedIn regarding PayFast (Avanza Premier Payment Services - APPS), Flutter integrations, and mobile payment enablement in Pakistan. The project was motivated by a perceived gap in Rust tooling for Pakistani financial infrastructure; its ecosystem-wide size has not been measured here.

For financial institutions and payment service operators (PSOs/PSPs), typed payload construction and validation can reduce reliance on manual string concatenation in transaction pipelines.

## 2. Landscape Analysis: Pakistan Payment Rails

1. **State Bank of Pakistan (SBP) & Raast:**
   - Raast is Pakistan's instant payment system developed by the SBP to drive financial inclusion.
   - This project targets EMVCo-style merchant-presented (P2M) QR payloads. P2P profile claims and official SBP Raast field assignments require separate evidence.
2. **Implementation Risks Addressed:**
   - Manually assembled payloads (e.g., `format!("00020101021226{}...", account)`) can be hard to validate.
   - Non-ASCII merchant strings and variable-length aliases can expose mistakes in byte-counted tag lengths.
   - An incorrect Tag 63 CRC16-CCITT or use of floating-point amounts can produce incorrect payment payloads.
   - `raast-qr` now provides a CRC-checking parser for its supported subset; parsing alone cannot certify SBP compliance or authenticate a payee.

## 3. The Multi-Phase Strategy

As a long-term goal to strengthen Rust infrastructure in Pakistan's fintech domain, four projects were evaluated:

- **Project 1: `raast-qr` (The Primitive)** — A `no_std`-capable borrowed parser and validator for an EMVCo MPM-style subset, plus an allocating `std`/`alloc` builder. Live acquirer payload calibration is still needed before claiming SBP Raast profile compliance.
- **Project 2: `payfast-rs` (The Industrial Gateway SDK)** — Async Axum/Actix client and webhook sentinel for APPS/PayFast integrations.
- **Project 3: `veraslip` (Cryptographic Anti-Slip-Scam Protocol)** — Signed Ed25519 CBOR receipts verifying transactions offline (combining C2PA concepts with payment confirmation).
- **Project 4: Flutter POS Bridge** — Cross-platform state machine (deprioritized to focus on pure Rust systems).

**Decision:** Execute **Project 1 (`raast-qr`)** as the initial beachhead. Its local payload tests do not require a payment network; the project aims to provide a foundation for future `payfast-rs` and `veraslip` work.
