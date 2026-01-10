use anyhow::{Context, Result};

use fastembed::{EmbeddingModel, InitOptions, TextEmbedding};
use serde::{Deserialize, Serialize};
use std::{fs::File, io::BufReader};
use surrealdb::{
    Surreal,
    engine::local::{Db, RocksDb},
};

#[derive(Serialize, Deserialize)]
struct CoursesData {
    title: String,
    description: String,
    channel_name: String,
    published_date: String,
    skill_path: String,
    level: String,
    ctype: String,
    content: String,
    topic: String,
    prerequisite_topics: Vec<String>,
    embedding: Vec<Vec<f32>>,
}

#[derive(Serialize, Deserialize)]
struct JsonData {
    title: String,
    description: String,
    channel_name: String,
    published_date: String,
    skill_path: String,
    level: String,
    #[serde(rename = "type")]
    ctype: String,
    content: String,
    topic: String,
    prerequisite_topics: Vec<String>,
}

#[derive(Serialize, Deserialize)]
struct JsonDataCollection {
    data: Vec<JsonData>,
}

const MODEL: EmbeddingModel = EmbeddingModel::ModernBertEmbedLarge;
const LOAD_AND_EMBED_JSON: bool = false; // NOTE: Will also delete all the previous info

#[tokio::main]
async fn main() -> Result<()> {
    let db: Surreal<Db> = Surreal::new::<RocksDb>("skillforge")
        .await
        .context("Failed to connect to Database")?;

    // db.query("DEFINE INDEX course_search ON courses FIELDS embedding HNSW DIMENSION 1024 DISTANCE COSINE EFC 200 M 16;")
    // .await?; -> For Performance optimisation
    db.use_ns("main")
        .use_db("main")
        .await
        .context("Couldn't connect to namespace and/or database")?;

    if LOAD_AND_EMBED_JSON {
        db.query("DELETE courses;").await?; // Comment out if you do NOT want to clear the table while processing the new data.
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
            let embedding = model.embed(vec![str_to_embed], None)?;
            let data_to_insert = CoursesData {
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
            let res: Option<CoursesData> = db.create("courses").content(data_to_insert).await?;
            match res {
                Some(_) => {}
                None => println!("Failed creating entry for {}", data.title),
            }
        }

        println!("Data embedding and storage successfull");
    }
    Ok(())
}
