use std::collections::HashMap;

use crate::identity::{ClubId, EntrantId};
use crate::pairing::EloRating;
use crate::pairing::algorithms::blossom_v1;

use super::{
    BlossomV2Policy, PairingEntrant, PairingRequest, PerformanceScore, PreviousMatch, RoundNumber,
    propose_pairings,
};

#[test]
fn seeded_round_four_avoids_unearned_extreme_elo_pairings() {
    let request = seeded_round_four_request();
    let v1_request = blossom_v1::PairingRequest {
        round_number: request.round_number,
        entrants: request.entrants.clone(),
        previous_matches: request.previous_matches.clone(),
        policy: blossom_v1::BlossomV1Policy::default(),
    };

    let v1_proposal = blossom_v1::propose_pairings(&v1_request).unwrap();
    let v2_proposal = propose_pairings(&request).unwrap();

    assert_eq!(maximum_elo_gap(&v1_proposal, &request.entrants), 486);
    assert_eq!(maximum_elo_gap(&v2_proposal, &request.entrants), 143);
}

fn maximum_elo_gap(proposal: &blossom_v1::PairingProposal, entrants: &[PairingEntrant]) -> u32 {
    let elo_by_id = entrants
        .iter()
        .map(|entrant| (&entrant.entrant_id, entrant.starting_elo.value()))
        .collect::<HashMap<_, _>>();
    proposal
        .matches
        .iter()
        .map(|pairing| {
            elo_by_id[&pairing.first_entrant_id].abs_diff(elo_by_id[&pairing.second_entrant_id])
        })
        .max()
        .unwrap()
}

fn seeded_round_four_request() -> PairingRequest {
    PairingRequest {
        round_number: RoundNumber::try_from(4).unwrap(),
        entrants: ENTRANTS
            .iter()
            .map(
                |&(entrant_id, club_id, elo, performance, wins, opponent_score)| PairingEntrant {
                    entrant_id: EntrantId::new(entrant_id),
                    club_id: ClubId::new(club_id),
                    starting_elo: EloRating::new(elo),
                    performance_score: PerformanceScore::from_scaled(performance),
                    matches_won: wins,
                    opponent_score_sum: PerformanceScore::from_scaled(opponent_score),
                    bye_count: 0,
                },
            )
            .collect(),
        previous_matches: HISTORY
            .iter()
            .map(|&(first, second, round)| PreviousMatch {
                first_entrant_id: EntrantId::new(first),
                second_entrant_id: EntrantId::new(second),
                round_number: RoundNumber::try_from(round).unwrap(),
            })
            .collect(),
        policy: BlossomV2Policy::default(),
    }
}

const ENTRANTS: &[(&str, &str, u32, i64, u16, i64)] = &[
    ("entrant-000001", "club-000001", 900, -146_163, 1, 1_149_023),
    ("entrant-000002", "club-000002", 928, 582_731, 2, -1_074_559),
    ("entrant-000003", "club-000003", 957, -1_186_946, 0, -96_142),
    ("entrant-000004", "club-000004", 985, 258_550, 2, -750_378),
    (
        "entrant-000005",
        "club-000001",
        1_014,
        1_020_568,
        2,
        1_020_525,
    ),
    ("entrant-000006", "club-000002", 1_042, 307_742, 2, -231_275),
    (
        "entrant-000007",
        "club-000003",
        1_071,
        -1_105_680,
        0,
        -1_013_331,
    ),
    (
        "entrant-000008",
        "club-000004",
        1_100,
        -469_247,
        1,
        -654_460,
    ),
    ("entrant-000009", "club-000001", 1_128, -62_577, 1, 597_314),
    (
        "entrant-000010",
        "club-000002",
        1_157,
        -265_427,
        1,
        -737_404,
    ),
    ("entrant-000011", "club-000003", 1_185, 798_734, 2, 121_940),
    (
        "entrant-000012",
        "club-000004",
        1_214,
        -937_423,
        1,
        -653_639,
    ),
    (
        "entrant-000013",
        "club-000001",
        1_242,
        -569_348,
        1,
        -432_082,
    ),
    (
        "entrant-000014",
        "club-000002",
        1_271,
        -225_431,
        1,
        -528_928,
    ),
    ("entrant-000015", "club-000003", 1_300, 262_596, 2, 347_255),
    (
        "entrant-000016",
        "club-000004",
        1_328,
        1_182_030,
        3,
        1_347_171,
    ),
    ("entrant-000017", "club-000001", 1_357, -386_179, 1, 19_326),
    ("entrant-000018", "club-000002", 1_385, 64_007, 2, 733_274),
    ("entrant-000019", "club-000003", 1_414, 426_599, 2, 44_683),
    (
        "entrant-000020",
        "club-000004",
        1_442,
        -851_826,
        1,
        -498_331,
    ),
    (
        "entrant-000021",
        "club-000001",
        1_471,
        1_121_940,
        3,
        1_406_083,
    ),
    ("entrant-000022", "club-000002", 1_500, 180_750, 2, -116_065),
];

const HISTORY: &[(&str, &str, i64)] = &[
    ("entrant-000021", "entrant-000022", 1),
    ("entrant-000019", "entrant-000020", 1),
    ("entrant-000017", "entrant-000018", 1),
    ("entrant-000015", "entrant-000016", 1),
    ("entrant-000013", "entrant-000014", 1),
    ("entrant-000011", "entrant-000012", 1),
    ("entrant-000009", "entrant-000010", 1),
    ("entrant-000007", "entrant-000008", 1),
    ("entrant-000005", "entrant-000006", 1),
    ("entrant-000003", "entrant-000004", 1),
    ("entrant-000001", "entrant-000002", 1),
    ("entrant-000020", "entrant-000022", 2),
    ("entrant-000019", "entrant-000021", 2),
    ("entrant-000016", "entrant-000018", 2),
    ("entrant-000014", "entrant-000017", 2),
    ("entrant-000013", "entrant-000015", 2),
    ("entrant-000010", "entrant-000012", 2),
    ("entrant-000009", "entrant-000011", 2),
    ("entrant-000005", "entrant-000008", 2),
    ("entrant-000006", "entrant-000007", 2),
    ("entrant-000001", "entrant-000004", 2),
    ("entrant-000002", "entrant-000003", 2),
    ("entrant-000017", "entrant-000022", 3),
    ("entrant-000014", "entrant-000019", 3),
    ("entrant-000011", "entrant-000021", 3),
    ("entrant-000007", "entrant-000020", 3),
    ("entrant-000009", "entrant-000018", 3),
    ("entrant-000010", "entrant-000015", 3),
    ("entrant-000005", "entrant-000016", 3),
    ("entrant-000008", "entrant-000013", 3),
    ("entrant-000003", "entrant-000012", 3),
    ("entrant-000001", "entrant-000006", 3),
    ("entrant-000002", "entrant-000004", 3),
];
