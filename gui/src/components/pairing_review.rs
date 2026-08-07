use std::collections::HashMap;

use yew::prelude::*;

use tabletennis_tournament::application::TournamentEntrant;
use tabletennis_tournament::pairing::algorithms::blossom_v1::{
    PairingCostBreakdown, PairingProposal, PairingWarning,
};

use crate::formatting::{compact_u64, duration, grouped_u64, relaxation_tier};
use crate::language::{Language, Text, use_language};

#[derive(Properties, PartialEq)]
pub struct PairingReviewProps {
    pub proposal: PairingProposal,
    pub entrants: Vec<TournamentEntrant>,
    pub on_publish: Callback<()>,
    pub on_recalculate: Callback<()>,
}

#[component]
pub fn PairingReview(props: &PairingReviewProps) -> Html {
    let language = use_language();
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
                        <p class="eyebrow">{language.text(Text::UnpublishedPairingPreview)}</p>
                        <h2>{language.pairing_heading(relaxation_tier(props.proposal.relaxation_tier, language))}</h2>
                    </div>
                    <div class="button-row">
                        <button class="secondary" onclick={recalculate}>{language.text(Text::Recalculate)}</button>
                        <button class="primary" onclick={publish}>{language.text(Text::PublishRound)}</button>
                    </div>
                </div>
                <div class="pairing-list">
                    {for props.proposal.matches.iter().map(|pairing| {
                        let first = entrants.get(&pairing.first_entrant_id);
                        let second = entrants.get(&pairing.second_entrant_id);
                        html! {
                            <article class="pairing-card" key={format!("{}-{}", pairing.first_entrant_id.as_str(), pairing.second_entrant_id.as_str())}>
                                <div class="contestants">
                                    {entrant_label(first.copied(), language)}
                                    <span class="versus">{language.text(Text::Versus)}</span>
                                    {entrant_label(second.copied(), language)}
                                </div>
                                <details>
                                    <summary>{language.text(Text::SelectionCost)}{" "}{cost_number(pairing.cost.total)}</summary>
                                    {cost_breakdown(&pairing.cost, language)}
                                </details>
                            </article>
                        }
                    })}
                    {props.proposal.bye.as_ref().map(|bye| html! {
                        <article class="pairing-card bye-card">
                            <strong>{language.text(Text::Bye)}{": "}{entrant_name(entrants.get(&bye.entrant_id).copied(), language)}</strong>
                            <span>{language.text(Text::Cost)}{" "}{cost_number(bye.cost.total)}</span>
                        </article>
                    }).unwrap_or_default()}
                </div>
            </section>
            <aside class="panel diagnostics-panel">
                <p class="eyebrow">{language.text(Text::DeveloperDiagnostics)}</p>
                <h2>{language.text(Text::PairingCalculation)}</h2>
                <dl class="diagnostics-grid">
                    <dt>{language.text(Text::RelaxationTier)}</dt><dd>{relaxation_tier(props.proposal.relaxation_tier, language)}</dd>
                    <dt>{language.text(Text::TotalCost)}</dt><dd>{cost_number(props.proposal.total_cost.value())}</dd>
                    <dt>{language.text(Text::CandidatePairs)}</dt><dd>{diagnostics.candidate_pair_count}</dd>
                    <dt>{language.text(Text::EligibleEdges)}</dt><dd>{diagnostics.eligible_edge_count}</dd>
                    <dt>{language.text(Text::SameClubRejected)}</dt><dd>{diagnostics.rejected_same_club_edges}</dd>
                    <dt>{language.text(Text::RematchesRejected)}</dt><dd>{diagnostics.rejected_rematch_edges}</dd>
                    <dt>{language.text(Text::EdgeGeneration)}</dt><dd>{duration(diagnostics.edge_generation_duration)}</dd>
                    <dt>{language.text(Text::CostCalculation)}</dt><dd>{duration(diagnostics.cost_calculation_duration)}</dd>
                    <dt>{language.text(Text::Solver)}</dt><dd>{duration(diagnostics.solver_duration)}</dd>
                    <dt>{language.text(Text::Validation)}</dt><dd>{duration(diagnostics.validation_duration)}</dd>
                </dl>
                <h3>{language.text(Text::Warnings)}</h3>
                if props.proposal.warnings.is_empty() {
                    <p class="success-text">{language.text(Text::NoRelaxationWarnings)}</p>
                } else {
                    <ul class="warning-list">
                        {for props.proposal.warnings.iter().map(|warning| html! {
                            <li>{warning_label(warning, &entrants, language)}</li>
                        })}
                    </ul>
                }
            </aside>
        </div>
    }
}

fn entrant_label(entrant: Option<&TournamentEntrant>, language: Language) -> Html {
    html! {
        <div class="entrant-label">
            <strong>{entrant_name(entrant, language)}</strong>
            <span>{entrant.map_or(language.text(Text::UnknownClub), |entrant| entrant.club_name.as_str())}</span>
            <small>{entrant.map_or_else(|| language.text(Text::EloUnavailable).to_owned(), |entrant| format!("ELO {}", entrant.starting_elo.value()))}</small>
        </div>
    }
}

fn cost_breakdown(cost: &PairingCostBreakdown, language: Language) -> Html {
    html! {
        <dl class="cost-grid">
            <dt>{language.text(Text::PerformanceComponent)}</dt><dd>{cost_number(cost.performance_score_gap)}</dd>
            <dt>{language.text(Text::MatchWinComponent)}</dt><dd>{cost_number(cost.match_win_gap)}</dd>
            <dt>{language.text(Text::OpponentComponent)}</dt><dd>{cost_number(cost.opponent_strength_gap)}</dd>
            <dt>{language.text(Text::EloComponent)}</dt><dd>{cost_number(cost.elo_gap)}</dd>
            <dt>{language.text(Text::SameClubPenalty)}</dt><dd>{cost_number(cost.same_club_penalty)}</dd>
            <dt>{language.text(Text::RematchPenalty)}</dt><dd>{cost_number(cost.rematch_penalty)}</dd>
            <dt>{language.text(Text::ByePenalty)}</dt><dd>{cost_number(cost.bye_penalty)}</dd>
            <dt>{language.text(Text::TieBreak)}</dt><dd>{cost_number(cost.deterministic_tie_break)}</dd>
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

fn entrant_name(entrant: Option<&TournamentEntrant>, language: Language) -> &str {
    entrant.map_or(language.text(Text::UnknownContestant), |entrant| {
        entrant.name.as_str()
    })
}

fn warning_label(
    warning: &PairingWarning,
    entrants: &HashMap<&tabletennis_tournament::identity::EntrantId, &TournamentEntrant>,
    language: Language,
) -> String {
    let name = |id| entrant_name(entrants.get(id).copied(), language);
    match warning {
        PairingWarning::SameClubPairingRequired {
            first_entrant_id,
            second_entrant_id,
        } => language.same_club_warning(name(first_entrant_id), name(second_entrant_id)),
        PairingWarning::RematchRequired {
            first_entrant_id,
            second_entrant_id,
        } => language.rematch_warning(name(first_entrant_id), name(second_entrant_id)),
        PairingWarning::ByeAssigned { entrant_id } => language.bye_warning(name(entrant_id)),
        PairingWarning::RelaxedPairingRequired { tier } => {
            language.relaxation_warning(relaxation_tier(*tier, language))
        }
    }
}
