use sea_orm::{DatabaseBackend, MockDatabase};

pub fn create_mock_conn() -> MockDatabase {
    MockDatabase::new(DatabaseBackend::Postgres)
}
