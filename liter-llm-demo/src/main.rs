use anyhow::Result;
use liter_llm::{
    AssistantContent, ChatCompletionRequest, ClientConfigBuilder, DefaultClient, LlmClient,
    Message, UserContent, UserMessage,
};
use std::env;

#[tokio::main]
async fn main() -> Result<()> {
    dotenvy::dotenv().ok();

    let api_key = env::var("DEEPSEEK_API_KEY")?;

    let config = ClientConfigBuilder::new(api_key).build();

    let client = DefaultClient::new(config, Some("deepseek-v4-flash"))?;

    let request = ChatCompletionRequest {
        model: "deepseek-v4-flash".to_string(),
        messages: vec![Message::User(UserMessage {
            content: UserContent::Text("你好，请用一句话介绍你自己".to_string()),
            name: None,
        })],
        ..Default::default()
    };

    let response = client.chat(request).await?;

    if let Some(choice) = response.choices.first() {
        match choice
            .message
            .content
            .as_ref()
            .and_then(AssistantContent::as_text)
        {
            Some(text) => println!("{}", text),
            None => println!("{:?}", choice.message.content),
        }
    }

    Ok(())
}
