use axum::{routing::post, extract::Json, Router};
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use cedar_policy::*;


#[derive(Debug, Deserialize, Serialize)]
struct McpRequest {
    jsonrpc: String,
    method: String,
    params: serde_json::Value,
    id: u64,
}

#[derive(Debug, Serialize)]
struct McpResponse {
    jsonrpc: String,
    id: u64,
    result: serde_json::Value,
}


async fn handle_mcp_request(Json(payload): Json<McpRequest>) -> Json<McpResponse> {

    let tool_name = payload.params.get("name")
        .and_then(|v| v.as_str())
        .unwrap_or("unknown");

    println!("🛡️ Aegis Intercepting tool: {}", tool_name);


    let policy_src = std::fs::read_to_string("policies.cedar").expect("Missing policies.cedar file");
    let policy_set = policy_src.parse::<PolicySet>().expect("Policy syntax error");


    let principal = "User::\"AI_Agent\"".parse().expect("Invalid Principal");
    let action = "Action::\"call_tool\"".parse().expect("Invalid Action");
    let resource = format!("Resource::\"{}\"", tool_name).parse().unwrap_or("Resource::\"Unknown\"".parse().unwrap());

    // Capital 'C' on Context
    let request = Request::new(principal, action, resource, Context::empty(), None).expect("Request failed");

    let authorizer = Authorizer::new();
    let answer = authorizer.is_authorized(&request, &policy_set, &Entities::empty());

    if answer.decision() == Decision::Allow {
        println!("✅ Access Granted");
        Json(McpResponse {
            jsonrpc: "2.0".to_string(),
            id: payload.id,
            result: serde_json::json!({ "status": "allowed" }),
        })
    } else {
        println!("🚫 Access Denied");
        Json(McpResponse {
            jsonrpc: "2.0".to_string(),
            id: payload.id,
            result: serde_json::json!({ "status": "denied" }),
        })
    }
}

#[tokio::main]
async fn main() {
    let app = Router::new().route("/mcp", post(handle_mcp_request));
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    
    // Fixed: Removed the extra set of parentheses
    println!("🚀 Aegis MCP Interceptor running on {}", addr);
    
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}