#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BlossomV1Policy {
    pub avoid_same_club: bool,
    pub avoid_rematches: bool,
    pub recent_rematch_window: u16,
    pub performance_score_weight: u32,
    pub match_win_weight: u32,
    pub opponent_strength_weight: u32,
    pub elo_difference_weight: u32,
    pub bye_repeat_penalty: u64,
    pub same_club_penalty: u64,
    pub rematch_penalty: u64,
    pub maximum_entrant_count: usize,
}

impl Default for BlossomV1Policy {
    fn default() -> Self {
        Self {
            avoid_same_club: true,
            avoid_rematches: true,
            recent_rematch_window: 3,
            performance_score_weight: 4,
            match_win_weight: 1_000_000,
            opponent_strength_weight: 1,
            elo_difference_weight: 2_000,
            bye_repeat_penalty: 100_000_000_000,
            same_club_penalty: 10_000_000_000,
            rematch_penalty: 100_000_000_000,
            maximum_entrant_count: 64,
        }
    }
}
