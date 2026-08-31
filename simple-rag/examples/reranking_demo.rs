use std::time::Instant;

use anyhow::Result;
use fastembed::{
    EmbeddingModel, RerankInitOptions, RerankerModel, TextEmbedding, TextInitOptions, TextRerank,
    similarity,
};

const TOP_K: usize = 4;
const TOP_N: usize = 2;

#[derive(Debug, Clone, Copy)]
struct ScoredDoc {
    doc_id: usize,
    score: f32,
}

fn step_back_queries(user_question: &str) -> (String, String) {
    let abstract_question = format!(
        "与下列问题相关的背景原理与定义是什么？
          {user_question}"
    );

    (user_question.to_owned(), abstract_question)
}

fn bi_encoder_retrieve(
    query: &str,
    corpus: &[&str],
    model: &mut TextEmbedding,
    top_k: usize,
) -> Result<Vec<ScoredDoc>> {
    let document_embeddings = model.embed(corpus, None)?;

    let query_embedding = model
        .embed(vec![query], None)?
        .into_iter()
        .next()
        .expect("embedding model should return one query vector");

    let mut results: Vec<ScoredDoc> = similarity::top_k(
        &query_embedding,
        &document_embeddings,
        top_k.min(document_embeddings.len()),
    )
    .into_iter()
    .map(|(doc_id, score)| ScoredDoc { doc_id, score })
    .collect();

    results.sort_by(|a, b| {
        b.score
            .total_cmp(&a.score)
            .then_with(|| a.doc_id.cmp(&b.doc_id))
    });

    Ok(results)
}

fn cross_encoder_rerank(
    query: &str,
    candidates: &[ScoredDoc],
    corpus: &[&str],
    reranker: &mut TextRerank,
    top_n: usize,
) -> Result<Vec<ScoredDoc>> {
    if candidates.is_empty() || top_n == 0 {
        return Ok(Vec::new());
    }

    let candidate_docs: Vec<&str> = candidates
        .iter()
        .map(|candidate| corpus[candidate.doc_id])
        .collect();

    let rerank_results = reranker.rerank(query, candidate_docs, false, None)?;

    let mut results: Vec<ScoredDoc> = rerank_results
        .into_iter()
        .map(|result| {
            let candidate = candidates.get(result.index).expect("e");

            ScoredDoc {
                doc_id: candidate.doc_id,
                score: result.score,
            }
        })
        .collect();

    results.sort_by(|a, b| {
        b.score
            .total_cmp(&a.score)
            .then_with(|| a.doc_id.cmp(&b.doc_id))
    });

    results.truncate(top_n.min(results.len()));

    Ok(results)
}

fn main() -> Result<()> {
    let question = "公司每月几号发工资？";
    let (concrete_question, abstract_question) = step_back_queries(question);

    println!("【Step-back 查询】");
    println!("  具体问题：{concrete_question}");
    println!("  抽象问题：{abstract_question}");

    // 真实项目中，这里接入 retriever 和 LLM：
    // let concrete_ctx = retriever.invoke(&concrete_question);
    // let background_ctx = retriever.invoke(&abstract_question);
    // let final_prompt = merge(concrete_ctx, background_ctx);

    let corpus = [
        "公司成立于2010年，专注于人工智能领域的研发与应用。",
        "公司每月15日发放工资，提供五险一金、带薪年假。",
        "标准工作时间为周一至周五，每天9:00-18:00。",
        "年假：入职满1年可享受5天带薪年假，最多15天。",
        "员工应遵守职业道德，保护公司机密。",
        "报销应提交发票原件及审批单。",
        "年假按工龄累计，最长不超过15天。",
        "员工需遵守考勤制度，迟到需补卡。",
        "公司每月15日发放工资，提供五险一金、带薪年假。",
        "标准工作时间为周一至周五，每天9:00-18:00。",
        "年假：入职满1年可享受5天带薪年假，最多15天。",
        "员工应遵守职业道德，保护公司机密。",
        "报销应提交发票原件及审批单。",
        "年假按工龄累计，最长不超过15天。",
        "员工需遵守考勤制度，迟到需补卡。",
    ];

    println!("\n加载 Bi-Encoder 和 Cross-Encoder...");

    let started = Instant::now();

    let mut bi_encoder = TextEmbedding::try_new(
        TextInitOptions::new(EmbeddingModel::BGESmallZHV15).with_show_download_progress(true),
    )?;

    let mut cross_encoder = TextRerank::try_new(
        RerankInitOptions::new(RerankerModel::BGERerankerBase).with_show_download_progress(true),
    )?;

    println!("模型加载耗时：{:?}", started.elapsed());

    let query = "发工资";

    let started = Instant::now();
    let bi_results = bi_encoder_retrieve(query, &corpus, &mut bi_encoder, TOP_K)?;
    println!("Bi-Encoder 耗时：{:?}", started.elapsed());

    println!("\n【Step 1】Bi-Encoder 粗召回 Top-{TOP_K}");

    for (rank, result) in bi_results.iter().enumerate() {
        println!(
            "  Top-{} (cos={:.4}): {}",
            rank + 1,
            result.score,
            corpus[result.doc_id]
        );
    }

    let started = Instant::now();
    let reranked = cross_encoder_rerank(query, &bi_results, &corpus, &mut cross_encoder, TOP_N)?;
    println!("Cross-Encoder 耗时：{:?}", started.elapsed());

    println!("\n【Step 2】Cross-Encoder 精排 Top-{TOP_N}");

    for (rank, result) in reranked.iter().enumerate() {
        println!(
            "  Top-{} (ce={:.4}): {}",
            rank + 1,
            result.score,
            corpus[result.doc_id]
        );
    }

    Ok(())
}
