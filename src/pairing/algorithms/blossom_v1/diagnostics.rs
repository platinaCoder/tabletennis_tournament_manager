use std::time::Duration;

use crate::identity::EntrantId;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RelaxationTier {
    Strict,
    SameClubAllowed,
    RematchesAllowed,
}

impl RelaxationTier {
    pub const ORDERED: [Self; 3] = [Self::Strict, Self::SameClubAllowed, Self::RematchesAllowed];

    pub(super) const fn allows_same_club(self) -> bool {
        !matches!(self, Self::Strict)
    }

    pub(super) const fn allows_rematches(self) -> bool {
        matches!(self, Self::RematchesAllowed)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PairingWarning {
    SameClubPairingRequired {
        first_entrant_id: EntrantId,
        second_entrant_id: EntrantId,
    },
    RematchRequired {
        first_entrant_id: EntrantId,
        second_entrant_id: EntrantId,
    },
    ByeAssigned {
        entrant_id: EntrantId,
    },
    RelaxedPairingRequired {
        tier: RelaxationTier,
    },
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct PairingDiagnostics {
    pub candidate_pair_count: usize,
    pub eligible_edge_count: usize,
    pub rejected_same_club_edges: usize,
    pub rejected_rematch_edges: usize,
    pub edge_generation_duration: Duration,
    pub cost_calculation_duration: Duration,
    pub solver_duration: Duration,
    pub validation_duration: Duration,
}
