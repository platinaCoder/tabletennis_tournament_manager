use super::Language;

impl Language {
    pub fn tournament_status(self, format: &str, tables: u16, rounds: u16) -> String {
        match self {
            Self::English => format!("{format} · {tables} tables · {rounds} rounds"),
            Self::Dutch => format!("{format} · {tables} tafels · {rounds} rondes"),
        }
    }

    pub const fn setup_explanation(self) -> &'static str {
        match self {
            Self::English => {
                "The match format, table count, and maximum rounds become fixed when play starts."
            }
            Self::Dutch => {
                "De wedstrijdvorm, het aantal tafels en het maximum aantal rondes liggen vast zodra het toernooi start."
            }
        }
    }

    pub const fn roster_edit_explanation(self) -> &'static str {
        match self {
            Self::English => {
                "Names, clubs, and starting ELOs can still be edited after the tournament starts."
            }
            Self::Dutch => {
                "Namen, verenigingen en ELO's bij aanvang kunnen na de toernooistart nog worden aangepast."
            }
        }
    }

    pub const fn roster_withdrawal_explanation(self) -> &'static str {
        match self {
            Self::English => {
                "Deleted contestants are withdrawn from future rounds. Published matches and historical standings remain unchanged."
            }
            Self::Dutch => {
                "Verwijderde deelnemers worden uit toekomstige rondes teruggetrokken. Gepubliceerde wedstrijden en historische standen blijven ongewijzigd."
            }
        }
    }

    pub fn table_count(self, count: u16) -> String {
        match self {
            Self::English => format!("{count} tables"),
            Self::Dutch => format!("{count} tafels"),
        }
    }

    pub fn round_count(self, count: usize) -> String {
        match self {
            Self::English => format!("{count} rounds"),
            Self::Dutch => format!("{count} rondes"),
        }
    }

    pub fn contestant_count(self, count: usize) -> String {
        match self {
            Self::English => format!("{count} contestants"),
            Self::Dutch => format!("{count} deelnemers"),
        }
    }

    pub fn match_count(self, count: usize) -> String {
        match self {
            Self::English => format!("{count} matches"),
            Self::Dutch => format!("{count} wedstrijden"),
        }
    }

    pub fn game_count(self, count: usize) -> String {
        match self {
            Self::English => format!("{count} games"),
            Self::Dutch => format!("{count} games"),
        }
    }

    pub fn contestant_number(self, number: usize) -> String {
        match self {
            Self::English => format!("Contestant {number}"),
            Self::Dutch => format!("Deelnemer {number}"),
        }
    }

    pub fn contestant_name_placeholder(self, number: usize) -> String {
        match self {
            Self::English => format!("Contestant {number} name"),
            Self::Dutch => format!("Naam van deelnemer {number}"),
        }
    }

    pub const fn club_placeholder(self) -> &'static str {
        match self {
            Self::English => "Start typing a club",
            Self::Dutch => "Begin een vereniging te typen",
        }
    }

    pub fn delete_contestant_label(self, number: usize) -> String {
        match self {
            Self::English => format!("Delete contestant {number}"),
            Self::Dutch => format!("Deelnemer {number} verwijderen"),
        }
    }

    pub fn after_completed_rounds(self, count: usize) -> String {
        match self {
            Self::English => format!("After {count} completed rounds"),
            Self::Dutch => format!("Na {count} afgeronde rondes"),
        }
    }

    pub fn calculate_round(self, round: usize) -> String {
        match self {
            Self::English => format!("Calculate round {round}"),
            Self::Dutch => format!("Ronde {round} berekenen"),
        }
    }

    pub fn round(self, round: u16) -> String {
        match self {
            Self::English => format!("Round {round}"),
            Self::Dutch => format!("Ronde {round}"),
        }
    }

    pub fn table(self, table: u16) -> String {
        match self {
            Self::English => format!("Table {table}"),
            Self::Dutch => format!("Tafel {table}"),
        }
    }

    pub fn pairing_heading(self, tier: &str) -> String {
        match self {
            Self::English => format!("{tier} pairings"),
            Self::Dutch => format!("Indeling · {tier}"),
        }
    }

    pub fn result_entry_round(self, round: u16) -> String {
        match self {
            Self::English => format!("Round {round} · result entry"),
            Self::Dutch => format!("Ronde {round} · uitslagen invoeren"),
        }
    }

    pub fn matches_complete(self, completed: usize, total: usize) -> String {
        match self {
            Self::English => format!("{completed} of {total} matches complete"),
            Self::Dutch => format!("{completed} van {total} wedstrijden afgerond"),
        }
    }

    pub const fn keyboard_hint(self) -> &'static str {
        match self {
            Self::English => {
                "Keyboard: home score → Tab → away score → Tab → next game. Press Enter to save once the match is complete."
            }
            Self::Dutch => {
                "Toetsenbord: thuisscore → Tab → uitscore → Tab → volgende game. Druk op Enter om een complete wedstrijd op te slaan."
            }
        }
    }

    pub fn bye_this_round(self, name: &str) -> String {
        match self {
            Self::English => format!("{name} has the bye this round."),
            Self::Dutch => format!("{name} heeft deze ronde een vrijloting."),
        }
    }

    pub fn game_home_points_label(self, game: usize) -> String {
        match self {
            Self::English => format!("Game {game} home points"),
            Self::Dutch => format!("Game {game} punten thuis"),
        }
    }

    pub fn game_away_points_label(self, game: usize) -> String {
        match self {
            Self::English => format!("Game {game} away points"),
            Self::Dutch => format!("Game {game} punten uit"),
        }
    }

    pub fn complete_winner(self, name: &str) -> String {
        match self {
            Self::English => format!("Complete · {name} wins"),
            Self::Dutch => format!("Afgerond · {name} wint"),
        }
    }

    pub fn winner(self, name: &str) -> String {
        match self {
            Self::English => format!("Winner: {name}"),
            Self::Dutch => format!("Winnaar: {name}"),
        }
    }

    pub fn same_club_warning(self, first: &str, second: &str) -> String {
        match self {
            Self::English => format!("Same-club pairing required: {first} vs {second}"),
            Self::Dutch => {
                format!("Indeling binnen dezelfde vereniging nodig: {first} tegen {second}")
            }
        }
    }

    pub fn rematch_warning(self, first: &str, second: &str) -> String {
        match self {
            Self::English => format!("Rematch required: {first} vs {second}"),
            Self::Dutch => format!("Herkansing nodig: {first} tegen {second}"),
        }
    }

    pub fn bye_warning(self, name: &str) -> String {
        match self {
            Self::English => format!("Bye assigned to {name}"),
            Self::Dutch => format!("Vrijloting toegewezen aan {name}"),
        }
    }

    pub fn relaxation_warning(self, tier: &str) -> String {
        match self {
            Self::English => format!("{tier} was required"),
            Self::Dutch => format!("{tier} was nodig"),
        }
    }
}
