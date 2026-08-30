use charabia::Tokenize;
use std::collections::HashMap;

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

pub fn tokenize_text(text: &str) -> Vec<String> {
    text.tokenize()
        .filter(|t| !t.is_separator())
        .map(|t| t.lemma().to_string())
        .filter(|w| !w.trim().is_empty())
        .collect()
}

fn main() {
    let texts = [
        "公司成立于2010年，专注于人工智能领域的研发与应用。",
        "标准工作时间为周一至周五，每天9:00-18:00。",
        "公司每月15日发放工资，提供五险一金、带薪年假。",
        "员工应遵守职业道德，保护公司机密。",
        "年假：入职满1年可享受5天带薪年假，最多15天。",
    ];

    println!("【1】使用 Charabia 自动切词并构建 BM25索引");

    let tokenized_corpus: Vec<Vec<String>> = texts.iter().map(|doc| tokenize_text(doc)).collect();

    let bm25 = BM25Okapi::new(&tokenized_corpus);
    println!("  语料库篇数: {}", texts.len());

    let query = "公司工资发放时间";
    println!("【2】关键词检索: \"{query}\"");

    let tokenized_query = tokenize_text(query);
    println!("  Charabia 分词结果: {:?}", tokenized_query);

    let scores = bm25.get_scores(&tokenized_query);
    println!("  各文档原始打分: {:?}", scores);

    // 4. 按得分从高到低排序取 Top-2
    let mut ranked: Vec<(usize, f32)> = scores.into_iter().enumerate().collect();
    ranked.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

    println!("\nTop-2 检索结果：");
    for (rank, (idx, score)) in ranked.iter().take(2).enumerate() {
        println!(
            "  Top-{} (BM25得分={:.4}) => {}",
            rank + 1,
            score,
            texts[*idx]
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_charabia_tokenizer() {
        let tokens = tokenize_text("DeepSeek-V3 在2026年 发薪！");

        assert!(tokens.contains(&"deepseek".to_string()));
        assert!(tokens.contains(&"v3".to_string()));

        assert!(tokens.contains(&"发薪".to_string()));
    }
}
