use tabletennis_tournament::backend::server::application_router;
use tower::ServiceBuilder;
use vercel_runtime::axum::VercelLayer;
use vercel_runtime::{Error, run};

#[tokio::main]
async fn main() -> Result<(), Error> {
    let router = application_router().await?;
    let service = ServiceBuilder::new()
        .layer(VercelLayer::new())
        .service(router);
    run(service).await
}
