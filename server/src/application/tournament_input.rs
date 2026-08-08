use std::collections::{HashMap, HashSet};

use uuid::Uuid;

use crate::api_contract::{EntrantInput, GameScoreInput};
use crate::application::{TournamentApplicationError, TournamentEntrant};
use crate::backend::persistence::StoredTournament;
use crate::backend::server::error::ApiError;
use crate::identity::{ClubId, EntrantId};
use crate::pairing::EloRating;
use crate::results::{GameNumber, GamePoints, GameScore};

pub(super) fn roster(
    stored: &StoredTournament,
    inputs: Vec<EntrantInput>,
) -> Result<Vec<TournamentEntrant>, ApiError> {
    if inputs.len() > 64 {
        return Err(ApiError::invalid(
            "entrant_limit_exceeded",
            "At most 64 active entrants are supported.",
        ));
    }
    let existing = stored
        .application
        .entrants()
        .iter()
        .map(|entrant| (entrant.entrant_id.as_str(), entrant))
        .collect::<HashMap<_, _>>();
    let mut used_ids = HashSet::new();
    let mut clubs = stored
        .application
        .entrants()
        .iter()
        .map(|entrant| (entrant.club_name.to_lowercase(), entrant.club_id.clone()))
        .collect::<HashMap<_, _>>();
    inputs
        .into_iter()
        .map(|input| {
            validate_roster_text(&input.display_name, "display_name")?;
            validate_roster_text(&input.club_name, "club_name")?;
            let entrant_id = match input.entrant_id {
                Some(value) => {
                    if !existing.contains_key(value.as_str()) {
                        return Err(ApiError::invalid(
                            "unknown_entrant",
                            "An entrant ID does not belong to this tournament.",
                        ));
                    }
                    EntrantId::new(value)
                }
                None => EntrantId::new(format!("entrant-{}", Uuid::new_v4())),
            };
            if !used_ids.insert(entrant_id.clone()) {
                return Err(ApiError::invalid(
                    "duplicate_entrant",
                    "An entrant occurs more than once in the roster.",
                ));
            }
            let club_key = input.club_name.trim().to_lowercase();
            let club_id = clubs
                .entry(club_key)
                .or_insert_with(|| ClubId::new(format!("club-{}", Uuid::new_v4())))
                .clone();
            let starting_elo = u32::try_from(input.starting_elo).map_err(|_| {
                ApiError::invalid("invalid_elo", "Starting ELO must be a positive integer.")
            })?;
            Ok(TournamentEntrant {
                entrant_id,
                name: input.display_name.trim().to_owned(),
                club_id,
                club_name: input.club_name.trim().to_owned(),
                starting_elo: EloRating::new(starting_elo),
            })
        })
        .collect()
}

pub(super) fn game_score(input: GameScoreInput) -> Result<GameScore, ApiError> {
    Ok(GameScore {
        game_number: GameNumber::try_from(input.game_number)
            .map_err(|error| ApiError::invalid("invalid_game_number", error.to_string()))?,
        home_points: GamePoints::try_from(input.home_points)
            .map_err(|error| ApiError::invalid("invalid_game_score", error.to_string()))?,
        away_points: GamePoints::try_from(input.away_points)
            .map_err(|error| ApiError::invalid("invalid_game_score", error.to_string()))?,
    })
}

pub(super) fn domain_error(error: TournamentApplicationError) -> ApiError {
    let code = match &error {
        TournamentApplicationError::MatchResult(error) => error.code(),
        TournamentApplicationError::Tournament(error) => error.code(),
        _ => "invalid_tournament_state",
    };
    ApiError::invalid(code, error.to_string())
}

fn validate_roster_text(value: &str, field: &'static str) -> Result<(), ApiError> {
    if value.trim().is_empty() || value.len() > 200 {
        Err(ApiError::invalid(
            "invalid_roster_field",
            format!("{field} must contain between 1 and 200 bytes."),
        ))
    } else {
        Ok(())
    }
}
