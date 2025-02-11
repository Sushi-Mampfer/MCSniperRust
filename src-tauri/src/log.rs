use tauri::window::Color;
use tauri::Emitter;

use crate::app_handle;

pub fn alert(msg: &str) {
    app_handle().emit("alert", msg).unwrap();
}

pub fn log(typ: &str, color: Color, payload: &str) {
    let color = format!("rgb({}, {}, {})", color.0, color.1, color.2);
    let _ = app_handle().emit(
        "log",
        format!(
            "<p>[<span style=\"color: {}\">{}</span>] {}</p>",
            color, typ, payload
        ),
    );
}
