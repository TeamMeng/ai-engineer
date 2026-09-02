use anyhow::{Context, Result, ensure};
use async_openai::{
    Client,
    config::OpenAIConfig,
    types::chat::{
        ChatCompletionRequestMessage, ChatCompletionRequestSystemMessageArgs,
        ChatCompletionRequestUserMessageArgs, CreateChatCompletionRequestArgs,
    },
};
use std::env;
use tiktoken_rs::{CoreBPE, cl100k_base};

const SUMMARY_PROMPT_ZH: &str = r#"请用中文增量更新对话摘要：在已有摘要基础上，合并本轮新对话，输出一段新
  的中文摘要。

  示例
  当前摘要：
  用户询问 AI 对人工智能的看法。AI 认为人工智能是积极的力量。

  新对话：
  用户：你为什么认为人工智能是积极的力量？
  助手：因为它能帮助人类发挥潜能。

  新摘要：
  用户询问 AI 对人工智能的看法。AI 认为人工智能是积极的力量，因为它能帮助人类发挥潜能。
  示例结束

  要求：摘要必须使用中文，保留关键事实，语言简洁。只输出摘要正文，不要解释。

  当前摘要：
  {summary}

  新对话：
  {new_lines}

  新摘要："#;

struct SummaryMemory {
    client: Client<OpenAIConfig>,
    model: String,
    buffer: String,
}

impl SummaryMemory {
    fn new(client: Client<OpenAIConfig>, model: String) -> Self {
        Self {
            client,
            model,
            buffer: String::new(),
        }
    }

    fn load_memory_variables(&self) -> &str {
        &self.buffer
    }

    async fn call_llm(&self, messages: Vec<ChatCompletionRequestMessage>) -> Result<String> {
        let request = CreateChatCompletionRequestArgs::default()
            .model(self.model.clone())
            .messages(messages)
            .temperature(0.0_f32)
            .build()?;

        let response = self.client.chat().create(request).await?;

        let text = response
            .choices
            .into_iter()
            .next()
            .and_then(|choice| choice.message.content)
            .context("LLM 未返回文本内容")?
            .trim()
            .to_owned();

        ensure!(!text.is_empty(), "LLM 返回了空文本");

        Ok(text)
    }

    async fn save_context(&mut self, user_text: &str, assistant_text: &str) -> Result<()> {
        let old_memory = if self.buffer.is_empty() {
            " （暂无） "
        } else {
            self.buffer.as_str()
        };

        let new_lines = format!("用户：{user_text}\n助手：{assistant_text}");

        let prompt = SUMMARY_PROMPT_ZH
            .replace("{summary}", old_memory)
            .replace("{new_lines}", &new_lines);

        let messages = vec![
            ChatCompletionRequestUserMessageArgs::default()
                .content(prompt)
                .build()?
                .into(),
        ];

        self.buffer = self.call_llm(messages).await?;

        Ok(())
    }

    async fn answer(&self, question: &str) -> Result<String> {
        let system_text = format!(
            "你是架构顾问。参考对话历史：\n{}",
            self.load_memory_variables()
        );

        let messages = vec![
            ChatCompletionRequestSystemMessageArgs::default()
                .content(system_text)
                .build()?
                .into(),
            ChatCompletionRequestUserMessageArgs::default()
                .content(question.to_owned())
                .build()?
                .into(),
        ];

        self.call_llm(messages).await
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

        println!("[LLM] DeepSeek {model}");

        return Ok((Client::with_config(config), model));
    }

    let api_key =
        env::var("OPENAI_API_KEY").context("请设置 DEEPSEEK_API_KEY 或 OPENAI_API_KEY")?;

    let model = env::var("OPENAI_MODEL").unwrap_or_else(|_| "gpt-4o-mini".to_owned());

    let config = OpenAIConfig::new().with_api_key(api_key);

    println!("[LLM] OpenAI {model}");

    Ok((Client::with_config(config), model))
}

fn token_count(encoder: &CoreBPE, text: &str) -> usize {
    encoder.encode_ordinary(text).len()
}

fn print_summary(memory: &SummaryMemory, turn: usize) {
    let summary = memory.load_memory_variables();

    println!("\n{}", "=".repeat(60));
    println!(
        "第 {turn} 轮对话后 · memory.buffer（{} 字）",
        summary.chars().count()
    );
    println!("{}", "=".repeat(60));
    println!("{summary}");
}

#[tokio::main]
async fn main() -> Result<()> {
    dotenvy::dotenv().ok();

    let (client, model) = create_client()?;

    let mut memory = SummaryMemory::new(client, model);
    let encoder = cl100k_base()?;

    let turns = [
        ("我在做电商客服项目。", "好的，需要我帮你梳理架构吗？"),
        ("我们使用向量库存 FAQ。", "明白，可再加一层重排。"),
        ("召回后还要做 MMR 去重。", "可以，避免重复段落占满上下文。"),
    ];

    let mut raw_history = Vec::new();

    for (turn, (user_text, assistant_text)) in turns.iter().enumerate() {
        memory.save_context(user_text, assistant_text).await?;

        raw_history.push(format!("用户: {user_text}"));
        raw_history.push(format!("助手: {assistant_text}"));

        print_summary(&memory, turn + 1);
    }

    let full_history = raw_history.join("\n");
    let summary = memory.load_memory_variables().to_owned();
    let next_question = "如何做检索层优化？";

    let full_tokens = token_count(&encoder, &full_history);
    let summary_tokens = token_count(&encoder, &summary);

    println!("\n=== Token 对比 ===");
    println!(
        "完整历史: {} 字 / {} tokens",
        full_history.chars().count(),
        full_tokens
    );
    println!(
        "摘要 buffer: {} 字 / {} tokens",
        summary.chars().count(),
        summary_tokens
    );
    println!(
        "节省: {} tokens",
        full_tokens.saturating_sub(summary_tokens)
    );

    println!("\n=== Prompt messages ===");
    println!("[system] 你是架构顾问。参考对话历史：\n{summary}");
    println!("[user] {next_question}");

    let answer = memory.answer(next_question).await?;

    println!("\n=== LLM 回答 ===");
    println!("{answer}");

    memory.save_context(next_question, &answer).await?;

    raw_history.push(format!("用户: {next_question}"));
    raw_history.push(format!("助手: {answer}"));

    print_summary(&memory, 4);

    ensure!(!memory.load_memory_variables().is_empty(), "摘要不应为空");

    Ok(())
}
