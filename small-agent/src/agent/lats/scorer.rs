use anyhow::{Context, Result};
use async_openai::{
    Client,
    config::OpenAIConfig,
    types::chat::{ChatCompletionRequestUserMessageArgs, CreateChatCompletionRequestArgs},
};
use tracing::debug;

pub async fn score_action(
    client: &Client<OpenAIConfig>,
    task: &str,
    action: &str,
    action_input: &str,
    observation: &str,
    model: &str,
) -> Result<f32> {
    if observation.starts_with("ERROR:") {
        return Ok(0.1_f32);
    }

    let prompt = format!(
        r#"You are an MCTS Value Scorer evaluating an agent's step towards solving a task.
Rate the usefulness and promise of this step on a scale from 0.0 to 1.0.

Task: {task}
Action Taken: {action} (Input: {action_input})
Observation: {observation}

CRITERIA:
- 1.0: Completely solves or directly answers the task.
- 0.7~0.9: Highly productive step bringing crucial progress.
- 0.3~0.6: Neutral/partial progress.
- 0.0~0.2: Irrelevant, redundant, or misleading step.

Reply in EXACTLY this format:
Score: <float 0.0 to 1.0>
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
        .context("未收到评分响应")?;

    debug!(raw_score = %content, "收到原始评分");

    let mut score = 0.5_f32;
    for line in content.lines() {
        if let Some((_, val)) = line.trim().split_once(':')
            && let Ok(parsed) = val.trim().parse::<f32>()
        {
            score = parsed;
            break;
        }
    }

    Ok(score.clamp(0.0_f32, 1.0_f32))
}
