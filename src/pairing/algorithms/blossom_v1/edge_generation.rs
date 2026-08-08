use crate::identity::EntrantId;
use crate::platform_time::DiagnosticInstant;
use std::collections::HashSet;

use super::edge_cost::{BlossomV1CostCalculator, CostContext};
use super::{
    BlossomPairingError, PairingCost, PairingCostBreakdown, PairingCostComponent,
    PairingDiagnostics, PairingEntrant, PairingRequest, RelaxationTier,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PairingEdgeTarget {
    Entrant(EntrantId),
    Bye,
}

/// Stable-ID graph view suitable for diagnostics and future visualization.
/// Node indexes and mutable search state remain private to the solver kernel.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PairingCandidateEdge {
    pub first_entrant_id: EntrantId,
    pub target: PairingEdgeTarget,
    /// Relationship facts retained independently from policy penalty values.
    pub same_club: bool,
    pub rematch: bool,
    pub cost: PairingCost,
    pub breakdown: PairingCostBreakdown,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PairingCandidateGraph {
    pub relaxation_tier: RelaxationTier,
    pub entrant_ids: Vec<EntrantId>,
    pub edges: Vec<PairingCandidateEdge>,
    pub diagnostics: PairingDiagnostics,
}

pub(crate) trait PairingEdgeCostCalculator {
    fn match_cost(
        &self,
        first: &PairingEntrant,
        second: &PairingEntrant,
        context: CostContext,
    ) -> Result<(PairingCost, PairingCostBreakdown), BlossomPairingError>;

    fn bye_cost(
        &self,
        entrant: &PairingEntrant,
        tie_break: u64,
        tie_break_scale: u64,
    ) -> Result<(PairingCost, PairingCostBreakdown), BlossomPairingError>;
}

pub fn build_candidate_graph(
    request: &PairingRequest,
    relaxation_tier: RelaxationTier,
) -> Result<PairingCandidateGraph, BlossomPairingError> {
    build_candidate_graph_with(
        request,
        relaxation_tier,
        &BlossomV1CostCalculator::new(request),
    )
}

pub(crate) fn build_candidate_graph_with(
    request: &PairingRequest,
    relaxation_tier: RelaxationTier,
    cost_calculator: &impl PairingEdgeCostCalculator,
) -> Result<PairingCandidateGraph, BlossomPairingError> {
    super::validate_request(request)?;

    let edge_generation_started = DiagnosticInstant::now();
    let mut entrants = request.entrants.iter().collect::<Vec<_>>();
    entrants.sort_by_key(|entrant| entrant.entrant_id.as_str());

    let candidate_pair_count = candidate_pair_count(entrants.len())?;
    let recent_rematches = recent_rematch_index(request);
    let mut rejected_same_club_edges = 0_usize;
    let mut rejected_rematch_edges = 0_usize;
    let mut candidates = Vec::with_capacity(candidate_pair_count);

    for first_index in 0..entrants.len() {
        for second in &entrants[first_index + 1..] {
            let first = entrants[first_index];
            let same_club = first.club_id == second.club_id;
            let rematch =
                recent_rematches.contains(&entrant_pair_key(&first.entrant_id, &second.entrant_id));
            let same_club_forbidden =
                request.policy.avoid_same_club && same_club && !relaxation_tier.allows_same_club();
            let rematch_forbidden =
                request.policy.avoid_rematches && rematch && !relaxation_tier.allows_rematches();

            rejected_same_club_edges += usize::from(same_club_forbidden);
            rejected_rematch_edges += usize::from(rematch_forbidden);

            if !same_club_forbidden && !rematch_forbidden {
                candidates.push(CandidateEdge::Match {
                    first,
                    second,
                    same_club,
                    rematch,
                });
            }
        }
    }

    if entrants.len() % 2 == 1 {
        candidates.extend(
            entrants
                .iter()
                .copied()
                .map(|entrant| CandidateEdge::Bye { entrant }),
        );
    }
    candidates.sort_by(|first, second| first.key().cmp(&second.key()));

    let edge_generation_duration = edge_generation_started.elapsed();
    let cost_started = DiagnosticInstant::now();
    let tie_break_scale = tie_break_scale(candidates.len(), entrants.len())?;
    let edges = candidates
        .into_iter()
        .enumerate()
        .map(|(index, candidate)| {
            candidate.calculate(cost_calculator, u64_index(index)?, tie_break_scale)
        })
        .collect::<Result<Vec<_>, BlossomPairingError>>()?;
    let cost_calculation_duration = cost_started.elapsed();

    Ok(PairingCandidateGraph {
        relaxation_tier,
        entrant_ids: entrants
            .into_iter()
            .map(|entrant| entrant.entrant_id.clone())
            .collect(),
        diagnostics: PairingDiagnostics {
            candidate_pair_count,
            eligible_edge_count: edges.len(),
            rejected_same_club_edges,
            rejected_rematch_edges,
            edge_generation_duration,
            cost_calculation_duration,
            ..PairingDiagnostics::default()
        },
        edges,
    })
}

/// Builds the stable-ID candidate graph used at every relaxation tier.
///
/// Bye edges are filtered with the same fairness rule used by the production
/// solver. Private node indexes and solver state are intentionally absent.
pub fn build_relaxation_graphs(
    request: &PairingRequest,
) -> Result<Vec<PairingCandidateGraph>, BlossomPairingError> {
    RelaxationTier::ORDERED
        .into_iter()
        .map(|tier| {
            let mut graph = build_candidate_graph(request, tier)?;
            super::bye_eligibility::retain_fairest_feasible_byes(request, &mut graph);
            Ok(graph)
        })
        .collect()
}

enum CandidateEdge<'a> {
    Match {
        first: &'a PairingEntrant,
        second: &'a PairingEntrant,
        same_club: bool,
        rematch: bool,
    },
    Bye {
        entrant: &'a PairingEntrant,
    },
}

impl CandidateEdge<'_> {
    fn key(&self) -> (u8, &str, &str) {
        match self {
            Self::Match { first, second, .. } => {
                (0, first.entrant_id.as_str(), second.entrant_id.as_str())
            }
            Self::Bye { entrant } => (1, entrant.entrant_id.as_str(), ""),
        }
    }

    fn calculate(
        self,
        cost_calculator: &impl PairingEdgeCostCalculator,
        tie_break: u64,
        tie_break_scale: u64,
    ) -> Result<PairingCandidateEdge, BlossomPairingError> {
        match self {
            Self::Match {
                first,
                second,
                same_club,
                rematch,
            } => {
                let (cost, breakdown) = cost_calculator.match_cost(
                    first,
                    second,
                    CostContext {
                        tie_break,
                        tie_break_scale,
                        same_club,
                        rematch,
                    },
                )?;
                Ok(PairingCandidateEdge {
                    first_entrant_id: first.entrant_id.clone(),
                    target: PairingEdgeTarget::Entrant(second.entrant_id.clone()),
                    same_club,
                    rematch,
                    cost,
                    breakdown,
                })
            }
            Self::Bye { entrant } => {
                let (cost, breakdown) =
                    cost_calculator.bye_cost(entrant, tie_break, tie_break_scale)?;
                Ok(PairingCandidateEdge {
                    first_entrant_id: entrant.entrant_id.clone(),
                    target: PairingEdgeTarget::Bye,
                    same_club: false,
                    rematch: false,
                    cost,
                    breakdown,
                })
            }
        }
    }
}

fn candidate_pair_count(entrant_count: usize) -> Result<usize, BlossomPairingError> {
    let other_entrants =
        entrant_count
            .checked_sub(1)
            .ok_or(BlossomPairingError::PairingCostOverflow {
                component: PairingCostComponent::DeterministicTieBreak,
            })?;
    entrant_count
        .checked_mul(other_entrants)
        .and_then(|value| value.checked_div(2))
        .ok_or(BlossomPairingError::PairingCostOverflow {
            component: PairingCostComponent::DeterministicTieBreak,
        })
}

fn tie_break_scale(
    eligible_edge_count: usize,
    entrant_count: usize,
) -> Result<u64, BlossomPairingError> {
    let edge_count = u64_index(eligible_edge_count)?;
    let selected_edge_count = entrant_count
        .checked_add(1)
        .and_then(|value| value.checked_div(2))
        .ok_or(BlossomPairingError::PairingCostOverflow {
            component: PairingCostComponent::DeterministicTieBreak,
        })?;
    let selected_edge_count = u64_index(selected_edge_count)?;

    edge_count
        .checked_mul(selected_edge_count)
        .and_then(|value| value.checked_add(1))
        .ok_or(BlossomPairingError::PairingCostOverflow {
            component: PairingCostComponent::DeterministicTieBreak,
        })
}

fn u64_index(value: usize) -> Result<u64, BlossomPairingError> {
    u64::try_from(value).map_err(|_| BlossomPairingError::PairingCostOverflow {
        component: PairingCostComponent::DeterministicTieBreak,
    })
}

fn recent_rematch_index(request: &PairingRequest) -> HashSet<(&str, &str)> {
    if request.policy.recent_rematch_window == 0 {
        return HashSet::new();
    }

    request
        .previous_matches
        .iter()
        .filter(|previous_match| {
            let rounds_ago = request.round_number.value() - previous_match.round_number.value();
            rounds_ago <= request.policy.recent_rematch_window
        })
        .map(|previous_match| {
            entrant_pair_key(
                &previous_match.first_entrant_id,
                &previous_match.second_entrant_id,
            )
        })
        .collect()
}

fn entrant_pair_key<'a>(first: &'a EntrantId, second: &'a EntrantId) -> (&'a str, &'a str) {
    if first.as_str() <= second.as_str() {
        (first.as_str(), second.as_str())
    } else {
        (second.as_str(), first.as_str())
    }
}
