use crate::identity::{EntrantId, MatchId};
use crate::scheduling::{MatchPublicationStatus, RoundActivity};
use crate::tournament::TableCount;

use super::*;

fn publication(id: &str, first: &str, second: &str) -> MatchPublication {
    MatchPublication {
        match_id: MatchId::new(id),
        first_entrant_id: EntrantId::new(first),
        second_entrant_id: EntrantId::new(second),
    }
}

fn entrant(id: &str, elo: u32) -> TableAssignmentEntrant {
    TableAssignmentEntrant {
        entrant_id: EntrantId::new(id),
        starting_elo: EloRating::new(elo),
    }
}

fn tables(count: i64) -> TableCount {
    TableCount::try_from(count).unwrap()
}

#[test]
fn publication_uses_application_match_ids_without_assigning_tables() {
    let scheduled = publish_scheduled_matches(
        vec![publication("generated-id", "first", "second")],
        RoundActivity::Active,
    );

    assert_eq!(scheduled[0].match_id.as_str(), "generated-id");
    assert_eq!(scheduled[0].home_entrant_id.as_str(), "first");
    assert_eq!(scheduled[0].away_entrant_id.as_str(), "second");
    assert_eq!(
        scheduled[0].publication_status,
        MatchPublicationStatus::Published
    );
    assert_eq!(scheduled[0].table_number(), None);
}

#[test]
fn table_assignment_uses_average_elo_not_input_order() {
    let published = publish_scheduled_matches(
        vec![
            publication("low", "low-a", "low-b"),
            publication("high", "high-a", "high-b"),
            publication("middle", "middle-a", "middle-b"),
        ],
        RoundActivity::Active,
    );
    let snapshots = [
        entrant("low-a", 900),
        entrant("low-b", 1100),
        entrant("high-a", 2000),
        entrant("high-b", 2200),
        entrant("middle-a", 1400),
        entrant("middle-b", 1600),
    ];

    let assigned = assign_tables(tables(3), published, &snapshots).unwrap();

    assert_eq!(assigned[0].match_id.as_str(), "high");
    assert_eq!(assigned[0].table_number().unwrap().value(), 1);
    assert_eq!(assigned[1].match_id.as_str(), "middle");
    assert_eq!(assigned[1].table_number().unwrap().value(), 2);
    assert_eq!(assigned[2].match_id.as_str(), "low");
    assert_eq!(assigned[2].table_number().unwrap().value(), 3);
}

#[test]
fn equal_average_elo_uses_deterministic_participant_tie_break() {
    let first_order = publish_scheduled_matches(
        vec![
            publication("match-z", "charlie", "delta"),
            publication("match-a", "alpha", "bravo"),
        ],
        RoundActivity::Active,
    );
    let reverse_order = publish_scheduled_matches(
        vec![
            publication("match-a", "alpha", "bravo"),
            publication("match-z", "charlie", "delta"),
        ],
        RoundActivity::Active,
    );
    let snapshots = [
        entrant("alpha", 1500),
        entrant("bravo", 1500),
        entrant("charlie", 1500),
        entrant("delta", 1500),
    ];

    let first = assign_tables(tables(2), first_order, &snapshots).unwrap();
    let second = assign_tables(tables(2), reverse_order, &snapshots).unwrap();

    let first_ids = first
        .iter()
        .map(|item| item.match_id.as_str())
        .collect::<Vec<_>>();
    let second_ids = second
        .iter()
        .map(|item| item.match_id.as_str())
        .collect::<Vec<_>>();
    assert_eq!(first_ids, vec!["match-a", "match-z"]);
    assert_eq!(first_ids, second_ids);
}

#[test]
fn excess_matches_remain_unassigned() {
    let published = publish_scheduled_matches(
        vec![
            publication("first", "a", "b"),
            publication("second", "c", "d"),
        ],
        RoundActivity::Active,
    );
    let snapshots = [
        entrant("a", 2000),
        entrant("b", 2000),
        entrant("c", 1000),
        entrant("d", 1000),
    ];

    let assigned = assign_tables(tables(1), published, &snapshots).unwrap();

    assert_eq!(assigned[0].table_number().unwrap().value(), 1);
    assert_eq!(assigned[1].table_number(), None);
}

#[test]
fn invalid_table_ranking_snapshots_return_typed_errors() {
    let duplicate_snapshots = [entrant("a", 1500), entrant("a", 1600)];
    assert!(matches!(
        assign_tables(tables(1), Vec::new(), &duplicate_snapshots),
        Err(TableAssignmentError::DuplicateEntrant { .. })
    ));

    let published = publish_scheduled_matches(
        vec![publication("match", "a", "missing")],
        RoundActivity::Active,
    );
    assert!(matches!(
        assign_tables(tables(1), published, &[entrant("a", 1500)]),
        Err(TableAssignmentError::MissingEntrantSnapshot { .. })
    ));
}
