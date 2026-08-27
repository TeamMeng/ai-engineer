pub mod mcts;
pub mod scorer;
pub mod types;

use anyhow::{Context, Result};
use async_openai::{
    Client,
    config::OpenAIConfig,
    types::chat::{ChatCompletionRequestUserMessageArgs, CreateChatCompletionRequestArgs},
};
use tracing::{info, info_span, instrument, warn};

use crate::{
    agent::{
        lats::{
            mcts::{backpropagate, get_best_path, select_node},
            scorer::score_action,
            types::SearchTree,
        },
        prompt::parse_step,
        types::{AgentConfig, StepOutcome},
    },
    tools::ToolRegistry,
};

#[instrument(
    name = "lats_tree_search_engine",
    skip(client, registry),
    fields(task = %task, model = %config.model, budget = budget)
)]
pub async fn lats_solve(
    client: &Client<OpenAIConfig>,
    task: &str,
    registry: &ToolRegistry,
    config: &AgentConfig,
    budget: usize,   // MCTS 迭代预算（例如 6 次）
    branch_k: usize, // 单节点发散候选动作数（例如 3 条）
) -> Result<String> {
    info!(
        budget = budget,
        branch_k = branch_k,
        "🌲 启动 LATS 蒙特卡洛树搜索推演引擎"
    );

    let mut tree = SearchTree::new();
    let tool_descriptions = registry.tool_descriptions().join("\n");

    for iter in 1..=budget {
        let iter_span = info_span!("mcts_iteration", iter = iter, budget = budget);
        let _enter = iter_span.entered();

        // 1. Selection: 沿 UCB 选择当前最优叶节点
        let selected_id = select_node(&tree);
        let selected_depth = tree.nodes[selected_id].depth;

        // 2. Expansion: 若未到最大深度，调用 LLM 裂变生成 K 个候选 Action
        if selected_depth < config.max_steps {
            let expand_prompt = format!(
                r#"You are a Tree-Search Decision Generator.
Generate {branch_k} DISTINCT, completely different next candidate actions to solve the task.
Each candidate MUST follow: 'Action: <tool>\nAction Input: <input>'

Available Tools:
{tool_descriptions}

Task: {task}
Format: Output exactly {branch_k} actions separated by '---'
"#
            );

            let request = CreateChatCompletionRequestArgs::default()
                .model(&config.model)
                .temperature(0.7_f32) // 适度温度以激发发散思维
                .messages([ChatCompletionRequestUserMessageArgs::default()
                    .content(expand_prompt)
                    .build()?
                    .into()])
                .build()?;

            let response = client.chat().create(request).await?;
            let content = response
                .choices
                .into_iter()
                .next()
                .and_then(|c| c.message.content)
                .unwrap_or_default();

            // ⭐️ 核心改进（方案 2）：使用 && 合并模式匹配与布尔判断，彻底消除 collapsible_if
            for candidate_str in content.split("---") {
                if let StepOutcome::Action {
                    action,
                    action_input,
                    ..
                } = parse_step(candidate_str)
                    && registry.contains(&action)
                {
                    let observation = registry.execute(&action, &action_input);

                    // 3. Simulation: 价值函数打分（优雅降级：网络异常给 0.5 默认值，不崩溃）
                    let score = match score_action(
                        client,
                        task,
                        &action,
                        &action_input,
                        &observation,
                        &config.model,
                    )
                    .await
                    {
                        Ok(s) => s,
                        Err(err) => {
                            warn!(error = ?err, "价值打分失败，自动降级为默认值 0.5");
                            0.5_f32
                        }
                    };

                    let child_id = tree.add_child(selected_id, action, action_input, observation);

                    // 4. Backpropagation: 向上回传得分
                    backpropagate(&mut tree, child_id, score);
                }
            }
        }
    }

    // 5. 提取 visits 最多的最优决策路径
    let best_path = get_best_path(&tree);
    info!(best_path = ?best_path, "🏆 树搜索推演完成，锁定最优决策链");

    // 提取最优路径的三元组
    let mut final_context = String::new();
    for (a, inp, obs) in best_path.iter().filter_map(|&id| {
        let node = tree.nodes.get(id)?;
        Some((
            node.action.as_ref()?,
            node.action_input.as_ref()?,
            node.observation.as_ref()?,
        ))
    }) {
        final_context.push_str(&format!("- Action: {a}({inp}) -> Observation: {obs}\n"));
    }

    let summary_prompt = format!(
        r#"Task: {task}
Optimal Execution Path Results:
{final_context}

Please provide the final clean answer to the user in Chinese based on the optimal path above.
"#
    );

    let summary_request = CreateChatCompletionRequestArgs::default()
        .model(&config.model)
        .temperature(0.0_f32)
        .messages([ChatCompletionRequestUserMessageArgs::default()
            .content(summary_prompt)
            .build()?
            .into()])
        .build()?;

    let summary_resp = client.chat().create(summary_request).await?;
    let final_answer = summary_resp
        .choices
        .into_iter()
        .next()
        .and_then(|c| c.message.content)
        .context("大模型未能成功汇总最终答案")?;

    Ok(final_answer)
}
