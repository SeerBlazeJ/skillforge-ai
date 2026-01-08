use csv::ReaderBuilder;
use dotenv::dotenv;
use futures::stream::{self, StreamExt};
use indicatif::{ProgressBar, ProgressStyle};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::env;
use std::error::Error;
use std::fs::File;
use std::io::Write;
use std::sync::Arc;
use tokio::sync::Semaphore;

// 1. Define the Input Structure (Matches CSV Headers)
#[derive(Debug, Deserialize, Clone)]
struct CourseRecord {
    #[serde(rename = "Course Name")]
    name: String,

    #[serde(rename = "Course ID")]
    course_id: String,

    #[serde(rename = "Course Type")]
    course_type: String,

    #[serde(rename = "Course Category")]
    category: String,

    // This field seems to hold the bulk of the content now
    #[serde(rename = "Course Name and Summary")]
    summary: String,
}

// 2. Define the Output Structure (Matches Desired JSON)
#[derive(Debug, Serialize, Deserialize)]
struct VideoMetadata {
    video_id: String,
    title: String,
    description: String,
    channel_name: String,
    published_date: String,
    views: u32,
    likes: u32,
    duration: String,
    skill_path: String,
    level: String,
    #[serde(rename = "type")]
    video_type: String, // "type" is a reserved keyword in Rust
    content: String,
    topic: String,
    prerequisite_topics: Vec<String>,
    enhanced_with_llm: bool,
}

// Wrapper for the Final JSON Output
#[derive(Serialize)]
struct FinalOutput {
    videos: Vec<VideoMetadata>,
}

// OpenRouter Response Structure
#[derive(Deserialize)]
struct OpenRouterResponse {
    choices: Vec<Choice>,
}
#[derive(Deserialize)]
struct Choice {
    message: Message,
}
#[derive(Deserialize)]
struct Message {
    content: String,
}

const MODEL: &str = "xiaomi/mimo-v2-flash:free";
const MAX_CONCURRENT_REQUESTS: usize = 250; // Adjust based on rate limits

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    dotenv().ok();
    let api_key = env::var("OPENROUTER_API_KEY").expect("OPENROUTER_API_KEY must be set");

    // Initialize HTTP Client
    let client = Client::new();

    // Read CSV
    let file_path = "../datasets/Courses_w_Clean_Summaries.csv"; // INPUT FILE -> Modify as per your path
    let mut rdr = ReaderBuilder::new().from_path(file_path)?;

    // Collect records
    let records: Vec<CourseRecord> = rdr.deserialize().filter_map(Result::ok).collect();
    let total_records = records.len();

    println!("Found {} records. Processing...", total_records);

    // Setup Progress Bar
    let pb = ProgressBar::new(total_records as u64);
    pb.set_style(ProgressStyle::default_bar()
        .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} ({eta})")?
        .progress_chars("#>-"));

    // Semaphore for Concurrency Control
    let semaphore = Arc::new(Semaphore::new(MAX_CONCURRENT_REQUESTS));
    let client_arc = Arc::new(client);
    let api_key_arc = Arc::new(api_key);

    // Process in Parallel
    let results = stream::iter(records)
        .map(|record| {
            let client = Arc::clone(&client_arc);
            let api_key = Arc::clone(&api_key_arc);
            let permit = Arc::clone(&semaphore);
            let pb = pb.clone();

            async move {
                let _permit = permit.acquire().await.unwrap();
                let result = process_row(client, api_key, record).await;
                pb.inc(1);
                result
            }
        })
        .buffer_unordered(MAX_CONCURRENT_REQUESTS) // Run N tasks in parallel
        .collect::<Vec<_>>()
        .await;

    pb.finish_with_message("Processing complete");

    // Filter valid results
    let valid_videos: Vec<VideoMetadata> = results.into_iter().filter_map(|r| r).collect();

    // Write to JSON file
    let final_output = FinalOutput { videos: valid_videos };
    let json_data = serde_json::to_string_pretty(&final_output)?;

    let mut file = File::create("../processed_datasets/courses_processed.json")?; // Output File Path
    file.write_all(json_data.as_bytes())?;

    println!("Successfully wrote output.json");

    Ok(())
}

async fn process_row(
    client: Arc<Client>,
    api_key: Arc<String>,
    record: CourseRecord,
) -> Option<VideoMetadata> {
    // System Prompt: Modify if needed...
    let system_prompt = r#"
    You are a data processing API. You will receive raw Course data.
    Your job is to transform this into a specific JSON schema representing a Video/Course object.

    Rules:
    1. Output strictly valid JSON. No markdown, no conversational text.
    2. Infer missing fields based on context.
    3. Generate 'topic' (snake_case), 'prerequisite_topics' (array), 'skill_path', and 'duration' (ISO 8601 format like PT1H).
    4. For 'views' and 'likes', you can use the provided 'User_vote' and 'Rating' to estimate realistic numbers if needed, or stick to the input.
    5. 'video_id' should be a generated short hash.
    6. 'enhanced_with_llm' must be true.

    Schema required:
    {
      "video_id": "string",
      "title": "string",
      "description": "string",
      "channel_name": "string",
      "published_date": "YYYY-MM-DDTHH:MM:SSZ",
      "views": number,
      "likes": number,
      "duration": "string",
      "skill_path": "string",
      "level": "string",
      "type": "string (macro/micro)",
      "content": "",
      "topic": "string",
      "prerequisite_topics": ["string"],
      "enhanced_with_llm": true
    }
    "#;

    // UPDATE: We now use the new record fields here
    let user_prompt = format!(
        "Convert this record:\nTitle: {}\nID: {}\nType: {}\nCategory: {}\nFull Description: {}",
        record.name,
        record.course_id,
        record.course_type,
        record.category,
        record.summary
    );

    let payload = json!({
        "model": MODEL,
        "messages": [
            {"role": "system", "content": system_prompt},
            {"role": "user", "content": user_prompt}
        ]
    });

    // ... The rest of the function (sending request, parsing response) remains exactly the same
    match client
        .post("https://openrouter.ai/api/v1/chat/completions")
        .header("Authorization", format!("Bearer {}", api_key))
        .header("Content-Type", "application/json")
        .json(&payload)
        .send()
        .await
    {
        // ... (Keep existing error handling logic)
        Ok(resp) => {
             if let Ok(open_router_res) = resp.json::<OpenRouterResponse>().await {
                if let Some(choice) = open_router_res.choices.first() {
                    let content = &choice.message.content;
                    let clean_json = content
                        .trim()
                        .trim_start_matches("```json")
                        .trim_start_matches("```")
                        .trim_end_matches("```");

                    match serde_json::from_str::<VideoMetadata>(clean_json) {
                        Ok(video) => return Some(video),
                        Err(e) => {
                            eprintln!("JSON Parse Error for {}: {}", record.name, e); // Use record.title here
                            return None;
                        }
                    }
                }
            }
        }
        Err(e) => eprintln!("Request Error: {}", e),
    }
    None
}
