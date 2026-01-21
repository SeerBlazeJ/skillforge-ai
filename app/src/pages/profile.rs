use crate::{
    models::{User, UserPreferences},
    server_functions::{change_password, get_user_data, update_user_profile},
    Route,
};
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
    let user_data =
        use_resource(|| async move { get_user_data("user_id_placeholder".to_string()).await });

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
                    h1 { class: "text-xl font-bold text-gray-900", "Profile Settings" }
                }
            }

            main { class: "container mx-auto px-6 py-8 max-w-5xl",
                match user_data.read_unchecked().as_ref() {
                    Some(Ok(user)) => rsx! {
                        div { class: "bg-white rounded-2xl shadow-lg overflow-hidden",
                            // Tabs
                            div { class: "border-b border-gray-200",
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
                        div { class: "bg-red-50 text-red-700 p-6 rounded-lg", "Error loading profile: {e}" }
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
                    "border-indigo-600 text-indigo-600"
                } else {
                    "border-transparent text-gray-600 hover:text-gray-900"
                },
            ),
            {label}
        }
    }
}

#[component]
fn GeneralTab(user: User) -> Element {
    let user_id = user.id.clone().unwrap_or_default(); // Clone user_id upfront
    let mut name = use_signal(|| user.name.clone());
    let username = use_signal(|| user.username.clone());
    let mut success = use_signal(|| None::<String>);
    let mut error = use_signal(|| None::<String>);

    let save_changes = move |_| {
        let user_id = user_id.clone(); // Clone inside closure
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
            h2 { class: "text-2xl font-bold text-gray-900 mb-6", "General Information" }

            if let Some(msg) = success() {
                div { class: "mb-4 p-4 bg-green-50 text-green-700 rounded-lg", {msg} }
            }

            if let Some(err) = error() {
                div { class: "mb-4 p-4 bg-red-50 text-red-700 rounded-lg", {err} }
            }

            div { class: "space-y-6",
                div {
                    label { class: "block text-sm font-medium text-gray-700 mb-2", "Full Name" }
                    input {
                        r#type: "text",
                        class: "w-full px-4 py-3 border border-gray-300 rounded-lg focus:ring-2 focus:ring-indigo-500 focus:border-transparent outline-none",
                        value: "{name}",
                        oninput: move |e| name.set(e.value()),
                    }
                }

                div {
                    label { class: "block text-sm font-medium text-gray-700 mb-2", "Username" }
                    input {
                        r#type: "text",
                        class: "w-full px-4 py-3 border border-gray-300 rounded-lg bg-gray-50 cursor-not-allowed",
                        value: "{username}",
                        disabled: true,
                    }
                    p { class: "mt-1 text-sm text-gray-500", "Username cannot be changed" }
                }

                div {
                    label { class: "block text-sm font-medium text-gray-700 mb-2", "Member Since" }
                    p { class: "text-gray-600", {user.created_at.format("%B %d, %Y").to_string()} }
                }

                button {
                    onclick: save_changes,
                    class: "px-6 py-3 bg-indigo-600 text-white rounded-lg hover:bg-indigo-700 transition font-medium",
                    "Save Changes"
                }
            }
        }
    }
}

#[component]
fn SkillsTab(user: User) -> Element {
    let user_id = user.id.clone().unwrap_or_default();
    let mut skills = use_signal(|| user.skills_learned.clone());
    let mut new_skill = use_signal(String::new);
    let mut success = use_signal(|| None::<String>);
    let mut error = use_signal(|| None::<String>);

    let mut add_skill = move |_| {
        if !new_skill().trim().is_empty() {
            let mut current_skills = skills();
            if !current_skills.contains(&new_skill()) {
                current_skills.push(new_skill());
                skills.set(current_skills);
                new_skill.set(String::new());
            }
        }
    };

    let mut remove_skill = move |skill: String| {
        let mut current_skills = skills();
        current_skills.retain(|s| s != &skill);
        skills.set(current_skills);
    };

    let save_skills = move |_| {
        let user_id = user_id.clone();
        spawn(async move {
            match update_user_profile(user_id, None, Some(skills()), None).await {
                Ok(_) => {
                    success.set(Some("Skills updated successfully!".to_string()));
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
            h2 { class: "text-2xl font-bold text-gray-900 mb-2", "Skills You've Learned" }
            p { class: "text-gray-600 mb-6",
                "Keep track of skills you've already mastered to get better roadmap recommendations."
            }

            if let Some(msg) = success() {
                div { class: "mb-4 p-4 bg-green-50 text-green-700 rounded-lg", {msg} }
            }

            if let Some(err) = error() {
                div { class: "mb-4 p-4 bg-red-50 text-red-700 rounded-lg", {err} }
            }

            div { class: "mb-6",
                label { class: "block text-sm font-medium text-gray-700 mb-2", "Add New Skill" }
                div { class: "flex gap-3",
                    input {
                        r#type: "text",
                        class: "flex-1 px-4 py-3 border border-gray-300 rounded-lg focus:ring-2 focus:ring-indigo-500 focus:border-transparent outline-none",
                        value: "{new_skill}",
                        oninput: move |e| new_skill.set(e.value()),
                        onkeypress: move |e| {
                            if e.key() == Key::Enter {
                                add_skill(());
                            }
                        },
                        placeholder: "e.g., Python, React, Machine Learning...",
                    }
                    button {
                        onclick: move |_| add_skill(()), // Fix: call the closure properly
                        class: "px-6 py-3 bg-indigo-600 text-white rounded-lg hover:bg-indigo-700 transition font-medium",
                        "Add"
                    }
                }
            }

            div { class: "mb-6",
                if skills().is_empty() {
                    div { class: "text-center py-12 text-gray-500",
                        "No skills added yet. Add your first skill above!"
                    }
                } else {
                    div { class: "flex flex-wrap gap-2",
                        for skill in skills() {
                            {
                                let skill_clone = skill.clone();
                                rsx! {
                                    div {
                                        key: "{skill}",
                                        class: "inline-flex items-center px-4 py-2 bg-indigo-100 text-indigo-800 rounded-lg font-medium",
                                        span { {skill.clone()} }
                                        button {
                                            onclick: move |_| remove_skill(skill_clone.clone()),
                                            class: "ml-2 text-indigo-600 hover:text-indigo-800",
                                            "×"
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }

            button {
                onclick: save_skills,
                class: "px-6 py-3 bg-indigo-600 text-white rounded-lg hover:bg-indigo-700 transition font-medium",
                "Save Skills"
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
            h2 { class: "text-2xl font-bold text-gray-900 mb-6", "Learning Preferences" }

            if let Some(msg) = success() {
                div { class: "mb-4 p-4 bg-green-50 text-green-700 rounded-lg", {msg} }
            }

            if let Some(err) = error() {
                div { class: "mb-4 p-4 bg-red-50 text-red-700 rounded-lg", {err} }
            }

            div { class: "space-y-6",
                div {
                    label { class: "block text-sm font-medium text-gray-700 mb-2", "Learning Style" }
                    select {
                        class: "w-full px-4 py-3 border border-gray-300 rounded-lg focus:ring-2 focus:ring-indigo-500 focus:border-transparent outline-none",
                        value: "{learning_style}",
                        onchange: move |e| learning_style.set(e.value()),
                        option { value: "visual", "Visual (videos, diagrams)" }
                        option { value: "reading", "Reading (articles, documentation)" }
                        option { value: "hands-on", "Hands-on (projects, exercises)" }
                        option { value: "mixed", "Mixed approach" }
                    }
                }

                div {
                    label { class: "block text-sm font-medium text-gray-700 mb-2", "Time Commitment" }
                    select {
                        class: "w-full px-4 py-3 border border-gray-300 rounded-lg focus:ring-2 focus:ring-indigo-500 focus:border-transparent outline-none",
                        value: "{time_commitment}",
                        onchange: move |e| time_commitment.set(e.value()),
                        option { value: "1-2", "1-2 hours/week" }
                        option { value: "3-5", "3-5 hours/week" }
                        option { value: "6-10", "6-10 hours/week" }
                        option { value: "10+", "10+ hours/week" }
                    }
                }

                div {
                    label { class: "block text-sm font-medium text-gray-700 mb-2",
                        "Difficulty Preference"
                    }
                    select {
                        class: "w-full px-4 py-3 border border-gray-300 rounded-lg focus:ring-2 focus:ring-indigo-500 focus:border-transparent outline-none",
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
                    class: "px-6 py-3 bg-indigo-600 text-white rounded-lg hover:bg-indigo-700 transition font-medium",
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
            h2 { class: "text-2xl font-bold text-gray-900 mb-6", "Security Settings" }

            if let Some(msg) = success() {
                div { class: "mb-4 p-4 bg-green-50 text-green-700 rounded-lg", {msg} }
            }

            if let Some(err) = error() {
                div { class: "mb-4 p-4 bg-red-50 text-red-700 rounded-lg", {err} }
            }

            div { class: "space-y-6",
                div {
                    label { class: "block text-sm font-medium text-gray-700 mb-2", "Current Password" }
                    input {
                        r#type: "password",
                        class: "w-full px-4 py-3 border border-gray-300 rounded-lg focus:ring-2 focus:ring-indigo-500 focus:border-transparent outline-none",
                        value: "{old_password}",
                        oninput: move |e| old_password.set(e.value()),
                    }
                }

                div {
                    label { class: "block text-sm font-medium text-gray-700 mb-2", "New Password" }
                    input {
                        r#type: "password",
                        class: "w-full px-4 py-3 border border-gray-300 rounded-lg focus:ring-2 focus:ring-indigo-500 focus:border-transparent outline-none",
                        value: "{new_password}",
                        oninput: move |e| new_password.set(e.value()),
                    }
                }

                div {
                    label { class: "block text-sm font-medium text-gray-700 mb-2", "Confirm New Password" }
                    input {
                        r#type: "password",
                        class: "w-full px-4 py-3 border border-gray-300 rounded-lg focus:ring-2 focus:ring-indigo-500 focus:border-transparent outline-none",
                        value: "{confirm_password}",
                        oninput: move |e| confirm_password.set(e.value()),
                    }
                }

                button {
                    onclick: change_pwd,
                    class: "px-6 py-3 bg-indigo-600 text-white rounded-lg hover:bg-indigo-700 transition font-medium",
                    "Change Password"
                }
            }
        }
    }
}
