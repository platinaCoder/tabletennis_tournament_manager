use web_sys::{HtmlInputElement, HtmlSelectElement};
use yew::prelude::*;

use tabletennis_tournament::results::MatchFormat;

use crate::model::CreateTournamentCommand;

#[derive(Properties, PartialEq)]
pub struct TournamentSetupProps {
    pub on_create: Callback<CreateTournamentCommand>,
}

#[component]
pub fn TournamentSetup(props: &TournamentSetupProps) -> Html {
    let tournament_id = use_state(|| "local-tournament".to_owned());
    let table_count = use_state(|| "8".to_owned());
    let contestant_count = use_state(|| "16".to_owned());
    let maximum_round_count = use_state(|| "5".to_owned());
    let match_format = use_state(|| MatchFormat::BestOfFive);

    let onsubmit = {
        let tournament_id = tournament_id.clone();
        let table_count = table_count.clone();
        let contestant_count = contestant_count.clone();
        let maximum_round_count = maximum_round_count.clone();
        let match_format = match_format.clone();
        let on_create = props.on_create.clone();
        Callback::from(move |event: SubmitEvent| {
            event.prevent_default();
            on_create.emit(CreateTournamentCommand {
                tournament_id: (*tournament_id).clone(),
                match_format: *match_format,
                table_count: table_count.parse().unwrap_or(0),
                contestant_count: contestant_count.parse().unwrap_or(0),
                maximum_round_count: maximum_round_count.parse().unwrap_or(0),
            });
        })
    };
    let on_id = text_input(tournament_id.clone());
    let on_tables = text_input(table_count.clone());
    let on_contestants = text_input(contestant_count.clone());
    let on_rounds = text_input(maximum_round_count.clone());
    let on_format = {
        let match_format = match_format.clone();
        Callback::from(move |event: Event| {
            let select = event.target_unchecked_into::<HtmlSelectElement>();
            match_format.set(if select.value() == "best_of_three" {
                MatchFormat::BestOfThree
            } else {
                MatchFormat::BestOfFive
            });
        })
    };

    html! {
        <section class="panel setup-panel">
            <p class="eyebrow">{"Tournament setup"}</p>
            <h2>{"Create a tournament"}</h2>
            <p class="muted">
                {"The match format, table count, and maximum rounds become fixed when play starts."}
            </p>
            <form {onsubmit} class="form-grid">
                <label>
                    <span>{"Tournament identifier"}</span>
                    <input required=true value={(*tournament_id).clone()} oninput={on_id} />
                </label>
                <label>
                    <span>{"Match format"}</span>
                    <select onchange={on_format}>
                        <option value="best_of_five" selected=true>{"Best of five"}</option>
                        <option value="best_of_three">{"Best of three"}</option>
                    </select>
                </label>
                <label>
                    <span>{"Available tables"}</span>
                    <input required=true type="number" min="1" value={(*table_count).clone()} oninput={on_tables} />
                </label>
                <label>
                    <span>{"Contestant count"}</span>
                    <input required=true type="number" min="2" max="64" value={(*contestant_count).clone()} oninput={on_contestants} />
                </label>
                <label>
                    <span>{"Maximum rounds"}</span>
                    <input required=true type="number" min="1" max="65535" value={(*maximum_round_count).clone()} oninput={on_rounds} />
                </label>
                <button class="primary" type="submit">{"Create tournament"}</button>
            </form>
        </section>
    }
}

fn text_input(state: UseStateHandle<String>) -> Callback<InputEvent> {
    Callback::from(move |event: InputEvent| {
        state.set(event.target_unchecked_into::<HtmlInputElement>().value());
    })
}
