use crate::{utils::get_session_token, Route};
use dioxus::prelude::*;

#[component]
pub fn Landing() -> Element {
    match get_session_token() {
        Some(token) => token,
        None => {
            return rsx! {
                div { class: "min-h-screen bg-gradient-to-br from-indigo-50 via-white to-purple-50",
                    nav { class: "container mx-auto px-6 py-6 flex justify-between items-center",
                        h1 { class: "text-3xl font-bold bg-gradient-to-r from-indigo-600 to-purple-600 bg-clip-text text-transparent",
                            "SkillForge"
                        }
                        div { class: "space-x-4",
                            Link {
                                to: Route::Login {},
                                class: "px-6 py-2 text-indigo-600 hover:text-indigo-700 font-medium",
                                "Login"
                            }
                            Link {
                                to: Route::Signup {},
                                class: "px-6 py-2 bg-indigo-600 text-white rounded-lg hover:bg-indigo-700 transition",
                                "Sign Up"
                            }
                        }
                    }

                    main { class: "container mx-auto px-6 py-20 text-center",
                        h2 { class: "text-5xl font-bold text-gray-900 mb-6",
                            "Your Personalized Learning Journey"
                        }
                        p { class: "text-xl text-gray-600 mb-12 max-w-2xl mx-auto",
                            "AI-powered roadmaps tailored to your skills, preferences, and goals. Learn smarter, not harder."
                        }

                        div { class: "grid md:grid-cols-3 gap-8 mt-16 max-w-5xl mx-auto",
                            div { class: "p-8 bg-white rounded-2xl shadow-lg hover:shadow-xl transition",
                                div { class: "text-4xl mb-4", "🎯" }
                                h3 { class: "text-xl font-bold mb-3", "Personalized Assessment" }
                                p { class: "text-gray-600",
                                    "Answer targeted questions to help AI understand your learning style and existing knowledge"
                                }
                            }
                            div { class: "p-8 bg-white rounded-2xl shadow-lg hover:shadow-xl transition",
                                div { class: "text-4xl mb-4", "🗺️" }
                                h3 { class: "text-xl font-bold mb-3", "Visual Roadmaps" }
                                p { class: "text-gray-600",
                                    "Get interactive, node-based roadmaps with curated resources and clear prerequisites"
                                }
                            }
                            div { class: "p-8 bg-white rounded-2xl shadow-lg hover:shadow-xl transition",
                                div { class: "text-4xl mb-4", "📊" }
                                h3 { class: "text-xl font-bold mb-3", "Track Progress" }
                                p { class: "text-gray-600",
                                    "Mark skills as completed and watch your learning journey unfold"
                                }
                            }
                        }
                    }
                }
            }
        }
    };
    rsx! {
        div { class: "min-h-screen flex items-center justify-center",
            p { "Redirecting to dashboard..." }
            script { "window.location.href = '/dashboard';" }
        }
    }
}
