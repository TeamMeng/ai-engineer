use anyhow::Result;
use async_openai::{Client, config::OpenAIConfig};
use std::{
    env,
    io::{self, Write},
};

use crate::{agent::chat_with_tools, tools::ToolRegistry};

mod agent;
mod tools;

#[tokio::main]
async fn main() -> Result<()> {
    dotenvy::dotenv().ok();

    let api_key = env::var("DEEPSEEK_API_KEY").expect("not found DEEPSEEK_API_KEY");

    let config = OpenAIConfig::default()
        .with_api_key(api_key)
        .with_api_base("https://api.deepseek.com/v1");
    let client = Client::with_config(config);

    let registry = ToolRegistry::default_tools();

    println!("{}", "=".repeat(60));
    println!("LLM Function Calling (Rust + async-openai 0.41)");
    println!("可用工具：reverse_string / basic_calculator");
    println!("输入 'exit' 或 'quit' 退出");
    println!("{}", "=".repeat(60));

    let stdin = io::stdin();
    loop {
        print!("\n[你] ");
        io::stdout().flush()?;

        let mut user_input = String::new();
        if stdin.read_line(&mut user_input)? == 0 {
            println!("\n再见！");
            break;
        }

        let user_input = user_input.trim();
        if user_input.is_empty() {
            continue;
        }
        if user_input.eq_ignore_ascii_case("exit") || user_input.eq_ignore_ascii_case("quit") {
            println!("再见！");
            break;
        }

        println!("\n\n{}", "*".repeat(60));
        println!("User>\t {user_input}");
        println!("{}", "*".repeat(60));

        let model = "deepseek-chat";
        match chat_with_tools(&client, user_input, model, &registry).await {
            Ok(answer) => {
                println!("\n\n{}", "=".repeat(60));
                println!("Assistant>\t {answer}");
                println!("{}", "*".repeat(60));
                println!("\n[助手] {answer}\n{}\n", "─".repeat(60));
            }
            Err(err) => {
                eprintln!("\n[Error] 执行失败: {err:?}");
            }
        }
    }

    Ok(())
}
