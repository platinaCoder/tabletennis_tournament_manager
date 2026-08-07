use std::error::Error;
use std::fmt::{self, Display, Formatter};

use crate::identity::EntrantId;

use super::{RelaxationTier, RoundNumber};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PairingCostComponent {
    PerformanceScoreGap,
    MatchWinGap,
    OpponentStrengthGap,
    EloGap,
    SameClubPenalty,
    RematchPenalty,
    ByePenalty,
    DeterministicTieBreak,
    SolverWeightProjection,
    Total,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum InvalidSolverOutputReason {
    DuplicateEntrant,
    MissingEntrant,
    SelfPair,
    UnknownEdge,
    MultipleByes,
    UnexpectedBye,
    AvoidableRepeatedBye,
    InconsistentEdgeCost,
    InvalidDualState,
    NegativeReducedCost {
        first_node: usize,
        second_node: usize,
        dual_sum: u128,
        doubled_weight: u128,
    },
    NonIntegralDualAdjustment,
    InvalidBlossomStructure,
    ForbiddenSameClubPairing,
    ForbiddenRematch,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SolverError {
    message: String,
}

impl SolverError {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

impl Display for SolverError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.message)
    }
}

impl Error for SolverError {}

#[derive(Debug)]
pub enum BlossomPairingError {
    NotEnoughEntrants {
        entrant_count: usize,
    },
    EntrantLimitExceeded {
        entrant_count: usize,
        maximum: usize,
    },
    DuplicateEntrant {
        entrant_id: EntrantId,
    },
    UnknownEntrantInHistory {
        unknown_entrant_id: EntrantId,
    },
    SelfMatchInHistory {
        entrant_id: EntrantId,
    },
    InvalidHistoryRound {
        history_round: RoundNumber,
        requested_round: RoundNumber,
    },
    PairingCostOverflow {
        component: PairingCostComponent,
    },
    NoCompleteMatching {
        final_tier: RelaxationTier,
        unmatched_entrants: Vec<EntrantId>,
    },
    SolverFailure {
        source: SolverError,
    },
    InvalidSolverOutput {
        reason: InvalidSolverOutputReason,
    },
}

impl Display for BlossomPairingError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotEnoughEntrants { entrant_count } => write!(
                formatter,
                "at least two active entrants are required, received {entrant_count}"
            ),
            Self::EntrantLimitExceeded {
                entrant_count,
                maximum,
            } => write!(
                formatter,
                "entrant count {entrant_count} exceeds the supported maximum of {maximum}"
            ),
            Self::DuplicateEntrant { entrant_id } => {
                write!(formatter, "entrant {entrant_id:?} occurs more than once")
            }
            Self::UnknownEntrantInHistory { unknown_entrant_id } => write!(
                formatter,
                "match history references unknown entrant {unknown_entrant_id:?}"
            ),
            Self::SelfMatchInHistory { entrant_id } => {
                write!(
                    formatter,
                    "entrant {entrant_id:?} is recorded as playing itself"
                )
            }
            Self::InvalidHistoryRound {
                history_round,
                requested_round,
            } => write!(
                formatter,
                "history round {} is later than requested round {}",
                history_round.value(),
                requested_round.value()
            ),
            Self::PairingCostOverflow { component } => write!(
                formatter,
                "pairing cost overflowed while calculating {component:?}"
            ),
            Self::NoCompleteMatching {
                final_tier,
                unmatched_entrants,
            } => write!(
                formatter,
                "no complete matching exists after relaxation tier {final_tier:?}; {} entrants unmatched",
                unmatched_entrants.len()
            ),
            Self::SolverFailure { .. } => formatter.write_str("the matching solver failed"),
            Self::InvalidSolverOutput { reason } => write!(
                formatter,
                "the matching solver returned an invalid result: {reason:?}"
            ),
        }
    }
}

impl Error for BlossomPairingError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::SolverFailure { source } => Some(source),
            _ => None,
        }
    }
}
