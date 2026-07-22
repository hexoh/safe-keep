pub mod compare;
pub mod migrations;
pub mod models;
pub mod queries;

use rusqlite::Connection;
use std::sync::Mutex;

pub struct Database {
  pub conn: Mutex<Connection>,
}

impl Database {
  pub fn new(db_path: &std::path::Path) -> Result<Self, String> {
    let conn = Connection::open(db_path).map_err(|e| e.to_string())?;
    conn
      .execute_batch("PRAGMA journal_mode=WAL; PRAGMA foreign_keys=ON;")
      .map_err(|e| e.to_string())?;
    migrations::run_migrations(&conn).map_err(|e| e.to_string())?;
    Ok(Self {
      conn: Mutex::new(conn),
    })
  }
}
