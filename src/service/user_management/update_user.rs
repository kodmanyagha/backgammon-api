use entity::roles::role_keys;

use crate::{
    service::entity_service::users_entity_service::{CreateUserResultDto, UpdateUserInputDto},
    state::app_state::AppState,
};

pub async fn handle(
    input: &UpdateUserInputDto,
    app_state: &AppState,
    user: &entity::users::Model,
) -> anyhow::Result<CreateUserResultDto> {
    let current_user_role = app_state
        .users_entity_service
        .get_user_roles_str(user.id)
        .await;

    if !current_user_role.contains(&role_keys::ADMIN.to_string()) {
        return Err(anyhow::anyhow!("Unauthorized"));
    }

    app_state
        .users_entity_service
        .update_user(user, input)
        .await
}
