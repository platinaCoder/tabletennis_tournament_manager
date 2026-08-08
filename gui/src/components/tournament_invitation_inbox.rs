use tabletennis_tournament::api_contract::ReceivedTournamentInvitationView;
use yew::prelude::*;

use crate::language::{Language, use_language};

use super::tournament_dashboard::role_label;

#[derive(Properties, PartialEq)]
pub struct TournamentInvitationInboxProps {
    pub invitations: Vec<ReceivedTournamentInvitationView>,
    pub on_accept: Callback<String>,
    pub on_decline: Callback<String>,
}

#[component]
pub fn TournamentInvitationInbox(props: &TournamentInvitationInboxProps) -> Html {
    let language = use_language();
    if props.invitations.is_empty() {
        return Html::default();
    }
    html! {
        <section class="panel invitation-inbox" aria-labelledby="invitation-inbox-title">
            <p class="eyebrow">{eyebrow(language)}</p>
            <h2 id="invitation-inbox-title">{title(props.invitations.len(), language)}</h2>
            <div class="invitation-list">
                {for props.invitations.iter().map(|invitation| {
                    invitation_row(props, invitation, language)
                })}
            </div>
        </section>
    }
}

fn invitation_row(
    props: &TournamentInvitationInboxProps,
    invitation: &ReceivedTournamentInvitationView,
    language: Language,
) -> Html {
    let inviter = invitation
        .invited_by_display_name
        .as_deref()
        .unwrap_or(&invitation.invited_by_email);
    let invitation_id = invitation.id.clone();
    let accept = {
        let callback = props.on_accept.clone();
        let invitation_id = invitation_id.clone();
        Callback::from(move |_| callback.emit(invitation_id.clone()))
    };
    let decline = {
        let callback = props.on_decline.clone();
        Callback::from(move |_| callback.emit(invitation_id.clone()))
    };
    html! {
        <article class="invitation-row" key={invitation.id.clone()}>
            <div>
                <strong>{&invitation.tournament_title}</strong>
                <p>{invitation_description(
                    inviter,
                    role_label(invitation.role, language),
                    language,
                )}</p>
            </div>
            <div class="invitation-actions">
                <button class="primary compact" onclick={accept}>{accept_label(language)}</button>
                <button class="secondary compact" onclick={decline}>{decline_label(language)}</button>
            </div>
        </article>
    }
}

fn invitation_description(inviter: &str, role: &str, language: Language) -> String {
    match language {
        Language::English => format!("{inviter} invited you with {role} access."),
        Language::Dutch => format!("{inviter} heeft je uitgenodigd met de rol {role}."),
    }
}

fn title(count: usize, language: Language) -> String {
    match (count, language) {
        (1, Language::English) => "Tournament invitation".to_owned(),
        (_, Language::English) => format!("{count} tournament invitations"),
        (1, Language::Dutch) => "Toernooi-uitnodiging".to_owned(),
        (_, Language::Dutch) => format!("{count} toernooi-uitnodigingen"),
    }
}

const fn eyebrow(language: Language) -> &'static str {
    match language {
        Language::English => "Action required",
        Language::Dutch => "Actie vereist",
    }
}

const fn accept_label(language: Language) -> &'static str {
    match language {
        Language::English => "Accept",
        Language::Dutch => "Accepteren",
    }
}

const fn decline_label(language: Language) -> &'static str {
    match language {
        Language::English => "Decline",
        Language::Dutch => "Weigeren",
    }
}
