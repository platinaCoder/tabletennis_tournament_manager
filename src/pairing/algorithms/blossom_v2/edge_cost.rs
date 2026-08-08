use super::super::blossom_v1::{
    BlossomPairingError, CostContext, PairingCost, PairingCostBreakdown, PairingCostComponent,
    PairingEdgeCostCalculator, PairingEntrant, finish_cost,
};
use super::PairingRequest;

const PERFORMANCE_SCORE_NORMALIZATION: u64 = 1_000;

pub(super) struct BlossomV2CostCalculator<'a> {
    request: &'a PairingRequest,
}

impl<'a> BlossomV2CostCalculator<'a> {
    pub(super) const fn new(request: &'a PairingRequest) -> Self {
        Self { request }
    }
}

impl PairingEdgeCostCalculator for BlossomV2CostCalculator<'_> {
    fn match_cost(
        &self,
        first: &PairingEntrant,
        second: &PairingEntrant,
        context: CostContext,
    ) -> Result<(PairingCost, PairingCostBreakdown), BlossomPairingError> {
        let later_round = self.request.round_number.value() > 1;
        let performance_score_gap = if later_round {
            normalized_weighted(
                first
                    .performance_score
                    .scaled_value()
                    .abs_diff(second.performance_score.scaled_value()),
                self.request.policy.performance_score_weight,
                PairingCostComponent::PerformanceScoreGap,
            )?
        } else {
            0
        };
        let match_win_gap = if later_round {
            squared_weighted_u16(
                first.matches_won.abs_diff(second.matches_won),
                self.request.policy.match_record_weight,
                PairingCostComponent::MatchWinGap,
            )?
        } else {
            0
        };
        let opponent_strength_gap = if later_round {
            normalized_weighted(
                first
                    .opponent_score_sum
                    .scaled_value()
                    .abs_diff(second.opponent_score_sum.scaled_value()),
                self.request.policy.opponent_strength_weight,
                PairingCostComponent::OpponentStrengthGap,
            )?
        } else {
            0
        };
        let elo_gap = squared_weighted_u32(
            first
                .starting_elo
                .value()
                .abs_diff(second.starting_elo.value()),
            self.request.policy.squared_elo_difference_weight,
            PairingCostComponent::EloGap,
        )?;

        finish_cost(
            PairingCostBreakdown {
                performance_score_gap,
                match_win_gap,
                opponent_strength_gap,
                elo_gap,
                same_club_penalty: u64::from(context.same_club)
                    .checked_mul(self.request.policy.same_club_penalty)
                    .ok_or(overflow(PairingCostComponent::SameClubPenalty))?,
                rematch_penalty: u64::from(context.rematch)
                    .checked_mul(self.request.policy.rematch_penalty)
                    .ok_or(overflow(PairingCostComponent::RematchPenalty))?,
                bye_penalty: 0,
                deterministic_tie_break: context.tie_break,
                total: 0,
            },
            context.tie_break_scale,
        )
    }

    fn bye_cost(
        &self,
        entrant: &PairingEntrant,
        tie_break: u64,
        tie_break_scale: u64,
    ) -> Result<(PairingCost, PairingCostBreakdown), BlossomPairingError> {
        let bye_penalty = u64::from(entrant.bye_count)
            .checked_mul(self.request.policy.bye_repeat_penalty)
            .ok_or(overflow(PairingCostComponent::ByePenalty))?;
        finish_cost(
            PairingCostBreakdown {
                performance_score_gap: 0,
                match_win_gap: 0,
                opponent_strength_gap: 0,
                elo_gap: 0,
                same_club_penalty: 0,
                rematch_penalty: 0,
                bye_penalty,
                deterministic_tie_break: tie_break,
                total: 0,
            },
            tie_break_scale,
        )
    }
}

fn normalized_weighted(
    gap: u64,
    weight: u32,
    component: PairingCostComponent,
) -> Result<u64, BlossomPairingError> {
    gap.checked_div(PERFORMANCE_SCORE_NORMALIZATION)
        .and_then(|normalized| normalized.checked_mul(u64::from(weight)))
        .ok_or(overflow(component))
}

fn squared_weighted_u16(
    gap: u16,
    weight: u64,
    component: PairingCostComponent,
) -> Result<u64, BlossomPairingError> {
    squared_weighted(u64::from(gap), weight, component)
}

fn squared_weighted_u32(
    gap: u32,
    weight: u32,
    component: PairingCostComponent,
) -> Result<u64, BlossomPairingError> {
    squared_weighted(u64::from(gap), u64::from(weight), component)
}

fn squared_weighted(
    gap: u64,
    weight: u64,
    component: PairingCostComponent,
) -> Result<u64, BlossomPairingError> {
    gap.checked_mul(gap)
        .and_then(|squared| squared.checked_mul(weight))
        .ok_or(overflow(component))
}

const fn overflow(component: PairingCostComponent) -> BlossomPairingError {
    BlossomPairingError::PairingCostOverflow { component }
}
