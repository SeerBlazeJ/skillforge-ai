// Client-side function to read cookie
#[cfg(target_arch = "wasm32")]
pub fn get_session_token() -> Option<String> {
    use web_sys::window;

    window()
        .and_then(|w| w.document())
        .and_then(|doc| doc.cookie().ok())
        .and_then(|cookies| {
            cookies.split(';').find_map(|cookie| {
                let parts: Vec<&str> = cookie.trim().splitn(2, '=').collect();
                if parts.len() == 2 && parts[0] == "skillforge_session" {
                    Some(parts[1].to_string())
                } else {
                    None
                }
            })
        })
}

#[cfg(not(target_arch = "wasm32"))]
pub fn get_session_token() -> Option<String> {
    None
}

// Client-side function to set cookie
#[cfg(target_arch = "wasm32")]
pub fn set_session_cookie(token: &str, days: i64) {
    use wasm_bindgen::JsValue;
    use web_sys::window;

    if let Some(window) = window() {
        if let Some(document) = window.document() {
            let expires = js_sys::Date::new(&JsValue::from_f64(
                js_sys::Date::now() + (days as f64 * 24.0 * 60.0 * 60.0 * 1000.0),
            ));

            let cookie = format!(
                "skillforge_session={}; expires={}; path=/; SameSite=Strict",
                token,
                expires.to_utc_string()
            );

            let _ = document.set_cookie(&cookie);
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub fn set_session_cookie(_token: &str, _days: i64) {}
