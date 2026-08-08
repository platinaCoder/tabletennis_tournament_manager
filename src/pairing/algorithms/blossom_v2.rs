//! Record-first pairing policy using the shared in-house Blossom kernel.
//!
//! V2 retains the stable snapshot boundary and relationship relaxation from
//! BlossomV1 while replacing its linear competitive edge costs.

mod edge_cost;
mod policy;
mod request;
mod solve;

pub use policy::BlossomV2Policy;
pub use request::PairingRequest;
pub use solve::{build_candidate_graph, build_relaxation_graphs, propose_pairings};

pub use super::blossom_v1::{
    BlossomPairingError, ClubId, PairingCandidateEdge, PairingCandidateGraph, PairingCost,
    PairingCostBreakdown, PairingCostComponent, PairingDiagnostics, PairingEdgeTarget,
    PairingEntrant, PairingPolicyVersion, PairingProposal, PairingWarning, PerformanceScore,
    PreviousMatch, ProposedBye, ProposedMatch, RelaxationTier, RoundNumber, RoundNumberError,
};

#[cfg(test)]
mod seeded_regression_tests;
#[cfg(test)]
mod tests;
