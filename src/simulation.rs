//! Deterministic end-to-end tournament simulations.
//!
//! The harness intentionally uses [`crate::application::TournamentApplication`]
//! rather than calling pairing or result internals directly.

mod config;
mod error;
mod report;
mod result_generator;
mod runner;

pub use config::{SimulationConfig, SimulationEntrantPattern, standard_scenarios};
pub use error::SimulationError;
pub use report::{SimulationReport, SimulationRoundReport};
pub use result_generator::simulate_match_games;
pub use runner::{run_simulation, run_standard_scenarios};

#[cfg(test)]
mod tests;
