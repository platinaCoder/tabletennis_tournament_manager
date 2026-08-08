use tabletennis_tournament::application::TournamentApplication;

pub(crate) fn simulation_json(
    application: &TournamentApplication,
    run_seed: u64,
) -> Result<String, String> {
    let trace = application
        .simulation_trace_with_result_seed(run_seed)
        .map_err(error)?;
    serde_json::to_string_pretty(&trace).map_err(error)
}

#[cfg(target_arch = "wasm32")]
pub(crate) fn download_simulation_json(
    application: &TournamentApplication,
    run_seed: u64,
) -> Result<(), String> {
    use wasm_bindgen::JsValue;
    use web_sys::{Blob, BlobPropertyBag, Url};

    let json = simulation_json(application, run_seed)?;
    let parts = js_sys::Array::new();
    parts.push(&JsValue::from_str(&json));
    let options = BlobPropertyBag::new();
    options.set_type("application/json;charset=utf-8");
    let blob = Blob::new_with_str_sequence_and_options(&parts, &options).map_err(js_error)?;
    let object_url = Url::create_object_url_with_blob(&blob).map_err(js_error)?;
    let result = click_download_link(application, run_seed, &object_url);
    let _ = Url::revoke_object_url(&object_url);
    result
}

#[cfg(target_arch = "wasm32")]
fn click_download_link(
    application: &TournamentApplication,
    run_seed: u64,
    object_url: &str,
) -> Result<(), String> {
    use wasm_bindgen::JsCast;
    use web_sys::HtmlAnchorElement;

    let window = web_sys::window().ok_or_else(|| "browser window is unavailable".to_owned())?;
    let document = window
        .document()
        .ok_or_else(|| "browser document is unavailable".to_owned())?;
    let anchor = document
        .create_element("a")
        .map_err(js_error)?
        .dyn_into::<HtmlAnchorElement>()
        .map_err(|_| "browser could not create a download link".to_owned())?;
    anchor.set_href(object_url);
    anchor.set_download(&download_filename(application, run_seed));
    let body = document
        .body()
        .ok_or_else(|| "browser document body is unavailable".to_owned())?;
    body.append_child(&anchor).map_err(js_error)?;
    anchor.click();
    anchor.remove();
    Ok(())
}

#[cfg(not(target_arch = "wasm32"))]
pub(crate) fn download_simulation_json(
    application: &TournamentApplication,
    run_seed: u64,
) -> Result<(), String> {
    let filename = download_filename(application, run_seed);
    let _json = simulation_json(application, run_seed)?;
    Err(format!(
        "{filename} can only be downloaded from the browser"
    ))
}

fn download_filename(application: &TournamentApplication, run_seed: u64) -> String {
    let identifier = application
        .tournament()
        .id()
        .as_str()
        .chars()
        .map(|character| {
            if character.is_ascii_alphanumeric() || matches!(character, '-' | '_') {
                character
            } else {
                '-'
            }
        })
        .collect::<String>();
    let completed_rounds = application.completed_rounds().len();
    format!("{identifier}-simulation-{run_seed}-{completed_rounds}-rounds.json")
}

#[cfg(target_arch = "wasm32")]
fn js_error(error: wasm_bindgen::JsValue) -> String {
    error
        .as_string()
        .unwrap_or_else(|| "browser rejected the JSON download".to_owned())
}

fn error(error: impl std::fmt::Display) -> String {
    error.to_string()
}

#[cfg(test)]
mod tests {
    use super::{download_filename, simulation_json};
    use tabletennis_tournament::application::TournamentApplication;
    use tabletennis_tournament::results::MatchFormat;
    use tabletennis_tournament::tournament::{
        MaximumRoundCount, TableCount, Tournament, TournamentId,
    };

    #[test]
    fn filename_is_safe_and_describes_trace_progress() {
        let tournament = Tournament::new(
            TournamentId::new("Friday / test"),
            MatchFormat::BestOfThree,
            TableCount::try_from(2_i64).unwrap(),
            MaximumRoundCount::try_from(4_i64).unwrap(),
        );
        let application = TournamentApplication::new(tournament);

        assert_eq!(
            download_filename(&application, 42),
            "Friday---test-simulation-42-0-rounds.json"
        );

        let json = simulation_json(&application, 42).unwrap();
        assert!(json.contains("\"schema_version\": 2"));
        assert!(json.contains("\"run_seed\": 42"));
        assert!(json.contains("\"match_format\": \"best_of_three\""));
    }
}
