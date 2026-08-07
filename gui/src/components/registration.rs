use yew::prelude::*;

use tabletennis_tournament::results::MatchFormat;

use crate::formatting::match_format;
use crate::model::RosterEntryCommand;

use super::roster_form::RosterForm;

#[derive(Properties, PartialEq)]
pub struct RegistrationProps {
    pub contestant_count: usize,
    pub match_format: MatchFormat,
    pub table_count: u16,
    pub maximum_round_count: u16,
    pub allow_simulation: bool,
    pub on_start: Callback<Vec<RosterEntryCommand>>,
}

#[component]
pub fn Registration(props: &RegistrationProps) -> Html {
    html! {
        <section class="panel roster-panel">
            <div class="section-heading">
                <div>
                    <p class="eyebrow">{"Registration"}</p>
                    <h2>{"Enter the contestant roster"}</h2>
                </div>
                <div class="summary-strip">
                    <span>{match_format(props.match_format)}</span>
                    <span>{format!("{} tables", props.table_count)}</span>
                    <span>{format!("{} rounds", props.maximum_round_count)}</span>
                </div>
            </div>
            <p class="muted">
                {"Names, clubs, and starting ELOs can still be edited after the tournament starts."}
            </p>
            <RosterForm
                entrants={Vec::new()}
                initial_row_count={props.contestant_count}
                allow_simulation={props.allow_simulation}
                submit_label={"Start tournament"}
                on_submit={props.on_start.clone()}
            />
        </section>
    }
}
