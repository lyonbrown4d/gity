use super::{OciGateway, State, StatusCode};

pub(super) async fn ready(State(gateway): State<OciGateway>) -> StatusCode {
  // The fixed probe never accepts client paths, credentials, or response bodies.
  let target = format!("{}/v2/", gateway.settings.upstream.trim_end_matches('/'));
  match gateway.client.get(target).send().await {
    Ok(response) if response.status().is_success() => StatusCode::OK,
    _ => StatusCode::SERVICE_UNAVAILABLE,
  }
}
