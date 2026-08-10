use tabletennis_tournament::results::MatchResultError;

use super::Language;

impl Language {
    pub const fn simulation_route_error(self) -> &'static str {
        match self {
            Self::English => "Simulation tools are only available at /dev.",
            Self::Dutch => "Simulatiehulpmiddelen zijn alleen beschikbaar via /dev.",
        }
    }

    pub fn simulation_export_error(self, detail: &str) -> String {
        match self {
            Self::English => format!("Could not export the simulation trace: {detail}"),
            Self::Dutch => format!("Kon het simulatieverslag niet exporteren: {detail}"),
        }
    }

    pub const fn simulation_seed_error(self) -> &'static str {
        match self {
            Self::English => "Create a new /dev tournament to initialize its simulation seed.",
            Self::Dutch => {
                "Maak een nieuw /dev-toernooi aan om de simulatiereeks te initialiseren."
            }
        }
    }

    pub const fn contestant_range_error(self) -> &'static str {
        match self {
            Self::English => "Contestant count must be between 2 and 64.",
            Self::Dutch => "Het aantal deelnemers moet tussen 2 en 64 liggen.",
        }
    }

    pub const fn create_tournament_first_error(self) -> &'static str {
        match self {
            Self::English => "Create a tournament first.",
            Self::Dutch => "Maak eerst een toernooi aan.",
        }
    }

    pub const fn roster_fields_error(self) -> &'static str {
        match self {
            Self::English => "Every contestant needs a name and club.",
            Self::Dutch => "Iedere deelnemer heeft een naam en vereniging nodig.",
        }
    }

    pub const fn duplicate_roster_error(self) -> &'static str {
        match self {
            Self::English => "The roster contains the same contestant twice.",
            Self::Dutch => "De deelnemerslijst bevat dezelfde deelnemer twee keer.",
        }
    }

    pub fn match_result_error(self, error: &MatchResultError) -> String {
        if self == Self::English {
            return error.to_string();
        }
        match error {
            MatchResultError::InvalidGameScore {
                game_number,
                home_points,
                away_points,
            } => {
                format!("game {game_number} heeft een ongeldige score: {home_points}-{away_points}")
            }
            MatchResultError::GameNumbersNotSequential { expected, actual } => {
                format!("gamenummers moeten oplopend zijn: {expected} verwacht, {actual} ontvangen")
            }
            MatchResultError::MatchNotComplete {
                home_games_won,
                away_games_won,
            } => format!("wedstrijd is nog niet compleet bij {home_games_won}-{away_games_won}"),
            MatchResultError::TooManyGames { maximum, submitted } => {
                format!("wedstrijd staat maximaal {maximum} games toe; {submitted} ingevoerd")
            }
            MatchResultError::GamesRecordedAfterMatchCompletion {
                winning_game_number,
            } => format!(
                "games ingevoerd nadat de wedstrijd eindigde bij game {winning_game_number}"
            ),
            MatchResultError::MatchNotPublished => "wedstrijd is niet gepubliceerd".to_owned(),
            MatchResultError::RoundNotActive => {
                "wedstrijd hoort niet bij de actieve ronde".to_owned()
            }
            MatchResultError::ResultDoesNotBelongToMatch => {
                "de bestaande uitslag hoort bij een andere wedstrijd".to_owned()
            }
            MatchResultError::CorrectionReasonTooLong { maximum } => {
                format!("de reden voor correctie mag maximaal {maximum} bytes bevatten")
            }
            MatchResultError::UnexpectedCorrectionReason => {
                "een eerste uitslag kan geen reden voor correctie bevatten".to_owned()
            }
            MatchResultError::CorrectionTimestampRequired => {
                "een gecorrigeerde uitslag vereist een correctietijdstip".to_owned()
            }
            MatchResultError::MatchResultRevisionOverflow => {
                "de uitslagrevisie overschrijdt de ondersteunde limiet".to_owned()
            }
        }
    }

    pub const fn sequential_games_error(self) -> &'static str {
        match self {
            Self::English => "Enter games sequentially without blank rows.",
            Self::Dutch => "Vul games op volgorde in, zonder lege tussenregels.",
        }
    }

    pub const fn whole_points_error(self) -> &'static str {
        match self {
            Self::English => "Enter both point totals using whole numbers.",
            Self::Dutch => "Vul beide puntentotalen in als gehele getallen.",
        }
    }

    pub const fn game_number_limit_error(self) -> &'static str {
        match self {
            Self::English => "Game number exceeds the supported limit.",
            Self::Dutch => "Het gamenummer overschrijdt de ondersteunde limiet.",
        }
    }

    pub const fn invalid_game_number_error(self) -> &'static str {
        match self {
            Self::English => "Game number is invalid.",
            Self::Dutch => "Het gamenummer is ongeldig.",
        }
    }
}
