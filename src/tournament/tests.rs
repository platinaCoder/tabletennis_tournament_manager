use crate::results::{
    EntrantId, GameScore, MatchId, MatchResultError, ScheduledMatch, validate_and_complete_match,
};

use super::*;

fn tournament(match_format: MatchFormat) -> Tournament {
    Tournament::new(
        TournamentId::new("tournament-1"),
        match_format,
        TableCount::try_from(8_i64).unwrap(),
        MaximumRoundCount::try_from(5_i64).unwrap(),
    )
}

fn game(number: u8, home: u16, away: u16) -> GameScore {
    GameScore::new(number, home, away).unwrap()
}

fn scheduled_match() -> ScheduledMatch {
    ScheduledMatch::published_in_active_round(
        MatchId::new("match-1"),
        EntrantId::new("home"),
        EntrantId::new("away"),
    )
}

#[test]
fn tournament_is_created_in_draft_with_required_configuration() {
    let tournament = tournament(MatchFormat::BestOfThree);

    assert_eq!(tournament.id().as_str(), "tournament-1");
    assert_eq!(tournament.state(), TournamentState::Draft);
    assert_eq!(tournament.match_format(), MatchFormat::BestOfThree);
    assert_eq!(tournament.table_count().value(), 8);
    assert_eq!(tournament.maximum_round_count().value(), 5);
}

#[test]
fn table_count_rejects_zero_negative_and_overflowing_input() {
    assert_eq!(
        TableCount::try_from(0_i64),
        Err(TableCountError::NotPositive)
    );
    assert_eq!(
        TableCount::try_from(-1_i64),
        Err(TableCountError::NotPositive)
    );
    assert_eq!(
        TableCount::try_from(i64::from(u16::MAX) + 1),
        Err(TableCountError::ExceedsStorageLimit)
    );
}

#[test]
fn maximum_round_count_rejects_zero_negative_and_overflowing_input() {
    assert_eq!(
        MaximumRoundCount::try_from(0_i64),
        Err(MaximumRoundCountError::NotPositive)
    );
    assert_eq!(
        MaximumRoundCount::try_from(-1_i64),
        Err(MaximumRoundCountError::NotPositive)
    );
    assert_eq!(
        MaximumRoundCount::try_from(i64::from(u16::MAX) + 1),
        Err(MaximumRoundCountError::ExceedsStorageLimit)
    );
}

#[test]
fn draft_configuration_can_change() {
    let mut tournament = tournament(MatchFormat::BestOfThree);

    tournament
        .change_match_format(MatchFormat::BestOfFive)
        .unwrap();
    tournament
        .change_table_count(TableCount::try_from(12_i64).unwrap())
        .unwrap();
    tournament
        .change_maximum_round_count(MaximumRoundCount::try_from(7_i64).unwrap())
        .unwrap();

    assert_eq!(tournament.match_format(), MatchFormat::BestOfFive);
    assert_eq!(tournament.table_count().value(), 12);
    assert_eq!(tournament.maximum_round_count().value(), 7);
}

#[test]
fn starting_tournament_freezes_configuration() {
    let mut tournament = tournament(MatchFormat::BestOfThree);
    tournament.start().unwrap();

    let format_error = tournament
        .change_match_format(MatchFormat::BestOfFive)
        .unwrap_err();
    let tables_error = tournament
        .change_table_count(TableCount::try_from(6_i64).unwrap())
        .unwrap_err();
    let rounds_error = tournament
        .change_maximum_round_count(MaximumRoundCount::try_from(6_i64).unwrap())
        .unwrap_err();

    assert_eq!(format_error, TournamentError::MatchFormatFrozen);
    assert_eq!(tables_error, TournamentError::TableCountFrozen);
    assert_eq!(rounds_error, TournamentError::MaximumRoundCountFrozen);
    assert_eq!(tournament.match_format(), MatchFormat::BestOfThree);
    assert_eq!(tournament.table_count().value(), 8);
    assert_eq!(tournament.maximum_round_count().value(), 5);
}

#[test]
fn tournament_cannot_be_started_twice() {
    let mut tournament = tournament(MatchFormat::BestOfThree);
    tournament.start().unwrap();

    assert_eq!(tournament.start(), Err(TournamentError::AlreadyStarted));
    assert_eq!(tournament.state(), TournamentState::Started);
}

#[test]
fn configured_format_drives_match_completion_rules() {
    let mut tournament = tournament(MatchFormat::BestOfFive);
    tournament.start().unwrap();

    let error = validate_and_complete_match(
        &scheduled_match(),
        tournament.match_format(),
        vec![game(1, 11, 5), game(2, 11, 5)],
    )
    .unwrap_err();
    assert!(matches!(error, MatchResultError::MatchNotComplete { .. }));

    let completed = validate_and_complete_match(
        &scheduled_match(),
        tournament.match_format(),
        vec![game(1, 11, 5), game(2, 11, 5), game(3, 11, 5)],
    )
    .unwrap();
    assert_eq!(completed.home_games_won().value(), 3);
}
