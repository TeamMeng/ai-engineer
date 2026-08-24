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
    name = "react_agent_run",
    skip(client, registry),
    fields(
        question = %question,
        model = %config.model,
        max_steps = config.max_steps,
        timeout_secs = config.timeout.as_secs()
    )
)]
pub async fn react_loop(
    client: &Client<OpenAIConfig>,
    question: &str,
    registry: &ToolRegistry,
    config: &AgentConfig,
) -> Result<String> {
    let mut history: Vec<HistoryEntry> = Vec::with_capacity(config.max_steps);

    for step in 1..=config.max_steps {
        let step_span = info_span!("step", step = step, max_steps = config.max_steps);
        let _enter = step_span.entered();

        info!("开始构建 Prompt 并调用大模型");

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
                    " 大模型在 {} 秒内未返回响应，已触发超时熔断",
                    config.timeout.as_secs()
                )
            })??;

        let model_output = response
            .choices
            .into_iter()
            .next()
            .and_then(|c| c.message.content)
            .context("大模型未回复")?;

        // debug!(raw_output = %model_output, "收到模型原始响应");

        match parse_step(&model_output) {
            StepOutcome::FinalAnswer(answer) => {
                info!(step = step, final_answer = %answer, "🎉 [Final Answer] 推理达成目标");
                return Ok(answer);
            }
            StepOutcome::Action {
                thought,
                action,
                action_input,
            } => {
                info!(thought = %thought, action = %action, action_input = %action_input, "💭 [Thought & Action] 模型决定执行工具");

                let observation = if registry.contains(&action) {
                    registry.execute(&action, &action_input)
                } else {
                    let err = format!(
                        "ERROR: 工具 '{}' 不存在. 可选工具: {:?}",
                        action,
                        registry.tool_names()
                    );
                    err
                };

                info!(action = %action, observation = %observation, "👁️ [Observation] 获得工具执行结果");

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
                warn!(raw_output = %raw_output, error = %error_msg, "⚠️ [Format Error] 输出格式异常，注入自愈提示");

                history.push(HistoryEntry { thought: "Format correction required".to_string(), action: "system_validator".to_string(), action_input: raw_output, observation: format!("ERROR: {}. Please strictly use 'Thought: ... Action: ... Action Input: ...'", error_msg) });
            }
        }
    }

    error!(max_steps = config.max_steps, "超过最大步数仍未得出答案");
    anyhow::bail!(
        "Failed: Exceeded maximum steps ({}) without reaching a Final Answer.",
        config.max_steps
    );
}
