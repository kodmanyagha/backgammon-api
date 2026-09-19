use sea_orm_migration::prelude::*;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let _: () = cli::run_cli(migration::Migrator).await;
    Ok(())
}
