use anyhow::Result;
use scylla::client::session::Session;
use std::fs;
use std::path::Path;
use tracing::{debug, info, warn};

pub struct MigrationRunner<'a> {
    session: &'a Session,
}

impl<'a> MigrationRunner<'a> {
    pub fn new(session: &'a Session) -> Self {
        Self { session }
    }

    pub async fn run_migrations(&self) -> Result<()> {
        let migrations_dir = Path::new("migrations");

        if !migrations_dir.exists() {
            warn!("No migrations directory found, skipping migrations");
            return Ok(());
        }

        let mut migration_files = fs::read_dir(migrations_dir)?
            .filter_map(|entry| {
                let entry = entry.ok()?;
                let path = entry.path();
                if path.extension()? == "cql" {
                    Some(path)
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();

        migration_files.sort();

        let migration_count = migration_files.len();

        for migration_file in migration_files {
            self.run_migration(&migration_file).await?;
        }

        info!("Executed {} migrations", migration_count);

        Ok(())
    }

    async fn run_migration(&self, file_path: &Path) -> Result<()> {
        let filename = file_path
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("unknown");

        debug!("Running migration: {}", filename);

        let content = fs::read_to_string(file_path)?;

        let cleaned_content = content
            .lines()
            .filter(|line| !line.trim().starts_with("--") && !line.trim().is_empty())
            .collect::<Vec<_>>()
            .join(" ");

        let statements: Vec<&str> = cleaned_content
            .split(';')
            .map(|s| s.trim())
            .filter(|s| !s.is_empty())
            .collect();

        for statement in statements {
            debug!("Executing: {}", statement);
            self.session.query_unpaged(statement, &[]).await?;
        }

        debug!("Completed migration: {}", filename);
        Ok(())
    }
}
