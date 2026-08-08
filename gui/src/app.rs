use yew::prelude::*;

use tabletennis_tournament::api_contract::{
    AuthenticatedUserView, AuthenticationView, TournamentSummaryView, TournamentView,
};
use tabletennis_tournament::application::TournamentApplication;
use tabletennis_tournament::tournament::TournamentState;

use crate::components::{RosterEditor, SubmittedResult};
use crate::formatting::match_format;
use crate::language::{Language, Text};
use crate::model::{CreateTournamentCommand, RosterEntryCommand};

mod authentication_view;
mod mutations;

pub struct App {
    pub(crate) application: Option<TournamentApplication>,
    pub(crate) tournament_record_id: Option<String>,
    pub(crate) tournament_revision: Option<u64>,
    authentication: AuthenticationState,
    tournaments: Vec<TournamentSummaryView>,
    error: Option<String>,
    busy: bool,
    pub(crate) initial_contestant_count: usize,
    roster_open: bool,
    close_roster_after_mutation: bool,
    dark_mode: bool,
    pub(crate) language: Language,
    pub(crate) development_tools_enabled: bool,
    pub(crate) simulation_run_seed: Option<u64>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
enum AuthenticationState {
    Loading,
    SignedOut,
    SignedIn(AuthenticatedUserView),
    Error(String),
}

pub enum Msg {
    SessionLoaded(Result<AuthenticationView, String>),
    TournamentListLoaded(Result<Vec<TournamentSummaryView>, String>),
    LoadTournament(String),
    NewTournament,
    CreateTournament(CreateTournamentCommand),
    StartTournament(Vec<RosterEntryCommand>),
    SaveRoster(Vec<RosterEntryCommand>),
    ToggleRoster,
    ToggleDarkMode,
    ToggleLanguage,
    CalculatePairings,
    PublishPairings,
    SubmitResult(SubmittedResult),
    SimulateRemainingResults,
    ExportSimulationTrace,
    CompleteRound,
    TournamentMutationFinished(Box<Result<TournamentView, String>>),
    Logout,
    LogoutFinished(Result<(), String>),
    DismissError,
}

impl Component for App {
    type Message = Msg;
    type Properties = ();

    fn create(context: &Context<Self>) -> Self {
        let development_tools_enabled = crate::route::current_route().development_tools_enabled();
        let language = crate::language::load_language();
        crate::language::apply_to_document(language);
        context
            .link()
            .send_future(async { Msg::SessionLoaded(crate::api_client::current_session().await) });
        Self {
            application: None,
            tournament_record_id: None,
            tournament_revision: None,
            authentication: AuthenticationState::Loading,
            tournaments: Vec::new(),
            error: None,
            busy: false,
            initial_contestant_count: 16,
            roster_open: false,
            close_roster_after_mutation: false,
            dark_mode: crate::theme::load_dark_mode(),
            language,
            development_tools_enabled,
            simulation_run_seed: None,
        }
    }

    fn update(&mut self, context: &Context<Self>, message: Self::Message) -> bool {
        match message {
            Msg::SessionLoaded(result) => self.session_loaded(context, result),
            Msg::TournamentListLoaded(result) => match result {
                Ok(tournaments) => self.tournaments = tournaments,
                Err(error) => self.error = Some(error),
            },
            Msg::LoadTournament(id) => {
                self.busy = true;
                context.link().send_future(async move {
                    Msg::TournamentMutationFinished(Box::new(
                        crate::api_client::load_tournament(&id).await,
                    ))
                });
            }
            Msg::NewTournament => {
                self.application = None;
                self.tournament_record_id = None;
                self.tournament_revision = None;
                self.error = None;
            }
            Msg::CreateTournament(command) => self.create_tournament(context, command),
            Msg::StartTournament(roster) => self.start_tournament(context, roster),
            Msg::SaveRoster(roster) => self.save_roster(context, roster),
            Msg::CalculatePairings => self
                .tournament_mutation(context, |id, revision| async move {
                    crate::api_client::calculate_pairings(&id, revision).await
                }),
            Msg::PublishPairings => self.tournament_mutation(context, |id, revision| async move {
                crate::api_client::publish_pairings(&id, revision).await
            }),
            Msg::CompleteRound => self.tournament_mutation(context, |id, revision| async move {
                crate::api_client::complete_round(&id, revision).await
            }),
            Msg::SubmitResult(submission) => self.submit_result(context, submission),
            Msg::SimulateRemainingResults => self.simulate_remaining(context),
            Msg::ExportSimulationTrace => self.export_simulation(),
            Msg::TournamentMutationFinished(result) => self.mutation_finished(*result),
            Msg::Logout => {
                self.busy = true;
                context
                    .link()
                    .send_future(async { Msg::LogoutFinished(crate::api_client::logout().await) });
            }
            Msg::LogoutFinished(result) => {
                self.busy = false;
                match result {
                    Ok(()) => {
                        self.authentication = AuthenticationState::SignedOut;
                        self.application = None;
                        self.tournament_record_id = None;
                        self.tournament_revision = None;
                        self.tournaments.clear();
                    }
                    Err(error) => self.error = Some(error),
                }
            }
            Msg::DismissError => self.error = None,
            Msg::ToggleRoster => self.roster_open = !self.roster_open,
            Msg::ToggleDarkMode => {
                self.dark_mode = !self.dark_mode;
                crate::theme::store_dark_mode(self.dark_mode);
            }
            Msg::ToggleLanguage => {
                self.language = self.language.toggled();
                crate::language::store_language(self.language);
                crate::language::apply_to_document(self.language);
                self.error = None;
            }
        }
        true
    }

    fn view(&self, context: &Context<Self>) -> Html {
        let language = self.language;
        html! {
            <ContextProvider<Language> context={language}>
            <div class={classes!("theme-root", self.dark_mode.then_some("dark-mode"), self.busy.then_some("api-busy"))}>
            <main class="app-shell">
                <header class="app-header">
                    <div>
                        <p class="eyebrow">{language.text(Text::LocalTournamentControl)}</p>
                        <h1>{language.text(Text::TableTennisTournament)}</h1>
                        if self.development_tools_enabled {
                            <span class="developer-mode-label">{language.text(Text::DeveloperSimulationMode)}</span>
                        }
                    </div>
                    <div class="header-actions">
                        {self.application.as_ref().map(|application| tournament_status(application, language)).unwrap_or_default()}
                        {self.roster_button(context)}
                        {self.simulation_export_button(context)}
                        {self.identity_controls(context)}
                        {self.theme_button(context)}
                        {self.language_button(context)}
                    </div>
                </header>
                {self.error_banner(context)}
                {self.roster_editor(context)}
                {self.content(context)}
            </main>
            </div>
            </ContextProvider<Language>>
        }
    }
}

impl App {
    fn theme_button(&self, context: &Context<Self>) -> Html {
        let toggle = context.link().callback(|_| Msg::ToggleDarkMode);
        html! {
            <button class="secondary compact theme-toggle" aria-pressed={self.dark_mode.to_string()} onclick={toggle}>
                {if self.dark_mode { self.language.text(Text::LightMode) } else { self.language.text(Text::DarkMode) }}
            </button>
        }
    }

    fn language_button(&self, context: &Context<Self>) -> Html {
        let toggle = context.link().callback(|_| Msg::ToggleLanguage);
        html! { <button class="secondary compact" onclick={toggle}>{self.language.toggle_label()}</button> }
    }

    fn roster_button(&self, context: &Context<Self>) -> Html {
        let Some(application) = &self.application else {
            return Html::default();
        };
        if application.tournament().state() != TournamentState::Started {
            return Html::default();
        }
        let toggle = context.link().callback(|_| Msg::ToggleRoster);
        html! {
            <button class="secondary compact" onclick={toggle}>
                {if self.roster_open { self.language.text(Text::CloseRoster) } else { self.language.text(Text::ManageContestants) }}
            </button>
        }
    }

    fn simulation_export_button(&self, context: &Context<Self>) -> Html {
        if !self.development_tools_enabled || self.application.is_none() {
            return Html::default();
        }
        let export = context.link().callback(|_| Msg::ExportSimulationTrace);
        html! { <button class="test-action compact" onclick={export}>{self.language.text(Text::ExportSimulationJson)}</button> }
    }

    fn roster_editor(&self, context: &Context<Self>) -> Html {
        if !self.roster_open {
            return Html::default();
        }
        let Some(application) = &self.application else {
            return Html::default();
        };
        html! {
            <RosterEditor
                entrants={application.active_entrants().cloned().collect::<Vec<_>>()}
                on_save={context.link().callback(Msg::SaveRoster)}
                on_cancel={context.link().callback(|()| Msg::ToggleRoster)}
            />
        }
    }

    fn error_banner(&self, context: &Context<Self>) -> Html {
        self.error.as_ref().map(|error| {
            let dismiss = context.link().callback(|_| Msg::DismissError);
            html! {
                <div class="error-banner" role="alert">
                    <span>{error}</span>
                    <button aria-label={self.language.text(Text::DismissError)} onclick={dismiss}>{"×"}</button>
                </div>
            }
        }).unwrap_or_default()
    }
}

fn tournament_status(application: &TournamentApplication, language: Language) -> Html {
    html! {
        <div class="tournament-status">
            <strong>{application.tournament().id().as_str()}</strong>
            <span>{language.tournament_status(
                match_format(application.tournament().match_format(), language),
                application.tournament().table_count().value(),
                application.tournament().maximum_round_count().value(),
            )}</span>
        </div>
    }
}
