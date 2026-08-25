use anyhow::{Context, Result};
use async_openai::{
    Client,
    config::OpenAIConfig,
    types::chat::{ChatCompletionRequestUserMessageArgs, CreateChatCompletionRequestArgs},
};
use tracing::{debug, info, instrument};

#[instrument(
    name = "planner",
    skip(client),
    fields(task = %task)
)]
pub async fn plan(client: &Client<OpenAIConfig>, task: &str, model: &str) -> Result<Vec<String>> {
    let prompt = format!(
        r#"You are a planning assistant. Break the following task into 3 to 6 concrete, sequential steps.
Respond with ONLY the numbered steps, one per line (e.g., '1. ...', '2. ...').

Task: {task}
"#
    );

    let request = CreateChatCompletionRequestArgs::default()
        .model(model)
        .temperature(0.0_f32)
        .messages([ChatCompletionRequestUserMessageArgs::default()
            .content(prompt)
            .build()?
            .into()])
        .build()?;

    let response = client.chat().create(request).await?;
    let content = response
        .choices
        .into_iter()
        .next()
        .and_then(|c| c.message.content)
        .context("未收到规划响应")?;

    debug!(raw_plan = %content, "收到原始规划文本");

    let steps: Vec<String> = content
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .map(|line| {
            line.trim_start_matches(|c: char| {
                c.is_ascii_digit() || c == '.' || c == '-' || c == ' '
            })
            .to_string()
        })
        .collect();

    info!(total = steps.len(), "📋 任务拆解计划完成");
    Ok(steps)
}
