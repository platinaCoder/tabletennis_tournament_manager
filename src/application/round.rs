use crate::identity::EntrantId;
use crate::pairing::algorithms::PairingSnapshot;
use crate::pairing::algorithms::blossom_v1::{PairingProposal, RoundNumber};
use crate::results::MatchResult;
use crate::scheduling::ScheduledMatch;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ActiveRound {
    pub round_number: RoundNumber,
    pub pairing_request: PairingSnapshot,
    pub proposal: PairingProposal,
    pub scheduled_matches: Vec<ScheduledMatch>,
    pub results: Vec<MatchResult>,
    pub bye: Option<EntrantId>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CompletedRound {
    pub round_number: RoundNumber,
    pub pairing_request: PairingSnapshot,
    pub proposal: PairingProposal,
    pub scheduled_matches: Vec<ScheduledMatch>,
    pub results: Vec<MatchResult>,
    pub bye: Option<EntrantId>,
}
