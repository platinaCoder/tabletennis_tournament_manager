//! Versioned minimum-cost general-graph pairing policy.
//!
//! This module accepts stable-ID tournament snapshots and returns validated
//! contestant pairings. Publication, match IDs, and table assignment remain
//! outside this boundary.

mod blossom_feasibility;
mod bye_eligibility;
mod diagnostics;
mod edge_cost;
mod edge_generation;
mod error;
mod policy;
mod proposal;
mod proposal_builder;
mod proposal_validation;
mod relaxation;
mod request;
mod solve;
mod solver_graph;
mod validation;
mod weighted_graph;
mod weighted_kernel;

pub use diagnostics::{PairingDiagnostics, PairingWarning, RelaxationTier};
pub use edge_generation::{
    PairingCandidateEdge, PairingCandidateGraph, PairingEdgeTarget, build_candidate_graph,
    build_relaxation_graphs,
};
pub use error::{
    BlossomPairingError, InvalidSolverOutputReason, PairingCostComponent, SolverError,
};
pub use policy::BlossomV1Policy;
pub use proposal::{
    PairingCost, PairingCostBreakdown, PairingPolicyVersion, PairingProposal, ProposedBye,
    ProposedMatch,
};
pub use relaxation::build_first_feasible_graph;
pub use request::{
    ClubId, PairingEntrant, PairingRequest, PerformanceScore, PreviousMatch, RoundNumber,
    RoundNumberError,
};
pub use solve::propose_pairings;
pub use validation::validate_request;

pub(crate) use bye_eligibility::retain_fairest_feasible_byes;
pub(crate) use edge_cost::{CostContext, finish_cost};
pub(crate) use edge_generation::{PairingEdgeCostCalculator, build_candidate_graph_with};
pub(crate) use solve::propose_pairings_with;

#[cfg(test)]
mod blossom_feasibility_tests;
#[cfg(test)]
mod edge_generation_tests;
#[cfg(test)]
mod proposal_builder_tests;
#[cfg(test)]
mod reference_solver;
#[cfg(test)]
mod reference_solver_tests;
#[cfg(test)]
mod solve_tests;
#[cfg(test)]
mod validation_tests;
#[cfg(test)]
mod weighted_graph_tests;
#[cfg(test)]
mod weighted_kernel_tests;
