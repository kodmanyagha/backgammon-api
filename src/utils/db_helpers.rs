use migration::{Migrator, MigratorTrait, SchemaManagerConnection};
use sea_orm::{
    sqlx::{self, mysql::MySqlPoolOptions, MySqlPool},
    ConnectionTrait, DatabaseConnection, DbErr,
};
use serde::Serialize;
use std::time::Duration;

use crate::{state::app_state::AppState, CONFIG};

pub async fn create_db_conn() -> anyhow::Result<DatabaseConnection> {
    let mut opt = sea_orm::ConnectOptions::new(CONFIG.get_database_url());
    opt.max_connections(200)
        .min_connections(10)
        .connect_timeout(Duration::from_secs(8))
        .acquire_timeout(Duration::from_secs(8))
        .idle_timeout(Duration::from_secs(8))
        .max_lifetime(Duration::from_secs(30))
        .sqlx_logging(true)
        .sqlx_logging_level(log::LevelFilter::Debug);

    match sea_orm::Database::connect(opt).await {
        Ok(conn) => Ok(conn),
        Err(err) => {
            tracing::error!("DB connection error: {}", &err);
            Err(anyhow::anyhow!("DB connection error: {}", &err))
        }
    }
}

pub async fn create_db_conn_pool() -> anyhow::Result<DatabaseConnection> {
    let pool: MySqlPool = MySqlPoolOptions::new()
        .max_connections(200)
        .min_connections(50)
        .acquire_timeout(Duration::from_secs(30))
        .idle_timeout(Duration::from_secs(30))
        .max_lifetime(Duration::from_secs(60))
        .after_connect(|conn, _meta| {
            Box::pin(async move {
                sqlx::query("SET time_zone = '+03:00'")
                    .execute(conn)
                    .await?;

                Ok(())
            })
        })
        .connect(CONFIG.get_database_url())
        .await?;

    Ok(DatabaseConnection::from(pool))
}

pub async fn run_migrations(app_state: &AppState) -> anyhow::Result<()> {
    Migrator::up(&*app_state.db_conn, None)
        .await
        .map_err(|err| {
            tracing::error!("Error occured when running migrations: {}", err);
            anyhow::anyhow!(err.to_string())
        })
}

pub async fn run_seeds(app_state: &AppState) -> anyhow::Result<()> {
    migration::seeds::permissions::seed(&SchemaManagerConnection::Connection(&app_state.db_conn))
        .await
        .map_err(|err| DbErr::Custom(format!("Seed error: {err}")))?;

    migration::seeds::roles::seed(&SchemaManagerConnection::Connection(&app_state.db_conn))
        .await
        .map_err(|err| DbErr::Custom(format!("Seed error: {err}")))?;

    migration::seeds::users::seed(&SchemaManagerConnection::Connection(&app_state.db_conn))
        .await
        .map_err(|err| DbErr::Custom(format!("Seed error: {err}")))?;

    Ok(())
}

pub async fn mysql_import_csv<T: Serialize>(
    app_state: &AppState,
    data: Vec<T>,
    table_name_str: &str,
    ignore_first_row: bool,
) -> anyhow::Result<()> {
    let ignore_line_str = if ignore_first_row {
        "IGNORE 1 ROWS"
    } else {
        ""
    };

    let file_path_str = "";
    let columns_str = "";

    let sql = format!(
        r#"
LOAD DATA INFILE
'{file_path_str}'
IGNORE INTO TABLE `{table_name_str}`
FIELDS TERMINATED BY ';'
ENCLOSED BY '"'
LINES TERMINATED BY '\n'

    {ignore_line_str}

    ({columns_str});
"#
    );

    (*app_state.db_conn).execute_unprepared(&sql).await?;

    Ok(())
}
