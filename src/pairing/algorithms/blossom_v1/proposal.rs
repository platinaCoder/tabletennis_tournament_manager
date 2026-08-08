use crate::identity::EntrantId;

use super::{PairingDiagnostics, PairingWarning, RelaxationTier};

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct PairingCost(u64);

impl PairingCost {
    pub const fn new(value: u64) -> Self {
        Self(value)
    }

    pub const fn value(self) -> u64 {
        self.0
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PairingCostBreakdown {
    /// Weighted component values before deterministic scaling.
    pub performance_score_gap: u64,
    pub match_win_gap: u64,
    pub opponent_strength_gap: u64,
    pub elo_gap: u64,
    pub same_club_penalty: u64,
    pub rematch_penalty: u64,
    pub bye_penalty: u64,
    pub deterministic_tie_break: u64,
    /// The weighted component sum, scaled so tie-breaking cannot change its
    /// ordering, plus `deterministic_tie_break`.
    pub total: u64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PairingPolicyVersion {
    BlossomV1,
    BlossomV2,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PairingProposal {
    /// Order is deterministic for auditing but has no sporting meaning.
    pub matches: Vec<ProposedMatch>,
    pub bye: Option<ProposedBye>,
    pub relaxation_tier: RelaxationTier,
    pub total_cost: PairingCost,
    pub policy_version: PairingPolicyVersion,
    pub warnings: Vec<PairingWarning>,
    pub diagnostics: PairingDiagnostics,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProposedMatch {
    pub first_entrant_id: EntrantId,
    pub second_entrant_id: EntrantId,
    pub cost: PairingCostBreakdown,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProposedBye {
    pub entrant_id: EntrantId,
    pub cost: PairingCostBreakdown,
}
