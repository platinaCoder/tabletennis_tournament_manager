use web_sys::{HtmlInputElement, HtmlSelectElement};
use yew::prelude::*;

use tabletennis_tournament::results::MatchFormat;

use crate::language::{Text, use_language};
use crate::model::CreateTournamentCommand;

#[derive(Properties, PartialEq)]
pub struct TournamentSetupProps {
    pub on_create: Callback<CreateTournamentCommand>,
}

#[component]
pub fn TournamentSetup(props: &TournamentSetupProps) -> Html {
    let language = use_language();
    let tournament_id = use_state(|| "local-tournament".to_owned());
    let table_count = use_state(|| "8".to_owned());
    let contestant_count = use_state(|| "16".to_owned());
    let maximum_round_count = use_state(|| "5".to_owned());
    let match_format = use_state(|| MatchFormat::BestOfFive);
    let match_format_select = use_node_ref();

    let onsubmit = {
        let tournament_id = tournament_id.clone();
        let table_count = table_count.clone();
        let contestant_count = contestant_count.clone();
        let maximum_round_count = maximum_round_count.clone();
        let match_format = match_format.clone();
        let match_format_select = match_format_select.clone();
        let on_create = props.on_create.clone();
        Callback::from(move |event: SubmitEvent| {
            event.prevent_default();
            let submitted_match_format = match_format_select
                .cast::<HtmlSelectElement>()
                .and_then(|select| match_format_from_value(&select.value()))
                .unwrap_or(*match_format);
            on_create.emit(CreateTournamentCommand {
                tournament_id: (*tournament_id).clone(),
                match_format: submitted_match_format,
                table_count: table_count.parse().unwrap_or(0),
                contestant_count: contestant_count.parse().unwrap_or(0),
                maximum_round_count: maximum_round_count.parse().unwrap_or(0),
            });
        })
    };
    let on_id = text_input(tournament_id.clone());
    let on_tables = text_input(table_count.clone());
    let on_contestants = text_input(contestant_count.clone());
    let on_rounds = text_input(maximum_round_count.clone());
    let on_format = {
        let match_format = match_format.clone();
        Callback::from(move |event: Event| {
            let select = event.target_unchecked_into::<HtmlSelectElement>();
            if let Some(format) = match_format_from_value(&select.value()) {
                match_format.set(format);
            }
        })
    };

    html! {
        <section class="panel setup-panel">
            <p class="eyebrow">{language.text(Text::TournamentSetup)}</p>
            <h2>{language.text(Text::CreateTournament)}</h2>
            <p class="muted">
                {language.setup_explanation()}
            </p>
            <form {onsubmit} class="form-grid">
                <label>
                    <span>{language.text(Text::TournamentIdentifier)}</span>
                    <input required=true value={(*tournament_id).clone()} oninput={on_id} />
                </label>
                <label>
                    <span>{language.text(Text::MatchFormat)}</span>
                    <select
                        ref={match_format_select}
                        value={match *match_format {
                            MatchFormat::BestOfThree => "best_of_three",
                            MatchFormat::BestOfFive => "best_of_five",
                        }}
                        onchange={on_format}
                    >
                        <option value="best_of_five">{crate::formatting::match_format(MatchFormat::BestOfFive, language)}</option>
                        <option value="best_of_three">{crate::formatting::match_format(MatchFormat::BestOfThree, language)}</option>
                    </select>
                </label>
                <label>
                    <span>{language.text(Text::AvailableTables)}</span>
                    <input required=true type="number" min="1" value={(*table_count).clone()} oninput={on_tables} />
                </label>
                <label>
                    <span>{language.text(Text::ContestantCount)}</span>
                    <input required=true type="number" min="2" max="64" value={(*contestant_count).clone()} oninput={on_contestants} />
                </label>
                <label>
                    <span>{language.text(Text::MaximumRounds)}</span>
                    <input required=true type="number" min="1" max="65535" value={(*maximum_round_count).clone()} oninput={on_rounds} />
                </label>
                <button class="primary" type="submit">{language.text(Text::CreateTournament)}</button>
            </form>
        </section>
    }
}

fn match_format_from_value(value: &str) -> Option<MatchFormat> {
    match value {
        "best_of_three" => Some(MatchFormat::BestOfThree),
        "best_of_five" => Some(MatchFormat::BestOfFive),
        _ => None,
    }
}

fn text_input(state: UseStateHandle<String>) -> Callback<InputEvent> {
    Callback::from(move |event: InputEvent| {
        state.set(event.target_unchecked_into::<HtmlInputElement>().value());
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn setup_values_map_to_the_selected_match_format() {
        assert_eq!(
            match_format_from_value("best_of_three"),
            Some(MatchFormat::BestOfThree)
        );
        assert_eq!(
            match_format_from_value("best_of_five"),
            Some(MatchFormat::BestOfFive)
        );
        assert_eq!(match_format_from_value("unexpected"), None);
    }
}
