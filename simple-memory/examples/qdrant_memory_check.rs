use std::time::Duration;

use anyhow::{Context, Result, ensure};
use fastembed::{EmbeddingModel, TextEmbedding, TextInitOptions};
use qdrant_client::{
    Payload, Qdrant,
    qdrant::{
        Condition, CreateCollectionBuilder, Distance, Filter, PointStruct, QueryPointsBuilder,
        ScoredPoint, UpsertPointsBuilder, Value, VectorParamsBuilder,
    },
};

const COLLECTION: &str = "long_term_memory";
const VECTOR_DIM: usize = 512;

struct Memory {
    id: u64,
    user_id: String,
    text: String,
    importance: f32,
}

impl Memory {
    fn new(id: u64, user_id: impl Into<String>, text: impl Into<String>, importance: f32) -> Self {
        Self {
            id,
            user_id: user_id.into(),
            text: text.into(),
            importance,
        }
    }

    fn payload(&self) -> Payload {
        Payload::from([
            ("user_id", Value::from(self.user_id.clone())),
            ("text", Value::from(self.text.clone())),
            ("importance", Value::from(self.importance)),
        ])
    }
}

fn embed_one(model: &mut TextEmbedding, text: &str) -> Result<Vec<f32>> {
    model
        .embed(vec![text.to_owned()], None)?
        .into_iter()
        .next()
        .context("fastembed 没有返回向量")
}

async fn ensure_collection(client: &Qdrant) -> Result<()> {
    if !client.collection_exists(COLLECTION).await? {
        client
            .create_collection(CreateCollectionBuilder::new(COLLECTION).vectors_config(
                VectorParamsBuilder::new(VECTOR_DIM as u64, Distance::Cosine),
            ))
            .await?;
    }

    Ok(())
}

async fn remember(client: &Qdrant, model: &mut TextEmbedding, memories: &[Memory]) -> Result<()> {
    ensure!(!memories.is_empty(), "不能写入空记忆");

    let texts: Vec<String> = memories.iter().map(|memory| memory.text.clone()).collect();

    let vectors = model.embed(texts, None)?;

    ensure!(
        vectors.len() == memories.len(),
        "向量数量不匹配：{} != {}",
        vectors.len(),
        memories.len()
    );

    ensure!(
        vectors.iter().all(|vector| vector.len() == VECTOR_DIM),
        "向量维度不是 {}",
        VECTOR_DIM
    );

    ensure_collection(client).await?;

    let points: Vec<PointStruct> = memories
        .iter()
        .zip(vectors)
        .map(|(memory, vector)| PointStruct::new(memory.id, vector, memory.payload()))
        .collect();

    client
        .upsert_points(UpsertPointsBuilder::new(COLLECTION, points).wait(true))
        .await?;

    Ok(())
}

async fn recall(
    client: &Qdrant,
    model: &mut TextEmbedding,
    user_id: &str,
    query: &str,
    top_k: u64,
) -> Result<Vec<ScoredPoint>> {
    if top_k == 0 {
        return Ok(Vec::new());
    }

    let query_vector = embed_one(model, query)?;

    let response = client
        .query(
            QueryPointsBuilder::new(COLLECTION)
                .query(query_vector)
                .limit(top_k)
                .filter(Filter::must([Condition::matches(
                    "user_id",
                    user_id.to_owned(),
                )]))
                .with_payload(true),
        )
        .await?;

    Ok(response.result)
}

#[tokio::main]
async fn main() -> Result<()> {
    // Qdrant Rust 客户端使用 gRPC 端口 6334。
    let client = Qdrant::from_url("http://localhost:6334")
        .timeout(Duration::from_secs(10))
        .build()
        .context("连接 Qdrant 失败")?;

    let options =
        TextInitOptions::new(EmbeddingModel::BGESmallZHV15).with_show_download_progress(true);

    let mut model = TextEmbedding::try_new(options).context("初始化 fastembed 失败")?;

    let memories = vec![
        Memory::new(1, "user-001", "用户偏好：沟通风格简洁", 0.9),
        Memory::new(2, "user-001", "用户喜欢上午开会", 0.7),
        Memory::new(3, "user-002", "用户住在上海", 0.8),
    ];

    remember(&client, &mut model, &memories).await?;
    println!("已写入 {} 条记忆", memories.len());

    let hits = recall(
        &client,
        &mut model,
        "user-001",
        "用户喜欢什么样的沟通方式？",
        2,
    )
    .await?;

    println!("\n检索结果：");

    for (rank, point) in hits.into_iter().enumerate() {
        let text = point
            .payload
            .get("text")
            .and_then(Value::as_str)
            .cloned()
            .unwrap_or_else(|| "<缺少 text>".to_owned());

        println!(
            "Top-{} score={:.4} id={:?} {}",
            rank + 1,
            point.score,
            point.id,
            text
        );
    }

    Ok(())
}
