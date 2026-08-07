use yew::prelude::*;

use tabletennis_tournament::application::TournamentEntrant;

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
    let cancel = {
        let callback = props.on_cancel.clone();
        Callback::from(move |_| callback.emit(()))
    };
    html! {
        <section class="panel roster-panel active-roster-editor">
            <div class="section-heading">
                <div>
                    <p class="eyebrow">{"Active roster"}</p>
                    <h2>{"Edit contestants"}</h2>
                </div>
                <button type="button" class="secondary" onclick={cancel}>{"Cancel"}</button>
            </div>
            <p class="muted">
                {"Deleted contestants are withdrawn from future rounds. Published matches and historical standings remain unchanged."}
            </p>
            <RosterForm
                entrants={props.entrants.clone()}
                initial_row_count={0}
                allow_simulation={false}
                submit_label={"Save roster"}
                on_submit={props.on_save.clone()}
            />
        </section>
    }
}
