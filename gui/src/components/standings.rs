use std::collections::{HashMap, HashSet};

use yew::prelude::*;

use tabletennis_tournament::application::{ContestantStanding, TournamentEntrant};
use tabletennis_tournament::identity::EntrantId;

use crate::language::{Text, use_language};

#[derive(Properties, PartialEq)]
pub struct StandingsProps {
    pub standings: Vec<ContestantStanding>,
    pub entrants: Vec<TournamentEntrant>,
    pub active_entrant_ids: Vec<EntrantId>,
}

#[component]
pub fn Standings(props: &StandingsProps) -> Html {
    let language = use_language();
    let entrants = props
        .entrants
        .iter()
        .map(|entrant| (&entrant.entrant_id, entrant))
        .collect::<HashMap<_, _>>();
    let active_entrant_ids = props.active_entrant_ids.iter().collect::<HashSet<_>>();

    html! {
        <div class="table-wrap standings-table">
            <table>
                <thead>
                    <tr>
                        <th>{"#"}</th><th>{language.text(Text::Contestant)}</th><th>{language.text(Text::Score)}</th>
                        <th>{language.text(Text::WinsLosses)}</th><th>{language.text(Text::Games)}</th><th>{language.text(Text::Points)}</th>
                        <th>{language.text(Text::OpponentShort)}</th><th>{language.text(Text::Bye)}</th>
                    </tr>
                </thead>
                <tbody>
                    {for props.standings.iter().enumerate().map(|(index, standing)| {
                        let entrant = entrants.get(&standing.entrant_id);
                        html! {
                            <tr key={standing.entrant_id.as_str().to_owned()}>
                                <td class="rank">{index + 1}</td>
                                <td>
                                    <strong>{entrant.map_or(language.text(Text::UnknownContestant), |entrant| entrant.name.as_str())}</strong>
                                    <small>
                                        {entrant.map_or(language.text(Text::UnknownClub), |entrant| entrant.club_name.as_str())}
                                        if !active_entrant_ids.contains(&standing.entrant_id) {
                                            <span class="withdrawn-badge">{language.text(Text::Withdrawn)}</span>
                                        }
                                    </small>
                                </td>
                                <td>{format_score(standing.performance_score.scaled_value())}</td>
                                <td>{format!("{}-{}", standing.matches_won, standing.matches_lost)}</td>
                                <td>{format!("{}-{}", standing.games_won, standing.games_lost)}</td>
                                <td>{format!("{}-{}", standing.points_won, standing.points_lost)}</td>
                                <td>{format_score(standing.opponent_score_sum.scaled_value())}</td>
                                <td>{standing.bye_count}</td>
                            </tr>
                        }
                    })}
                </tbody>
            </table>
        </div>
    }
}

fn format_score(value: i64) -> String {
    format!("{:+.3}", value as f64 / 1_000_000.0)
}
