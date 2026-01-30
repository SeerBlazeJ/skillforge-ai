#[cfg(feature = "server")]
use anyhow::{Context, Result};
use dioxus::prelude::*;
use std::str::FromStr;
use std::{env, fs::File, io::BufReader};

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
use crate::{LOAD_AND_EMBED_JSON, SESSION_DURATION_DAYS};

#[cfg(feature = "server")]
const MODEL: EmbeddingModel = EmbeddingModel::ModernBertEmbedLarge;
const LLM_MODEL: &str = "tngtech/deepseek-r1t2-chimera:free";

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

            db.query("DEFINE TABLE sessions;").await?;
            db.query("DEFINE FIELD user_id ON sessions TYPE string;")
                .await?;
            db.query("DEFINE FIELD session_token ON sessions TYPE string;")
                .await?;
            db.query("DEFINE FIELD created_at ON sessions TYPE string;")
                .await?;
            db.query("DEFINE FIELD expires_at ON sessions TYPE string;")
                .await?;
            db.query("DEFINE INDEX unique_session_token ON sessions FIELDS session_token UNIQUE;")
                .await?;

            db.query("DEFINE TABLE roadmaps;").await?;
            db.query("DEFINE FIELD user_id ON roadmaps TYPE string;")
                .await?;
            db.query("DEFINE FIELD skill_name ON roadmaps TYPE string;")
                .await?;

    if LOAD_AND_EMBED_JSON {
                let file = File::open("../final_data.json")
            .context("Failed to read file '../final_data.json' ")?;
        let reader = BufReader::new(file);
        let collection: JsonDataCollection =
            serde_json::from_reader(reader).context("Couldn't parse data properly")?;
        let mut model = TextEmbedding::try_new(InitOptions::new(MODEL))?;
        let data_len = collection.data.len();
        for (i, data) in collection.data.into_iter().enumerate() {
            println!("Processing and storing: {i} / {data_len}");
            let str_to_embed = format!(
                "Title: {}, topic: {}, description: {}, content: {}, Skill Path: {}, Prerequisites: {}, level: {}, Topic Size : {}",
                data.title,
                data.topic,
                data.description,
                data.content,
                data.skill_path,
                data.prerequisite_topics.join(", "),
                data.level,
                data.ctype
            );
            let embedding_batch = model.embed(vec![str_to_embed], None)?;
            let embedding = embedding_batch
            .into_iter()
            .next()
            .ok_or_else(|| anyhow::anyhow!("Empty embedding returned"))?;
            let data_to_insert = CoursesDataWithEmbeddings {
                id: None,
                title: data.title.clone(),
                description: data.description,
                topic: data.topic,
                prerequisite_topics: data.prerequisite_topics,
                channel_name: data.channel_name,
                published_date: data.published_date,
                skill_path: data.skill_path,
                level: data.level,
                ctype: data.ctype,
                content: data.content,
                embedding,
            };
            let res: Option<CoursesDataWithEmbeddings> = db.create("courses").content(data_to_insert).await?;
            match res {
                Some(_) => {}
                None => println!("Failed creating entry for {}", data.title),
            }
        }

        println!("Data embedding and storage successfull");
    }
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

    let roadmaps_db: Vec<RoadmapDB> = result.take(0).into_server_error()?;
    let roadmaps: Vec<Roadmap> = roadmaps_db.into_iter().map(Roadmap::from).collect();

    Ok(roadmaps)
}

#[server]
pub async fn get_roadmap(roadmap_id: String) -> Result<Roadmap, ServerFnError> {
    let db = get_db().await?;
    let id = RecordId::from_str(&roadmap_id)
        .map_err(|_| ServerFnError::new("Could not parse RecordID"))?;
    let roadmap_db: RoadmapDB = db
        .select(id)
        .await
        .into_server_error()?
        .ok_or_else(|| ServerFnError::new("Roadmap not found"))?;
    Ok(Roadmap::from(roadmap_db))
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
    let query_variations = generate_rag_queries(&skill_name, &user, &responses)
        .await
        .into_server_error()?;
    let relevant_resources = search_vector_db_multi_query(&query_variations).await?;
    let roadmap_nodes =
        generate_roadmap_with_llm(&skill_name, &user, &responses, &relevant_resources).await?;
    eprintln!("=============== nodes generated ===========");
    let roadmap = RoadmapDB {
        id: None,
        user_id: user_id.clone(),
        skill_name: skill_name.clone(),
        nodes: roadmap_nodes,
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };

    let created: Option<RoadmapDB> = db
        .create("roadmaps")
        .content(roadmap)
        .await
        .into_server_error()?;

    Ok(created.unwrap().id.map(|r| r.to_string()).unwrap())
}

fn clean_json_response(input: &str) -> String {
    input
        .trim()
        .trim_start_matches("```json")
        .trim_start_matches("```")
        .trim_end_matches("```")
        .trim()
        .to_string()
}

#[cfg(feature = "server")]
async fn generate_rag_queries(
    skill_name: &str,
    user: &User,
    responses: &[QuestionResponse],
) -> Result<Vec<String>> {
    let client = reqwest::Client::new();
    let api_key = env::var("OPENROUTER_API_KEY").expect("OPENROUTER_API_KEY must be set");

    let sys_prompt: &str = r#"You are a Query Generation AI for an educational RAG system. Your goal is to generate 5 distinct, high-quality search queries to retrieve relevant course material based on a user's intent.

THE RAG SCHEMA:
The database contains course nodes with these fields:
- Title, Topic (Parent/Main), Description, Content
- Skill Path, Prerequisites
- Level (Beginner, Intermediate, Advanced)
- Topic Size (Macro, Micro)

INSTRUCTIONS:
1. Analyze the 'Skill to learn' and 'User Responses'.
2. If 'User Responses' indicate a specific knowledge gap or preference (e.g., "I prefer video" or "I know the basics"), prioritize that over general user preferences.
3. Formulate 5 specific semantic queries. Mix general broad searches (Macro) and specific technical searches (Micro).
4. Include specific keywords related to the 'Level' (e.g., "Beginner tutorial", "Advanced concepts") if the user context suggests it.

OUTPUT FORMAT RULES:
- Return ONLY a raw JSON array of strings.
- DO NOT use Markdown formatting (no ```json ... ```).
- DO NOT include explanations or conversational filler.

EXAMPLE INPUT:
Skill: Rust, Level: Beginner, Context: "I want to learn memory management"

EXAMPLE OUTPUT:
["Rust programming for absolute beginners", "Rust ownership and borrowing explained", "Rust memory management deep dive", "Introduction to systems programming with Rust", "Rust macro skill path basics"]
"#;

    let user_prompt = format!(
        "Skill to learn: {}\nUser Knowledge Context: {:?}\nUser Preferences: {:?}\nUser Skills: {:?}",
        skill_name, responses, user.preferences, user.skills_learned
    );
    let body = serde_json::json!({
        "model": LLM_MODEL,
        "messages": [
            {
                "role": "system",
                "content": sys_prompt,
            },
            {
                "role": "user",
                "content": user_prompt
            }
        ],
        "temperature": 0.3
    });
    let response = client
        .post("https://openrouter.ai/api/v1/chat/completions")
        .header("Authorization", format!("Bearer {api_key}"))
        .header("Content-Type", "application/json")
        .json(&body)
        .send()
        .await?;
    if !response.status().is_success() {
        let error_text = response.text().await?;
        return Err(anyhow::anyhow!("API Error: {}", error_text));
    }
    let json: serde_json::Value = response.json().await?;
    let content = json["choices"][0]["message"]["content"]
        .as_str()
        .ok_or_else(|| anyhow::anyhow!("No response content"))?;
    let content = clean_json_response(content);
    let queries: Vec<String> = serde_json::from_str(&content)
        .map_err(|e| anyhow::anyhow!("Failed to parse JSON: {} | Content: {}", e, content))?;
    Ok(queries)
}

#[cfg(feature = "server")]
async fn search_vector_db_multi_query(queries: &[String]) -> Result<Vec<CoursesDataClean>> {
    let db = get_db().await?;
    let mut model = TextEmbedding::try_new(InitOptions::new(MODEL))?;

    let mut all_results = std::collections::HashMap::new();

    for query in queries {
        let embedding_batch = model.embed(vec![query.clone()], None)?;
        let embedding = embedding_batch
            .into_iter()
            .next()
            .ok_or_else(|| anyhow::anyhow!("Empty embedding returned"))?;

        let mut result = db
            .query("SELECT * FROM courses WHERE embedding <|10,COSINE|> $embedding LIMIT 5")
            .bind(("embedding", embedding))
            .await?;
        let courses: Vec<CoursesDataWithEmbeddings> = result.take(0)?;
        for course in courses {
            if let Some(id) = &course.id {
                all_results.insert(id.clone(), course);
            }
        }
    }

    let mut results: Vec<CoursesDataClean> = all_results
        .into_values()
        .map(CoursesDataClean::from)
        .collect();
    results.truncate(20);
    eprintln!("RAG result generated");
    Ok(results)
}
#[cfg(feature = "server")]
async fn generate_roadmap_with_llm(
    skill_name: &str,
    user: &User,
    responses: &[QuestionResponse],
    resources: &[CoursesDataClean],
) -> Result<Vec<RoadmapNode>> {
    use std::collections::HashMap;

    let resources_json = serde_json::to_string_pretty(resources)?;

    let prompt = format!(
        "Create a detailed learning roadmap for '{skill_name}'.\n\n\
User Profile:\n\
- Existing skills: {:?}\n\
- Learning preferences: {:?}\n\
- Question responses: {:?}\n\n\
Available Resources:\n\
{}\n\n\
OUTPUT FORMAT (STRICT):\n\
Return ONLY valid JSON in this exact shape:\n\
{{\"nodes\": [ ... ]}}\n\
- Do NOT return a top-level array.\n\
- Do NOT wrap in markdown code fences.\n\
- Do NOT include id field. Server assigns IDs.\n\
- Do NOT include any text outside JSON.\n\n\
IMPORTANT LINKING RULES:\n\
- `prerequisites` must be an array of OTHER NODE `skill_name` strings (not IDs).\n\
- `prev_node_id` and `next_node_id` must be the adjacent node's `skill_name` (or null).\n\n\
Each node must match:\n\
{{\n\
  \"skill_name\": \"...\",\n\
  \"description\": \"...\",\n\
  \"resources\": [{{\"title\":\"...\",\"platform\":\"...\",\"url\":null,\"resource_type\":\"...\"}}],\n\
  \"prerequisites\": [\"...\"],\n\
  \"prev_node_id\": null,\n\
  \"next_node_id\": null,\n\
  \"is_completed\": false\n\
}}",
        user.skills_learned,
        user.preferences,
        responses,
        resources_json
    );

    #[derive(serde::Deserialize)]
    struct RoadmapNodesOut {
        nodes: Vec<RoadmapNode>,
    }

    let mut nodes_out: RoadmapNodesOut =
        serde_json::from_str(&call_openrouter_for_roadmap(&prompt).await?).into_server_error()?;

    for node in &mut nodes_out.nodes {
        node.id = Uuid::new_v4().to_string();
        node.is_completed = false;
    }

    let name_to_id: HashMap<String, String> = nodes_out
        .nodes
        .iter()
        .map(|n| (n.skill_name.clone(), n.id.clone()))
        .collect();

    let map_ref =
        |s: &str| -> String { name_to_id.get(s).cloned().unwrap_or_else(|| s.to_string()) };

    for node in &mut nodes_out.nodes {
        node.prerequisites = node.prerequisites.iter().map(|p| map_ref(p)).collect();

        node.prev_node_id = node
            .prev_node_id
            .as_ref()
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .map(|s| map_ref(&s));

        node.next_node_id = node
            .next_node_id
            .as_ref()
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .map(|s| map_ref(&s));
    }

    Ok(nodes_out.nodes)
}

#[cfg(feature = "server")]
async fn call_openrouter_for_questions(prompt: &str) -> Result<Vec<Question>> {
    let client = reqwest::Client::new();
    let api_key = env::var("OPENROUTER_API_KEY").expect("OPENROUTER_API_KEY must be set");
    let sys_prompt = "You are an educational assessment expert that generates personalized learning evaluation questions. \
        Your goal is to understand both HOW the user prefers to learn and WHAT they already know.\n\n\
        RESPONSE FORMAT RULES:\n\
        - Return ONLY valid JSON in this exact structure: {\"questions\": [...]}\n\
        - Never add explanatory text before or after the JSON\n\
        - Never use markdown code blocks\n\n\
        QUESTION QUALITY GUIDELINES:\n\
        1. Preference Questions (5 questions):\n\
           - Ask about learning style (visual/text/hands-on/video)\n\
           - Ask about time commitment and pacing preferences\n\
           - Ask about content depth preferences (deep-dive vs overview)\n\
           - Ask about preferred resource types (courses/books/projects/docs)\n\
           - Ask about learning goals (career/hobby/certification)\n\
        2. Knowledge Questions (3 questions):\n\
           - Assess prerequisite knowledge relevant to the skill\n\
           - Test understanding of fundamental concepts\n\
           - Gauge experience level accurately\n\n\
        QUESTION TYPES:\n\
        - MCQ: Single correct answer (4 options)\n\
        - MSQ: Multiple correct answers (4-5 options)\n\
        - TrueFalse: Binary choice (2 options: 'True', 'False')\n\
        - OneWord: Short text answer (empty options array)\n\n\
        OUTPUT SCHEMA:\n\
        {\n\
          \"questions\": [\n\
            {\n\
              \"question_text\": \"Clear, concise question\",\n\
              \"question_type\": \"MCQ\" | \"MSQ\" | \"TrueFalse\" | \"OneWord\",\n\
              \"options\": [\"Option 1\", \"Option 2\", \"Option 3\", \"Option 4\"]\n\
            }\n\
          ]\n\
        }\n\n\
        Make questions conversational, relevant to the specific skill, and ensure options are realistic and well-balanced.c";
    let body = serde_json::json!({
        "model": LLM_MODEL,
        "messages": [
            {
               "role": "system",
                "content": sys_prompt
            },
            {
                "role": "user",
                "content": prompt
            }
        ],
        "response_format": { "type": "json_object" }
    });

    let response = client
        .post("https://openrouter.ai/api/v1/chat/completions")
        .header("Authorization", format!("Bearer {api_key}"))
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
    let api_key = env::var("OPENROUTER_API_KEY").expect("OPENROUTER_API_KEY must be set");
    let system_prompt = "You are a JSON-only API. Return ONLY valid JSON with top-level object \
{\"nodes\": [...]} and nothing else. No markdown. No commentary.";

    let body = serde_json::json!({
        "model": LLM_MODEL,
        "messages": [
            {
                "role": "system",
                "content": system_prompt
            },
            {
                "role": "user",
                "content": prompt
            }
        ]
    });

    let response = client
        .post("https://openrouter.ai/api/v1/chat/completions")
        .header("Authorization", format!("Bearer {api_key}"))
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
    let id = RecordId::from_str(&roadmap_id).into_server_error()?;
    let mut roadmap: RoadmapDB = db
        .select(&id)
        .await
        .into_server_error()?
        .ok_or_else(|| ServerFnError::new("Roadmap not found"))?;

    if let Some(node) = roadmap.nodes.iter_mut().find(|n| n.id == node_id) {
        node.is_completed = !node.is_completed;
    }

    roadmap.updated_at = Utc::now();

    let _: Option<RoadmapDB> = db.update(id).content(roadmap).await.into_server_error()?;

    Ok(())
}
