use std::collections::{HashMap, HashSet};

use crate::pairing::algorithms::blossom_v1::{
    BlossomV1Policy, PairingEntrant, PairingRequest, PreviousMatch, RoundNumber,
};

use super::{CompletedRound, ContestantStanding, TournamentApplicationError, TournamentEntrant};

pub(super) fn build_pairing_request(
    entrants: &[TournamentEntrant],
    standings: &[ContestantStanding],
    completed_rounds: &[CompletedRound],
    round_number: RoundNumber,
    policy: BlossomV1Policy,
) -> Result<PairingRequest, TournamentApplicationError> {
    let standing_by_id = standings
        .iter()
        .map(|standing| (&standing.entrant_id, standing))
        .collect::<HashMap<_, _>>();
    let active_entrant_ids = entrants
        .iter()
        .map(|entrant| &entrant.entrant_id)
        .collect::<HashSet<_>>();
    let entrants = entrants
        .iter()
        .map(|entrant| {
            let standing = standing_by_id.get(&entrant.entrant_id).ok_or_else(|| {
                TournamentApplicationError::UnknownEntrantInRound {
                    entrant_id: entrant.entrant_id.clone(),
                }
            })?;
            Ok(PairingEntrant {
                entrant_id: entrant.entrant_id.clone(),
                club_id: entrant.club_id.clone(),
                starting_elo: entrant.starting_elo,
                performance_score: standing.performance_score,
                matches_won: u16::try_from(standing.matches_won)
                    .map_err(|_| overflow("pairing match wins"))?,
                opponent_score_sum: standing.opponent_score_sum,
                bye_count: u16::try_from(standing.bye_count)
                    .map_err(|_| overflow("pairing bye count"))?,
            })
        })
        .collect::<Result<Vec<_>, TournamentApplicationError>>()?;
    let previous_matches = completed_rounds
        .iter()
        .flat_map(|round| {
            round
                .scheduled_matches
                .iter()
                .filter(|scheduled| {
                    active_entrant_ids.contains(&scheduled.home_entrant_id)
                        && active_entrant_ids.contains(&scheduled.away_entrant_id)
                })
                .map(|scheduled| PreviousMatch {
                    first_entrant_id: scheduled.home_entrant_id.clone(),
                    second_entrant_id: scheduled.away_entrant_id.clone(),
                    round_number: round.round_number,
                })
        })
        .collect();

    Ok(PairingRequest {
        round_number,
        entrants,
        previous_matches,
        policy,
    })
}

const fn overflow(component: &'static str) -> TournamentApplicationError {
    TournamentApplicationError::StandingOverflow { component }
}
