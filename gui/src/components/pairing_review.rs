use std::collections::HashMap;

use yew::prelude::*;

use tabletennis_tournament::application::TournamentEntrant;
use tabletennis_tournament::pairing::algorithms::blossom_v1::{
    PairingCostBreakdown, PairingProposal, PairingWarning,
};

use crate::formatting::{compact_u64, duration, grouped_u64, relaxation_tier};

#[derive(Properties, PartialEq)]
pub struct PairingReviewProps {
    pub proposal: PairingProposal,
    pub entrants: Vec<TournamentEntrant>,
    pub on_publish: Callback<()>,
    pub on_recalculate: Callback<()>,
}

#[component]
pub fn PairingReview(props: &PairingReviewProps) -> Html {
    let entrants = props
        .entrants
        .iter()
        .map(|entrant| (&entrant.entrant_id, entrant))
        .collect::<HashMap<_, _>>();
    let publish = emit_unit(props.on_publish.clone());
    let recalculate = emit_unit(props.on_recalculate.clone());
    let diagnostics = &props.proposal.diagnostics;

    html! {
        <div class="workspace-grid pairing-workspace">
            <section class="panel">
                <div class="section-heading">
                    <div>
                        <p class="eyebrow">{"Unpublished pairing preview"}</p>
                        <h2>{format!("{} pairings", relaxation_tier(props.proposal.relaxation_tier))}</h2>
                    </div>
                    <div class="button-row">
                        <button class="secondary" onclick={recalculate}>{"Recalculate"}</button>
                        <button class="primary" onclick={publish}>{"Publish round"}</button>
                    </div>
                </div>
                <div class="pairing-list">
                    {for props.proposal.matches.iter().map(|pairing| {
                        let first = entrants.get(&pairing.first_entrant_id);
                        let second = entrants.get(&pairing.second_entrant_id);
                        html! {
                            <article class="pairing-card" key={format!("{}-{}", pairing.first_entrant_id.as_str(), pairing.second_entrant_id.as_str())}>
                                <div class="contestants">
                                    {entrant_label(first.copied())}
                                    <span class="versus">{"vs"}</span>
                                    {entrant_label(second.copied())}
                                </div>
                                <details>
                                    <summary>{"Selection cost "}{cost_number(pairing.cost.total)}</summary>
                                    {cost_breakdown(&pairing.cost)}
                                </details>
                            </article>
                        }
                    })}
                    {props.proposal.bye.as_ref().map(|bye| html! {
                        <article class="pairing-card bye-card">
                            <strong>{"Bye: "}{entrant_name(entrants.get(&bye.entrant_id).copied())}</strong>
                            <span>{"Cost "}{cost_number(bye.cost.total)}</span>
                        </article>
                    }).unwrap_or_default()}
                </div>
            </section>
            <aside class="panel diagnostics-panel">
                <p class="eyebrow">{"Developer diagnostics"}</p>
                <h2>{"Pairing calculation"}</h2>
                <dl class="diagnostics-grid">
                    <dt>{"Relaxation tier"}</dt><dd>{relaxation_tier(props.proposal.relaxation_tier)}</dd>
                    <dt>{"Total cost"}</dt><dd>{cost_number(props.proposal.total_cost.value())}</dd>
                    <dt>{"Candidate pairs"}</dt><dd>{diagnostics.candidate_pair_count}</dd>
                    <dt>{"Eligible edges"}</dt><dd>{diagnostics.eligible_edge_count}</dd>
                    <dt>{"Same-club rejected"}</dt><dd>{diagnostics.rejected_same_club_edges}</dd>
                    <dt>{"Rematches rejected"}</dt><dd>{diagnostics.rejected_rematch_edges}</dd>
                    <dt>{"Edge generation"}</dt><dd>{duration(diagnostics.edge_generation_duration)}</dd>
                    <dt>{"Cost calculation"}</dt><dd>{duration(diagnostics.cost_calculation_duration)}</dd>
                    <dt>{"Solver"}</dt><dd>{duration(diagnostics.solver_duration)}</dd>
                    <dt>{"Validation"}</dt><dd>{duration(diagnostics.validation_duration)}</dd>
                </dl>
                <h3>{"Warnings"}</h3>
                if props.proposal.warnings.is_empty() {
                    <p class="success-text">{"No relaxation warnings."}</p>
                } else {
                    <ul class="warning-list">
                        {for props.proposal.warnings.iter().map(|warning| html! {
                            <li>{warning_label(warning, &entrants)}</li>
                        })}
                    </ul>
                }
            </aside>
        </div>
    }
}

fn entrant_label(entrant: Option<&TournamentEntrant>) -> Html {
    html! {
        <div class="entrant-label">
            <strong>{entrant_name(entrant)}</strong>
            <span>{entrant.map_or("Unknown club", |entrant| entrant.club_name.as_str())}</span>
            <small>{entrant.map_or_else(|| "ELO unavailable".to_owned(), |entrant| format!("ELO {}", entrant.starting_elo.value()))}</small>
        </div>
    }
}

fn cost_breakdown(cost: &PairingCostBreakdown) -> Html {
    html! {
        <dl class="cost-grid">
            <dt>{"Performance component"}</dt><dd>{cost_number(cost.performance_score_gap)}</dd>
            <dt>{"Match-win component"}</dt><dd>{cost_number(cost.match_win_gap)}</dd>
            <dt>{"Opponent component"}</dt><dd>{cost_number(cost.opponent_strength_gap)}</dd>
            <dt>{"ELO component"}</dt><dd>{cost_number(cost.elo_gap)}</dd>
            <dt>{"Same-club penalty"}</dt><dd>{cost_number(cost.same_club_penalty)}</dd>
            <dt>{"Rematch penalty"}</dt><dd>{cost_number(cost.rematch_penalty)}</dd>
            <dt>{"Bye penalty"}</dt><dd>{cost_number(cost.bye_penalty)}</dd>
            <dt>{"Tie-break"}</dt><dd>{cost_number(cost.deterministic_tie_break)}</dd>
        </dl>
    }
}

fn emit_unit(callback: Callback<()>) -> Callback<MouseEvent> {
    Callback::from(move |_| callback.emit(()))
}

fn cost_number(value: u64) -> Html {
    html! {
        <span class="metric-number" title={grouped_u64(value)}>{compact_u64(value)}</span>
    }
}

fn entrant_name(entrant: Option<&TournamentEntrant>) -> &str {
    entrant.map_or("Unknown contestant", |entrant| entrant.name.as_str())
}

fn warning_label(
    warning: &PairingWarning,
    entrants: &HashMap<&tabletennis_tournament::identity::EntrantId, &TournamentEntrant>,
) -> String {
    let name = |id| entrant_name(entrants.get(id).copied());
    match warning {
        PairingWarning::SameClubPairingRequired {
            first_entrant_id,
            second_entrant_id,
        } => format!(
            "Same-club pairing required: {} vs {}",
            name(first_entrant_id),
            name(second_entrant_id)
        ),
        PairingWarning::RematchRequired {
            first_entrant_id,
            second_entrant_id,
        } => format!(
            "Rematch required: {} vs {}",
            name(first_entrant_id),
            name(second_entrant_id)
        ),
        PairingWarning::ByeAssigned { entrant_id } => {
            format!("Bye assigned to {}", name(entrant_id))
        }
        PairingWarning::RelaxedPairingRequired { tier } => {
            format!("{} was required", relaxation_tier(*tier))
        }
    }
}
