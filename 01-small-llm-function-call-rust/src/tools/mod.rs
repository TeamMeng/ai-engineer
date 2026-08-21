pub mod basic_calculator;
pub mod reverse_string;

use anyhow::Result;
use async_openai::types::chat::{ChatCompletionTool, ChatCompletionTools, FunctionObjectArgs};
use std::collections::BTreeMap;

pub trait Tool: Send + Sync {
    fn name(&self) -> &'static str;

    fn description(&self) -> &'static str;

    fn parameters(&self) -> serde_json::Value;

    fn call(&self, arguments: &str) -> String;

    fn to_chat_tool(&self) -> Result<ChatCompletionTools> {
        let function = FunctionObjectArgs::default()
            .name(self.name())
            .description(self.description())
            .parameters(self.parameters())
            .build()?;

        Ok(ChatCompletionTools::Function(ChatCompletionTool {
            function,
        }))
    }
}

#[derive(Default)]
pub struct ToolRegistry {
    tools: BTreeMap<&'static str, Box<dyn Tool>>,
}

impl ToolRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn default_tools() -> Self {
        let mut registry = Self::new();
        registry.registry(reverse_string::ReverseStringTool);
        registry.registry(basic_calculator::BasicCalculatorTool);
        registry
    }

    pub fn registry(&mut self, tool: impl Tool + 'static) {
        self.tools.insert(tool.name(), Box::new(tool));
    }

    pub fn get_schemas(&self) -> Result<Vec<ChatCompletionTools>> {
        self.tools
            .values()
            .map(|tool| tool.to_chat_tool())
            .collect()
    }

    pub fn execute(&self, name: &str, arguments: &str) -> String {
        match self.tools.get(name) {
            Some(tool) => tool.call(arguments),
            None => format!("Error: unknown tool: '{}'", name,),
        }
    }
}
