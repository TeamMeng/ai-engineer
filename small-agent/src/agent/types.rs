use std::time::Duration;

#[derive(Debug, Clone)]
pub struct HistoryEntry {
    pub thought: String,
    pub action: String,
    pub action_input: String,
    pub observation: String,
}

#[derive(Debug, PartialEq, Eq)]
pub enum StepOutcome {
    Action {
        thought: String,
        action: String,
        action_input: String,
    },
    FinalAnswer(String),
    InvalidFormat {
        raw_output: String,
        error_msg: String,
    },
}

#[derive(Debug, Clone)]
pub struct AgentConfig {
    pub model: String,
    pub max_steps: usize,
    pub temperature: f32,
    pub timeout: Duration,
}

#[derive(Debug, Clone, PartialEq)]
pub struct EvaluationResult {
    pub success: bool,
    pub score: f32,
    pub feedback: String,
}

impl Default for AgentConfig {
    fn default() -> Self {
        Self {
            model: "deepseek-chat".to_owned(),
            max_steps: 6,
            temperature: 0.0_f32,
            timeout: Duration::from_secs(30),
        }
    }
}
