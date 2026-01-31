#[cfg(target_arch = "wasm32")]
pub fn get_session_token() -> Option<String> {
    use wasm_bindgen::JsCast;
    use web_sys::HtmlDocument;

    let window = web_sys::window()?;
    let document = window.document()?;
    let html_document = document.dyn_into::<HtmlDocument>().ok()?;
    let cookie = html_document.cookie().ok()?;

    cookie.split(';').find_map(|s| {
        let parts: Vec<&str> = s.trim().split('=').collect();
        if parts.len() == 2 && parts[0] == "skillforge_session" {
            Some(parts[1].to_string())
        } else {
            None
        }
    })
}

#[cfg(target_arch = "wasm32")]
pub fn clear_session_token() -> Option<()> {
    use wasm_bindgen::JsCast;
    use web_sys::HtmlDocument;

    let window = web_sys::window()?;
    let document = window.document()?;
    let html_document = document.dyn_into::<HtmlDocument>().ok()?;

    html_document
        .set_cookie("skillforge_session=; expires=Thu, 01 Jan 1970 00:00:00 GMT; path=/;")
        .ok()
}

#[cfg(target_arch = "wasm32")]
pub fn set_session_cookie(token: &str, days: i64) {
    use wasm_bindgen::JsCast;
    use web_sys::HtmlDocument;

    if let Some(window) = web_sys::window() {
        if let Some(document) = window.document() {
            if let Ok(html_document) = document.dyn_into::<HtmlDocument>() {
                // Set cookie with expiration
                let date = js_sys::Date::new_0();
                let time = date.get_time() + (days as f64 * 24.0 * 60.0 * 60.0 * 1000.0);
                date.set_time(time);

                let cookie_string = format!(
                    "skillforge_session={}; expires={}; path=/",
                    token,
                    date.to_utc_string()
                );
                let _ = html_document.set_cookie(&cookie_string);
            }
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub fn get_session_token() -> Option<String> {
    None
}

#[cfg(not(target_arch = "wasm32"))]
pub fn set_session_cookie(_token: &str, _days: i64) {}

#[cfg(not(target_arch = "wasm32"))]
pub fn clear_session_token() -> Option<()> {
    None
}
