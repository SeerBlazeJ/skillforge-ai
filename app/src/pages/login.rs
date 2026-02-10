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
        div { class: "min-h-screen bg-[#050505] text-gray-100 font-sans selection:bg-teal-500/30 selection:text-teal-200 overflow-x-hidden relative flex items-center justify-center px-6",
            // Ambient Background Effects
            div { class: "fixed inset-0 pointer-events-none overflow-hidden",
                div { class: "absolute top-[-10%] left-[-10%] w-[50vw] h-[50vw] bg-teal-500/5 rounded-full blur-[100px] animate-float-slow" }
                div { class: "absolute bottom-[-10%] right-[-10%] w-[50vw] h-[50vw] bg-blue-600/5 rounded-full blur-[100px] animate-float-slow delay-2000" }
            }

            // Grid Pattern Overlay
            div { class: "fixed inset-0 bg-[linear-gradient(rgba(20,184,166,0.03)_1px,transparent_1px),linear-gradient(90deg,rgba(20,184,166,0.03)_1px,transparent_1px)] bg-[size:4rem_4rem] [mask-image:radial-gradient(ellipse_60%_50%_at_50%_0%,#000_70%,transparent_100%)] pointer-events-none" }

            // Login Card
            div { class: "w-full max-w-md relative z-10 animate-slide-up",
                div { class: "bg-[#0f1012]/60 backdrop-blur-xl border border-white/5 rounded-2xl shadow-[0_0_40px_-10px_rgba(0,0,0,0.5)] p-8 md:p-10 overflow-hidden relative",
                    // Subtle top glow on card
                    div { class: "absolute top-0 inset-x-0 h-px bg-gradient-to-r from-transparent via-teal-500/20 to-transparent" }

                    h2 { class: "text-3xl font-bold text-center mb-8 tracking-tight",
                        span { class: "bg-gradient-to-r from-teal-400 to-blue-500 bg-clip-text text-transparent animate-gradient-text",
                            "Welcome Back"
                        }
                    }

                    if let Some(err) = error() {
                        div { class: "mb-6 p-4 bg-red-500/10 border border-red-500/20 text-red-200 rounded-lg text-sm flex items-center",
                            span { class: "mr-2", "⚠️" }
                            "{err}"
                        }
                    }

                    form {
                        onsubmit: move |e| {
                            if !is_loading() {
                                is_loading.set(true);
                                on_submit(e);
                            }
                        },
                        class: "space-y-6",

                        // Username Input
                        div { class: "space-y-2",
                            label { class: "block text-sm font-medium text-gray-400 ml-1",
                                "Username"
                            }
                            div { class: "relative group",
                                input {
                                    r#type: "text",
                                    disabled: is_loading(),
                                    class: "w-full bg-[#0a0a0a]/50 text-gray-100 px-4 py-3 rounded-xl border border-gray-800 focus:border-teal-500/50 focus:ring-2 focus:ring-teal-500/20 outline-none transition-all duration-300 placeholder:text-gray-700 disabled:opacity-50 disabled:cursor-not-allowed hover:border-gray-700",
                                    value: "{username}",
                                    oninput: move |e| username.set(e.value()),
                                    placeholder: "Enter your username",
                                }
                            }
                        }

                        // Password Input
                        div { class: "space-y-2",
                            label { class: "block text-sm font-medium text-gray-400 ml-1",
                                "Password"
                            }
                            div { class: "relative group",
                                input {
                                    r#type: "password",
                                    disabled: is_loading(),
                                    class: "w-full bg-[#0a0a0a]/50 text-gray-100 px-4 py-3 rounded-xl border border-gray-800 focus:border-blue-500/50 focus:ring-2 focus:ring-blue-500/20 outline-none transition-all duration-300 placeholder:text-gray-700 disabled:opacity-50 disabled:cursor-not-allowed hover:border-gray-700",
                                    value: "{password}",
                                    oninput: move |e| password.set(e.value()),
                                    placeholder: "••••••••",
                                }
                            }
                        }

                        // Submit Button
                        button {
                            r#type: "submit",
                            disabled: is_loading(),
                            class: "w-full relative group py-3 rounded-xl bg-gradient-to-r from-teal-500 to-blue-600 text-white font-medium shadow-lg shadow-teal-900/20 hover:shadow-teal-500/20 hover:shadow-[0_0_20px_rgba(20,184,166,0.3)] transition-all duration-300 transform active:scale-[0.98] disabled:opacity-70 disabled:cursor-not-allowed overflow-hidden flex justify-center items-center",
                            div { class: "absolute inset-0 bg-white/20 opacity-0 group-hover:opacity-100 transition-opacity duration-300" }

                            if is_loading() {
                                svg {
                                    class: "animate-spin -ml-1 mr-3 h-5 w-5 text-white/90",
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
                                span { "Logging in..." }
                            } else {
                                span { "Login" }
                            }
                        }
                    }

                    // Footer Link
                    div { class: "text-center mt-8 text-sm text-gray-500",
                        "Don't have an account? "
                        Link {
                            to: Route::Signup {},
                            class: if is_loading() { "text-teal-500/50 cursor-not-allowed pointer-events-none" } else { "text-teal-400 hover:text-teal-300 font-medium transition-colors hover:underline decoration-teal-500/30 underline-offset-4" },
                            "Sign up now"
                        }
                    }
                }
            }
        }
    }
}
