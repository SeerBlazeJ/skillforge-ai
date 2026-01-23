use dioxus::prelude::*;

#[cfg(feature = "server")]
use anyhow::{Context, Result};

#[cfg(feature = "server")]
use fastembed::{EmbeddingModel, InitOptions, TextEmbedding};

#[cfg(feature = "server")]
use surrealdb::{engine::local::RocksDb, RecordId, Surreal};

#[cfg(feature = "server")]
use bcrypt::{hash, verify, DEFAULT_COST};

#[cfg(feature = "server")]
use chrono::Utc;

#[cfg(feature = "server")]
use uuid::Uuid;

#[cfg(feature = "server")]
use base64::{engine::general_purpose, Engine as _};

#[cfg(feature = "server")]
use rand::Rng;

use crate::models::*;
use crate::SESSION_DURATION_DAYS;

#[cfg(feature = "server")]
const MODEL: EmbeddingModel = EmbeddingModel::ModernBertEmbedLarge;

#[cfg(feature = "server")]
static DB_INSTANCE: tokio::sync::OnceCell<Surreal<surrealdb::engine::local::Db>> =
    tokio::sync::OnceCell::const_new();

#[cfg(feature = "server")]
trait IntoServerError<T> {
    fn into_server_error(self) -> Result<T, ServerFnError>;
}

#[cfg(feature = "server")]
impl<T, E: std::fmt::Display> IntoServerError<T> for Result<T, E> {
    fn into_server_error(self) -> Result<T, ServerFnError> {
        self.map_err(|e| ServerFnError::new(e.to_string()))
    }
}

// const SESSION_COOKIE_NAME: &str = "skillforge_session";

#[cfg(feature = "server")]
async fn get_db() -> Result<&'static Surreal<surrealdb::engine::local::Db>> {
    DB_INSTANCE
        .get_or_try_init(|| async {
            let db = Surreal::new::<RocksDb>("skillforge")
                .await
                .context("Failed to connect to Database")?;

            db.use_ns("main").use_db("main").await?;

            db.query("DEFINE TABLE users;").await?;
            db.query("DEFINE FIELD username ON users TYPE string;")
                .await?;
            db.query("DEFINE FIELD password_hash ON users TYPE string;")
                .await?;
            db.query("DEFINE FIELD name ON users TYPE string;").await?;
            db.query("DEFINE INDEX unique_username ON users FIELDS username UNIQUE;")
                .await?;

            db.query("DEFINE TABLE roadmaps;").await?;
            db.query("DEFINE FIELD user_id ON roadmaps TYPE string;")
                .await?;
            db.query("DEFINE FIELD skill_name ON roadmaps TYPE string;")
                .await?;

            Ok(db)
        })
        .await
}

// SESSION FUNCTIONS
#[cfg(feature = "server")]
fn generate_session_token() -> String {
    let mut rng = rand::rng();
    let random_bytes: Vec<u8> = (0..32).map(|_| rng.random()).collect();
    general_purpose::URL_SAFE_NO_PAD.encode(random_bytes)
}

#[cfg(feature = "server")]
async fn create_session(user_id: String) -> Result<String, ServerFnError> {
    use chrono::Duration;

    let db = get_db().await.into_server_error()?;
    let session_token = generate_session_token();

    let session = Session {
        id: None,
        user_id: user_id.clone(),
        session_token: session_token.clone(),
        created_at: Utc::now(),
        expires_at: Utc::now() + Duration::days(SESSION_DURATION_DAYS),
    };

    let _: Option<Session> = db
        .create("sessions")
        .content(session)
        .await
        .into_server_error()?;
    // .ok_or(ServerFnError::new(
    //     "No response upon creation of entry by the database",
    // ))?;

    Ok(session_token)
}

#[cfg(feature = "server")]
async fn get_user_from_session(session_token: String) -> Result<Option<User>, ServerFnError> {
    let db = get_db().await.into_server_error()?;

    let mut result = db
        .query("SELECT * FROM sessions WHERE session_token = $session_token") // Removed expires check for debugging
        .bind(("session_token", session_token))
        .await
        .into_server_error()?;

    let sessions: Vec<Session> = result.take(0).into_server_error()?;

    if let Some(session) = sessions.first() {
        if session.expires_at < Utc::now() {
            return Ok(None);
        }
        let user_id: RecordId = session
            .user_id
            .parse()
            .ok()
            .ok_or(ServerFnError::new("Could not parse user ID"))?;
        let user: UserDB = db
            .select(user_id)
            .await
            .into_server_error()?
            .ok_or(ServerFnError::new("User not found"))?;
        let user = User::from(user);
        Ok(Some(user))
    } else {
        Ok(None)
    }
}

#[cfg(feature = "server")]
async fn delete_session(session_token: String) -> Result<(), ServerFnError> {
    let db = get_db().await.into_server_error()?;

    let mut result = db
        .query("DELETE sessions WHERE session_token = $session_token")
        .bind(("session_token", session_token))
        .await
        .into_server_error()?;

    let _: Vec<Session> = result.take(0).into_server_error()?;

    Ok(())
}

// SERVER FUNCTIONS
#[server]
pub async fn signup_user(
    username: String,
    password: String,
    name: String,
) -> Result<String, ServerFnError> {
    let db = get_db().await?;

    let password_hash = hash(password.as_bytes(), DEFAULT_COST).into_server_error()?;

    let user_db = UserDB::from(User {
        id: None,
        username: username.clone(),
        password_hash,
        name,
        skills_learned: Vec::new(),
        preferences: UserPreferences::default(),
        created_at: Utc::now(),
    });

    let created: UserDB = db
        .create("users")
        .content(user_db)
        .await
        .map_err(|e| ServerFnError::new(format!("Failed to create user: {}", e)))?
        .ok_or(ServerFnError::ServerError {
            message: "Couldn't create entry, database returned none or error value".to_string(),
            code: 500,
            details: None,
        })?;

    let user = User::from(created.to_owned());

    Ok(user.id.unwrap_or_else(String::new))
}

#[server]
pub async fn login_user(username: String, password: String) -> Result<String, ServerFnError> {
    let db = get_db().await?;
    eprintln!("Found request for {}", &username);
    let users: Vec<UserDB> = db
        .query("SELECT * FROM users where username = $username;")
        .bind(("username", username.clone()))
        .await
        .into_server_error()?
        .take(0)
        .into_server_error()?;
    if let Some(user) = users.first() {
        if verify(password.as_bytes(), &user.password_hash).into_server_error()? {
            let user = User::from(user.to_owned());
            let user_id = user
                .id
                .clone()
                .ok_or(ServerFnError::new("User has no ID"))?;
            let session_token = create_session(user_id).await?;
            return Ok(session_token);
        }
    } else {
        eprintln!("Record not found");
    }

    Err(ServerFnError::new("Invalid credentials"))
}

#[server]
pub async fn get_user_data(session_token: String) -> Result<User, ServerFnError> {
    let user = get_user_from_session(session_token)
        .await?
        .ok_or_else(|| ServerFnError::new("Invalid or expired session"))?;

    Ok(user)
}

#[server]
pub async fn update_user_profile(
    user_id: String,
    name: Option<String>,
    skills_learned: Option<Vec<String>>,
    preferences: Option<UserPreferences>,
) -> Result<(), ServerFnError> {
    let db = get_db().await?;
    let user_id: RecordId = user_id
        .parse()
        .ok()
        .ok_or(ServerFnError::new("Could not parse user ID"))?;
    let mut user: User = db
        .select(user_id)
        .await
        .into_server_error()?
        .ok_or_else(|| ServerFnError::new("User not found"))?;
    // let mut user: User = get_user_data(session_token).await?;

    if let Some(name) = name {
        user.name = name;
    }
    if let Some(skills) = skills_learned {
        user.skills_learned = skills;
    }
    if let Some(prefs) = preferences {
        user.preferences = prefs;
    }

    let _: Vec<Option<User>> = db
        .update(user.id.clone().unwrap())
        .content(user)
        .await
        .into_server_error()?;

    Ok(())
}

#[server]
pub async fn change_password(
    user_id: String,
    old_password: String,
    new_password: String,
) -> Result<(), ServerFnError> {
    let db = get_db().await?;
    let user_id: RecordId = user_id
        .parse()
        .ok()
        .ok_or(ServerFnError::new("Could not parse user ID"))?;
    let mut user: UserDB = db
        .select(user_id.clone())
        .await
        .into_server_error()?
        .ok_or_else(|| ServerFnError::new("User not found"))?;
    // let mut user = get_user_data(session_token).await?;
    // let user_id = user
    //     .id
    //     .clone()
    //     .ok_or(ServerFnError::new("User ID not found"))?;

    if !verify(old_password.as_bytes(), &user.password_hash).into_server_error()? {
        return Err(ServerFnError::new("Invalid old password"));
    }

    user.password_hash = hash(new_password.as_bytes(), DEFAULT_COST).into_server_error()?;

    let _: Option<User> = db.update(user_id).content(user).await.into_server_error()?;

    Ok(())
}

#[server]
pub async fn get_user_roadmaps(session_token: String) -> Result<Vec<Roadmap>, ServerFnError> {
    let db = get_db().await?;

    let user = get_user_data(session_token).await?;
    let user_id = user.id.ok_or(ServerFnError::new("User ID not found"))?;

    let mut result = db
        .query("SELECT * FROM roadmaps WHERE user_id = $user_id ORDER BY updated_at DESC")
        .bind(("user_id", user_id))
        .await
        .into_server_error()?;

    let roadmaps: Vec<Roadmap> = result.take(0).into_server_error()?;

    Ok(roadmaps)
}

#[server]
pub async fn get_roadmap(roadmap_id: String) -> Result<Roadmap, ServerFnError> {
    let db = get_db().await?;

    let roadmap: Option<Roadmap> = db
        .select(("roadmaps", roadmap_id))
        .await
        .into_server_error()?;

    roadmap.ok_or_else(|| ServerFnError::new("Roadmap not found"))
}

#[server]
pub async fn generate_questions(
    skill_name: String,
    session_token: String,
) -> Result<Vec<Question>, ServerFnError> {
    let user: User = get_user_data(session_token).await?;

    let prompt = format!(
        "Generate 8 questions to evaluate a user's learning preferences and existing knowledge for learning {}. \n\
        User's existing skills: {:?}\n\
        User's preferences: {:?}\n\n\
        Generate:\n\
        - 5 preference questions (learning style, time commitment, content type preferences)\n\
        - 3 knowledge evaluation questions (to test existing knowledge)\n\n\
        Format strictly as JSON array with: question_text, question_type (MCQ/MSQ/TrueFalse/OneWord), options (array of strings, empty for OneWord)",
        skill_name,
        user.skills_learned,
        user.preferences
    );

    let questions = call_openrouter_for_questions(&prompt).await?;

    Ok(questions)
}

#[server]
pub async fn generate_roadmap(
    skill_name: String,
    session_token: String,
    responses: Vec<QuestionResponse>,
) -> Result<String, ServerFnError> {
    let db = get_db().await?;
    let user: User = get_user_data(session_token).await?;
    let user_id = user
        .id
        .clone()
        .ok_or(ServerFnError::new("User ID not found"))?;

    let query_variations = generate_rag_queries(&skill_name, &user, &responses);

    let relevant_resources = search_vector_db_multi_query(&query_variations).await?;

    let roadmap_nodes =
        generate_roadmap_with_llm(&skill_name, &user, &responses, &relevant_resources).await?;

    let roadmap = Roadmap {
        id: None,
        user_id: user_id.clone(),
        skill_name: skill_name.clone(),
        nodes: roadmap_nodes,
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };

    let created: Vec<Roadmap> = db
        .create("roadmaps")
        .content(roadmap)
        .await
        .into_server_error()?
        .ok_or(ServerFnError::ServerError {
            message: "Couldn't create entry".to_string(),
            code: 500,
            details: None,
        })?;

    Ok(created.first().unwrap().id.clone().unwrap())
}

#[cfg(feature = "server")]
#[allow(unused)]
fn generate_rag_queries(
    skill_name: &str,
    user: &User,
    responses: &[QuestionResponse],
) -> Vec<String> {
    let base_queries = vec![
        format!("Learn {} fundamentals", skill_name),
        format!("{} beginner tutorial", skill_name),
        format!("{} step by step guide", skill_name),
        format!("Best resources to learn {}", skill_name),
        format!("{} prerequisites and basics", skill_name),
    ];

    let mut queries = base_queries;

    if !user.skills_learned.is_empty() {
        queries.push(format!(
            "{} for developers with {} background",
            skill_name,
            user.skills_learned.join(", ")
        ));
    }

    if user.preferences.difficulty_preference == "intermediate" {
        queries.push(format!("{} intermediate concepts", skill_name));
    }

    for content_type in &user.preferences.preferred_content_types {
        queries.push(format!("{} {} tutorial", skill_name, content_type));
    }

    queries.truncate(30);
    queries
}

#[cfg(feature = "server")]
async fn search_vector_db_multi_query(queries: &[String]) -> Result<Vec<CoursesData>> {
    let db = get_db().await?;
    let mut model = TextEmbedding::try_new(InitOptions::new(MODEL))?;

    let mut all_results = std::collections::HashMap::new();

    for query in queries {
        let embedding = model.embed(vec![query.clone()], None)?;

        let mut result = db
            .query("SELECT * FROM courses WHERE embedding <|10,COSINE|> $embedding LIMIT 5")
            .bind(("embedding", embedding))
            .await?;

        let courses: Vec<CoursesData> = result.take(0)?;

        for course in courses {
            if let Some(id) = &course.id {
                all_results.insert(id.clone(), course);
            }
        }
    }

    let mut results: Vec<CoursesData> = all_results.into_values().collect();
    results.truncate(20);

    Ok(results)
}

#[cfg(feature = "server")]
async fn generate_roadmap_with_llm(
    skill_name: &str,
    user: &User,
    responses: &[QuestionResponse],
    resources: &[CoursesData],
) -> Result<Vec<RoadmapNode>> {
    let resources_json = serde_json::to_string_pretty(resources)?;

    let prompt = format!(
        "Create a detailed learning roadmap for '{}'. \n\n\
        User Profile:\n\
        - Existing skills: {:?}\n\
        - Learning preferences: {:?}\n\
        - Question responses: {:?}\n\n\
        Available Resources:\n\
        {}\n\n\
        Generate a roadmap with 8-12 nodes. Each node should have:\n\
        - skill_name: specific skill/topic to learn\n\
        - description: brief explanation (1-2 sentences)\n\
        - resources: array of relevant resources from the provided data (match by topic/skill)\n\
        - prerequisites: array of skill IDs that must be completed first\n\
        - position: {{x, y}} coordinates for visualization (arrange logically)\n\n\
        Return ONLY valid JSON array of nodes. Ensure resources are matched accurately from the provided data.",
        skill_name,
        user.skills_learned,
        user.preferences,
        responses,
        resources_json
    );

    let nodes_json = call_openrouter_for_roadmap(&prompt).await?;
    let mut nodes: Vec<RoadmapNode> = serde_json::from_str(&nodes_json)?;

    for node in &mut nodes {
        node.id = Uuid::new_v4().to_string();
        node.is_completed = false;
    }

    Ok(nodes)
}

#[cfg(feature = "server")]
async fn call_openrouter_for_questions(prompt: &str) -> Result<Vec<Question>> {
    let client = reqwest::Client::new();

    let body = serde_json::json!({
        "model": "google/gemini-2.0-flash-exp:free",
        "messages": [
            {
                "role": "user",
                "content": prompt
            }
        ],
        "response_format": { "type": "json_object" }
    });

    let response = client
        .post("https://openrouter.ai/api/v1/chat/completions")
        .header("Authorization", "Bearer YOUR_API_KEY")
        .header("Content-Type", "application/json")
        .json(&body)
        .send()
        .await?;

    let json: serde_json::Value = response.json().await?;
    let content = json["choices"][0]["message"]["content"]
        .as_str()
        .ok_or_else(|| anyhow::anyhow!("No response content"))?;

    let parsed: serde_json::Value = serde_json::from_str(content)?;
    let questions_array = parsed["questions"]
        .as_array()
        .ok_or_else(|| anyhow::anyhow!("No questions array"))?;

    let mut questions = Vec::new();
    for q in questions_array {
        questions.push(Question {
            id: Uuid::new_v4().to_string(),
            question_text: q["question_text"].as_str().unwrap_or("").to_string(),
            question_type: match q["question_type"].as_str() {
                Some("MCQ") => QuestionType::MCQ,
                Some("MSQ") => QuestionType::MSQ,
                Some("TrueFalse") => QuestionType::TrueFalse,
                _ => QuestionType::OneWord,
            },
            options: q["options"]
                .as_array()
                .map(|arr| {
                    arr.iter()
                        .filter_map(|v| v.as_str())
                        .map(String::from)
                        .collect()
                })
                .unwrap_or_default(),
        });
    }

    Ok(questions)
}

#[cfg(feature = "server")]
async fn call_openrouter_for_roadmap(prompt: &str) -> Result<String> {
    let client = reqwest::Client::new();

    let body = serde_json::json!({
        "model": "google/gemini-2.0-flash-exp:free",
        "messages": [
            {
                "role": "user",
                "content": prompt
            }
        ]
    });

    let response = client
        .post("https://openrouter.ai/api/v1/chat/completions")
        .header(
            "Authorization",
            "Bearer  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",
        )
        .header("Content-Type", "application/json")
        .json(&body)
        .send()
        .await?;

    let json: serde_json::Value = response.json().await?;
    let content = json["choices"][0]["message"]["content"]
        .as_str()
        .ok_or_else(|| anyhow::anyhow!("No response content"))?;

    Ok(content.to_string())
}

#[server]
pub async fn toggle_node_completion(
    roadmap_id: String,
    node_id: String,
) -> Result<(), ServerFnError> {
    let db = get_db().await?;

    let mut roadmap: Roadmap = db
        .select(("roadmaps", roadmap_id.clone()))
        .await
        .into_server_error()?
        .ok_or_else(|| ServerFnError::new("Roadmap not found"))?;

    if let Some(node) = roadmap.nodes.iter_mut().find(|n| n.id == node_id) {
        node.is_completed = !node.is_completed;
    }

    roadmap.updated_at = Utc::now();

    let _: Option<Roadmap> = db
        .update(("roadmaps", roadmap_id))
        .content(roadmap)
        .await
        .into_server_error()?;

    Ok(())
}
