use tabletennis_tournament::results::{MatchFormat, MatchResult};
use yew::prelude::*;

use crate::language::Language;

use super::form_state::GameInput;

pub(super) fn result_rows(
    match_format: MatchFormat,
    result: Option<&MatchResult>,
) -> Vec<GameInput> {
    let mut rows = vec![GameInput::default(); match_format.maximum_games()];
    if let Some(result) = result {
        for game in result.games() {
            if let Some(row) = rows.get_mut(usize::from(game.game_number.value() - 1)) {
                row.home = game.home_points.value().to_string();
                row.away = game.away_points.value().to_string();
            }
        }
    }
    rows
}

pub(super) fn cancel_correction(
    correcting: UseStateHandle<bool>,
    rows: UseStateHandle<Vec<GameInput>>,
    match_format: MatchFormat,
    result: Option<MatchResult>,
) -> Callback<MouseEvent> {
    Callback::from(move |_| {
        rows.set(result_rows(match_format, result.as_ref()));
        correcting.set(false);
    })
}

pub(super) const fn save_label(language: Language) -> &'static str {
    match language {
        Language::English => "Save correction",
        Language::Dutch => "Correctie opslaan",
    }
}

pub(super) const fn cancel_label(language: Language) -> &'static str {
    match language {
        Language::English => "Cancel",
        Language::Dutch => "Annuleren",
    }
}

#[cfg(test)]
mod tests {
    use tabletennis_tournament::identity::{EntrantId, MatchId};
    use tabletennis_tournament::results::{GameScore, ScheduledMatch, validate_and_complete_match};

    use super::*;

    #[test]
    fn correction_rows_are_prefilled_from_the_active_result() {
        let scheduled = ScheduledMatch::published_in_active_round(
            MatchId::new("match"),
            EntrantId::new("home"),
            EntrantId::new("away"),
        );
        let result = validate_and_complete_match(
            &scheduled,
            MatchFormat::BestOfThree,
            vec![
                GameScore::new(1, 11, 7).unwrap(),
                GameScore::new(2, 12, 10).unwrap(),
            ],
        )
        .unwrap();

        let rows = result_rows(MatchFormat::BestOfThree, Some(&result));

        assert_eq!(rows.len(), 3);
        assert_eq!(rows[0].home, "11");
        assert_eq!(rows[0].away, "7");
        assert_eq!(rows[1].home, "12");
        assert_eq!(rows[1].away, "10");
        assert_eq!(rows[2], GameInput::default());
    }
}
