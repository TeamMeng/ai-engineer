use std::collections::HashMap;

fn rrf_fuse(ranked_lists: &[&[&str]], k: usize) -> Vec<(String, f64)> {
    let mut scores = HashMap::<String, f64>::new();

    for ranks in ranked_lists {
        for (rank, doc_id) in ranks.iter().enumerate() {
            let rank = rank + 1;
            let contribution = 1.0 / ((k + rank) as f64);

            *scores.entry((*doc_id).to_owned()).or_insert(0.0) += contribution;
        }
    }

    let mut fused: Vec<(String, f64)> = scores.into_iter().collect();

    fused.sort_by(|left, right| {
        right
            .1
            .total_cmp(&left.1)
            .then_with(|| left.0.cmp(&right.0))
    });

    fused
}

fn main() {
    let texts = [
        "公司成立于2010年，专注于人工智能领域的研发与应用。",
        "标准工作时间为周一至周五，每天9:00-18:00。",
        "公司每月15日发放工资，提供五险一金、带薪年假。",
        "员工应遵守职业道德，保护公司机密。",
        "年假：入职满1年可享受5天带薪年假，最多15天。",
    ];

    let vec_ranked = ["doc_2", "doc_4", "doc_0", "doc_1", "doc_3"];

    let bm25_ranked = ["doc_2", "doc_3", "doc_1", "doc_4", "doc_0"];

    let fused = rrf_fuse(&[&vec_ranked, &bm25_ranked], 60);

    debug_assert_eq!(
        fused.first().map(|(doc_id, _)| doc_id.as_str()),
        Some("doc_2")
    );

    println!("RRF 融合结果：");

    for (rank, (doc_id, score)) in fused.iter().enumerate() {
        let rank = rank + 1;

        let index = doc_id
            .strip_prefix("doc_")
            .and_then(|value| value.parse::<usize>().ok())
            .expect("doc_id 必须符合 doc_<数字> 格式");

        println!("  Top-{rank} (rrf={score:.6}) [{doc_id}]: {}", texts[index]);
    }
}
