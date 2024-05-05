use rusqlite::{Connection, Result};

pub fn get_db(path: Option<&str>) -> Result<Connection> {
    let db = match path {
        Some(path) => {
            let path = path;
            Connection::open(&path)?
        }
        None => {
            Connection::open_in_memory()?
        }
    };
    run_migrations(&db)?;
    Ok(db)
}

fn run_migrations(conn: &Connection) -> Result<()> {
    
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS log (
            date	    TEXT,
            path	    TEXT,
            id	        TEXT,
            username	TEXT,
            first_name	TEXT,
            last_name	TEXT);
            
            CREATE TABLE IF NOT EXISTS paths (
            name	    TEXT,
            path	    TEXT,
            hash	    TEXT);
            
            CREATE TABLE IF NOT EXISTS users (
            id          TEXT,
            username	TEXT,
            first_name	TEXT,
            last_name	TEXT);
            
            CREATE INDEX IF NOT EXISTS hash ON paths (
	        hash	    ASC);"
    )?;
    Ok(())
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_setup() {
        let db = get_db(None);
        assert_eq!(db.is_ok(), true);
    }
}