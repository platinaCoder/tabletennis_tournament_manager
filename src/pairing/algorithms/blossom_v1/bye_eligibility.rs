use std::collections::{HashMap, HashSet};

use crate::identity::EntrantId;

use super::blossom_feasibility::maximum_cardinality_matching;
use super::solver_graph::SolverGraph;
use super::{PairingCandidateGraph, PairingEdgeTarget, PairingRequest};

pub(super) fn retain_fairest_feasible_byes(
    request: &PairingRequest,
    graph: &mut PairingCandidateGraph,
) {
    if request.entrants.len().is_multiple_of(2) {
        return;
    }
    let bye_counts = request
        .entrants
        .iter()
        .map(|entrant| (entrant.entrant_id.as_str(), entrant.bye_count))
        .collect::<HashMap<_, _>>();
    let feasible = graph
        .edges
        .iter()
        .filter(|edge| edge.target == PairingEdgeTarget::Bye)
        .filter(|edge| remainder_is_matchable(graph, &edge.first_entrant_id))
        .map(|edge| edge.first_entrant_id.clone())
        .collect::<HashSet<_>>();
    let Some(minimum_bye_count) = feasible
        .iter()
        .filter_map(|entrant_id| bye_counts.get(entrant_id.as_str()))
        .copied()
        .min()
    else {
        graph
            .edges
            .retain(|edge| edge.target != PairingEdgeTarget::Bye);
        graph.diagnostics.eligible_edge_count = graph.edges.len();
        return;
    };

    graph.edges.retain(|edge| match &edge.target {
        PairingEdgeTarget::Entrant(_) => true,
        PairingEdgeTarget::Bye => {
            feasible.contains(&edge.first_entrant_id)
                && bye_counts.get(edge.first_entrant_id.as_str()) == Some(&minimum_bye_count)
        }
    });
    graph.diagnostics.eligible_edge_count = graph.edges.len();
}

fn remainder_is_matchable(graph: &PairingCandidateGraph, bye: &EntrantId) -> bool {
    let remainder = PairingCandidateGraph {
        relaxation_tier: graph.relaxation_tier,
        entrant_ids: graph
            .entrant_ids
            .iter()
            .filter(|entrant_id| *entrant_id != bye)
            .cloned()
            .collect(),
        edges: graph
            .edges
            .iter()
            .filter(|edge| {
                edge.first_entrant_id != *bye
                    && matches!(&edge.target, PairingEdgeTarget::Entrant(id) if id != bye)
            })
            .cloned()
            .collect(),
        diagnostics: graph.diagnostics.clone(),
    };
    SolverGraph::from_candidate_graph(&remainder)
        .map(|solver| maximum_cardinality_matching(&solver).is_complete())
        .unwrap_or(false)
}
