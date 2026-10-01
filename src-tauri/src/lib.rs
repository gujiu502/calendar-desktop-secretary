mod database;
mod desktop;
mod files;
mod monitor;
mod settings;

use database::{DailyNote, Database, NoteInput, Result};
use settings::Settings;
use tauri::{AppHandle, Emitter, Manager, State};
use tauri_plugin_autostart::ManagerExt as AutostartExt;
use tauri_plugin_dialog::DialogExt;

#[tauri::command]
fn list_notes(db: State<Database>, start: String, end: String) -> Result<Vec<DailyNote>> {
    db.list(&start, &end)
}
#[tauri::command]
fn upcoming_notes(db: State<Database>, limit: u8) -> Result<Vec<DailyNote>> {
    let now = chrono::Local::now();
    db.upcoming(
        &now.format("%Y-%m-%d").to_string(),
        &now.format("%H:%M").to_string(),
        limit,
    )
}
#[tauri::command]
fn save_note(db: State<Database>, input: NoteInput) -> Result<String> {
    db.save(input)
}
#[tauri::command]
fn complete_note(db: State<Database>, id: String, completed: bool) -> Result<()> {
    db.complete(&id, completed)
}
#[tauri::command]
fn delete_note(db: State<Database>, id: String) -> Result<()> {
    db.remove(&id)
}
#[tauri::command]
fn get_settings(app: AppHandle, db: State<Database>) -> Result<Settings> {
    let mut settings = db.get_preference::<Settings>("settings")?;
    settings.autostart = app.autolaunch().is_enabled().map_err(|e| e.to_string())?;
    Ok(settings)
}
#[tauri::command]
fn save_settings(app: AppHandle, db: State<Database>, settings: Settings) -> Result<()> {
    settings.validate()?;
    let old = db.get_preference::<Settings>("settings")?;
    let was_enabled = app.autolaunch().is_enabled().map_err(|e| e.to_string())?;
    if was_enabled != settings.autostart {
        if settings.autostart {
            app.autolaunch().enable()
        } else {
            app.autolaunch().disable()
        }
        .map_err(|e| e.to_string())?;
    }
    if let Err(e) = db.put_preference("settings", &settings) {
        let _ = if was_enabled {
            app.autolaunch().enable()
        } else {
            app.autolaunch().disable()
        };
        return Err(e);
    }
    desktop::apply_effect(&app, &settings)?;
    if old.preferred_display != settings.preferred_display
        || old.manual_monitor != settings.manual_monitor
    {
        monitor::save_position(&app)?;
        monitor::place(&app, false)?;
    }
    app.emit("settings-changed", ())
        .map_err(|e| e.to_string())?;
    Ok(())
}
#[tauri::command]
fn get_monitors() -> Result<Vec<monitor::Monitor>> {
    monitor::detect()
}
#[tauri::command]
fn window_ready(app: AppHandle, db: State<Database>) -> Result<()> {
    if db.get_preference::<Settings>("settings")?.show_on_start {
        desktop::show(&app)?;
    }
    Ok(())
}
#[tauri::command]
fn backup_data(db: State<Database>) -> Result<String> {
    Ok(db.backup()?.display().to_string())
}
#[tauri::command]
fn open_data_folder(db: State<Database>) -> Result<()> {
    std::process::Command::new("explorer.exe")
        .arg(&db.folder)
        .spawn()
        .map_err(|e| e.to_string())?;
    Ok(())
}
#[tauri::command]
async fn export_data(app: AppHandle, format: String) -> Result<Option<String>> {
    if !["json", "csv"].contains(&format.as_str()) {
        return Err("导出格式无效".into());
    }
    let file = app
        .dialog()
        .file()
        .set_title("导出日签")
        .add_filter(format.to_uppercase(), &[&format])
        .set_file_name(format!(
            "日签-{}.{}",
            chrono::Local::now().format("%Y-%m-%d"),
            format
        ))
        .blocking_save_file();
    let Some(file) = file else {
        return Ok(None);
    };
    let path = file.into_path().map_err(|e| e.to_string())?;
    let notes = app.state::<Database>().list("1900-01-01", "9999-12-31")?;
    let bytes = if format == "json" {
        serde_json::to_vec_pretty(&notes).map_err(|e| e.to_string())?
    } else {
        desktop::csv(&notes).into_bytes()
    };
    let tmp = path.with_file_name(format!(".cds-export-{}.tmp", uuid::Uuid::new_v4()));
    std::fs::write(&tmp, bytes).map_err(|e| e.to_string())?;
    if let Err(e) = files::publish(&tmp, &path) {
        let _ = std::fs::remove_file(&tmp);
        return Err(e);
    }
    Ok(Some(path.display().to_string()))
}
#[tauri::command]
async fn import_data(app: AppHandle) -> Result<Option<usize>> {
    let file = app
        .dialog()
        .file()
        .set_title("导入日签 JSON（合并，不覆盖已有日签）")
        .add_filter("日签 JSON", &["json"])
        .blocking_pick_file();
    let Some(file) = file else {
        return Ok(None);
    };
    let count = app
        .state::<Database>()
        .import(&file.into_path().map_err(|e| e.to_string())?)?;
    Ok(Some(count))
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|app, _, _| {
            let _ = desktop::show(app);
        }))
        .plugin(tauri_plugin_autostart::Builder::new().build())
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            let folder = app.path().app_data_dir()?;
            app.manage(Database::open(folder).map_err(std::io::Error::other)?);
            app.manage(monitor::ActiveMonitor::default());
            let settings = app
                .state::<Database>()
                .get_preference::<Settings>("settings")
                .map_err(std::io::Error::other)?;
            desktop::setup_tray(app.handle())?;
            monitor::place(app.handle(), false).map_err(std::io::Error::other)?;
            desktop::apply_effect(app.handle(), &settings).map_err(std::io::Error::other)?;
            monitor::start(app.handle()).map_err(std::io::Error::other)?;
            Ok(())
        })
        .on_window_event(|window, event| match event {
            tauri::WindowEvent::CloseRequested { api, .. } => {
                let db = window.state::<Database>();
                let _ = monitor::save_position(window.app_handle());
                if db
                    .get_preference::<Settings>("settings")
                    .map(|s| s.close_to_tray)
                    .unwrap_or(true)
                {
                    api.prevent_close();
                    let _ = window.hide();
                } else {
                    window.app_handle().exit(0);
                }
            }
            tauri::WindowEvent::Moved(_) | tauri::WindowEvent::Resized(_) => {
                if let Some(watcher) = window.try_state::<monitor::Watcher>() {
                    let _ = watcher.0.send(monitor::Change::Position);
                }
            }
            tauri::WindowEvent::ScaleFactorChanged { .. } => {
                if let Some(watcher) = window.try_state::<monitor::Watcher>() {
                    let _ = watcher.0.send(monitor::Change::Position);
                }
            }
            _ => {}
        })
        .invoke_handler(tauri::generate_handler![
            list_notes,
            upcoming_notes,
            save_note,
            complete_note,
            delete_note,
            get_settings,
            save_settings,
            get_monitors,
            window_ready,
            backup_data,
            open_data_folder,
            export_data,
            import_data
        ])
        .run(tauri::generate_context!())
        .expect("Calendar Desktop Secretary failed to start");
}
