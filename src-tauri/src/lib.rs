mod account;
mod log;
mod sniper;

use std::sync::{Mutex, OnceLock};
use tauri::{AppHandle, Manager};

static APP_HANDLE: OnceLock<AppHandle> = OnceLock::new();
static THREAD_STATUS: OnceLock<Mutex<bool>> = OnceLock::new();

fn app_handle<'a>() -> &'a AppHandle {
    APP_HANDLE.get().unwrap()
}

fn get_thread_status() -> bool {
    let mutex = THREAD_STATUS.get_or_init(|| Mutex::new(false));
    let data = mutex.lock().unwrap();
    *data
}

fn set_thread_status(param: bool) {
    let mutex = THREAD_STATUS.get_or_init(|| Mutex::new(false));
    let mut data = mutex.lock().unwrap();
    *data = param;
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            let main_window = app.get_webview_window("main").unwrap();
            main_window.set_title("MCSniperRust - Idle").unwrap();
            APP_HANDLE.set(app.handle().to_owned()).unwrap();
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![sniper::start, sniper::stop])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
