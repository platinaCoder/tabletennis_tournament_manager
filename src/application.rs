//! Application operations shared by interactive frontends and simulations.
//!
//! This layer owns tournament orchestration. Pairing algorithms and result
//! validation remain isolated domain services beneath it.

mod entrant;
mod error;
mod pairing_snapshot;
mod round;
mod round_operations;
mod scoring;
mod simulation_export;
mod snapshot;
mod standing_accumulator;
mod standings;
mod tournament;

pub use entrant::TournamentEntrant;
pub use error::TournamentApplicationError;
pub use round::{ActiveRound, CompletedRound};
pub use scoring::{EloExpectationDeltaV1, MatchPerformanceDelta};
pub use simulation_export::{SimulationTrace, SimulationTraceError};
pub use snapshot::{
    PairingPreviewSnapshot, TournamentApplicationSnapshot, TournamentSnapshotError,
};
pub use standings::ContestantStanding;
pub use tournament::TournamentApplication;

#[cfg(test)]
mod tests;
