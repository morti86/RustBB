use wasm_bindgen::JsValue;
use web_sys::window;

pub fn get_item(key: &str) -> Option<String> {
    window()
        .and_then(|win| win.local_storage().ok())
        .flatten()
        .and_then(|storage| storage.get_item(key).ok())
        .flatten()
}

pub fn set_item(key: &str, value: &str) -> Result<(), JsValue> {
    if let Some(window) = window() {
        if let Ok(Some(storage)) = window.local_storage() {
            return storage.set_item(key, value);
        }
    }
    Err(JsValue::from_str("Failed to access local storage"))
}

pub fn remove_item(key: &str) -> Result<(), JsValue> {
    if let Some(window) = window() {
        if let Ok(Some(storage)) = window.local_storage() {
            return storage.remove_item(key);
        }
    }
    Err(JsValue::from_str("Failed to access local storage"))
}

pub fn clear() -> Result<(), JsValue> {
    if let Some(window) = window() {
        if let Ok(Some(storage)) = window.local_storage() {
            return storage.clear();
        }
    }
    Err(JsValue::from_str("Failed to access local storage"))
}
