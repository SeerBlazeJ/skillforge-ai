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

    // Animation triggers
    let mut animate_cards = use_signal(|| false);
    let mut animate_line = use_signal(|| false);

    use_effect(move || {
        // Trigger animations in sequence
        spawn(async move {
            // Phase 1: Arrange cards
            gloo_timers::future::TimeoutFuture::new(100).await;
            animate_cards.set(true);

            // Phase 2: Draw the connecting line
            gloo_timers::future::TimeoutFuture::new(800).await;
            animate_line.set(true);
        });
    });

    return rsx! {
        div { class: "min-h-screen bg-[#050505] text-gray-100 font-sans selection:bg-teal-500/30 selection:text-teal-200 overflow-x-hidden",
            // Top nav
            nav { class: "bg-[#050505]/80 backdrop-blur-md border-b border-white/5 sticky top-0 z-40",
                div { class: "container mx-auto px-6 py-4 flex justify-between items-center",
                    Link {
                        to: Route::Dashboard {},
                        class: "text-teal-400 hover:text-teal-300 transition-colors font-medium flex items-center gap-2",
                        span { "←" }
                        "Back to Dashboard"
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

                    // Sidebar logic remains the same
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
                        div { class: "flex h-[calc(100vh-72px)] relative",
                            // Main timeline area
                            div { class: "flex-1 overflow-y-auto custom-scroll bg-[#050505] relative",
                                div { class: "max-w-5xl mx-auto px-6 py-12 pb-32", // 1. The Background Track (Always visible but dim)

                                    // Header Area
                                    div {
                                        class: format!(
                                            "flex items-center justify-between mb-16 transition-all duration-700 transform {}",
                                            if animate_cards() {
                                                "opacity-100 translate-y-0"
                                            } else {
                                                "opacity-0 -translate-y-4"
                                            },
                                        ),
                                        h2 { class: "text-2xl font-bold text-gray-100 flex items-center gap-3",
                                            span { class: "text-teal-500", "◈" }
                                            "Learning Path"
                                        }
                                        RoadmapProgressPill { roadmap: roadmap_data.clone() }
                                    }

                                    // Timeline Container
                                    div { class: "relative",

                                        // 1. The Background Track (Always visible but dim)
                                        div { class: "absolute left-6 md:left-1/2 top-8 bottom-0 w-0.5 bg-white/5 -translate-x-1/2 rounded-full" } // Alternating layout  Alternating layout // Alternating layout  Alternating layout  Alternating layout  Alternating layout

                                        // 2. The Animated Connecting Line (Expands height)
                                        div {
                                            class: "absolute left-6 md:left-1/2 top-8 w-0.5 bg-gradient-to-b from-teal-500 via-blue-500 to-teal-500 -translate-x-1/2 rounded-full shadow-[0_0_12px_rgba(20,184,166,0.6)] transition-all duration-[1500ms] ease-out",
                                            style: format!("height: {}", if animate_line() { "calc(100% - 2rem)" } else { "0%" }),
                                        }

                                        // 3. The Nodes
                                        div { class: "space-y-12 md:space-y-0",
                                            for (idx , node) in ordered.into_iter().enumerate() {
                                                {
                                                    let node_id = node.id.clone();
                                                    let is_selected = selected_node_id().as_deref() == Some(&node_id);
                                                    let is_left = idx % 2 == 0;
                                                    rsx! {
                                                        TimelineNode {
                                                            key: "step-{node_id}",
                                                            idx: idx + 1,
                                                            node,
                                                            is_left,
                                                            is_selected,
                                                            show: animate_cards(),
                                                            on_select: move |_| selected_node_id.set(Some(node_id.clone())),
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }

                            // Sidebar (Fixed width)
                            div { class: "w-[28rem] bg-[#0b0c0e] border-l border-white/10 overflow-y-auto custom-scroll shadow-2xl z-20",
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
                    div { class: "flex items-center justify-center h-screen",
                        div { class: "animate-pulse flex flex-col items-center gap-4",
                            div { class: "w-12 h-12 border-4 border-teal-500/30 border-t-teal-500 rounded-full animate-spin" }
                            div { class: "text-gray-500 font-medium", "Loading roadmap..." }
                        }
                    }
                },
            }
        }
    };
}
#[component]
fn TimelineNode(
    idx: usize,
    node: RoadmapNode,
    is_left: bool,
    is_selected: bool,
    show: bool,
    on_select: EventHandler<()>,
) -> Element {
    // Calculate delays based on index for the "arrangement" phase
    let delay = idx * 100;

    // Status Styles
    let (dot_color, dot_glow) = if node.is_completed {
        ("bg-green-500", "shadow-[0_0_15px_rgba(34,197,94,0.6)]")
    } else if is_selected {
        (
            "bg-teal-400",
            "shadow-[0_0_20px_rgba(45,212,191,0.8)] scale-125",
        )
    } else {
        ("bg-[#1a1b1e] border-2 border-white/20", "shadow-none")
    };

    let container_alignment = if is_left {
        "md:flex-row-reverse"
    } else {
        "md:flex-row"
    };
    let text_alignment = if is_left {
        "md:text-right md:items-end"
    } else {
        "md:text-left md:items-start"
    };
    let arrow_alignment = if is_left { "md:-right-2" } else { "md:-left-2" };

    // FIX: Extract logic here to satisfy the parser
    let card_classes = format!(
        "group relative p-5 rounded-xl border backdrop-blur-sm transition-all duration-300 cursor-pointer hover:-translate-y-1 flex flex-col {} {}",
        if is_selected {
            "bg-teal-500/10 border-teal-500/50 shadow-[0_0_30px_rgba(20,184,166,0.1)]"
        } else {
            "bg-[#0f1012]/80 border-white/10 hover:border-teal-500/30 hover:shadow-lg"
        },
        text_alignment
    );

    rsx! {
        div {
            class: format!(
                "relative flex items-center md:justify-between mb-8 md:mb-0 transition-all duration-700 ease-out transform {}",
                container_alignment,
            ),
            style: format!(
                "transition-delay: {}ms; opacity: {}; transform: {}",
                delay,
                if show { 1 } else { 0 },
                if show { "translateY(0)" } else { "translateY(20px)" },
            ),

            // 1. Empty Spacer for the other side (Desktop only)
            div { class: "hidden md:block md:w-5/12" }

            // 2. Center Connector Dot
            div { class: "absolute left-6 md:left-1/2 -translate-x-1/2 z-10 flex items-center justify-center w-8 h-8",
                div {
                    class: format!(
                        "w-4 h-4 rounded-full transition-all duration-500 {} {}",
                        dot_color,
                        dot_glow,
                    ),
                }
            }

            // 3. Content Card
            div {
                class: "pl-16 md:pl-0 md:w-5/12 w-full",
                onclick: move |_| on_select.call(()),

                div { class: "{card_classes}", // Uses the pre-formatted variable

                    // Decorative tiny arrow pointing to the line
                    div {
                        class: format!(
                            "absolute top-1/2 -translate-y-1/2 w-3 h-3 bg-inherit border-inherit border-b border-l rotate-45 hidden md:block {}",
                            arrow_alignment,
                        ),
                        style: "border-right: 0; border-top: 0;",
                    }

                    div { class: "flex items-center gap-2 mb-2 opacity-60 text-xs font-mono tracking-wider",
                        span { class: "text-teal-400", "0{idx}" }
                        span { "—" }
                        span {
                            if node.is_completed {
                                "COMPLETED"
                            } else {
                                "PENDING"
                            }
                        }
                    }

                    h3 { class: "text-lg font-bold text-gray-100 mb-2 group-hover:text-teal-300 transition-colors",
                        "{node.skill_name}"
                    }

                    p { class: "text-sm text-gray-400 leading-relaxed line-clamp-2 mb-3",
                        "{node.description}"
                    }

                    // Resources pill
                    if !node.resources.is_empty() {
                        div { class: "inline-flex items-center gap-2 px-2 py-1 rounded bg-white/5 border border-white/5 text-xs text-gray-400",
                            span { "📚" }
                            "{node.resources.len()} Resources"
                        }
                    }
                }
            }
        }
    }
}

// Helpers & Sidebar Components (Kept mostly similar but cleaned up)
fn ordered_nodes(roadmap: &Roadmap) -> Vec<RoadmapNode> {
    let by_id: HashMap<String, RoadmapNode> = roadmap
        .nodes
        .iter()
        .cloned()
        .map(|n| (n.id.clone(), n))
        .collect();

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
        div { class: "hidden sm:flex items-center gap-3 bg-[#0f1012] border border-white/10 px-4 py-2 rounded-full",
            div { class: "flex items-center gap-2",
                div { class: "w-16 h-1.5 bg-white/10 rounded-full overflow-hidden",
                    div {
                        class: "h-full bg-teal-500 rounded-full",
                        style: "width: {progress}%",
                    }
                }
                span { class: "text-sm font-medium text-teal-400", "{progress}%" }
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

    // extracted button class to keep rsx clean
    let button_class = if node.is_completed {
        "w-full py-3.5 rounded-lg font-bold text-sm transition-all duration-300 transform active:scale-[0.98] bg-[#1a1b1e] text-gray-400 border border-white/10 hover:bg-white/5 hover:text-white"
    } else {
        "w-full py-3.5 rounded-lg font-bold text-sm transition-all duration-300 transform active:scale-[0.98] bg-gradient-to-r from-teal-600 to-blue-600 text-white shadow-lg shadow-teal-900/20 hover:shadow-teal-500/20 hover:brightness-110"
    };

    rsx! {
        div { class: "p-8 h-full flex flex-col",
            // Header
            div { class: "flex justify-between items-start mb-8 shrink-0",
                div {
                    h2 { class: "text-2xl font-bold text-gray-100 leading-tight mb-2",
                        "{node.skill_name.clone()}"
                    }
                    div {
                        class: format!(
                            "inline-flex items-center gap-1.5 px-2.5 py-0.5 rounded text-xs font-semibold tracking-wide uppercase {}",
                            if node.is_completed {
                                "bg-green-500/20 text-green-400"
                            } else {
                                "bg-yellow-500/10 text-yellow-500"
                            },
                        ),
                        if node.is_completed {
                            "Completed"
                        } else {
                            "In Progress"
                        }
                    }
                }
                button {
                    onclick: move |_| on_close.call(()),
                    class: "p-2 rounded hover:bg-white/10 text-gray-500 hover:text-white transition",
                    "✕"
                }
            }

            // Scrollable Content
            div { class: "flex-1 overflow-y-auto custom-scroll pr-2 space-y-8",

                // Navigation
                if prev_label.is_some() || next_label.is_some() {
                    div { class: "grid grid-cols-2 gap-3",
                        if let Some(prev) = prev_label {
                            button {
                                class: "group p-3 bg-white/5 border border-white/10 rounded-lg text-left hover:bg-white/10 hover:border-white/20 transition",
                                onclick: move |_| {
                                    if let Some(id) = node.prev_node_id.clone() {
                                        selected_node_id.set(Some(id))
                                    }
                                },
                                div { class: "text-[10px] text-gray-500 font-bold uppercase tracking-wider mb-1",
                                    "PREVIOUS"
                                }
                                div { class: "text-sm text-gray-200 font-medium truncate group-hover:text-teal-400 transition-colors",
                                    "{prev}"
                                }
                            }
                        }
                        if let Some(next) = next_label {
                            button {
                                class: "group p-3 bg-white/5 border border-white/10 rounded-lg text-left hover:bg-white/10 hover:border-white/20 transition",
                                onclick: move |_| {
                                    if let Some(id) = node.next_node_id.clone() {
                                        selected_node_id.set(Some(id))
                                    }
                                },
                                div { class: "text-[10px] text-gray-500 font-bold uppercase tracking-wider mb-1",
                                    "NEXT"
                                }
                                div { class: "text-sm text-gray-200 font-medium truncate group-hover:text-teal-400 transition-colors",
                                    "{next}"
                                }
                            }
                        }
                    }
                }

                // Description
                div {
                    h3 { class: "text-xs font-bold text-gray-500 uppercase tracking-widest mb-3 flex items-center gap-2",
                        "ABOUT THIS SKILL"
                    }
                    p { class: "text-gray-300 leading-7 text-sm whitespace-pre-line",
                        "{node.description.clone()}"
                    }
                }

                // Prerequisites
                if !node.prerequisites.is_empty() {
                    div {
                        h3 { class: "text-xs font-bold text-gray-500 uppercase tracking-widest mb-3",
                            "PREREQUISITES"
                        }
                        div { class: "flex flex-wrap gap-2",
                            for prereq in &node.prerequisites {
                                {
                                    let id = prereq.clone();
                                    let label = label_for_ref(&roadmap, prereq);
                                    let exists = roadmap.nodes.iter().any(|n| n.id == prereq.as_str());

                                    // FIX: Extracted logic to variable to prevent parser errors
                                    let badge_class = format!(
                                        "text-xs px-3 py-1.5 rounded-md border transition-colors {}",
                                        if exists {
                                            "bg-white/5 border-white/10 text-gray-300 hover:border-teal-500/50 cursor-pointer"
                                        } else {
                                            "bg-red-500/5 border-red-500/10 text-red-400/70"
                                        },
                                    );

                                    rsx! {
                                        span {
                                            class: "{badge_class}",
                                            onclick: move |_| {
                                                if exists {
                                                    selected_node_id.set(Some(id.clone()))
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

                // Resources
                if !node.resources.is_empty() {
                    div {
                        h3 { class: "text-xs font-bold text-gray-500 uppercase tracking-widest mb-3",
                            "LEARNING RESOURCES"
                        }
                        div { class: "space-y-3",
                            for resource in &node.resources {
                                ResourceCard { resource: resource.clone() }
                            }
                        }
                    }
                }
            }

            // Footer Action
            div { class: "mt-6 pt-6 border-t border-white/10 shrink-0",
                button {
                    class: "{button_class}",
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
                        "Complete Skill"
                    }
                }
            }
        }
    }
}

#[component]
fn ResourceCard(resource: LearningResource) -> Element {
    rsx! {
        div { class: "group p-4 bg-[#0f1012] border border-white/5 rounded-lg hover:border-teal-500/30 transition-colors",
            div { class: "flex items-start justify-between mb-1",
                span { class: "text-[10px] font-bold text-teal-500 uppercase tracking-wide",
                    "{resource.resource_type}"
                }
            }
            h4 { class: "font-medium text-gray-200 text-sm mb-1 group-hover:text-teal-300 transition-colors",
                "{resource.title}"
            }
            p { class: "text-xs text-gray-500 mb-3", "{resource.platform}" }
            if let Some(url) = &resource.url {
                a {
                    href: "{url}",
                    target: "_blank",
                    class: "inline-flex items-center text-xs font-medium text-teal-400 hover:text-teal-300 transition-colors",
                    "Open Link "
                    span { class: "ml-1 text-[10px]", "↗" }
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
        div { class: "h-full flex flex-col",
            // Sticky Header Section (Progress Circle)
            div { class: "p-8 pb-6 shrink-0 flex flex-col items-center text-center border-b border-white/5 bg-[#0b0c0e]",
                div { class: "relative w-32 h-32 mb-6",
                    // Circular Progress SVG
                    svg {
                        class: "w-full h-full -rotate-90",
                        view_box: "0 0 100 100",
                        circle {
                            class: "text-[#1a1b1e] stroke-current",
                            stroke_width: "8",
                            cx: "50",
                            cy: "50",
                            r: "40",
                            fill: "none",
                        }
                        circle {
                            class: "text-teal-500 stroke-current transition-all duration-1000 ease-out",
                            stroke_width: "8",
                            stroke_linecap: "round",
                            cx: "50",
                            cy: "50",
                            r: "40",
                            fill: "none",
                            stroke_dasharray: "251.2",
                            stroke_dashoffset: format!("{}", 251.2 - (251.2 * progress as f32 / 100.0)),
                        }
                    }
                    div { class: "absolute inset-0 flex flex-col items-center justify-center",
                        span { class: "text-2xl font-bold text-white", "{progress}%" }
                    }
                }

                h2 { class: "text-xl font-bold text-gray-100 mb-2", "Roadmap Overview" }

                // Stats Grid
                div { class: "grid grid-cols-2 gap-3 w-full mt-4",
                    div { class: "p-3 bg-[#1a1b1e] rounded-lg border border-white/5",
                        div { class: "text-xl font-bold text-teal-400", "{completed}" }
                        div { class: "text-[10px] text-gray-500 uppercase tracking-wider",
                            "Completed"
                        }
                    }
                    div { class: "p-3 bg-[#1a1b1e] rounded-lg border border-white/5",
                        div { class: "text-xl font-bold text-gray-300", "{total - completed}" }
                        div { class: "text-[10px] text-gray-500 uppercase tracking-wider",
                            "Remaining"
                        }
                    }
                }
            }

            // Scrollable Details Section
            div { class: "flex-1 overflow-y-auto custom-scroll p-8 space-y-8",

                // Learning Outcomes Section
                if !roadmap.learning_outcomes.is_empty() {
                    div {
                        h3 { class: "text-xs font-bold text-gray-500 uppercase tracking-widest mb-4 flex items-center gap-2",
                            span { "🎯" }
                            "Learning Outcomes"
                        }
                        ul { class: "space-y-3",
                            for outcome in &roadmap.learning_outcomes {
                                li { class: "flex items-start gap-3 text-sm text-gray-300 leading-relaxed",
                                    span { class: "mt-1.5 w-1.5 h-1.5 rounded-full bg-teal-500 shadow-[0_0_8px_rgba(20,184,166,0.5)] shrink-0" }
                                    "{outcome}"
                                }
                            }
                        }
                    }
                }

                // Previously Known Skills Section
                if !roadmap.skills_prev_known.is_empty() {
                    div {
                        h3 { class: "text-xs font-bold text-gray-500 uppercase tracking-widest mb-4 flex items-center gap-2",
                            span { "✓" }
                            "Prerequisites Met"
                        }
                        div { class: "flex flex-wrap gap-2",
                            for skill in &roadmap.skills_prev_known {
                                span { class: "px-3 py-1.5 rounded text-xs font-medium bg-[#1a1b1e] border border-white/10 text-gray-400 hover:text-gray-200 hover:border-teal-500/30 transition-colors cursor-default",
                                    "{skill}"
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
