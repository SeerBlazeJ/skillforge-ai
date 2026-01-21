// use std::str::FromStr;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
#[cfg(feature = "server")]
use surrealdb::RecordId;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct User {
    pub id: Option<String>,
    pub username: String,
    pub password_hash: String,
    pub name: String,
    pub skills_learned: Vec<String>,
    pub preferences: UserPreferences,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[cfg(feature = "server")]
pub struct UserDB {
    pub id: Option<RecordId>,
    pub username: String,
    pub password_hash: String,
    pub name: String,
    pub skills_learned: Vec<String>,
    pub preferences: UserPreferences,
    pub created_at: DateTime<Utc>,
}

#[cfg(feature = "server")]
impl From<User> for UserDB {
    fn from(value: User) -> Self {
        UserDB {
            id: value.id.and_then(|s| s.parse().ok()),
            username: value.username,
            password_hash: value.password_hash,
            name: value.name,
            skills_learned: value.skills_learned,
            preferences: value.preferences,
            created_at: value.created_at,
        }
    }
}

#[cfg(feature = "server")]
impl From<UserDB> for User {
    fn from(value: UserDB) -> Self {
        User {
            id: value.id.map(|rid| rid.to_string()),
            username: value.username,
            password_hash: value.password_hash,
            name: value.name,
            skills_learned: value.skills_learned,
            preferences: value.preferences,
            created_at: value.created_at,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct UserPreferences {
    pub learning_style: String,
    pub time_commitment: String,
    pub preferred_content_types: Vec<String>,
    pub difficulty_preference: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SessionInfo {
    pub user_id: String,
    pub username: String,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    pub id: Option<String>,
    pub user_id: String,
    pub session_token: String,
    pub created_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)] // Added PartialEq
pub struct Roadmap {
    pub id: Option<String>,
    pub user_id: String,
    pub skill_name: String,
    pub nodes: Vec<RoadmapNode>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RoadmapNode {
    pub id: String,
    pub skill_name: String,
    pub description: String,
    pub resources: Vec<LearningResource>,
    pub prerequisites: Vec<String>,
    pub is_completed: bool,
    pub position: NodePosition,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct NodePosition {
    pub x: i32,
    pub y: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LearningResource {
    pub title: String,
    pub platform: String,
    pub url: Option<String>,
    pub resource_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoursesData {
    pub id: Option<String>,
    pub title: String,
    pub description: String,
    pub channel_name: String,
    pub published_date: String,
    pub skill_path: String,
    pub level: String,
    pub ctype: String,
    pub content: String,
    pub topic: String,
    pub prerequisite_topics: Vec<String>,
    pub embedding: Vec<Vec<f32>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)] // Added PartialEq
pub struct Question {
    pub id: String,
    pub question_text: String,
    pub question_type: QuestionType,
    pub options: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum QuestionType {
    MCQ,
    MSQ,
    TrueFalse,
    OneWord,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuestionResponse {
    pub question_id: String,
    pub answer: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoadmapRequest {
    pub skill_name: String,
    pub user_responses: Vec<QuestionResponse>,
}
