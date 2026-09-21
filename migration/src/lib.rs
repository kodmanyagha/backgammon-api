pub mod seeds;
pub mod utils;

pub use sea_orm_migration::prelude::*;

mod m20240101_000001_create_users;
mod m20250311_100610_create_permissions;
mod m20250311_100614_create_roles;
mod m20250311_104157_create_roles_permissions_pivot;
mod m20250311_104208_create_users_roles_pivot;
mod m20250311_104213_create_users_permissions_pivot;
mod m20260205_141324_create_system_logs_table;
mod m20260911_000001_create_user_throttles_table;
mod m20260911_000003_drop_users_tfa_columns;
mod m20260914_100001_add_guest_support_to_users;
mod m20260914_100002_create_games_table;
mod m20260914_100003_create_game_moves_table;
mod m20260914_100004_create_game_invites_table;
mod m20260914_100005_create_scores_table;
mod m20260914_100006_create_game_rounds_table;
mod m20260919_100001_add_guest_unique_id_to_users;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20240101_000001_create_users::Migration),
            Box::new(m20250311_100610_create_permissions::Migration),
            Box::new(m20250311_100614_create_roles::Migration),
            Box::new(m20250311_104157_create_roles_permissions_pivot::Migration),
            Box::new(m20250311_104208_create_users_roles_pivot::Migration),
            Box::new(m20250311_104213_create_users_permissions_pivot::Migration),
            Box::new(m20260205_141324_create_system_logs_table::Migration),
            Box::new(m20260911_000001_create_user_throttles_table::Migration),
            Box::new(m20260911_000003_drop_users_tfa_columns::Migration),
            Box::new(m20260914_100001_add_guest_support_to_users::Migration),
            Box::new(m20260914_100002_create_games_table::Migration),
            Box::new(m20260914_100003_create_game_moves_table::Migration),
            Box::new(m20260914_100004_create_game_invites_table::Migration),
            Box::new(m20260914_100005_create_scores_table::Migration),
            Box::new(m20260914_100006_create_game_rounds_table::Migration),
            Box::new(m20260919_100001_add_guest_unique_id_to_users::Migration),
        ]
    }
}
