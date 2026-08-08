mod builder;
mod model;
mod pairing_trace;

use std::error::Error;
use std::fmt::{self, Display, Formatter};

use crate::pairing::algorithms::blossom_v1::BlossomPairingError;

use super::TournamentApplicationError;

pub use model::SimulationTrace;

#[derive(Debug)]
pub enum SimulationTraceError {
    Application(TournamentApplicationError),
    Pairing(BlossomPairingError),
}

impl Display for SimulationTraceError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::Application(error) => {
                write!(formatter, "could not calculate trace standings: {error}")
            }
            Self::Pairing(error) => write!(formatter, "could not build pairing trace: {error}"),
        }
    }
}

impl Error for SimulationTraceError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Application(error) => Some(error),
            Self::Pairing(error) => Some(error),
        }
    }
}

impl From<TournamentApplicationError> for SimulationTraceError {
    fn from(error: TournamentApplicationError) -> Self {
        Self::Application(error)
    }
}

impl From<BlossomPairingError> for SimulationTraceError {
    fn from(error: BlossomPairingError) -> Self {
        Self::Pairing(error)
    }
}
