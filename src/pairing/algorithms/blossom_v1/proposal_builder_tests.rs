use crate::identity::EntrantId;
use crate::pairing::EloRating;

use super::proposal_builder::build_proposal;
use super::reference_solver::solve_exactly;
use super::*;

fn request(ids: &[&str]) -> PairingRequest {
    PairingRequest {
        round_number: RoundNumber::try_from(1_i64).unwrap(),
        entrants: ids
            .iter()
            .enumerate()
            .map(|(index, id)| PairingEntrant {
                entrant_id: EntrantId::new(*id),
                club_id: ClubId::new(format!("club-{index}")),
                starting_elo: EloRating::new(1_500),
                performance_score: PerformanceScore::ZERO,
                matches_won: 0,
                opponent_score_sum: PerformanceScore::ZERO,
                bye_count: 0,
            })
            .collect(),
        previous_matches: Vec::new(),
        policy: BlossomV1Policy {
            avoid_same_club: true,
            avoid_rematches: true,
            recent_rematch_window: 3,
            performance_score_weight: 1,
            match_win_weight: 1,
            opponent_strength_weight: 1,
            elo_difference_weight: 1,
            bye_repeat_penalty: 10_000,
            same_club_penalty: 100,
            rematch_penalty: 1_000,
            maximum_entrant_count: 16,
        },
    }
}

#[test]
fn exact_selection_becomes_a_complete_deterministic_proposal() {
    let request = request(&["d", "b", "a", "c"]);
    let graph = build_candidate_graph(&request, RelaxationTier::Strict).unwrap();
    let matching = solve_exactly(&graph).unwrap().unwrap();

    let proposal = build_proposal(&request, &graph, &matching.edge_indices).unwrap();

    assert_eq!(proposal.matches.len(), 2);
    assert_eq!(proposal.bye, None);
    assert_eq!(proposal.total_cost.value(), matching.total_cost);
    assert_eq!(proposal.relaxation_tier, RelaxationTier::Strict);
    assert_eq!(proposal.policy_version, PairingPolicyVersion::BlossomV1);
    assert!(proposal.warnings.is_empty());
    assert!(
        proposal
            .matches
            .windows(2)
            .all(|pair| { pair[0].first_entrant_id.as_str() < pair[1].first_entrant_id.as_str() })
    );
}

#[test]
fn solver_edge_order_has_no_effect_on_proposal_order() {
    let request = request(&["a", "b", "c", "d"]);
    let graph = build_candidate_graph(&request, RelaxationTier::Strict).unwrap();
    let matching = solve_exactly(&graph).unwrap().unwrap();
    let mut reversed = matching.edge_indices.clone();
    reversed.reverse();

    let first = build_proposal(&request, &graph, &matching.edge_indices).unwrap();
    let second = build_proposal(&request, &graph, &reversed).unwrap();

    assert_eq!(first.matches, second.matches);
    assert_eq!(first.bye, second.bye);
    assert_eq!(first.total_cost, second.total_cost);
    assert_eq!(first.warnings, second.warnings);
}

#[test]
fn relaxed_same_club_pairing_produces_diagnostics_not_an_error() {
    let mut request = request(&["a", "b"]);
    request.entrants[1].club_id = request.entrants[0].club_id.clone();
    let graph = build_candidate_graph(&request, RelaxationTier::SameClubAllowed).unwrap();
    let matching = solve_exactly(&graph).unwrap().unwrap();

    let proposal = build_proposal(&request, &graph, &matching.edge_indices).unwrap();

    assert_eq!(
        proposal.warnings,
        vec![
            PairingWarning::RelaxedPairingRequired {
                tier: RelaxationTier::SameClubAllowed,
            },
            PairingWarning::SameClubPairingRequired {
                first_entrant_id: EntrantId::new("a"),
                second_entrant_id: EntrantId::new("b"),
            }
        ]
    );
}

#[test]
fn duplicate_entrant_from_solver_is_rejected() {
    let request = request(&["a", "b", "c", "d"]);
    let graph = build_candidate_graph(&request, RelaxationTier::Strict).unwrap();
    let first_edge = graph
        .edges
        .iter()
        .position(|edge| edge.target != PairingEdgeTarget::Bye)
        .unwrap();

    assert!(matches!(
        build_proposal(&request, &graph, &[first_edge, first_edge]),
        Err(BlossomPairingError::InvalidSolverOutput {
            reason: InvalidSolverOutputReason::DuplicateEntrant
        })
    ));
}

#[test]
fn avoidable_repeated_bye_is_rejected() {
    let mut request = request(&["a", "b", "c"]);
    request.entrants[0].bye_count = 1;
    let graph = build_candidate_graph(&request, RelaxationTier::Strict).unwrap();
    let repeated_bye = edge_index(&graph, "a", None);
    let remaining_match = edge_index(&graph, "b", Some("c"));

    assert!(matches!(
        build_proposal(&request, &graph, &[repeated_bye, remaining_match]),
        Err(BlossomPairingError::InvalidSolverOutput {
            reason: InvalidSolverOutputReason::AvoidableRepeatedBye
        })
    ));
}

fn edge_index(graph: &PairingCandidateGraph, first: &str, second: Option<&str>) -> usize {
    graph
        .edges
        .iter()
        .position(|edge| {
            edge.first_entrant_id.as_str() == first
                && match (&edge.target, second) {
                    (PairingEdgeTarget::Bye, None) => true,
                    (PairingEdgeTarget::Entrant(id), Some(second)) => id.as_str() == second,
                    _ => false,
                }
        })
        .unwrap()
}
