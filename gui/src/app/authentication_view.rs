use tabletennis_tournament::tournament::TournamentState;
use yew::prelude::*;

use super::{App, AuthenticationState, Msg};
use crate::components::{
    Registration, TournamentDashboard, TournamentInvitationInbox, TournamentSetup,
    TournamentSharing,
};
use crate::language::Language;
use crate::model::WorkspacePage;

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
            AuthenticationState::SignedIn(_) => self.signed_in_content(context),
        }
    }

    fn signed_in_content(&self, context: &Context<Self>) -> Html {
        match self.page {
            WorkspacePage::Dashboard => self.dashboard(context),
            WorkspacePage::CreateTournament => html! {
                <TournamentSetup
                    on_create={context.link().callback(Msg::CreateTournament)}
                    on_cancel={context.link().callback(|()| Msg::ShowDashboard)}
                />
            },
            WorkspacePage::Tournament => self.tournament(context),
        }
    }

    fn dashboard(&self, context: &Context<Self>) -> Html {
        html! { <>
            <TournamentInvitationInbox
                invitations={self.received_invitations.clone()}
                on_accept={context.link().callback(Msg::AcceptInvitation)}
                on_decline={context.link().callback(Msg::DeclineInvitation)}
            />
            <TournamentDashboard
                tournaments={self.tournaments.clone()}
                on_create={context.link().callback(|()| Msg::ShowCreateTournament)}
                on_open={context.link().callback(Msg::LoadTournament)}
                on_delete={context.link().callback(|(id, revision)| Msg::DeleteTournament(id, revision))}
                on_share={context.link().callback(Msg::OpenSharing)}
            />
            {self.sharing.as_ref().map(|sharing| html! {
                <TournamentSharing
                    sharing={sharing.clone()}
                    on_grant={context.link().callback(Msg::GrantAccess)}
                    on_update_member={context.link().callback(Msg::UpdateMemberRole)}
                    on_remove_member={context.link().callback(Msg::RemoveMember)}
                    on_delete_invitation={context.link().callback(Msg::DeleteInvitation)}
                    on_close={context.link().callback(|()| Msg::CloseSharing)}
                />
            }).unwrap_or_default()}
        </> }
    }

    fn tournament(&self, context: &Context<Self>) -> Html {
        let Some(application) = &self.application else {
            return html! { <section class="panel"><p>{tournament_loading(self.language)}</p></section> };
        };
        if application.tournament().state() == TournamentState::Draft && self.can_edit_tournament()
        {
            return html! {
                <Registration
                    contestant_count={self.initial_contestant_count}
                    match_format={application.tournament().match_format()}
                    table_count={application.tournament().table_count().value()}
                    maximum_round_count={application.tournament().maximum_round_count().value()}
                    allow_simulation={self.development_tools_enabled}
                    on_start={context.link().callback(Msg::StartTournament)}
                />
            };
        }
        if application.tournament().state() == TournamentState::Draft {
            return self.read_only_draft(application);
        }
        html! { <>
            {self.read_only_notice()}
            {self.started_view(context, application)}
        </> }
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

    fn read_only_draft(
        &self,
        application: &tabletennis_tournament::application::TournamentApplication,
    ) -> Html {
        html! {
            <section class="panel read-only-panel">
                <p class="eyebrow">{read_only_label(self.language)}</p>
                <h2>{application.tournament().id().as_str()}</h2>
                <p>{draft_waiting_label(self.language)}</p>
            </section>
        }
    }

    fn read_only_notice(&self) -> Html {
        if self.can_edit_tournament() {
            Html::default()
        } else {
            html! {
                <div class="read-only-notice">
                    <strong>{read_only_label(self.language)}</strong>
                    <span>{viewer_explanation(self.language)}</span>
                </div>
            }
        }
    }

    pub(super) fn identity_controls(&self, context: &Context<Self>) -> Html {
        let AuthenticationState::SignedIn(user) = &self.authentication else {
            return Html::default();
        };
        let logout = context.link().callback(|_| Msg::Logout);
        let show_dashboard = context.link().callback(|_| Msg::ShowDashboard);
        html! {
            <div class="identity-controls">
                <span>{user.display_name.as_deref().unwrap_or(&user.email)}</span>
                if self.page != WorkspacePage::Dashboard {
                    <button class="secondary compact" onclick={show_dashboard}>{dashboard_label(self.language)}</button>
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

const fn read_only_label(language: Language) -> &'static str {
    match language {
        Language::English => "Read-only access",
        Language::Dutch => "Alleen-lezen toegang",
    }
}

const fn viewer_explanation(language: Language) -> &'static str {
    match language {
        Language::English => {
            "You can inspect this tournament, but only its owner or editors can change it."
        }
        Language::Dutch => {
            "Je kunt dit toernooi bekijken, maar alleen de eigenaar of bewerkers kunnen het wijzigen."
        }
    }
}

const fn draft_waiting_label(language: Language) -> &'static str {
    match language {
        Language::English => "This tournament is still being prepared by an editor.",
        Language::Dutch => "Dit toernooi wordt nog voorbereid door een bewerker.",
    }
}

const fn dashboard_label(language: Language) -> &'static str {
    match language {
        Language::English => "Dashboard",
        Language::Dutch => "Dashboard",
    }
}

const fn tournament_loading(language: Language) -> &'static str {
    match language {
        Language::English => "Loading tournament…",
        Language::Dutch => "Toernooi laden…",
    }
}
