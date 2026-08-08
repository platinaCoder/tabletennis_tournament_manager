use super::blossom_feasibility::maximum_cardinality_matching;
use super::bye_eligibility::retain_fairest_feasible_byes;
use super::proposal_builder::build_proposal;
use super::weighted_graph::WeightedSolverGraph;
use super::weighted_kernel::solve_minimum_cost;
use super::{
    BlossomPairingError, InvalidSolverOutputReason, PairingCandidateGraph, PairingPolicyVersion,
    PairingProposal, PairingRequest, RelaxationTier, build_candidate_graph,
};
use crate::platform_time::DiagnosticInstant;

/// Produces a complete minimum-cost pairing proposal at the strictest feasible
/// relaxation tier. Match identifiers and table assignments belong to later
/// application operations.
pub fn propose_pairings(request: &PairingRequest) -> Result<PairingProposal, BlossomPairingError> {
    propose_pairings_with(request, PairingPolicyVersion::BlossomV1, |tier| {
        build_candidate_graph(request, tier)
    })
}

pub(crate) fn propose_pairings_with(
    request: &PairingRequest,
    policy_version: PairingPolicyVersion,
    mut build_graph: impl FnMut(RelaxationTier) -> Result<PairingCandidateGraph, BlossomPairingError>,
) -> Result<PairingProposal, BlossomPairingError> {
    let mut final_unmatched = Vec::new();

    for tier in RelaxationTier::ORDERED {
        let mut candidate_graph = build_graph(tier)?;
        retain_fairest_feasible_byes(request, &mut candidate_graph);
        let solver_graph = WeightedSolverGraph::from_candidate_graph(&candidate_graph)?;
        let solver_started = DiagnosticInstant::now();
        let feasibility = maximum_cardinality_matching(solver_graph.unweighted());
        if !feasibility.is_complete() {
            final_unmatched =
                feasibility.unmatched_entrants(&solver_graph.unweighted().entrant_ids);
            candidate_graph.diagnostics.solver_duration = solver_started.elapsed();
            continue;
        }

        let selected =
            solve_minimum_cost(&solver_graph)?.ok_or(BlossomPairingError::InvalidSolverOutput {
                reason: InvalidSolverOutputReason::MissingEntrant,
            })?;
        candidate_graph.diagnostics.solver_duration = solver_started.elapsed();
        return build_proposal(request, &candidate_graph, &selected, policy_version);
    }

    Err(BlossomPairingError::NoCompleteMatching {
        final_tier: RelaxationTier::RematchesAllowed,
        unmatched_entrants: final_unmatched,
    })
}
