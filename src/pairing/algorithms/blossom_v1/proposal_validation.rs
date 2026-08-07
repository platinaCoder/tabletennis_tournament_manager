use std::collections::{HashMap, HashSet};

use crate::identity::EntrantId;

use super::blossom_feasibility::maximum_cardinality_matching;
use super::solver_graph::SolverGraph;
use super::{
    BlossomPairingError, InvalidSolverOutputReason, PairingCandidateEdge, PairingCandidateGraph,
    PairingEdgeTarget, PairingEntrant, PairingRequest, ProposedBye, RelaxationTier,
};

pub(super) fn validate_edge(
    request: &PairingRequest,
    tier: RelaxationTier,
    edge: &PairingCandidateEdge,
    entrants: &HashMap<&str, &PairingEntrant>,
) -> Result<(), BlossomPairingError> {
    if !entrants.contains_key(edge.first_entrant_id.as_str()) {
        return Err(invalid(InvalidSolverOutputReason::UnknownEdge));
    }
    if edge.breakdown.total != edge.cost.value() {
        return Err(invalid(InvalidSolverOutputReason::InconsistentEdgeCost));
    }
    if request.policy.avoid_same_club && edge.same_club && !tier.allows_same_club() {
        return Err(invalid(InvalidSolverOutputReason::ForbiddenSameClubPairing));
    }
    if request.policy.avoid_rematches && edge.rematch && !tier.allows_rematches() {
        return Err(invalid(InvalidSolverOutputReason::ForbiddenRematch));
    }
    if let PairingEdgeTarget::Entrant(second) = &edge.target {
        if edge.first_entrant_id == *second {
            return Err(invalid(InvalidSolverOutputReason::SelfPair));
        }
        if !entrants.contains_key(second.as_str()) {
            return Err(invalid(InvalidSolverOutputReason::UnknownEdge));
        }
    }
    Ok(())
}

pub(super) fn mark_once(
    seen: &mut HashSet<EntrantId>,
    entrant_id: &EntrantId,
) -> Result<(), BlossomPairingError> {
    if seen.insert(entrant_id.clone()) {
        Ok(())
    } else {
        Err(invalid(InvalidSolverOutputReason::DuplicateEntrant))
    }
}

pub(super) fn validate_coverage(
    request: &PairingRequest,
    seen: &HashSet<EntrantId>,
    bye: Option<&ProposedBye>,
) -> Result<(), BlossomPairingError> {
    if request
        .entrants
        .iter()
        .any(|entrant| !seen.contains(&entrant.entrant_id))
    {
        return Err(invalid(InvalidSolverOutputReason::MissingEntrant));
    }
    match (request.entrants.len() % 2, bye) {
        (0, Some(_)) => Err(invalid(InvalidSolverOutputReason::UnexpectedBye)),
        (1, None) => Err(invalid(InvalidSolverOutputReason::MissingEntrant)),
        _ => Ok(()),
    }
}

pub(super) fn validate_bye_fairness(
    request: &PairingRequest,
    graph: &PairingCandidateGraph,
    selected_bye: &ProposedBye,
) -> Result<(), BlossomPairingError> {
    let selected_count = request
        .entrants
        .iter()
        .find(|entrant| entrant.entrant_id == selected_bye.entrant_id)
        .map_or(0, |entrant| entrant.bye_count);
    let avoidable = request.entrants.iter().any(|entrant| {
        entrant.bye_count < selected_count
            && has_bye_edge(graph, &entrant.entrant_id)
            && remainder_has_complete_matching(graph, &entrant.entrant_id)
    });

    if avoidable {
        Err(invalid(InvalidSolverOutputReason::AvoidableRepeatedBye))
    } else {
        Ok(())
    }
}

fn has_bye_edge(graph: &PairingCandidateGraph, entrant_id: &EntrantId) -> bool {
    graph
        .edges
        .iter()
        .any(|edge| edge.first_entrant_id == *entrant_id && edge.target == PairingEdgeTarget::Bye)
}

fn remainder_has_complete_matching(graph: &PairingCandidateGraph, bye: &EntrantId) -> bool {
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
        .map(|solver_graph| maximum_cardinality_matching(&solver_graph).is_complete())
        .unwrap_or(false)
}

const fn invalid(reason: InvalidSolverOutputReason) -> BlossomPairingError {
    BlossomPairingError::InvalidSolverOutput { reason }
}
