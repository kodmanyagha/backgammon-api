use entity::{
    repository::{roles_repo::RolesRepository, users_repo::UsersRepository},
    utils::{enums::active_passive_status::ActivePassiveStatus, password_helper::hash_password},
};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use validator::Validate;

use crate::service::authentication::jwt::AppRole;

#[derive(Clone, Debug, Default, Serialize, Deserialize, Validate, ToSchema)]
pub struct AddDeductCreditInputDto {
    #[validate(length(min = 1, message = "Can not be empty"))]
    pub user_id: String,
    #[validate(length(min = 1, message = "Can not be empty"))]
    pub credit_amount: String,
    #[validate(length(min = 1, message = "Can not be empty"))]
    pub price: String,
    #[validate(length(min = 1, message = "Can not be empty"))]
    pub operation: String,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize, Validate, ToSchema)]
pub struct CreateUserInputDto {
    #[validate(length(min = 1, message = "Can not be empty"))]
    pub firstname: String,
    #[validate(length(min = 1, message = "Can not be empty"))]
    pub lastname: String,
    #[validate(email())]
    pub email: String,
    #[validate(length(min = 6, message = "Can not be empty"))]
    pub password: String,
    #[validate(length(min = 6, message = "Can not be empty"))]
    pub password_again: String,
    #[validate(length(min = 1, message = "Can not be empty"))]
    pub role: String,
    #[validate(length(min = 1, message = "Can not be empty"))]
    pub status: String,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize, Validate, ToSchema)]
pub struct UpdateUserInputDto {
    #[validate(range(min = 0))]
    pub id: u64,

    pub firstname: Option<String>,
    pub lastname: Option<String>,
    pub email: Option<String>,
    pub password: Option<String>,
    pub password_again: Option<String>,
    pub role: Option<String>,
    pub status: Option<String>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct CreateUserResultDto {
    pub id: u64,
    pub firstname: Option<String>,
    pub lastname: Option<String>,
    pub email: Option<String>,
}

#[derive(Clone)]
pub struct UsersEntityService {
    users_repo: UsersRepository,
    roles_repo: RolesRepository,
}

impl UsersEntityService {
    pub fn new(users_repo: UsersRepository, roles_repo: RolesRepository) -> Self {
        Self {
            users_repo,
            roles_repo,
        }
    }

    pub async fn get_by_id(&self, id: u64) -> Option<entity::users::Model> {
        self.users_repo.get_by_id(id).await
    }

    pub async fn get_by_email(&self, email: &str) -> Option<entity::users::Model> {
        self.users_repo.get_by_email(email).await
    }

    pub async fn get_user_roles(&self, user_id: u64) -> Vec<entity::roles::Model> {
        self.users_repo.get_user_roles(user_id).await
    }

    pub async fn get_user_roles_str(&self, user_id: u64) -> Vec<String> {
        self.users_repo
            .get_user_roles(user_id)
            .await
            .iter()
            .map(|item| &item.key)
            .cloned()
            .collect()
    }

    pub async fn get_user_roles_enum(&self, user_id: u64) -> Vec<AppRole> {
        let str_roles = self.get_user_roles_str(user_id).await;
        str_roles
            .iter()
            .map(|item| AppRole::try_from(item.as_str()).unwrap_or(AppRole::Guest))
            .collect()
    }

    pub async fn user_has_role_opt(&self, user_id: u64, role: &str) -> Option<String> {
        let user_roles = self.get_user_roles_str(user_id).await;
        user_roles
            .iter()
            .find(|item| PartialEq::eq(*item, &role.to_string()))
            .cloned()
    }

    pub async fn user_has_role(&self, user_id: u64, role: &str) -> bool {
        let user_roles = self.get_user_roles_str(user_id).await;
        user_roles.contains(&role.to_string())
    }

    pub async fn create_user(
        &self,
        current_user: &entity::users::Model,
        input: &CreateUserInputDto,
    ) -> anyhow::Result<CreateUserResultDto> {
        tracing::info!(?input, "create_user input");

        let input_email_row = self.get_by_email(&input.email).await;
        if input_email_row.is_some() {
            return Err(anyhow::anyhow!("Email exist"));
        }

        if !input.password.eq(&input.password_again) {
            return Err(anyhow::anyhow!("Passwords must be match."));
        }

        let input_role_row = self
            .roles_repo
            .get_by_key(&input.role)
            .await
            .ok_or(anyhow::anyhow!("Role not found"))?;

        let status_normalized = if input.status.eq("active") {
            ActivePassiveStatus::Active
        } else {
            ActivePassiveStatus::Passive
        };

        let user_row = self
            .users_repo
            .create_user(
                current_user.id,
                Some(&input.email),
                Some(&hash_password(&input.password)?),
                &input.firstname,
                &input.lastname,
                status_normalized,
            )
            .await?;

        let _ = self
            .users_repo
            .bind_user_with_role(user_row.id, input_role_row.id)
            .await?;

        Ok(CreateUserResultDto {
            id: user_row.id,
            email: user_row.email,
            firstname: user_row.firstname,
            lastname: user_row.lastname,
        })
    }

    pub async fn update_user(
        &self,
        current_user: &entity::users::Model,
        input: &UpdateUserInputDto,
    ) -> anyhow::Result<CreateUserResultDto> {
        tracing::info!(?input, "update_user input");

        let mut password: Option<String> = None;

        if !input.password.clone().unwrap_or_default().is_empty() {
            password = Some(hash_password(&input.password.clone().unwrap_or_default())?);
        }

        let status_normalized = match input.status.clone() {
            Some(input_status) => {
                if input_status.eq("active") {
                    Some(ActivePassiveStatus::Active)
                } else {
                    Some(ActivePassiveStatus::Passive)
                }
            }
            None => None,
        };

        let user_row = self
            .users_repo
            .update_user(
                input.id,
                input.email.as_deref(),
                password.as_deref(),
                input.firstname.as_deref(),
                input.lastname.as_deref(),
                status_normalized,
            )
            .await?;

        Ok(CreateUserResultDto {
            id: user_row.id,
            email: user_row.email,
            firstname: user_row.firstname,
            lastname: user_row.lastname,
        })
    }

    pub async fn assign_role_by_key(&self, user_id: u64, role_key: &str) -> anyhow::Result<()> {
        let role = self
            .roles_repo
            .get_by_key(role_key)
            .await
            .ok_or_else(|| anyhow::anyhow!("Role not found"))?;

        self.users_repo.bind_user_with_role(user_id, role.id).await?;

        Ok(())
    }
}
