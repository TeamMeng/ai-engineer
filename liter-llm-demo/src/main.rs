use anyhow::Result;
use axum::{Json, Router, extract::State, http::HeaderMap, response::IntoResponse, routing::post};
use reqwest::StatusCode;
use std::{collections::HashMap, sync::Arc};
use tokio::sync::RwLock;

#[allow(unused)]
#[derive(Debug, Clone)]
struct Tenant {
    id: String,
    name: String,
    api_key: String,
    allowed_models: Vec<String>,
    budget_usd: f64,
    current_spend: f64,
}

struct GatewayConfig {
    tenants: RwLock<HashMap<String, Tenant>>,
    http_client: reqwest::Client,
    deepseek_api_key: String,
    ollama_base_url: String,
}

type SharedState = Arc<GatewayConfig>;

async fn chat_handler(
    State(state): State<SharedState>,
    headers: HeaderMap,
    Json(mut payload): Json<serde_json::Value>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let auth_header = headers
        .get("Authorization")
        .and_then(|h| h.to_str().ok())
        .ok_or((StatusCode::UNAUTHORIZED, "缺少 Authorization 头".into()))?;

    let token = auth_header.strip_prefix("Bearer ").ok_or((
        StatusCode::UNAUTHORIZED,
        "Token 格式错误，请使用 Bearer <token>".into(),
    ))?;

    let mut tenants_guard = state.tenants.write().await;
    let tenant = tenants_guard
        .values_mut()
        .find(|t| t.api_key == token)
        .ok_or((StatusCode::UNAUTHORIZED, "无效的 API 密钥".into()))?;

    let requested_model = payload
        .get("model")
        .and_then(|m| m.as_str())
        .ok_or((StatusCode::BAD_REQUEST, "请求体缺少 'model' 字段".into()))?
        .to_owned();

    println!(
        "\n>>> 收到来自【{}】的请求，请求模型：{}",
        tenant.name, requested_model
    );

    let is_allowed = tenant.allowed_models.iter().any(|allowed| {
        if allowed.ends_with('*') {
            requested_model.starts_with(allowed.trim_end_matches('*'))
        } else {
            &requested_model == allowed
        }
    });

    if !is_allowed {
        let msg = format!(
            "403 Forbidden：租户【{}】未开通模型【{}】的使用权限！你的可用权限: {:?}",
            tenant.name, requested_model, tenant.allowed_models,
        );
        println!("❌ 权限拦截: {}", msg);
        return Err((StatusCode::FORBIDDEN, msg));
    }

    let (target_url, auth_token, actual_model_name, is_paid) = if requested_model
        .starts_with("ollama/")
    {
        let model_name = requested_model.strip_prefix("ollama/").unwrap();
        (
            format!("{}/chat/completions", state.ollama_base_url),
            None,
            model_name,
            false,
        )
    } else if requested_model.starts_with("deepseek/") || requested_model.starts_with("deepseek-") {
        let model_name = requested_model
            .strip_prefix("deepseek/")
            .unwrap_or(&requested_model);
        (
            "https://api.deepseek.com/chat/completions".to_string(),
            Some(state.deepseek_api_key.clone()),
            model_name,
            true,
        )
    } else {
        return Err((
            StatusCode::BAD_REQUEST,
            "未知模型，请以 ollama/ 或 deepseek/ 开头".into(),
        ));
    };

    if is_paid && tenant.current_spend >= tenant.budget_usd {
        let msg = format!(
            "429 Too Many Requests: 租户【{}】的 DeepSeek 预算已耗尽！已花费: ${:.6} / 预算限额:
${:.6}。网关已自动熔断，拒绝向公网发包！",
            tenant.name, tenant.current_spend, tenant.budget_usd
        );
        println!("🚫 触发预算熔断: {}", msg);
        return Err((StatusCode::TOO_MANY_REQUESTS, msg));
    }

    // 还原实际模型名到请求体
    payload["model"] = serde_json::Value::String(actual_model_name.to_string());

    println!("🚀 正在转发请求至真实上游：{}", target_url);

    let mut req_builder = state.http_client.post(&target_url).json(&payload);
    if let Some(key) = auth_token {
        req_builder = req_builder.header("Authorization", format!("Bearer {key}"));
    }

    let upstream_res = req_builder
        .send()
        .await
        .map_err(|e| (StatusCode::BAD_GATEWAY, format!("连接上游失败: {e}")))?;

    let upstream_status = upstream_res.status();
    let upstream_body: serde_json::Value = upstream_res
        .json()
        .await
        .map_err(|e| (StatusCode::BAD_GATEWAY, format!("解析上游响应失败: {e}")))?;

    if !upstream_status.is_success() {
        return Err((
            StatusCode::from_u16(upstream_status.as_u16()).unwrap_or(StatusCode::BAD_GATEWAY),
            format!("上游返回错误: {upstream_body}"),
        ));
    }

    if is_paid {
        if let Some(usage) = upstream_body.get("usage") {
            let total_tokens = usage
                .get("total_tokens")
                .and_then(|t| t.as_u64())
                .unwrap_or(0);
            // 按照约 $0.000002 / Token 计算
            let cost = (total_tokens as f64) * 0.000002;
            tenant.current_spend += cost;
            println!(
                "💰 计费账单: 租户【{}】本次消耗 {} tokens，花费约 ${:.6}，累计消费: ${:.6} / 限额: ${:.6}",
                tenant.name, total_tokens, cost, tenant.current_spend, tenant.budget_usd
            );
        }
    } else {
        println!("✨ 本地 Ollama 调用完成（免费资源，无需记账）");
    }

    Ok(Json(upstream_body))
}

#[tokio::main]
async fn main() -> Result<()> {
    dotenvy::dotenv().ok();

    let deepseek_key = std::env::var("DEEPSEEK_API_KEY")
        .unwrap_or_else(|_| "请在环境变量设置你的DeepSeekKey".to_string());
    let ollama_url = std::env::var("OLLAMA_BASE_URL")
        .unwrap_or_else(|_| "http://localhost:11434/v1".to_string());

    let mut tenants = HashMap::new();

    tenants.insert(
        "intern".to_string(),
        Tenant {
            id: "tenant-intern".to_string(),
            name: "实习生组".to_string(),
            api_key: "sk-intern-123".to_string(),
            allowed_models: vec!["ollama/*".to_string()], // 仅限 ollama/*
            budget_usd: 0.0,
            current_spend: 0.0,
        },
    );

    tenants.insert(
        "dev".to_string(),
        Tenant {
            id: "tenant-dev".to_string(),
            name: "研发核心组".to_string(),
            api_key: "sk-dev-456".to_string(),
            allowed_models: vec!["ollama/*".to_string(), "deepseek/*".to_string()],
            budget_usd: 0.0001, // 👈 超低限额：通常调用 1 次就会触顶，再次调用直接熔断
            current_spend: 0.0,
        },
    );

    let config = Arc::new(GatewayConfig {
        tenants: RwLock::new(tenants),
        http_client: reqwest::Client::new(),
        deepseek_api_key: deepseek_key,
        ollama_base_url: ollama_url,
    });

    let app = Router::new()
        .route("/v1/chat/completions", post(chat_handler))
        .with_state(config);

    let addr = "0.0.0.0:3000";
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();

    println!("=================================================================");
    println!("🚀 多租户 AI 智能网关已就绪！监听端口：http://127.0.0.1:3000");
    println!("👉 本地 Ollama 后端: http://localhost:11434/v1");
    println!("👉 租户 1【实习生组】: Key = sk-intern-123 (只能用 ollama/*)");
    println!("👉 租户 2【研发核心组】: Key = sk-dev-456    (限额仅 $0.0001 美元)");
    println!("=================================================================");

    axum::serve(listener, app).await?;

    Ok(())
}
