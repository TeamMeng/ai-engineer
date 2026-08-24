use anyhow::Result;
use async_openai::{Client, config::OpenAIConfig};
use small_agent::{
    agent::{react_loop, types::AgentConfig},
    tools::ToolRegistry,
};
use std::{
    env,
    io::{self, Write},
};
use tracing_subscriber::{EnvFilter, layer::SubscriberExt, util::SubscriberInitExt};

fn init_tracing() {
    let filter_layer = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("small_agent=info,warn"));

    let fmt_layer = tracing_subscriber::fmt::layer()
        .with_target(false)
        .with_thread_ids(false)
        .with_file(false)
        .with_line_number(false);

    tracing_subscriber::registry()
        .with(filter_layer)
        .with(fmt_layer)
        .init();
}

#[tokio::main]
async fn main() -> Result<()> {
    dotenvy::dotenv().ok();
    init_tracing();

    let api_key = env::var("DEEPSEEK_API_KEY").expect("not found DEEPSEEK_API_KEY");

    let config = OpenAIConfig::default()
        .with_api_key(api_key)
        .with_api_base("https://api.deepseek.com/v1");
    let client = Client::with_config(config);

    let registry = ToolRegistry::default_tools();
    let agent_config = AgentConfig::default();

    tracing::info!(
        tools = ?registry.tool_names(),
        model = %agent_config.model,
        "ReAct Agent 初始化就绪"
    );

    println!("{}", "=".repeat(60));
    println!("ReAct Agent (Rust + DeepSeek + Tracing)");
    println!("可用工具: {:?}", registry.tool_names());
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

        match react_loop(&client, user_input, &registry, &agent_config).await {
            Ok(answer) => {
                println!("\n[助手] {answer}\n{}", "─".repeat(60));
            }
            Err(err) => {
                tracing::error!(error = ?err, "ReAct 执行失败");
                eprintln!("\n[Error] 执行失败: {err:?}");
            }
        }
    }

    Ok(())
}
