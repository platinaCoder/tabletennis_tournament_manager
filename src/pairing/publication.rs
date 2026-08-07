use crate::identity::{EntrantId, MatchId};
use crate::scheduling::{RoundActivity, ScheduledMatch};

/// Application-supplied identity for publishing one contestant pairing.
/// Pairing algorithms do not create match identifiers.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MatchPublication {
    pub match_id: MatchId,
    pub first_entrant_id: EntrantId,
    pub second_entrant_id: EntrantId,
}

pub fn publish_scheduled_matches(
    matches: Vec<MatchPublication>,
    round_activity: RoundActivity,
) -> Vec<ScheduledMatch> {
    matches
        .into_iter()
        .map(|published_match| {
            ScheduledMatch::published(
                published_match.match_id,
                published_match.first_entrant_id,
                published_match.second_entrant_id,
                None,
                round_activity,
            )
        })
        .collect()
}
