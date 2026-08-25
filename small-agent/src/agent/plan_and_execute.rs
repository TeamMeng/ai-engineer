use std::collections::HashMap;

use anyhow::Result;
use async_openai::{Client, config::OpenAIConfig};
use tracing::{info, instrument};

use crate::{
    agent::{planner::plan, react_loop, types::AgentConfig},
    tools::ToolRegistry,
};

#[instrument(name = "plan_and_execute_engine", skip(client, registry), fields(task = %task, model = %config.model))]
pub async fn run_plan_and_execute(
    client: &Client<OpenAIConfig>,
    task: &str,
    registry: &ToolRegistry,
    config: &AgentConfig,
) -> Result<String> {
    let steps = plan(client, task, &config.model).await?;

    println!("\n📋 [计划清单]");
    for (i, step) in steps.iter().enumerate() {
        println!("  {}. {}", i + 1, step);
    }
    println!("{}", "─".repeat(50));

    let mut state: HashMap<String, String> = HashMap::new();
    let total_steps = steps.len();

    for (i, step) in steps.iter().enumerate() {
        let step_num = i + 1;
        info!("🚀 Step {}/{}: {}", step_num, total_steps, step);

        let mut step_prompt = format!("Current Step to complete: {}\n", step);
        if !state.is_empty() {
            step_prompt.push_str("Prior step results:\n");
            for (prev_k, prev_v) in &state {
                step_prompt.push_str(&format!("- {}: {}\n", prev_k, prev_v));
            }
        }

        let step_result = react_loop(client, &step_prompt, registry, config).await?;
        info!("✓ Step {} 完成", step_num);

        state.insert(format!("Step {} ({})", step_num, step), step_result);
    }

    info!("✨ 所有步骤完成，正在汇总交付结果...");

    let mut summary_prompt =
        format!("The original task was: {task}\nHere are the results of all executed steps:\n");
    for (k, v) in &state {
        summary_prompt.push_str(&format!("{}: {}\n", k, v));
    }
    summary_prompt.push_str(
        "\nPlease provide a clean, complete final answer to the user in Chinese based on the results above.",
    );

    react_loop(client, &summary_prompt, registry, config).await
}
