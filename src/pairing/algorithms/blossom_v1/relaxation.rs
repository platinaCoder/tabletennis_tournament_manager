use super::blossom_feasibility::maximum_cardinality_matching;
use super::weighted_graph::WeightedSolverGraph;
use super::{
    BlossomPairingError, PairingCandidateGraph, PairingRequest, RelaxationTier,
    build_candidate_graph,
};
use crate::platform_time::DiagnosticInstant;

/// Builds candidate graphs in tier order and returns the first graph that has a
/// complete general-graph matching. Minimum-cost selection happens afterward.
pub fn build_first_feasible_graph(
    request: &PairingRequest,
) -> Result<PairingCandidateGraph, BlossomPairingError> {
    let mut final_unmatched = Vec::new();

    for tier in RelaxationTier::ORDERED {
        let mut candidate_graph = build_candidate_graph(request, tier)?;
        let solver_graph = WeightedSolverGraph::from_candidate_graph(&candidate_graph)?;
        let solver_started = DiagnosticInstant::now();
        let matching = maximum_cardinality_matching(solver_graph.unweighted());
        candidate_graph.diagnostics.solver_duration = solver_started.elapsed();

        if matching.is_complete() {
            return Ok(candidate_graph);
        }
        final_unmatched = matching.unmatched_entrants(&solver_graph.unweighted().entrant_ids);
    }

    Err(BlossomPairingError::NoCompleteMatching {
        final_tier: RelaxationTier::RematchesAllowed,
        unmatched_entrants: final_unmatched,
    })
}
