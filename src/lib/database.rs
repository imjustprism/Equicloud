use crate::lib::utils::hash_user_id;
use anyhow::Result;
use scylla::client::session::Session;
use std::sync::Arc;

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
        let result = self.session.query_unpaged(query, (&hash_key,)).await?;

        let rows_result = result.into_rows_result()?;
        for row in rows_result.rows::<(i64,)>()? {
            let (updated_at,) = row?;
            return Ok(Some(updated_at.to_string()));
        }

        Ok(None)
    }

    pub async fn get_user_settings(&self, user_id: &str) -> Result<Option<(Vec<u8>, String)>> {
        let hash_key = hash_user_id(user_id);

        let query = "SELECT settings, updated_at FROM users WHERE id = ?";
        let result = self.session.query_unpaged(query, (&hash_key,)).await?;

        let rows_result = result.into_rows_result()?;
        for row in rows_result.rows::<(Vec<u8>, i64)>()? {
            let (settings, updated_at) = row?;
            return Ok(Some((settings, updated_at.to_string())));
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

        Ok(now)
    }

    pub async fn delete_user_settings(&self, user_id: &str) -> Result<()> {
        let hash_key = hash_user_id(user_id);

        let query = "DELETE FROM users WHERE id = ?";
        self.session.query_unpaged(query, (&hash_key,)).await?;

        Ok(())
    }
}
