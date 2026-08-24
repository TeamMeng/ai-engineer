use std::time::SystemTime;

use crate::tools::Tool;

pub struct CurrentTimeTool;

impl Tool for CurrentTimeTool {
    fn name(&self) -> &'static str {
        "current_time"
    }

    fn description(&self) -> &'static str {
        "获取当前系统的 UTC 统一时间戳。无需任何输入参数"
    }

    fn call(&self, _arguments: &str) -> String {
        match SystemTime::now().duration_since(SystemTime::UNIX_EPOCH) {
            Ok(n) => format!("当前 UTC 时间戳: {} 秒", n.as_secs()),
            Err(e) => format!("ERROR: 获取系统时间失败: {e}"),
        }
    }
}
