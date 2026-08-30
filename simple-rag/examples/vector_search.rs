use anyhow::Result;
use fastembed::{EmbeddingModel, InitOptions, TextEmbedding};
use usearch::{Index, IndexOptions, MetricKind, ScalarKind};

fn main() -> Result<()> {
    let mut model = TextEmbedding::try_new(InitOptions::new(EmbeddingModel::BGESmallZHV15))?;

    let texts = vec![
        "公司成立于2010年，专注于人工智能领域的研发与应用。",
        "标准工作时间为周一至周五，每天9:00-18:00。",
        "公司每月15日发放工资，提供五险一金、带薪年假。",
        "员工应遵守职业道德，保护公司机密。",
        "年假：入职满1年可享受5天带薪年假，最多15天。",
    ];

    let corpus_embeddings = model.embed(texts.clone(), None)?;
    let dim = corpus_embeddings[0].len();

    println!(
        "  成功计算 {} 条语料向量，维度: {}",
        corpus_embeddings.len(),
        dim
    );

    let options = IndexOptions {
        dimensions: dim,
        metric: MetricKind::Cos,
        quantization: ScalarKind::F32,
        ..Default::default()
    };

    let index = Index::new(&options)?;
    index.reserve(texts.len())?;

    for (i, vec) in corpus_embeddings.iter().enumerate() {
        index.add(i as u64, vec)?;
    }

    let query = "公司什么时候发工资？";

    let q_embeddings = model.embed(vec![query], None)?;
    let q_vec = &q_embeddings[0];

    let matches = index.search(q_vec, 2)?;

    for (rank, (key, dist)) in matches
        .keys
        .iter()
        .zip(matches.distances.iter())
        .enumerate()
    {
        let similarity = 1.0 - dist;
        let matched_text = &texts[*key as usize];
        println!(
            "  Top-{} (相似度得分: {:.4}) => {}",
            rank + 1,
            similarity,
            matched_text
        );
    }

    Ok(())
}
