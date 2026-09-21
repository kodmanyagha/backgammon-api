use entity::roles::role_keys;

use crate::{
    service::entity_service::users_entity_service::{CreateUserInputDto, CreateUserResultDto},
    state::app_state::AppState,
};

pub async fn handle(
    input: &CreateUserInputDto,
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
        .create_user(user, input)
        .await
}
