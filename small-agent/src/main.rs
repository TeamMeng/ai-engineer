use anyhow::Result;
use async_openai::{Client, config::OpenAIConfig};
use small_agent::{
    agent::{plan_and_execute::run_plan_and_execute, react_loop, types::AgentConfig},
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

    println!("{}", "=".repeat(60));
    println!("ReAct Agent (Rust + DeepSeek + Tracing)");
    println!("可用工具: {:?}", registry.tool_names());
    println!("输入 '/plan <任务>' 开启规划执行模式，输入 'exit' 退出");
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

        if let Some(task) = user_input.strip_prefix("/plan") {
            let task = task.trim();
            println!("\n[模式] 已开启 Plan-and-Execute 深度规划引擎");
            match run_plan_and_execute(&client, task, &registry, &agent_config).await {
                Ok(answer) => println!("\n[最终交付] {answer}\n{}", "═".repeat(60)),
                Err(e) => eprintln!("\n[Error] 执行失败: {e:?}"),
            }
        } else if let Some(task) = user_input.strip_prefix("/lats") {
            let task = task.trim();
            println!("\n[模式] 已开启 LATS 蒙特卡洛树搜索推演引擎 (AlphaGo 算法");
            match run_plan_and_execute(&client, task, &registry, &agent_config).await {
                Ok(answer) => println!("\n[最终交付] {answer}\n{}", "═".repeat(60)),
                Err(e) => eprintln!("\n[Error] 树搜索执行失败: {e:?}"),
            }
        } else {
            match react_loop(&client, user_input, &registry, &agent_config).await {
                Ok(answer) => println!("\n[助手] {answer}\n{}", "─".repeat(60)),
                Err(e) => eprintln!("\n[Error] 执行失败: {e:?}"),
            }
        }
    }

    Ok(())
}
