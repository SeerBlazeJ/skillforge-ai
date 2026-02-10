use crate::utils::{clear_session_token, get_session_token};
use crate::{
    models::Roadmap,
    server_functions::{delete_roadmap, delete_session, get_progress_report, get_user_roadmaps},
    Route,
};
use chrono::{DateTime, Duration, Utc};
use dioxus::prelude::*;
use std::collections::HashMap;

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
    let session_token_for_progress = session_token.clone();

    // Default graph duration
    let report_days = use_signal(|| 7u16);

    let roadmaps = use_resource(move || {
        let session_token = session_token.clone();
        async move { get_user_roadmaps(session_token).await }
    });

    // Fetch progress report
    let progress = use_resource(move || {
        let session_token = session_token_for_progress.clone();
        let days = *report_days.read();
        async move { get_progress_report(days, session_token).await }
    });

    rsx! {
        div { class: "min-h-screen bg-[#050505] text-gray-100 font-sans selection:bg-teal-500/30 selection:text-teal-200",
            // Navigation
            nav { class: "sticky top-0 z-50 bg-[#050505]/80 backdrop-blur-md border-b border-white/5",
                div { class: "container mx-auto px-6 py-4 flex justify-between items-center",
                    h1 { class: "text-2xl font-bold tracking-tight",
                        span { class: "bg-gradient-to-r from-teal-400 to-blue-500 bg-clip-text text-transparent",
                            "Skill"
                        }
                        span { class: "text-gray-100", "Forge" }
                    }
                    div { class: "flex items-center gap-6",
                        Link {
                            to: Route::Profile {},
                            class: "text-gray-400 hover:text-white transition-colors text-sm font-medium",
                            "Profile"
                        }
                        button {
                            onclick: move |_| {
                                let session_token_clone = session_token_clone.clone();
                                clear_session_token();
                                nav.push(Route::Login {});
                                async move {
                                    let _ = delete_session(session_token_clone.clone()).await;
                                }
                            },
                            class: "group flex items-center justify-center w-10 h-10 rounded-lg bg-red-500/10 text-red-400 hover:bg-red-500 hover:text-white transition-all duration-300",
                            title: "Logout",
                            svg {
                                class: "w-5 h-5",
                                view_box: "0 0 24 24",
                                fill: "none",
                                stroke: "currentColor",
                                stroke_width: "2",
                                path { d: "M17 16l4-4m0 0l-4-4m4 4H7m6 4v1a3 3 0 01-3 3H6a3 3 0 01-3-3V7a3 3 0 013-3h4a3 3 0 013 3v1" }
                            }
                        }
                    }
                }
            }

            main { class: "container mx-auto px-6 py-10",
                // Activity Graph Section
                div { class: "mb-12",
                    h2 { class: "text-3xl font-bold text-gray-100 mb-6", "Activity" }
                    match progress.read_unchecked().as_ref() {
                        Some(Ok(Some(data))) => rsx! {
                            ActivityChart { data: data.clone(), days: *report_days.read() }
                        },
                        Some(Ok(None)) => rsx! {
                            div { class: "p-6 bg-[#0f1012]/60 border border-white/5 rounded-xl text-gray-400 text-center",
                                "No learning activity recorded in the last {report_days} days."
                            }
                        },
                        Some(Err(e)) => rsx! {
                            div { class: "p-4 bg-red-500/10 border border-red-500/20 text-red-400 rounded-lg",
                                "Error loading progress: {e}"
                            }
                        },
                        None => rsx! {
                            div { class: "h-64 bg-[#0f1012]/60 border border-white/5 rounded-xl flex items-center justify-center",
                                div { class: "animate-spin rounded-full h-8 w-8 border-t-2 border-b-2 border-teal-500" }
                            }
                        },
                    }
                }

                div { class: "flex justify-between items-center mb-10",
                    h2 { class: "text-3xl font-bold text-gray-100", "My Roadmaps" }
                    Link {
                        to: Route::CreateRoadmap {},
                        class: "px-5 py-2.5 bg-gradient-to-r from-teal-500 to-blue-600 text-white rounded-lg hover:shadow-[0_0_20px_rgba(20,184,166,0.3)] hover:-translate-y-0.5 transition-all duration-300 font-medium text-sm flex items-center gap-2",
                        span { class: "text-lg leading-none", "+" }
                        "New Roadmap"
                    }
                }

                match roadmaps.read_unchecked().as_ref() {
                    Some(Ok(roadmaps_data)) => rsx! {
                        div { class: "grid md:grid-cols-2 lg:grid-cols-3 gap-6 animate-slide-up",
                            for roadmap in roadmaps_data {
                                RoadmapCard { roadmap: roadmap.clone(), roadmaps_resource: roadmaps }
                            }
                        }
                    },
                    Some(Err(e)) => rsx! {
                        div { class: "p-4 bg-red-500/10 border border-red-500/20 text-red-400 rounded-lg",
                            "Error loading roadmaps: {e}"
                        }
                    },
                    None => rsx! {
                        div { class: "flex justify-center py-12",
                            div { class: "animate-spin rounded-full h-8 w-8 border-t-2 border-b-2 border-teal-500" }
                        }
                    },
                }
            }
        }
    }
}

#[component]
fn ActivityChart(data: HashMap<DateTime<Utc>, u8>, days: u16) -> Element {
    // Generate the list of dates for the X-axis
    let now = Utc::now();
    let dates: Vec<DateTime<Utc>> = (0..days)
        .map(|i| now - Duration::days((days - 1 - i) as i64))
        .collect();

    // Determine Y-axis max value for scaling (min height 5 to avoid flat graphs on low data)
    let max_val = data.values().max().copied().unwrap_or(0).max(5) as f32;

    // We generate the chart bars HERE, outside the rsx! macro, to avoid parser errors
    let chart_bars = dates.into_iter().map(|date| {
        // Normalize lookup: find the entry in data where the date matches
        let day_count = data.iter()
            .find(|(k, _)| k.date_naive() == date.date_naive())
            .map(|(_, v)| *v)
            .unwrap_or(0);

        let height_pct = ((day_count as f32 / max_val) * 100.0).min(100.0);
        let is_today = date.date_naive() == now.date_naive();
        let date_str = date.format("%a").to_string();
        let key = date.timestamp();

        let bar_color = if is_today {
            "bg-gradient-to-t from-teal-500 to-teal-400"
        } else {
            "bg-gray-700 group-hover:bg-gray-600"
        };

        let label_color = if is_today {
            "text-teal-400"
        } else {
            "text-gray-500"
        };

        rsx! {
            div {
                key: "{key}",
                class: "flex-1 flex flex-col items-center gap-3 group h-full justify-end",
                div { class: "relative w-full h-full flex items-end justify-center",
                    // Bar
                    div {
                        class: "w-full max-w-[3rem] rounded-t-sm transition-all duration-700 ease-out relative {bar_color}",
                        style: "height: {height_pct}%",

                        // Tooltip
                        div { class: "absolute -top-10 left-1/2 -translate-x-1/2 bg-[#1a1b1e] text-white text-xs font-medium px-2 py-1.5 rounded-md opacity-0 group-hover:opacity-100 transition-opacity border border-white/10 shadow-xl whitespace-nowrap pointer-events-none z-10",
                            "{day_count} skills"
                            div { class: "absolute -bottom-1 left-1/2 -translate-x-1/2 w-2 h-2 bg-[#1a1b1e] rotate-45 border-r border-b border-white/10" }
                        }
                    }
                }
                // X-axis Label
                span { class: "text-[10px] sm:text-xs font-medium uppercase tracking-wider {label_color}",
                    "{date_str}"
                }
            }
        }
    });

    rsx! {
        div { class: "w-full bg-[#0f1012]/60 backdrop-blur-md border border-white/5 rounded-xl p-6",
            div { class: "flex items-end justify-between h-40 gap-2 sm:gap-4", {chart_bars} }
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
        div { class: "group relative bg-[#0f1012]/60 backdrop-blur-md border border-white/5 rounded-xl p-6 hover:border-teal-500/30 transition-all duration-300 hover:-translate-y-1 hover:shadow-lg hover:shadow-black/50",
            // Delete button
            button {
                onclick: move |e| {
                    e.stop_propagation();
                    show_confirm.set(true);
                },
                class: "absolute top-4 right-4 p-2 text-gray-500 hover:text-red-400 hover:bg-red-500/10 rounded-lg transition-colors opacity-0 group-hover:opacity-100",
                title: "Delete roadmap",
                svg {
                    class: "w-4 h-4",
                    view_box: "0 0 24 24",
                    fill: "none",
                    stroke: "currentColor",
                    stroke_width: "2",
                    path { d: "M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16" }
                }
            }

            // Confirmation Modal (Portal-like overlay)
            if *show_confirm.read() {
                div {
                    class: "fixed inset-0 z-[100] flex items-center justify-center bg-black/80 backdrop-blur-sm p-4",
                    onclick: move |_| show_confirm.set(false),
                    div {
                        class: "bg-[#1a1b1e] border border-white/10 rounded-xl p-6 max-w-sm w-full shadow-2xl animate-scale-in",
                        onclick: move |e| e.stop_propagation(),
                        h3 { class: "text-lg font-bold text-gray-100 mb-2", "Delete Roadmap?" }
                        p { class: "text-gray-400 mb-6 text-sm",
                            "Are you sure you want to delete \"{roadmap.skill_name}\"? This cannot be undone."
                        }
                        div { class: "flex gap-3 justify-end",
                            button {
                                onclick: move |_| show_confirm.set(false),
                                class: "px-4 py-2 text-gray-400 hover:text-white hover:bg-white/5 rounded-lg transition text-sm font-medium",
                                "Cancel"
                            }
                            button {
                                onclick: move |_| {
                                    let roadmap_id = roadmap_id_clone.clone();
                                    spawn(async move {
                                        if delete_roadmap(roadmap_id).await.is_ok() {
                                            roadmaps_resource.restart();
                                        }
                                    });
                                    show_confirm.set(false);
                                },
                                class: "px-4 py-2 bg-red-500/10 text-red-400 hover:bg-red-500 hover:text-white rounded-lg transition text-sm font-medium",
                                "Delete"
                            }
                        }
                    }
                }
            }

            Link {
                to: Route::RoadmapView {
                    id: roadmap_id.clone(),
                },
                class: "block",
                h3 { class: "text-xl font-bold text-gray-100 mb-4 pr-8 truncate group-hover:text-teal-400 transition-colors",
                    "{roadmap.skill_name}"
                }

                div { class: "mb-4 space-y-2",
                    div { class: "flex justify-between text-xs text-gray-400",
                        span { "{completed}/{total} steps" }
                        span { "{progress}%" }
                    }
                    div { class: "w-full bg-gray-800 rounded-full h-1.5 overflow-hidden",
                        div {
                            class: "bg-gradient-to-r from-teal-500 to-blue-500 h-full rounded-full transition-all duration-500",
                            style: "width: {progress}%",
                        }
                    }
                }

                p { class: "text-xs text-gray-600",
                    "Updated {roadmap.updated_at.format(\"%b %d, %Y\")}"
                }
            }
        }
    }
}
