use std::time::SystemTime;

use super::match_result::validate_and_complete_match_at;
use super::*;

fn scheduled_match() -> ScheduledMatch {
    ScheduledMatch::published_in_active_round(
        MatchId::new("match-1"),
        EntrantId::new("home"),
        EntrantId::new("away"),
    )
}

fn game(number: u8, home: u16, away: u16) -> GameScore {
    GameScore::new(number, home, away).unwrap()
}

fn complete(
    match_format: MatchFormat,
    games: Vec<GameScore>,
) -> Result<MatchResult, MatchResultError> {
    validate_and_complete_match(&scheduled_match(), match_format, games)
}

#[test]
fn accepts_normal_and_deuce_game_scores() {
    for (home, away) in [(11, 0), (11, 9), (12, 10), (24, 22)] {
        let result = complete(
            MatchFormat::BestOfThree,
            vec![game(1, home, away), game(2, 11, 0)],
        );

        assert!(result.is_ok(), "expected {home}-{away} to be valid");
    }
}

#[test]
fn rejects_games_without_a_two_point_winning_margin() {
    for (home, away) in [(11, 10), (12, 11), (11, 11), (10, 8), (9, 7)] {
        let error = complete(
            MatchFormat::BestOfThree,
            vec![game(1, home, away), game(2, 11, 0)],
        )
        .unwrap_err();

        assert_eq!(error.code(), "invalid_game_score");
    }
}

#[test]
fn rejects_skipped_and_duplicate_game_numbers() {
    for games in [
        vec![game(1, 11, 5), game(3, 11, 5)],
        vec![game(1, 11, 5), game(1, 11, 5)],
    ] {
        assert!(matches!(
            complete(MatchFormat::BestOfThree, games),
            Err(MatchResultError::GameNumbersNotSequential { .. })
        ));
    }
}

#[test]
fn empty_match_progress_is_valid_and_incomplete() {
    let progress = evaluate_match_progress(MatchFormat::BestOfThree, &[]).unwrap();

    assert_eq!(progress.home_games_won().value(), 0);
    assert_eq!(progress.away_games_won().value(), 0);
    assert_eq!(progress.status(), MatchProgressStatus::Incomplete);
    assert_eq!(progress.winner(), None);
    assert!(!progress.is_complete());
}

#[test]
fn match_progress_derives_live_game_totals() {
    let games = [game(1, 11, 5), game(2, 9, 11)];
    let progress = evaluate_match_progress(MatchFormat::BestOfThree, &games).unwrap();

    assert_eq!(progress.home_games_won().value(), 1);
    assert_eq!(progress.away_games_won().value(), 1);
    assert_eq!(progress.status(), MatchProgressStatus::Incomplete);
}

#[test]
fn match_progress_derives_completion_and_winner_side() {
    let home_games = [game(1, 11, 5), game(2, 11, 9)];
    let home_progress = evaluate_match_progress(MatchFormat::BestOfThree, &home_games).unwrap();
    assert_eq!(home_progress.winner(), Some(MatchSide::Home));
    assert!(home_progress.is_complete());

    let away_games = [game(1, 8, 11), game(2, 10, 12)];
    let away_progress = evaluate_match_progress(MatchFormat::BestOfThree, &away_games).unwrap();
    assert_eq!(away_progress.winner(), Some(MatchSide::Away));
}

#[test]
fn completes_both_valid_best_of_three_shapes() {
    let two_zero = complete(
        MatchFormat::BestOfThree,
        vec![game(1, 11, 5), game(2, 12, 10)],
    )
    .unwrap();
    assert_eq!(two_zero.home_games_won().value(), 2);
    assert_eq!(two_zero.away_games_won().value(), 0);

    let two_one = complete(
        MatchFormat::BestOfThree,
        vec![game(1, 11, 5), game(2, 8, 11), game(3, 13, 11)],
    )
    .unwrap();
    assert_eq!(two_one.home_games_won().value(), 2);
    assert_eq!(two_one.away_games_won().value(), 1);
}

#[test]
fn rejects_incomplete_best_of_three_results() {
    for games in [vec![game(1, 11, 5)], vec![game(1, 11, 5), game(2, 8, 11)]] {
        assert!(matches!(
            complete(MatchFormat::BestOfThree, games),
            Err(MatchResultError::MatchNotComplete { .. })
        ));
    }
}

#[test]
fn rejects_excess_best_of_three_games() {
    let fourth_game = vec![
        game(1, 11, 5),
        game(2, 8, 11),
        game(3, 13, 11),
        game(4, 11, 9),
    ];
    assert_eq!(
        complete(MatchFormat::BestOfThree, fourth_game)
            .unwrap_err()
            .code(),
        "too_many_games"
    );

    let after_two_zero = vec![game(1, 11, 5), game(2, 11, 5), game(3, 11, 5)];
    assert_eq!(
        complete(MatchFormat::BestOfThree, after_two_zero)
            .unwrap_err()
            .code(),
        "games_recorded_after_match_completion"
    );
}

#[test]
fn completes_all_valid_best_of_five_shapes() {
    let cases = [
        (vec![game(1, 11, 5), game(2, 11, 5), game(3, 11, 5)], (3, 0)),
        (
            vec![
                game(1, 8, 11),
                game(2, 11, 5),
                game(3, 11, 5),
                game(4, 11, 5),
            ],
            (3, 1),
        ),
        (
            vec![
                game(1, 8, 11),
                game(2, 11, 5),
                game(3, 7, 11),
                game(4, 11, 5),
                game(5, 11, 9),
            ],
            (3, 2),
        ),
    ];

    for (games, expected) in cases {
        let result = complete(MatchFormat::BestOfFive, games).unwrap();
        assert_eq!(
            (
                result.home_games_won().value(),
                result.away_games_won().value()
            ),
            expected
        );
    }
}

#[test]
fn rejects_incomplete_best_of_five_results() {
    for games in [
        vec![game(1, 11, 5), game(2, 11, 5)],
        vec![
            game(1, 11, 5),
            game(2, 8, 11),
            game(3, 11, 5),
            game(4, 8, 11),
        ],
    ] {
        assert!(matches!(
            complete(MatchFormat::BestOfFive, games),
            Err(MatchResultError::MatchNotComplete { .. })
        ));
    }
}

#[test]
fn rejects_excess_best_of_five_games() {
    let sixth_game = vec![
        game(1, 11, 5),
        game(2, 8, 11),
        game(3, 11, 5),
        game(4, 8, 11),
        game(5, 11, 5),
        game(6, 11, 5),
    ];
    assert_eq!(
        complete(MatchFormat::BestOfFive, sixth_game)
            .unwrap_err()
            .code(),
        "too_many_games"
    );

    for games in [
        vec![
            game(1, 11, 5),
            game(2, 11, 5),
            game(3, 11, 5),
            game(4, 11, 5),
        ],
        vec![
            game(1, 8, 11),
            game(2, 11, 5),
            game(3, 11, 5),
            game(4, 11, 5),
            game(5, 11, 5),
        ],
    ] {
        assert!(matches!(
            complete(MatchFormat::BestOfFive, games),
            Err(MatchResultError::GamesRecordedAfterMatchCompletion { .. })
        ));
    }
}

#[test]
fn derives_home_and_away_winners_from_games() {
    let home = complete(
        MatchFormat::BestOfThree,
        vec![game(1, 11, 5), game(2, 11, 5)],
    )
    .unwrap();
    assert_eq!(home.winner_id(), &EntrantId::new("home"));

    let away = complete(
        MatchFormat::BestOfThree,
        vec![game(1, 9, 11), game(2, 10, 12)],
    )
    .unwrap();
    assert_eq!(away.winner_id(), &EntrantId::new("away"));
}

#[test]
fn rejects_unpublished_matches_and_inactive_rounds() {
    let mut scheduled = scheduled_match();
    scheduled.publication_status = MatchPublicationStatus::Draft;
    assert_eq!(
        validate_and_complete_match(
            &scheduled,
            MatchFormat::BestOfThree,
            vec![game(1, 11, 5), game(2, 11, 5)]
        )
        .unwrap_err(),
        MatchResultError::MatchNotPublished
    );

    scheduled.publication_status = MatchPublicationStatus::Published;
    scheduled.round_activity = RoundActivity::Inactive;
    assert_eq!(
        validate_and_complete_match(
            &scheduled,
            MatchFormat::BestOfThree,
            vec![game(1, 11, 5), game(2, 11, 5)]
        )
        .unwrap_err(),
        MatchResultError::RoundNotActive
    );
}

#[test]
fn new_result_has_initial_audit_metadata() {
    let entered_at = SystemTime::UNIX_EPOCH;
    let result = validate_and_complete_match_at(
        &scheduled_match(),
        MatchFormat::BestOfThree,
        vec![game(1, 11, 5), game(2, 11, 5)],
        entered_at,
    )
    .unwrap();

    assert_eq!(result.entered_at(), entered_at);
    assert_eq!(result.corrected_at(), None);
    assert_eq!(result.revision().value(), 1);
    assert_eq!(result.games().len(), 2);
}
