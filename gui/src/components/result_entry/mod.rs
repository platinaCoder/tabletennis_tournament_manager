mod form_state;
mod match_form;

use std::collections::{HashMap, HashSet};

use yew::prelude::*;

use tabletennis_tournament::application::{ActiveRound, TournamentEntrant};
use tabletennis_tournament::identity::MatchId;
use tabletennis_tournament::results::{GameScore, MatchFormat};

use match_form::MatchForm;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SubmittedResult {
    pub match_id: MatchId,
    pub games: Vec<GameScore>,
}

#[derive(Properties, PartialEq)]
pub struct ResultEntryProps {
    pub round: ActiveRound,
    pub entrants: Vec<TournamentEntrant>,
    pub match_format: MatchFormat,
    pub allow_simulation: bool,
    pub on_submit: Callback<SubmittedResult>,
    pub on_simulate_remaining: Callback<()>,
    pub on_complete_round: Callback<()>,
}

#[component]
pub fn ResultEntry(props: &ResultEntryProps) -> Html {
    let entrants = props
        .entrants
        .iter()
        .map(|entrant| (&entrant.entrant_id, entrant))
        .collect::<HashMap<_, _>>();
    let results = props
        .round
        .results
        .iter()
        .map(|result| (result.match_id(), result))
        .collect::<HashMap<_, _>>();
    let completed_ids = results
        .keys()
        .map(|id| (*id).clone())
        .collect::<HashSet<_>>();
    let first_pending = props
        .round
        .scheduled_matches
        .iter()
        .find(|scheduled| {
            scheduled.table_number().is_some() && !completed_ids.contains(&scheduled.match_id)
        })
        .map(|scheduled| scheduled.match_id.clone());
    let complete = results.len() == props.round.scheduled_matches.len();
    let on_complete = {
        let callback = props.on_complete_round.clone();
        Callback::from(move |_| callback.emit(()))
    };
    let on_simulate = {
        let callback = props.on_simulate_remaining.clone();
        Callback::from(move |_| callback.emit(()))
    };

    html! {
        <section class="panel result-entry-panel">
            <div class="section-heading">
                <div>
                    <p class="eyebrow">{format!("Round {} · result entry", props.round.round_number.value())}</p>
                    <h2>{format!("{} of {} matches complete", results.len(), props.round.scheduled_matches.len())}</h2>
                </div>
                <div class="button-row">
                    if props.allow_simulation {
                        <button class="test-action" disabled={complete} onclick={on_simulate}>
                            {"Simulate remaining games"}
                        </button>
                    }
                    <button class="primary" disabled={!complete} onclick={on_complete}>
                        {"Complete round"}
                    </button>
                </div>
            </div>
            <p class="keyboard-hint">{"Keyboard: home score → Tab → away score → Tab → next game. Press Enter to save once the match is complete."}</p>
            {props.round.bye.as_ref().map(|bye| {
                let name = entrants
                    .get(bye)
                    .map_or("Unknown contestant", |entrant| entrant.name.as_str());
                html! { <p class="bye-notice"><strong>{name}</strong>{" has the bye this round."}</p> }
            }).unwrap_or_default()}
            <div class="result-grid">
                {for props.round.scheduled_matches.iter().map(|scheduled| {
                    let home = entrants.get(&scheduled.home_entrant_id).map(|entrant| (*entrant).clone());
                    let away = entrants.get(&scheduled.away_entrant_id).map(|entrant| (*entrant).clone());
                    let existing_result = results.get(&scheduled.match_id).map(|result| (*result).clone());
                    html! {
                        <MatchForm
                            key={scheduled.match_id.as_str().to_owned()}
                            scheduled_match={scheduled.clone()}
                            {home}
                            {away}
                            match_format={props.match_format}
                            {existing_result}
                            autofocus={first_pending.as_ref() == Some(&scheduled.match_id)}
                            on_submit={props.on_submit.clone()}
                        />
                    }
                })}
            </div>
        </section>
    }
}
