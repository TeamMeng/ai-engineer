use anyhow::Result;
use liter_llm::{
    ChatCompletionRequest, ClientConfigBuilder, DefaultClient, LlmClient, Message, UserContent,
    UserMessage,
};
use std::env;

async fn ask_question(api_key: &str, model: &str, question: &str) -> Result<String> {
    let config = ClientConfigBuilder::new(api_key).build();

    let client = DefaultClient::new(config, Some(model))?;

    let request = ChatCompletionRequest {
        model: model.into(),
        messages: vec![Message::User(UserMessage {
            content: UserContent::Text(question.into()),
            name: None,
        })],
        ..Default::default()
    };

    let response = client.chat(request).await?;

    let text = response.choices[0]
        .message
        .content
        .as_ref()
        .and_then(|c| c.as_text())
        .unwrap_or_default()
        .to_string();

    Ok(text)
}

#[tokio::main]
async fn main() -> Result<()> {
    dotenvy::dotenv().ok();

    let api_key = env::var("DEEPSEEK_API_KEY")?;

    let question = "用一句话解释什么是人工智能";

    let answer1 = ask_question(&api_key, "deepseek-v4-flash", question).await?;

    println!("{}", answer1);

    let answer2 = ask_question(&api_key, "deepseek-v4-flash", question).await?;

    println!("{}", answer2);

    Ok(())
}
