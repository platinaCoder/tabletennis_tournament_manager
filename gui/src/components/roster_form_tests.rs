use tabletennis_tournament::identity::{ClubId, EntrantId};
use tabletennis_tournament::pairing::EloRating;

use super::*;

#[test]
fn requested_count_creates_that_many_blank_rows() {
    let rows = initial_rows(&[], 12);

    assert_eq!(rows.len(), 12);
    assert!(rows.iter().all(|row| row.entrant_id.is_none()));
    assert!(rows.iter().all(|row| row.elo == "1200"));
}

#[test]
fn existing_roster_keeps_stable_ids_hidden_in_form_state() {
    let entrant = TournamentEntrant {
        entrant_id: EntrantId::new("internal-id"),
        name: "Ada".to_owned(),
        club_id: ClubId::new("internal-club"),
        club_name: "Local Club".to_owned(),
        starting_elo: EloRating::new(1_350),
    };

    let rows = initial_rows(&[entrant], 16);

    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].entrant_id.as_deref(), Some("internal-id"));
    assert_eq!(rows[0].name, "Ada");
}

#[test]
fn simulated_roster_spans_the_visible_elo_test_range() {
    let rows = simulated_rows(16);

    assert_eq!(rows.len(), 16);
    assert_eq!(rows.first().unwrap().elo, "900");
    assert_eq!(rows.last().unwrap().elo, "1500");
    assert!(rows.iter().all(|row| row.entrant_id.is_none()));
}
