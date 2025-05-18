use sqlx::{sqlite::SqlitePool, Row};
use crate::config::Config;
use crate::download::DownloadStatus;
use crate::error::{DownloadError, Result};

pub struct Storage {
    pool: SqlitePool,
}

impl Storage {
    pub async fn new(config: &Config) -> Result<Self> {
        let pool = SqlitePool::connect(&format!("sqlite:{}", config.storage.db_path))
            .await
            .map_err(|e| DownloadError::DatabaseError(e.to_string()))?;

        sqlx::migrate!("./migrations")
            .run(&pool)
            .await
            .map_err(|e| DownloadError::DatabaseError(e.to_string()))?;

        Ok(Self { pool })
    }

    pub async fn add_download(&mut self, download: &DownloadStatus) -> Result<()> {
        sqlx::query(
            r#"
            INSERT INTO downloads (
                id, url, filename, progress, speed, state, error
            ) VALUES (?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(&download.id)
        .bind(&download.url)
        .bind(&download.filename)
        .bind(download.progress)
        .bind(download.speed)
        .bind(serde_json::to_string(&download.state).unwrap())
        .bind(&download.error)
        .execute(&self.pool)
        .await
        .map_err(|e| DownloadError::DatabaseError(e.to_string()))?;

        Ok(())
    }

    pub async fn update_download(&mut self, download: &DownloadStatus) -> Result<()> {
        sqlx::query(
            r#"
            UPDATE downloads
            SET progress = ?, speed = ?, state = ?, error = ?
            WHERE id = ?
            "#,
        )
        .bind(download.progress)
        .bind(download.speed)
        .bind(serde_json::to_string(&download.state).unwrap())
        .bind(&download.error)
        .bind(&download.id)
        .execute(&self.pool)
        .await
        .map_err(|e| DownloadError::DatabaseError(e.to_string()))?;

        Ok(())
    }

    pub async fn get_download(&self, id: &str) -> Result<Option<DownloadStatus>> {
        sqlx::query("SELECT * FROM downloads WHERE id = ?")
            .bind(id)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| DownloadError::DatabaseError(e.to_string()))?
            .map(|row| {
                Ok(DownloadStatus {
                    id: row.get("id"),
                    url: row.get("url"),
                    filename: row.get("filename"),
                    progress: row.get("progress"),
                    speed: row.get("speed"),
                    state: serde_json::from_str(&row.get::<String, _>("state"))
                        .map_err(|e| DownloadError::DatabaseError(e.to_string()))?,
                    error: row.get("error"),
                })
            })
            .transpose()
    }
}