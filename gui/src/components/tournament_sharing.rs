use tabletennis_tournament::api_contract::{TournamentAccessRole, TournamentSharingView};
use web_sys::{HtmlInputElement, HtmlSelectElement};
use yew::prelude::*;

use crate::language::{Language, use_language};

use super::tournament_dashboard::role_label;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ShareAccessCommand {
    pub email: String,
    pub role: TournamentAccessRole,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MemberRoleCommand {
    pub user_id: String,
    pub role: TournamentAccessRole,
}

#[derive(Properties, PartialEq)]
pub struct TournamentSharingProps {
    pub sharing: TournamentSharingView,
    pub on_grant: Callback<ShareAccessCommand>,
    pub on_update_member: Callback<MemberRoleCommand>,
    pub on_remove_member: Callback<String>,
    pub on_delete_invitation: Callback<String>,
    pub on_close: Callback<()>,
}

#[component]
pub fn TournamentSharing(props: &TournamentSharingProps) -> Html {
    let language = use_language();
    let email = use_state(String::new);
    let role = use_state(|| TournamentAccessRole::Editor);
    let onsubmit = {
        let email = email.clone();
        let role = role.clone();
        let callback = props.on_grant.clone();
        Callback::from(move |event: SubmitEvent| {
            event.prevent_default();
            let value = email.trim().to_owned();
            if !value.is_empty() {
                callback.emit(ShareAccessCommand {
                    email: value,
                    role: *role,
                });
                email.set(String::new());
            }
        })
    };
    let update_email = {
        let email = email.clone();
        Callback::from(move |event: InputEvent| {
            email.set(event.target_unchecked_into::<HtmlInputElement>().value());
        })
    };
    let update_role = {
        let role = role.clone();
        Callback::from(move |event: Event| {
            role.set(role_from_value(
                &event.target_unchecked_into::<HtmlSelectElement>().value(),
            ));
        })
    };
    let close = {
        let callback = props.on_close.clone();
        Callback::from(move |_| callback.emit(()))
    };

    html! {
        <section class="panel sharing-panel">
            <div class="section-heading">
                <div>
                    <p class="eyebrow">{sharing_eyebrow(language)}</p>
                    <h2>{sharing_title(language)}</h2>
                </div>
                <button class="secondary compact" onclick={close}>{close_label(language)}</button>
            </div>
            <p class="muted">{sharing_explanation(language)}</p>
            <form class="sharing-form" {onsubmit}>
                <label>
                    {email_label(language)}
                    <input type="email" required=true maxlength="320" value={(*email).clone()} oninput={update_email} />
                </label>
                <label>
                    {permission_label(language)}
                    <select onchange={update_role}>
                        <option value="editor" selected={*role == TournamentAccessRole::Editor}>{role_label(TournamentAccessRole::Editor, language)}</option>
                        <option value="viewer" selected={*role == TournamentAccessRole::Viewer}>{role_label(TournamentAccessRole::Viewer, language)}</option>
                    </select>
                </label>
                <button class="primary align-end" type="submit">{grant_label(language)}</button>
            </form>
            <div class="sharing-list">
                <h3>{members_label(language)}</h3>
                {for props.sharing.members.iter().map(|member| {
                    let label = member.display_name.as_deref().unwrap_or(&member.email);
                    let user_id = member.user_id.clone();
                    let update = {
                        let callback = props.on_update_member.clone();
                        let user_id = user_id.clone();
                        Callback::from(move |event: Event| {
                            callback.emit(MemberRoleCommand {
                                user_id: user_id.clone(),
                                role: role_from_value(&event.target_unchecked_into::<HtmlSelectElement>().value()),
                            });
                        })
                    };
                    let remove = {
                        let callback = props.on_remove_member.clone();
                        let user_id = user_id.clone();
                        Callback::from(move |_| callback.emit(user_id.clone()))
                    };
                    html! {
                        <div class="sharing-row" key={member.user_id.clone()}>
                            <div>
                                <strong>{label}</strong>
                                if member.display_name.is_some() {
                                    <small>{&member.email}</small>
                                }
                            </div>
                            if member.role.is_owner() {
                                <span class="access-badge owner-access">{role_label(member.role, language)}</span>
                            } else {
                                <select aria-label={permission_label(language)} onchange={update}>
                                    <option value="editor" selected={member.role == TournamentAccessRole::Editor}>{role_label(TournamentAccessRole::Editor, language)}</option>
                                    <option value="viewer" selected={member.role == TournamentAccessRole::Viewer}>{role_label(TournamentAccessRole::Viewer, language)}</option>
                                </select>
                                <button class="danger-link compact" onclick={remove}>{remove_label(language)}</button>
                            }
                        </div>
                    }
                })}
                if !props.sharing.invitations.is_empty() {
                    <h3>{pending_label(language)}</h3>
                    {for props.sharing.invitations.iter().map(|invitation| {
                        let invitation_id = invitation.id.clone();
                        let remove = {
                            let callback = props.on_delete_invitation.clone();
                            Callback::from(move |_| callback.emit(invitation_id.clone()))
                        };
                        html! {
                            <div class="sharing-row pending-invitation" key={invitation.id.clone()}>
                                <div>
                                    <strong>{&invitation.email}</strong>
                                    <small>{pending_explanation(language)}</small>
                                </div>
                                <span class="access-badge">{role_label(invitation.role, language)}</span>
                                <button class="danger-link compact" onclick={remove}>{revoke_label(language)}</button>
                            </div>
                        }
                    })}
                }
            </div>
        </section>
    }
}

fn role_from_value(value: &str) -> TournamentAccessRole {
    if value == "viewer" {
        TournamentAccessRole::Viewer
    } else {
        TournamentAccessRole::Editor
    }
}

const fn sharing_eyebrow(language: Language) -> &'static str {
    match language {
        Language::English => "Access management",
        Language::Dutch => "Toegangsbeheer",
    }
}

const fn sharing_title(language: Language) -> &'static str {
    match language {
        Language::English => "Share tournament",
        Language::Dutch => "Toernooi delen",
    }
}

const fn sharing_explanation(language: Language) -> &'static str {
    match language {
        Language::English => {
            "Editors can manage the tournament and enter results. Viewers have read-only access. The recipient must accept the invitation from their dashboard."
        }
        Language::Dutch => {
            "Bewerkers kunnen het toernooi beheren en uitslagen invoeren. Lezers hebben alleen leestoegang. De ontvanger moet de uitnodiging via het dashboard accepteren."
        }
    }
}

const fn email_label(language: Language) -> &'static str {
    match language {
        Language::English => "Google account email",
        Language::Dutch => "E-mailadres van Google-account",
    }
}

const fn permission_label(language: Language) -> &'static str {
    match language {
        Language::English => "Permission",
        Language::Dutch => "Rechten",
    }
}

const fn grant_label(language: Language) -> &'static str {
    match language {
        Language::English => "Send invitation",
        Language::Dutch => "Uitnodiging sturen",
    }
}

const fn members_label(language: Language) -> &'static str {
    match language {
        Language::English => "People with access",
        Language::Dutch => "Personen met toegang",
    }
}

const fn pending_label(language: Language) -> &'static str {
    match language {
        Language::English => "Pending invitations",
        Language::Dutch => "Openstaande uitnodigingen",
    }
}

const fn pending_explanation(language: Language) -> &'static str {
    match language {
        Language::English => "Waiting for the recipient to accept",
        Language::Dutch => "Wacht op acceptatie door de ontvanger",
    }
}

const fn remove_label(language: Language) -> &'static str {
    match language {
        Language::English => "Remove",
        Language::Dutch => "Verwijderen",
    }
}

const fn revoke_label(language: Language) -> &'static str {
    match language {
        Language::English => "Revoke",
        Language::Dutch => "Intrekken",
    }
}

const fn close_label(language: Language) -> &'static str {
    match language {
        Language::English => "Close",
        Language::Dutch => "Sluiten",
    }
}
