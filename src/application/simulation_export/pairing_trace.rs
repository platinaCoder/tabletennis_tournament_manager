use std::collections::HashSet;

use crate::pairing::algorithms::blossom_v1::{
    BlossomPairingError, PairingCandidateEdge, PairingCandidateGraph, PairingCostBreakdown,
    PairingDiagnostics, PairingEdgeTarget, PairingEntrant, PairingPolicyVersion, PairingProposal,
    PairingWarning, PreviousMatch, RelaxationTier, RoundNumber,
};
use crate::pairing::algorithms::{PairingSnapshot, blossom_v1, blossom_v2};

use super::model::{
    CandidateEdgeTargetTrace, CandidateEdgeTrace, CandidateGraphTrace, PairingCalculationTrace,
    PairingCostBreakdownTrace, PairingDiagnosticsTrace, PairingEntrantTrace, PairingPolicyTrace,
    PairingProposalTrace, PairingRequestTrace, PairingWarningTrace, PreviousMatchTrace,
    ProposedByeTrace, ProposedMatchTrace,
};

pub(super) fn pairing_calculation(
    request: &PairingSnapshot,
    proposal: &PairingProposal,
) -> Result<PairingCalculationTrace, BlossomPairingError> {
    let selected_matches = proposal
        .matches
        .iter()
        .map(|pairing| ordered_pair(&pairing.first_entrant_id, &pairing.second_entrant_id))
        .collect::<HashSet<_>>();
    let selected_bye = proposal.bye.as_ref().map(|bye| bye.entrant_id.as_str());
    let graphs = relaxation_graphs(request)?
        .iter()
        .map(|graph| candidate_graph(graph, proposal, &selected_matches, selected_bye))
        .collect();

    Ok(PairingCalculationTrace {
        request: pairing_request(request),
        relaxation_graphs: graphs,
        proposal: pairing_proposal(proposal),
    })
}

fn relaxation_graphs(
    request: &PairingSnapshot,
) -> Result<Vec<PairingCandidateGraph>, BlossomPairingError> {
    match request {
        PairingSnapshot::BlossomV1(request) => blossom_v1::build_relaxation_graphs(request),
        PairingSnapshot::BlossomV2(request) => blossom_v2::build_relaxation_graphs(request),
    }
}

fn pairing_request(request: &PairingSnapshot) -> PairingRequestTrace {
    match request {
        PairingSnapshot::BlossomV1(request) => pairing_request_parts(
            request.round_number,
            &request.entrants,
            &request.previous_matches,
            pairing_policy_v1(&request.policy),
        ),
        PairingSnapshot::BlossomV2(request) => pairing_request_parts(
            request.round_number,
            &request.entrants,
            &request.previous_matches,
            pairing_policy_v2(&request.policy),
        ),
    }
}

fn pairing_request_parts(
    round_number: RoundNumber,
    entrants: &[PairingEntrant],
    previous_matches: &[PreviousMatch],
    policy: PairingPolicyTrace,
) -> PairingRequestTrace {
    PairingRequestTrace {
        round_number: round_number.value(),
        entrants: entrants
            .iter()
            .map(|entrant| PairingEntrantTrace {
                entrant_id: entrant.entrant_id.as_str().to_owned(),
                club_id: entrant.club_id.as_str().to_owned(),
                starting_elo: entrant.starting_elo.value(),
                performance_score_scaled: entrant.performance_score.scaled_value(),
                matches_won: entrant.matches_won,
                opponent_score_sum_scaled: entrant.opponent_score_sum.scaled_value(),
                bye_count: entrant.bye_count,
            })
            .collect(),
        previous_matches: previous_matches
            .iter()
            .map(|previous| PreviousMatchTrace {
                first_entrant_id: previous.first_entrant_id.as_str().to_owned(),
                second_entrant_id: previous.second_entrant_id.as_str().to_owned(),
                round_number: previous.round_number.value(),
            })
            .collect(),
        policy,
    }
}

fn pairing_policy_v1(policy: &blossom_v1::BlossomV1Policy) -> PairingPolicyTrace {
    PairingPolicyTrace {
        version: "blossom_v1".to_owned(),
        avoid_same_club: policy.avoid_same_club,
        avoid_rematches: policy.avoid_rematches,
        recent_rematch_window: policy.recent_rematch_window,
        performance_score_weight: policy.performance_score_weight,
        performance_score_normalization: 1,
        match_win_weight: Some(policy.match_win_weight),
        match_record_weight: None,
        opponent_strength_weight: policy.opponent_strength_weight,
        opponent_strength_normalization: 1,
        elo_difference_weight: Some(policy.elo_difference_weight),
        squared_elo_difference_weight: None,
        bye_repeat_penalty: policy.bye_repeat_penalty,
        same_club_penalty: policy.same_club_penalty,
        rematch_penalty: policy.rematch_penalty,
        maximum_entrant_count: policy.maximum_entrant_count,
    }
}

fn pairing_policy_v2(policy: &blossom_v2::BlossomV2Policy) -> PairingPolicyTrace {
    PairingPolicyTrace {
        version: "blossom_v2".to_owned(),
        avoid_same_club: policy.avoid_same_club,
        avoid_rematches: policy.avoid_rematches,
        recent_rematch_window: policy.recent_rematch_window,
        performance_score_weight: policy.performance_score_weight,
        performance_score_normalization: 1_000,
        match_win_weight: None,
        match_record_weight: Some(policy.match_record_weight),
        opponent_strength_weight: policy.opponent_strength_weight,
        opponent_strength_normalization: 1_000,
        elo_difference_weight: None,
        squared_elo_difference_weight: Some(policy.squared_elo_difference_weight),
        bye_repeat_penalty: policy.bye_repeat_penalty,
        same_club_penalty: policy.same_club_penalty,
        rematch_penalty: policy.rematch_penalty,
        maximum_entrant_count: policy.maximum_entrant_count,
    }
}

fn candidate_graph(
    graph: &PairingCandidateGraph,
    proposal: &PairingProposal,
    selected_matches: &HashSet<(&str, &str)>,
    selected_bye: Option<&str>,
) -> CandidateGraphTrace {
    CandidateGraphTrace {
        relaxation_tier: relaxation_tier(graph.relaxation_tier).to_owned(),
        entrant_ids: graph
            .entrant_ids
            .iter()
            .map(|id| id.as_str().to_owned())
            .collect(),
        edges: graph
            .edges
            .iter()
            .map(|edge| {
                candidate_edge(
                    edge,
                    graph.relaxation_tier == proposal.relaxation_tier,
                    selected_matches,
                    selected_bye,
                )
            })
            .collect(),
        diagnostics: diagnostics(&graph.diagnostics),
    }
}

fn candidate_edge(
    edge: &PairingCandidateEdge,
    successful_tier: bool,
    selected_matches: &HashSet<(&str, &str)>,
    selected_bye: Option<&str>,
) -> CandidateEdgeTrace {
    let (target, selected) = match &edge.target {
        PairingEdgeTarget::Entrant(second) => {
            let pair = ordered_pair(&edge.first_entrant_id, second);
            (
                CandidateEdgeTargetTrace::Entrant {
                    entrant_id: second.as_str().to_owned(),
                },
                successful_tier && selected_matches.contains(&pair),
            )
        }
        PairingEdgeTarget::Bye => (
            CandidateEdgeTargetTrace::Bye,
            successful_tier && selected_bye == Some(edge.first_entrant_id.as_str()),
        ),
    };
    CandidateEdgeTrace {
        first_entrant_id: edge.first_entrant_id.as_str().to_owned(),
        target,
        same_club: edge.same_club,
        rematch: edge.rematch,
        selected,
        cost: edge.cost.value(),
        breakdown: cost_breakdown(&edge.breakdown),
    }
}

fn pairing_proposal(proposal: &PairingProposal) -> PairingProposalTrace {
    PairingProposalTrace {
        policy_version: policy_version(proposal.policy_version).to_owned(),
        relaxation_tier: relaxation_tier(proposal.relaxation_tier).to_owned(),
        total_cost: proposal.total_cost.value(),
        matches: proposal
            .matches
            .iter()
            .map(|pairing| ProposedMatchTrace {
                first_entrant_id: pairing.first_entrant_id.as_str().to_owned(),
                second_entrant_id: pairing.second_entrant_id.as_str().to_owned(),
                cost: cost_breakdown(&pairing.cost),
            })
            .collect(),
        bye: proposal.bye.as_ref().map(|bye| ProposedByeTrace {
            entrant_id: bye.entrant_id.as_str().to_owned(),
            cost: cost_breakdown(&bye.cost),
        }),
        warnings: proposal.warnings.iter().map(warning).collect(),
        diagnostics: diagnostics(&proposal.diagnostics),
    }
}

fn warning(warning: &PairingWarning) -> PairingWarningTrace {
    match warning {
        PairingWarning::SameClubPairingRequired {
            first_entrant_id,
            second_entrant_id,
        } => PairingWarningTrace::SameClubPairingRequired {
            first_entrant_id: first_entrant_id.as_str().to_owned(),
            second_entrant_id: second_entrant_id.as_str().to_owned(),
        },
        PairingWarning::RematchRequired {
            first_entrant_id,
            second_entrant_id,
        } => PairingWarningTrace::RematchRequired {
            first_entrant_id: first_entrant_id.as_str().to_owned(),
            second_entrant_id: second_entrant_id.as_str().to_owned(),
        },
        PairingWarning::ByeAssigned { entrant_id } => PairingWarningTrace::ByeAssigned {
            entrant_id: entrant_id.as_str().to_owned(),
        },
        PairingWarning::RelaxedPairingRequired { tier } => {
            PairingWarningTrace::RelaxedPairingRequired {
                tier: relaxation_tier(*tier).to_owned(),
            }
        }
    }
}

fn cost_breakdown(cost: &PairingCostBreakdown) -> PairingCostBreakdownTrace {
    PairingCostBreakdownTrace {
        performance_score_gap: cost.performance_score_gap,
        match_win_gap: cost.match_win_gap,
        opponent_strength_gap: cost.opponent_strength_gap,
        elo_gap: cost.elo_gap,
        same_club_penalty: cost.same_club_penalty,
        rematch_penalty: cost.rematch_penalty,
        bye_penalty: cost.bye_penalty,
        deterministic_tie_break: cost.deterministic_tie_break,
        total: cost.total,
    }
}

fn diagnostics(value: &PairingDiagnostics) -> PairingDiagnosticsTrace {
    PairingDiagnosticsTrace {
        candidate_pair_count: value.candidate_pair_count,
        eligible_edge_count: value.eligible_edge_count,
        rejected_same_club_edges: value.rejected_same_club_edges,
        rejected_rematch_edges: value.rejected_rematch_edges,
        edge_generation_microseconds: value.edge_generation_duration.as_micros(),
        cost_calculation_microseconds: value.cost_calculation_duration.as_micros(),
        solver_microseconds: value.solver_duration.as_micros(),
        validation_microseconds: value.validation_duration.as_micros(),
    }
}

fn ordered_pair<'a>(
    first: &'a crate::identity::EntrantId,
    second: &'a crate::identity::EntrantId,
) -> (&'a str, &'a str) {
    if first.as_str() <= second.as_str() {
        (first.as_str(), second.as_str())
    } else {
        (second.as_str(), first.as_str())
    }
}

const fn relaxation_tier(tier: RelaxationTier) -> &'static str {
    match tier {
        RelaxationTier::Strict => "strict",
        RelaxationTier::SameClubAllowed => "same_club_allowed",
        RelaxationTier::RematchesAllowed => "rematches_allowed",
    }
}

const fn policy_version(version: PairingPolicyVersion) -> &'static str {
    match version {
        PairingPolicyVersion::BlossomV1 => "blossom_v1",
        PairingPolicyVersion::BlossomV2 => "blossom_v2",
    }
}
