use crate::utils::{clear_session_token, get_session_token};

use crate::{
    models::Roadmap,
    server_functions::{delete_roadmap, delete_session, get_user_roadmaps},
    Route,
};
use dioxus::prelude::*;

#[component]
pub fn Dashboard() -> Element {
    let nav = navigator();
    let token = get_session_token();

    if token.is_none() {
        nav.push(Route::Login {});
        return rsx! { "Redirecting..." };
    }

    let session_token = token.unwrap();
    let session_token_clone = session_token.clone();

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
                        div { class: "flex items-center gap-6 p-4",
                            Link {
                                to: Route::Profile {},
                                class: "text-gray-700 hover:text-indigo-600",
                                "Profile"
                            }
                        }
                        div { class: "flex items-center gap-6 p-4",
                            button {
                                onclick: move |_| {
                                    let session_token_clone = session_token_clone.clone();
                                    clear_session_token();
                                    nav.push(Route::Login {});
                                    async move {
                                        let _ = delete_session(session_token_clone.clone()).await;
                                    }
                                },
                                class: "group flex items-center justify-start w-11 h-11 bg-red-600 rounded-xl cursor-pointer relative overflow-hidden transition-all duration-200 shadow-lg hover:w-32 hover:rounded-lg active:translate-x-1 active:translate-y-1",
                                div { class: "flex items-center justify-center w-full transition-all duration-300 group-hover:justify-start group-hover:px-3",
                                    svg {
                                        class: "w-4 h-4",
                                        view_box: "0 0 512 512",
                                        fill: "white",
                                        path { d: "M377.9 105.9L500.7 228.7c7.2 7.2 11.3 17.1 11.3 27.3s-4.1 20.1-11.3 27.3L377.9 406.1c-6.4 6.4-15 9.9-24 9.9c-18.7 0-33.9-15.2-33.9-33.9l0-62.1-128 0c-17.7 0-32-14.3-32-32l0-64c0-17.7 14.3-32 32-32l128 0 0-62.1c0-18.7 15.2-33.9 33.9-33.9c9 0 17.6 3.6 24 9.9zM160 96L96 96c-17.7 0-32 14.3-32 32l0 256c0 17.7 14.3 32 32 32l64 0c17.7 0 32 14.3 32 32s-14.3 32-32 32l-64 0c-53 0-96-43-96-96L0 128C0 75 43 32 96 32l64 0c17.7 0 32 14.3 32 32s-14.3 32-32 32z" }
                                    }
                                }
                                div { class: "absolute right-5 transform translate-x-full opacity-0 text-white text-lg font-semibold transition-all duration-300 group-hover:translate-x-0 group-hover:opacity-100",
                                    "Logout"
                                }
                            }
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
                    Some(Ok(roadmaps_data)) => rsx! {
                        div { class: "grid md:grid-cols-2 lg:grid-cols-3 gap-6",
                            for roadmap in roadmaps_data {
                                RoadmapCard { roadmap: roadmap.clone(), roadmaps_resource: roadmaps }
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
fn RoadmapCard(
    roadmap: Roadmap,
    roadmaps_resource: Resource<Result<Vec<Roadmap>, ServerFnError>>,
) -> Element {
    let completed = roadmap.nodes.iter().filter(|n| n.is_completed).count();
    let total = roadmap.nodes.len();
    let progress = if total > 0 {
        (completed * 100) / total
    } else {
        0
    };

    let roadmap_id = roadmap.id.clone().unwrap_or_default();
    let roadmap_id_clone = roadmap_id.clone();
    let mut show_confirm = use_signal(|| false);

    rsx! {
        div { class: "relative bg-white rounded-xl shadow-md hover:shadow-xl transition p-6",
            // Delete button - positioned in top right
            button {
                onclick: move |e| {
                    e.stop_propagation();
                    show_confirm.set(true);
                },
                class: "absolute top-4 right-4 p-2 text-gray-400 hover:text-red-600 hover:bg-red-50 rounded-lg transition",
                title: "Delete roadmap",
                svg {
                    class: "w-5 h-5",
                    view_box: "0 0 24 24",
                    fill: "none",
                    stroke: "currentColor",
                    stroke_width: "2",
                    path { d: "M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16" }
                }
            }

            // Confirmation modal
            if *show_confirm.read() {
                div {
                    class: "fixed inset-0 z-50 flex items-center justify-center bg-black bg-opacity-50",
                    onclick: move |_| show_confirm.set(false),

                    div {
                        class: "bg-white rounded-lg p-6 max-w-md mx-4",
                        onclick: move |e| e.stop_propagation(),

                        h3 { class: "text-xl font-bold text-gray-900 mb-2", "Delete Roadmap?" }
                        p { class: "text-gray-600 mb-6",
                            "Are you sure you want to delete \"{roadmap.skill_name}\"? This action cannot be undone."
                        }

                        div { class: "flex gap-3 justify-end",
                            button {
                                onclick: move |_| show_confirm.set(false),
                                class: "px-4 py-2 text-gray-700 bg-gray-100 rounded-lg hover:bg-gray-200 transition font-medium",
                                "Cancel"
                            }
                            button {
                                onclick: move |_| {
                                    let roadmap_id = roadmap_id_clone.clone();
                                    spawn(async move {
                                        match delete_roadmap(roadmap_id).await {
                                            Ok(_) => {
                                                roadmaps_resource.restart();
                                            }
                                            Err(e) => {
                                                eprintln!("Failed to delete roadmap: {}", e);
                                            }
                                        }
                                    });
                                    show_confirm.set(false);
                                },
                                class: "px-4 py-2 text-white bg-red-600 rounded-lg hover:bg-red-700 transition font-medium",
                                "Delete"
                            }
                        }
                    }
                }
            }

            // Roadmap content - clickable link
            Link {
                to: Route::RoadmapView {
                    id: roadmap_id.clone(),
                },
                class: "block",

                h3 { class: "text-xl font-bold text-gray-900 mb-2 pr-8", {roadmap.skill_name} }

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
