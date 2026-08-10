use web_sys::HtmlInputElement;
use yew::prelude::*;

use tabletennis_tournament::application::TournamentEntrant;
use tabletennis_tournament::results::{MatchFormat, MatchProgress, MatchResult};
use tabletennis_tournament::scheduling::ScheduledMatch;

use crate::language::{Text, use_language};

use super::SubmittedResult;
use super::correction_controls::{cancel_correction, cancel_label, result_rows, save_label};
use super::form_state::{GameInput, evaluate_rows};
use super::match_display::{completed_match, match_header, progress_label, read_only_result_label};

#[derive(Properties, PartialEq)]
pub struct MatchFormProps {
    pub scheduled_match: ScheduledMatch,
    pub home: Option<TournamentEntrant>,
    pub away: Option<TournamentEntrant>,
    pub match_format: MatchFormat,
    pub can_edit: bool,
    pub existing_result: Option<MatchResult>,
    pub autofocus: bool,
    pub on_submit: Callback<SubmittedResult>,
}

#[component]
pub fn MatchForm(props: &MatchFormProps) -> Html {
    let language = use_language();
    let rows = use_state(|| result_rows(props.match_format, props.existing_result.as_ref()));
    let correcting = use_state(|| false);
    let first_home_input = use_node_ref();
    {
        let first_home_input = first_home_input.clone();
        let should_focus = *correcting
            || (props.autofocus
                && props.existing_result.is_none()
                && props.scheduled_match.table_number().is_some());
        use_effect_with(should_focus, move |should_focus| {
            if *should_focus && let Some(input) = first_home_input.cast::<HtmlInputElement>() {
                let _ = input.focus();
            }
        });
    }
    if let Some(result) = &props.existing_result
        && !*correcting
    {
        let begin_correction = {
            let correcting = correcting.clone();
            Callback::from(move |_| correcting.set(true))
        };
        return completed_match(props, result, language, begin_correction);
    }
    if !props.can_edit {
        return html! {
            <article class="match-card waiting-card">
                {match_header(props, language)}
                <p class="muted">{read_only_result_label(language)}</p>
            </article>
        };
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
    let expected_revision = props
        .existing_result
        .as_ref()
        .map_or(0, |result| u64::from(result.revision().value()));
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
                    expected_revision,
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
                    <div class="button-row">
                        if props.existing_result.is_some() {
                            <button
                                class="secondary compact"
                                type="button"
                                onclick={cancel_correction(
                                    correcting.clone(),
                                    rows.clone(),
                                    props.match_format,
                                    props.existing_result.clone(),
                                )}
                            >{cancel_label(language)}</button>
                        }
                        <button class="primary compact" type="submit" disabled={!is_complete}>
                            {if props.existing_result.is_some() {
                                save_label(language)
                            } else {
                                language.text(Text::SaveResult)
                            }}
                        </button>
                    </div>
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
