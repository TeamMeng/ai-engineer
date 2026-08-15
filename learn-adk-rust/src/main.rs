use adk_rust::{Launcher, prelude::*};
use anyhow::Result;
use std::env;

#[tokio::main]
async fn main() -> Result<()> {
    dotenvy::dotenv().ok();

    let api_key = env::var("DEEPSEEK_API_KEY")?;

    let model: Arc<dyn Llm> = Arc::new(DeepSeekClient::v4_flash(api_key)?);

    let researcher = Arc::new(
        LlmAgentBuilder::new("researcher")
            .description("负责搜集和整理资料")
            .instruction(
                "
            你是一个研究员，根据用户给出的主题，\
            搜集相关资料，整理成结构化的调研笔记，\
            写入状态 key: research_notes。
        ",
            )
            .model(Arc::clone(&model))
            .build()?,
    );

    let writer = Arc::new(
        LlmAgentBuilder::new("writer")
            .description("根据调研笔记撰写文章")
            .instruction(
                "
            你是一个文章写作者。读取状态中的 research_notes，\
            撰写一篇 500 字左右的科普文章，\
            写入状态 key: article_draft。
        ",
            )
            .model(Arc::clone(&model))
            .build()?,
    );

    let reviewer = Arc::new(
        LlmAgentBuilder::new("reviewer")
            .description("审核文章质量并给出修改建议")
            .instruction(
                "
            你是一个资深编辑，读取状态中的 article_draft，\
            检查语法，逻辑和可读性，\
            输出最终修改后的文章。
        ",
            )
            .model(Arc::clone(&model))
            .build()?,
    );

    let pipeline = SequentialAgent::new("content_pipeline", vec![researcher, writer, reviewer]);

    let agent = Arc::new(pipeline);

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
