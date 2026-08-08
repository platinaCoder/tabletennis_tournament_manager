mod maximum_round_count;
mod table_count;

use std::error::Error;
use std::fmt::{self, Display, Formatter};

use serde::{Deserialize, Serialize};

use crate::results::MatchFormat;

pub use maximum_round_count::{MaximumRoundCount, MaximumRoundCountError};
pub use table_count::{TableCount, TableCountError};

#[derive(Clone, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub struct TournamentId(String);

impl TournamentId {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum TournamentState {
    Draft,
    Started,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct Tournament {
    id: TournamentId,
    state: TournamentState,
    match_format: MatchFormat,
    table_count: TableCount,
    maximum_round_count: MaximumRoundCount,
}

impl Tournament {
    pub fn new(
        id: TournamentId,
        match_format: MatchFormat,
        table_count: TableCount,
        maximum_round_count: MaximumRoundCount,
    ) -> Self {
        Self {
            id,
            state: TournamentState::Draft,
            match_format,
            table_count,
            maximum_round_count,
        }
    }

    pub fn id(&self) -> &TournamentId {
        &self.id
    }

    pub const fn state(&self) -> TournamentState {
        self.state
    }

    pub const fn match_format(&self) -> MatchFormat {
        self.match_format
    }

    pub const fn table_count(&self) -> TableCount {
        self.table_count
    }

    pub const fn maximum_round_count(&self) -> MaximumRoundCount {
        self.maximum_round_count
    }

    pub fn change_match_format(&mut self, replacement: MatchFormat) -> Result<(), TournamentError> {
        self.ensure_draft(TournamentError::MatchFormatFrozen)?;
        self.match_format = replacement;
        Ok(())
    }

    pub fn change_table_count(&mut self, replacement: TableCount) -> Result<(), TournamentError> {
        self.ensure_draft(TournamentError::TableCountFrozen)?;
        self.table_count = replacement;
        Ok(())
    }

    pub fn change_maximum_round_count(
        &mut self,
        replacement: MaximumRoundCount,
    ) -> Result<(), TournamentError> {
        self.ensure_draft(TournamentError::MaximumRoundCountFrozen)?;
        self.maximum_round_count = replacement;
        Ok(())
    }

    pub fn start(&mut self) -> Result<(), TournamentError> {
        self.ensure_draft(TournamentError::AlreadyStarted)?;
        self.state = TournamentState::Started;
        Ok(())
    }

    fn ensure_draft(&self, error: TournamentError) -> Result<(), TournamentError> {
        match self.state {
            TournamentState::Draft => Ok(()),
            TournamentState::Started => Err(error),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TournamentError {
    MatchFormatFrozen,
    TableCountFrozen,
    MaximumRoundCountFrozen,
    AlreadyStarted,
}

impl TournamentError {
    pub const fn code(self) -> &'static str {
        match self {
            Self::MatchFormatFrozen => "match_format_frozen",
            Self::TableCountFrozen => "table_count_frozen",
            Self::MaximumRoundCountFrozen => "maximum_round_count_frozen",
            Self::AlreadyStarted => "tournament_already_started",
        }
    }
}

impl Display for TournamentError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::MatchFormatFrozen => {
                formatter.write_str("match format is frozen after the tournament starts")
            }
            Self::TableCountFrozen => {
                formatter.write_str("table count is frozen after the tournament starts")
            }
            Self::MaximumRoundCountFrozen => {
                formatter.write_str("maximum round count is frozen after the tournament starts")
            }
            Self::AlreadyStarted => formatter.write_str("tournament has already started"),
        }
    }
}

impl Error for TournamentError {}

#[cfg(test)]
mod tests;
