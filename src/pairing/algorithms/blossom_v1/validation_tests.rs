use crate::identity::EntrantId;
use crate::pairing::EloRating;

use super::*;

fn round(number: i64) -> RoundNumber {
    RoundNumber::try_from(number).unwrap()
}

fn policy(maximum_entrant_count: usize) -> BlossomV1Policy {
    BlossomV1Policy {
        avoid_same_club: true,
        avoid_rematches: true,
        recent_rematch_window: 3,
        performance_score_weight: 100,
        match_win_weight: 100,
        opponent_strength_weight: 10,
        elo_difference_weight: 1,
        bye_repeat_penalty: 1_000_000,
        same_club_penalty: 100_000,
        rematch_penalty: 500_000,
        maximum_entrant_count,
    }
}

fn entrant(id: &str) -> PairingEntrant {
    PairingEntrant {
        entrant_id: EntrantId::new(id),
        club_id: ClubId::new(format!("club-{id}")),
        starting_elo: EloRating::new(1500),
        performance_score: PerformanceScore::ZERO,
        matches_won: 0,
        opponent_score_sum: PerformanceScore::ZERO,
        bye_count: 0,
    }
}

fn request(ids: &[&str]) -> PairingRequest {
    PairingRequest {
        round_number: round(1),
        entrants: ids.iter().map(|id| entrant(id)).collect(),
        previous_matches: Vec::new(),
        policy: policy(64),
    }
}

#[test]
fn accepts_an_immutable_round_one_snapshot_with_zero_scores() {
    let request = request(&["a", "b", "c"]);

    validate_request(&request).unwrap();

    assert!(request.entrants.iter().all(|entrant| {
        entrant.performance_score == PerformanceScore::ZERO
            && entrant.opponent_score_sum == PerformanceScore::ZERO
    }));
}

#[test]
fn rejects_too_few_or_too_many_entrants() {
    assert!(matches!(
        validate_request(&request(&["only"])),
        Err(BlossomPairingError::NotEnoughEntrants { entrant_count: 1 })
    ));

    let mut limited = request(&["a", "b", "c"]);
    limited.policy.maximum_entrant_count = 2;
    assert!(matches!(
        validate_request(&limited),
        Err(BlossomPairingError::EntrantLimitExceeded {
            entrant_count: 3,
            maximum: 2
        })
    ));
}

#[test]
fn rejects_duplicate_entrants() {
    let request = request(&["a", "a"]);

    assert!(matches!(
        validate_request(&request),
        Err(BlossomPairingError::DuplicateEntrant { .. })
    ));
}

#[test]
fn rejects_unknown_or_self_matched_history_entrants() {
    let mut unknown = request(&["a", "b"]);
    unknown.previous_matches.push(PreviousMatch {
        first_entrant_id: EntrantId::new("a"),
        second_entrant_id: EntrantId::new("unknown"),
        round_number: round(1),
    });
    assert!(matches!(
        validate_request(&unknown),
        Err(BlossomPairingError::UnknownEntrantInHistory { .. })
    ));

    let mut self_match = request(&["a", "b"]);
    self_match.previous_matches.push(PreviousMatch {
        first_entrant_id: EntrantId::new("a"),
        second_entrant_id: EntrantId::new("a"),
        round_number: round(1),
    });
    assert!(matches!(
        validate_request(&self_match),
        Err(BlossomPairingError::SelfMatchInHistory { .. })
    ));
}

#[test]
fn rejects_history_from_a_later_round() {
    let mut request = request(&["a", "b"]);
    request.previous_matches.push(PreviousMatch {
        first_entrant_id: EntrantId::new("a"),
        second_entrant_id: EntrantId::new("b"),
        round_number: round(2),
    });

    assert!(matches!(
        validate_request(&request),
        Err(BlossomPairingError::InvalidHistoryRound { .. })
    ));
}

#[test]
fn checked_round_number_rejects_zero_and_overflow() {
    assert_eq!(RoundNumber::try_from(0_i64), Err(RoundNumberError));
    assert_eq!(
        RoundNumber::try_from(i64::from(u16::MAX) + 1),
        Err(RoundNumberError)
    );
}
