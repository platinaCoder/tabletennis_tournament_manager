use std::collections::HashMap;
use std::error::Error;
use std::fmt::{self, Display, Formatter};

use crate::identity::EntrantId;
use crate::scheduling::ScheduledMatch;
use crate::table::TableNumber;
use crate::tournament::TableCount;

use super::EloRating;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TableAssignmentEntrant {
    pub entrant_id: EntrantId,
    pub starting_elo: EloRating,
}

/// Orders published matches independently of solver output and assigns tables.
/// Descending ELO sum is equivalent to descending average for two contestants.
pub fn assign_tables(
    table_count: TableCount,
    published_matches: Vec<ScheduledMatch>,
    entrants: &[TableAssignmentEntrant],
) -> Result<Vec<ScheduledMatch>, TableAssignmentError> {
    let elo_by_entrant = entrant_elo_index(entrants)?;
    let mut ranked_matches = published_matches
        .into_iter()
        .map(|scheduled_match| {
            let elo_sum = match_elo_sum(&scheduled_match, &elo_by_entrant)?;
            Ok((scheduled_match, elo_sum))
        })
        .collect::<Result<Vec<_>, TableAssignmentError>>()?;

    ranked_matches.sort_by(|(first_match, first_sum), (second_match, second_sum)| {
        second_sum.cmp(first_sum).then_with(|| {
            deterministic_match_key(first_match).cmp(&deterministic_match_key(second_match))
        })
    });

    Ok(ranked_matches
        .into_iter()
        .enumerate()
        .map(|(rank_index, (scheduled_match, _))| {
            scheduled_match.with_table_number(table_number_for_rank(rank_index, table_count))
        })
        .collect())
}

fn entrant_elo_index(
    entrants: &[TableAssignmentEntrant],
) -> Result<HashMap<&EntrantId, EloRating>, TableAssignmentError> {
    let mut index = HashMap::with_capacity(entrants.len());
    for entrant in entrants {
        if index
            .insert(&entrant.entrant_id, entrant.starting_elo)
            .is_some()
        {
            return Err(TableAssignmentError::DuplicateEntrant {
                entrant_id: entrant.entrant_id.clone(),
            });
        }
    }
    Ok(index)
}

fn match_elo_sum(
    scheduled_match: &ScheduledMatch,
    elo_by_entrant: &HashMap<&EntrantId, EloRating>,
) -> Result<u64, TableAssignmentError> {
    let home = entrant_elo(&scheduled_match.home_entrant_id, elo_by_entrant)?;
    let away = entrant_elo(&scheduled_match.away_entrant_id, elo_by_entrant)?;
    Ok(u64::from(home.value()) + u64::from(away.value()))
}

fn entrant_elo(
    entrant_id: &EntrantId,
    elo_by_entrant: &HashMap<&EntrantId, EloRating>,
) -> Result<EloRating, TableAssignmentError> {
    elo_by_entrant.get(entrant_id).copied().ok_or_else(|| {
        TableAssignmentError::MissingEntrantSnapshot {
            entrant_id: entrant_id.clone(),
        }
    })
}

fn deterministic_match_key(scheduled_match: &ScheduledMatch) -> (&str, &str, &str) {
    let first = scheduled_match.home_entrant_id.as_str();
    let second = scheduled_match.away_entrant_id.as_str();
    let (lower, higher) = if first <= second {
        (first, second)
    } else {
        (second, first)
    };
    (lower, higher, scheduled_match.match_id.as_str())
}

fn table_number_for_rank(index: usize, table_count: TableCount) -> Option<TableNumber> {
    let number = u16::try_from(index + 1).ok()?;
    TableNumber::within_configured_count(number, table_count.value())
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TableAssignmentError {
    DuplicateEntrant { entrant_id: EntrantId },
    MissingEntrantSnapshot { entrant_id: EntrantId },
}

impl Display for TableAssignmentError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::DuplicateEntrant { entrant_id } => write!(
                formatter,
                "entrant {} occurs more than once in the table-ranking snapshot",
                entrant_id.as_str()
            ),
            Self::MissingEntrantSnapshot { entrant_id } => write!(
                formatter,
                "published match entrant {} is missing from the table-ranking snapshot",
                entrant_id.as_str()
            ),
        }
    }
}

impl Error for TableAssignmentError {}
