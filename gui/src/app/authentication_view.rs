use tabletennis_tournament::tournament::TournamentState;
use yew::prelude::*;

use super::{App, AuthenticationState, Msg};
use crate::components::{Registration, TournamentSetup};
use crate::language::Language;

impl App {
    pub(super) fn content(&self, context: &Context<Self>) -> Html {
        match &self.authentication {
            AuthenticationState::Loading => {
                html! { <section class="panel"><p>{auth_loading(self.language)}</p></section> }
            }
            AuthenticationState::SignedOut => self.signed_out_view(),
            AuthenticationState::Error(error) => html! {
                <section class="panel auth-panel">
                    <h2>{auth_error_title(self.language)}</h2>
                    <p class="error-text">{error}</p>
                    {self.sign_in_link()}
                </section>
            },
            AuthenticationState::SignedIn(_) => match &self.application {
                None => html! { <>
                    {self.tournament_picker(context)}
                    <TournamentSetup on_create={context.link().callback(Msg::CreateTournament)} />
                </> },
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
            },
        }
    }

    fn signed_out_view(&self) -> Html {
        html! {
            <section class="panel auth-panel">
                <p class="eyebrow">{auth_required(self.language)}</p>
                <h2>{sign_in_label(self.language)}</h2>
                <p class="muted">{sign_in_explanation(self.language)}</p>
                {self.sign_in_link()}
            </section>
        }
    }

    fn sign_in_link(&self) -> Html {
        let return_to = if self.development_tools_enabled {
            "%2Fdev"
        } else {
            "%2F"
        };
        html! {
            <a class="button primary" href={format!("/api/auth/google/login?return_to={return_to}")}>
                {sign_in_label(self.language)}
            </a>
        }
    }

    fn tournament_picker(&self, context: &Context<Self>) -> Html {
        if self.tournaments.is_empty() {
            return Html::default();
        }
        html! {
            <section class="panel tournament-picker">
                <h2>{my_tournaments(self.language)}</h2>
                <div class="button-row">
                    {for self.tournaments.iter().map(|tournament| {
                        let id = tournament.id.clone();
                        let load = context.link().callback(move |_| Msg::LoadTournament(id.clone()));
                        html! {
                            <button class="secondary" onclick={load}>
                                <strong>{&tournament.title}</strong>
                                <span>{format!(" · {}", tournament.status)}</span>
                            </button>
                        }
                    })}
                </div>
            </section>
        }
    }

    pub(super) fn identity_controls(&self, context: &Context<Self>) -> Html {
        let AuthenticationState::SignedIn(user) = &self.authentication else {
            return Html::default();
        };
        let logout = context.link().callback(|_| Msg::Logout);
        let new_tournament = context.link().callback(|_| Msg::NewTournament);
        html! {
            <div class="identity-controls">
                <span>{user.display_name.as_deref().unwrap_or(&user.email)}</span>
                if self.application.is_some() {
                    <button class="secondary compact" onclick={new_tournament}>{new_tournament_label(self.language)}</button>
                }
                <button class="secondary compact" onclick={logout}>{sign_out_label(self.language)}</button>
            </div>
        }
    }
}

const fn sign_in_label(language: Language) -> &'static str {
    match language {
        Language::English => "Sign in with Google",
        Language::Dutch => "Inloggen met Google",
    }
}

const fn sign_out_label(language: Language) -> &'static str {
    match language {
        Language::English => "Sign out",
        Language::Dutch => "Uitloggen",
    }
}

const fn auth_loading(language: Language) -> &'static str {
    match language {
        Language::English => "Loading your session…",
        Language::Dutch => "Sessie laden…",
    }
}

const fn auth_required(language: Language) -> &'static str {
    match language {
        Language::English => "Authentication required",
        Language::Dutch => "Inloggen vereist",
    }
}

const fn sign_in_explanation(language: Language) -> &'static str {
    match language {
        Language::English => {
            "Sign in to create and resume tournaments stored securely on the server."
        }
        Language::Dutch => "Log in om toernooien veilig op de server aan te maken en te hervatten.",
    }
}

const fn auth_error_title(language: Language) -> &'static str {
    match language {
        Language::English => "Authentication error",
        Language::Dutch => "Inlogfout",
    }
}

const fn my_tournaments(language: Language) -> &'static str {
    match language {
        Language::English => "Resume a tournament",
        Language::Dutch => "Toernooi hervatten",
    }
}

const fn new_tournament_label(language: Language) -> &'static str {
    match language {
        Language::English => "Tournaments",
        Language::Dutch => "Toernooien",
    }
}
