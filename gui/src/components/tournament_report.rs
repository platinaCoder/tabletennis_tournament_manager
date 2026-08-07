use std::collections::HashMap;

use yew::prelude::*;

use tabletennis_tournament::application::{CompletedRound, ContestantStanding, TournamentEntrant};
use tabletennis_tournament::identity::EntrantId;
use tabletennis_tournament::results::MatchResult;

use crate::formatting::relaxation_tier;

use super::standings::Standings;

#[derive(Properties, PartialEq)]
pub struct TournamentReportProps {
    pub rounds: Vec<CompletedRound>,
    pub entrants: Vec<TournamentEntrant>,
    pub standings: Vec<ContestantStanding>,
    pub active_entrant_ids: Vec<EntrantId>,
}

#[component]
pub fn TournamentReport(props: &TournamentReportProps) -> Html {
    let show_matches = use_state(|| false);
    let toggle_matches = {
        let show_matches = show_matches.clone();
        Callback::from(move |_| show_matches.set(!*show_matches))
    };
    let match_count = props
        .rounds
        .iter()
        .map(|round| round.results.len())
        .sum::<usize>();
    let game_count = props
        .rounds
        .iter()
        .flat_map(|round| &round.results)
        .map(|result| result.games().len())
        .sum::<usize>();

    html! {
        <div class="report-stack">
            <section class="panel final-report">
                <div class="section-heading">
                    <div>
                        <p class="eyebrow">{"Tournament complete"}</p>
                        <h2>{"Final standings"}</h2>
                    </div>
                    <button class="secondary" onclick={toggle_matches}>
                        {if *show_matches { "Hide match results" } else { "Show match results" }}
                    </button>
                </div>
                <div class="summary-strip">
                    <span>{format!("{} rounds", props.rounds.len())}</span>
                    <span>{format!("{match_count} matches")}</span>
                    <span>{format!("{game_count} games")}</span>
                </div>
                <Standings
                    standings={props.standings.clone()}
                    entrants={props.entrants.clone()}
                    active_entrant_ids={props.active_entrant_ids.clone()}
                />
            </section>
            if *show_matches {
                <section class="panel match-report">
                    <p class="eyebrow">{"Complete tournament record"}</p>
                    <h2>{"Match results by round"}</h2>
                    {match_results(props)}
                </section>
            }
        </div>
    }
}

fn match_results(props: &TournamentReportProps) -> Html {
    let entrants = props
        .entrants
        .iter()
        .map(|entrant| (&entrant.entrant_id, entrant.name.as_str()))
        .collect::<HashMap<_, _>>();
    html! {
        <div class="report-rounds">
            {for props.rounds.iter().map(|round| round_results(round, &entrants))}
        </div>
    }
}

fn round_results(round: &CompletedRound, entrants: &HashMap<&EntrantId, &str>) -> Html {
    let results = round
        .results
        .iter()
        .map(|result| (result.match_id(), result))
        .collect::<HashMap<_, _>>();
    html! {
        <article class="report-round">
            <header>
                <h3>{format!("Round {}", round.round_number.value())}</h3>
                <span>{relaxation_tier(round.proposal.relaxation_tier)}</span>
            </header>
            <div class="report-match-list">
                {for round.scheduled_matches.iter().map(|scheduled| {
                    let result = results.get(&scheduled.match_id).copied();
                    let home = contestant_name(entrants, &scheduled.home_entrant_id);
                    let away = contestant_name(entrants, &scheduled.away_entrant_id);
                    let table = scheduled.table_number().map_or_else(
                        || "Unassigned".to_owned(),
                        |table| format!("Table {}", table.value()),
                    );
                    html! {
                        <div class="report-match" key={scheduled.match_id.as_str().to_owned()}>
                            <span class="table-badge">{table}</span>
                            <strong>{home}{" vs "}{away}</strong>
                            {result.map(result_score).unwrap_or_else(|| html! { <span>{"No result"}</span> })}
                        </div>
                    }
                })}
                {round.bye.as_ref().map(|bye| html! {
                    <div class="report-match report-bye">
                        <span class="table-badge">{"Bye"}</span>
                        <strong>{contestant_name(entrants, bye)}</strong>
                    </div>
                }).unwrap_or_default()}
            </div>
        </article>
    }
}

fn result_score(result: &MatchResult) -> Html {
    let games = result
        .games()
        .iter()
        .map(|game| format!("{}-{}", game.home_points.value(), game.away_points.value()))
        .collect::<Vec<_>>()
        .join(", ");
    html! {
        <span class="report-score">
            <strong>{format!("{}-{}", result.home_games_won().value(), result.away_games_won().value())}</strong>
            <small>{games}</small>
        </span>
    }
}

fn contestant_name<'a>(entrants: &'a HashMap<&EntrantId, &'a str>, id: &EntrantId) -> &'a str {
    entrants.get(id).copied().unwrap_or("Unknown contestant")
}
