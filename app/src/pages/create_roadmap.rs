use crate::utils::get_session_token;
use crate::{
    models::{Question, QuestionResponse, QuestionType},
    server_functions::{generate_questions, generate_roadmap},
    Route,
};
use dioxus::prelude::*;

#[derive(Clone, PartialEq)]
enum FlowStep {
    SkillInput,
    Questions,
    Generating,
    Complete(String),
}

#[component]
pub fn CreateRoadmap() -> Element {
    let mut step = use_signal(|| FlowStep::SkillInput);
    let skill_name = use_signal(String::new); // Removed mut
    let mut questions = use_signal(Vec::<Question>::new);
    let mut current_question_idx = use_signal(|| 0);
    let mut responses = use_signal(Vec::<QuestionResponse>::new);
    let mut current_answer = use_signal(Vec::<String>::new);
    let mut error = use_signal(|| None::<String>);

    let nav = navigator();
    let token = get_session_token();
    if token.is_none() {
        nav.push(Route::Login {});
        return rsx! { "Redirecting..." };
    }

    let session_token_for_roadmap = token.unwrap();

    let session_token_for_questions = session_token_for_roadmap.clone();

    let load_questions = move |_| {
        let skill = skill_name();
        let session_token = session_token_for_questions.clone();
        if skill.trim().is_empty() {
            error.set(Some("Please enter a skill name".to_string()));
            return;
        }
        spawn(async move {
            match generate_questions(skill.clone(), session_token).await {
                Ok(qs) => {
                    questions.set(qs);
                    current_question_idx.set(0);
                    responses.set(Vec::new());
                    step.set(FlowStep::Questions);
                    error.set(None);
                }
                Err(e) => {
                    error.set(Some(format!("Failed to generate questions: {}", e)));
                }
            }
        });
    };

    let submit_answer = move |_| {
        let question = &questions()[current_question_idx()];
        let answer = current_answer();

        if answer.is_empty() {
            error.set(Some("Please provide an answer".to_string()));
            return;
        }

        let mut all_responses = responses();
        all_responses.push(QuestionResponse {
            question_id: question.id.clone(),
            answer: answer.clone(),
        });
        responses.set(all_responses.clone());

        current_answer.set(Vec::new());
        error.set(None);

        if current_question_idx() + 1 < questions().len() {
            current_question_idx.set(current_question_idx() + 1);
        } else {
            step.set(FlowStep::Generating);
            let skill = skill_name();

            let session_token = session_token_for_roadmap.clone();
            spawn(async move {
                match generate_roadmap(skill, session_token, all_responses).await {
                    Ok(roadmap_id) => {
                        step.set(FlowStep::Complete(roadmap_id));
                    }
                    Err(e) => {
                        error.set(Some(format!("Failed to generate roadmap: {}", e)));
                        step.set(FlowStep::Questions);
                    }
                }
            });
        }
    };

    let go_back = move |_| {
        if current_question_idx() > 0 {
            current_question_idx.set(current_question_idx() - 1);
            let mut all_responses = responses();
            if !all_responses.is_empty() {
                all_responses.pop();
                responses.set(all_responses);
            }
        }
    };

    rsx! {
        div { class: "min-h-screen bg-gradient-to-br from-indigo-50 to-purple-50",
            // Navigation
            nav { class: "bg-white shadow-sm",
                div { class: "container mx-auto px-6 py-4",
                    Link {
                        to: Route::Dashboard {},
                        class: "text-indigo-600 hover:text-indigo-700 font-medium flex items-center",
                        "← Back to Dashboard"
                    }
                }
            }

            main { class: "container mx-auto px-6 py-12 max-w-3xl",
                match step() {
                    FlowStep::SkillInput => rsx! {
                        SkillInputStep { skill_name, error, on_continue: load_questions }
                    },
                    FlowStep::Questions => rsx! {
                        QuestionStep {
                            question: questions()[current_question_idx()].clone(),
                            question_number: current_question_idx() + 1,
                            total_questions: questions().len(),
                            current_answer,
                            error,
                            on_submit: submit_answer,
                            on_back: go_back,
                            show_back: current_question_idx() > 0,
                        }
                    },
                    FlowStep::Generating => rsx! {
                        GeneratingStep {}
                    },
                    FlowStep::Complete(roadmap_id) => rsx! {
                        CompleteStep { roadmap_id }
                    },
                }
            }
        }
    }
}

#[component]
fn SkillInputStep(
    skill_name: Signal<String>,
    error: Signal<Option<String>>,
    on_continue: EventHandler<()>,
) -> Element {
    let mut is_loading = use_signal(|| false);
    rsx! {

        div { class: "bg-white rounded-2xl shadow-xl p-8",
            h2 { class: "text-3xl font-bold text-gray-900 mb-2", "What do you want to learn?" }
            p { class: "text-gray-600 mb-8",
                "Enter a skill or topic you'd like to master, and we'll create a personalized learning roadmap for you."
            }

            if let Some(err) = error() {
                div { class: "mb-6 p-4 bg-red-50 text-red-700 rounded-lg", {err} }
            }

            input {
                r#type: "text",
                // Disable input while loading to prevent changes during fetch
                disabled: is_loading(),
                class: "w-full px-6 py-4 text-lg border-2 border-gray-300 rounded-xl focus:ring-2 focus:ring-indigo-500 focus:border-transparent outline-none transition disabled:bg-gray-100 disabled:text-gray-500",
                value: "{skill_name}",
                oninput: move |e| skill_name.set(e.value()),
                placeholder: "e.g., Machine Learning, Web Development, Python...",
                autofocus: true,
                onkeypress: move |e| {
                    if e.key() == Key::Enter && !is_loading() {
                        is_loading.set(true);
                        on_continue.call(());
                    }
                },
            }

            button {
                // 2. Disable the button via HTML attribute when loading
                disabled: is_loading(),
                onclick: move |_| {
                    // 3. Set loading to true and trigger callback
                    is_loading.set(true);
                    on_continue.call(());
                },
                // 4. Added flex utilities for centering the spinner and disabled styles
                class: "mt-6 w-full py-4 bg-indigo-600 text-white rounded-xl hover:bg-indigo-700 transition font-semibold text-lg flex justify-center items-center disabled:opacity-70 disabled:cursor-not-allowed",

                // 5. Conditionally render the content
                if is_loading() {
                    // Standard Tailwind Spin SVG
                    svg {
                        class: "animate-spin -ml-1 mr-3 h-5 w-5 text-white",
                        xmlns: "http://www.w3.org/2000/svg",
                        fill: "none",
                        view_box: "0 0 24 24",
                        circle {
                            class: "opacity-25",
                            cx: "12",
                            cy: "12",
                            r: "10",
                            stroke: "currentColor",
                            stroke_width: "4",
                        }
                        path {
                            class: "opacity-75",
                            fill: "currentColor",
                            d: "M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z",
                        }
                    }
                    "Generating..."
                } else {
                    "Continue →"
                }
            }
        }
    }
}

#[component]
fn QuestionStep(
    question: Question,
    question_number: usize,
    total_questions: usize,
    current_answer: Signal<Vec<String>>,
    error: Signal<Option<String>>,
    on_submit: EventHandler<()>,
    on_back: EventHandler<()>,
    show_back: bool,
) -> Element {
    let is_msq = question.question_type == QuestionType::MSQ;

    let mut toggle_option = move |option: String| {
        let mut answers = current_answer();
        if let Some(pos) = answers.iter().position(|x| x == &option) {
            answers.remove(pos);
        } else {
            if !is_msq {
                answers.clear();
            }
            answers.push(option);
        }
        current_answer.set(answers);
    };

    rsx! {
        div { class: "bg-white rounded-2xl shadow-xl p-8",
            // Progress bar
            div { class: "mb-8",
                div { class: "flex justify-between text-sm text-gray-600 mb-2",
                    span { "Question {question_number} of {total_questions}" }
                    span { "{((question_number * 100) / total_questions)}%" }
                }
                div { class: "w-full bg-gray-200 rounded-full h-2",
                    div {
                        class: "bg-indigo-600 h-2 rounded-full transition-all duration-300",
                        style: "width: {((question_number * 100) / total_questions)}%",
                    }
                }
            }

            h3 { class: "text-2xl font-bold text-gray-900 mb-2", {question.question_text} }

            div { class: "mb-6 inline-block px-3 py-1 bg-indigo-100 text-indigo-700 rounded-full text-sm font-medium",
                match question.question_type {
                    QuestionType::MCQ => "Single Choice",
                    QuestionType::MSQ => "Multiple Choice",
                    QuestionType::TrueFalse => "True or False",
                    QuestionType::OneWord => "Short Answer",
                }
            }

            if let Some(err) = error() {
                div { class: "mb-4 p-3 bg-red-50 text-red-700 rounded-lg text-sm", {err} }
            }

            div { class: "space-y-3 mb-8",
                match question.question_type {
                    QuestionType::OneWord => rsx! {
                        input {
                            r#type: "text",
                            class: "w-full px-4 py-3 border-2 border-gray-300 rounded-lg focus:ring-2 focus:ring-indigo-500 focus:border-transparent outline-none",
                            placeholder: "Type your answer...",
                            value: "{current_answer().first().unwrap_or(&String::new())}",
                            oninput: move |e| current_answer.set(vec![e.value()]),
                            onkeypress: move |e| {
                                if e.key() == Key::Enter {
                                    on_submit.call(());
                                }
                            },
                        }
                    },
                    _ => rsx! {
                        for option in &question.options {
                            {
                                let option_clone = option.clone();
                                let is_selected = current_answer().contains(&option_clone);
                                rsx! {
                                    button {
                                        key: "{option}",
                                        onclick: move |_| toggle_option(option_clone.clone()),
                                        class: format!(
                                            "w-full p-4 text-left border-2 rounded-lg transition {}",
                                            if is_selected {
                                                "border-indigo-500 bg-indigo-50 text-indigo-900"
                                            } else {
                                                "border-gray-300 hover:border-indigo-300 hover:bg-gray-50"
                                            },
                                        ),
                                        div { class: "flex items-center",
                                            div {
                                                class: format!(
                                                    "w-5 h-5 rounded-full border-2 mr-3 flex items-center justify-center {}",
                                                    if is_selected { "border-indigo-500 bg-indigo-500" } else { "border-gray-400" },
                                                ),
                                                if is_selected {
                                                    svg {
                                                        class: "w-3 h-3 text-white",
                                                        fill: "currentColor",
                                                        view_box: "0 0 20 20",
                                                        path { d: "M16.707 5.293a1 1 0 010 1.414l-8 8a1 1 0 01-1.414 0l-4-4a1 1 0 011.414-1.414L8 12.586l7.293-7.293a1 1 0 011.414 0z" }
                                                    }
                                                }
                                            }
                                            span { class: "font-medium", {option.to_string()} }
                                        }
                                    }
                                }
                            }
                        }
                    },
                }
            }

            div { class: "flex gap-4",
                if show_back {
                    button {
                        onclick: move |_| on_back.call(()),
                        class: "px-6 py-3 border-2 border-gray-300 text-gray-700 rounded-lg hover:bg-gray-50 transition font-medium",
                        "← Back"
                    }
                }
                button {
                    onclick: move |_| on_submit.call(()),
                    class: "flex-1 py-3 bg-indigo-600 text-white rounded-lg hover:bg-indigo-700 transition font-semibold",
                    if question_number == total_questions {
                        "Generate Roadmap 🎯"
                    } else {
                        "Next →"
                    }
                }
            }
        }
    }
}

#[component]
fn GeneratingStep() -> Element {
    rsx! {
        div { class: "bg-white rounded-2xl shadow-xl p-12 text-center",
            div { class: "mb-6",
                svg {
                    class: "animate-spin h-16 w-16 mx-auto text-indigo-600",
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
            }
            h3 { class: "text-2xl font-bold text-gray-900 mb-3", "Crafting Your Roadmap..." }
            p { class: "text-gray-600 mb-6",
                "Our AI is analyzing thousands of resources to create the perfect learning path for you."
            }
            div { class: "space-y-2 text-sm text-gray-500",
                p { "✓ Analyzing your responses" }
                p { "✓ Matching with relevant resources" }
                p { "✓ Organizing learning path" }
            }
        }
    }
}

#[component]
fn CompleteStep(roadmap_id: String) -> Element {
    let nav = navigator();

    rsx! {
        div { class: "bg-white rounded-2xl shadow-xl p-12 text-center",
            div { class: "text-6xl mb-6", "🎉" }
            h3 { class: "text-3xl font-bold text-gray-900 mb-3", "Your Roadmap is Ready!" }
            p { class: "text-gray-600 mb-8",
                "We've created a personalized learning path tailored to your goals and existing knowledge."
            }
            button {
                onclick: move |_| {
                    nav.push(Route::RoadmapView {
                        id: roadmap_id.clone(),
                    });
                },
                class: "px-8 py-4 bg-indigo-600 text-white rounded-xl hover:bg-indigo-700 transition font-semibold text-lg",
                "View Your Roadmap →"
            }
        }
    }
}
