use anyhow::Result;
use std::collections::HashMap;
use usearch::{Index, IndexOptions, MetricKind, ScalarKind};

#[derive(Debug, Clone, PartialEq)]
pub struct SearchResult {
    pub score: f32,
    pub text: String,
}

pub struct VectorDatabase {
    index: Index,
    tests: HashMap<u64, String>,
    next_key: u64,
}

impl VectorDatabase {
    pub fn new(dimensions: usize, capacity: usize) -> Result<Self> {
        let options = IndexOptions {
            dimensions,
            metric: MetricKind::Cos,
            quantization: ScalarKind::F32,
            ..Default::default()
        };

        let index = Index::new(&options)?;
        index.reserve(capacity)?;

        Ok(Self {
            index,
            tests: HashMap::with_capacity(capacity),
            next_key: 0,
        })
    }

    pub fn add(&mut self, text: impl Into<String>, vector: &[f32]) -> Result<u64> {
        let key = self.next_key;
        self.index.add(key, vector)?;
        self.tests.insert(key, text.into());
        self.next_key += 1;

        Ok(key)
    }

    pub fn search(&self, query_vector: &[f32], top_k: usize) -> Result<Vec<SearchResult>> {
        if self.tests.is_empty() || top_k == 0 {
            return Ok(Vec::new());
        }

        let matches = self.index.search(query_vector, top_k)?;

        let result = matches
            .keys
            .iter()
            .zip(matches.distances.iter())
            .filter_map(|(key, dist)| {
                self.tests.get(key).map(|text| SearchResult {
                    score: (1.0 - dist).clamp(0.0, 1.0),

                    text: text.clone(),
                })
            })
            .collect();

        Ok(result)
    }

    pub fn len(&self) -> usize {
        self.tests.len()
    }

    pub fn is_empty(&self) -> bool {
        self.tests.is_empty()
    }
}

fn main() -> Result<()> {
    let texts = [
        "公司成立于2010年，专注于人工智能领域的研发与应用。",
        "标准工作时间为周一至周五，每天9:00-18:00。",
        "公司每月15日发放工资，提供五险一金、带薪年假。",
        "员工应遵守职业道德，保护公司机密。",
        "年假：入职满1年可享受5天带薪年假，最多15天。",
    ];

    let dim = 8;
    let mut db = VectorDatabase::new(dim, 100)?;

    let corpus_vectors = [
        [0.8, 0.2, 0.1, 0.0, 0.0, 0.1, 0.0, 0.1], // 成立/AI
        [0.1, 0.9, 0.3, 0.0, 0.1, 0.0, 0.1, 0.0], // 工作时间
        [0.0, 0.2, 0.9, 0.8, 0.1, 0.0, 0.0, 0.1], // 发工资/福利
        [0.1, 0.0, 0.0, 0.0, 0.9, 0.8, 0.1, 0.0], // 职业道德
        [0.0, 0.1, 0.7, 0.9, 0.0, 0.0, 0.0, 0.2], // 年假天数
    ];

    for (text, vector) in texts.iter().zip(corpus_vectors.iter()) {
        db.add(*text, vector)?;
    }

    println!("  语料数量: {}", db.len());
    println!("  向量维度: {}", dim);

    let queries = [
        (
            "公司什么时候发工资？",
            [0.0, 0.1, 0.9, 0.7, 0.1, 0.0, 0.0, 0.0],
        ),
        (
            "每年可以休多少天假？",
            [0.0, 0.0, 0.6, 0.9, 0.0, 0.0, 0.0, 0.1],
        ),
    ];

    println!("\n【2】检索 Top-2 最相似文本");
    println!("{}", "-".repeat(60));

    for (query_text, query_vector) in &queries {
        let results = db.search(query_vector, 2)?;
        println!("Query: {query_text}");
        for (rank, res) in results.iter().enumerate() {
            println!("  Top-{} (score={:.4}): {}", rank + 1, res.score, res.text);
        }
        println!();
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vector_db_search() {
        let mut db = VectorDatabase::new(3, 10).unwrap();
        db.add("文档A", &[1.0, 0.0, 0.0]).unwrap();
        db.add("文档B", &[0.0, 1.0, 0.0]).unwrap();
        db.add("文档C", &[0.0, 0.0, 1.0]).unwrap();

        let query = [0.0, 0.95, 0.05];
        let hits = db.search(&query, 1).unwrap();

        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].text, "文档B");
        assert!(hits[0].score > 0.9);
    }
}
