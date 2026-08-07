use crate::pairing::algorithms::blossom_v1::BlossomV1Policy;
use crate::results::MatchFormat;
use crate::tournament::{TableCount, TournamentId};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SimulationEntrantPattern {
    Varied,
    EloRange900To1500,
    IdenticalElo,
    DominantClub,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SimulationConfig {
    pub name: String,
    pub tournament_id: TournamentId,
    pub entrant_count: usize,
    pub club_count: usize,
    pub table_count: TableCount,
    pub round_count: u16,
    pub match_format: MatchFormat,
    pub entrant_pattern: SimulationEntrantPattern,
    pub pairing_policy: BlossomV1Policy,
    pub random_seed: u64,
}

impl SimulationConfig {
    pub fn baseline() -> Self {
        Self {
            name: "baseline".to_owned(),
            tournament_id: TournamentId::new("simulation-baseline"),
            entrant_count: 16,
            club_count: 8,
            table_count: TableCount::try_from(8_i64).expect("eight is a valid table count"),
            round_count: 5,
            match_format: MatchFormat::BestOfFive,
            entrant_pattern: SimulationEntrantPattern::Varied,
            pairing_policy: BlossomV1Policy::default(),
            random_seed: 0x5eed,
        }
    }
}

pub fn standard_scenarios() -> Vec<SimulationConfig> {
    let baseline = SimulationConfig::baseline();
    vec![
        baseline.clone(),
        SimulationConfig {
            name: "elo-range-900-1500".to_owned(),
            tournament_id: TournamentId::new("simulation-elo-range"),
            entrant_pattern: SimulationEntrantPattern::EloRange900To1500,
            random_seed: 0xe10_900,
            ..baseline.clone()
        },
        SimulationConfig {
            name: "odd-entrant-count".to_owned(),
            tournament_id: TournamentId::new("simulation-odd"),
            entrant_count: 15,
            table_count: TableCount::try_from(5_i64).expect("five is a valid table count"),
            match_format: MatchFormat::BestOfThree,
            random_seed: 0x0dd,
            ..baseline.clone()
        },
        SimulationConfig {
            name: "dominant-club".to_owned(),
            tournament_id: TournamentId::new("simulation-club"),
            entrant_pattern: SimulationEntrantPattern::DominantClub,
            club_count: 3,
            random_seed: 0xc1ab,
            ..baseline.clone()
        },
        SimulationConfig {
            name: "identical-elo".to_owned(),
            tournament_id: TournamentId::new("simulation-identical"),
            entrant_pattern: SimulationEntrantPattern::IdenticalElo,
            random_seed: 0xe10,
            ..baseline.clone()
        },
        SimulationConfig {
            name: "unavoidable-rematches".to_owned(),
            tournament_id: TournamentId::new("simulation-rematches"),
            entrant_count: 4,
            club_count: 4,
            round_count: 4,
            match_format: MatchFormat::BestOfThree,
            pairing_policy: BlossomV1Policy {
                recent_rematch_window: 10,
                ..BlossomV1Policy::default()
            },
            random_seed: 0x1e,
            ..baseline.clone()
        },
        SimulationConfig {
            name: "fewer-tables".to_owned(),
            tournament_id: TournamentId::new("simulation-tables"),
            table_count: TableCount::try_from(3_i64).expect("three is a valid table count"),
            ..baseline
        },
    ]
}
