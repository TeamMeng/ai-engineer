use anyhow::{Context, Result};
use async_openai::{
    Client,
    config::OpenAIConfig,
    types::chat::{
        ChatCompletionMessageToolCalls, ChatCompletionRequestAssistantMessageArgs,
        ChatCompletionRequestMessage, ChatCompletionRequestSystemMessageArgs,
        ChatCompletionRequestToolMessageArgs, ChatCompletionRequestUserMessageArgs,
        CreateChatCompletionRequestArgs,
    },
};

use crate::tools::ToolRegistry;

pub const SYSTEM_PROMPT: &str = "
    你是一个有用的助手，拥有反转字符串和四则运算等工具。
    需要时调用工具，然后用自然语言回答用户。
";

pub async fn chat_with_tools(
    client: &Client<OpenAIConfig>,
    user_message: &str,
    model: &str,
    registry: &ToolRegistry,
) -> Result<String> {
    let mut messages: Vec<ChatCompletionRequestMessage> = vec![
        ChatCompletionRequestSystemMessageArgs::default()
            .content(SYSTEM_PROMPT)
            .build()?
            .into(),
        ChatCompletionRequestUserMessageArgs::default()
            .content(user_message)
            .build()?
            .into(),
    ];

    let mut request_builder = CreateChatCompletionRequestArgs::default();
    request_builder
        .model(model)
        .messages(messages.clone())
        .temperature(0.0_f32);

    let tools = registry.get_schemas()?;
    if !tools.is_empty() {
        request_builder.tools(tools);
    }

    let request = request_builder.build()?;
    let response = client.chat().create(request).await?;
    let choice = response
        .choices
        .into_iter()
        .next()
        .context("未收到大模型回复")?;

    let response_message = choice.message;

    let tool_calls = match response_message.tool_calls {
        Some(calls) if !calls.is_empty() => calls,
        _ => return Ok(response_message.content.unwrap_or_default()),
    };

    let mut assistant_builder = ChatCompletionRequestAssistantMessageArgs::default();
    if let Some(content) = response_message.content {
        assistant_builder.content(content);
    }
    assistant_builder.tool_calls(tool_calls.clone());
    messages.push(assistant_builder.build()?.into());

    for tool_call_enum in tool_calls {
        if let ChatCompletionMessageToolCalls::Function(tool_call) = tool_call_enum {
            let name = tool_call.function.name;
            let args = tool_call.function.arguments;
            let call_id = tool_call.id;

            println!(">>> [Tool Call]: {}{}", name, args);
            let result = registry.execute(&name, &args);
            println!("<<< [Tool Output]: {}", result);

            let tool_message = ChatCompletionRequestToolMessageArgs::default()
                .tool_call_id(call_id)
                .content(result)
                .build()?;
            messages.push(tool_message.into());
        }
    }

    let final_request = CreateChatCompletionRequestArgs::default()
        .model(model)
        .messages(messages)
        .temperature(0.0_f32)
        .build()?;

    let final_response = client.chat().create(final_request).await?;
    let final_answer = final_response
        .choices
        .into_iter()
        .next()
        .and_then(|c| c.message.content)
        .unwrap_or_default();

    Ok(final_answer)
}
