use yew::prelude::*;

use tabletennis_tournament::application::TournamentEntrant;

use crate::language::{Text, use_language};
use crate::model::RosterEntryCommand;

use super::roster_form::RosterForm;

#[derive(Properties, PartialEq)]
pub struct RosterEditorProps {
    pub entrants: Vec<TournamentEntrant>,
    pub on_save: Callback<Vec<RosterEntryCommand>>,
    pub on_cancel: Callback<()>,
}

#[component]
pub fn RosterEditor(props: &RosterEditorProps) -> Html {
    let language = use_language();
    let cancel = {
        let callback = props.on_cancel.clone();
        Callback::from(move |_| callback.emit(()))
    };
    html! {
        <section class="panel roster-panel active-roster-editor">
            <div class="section-heading">
                <div>
                    <p class="eyebrow">{language.text(Text::ActiveRoster)}</p>
                    <h2>{language.text(Text::EditContestants)}</h2>
                </div>
                <button type="button" class="secondary" onclick={cancel}>{language.text(Text::Cancel)}</button>
            </div>
            <p class="muted">
                {language.roster_withdrawal_explanation()}
            </p>
            <RosterForm
                entrants={props.entrants.clone()}
                initial_row_count={0}
                allow_simulation={false}
                submit_label={language.text(Text::SaveRoster)}
                on_submit={props.on_save.clone()}
            />
        </section>
    }
}
