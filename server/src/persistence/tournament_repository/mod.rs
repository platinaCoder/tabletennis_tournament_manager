mod access;
mod error;
mod load;
mod match_writer;
mod model;
mod round_writer;
mod row;
mod save;
mod sharing;

pub use error::TournamentRepositoryError;
pub use model::{
    NewTournament, StoredTournament, TournamentInvitation, TournamentMember, TournamentSharing,
    TournamentSummary,
};

use sqlx_postgres::PgPool;

#[derive(Clone)]
pub struct TournamentRepository {
    pub(super) pool: PgPool,
}

impl TournamentRepository {
    pub const fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[cfg(test)]
mod postgres_tests;
