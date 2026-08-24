use crate::{
    agent::types::{HistoryEntry, StepOutcome},
    tools::ToolRegistry,
};

pub fn build_prompt(question: &str, history: &[HistoryEntry], registry: &ToolRegistry) -> String {
    let tool_names = registry.tool_names().join(", ");
    let tool_descriptions = registry.tool_descriptions().join("\n");

    let mut prompt = format!(
        r#"
        You solve tasks using tools through an iterative reasoning process.
        You MUST respond using EXACTLY this format for each step:

        Thought: Describe your step-by-step thinking process.
        Action: Select exactly one tool from [{}]
        Action Input: The specific raw input argument for the tool.

        When you have gathered enough information to completely answer the user's question, output:
        Thought: I now have the final answer.
        Final Answer: The full and direct response to the user.

        IMPORTANT RULES:
        1. Output only ONE Thought/Action/Action Input cycle per turn.
        2. Wait for the Observation from the tool before proceeding.
        3. NEVER generate the "Observation:" yourself; it will be provided by the system.
        4. If you have the answer, do NOT call any more tools and output "Final Answer:" directly.

        Available Tools:
        {}

        Question: {}
    "#,
        tool_names, tool_descriptions, question
    );

    for entry in history {
        prompt.push_str(&format!(
            r#"
            Thought: {},
            Action: {},
            Action Input: {},
            Observation: {},
        "#,
            entry.thought, entry.action, entry.action_input, entry.observation
        ));
    }

    prompt
}

pub fn parse_step(raw_output: &str) -> StepOutcome {
    let mut thought = String::new();
    let mut action = String::new();
    let mut action_input = String::new();

    let mut lines = raw_output.lines().map(str::trim);

    for line in &mut lines {
        if let Some(t) = line.strip_prefix("Thought:") {
            thought = t.trim().to_string();
        } else if let Some(a) = line.strip_prefix("Action:") {
            action = a.trim().to_string();
        } else if let Some(i) = line.strip_prefix("Action Input:") {
            action_input = i.trim().to_string();
            break;
        }
    }

    if !action.is_empty() {
        return StepOutcome::Action {
            thought,
            action,
            action_input,
        };
    }

    if let Some((_, answer_part)) = raw_output.split_once("Final Answer:") {
        let answer = answer_part.trim();
        if !answer.is_empty() {
            return StepOutcome::FinalAnswer(answer.to_string());
        }
    }

    StepOutcome::InvalidFormat {
        raw_output: raw_output.to_string(),
        error_msg: "未检测到有效的 'Action:' 或 'Final Answer:' 标记".to_string(),
    }
}
