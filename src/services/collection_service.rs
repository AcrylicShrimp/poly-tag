use crate::db::models::{Collection, CreatingCollection, UpdatingCollection};
use diesel::{ExpressionMethods, OptionalExtension, QueryDsl};
use diesel_async::{pooled_connection::deadpool::Pool, AsyncPgConnection, RunQueryDsl};
use std::sync::Arc;
use thiserror::Error;
use uuid::Uuid;

#[derive(Error, Debug)]
pub enum CollectionServiceError {
    #[error("database pool error: {0}")]
    PoolError(#[from] diesel_async::pooled_connection::deadpool::PoolError),
    #[error("diesel error: {0}")]
    DieselError(#[from] diesel::result::Error),
}

pub struct CollectionService {
    db_pool: Pool<AsyncPgConnection>,
}

impl CollectionService {
    pub fn new(db_pool: Pool<AsyncPgConnection>) -> Arc<Self> {
        Arc::new(Self { db_pool })
    }

    /// Creates a new collection.
    pub async fn create_collection(
        &self,
        name: &str,
        description: Option<&str>,
    ) -> Result<Collection, CollectionServiceError> {
        use crate::db::schema;

        let db = &mut self.db_pool.get().await?;
        let collection = diesel::insert_into(schema::collections::table)
            .values(CreatingCollection { name, description })
            .returning((
                schema::collections::id,
                schema::collections::name,
                schema::collections::description,
                schema::collections::created_at,
            ))
            .get_result::<Collection>(db)
            .await?;

        Ok(collection)
    }

    /// Removes a collection by its ID.
    /// Returns the collection that was removed, or `None` if no collection was found.
    pub async fn remove_collection_by_id(
        &self,
        collection_id: Uuid,
    ) -> Result<Option<Collection>, CollectionServiceError> {
        use crate::db::schema;

        let db = &mut self.db_pool.get().await?;
        let collection = diesel::delete(
            schema::collections::dsl::collections.filter(schema::collections::id.eq(collection_id)),
        )
        .returning((
            schema::collections::id,
            schema::collections::name,
            schema::collections::description,
            schema::collections::created_at,
        ))
        .get_result::<Collection>(db)
        .await
        .optional()?;

        Ok(collection)
    }

    /// Retrieves a list of collections.
    /// The result will be sorted by name and ID (name first) in ascending order.
    /// If `last_collection_id` is provided, the result will start after the collection with that ID.
    pub async fn get_collections(
        &self,
        last_collection_id: Option<Uuid>,
        limit: u32,
    ) -> Result<Vec<Collection>, CollectionServiceError> {
        use crate::db::schema;

        let db = &mut self.db_pool.get().await?;
        let query = schema::collections::dsl::collections
            .select((
                schema::collections::id,
                schema::collections::name,
                schema::collections::description,
                schema::collections::created_at,
            ))
            .order((
                schema::collections::name.asc(),
                schema::collections::id.asc(),
            ))
            .limit(limit as i64);
        let collections = match last_collection_id {
            Some(last_collection_id) => query
                .filter(schema::collections::id.gt(last_collection_id))
                .load::<Collection>(db),
            None => query.load::<Collection>(db),
        };
        let collections = collections.await?;

        Ok(collections)
    }

    /// Retrieves a collection by its ID.
    pub async fn get_collection_by_id(
        &self,
        collection_id: Uuid,
    ) -> Result<Option<Collection>, CollectionServiceError> {
        use crate::db::schema;

        let db = &mut self.db_pool.get().await?;
        let collection = schema::collections::dsl::collections
            .filter(schema::collections::id.eq(collection_id))
            .select((
                schema::collections::id,
                schema::collections::name,
                schema::collections::description,
                schema::collections::created_at,
            ))
            .first::<Collection>(db)
            .await
            .optional()?;

        Ok(collection)
    }

    /// Updates a collection by its ID.
    /// Returns the collection that was updated, or `None` if no collection was found.
    pub async fn update_collection_by_id(
        &self,
        collection_id: Uuid,
        new_name: &str,
        new_description: Option<&str>,
    ) -> Result<Option<Collection>, CollectionServiceError> {
        use crate::db::schema;

        let db = &mut self.db_pool.get().await?;
        let collection = diesel::update(
            schema::collections::dsl::collections.filter(schema::collections::id.eq(collection_id)),
        )
        .set(UpdatingCollection {
            name: new_name,
            description: new_description,
        })
        .returning((
            schema::collections::id,
            schema::collections::name,
            schema::collections::description,
            schema::collections::created_at,
        ))
        .get_result::<Collection>(db)
        .await
        .optional()?;

        Ok(collection)
    }
}