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
        // Added mut
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
        div { class: "min-h-screen bg-gradient-to-br from-indigo-50 to-purple-50 flex items-center justify-center px-6 py-12",
            div { class: "max-w-md w-full bg-white rounded-2xl shadow-2xl p-8",
                // Header
                div { class: "text-center mb-8",
                    h2 { class: "text-3xl font-bold bg-gradient-to-r from-indigo-600 to-purple-600 bg-clip-text text-transparent",
                        "Create Your Account"
                    }
                    p { class: "text-gray-600 mt-2", "Start your personalized learning journey" }
                }

                // Error messages
                if let Some(err) = error() {
                    div { class: "mb-4 p-4 bg-red-50 border border-red-200 text-red-700 rounded-lg flex items-start",
                        svg {
                            class: "w-5 h-5 mr-2 mt-0.5 flex-shrink-0",
                            fill: "currentColor",
                            view_box: "0 0 20 20",
                            path { d: "M10 18a8 8 0 100-16 8 8 0 000 16zM8.707 7.293a1 1 0 00-1.414 1.414L8.586 10l-1.293 1.293a1 1 0 101.414 1.414L10 11.414l1.293 1.293a1 1 0 001.414-1.414L11.414 10l1.293-1.293a1 1 0 00-1.414-1.414L10 8.586 8.707 7.293z" }
                        }
                        span { {err} }
                    }
                }

                // Validation errors
                if !validation_errors().is_empty() {
                    div { class: "mb-4 p-4 bg-yellow-50 border border-yellow-200 rounded-lg",
                        p { class: "text-sm font-medium text-yellow-800 mb-2",
                            "Please fix the following errors:"
                        }
                        ul { class: "text-sm text-yellow-700 space-y-1 list-disc list-inside",
                            for err in validation_errors() {
                                li { key: "{err}", {err} }
                            }
                        }
                    }
                }

                // Form
                form { onsubmit: on_submit, class: "space-y-5",
                    // Name field
                    div {
                        label {
                            r#for: "name",
                            class: "block text-sm font-medium text-gray-700 mb-2",
                            "Full Name"
                            span { class: "text-red-500", " *" }
                        }
                        input {
                            id: "name",
                            r#type: "text",
                            class: "w-full px-4 py-3 border border-gray-300 rounded-lg focus:ring-2 focus:ring-indigo-500 focus:border-transparent outline-none transition",
                            value: "{name}",
                            oninput: move |e| {
                                name.set(e.value());
                                validation_errors.set(Vec::new());
                            },
                            placeholder: "Enter your full name",
                            disabled: is_loading(),
                        }
                    }

                    // Username field
                    div {
                        label {
                            r#for: "username",
                            class: "block text-sm font-medium text-gray-700 mb-2",
                            "Username"
                            span { class: "text-red-500", " *" }
                        }
                        input {
                            id: "username",
                            r#type: "text",
                            class: "w-full px-4 py-3 border border-gray-300 rounded-lg focus:ring-2 focus:ring-indigo-500 focus:border-transparent outline-none transition",
                            value: "{username}",
                            oninput: move |e| {
                                username.set(e.value());
                                validation_errors.set(Vec::new());
                            },
                            placeholder: "Choose a username",
                            disabled: is_loading(),
                            autocomplete: "username",
                        }
                        p { class: "mt-1 text-xs text-gray-500",
                            "3+ characters, letters, numbers, and underscores only"
                        }
                    }

                    // Password field
                    div {
                        label {
                            r#for: "password",
                            class: "block text-sm font-medium text-gray-700 mb-2",
                            "Password"
                            span { class: "text-red-500", " *" }
                        }
                        input {
                            id: "password",
                            r#type: "password",
                            class: "w-full px-4 py-3 border border-gray-300 rounded-lg focus:ring-2 focus:ring-indigo-500 focus:border-transparent outline-none transition",
                            value: "{password}",
                            oninput: move |e| {
                                password.set(e.value());
                                validation_errors.set(Vec::new());
                            },
                            placeholder: "Create a strong password",
                            disabled: is_loading(),
                            autocomplete: "new-password",
                        }
                        p { class: "mt-1 text-xs text-gray-500",
                            "8+ characters with uppercase, lowercase, and numbers"
                        }
                    }

                    // Confirm password field
                    div {
                        label {
                            r#for: "confirm-password",
                            class: "block text-sm font-medium text-gray-700 mb-2",
                            "Confirm Password"
                            span { class: "text-red-500", " *" }
                        }
                        input {
                            id: "confirm-password",
                            r#type: "password",
                            class: "w-full px-4 py-3 border border-gray-300 rounded-lg focus:ring-2 focus:ring-indigo-500 focus:border-transparent outline-none transition",
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

                    // Submit button
                    button {
                        r#type: "submit",
                        class: "w-full py-3 bg-indigo-600 text-white rounded-lg hover:bg-indigo-700 transition font-medium disabled:bg-gray-400 disabled:cursor-not-allowed flex items-center justify-center",
                        disabled: is_loading(),

                        if is_loading() {
                            svg {
                                class: "animate-spin h-5 w-5 mr-2",
                                view_box: "0 0 24 24",
                                circle {
                                    class: "opacity-25",
                                    cx: "12",
                                    cy: "12",
                                    r: "10",
                                    stroke: "currentColor",
                                    stroke_width: "4",
                                    fill: "none",
                                }
                                path {
                                    class: "opacity-75",
                                    fill: "currentColor",
                                    d: "M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z",
                                }
                            }
                            "Creating account..."
                        } else {
                            "Create Account"
                        }
                    }
                }

                // Footer
                p { class: "text-center mt-6 text-gray-600",
                    "Already have an account? "
                    Link {
                        to: Route::Login {},
                        class: "text-indigo-600 hover:text-indigo-700 font-medium hover:underline",
                        "Login"
                    }
                }

                // Terms and privacy
                p { class: "text-center mt-4 text-xs text-gray-500",
                    "By signing up, you agree to our "
                    a { href: "#", class: "text-indigo-600 hover:underline", "Terms of Service" }
                    " and "
                    a { href: "#", class: "text-indigo-600 hover:underline", "Privacy Policy" }
                }
            }
        }
    }
}
