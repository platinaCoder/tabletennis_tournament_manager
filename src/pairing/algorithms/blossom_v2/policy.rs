use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct BlossomV2Policy {
    pub avoid_same_club: bool,
    pub avoid_rematches: bool,
    pub recent_rematch_window: u16,

    /// Applied to the squared match-win difference. This establishes the
    /// record bracket as the primary competitive signal.
    pub match_record_weight: u64,
    /// Applied after performance-score gaps are normalized to thousandths.
    pub performance_score_weight: u32,
    /// Applied after opponent-strength gaps are normalized to thousandths.
    pub opponent_strength_weight: u32,
    /// Applied to the squared starting-ELO difference.
    pub squared_elo_difference_weight: u32,

    pub bye_repeat_penalty: u64,
    pub same_club_penalty: u64,
    pub rematch_penalty: u64,
    pub maximum_entrant_count: usize,
}

impl Default for BlossomV2Policy {
    fn default() -> Self {
        Self {
            avoid_same_club: true,
            avoid_rematches: true,
            recent_rematch_window: 3,
            match_record_weight: 1_000_000_000,
            performance_score_weight: 100,
            opponent_strength_weight: 25,
            squared_elo_difference_weight: 10,
            bye_repeat_penalty: 100_000_000_000,
            same_club_penalty: 10_000_000_000,
            rematch_penalty: 100_000_000_000,
            maximum_entrant_count: 64,
        }
    }
}
