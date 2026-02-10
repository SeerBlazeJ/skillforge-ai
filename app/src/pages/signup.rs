use crate::{server_functions::signup_user, Route};
use dioxus::prelude::*;

#[component]
pub fn Signup() -> Element {
    let mut username = use_signal(String::new);
    let mut password = use_signal(String::new);
    let mut confirm_password = use_signal(String::new);
    let mut name = use_signal(String::new);
    let mut error = use_signal(|| None::<String>);
    let mut validation_errors = use_signal(Vec::<String>::new);
    let mut is_loading = use_signal(|| false);
    let nav = navigator();

    let mut validate_form = move || -> bool {
        let mut errors = Vec::new();

        // Username validation
        if username().trim().is_empty() {
            errors.push("Username is required".to_string());
        } else if username().len() < 3 {
            errors.push("Username must be at least 3 characters".to_string());
        } else if !username().chars().all(|c| c.is_alphanumeric() || c == '_') {
            errors.push("Username can only contain letters, numbers, and underscores".to_string());
        }

        // Name validation
        if name().trim().is_empty() {
            errors.push("Name is required".to_string());
        }

        // Password validation
        if password().is_empty() {
            errors.push("Password is required".to_string());
        } else if password().len() < 8 {
            errors.push("Password must be at least 8 characters".to_string());
        } else if !password().chars().any(|c| c.is_uppercase()) {
            errors.push("Password must contain at least one uppercase letter".to_string());
        } else if !password().chars().any(|c| c.is_lowercase()) {
            errors.push("Password must contain at least one lowercase letter".to_string());
        } else if !password().chars().any(|c| c.is_numeric()) {
            errors.push("Password must contain at least one number".to_string());
        }

        // Confirm password validation
        if password() != confirm_password() {
            errors.push("Passwords do not match".to_string());
        }

        validation_errors.set(errors.clone());
        errors.is_empty()
    };

    let on_submit = move |evt: Event<FormData>| {
        evt.prevent_default();
        if !validate_form() {
            return;
        }

        is_loading.set(true);
        error.set(None);

        spawn(async move {
            match signup_user(username(), password(), name()).await {
                Ok(_) => {
                    nav.push(Route::Login {});
                }
                Err(e) => {
                    is_loading.set(false);
                    let error_msg = if e.to_string().contains("unique")
                        || e.to_string().contains("duplicate")
                    {
                        "Username already exists. Please choose a different username.".to_string()
                    } else {
                        format!("Signup failed: {}", e)
                    };
                    error.set(Some(error_msg));
                }
            }
        });
    };

    rsx! {
        div { class: "min-h-screen bg-[#050505] text-gray-100 font-sans selection:bg-teal-500/30 selection:text-teal-200 overflow-x-hidden relative flex items-center justify-center px-6 py-12",
            // Ambient Background Effects
            div { class: "fixed inset-0 pointer-events-none overflow-hidden",
                div { class: "absolute top-[-10%] left-[-10%] w-[50vw] h-[50vw] bg-teal-500/5 rounded-full blur-[100px] animate-float-slow" }
                div { class: "absolute bottom-[-10%] right-[-10%] w-[50vw] h-[50vw] bg-blue-600/5 rounded-full blur-[100px] animate-float-slow delay-2000" }
                div { class: "absolute top-[40%] left-[40%] w-[30vw] h-[30vw] bg-emerald-500/5 rounded-full blur-[80px] animate-pulse-slow" }
            }

            // Grid Pattern Overlay
            div { class: "fixed inset-0 bg-[linear-gradient(rgba(20,184,166,0.03)_1px,transparent_1px),linear-gradient(90deg,rgba(20,184,166,0.03)_1px,transparent_1px)] bg-[size:4rem_4rem] [mask-image:radial-gradient(ellipse_60%_50%_at_50%_0%,#000_70%,transparent_100%)] pointer-events-none" }

            // Signup Card
            div { class: "w-full max-w-lg relative z-10 animate-slide-up",
                div { class: "bg-[#0f1012]/60 backdrop-blur-xl border border-white/5 rounded-2xl shadow-[0_0_40px_-10px_rgba(0,0,0,0.5)] p-8 md:p-10 overflow-hidden relative",
                    // Subtle top glow
                    div { class: "absolute top-0 inset-x-0 h-px bg-gradient-to-r from-transparent via-teal-500/20 to-transparent" }

                    // Header
                    div { class: "text-center mb-8",
                        h2 { class: "text-3xl font-bold tracking-tight mb-2",
                            span { class: "bg-gradient-to-r from-teal-400 to-blue-500 bg-clip-text text-transparent animate-gradient-text",
                                "Create Account"
                            }
                        }
                        p { class: "text-gray-400 text-sm",
                            "Start your personalized learning journey today"
                        }
                    }

                    // Main Error Display
                    if let Some(err) = error() {
                        div { class: "mb-6 p-4 bg-red-500/10 border border-red-500/20 text-red-200 rounded-lg flex items-start animate-fade-in",
                            span { class: "mr-3 text-lg", "⚠️" }
                            span { class: "text-sm", "{err}" }
                        }
                    }

                    // Validation Errors List
                    if !validation_errors().is_empty() {
                        div { class: "mb-6 p-4 bg-yellow-500/10 border border-yellow-500/20 rounded-lg animate-fade-in",
                            p { class: "text-sm font-medium text-yellow-200 mb-2",
                                "Please check the following:"
                            }
                            ul { class: "text-sm text-yellow-200/80 space-y-1 list-disc list-inside",
                                for err in validation_errors() {
                                    li { key: "{err}", "{err}" }
                                }
                            }
                        }
                    }

                    form { onsubmit: on_submit, class: "space-y-5",
                        // Name Field
                        div { class: "space-y-1.5",
                            label {
                                r#for: "name",
                                class: "block text-sm font-medium text-gray-400 ml-1",
                                "Full Name"
                            }
                            div { class: "relative group",
                                input {
                                    id: "name",
                                    r#type: "text",
                                    class: "w-full bg-[#0a0a0a]/50 text-gray-100 px-4 py-3 rounded-xl border border-gray-800 focus:border-teal-500/50 focus:ring-2 focus:ring-teal-500/20 outline-none transition-all duration-300 placeholder:text-gray-700 disabled:opacity-50 disabled:cursor-not-allowed hover:border-gray-700",
                                    value: "{name}",
                                    oninput: move |e| {
                                        name.set(e.value());
                                        validation_errors.set(Vec::new());
                                    },
                                    placeholder: "Enter your full name",
                                    disabled: is_loading(),
                                }
                            }
                        }

                        // Username Field
                        div { class: "space-y-1.5",
                            label {
                                r#for: "username",
                                class: "block text-sm font-medium text-gray-400 ml-1",
                                "Username"
                            }
                            div { class: "relative group",
                                input {
                                    id: "username",
                                    r#type: "text",
                                    class: "w-full bg-[#0a0a0a]/50 text-gray-100 px-4 py-3 rounded-xl border border-gray-800 focus:border-teal-500/50 focus:ring-2 focus:ring-teal-500/20 outline-none transition-all duration-300 placeholder:text-gray-700 disabled:opacity-50 disabled:cursor-not-allowed hover:border-gray-700",
                                    value: "{username}",
                                    oninput: move |e| {
                                        username.set(e.value());
                                        validation_errors.set(Vec::new());
                                    },
                                    placeholder: "Choose a username",
                                    disabled: is_loading(),
                                    autocomplete: "username",
                                }
                            }
                            p { class: "text-xs text-gray-600 ml-1",
                                "3+ chars, letters, numbers & underscores"
                            }
                        }

                        // Password Field
                        div { class: "space-y-1.5",
                            label {
                                r#for: "password",
                                class: "block text-sm font-medium text-gray-400 ml-1",
                                "Password"
                            }
                            div { class: "relative group",
                                input {
                                    id: "password",
                                    r#type: "password",
                                    class: "w-full bg-[#0a0a0a]/50 text-gray-100 px-4 py-3 rounded-xl border border-gray-800 focus:border-blue-500/50 focus:ring-2 focus:ring-blue-500/20 outline-none transition-all duration-300 placeholder:text-gray-700 disabled:opacity-50 disabled:cursor-not-allowed hover:border-gray-700",
                                    value: "{password}",
                                    oninput: move |e| {
                                        password.set(e.value());
                                        validation_errors.set(Vec::new());
                                    },
                                    placeholder: "Create a strong password",
                                    disabled: is_loading(),
                                    autocomplete: "new-password",
                                }
                            }
                            p { class: "text-xs text-gray-600 ml-1",
                                "8+ chars, uppercase, lowercase & number"
                            }
                        }

                        // Confirm Password Field
                        div { class: "space-y-1.5",
                            label {
                                r#for: "confirm-password",
                                class: "block text-sm font-medium text-gray-400 ml-1",
                                "Confirm Password"
                            }
                            div { class: "relative group",
                                input {
                                    id: "confirm-password",
                                    r#type: "password",
                                    class: "w-full bg-[#0a0a0a]/50 text-gray-100 px-4 py-3 rounded-xl border border-gray-800 focus:border-blue-500/50 focus:ring-2 focus:ring-blue-500/20 outline-none transition-all duration-300 placeholder:text-gray-700 disabled:opacity-50 disabled:cursor-not-allowed hover:border-gray-700",
                                    value: "{confirm_password}",
                                    oninput: move |e| {
                                        confirm_password.set(e.value());
                                        validation_errors.set(Vec::new());
                                    },
                                    placeholder: "Confirm your password",
                                    disabled: is_loading(),
                                    autocomplete: "new-password",
                                }
                            }
                        }

                        // Submit Button
                        button {
                            r#type: "submit",
                            class: "w-full relative group py-3.5 mt-4 rounded-xl bg-gradient-to-r from-teal-500 to-blue-600 text-white font-medium shadow-lg shadow-teal-900/20 hover:shadow-teal-500/20 hover:shadow-[0_0_20px_rgba(20,184,166,0.3)] transition-all duration-300 transform active:scale-[0.98] disabled:opacity-70 disabled:cursor-not-allowed overflow-hidden flex justify-center items-center",
                            disabled: is_loading(),

                            // Shine effect
                            div { class: "absolute inset-0 bg-white/20 opacity-0 group-hover:opacity-100 transition-opacity duration-300" }

                            if is_loading() {
                                svg {
                                    class: "animate-spin -ml-1 mr-3 h-5 w-5 text-white/90",
                                    view_box: "0 0 24 24",
                                    fill: "none",
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
                                span { "Creating Account..." }
                            } else {
                                span { "Create Account" }
                            }
                        }
                    }

                    // Footer Link
                    div { class: "text-center mt-8 text-sm text-gray-500",
                        "Already have an account? "
                        Link {
                            to: Route::Login {},
                            class: if is_loading() { "text-teal-500/50 cursor-not-allowed pointer-events-none" } else { "text-teal-400 hover:text-teal-300 font-medium transition-colors hover:underline decoration-teal-500/30 underline-offset-4" },
                            "Login"
                        }
                    }

                    // Terms
                    div { class: "text-center mt-6 pt-6 border-t border-gray-800",
                        p { class: "text-xs text-gray-600",
                            "By signing up, you agree to our "
                            a {
                                href: "#",
                                class: "text-gray-500 hover:text-teal-400 transition-colors underline decoration-gray-700",
                                "Terms of Service"
                            }
                            " and "
                            a {
                                href: "#",
                                class: "text-gray-500 hover:text-teal-400 transition-colors underline decoration-gray-700",
                                "Privacy Policy"
                            }
                        }
                    }
                }
            }
        }
    }
}
