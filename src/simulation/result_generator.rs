use crate::pairing::EloRating;
use crate::results::{GameScore, MatchFormat};

use super::SimulationError;

pub fn simulate_match_games(
    match_format: MatchFormat,
    home_elo: EloRating,
    away_elo: EloRating,
    seed: u64,
) -> Result<Vec<GameScore>, SimulationError> {
    simulate_games(
        match_format,
        home_elo,
        away_elo,
        &mut DeterministicRandom::new(seed),
    )
}

pub(super) fn simulate_games(
    match_format: MatchFormat,
    home_elo: EloRating,
    away_elo: EloRating,
    random: &mut DeterministicRandom,
) -> Result<Vec<GameScore>, SimulationError> {
    let home_win_threshold = elo_expectation(home_elo, away_elo);
    let required_wins = match_format.games_required_to_win();
    let mut home_wins = 0_u8;
    let mut away_wins = 0_u8;
    let mut games = Vec::with_capacity(match_format.maximum_games());

    while home_wins < required_wins && away_wins < required_wins {
        let home_wins_game = random.next_bounded(1_000_000) < home_win_threshold;
        if home_wins_game {
            home_wins += 1;
        } else {
            away_wins += 1;
        }
        let (winner_points, loser_points) = simulated_valid_score(random);
        let (home_points, away_points) = if home_wins_game {
            (winner_points, loser_points)
        } else {
            (loser_points, winner_points)
        };
        let game_number = u8::try_from(games.len() + 1)
            .map_err(|_| SimulationError::GeneratedInvalidGameNumber)?;
        games.push(
            GameScore::new(game_number, home_points, away_points)
                .map_err(|_| SimulationError::GeneratedInvalidGameNumber)?,
        );
    }
    Ok(games)
}

fn elo_expectation(home: EloRating, away: EloRating) -> u64 {
    let difference = f64::from(away.value()) - f64::from(home.value());
    let expected = 1.0 / (1.0 + 10_f64.powf(difference / 400.0));
    (expected * 1_000_000.0).round() as u64
}

fn simulated_valid_score(random: &mut DeterministicRandom) -> (u16, u16) {
    if random.next_bounded(8) == 0 {
        let extra = u16::try_from(random.next_bounded(8)).unwrap_or(0);
        (12 + extra, 10 + extra)
    } else {
        (11, u16::try_from(random.next_bounded(10)).unwrap_or(0))
    }
}

pub(super) struct DeterministicRandom(u64);

impl DeterministicRandom {
    pub(super) const fn new(seed: u64) -> Self {
        Self(seed)
    }

    fn next_bounded(&mut self, upper_bound: u64) -> u64 {
        self.0 = self
            .0
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1_442_695_040_888_963_407);
        self.0 % upper_bound
    }
}
