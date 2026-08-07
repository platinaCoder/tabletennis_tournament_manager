use crate::identity::EntrantId;
use crate::pairing::EloRating;

use super::*;

fn round(number: i64) -> RoundNumber {
    RoundNumber::try_from(number).unwrap()
}

fn entrant(id: &str, club: &str, elo: u32) -> PairingEntrant {
    PairingEntrant {
        entrant_id: EntrantId::new(id),
        club_id: ClubId::new(club),
        starting_elo: EloRating::new(elo),
        performance_score: PerformanceScore::ZERO,
        matches_won: 0,
        opponent_score_sum: PerformanceScore::ZERO,
        bye_count: 0,
    }
}

fn policy() -> BlossomV1Policy {
    BlossomV1Policy {
        avoid_same_club: true,
        avoid_rematches: true,
        recent_rematch_window: 3,
        performance_score_weight: 10,
        match_win_weight: 20,
        opponent_strength_weight: 5,
        elo_difference_weight: 2,
        bye_repeat_penalty: 0,
        same_club_penalty: 1_000,
        rematch_penalty: 10_000,
        maximum_entrant_count: 64,
    }
}

fn request(entrants: Vec<PairingEntrant>) -> PairingRequest {
    PairingRequest {
        round_number: round(1),
        entrants,
        previous_matches: Vec::new(),
        policy: policy(),
    }
}

#[test]
fn public_operation_returns_a_valid_strict_proposal() {
    let request = request(vec![
        entrant("a", "one", 1_500),
        entrant("b", "two", 1_510),
        entrant("c", "three", 1_700),
        entrant("d", "four", 1_710),
    ]);

    let proposal = propose_pairings(&request).unwrap();

    assert_eq!(proposal.relaxation_tier, RelaxationTier::Strict);
    assert_eq!(proposal.matches.len(), 2);
    assert_eq!(proposal.bye, None);
    assert!(proposal.warnings.is_empty());
    assert_eq!(proposal.matches[0].first_entrant_id, EntrantId::new("a"));
    assert_eq!(proposal.matches[0].second_entrant_id, EntrantId::new("b"));
}

#[test]
fn public_operation_relaxes_same_club_before_rematches() {
    let request = request(
        ["a", "b", "c", "d"]
            .into_iter()
            .map(|id| entrant(id, "same", 1_500))
            .collect(),
    );

    let proposal = propose_pairings(&request).unwrap();

    assert_eq!(proposal.relaxation_tier, RelaxationTier::SameClubAllowed);
    assert!(matches!(
        proposal.warnings.first(),
        Some(PairingWarning::RelaxedPairingRequired {
            tier: RelaxationTier::SameClubAllowed
        })
    ));
}

#[test]
fn public_operation_reaches_rematch_tier_only_when_required() {
    let mut request = request(
        ["a", "b", "c", "d"]
            .into_iter()
            .enumerate()
            .map(|(index, id)| entrant(id, &format!("club-{index}"), 1_500))
            .collect(),
    );
    request.round_number = round(2);
    for first in 0..request.entrants.len() {
        for second in first + 1..request.entrants.len() {
            request.previous_matches.push(PreviousMatch {
                first_entrant_id: request.entrants[first].entrant_id.clone(),
                second_entrant_id: request.entrants[second].entrant_id.clone(),
                round_number: round(1),
            });
        }
    }

    let proposal = propose_pairings(&request).unwrap();

    assert_eq!(proposal.relaxation_tier, RelaxationTier::RematchesAllowed);
    assert_eq!(
        proposal
            .warnings
            .iter()
            .filter(|warning| matches!(warning, PairingWarning::RematchRequired { .. }))
            .count(),
        2
    );
}

#[test]
fn avoidable_repeated_bye_is_excluded_even_with_zero_penalty() {
    let mut repeated = entrant("a", "one", 1_500);
    repeated.bye_count = 1;
    let request = request(vec![
        repeated,
        entrant("b", "two", 1_500),
        entrant("c", "three", 1_500),
    ]);

    let proposal = propose_pairings(&request).unwrap();

    assert_ne!(proposal.bye.unwrap().entrant_id, EntrantId::new("a"));
}

#[test]
fn repeated_bye_is_allowed_when_every_first_bye_blocks_strict_matching() {
    let mut repeated = entrant("a", "one", 1_500);
    repeated.bye_count = 1;
    let mut request = request(vec![
        repeated,
        entrant("b", "two", 1_500),
        entrant("c", "three", 1_500),
    ]);
    request.round_number = round(2);
    request.previous_matches = vec![
        PreviousMatch {
            first_entrant_id: EntrantId::new("a"),
            second_entrant_id: EntrantId::new("b"),
            round_number: round(1),
        },
        PreviousMatch {
            first_entrant_id: EntrantId::new("a"),
            second_entrant_id: EntrantId::new("c"),
            round_number: round(1),
        },
    ];

    let proposal = propose_pairings(&request).unwrap();

    assert_eq!(proposal.relaxation_tier, RelaxationTier::Strict);
    assert_eq!(proposal.bye.unwrap().entrant_id, EntrantId::new("a"));
}

#[test]
fn odd_same_club_field_relaxes_instead_of_reporting_solver_failure() {
    let request = request(
        ["a", "b", "c"]
            .into_iter()
            .map(|id| entrant(id, "same", 1_500))
            .collect(),
    );

    let proposal = propose_pairings(&request).unwrap();

    assert_eq!(proposal.relaxation_tier, RelaxationTier::SameClubAllowed);
    assert_eq!(proposal.matches.len(), 1);
    assert!(proposal.bye.is_some());
}

#[test]
fn input_order_does_not_change_the_sporting_proposal() {
    let mut request = request(vec![
        entrant("a", "one", 1_500),
        entrant("b", "two", 1_520),
        entrant("c", "three", 1_610),
        entrant("d", "four", 1_630),
        entrant("e", "five", 1_700),
    ]);
    request.round_number = round(2);
    request.previous_matches = vec![PreviousMatch {
        first_entrant_id: EntrantId::new("a"),
        second_entrant_id: EntrantId::new("c"),
        round_number: round(1),
    }];
    let first = propose_pairings(&request).unwrap();

    request.entrants.reverse();
    request.previous_matches.reverse();
    let second = propose_pairings(&request).unwrap();

    assert_eq!(first.matches, second.matches);
    assert_eq!(first.bye, second.bye);
    assert_eq!(first.relaxation_tier, second.relaxation_tier);
    assert_eq!(first.total_cost, second.total_cost);
    assert_eq!(first.warnings, second.warnings);
    assert_eq!(
        first.diagnostics.candidate_pair_count,
        second.diagnostics.candidate_pair_count
    );
    assert_eq!(
        first.diagnostics.eligible_edge_count,
        second.diagnostics.eligible_edge_count
    );
}

#[test]
fn solver_handles_the_configured_sixty_four_entrant_limit() {
    let request = request(
        (0..64)
            .map(|index| {
                entrant(
                    &format!("entrant-{index:02}"),
                    &format!("club-{}", index % 12),
                    1_000 + u32::try_from(index).unwrap() * 10,
                )
            })
            .collect(),
    );

    let proposal = propose_pairings(&request).unwrap();

    assert_eq!(proposal.matches.len(), 32);
    assert_eq!(proposal.bye, None);
}

#[test]
fn solver_handles_a_sixty_three_entrant_field_with_one_bye() {
    let request = request(
        (0..63)
            .map(|index| {
                entrant(
                    &format!("entrant-{index:02}"),
                    &format!("club-{}", index % 12),
                    1_000 + u32::try_from(index).unwrap() * 10,
                )
            })
            .collect(),
    );

    let proposal = propose_pairings(&request).unwrap();

    assert_eq!(proposal.matches.len(), 31);
    assert!(proposal.bye.is_some());
}
