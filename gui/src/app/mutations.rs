use tabletennis_tournament::api_contract::{
    AuthenticationView, CreateTournamentRequest, TournamentView,
};
use yew::prelude::*;

use super::{App, AuthenticationState, Msg};
use crate::app_actions::result_request;
use crate::components::SubmittedResult;
use crate::model::{CreateTournamentCommand, RosterEntryCommand};

impl App {
    pub(super) fn session_loaded(
        &mut self,
        context: &Context<Self>,
        result: Result<AuthenticationView, String>,
    ) {
        match result {
            Ok(view) if view.authenticated => {
                if let Some(user) = view.user {
                    self.authentication = AuthenticationState::SignedIn(user);
                    context.link().send_future(async {
                        Msg::TournamentListLoaded(crate::api_client::list_tournaments().await)
                    });
                } else {
                    self.authentication = AuthenticationState::Error(
                        "The server returned an incomplete authentication response.".to_owned(),
                    );
                }
            }
            Ok(_) => self.authentication = AuthenticationState::SignedOut,
            Err(error) => self.authentication = AuthenticationState::Error(error),
        }
    }

    pub(super) fn create_tournament(
        &mut self,
        context: &Context<Self>,
        command: CreateTournamentCommand,
    ) {
        if !(2..=64).contains(&command.contestant_count) {
            self.error = Some(self.language.contestant_range_error().to_owned());
            return;
        }
        self.initial_contestant_count = command.contestant_count;
        self.simulation_run_seed = self
            .development_tools_enabled
            .then(crate::simulation_seed::fresh_simulation_seed);
        self.busy = true;
        let request = CreateTournamentRequest {
            title: command.tournament_id,
            match_format: command.match_format,
            table_count: command.table_count,
            maximum_round_count: command.maximum_round_count,
        };
        context.link().send_future(async move {
            Msg::TournamentMutationFinished(Box::new(
                crate::api_client::create_tournament(&request).await,
            ))
        });
    }

    pub(super) fn start_tournament(
        &mut self,
        context: &Context<Self>,
        roster: Vec<RosterEntryCommand>,
    ) {
        let Ok(request) = self
            .roster_request(roster)
            .inspect_err(|error| self.error = Some(error.clone()))
        else {
            return;
        };
        let Some(id) = self.tournament_record_id.clone() else {
            self.error = Some(self.language.create_tournament_first_error().to_owned());
            return;
        };
        self.busy = true;
        context.link().send_future(async move {
            let result = async {
                let roster_view = crate::api_client::replace_roster(&id, &request).await?;
                crate::api_client::start_tournament(&id, roster_view.revision).await
            }
            .await;
            Msg::TournamentMutationFinished(Box::new(result))
        });
    }

    pub(super) fn save_roster(&mut self, context: &Context<Self>, roster: Vec<RosterEntryCommand>) {
        let Ok(request) = self
            .roster_request(roster)
            .inspect_err(|error| self.error = Some(error.clone()))
        else {
            return;
        };
        let Some(id) = self.tournament_record_id.clone() else {
            self.error = Some(self.language.create_tournament_first_error().to_owned());
            return;
        };
        self.close_roster_after_mutation = true;
        self.busy = true;
        context.link().send_future(async move {
            Msg::TournamentMutationFinished(Box::new(
                crate::api_client::replace_roster(&id, &request).await,
            ))
        });
    }

    pub(super) fn submit_result(&mut self, context: &Context<Self>, submission: SubmittedResult) {
        let Some(id) = self.tournament_record_id.clone() else {
            self.error = Some(self.language.create_tournament_first_error().to_owned());
            return;
        };
        let match_id = submission.match_id.as_str().to_owned();
        let request = result_request(&submission);
        self.busy = true;
        context.link().send_future(async move {
            Msg::TournamentMutationFinished(Box::new(
                crate::api_client::record_result(&id, &match_id, &request).await,
            ))
        });
    }

    pub(super) fn simulate_remaining(&mut self, context: &Context<Self>) {
        if !self.development_tools_enabled {
            self.error = Some(self.language.simulation_route_error().to_owned());
            return;
        }
        let Some(id) = self.tournament_record_id.clone() else {
            self.error = Some(self.language.create_tournament_first_error().to_owned());
            return;
        };
        let Some(seed) = self.simulation_run_seed else {
            self.error = Some(self.language.simulation_seed_error().to_owned());
            return;
        };
        let Some(_application) = self.application.as_ref() else {
            return;
        };
        self.busy = true;
        context.link().send_future(async move {
            Msg::TournamentMutationFinished(Box::new(
                crate::app_actions::simulate_remaining_results(&id, seed).await,
            ))
        });
    }

    pub(super) fn export_simulation(&mut self) {
        if !self.development_tools_enabled {
            self.error = Some(self.language.simulation_route_error().to_owned());
            return;
        }
        let result = self
            .application
            .as_ref()
            .zip(self.simulation_run_seed)
            .ok_or_else(|| self.language.simulation_seed_error().to_owned())
            .and_then(|(application, seed)| {
                crate::developer_export::download_simulation_json(application, seed)
                    .map_err(|error| self.language.simulation_export_error(&error))
            });
        if let Err(error) = result {
            self.error = Some(error);
        }
    }

    pub(super) fn tournament_mutation<F, Future>(&mut self, context: &Context<Self>, operation: F)
    where
        F: FnOnce(String, u64) -> Future + 'static,
        Future: std::future::Future<Output = Result<TournamentView, String>> + 'static,
    {
        let Some(id) = self.tournament_record_id.clone() else {
            self.error = Some(self.language.create_tournament_first_error().to_owned());
            return;
        };
        let Some(revision) = self.tournament_revision else {
            self.error = Some(self.language.create_tournament_first_error().to_owned());
            return;
        };
        self.busy = true;
        context.link().send_future(async move {
            Msg::TournamentMutationFinished(Box::new(operation(id, revision).await))
        });
    }

    pub(super) fn mutation_finished(&mut self, result: Result<TournamentView, String>) {
        self.busy = false;
        match result.and_then(|view| self.install_tournament_view(view)) {
            Ok(()) => {
                self.error = None;
                if self.close_roster_after_mutation {
                    self.roster_open = false;
                }
            }
            Err(error) => self.error = Some(error),
        }
        self.close_roster_after_mutation = false;
    }
}
