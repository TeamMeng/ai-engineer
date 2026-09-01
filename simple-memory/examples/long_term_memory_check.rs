use anyhow::{Context, Result};
use fastembed::{EmbeddingModel, TextEmbedding, TextInitOptions};

#[derive(Debug, Clone)]
struct Memory {
    text: String,
    vector: Vec<f32>,
}

#[derive(Debug, Clone)]
struct Hit {
    text: String,
    score: f32,
}

struct MemoryStore {
    model: TextEmbedding,
    memories: Vec<Memory>,
}

impl MemoryStore {
    fn new() -> Result<Self> {
        let options =
            TextInitOptions::new(EmbeddingModel::BGESmallZHV15).with_show_download_progress(true);

        let model = TextEmbedding::try_new(options).context("初始化 fastembed 失败")?;

        Ok(Self {
            model,
            memories: Vec::new(),
        })
    }

    fn remember(&mut self, text: impl Into<String>) -> Result<()> {
        let text = text.into();

        let vector = self.embed_one(&text)?;

        self.memories.push(Memory { text, vector });

        Ok(())
    }

    pub fn recall(&mut self, query: &str, top_k: usize) -> Result<Vec<Hit>> {
        if top_k == 0 {
            return Ok(Vec::new());
        }

        let query_vector = self.embed_one(query)?;

        let mut hits = self
            .memories
            .iter()
            .map(|memory| Hit {
                text: memory.text.clone(),
                score: cosine(&query_vector, &memory.vector),
            })
            .collect::<Vec<_>>();

        hits.sort_by(|left, right| right.score.total_cmp(&left.score));

        hits.truncate(top_k);

        Ok(hits)
    }

    fn embed_one(&mut self, text: &str) -> Result<Vec<f32>> {
        self.model
            .embed(vec![text.to_owned()], None)?
            .into_iter()
            .next()
            .context("fastembed 没有返回向量")
    }
}

fn cosine(left: &[f32], right: &[f32]) -> f32 {
    assert_eq!(left.len(), right.len());

    let dot: f32 = left.iter().zip(right).map(|(a, b)| a * b).sum();

    let left_norm = left.iter().map(|value| value * value).sum::<f32>().sqrt();

    let right_norm = right.iter().map(|value| value * value).sum::<f32>().sqrt();

    if left_norm == 0.0 || right_norm == 0.0 {
        return 0.0;
    }

    dot / (left_norm * right_norm)
}

fn main() -> Result<()> {
    let mut store = MemoryStore::new()?;

    store.remember("用户偏好：沟通风格简洁")?;
    store.remember("用户住在上海")?;
    store.remember("用户喜欢上午开会")?;

    let hits = store.recall("用户喜欢什么样的沟通方式？", 2)?;

    for (rank, hit) in hits.iter().enumerate() {
        println!("Top-{} score={:.4} {}", rank + 1, hit.score, hit.text);
    }

    Ok(())
}
