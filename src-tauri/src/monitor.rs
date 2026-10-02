use crate::{
    database::{Database, Result},
    settings::Settings,
};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    sync::{
        mpsc::{self, Sender},
        Mutex,
    },
    time::Duration,
};
use tauri::{AppHandle, Emitter, Manager, PhysicalPosition, PhysicalSize, WebviewWindow};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Monitor {
    pub id: String,
    pub name: String,
    pub number: Option<u32>,
    pub primary: bool,
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
    pub scale: f64,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Placement {
    pub offset_x: f64,
    pub offset_y: f64,
    pub logical_width: f64,
    pub logical_height: f64,
    pub physical_width: u32,
    pub physical_height: u32,
    pub scale_factor: f64,
}
#[derive(Default, Serialize, Deserialize)]
struct Positions {
    last: Option<String>,
    monitors: HashMap<String, Placement>,
}

#[derive(Default)]
pub struct ActiveMonitor {
    pub current: Mutex<Option<String>>,
    pub connected: Mutex<Vec<String>>,
}

pub fn topology_target<'a>(
    monitors: &'a [Monitor],
    settings: &Settings,
    previous: Option<&str>,
    last: Option<&str>,
    newly_connected: bool,
) -> Option<&'a Monitor> {
    if !settings.auto_return || !newly_connected {
        return monitors
            .iter()
            .find(|m| Some(m.id.as_str()) == previous)
            .or_else(|| monitors.iter().find(|m| m.primary))
            .or_else(|| monitors.first());
    }
    choose_monitor(monitors, settings, last)
}

pub fn choose_monitor<'a>(
    monitors: &'a [Monitor],
    settings: &Settings,
    last: Option<&str>,
) -> Option<&'a Monitor> {
    let preferred = match settings.preferred_display.as_str() {
        "display2" => monitors.iter().find(|m| m.number == Some(2)),
        "last" => monitors.iter().find(|m| Some(m.id.as_str()) == last),
        "manual" => monitors
            .iter()
            .find(|m| Some(&m.id) == settings.manual_monitor.as_ref()),
        _ => monitors.iter().find(|m| m.primary),
    };
    preferred
        .or_else(|| monitors.iter().find(|m| m.primary))
        .or_else(|| monitors.first())
}

pub fn bounds(
    m: &Monitor,
    saved: Option<&Placement>,
    remember_position: bool,
    remember_size: bool,
) -> (i32, i32, u32, u32) {
    let size = saved
        .filter(|_| remember_size)
        .map(|p| (p.logical_width, p.logical_height))
        .unwrap_or((1120.0, 820.0));
    let w = (size.0 * m.scale).round().max(1.0).min(m.width as f64) as u32;
    let h = (size.1 * m.scale).round().max(1.0).min(m.height as f64) as u32;
    let (dx, dy) = saved
        .filter(|_| remember_position)
        .map(|p| {
            (
                (p.offset_x * m.scale).round() as i32,
                (p.offset_y * m.scale).round() as i32,
            )
        })
        .unwrap_or((((m.width - w) / 2) as i32, ((m.height - h) / 2) as i32));
    (
        m.x + dx.clamp(0, (m.width - w) as i32),
        m.y + dy.clamp(0, (m.height - h) as i32),
        w,
        h,
    )
}

fn current<'a>(window: &WebviewWindow, monitors: &'a [Monitor]) -> Option<&'a Monitor> {
    let p = window.outer_position().ok()?;
    let s = window.outer_size().ok()?;
    let x = p.x as i64 + s.width as i64 / 2;
    let y = p.y as i64 + s.height as i64 / 2;
    monitors.iter().find(|m| {
        x >= m.x as i64
            && y >= m.y as i64
            && x < m.x as i64 + m.width as i64
            && y < m.y as i64 + m.height as i64
    })
}

pub fn place(app: &AppHandle, reconnect: bool) -> Result<()> {
    let monitors = detect()?;
    let window = app.get_webview_window("main").ok_or("主窗口不存在")?;
    let db = app.state::<Database>();
    let settings = db.get_preference::<Settings>("settings")?;
    let positions = db.get_preference::<Positions>("positions")?;
    let active = app.state::<ActiveMonitor>();
    let previous = active.current.lock().map_err(|e| e.to_string())?.clone();
    let preferred = choose_monitor(&monitors, &settings, positions.last.as_deref());
    let known = active.connected.lock().map_err(|e| e.to_string())?.clone();
    let newly_connected = preferred.is_some_and(|m| !known.contains(&m.id));
    let target = if reconnect {
        topology_target(
            &monitors,
            &settings,
            previous.as_deref(),
            positions.last.as_deref(),
            newly_connected,
        )
    } else {
        choose_monitor(&monitors, &settings, positions.last.as_deref())
    }
    .ok_or("未检测到显示器")?;
    *active.connected.lock().map_err(|e| e.to_string())? =
        monitors.iter().map(|m| m.id.clone()).collect();
    let saved = positions.monitors.get(&target.id);
    // Keep a valid current placement during topology changes; return only when target changes.
    if reconnect
        && previous.as_deref() == Some(target.id.as_str())
        && current(&window, &monitors).is_some_and(|m| m.id == target.id)
    {
        let p = window.outer_position().map_err(|e| e.to_string())?;
        let s = window.outer_size().map_err(|e| e.to_string())?;
        if p.x >= target.x
            && p.y >= target.y
            && p.x as i64 + s.width as i64 <= target.x as i64 + target.width as i64
            && p.y as i64 + s.height as i64 <= target.y as i64 + target.height as i64
        {
            return Ok(());
        }
    }
    let inner = window.inner_size().map_err(|e| e.to_string())?;
    let outer = window.outer_size().map_err(|e| e.to_string())?;
    let scale = window.scale_factor().map_err(|e| e.to_string())?;
    let frame_width =
        (outer.width.saturating_sub(inner.width) as f64 / scale * target.scale).round() as u32;
    let frame_height =
        (outer.height.saturating_sub(inner.height) as f64 / scale * target.scale).round() as u32;
    let client_area = Monitor {
        width: target.width.saturating_sub(frame_width).max(1),
        height: target.height.saturating_sub(frame_height).max(1),
        ..target.clone()
    };
    let (x, y, w, h) = bounds(
        &client_area,
        saved,
        settings.remember_position,
        settings.remember_size,
    );
    window
        .set_min_size(Some(PhysicalSize::new(
            ((760.0 * target.scale) as u32).min(client_area.width),
            ((620.0 * target.scale) as u32).min(client_area.height),
        )))
        .map_err(|e| e.to_string())?;
    window
        .set_position(PhysicalPosition::new(x, y))
        .map_err(|e| e.to_string())?;
    window
        .set_size(PhysicalSize::new(w, h))
        .map_err(|e| e.to_string())?;
    *active.current.lock().map_err(|e| e.to_string())? = Some(target.id.clone());
    Ok(())
}

pub fn save_position(app: &AppHandle) -> Result<()> {
    let window = app.get_webview_window("main").ok_or("主窗口不存在")?;
    if window.is_minimized().unwrap_or(false) || window.is_maximized().unwrap_or(false) {
        return Ok(());
    }
    let monitors = detect()?;
    let Some(m) = current(&window, &monitors) else {
        return Ok(());
    };
    let p = window.outer_position().map_err(|e| e.to_string())?;
    // set_size takes client dimensions; saving outer dimensions makes every restart grow.
    let size = window.inner_size().map_err(|e| e.to_string())?;
    if size.width == 0 || size.height == 0 {
        return Ok(());
    }
    let db = app.state::<Database>();
    let mut positions = db.get_preference::<Positions>("positions")?;
    positions.last = Some(m.id.clone());
    positions.monitors.insert(
        m.id.clone(),
        Placement {
            offset_x: (p.x - m.x) as f64 / m.scale,
            offset_y: (p.y - m.y) as f64 / m.scale,
            logical_width: size.width as f64 / m.scale,
            logical_height: size.height as f64 / m.scale,
            physical_width: size.width,
            physical_height: size.height,
            scale_factor: m.scale,
        },
    );
    db.put_preference("positions", &positions)?;
    *app.state::<ActiveMonitor>()
        .current
        .lock()
        .map_err(|e| e.to_string())? = Some(m.id.clone());
    Ok(())
}

#[derive(Clone, Copy)]
pub enum Change {
    Position,
    Topology,
}
pub struct Watcher(pub Sender<Change>);

pub fn start(app: &AppHandle) -> Result<()> {
    let (tx, rx) = mpsc::channel();
    app.manage(Watcher(tx.clone()));
    #[cfg(windows)]
    native::install(app, tx)?;
    let app = app.clone();
    std::thread::spawn(move || {
        let mut pending = false;
        let mut topology = false;
        let mut last_backup = match app.state::<Database>().backup() {
            Ok(_) => chrono::Local::now().format("%Y-%m-%d").to_string(),
            Err(e) => {
                let _ = app.emit("app-error", format!("自动备份失败：{e}"));
                String::new()
            }
        };
        loop {
            // Events drive window updates; the timeout also handles backups and expiry while hidden.
            match rx.recv_timeout(if pending {
                Duration::from_millis(650)
            } else {
                Duration::from_secs(60)
            }) {
                Ok(change) => {
                    pending = true;
                    topology |= matches!(change, Change::Topology);
                    continue;
                }
                Err(mpsc::RecvTimeoutError::Disconnected) => break,
                Err(mpsc::RecvTimeoutError::Timeout) => {}
            }
            if pending {
                let result = if topology {
                    place(&app, true)
                } else {
                    save_position(&app)
                };
                if let Err(e) = result {
                    let _ = app.emit("app-error", e);
                }
                if topology {
                    let _ = app.emit("monitors-changed", ());
                }
                pending = false;
                topology = false;
            }
            let today = chrono::Local::now().format("%Y-%m-%d").to_string();
            match app.state::<Database>().prune_expired(&today) {
                Ok(count) if count > 0 => {
                    let _ = app.emit("notes-changed", ());
                }
                Ok(_) => {}
                Err(e) => {
                    let _ = app.emit("app-error", format!("自动清理失败：{e}"));
                }
            }
            if last_backup != today {
                match app.state::<Database>().backup() {
                    Ok(_) => last_backup = today,
                    Err(e) => {
                        let _ = app.emit("app-error", format!("自动备份失败：{e}"));
                    }
                }
            }
        }
    });
    Ok(())
}

#[cfg(windows)]
pub fn detect() -> Result<Vec<Monitor>> {
    native::detect()
}
#[cfg(not(windows))]
pub fn detect() -> Result<Vec<Monitor>> {
    Err("此版本需要 Windows 10 / 11".into())
}

#[cfg(windows)]
mod native {
    use super::*;
    use std::{mem::size_of, ptr};
    use windows_sys::Win32::{
        Devices::Display::*,
        Foundation::*,
        Graphics::Gdi::*,
        UI::{HiDpi::*, Shell::*, WindowsAndMessaging::*},
    };
    fn wide(s: &[u16]) -> String {
        String::from_utf16_lossy(&s[..s.iter().position(|c| *c == 0).unwrap_or(s.len())])
    }
    struct DpiContext(DPI_AWARENESS_CONTEXT);
    impl Drop for DpiContext {
        fn drop(&mut self) {
            if !self.0.is_null() {
                unsafe {
                    SetThreadDpiAwarenessContext(self.0);
                }
            }
        }
    }
    pub fn detect() -> Result<Vec<Monitor>> {
        let mut names = HashMap::<String, (String, String)>::new();
        unsafe {
            // Also required on the worker thread and in the standalone test executable.
            let _dpi_context = DpiContext(SetThreadDpiAwarenessContext(
                DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2,
            ));
            for _ in 0..4 {
                let (mut pc, mut mc) = (0, 0);
                let err = GetDisplayConfigBufferSizes(QDC_ONLY_ACTIVE_PATHS, &mut pc, &mut mc);
                if err != 0 {
                    break;
                }
                let mut paths = vec![DISPLAYCONFIG_PATH_INFO::default(); pc as usize];
                let mut modes = vec![DISPLAYCONFIG_MODE_INFO::default(); mc as usize];
                let err = QueryDisplayConfig(
                    QDC_ONLY_ACTIVE_PATHS,
                    &mut pc,
                    paths.as_mut_ptr(),
                    &mut mc,
                    modes.as_mut_ptr(),
                    ptr::null_mut(),
                );
                if err == ERROR_INSUFFICIENT_BUFFER {
                    continue;
                }
                if err != 0 {
                    break;
                }
                for p in paths.iter().take(pc as usize) {
                    let mut source = DISPLAYCONFIG_SOURCE_DEVICE_NAME::default();
                    source.header = DISPLAYCONFIG_DEVICE_INFO_HEADER {
                        r#type: DISPLAYCONFIG_DEVICE_INFO_GET_SOURCE_NAME,
                        size: size_of::<DISPLAYCONFIG_SOURCE_DEVICE_NAME>() as u32,
                        adapterId: p.sourceInfo.adapterId,
                        id: p.sourceInfo.id,
                    };
                    let mut target = DISPLAYCONFIG_TARGET_DEVICE_NAME::default();
                    target.header = DISPLAYCONFIG_DEVICE_INFO_HEADER {
                        r#type: DISPLAYCONFIG_DEVICE_INFO_GET_TARGET_NAME,
                        size: size_of::<DISPLAYCONFIG_TARGET_DEVICE_NAME>() as u32,
                        adapterId: p.targetInfo.adapterId,
                        id: p.targetInfo.id,
                    };
                    if DisplayConfigGetDeviceInfo(&mut source.header) == 0
                        && DisplayConfigGetDeviceInfo(&mut target.header) == 0
                    {
                        names.insert(
                            wide(&source.viewGdiDeviceName),
                            (
                                wide(&target.monitorDevicePath),
                                wide(&target.monitorFriendlyDeviceName),
                            ),
                        );
                    }
                }
                break;
            }
            let mut monitors = Vec::<Monitor>::new();
            if EnumDisplayMonitors(
                ptr::null_mut(),
                ptr::null(),
                Some(enumerate),
                &mut monitors as *mut _ as isize,
            ) == 0
            {
                return Err("Windows 显示器枚举失败".into());
            }
            for m in &mut monitors {
                if let Some((id, name)) = names.get(&m.id) {
                    if !id.is_empty() {
                        m.id = id.clone();
                    }
                    if !name.is_empty() {
                        m.name = name.clone();
                    }
                }
            }
            Ok(monitors)
        }
    }
    unsafe extern "system" fn enumerate(
        handle: HMONITOR,
        _: HDC,
        _: *mut RECT,
        data: LPARAM,
    ) -> i32 {
        let mut info = MONITORINFOEXW::default();
        info.monitorInfo.cbSize = size_of::<MONITORINFOEXW>() as u32;
        if GetMonitorInfoW(handle, &mut info.monitorInfo) == 0 {
            return 1;
        }
        let gdi = wide(&info.szDevice);
        let number = gdi
            .strip_prefix("\\\\.\\DISPLAY")
            .and_then(|n| n.parse().ok());
        let (mut dpi, mut other) = (96, 96);
        let _ = GetDpiForMonitor(handle, MDT_EFFECTIVE_DPI, &mut dpi, &mut other);
        let r = info.monitorInfo.rcWork;
        (&mut *(data as *mut Vec<Monitor>)).push(Monitor {
            id: gdi.clone(),
            name: gdi,
            number,
            primary: info.monitorInfo.dwFlags & 1 != 0,
            x: r.left,
            y: r.top,
            width: (r.right - r.left).max(1) as u32,
            height: (r.bottom - r.top).max(1) as u32,
            scale: dpi.max(96) as f64 / 96.0,
        });
        1
    }
    pub fn install(app: &AppHandle, tx: Sender<Change>) -> Result<()> {
        let window = app.get_webview_window("main").ok_or("主窗口不存在")?;
        let hwnd = window.hwnd().map_err(|e| e.to_string())?.0 as HWND;
        let data = Box::into_raw(Box::new(tx));
        unsafe {
            if SetWindowSubclass(hwnd, Some(subclass), 0xCDA1, data as usize) == 0 {
                drop(Box::from_raw(data));
                return Err("显示器变化监听初始化失败".into());
            }
        }
        Ok(())
    }
    unsafe extern "system" fn subclass(
        hwnd: HWND,
        msg: u32,
        w: WPARAM,
        l: LPARAM,
        id: usize,
        data: usize,
    ) -> LRESULT {
        if matches!(msg, WM_DISPLAYCHANGE | WM_SETTINGCHANGE | WM_POWERBROADCAST) {
            let _ = (*(data as *const Sender<Change>)).send(Change::Topology);
        }
        if msg == WM_DPICHANGED {
            let _ = (*(data as *const Sender<Change>)).send(Change::Position);
        }
        if msg == WM_NCDESTROY {
            RemoveWindowSubclass(hwnd, Some(subclass), id);
            drop(Box::from_raw(data as *mut Sender<Change>));
        }
        DefSubclassProc(hwnd, msg, w, l)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn monitor(id: &str, number: u32, x: i32, y: i32, scale: f64) -> Monitor {
        Monitor {
            id: id.into(),
            name: id.into(),
            number: Some(number),
            primary: number == 1,
            x,
            y,
            width: 1920,
            height: 1080,
            scale,
        }
    }
    #[test]
    fn display_identity_not_enumeration_order() {
        let monitors = vec![
            monitor("secondary", 2, -1920, -1080, 1.5),
            monitor("primary", 1, 0, 0, 1.0),
        ];
        let settings = Settings::default();
        assert_eq!(
            choose_monitor(&monitors, &settings, None).unwrap().id,
            "secondary"
        );
        assert_eq!(
            choose_monitor(&monitors[1..], &settings, None).unwrap().id,
            "primary"
        );
        assert!(choose_monitor(&[], &settings, None).is_none());
    }
    #[test]
    fn restored_bounds_fit_negative_coordinates_and_new_dpi() {
        let m = monitor("secondary", 2, -1920, -1080, 1.5);
        let p = Placement {
            offset_x: 5000.0,
            offset_y: -100.0,
            logical_width: 1000.0,
            logical_height: 700.0,
            physical_width: 1000,
            physical_height: 700,
            scale_factor: 1.0,
        };
        let (x, y, w, h) = bounds(&m, Some(&p), true, true);
        assert_eq!(w, 1500);
        assert_eq!(h, 1050);
        assert_eq!(x, -1500);
        assert_eq!(y, -1080);
        let m = Monitor {
            width: 640,
            height: 480,
            ..m
        };
        let (x, y, w, h) = bounds(&m, Some(&p), true, true);
        assert_eq!(w, 640);
        assert_eq!(h, 480);
        assert_eq!(x, m.x);
        assert_eq!(y, m.y);
    }
    #[test]
    fn reconnect_and_dpi_matrix() {
        let primary = monitor("primary", 1, 0, 0, 1.0);
        let second = monitor("secondary", 2, -2560, 0, 1.25);
        let mut settings = Settings::default();
        let connected = vec![primary.clone(), second.clone()];
        assert_eq!(
            topology_target(&connected, &settings, Some("primary"), None, true)
                .unwrap()
                .id,
            "secondary"
        );
        assert_eq!(
            topology_target(&connected, &settings, Some("primary"), None, false)
                .unwrap()
                .id,
            "primary"
        );
        settings.auto_return = false;
        assert_eq!(
            topology_target(&connected, &settings, Some("primary"), None, true)
                .unwrap()
                .id,
            "primary"
        );
        assert_eq!(
            topology_target(
                &[primary.clone()],
                &settings,
                Some("secondary"),
                None,
                false
            )
            .unwrap()
            .id,
            "primary"
        );
        for scale in [1.0, 1.25, 1.5, 1.75, 2.0] {
            for (x, y) in [(-2560, 0), (1920, 0), (0, -1440), (0, 1080)] {
                let m = Monitor {
                    x,
                    y,
                    scale,
                    width: 2560,
                    height: 1440,
                    ..second.clone()
                };
                let (px, py, w, h) = bounds(&m, None, true, true);
                assert!(
                    px >= x
                        && py >= y
                        && px as i64 + w as i64 <= x as i64 + 2560
                        && py as i64 + h as i64 <= y as i64 + 1440
                );
            }
        }
    }
    #[test]
    #[cfg(windows)]
    fn live_windows_monitor_enumeration() {
        // Match Tauri's per-monitor-aware process before checking physical geometry.
        unsafe {
            windows_sys::Win32::UI::HiDpi::SetProcessDpiAwarenessContext(
                windows_sys::Win32::UI::HiDpi::DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2,
            );
        }
        let monitors = detect().unwrap();
        assert!(!monitors.is_empty());
        assert!(monitors.iter().any(|m| m.primary));
        for m in &monitors {
            assert!(!m.id.is_empty());
            assert!(m.width > 0 && m.height > 0 && m.scale >= 1.0);
        }
        println!(
            "LIVE MONITORS: {}",
            serde_json::to_string(&monitors).unwrap()
        );
    }
}
