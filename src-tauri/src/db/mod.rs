pub mod schema;

use rusqlite::Connection;
use std::path::PathBuf;
use crate::error::AppResult;

pub fn open_db() -> AppResult<Connection> {
    let path = db_path()?;
    let conn = Connection::open(&path)?;
    conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA foreign_keys=ON;")?;
    schema::run_migrations(&conn)?;
    Ok(conn)
}

fn db_path() -> AppResult<PathBuf> {
    let base = dirs::data_dir()
        .ok_or_else(|| crate::error::AppError::Config("No se pudo determinar data_dir".to_string()))?;
    let dir = base.join("ia-middleware");
    std::fs::create_dir_all(&dir)?;
    Ok(dir.join("middleware.db"))
}
