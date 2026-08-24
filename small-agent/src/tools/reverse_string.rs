use serde::Deserialize;

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

    fn call(&self, arguments: &str) -> String {
        match serde_json::from_str::<ReverseStringArgs>(arguments) {
            Ok(args) => args.text.chars().rev().collect(),
            Err(e) => format!("ERROR: invalid JSON args for reverse_string: {}", e),
        }
    }
}
