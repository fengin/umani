pub mod schema;

use rusqlite::Connection;
use std::sync::Mutex;

pub struct Database {
    pub conn: Mutex<Connection>,
}

impl Database {
    pub fn new(app_data_dir: &std::path::Path) -> Result<Self, rusqlite::Error> {
        std::fs::create_dir_all(app_data_dir).ok();
        let db_path = app_data_dir.join("savor.db");
        let conn = Connection::open(db_path)?;

        // 启用 WAL 模式提升并发性能
        conn.execute_batch("PRAGMA journal_mode=WAL;")?;
        conn.execute_batch("PRAGMA foreign_keys=ON;")?;

        // 初始化表结构
        schema::create_tables(&conn)?;

        Ok(Database {
            conn: Mutex::new(conn),
        })
    }
}
