use tabletennis_tournament::api_contract::{
    ReceivedTournamentInvitationView, TournamentInvitationDecisionView,
};
use yew::prelude::*;

use super::{App, Msg};

impl App {
    pub(super) fn refresh_dashboard(&self, context: &Context<Self>) {
        self.refresh_tournaments(context);
        context.link().send_future(async {
            Msg::InvitationListLoaded(crate::api_client::list_received_invitations().await)
        });
    }

    pub(super) fn decide_invitation(
        &mut self,
        context: &Context<Self>,
        invitation_id: String,
        accept: bool,
    ) {
        self.busy = true;
        context.link().send_future(async move {
            let result = if accept {
                crate::api_client::accept_invitation(&invitation_id).await
            } else {
                crate::api_client::decline_invitation(&invitation_id).await
            };
            Msg::InvitationDecisionFinished(result)
        });
    }

    pub(super) fn invitation_list_loaded(
        &mut self,
        result: Result<Vec<ReceivedTournamentInvitationView>, String>,
    ) {
        match result {
            Ok(invitations) => self.received_invitations = invitations,
            Err(error) => self.error = Some(error),
        }
    }

    pub(super) fn invitation_decision_finished(
        &mut self,
        context: &Context<Self>,
        result: Result<TournamentInvitationDecisionView, String>,
    ) {
        self.busy = false;
        match result {
            Ok(_) => {
                self.error = None;
                self.refresh_dashboard(context);
            }
            Err(error) => self.error = Some(error),
        }
    }
}
