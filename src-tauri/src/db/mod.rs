pub mod migrations;
pub mod models;

use std::sync::Mutex;
use tauri::Manager;

pub struct Db(pub Mutex<rusqlite::Connection>);

pub fn init(app: &tauri::AppHandle) -> Result<(), Box<dyn std::error::Error>> {
    let app_dir = app.path().app_data_dir()?;
    std::fs::create_dir_all(&app_dir)?;
    let db_path = app_dir.join("scx-music.db");

    let conn = rusqlite::Connection::open(&db_path)?;
    migrations::run_migrations(&conn)?;

    app.manage(Db(Mutex::new(conn)));
    Ok(())
}
