#[cfg(feature = "std")]
use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "std", derive(Error))]
pub enum RaastError {
    #[cfg_attr(feature = "std", error("Invalid CRC16 checksum: expected {expected:#06X}, found {found:#06X}"))]
    InvalidChecksum { expected: u16, found: u16 },

    #[cfg_attr(feature = "std", error("Payload exceeds maximum EMVCo length (512 bytes)"))]
    PayloadTooLong,

    #[cfg_attr(feature = "std", error("Malformed Tag-Length-Value format: {0}"))]
    MalformedTlv(&'static str),

    #[cfg_attr(feature = "std", error("Missing mandatory tag: {0}"))]
    MissingMandatoryTag(&'static str),

    #[cfg_attr(feature = "std", error("Invalid amount decimal formatting: {0}"))]
    InvalidAmount(&'static str),

    #[cfg_attr(feature = "std", error("Unsupported currency: {0} (only 586 / PKR is supported for Raast)"))]
    UnsupportedCurrency(u16),

    #[cfg_attr(feature = "std", error("Tag 00 (Payload Format Indicator) must be the first data object"))]
    Tag00NotFirst,

    #[cfg_attr(feature = "std", error("Tag 63 (CRC) must be the final data object"))]
    Tag63NotLast,
}
