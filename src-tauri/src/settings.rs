use crate::database::{Database, Result};
use rusqlite::params;
use serde::{de::DeserializeOwned, Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default, deny_unknown_fields)]
pub struct Settings {
    pub preferred_display: String,
    pub manual_monitor: Option<String>,
    pub auto_return: bool,
    pub remember_position: bool,
    pub remember_size: bool,
    pub close_to_tray: bool,
    pub show_on_start: bool,
    pub autostart: bool,
    pub opacity: f64,
    pub effect: String,
    pub corner_radius: u8,
    pub week_start: u8,
    pub upcoming_count: u8,
}
impl Default for Settings {
    fn default() -> Self {
        Self {
            preferred_display: "display2".into(),
            manual_monitor: None,
            auto_return: true,
            remember_position: true,
            remember_size: true,
            close_to_tray: true,
            show_on_start: true,
            autostart: false,
            opacity: 0.72,
            effect: "acrylic".into(),
            corner_radius: 18,
            week_start: 1,
            upcoming_count: 8,
        }
    }
}
impl Settings {
    pub fn validate(&self) -> Result<()> {
        if !["display2", "primary", "last", "manual"].contains(&self.preferred_display.as_str())
            || !["mica", "acrylic", "blur", "none"].contains(&self.effect.as_str())
            || !self.opacity.is_finite()
            || !(0.35..=1.0).contains(&self.opacity)
            || self.corner_radius > 28
            || self.week_start > 1
            || ![5, 8, 10].contains(&self.upcoming_count)
        {
            return Err("设置参数无效".into());
        }
        if self.preferred_display == "manual"
            && self
                .manual_monitor
                .as_ref()
                .is_none_or(|s| s.is_empty() || s.len() > 1000)
        {
            return Err("请选择显示器".into());
        }
        Ok(())
    }
}

impl Database {
    pub fn get_preference<T: DeserializeOwned + Default>(&self, key: &str) -> Result<T> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        let mut stmt = conn
            .prepare("SELECT value FROM preferences WHERE key=?1")
            .map_err(|e| e.to_string())?;
        let mut rows = stmt.query([key]).map_err(|e| e.to_string())?;
        match rows.next().map_err(|e| e.to_string())? {
            Some(row) => serde_json::from_str(&row.get::<_, String>(0).map_err(|e| e.to_string())?)
                .map_err(|e| format!("设置读取失败：{e}")),
            None => Ok(T::default()),
        }
    }
    pub fn put_preference<T: Serialize>(&self, key: &str, value: &T) -> Result<()> {
        self.conn.lock().map_err(|e|e.to_string())?.execute("INSERT INTO preferences(key,value) VALUES(?1,?2) ON CONFLICT(key) DO UPDATE SET value=excluded.value",params![key,serde_json::to_string(value).map_err(|e|e.to_string())?]).map_err(|e|e.to_string())?;
        Ok(())
    }
}
