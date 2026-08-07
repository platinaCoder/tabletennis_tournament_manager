use std::fmt::{self, Display, Formatter};

use crate::application::ContestantStanding;
use crate::identity::EntrantId;
use crate::pairing::EloRating;
use crate::pairing::algorithms::blossom_v1::{
    PairingCost, PairingDiagnostics, PairingWarning, ProposedMatch, RelaxationTier, RoundNumber,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SimulationRoundReport {
    pub round_number: RoundNumber,
    pub relaxation_tier: RelaxationTier,
    pub total_cost: PairingCost,
    pub pairings: Vec<ProposedMatch>,
    pub warnings: Vec<PairingWarning>,
    pub diagnostics: PairingDiagnostics,
    pub bye: Option<EntrantId>,
    pub match_count: usize,
    pub unassigned_match_count: usize,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SimulationReport {
    pub name: String,
    pub entrant_count: usize,
    pub configured_round_count: u16,
    pub minimum_starting_elo: EloRating,
    pub maximum_starting_elo: EloRating,
    pub completed_match_count: usize,
    pub rounds: Vec<SimulationRoundReport>,
    pub final_standings: Vec<ContestantStanding>,
}

impl Display for SimulationReport {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        writeln!(
            formatter,
            "{}: {} entrants, ELO {}-{}, {} rounds, {} completed matches",
            self.name,
            self.entrant_count,
            self.minimum_starting_elo.value(),
            self.maximum_starting_elo.value(),
            self.configured_round_count,
            self.completed_match_count
        )?;
        for round in &self.rounds {
            writeln!(
                formatter,
                "  round {}: {:?}, cost {}, {} matches, {} unassigned, solver {}us",
                round.round_number.value(),
                round.relaxation_tier,
                round.total_cost.value(),
                round.match_count,
                round.unassigned_match_count,
                round.diagnostics.solver_duration.as_micros()
            )?;
        }
        writeln!(formatter, "  final top three:")?;
        for (rank, standing) in self.final_standings.iter().take(3).enumerate() {
            writeln!(
                formatter,
                "    {}. {}: score {}, wins {}, opponent sum {}",
                rank + 1,
                standing.entrant_id.as_str(),
                standing.performance_score.scaled_value(),
                standing.matches_won,
                standing.opponent_score_sum.scaled_value()
            )?;
        }
        Ok(())
    }
}
