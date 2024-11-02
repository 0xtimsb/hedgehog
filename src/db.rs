use crate::download_item::{DownloadItem, DownloadStatus};
use rusqlite::{Connection, Result};

pub fn init_db() -> Result<Connection> {
    let conn = Connection::open("downloads.db")?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS downloads (
            id INTEGER PRIMARY KEY,
            url TEXT NOT NULL,
            file_path TEXT NOT NULL,
            total_size INTEGER,
            status TEXT NOT NULL,
            downloaded_bytes INTEGER DEFAULT 0,
            created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
        )",
        [],
    )?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS chunks (
            id INTEGER PRIMARY KEY,
            download_id INTEGER,
            chunk_number INTEGER,
            start_byte INTEGER,
            end_byte INTEGER,
            status TEXT NOT NULL,
            FOREIGN KEY(download_id) REFERENCES downloads(id)
        )",
        [],
    )?;

    Ok(conn)
}

pub fn save_download(conn: &Connection, item: &DownloadItem) -> Result<()> {
    let (status_str, downloaded_bytes) = match &item.status {
        DownloadStatus::InProgress {
            downloaded_bytes, ..
        } => ("InProgress".to_string(), *downloaded_bytes),
        DownloadStatus::Completed => ("Completed".to_string(), item.total_size.unwrap_or(0) as u64),
        _ => (item.status.to_string(), 0),
    };

    println!(
        "Saving download status: {}, bytes: {}",
        status_str, downloaded_bytes
    ); // Debug log

    conn.execute(
        "INSERT OR REPLACE INTO downloads (id, url, file_path, total_size, status, downloaded_bytes) 
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        (
            item.id,
            &item.url,
            &item.file_path,
            item.total_size,
            status_str,
            downloaded_bytes,
        ),
    )?;
    Ok(())
}

pub fn load_downloads(conn: &Connection) -> Result<Vec<DownloadItem>> {
    let mut stmt = conn.prepare(
        "SELECT id, url, file_path, total_size, status, downloaded_bytes FROM downloads",
    )?;

    let items = stmt.query_map([], |row| {
        let status_str: String = row.get(4)?;
        let downloaded_bytes: u64 = row.get(5)?;
        let total_size: Option<i64> = row.get(3)?;

        let status = match status_str.as_str() {
            "InProgress" | "Pending" => DownloadStatus::InProgress {
                progress: if let Some(total) = total_size {
                    (downloaded_bytes as f32 / total as f32) * 100.0
                } else {
                    0.0
                },
                downloaded_bytes,
            },
            "Completed" => DownloadStatus::Completed,
            "Cancelled" => DownloadStatus::Cancelled,
            s if s.starts_with("Failed: ") => DownloadStatus::Failed(s[8..].to_string()),
            _ => DownloadStatus::Pending,
        };

        println!(
            "Loading download - status: {}, bytes: {}",
            status_str, downloaded_bytes
        ); // Debug log
        println!("Total size: {:?}", total_size);
        println!("Downloaded bytes: {}", downloaded_bytes);

        Ok(DownloadItem {
            id: row.get(0)?,
            url: row.get(1)?,
            file_path: row.get(2)?,
            total_size,
            status,
        })
    })?;

    items.collect()
}
