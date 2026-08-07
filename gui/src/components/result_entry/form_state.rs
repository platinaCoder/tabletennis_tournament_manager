use tabletennis_tournament::results::{
    GameScore, MatchFormat, MatchProgress, evaluate_match_progress,
};

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(super) struct GameInput {
    pub home: String,
    pub away: String,
}

pub(super) struct FormEvaluation {
    pub games: Vec<GameScore>,
    pub progress: Option<MatchProgress>,
    pub error: Option<String>,
}

pub(super) fn evaluate_rows(match_format: MatchFormat, rows: &[GameInput]) -> FormEvaluation {
    let mut games = Vec::new();
    let mut found_empty = false;
    for (index, row) in rows.iter().enumerate() {
        if row.home.is_empty() && row.away.is_empty() {
            found_empty = true;
            continue;
        }
        if found_empty {
            return form_error(games, "Enter games sequentially without blank rows.");
        }
        let (Ok(home), Ok(away)) = (row.home.parse::<u16>(), row.away.parse::<u16>()) else {
            return form_error(games, "Enter both point totals using whole numbers.");
        };
        let Ok(game_number) = u8::try_from(index + 1) else {
            return form_error(games, "Game number exceeds the supported limit.");
        };
        let Ok(game) = GameScore::new(game_number, home, away) else {
            return form_error(games, "Game number is invalid.");
        };
        games.push(game);
    }

    match evaluate_match_progress(match_format, &games) {
        Ok(progress) => FormEvaluation {
            games,
            progress: Some(progress),
            error: None,
        },
        Err(error) => FormEvaluation {
            games,
            progress: None,
            error: Some(error.to_string()),
        },
    }
}

fn form_error(games: Vec<GameScore>, message: &str) -> FormEvaluation {
    FormEvaluation {
        games,
        progress: None,
        error: Some(message.to_owned()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn row(home: u16, away: u16) -> GameInput {
        GameInput {
            home: home.to_string(),
            away: away.to_string(),
        }
    }

    #[test]
    fn complete_best_of_three_is_derived_by_the_results_domain() {
        let evaluation = evaluate_rows(
            MatchFormat::BestOfThree,
            &[row(11, 7), row(8, 11), row(13, 11)],
        );

        assert!(evaluation.error.is_none());
        assert!(evaluation.progress.is_some_and(MatchProgress::is_complete));
    }

    #[test]
    fn an_extra_game_after_completion_is_rejected() {
        let evaluation = evaluate_rows(
            MatchFormat::BestOfThree,
            &[row(11, 7), row(11, 4), row(11, 9)],
        );

        assert!(evaluation.progress.is_none());
        assert!(evaluation.error.is_some());
    }

    #[test]
    fn blank_rows_between_entered_games_are_rejected() {
        let evaluation = evaluate_rows(
            MatchFormat::BestOfFive,
            &[row(11, 7), GameInput::default(), row(8, 11)],
        );

        assert_eq!(
            evaluation.error.as_deref(),
            Some("Enter games sequentially without blank rows.")
        );
    }
}
