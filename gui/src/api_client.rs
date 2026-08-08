use tabletennis_tournament::api_contract::{
    AuthenticationView, CreateTournamentRequest, DeleteTournamentRequest, DeleteTournamentResponse,
    ReceivedTournamentInvitationView, RecordMatchResultRequest, ReplaceRosterRequest,
    ShareTournamentRequest, TournamentInvitationDecisionView, TournamentMutationRequest,
    TournamentSharingView, TournamentSummaryView, TournamentView, UpdateTournamentMemberRequest,
};

#[cfg(target_arch = "wasm32")]
use tabletennis_tournament::api_contract::ApiErrorView;

pub async fn current_session() -> Result<AuthenticationView, String> {
    get_json("/api/auth/me").await
}

pub async fn logout() -> Result<(), String> {
    send_without_response("POST", "/api/auth/logout", None).await
}

pub async fn list_tournaments() -> Result<Vec<TournamentSummaryView>, String> {
    get_json("/api/tournaments").await
}

pub async fn list_received_invitations() -> Result<Vec<ReceivedTournamentInvitationView>, String> {
    get_json("/api/tournament-invitations").await
}

pub async fn accept_invitation(id: &str) -> Result<TournamentInvitationDecisionView, String> {
    send_json(
        "POST",
        &format!("/api/tournament-invitations/{id}/accept"),
        &(),
    )
    .await
}

pub async fn decline_invitation(id: &str) -> Result<TournamentInvitationDecisionView, String> {
    send_json(
        "POST",
        &format!("/api/tournament-invitations/{id}/decline"),
        &(),
    )
    .await
}

pub async fn create_tournament(
    request: &CreateTournamentRequest,
) -> Result<TournamentView, String> {
    send_json("POST", "/api/tournaments", request).await
}

pub async fn load_tournament(id: &str) -> Result<TournamentView, String> {
    get_json(&format!("/api/tournaments/{id}")).await
}

pub async fn delete_tournament(
    id: &str,
    expected_tournament_revision: u64,
) -> Result<DeleteTournamentResponse, String> {
    send_json(
        "DELETE",
        &format!("/api/tournaments/{id}"),
        &DeleteTournamentRequest {
            expected_tournament_revision,
        },
    )
    .await
}

pub async fn load_sharing(id: &str) -> Result<TournamentSharingView, String> {
    get_json(&format!("/api/tournaments/{id}/sharing")).await
}

pub async fn share_tournament(
    id: &str,
    request: &ShareTournamentRequest,
) -> Result<TournamentSharingView, String> {
    send_json("POST", &format!("/api/tournaments/{id}/sharing"), request).await
}

pub async fn update_member_role(
    tournament_id: &str,
    user_id: &str,
    request: &UpdateTournamentMemberRequest,
) -> Result<TournamentSharingView, String> {
    send_json(
        "PUT",
        &format!("/api/tournaments/{tournament_id}/members/{user_id}"),
        request,
    )
    .await
}

pub async fn remove_member(
    tournament_id: &str,
    user_id: &str,
) -> Result<TournamentSharingView, String> {
    send_json(
        "DELETE",
        &format!("/api/tournaments/{tournament_id}/members/{user_id}"),
        &(),
    )
    .await
}

pub async fn delete_invitation(
    tournament_id: &str,
    invitation_id: &str,
) -> Result<TournamentSharingView, String> {
    send_json(
        "DELETE",
        &format!("/api/tournaments/{tournament_id}/invitations/{invitation_id}"),
        &(),
    )
    .await
}

pub async fn replace_roster(
    id: &str,
    request: &ReplaceRosterRequest,
) -> Result<TournamentView, String> {
    send_json("PUT", &format!("/api/tournaments/{id}/entrants"), request).await
}

pub async fn start_tournament(id: &str, revision: u64) -> Result<TournamentView, String> {
    mutation(id, "start", revision).await
}

pub async fn calculate_pairings(id: &str, revision: u64) -> Result<TournamentView, String> {
    mutation(id, "pairings/calculate", revision).await
}

pub async fn publish_pairings(id: &str, revision: u64) -> Result<TournamentView, String> {
    mutation(id, "pairings/publish", revision).await
}

pub async fn complete_round(id: &str, revision: u64) -> Result<TournamentView, String> {
    mutation(id, "rounds/complete", revision).await
}

pub async fn record_result(
    tournament_id: &str,
    match_id: &str,
    request: &RecordMatchResultRequest,
) -> Result<TournamentView, String> {
    send_json(
        "PUT",
        &format!("/api/tournaments/{tournament_id}/matches/{match_id}/result"),
        request,
    )
    .await
}

async fn mutation(id: &str, path: &str, revision: u64) -> Result<TournamentView, String> {
    send_json(
        "POST",
        &format!("/api/tournaments/{id}/{path}"),
        &TournamentMutationRequest {
            expected_tournament_revision: revision,
        },
    )
    .await
}

#[cfg(target_arch = "wasm32")]
async fn get_json<T: serde::de::DeserializeOwned>(url: &str) -> Result<T, String> {
    let response = gloo_net::http::Request::get(url)
        .send()
        .await
        .map_err(network_error)?;
    decode(response).await
}

#[cfg(not(target_arch = "wasm32"))]
async fn get_json<T: serde::de::DeserializeOwned>(_url: &str) -> Result<T, String> {
    Err("API calls require a WASM browser build".to_owned())
}

#[cfg(target_arch = "wasm32")]
async fn send_json<T, R>(method: &str, url: &str, body: &T) -> Result<R, String>
where
    T: serde::Serialize,
    R: serde::de::DeserializeOwned,
{
    let builder = match method {
        "POST" => gloo_net::http::Request::post(url),
        "PUT" => gloo_net::http::Request::put(url),
        "DELETE" => gloo_net::http::Request::delete(url),
        _ => return Err("unsupported API method".to_owned()),
    };
    let response = builder
        .header("Content-Type", "application/json")
        .json(body)
        .map_err(network_error)?
        .send()
        .await
        .map_err(network_error)?;
    decode(response).await
}

#[cfg(not(target_arch = "wasm32"))]
async fn send_json<T, R>(_method: &str, _url: &str, _body: &T) -> Result<R, String>
where
    T: serde::Serialize,
    R: serde::de::DeserializeOwned,
{
    Err("API calls require a WASM browser build".to_owned())
}

#[cfg(target_arch = "wasm32")]
async fn send_without_response(method: &str, url: &str, _body: Option<&str>) -> Result<(), String> {
    let response = match method {
        "POST" => gloo_net::http::Request::post(url),
        _ => return Err("unsupported API method".to_owned()),
    }
    .send()
    .await
    .map_err(network_error)?;
    if response.ok() {
        Ok(())
    } else {
        Err(api_error(response).await)
    }
}

#[cfg(not(target_arch = "wasm32"))]
async fn send_without_response(
    _method: &str,
    _url: &str,
    _body: Option<&str>,
) -> Result<(), String> {
    Err("API calls require a WASM browser build".to_owned())
}

#[cfg(target_arch = "wasm32")]
async fn decode<T: serde::de::DeserializeOwned>(
    response: gloo_net::http::Response,
) -> Result<T, String> {
    if response.ok() {
        response.json().await.map_err(network_error)
    } else {
        Err(api_error(response).await)
    }
}

#[cfg(target_arch = "wasm32")]
async fn api_error(response: gloo_net::http::Response) -> String {
    let status = response.status();
    response.json::<ApiErrorView>().await.map_or_else(
        |_| format!("API request failed with status {status}"),
        |error| error.message,
    )
}

#[cfg(target_arch = "wasm32")]
fn network_error(error: impl std::fmt::Display) -> String {
    error.to_string()
}
