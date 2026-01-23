// Client-side function to read cookie
#[cfg(target_arch = "wasm32")]
pub fn get_session_token() -> Option<String> {
    use wasm_bindgen::JsCast;
    use web_sys::window;

    let result = window()
        .and_then(|w| w.document())
        .and_then(|doc| doc.dyn_into::<web_sys::HtmlDocument>().ok())
        .and_then(|html_doc| html_doc.cookie().ok())
        .and_then(|cookies: String| {
            // Debug: log all cookies
            web_sys::console::log_1(&format!("All cookies: {}", cookies).into());

            cookies.split(';').find_map(|cookie: &str| {
                let parts: Vec<&str> = cookie.trim().splitn(2, '=').collect();
                web_sys::console::log_1(&format!("Checking cookie part: {:?}", parts).into());

                if parts.len() == 2 && parts[0] == "skillforge_session" {
                    Some(parts[1].to_string())
                } else {
                    None
                }
            })
        });

    web_sys::console::log_1(&format!("Session token result: {:?}", result).into());
    result
}

#[cfg(not(target_arch = "wasm32"))]
pub fn get_session_token() -> Option<String> {
    None
}

// Client-side function to set cookie
#[cfg(target_arch = "wasm32")]
pub fn set_session_cookie(token: &str, days: i64) {
    use wasm_bindgen::JsCast;
    use web_sys::window;

    if let Some(window) = window() {
        if let Some(document) = window.document() {
            if let Ok(html_doc) = document.dyn_into::<web_sys::HtmlDocument>() {
                let expires = js_sys::Date::new(&wasm_bindgen::JsValue::from_f64(
                    js_sys::Date::now() + (days as f64 * 24.0 * 60.0 * 60.0 * 1000.0),
                ));

                let cookie = format!(
                    "skillforge_session={}; expires={}; path=/; SameSite=Lax", // Changed to Lax
                    token,
                    expires.to_utc_string()
                );

                web_sys::console::log_1(&format!("Setting cookie: {}", cookie).into());

                match html_doc.set_cookie(&cookie) {
                    Ok(_) => {
                        web_sys::console::log_1(&"Cookie set successfully".into());

                        // Verify it was set
                        if let Ok(cookies) = html_doc.cookie() {
                            web_sys::console::log_1(
                                &format!("Cookies after set: {}", cookies).into(),
                            );
                        }
                    }
                    Err(e) => {
                        web_sys::console::log_1(&format!("Failed to set cookie: {:?}", e).into());
                    }
                }
            }
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub fn set_session_cookie(_token: &str, _days: i64) {}
