# Operational Guidelines: AI Agents & Human Maintainers

These rules are strictly enforced for any autonomous agent (Claude Code, Cursor, Copilot) or contributor submitting pull requests to `raast-qr`.

---

## 1. Architectural Invariants
1. **Never use `unwrap()` or `expect()` in library code (`crates/raast-qr/src/`).**
   - Every failure must bubble up via `raast_qr::error::RaastError`.
2. **Deterministic Financial Math:**
   - Never represent money as `f32` or `f64`. Use `rust_decimal::Decimal` or integer paisas (`u64`).
3. **Fail-Closed Parsing:**
   - If an EMV string has trailing bytes after the Tag 63 CRC value, the parser **must reject** it.
   - If the CRC does not match the computed CRC16-CCITT checksum, return `RaastError::InvalidChecksum`.
4. **Length Enforcement:**
   - Tag lengths in EMVCo are byte counts, not UTF-8 character counts. Ensure multi-byte UTF-8 strings do not trigger buffer over-reads or under-reads.

---

## 2. Coding Patterns

### CRC16-CCITT Verification
CRC must be computed across all bytes preceding the checksum value:
```text
[ Tag 00 ... up to and including "6304" ]  --> Compute CRC16
Compare with value in indices [len - 4 .. len]
```

### Zero-Allocation AST
Prefer borrowing where possible:
```rust
pub struct RawTlv<'a> {
    pub tag: &'a str,
    pub length: usize,
    pub value: &'a str,
}
```

---

## 3. Pre-Commit Verification Checklist
Before submitting a patch, execute:
```bash
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo test --workspace --no-default-features
```
