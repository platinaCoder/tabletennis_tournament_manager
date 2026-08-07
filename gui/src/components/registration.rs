use yew::prelude::*;

use tabletennis_tournament::results::MatchFormat;

use crate::formatting::match_format;
use crate::language::{Text, use_language};
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
    let language = use_language();
    html! {
        <section class="panel roster-panel">
            <div class="section-heading">
                <div>
                    <p class="eyebrow">{language.text(Text::Registration)}</p>
                    <h2>{language.text(Text::EnterRoster)}</h2>
                </div>
                <div class="summary-strip">
                    <span>{match_format(props.match_format, language)}</span>
                    <span>{language.table_count(props.table_count)}</span>
                    <span>{language.round_count(usize::from(props.maximum_round_count))}</span>
                </div>
            </div>
            <p class="muted">
                {language.roster_edit_explanation()}
            </p>
            <RosterForm
                entrants={Vec::new()}
                initial_row_count={props.contestant_count}
                allow_simulation={props.allow_simulation}
                submit_label={language.text(Text::StartTournament)}
                on_submit={props.on_start.clone()}
            />
        </section>
    }
}
