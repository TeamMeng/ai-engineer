use adk_rust::{Launcher, prelude::*};
use anyhow::Result;
use std::env;

#[tokio::main]
async fn main() -> Result<()> {
    dotenvy::dotenv().ok();

    let api_key = env::var("DEEPSEEK_API_KEY")?;

    let model: Arc<dyn Llm> = Arc::new(DeepSeekClient::v4_flash(api_key)?);

    let tech_analyst = Arc::new(
        LlmAgentBuilder::new("tech_analyst ")
            .description("从技术面分析股票")
            .instruction(
                "
            你是技术分析师，分析用户提供的股票代码的 \
            k线形态，均线系统，成交量等技术指标，\
            给出技术面评级（强烈买入/买入/中性/卖出/强烈卖出）。
        ",
            )
            .model(Arc::clone(&model))
            .build()?,
    );

    let market_analyst = Arc::new(
        LlmAgentBuilder::new("market_analyst ")
            .description("从基本面分析股票")
            .instruction(
                "
            你是基本面分析师，分析用户提供的股票代码的 \
            市盈率，市净率，营收增长等基本面指标， \
            给出基本面评级。
        ",
            )
            .model(Arc::clone(&model))
            .build()?,
    );

    let risk_analyst = Arc::new(
        LlmAgentBuilder::new("risk_analyst")
            .description("从基本面分析股票")
            .instruction(
                "
            你是风险评估师，分析用户提供的股票代码的，\
            波动率，行业风险，政策风险，\
            给出风险等级（低/中/高）。
        ",
            )
            .model(Arc::clone(&model))
            .build()?,
    );

    let parallel_analysis = ParallelAgent::new(
        "stock_analysis",
        vec![tech_analyst, market_analyst, risk_analyst],
    );

    let agent = Arc::new(parallel_analysis);

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
