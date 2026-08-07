use crate::identity::{EntrantId, MatchId};

pub use crate::table::TableNumber;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MatchPublicationStatus {
    Draft,
    Published,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RoundActivity {
    Active,
    Inactive,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ScheduledMatch {
    pub match_id: MatchId,
    pub home_entrant_id: EntrantId,
    pub away_entrant_id: EntrantId,
    pub publication_status: MatchPublicationStatus,
    pub round_activity: RoundActivity,
    table_number: Option<TableNumber>,
}

impl ScheduledMatch {
    pub fn published(
        match_id: MatchId,
        home_entrant_id: EntrantId,
        away_entrant_id: EntrantId,
        table_number: Option<TableNumber>,
        round_activity: RoundActivity,
    ) -> Self {
        Self {
            match_id,
            home_entrant_id,
            away_entrant_id,
            publication_status: MatchPublicationStatus::Published,
            round_activity,
            table_number,
        }
    }

    pub fn published_in_active_round(
        match_id: MatchId,
        home_entrant_id: EntrantId,
        away_entrant_id: EntrantId,
    ) -> Self {
        Self::published(
            match_id,
            home_entrant_id,
            away_entrant_id,
            None,
            RoundActivity::Active,
        )
    }

    pub fn contains_entrant(&self, entrant_id: &EntrantId) -> bool {
        entrant_id == &self.home_entrant_id || entrant_id == &self.away_entrant_id
    }

    pub const fn table_number(&self) -> Option<TableNumber> {
        self.table_number
    }

    pub(crate) fn with_table_number(mut self, table_number: Option<TableNumber>) -> Self {
        self.table_number = table_number;
        self
    }
}
