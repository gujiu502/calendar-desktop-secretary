use crate::database::{DailyNote, Result};
use tauri::{
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    AppHandle, Emitter, Manager,
};

pub fn show(app: &AppHandle) -> Result<()> {
    let window = app.get_webview_window("main").ok_or("主窗口不存在")?;
    window.unminimize().map_err(|e| e.to_string())?;
    window.show().map_err(|e| e.to_string())?;
    window.set_focus().map_err(|e| e.to_string())
}
pub fn setup_tray(app: &AppHandle) -> tauri::Result<()> {
    let open = MenuItem::with_id(app, "open", "打开", true, None::<&str>)?;
    let add = MenuItem::with_id(app, "add", "快速添加日签", true, None::<&str>)?;
    let today = MenuItem::with_id(app, "today", "今天", true, None::<&str>)?;
    let settings = MenuItem::with_id(app, "settings", "设置", true, None::<&str>)?;
    let quit = MenuItem::with_id(app, "quit", "退出", true, None::<&str>)?;
    let menu = Menu::with_items(app, &[&open, &add, &today, &settings, &quit])?;
    let mut tray = TrayIconBuilder::with_id("calendar")
        .tooltip("日签 · Calendar Desktop Secretary")
        .menu(&menu)
        .show_menu_on_left_click(false)
        .on_menu_event(|app, event| {
            let action = event.id.as_ref();
            if action == "quit" {
                let _ = crate::monitor::save_position(app);
                app.exit(0);
                return;
            }
            let _ = show(app);
            if action != "open" {
                let _ = app.emit("tray-action", action);
            }
        })
        .on_tray_icon_event(|tray, event| {
            if matches!(
                event,
                TrayIconEvent::Click {
                    button: MouseButton::Left,
                    button_state: MouseButtonState::Up,
                    ..
                }
            ) {
                let _ = show(tray.app_handle());
            }
        });
    if let Some(icon) = app.default_window_icon() {
        tray = tray.icon(icon.clone());
    }
    tray.build(app)?;
    Ok(())
}
pub fn csv(notes: &[DailyNote]) -> String {
    fn field(s: &str) -> String {
        let trimmed = s.trim_start();
        let safe = if trimmed.starts_with(['=', '+', '-', '@', '\t', '\r']) {
            format!("'{s}")
        } else {
            s.to_string()
        };
        format!("\"{}\"", safe.replace('"', "\"\""))
    }
    let mut csv = String::from(
        "\u{feff}id,date,endDate,time,title,content,category,priority,completed,created_at,updated_at\r\n",
    );
    for n in notes {
        csv.push_str(
            &[
                &n.id,
                &n.date,
                n.end_date.as_deref().unwrap_or(""),
                n.time.as_deref().unwrap_or(""),
                &n.title,
                &n.content,
                &n.category,
                &n.priority.to_string(),
                &n.completed.to_string(),
                &n.created_at,
                &n.updated_at,
            ]
            .map(field)
            .join(","),
        );
        csv.push_str("\r\n");
    }
    csv
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn csv_escapes_formula_quotes_and_newlines() {
        let note = DailyNote {
            id: "id".into(),
            date: "2026-10-01".into(),
            end_date: Some("2026-10-03".into()),
            time: None,
            title: "=SUM(1,2)".into(),
            content: "hello \"world\"\nline".into(),
            category: "普通".into(),
            priority: 0,
            completed: false,
            created_at: String::new(),
            updated_at: String::new(),
        };
        let csv = csv(&[note]);
        assert!(csv.contains("\"'=SUM(1,2)\""));
        assert!(csv.contains("hello \"\"world\"\"\nline"));
        assert!(csv.contains("\"2026-10-01\",\"2026-10-03\""));
    }
}
