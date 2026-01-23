use crate::utils::get_session_token;
use crate::{models::Roadmap, server_functions::get_user_roadmaps, Route};
use dioxus::prelude::*;

#[component]
pub fn Dashboard() -> Element {
    let session_token = match get_session_token() {
        Some(token) => token,
        None => {
            return rsx! {
                div { class: "min-h-screen flex items-center justify-center",
                    p { "Redirecting to login..." }
                    script { "window.location.href = '/login';" }
                }
            };
        }
    };
    eprintln!("Session token recieved: {}", &session_token);
    let roadmaps = use_resource(move || {
        let session_token = session_token.clone();
        async move { get_user_roadmaps(session_token).await }
    });

    rsx! {
        div { class: "min-h-screen bg-gray-50",
            nav { class: "bg-white shadow-sm",
                div { class: "container mx-auto px-6 py-4 flex justify-between items-center",
                    h1 { class: "text-2xl font-bold text-indigo-600", "SkillForge" }
                    div { class: "space-x-4",
                        Link {
                            to: Route::Profile {},
                            class: "text-gray-700 hover:text-indigo-600",
                            "Profile"
                        }
                    }
                }
            }

            main { class: "container mx-auto px-6 py-8",
                div { class: "flex justify-between items-center mb-8",
                    h2 { class: "text-3xl font-bold text-gray-900", "My Roadmaps" }
                    Link {
                        to: Route::CreateRoadmap {},
                        class: "px-6 py-3 bg-indigo-600 text-white rounded-lg hover:bg-indigo-700 transition font-medium",
                        "+ New Roadmap"
                    }
                }

                match roadmaps.read_unchecked().as_ref() {
                    Some(Ok(roadmaps)) => rsx! {
                        div { class: "grid md:grid-cols-2 lg:grid-cols-3 gap-6",
                            for roadmap in roadmaps {
                                RoadmapCard { roadmap: roadmap.clone() }
                            }
                        }
                    },
                    Some(Err(e)) => rsx! {
                        div { class: "text-red-600", "Error loading roadmaps: {e}" }
                    },
                    None => rsx! {
                        div { class: "text-gray-500", "Loading..." }
                    },
                }
            }
        }
    }
}

#[component]
fn RoadmapCard(roadmap: Roadmap) -> Element {
    let completed = roadmap.nodes.iter().filter(|n| n.is_completed).count();
    let total = roadmap.nodes.len();
    let progress = if total > 0 {
        (completed * 100) / total
    } else {
        0
    };

    rsx! {
        Link {
            to: Route::RoadmapView {
                id: roadmap.id.clone().unwrap_or_default(),
            },
            class: "block",
            div { class: "bg-white rounded-xl shadow-md hover:shadow-xl transition p-6",
                h3 { class: "text-xl font-bold text-gray-900 mb-2", {roadmap.skill_name} }
                div { class: "mb-4",
                    div { class: "flex justify-between text-sm text-gray-600 mb-2",
                        span { "{completed}/{total} completed" }
                        span { "{progress}%" }
                    }
                    div { class: "w-full bg-gray-200 rounded-full h-2",
                        div {
                            class: "bg-indigo-600 h-2 rounded-full transition-all",
                            style: "width: {progress}%",
                        }
                    }
                }
                p { class: "text-sm text-gray-500",
                    "Updated {roadmap.updated_at.format(\"%B %d, %Y\")}"
                }
            }
        }
    }
}
