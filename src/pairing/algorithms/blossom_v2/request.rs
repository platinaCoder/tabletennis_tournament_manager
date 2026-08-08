use super::super::blossom_v1;
use super::{BlossomV2Policy, PairingEntrant, PreviousMatch, RoundNumber};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PairingRequest {
    pub round_number: RoundNumber,
    pub entrants: Vec<PairingEntrant>,
    pub previous_matches: Vec<PreviousMatch>,
    pub policy: BlossomV2Policy,
}

pub(super) fn compatibility_request(request: &PairingRequest) -> blossom_v1::PairingRequest {
    blossom_v1::PairingRequest {
        round_number: request.round_number,
        entrants: request.entrants.clone(),
        previous_matches: request.previous_matches.clone(),
        policy: blossom_v1::BlossomV1Policy {
            avoid_same_club: request.policy.avoid_same_club,
            avoid_rematches: request.policy.avoid_rematches,
            recent_rematch_window: request.policy.recent_rematch_window,
            performance_score_weight: 0,
            match_win_weight: 0,
            opponent_strength_weight: 0,
            elo_difference_weight: 0,
            bye_repeat_penalty: request.policy.bye_repeat_penalty,
            same_club_penalty: request.policy.same_club_penalty,
            rematch_penalty: request.policy.rematch_penalty,
            maximum_entrant_count: request.policy.maximum_entrant_count,
        },
    }
}
