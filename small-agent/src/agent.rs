pub mod plan_and_execute;
pub mod planner;
pub mod prompt;
pub mod types;

use anyhow::{Context, Result};
use async_openai::{
    Client,
    config::OpenAIConfig,
    types::chat::{ChatCompletionRequestUserMessageArgs, CreateChatCompletionRequestArgs},
};
use tokio::time::timeout;
use tracing::{debug, error, info, info_span, instrument, warn};

use crate::{
    agent::{
        prompt::{build_prompt, parse_step},
        types::{AgentConfig, HistoryEntry, StepOutcome},
    },
    tools::ToolRegistry,
};

#[instrument(
    name = "react_run",
    skip(client, registry),
    fields(model = %config.model)
)]
pub async fn react_loop(
    client: &Client<OpenAIConfig>,
    question: &str,
    registry: &ToolRegistry,
    config: &AgentConfig,
) -> Result<String> {
    let mut history: Vec<HistoryEntry> = Vec::with_capacity(config.max_steps);

    for step in 1..=config.max_steps {
        let step_span = info_span!("step", step = step);
        let _enter = step_span.entered();

        debug!("开始构建 Prompt 并请求大模型");

        let prompt_text = build_prompt(question, &history, registry);
        debug!(prompt_len = prompt_text.len(), "Prompt 构建完成");

        let request = CreateChatCompletionRequestArgs::default()
            .model(&config.model)
            .temperature(config.temperature)
            .messages(vec![
                ChatCompletionRequestUserMessageArgs::default()
                    .content(prompt_text)
                    .build()?
                    .into(),
            ])
            .build()?;

        let response = timeout(config.timeout, client.chat().create(request))
            .await
            .with_context(|| {
                format!(
                    "大模型在 {} 秒内未返回响应，已触发超时熔断",
                    config.timeout.as_secs()
                )
            })??;

        let model_output = response
            .choices
            .into_iter()
            .next()
            .and_then(|c| c.message.content)
            .context("大模型未回复")?;

        debug!(raw_output = %model_output, "收到模型原始响应");

        match parse_step(&model_output) {
            StepOutcome::FinalAnswer(answer) => {
                debug!(step = step, answer = %answer, "推理达成目标");
                return Ok(answer);
            }
            StepOutcome::Action {
                thought,
                action,
                action_input,
            } => {
                debug!(thought = %thought, "模型思考细节");
                info!(tool = %action, input = %action_input, "🛠️  调用工具");

                let observation = if registry.contains(&action) {
                    registry.execute(&action, &action_input)
                } else {
                    format!(
                        "ERROR: 工具 '{}' 不存在. 可选工具: {:?}",
                        action,
                        registry.tool_names()
                    )
                };

                info!(output = %observation, "👁️  工具返回");

                history.push(HistoryEntry {
                    thought,
                    action,
                    action_input,
                    observation,
                });
            }

            StepOutcome::InvalidFormat {
                raw_output,
                error_msg,
            } => {
                warn!(error = %error_msg, "⚠️ 输出格式异常，注入自愈提示");

                history.push(HistoryEntry {
                    thought: "Format correction required".to_string(),
                    action: "system_validator".to_string(),
                    action_input: raw_output,
                    observation: format!(
                        "ERROR: {error_msg}. Please strictly use 'Thought: ... Action: ... Action Input: ...'"
                    ),
                });
            }
        }
    }

    error!(max_steps = config.max_steps, "超过最大步数仍未得出答案");
    anyhow::bail!(
        "Failed: Exceeded maximum steps ({}) without reaching a Final Answer.",
        config.max_steps
    );
}
