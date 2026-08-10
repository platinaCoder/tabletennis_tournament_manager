use tabletennis_tournament::application::TournamentEntrant;
use tabletennis_tournament::results::{MatchProgress, MatchProgressStatus, MatchResult, MatchSide};
use yew::prelude::*;

use crate::formatting::match_format;
use crate::language::{Language, Text};

use super::form_state::{FormError, FormEvaluation};
use super::match_form::MatchFormProps;

pub(super) fn progress_label(
    evaluation: &FormEvaluation,
    props: &MatchFormProps,
    language: Language,
) -> Html {
    if let Some(error) = &evaluation.error {
        return html! { <span class="error-text">{form_error_label(error, language)}</span> };
    }
    match evaluation.progress.map(MatchProgress::status) {
        Some(MatchProgressStatus::Complete { winner }) => {
            let winner_name = match winner {
                MatchSide::Home => props.home.as_ref().map(|entrant| entrant.name.as_str()),
                MatchSide::Away => props.away.as_ref().map(|entrant| entrant.name.as_str()),
            }
            .unwrap_or(language.text(Text::UnknownContestant));
            html! { <strong class="success-text">{language.complete_winner(winner_name)}</strong> }
        }
        Some(_) => html! { <span class="muted">{language.text(Text::EnterRemainingGames)}</span> },
        None => Html::default(),
    }
}

pub(super) fn completed_match(
    props: &MatchFormProps,
    result: &MatchResult,
    language: Language,
    begin_correction: Callback<MouseEvent>,
) -> Html {
    let scores = result
        .games()
        .iter()
        .map(|game| format!("{}-{}", game.home_points.value(), game.away_points.value()))
        .collect::<Vec<_>>()
        .join(", ");
    let winner_name = if result.winner_id() == &props.scheduled_match.home_entrant_id {
        props.home.as_ref().map(|entrant| entrant.name.as_str())
    } else if result.winner_id() == &props.scheduled_match.away_entrant_id {
        props.away.as_ref().map(|entrant| entrant.name.as_str())
    } else {
        None
    }
    .unwrap_or(language.text(Text::UnknownContestant));
    html! {
        <article class="match-card complete-card">
            {match_header(props, language)}
            <div class="completed-score">
                <strong>{format!("{}-{}", result.home_games_won().value(), result.away_games_won().value())}</strong>
                <span>{scores}</span>
                <small>{language.winner(winner_name)}</small>
                if result.revision().value() > 1 {
                    <small class="correction-audit-label">
                        {corrected_revision_label(result.revision().value(), language)}
                    </small>
                    if let Some(reason) = result.correction_reason() {
                        <small>{correction_reason_summary(reason, language)}</small>
                    }
                }
            </div>
            if props.can_edit {
                <button class="secondary compact correct-result-button" onclick={begin_correction}>
                    {correct_result_label(language)}
                </button>
            }
        </article>
    }
}

pub(super) fn match_header(props: &MatchFormProps, language: Language) -> Html {
    let table = props.scheduled_match.table_number().map_or_else(
        || language.text(Text::WaitingForTable).to_owned(),
        |table| language.table(table.value()),
    );
    html! {
        <header class="match-header">
            <span class="table-badge">{table}</span>
            <div>
                {entrant_line(props.home.as_ref(), language.text(Text::Home), language)}
                {entrant_line(props.away.as_ref(), language.text(Text::Away), language)}
            </div>
            <small>{match_format(props.match_format, language)}</small>
        </header>
    }
}

fn entrant_line(entrant: Option<&TournamentEntrant>, side: &str, language: Language) -> Html {
    html! {
        <div class="match-entrant">
            <span>{side}</span>
            <strong>{entrant.map_or(language.text(Text::UnknownContestant), |entrant| entrant.name.as_str())}</strong>
            <small>
                {entrant.map_or(language.text(Text::UnknownClub), |entrant| entrant.club_name.as_str())}
                {entrant.map_or_else(|| format!(" · {}", language.text(Text::EloUnavailable)), |entrant| format!(" · ELO {}", entrant.starting_elo.value()))}
            </small>
        </div>
    }
}

fn form_error_label(error: &FormError, language: Language) -> String {
    match error {
        FormError::BlankRows => language.sequential_games_error().to_owned(),
        FormError::WholeNumbers => language.whole_points_error().to_owned(),
        FormError::GameNumberLimit => language.game_number_limit_error().to_owned(),
        FormError::InvalidGameNumber => language.invalid_game_number_error().to_owned(),
        FormError::MatchResult(error) => language.match_result_error(error),
    }
}

pub(super) const fn read_only_result_label(language: Language) -> &'static str {
    match language {
        Language::English => "Waiting for an editor to enter this result.",
        Language::Dutch => "Wachten tot een bewerker deze uitslag invoert.",
    }
}

const fn correct_result_label(language: Language) -> &'static str {
    match language {
        Language::English => "Correct result",
        Language::Dutch => "Uitslag corrigeren",
    }
}

fn corrected_revision_label(revision: u32, language: Language) -> String {
    match language {
        Language::English => format!("Corrected · revision {revision}"),
        Language::Dutch => format!("Gecorrigeerd · revisie {revision}"),
    }
}

fn correction_reason_summary(reason: &str, language: Language) -> String {
    match language {
        Language::English => format!("Reason: {reason}"),
        Language::Dutch => format!("Reden: {reason}"),
    }
}
