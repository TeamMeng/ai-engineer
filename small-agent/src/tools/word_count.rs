use crate::tools::Tool;

pub struct WordCountTool;

impl Tool for WordCountTool {
    fn name(&self) -> &'static str {
        "word_count"
    }

    fn description(&self) -> &'static str {
        "统计文本的字符数与词数。直接输入待统计的文本内容"
    }

    fn call(&self, arguments: &str) -> String {
        let input_text = arguments.trim();

        let text_to_count = if let Ok(v) = serde_json::from_str::<serde_json::Value>(input_text) {
            v.get("text")
                .and_then(|t| t.as_str())
                .unwrap_or(input_text)
                .to_string()
        } else {
            input_text.trim_matches('"').trim_matches('\'').to_string()
        };

        let total_chars = text_to_count.chars().count();
        let non_whitespace_chars = text_to_count.chars().filter(|c| !c.is_whitespace()).count();
        let words = text_to_count.split_whitespace().count();

        format!(
            "字符数(含空格): {}, 字符数(不含空格): {}, 词数: {}",
            total_chars, non_whitespace_chars, words
        )
    }
}
