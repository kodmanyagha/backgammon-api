use entity::roles::role_keys;

use crate::state::app_state::AppState;

pub async fn handle(
    input: u64,
    app_state: &AppState,
    user: &entity::users::Model,
) -> anyhow::Result<entity::users::Model> {
    let current_user_role = app_state
        .users_entity_service
        .get_user_roles_str(user.id)
        .await;

    // TODO Move this control logic to route layer
    if !current_user_role.contains(&role_keys::ADMIN.to_string()) {
        return Err(anyhow::anyhow!("Unauthorized"));
    }

    app_state
        .users_entity_service
        .get_by_id(input)
        .await
        .ok_or(anyhow::anyhow!("User not found"))
}
