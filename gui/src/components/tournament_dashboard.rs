use tabletennis_tournament::api_contract::{TournamentAccessRole, TournamentSummaryView};
use yew::prelude::*;

use crate::language::{Language, use_language};

#[derive(Properties, PartialEq)]
pub struct TournamentDashboardProps {
    pub tournaments: Vec<TournamentSummaryView>,
    pub on_create: Callback<()>,
    pub on_open: Callback<String>,
    pub on_delete: Callback<(String, u64)>,
    pub on_share: Callback<String>,
}

#[component]
pub fn TournamentDashboard(props: &TournamentDashboardProps) -> Html {
    let language = use_language();
    let confirming_delete = use_state(|| None::<String>);
    let create = {
        let callback = props.on_create.clone();
        Callback::from(move |_| callback.emit(()))
    };

    html! {
        <section class="panel tournament-dashboard">
            <div class="section-heading">
                <div>
                    <p class="eyebrow">{dashboard_eyebrow(language)}</p>
                    <h2>{dashboard_title(language)}</h2>
                </div>
                <div class="dashboard-heading-actions">
                    <span class="muted">{tournament_count(props.tournaments.len(), language)}</span>
                    <button class="primary" onclick={create}>{create_label(language)}</button>
                </div>
            </div>
            if props.tournaments.is_empty() {
                <p class="dashboard-empty muted">{empty_dashboard(language)}</p>
            } else {
                <div class="tournament-card-grid">
                    {for props.tournaments.iter().map(|tournament| {
                        tournament_card(props, tournament, &confirming_delete, language)
                    })}
                </div>
            }
        </section>
    }
}

fn tournament_card(
    props: &TournamentDashboardProps,
    tournament: &TournamentSummaryView,
    confirming_delete: &UseStateHandle<Option<String>>,
    language: Language,
) -> Html {
    let id = tournament.id.clone();
    let open = {
        let callback = props.on_open.clone();
        let id = id.clone();
        Callback::from(move |_| callback.emit(id.clone()))
    };
    let share = {
        let callback = props.on_share.clone();
        let id = id.clone();
        Callback::from(move |_| callback.emit(id.clone()))
    };
    let is_confirming = confirming_delete.as_ref() == Some(&id);
    let begin_delete = {
        let confirming_delete = confirming_delete.clone();
        let id = id.clone();
        Callback::from(move |_| confirming_delete.set(Some(id.clone())))
    };
    let cancel_delete = {
        let confirming_delete = confirming_delete.clone();
        Callback::from(move |_| confirming_delete.set(None))
    };
    let confirm_delete = {
        let confirming_delete = confirming_delete.clone();
        let callback = props.on_delete.clone();
        let id = id.clone();
        let revision = tournament.revision;
        Callback::from(move |_| {
            confirming_delete.set(None);
            callback.emit((id.clone(), revision));
        })
    };

    html! {
        <article class="tournament-dashboard-card" key={id}>
            <div class="tournament-card-heading">
                <div>
                    <h3>{&tournament.title}</h3>
                    <span class="muted">{status_label(&tournament.status, language)}</span>
                </div>
                <span class={classes!("access-badge", role_class(tournament.access_role))}>
                    {role_label(tournament.access_role, language)}
                </span>
            </div>
            <small>{updated_label(&tournament.updated_at, language)}</small>
            <div class="tournament-card-actions">
                <button class="primary compact" onclick={open}>{open_label(language)}</button>
                if tournament.access_role.is_owner() {
                    <button class="secondary compact" onclick={share}>{share_label(language)}</button>
                    if is_confirming {
                        <span class="delete-confirmation">{delete_warning(language)}</span>
                        <button class="danger-button compact" onclick={confirm_delete}>{confirm_label(language)}</button>
                        <button class="secondary compact" onclick={cancel_delete}>{cancel_label(language)}</button>
                    } else {
                        <button class="danger-button compact" onclick={begin_delete}>{delete_label(language)}</button>
                    }
                }
            </div>
        </article>
    }
}

const fn role_class(role: TournamentAccessRole) -> &'static str {
    match role {
        TournamentAccessRole::Owner => "owner-access",
        TournamentAccessRole::Editor => "editor-access",
        TournamentAccessRole::Viewer => "viewer-access",
    }
}

pub(crate) const fn role_label(role: TournamentAccessRole, language: Language) -> &'static str {
    match (role, language) {
        (TournamentAccessRole::Owner, Language::English) => "Owner",
        (TournamentAccessRole::Owner, Language::Dutch) => "Eigenaar",
        (TournamentAccessRole::Editor, Language::English) => "Editor",
        (TournamentAccessRole::Editor, Language::Dutch) => "Bewerker",
        (TournamentAccessRole::Viewer, Language::English) => "Viewer",
        (TournamentAccessRole::Viewer, Language::Dutch) => "Lezer",
    }
}

fn status_label(status: &str, language: Language) -> &str {
    match (status, language) {
        ("draft", Language::English) => "Draft",
        ("draft", Language::Dutch) => "Concept",
        ("started", Language::English) => "Started",
        ("started", Language::Dutch) => "Gestart",
        _ => status,
    }
}

fn updated_label(value: &str, language: Language) -> String {
    let timestamp = value.get(..16).unwrap_or(value).replace('T', " ");
    match language {
        Language::English => format!("Last updated {timestamp} UTC"),
        Language::Dutch => format!("Laatst bijgewerkt {timestamp} UTC"),
    }
}

fn tournament_count(count: usize, language: Language) -> String {
    match language {
        Language::English => format!("{count} tournaments"),
        Language::Dutch => format!("{count} toernooien"),
    }
}

const fn dashboard_eyebrow(language: Language) -> &'static str {
    match language {
        Language::English => "Tournament management",
        Language::Dutch => "Toernooibeheer",
    }
}

const fn dashboard_title(language: Language) -> &'static str {
    match language {
        Language::English => "Your tournaments",
        Language::Dutch => "Jouw toernooien",
    }
}

const fn empty_dashboard(language: Language) -> &'static str {
    match language {
        Language::English => "No tournaments yet. Create your first tournament to get started.",
        Language::Dutch => "Nog geen toernooien. Maak je eerste toernooi aan om te beginnen.",
    }
}

const fn create_label(language: Language) -> &'static str {
    match language {
        Language::English => "Create tournament",
        Language::Dutch => "Toernooi aanmaken",
    }
}

const fn open_label(language: Language) -> &'static str {
    match language {
        Language::English => "Open",
        Language::Dutch => "Openen",
    }
}

const fn share_label(language: Language) -> &'static str {
    match language {
        Language::English => "Share",
        Language::Dutch => "Delen",
    }
}

const fn delete_label(language: Language) -> &'static str {
    match language {
        Language::English => "Delete",
        Language::Dutch => "Verwijderen",
    }
}

const fn delete_warning(language: Language) -> &'static str {
    match language {
        Language::English => "Delete all tournament data?",
        Language::Dutch => "Alle toernooigegevens verwijderen?",
    }
}

const fn confirm_label(language: Language) -> &'static str {
    match language {
        Language::English => "Delete permanently",
        Language::Dutch => "Definitief verwijderen",
    }
}

const fn cancel_label(language: Language) -> &'static str {
    match language {
        Language::English => "Cancel",
        Language::Dutch => "Annuleren",
    }
}
