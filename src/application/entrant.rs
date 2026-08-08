use crate::identity::{ClubId, EntrantId};
use crate::pairing::EloRating;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct TournamentEntrant {
    pub entrant_id: EntrantId,
    pub name: String,
    pub club_id: ClubId,
    pub club_name: String,
    pub starting_elo: EloRating,
}
