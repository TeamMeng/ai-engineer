use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq)]
pub struct ChildChunk {
    pub child_id: String,
    pub parent_id: String,
    pub text: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ParentChunk {
    pub parent_id: String,
    pub text: String,
    pub children: Vec<ChildChunk>,
}

pub fn build_parent_child_chunks(doc_id: &str, full_text: &str, child_size: usize) -> ParentChunk {
    let chars: Vec<char> = full_text.chars().collect();
    let total_len = chars.len();

    let mut children = Vec::new();
    let mut start = 0;
    let mut idx = 0;

    while start < total_len {
        let end = (start + child_size).min(total_len);
        let child_text: String = chars[start..end].iter().collect();

        if !child_text.trim().is_empty() {
            children.push(ChildChunk {
                child_id: format!("{}-child-{}", doc_id, idx),
                parent_id: doc_id.to_string(),
                text: child_text,
            });
            idx += 1;
        }

        start = end
    }

    ParentChunk {
        parent_id: doc_id.to_string(),
        text: full_text.to_string(),
        children,
    }
}

#[derive(Default)]
pub struct HierarchicalStore {
    pub parent_store: HashMap<String, ParentChunk>,
    pub child_to_parent: HashMap<String, String>,
}

impl HierarchicalStore {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register(&mut self, parent: ParentChunk) {
        for child in &parent.children {
            self.child_to_parent
                .insert(child.child_id.clone(), parent.parent_id.clone());
        }
        self.parent_store.insert(parent.parent_id.clone(), parent);
    }

    pub fn retrieve_context(&self, child_id: &str) -> Option<&str> {
        let parent_id = self.child_to_parent.get(child_id)?;
        let parent = self.parent_store.get(parent_id)?;
        Some(&parent.text)
    }
}

fn main() {
    let doc_text_001 = "\
公司成立于2010年，专注于人工智能领域的研发与应用。\
标准工作时间为周一至周五，每天9:00-18:00，午休12:00-13:00。\
公司每月15日发放工资，提供五险一金、带薪年假、节日福利及年终奖金。\
员工应遵守职业道德，保护公司机密，禁止从事与公司利益相冲突的行为。\
年假：入职满1年可享受5天带薪年假，每增加1年增加1天，最多15天。";

    let doc_text_002 = "\
公司配备完善的培训体系，新员工入职后需完成为期三天的岗前培训。\
技术团队每季度举办内部分享会，鼓励知识共享与创新实践。\
公司提供弹性福利积分，员工可自主选择健身、餐饮或交通补贴。\
远程办公政策：经主管审批后，每周最多可申请两天居家办公。";

    let parent_001 = build_parent_child_chunks("doc-001", doc_text_001, 20);
    let parent_002 = build_parent_child_chunks("doc-002", doc_text_002, 20);

    let mut store = HierarchicalStore::new();
    store.register(parent_001.clone());
    store.register(parent_002.clone());

    // 打印分块结构
    for parent in [&parent_001, &parent_002] {
        println!("==================================================");
        println!("=== ParentChunk: {} ===", parent.parent_id);
        let preview: String = parent.text.chars().take(30).collect();
        println!("text (前30字): {preview}…");
        println!("子块数量     : {}", parent.children.len());
        for child in &parent.children {
            println!("  [{}] {}", child.child_id, child.text);
        }
        println!();
    }

    println!("==================================================");
    println!("  模拟检索命中子块 -> 向上寻找完整父块");
    println!("==================================================");

    let hit_cases = [
        (&parent_001.children[2], "doc-001 命中子块[2]"),
        (&parent_002.children[1], "doc-002 命中子块[1]"),
    ];

    for (hit_child, label) in hit_cases {
        println!("=== {label}: {} ===", hit_child.child_id);
        println!("检索命中子块文本: {}", hit_child.text);

        if let Some(context) = store.retrieve_context(&hit_child.child_id) {
            let preview: String = context.chars().take(40).collect();
            println!("提取完整父块 (前40字): {preview}…\n");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parent_child_retrieval() {
        let mut store = HierarchicalStore::new();
        let parent = build_parent_child_chunks("doc-test", "AAABBBCCCDDD", 3);

        assert_eq!(parent.children.len(), 4);
        assert_eq!(parent.children[0].text, "AAA");
        assert_eq!(parent.children[1].text, "BBB");

        store.register(parent);

        let ctx = store.retrieve_context("doc-test-child-1").unwrap();
        assert_eq!(ctx, "AAABBBCCCDDD");
    }
}
