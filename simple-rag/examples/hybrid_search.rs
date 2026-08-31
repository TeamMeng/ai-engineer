use std::collections::HashMap;

use anyhow::Result;
use charabia::Tokenize;
use fastembed::{EmbeddingModel, InitOptions, TextEmbedding};
use usearch::{Index, IndexOptions, MetricKind, ScalarKind};

pub struct BM25Okapi {
    corpus_size: usize,
    avgdl: f32,
    doc_lens: Vec<usize>,
    doc_frequencies: Vec<HashMap<String, usize>>,
    idf: HashMap<String, f32>,
    k1: f32,
    b: f32,
}

impl BM25Okapi {
    pub fn new(corpus: &[Vec<String>]) -> Self {
        let k1 = 1.5;
        let b = 0.75;
        let corpus_size = corpus.len();
        let mut total_len = 0;
        let mut doc_lens = Vec::with_capacity(corpus_size);
        let mut doc_frequencies = Vec::with_capacity(corpus_size);
        let mut doc_counts: HashMap<String, usize> = HashMap::new();

        for doc in corpus {
            total_len += doc.len();
            doc_lens.push(doc.len());

            let mut freq = HashMap::new();
            for word in doc {
                *freq.entry(word.clone()).or_insert(0) += 1;
            }

            for word in freq.keys() {
                *doc_counts.entry(word.clone()).or_insert(0) += 1;
            }
            doc_frequencies.push(freq);
        }

        let avgdl = if corpus_size > 0 {
            total_len as f32 / corpus_size as f32
        } else {
            0.0
        };

        let mut idf = HashMap::with_capacity(doc_counts.len());
        for (word, n) in doc_counts {
            let idf_val = (((corpus_size as f32 - n as f32 + 0.5) / (n as f32 + 0.5)) + 1.0).ln();
            idf.insert(word, idf_val);
        }

        Self {
            corpus_size,
            avgdl,
            doc_lens,
            doc_frequencies,
            idf,
            k1,
            b,
        }
    }

    pub fn get_scores(&self, query: &[String]) -> Vec<f32> {
        let mut scores = vec![0.0; self.corpus_size];

        for (doc_idx, freq_map) in self.doc_frequencies.iter().enumerate() {
            let doc_len = self.doc_lens[doc_idx] as f32;
            let mut score = 0.0;

            for word in query {
                if let Some(&tf) = freq_map.get(word)
                    && let Some(&idf_val) = self.idf.get(word)
                {
                    let tf = tf as f32;
                    let denom = tf + self.k1 * (1.0 - self.b + self.b * (doc_len / self.avgdl));
                    let num = tf * (self.k1 + 1.0);
                    score += idf_val * (num / denom);
                }
            }
            scores[doc_idx] = score;
        }

        scores
    }
}

pub fn min_max_norm(scores: &[f32]) -> Vec<f32> {
    if scores.is_empty() {
        return Vec::new();
    }

    let mut s_min = f32::INFINITY;
    let mut s_max = f32::NEG_INFINITY;

    for &s in scores {
        if s < s_min {
            s_min = s;
        }
        if s > s_max {
            s_max = s;
        }
    }

    if (s_max - s_min).abs() < f32::EPSILON {
        return vec![1.0; scores.len()];
    }

    scores
        .iter()
        .map(|&s| (s - s_min) / (s_max - s_min))
        .collect()
}

pub fn tokenize_text(text: &str) -> Vec<String> {
    text.tokenize()
        .filter(|t| !t.is_separator())
        .map(|t| t.lemma().to_string())
        .filter(|w| !w.trim().is_empty())
        .collect()
}

pub fn hybrid_fuse(vec_scores: &[f32], bm25_scores: &[f32], alpha: f32) -> Vec<f32> {
    assert_eq!(
        vec_scores.len(),
        bm25_scores.len(),
        "两路分数长度必须严格对齐"
    );

    let norm_vec = min_max_norm(vec_scores);
    let norm_bm25 = min_max_norm(bm25_scores);

    norm_vec
        .iter()
        .zip(norm_bm25.iter())
        .map(|(&v, &b)| alpha * v + (1.0 - alpha) * b)
        .collect()
}

fn main() -> Result<()> {
    let texts = [
        "公司成立于2010年，专注于人工智能领域的研发与应用。",
        "标准工作时间为周一至周五，每天9:00-18:00。",
        "公司每月15日发放工资，提供五险一金、带薪年假。",
        "员工应遵守职业道德，保护公司机密。",
        "年假：入职满1年可享受5天带薪年假，最多15天。",
    ];

    println!("【1】构建双路检索索引（BGE 向量索引 + BM25 倒排索引）");

    let mut model = TextEmbedding::try_new(InitOptions::new(EmbeddingModel::BGESmallZHV15))?;
    let corpus_embeddings = model.embed(texts, None)?;
    let dim = corpus_embeddings[0].len();

    let options = IndexOptions {
        dimensions: dim,
        metric: MetricKind::Cos,
        quantization: ScalarKind::F32,
        ..Default::default()
    };
    let vec_index = Index::new(&options)?;
    vec_index.reserve(texts.len())?;
    for (i, vec) in corpus_embeddings.iter().enumerate() {
        vec_index.add(i as u64, vec)?;
    }
    println!("  ✅ 向量索引构建完成 (usearch + BGE-small-zh)");

    let tokenized_corpus: Vec<Vec<String>> = texts.iter().map(|doc| tokenize_text(doc)).collect();
    let bm25 = BM25Okapi::new(&tokenized_corpus);
    println!("  ✅ BM25 倒排索引构建完成 (charabia)");

    let query = "公司工资发放时间";

    let q_embeddings = model.embed(vec![query], None)?;
    let matches = vec_index.search(&q_embeddings[0], texts.len())?;

    let mut vec_scores_ordered = vec![0.0; texts.len()];
    for (key, dist) in matches.keys.iter().zip(matches.distances.iter()) {
        vec_scores_ordered[*key as usize] = (1.0 - dist).clamp(0.0, 1.0);
    }
    println!("  向量原始打分: {:?}", vec_scores_ordered);

    // (2) BM25 检索路打分
    let tokenized_query = tokenize_text(query);
    let bm25_scores_ordered = bm25.get_scores(&tokenized_query);
    println!("  BM25 原始打分: {:?}", bm25_scores_ordered);

    // (3) 双路归一化与加权融合 (alpha = 0.5)
    let alpha = 0.5;
    let hybrid_scores = hybrid_fuse(&vec_scores_ordered, &bm25_scores_ordered, alpha);
    println!("  混合融合得分: {:?}", hybrid_scores);

    let mut ranked: Vec<(usize, f32)> = hybrid_scores.into_iter().enumerate().collect();
    ranked.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

    println!("\n============================================================");
    println!("【3】最终 Top-2 混合融合检索结果 (α = {alpha}):");
    println!("============================================================");

    for (rank, (idx, score)) in ranked.iter().take(2).enumerate() {
        println!(
            "  Top-{} (Hybrid分={:.4}) => {}",
            rank + 1,
            score,
            texts[*idx]
        );
    }

    Ok(())
}
