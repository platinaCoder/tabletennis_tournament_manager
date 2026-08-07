use web_sys::HtmlInputElement;
use yew::prelude::*;

use tabletennis_tournament::application::TournamentEntrant;
use tabletennis_tournament::results::{
    MatchFormat, MatchProgress, MatchProgressStatus, MatchResult, MatchSide,
};
use tabletennis_tournament::scheduling::ScheduledMatch;

use crate::formatting::match_format;
use crate::language::{Language, Text, use_language};

use super::SubmittedResult;
use super::form_state::{FormError, FormEvaluation, GameInput, evaluate_rows};

#[derive(Properties, PartialEq)]
pub struct MatchFormProps {
    pub scheduled_match: ScheduledMatch,
    pub home: Option<TournamentEntrant>,
    pub away: Option<TournamentEntrant>,
    pub match_format: MatchFormat,
    pub existing_result: Option<MatchResult>,
    pub autofocus: bool,
    pub on_submit: Callback<SubmittedResult>,
}

#[component]
pub fn MatchForm(props: &MatchFormProps) -> Html {
    let language = use_language();
    let rows = use_state(|| vec![GameInput::default(); props.match_format.maximum_games()]);
    let first_home_input = use_node_ref();
    {
        let first_home_input = first_home_input.clone();
        let should_focus = props.autofocus
            && props.existing_result.is_none()
            && props.scheduled_match.table_number().is_some();
        use_effect_with(should_focus, move |should_focus| {
            if *should_focus && let Some(input) = first_home_input.cast::<HtmlInputElement>() {
                let _ = input.focus();
            }
        });
    }
    if let Some(result) = &props.existing_result {
        return completed_match(props, result, language);
    }
    if props.scheduled_match.table_number().is_none() {
        return html! {
            <article class="match-card waiting-card">
                {match_header(props, language)}
                <p class="muted">{language.text(Text::ResultEntryWaitsForTable)}</p>
            </article>
        };
    }

    let evaluation = evaluate_rows(props.match_format, &rows);
    let is_complete = evaluation.progress.is_some_and(MatchProgress::is_complete);
    let onsubmit = {
        let on_submit = props.on_submit.clone();
        let match_id = props.scheduled_match.match_id.clone();
        let games = evaluation.games.clone();
        Callback::from(move |event: SubmitEvent| {
            event.prevent_default();
            if is_complete {
                on_submit.emit(SubmittedResult {
                    match_id: match_id.clone(),
                    games: games.clone(),
                });
            }
        })
    };

    html! {
        <article class="match-card">
            {match_header(props, language)}
            <form {onsubmit}>
                <div class="game-grid game-grid-header">
                    <span>{language.text(Text::Game)}</span><span>{language.text(Text::Home)}</span><span>{language.text(Text::Away)}</span>
                </div>
                {for rows.iter().enumerate().map(|(index, row)| {
                    let disabled = is_complete && index >= evaluation.games.len();
                    html! {
                        <div class="game-grid" key={index}>
                            <span class="game-number">{index + 1}</span>
                            <input
                                aria-label={language.game_home_points_label(index + 1)}
                                type="number"
                                min="0"
                                max="65535"
                                value={row.home.clone()}
                                disabled={disabled}
                                autofocus={props.autofocus && index == 0}
                                ref={if index == 0 { first_home_input.clone() } else { NodeRef::default() }}
                                oninput={update_score(rows.clone(), index, true)}
                            />
                            <input
                                aria-label={language.game_away_points_label(index + 1)}
                                type="number"
                                min="0"
                                max="65535"
                                value={row.away.clone()}
                                disabled={disabled}
                                oninput={update_score(rows.clone(), index, false)}
                            />
                        </div>
                    }
                })}
                <div class="match-progress">
                    {progress_label(&evaluation, props, language)}
                    <button class="primary compact" type="submit" disabled={!is_complete}>{language.text(Text::SaveResult)}</button>
                </div>
            </form>
        </article>
    }
}

fn update_score(
    rows: UseStateHandle<Vec<GameInput>>,
    index: usize,
    home: bool,
) -> Callback<InputEvent> {
    Callback::from(move |event: InputEvent| {
        let mut replacement = (*rows).clone();
        let value = event.target_unchecked_into::<HtmlInputElement>().value();
        if let Some(row) = replacement.get_mut(index) {
            if home {
                row.home = value;
            } else {
                row.away = value;
            }
        }
        rows.set(replacement);
    })
}

fn progress_label(evaluation: &FormEvaluation, props: &MatchFormProps, language: Language) -> Html {
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
            html! {
                <strong class="success-text">{language.complete_winner(winner_name)}</strong>
            }
        }
        Some(_) => html! { <span class="muted">{language.text(Text::EnterRemainingGames)}</span> },
        None => Html::default(),
    }
}

fn completed_match(props: &MatchFormProps, result: &MatchResult, language: Language) -> Html {
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
            </div>
        </article>
    }
}

fn match_header(props: &MatchFormProps, language: Language) -> Html {
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
