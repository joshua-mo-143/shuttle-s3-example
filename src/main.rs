use aws_config::Region;
use aws_credential_types::Credentials;
use aws_sdk_s3::Client;
use axum::routing::{get, post};
use axum::Router;
use shuttle_runtime::SecretStore;
use std::time::Duration;
use tower_http::timeout::TimeoutLayer;

mod errors;
mod routing;

#[derive(Clone, Debug)]
pub struct AppState {
    s3: Client,
}
async fn hello_world() -> &'static str {
    "Hello, world!"
}

#[shuttle_runtime::main]
async fn main(#[shuttle_runtime::Secrets] secrets: SecretStore) -> shuttle_axum::ShuttleAxum {
    let access_key_id = secrets
        .get("AWS_ACCESS_KEY_ID")
        .expect("AWS_ACCESS_KEY_ID not set in Secrets.toml");
    let secret_access_key = secrets
        .get("AWS_SECRET_ACCESS_KEY")
        .expect("AWS_ACCESS_KEY_ID not set in Secrets.toml");
    let aws_url = secrets
        .get("AWS_URL")
        .expect("AWS_URL not set in Secrets.toml");

    let creds = Credentials::from_keys(access_key_id, secret_access_key, None);

    let cfg = aws_config::from_env()
        .endpoint_url(&aws_url)
        .region(Region::new("eu-west-2"))
        .credentials_provider(creds)
        .load()
        .await;

    let s3 = Client::new(&cfg);

    let state = AppState { s3 };

    let rtr = init_router(state);

    Ok(rtr.into())
}

fn init_router(state: AppState) -> Router {
    Router::new()
        .route("/", get(hello_world))
        .route("/images/upload", post(routing::upload_image))
        .route(
            "/images/:filename",
            get(routing::retrieve_image).delete(routing::delete_image),
        )
        .with_state(state)
        .layer(TimeoutLayer::new(Duration::from_secs(20)))
}
