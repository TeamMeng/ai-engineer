use adk_rust::{Launcher, prelude::*};
use anyhow::Result;
use std::env;

#[tokio::main]
async fn main() -> Result<()> {
    dotenvy::dotenv().ok();

    let api_key = env::var("DEEPSEEK_API_KEY")?;

    let model = Arc::new(DeepSeekClient::v4_flash(api_key)?);

    let code_agent = Arc::new(
        LlmAgentBuilder::new("code_expert")
            .description("编写，调试和解释 Rust，Python，JavaScript 代码")
            .instruction(
                "
            你是一名资深程序员，提供高质量的代码实例和详细解释。\
            代码要有注释，解释要通俗易懂
        ",
            )
            .model(model.clone())
            .build()?,
    );

    let math_agent = Arc::new(
        LlmAgentBuilder::new("math_expert")
            .description("解答数学问题，包括代数，微积分，统计学")
            .instruction(
                "
            你是一名数学教授。用清晰的步骤解题，\
            并解释每一步的原理。
        ",
            )
            .model(model.clone())
            .build()?,
    );

    let coordinator = Arc::new(
        LlmAgentBuilder::new("coordinator")
            .description("综合助手，协调各领域专家")
            .instruction(
                "
            你是一名综合助手。根据用户的问题类型，\
            委托给合适的专家处理。\
            如果问题涉及多个领域，可以先后委托多个专家，\
            然后综合他们的回答给用户一个完整的答复。
        ",
            )
            .sub_agent(code_agent)
            .sub_agent(math_agent)
            .model(model.clone())
            .build()?,
    );

    Launcher::new(coordinator).run().await?;

    Ok(())
}

#[cfg(test)]
mod tests {
    #[test]
    fn func() {
        assert_eq!(1, 1);
    }
}
