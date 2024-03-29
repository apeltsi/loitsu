use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    fn set_status(status: u32);
    #[cfg(feature = "editor")]
    fn add_loading_task(task: String);
    #[cfg(feature = "editor")]
    fn remove_loading_task(task: String);
    #[cfg(feature = "editor")]
    fn add_notification(r#type: u32, title: String, message: String);
}

#[cfg(feature = "editor")]
pub fn add_editor_loading_task(task: &str) {
    add_loading_task(task.to_string());
}

#[cfg(feature = "editor")]
pub fn remove_editor_loading_task(task: &str) {
    remove_loading_task(task.to_string());
}
#[cfg(feature = "editor")]
pub fn notify(r#type: u32, title: &str, message: &str) {
    add_notification(r#type, title.to_string(), message.to_string());
}

pub fn update_loading_status(status: u32) {
    set_status(status);
}
