use yew::prelude::*;

use tabletennis_tournament::application::TournamentApplication;

use crate::app::{App, Msg};
use crate::components::{PairingReview, ResultEntry, RoundDashboard, TournamentReport};

impl App {
    pub(crate) fn started_view(
        &self,
        context: &Context<Self>,
        application: &TournamentApplication,
    ) -> Html {
        if let Some(proposal) = application.pending_pairing() {
            return html! {
                <PairingReview
                    proposal={proposal.clone()}
                    entrants={application.entrants().to_vec()}
                    can_edit={self.can_edit_tournament()}
                    on_publish={context.link().callback(|()| Msg::PublishPairings)}
                    on_recalculate={context.link().callback(|()| Msg::CalculatePairings)}
                />
            };
        }
        if let Some(round) = application.active_round() {
            return html! {
                <ResultEntry
                    round={round.clone()}
                    entrants={application.entrants().to_vec()}
                    match_format={application.tournament().match_format()}
                    can_edit={self.can_edit_tournament()}
                    allow_simulation={self.development_tools_enabled}
                    on_submit={context.link().callback(Msg::SubmitResult)}
                    on_simulate_remaining={context.link().callback(|()| Msg::SimulateRemainingResults)}
                    on_complete_round={context.link().callback(|()| Msg::CompleteRound)}
                />
            };
        }
        if application.completed_rounds().len()
            >= usize::from(application.tournament().maximum_round_count().value())
        {
            return html! {
                <TournamentReport
                    rounds={application.completed_rounds().to_vec()}
                    entrants={application.entrants().to_vec()}
                    standings={application.standings().to_vec()}
                    active_entrant_ids={application.active_entrants().map(|entrant| entrant.entrant_id.clone()).collect::<Vec<_>>()}
                />
            };
        }
        html! {
            <RoundDashboard
                completed_round_count={application.completed_rounds().len()}
                entrants={application.entrants().to_vec()}
                standings={application.standings().to_vec()}
                active_entrant_ids={application.active_entrants().map(|entrant| entrant.entrant_id.clone()).collect::<Vec<_>>()}
                can_edit={self.can_edit_tournament()}
                on_calculate={context.link().callback(|()| Msg::CalculatePairings)}
            />
        }
    }
}
