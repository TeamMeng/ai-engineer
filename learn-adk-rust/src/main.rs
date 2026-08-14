use adk_rust::{Launcher, prelude::*};
use anyhow::Result;
use std::env;

#[tokio::main]
async fn main() -> Result<()> {
    dotenvy::dotenv().ok();

    let api_key = env::var("DEEPSEEK_API_KEY")?;

    let model = DeepSeekClient::v4_flash(api_key)?;

    let agent = LlmAgentBuilder::new("assistant")
        .description("一个友好的AI助手")
        .instruction("你是一个友好的助手，用简洁的中文回答问题")
        .model(Arc::new(model))
        .build()?;

    Launcher::new(Arc::new(agent)).run().await?;

    Ok(())
}

#[cfg(test)]
mod tests {
    #[test]
    fn func() {
        assert_eq!(1, 1);
    }
}
