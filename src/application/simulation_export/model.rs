use serde::Serialize;

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct SimulationTrace {
    pub schema_version: u16,
    pub simulation: SimulationMetadataTrace,
    pub tournament: TournamentTrace,
    pub entrants: Vec<EntrantTrace>,
    pub completed_rounds: Vec<RoundTrace>,
    pub active_round: Option<RoundTrace>,
    pub pending_pairing: Option<PairingCalculationTrace>,
    pub current_standings: Vec<StandingTrace>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct SimulationMetadataTrace {
    pub result_generator: Option<String>,
    pub run_seed: Option<u64>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct TournamentTrace {
    pub tournament_id: String,
    pub state: String,
    pub match_format: String,
    pub table_count: u16,
    pub maximum_round_count: u16,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct EntrantTrace {
    pub entrant_id: String,
    pub name: String,
    pub club_id: String,
    pub club_name: String,
    pub starting_elo: u32,
    pub active: bool,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct RoundTrace {
    pub round_number: u16,
    pub pairing: PairingCalculationTrace,
    pub scheduled_matches: Vec<ScheduledMatchTrace>,
    pub results: Vec<MatchResultTrace>,
    pub bye_entrant_id: Option<String>,
    pub standings_before_round: Vec<StandingTrace>,
    pub standings_after_round: Option<Vec<StandingTrace>>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct PairingCalculationTrace {
    pub request: PairingRequestTrace,
    pub relaxation_graphs: Vec<CandidateGraphTrace>,
    pub proposal: PairingProposalTrace,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct PairingRequestTrace {
    pub round_number: u16,
    pub entrants: Vec<PairingEntrantTrace>,
    pub previous_matches: Vec<PreviousMatchTrace>,
    pub policy: PairingPolicyTrace,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct PairingEntrantTrace {
    pub entrant_id: String,
    pub club_id: String,
    pub starting_elo: u32,
    pub performance_score_scaled: i64,
    pub matches_won: u16,
    pub opponent_score_sum_scaled: i64,
    pub bye_count: u16,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct PreviousMatchTrace {
    pub first_entrant_id: String,
    pub second_entrant_id: String,
    pub round_number: u16,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct PairingPolicyTrace {
    pub version: String,
    pub avoid_same_club: bool,
    pub avoid_rematches: bool,
    pub recent_rematch_window: u16,
    pub performance_score_weight: u32,
    pub performance_score_normalization: u64,
    pub match_win_weight: Option<u32>,
    pub match_record_weight: Option<u64>,
    pub opponent_strength_weight: u32,
    pub opponent_strength_normalization: u64,
    pub elo_difference_weight: Option<u32>,
    pub squared_elo_difference_weight: Option<u32>,
    pub bye_repeat_penalty: u64,
    pub same_club_penalty: u64,
    pub rematch_penalty: u64,
    pub maximum_entrant_count: usize,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct CandidateGraphTrace {
    pub relaxation_tier: String,
    pub entrant_ids: Vec<String>,
    pub edges: Vec<CandidateEdgeTrace>,
    pub diagnostics: PairingDiagnosticsTrace,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct CandidateEdgeTrace {
    pub first_entrant_id: String,
    pub target: CandidateEdgeTargetTrace,
    pub same_club: bool,
    pub rematch: bool,
    pub selected: bool,
    pub cost: u64,
    pub breakdown: PairingCostBreakdownTrace,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum CandidateEdgeTargetTrace {
    Entrant { entrant_id: String },
    Bye,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct PairingProposalTrace {
    pub policy_version: String,
    pub relaxation_tier: String,
    pub total_cost: u64,
    pub matches: Vec<ProposedMatchTrace>,
    pub bye: Option<ProposedByeTrace>,
    pub warnings: Vec<PairingWarningTrace>,
    pub diagnostics: PairingDiagnosticsTrace,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct ProposedMatchTrace {
    pub first_entrant_id: String,
    pub second_entrant_id: String,
    pub cost: PairingCostBreakdownTrace,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct ProposedByeTrace {
    pub entrant_id: String,
    pub cost: PairingCostBreakdownTrace,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum PairingWarningTrace {
    SameClubPairingRequired {
        first_entrant_id: String,
        second_entrant_id: String,
    },
    RematchRequired {
        first_entrant_id: String,
        second_entrant_id: String,
    },
    ByeAssigned {
        entrant_id: String,
    },
    RelaxedPairingRequired {
        tier: String,
    },
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct PairingCostBreakdownTrace {
    pub performance_score_gap: u64,
    pub match_win_gap: u64,
    pub opponent_strength_gap: u64,
    pub elo_gap: u64,
    pub same_club_penalty: u64,
    pub rematch_penalty: u64,
    pub bye_penalty: u64,
    pub deterministic_tie_break: u64,
    pub total: u64,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct PairingDiagnosticsTrace {
    pub candidate_pair_count: usize,
    pub eligible_edge_count: usize,
    pub rejected_same_club_edges: usize,
    pub rejected_rematch_edges: usize,
    pub edge_generation_microseconds: u128,
    pub cost_calculation_microseconds: u128,
    pub solver_microseconds: u128,
    pub validation_microseconds: u128,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct ScheduledMatchTrace {
    pub match_id: String,
    pub home_entrant_id: String,
    pub away_entrant_id: String,
    pub table_number: Option<u16>,
    pub publication_status: String,
    pub round_activity: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct MatchResultTrace {
    pub match_id: String,
    pub games: Vec<GameScoreTrace>,
    pub home_games_won: u8,
    pub away_games_won: u8,
    pub winner_entrant_id: String,
    pub entered_at_unix_milliseconds: u128,
    pub corrected_at_unix_milliseconds: Option<u128>,
    pub revision: u32,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct GameScoreTrace {
    pub game_number: u8,
    pub home_points: u16,
    pub away_points: u16,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct StandingTrace {
    pub rank: usize,
    pub entrant_id: String,
    pub performance_score_scaled: i64,
    pub matches_played: u32,
    pub matches_won: u32,
    pub matches_lost: u32,
    pub games_won: u32,
    pub games_lost: u32,
    pub game_difference: i32,
    pub points_won: u32,
    pub points_lost: u32,
    pub point_difference: i32,
    pub opponent_score_sum_scaled: i64,
    pub bye_count: u32,
}
