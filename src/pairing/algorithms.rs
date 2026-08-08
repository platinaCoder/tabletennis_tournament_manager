//! Pairing algorithm implementations live in focused child modules.
//!
//! Each implementation owns an immutable snapshot input, validation,
//! algorithm-specific errors, and diagnostic proposal output. Match publication
//! and table assignment remain downstream so adding an algorithm does not
//! duplicate them or give solver output order sporting meaning.

pub mod blossom_v1;
pub mod blossom_v2;

use blossom_v1::PairingProposal;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PairingPolicy {
    BlossomV1(blossom_v1::BlossomV1Policy),
    BlossomV2(blossom_v2::BlossomV2Policy),
}

impl Default for PairingPolicy {
    fn default() -> Self {
        Self::BlossomV2(blossom_v2::BlossomV2Policy::default())
    }
}

impl From<blossom_v1::BlossomV1Policy> for PairingPolicy {
    fn from(policy: blossom_v1::BlossomV1Policy) -> Self {
        Self::BlossomV1(policy)
    }
}

impl From<blossom_v2::BlossomV2Policy> for PairingPolicy {
    fn from(policy: blossom_v2::BlossomV2Policy) -> Self {
        Self::BlossomV2(policy)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PairingSnapshot {
    BlossomV1(blossom_v1::PairingRequest),
    BlossomV2(blossom_v2::PairingRequest),
}

pub fn propose_pairings(
    snapshot: &PairingSnapshot,
) -> Result<PairingProposal, blossom_v1::BlossomPairingError> {
    match snapshot {
        PairingSnapshot::BlossomV1(request) => blossom_v1::propose_pairings(request),
        PairingSnapshot::BlossomV2(request) => blossom_v2::propose_pairings(request),
    }
}
