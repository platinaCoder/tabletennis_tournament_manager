use yew::prelude::*;

use tabletennis_tournament::application::TournamentApplication;
use tabletennis_tournament::tournament::TournamentState;

use crate::components::{Registration, RosterEditor, SubmittedResult, TournamentSetup};
use crate::formatting::match_format;
use crate::language::{Language, Text};
use crate::model::{CreateTournamentCommand, RosterEntryCommand};

pub struct App {
    pub(crate) application: Option<TournamentApplication>,
    error: Option<String>,
    pub(crate) initial_contestant_count: usize,
    pub(crate) next_entrant_number: u64,
    pub(crate) next_club_number: u64,
    roster_open: bool,
    dark_mode: bool,
    pub(crate) language: Language,
    pub(crate) development_tools_enabled: bool,
    pub(crate) simulation_run_seed: Option<u64>,
}

pub enum Msg {
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
    DismissError,
}

impl Component for App {
    type Message = Msg;
    type Properties = ();

    fn create(_context: &Context<Self>) -> Self {
        let development_tools_enabled = crate::route::current_route().development_tools_enabled();
        let language = crate::language::load_language();
        crate::language::apply_to_document(language);
        Self {
            application: None,
            error: None,
            initial_contestant_count: 0,
            next_entrant_number: 1,
            next_club_number: 1,
            roster_open: false,
            dark_mode: crate::theme::load_dark_mode(),
            language,
            development_tools_enabled,
            simulation_run_seed: None,
        }
    }

    fn update(&mut self, _context: &Context<Self>, message: Self::Message) -> bool {
        if matches!(message, Msg::DismissError) {
            self.error = None;
            return true;
        }
        if matches!(message, Msg::ToggleRoster) {
            self.roster_open = !self.roster_open;
            return true;
        }
        if matches!(message, Msg::ToggleDarkMode) {
            self.dark_mode = !self.dark_mode;
            crate::theme::store_dark_mode(self.dark_mode);
            return true;
        }
        if matches!(message, Msg::ToggleLanguage) {
            self.language = self.language.toggled();
            crate::language::store_language(self.language);
            crate::language::apply_to_document(self.language);
            self.error = None;
            return true;
        }
        match self.handle(message) {
            Ok(RosterAction::CloseEditor) => {
                self.roster_open = false;
                self.error = None;
            }
            Ok(RosterAction::None) => self.error = None,
            Err(error) => self.error = Some(error),
        }
        true
    }

    fn view(&self, context: &Context<Self>) -> Html {
        let content = match &self.application {
            None => html! {
                <TournamentSetup on_create={context.link().callback(Msg::CreateTournament)} />
            },
            Some(application) if application.tournament().state() == TournamentState::Draft => {
                html! {
                    <Registration
                        contestant_count={self.initial_contestant_count}
                        match_format={application.tournament().match_format()}
                        table_count={application.tournament().table_count().value()}
                        maximum_round_count={application.tournament().maximum_round_count().value()}
                        allow_simulation={self.development_tools_enabled}
                        on_start={context.link().callback(Msg::StartTournament)}
                    />
                }
            }
            Some(application) => self.started_view(context, application),
        };

        let language = self.language;
        html! {
            <ContextProvider<Language> context={language}>
            <div class={classes!("theme-root", self.dark_mode.then_some("dark-mode"))}>
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
                        {self.theme_button(context)}
                        {self.language_button(context)}
                    </div>
                </header>
                {self.error.as_ref().map(|error| {
                    let dismiss = context.link().callback(|_| Msg::DismissError);
                    html! {
                        <div class="error-banner" role="alert">
                            <span>{error}</span>
                            <button aria-label={language.text(Text::DismissError)} onclick={dismiss}>{"×"}</button>
                        </div>
                    }
                }).unwrap_or_default()}
                {self.roster_editor(context)}
                {content}
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
            <button
                class="secondary compact theme-toggle"
                aria-pressed={self.dark_mode.to_string()}
                onclick={toggle}
            >
                {if self.dark_mode { self.language.text(Text::LightMode) } else { self.language.text(Text::DarkMode) }}
            </button>
        }
    }

    fn language_button(&self, context: &Context<Self>) -> Html {
        let toggle = context.link().callback(|_| Msg::ToggleLanguage);
        html! {
            <button class="secondary compact" onclick={toggle}>
                {self.language.toggle_label()}
            </button>
        }
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
        html! {
            <button class="test-action compact" onclick={export}>
                {self.language.text(Text::ExportSimulationJson)}
            </button>
        }
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

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum RosterAction {
    None,
    CloseEditor,
}
