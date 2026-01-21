use crate::{
    models::{LearningResource, Roadmap, RoadmapNode},
    server_functions::{get_roadmap, toggle_node_completion},
    Route,
};
use dioxus::prelude::*;

#[component]
pub fn RoadmapView(id: String) -> Element {
    let id_clone = id.clone(); // Clone for use in sidebar
    let roadmap = use_resource(move || {
        let id = id.clone();
        async move { get_roadmap(id).await }
    });

    let mut selected_node = use_signal(|| None::<RoadmapNode>);
    let view_box = use_signal(|| (0, 0, 1200, 800)); // Removed mut

    rsx! {
        div { class: "min-h-screen bg-gray-50",
            // Navigation
            nav { class: "bg-white shadow-sm",
                div { class: "container mx-auto px-6 py-4 flex justify-between items-center",
                    Link {
                        to: Route::Dashboard {},
                        class: "text-indigo-600 hover:text-indigo-700 font-medium",
                        "← Back to Dashboard"
                    }
                    match roadmap.read_unchecked().as_ref() {
                        Some(Ok(r)) => rsx! {
                            h1 { class: "text-xl font-bold text-gray-900", {r.skill_name.clone()} }
                        },
                        _ => rsx! {
                            h1 { "Loading..." }
                        },
                    }
                }
            }
            match roadmap.read_unchecked().as_ref() {
                Some(Ok(roadmap_data)) => rsx! {
                    div { class: "flex h-[calc(100vh-72px)]",
                        // SVG Roadmap View
                        div { class: "flex-1 bg-white overflow-auto",
                            RoadmapSVG {
                                roadmap: roadmap_data.clone(),
                                selected_node,
                                view_box,
                                roadmap_id: id_clone.clone(),
                            }
                        }

                    // Sidebar


                        div { class: "w-96 bg-gray-50 border-l border-gray-200 overflow-y-auto",
                            if let Some(node) = selected_node() {
                                NodeDetailSidebar {
                                    node,
                                    roadmap_id: id_clone.clone(), // Use id_clone
                                    on_close: move |_| selected_node.set(None),
                                }
                            } else {
                                RoadmapOverview { roadmap: roadmap_data.clone() }
                            }
                        }
                    }
                },
                Some(Err(e)) => rsx! {
                    div { class: "container mx-auto px-6 py-12",
                        div { class: "bg-red-50 text-red-700 p-6 rounded-lg", "Error loading roadmap: {e}" }
                    }
                },
                None => rsx! {
                    div { class: "container mx-auto px-6 py-12 text-center",
                        div { class: "text-gray-500", "Loading roadmap..." }
                    }
                },
            }
        }
    }
}

#[component]
fn RoadmapSVG(
    roadmap: Roadmap,
    selected_node: Signal<Option<RoadmapNode>>,
    view_box: Signal<(i32, i32, i32, i32)>,
    roadmap_id: String,
) -> Element {
    let (vb_x, vb_y, vb_width, vb_height) = view_box();

    rsx! {
        svg {
            class: "w-full h-full",
            view_box: "{vb_x} {vb_y} {vb_width} {vb_height}",

            // Draw edges (connections between nodes)
            for node in &roadmap.nodes {
                for prereq_id in &node.prerequisites {
                    if let Some(prereq_node) = roadmap.nodes.iter().find(|n| &n.id == prereq_id) {
                        {
                            let from_x = prereq_node.position.x + 100;
                            let from_y = prereq_node.position.y + 40;
                            let to_x = node.position.x;
                            let to_y = node.position.y + 40;

                            rsx! {
                                line {
                                    key: "edge-{prereq_node.id}-{node.id}",
                                    x1: "{from_x}",
                                    y1: "{from_y}",
                                    x2: "{to_x}",
                                    y2: "{to_y}",
                                    stroke: if node.is_completed { "#10b981" } else { "#d1d5db" },
                                    stroke_width: "3",
                                    stroke_dasharray: if node.is_completed { "0" } else { "5,5" },
                                    marker_end: "url(#arrowhead)",
                                }
                            }
                        }
                    }
                }
            }

            // Arrow marker definition
            defs {
                marker {
                    id: "arrowhead",
                    marker_width: "10",
                    marker_height: "10",
                    ref_x: "9",
                    ref_y: "3",
                    orient: "auto",
                    marker_units: "strokeWidth",
                    path { d: "M0,0 L0,6 L9,3 z", fill: "#d1d5db" }
                }

            }

            // Draw nodes
            for node in &roadmap.nodes {
                {
                    let node_clone = node.clone();
                    let roadmap_id_clone = roadmap_id.clone();

                    rsx! {
                        g {
                            key: "node-{node.id}",
                            transform: "translate({node.position.x}, {node.position.y})",
                            cursor: "pointer",
                            onclick: move |_| selected_node.set(Some(node_clone.clone())),

                        // Node background

                        // Node title

                        // Resource count

                        // Completion checkbox






                            rect {
                                width: "200",
                                height: "80",
                                rx: "12",
                                fill: if node.is_completed { "#10b981" } else { "#ffffff" },
                                stroke: if node.is_completed { "#059669" } else { "#e5e7eb" },
                                stroke_width: "2",
                                filter: "drop-shadow(0 4px 6px rgba(0,0,0,0.1))",
                            }

                            text {
                                x: "100",
                                y: "30",
                                text_anchor: "middle",
                                font_size: "14",
                                font_weight: "600",
                                fill: if node.is_completed { "#ffffff" } else { "#111827" },
                                {truncate_text(&node.skill_name, 20)}
                            }

                            text {
                                x: "100",
                                y: "50",
                                text_anchor: "middle",
                                font_size: "12",
                                fill: if node.is_completed { "#ffffff" } else { "#6b7280" },
                                "{node.resources.len()} resources"
                            }

                            {
                                let node_id = node.id.clone();
                                rsx! {
                                    circle {
                                        cx: "180",
                                        cy: "15",
                                        r: "12",
                                        fill: if node.is_completed { "#ffffff" } else { "#f3f4f6" },
                                        stroke: if node.is_completed { "#ffffff" } else { "#d1d5db" },
                                        stroke_width: "2",
                                        onclick: move |e| {
                                            e.stop_propagation();
                                            spawn({
                                                let roadmap_id = roadmap_id_clone.clone();
                                                let node_id = node_id.clone();
                                                async move {
                                                    let _ = toggle_node_completion(roadmap_id, node_id).await;
                                                }
                                            });
                                        },
                                    }


                                    if node.is_completed {
                                        path {
                                            d: "M175 15 L178 18 L185 11",
                                            stroke: "#10b981",
                                            stroke_width: "2",
                                            fill: "none",
                                            stroke_linecap: "round",
                                            stroke_linejoin: "round",
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

fn truncate_text(text: &str, max_len: usize) -> String {
    if text.len() <= max_len {
        text.to_string()
    } else {
        format!("{}...", &text[..max_len - 3])
    }
}

#[component]
fn NodeDetailSidebar(node: RoadmapNode, roadmap_id: String, on_close: EventHandler<()>) -> Element {
    rsx! {
        div { class: "p-6",
            div { class: "flex justify-between items-start mb-6",
                h2 { class: "text-2xl font-bold text-gray-900", {node.skill_name.clone()} }
                button {
                    onclick: move |_| on_close.call(()),
                    class: "text-gray-400 hover:text-gray-600",
                    "✕"
                }
            }

            div { class: "mb-6",
                div {
                    class: format!(
                        "inline-flex items-center px-3 py-1 rounded-full text-sm font-medium {}",
                        if node.is_completed {
                            "bg-green-100 text-green-800"
                        } else {
                            "bg-yellow-100 text-yellow-800"
                        },
                    ),
                    if node.is_completed {
                        "✓ Completed"
                    } else {
                        "⏳ In Progress"
                    }
                }
            }

            div { class: "mb-6",
                h3 { class: "text-sm font-semibold text-gray-700 uppercase tracking-wide mb-2",
                    "Description"
                }
                p { class: "text-gray-600 leading-relaxed", {node.description.clone()} }
            }

            if !node.prerequisites.is_empty() {
                div { class: "mb-6",
                    h3 { class: "text-sm font-semibold text-gray-700 uppercase tracking-wide mb-2",
                        "Prerequisites"
                    }
                    ul { class: "space-y-2",
                        for prereq in &node.prerequisites {
                            li {
                                key: "{prereq}",
                                class: "text-sm text-gray-600 flex items-center",
                                span { class: "mr-2", "→" }
                                {prereq.clone()}
                            }
                        }
                    }
                }
            }

            if !node.resources.is_empty() {
                div { class: "mb-6",
                    h3 { class: "text-sm font-semibold text-gray-700 uppercase tracking-wide mb-3",
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
                onclick: move |_| {
                    spawn({
                        let roadmap_id = roadmap_id.clone();
                        let node_id = node.id.clone();
                        async move {
                            let _ = toggle_node_completion(roadmap_id, node_id).await;
                        }
                    });
                },
                class: format!(
                    "w-full py-3 rounded-lg font-semibold transition {}",
                    if node.is_completed {
                        "bg-gray-200 text-gray-700 hover:bg-gray-300"
                    } else {
                        "bg-indigo-600 text-white hover:bg-indigo-700"
                    },
                ),
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
        div { class: "p-4 bg-white border border-gray-200 rounded-lg hover:shadow-md transition",
            div { class: "flex items-start justify-between mb-2",
                h4 { class: "font-semibold text-gray-900 flex-1", {resource.title.clone()} }
                span { class: "text-xs px-2 py-1 bg-indigo-100 text-indigo-700 rounded",
                    {resource.resource_type.clone()}
                }
            }
            p { class: "text-sm text-gray-600 mb-2", "📚 {resource.platform}" }
            if let Some(url) = &resource.url {
                a {
                    href: "{url}",
                    target: "_blank",
                    class: "text-sm text-indigo-600 hover:text-indigo-700 font-medium",
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
            h2 { class: "text-2xl font-bold text-gray-900 mb-4", "Roadmap Overview" }

            div { class: "mb-6",
                div { class: "flex justify-between text-sm text-gray-600 mb-2",
                    span { "Progress" }
                    span { "{completed}/{total} completed" }
                }
                div { class: "w-full bg-gray-200 rounded-full h-3",
                    div {
                        class: "bg-indigo-600 h-3 rounded-full transition-all",
                        style: "width: {progress}%",
                    }
                }
            }

            div { class: "space-y-4",
                div { class: "p-4 bg-white rounded-lg border border-gray-200",
                    div { class: "text-3xl font-bold text-indigo-600", "{progress}%" }
                    p { class: "text-sm text-gray-600", "Overall completion" }
                }

                div { class: "p-4 bg-white rounded-lg border border-gray-200",
                    div { class: "text-3xl font-bold text-gray-900", "{total}" }
                    p { class: "text-sm text-gray-600", "Total skills to learn" }
                }

                div { class: "p-4 bg-white rounded-lg border border-gray-200",
                    div { class: "text-3xl font-bold text-green-600", "{completed}" }
                    p { class: "text-sm text-gray-600", "Skills completed" }
                }
            }

            div { class: "mt-8",
                h3 { class: "text-sm font-semibold text-gray-700 uppercase tracking-wide mb-3",
                    "Quick Tips"
                }
                ul { class: "space-y-2 text-sm text-gray-600",
                    li { "💡 Click on any node to view details" }
                    li { "✅ Check the circle to mark skills as complete" }
                    li { "📚 Explore resources for each skill" }
                    li { "🎯 Follow the arrows for the recommended path" }
                }
            }
        }
    }
}
