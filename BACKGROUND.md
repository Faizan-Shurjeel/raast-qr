# Background & Genesis: The `raast-qr` Initiative

## 1. Executive Origin
This initiative was conceptualized by **Muhammad Faizan Shurjeel** (Systems & Backend Engineer; Author of `okf-attestor` on crates.io and `Verascope` offline C2PA provenance verifier). 

The catalyst was an industry discourse on LinkedIn regarding PayFast (Avanza Premier Payment Services - APPS), Flutter integrations, and mobile payment enablement in Pakistan. An architectural audit of Pakistan's digital payment rails revealed a critical deficiency: **the Rust ecosystem for Pakistani financial infrastructure is virtually non-existent.**

While modern financial institutions and payment service operators (PSOs/PSPs) are scaling digital adoption, developers are relegated to fragile PHP wrappers, untyped Node.js snippets, and manual string concatenations for critical transaction pipelines.

## 2. Landscape Analysis: Pakistan Payment Rails
1. **State Bank of Pakistan (SBP) & Raast:**
   - Raast is Pakistan's instant payment system developed by the SBP to drive financial inclusion.
   - For merchant payments (P2M) and person-to-person payments (P2P), SBP standardized on the **EMVCo Merchant-Presented Mode (MPM)** Quick Response Code specification.
2. **Current Deficiencies:**
   - Most implementations construct the payload via raw string formatting (e.g., `format!("00020101021226{}...", account)`).
   - Tag lengths are routinely miscalculated when merchant strings contain non-ASCII characters or variable-length aliases.
   - Tag 63 (CRC16-CCITT) calculation is frequently buggy or implemented with floating-point amounts that trigger silent failures at POS scanners.
   - Zero fail-closed parsers exist in the ecosystem to validate whether an incoming QR code payload satisfies SBP compliance before initiating bank transfers.

## 3. The Multi-Phase Strategy
To establish authoritative Rust infrastructure in Pakistan's fintech domain, four projects were evaluated:
- **Project 1: `raast-qr` (The Primitive)** — A `#![no_std]`, zero-allocation parser, builder, and validator for EMVCo MPM compliant with SBP Raast specifications.
- **Project 2: `payfast-rs` (The Industrial Gateway SDK)** — Async Axum/Actix client and webhook sentinel for APPS/PayFast integrations.
- **Project 3: `veraslip` (Cryptographic Anti-Slip-Scam Protocol)** — Signed Ed25519 CBOR receipts verifying transactions offline (combining C2PA concepts with payment confirmation).
- **Project 4: Flutter POS Bridge** — Cross-platform state machine (deprioritized to focus on pure Rust systems).

**Decision:** Execute **Project 1 (`raast-qr`)** as the initial beachhead. It solves an immediate real-world failure mode, requires zero third-party network access to test, delivers maximum crate utility, and forms the bedrock upon which `payfast-rs` and `veraslip` will operate.
