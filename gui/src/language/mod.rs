mod dutch;
mod english;
mod errors;
mod phrasing;

use yew::prelude::*;

const STORAGE_KEY: &str = "tabletennis-tournament-language";

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum Language {
    #[default]
    English,
    Dutch,
}

impl Language {
    pub const fn text(self, key: Text) -> &'static str {
        match self {
            Self::English => english::text(key),
            Self::Dutch => dutch::text(key),
        }
    }

    pub const fn toggled(self) -> Self {
        match self {
            Self::English => Self::Dutch,
            Self::Dutch => Self::English,
        }
    }

    pub const fn toggle_label(self) -> &'static str {
        match self {
            Self::English => "Nederlands",
            Self::Dutch => "English",
        }
    }

    const fn storage_value(self) -> &'static str {
        match self {
            Self::English => "en",
            Self::Dutch => "nl",
        }
    }

    const fn document_code(self) -> &'static str {
        self.storage_value()
    }
}

#[derive(Clone, Copy)]
pub enum Text {
    ActiveRoster,
    AddContestant,
    AvailableTables,
    Away,
    Bye,
    ByePenalty,
    CandidatePairs,
    Cancel,
    Club,
    CloseRoster,
    CompleteRound,
    CompleteTournamentRecord,
    Contestant,
    ContestantCount,
    Cost,
    CostCalculation,
    CreateTournament,
    DarkMode,
    Delete,
    DismissError,
    DeveloperDiagnostics,
    DeveloperSimulationMode,
    EdgeGeneration,
    EditContestants,
    EligibleEdges,
    EloComponent,
    EloUnavailable,
    EnterRemainingGames,
    EnterRoster,
    ExportSimulationJson,
    FinalStandings,
    FillTestContestants,
    Game,
    Games,
    HideMatchResults,
    Home,
    LightMode,
    LocalTournamentControl,
    ManageContestants,
    MatchFormat,
    MatchResultsByRound,
    MatchWinComponent,
    MaximumRounds,
    NoRelaxationWarnings,
    NoResult,
    OpponentComponent,
    OpponentShort,
    PairingCalculation,
    PairingPolicy,
    PerformanceComponent,
    Points,
    PublishRound,
    Recalculate,
    Registration,
    RelaxationTier,
    RematchPenalty,
    RematchesRejected,
    ResultEntryWaitsForTable,
    SameClubPenalty,
    SameClubRejected,
    SaveResult,
    SaveRoster,
    Score,
    SelectionCost,
    ShowMatchResults,
    SimulateRemainingGames,
    Solver,
    StartTournament,
    StartingElo,
    TableTennisTournament,
    TieBreak,
    TotalCost,
    TournamentComplete,
    TournamentIdentifier,
    TournamentSetup,
    TournamentStandings,
    Unassigned,
    UnknownClub,
    UnknownContestant,
    UnpublishedPairingPreview,
    Validation,
    Versus,
    WaitingForTable,
    Warnings,
    WinsLosses,
    Withdrawn,
}

#[hook]
pub fn use_language() -> Language {
    use_context::<Language>().unwrap_or_default()
}

pub fn load_language() -> Language {
    storage()
        .and_then(|storage| storage.get_item(STORAGE_KEY).ok().flatten())
        .map_or(Language::English, |value| match value.as_str() {
            "nl" => Language::Dutch,
            _ => Language::English,
        })
}

pub fn store_language(language: Language) {
    if let Some(storage) = storage() {
        let _ = storage.set_item(STORAGE_KEY, language.storage_value());
    }
}

pub fn apply_to_document(language: Language) {
    let Some(document) = web_sys::window().and_then(|window| window.document()) else {
        return;
    };
    document.set_title(language.text(Text::TableTennisTournament));
    if let Some(root) = document.document_element() {
        let _ = root.set_attribute("lang", language.document_code());
    }
}

fn storage() -> Option<web_sys::Storage> {
    web_sys::window()
        .and_then(|window| window.local_storage().ok())
        .flatten()
}

#[cfg(test)]
mod tests {
    use super::{Language, Text};

    #[test]
    fn languages_have_distinct_page_text_and_toggle_targets() {
        assert_eq!(
            Language::English.text(Text::CreateTournament),
            "Create tournament"
        );
        assert_eq!(
            Language::Dutch.text(Text::CreateTournament),
            "Toernooi aanmaken"
        );
        assert_eq!(Language::English.toggled(), Language::Dutch);
        assert_eq!(Language::Dutch.toggled(), Language::English);
    }
}
