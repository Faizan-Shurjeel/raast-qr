# raast-qr-cli Parity Roadmap

**Completed in 0.3.0.** Both `raast-qr` and `raast-qr-cli` are published at `0.3.0`, and the workspace version is `0.3.0`. The `generate --bank-code` and `verify` / `decode --scheme-guid` gaps below are closed; CLI integration tests cover them. Tags `55`–`57` fee/tip handling was also added to the library and CLI in 0.3.0. Live acquirer payload calibration remains open; these features do not prove official SBP Raast profile compliance.

**Historical scope (0.2.0 library / 0.1.11 CLI):** bring the published CLI back in sync with the published library. This plan was based on a comparison of their published source. The table and steps below preserve the original gap analysis and are **not current instructions to edit or publish 0.3.0 again**.

## Historical status (before 0.3.0)

| Capability                               | `raast-qr` 0.2.0 (library)                                                                                  | `raast-qr-cli` 0.1.11                                                             |
| ---------------------------------------- | ----------------------------------------------------------------------------------------------------------- | --------------------------------------------------------------------------------- |
| Bank / participant code (MAI sub-tag 02) | Settable via `RaastQrBuilder::bank_code()`, parsed into `RaastQr.bank_code`                                 | `decode` reads and prints it; `generate` has **no flag** to set it                |
| Multi-scheme GUID selection              | `RaastQr::parse_with_guid(raw, guid)` picks a specific MAI template out of a payload carrying more than one | `verify` / `decode` only call `RaastQr::parse`, which always assumes `"pk.raast"` |
| Workspace version                        | `0.2.0`                                                                                                     | Still published at `0.1.11`                                                       |

At the time, these were two CLI gaps in `crates/raast-qr-cli/src/main.rs`; the bank-code and GUID APIs already existed in the library. Both CLI gaps are now resolved. Fee support required separate library and CLI changes for 0.3.0.

## 1. `--bank-code` on `generate` (historical plan; completed)

**Why (at the time)**: the CLI could read a bank code from a QR (`decode`) but could not set one on `generate`. Asymmetric.

**Original implementation steps (completed)**:

1. Add a field to the `Generate` variant, next to `mai_tag` / `scheme_guid`:
   ```rust
   /// Bank / Participant Code (MAI sub-tag 02, e.g. 1LINK/SBP participant code)
   #[arg(long)]
   bank_code: Option<String>,
   ```
2. Add `bank_code` to the destructure in the `Commands::Generate { ... }` arm.
3. Wire it into the builder chain, same shape as `reference`:
   ```rust
   if let Some(bank) = bank_code {
       builder = builder.bank_code(bank);
   }
   ```
4. No library changes needed. `RaastQrBuilder::bank_code` already validates length (1..=20 bytes, `FieldLengthExceeded` otherwise).

## 2. `--scheme-guid` on `verify` / `decode` (historical plan; completed)

**Why (at the time)**: `parse_with_guid` could select a non-default MAI template from a multi-scheme payload (see `test_multi_scheme_guid_disambiguation`), but the CLI could not expose that choice and defaulted to `"pk.raast"`.

**Original implementation steps (completed)**:

1. Add the same optional flag to both `Verify` and `Decode`:
   ```rust
   /// Scheme GUID to select when a payload carries more than one MAI template
   #[arg(long, default_value = "pk.raast")]
   scheme_guid: String,
   ```
2. Replace `RaastQr::parse(&payload)` with `RaastQr::parse_with_guid(&payload, &scheme_guid)` in both match arms.
3. Default stays `"pk.raast"`, so existing default-scheme payloads behave as before. Purely additive.

## 3. Version + publish (historical; do not rerun)

Both crates are now published at `0.3.0`; `raast-qr-cli` depends on `raast-qr = "0.3.0"`. Any subsequent release needs a new version. Original 0.2.0-era steps, retained for context:

1. No manual version bump needed — both crates use `version.workspace = true`, and the workspace was pinned at `0.2.0`. Publishing `raast-qr-cli` after steps 1–2 would pick that up automatically.
2. At the time, `raast-qr-cli`'s `Cargo.toml` pinned `raast-qr = "0.2.0"` (already published) — the original plan required only a CLI release.
3. Original proposed command: `cargo publish -p raast-qr-cli` from the workspace root once 1–2 were implemented and tested. Do not run it for 0.3.0.

## 4. Test coverage (historical plan; completed)

At the time, `crates/raast-qr-cli` had no `tests/` directory. The proposed tests below now live in `crates/raast-qr-cli/tests/cli.rs`, alongside fee-mode CLI tests:

- **Bank code round-trip**: `generate --bank-code 1234 ...` → parse the output → assert `bank_code == Some("1234")`.
- **Scheme GUID selection**: a payload with two MAI templates → `decode --scheme-guid other` → assert the _other_ template's fields come back, not the default `"pk.raast"` one.

## Not doing (for now)

- `--currency` / `--country-code` flags — both are hard-validated to exactly one legal value (`PKR` / `"PK"`) by the builder today. A flag that only ever accepts one input isn't worth the surface area yet.
- A separate `--iban` flag distinct from `--alias` — `raast_alias` and `raast_iban` both just set the same underlying `raast_id` field. `--alias` already accepts either string as-is; the distinction is cosmetic in the builder API, not functional.
