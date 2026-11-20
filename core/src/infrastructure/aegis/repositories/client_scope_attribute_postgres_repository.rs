use crate::entity::client_scope_attributes::COLUMN;
use sea_orm::{ActiveModelTrait, ActiveValue::Set, DatabaseConnection, EntityTrait, QueryFilter};
use uuid::Uuid;

use crate::domain::aegis::entities::ClientScopeAttribute;
use crate::domain::aegis::ports::ClientScopeAttributeRepository;
use crate::domain::common::entities::app_errors::CoreError;
use crate::domain::common::generate_uuid_v7;
use crate::entity::client_scope_attributes;

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct PostgresClientScopeAttributeRepository {
    pub db: DatabaseConnection,
}

impl PostgresClientScopeAttributeRepository {
    #[allow(dead_code)]
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }
}

impl ClientScopeAttributeRepository for PostgresClientScopeAttributeRepository {
    async fn set_attribute(
        &self,
        scope_id: Uuid,
        name: String,
        value: Option<String>,
    ) -> Result<ClientScopeAttribute, CoreError> {
        let existing = client_scope_attributes::Entity::find()
            .filter(COLUMN.client_scope_id.eq(scope_id))
            .filter(COLUMN.name.eq(&name))
            .one(&self.db)
            .await
            .map_err(|e| {
                tracing::error!("Failed to find client scope attribute: {}", e);
                CoreError::InternalServerError
            })?;

        let model = if let Some(existing) = existing {
            let mut active: client_scope_attributes::ActiveModel = existing.into();
            active.value = Set(value);
            active.update(&self.db).await.map_err(|e| {
                tracing::error!("Failed to update client scope attribute: {}", e);
                CoreError::InternalServerError
            })?
        } else {
            let active = client_scope_attributes::ActiveModel {
                id: Set(generate_uuid_v7()),
                client_scope_id: Set(scope_id),
                name: Set(name),
                value: Set(value),
            };
            active.insert(&self.db).await.map_err(|e| {
                tracing::error!("Failed to insert client scope attribute: {}", e);
                CoreError::InternalServerError
            })?
        };

        Ok(model.into())
    }

    async fn get_attributes(&self, scope_id: Uuid) -> Result<Vec<ClientScopeAttribute>, CoreError> {
        let models = client_scope_attributes::Entity::find()
            .filter(COLUMN.client_scope_id.eq(scope_id))
            .all(&self.db)
            .await
            .map_err(|e| {
                tracing::error!("Failed to get client scope attributes: {}", e);
                CoreError::InternalServerError
            })?;

        Ok(models.into_iter().map(ClientScopeAttribute::from).collect())
    }

    async fn remove_attribute(&self, scope_id: Uuid, name: String) -> Result<(), CoreError> {
        let result = client_scope_attributes::Entity::delete_many()
            .filter(COLUMN.client_scope_id.eq(scope_id))
            .filter(COLUMN.name.eq(name))
            .exec(&self.db)
            .await
            .map_err(|e| {
                tracing::error!("Failed to delete client scope attribute: {}", e);
                CoreError::InternalServerError
            })?;

        if result.rows_affected == 0 {
            return Err(CoreError::NotFound);
        }

        Ok(())
    }
}
