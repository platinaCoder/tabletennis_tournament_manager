use tabletennis_tournament::application::TournamentEntrant;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct RosterRow {
    pub key: usize,
    pub entrant_id: Option<String>,
    pub name: String,
    pub club_name: String,
    pub elo: String,
}

pub(super) fn initial_rows(
    entrants: &[TournamentEntrant],
    requested_count: usize,
) -> Vec<RosterRow> {
    if !entrants.is_empty() {
        return entrants
            .iter()
            .enumerate()
            .map(|(index, entrant)| RosterRow {
                key: index + 1,
                entrant_id: Some(entrant.entrant_id.as_str().to_owned()),
                name: entrant.name.clone(),
                club_name: entrant.club_name.clone(),
                elo: entrant.starting_elo.value().to_string(),
            })
            .collect();
    }
    (1..=requested_count).map(blank_row).collect()
}

pub(super) fn blank_row(key: usize) -> RosterRow {
    RosterRow {
        key,
        entrant_id: None,
        name: String::new(),
        club_name: String::new(),
        elo: "1200".to_owned(),
    }
}

pub(super) fn simulated_rows(count: usize) -> Vec<RosterRow> {
    const CLUBS: [&str; 4] = ["Club Alpha", "Club Bravo", "Club Charlie", "Club Delta"];
    let denominator = count.saturating_sub(1).max(1);
    (0..count)
        .map(|index| RosterRow {
            key: index + 1,
            entrant_id: None,
            name: format!("Test contestant {:02}", index + 1),
            club_name: CLUBS[index % CLUBS.len()].to_owned(),
            elo: (900 + index.saturating_mul(600) / denominator).to_string(),
        })
        .collect()
}
