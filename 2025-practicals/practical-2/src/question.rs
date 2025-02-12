use rusqlite::{params, Connection, Result};

pub struct Database {
    conn: Connection,
}

impl Database {
    pub fn new(db_path: &str) -> rusqlite::Result<Self> {
        let conn = Connection::open(db_path)?;
        conn.execute(
            "CREATE TABLE IF NOT EXISTS friends (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT NOT NULL UNIQUE,
                phone TEXT NOT NULL
            );",
            [],
        )?;
        Ok(Self { conn })
    }

    pub fn add_friend(&self, name: &str, phone: &str) -> rusqlite::Result<()> {
        self.conn.execute(
            "INSERT INTO friends (name, phone) VALUES (?1, ?2);",
            params![name, phone],
        )?;
        Ok(())
    }

    pub fn get_friend(&self, name: &str) -> rusqlite::Result<Option<String>> {
        let mut stmt = self
            .conn
            .prepare("SELECT phone FROM friends WHERE name = ?1;")?;
        let mut rows = stmt.query(params![name])?;
        if let Some(row) = rows.next()? {
            let phone: String = row.get(0)?;
            Ok(Some(phone))
        } else {
            Ok(None)
        }
    }

    pub fn delete_friend(&self, name: &str) -> rusqlite::Result<()> {
        self.conn
            .execute("DELETE FROM friends WHERE name = ?1;", params![name])?;
        Ok(())
    }
}
