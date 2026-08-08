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
    let home_wins_match = random.next_bounded(1_000_000) < home_win_threshold;
    let game_winners = simulated_game_winners(home_wins_match, required_wins, random);
    let mut games = Vec::with_capacity(game_winners.len());

    for home_wins_game in game_winners {
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

fn simulated_game_winners(
    home_wins_match: bool,
    required_wins: u8,
    random: &mut DeterministicRandom,
) -> Vec<bool> {
    let losing_game_count = usize::try_from(random.next_bounded(u64::from(required_wins)))
        .expect("a generated losing-game count always fits usize");
    let winning_game_count_before_final = usize::from(required_wins.saturating_sub(1));
    let mut winners = Vec::with_capacity(
        winning_game_count_before_final
            .saturating_add(losing_game_count)
            .saturating_add(1),
    );
    winners.extend(std::iter::repeat_n(
        home_wins_match,
        winning_game_count_before_final,
    ));
    winners.extend(std::iter::repeat_n(!home_wins_match, losing_game_count));
    shuffle(&mut winners, random);
    winners.push(home_wins_match);
    winners
}

fn shuffle(values: &mut [bool], random: &mut DeterministicRandom) {
    for index in (1..values.len()).rev() {
        let upper_bound = u64::try_from(index + 1).unwrap_or(u64::MAX);
        let swap_index = usize::try_from(random.next_bounded(upper_bound))
            .expect("a generated shuffle index always fits usize");
        values.swap(index, swap_index);
    }
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
