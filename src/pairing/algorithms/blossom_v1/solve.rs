use super::blossom_feasibility::maximum_cardinality_matching;
use super::bye_eligibility::retain_fairest_feasible_byes;
use super::proposal_builder::build_proposal;
use super::weighted_graph::WeightedSolverGraph;
use super::weighted_kernel::solve_minimum_cost;
use super::{
    BlossomPairingError, InvalidSolverOutputReason, PairingProposal, PairingRequest,
    RelaxationTier, build_candidate_graph,
};
use crate::platform_time::DiagnosticInstant;

/// Produces a complete minimum-cost pairing proposal at the strictest feasible
/// relaxation tier. Match identifiers and table assignments belong to later
/// application operations.
pub fn propose_pairings(request: &PairingRequest) -> Result<PairingProposal, BlossomPairingError> {
    let mut final_unmatched = Vec::new();

    for tier in RelaxationTier::ORDERED {
        let mut candidate_graph = build_candidate_graph(request, tier)?;
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
        return build_proposal(request, &candidate_graph, &selected);
    }

    Err(BlossomPairingError::NoCompleteMatching {
        final_tier: RelaxationTier::RematchesAllowed,
        unmatched_entrants: final_unmatched,
    })
}
