use std::{fs::File, io, io::Cursor};
use std::fs::remove_file;
use std::sync::{Arc, Mutex};
use std::time::SystemTime;

use chrono::DateTime;
use chrono::offset::Utc;
use rusqlite::{Connection, params, Result};
use teloxide::types::{InlineKeyboardButton, InlineKeyboardMarkup};

use crate::db::get_db;

#[derive(Debug)]
pub struct ChatServer {
    pub database: Arc<Mutex<Connection>>,
    pub bot_name: String,
    pub yauth: String,
    pub access_key: String,
    pub admin_id: u64,
    pub root_dir: String,
    pub tmp_dir: String,
}

pub struct PathData {
    pub name: String,
    pub path: String,
    pub hash: String,
}

impl ChatServer {
    pub fn new(db_path: String,
               bot_name: String,
               yauth: String,
               access_key: String,
               admin_id: u64,
               root_dir: String,
               tmp_dir: String,
    ) -> Self {
        let conn = get_db(Some(db_path.as_str())).unwrap();

        ChatServer {
            database: Arc::new(Mutex::new(conn)),
            bot_name,
            yauth,
            access_key,
            admin_id,
            root_dir,
            tmp_dir,
        }
    }

    pub async fn make_keyboard(&self, paths: Vec<PathData>) -> Result<InlineKeyboardMarkup> {
        let mut keyboard: Vec<Vec<InlineKeyboardButton>> = Vec::new();

        for paths in paths.chunks(1) {
            let row = paths
                .iter()
                .map(|path| InlineKeyboardButton::callback(path.name.to_owned(), path.hash.to_owned()))
                .collect();

            keyboard.push(row);
        }

        Ok(InlineKeyboardMarkup::new(keyboard))
    }

    pub async fn download_file(&self, url: &str, path: &str) -> std::io::Result<()> {
        let response = reqwest::get(url).await;
        let mut content = Cursor::new(response.unwrap().bytes().await.unwrap());
        let mut file = File::create(path)?;
        std::io::copy(
            &mut content,
            &mut file,
        )?;

        Ok(())
    }

    pub async fn delete_file(&self, path: &str) -> io::Result<()> {
        remove_file(path)?;
        Ok(())
    }

    pub async fn get_paths(&self, search_string: &str) -> Result<Vec<PathData>, rusqlite::Error> {
        let lock = self.database.lock().unwrap();
        let mut stmt = lock.prepare("
            SELECT
                name,
                path,
                hash
            FROM paths
            WHERE name like ?
            LIMIT 20")?;

        let path_data = stmt.query_map([format!("%{}%", search_string)], |row|
            Ok(PathData { name: row.get(0)?, path: row.get(1)?, hash: row.get(2)? }))?;

        let path_data_vec: Vec<PathData> = path_data.map(|d| { d.unwrap() }).collect();

        Ok(path_data_vec)
    }

    pub async fn get_file_path(&self, hash: &str) -> Result<PathData, rusqlite::Error> {
        let lock = self.database.lock().unwrap();
        let mut stmt = lock.prepare("
            SELECT
                name,
                path,
                hash
            FROM paths
            WHERE hash = ? ")?;

        let path_data = stmt.query_row([hash], |row|
            Ok(PathData { name: row.get(0)?, path: row.get(1)?, hash: row.get(2)? }))?;

        Ok(path_data)
    }

    pub async fn access_is_allowed(&self, id: &String) -> Result<bool, rusqlite::Error> {
        let lock = self.database.lock().unwrap();
        let mut stmt = lock.prepare("
            SELECT
                id
            FROM users
            WHERE id = ?")?;

        Ok(stmt.exists([id])?)
    }

    pub async fn add_new_user(
        &self,
        id: String,
        username: &str,
        first_name: &str,
        last_name: &str,
    ) -> Result<(), rusqlite::Error> {
        let lock = self.database.lock().unwrap();
        let mut stmt = lock.prepare("INSERT INTO users VALUES (?,?,?,?);")?;
        stmt.execute(params![
            id,
            username,
            first_name,
            last_name])?;

        Ok(())
    }

    pub async fn log_user_activity(
        &self,
        path: &String,
        id: &String,
        username: &str,
        first_name: &str,
        last_name: &str,
    ) -> Result<(), rusqlite::Error> {
        let system_time = SystemTime::now();
        let datetime: DateTime<Utc> = system_time.into();

        let lock = self.database.lock().unwrap();
        let mut stmt = lock.prepare("INSERT INTO log VALUES (?,?,?,?,?,?);")?;
        stmt.execute(params![
            datetime.format("%d/%m/%Y %T").to_string(),
            path,
            id,
            username,
            first_name,
            last_name])?;

        Ok(())
    }
}
