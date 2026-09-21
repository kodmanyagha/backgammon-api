use std::sync::Arc;

use sea_orm::ActiveModelTrait;
use sea_orm::ActiveValue::Set;
use sea_orm::ColumnTrait;
use sea_orm::DatabaseConnection;
use sea_orm::EntityTrait;
use sea_orm::QueryFilter;
use sea_orm::QuerySelect;

use crate::utils::enums::active_passive_status::ActivePassiveStatus;

#[derive(Clone)]
pub struct UsersRepository {
    db_conn: Arc<DatabaseConnection>,
}

impl UsersRepository {
    pub fn new(db_conn: Arc<DatabaseConnection>) -> Self {
        Self { db_conn }
    }

    pub async fn get_by_id(&self, id: u64) -> Option<crate::users::Model> {
        crate::users::Entity::find_by_id(id)
            .one(&*self.db_conn)
            .await
            .ok()?
    }

    pub async fn get_by_id_active(&self, id: u64) -> Option<crate::users::Model> {
        crate::users::Entity::find()
            .filter(crate::users::Column::Id.eq(id))
            .filter(crate::users::Column::Status.eq(ActivePassiveStatus::Active))
            .one(&*self.db_conn)
            .await
            .ok()?
    }

    pub async fn get_by_email(&self, email: &str) -> Option<crate::users::Model> {
        crate::users::Entity::find()
            .filter(crate::users::Column::Email.eq(email))
            .one(&*self.db_conn)
            .await
            .ok()?
    }

    pub async fn get_by_username(&self, username: &str) -> Option<crate::users::Model> {
        crate::users::Entity::find()
            .filter(crate::users::Column::Username.eq(username))
            .one(&*self.db_conn)
            .await
            .ok()?
    }

    pub async fn get_by_guest_unique_id_hash(
        &self,
        guest_unique_id_hash: &str,
    ) -> Option<crate::users::Model> {
        crate::users::Entity::find()
            .filter(crate::users::Column::GuestUniqueId.eq(guest_unique_id_hash))
            .one(&*self.db_conn)
            .await
            .ok()?
    }

    pub async fn get_user_roles(&self, user_id: u64) -> Vec<crate::roles::Model> {
        crate::roles::Entity::find()
            .filter(crate::users_roles_pivot::Column::UserId.eq(user_id))
            .join_rev(
                sea_orm::JoinType::Join,
                crate::users_roles_pivot::Entity::belongs_to(crate::roles::Entity)
                    .from(crate::users_roles_pivot::Column::RoleId)
                    .to(crate::roles::Column::Id)
                    .into(),
            )
            .all(&*self.db_conn)
            .await
            .ok()
            .unwrap_or_default()
    }

    pub async fn bind_user_with_role(
        &self,
        user_id: u64,
        role_id: u64,
    ) -> anyhow::Result<crate::users_roles_pivot::Model> {
        let existing_pivot_row = crate::users_roles_pivot::Entity::find()
            .filter(crate::users_roles_pivot::Column::UserId.eq(user_id))
            .filter(crate::users_roles_pivot::Column::RoleId.eq(role_id))
            .one(&*self.db_conn)
            .await
            .map_err(|err| anyhow::anyhow!(format!("{}", err)))?;

        if let Some(pivot_row) = existing_pivot_row {
            return Ok(pivot_row);
        }

        let pivot_row = crate::users_roles_pivot::ActiveModel {
            role_id: Set(role_id),
            user_id: Set(user_id),
            ..Default::default()
        };

        pivot_row
            .insert(&*self.db_conn)
            .await
            .map_err(|err| anyhow::anyhow!(format!("{}", err)))
    }

    pub async fn create_user(
        &self,
        parent_id: u64,
        email: Option<&str>,
        password: Option<&str>,
        firstname: &str,
        lastname: &str,
        status: ActivePassiveStatus,
    ) -> anyhow::Result<crate::users::Model> {
        let row = crate::users::ActiveModel {
            parent_id: Set(Some(parent_id)),
            email: Set(email.map(str::to_string)),
            password: Set(password.map(str::to_string)),
            firstname: Set(Some(firstname.into())),
            lastname: Set(Some(lastname.into())),
            status: Set(status),
            ..Default::default()
        };

        Ok(row.insert(&*self.db_conn).await?)
    }

    pub async fn update_user(
        &self,
        id: u64,
        email: Option<&str>,
        password: Option<&str>,
        firstname: Option<&str>,
        lastname: Option<&str>,
        status: Option<ActivePassiveStatus>,
    ) -> anyhow::Result<crate::users::Model> {
        let row = self
            .get_by_id(id)
            .await
            .ok_or(anyhow::anyhow!("Not found"))?;

        let mut row: crate::users::ActiveModel = row.into();

        if firstname.is_some() {
            row.firstname = Set(Some(firstname.unwrap_or_default().into()));
        }
        if lastname.is_some() {
            row.lastname = Set(Some(lastname.unwrap_or_default().into()));
        }
        if password.is_some() {
            row.password = Set(password.map(str::to_string));
        }
        if email.is_some() {
            row.email = Set(email.map(str::to_string));
        }
        if status.is_some() {
            row.status = Set(status.unwrap_or_default());
        }

        let row = row.update(&*self.db_conn).await?;

        Ok(row)
    }

    pub async fn register_user(
        &self,
        email: &str,
        password_hash: &str,
        username: Option<&str>,
    ) -> anyhow::Result<crate::users::Model> {
        let row = crate::users::ActiveModel {
            email: Set(Some(email.to_string())),
            password: Set(Some(password_hash.to_string())),
            username: Set(username.map(str::to_string)),
            is_guest: Set(false),
            status: Set(ActivePassiveStatus::Active),
            ..Default::default()
        };

        Ok(row.insert(&*self.db_conn).await?)
    }

    pub async fn create_guest_user(
        &self,
        username: &str,
        guest_unique_id_hash: &str,
    ) -> anyhow::Result<crate::users::Model> {
        let row = crate::users::ActiveModel {
            username: Set(Some(username.to_string())),
            guest_unique_id: Set(Some(guest_unique_id_hash.to_string())),
            is_guest: Set(true),
            status: Set(ActivePassiveStatus::Active),
            ..Default::default()
        };

        Ok(row.insert(&*self.db_conn).await?)
    }

    pub async fn update_username(
        &self,
        id: u64,
        username: &str,
    ) -> anyhow::Result<crate::users::Model> {
        let row = self
            .get_by_id(id)
            .await
            .ok_or(anyhow::anyhow!("Not found"))?;

        let mut row: crate::users::ActiveModel = row.into();
        row.username = Set(Some(username.to_string()));

        Ok(row.update(&*self.db_conn).await?)
    }
}
