use serde::Deserialize;
use serde_json::json;

use crate::tools::Tool;

#[derive(Deserialize)]
struct CalculatorArgs {
    operation: String,
    a: f64,
    b: f64,
}

pub struct BasicCalculatorTool;

impl Tool for BasicCalculatorTool {
    fn name(&self) -> &'static str {
        "basic_calculator"
    }

    fn description(&self) -> &'static str {
        "执行两个数的基本四则运算（加减乘除）"
    }

    fn parameters(&self) -> serde_json::Value {
        json!({
            "type": "object",
            "properties": {
                "operation": {
                    "type": "string",
                    "enum": ["add", "subtract", "multiply", "divide"],
                    "description": "运算类型：add/subtract/multiply/divide。"
                },
                "a": { "type": "number", "description": "第一个操作数。" },
                "b": { "type": "number", "description": "第二个操作数。" }
            },
            "required": ["operation", "a", "b"],
            "additionalProperties": false
        })
    }

    fn call(&self, arguments: &str) -> String {
        let args: CalculatorArgs = match serde_json::from_str(arguments) {
            Ok(a) => a,
            Err(e) => return format!("ERROR: invalid JSON args for basic_calculator: {}", e),
        };

        let op = args.operation.trim().to_lowercase();

        match op.as_str() {
            "add" => format!("{} + {} = {}", args.a, args.b, args.a + args.b),
            "subtract" => format!("{} - {} = {}", args.a, args.b, args.a - args.b),
            "multiply" => format!("{} x {} = {}", args.a, args.b, args.a * args.b),
            "divide" => {
                if args.b == 0.0 {
                    "ERROR: division by zero".to_owned()
                } else {
                    format!("{} / {} = {}", args.a, args.b, args.a / args.b)
                }
            }
            _ => format!("ERROR: unsupported operation: '{}'", op),
        }
    }
}
