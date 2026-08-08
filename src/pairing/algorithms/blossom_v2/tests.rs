use crate::identity::{ClubId, EntrantId};
use crate::pairing::EloRating;

use super::*;

fn entrant(id: &str, elo: u32, wins: u16, performance: i64) -> PairingEntrant {
    PairingEntrant {
        entrant_id: EntrantId::new(id),
        club_id: ClubId::new(format!("club-{id}")),
        starting_elo: EloRating::new(elo),
        performance_score: PerformanceScore::from_scaled(performance),
        matches_won: wins,
        opponent_score_sum: PerformanceScore::ZERO,
        bye_count: 0,
    }
}

fn request(round: i64, entrants: Vec<PairingEntrant>) -> PairingRequest {
    PairingRequest {
        round_number: RoundNumber::try_from(round).unwrap(),
        entrants,
        previous_matches: Vec::new(),
        policy: BlossomV2Policy::default(),
    }
}

#[test]
fn round_one_uses_squared_elo_distance() {
    let request = request(
        1,
        vec![entrant("a", 1_000, 0, 0), entrant("b", 1_200, 0, 0)],
    );
    let graph = build_candidate_graph(&request, RelaxationTier::Strict).unwrap();

    assert_eq!(graph.edges[0].breakdown.elo_gap, 400_000);
    assert_eq!(graph.edges[0].breakdown.match_win_gap, 0);
    assert_eq!(graph.edges[0].breakdown.performance_score_gap, 0);
}

#[test]
fn record_gap_dominates_extreme_relative_performance() {
    let request = request(
        3,
        vec![
            entrant("a", 1_000, 2, 2_000_000),
            entrant("b", 1_050, 1, 2_000_000),
        ],
    );
    let graph = build_candidate_graph(&request, RelaxationTier::Strict).unwrap();

    assert_eq!(graph.edges[0].breakdown.match_win_gap, 1_000_000_000);
    assert_eq!(graph.edges[0].breakdown.elo_gap, 25_000);
}

#[test]
fn proposal_reports_the_v2_policy_version() {
    let request = request(
        1,
        vec![
            entrant("a", 1_000, 0, 0),
            entrant("b", 1_050, 0, 0),
            entrant("c", 1_200, 0, 0),
            entrant("d", 1_250, 0, 0),
        ],
    );

    let proposal = propose_pairings(&request).unwrap();

    assert_eq!(proposal.policy_version, PairingPolicyVersion::BlossomV2);
}

#[test]
fn matching_by_record_can_earn_a_large_elo_gap() {
    let request = request(
        4,
        vec![
            entrant("low-undefeated", 900, 3, 1_000_000),
            entrant("low-one-win", 950, 1, -500_000),
            entrant("high-one-win", 1_450, 1, -500_000),
            entrant("high-undefeated", 1_500, 3, 1_000_000),
        ],
    );

    let proposal = propose_pairings(&request).unwrap();
    let pairs = proposal
        .matches
        .iter()
        .map(|pairing| {
            (
                pairing.first_entrant_id.as_str(),
                pairing.second_entrant_id.as_str(),
            )
        })
        .collect::<Vec<_>>();

    assert!(pairs.contains(&("high-undefeated", "low-undefeated")));
    assert!(pairs.contains(&("high-one-win", "low-one-win")));
}
