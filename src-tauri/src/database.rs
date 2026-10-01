use chrono::{Local, NaiveDate, NaiveTime, Utc};
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use std::{
    fs,
    path::{Path, PathBuf},
    sync::Mutex,
};

pub type Result<T> = std::result::Result<T, String>;

pub struct Database {
    // ponytail: one SQLite connection; a pool only if measured concurrent load requires it.
    pub conn: Mutex<Connection>,
    pub folder: PathBuf,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct DailyNote {
    pub id: String,
    pub date: String,
    pub time: Option<String>,
    pub title: String,
    pub content: String,
    pub category: String,
    pub priority: u8,
    pub completed: bool,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct NoteInput {
    pub id: Option<String>,
    pub date: String,
    pub time: Option<String>,
    pub title: String,
    pub content: String,
    pub category: String,
    pub priority: u8,
}

pub fn validate_date(value: &str) -> Result<()> {
    let date = NaiveDate::parse_from_str(value, "%Y-%m-%d").map_err(|_| "日期无效")?;
    if value.len() != 10
        || date.format("%Y-%m-%d").to_string() != value
        || value < "1900-01-01"
        || value > "9999-12-31"
    {
        return Err("日期必须在 1900 至 9999 年之间".into());
    }
    Ok(())
}

fn validate_fields(
    date: &str,
    time: &Option<String>,
    title: &str,
    content: &str,
    category: &str,
    priority: u8,
) -> Result<()> {
    validate_date(date)?;
    if let Some(t) = time {
        if t.len() != 5 || NaiveTime::parse_from_str(t, "%H:%M").is_err() {
            return Err("时间无效".into());
        }
    }
    if title.trim().is_empty() || title.chars().count() > 200 {
        return Err("标题需要 1–200 个字符".into());
    }
    if content.len() > 50_000 {
        return Err("内容过长".into());
    }
    if !["普通", "重要", "学习", "生活"].contains(&category) || priority > 3 {
        return Err("分类或优先级无效".into());
    }
    Ok(())
}

fn read_note(row: &rusqlite::Row<'_>) -> rusqlite::Result<DailyNote> {
    Ok(DailyNote {
        id: row.get(0)?,
        date: row.get(1)?,
        time: row.get(2)?,
        title: row.get(3)?,
        content: row.get(4)?,
        category: row.get(5)?,
        priority: row.get(6)?,
        completed: row.get(7)?,
        created_at: row.get(8)?,
        updated_at: row.get(9)?,
    })
}

const SELECT: &str = "SELECT id,date,time,title,content,category,priority,completed,created_at,updated_at FROM daily_notes";

impl Database {
    pub fn open(folder: PathBuf) -> Result<Self> {
        fs::create_dir_all(&folder).map_err(|e| e.to_string())?;
        let conn = Connection::open(folder.join("database.sqlite")).map_err(|e| e.to_string())?;
        conn.busy_timeout(std::time::Duration::from_secs(5))
            .map_err(|e| e.to_string())?;
        Self::migrate(&conn)?;
        Ok(Self {
            conn: Mutex::new(conn),
            folder,
        })
    }

    fn migrate(conn: &Connection) -> Result<()> {
        conn.execute_batch(
            "PRAGMA journal_mode=WAL; PRAGMA foreign_keys=ON;
            CREATE TABLE IF NOT EXISTS daily_notes (
                id TEXT PRIMARY KEY, date TEXT NOT NULL, time TEXT, title TEXT NOT NULL,
                content TEXT NOT NULL DEFAULT '', category TEXT NOT NULL DEFAULT '普通',
                priority INTEGER NOT NULL DEFAULT 0 CHECK(priority BETWEEN 0 AND 3),
                completed INTEGER NOT NULL DEFAULT 0 CHECK(completed IN (0,1)),
                created_at TEXT NOT NULL, updated_at TEXT NOT NULL);
            CREATE INDEX IF NOT EXISTS idx_daily_notes_date ON daily_notes(date);
            CREATE INDEX IF NOT EXISTS idx_daily_notes_datetime ON daily_notes(date,time);
            CREATE INDEX IF NOT EXISTS idx_daily_notes_completed ON daily_notes(completed);
            CREATE TABLE IF NOT EXISTS preferences (key TEXT PRIMARY KEY, value TEXT NOT NULL);
            PRAGMA user_version=1;",
        )
        .map_err(|e| e.to_string())
    }

    pub fn list(&self, start: &str, end: &str) -> Result<Vec<DailyNote>> {
        validate_date(start)?;
        validate_date(end)?;
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        let mut stmt = conn.prepare(&format!("{SELECT} WHERE date BETWEEN ?1 AND ?2 ORDER BY date,COALESCE(time,'23:59'),priority DESC,created_at")).map_err(|e| e.to_string())?;
        let rows = stmt
            .query_map(params![start, end], read_note)
            .map_err(|e| e.to_string())?;
        rows.collect::<rusqlite::Result<Vec<_>>>()
            .map_err(|e| e.to_string())
    }

    pub fn upcoming(&self, date: &str, time: &str, limit: u8) -> Result<Vec<DailyNote>> {
        validate_date(date)?;
        if ![5, 8, 10].contains(&limit) || NaiveTime::parse_from_str(time, "%H:%M").is_err() {
            return Err("查询参数无效".into());
        }
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        let mut stmt = conn.prepare(&format!("{SELECT} WHERE completed=0 AND (date>?1 OR (date=?1 AND (time IS NULL OR time>=?2))) ORDER BY date,COALESCE(time,'23:59'),priority DESC,created_at LIMIT ?3")).map_err(|e| e.to_string())?;
        let rows = stmt
            .query_map(params![date, time, limit], read_note)
            .map_err(|e| e.to_string())?;
        rows.collect::<rusqlite::Result<Vec<_>>>()
            .map_err(|e| e.to_string())
    }

    pub fn save(&self, input: NoteInput) -> Result<String> {
        validate_fields(
            &input.date,
            &input.time,
            &input.title,
            &input.content,
            &input.category,
            input.priority,
        )?;
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        let now = Utc::now().to_rfc3339();
        if let Some(id) = input.id {
            let count = conn.execute("UPDATE daily_notes SET date=?2,time=?3,title=?4,content=?5,category=?6,priority=?7,updated_at=?8 WHERE id=?1", params![id,input.date,input.time,input.title.trim(),input.content,input.category,input.priority,now]).map_err(|e| e.to_string())?;
            if count == 0 {
                return Err("日签已不存在，请刷新".into());
            }
            Ok(id)
        } else {
            let id = uuid::Uuid::new_v4().to_string();
            conn.execute(
                "INSERT INTO daily_notes VALUES(?1,?2,?3,?4,?5,?6,?7,0,?8,?8)",
                params![
                    id,
                    input.date,
                    input.time,
                    input.title.trim(),
                    input.content,
                    input.category,
                    input.priority,
                    now
                ],
            )
            .map_err(|e| e.to_string())?;
            Ok(id)
        }
    }

    pub fn complete(&self, id: &str, completed: bool) -> Result<()> {
        let count = self
            .conn
            .lock()
            .map_err(|e| e.to_string())?
            .execute(
                "UPDATE daily_notes SET completed=?2,updated_at=?3 WHERE id=?1",
                params![id, completed, Utc::now().to_rfc3339()],
            )
            .map_err(|e| e.to_string())?;
        if count == 0 {
            return Err("日签已不存在".into());
        }
        Ok(())
    }

    pub fn remove(&self, id: &str) -> Result<()> {
        self.conn
            .lock()
            .map_err(|e| e.to_string())?
            .execute("DELETE FROM daily_notes WHERE id=?1", [id])
            .map_err(|e| e.to_string())?;
        Ok(())
    }

    pub fn backup(&self) -> Result<PathBuf> {
        let dir = self.folder.join("backup");
        fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
        let dest = dir.join(format!("{}.sqlite", Local::now().format("%Y-%m-%d")));
        if dest.exists() {
            return Ok(dest);
        }
        let tmp = dest.with_extension("sqlite.tmp");
        self.conn
            .lock()
            .map_err(|e| e.to_string())?
            .backup("main", &tmp, None)
            .map_err(|e| e.to_string())?;
        crate::files::publish(&tmp, &dest)?;
        let mut backups = fs::read_dir(&dir)
            .map_err(|e| e.to_string())?
            .filter_map(|e| e.ok())
            .map(|e| e.path())
            .filter(|p| {
                p.extension().is_some_and(|s| s == "sqlite")
                    && p.file_stem()
                        .and_then(|s| s.to_str())
                        .is_some_and(|s| NaiveDate::parse_from_str(s, "%Y-%m-%d").is_ok())
            })
            .collect::<Vec<_>>();
        backups.sort();
        for path in backups.iter().take(backups.len().saturating_sub(30)) {
            fs::remove_file(path).map_err(|e| e.to_string())?;
        }
        Ok(dest)
    }

    pub fn import(&self, path: &Path) -> Result<usize> {
        if fs::metadata(path).map_err(|e| e.to_string())?.len() > 20_000_000 {
            return Err("导入文件不得超过 20 MB".into());
        }
        let notes: Vec<DailyNote> =
            serde_json::from_slice(&fs::read(path).map_err(|e| e.to_string())?)
                .map_err(|e| format!("JSON 格式无效：{e}"))?;
        for n in &notes {
            validate_fields(
                &n.date,
                &n.time,
                &n.title,
                &n.content,
                &n.category,
                n.priority,
            )?;
            if uuid::Uuid::parse_str(&n.id).is_err()
                || chrono::DateTime::parse_from_rfc3339(&n.created_at).is_err()
                || chrono::DateTime::parse_from_rfc3339(&n.updated_at).is_err()
            {
                return Err("日签 ID 或时间戳无效".into());
            }
        }
        self.backup()?;
        let mut conn = self.conn.lock().map_err(|e| e.to_string())?;
        let tx = conn.transaction().map_err(|e| e.to_string())?;
        // Existing IDs are preserved: importing an old export must not overwrite newer work.
        let mut count = 0;
        for n in notes {
            count += tx
                .execute(
                    "INSERT OR IGNORE INTO daily_notes VALUES(?1,?2,?3,?4,?5,?6,?7,?8,?9,?10)",
                    params![
                        n.id,
                        n.date,
                        n.time,
                        n.title,
                        n.content,
                        n.category,
                        n.priority,
                        n.completed,
                        n.created_at,
                        n.updated_at
                    ],
                )
                .map_err(|e| e.to_string())?;
        }
        tx.commit().map_err(|e| e.to_string())?;
        Ok(count)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn db() -> Database {
        let conn = Connection::open_in_memory().unwrap();
        Database::migrate(&conn).unwrap();
        Database {
            conn: Mutex::new(conn),
            folder: std::env::temp_dir().join(uuid::Uuid::new_v4().to_string()),
        }
    }
    fn input(date: &str, time: Option<&str>, priority: u8) -> NoteInput {
        NoteInput {
            id: None,
            date: date.into(),
            time: time.map(str::to_string),
            title: "作业".into(),
            content: String::new(),
            category: "学习".into(),
            priority,
        }
    }
    #[test]
    fn crud_and_upcoming_contract() {
        let db = db();
        let id = db.save(input("2026-10-01", Some("09:00"), 0)).unwrap();
        db.save(input("2026-10-01", None, 0)).unwrap();
        db.save(input("2026-10-02", Some("08:00"), 1)).unwrap();
        db.save(input("2026-10-02", Some("08:00"), 3)).unwrap();
        let rows = db.upcoming("2026-10-01", "16:00", 8).unwrap();
        assert_eq!(rows.len(), 3);
        assert!(rows[0].time.is_none());
        assert_eq!(rows[1].priority, 3);
        db.complete(&rows[1].id, true).unwrap();
        assert_eq!(db.upcoming("2026-10-01", "16:00", 8).unwrap().len(), 2);
        let mut edit = input("2026-10-03", None, 0);
        edit.id = Some(id.clone());
        db.save(edit).unwrap();
        assert_eq!(db.list("2026-10-03", "2026-10-03").unwrap().len(), 1);
        db.remove(&id).unwrap();
        assert!(db.list("2026-10-03", "2026-10-03").unwrap().is_empty());
        assert!(db.save(input("2026-02-30", None, 0)).is_err());
        assert!(db.save(input("2026-10-01", Some("25:00"), 0)).is_err());
    }
    #[test]
    fn online_backup_and_transactional_import() {
        let root = std::env::temp_dir().join(uuid::Uuid::new_v4().to_string());
        let db = Database::open(root.clone()).unwrap();
        db.save(input("2026-10-01", None, 0)).unwrap();
        let backup = db.backup().unwrap();
        assert!(backup.exists());
        let snapshot = Connection::open(backup).unwrap();
        assert_eq!(
            snapshot
                .query_row("SELECT COUNT(*) FROM daily_notes", [], |r| r
                    .get::<_, u32>(0))
                .unwrap(),
            1
        );
        let export = root.join("export.json");
        fs::write(
            &export,
            serde_json::to_vec(&db.list("1900-01-01", "9999-12-31").unwrap()).unwrap(),
        )
        .unwrap();
        assert_eq!(db.import(&export).unwrap(), 0);
        fs::write(&export, b"[{\"date\":\"invalid\"}]").unwrap();
        assert!(db.import(&export).is_err());
        assert_eq!(db.list("1900-01-01", "9999-12-31").unwrap().len(), 1);
        drop(snapshot);
        drop(db);
        fs::remove_dir_all(root).unwrap();
    }
}
