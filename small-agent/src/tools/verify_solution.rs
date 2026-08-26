use serde::Deserialize;

use crate::tools::Tool;

#[derive(Deserialize)]
struct VerifyArgs {
    pub draft_answer: String,
}

pub struct VerifySolutionTool;

impl Tool for VerifySolutionTool {
    fn name(&self) -> &'static str {
        "verify_solution"
    }

    fn description(&self) -> &'static str {
        "
答案质检与自我反思工具。在准备给出 Final Answer
前调用，用于检查拟定答案是否完整回答了问题、是否存在遗漏。
入参为 JSON: {\"draft_answer\": \"你的拟定最终答案\"}
        "
    }

    fn call(&self, arguments: &str) -> String {
        let input_text = arguments.trim();

        let draft = if let Ok(args) = serde_json::from_str::<VerifyArgs>(input_text) {
            args.draft_answer
        } else if let Ok(v) = serde_json::from_str::<serde_json::Value>(input_text) {
            v.get("draft_answer")
                .and_then(|s| s.as_str())
                .unwrap_or(input_text)
                .to_string()
        } else {
            input_text.trim_matches('"').trim_matches('\'').to_string()
        };

        if draft.is_empty() {
            return "质检失败 (FAIL);草稿答案为空，请生成具体的解答内容。".to_string();
        }

        if draft.contains("Thought:") || draft.contains("Action:") {
            return "质检失败 (FAIL):草稿中包含了未清理的内部思考/动作标记，请提供纯净的用户答复。"
                .to_string();
        }

        format!(
            "质检通过 (PASS): 草稿「{}」逻辑完整、格式规范。请在下一步直接输出 'Final Answer: {}' 结束任务！",
            draft, draft
        )
    }
}
