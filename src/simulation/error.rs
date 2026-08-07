use std::error::Error;
use std::fmt::{self, Display, Formatter};

use crate::application::TournamentApplicationError;

#[derive(Debug)]
pub enum SimulationError {
    InvalidConfiguration { reason: &'static str },
    GeneratedInvalidGameNumber,
    Application(TournamentApplicationError),
}

impl Display for SimulationError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidConfiguration { reason } => {
                write!(formatter, "invalid simulation configuration: {reason}")
            }
            Self::GeneratedInvalidGameNumber => {
                formatter.write_str("simulator generated an invalid game number")
            }
            Self::Application(error) => Display::fmt(error, formatter),
        }
    }
}

impl Error for SimulationError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Application(error) => Some(error),
            _ => None,
        }
    }
}

impl From<TournamentApplicationError> for SimulationError {
    fn from(value: TournamentApplicationError) -> Self {
        Self::Application(value)
    }
}
