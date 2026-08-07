use std::collections::BTreeSet;

use web_sys::HtmlInputElement;
use yew::prelude::*;

use tabletennis_tournament::application::TournamentEntrant;

use super::roster_state::{RosterRow, blank_row, initial_rows, simulated_rows};
use crate::model::RosterEntryCommand;

#[derive(Clone, Copy)]
enum RosterField {
    Name,
    Club,
    Elo,
}

#[derive(Properties, PartialEq)]
pub struct RosterFormProps {
    pub entrants: Vec<TournamentEntrant>,
    pub initial_row_count: usize,
    pub allow_simulation: bool,
    pub submit_label: AttrValue,
    pub on_submit: Callback<Vec<RosterEntryCommand>>,
}

#[component]
pub fn RosterForm(props: &RosterFormProps) -> Html {
    let rows = use_state(|| initial_rows(&props.entrants, props.initial_row_count));
    let club_suggestions = rows
        .iter()
        .filter_map(|row| {
            let club = row.club_name.trim();
            (!club.is_empty()).then(|| club.to_owned())
        })
        .collect::<BTreeSet<_>>();
    let onsubmit = submit_roster(rows.clone(), props.on_submit.clone());
    let add_row = {
        let rows = rows.clone();
        Callback::from(move |_| {
            let mut replacement = (*rows).clone();
            let key = replacement.iter().map(|row| row.key).max().unwrap_or(0) + 1;
            replacement.push(blank_row(key));
            rows.set(replacement);
        })
    };
    let simulate = {
        let rows = rows.clone();
        Callback::from(move |_| rows.set(simulated_rows(rows.len())))
    };

    html! {
        <form {onsubmit}>
            <datalist id="club-suggestions">
                {for club_suggestions.iter().map(|club| html! { <option value={club.clone()} /> })}
            </datalist>
            <div class="roster-list">
                <div class="roster-row roster-header" aria-hidden="true">
                    <span>{"Contestant"}</span>
                    <span>{"Club"}</span>
                    <span>{"Starting ELO"}</span>
                    <span></span>
                </div>
                {for rows.iter().enumerate().map(|(index, row)| roster_row(rows.clone(), row, index))}
            </div>
            <div class="roster-actions">
                <button type="button" class="secondary" onclick={add_row}>{"+ Add contestant"}</button>
                if props.allow_simulation {
                    <button type="button" class="test-action" onclick={simulate}>
                        {"Fill with test contestants"}
                    </button>
                }
                <span class="muted">{format!("{} contestants", rows.len())}</span>
                <button type="submit" class="primary" disabled={rows.len() < 2}>
                    {props.submit_label.clone()}
                </button>
            </div>
        </form>
    }
}

fn roster_row(rows: UseStateHandle<Vec<RosterRow>>, row: &RosterRow, index: usize) -> Html {
    let key = row.key;
    let remove = {
        let rows = rows.clone();
        Callback::from(move |_| {
            let mut replacement = (*rows).clone();
            replacement.retain(|row| row.key != key);
            rows.set(replacement);
        })
    };
    html! {
        <div class="roster-row" key={key}>
            <label>
                <span class="mobile-label">{format!("Contestant {}", index + 1)}</span>
                <input
                    required=true
                    placeholder={format!("Contestant {} name", index + 1)}
                    value={row.name.clone()}
                    oninput={update_field(rows.clone(), key, RosterField::Name)}
                />
            </label>
            <label>
                <span class="mobile-label">{"Club"}</span>
                <input
                    required=true
                    list="club-suggestions"
                    placeholder="Start typing a club"
                    value={row.club_name.clone()}
                    oninput={update_field(rows.clone(), key, RosterField::Club)}
                />
            </label>
            <label>
                <span class="mobile-label">{"Starting ELO"}</span>
                <input
                    required=true
                    type="number"
                    min="0"
                    value={row.elo.clone()}
                    oninput={update_field(rows.clone(), key, RosterField::Elo)}
                />
            </label>
            <button
                type="button"
                class="danger-link"
                aria-label={format!("Delete contestant {}", index + 1)}
                onclick={remove}
            >
                {"Delete"}
            </button>
        </div>
    }
}

fn update_field(
    rows: UseStateHandle<Vec<RosterRow>>,
    key: usize,
    field: RosterField,
) -> Callback<InputEvent> {
    Callback::from(move |event: InputEvent| {
        let value = event.target_unchecked_into::<HtmlInputElement>().value();
        let mut replacement = (*rows).clone();
        if let Some(row) = replacement.iter_mut().find(|row| row.key == key) {
            match field {
                RosterField::Name => row.name = value,
                RosterField::Club => row.club_name = value,
                RosterField::Elo => row.elo = value,
            }
        }
        rows.set(replacement);
    })
}

fn submit_roster(
    rows: UseStateHandle<Vec<RosterRow>>,
    callback: Callback<Vec<RosterEntryCommand>>,
) -> Callback<SubmitEvent> {
    Callback::from(move |event: SubmitEvent| {
        event.prevent_default();
        callback.emit(
            rows.iter()
                .map(|row| RosterEntryCommand {
                    entrant_id: row.entrant_id.clone(),
                    name: row.name.trim().to_owned(),
                    club_name: row.club_name.trim().to_owned(),
                    starting_elo: row.elo.parse().unwrap_or(0),
                })
                .collect(),
        );
    })
}

#[cfg(test)]
#[path = "roster_form_tests.rs"]
mod tests;
