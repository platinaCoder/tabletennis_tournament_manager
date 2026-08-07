use super::{
    BlossomPairingError, PairingCost, PairingCostBreakdown, PairingCostComponent, PairingEntrant,
    PairingRequest,
};

pub(super) struct CostContext {
    pub tie_break: u64,
    pub tie_break_scale: u64,
    pub same_club: bool,
    pub rematch: bool,
}

pub(super) fn match_cost(
    request: &PairingRequest,
    first: &PairingEntrant,
    second: &PairingEntrant,
    context: CostContext,
) -> Result<(PairingCost, PairingCostBreakdown), BlossomPairingError> {
    let later_round = request.round_number.value() > 1;
    let performance_score_gap = if later_round {
        weighted(
            first
                .performance_score
                .scaled_value()
                .abs_diff(second.performance_score.scaled_value()),
            request.policy.performance_score_weight,
            PairingCostComponent::PerformanceScoreGap,
        )?
    } else {
        0
    };
    let match_win_gap = if later_round {
        weighted(
            u64::from(first.matches_won.abs_diff(second.matches_won)),
            request.policy.match_win_weight,
            PairingCostComponent::MatchWinGap,
        )?
    } else {
        0
    };
    let opponent_strength_gap = if later_round {
        weighted(
            first
                .opponent_score_sum
                .scaled_value()
                .abs_diff(second.opponent_score_sum.scaled_value()),
            request.policy.opponent_strength_weight,
            PairingCostComponent::OpponentStrengthGap,
        )?
    } else {
        0
    };
    let elo_gap = weighted(
        u64::from(
            first
                .starting_elo
                .value()
                .abs_diff(second.starting_elo.value()),
        ),
        request.policy.elo_difference_weight,
        PairingCostComponent::EloGap,
    )?;
    let same_club_penalty = if context.same_club {
        request.policy.same_club_penalty
    } else {
        0
    };
    let rematch_penalty = if context.rematch {
        request.policy.rematch_penalty
    } else {
        0
    };

    finish_cost(
        PairingCostBreakdown {
            performance_score_gap,
            match_win_gap,
            opponent_strength_gap,
            elo_gap,
            same_club_penalty,
            rematch_penalty,
            bye_penalty: 0,
            deterministic_tie_break: context.tie_break,
            total: 0,
        },
        context.tie_break_scale,
    )
}

pub(super) fn bye_cost(
    request: &PairingRequest,
    entrant: &PairingEntrant,
    tie_break: u64,
    tie_break_scale: u64,
) -> Result<(PairingCost, PairingCostBreakdown), BlossomPairingError> {
    let bye_penalty = u64::from(entrant.bye_count)
        .checked_mul(request.policy.bye_repeat_penalty)
        .ok_or(BlossomPairingError::PairingCostOverflow {
            component: PairingCostComponent::ByePenalty,
        })?;

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

fn weighted(
    gap: u64,
    weight: u32,
    component: PairingCostComponent,
) -> Result<u64, BlossomPairingError> {
    gap.checked_mul(u64::from(weight))
        .ok_or(BlossomPairingError::PairingCostOverflow { component })
}

fn finish_cost(
    mut breakdown: PairingCostBreakdown,
    tie_break_scale: u64,
) -> Result<(PairingCost, PairingCostBreakdown), BlossomPairingError> {
    let base_total = [
        breakdown.performance_score_gap,
        breakdown.match_win_gap,
        breakdown.opponent_strength_gap,
        breakdown.elo_gap,
        breakdown.same_club_penalty,
        breakdown.rematch_penalty,
        breakdown.bye_penalty,
    ]
    .into_iter()
    .try_fold(0_u64, |total, component| total.checked_add(component))
    .ok_or(BlossomPairingError::PairingCostOverflow {
        component: PairingCostComponent::Total,
    })?;

    breakdown.total = base_total
        .checked_mul(tie_break_scale)
        .and_then(|total| total.checked_add(breakdown.deterministic_tie_break))
        .ok_or(BlossomPairingError::PairingCostOverflow {
            component: PairingCostComponent::Total,
        })?;

    Ok((PairingCost::new(breakdown.total), breakdown))
}
