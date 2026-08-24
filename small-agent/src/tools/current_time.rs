use serde_json::json;
use std::time::SystemTime;

use crate::tools::Tool;

pub struct CurrentTimeTool;

impl Tool for CurrentTimeTool {
    fn name(&self) -> &'static str {
        "current_time"
    }

    fn description(&self) -> &'static str {
        "获取当前系统的 UTC 统一时间戳"
    }

    fn parameters(&self) -> serde_json::Value {
        json!({
            "type": "object",
            "properties": {},
            "additionalProperties": false
        })
    }

    fn call(&self, _arguments: &str) -> String {
        match SystemTime::now().duration_since(SystemTime::UNIX_EPOCH) {
            Ok(n) => format!("当前 UTC 时间戳: {}秒", n.as_secs()),
            Err(e) => format!("ERROR: 获取系统时间失败: {}", e),
        }
    }
}
