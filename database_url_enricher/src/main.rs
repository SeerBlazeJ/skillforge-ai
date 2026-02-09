use anyhow::Result; // Removed unused 'Context'
use dashmap::DashMap;
use flate2::read::GzDecoder;
use futures::stream::{self, StreamExt};
use quick_xml::events::Event;
use quick_xml::reader::Reader;
use rand::Rng;
use reqwest::Client;
use scraper::{Html, Selector};
use serde::{Deserialize, Serialize};
use std::io::{BufReader, Read};
use std::sync::Arc;
use std::time::Duration;
use surrealdb::Surreal;
use surrealdb::engine::local::{Db, RocksDb};
use surrealdb::sql::Thing as RecordId;
use tokio::sync::Semaphore;

// --- DATA STRUCTURES ---

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Courses {
    pub id: Option<RecordId>,
    pub title: String,
    pub channel_name: String,
    pub ctype: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
}

struct SitemapKnowledgeBase {
    catalog: DashMap<(String, String), String>,
}

impl SitemapKnowledgeBase {
    fn new() -> Self {
        Self {
            catalog: DashMap::new(),
        }
    }

    async fn hydrate(&self, client: &Client) {
        println!("⬇️  Phase 1: Hydrating Knowledge Base from Sitemaps...");

        // 1. Coursera
        self.fetch_coursera(client).await;

        // 2. edX
        self.fetch_generic_sitemap(client, "edx", "https://www.edx.org/sitemap.xml")
            .await;

        // 3. Udacity
        self.fetch_generic_sitemap(client, "udacity", "https://www.udacity.com/sitemap.xml")
            .await;

        println!(
            "✅ Knowledge Base Hydrated. Total Entries: {}\n",
            self.catalog.len()
        );
    }

    async fn fetch_coursera(&self, client: &Client) {
        println!("   ...Fetching Coursera Index");
        let index_url = "https://www.coursera.org/sitemap.xml";

        let response_text = match client.get(index_url).send().await {
            Ok(resp) => match resp.text().await {
                Ok(text) => text,
                Err(_) => return,
            },
            Err(_) => return,
        };

        let mut reader = Reader::from_str(&response_text);
        let mut buf = Vec::new();

        // State machine for Index parsing
        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Start(e)) if e.name().as_ref() == b"loc" => {
                    // For string slices, read_text works fine, but let's be consistent
                    if let Ok(url) = reader.read_text(e.name()) {
                        if url.contains("sitemap~www~courses.xml") {
                            println!("   ...Downloading Course Sub-map: {}", url);
                            self.fetch_compressed_sitemap(client, "coursera", &url)
                                .await;
                        }
                    }
                }
                Ok(Event::Eof) => break,
                _ => (),
            }
            buf.clear();
        }
    }

    async fn fetch_generic_sitemap(&self, client: &Client, channel: &str, url: &str) {
        println!("   ...Fetching {} sitemap", channel);
        if let Ok(resp) = client.get(url).send().await {
            if let Ok(bytes) = resp.bytes().await {
                if url.ends_with(".gz") {
                    let decoder = GzDecoder::new(&bytes[..]);
                    self.parse_sitemap_reader(channel, decoder);
                } else {
                    self.parse_sitemap_reader(channel, &bytes[..]);
                }
            }
        }
    }

    async fn fetch_compressed_sitemap(&self, client: &Client, channel: &str, url: &str) {
        if let Ok(resp) = client.get(url).send().await {
            if let Ok(bytes) = resp.bytes().await {
                let decoder = GzDecoder::new(&bytes[..]);
                self.parse_sitemap_reader(channel, decoder);
            }
        }
    }

    // FIXED: Robust state-machine parser for streams
    fn parse_sitemap_reader<R: Read>(&self, channel: &str, reader: R) {
        let mut reader = Reader::from_reader(BufReader::new(reader));
        let mut buf = Vec::new();
        let mut current_url = String::new();
        let mut in_loc = false;

        loop {
            match reader.read_event_into(&mut buf) {
                // Enter <loc>
                Ok(Event::Start(e)) if e.name().as_ref() == b"loc" => {
                    in_loc = true;
                    current_url.clear(); // Reset buffer
                }
                // Capture Text inside <loc>...</loc>
                Ok(Event::Text(e)) if in_loc => {
                    if let Ok(txt) = e.unescape() {
                        current_url = txt.into_owned();
                    }
                }
                // Exit </loc>
                Ok(Event::End(e)) if e.name().as_ref() == b"loc" => {
                    in_loc = false;
                }
                // End of Entry </url> -> Save to Catalog
                Ok(Event::End(e)) if e.name().as_ref() == b"url" => {
                    if !current_url.is_empty() {
                        let slug = self.extract_slug(&current_url);
                        self.catalog
                            .insert((channel.to_string(), slug), current_url.clone());
                    }
                }
                Ok(Event::Eof) => break,
                Err(_) => break,
                _ => (),
            }
            buf.clear();
        }
    }

    fn find_match(&self, channel: &str, title: &str) -> Option<String> {
        let channel_key = channel.to_lowercase();
        let target_slug = self.extract_slug(title);

        if let Some(entry) = self
            .catalog
            .get(&(channel_key.clone(), target_slug.clone()))
        {
            return Some(entry.value().clone());
        }
        None
    }

    fn extract_slug(&self, text: &str) -> String {
        text.to_lowercase()
            .split('/')
            .last()
            .unwrap_or(text)
            .split('?')
            .next()
            .unwrap_or(text)
            .replace("-", " ")
            .trim()
            .replace(" ", "-")
            .chars()
            .filter(|c| c.is_alphanumeric() || *c == '-')
            .collect()
    }
}

// --- MAIN ENGINE ---

struct AutoEnricher {
    client: Client,
    db: Arc<Surreal<Db>>,
    semaphore: Arc<Semaphore>,
    kb: Arc<SitemapKnowledgeBase>,
}

impl AutoEnricher {
    async fn new(db_path: &str, max_concurrent: usize) -> Result<Self> {
        let db = Surreal::new::<RocksDb>(db_path).await?;
        db.use_ns("main").use_db("main").await?;

        let client = Client::builder()
            .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/121.0.0.0 Safari/537.36")
            .timeout(Duration::from_secs(15))
            .redirect(reqwest::redirect::Policy::limited(10))
            .build()?;

        let kb = Arc::new(SitemapKnowledgeBase::new());
        kb.hydrate(&client).await;

        Ok(Self {
            client,
            db: Arc::new(db),
            semaphore: Arc::new(Semaphore::new(max_concurrent)),
            kb,
        })
    }

    async fn run(&self) -> Result<()> {
        println!("🚀 Phase 2: Processing Database...");

        let courses = self.fetch_pending_courses().await?;
        let total = courses.len();
        println!("📊 Processing {} courses.", total);

        let counter = Arc::new(tokio::sync::Mutex::new(0));

        stream::iter(courses)
            .map(|course| {
                let engine = self.clone();
                let counter = counter.clone();

                tokio::spawn(async move {
                    let _permit = engine.semaphore.acquire().await.unwrap();
                    let mut found_url: Option<String> = None;

                    // 1. Sitemap Lookup
                    if let Some(url) = engine.kb.find_match(&course.channel_name, &course.title) {
                        found_url = Some(url);
                    }

                    // 2. YouTube API
                    if found_url.is_none() && engine.is_youtube(&course) {
                        found_url = engine.search_youtube_api(&course).await;
                    }

                    // 3. Stealth Scraper (Fallback)
                    if found_url.is_none() {
                        let jitter = rand::thread_rng().gen_range(500..2000);
                        tokio::time::sleep(Duration::from_millis(jitter)).await;
                        found_url = engine.stealth_scrape(&course).await;
                    }

                    // Update DB
                    if let Some(url) = found_url {
                        engine.update_db(&course, &url).await;
                        println!(
                            "✅ FIXED: {} -> {}",
                            &course.title[..20.min(course.title.len())],
                            url
                        );
                    }

                    let mut c = counter.lock().await;
                    *c += 1;
                    if *c % 50 == 0 {
                        println!("📈 Progress: {}/{}", *c, total);
                    }
                })
            })
            .buffer_unordered(50)
            .collect::<Vec<_>>()
            .await;

        println!("✨ All Done!");
        Ok(())
    }

    async fn update_db(&self, course: &Courses, url: &str) {
        if let Some(id) = &course.id {
            let _ = self
                .db
                .update::<Option<Courses>>((id.tb.as_str(), id.id.to_string()))
                .merge(serde_json::json!({ "url": url }))
                .await;
        }
    }

    fn is_youtube(&self, course: &Courses) -> bool {
        let c = course.channel_name.to_lowercase();
        c.contains("youtube") || c.contains("yt") || course.ctype == "video"
    }

    async fn search_youtube_api(&self, course: &Courses) -> Option<String> {
        let query = format!("{} {}", course.channel_name, course.title);
        let url = format!(
            "https://www.youtube.com/results?search_query={}",
            urlencoding::encode(&query)
        );

        match self.client.get(&url).send().await {
            Ok(resp) => {
                if let Ok(text) = resp.text().await {
                    if let Some(start) = text.find("\"videoId\":\"") {
                        let remainder = &text[start + 11..];
                        if let Some(end) = remainder.find("\"") {
                            return Some(format!(
                                "https://www.youtube.com/watch?v={}",
                                &remainder[..end]
                            ));
                        }
                    }
                }
            }
            Err(_) => {}
        }
        None
    }

    async fn stealth_scrape(&self, course: &Courses) -> Option<String> {
        if let Some(url) = self.guess_direct_url(course).await {
            return Some(url);
        }

        let query = format!(
            "site:{} {}",
            self.get_domain(&course.channel_name),
            course.title
        );
        let url = format!(
            "https://lite.duckduckgo.com/lite/?q={}",
            urlencoding::encode(&query)
        );

        if let Ok(resp) = self.client.get(&url).send().await {
            if let Ok(html) = resp.text().await {
                let doc = Html::parse_document(&html);
                let selector = Selector::parse(".result-link").unwrap();

                for element in doc.select(&selector) {
                    if let Some(href) = element.value().attr("href") {
                        if !href.contains("duckduckgo") && !href.contains("google") {
                            return Some(href.to_string());
                        }
                    }
                }
            }
        }
        None
    }

    async fn guess_direct_url(&self, course: &Courses) -> Option<String> {
        let slug = self.kb.extract_slug(&course.title);
        let channel = course.channel_name.to_lowercase();
        let candidates = match channel.as_str() {
            "udemy" => vec![format!("https://www.udemy.com/course/{}/", slug)],
            _ => vec![],
        };
        for url in candidates {
            if self.verify(&url).await {
                return Some(url);
            }
        }
        None
    }

    async fn verify(&self, url: &str) -> bool {
        match self.client.get(url).send().await {
            Ok(r) => r.status().is_success(),
            Err(_) => false,
        }
    }

    fn get_domain(&self, channel: &str) -> &str {
        match channel.to_lowercase().as_str() {
            "coursera" => "coursera.org",
            "udemy" => "udemy.com",
            "edx" => "edx.org",
            _ => "google.com",
        }
    }

    async fn fetch_pending_courses(&self) -> Result<Vec<Courses>> {
        let mut response = self.db
            .query("SELECT * FROM courses WHERE url CONTAINS 'google.com/search' OR url = NONE OR url = ''")
            .await?;
        Ok(response.take(0).unwrap_or_default())
    }

    fn clone(&self) -> Self {
        Self {
            client: self.client.clone(),
            db: self.db.clone(),
            semaphore: self.semaphore.clone(),
            kb: self.kb.clone(),
        }
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let enricher = AutoEnricher::new("skillforge", 50).await?;
    enricher.run().await?;
    Ok(())
}
