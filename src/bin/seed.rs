use tavla_api::{
    state::app_state::AppState,
    utils::{db_helpers::run_seeds, logger::init_tracing_subscriber},
};

#[tokio::main(flavor = "multi_thread", worker_threads = 10)]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().expect(".env file not found.");
    init_tracing_subscriber();

    let app_state = AppState::try_new(None).await?;
    run_seeds(&app_state).await?;
    tracing::info!("All seeds executed.");

    Ok(())
}
