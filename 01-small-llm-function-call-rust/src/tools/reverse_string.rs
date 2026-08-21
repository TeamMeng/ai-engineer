use serde::Deserialize;
use serde_json::json;

use crate::tools::Tool;

#[derive(Deserialize)]
struct ReverseStringArgs {
    pub text: String,
}

pub struct ReverseStringTool;

impl Tool for ReverseStringTool {
    fn name(&self) -> &'static str {
        "reverse_string"
    }

    fn description(&self) -> &'static str {
        "反转输入的字符串"
    }

    fn parameters(&self) -> serde_json::Value {
        json!({
            "type": "object",
            "properties": {
                "text": {
                    "type": "string",
                    "description": "需要反转的文本。"
                }
            },
            "required": ["text"],
            "additionalProperties": false
        })
    }

    fn call(&self, arguments: &str) -> String {
        match serde_json::from_str::<ReverseStringArgs>(arguments) {
            Ok(args) => args.text.chars().rev().collect(),
            Err(e) => format!("ERROR: invalid JSON args for reverse_string: {}", e),
        }
    }
}
