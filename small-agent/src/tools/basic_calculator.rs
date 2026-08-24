use serde::Deserialize;

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
        "四则运算工具。入参为 JSON 格式: {\"operation\":\"add|subtract|multiply|divide\", \"a\":数字, \"b\":数字}"
    }

    fn call(&self, arguments: &str) -> String {
        let args: CalculatorArgs = match serde_json::from_str(arguments.trim()) {
            Ok(a) => a,
            Err(e) => {
                return format!(
                    "ERROR: invalid JSON args for basic_calculator: {}. Expected format: {{\"operation\":\"add\",\"a\":1,\"b\":2}}",
                    e
                );
            }
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
