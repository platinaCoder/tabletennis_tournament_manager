use yew::prelude::*;

use tabletennis_tournament::application::{ContestantStanding, TournamentEntrant};
use tabletennis_tournament::identity::EntrantId;

use crate::language::{Text, use_language};

use super::standings::Standings;

#[derive(Properties, PartialEq)]
pub struct RoundDashboardProps {
    pub completed_round_count: usize,
    pub entrants: Vec<TournamentEntrant>,
    pub standings: Vec<ContestantStanding>,
    pub active_entrant_ids: Vec<EntrantId>,
    pub can_edit: bool,
    pub on_calculate: Callback<()>,
}

#[component]
pub fn RoundDashboard(props: &RoundDashboardProps) -> Html {
    let language = use_language();
    let on_calculate = {
        let callback = props.on_calculate.clone();
        Callback::from(move |_| callback.emit(()))
    };
    let next_round = props.completed_round_count + 1;

    html! {
        <section class="panel">
            <div class="section-heading">
                <div>
                    <p class="eyebrow">{language.after_completed_rounds(props.completed_round_count)}</p>
                    <h2>{language.text(Text::TournamentStandings)}</h2>
                </div>
                if props.can_edit {
                    <button class="primary" onclick={on_calculate}>
                        {language.calculate_round(next_round)}
                    </button>
                }
            </div>
            <Standings
                standings={props.standings.clone()}
                entrants={props.entrants.clone()}
                active_entrant_ids={props.active_entrant_ids.clone()}
            />
        </section>
    }
}
