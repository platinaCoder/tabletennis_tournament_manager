use crate::identity::EntrantId;
use crate::pairing::algorithms::blossom_v1::PerformanceScore;

use super::{ContestantStanding, TournamentApplicationError};

pub(super) struct StandingAccumulator {
    entrant_id: EntrantId,
    performance_score: PerformanceScore,
    matches_played: u32,
    matches_won: u32,
    matches_lost: u32,
    games_won: u32,
    games_lost: u32,
    points_won: u32,
    points_lost: u32,
    opponent_score_sum: PerformanceScore,
    bye_count: u32,
}

impl StandingAccumulator {
    pub fn new(entrant_id: EntrantId) -> Self {
        Self {
            entrant_id,
            performance_score: PerformanceScore::ZERO,
            matches_played: 0,
            matches_won: 0,
            matches_lost: 0,
            games_won: 0,
            games_lost: 0,
            points_won: 0,
            points_lost: 0,
            opponent_score_sum: PerformanceScore::ZERO,
            bye_count: 0,
        }
    }

    pub const fn performance_score(&self) -> PerformanceScore {
        self.performance_score
    }

    pub fn record_bye(&mut self) -> Result<(), TournamentApplicationError> {
        self.bye_count = checked_add(self.bye_count, 1, "bye count")?;
        Ok(())
    }

    pub fn record_result(
        &mut self,
        games_won: u8,
        games_lost: u8,
        points_won: impl Iterator<Item = u16>,
        points_lost: impl Iterator<Item = u16>,
        won: bool,
        delta: PerformanceScore,
    ) -> Result<(), TournamentApplicationError> {
        self.matches_played = checked_add(self.matches_played, 1, "matches played")?;
        if won {
            self.matches_won = checked_add(self.matches_won, 1, "matches won")?;
        } else {
            self.matches_lost = checked_add(self.matches_lost, 1, "matches lost")?;
        }
        self.games_won = checked_add(self.games_won, u32::from(games_won), "games won")?;
        self.games_lost = checked_add(self.games_lost, u32::from(games_lost), "games lost")?;
        self.points_won = checked_sum(self.points_won, points_won, "points won")?;
        self.points_lost = checked_sum(self.points_lost, points_lost, "points lost")?;
        self.performance_score = PerformanceScore::from_scaled(
            self.performance_score
                .scaled_value()
                .checked_add(delta.scaled_value())
                .ok_or(overflow("performance score"))?,
        );
        Ok(())
    }

    pub fn add_opponent_score(
        &mut self,
        score: PerformanceScore,
    ) -> Result<(), TournamentApplicationError> {
        self.opponent_score_sum = PerformanceScore::from_scaled(
            self.opponent_score_sum
                .scaled_value()
                .checked_add(score.scaled_value())
                .ok_or(overflow("opponent score sum"))?,
        );
        Ok(())
    }

    pub fn finish(self) -> Result<ContestantStanding, TournamentApplicationError> {
        Ok(ContestantStanding {
            entrant_id: self.entrant_id,
            performance_score: self.performance_score,
            matches_played: self.matches_played,
            matches_won: self.matches_won,
            matches_lost: self.matches_lost,
            games_won: self.games_won,
            games_lost: self.games_lost,
            game_difference: difference(self.games_won, self.games_lost, "game difference")?,
            points_won: self.points_won,
            points_lost: self.points_lost,
            point_difference: difference(self.points_won, self.points_lost, "point difference")?,
            opponent_score_sum: self.opponent_score_sum,
            bye_count: self.bye_count,
        })
    }
}

fn checked_sum(
    initial: u32,
    mut values: impl Iterator<Item = u16>,
    component: &'static str,
) -> Result<u32, TournamentApplicationError> {
    values.try_fold(initial, |total, value| {
        checked_add(total, u32::from(value), component)
    })
}

fn checked_add(
    first: u32,
    second: u32,
    component: &'static str,
) -> Result<u32, TournamentApplicationError> {
    first.checked_add(second).ok_or(overflow(component))
}

fn difference(
    won: u32,
    lost: u32,
    component: &'static str,
) -> Result<i32, TournamentApplicationError> {
    i32::try_from(i64::from(won) - i64::from(lost)).map_err(|_| overflow(component))
}

const fn overflow(component: &'static str) -> TournamentApplicationError {
    TournamentApplicationError::StandingOverflow { component }
}
