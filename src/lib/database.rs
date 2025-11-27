use crate::hash_migration::legacy;
use crate::utils::hash_user_id;
use anyhow::Result;
use scylla::client::session::Session;
use std::sync::Arc;
use tracing::{info, warn};

fn get_legacy_key_if_different(user_id: &str, new_key: &str) -> Option<String> {
    let legacy_key = legacy::hash_user_id(user_id);
    if legacy_key != new_key {
        Some(legacy_key)
    } else {
        None
    }
}

#[derive(Clone)]
pub struct DatabaseService {
    session: Arc<Session>,
}

impl DatabaseService {
    pub async fn new(session: Session) -> Result<Self> {
        session.use_keyspace("equicloud", false).await?;
        Ok(Self {
            session: Arc::new(session),
        })
    }

    pub fn session(&self) -> &Session {
        &self.session
    }

    pub async fn get_settings_metadata(&self, user_id: &str) -> Result<Option<String>> {
        let hash_key = hash_user_id(user_id);
        let query = "SELECT updated_at FROM users WHERE id = ?";

        if let Some(updated_at) = self.query_updated_at(query, &hash_key).await? {
            return Ok(Some(updated_at.to_string()));
        }

        if let Some(legacy_key) = get_legacy_key_if_different(user_id, &hash_key)
            && let Some(updated_at) = self.query_updated_at(query, &legacy_key).await?
        {
            warn!("Found legacy data for user, will migrate on next write");
            return Ok(Some(updated_at.to_string()));
        }

        Ok(None)
    }

    async fn query_updated_at(&self, query: &str, key: &str) -> Result<Option<i64>> {
        let result = self.session.query_unpaged(query, (key,)).await?;
        let rows_result = result.into_rows_result()?;
        if let Some(row) = rows_result.rows::<(i64,)>()?.next() {
            let (updated_at,) = row?;
            return Ok(Some(updated_at));
        }
        Ok(None)
    }

    pub async fn get_user_settings(&self, user_id: &str) -> Result<Option<(Vec<u8>, String)>> {
        let hash_key = hash_user_id(user_id);
        let query = "SELECT settings, updated_at FROM users WHERE id = ?";

        if let Some((settings, updated_at)) = self.query_settings(query, &hash_key).await? {
            return Ok(Some((settings, updated_at.to_string())));
        }

        if let Some(legacy_key) = get_legacy_key_if_different(user_id, &hash_key)
            && let Some((settings, updated_at)) = self.query_settings(query, &legacy_key).await?
        {
            info!(
                "Found legacy settings for user {}, migrating to new hash format",
                user_id
            );

            if let Err(e) = self
                .migrate_user_data(user_id, &legacy_key, &hash_key, &settings, updated_at)
                .await
            {
                warn!("Failed to migrate user data: {}", e);
            }

            return Ok(Some((settings, updated_at.to_string())));
        }

        Ok(None)
    }

    async fn query_settings(&self, query: &str, key: &str) -> Result<Option<(Vec<u8>, i64)>> {
        let result = self.session.query_unpaged(query, (key,)).await?;
        let rows_result = result.into_rows_result()?;
        if let Some(row) = rows_result.rows::<(Vec<u8>, i64)>()?.next() {
            let (settings, updated_at) = row?;
            return Ok(Some((settings, updated_at)));
        }
        Ok(None)
    }

    pub async fn save_user_settings(&self, user_id: &str, settings: Vec<u8>) -> Result<i64> {
        let hash_key = hash_user_id(user_id);
        let now = chrono::Utc::now().timestamp_millis();

        let query = "INSERT INTO users (id, settings, created_at, updated_at) VALUES (?, ?, ?, ?)";
        self.session
            .query_unpaged(query, (&hash_key, &settings, now, now))
            .await?;

        self.cleanup_legacy_data(user_id, &hash_key).await;

        Ok(now)
    }

    pub async fn delete_user_settings(&self, user_id: &str) -> Result<()> {
        let hash_key = hash_user_id(user_id);

        let query = "DELETE FROM users WHERE id = ?";
        self.session.query_unpaged(query, (&hash_key,)).await?;

        self.cleanup_legacy_data(user_id, &hash_key).await;

        Ok(())
    }

    async fn cleanup_legacy_data(&self, user_id: &str, new_key: &str) {
        if let Some(legacy_key) = get_legacy_key_if_different(user_id, new_key)
            && let Err(e) = self.delete_legacy_data(&legacy_key).await
        {
            warn!("Failed to clean up legacy data: {}", e);
        }
    }

    async fn migrate_user_data(
        &self,
        user_id: &str,
        legacy_key: &str,
        new_key: &str,
        settings: &[u8],
        updated_at: i64,
    ) -> Result<()> {
        info!("Migrating user {} from legacy hash to SHA-256", user_id);

        let query = "SELECT created_at FROM users WHERE id = ?";
        let result = self.session.query_unpaged(query, (legacy_key,)).await?;
        let rows_result = result.into_rows_result()?;

        let created_at = rows_result
            .rows::<(i64,)>()?
            .next()
            .transpose()?
            .map(|row| row.0)
            .unwrap_or(updated_at);

        let insert_query =
            "INSERT INTO users (id, settings, created_at, updated_at) VALUES (?, ?, ?, ?)";
        self.session
            .query_unpaged(insert_query, (new_key, settings, created_at, updated_at))
            .await?;

        self.delete_legacy_data(legacy_key).await?;

        info!("Successfully migrated user {}", user_id);
        Ok(())
    }

    async fn delete_legacy_data(&self, legacy_key: &str) -> Result<()> {
        let query = "DELETE FROM users WHERE id = ?";
        self.session.query_unpaged(query, (legacy_key,)).await?;
        Ok(())
    }
}
