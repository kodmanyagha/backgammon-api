use tavla_api::{app, utils::logger::init_tracing_subscriber};

#[tokio::main(flavor = "multi_thread", worker_threads = 10)]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().expect(".env file not found.");
    init_tracing_subscriber();

    app::server::handle().await
}
