use super::super::blossom_v1;
use super::edge_cost::BlossomV2CostCalculator;
use super::request::compatibility_request;
use super::{
    BlossomPairingError, PairingCandidateGraph, PairingPolicyVersion, PairingProposal,
    PairingRequest, RelaxationTier,
};

pub fn build_candidate_graph(
    request: &PairingRequest,
    relaxation_tier: RelaxationTier,
) -> Result<PairingCandidateGraph, BlossomPairingError> {
    let compatibility = compatibility_request(request);
    blossom_v1::build_candidate_graph_with(
        &compatibility,
        relaxation_tier,
        &BlossomV2CostCalculator::new(request),
    )
}

pub fn build_relaxation_graphs(
    request: &PairingRequest,
) -> Result<Vec<PairingCandidateGraph>, BlossomPairingError> {
    let compatibility = compatibility_request(request);
    RelaxationTier::ORDERED
        .into_iter()
        .map(|tier| {
            let mut graph = build_candidate_graph(request, tier)?;
            blossom_v1::retain_fairest_feasible_byes(&compatibility, &mut graph);
            Ok(graph)
        })
        .collect()
}

pub fn propose_pairings(request: &PairingRequest) -> Result<PairingProposal, BlossomPairingError> {
    let compatibility = compatibility_request(request);
    blossom_v1::propose_pairings_with(&compatibility, PairingPolicyVersion::BlossomV2, |tier| {
        build_candidate_graph(request, tier)
    })
}
