use anyhow::Result;
use liter_llm::{
    AssistantMessage, ChatCompletionRequest, ClientConfigBuilder, DefaultClient, LlmClient,
    Message, SystemMessage, UserContent, UserMessage,
};
use std::env;

#[tokio::main]
async fn main() -> Result<()> {
    dotenvy::dotenv().ok();

    let api_key = env::var("DEEPSEEK_API_KEY")?;

    let config = ClientConfigBuilder::new(api_key).build();

    let client = DefaultClient::new(config, Some("deepseek-v4-flash"))?;

    let mut message: Vec<Message> = vec![Message::System(SystemMessage {
        content: "你是一位地理知识助手，回答要简洁。".into(),
        name: None,
    })];

    message.push(Message::User(UserMessage {
        content: UserContent::Text("法国的首都在哪里？".into()),
        name: None,
    }));

    let request = ChatCompletionRequest {
        model: "deepseek-v4-flash".into(),
        messages: message.clone(),
        ..Default::default()
    };

    let response = client.chat(request).await?;

    let first_reply = response.choices[0]
        .message
        .content
        .clone()
        .unwrap_or_default();
    println!("助手（第一轮）:{}", first_reply);

    message.push(Message::Assistant(AssistantMessage {
        content: Some(first_reply),
        ..Default::default()
    }));

    let request = ChatCompletionRequest {
        model: "deepseek-v4-flash".into(),
        messages: message.clone(),
        ..Default::default()
    };

    let response = client.chat(request).await?;

    let second_reply = response.choices[0]
        .message
        .content
        .clone()
        .unwrap_or_default();
    println!("助手（第一轮）:{}", second_reply);

    if let Some(usage) = response.usage {
        println!(
            "本次消耗：输入 {} tokens，输出 {} tokens",
            usage.prompt_tokens, usage.completion_tokens
        );
    }

    Ok(())
}
