use crate::{utils::get_session_token, Route};
use dioxus::prelude::*;

#[component]
pub fn Landing() -> Element {
    let nav = navigator();

    if get_session_token().is_some() {
        nav.push(Route::Dashboard {});
        return rsx! { "Redirecting to dashboard..." };
    } else {
        rsx! {
            div { class: "min-h-screen bg-[#050505] text-gray-100 font-sans selection:bg-teal-500/30 selection:text-teal-200 overflow-x-hidden relative",
                // Ambient Background Effects
                div { class: "fixed inset-0 pointer-events-none overflow-hidden",
                    div { class: "absolute top-[-10%] left-[-10%] w-[50vw] h-[50vw] bg-teal-500/5 rounded-full blur-[100px] animate-float-slow" }
                    div { class: "absolute bottom-[-10%] right-[-10%] w-[50vw] h-[50vw] bg-blue-600/5 rounded-full blur-[100px] animate-float-slow delay-2000" }
                    div { class: "absolute top-[20%] right-[20%] w-[30vw] h-[30vw] bg-emerald-500/5 rounded-full blur-[80px] animate-pulse-slow" }
                }

                // Grid Pattern Overlay
                div { class: "fixed inset-0 bg-[linear-gradient(rgba(20,184,166,0.03)_1px,transparent_1px),linear-gradient(90deg,rgba(20,184,166,0.03)_1px,transparent_1px)] bg-[size:4rem_4rem] [mask-image:radial-gradient(ellipse_60%_50%_at_50%_0%,#000_70%,transparent_100%)] pointer-events-none" }

                nav { class: "container mx-auto px-6 py-6 flex justify-between items-center relative z-50",
                    h1 { class: "text-3xl font-bold tracking-tight",
                        span { class: "bg-gradient-to-r from-teal-400 to-blue-500 bg-clip-text text-transparent animate-gradient-text",
                            "Skill"
                        }
                        span { class: "text-gray-100", "Forge" }
                    }

                    div { class: "space-x-4 flex items-center",
                        // Login Button: Premium Glass Variant
                        Link {
                            to: Route::Login {},
                            class: "group relative px-6 py-2 rounded-lg font-medium transition-all duration-300 no-underline overflow-hidden",
                            // Background Layer (Idle: Faint Glass | Hover: Teal Tint)
                            div { class: "absolute inset-0 bg-white/5 border border-white/10 group-hover:bg-teal-500/20 group-hover:border-teal-500/50 transition-all duration-300 rounded-lg" }
                            // Glow Layer (Hover only)
                            div { class: "absolute inset-0 opacity-0 group-hover:opacity-100 shadow-[0_0_20px_rgba(20,184,166,0.3)] transition-opacity duration-300 rounded-lg" }
                            // Text Layer
                            span { class: "relative z-10 text-gray-300 group-hover:text-white transition-colors duration-300",
                                "Login"
                            }
                        }

                        // Sign Up Button
                        Link {
                            to: Route::Signup {},
                            class: "group relative px-6 py-2 bg-gradient-to-r from-teal-500 to-blue-600 text-white rounded-lg overflow-hidden transition-all duration-300 hover:shadow-[0_0_25px_rgba(20,184,166,0.4)] hover:-translate-y-0.5 no-underline",
                            span { class: "relative z-10 font-medium", "Sign Up" }
                            div { class: "absolute inset-0 bg-gradient-to-r from-blue-600 to-teal-500 opacity-0 group-hover:opacity-100 transition-opacity duration-300" }
                        }
                    }
                }

                main { class: "container mx-auto px-6 py-24 text-center relative z-10",
                    div { class: "animate-slide-up",
                        h2 { class: "text-5xl md:text-7xl font-bold mb-8 tracking-tight",
                            span { class: "block text-gray-100 mb-2 drop-shadow-lg",
                                "Your Personalized"
                            }
                            span { class: "bg-gradient-to-r from-teal-400 via-blue-400 to-teal-400 bg-clip-text text-transparent bg-300% animate-gradient-text",
                                "Learning Journey"
                            }
                        }

                        p { class: "text-xl text-gray-400 mb-12 max-w-2xl mx-auto leading-relaxed",
                            "AI-powered roadmaps tailored to your skills, preferences, and goals. Learn smarter, not harder."
                        }
                    }

                    div { class: "grid md:grid-cols-3 gap-8 mt-20 max-w-6xl mx-auto perspective-1000",
                        // Feature 1: Assessment
                        div { class: "group p-8 rounded-2xl bg-[#0f1012]/80 border border-gray-800/50 backdrop-blur-md hover:bg-gray-800/80 transition-all duration-500 hover:-translate-y-2 hover:border-teal-500/30 hover:shadow-[0_0_50px_rgba(20,184,166,0.1)]",
                            div { class: "w-16 h-16 mx-auto mb-6 bg-gradient-to-br from-gray-800 to-gray-900 rounded-2xl flex items-center justify-center text-3xl group-hover:scale-110 group-hover:rotate-3 transition-transform duration-500 shadow-inner shadow-teal-500/10 border border-gray-700/50",
                                "🎯"
                            }
                            h3 { class: "text-xl font-bold mb-4 text-gray-100 group-hover:text-teal-400 transition-colors",
                                "Personalized Assessment"
                            }
                            p { class: "text-gray-400 leading-relaxed group-hover:text-gray-300 transition-colors",
                                "Answer targeted questions to help AI understand your learning style and existing knowledge"
                            }
                        }

                        // Feature 2: Roadmaps
                        div { class: "group p-8 rounded-2xl bg-[#0f1012]/80 border border-gray-800/50 backdrop-blur-md hover:bg-gray-800/80 transition-all duration-500 hover:-translate-y-2 hover:border-blue-500/30 hover:shadow-[0_0_50px_rgba(59,130,246,0.1)] delay-100",
                            div { class: "w-16 h-16 mx-auto mb-6 bg-gradient-to-br from-gray-800 to-gray-900 rounded-2xl flex items-center justify-center text-3xl group-hover:scale-110 group-hover:rotate-3 transition-transform duration-500 shadow-inner shadow-blue-500/10 border border-gray-700/50",
                                "🗺️"
                            }
                            h3 { class: "text-xl font-bold mb-4 text-gray-100 group-hover:text-blue-400 transition-colors",
                                "Visual Roadmaps"
                            }
                            p { class: "text-gray-400 leading-relaxed group-hover:text-gray-300 transition-colors",
                                "Get interactive, node-based roadmaps with curated resources and clear prerequisites"
                            }
                        }

                        // Feature 3: Track Progress
                        div { class: "group p-8 rounded-2xl bg-[#0f1012]/80 border border-gray-800/50 backdrop-blur-md hover:bg-gray-800/80 transition-all duration-500 hover:-translate-y-2 hover:border-teal-500/30 hover:shadow-[0_0_50px_rgba(20,184,166,0.1)] delay-200",
                            div { class: "w-16 h-16 mx-auto mb-6 bg-gradient-to-br from-gray-800 to-gray-900 rounded-2xl flex items-center justify-center text-3xl group-hover:scale-110 group-hover:rotate-3 transition-transform duration-500 shadow-inner shadow-teal-500/10 border border-gray-700/50",
                                "📊"
                            }
                            h3 { class: "text-xl font-bold mb-4 text-gray-100 group-hover:text-teal-400 transition-colors",
                                "Track Progress"
                            }
                            p { class: "text-gray-400 leading-relaxed group-hover:text-gray-300 transition-colors",
                                "Mark skills as completed and watch your learning journey unfold"
                            }
                        }
                    }
                }
            }
        }
    }
}
