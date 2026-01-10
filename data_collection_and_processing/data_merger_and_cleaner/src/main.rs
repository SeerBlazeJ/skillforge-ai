use anyhow::{Context, Result};
use dotenv::dotenv;
use futures::stream::{self, StreamExt};
use glob::glob;
use indicatif::{ProgressBar, ProgressStyle};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::env;
use std::fs::File;
use std::io::{BufReader, BufWriter};
use std::sync::Arc;
use tokio::sync::Semaphore;

// --- Data Structures ---

// 1. Input Structure (Matches your source JSON perfectly)
#[derive(Debug, Deserialize, Serialize, Clone)]
struct InputVideo {
    video_id: String,
    title: String,
    description: String,
    channel_name: String,
    published_date: String,
    // Fields we want to ignore in output but need for reading
    #[allow(dead_code)]
    views: Option<u64>, // Option handles cases where they might be missing/null
    #[allow(dead_code)]
    likes: Option<u64>,
    #[allow(dead_code)]
    duration: Option<String>,
    skill_path: Option<String>,
    level: Option<String>,
    #[serde(rename = "type")]
    video_type: Option<String>,
    content: Option<String>,
    topic: Option<String>,
    prerequisite_topics: Option<Vec<String>>,
    #[allow(dead_code)]
    enhanced_with_llm: Option<bool>,
}

// 2. Output Structure (Cleaned, minimal version)
#[derive(Debug, Serialize)]
struct OutputVideo {
    video_id: String,
    title: String,
    description: String,
    channel_name: String,
    published_date: String,
    skill_path: Option<String>,
    level: Option<String>,
    #[serde(rename = "type")]
    video_type: Option<String>,
    content: Option<String>,
    topic: Option<String>,
    prerequisite_topics: Option<Vec<String>>,
}

// Conversion logic
impl From<InputVideo> for OutputVideo {
    fn from(v: InputVideo) -> Self {
        OutputVideo {
            video_id: v.video_id,
            title: v.title,
            description: v.description,
            channel_name: v.channel_name,
            published_date: v.published_date,
            skill_path: v.skill_path,
            level: v.level,
            video_type: v.video_type,
            content: v.content,
            topic: v.topic,
            prerequisite_topics: v.prerequisite_topics,
        }
    }
}

// Wrapper for reading/writing files
#[derive(Debug, Deserialize, Serialize)]
struct VideoCollection {
    videos: Vec<InputVideo>,
}

#[derive(Debug, Serialize)]
struct OutputCollection {
    #[serde(rename="Data")]
    videos: Vec<OutputVideo>,
}

// AI Response Structure
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

#[derive(Deserialize)]
struct QualityCheck {
    valid: bool,
    #[allow(dead_code)]
    reason: Option<String>,
}

// --- Configuration ---
const INPUT_DIR: &str = "../processed_datasets/*.json"; // Change this to your directory
const OUTPUT_FILE: &str = "final_data.json";
const MODEL: &str = "xiaomi/mimo-v2-flash:free";
const MAX_CONCURRENT_REQUESTS: usize = 750; // Adjust based on rate limits

#[tokio::main]
async fn main() -> Result<()> {
    dotenv().ok();
    let api_key = env::var("OPENROUTER_API_KEY").context("OPENROUTER_API_KEY not set")?;

    println!("🚀 Starting Data Processor...");

    // 1. Read and aggregate all JSON files
    let mut all_videos: Vec<InputVideo> = Vec::new();
    let mut seen_ids: HashSet<String> = HashSet::new();

    let paths: Vec<_> = glob(INPUT_DIR)?.filter_map(Result::ok).collect();
    println!("found {} files. Reading...", paths.len());

    for path in paths {
        let file = File::open(&path)?;
        let reader = BufReader::new(file);

        // We use serde_json::from_reader.
        // Note: For truly massive files, we might need a streaming parser,
        // but standard huge files (up to few hundred MBs) fit in RAM fine.
        let collection: VideoCollection = serde_json::from_reader(reader)?;2

        for video in collection.videos {
            // Deduplication Logic: Check ID immediately
            if !seen_ids.contains(&video.video_id) {
                seen_ids.insert(video.video_id.clone());
                all_videos.push(video);
            }
        }
    }

    println!("Total unique videos loaded: {}", all_videos.len());

    // 2. Setup Parallel Processing
    let client = Client::new();
    let semaphore = Arc::new(Semaphore::new(MAX_CONCURRENT_REQUESTS));
    let pb = ProgressBar::new(all_videos.len() as u64);
    pb.set_style(ProgressStyle::default_bar()
        .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} ({eta})")?
        .progress_chars("#>-"));

    let client_arc = Arc::new(client);
    let api_key_arc = Arc::new(api_key);

    // 3. Process Stream
    let processed_results = stream::iter(all_videos)
        .map(|video| {
            let client = Arc::clone(&client_arc);
            let api_key = Arc::clone(&api_key_arc);
            let permit = Arc::clone(&semaphore);
            let pb = pb.clone();

            async move {
                let _permit = permit.acquire().await.unwrap(); // Limit concurrency
                let result = process_video(client, api_key, video).await;
                pb.inc(1);
                result
            }
        })
        .buffer_unordered(MAX_CONCURRENT_REQUESTS) // Run in parallel
        .collect::<Vec<_>>()
        .await;

    pb.finish_with_message("Processing complete!");

    // 4. Collect Valid Results
    let final_videos: Vec<OutputVideo> = processed_results
        .into_iter()
        .filter_map(|res| res) // Remove Nones (filtered out videos)
        .collect();

    println!("Final dataset size after cleaning: {}", final_videos.len());

    // 5. Write to Output
    let output_file = File::create(OUTPUT_FILE)?;
    let writer = BufWriter::new(output_file);
    let output_collection = OutputCollection { videos: final_videos };
    serde_json::to_writer_pretty(writer, &output_collection)?;

    println!("✅ Successfully saved to {}", OUTPUT_FILE);

    Ok(())
}

async fn process_video(
    client: Arc<Client>,
    api_key: Arc<String>,
    video: InputVideo,
) -> Option<OutputVideo> {
    // A. Basic Heuristic Checks (Save API calls for obviously bad data)
    if video.title.trim().is_empty() || video.description.trim().len() < 10 {
        return None;
    }

    // B. AI Quality Check
    let is_valid = check_quality_with_llm(&client, &api_key, &video).await;

    if is_valid {
        // C. Transformation (Remove unwanted fields)
        Some(OutputVideo::from(video))
    } else {
        None
    }
}

async fn check_quality_with_llm(
    client: &Client,
    api_key: &str,
    video: &InputVideo,
) -> bool {
    // Construct a lightweight prompt
    let prompt = format!(
        "Analyze this video metadata for a dataset. \
        Title: '{}'. \
        Description: '{}'. \
        Topic: '{:?}'. \
        Is this entry unambiguous, educational, and of acceptable quality? \
        Reject if it is spam, placeholder text, or completely ambiguous (e.g., title is just 'Video 1'). \
        Respond ONLY with valid JSON: {{\"valid\": true}} or {{\"valid\": false}}",
        video.title,
        video.description.chars().take(200).collect::<String>(), // Truncate desc to save tokens
        video.topic
    );

    let payload = serde_json::json!({
        "model": MODEL,
        "messages": [
            {"role": "user", "content": prompt}
        ],
        "response_format": { "type": "json_object" } // Force JSON if supported, otherwise prompt handles it
    });

    match client.post("https://openrouter.ai/api/v1/chat/completions")
        .header("Authorization", format!("Bearer {}", api_key))
        .header("Content-Type", "application/json")
        .json(&payload)
        .send()
        .await
    {
        Ok(resp) => {
            if let Ok(json_resp) = resp.json::<OpenRouterResponse>().await {
                if let Some(choice) = json_resp.choices.first() {
                    let content = &choice.message.content;
                    // Attempt to parse the boolean verdict
                    if let Ok(verdict) = serde_json::from_str::<QualityCheck>(content) {
                        return verdict.valid;
                    }
                    // Fallback cleanup if model outputs markdown code blocks
                    let clean = content.replace("```json", "").replace("```", "");
                    if let Ok(verdict) = serde_json::from_str::<QualityCheck>(&clean) {
                        return verdict.valid;
                    }
                }
            }
        }
        Err(e) => eprintln!("API Error for {}: {}", video.video_id, e),
    }

    // Default to true if API fails? Or false?
    // Usually better to fail open (true) to avoid losing data on network blips,
    // or implement retry logic. Here we default to true to be safe.
    true
}
