use crate::{
    models::{User, UserPreferences, UserSkills},
    server_functions::{change_password, get_user_data, update_user_profile},
    utils::get_session_token,
    Route,
};
use chrono::Utc;
use dioxus::prelude::*;

#[derive(Clone, PartialEq)]
enum ProfileTab {
    General,
    Skills,
    Preferences,
    Security,
}

#[component]
pub fn Profile() -> Element {
    let mut active_tab = use_signal(|| ProfileTab::General);
    let nav = navigator();
    let token = get_session_token();

    if token.is_none() {
        nav.push(Route::Login {});
        return rsx! { "Redirecting..." };
    }

    let session_token = token.unwrap();
    let user_data = use_resource(move || {
        let session_token = session_token.clone();
        async move { get_user_data(session_token).await }
    });

    rsx! {
        div { class: "min-h-screen bg-[#050505] text-gray-100 font-sans selection:bg-teal-500/30 selection:text-teal-200",
            // Navigation
            nav { class: "bg-[#050505]/80 backdrop-blur-md border-b border-white/5",
                div { class: "container mx-auto px-6 py-4 flex justify-between items-center",
                    Link {
                        to: Route::Dashboard {},
                        class: "text-teal-400 hover:text-indigo-700 font-medium",
                        "← Back to Dashboard"
                    }
                    h1 { class: "text-xl font-bold text-gray-100", "Profile Settings" }
                }
            }

            main { class: "container mx-auto px-6 py-8 max-w-5xl",
                match user_data.read_unchecked().as_ref() {
                    Some(Ok(user)) => rsx! {
                        div { class: "bg-[#0f1012]/60 rounded-2xl shadow-none overflow-hidden backdrop-blur-md border border-white/5",
                            // Tabs
                            div { class: "border-b border-white/10",
                                div { class: "flex",
                                    TabButton {
                                        active: active_tab() == ProfileTab::General,
                                        onclick: move |_| active_tab.set(ProfileTab::General),
                                        label: "General",
                                    }
                                    TabButton {
                                        active: active_tab() == ProfileTab::Skills,
                                        onclick: move |_| active_tab.set(ProfileTab::Skills),
                                        label: "Skills",
                                    }
                                    TabButton {
                                        active: active_tab() == ProfileTab::Preferences,
                                        onclick: move |_| active_tab.set(ProfileTab::Preferences),
                                        label: "Preferences",
                                    }
                                    TabButton {
                                        active: active_tab() == ProfileTab::Security,
                                        onclick: move |_| active_tab.set(ProfileTab::Security),
                                        label: "Security",
                                    }
                                }
                            }

                            // Tab Content
                            div { class: "p-8",
                                match active_tab() {
                                    ProfileTab::General => rsx! {
                                        GeneralTab { user: user.clone() }
                                    },
                                    ProfileTab::Skills => rsx! {
                                        SkillsTab { user: user.clone() }
                                    },
                                    ProfileTab::Preferences => rsx! {
                                        PreferencesTab { user: user.clone() }
                                    },
                                    ProfileTab::Security => rsx! {
                                        SecurityTab { user: user.clone() }
                                    },
                                }
                            }
                        }
                    },
                    Some(Err(e)) => rsx! {
                        div { class: "bg-red-500/10 text-red-300 p-6 rounded-lg backdrop-blur-md border border-white/5",
                            "Error loading profile: {e}"
                        }
                    },
                    None => rsx! {
                        div { class: "text-center text-gray-500", "Loading..." }
                    },
                }
            }
        }
    }
}

#[component]
fn TabButton(active: bool, onclick: EventHandler<()>, label: &'static str) -> Element {
    rsx! {
        button {
            onclick: move |_| onclick.call(()),
            class: format!(
                "px-6 py-4 font-medium transition border-b-2 {}",
                if active {
                    "border-teal-400 text-teal-400"
                } else {
                    "border-transparent text-gray-400 hover:text-gray-100"
                },
            ),
            {label}
        }
    }
}

#[component]
fn GeneralTab(user: User) -> Element {
    let user_id = user.id.clone().unwrap_or_default();
    let mut name = use_signal(|| user.name.clone());
    let username = use_signal(|| user.username.clone());
    let mut success = use_signal(|| None::<String>);
    let mut error = use_signal(|| None::<String>);

    let save_changes = move |_| {
        let user_id = user_id.clone();
        spawn(async move {
            match update_user_profile(user_id, Some(name()), None, None).await {
                Ok(_) => {
                    success.set(Some("Profile updated successfully!".to_string()));
                    error.set(None);
                }
                Err(e) => {
                    error.set(Some(format!("Failed to update: {}", e)));
                    success.set(None);
                }
            }
        });
    };

    rsx! {
        div { class: "max-w-2xl",
            h2 { class: "text-2xl font-bold text-gray-100 mb-6", "General Information" }

            if let Some(msg) = success() {
                div { class: "mb-4 p-4 bg-green-500/10 text-green-300 rounded-lg backdrop-blur-md border border-white/5",
                    {msg}
                }
            }

            if let Some(err) = error() {
                div { class: "mb-4 p-4 bg-red-500/10 text-red-300 rounded-lg backdrop-blur-md border border-white/5",
                    {err}
                }
            }

            div { class: "space-y-6",
                div {
                    label { class: "block text-sm font-medium text-gray-400 mb-2", "Full Name" }
                    input {
                        r#type: "text",
                        class: "w-full px-4 py-3 bg-[#050505] text-gray-100 border border-white/10 rounded-lg focus:ring-2 focus:ring-teal-500/30 focus:border-transparent outline-none transition",
                        value: "{name}",
                        oninput: move |e| name.set(e.value()),
                    }
                }

                div {
                    label { class: "block text-sm font-medium text-gray-400 mb-2", "Username" }
                    input {
                        r#type: "text",
                        class: "w-full px-4 py-3 border border-white/10 rounded-lg bg-white/5 text-gray-500 cursor-not-allowed backdrop-blur-md",
                        value: "{username}",
                        disabled: true,
                    }
                    p { class: "mt-1 text-sm text-gray-500", "Username cannot be changed" }
                }

                div {
                    label { class: "block text-sm font-medium text-gray-400 mb-2", "Member Since" }
                    p { class: "text-gray-400", {user.created_at.format("%B %d, %Y").to_string()} }
                }

                button {
                    onclick: save_changes,
                    class: "px-6 py-3 bg-gradient-to-r from-teal-500 to-blue-600 text-white rounded-lg hover:shadow-[0_0_20px_rgba(20,184,166,0.25)] transition font-medium",
                    "Save Changes"
                }
            }
        }
    }
}

#[component]
fn SkillsTab(user: User) -> Element {
    // We still need the user_id for the *API call* to save,
    // even though it's not in the UserSkills struct anymore.
    let user_id = user.id.clone().unwrap_or_default();

    let mut skills = use_signal(|| user.skills_learned.clone());
    let mut new_skill = use_signal(String::new);
    let mut success = use_signal(|| None::<String>);
    let mut error = use_signal(|| None::<String>);

    // --- Add Skill Logic ---
    let mut add_skill = move |_| {
        let input_val = new_skill();
        if !input_val.trim().is_empty() {
            let mut current_skills = skills();

            // Case-insensitive duplicate check
            let exists = current_skills
                .iter()
                .any(|s| s.skillname.eq_ignore_ascii_case(&input_val));

            if !exists {
                // Construct the simplified struct
                let new_entry: UserSkills = UserSkills {
                    skillname: input_val.trim().to_string(),
                    date_learnt: Utc::now(), // Auto-timestamp
                };

                current_skills.push(new_entry);
                skills.set(current_skills);
                new_skill.set(String::new()); // Clear input
                error.set(None);
            } else {
                error.set(Some("You have already added this skill.".to_string()));
                // Auto-clear error after 3 seconds
                spawn(async move {
                    gloo_timers::future::TimeoutFuture::new(3000).await;
                    error.set(None);
                });
            }
        }
    };

    // --- Remove Skill Logic ---
    let mut remove_skill = move |skill_name: String| {
        let mut current_skills = skills();
        current_skills.retain(|s| s.skillname != skill_name);
        skills.set(current_skills);
    };

    // --- Save to Backend Logic ---
    let save_skills = move |_| {
        let uid = user_id.clone();
        let skills_payload = skills();

        spawn(async move {
            // We pass the vector of UserSkills. The backend handles embedding it into the User record.
            match update_user_profile(uid, None, Some(skills_payload), None).await {
                Ok(_) => {
                    success.set(Some("Skills saved successfully!".to_string()));
                    error.set(None);
                    gloo_timers::future::TimeoutFuture::new(3000).await;
                    success.set(None);
                }
                Err(e) => {
                    error.set(Some(format!("Failed to save: {}", e)));
                    success.set(None);
                }
            }
        });
    };

    rsx! {
        div { class: "max-w-3xl",
            // Header
            div { class: "flex justify-between items-end mb-6",
                div {
                    h2 { class: "text-2xl font-bold text-gray-100 mb-2", "Skills & Expertise" }
                    p { class: "text-gray-400 text-sm",
                        "Showcase your tech stack. We use this to personalize your learning path."
                    }
                }
                span { class: "text-xs font-mono text-teal-500/80 bg-teal-500/10 px-2 py-1 rounded border border-teal-500/20",
                    "{skills().len()} Skills"
                }
            }

            // Notifications
            if let Some(msg) = success() {
                div { class: "mb-6 p-4 bg-emerald-500/10 text-emerald-300 rounded-lg backdrop-blur-md border border-emerald-500/20 flex items-center gap-2 animate-in fade-in slide-in-from-top-2",
                    span { class: "font-bold", "✓" }
                    "{msg}"
                }
            }

            if let Some(err) = error() {
                div { class: "mb-6 p-4 bg-red-500/10 text-red-300 rounded-lg backdrop-blur-md border border-red-500/20 flex items-center gap-2 animate-in fade-in slide-in-from-top-2",
                    span { class: "font-bold", "!" }
                    "{err}"
                }
            }

            // Input Section
            div { class: "mb-8 bg-[#0a0a0a] p-1 rounded-xl border border-white/5 shadow-inner",
                div { class: "flex gap-2",
                    div { class: "relative flex-1 group",
                        input {
                            r#type: "text",
                            class: "w-full px-4 py-3 bg-[#050505] text-gray-100 border border-white/10 rounded-lg focus:ring-2 focus:ring-teal-500/30 focus:border-teal-500/50 outline-none transition-all placeholder:text-gray-600",
                            value: "{new_skill}",
                            oninput: move |e| new_skill.set(e.value()),
                            onkeypress: move |e| {
                                if e.key() == Key::Enter {
                                    add_skill(());
                                }
                            },
                            placeholder: "Add a skill (e.g. Rust, Docker, CI/CD)...",
                        }
                        // Clear Input Button
                        if !new_skill().is_empty() {
                            button {
                                onclick: move |_| new_skill.set(String::new()),
                                class: "absolute right-3 top-1/2 -translate-y-1/2 text-gray-500 hover:text-gray-300 transition-colors",
                                "✕"
                            }
                        }
                    }

                    button {
                        onclick: move |_| add_skill(()),
                        class: "px-6 py-3 bg-gradient-to-r from-teal-600 to-blue-600 text-white rounded-lg hover:shadow-[0_0_18px_rgba(20,184,166,0.25)] hover:scale-[1.02] active:scale-[0.98] transition-all font-medium whitespace-nowrap",
                        "Add Skill"
                    }
                }
            }

            // Skills List / Chips
            div { class: "min-h-[100px] mb-8",
                if skills().is_empty() {
                    div { class: "flex flex-col items-center justify-center py-12 text-gray-600 border-2 border-dashed border-white/5 rounded-xl bg-white/[0.02]",
                        span { class: "text-4xl mb-3 opacity-20", "⚡" }
                        p { "Your skill set is empty." }
                        p { class: "text-sm mt-1", "Add your first skill above to get started." }
                    }
                } else {
                    div { class: "flex flex-wrap gap-3",
                        for skill in skills() {
                            {
                                let skill_name = skill.skillname.clone();
                                // Format date for tooltip
                                let date_display = skill
                                    .date_learnt
                                    .format("%b %Y")
                                    .to_string();
                                rsx! {
                                    div {
                                        key: "{skill_name}",
                                        class: "group relative inline-flex items-center px-4 py-2 bg-gradient-to-br from-teal-500/5 to-blue-500/5 text-teal-200 rounded-lg border border-teal-500/10 hover:border-teal-500/30 backdrop-blur-md transition-all duration-200 hover:shadow-[0_0_15px_rgba(20,184,166,0.1)] cursor-default",

                                        span { class: "font-medium tracking-wide", "{skill_name}" }

                                        // Tooltip: Shows Date Added
                                        div { class: "absolute bottom-full left-1/2 -translate-x-1/2 mb-2 px-2 py-1 bg-black text-xs text-gray-300 rounded opacity-0 group-hover:opacity-100 transition-opacity pointer-events-none whitespace-nowrap border border-white/10 z-10",
                                            "Added: {date_display}"
                                        }

                                        // Remove Button
                                        button {
                                            onclick: move |_| remove_skill(skill_name.clone()),
                                            class: "ml-3 -mr-1 p-0.5 rounded-full hover:bg-red-500/20 text-teal-500/50 hover:text-red-400 transition-colors cursor-pointer",
                                            span { "×" }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }

            // Save Button
            div { class: "flex justify-end pt-6 border-t border-white/5",
                button {
                    onclick: save_skills,
                    class: "flex items-center gap-2 px-8 py-3 bg-white/5 hover:bg-white/10 text-white rounded-lg border border-white/10 hover:border-white/20 transition-all font-medium",
                    span { "Save Changes" }
                }
            }
        }
    }
}

#[component]
fn PreferencesTab(user: User) -> Element {
    let user_id = user.id.clone().unwrap_or_default();
    let mut learning_style = use_signal(|| user.preferences.learning_style.clone());
    let mut time_commitment = use_signal(|| user.preferences.time_commitment.clone());
    let mut difficulty = use_signal(|| user.preferences.difficulty_preference.clone());
    let mut success = use_signal(|| None::<String>);
    let mut error = use_signal(|| None::<String>);

    let save_preferences = move |_| {
        let user_id = user_id.clone();
        let prefs = UserPreferences {
            learning_style: learning_style(),
            time_commitment: time_commitment(),
            preferred_content_types: vec![],
            difficulty_preference: difficulty(),
        };

        spawn(async move {
            match update_user_profile(user_id, None, None, Some(prefs)).await {
                Ok(_) => {
                    success.set(Some("Preferences updated successfully!".to_string()));
                    error.set(None);
                }
                Err(e) => {
                    error.set(Some(format!("Failed to update: {}", e)));
                    success.set(None);
                }
            }
        });
    };

    rsx! {
        div { class: "max-w-2xl",
            h2 { class: "text-2xl font-bold text-gray-100 mb-6", "Learning Preferences" }

            if let Some(msg) = success() {
                div { class: "mb-4 p-4 bg-green-500/10 text-green-300 rounded-lg backdrop-blur-md border border-white/5",
                    {msg}
                }
            }

            if let Some(err) = error() {
                div { class: "mb-4 p-4 bg-red-500/10 text-red-300 rounded-lg backdrop-blur-md border border-white/5",
                    {err}
                }
            }

            div { class: "space-y-6",
                div {
                    label { class: "block text-sm font-medium text-gray-400 mb-2", "Learning Style" }
                    select {
                        class: "w-full px-4 py-3 bg-[#050505] text-gray-100 border border-white/10 rounded-lg focus:ring-2 focus:ring-teal-500/30 focus:border-transparent outline-none",
                        value: "{learning_style}",
                        onchange: move |e| learning_style.set(e.value()),
                        option { value: "visual", "Visual (videos, diagrams)" }
                        option { value: "reading", "Reading (articles, documentation)" }
                        option { value: "hands-on", "Hands-on (projects, exercises)" }
                        option { value: "mixed", "Mixed approach" }
                    }
                }

                div {
                    label { class: "block text-sm font-medium text-gray-400 mb-2", "Time Commitment" }
                    select {
                        class: "w-full px-4 py-3 bg-[#050505] text-gray-100 border border-white/10 rounded-lg focus:ring-2 focus:ring-teal-500/30 focus:border-transparent outline-none",
                        value: "{time_commitment}",
                        onchange: move |e| time_commitment.set(e.value()),
                        option { value: "1-2", "1-2 hours/week" }
                        option { value: "3-5", "3-5 hours/week" }
                        option { value: "6-10", "6-10 hours/week" }
                        option { value: "10+", "10+ hours/week" }
                    }
                }

                div {
                    label { class: "block text-sm font-medium text-gray-400 mb-2",
                        "Difficulty Preference"
                    }
                    select {
                        class: "w-full px-4 py-3 bg-[#050505] text-gray-100 border border-white/10 rounded-lg focus:ring-2 focus:ring-teal-500/30 focus:border-transparent outline-none",
                        value: "{difficulty}",
                        onchange: move |e| difficulty.set(e.value()),
                        option { value: "beginner", "Beginner-friendly" }
                        option { value: "intermediate", "Intermediate" }
                        option { value: "advanced", "Advanced" }
                        option { value: "mixed", "Mixed levels" }
                    }
                }

                button {
                    onclick: save_preferences,
                    class: "px-6 py-3 bg-gradient-to-r from-teal-500 to-blue-600 text-white rounded-lg hover:shadow-[0_0_20px_rgba(20,184,166,0.25)] transition font-medium",
                    "Save Preferences"
                }
            }
        }
    }
}

#[component]
fn SecurityTab(user: User) -> Element {
    let user_id = user.id.clone().unwrap_or_default();
    let mut old_password = use_signal(String::new);
    let mut new_password = use_signal(String::new);
    let mut confirm_password = use_signal(String::new);
    let mut success = use_signal(|| None::<String>);
    let mut error = use_signal(|| None::<String>);

    let change_pwd = move |_| {
        let user_id = user_id.clone();

        if new_password() != confirm_password() {
            error.set(Some("Passwords don't match".to_string()));
            return;
        }

        if new_password().len() < 8 {
            error.set(Some("Password must be at least 8 characters".to_string()));
            return;
        }

        spawn(async move {
            match change_password(user_id, old_password(), new_password()).await {
                Ok(_) => {
                    success.set(Some("Password changed successfully!".to_string()));
                    error.set(None);
                    old_password.set(String::new());
                    new_password.set(String::new());
                    confirm_password.set(String::new());
                }
                Err(e) => {
                    error.set(Some(format!("Failed to change password: {}", e)));
                    success.set(None);
                }
            }
        });
    };

    rsx! {
        div { class: "max-w-2xl",
            h2 { class: "text-2xl font-bold text-gray-100 mb-6", "Security Settings" }

            if let Some(msg) = success() {
                div { class: "mb-4 p-4 bg-green-500/10 text-green-300 rounded-lg backdrop-blur-md border border-white/5",
                    {msg}
                }
            }

            if let Some(err) = error() {
                div { class: "mb-4 p-4 bg-red-500/10 text-red-300 rounded-lg backdrop-blur-md border border-white/5",
                    {err}
                }
            }

            div { class: "space-y-6",
                div {
                    label { class: "block text-sm font-medium text-gray-400 mb-2", "Current Password" }
                    input {
                        r#type: "password",
                        class: "w-full px-4 py-3 bg-[#050505] text-gray-100 border border-white/10 rounded-lg focus:ring-2 focus:ring-teal-500/30 focus:border-transparent outline-none",
                        value: "{old_password}",
                        oninput: move |e| old_password.set(e.value()),
                    }
                }

                div {
                    label { class: "block text-sm font-medium text-gray-400 mb-2", "New Password" }
                    input {
                        r#type: "password",
                        class: "w-full px-4 py-3 bg-[#050505] text-gray-100 border border-white/10 rounded-lg focus:ring-2 focus:ring-teal-500/30 focus:border-transparent outline-none",
                        value: "{new_password}",
                        oninput: move |e| new_password.set(e.value()),
                    }
                }

                div {
                    label { class: "block text-sm font-medium text-gray-400 mb-2", "Confirm New Password" }
                    input {
                        r#type: "password",
                        class: "w-full px-4 py-3 bg-[#050505] text-gray-100 border border-white/10 rounded-lg focus:ring-2 focus:ring-teal-500/30 focus:border-transparent outline-none",
                        value: "{confirm_password}",
                        oninput: move |e| confirm_password.set(e.value()),
                    }
                }

                button {
                    onclick: change_pwd,
                    class: "px-6 py-3 bg-gradient-to-r from-teal-500 to-blue-600 text-white rounded-lg hover:shadow-[0_0_20px_rgba(20,184,166,0.25)] transition font-medium",
                    "Change Password"
                }
            }
        }
    }
}
