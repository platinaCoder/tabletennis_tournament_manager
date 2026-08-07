use tabletennis_tournament::results::MatchFormat;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CreateTournamentCommand {
    pub tournament_id: String,
    pub match_format: MatchFormat,
    pub table_count: i64,
    pub contestant_count: usize,
    pub maximum_round_count: i64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RosterEntryCommand {
    pub entrant_id: Option<String>,
    pub name: String,
    pub club_name: String,
    pub starting_elo: u32,
}
