use crate::{
    agent::types::{HistoryEntry, StepOutcome},
    tools::ToolRegistry,
};

pub fn build_prompt(question: &str, history: &[HistoryEntry], registry: &ToolRegistry) -> String {
    let tool_names = registry.tool_names().join(", ");
    let tool_descriptions = registry.tool_descriptions().join("\n");

    let mut prompt = format!(
        r#"You solve tasks using tools through an iterative reasoning process.
You MUST respond using EXACTLY this format for each step:

Thought: Describe your step-by-step thinking process.
Action: Select exactly one tool from [{tool_names}]
Action Input: The specific raw input argument for the tool.

When you have received the Observation and are ready to answer the question, output:
Thought: I now have the final answer.
Final Answer: The full and direct response to the user.

IMPORTANT RULES:
1. Output only ONE Thought/Action/Action Input cycle per turn.
2. ALWAYS use a tool if an applicable tool exists.
3. DO NOT include "Final Answer:" in the first turn until you have received an Observation from a tool!
4. NEVER generate the "Observation:" yourself; it will be provided by the system.

Available Tools:
{tool_descriptions}

Question: {question}
        "#
    );

    for entry in history {
        prompt.push_str(&format!(
            r#"
Thought: {}
Action: {}
Action Input: {}
Observation: {}
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
            if thought.is_empty() {
                thought = t.trim().to_string();
            }
        } else if let Some(a) = line.strip_prefix("Action:") {
            action = a.trim().to_string();
        } else if let Some(i) = line.strip_prefix("Action Input:") {
            action_input = i.trim().to_string();
            break;
        }
    }

    // 优先判定 Action，防止同轮次模型臆想 Final Answer
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::agent::types::StepOutcome;

    #[test]
    fn test_parse_action_priority() {
        let raw = r#"
Thought: 先反转字符串
Action: reverse_string
Action Input: Rust2024
Final Answer: 4202tsuR
        "#;
        let outcome = parse_step(raw);
        match outcome {
            StepOutcome::Action {
                thought,
                action,
                action_input,
            } => {
                assert_eq!(thought, "先反转字符串");
                assert_eq!(action, "reverse_string");
                assert_eq!(action_input, "Rust2024");
            }
            _ => panic!("Expected Action, got {:?}", outcome),
        }
    }

    #[test]
    fn test_parse_pure_final_answer() {
        let raw = r#"
            Thought: 我已经知道了答案
            Final Answer: Rust 是安全快速的系统编程语言。
        "#;
        let outcome = parse_step(raw);
        assert_eq!(
            outcome,
            StepOutcome::FinalAnswer("Rust 是安全快速的系统编程语言。".to_string())
        );
    }
}
