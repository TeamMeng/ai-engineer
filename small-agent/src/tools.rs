pub mod basic_calculator;
pub mod current_time;
pub mod reverse_string;
pub mod word_count;

use std::collections::BTreeMap;
use tracing::{debug, instrument};

pub trait Tool: Send + Sync {
    fn name(&self) -> &'static str;
    fn description(&self) -> &'static str;
    fn call(&self, arguments: &str) -> String;
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
        Self::new()
            .registry(reverse_string::ReverseStringTool)
            .registry(basic_calculator::BasicCalculatorTool)
            .registry(word_count::WordCountTool)
            .registry(current_time::CurrentTimeTool)
    }

    pub fn registry(mut self, tool: impl Tool + 'static) -> Self {
        self.tools.insert(tool.name(), Box::new(tool));
        self
    }

    pub fn contains(&self, name: &str) -> bool {
        self.tools.contains_key(name)
    }

    pub fn tool_descriptions(&self) -> Vec<String> {
        self.tools
            .values()
            .map(|t| format!("- {}: {}", t.name(), t.description()))
            .collect()
    }

    pub fn tool_names(&self) -> Vec<&'static str> {
        self.tools.keys().copied().collect()
    }

    #[instrument(name = "tool_execution", skip(self), fields(tool = %name, input = %arguments))]
    pub fn execute(&self, name: &str, arguments: &str) -> String {
        debug!("开始执行底层工具调用");
        let result = match self.tools.get(name) {
            Some(tool) => tool.call(arguments),
            None => format!("Error: unknown tool: '{name}'"),
        };
        debug!(output = %result, "底层工具调用完毕");
        result
    }
}
