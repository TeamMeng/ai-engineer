use anyhow::{Context, Result};
use async_openai::{
    Client,
    config::OpenAIConfig,
    types::chat::{
        ChatCompletionRequestAssistantMessageArgs, ChatCompletionRequestMessage,
        ChatCompletionRequestSystemMessageArgs, ChatCompletionRequestUserMessageArgs,
        CreateChatCompletionRequestArgs,
    },
};
use std::{collections::VecDeque, env, fmt};
use tiktoken_rs::{CoreBPE, cl100k_base};

const CONTEXT_LIMIT: usize = 200;

#[derive(Debug, Clone, Copy)]
enum Role {
    System,
    User,
    Assistant,
}

impl fmt::Display for Role {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let role = match self {
            Self::System => "system",
            Self::User => "user",
            Self::Assistant => "assistant",
        };
        f.write_str(role)
    }
}

#[derive(Debug, Clone)]
struct ChatMessage {
    role: Role,
    content: String,
}

struct WindowedConversationMemory {
    max_tokens: usize,
    encoder: CoreBPE,
    system: Option<ChatMessage>,
    history: VecDeque<ChatMessage>,
}

impl WindowedConversationMemory {
    fn new(max_tokens: usize) -> Result<Self> {
        Ok(Self {
            max_tokens,
            encoder: cl100k_base()?,
            system: None,
            history: VecDeque::new(),
        })
    }

    fn add(&mut self, role: Role, content: impl Into<String>) -> Result<()> {
        let messages = ChatMessage {
            role,
            content: content.into(),
        };

        match role {
            Role::System => self.system = Some(messages),
            Role::User | Role::Assistant => {
                self.history.push_back(messages);
            }
        }

        self.shrink_to_budget()
    }

    fn count_tokens(&self) -> usize {
        let text = self
            .system
            .iter()
            .chain(self.history.iter())
            .map(|message| format!("{}: {}", message.role, message.content))
            .collect::<Vec<_>>()
            .join("\n");

        self.encoder.encode_ordinary(&text).len()
    }

    fn shrink_to_budget(&mut self) -> Result<()> {
        while self.history.len() > 1 && self.count_tokens() > self.max_tokens {
            let Some(dropped) = self.history.pop_front() else {
                break;
            };

            let preview: String = dropped.content.chars().take(20).collect();
            println!(
                "\n[budget] tokens > {}，pop_front 丢弃 [{}] {}...",
                self.max_tokens, dropped.role, preview
            );
        }

        let tokens = self.count_tokens();

        if tokens > self.max_tokens {
            anyhow::bail!("最新消息超过输入预算：{} > {}", tokens, self.max_tokens);
        }

        Ok(())
    }

    fn as_openai_messages(&self) -> Result<Vec<ChatCompletionRequestMessage>> {
        self.system
            .iter()
            .chain(self.history.iter())
            .map(|message| {
                let request_message: ChatCompletionRequestMessage = match message.role {
                    Role::System => ChatCompletionRequestSystemMessageArgs::default()
                        .content(message.content.clone())
                        .build()?
                        .into(),
                    Role::User => ChatCompletionRequestUserMessageArgs::default()
                        .content(message.content.clone())
                        .build()?
                        .into(),
                    Role::Assistant => ChatCompletionRequestAssistantMessageArgs::default()
                        .content(message.content.clone())
                        .build()?
                        .into(),
                };
                Ok(request_message)
            })
            .collect()
    }

    fn print_mem(&self, title: &str) {
        println!("--- {title} mem ---");
        for message in self.system.iter().chain(self.history.iter()) {
            println!("[{}] {}", message.role, message.content);
        }
        println!(
            "tokens ~= {} / max {}",
            self.count_tokens(),
            self.max_tokens
        );
    }
}

fn create_client() -> Result<(Client<OpenAIConfig>, String)> {
    if let Ok(api_key) = env::var("DEEPSEEK_API_KEY") {
        let base_url =
            env::var("DEEPSEEK_BASE_URL").unwrap_or_else(|_| "https://api.deepseek.com".to_owned());

        let model = env::var("DEEPSEEK_MODEL").unwrap_or_else(|_| "deepseek-chat".to_owned());

        let config = OpenAIConfig::new()
            .with_api_key(api_key)
            .with_api_base(base_url);

        println!("[LLM] {}", model);
        return Ok((Client::with_config(config), model));
    }

    let api_key =
        env::var("OPENAI_API_KEY").context("请设置 DEEPSEEK_API_KEY 或 OPENAI_API_KEY")?;
    let model = env::var("OPENAI_MODEL").unwrap_or_else(|_| "gpt-4o-mini".to_owned());
    let config = OpenAIConfig::new().with_api_key(api_key);

    println!("[LLM] {model}");
    Ok((Client::with_config(config), model))
}

async fn chat(
    memory: &mut WindowedConversationMemory,
    client: &Client<OpenAIConfig>,
    model: &str,
    user_text: &str,
) -> Result<String> {
    memory.add(Role::User, user_text)?;

    let request = CreateChatCompletionRequestArgs::default()
        .model(model)
        .messages(memory.as_openai_messages()?)
        .temperature(0.0_f32)
        .build()?;

    let response = client.chat().create(request).await?;

    let content = response
        .choices
        .into_iter()
        .next()
        .and_then(|choice| choice.message.content)
        .context("LLM 未返回文本内容")?
        .trim()
        .to_owned();

    memory.add(Role::Assistant, content.clone())?;

    Ok(content)
}

#[tokio::main]
async fn main() -> Result<()> {
    dotenvy::dotenv().ok();

    let (client, model) = create_client()?;

    let mut memory = WindowedConversationMemory::new(CONTEXT_LIMIT)?;
    memory.add(Role::System, "你是助手，回答尽量简洁。在100字以内")?;

    let user_turns = [
        "我叫阿明。",
        "什么是llm",
        "上什么是transformer",
        "请解释mem",
    ];

    for (index, user_text) in user_turns.iter().enumerate() {
        let turn = index + 1;

        println!("================= {turn} turn start ======================");

        let answer = chat(&mut memory, &client, &model, user_text).await?;

        println!("[user] {user_text}");
        println!("[assistant] {answer}\n");

        memory.print_mem(&format!("第 {turn} 轮结束后"));
        println!("============= {turn} turn end ==========================\n");
    }

    memory.print_mem("最终保留的消息");
    Ok(())
}
