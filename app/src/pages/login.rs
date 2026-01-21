use crate::{server_functions::login_user, Route};
use dioxus::prelude::*;

#[component]
pub fn Login() -> Element {
    let mut username = use_signal(String::new);
    let mut password = use_signal(String::new);
    let mut error = use_signal(|| None::<String>);
    let nav = navigator();

    let on_submit = move |_| {
        spawn(async move {
            match login_user(username(), password()).await {
                Ok(_) => {
                    nav.push(Route::Dashboard {});
                }
                Err(e) => {
                    error.set(Some(format!("Login failed: {}", e)));
                }
            }
        });
    };

    rsx! {
        div { class: "min-h-screen bg-gradient-to-br from-indigo-50 to-purple-50 flex items-center justify-center px-6",
            div { class: "max-w-md w-full bg-white rounded-2xl shadow-2xl p-8",
                h2 { class: "text-3xl font-bold text-center mb-8 bg-gradient-to-r from-indigo-600 to-purple-600 bg-clip-text text-transparent",
                    "Welcome Back"
                }

                if let Some(err) = error() {
                    div { class: "mb-4 p-4 bg-red-50 text-red-700 rounded-lg", {err} }
                }

                form { onsubmit: on_submit, class: "space-y-6",
                    div {
                        label { class: "block text-sm font-medium text-gray-700 mb-2",
                            "Username"
                        }
                        input {
                            r#type: "text",
                            class: "w-full px-4 py-3 border border-gray-300 rounded-lg focus:ring-2 focus:ring-indigo-500 focus:border-transparent outline-none transition",
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
                            class: "w-full px-4 py-3 border border-gray-300 rounded-lg focus:ring-2 focus:ring-indigo-500 focus:border-transparent outline-none transition",
                            value: "{password}",
                            oninput: move |e| password.set(e.value()),
                            placeholder: "Enter your password",
                        }
                    }

                    button {
                        r#type: "submit",
                        class: "w-full py-3 bg-indigo-600 text-white rounded-lg hover:bg-indigo-700 transition font-medium",
                        "Login"
                    }
                }

                p { class: "text-center mt-6 text-gray-600",
                    "Don't have an account? "
                    Link {
                        to: Route::Signup {},
                        class: "text-indigo-600 hover:text-indigo-700 font-medium",
                        "Sign up"
                    }
                }
            }
        }
    }
}
