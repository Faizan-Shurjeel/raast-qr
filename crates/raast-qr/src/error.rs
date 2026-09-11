#[cfg(feature = "std")]
use thiserror::Error;

#[derive(Debug, PartialEq, Eq)]
#[cfg_attr(feature = "std", derive(Error))]
pub enum RaastError {
    #[cfg_attr(feature = "std", error("Invalid CRC16 checksum: expected {expected:#06X}, found {found:#06X}"))]
    InvalidChecksum { expected: u16, found: u16 },

    #[cfg_attr(feature = "std", error("Payload exceeds maximum EMVCo length (512 bytes)"))]
    PayloadTooLong,

    #[cfg_attr(feature = "std", error("Malformed Tag-Length-Value format"))]
    MalformedTlv,

    #[cfg_attr(feature = "std", error("Missing mandatory tag: {0}"))]
    MissingMandatoryTag(&'static str),

    #[cfg_attr(feature = "std", error("Invalid amount decimal formatting"))]
    InvalidAmount,
}
