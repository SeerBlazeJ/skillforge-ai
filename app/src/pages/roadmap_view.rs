use crate::{
    models::{LearningResource, Roadmap, RoadmapNode},
    server_functions::{get_roadmap, toggle_node_completion},
    Route,
};
use dioxus::prelude::*;
use std::collections::{HashMap, HashSet};

#[component]
pub fn RoadmapView(id: String) -> Element {
    let roadmap_id = id.clone();

    let roadmap: Resource<Result<Roadmap, ServerFnError>> = use_resource(move || {
        let id = id.clone();
        async move { get_roadmap(id).await }
    });

    let mut selected_node_id = use_signal(|| None::<String>);

    return rsx! {
        div { class: "min-h-screen bg-[#050505] text-gray-100 font-sans selection:bg-teal-500/30 selection:text-teal-200",
            // Top nav
            nav { class: "bg-[#050505]/80 backdrop-blur-md border-b border-white/5 sticky top-0 z-40",
                div { class: "container mx-auto px-6 py-4 flex justify-between items-center",
                    Link {
                        to: Route::Dashboard {},
                        class: "text-teal-400 hover:text-teal-300 transition-colors font-medium",
                        "← Back to Dashboard"
                    }

                    match roadmap.read_unchecked().as_ref() {
                        Some(Ok(r)) => rsx! {
                            h1 { class: "text-xl font-bold text-gray-100 truncate max-w-[60vw]", "{r.skill_name.clone()}" }
                        },
                        Some(Err(_)) => rsx! {
                            h1 { class: "text-xl font-bold text-gray-100", "Roadmap" }
                        },
                        None => rsx! {
                            h1 { class: "text-xl font-bold text-gray-100", "Loading..." }
                        },
                    }
                }
            }

            match roadmap.read_unchecked().as_ref() {
                Some(Ok(roadmap_data)) => {
                    let ordered = ordered_nodes(roadmap_data);

                    // Build sidebar element in Rust (avoids RSX match parsing issues)
                    let sidebar: Element = match selected_node_id() {
                        Some(id) => {
                            let node = roadmap_data.nodes.iter().find(|n| n.id == id).cloned();
                            match node {
                                Some(node) => rsx! {
                                    NodeDetailSidebar {
                                        node,
                                        roadmap: roadmap_data.clone(),
                                        roadmap_id: roadmap_id.clone(),
                                        roadmap_resource: roadmap,
                                        selected_node_id,
                                        on_close: move |_| selected_node_id.set(None),
                                    }
                                },
                                None => rsx! {
                                    RoadmapOverview { roadmap: roadmap_data.clone() }
                                },
                            }
                        }
                        None => rsx! {
                            RoadmapOverview { roadmap: roadmap_data.clone() }
                        },
                    };

                    rsx! {
                        div { class: "flex h-[calc(100vh-72px)]",
                            // Main timeline
                            div { class: "flex-1 overflow-y-auto custom-scroll p-6 bg-[#050505]",
                                div { class: "max-w-4xl mx-auto",
                                    div { class: "flex items-center justify-between mb-6",
                                        h2 { class: "text-lg font-semibold text-gray-100", "Learning path" }
                                        RoadmapProgressPill { roadmap: roadmap_data.clone() }
                                    }

                                    div { class: "space-y-4",
                                        for (idx , node) in ordered.into_iter().enumerate() {
                                            {
                                                let node_id = node.id.clone();
                                                let is_selected = selected_node_id().as_deref() == Some(&node_id);

                                                rsx! {
                                                    RoadmapStepCard {
                                                        key: "step-{node_id}",
                                                        idx: idx + 1,
                                                        node,
                                                        roadmap: roadmap_data.clone(),
                                                        roadmap_id: roadmap_id.clone(),
                                                        roadmap_resource: roadmap,
                                                        selected: is_selected,
                                                        on_select: move |_| selected_node_id.set(Some(node_id.clone())),
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }

                            // Sidebar
                            div { class: "w-[26rem] bg-[#0b0c0e] border-l border-white/10 overflow-y-auto custom-scroll",
                                {sidebar}
                            }
                        }
                    }
                }

                Some(Err(e)) => rsx! {
                    div { class: "container mx-auto px-6 py-12",
                        div { class: "bg-red-500/10 text-red-300 p-6 rounded-lg border border-red-500/20 backdrop-blur-md",
                            "Error loading roadmap: {e}"
                        }
                    }
                },

                None => rsx! {
                    div { class: "container mx-auto px-6 py-12 text-center",
                        div { class: "text-gray-500", "Loading roadmap..." }
                    }
                },
            }
        }
    };
}

fn ordered_nodes(roadmap: &Roadmap) -> Vec<RoadmapNode> {
    let by_id: HashMap<String, RoadmapNode> = roadmap
        .nodes
        .iter()
        .cloned()
        .map(|n| (n.id.clone(), n))
        .collect();

    // Head candidates: nodes with no prev OR prev missing
    let mut heads: Vec<RoadmapNode> = roadmap
        .nodes
        .iter()
        .filter(|n| {
            n.prev_node_id
                .as_ref()
                .and_then(|pid| by_id.get(pid))
                .is_none()
        })
        .cloned()
        .collect();

    if heads.is_empty() {
        // Fallback if links are broken
        heads = roadmap.nodes.clone();
    }

    heads.sort_by(|a, b| a.skill_name.cmp(&b.skill_name));

    let mut visited = HashSet::<String>::new();
    let mut out = Vec::<RoadmapNode>::new();

    for head in heads {
        let mut cur_id = head.id.clone();
        loop {
            if !visited.insert(cur_id.clone()) {
                break;
            }
            let Some(node) = by_id.get(&cur_id).cloned() else {
                break;
            };
            out.push(node.clone());
            match node.next_node_id.as_ref() {
                Some(next) if by_id.contains_key(next) => cur_id = next.clone(),
                _ => break,
            }
        }
    }

    // Add any remaining nodes not reached (keeps UI robust)
    let mut remaining: Vec<RoadmapNode> = roadmap
        .nodes
        .iter()
        .filter(|n| !visited.contains(&n.id))
        .cloned()
        .collect();

    remaining.sort_by(|a, b| a.skill_name.cmp(&b.skill_name));
    out.extend(remaining);

    out
}

fn label_for_ref(roadmap: &Roadmap, reference: &str) -> String {
    roadmap
        .nodes
        .iter()
        .find(|n| n.id == reference)
        .map(|n| n.skill_name.clone())
        .unwrap_or_else(|| reference.to_string())
}

fn truncate_text(text: &str, max_len: usize) -> String {
    if text.len() <= max_len {
        text.to_string()
    } else {
        format!("{}...", &text[..max_len.saturating_sub(3)])
    }
}

#[component]
fn RoadmapProgressPill(roadmap: Roadmap) -> Element {
    let completed = roadmap.nodes.iter().filter(|n| n.is_completed).count();
    let total = roadmap.nodes.len();
    let progress = if total > 0 {
        (completed * 100) / total
    } else {
        0
    };

    rsx! {
        div { class: "inline-flex items-center gap-2 px-3 py-1 rounded-full bg-teal-500/10 text-teal-300 text-sm font-medium border border-teal-500/20",
            span { "{progress}% complete" }
            span { class: "text-gray-600", "•" }
            span { "{completed}/{total}" }
        }
    }
}

#[component]
fn RoadmapStepCard(
    idx: usize,
    node: RoadmapNode,
    roadmap: Roadmap,
    roadmap_id: String,
    roadmap_resource: Resource<Result<Roadmap, ServerFnError>>,
    selected: bool,
    on_select: EventHandler<()>,
) -> Element {
    let node_id = node.id.clone();

    let status_dot = if node.is_completed {
        "bg-green-500"
    } else {
        "bg-white/20"
    };

    rsx! {
        div {
            class: format!(
                "rounded-xl border p-5 transition cursor-pointer {}",
                if selected {
                    "border-teal-500/40 bg-teal-500/10 shadow-[0_0_18px_rgba(20,184,166,0.12)]"
                } else {
                    "border-white/10 bg-[#0f1012]/40 hover:bg-[#0f1012]/70 hover:border-white/20"
                },
            ),
            onclick: move |_| on_select.call(()),

            div { class: "flex items-start justify-between gap-4",
                div { class: "min-w-0",
                    div { class: "flex items-center gap-3",
                        div { class: format!("w-3 h-3 rounded-full {status_dot}") }
                        span { class: "text-xs font-semibold text-gray-500", "#{idx}" }
                        h3 { class: "text-base font-semibold text-gray-100 truncate",
                            "{node.skill_name.clone()}"
                        }
                    }

                    p { class: "mt-2 text-sm text-gray-400 leading-relaxed",
                        "{truncate_text(&node.description, 140)}"
                    }

                    if !node.prerequisites.is_empty() {
                        div { class: "mt-3 flex flex-wrap gap-2",
                            for prereq in &node.prerequisites {
                                {
                                    let label = label_for_ref(&roadmap, prereq);
                                    rsx! {
                                        span {
                                            key: "pr-{node.id}-{prereq}",
                                            class: "text-xs px-2 py-1 rounded-full bg-white/5 text-gray-300 border border-white/10",
                                            "{label}"
                                        }
                                    }
                                }
                            }
                        }
                    }

                    div { class: "mt-3 flex items-center gap-3 text-xs text-gray-500",
                        span { "Resources: {node.resources.len()}" }
                        span { "•" }
                        span {
                            if node.is_completed {
                                "Completed"
                            } else {
                                "In progress"
                            }
                        }
                    }
                }

                button {
                    class: format!(
                        "shrink-0 px-3 py-2 rounded-lg text-sm font-semibold transition {}",
                        if node.is_completed {
                            "bg-white/10 text-gray-100 hover:bg-white/15"
                        } else {
                            "bg-gradient-to-r from-teal-500 to-blue-600 text-white hover:shadow-[0_0_16px_rgba(20,184,166,0.25)]"
                        },
                    ),
                    onclick: move |e| {
                        e.stop_propagation();
                        spawn({
                            let roadmap_id = roadmap_id.clone();
                            let node_id = node_id.clone();
                            async move {
                                let _ = toggle_node_completion(roadmap_id, node_id).await;
                                roadmap_resource.restart();
                            }
                        });
                    },
                    if node.is_completed {
                        "Undo"
                    } else {
                        "Complete"
                    }
                }
            }
        }
    }
}

#[component]
fn NodeDetailSidebar(
    node: RoadmapNode,
    roadmap: Roadmap,
    roadmap_id: String,
    roadmap_resource: Resource<Result<Roadmap, ServerFnError>>,
    selected_node_id: Signal<Option<String>>,
    on_close: EventHandler<()>,
) -> Element {
    let prev_label = node
        .prev_node_id
        .as_deref()
        .map(|v| label_for_ref(&roadmap, v));

    let next_label = node
        .next_node_id
        .as_deref()
        .map(|v| label_for_ref(&roadmap, v));

    rsx! {
        div { class: "p-6",
            div { class: "flex justify-between items-start mb-6",
                h2 { class: "text-2xl font-bold text-gray-100", "{node.skill_name.clone()}" }
                button {
                    onclick: move |_| on_close.call(()),
                    class: "text-gray-500 hover:text-gray-200 transition",
                    "✕"
                }
            }

            div { class: "mb-4",
                div {
                    class: format!(
                        "inline-flex items-center px-3 py-1 rounded-full text-sm font-medium {}",
                        if node.is_completed {
                            "bg-green-500/10 text-green-300 border border-green-500/20"
                        } else {
                            "bg-yellow-500/10 text-yellow-200 border border-yellow-500/20"
                        },
                    ),
                    if node.is_completed {
                        "✓ Completed"
                    } else {
                        "In progress"
                    }
                }
            }

            if prev_label.is_some() || next_label.is_some() {
                div { class: "mb-6 grid grid-cols-2 gap-3",
                    if let Some(prev) = prev_label {
                        button {
                            class: "p-3 bg-white/5 border border-white/10 rounded-lg text-left hover:bg-white/10 transition",
                            onclick: move |_| {
                                if let Some(prev_id) = node.prev_node_id.clone() {
                                    selected_node_id.set(Some(prev_id));
                                }
                            },
                            div { class: "text-xs text-gray-500 font-semibold uppercase tracking-wide",
                                "Previous"
                            }
                            div { class: "text-sm text-gray-100 font-medium", "{prev}" }
                        }
                    }

                    if let Some(next) = next_label {
                        button {
                            class: "p-3 bg-white/5 border border-white/10 rounded-lg text-left hover:bg-white/10 transition",
                            onclick: move |_| {
                                if let Some(next_id) = node.next_node_id.clone() {
                                    selected_node_id.set(Some(next_id));
                                }
                            },
                            div { class: "text-xs text-gray-500 font-semibold uppercase tracking-wide",
                                "Next"
                            }
                            div { class: "text-sm text-gray-100 font-medium", "{next}" }
                        }
                    }
                }
            }

            div { class: "mb-6",
                h3 { class: "text-sm font-semibold text-gray-400 uppercase tracking-wide mb-2",
                    "Description"
                }
                p { class: "text-gray-400 leading-relaxed", "{node.description.clone()}" }
            }

            if !node.prerequisites.is_empty() {
                div { class: "mb-6",
                    h3 { class: "text-sm font-semibold text-gray-400 uppercase tracking-wide mb-2",
                        "Prerequisites"
                    }
                    div { class: "flex flex-wrap gap-2",
                        for prereq in &node.prerequisites {
                            {
                                let prereq_id = prereq.clone();
                                let label = label_for_ref(&roadmap, prereq);
                                let can_jump = roadmap.nodes.iter().any(|n| n.id == prereq.as_str());

                                rsx! {
                                    span {
                                        key: "chip-{node.id}-{prereq}",
                                        class: "text-xs px-2 py-1 rounded-full bg-white/5 text-gray-300 border border-white/10 cursor-pointer hover:bg-white/10",
                                        onclick: move |_| {
                                            if can_jump {
                                                selected_node_id.set(Some(prereq_id.clone()));
                                            }
                                        },
                                        "{label}"
                                    }
                                }
                            }
                        }
                    }
                }
            }

            if !node.resources.is_empty() {
                div { class: "mb-6",
                    h3 { class: "text-sm font-semibold text-gray-400 uppercase tracking-wide mb-3",
                        "Learning Resources"
                    }
                    div { class: "space-y-3",
                        for resource in &node.resources {
                            ResourceCard { resource: resource.clone() }
                        }
                    }
                }
            }

            button {
                class: format!(
                    "w-full py-3 rounded-lg font-semibold transition {}",
                    if node.is_completed {
                        "bg-white/10 text-gray-100 hover:bg-white/15"
                    } else {
                        "bg-gradient-to-r from-teal-500 to-blue-600 text-white hover:shadow-[0_0_16px_rgba(20,184,166,0.25)]"
                    },
                ),
                onclick: move |_| {
                    spawn({
                        let roadmap_id = roadmap_id.clone();
                        let node_id = node.id.clone();
                        async move {
                            let _ = toggle_node_completion(roadmap_id, node_id).await;
                            roadmap_resource.restart();
                        }
                    });
                },
                if node.is_completed {
                    "Mark as Incomplete"
                } else {
                    "Mark as Complete"
                }
            }
        }
    }
}

#[component]
fn ResourceCard(resource: LearningResource) -> Element {
    rsx! {
        div { class: "p-4 bg-white/5 border border-white/10 rounded-lg hover:bg-white/10 transition",
            div { class: "flex items-start justify-between mb-2",
                h4 { class: "font-semibold text-gray-100 flex-1", "{resource.title.clone()}" }
                span { class: "text-xs px-2 py-1 bg-teal-500/10 text-teal-300 rounded border border-teal-500/20",
                    "{resource.resource_type.clone()}"
                }
            }

            p { class: "text-sm text-gray-400 mb-2", "{resource.platform.clone()}" }

            if let Some(url) = &resource.url {
                a {
                    href: "{url}",
                    target: "_blank",
                    class: "text-sm text-teal-400 hover:text-teal-300 font-medium transition-colors",
                    "Open Resource →"
                }
            }
        }
    }
}

#[component]
fn RoadmapOverview(roadmap: Roadmap) -> Element {
    let completed = roadmap.nodes.iter().filter(|n| n.is_completed).count();
    let total = roadmap.nodes.len();
    let progress = if total > 0 {
        (completed * 100) / total
    } else {
        0
    };

    rsx! {
        div { class: "p-6",
            h2 { class: "text-2xl font-bold text-gray-100 mb-4", "Roadmap Overview" }

            div { class: "mb-6",
                div { class: "flex justify-between text-sm text-gray-400 mb-2",
                    span { "Progress" }
                    span { "{completed}/{total} completed" }
                }

                div { class: "w-full bg-white/10 rounded-full h-3",
                    div {
                        class: "bg-gradient-to-r from-teal-500 to-blue-600 h-3 rounded-full transition-all",
                        style: "width: {progress}%;",
                    }
                }
            }

            div { class: "space-y-4",
                div { class: "p-4 bg-white/5 rounded-lg border border-white/10",
                    div { class: "text-3xl font-bold text-teal-300", "{progress}%" }
                    p { class: "text-sm text-gray-400", "Overall completion" }
                }

                div { class: "p-4 bg-white/5 rounded-lg border border-white/10",
                    div { class: "text-3xl font-bold text-gray-100", "{total}" }
                    p { class: "text-sm text-gray-400", "Total skills to learn" }
                }

                div { class: "p-4 bg-white/5 rounded-lg border border-white/10",
                    div { class: "text-3xl font-bold text-green-300", "{completed}" }
                    p { class: "text-sm text-gray-400", "Skills completed" }
                }
            }
        }
    }
}
