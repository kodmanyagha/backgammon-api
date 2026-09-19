pub mod guest;
pub mod jwt;
pub mod login;
pub mod register;
pub mod types;
pub mod update_profile;

use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Clone, Debug, Serialize, Deserialize, ToSchema)]
pub struct AuthResultDto {
    pub token: String,
    pub user_id: u64,
    pub email: Option<String>,
    pub username: Option<String>,
    pub is_guest: bool,
}
