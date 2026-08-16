use adk_rust::{Launcher, prelude::*};
use anyhow::Result;
use std::env;

#[tokio::main]
async fn main() -> Result<()> {
    dotenvy::dotenv().ok();

    let api_key = env::var("DEEPSEEK_API_KEY")?;

    let model: Arc<dyn Llm> = Arc::new(DeepSeekClient::v4_flash(api_key)?);

    let writer = Arc::new(
        LlmAgentBuilder::new("writer")
            .description("根据反馈修改文章")
            .instruction(
                "
            你是一个写作者。读取状态中的 article 和 feedback，\
            根据反馈修改文章，更新状态 key: article。
        ",
            )
            .model(Arc::clone(&model))
            .build()?,
    );

    let editor = Arc::new(
        LlmAgentBuilder::new("editor")
            .description("审核文章质量")
            .instruction(
                "
            你是一名编辑，读取状态中的 article，\
            评估文章质量， \
            如果文章质量达到发布标准，调用 exit_loop 工具结束修改。\
            否则，把修改写入状态 key: feedback。
        ",
            )
            .model(Arc::clone(&model))
            .build()?,
    );

    let refine_loop = LoopAgent::new("stock_analysis", vec![writer, editor]).with_max_iterations(5);

    let agent = Arc::new(refine_loop);

    Launcher::new(agent).run().await?;

    Ok(())
}

#[cfg(test)]
mod tests {
    #[test]
    fn func() {
        assert_eq!(1, 1);
    }
}
