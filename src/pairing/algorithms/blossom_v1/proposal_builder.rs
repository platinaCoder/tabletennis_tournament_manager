use crate::identity::EntrantId;
use crate::platform_time::DiagnosticInstant;
use std::collections::{HashMap, HashSet};

use super::proposal_validation::{
    mark_once, validate_bye_fairness, validate_coverage, validate_edge,
};
use super::{
    BlossomPairingError, InvalidSolverOutputReason, PairingCandidateGraph, PairingCost,
    PairingCostComponent, PairingEdgeTarget, PairingPolicyVersion, PairingProposal, PairingRequest,
    PairingWarning, ProposedBye, ProposedMatch, RelaxationTier,
};

pub(super) fn build_proposal(
    request: &PairingRequest,
    graph: &PairingCandidateGraph,
    selected_edge_indices: &[usize],
    policy_version: PairingPolicyVersion,
) -> Result<PairingProposal, BlossomPairingError> {
    let validation_started = DiagnosticInstant::now();
    let entrants = request
        .entrants
        .iter()
        .map(|entrant| (entrant.entrant_id.as_str(), entrant))
        .collect::<HashMap<_, _>>();
    let mut selected_matches = Vec::with_capacity(request.entrants.len() / 2);
    let mut selected_bye = None;
    let mut seen_entrants = HashSet::with_capacity(request.entrants.len());
    let mut total_cost = 0_u64;

    for edge_index in selected_edge_indices {
        let edge = graph
            .edges
            .get(*edge_index)
            .ok_or(invalid(InvalidSolverOutputReason::UnknownEdge))?;
        validate_edge(request, graph.relaxation_tier, edge, &entrants)?;
        mark_once(&mut seen_entrants, &edge.first_entrant_id)?;
        total_cost = total_cost.checked_add(edge.cost.value()).ok_or(
            BlossomPairingError::PairingCostOverflow {
                component: PairingCostComponent::Total,
            },
        )?;

        match &edge.target {
            PairingEdgeTarget::Entrant(second) => {
                mark_once(&mut seen_entrants, second)?;
                let (first, second) = ordered_ids(&edge.first_entrant_id, second);
                selected_matches.push((
                    ProposedMatch {
                        first_entrant_id: first,
                        second_entrant_id: second,
                        cost: edge.breakdown.clone(),
                    },
                    edge.same_club,
                    edge.rematch,
                ));
            }
            PairingEdgeTarget::Bye => {
                if selected_bye.is_some() {
                    return Err(invalid(InvalidSolverOutputReason::MultipleByes));
                }
                selected_bye = Some(ProposedBye {
                    entrant_id: edge.first_entrant_id.clone(),
                    cost: edge.breakdown.clone(),
                });
            }
        }
    }

    validate_coverage(request, &seen_entrants, selected_bye.as_ref())?;
    if let Some(bye) = &selected_bye {
        validate_bye_fairness(request, graph, bye)?;
    }

    selected_matches.sort_by(|(first, ..), (second, ..)| {
        (
            first.first_entrant_id.as_str(),
            first.second_entrant_id.as_str(),
        )
            .cmp(&(
                second.first_entrant_id.as_str(),
                second.second_entrant_id.as_str(),
            ))
    });
    let mut warnings = warnings_for(request, graph.relaxation_tier, &selected_matches);
    if let Some(bye) = &selected_bye {
        warnings.push(PairingWarning::ByeAssigned {
            entrant_id: bye.entrant_id.clone(),
        });
    }
    let mut diagnostics = graph.diagnostics.clone();
    diagnostics.validation_duration = validation_started.elapsed();

    Ok(PairingProposal {
        matches: selected_matches
            .into_iter()
            .map(|(pairing, ..)| pairing)
            .collect(),
        bye: selected_bye,
        relaxation_tier: graph.relaxation_tier,
        total_cost: PairingCost::new(total_cost),
        policy_version,
        warnings,
        diagnostics,
    })
}

fn warnings_for(
    request: &PairingRequest,
    tier: RelaxationTier,
    matches: &[(ProposedMatch, bool, bool)],
) -> Vec<PairingWarning> {
    let mut warnings = Vec::new();
    if tier != RelaxationTier::Strict {
        warnings.push(PairingWarning::RelaxedPairingRequired { tier });
    }
    for (pairing, same_club, rematch) in matches {
        if request.policy.avoid_same_club && *same_club {
            warnings.push(PairingWarning::SameClubPairingRequired {
                first_entrant_id: pairing.first_entrant_id.clone(),
                second_entrant_id: pairing.second_entrant_id.clone(),
            });
        }
        if request.policy.avoid_rematches && *rematch {
            warnings.push(PairingWarning::RematchRequired {
                first_entrant_id: pairing.first_entrant_id.clone(),
                second_entrant_id: pairing.second_entrant_id.clone(),
            });
        }
    }
    warnings
}

fn ordered_ids(first: &EntrantId, second: &EntrantId) -> (EntrantId, EntrantId) {
    if first.as_str() <= second.as_str() {
        (first.clone(), second.clone())
    } else {
        (second.clone(), first.clone())
    }
}

const fn invalid(reason: InvalidSolverOutputReason) -> BlossomPairingError {
    BlossomPairingError::InvalidSolverOutput { reason }
}
