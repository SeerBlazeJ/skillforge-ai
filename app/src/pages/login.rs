use crate::utils::*;
use crate::SESSION_DURATION_DAYS;
use crate::{server_functions::login_user, Route};
use dioxus::prelude::*;

#[component]
pub fn Login() -> Element {
    let mut username = use_signal(String::new);
    let mut password = use_signal(String::new);
    let mut error = use_signal(|| None::<String>);
    let nav = navigator();
    let on_submit = move |evt: FormEvent| {
        evt.prevent_default();
        let u = username().trim().to_string();
        let p = password().to_string();
        spawn(async move {
            match login_user(u, p).await {
                Ok(token) => {
                    set_session_cookie(&token, SESSION_DURATION_DAYS);
                    if get_session_token().is_some() {
                        nav.push(Route::Dashboard {});
                    } else {
                        error.set(Some(
                            "Failed to save session. Please try again.".to_string(),
                        ));
                    }
                }
                Err(e) => {
                    error.set(Some(format!("Login failed: {}", e)));
                }
            }
        });
    };

    let mut is_loading = use_signal(|| false);

    rsx! {
        // 1. Initialize the loading state
        div { class: "min-h-screen bg-gradient-to-br from-indigo-50 to-purple-50 flex items-center justify-center px-6",
            div { class: "max-w-md w-full bg-white rounded-2xl shadow-2xl p-8",
                h2 { class: "text-3xl font-bold text-center mb-8 bg-gradient-to-r from-indigo-600 to-purple-600 bg-clip-text text-transparent",
                    "Welcome Back"
                }

                if let Some(err) = error() {
                    div { class: "mb-4 p-4 bg-red-50 text-red-700 rounded-lg", {err} }
                }

                form {
                    // 2. Wrap the submit handler to set loading state
                    onsubmit: move |e| {
                        if !is_loading() {
                            is_loading.set(true);
                            on_submit(e);
                        }
                    },
                    class: "space-y-6",
                    div {
                        label { class: "block text-sm font-medium text-gray-700 mb-2",
                            "Username"
                        }
                        input {
                            r#type: "text",
                            // 3. Disable input during loading
                            disabled: is_loading(),
                            class: "w-full px-4 py-3 border border-gray-300 rounded-lg focus:ring-2 focus:ring-indigo-500 focus:border-transparent outline-none transition disabled:bg-gray-100 disabled:text-gray-500",
                            value: "{username}",
                            oninput: move |e| username.set(e.value()),
                            placeholder: "Enter your username",
                        }
                    }

                    div {
                        label { class: "block text-sm font-medium text-gray-700 mb-2",
                            "Password"
                        }
                        input {
                            r#type: "password",
                            // 3. Disable input during loading
                            disabled: is_loading(),
                            class: "w-full px-4 py-3 border border-gray-300 rounded-lg focus:ring-2 focus:ring-indigo-500 focus:border-transparent outline-none transition disabled:bg-gray-100 disabled:text-gray-500",
                            value: "{password}",
                            oninput: move |e| password.set(e.value()),
                            placeholder: "Enter your password",
                        }
                    }

                    button {
                        r#type: "submit",
                        // 4. Disable button to prevent double-submit
                        disabled: is_loading(),
                        // 5. Flex utilities for centering and disabled styling
                        class: "w-full py-3 bg-indigo-600 text-white rounded-lg hover:bg-indigo-700 transition font-medium flex justify-center items-center disabled:opacity-70 disabled:cursor-not-allowed",

                        // 6. Conditional rendering for Spinner vs Text
                        if is_loading() {
                            svg {
                                class: "animate-spin -ml-1 mr-3 h-5 w-5 text-white",
                                xmlns: "http://www.w3.org/2000/svg",
                                fill: "none",
                                view_box: "0 0 24 24",
                                circle {
                                    class: "opacity-25",
                                    cx: "12",
                                    cy: "12",
                                    r: "10",
                                    stroke: "currentColor",
                                    stroke_width: "4",
                                }
                                path {
                                    class: "opacity-75",
                                    fill: "currentColor",
                                    d: "M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z",
                                }
                            }
                            "Logging in..."
                        } else {
                            "Login"
                        }
                    }
                }

                p { class: "text-center mt-6 text-gray-600",
                    "Don't have an account? "
                    Link {
                        to: Route::Signup {},
                        // Optional: Disable link pointer events if loading
                        class: if is_loading() { "text-indigo-400 cursor-not-allowed font-medium pointer-events-none" } else { "text-indigo-600 hover:text-indigo-700 font-medium" },
                        "Sign up"
                    }
                }
            }
        }
    }
}
