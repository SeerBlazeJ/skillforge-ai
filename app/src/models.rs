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
    #[serde(default)]
    pub skills_learned: Vec<String>,
    #[serde(default)]
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

#[cfg(feature = "server")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    pub id: Option<RecordId>,
    pub user_id: String,
    pub session_token: String,
    pub created_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Roadmap {
    pub id: Option<String>,
    pub user_id: String,
    pub skill_name: String,
    pub nodes: Vec<RoadmapNode>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[cfg(feature = "server")]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RoadmapDB {
    pub id: Option<RecordId>,
    pub user_id: String,
    pub skill_name: String,
    pub nodes: Vec<RoadmapNode>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[cfg(feature = "server")]
impl From<RoadmapDB> for Roadmap {
    fn from(value: RoadmapDB) -> Self {
        Self {
            id: value.id.map(|r| r.to_string()),
            user_id: value.user_id,
            skill_name: value.skill_name,
            nodes: value.nodes,
            created_at: value.created_at,
            updated_at: value.updated_at,
        }
    }
}

#[cfg(feature = "server")]
impl From<Roadmap> for RoadmapDB {
    fn from(value: Roadmap) -> Self {
        Self {
            id: value.id.and_then(|s| s.parse().ok()),
            user_id: value.user_id,
            skill_name: value.skill_name,
            nodes: value.nodes,
            created_at: value.created_at,
            updated_at: value.updated_at,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RoadmapNode {
    #[serde(default)]
    pub id: String,
    pub skill_name: String,
    pub description: String,
    #[serde(default)]
    pub resources: Vec<LearningResource>,
    #[serde(default)]
    pub prerequisites: Vec<String>,
    #[serde(default)]
    pub is_completed: bool,
    pub prev_node_id: Option<String>,
    pub next_node_id: Option<String>,
}

/*
#[cfg(feature = "server")]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RoadmapNodeDB {
    pub id: Option<RecordId>,
    pub skill_name: String,
    pub description: String,
    #[serde(default)]
    pub resources: Vec<LearningResource>,
    #[serde(default)]
    pub prerequisites: Vec<String>,
    #[serde(default)]
    pub is_completed: bool,
    #[serde(default)]
    pub position: NodePosition,
}

#[cfg(feature = "server")]
impl From<RoadmapNodeDB> for RoadmapNode {
    fn from(value: RoadmapNodeDB) -> Self {
        Self {
            id: value.id.map(|r| r.to_string()),
            skill_name: value.skill_name,
            description: value.description,
            resources: value.resources,
            prerequisites: value.prerequisites,
            is_completed: value.is_completed,
            position: value.position,
        }
    }
}

#[cfg(feature = "server")]
impl From<RoadmapNode> for RoadmapNodeDB {
    fn from(value: RoadmapNode) -> Self {
        Self {
            id: value.id.and_then(|s| s.parse().ok()),
            skill_name: value.skill_name,
            description: value.description,
            resources: value.resources,
            prerequisites: value.prerequisites,
            is_completed: value.is_completed,
            position: value.position,
        }
    }
}
*/
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
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

#[cfg(feature = "server")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoursesDataWithEmbeddings {
    pub id: Option<RecordId>,
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
    pub embedding: Vec<f32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]

pub struct CoursesDataClean {
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
}

#[cfg(feature = "server")]
impl From<CoursesDataWithEmbeddings> for CoursesDataClean {
    fn from(value: CoursesDataWithEmbeddings) -> Self {
        CoursesDataClean {
            title: value.title,
            description: value.description,
            channel_name: value.channel_name,
            published_date: value.published_date,
            skill_path: value.skill_path,
            level: value.level,
            ctype: value.ctype,
            content: value.content,
            topic: value.topic,
            prerequisite_topics: value.prerequisite_topics,
        }
    }
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

#[derive(Serialize, Deserialize)]
pub struct JsonData {
    pub title: String,
    pub description: String,
    pub channel_name: String,
    pub published_date: String,
    pub skill_path: String,
    pub level: String,
    #[serde(rename = "type")]
    pub ctype: String,
    pub content: String,
    pub topic: String,
    pub prerequisite_topics: Vec<String>,
}

#[derive(Serialize, Deserialize)]
pub struct JsonDataCollection {
    pub data: Vec<JsonData>,
}
