use anyhow::Result;
use futures_util::StreamExt;
use liter_llm::{
    ChatCompletionRequest, ClientConfigBuilder, DefaultClient, LlmClient, Message, UserContent,
    UserMessage,
};
use std::env;
use std::io::{self, Write};

#[tokio::main]
async fn main() -> Result<()> {
    dotenvy::dotenv().ok();

    let api_key = env::var("DEEPSEEK_API_KEY")?;

    let config = ClientConfigBuilder::new(api_key).build();

    let client = DefaultClient::new(config, Some("deepseek-v4-flash"))?;

    let request = ChatCompletionRequest {
        model: "deepseek-v4-flash".into(),
        messages: vec![Message::User(UserMessage {
            content: UserContent::Text("请写一首关于秋天的短诗，大约50字。".into()),
            name: None,
        })],
        ..Default::default()
    };

    let mut stream = client.chat_stream(request).await?;

    println!("助手：");
    while let Some(chunk_result) = stream.next().await {
        match chunk_result {
            Ok(chunk) => {
                for choice in &chunk.choices {
                    if let Some(text) = &choice.delta.content {
                        print!("{}", text);
                        io::stdout().flush()?;
                    }
                }
            }
            Err(e) => {
                eprintln!("{e}");
                break;
            }
        }
    }
    println!();

    Ok(())
}
