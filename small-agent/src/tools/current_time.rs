use std::time::SystemTime;

use crate::tools::Tool;

pub struct CurrentTimeTool;

impl Tool for CurrentTimeTool {
    fn name(&self) -> &'static str {
        "current_time"
    }

    fn description(&self) -> &'static str {
        "获取当前系统的 UTC 日期时间与时间戳（如 '2026-08-25 11:20:00 UTC'）。无需入参"
    }

    fn call(&self, _arguments: &str) -> String {
        match SystemTime::now().duration_since(SystemTime::UNIX_EPOCH) {
            Ok(n) => {
                let secs = n.as_secs();
                let datetime_str = format_utc_timestamp(secs);
                format!("当前时间: {datetime_str}")
            }
            Err(e) => format!("ERROR: 获取系统时间失败: {e}"),
        }
    }
}

/// 零依赖将 UNIX 秒级时间戳转换为格式化 UTC 日期字符串 (Howard Hinnant 算法)
fn format_utc_timestamp(secs: u64) -> String {
    let days = secs / 86400;
    let rem_secs = secs % 86400;
    let hours = rem_secs / 3600;
    let mins = (rem_secs % 3600) / 60;
    let seconds = rem_secs % 60;

    let z = (days as i64) + 719468;
    let era = if z >= 0 { z } else { z - 146096 } / 146097;
    let doe = (z - era * 146097) as u64;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let y = (yoe as i64) + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = if m <= 2 { y + 1 } else { y };

    format!("{y:04}-{m:02}-{d:02} {hours:02}:{mins:02}:{seconds:02} UTC (时间戳: {secs} 秒)")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_time() {
        let tool = CurrentTimeTool;
        let res = tool.call("");
        assert!(res.contains("UTC"));
        assert!(res.contains("时间戳"));
    }
}
