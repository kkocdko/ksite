//! Use Copilot as normal GPT, this is a lightweight alternative to https://github.com/aaamoon/copilot-gpt4-service .

use crate::log;
use crate::units::admin;
use crate::utils::{rand_id, CLIENT, CLIENT_NO_SNI};
use axum::body::Body;
use axum::http::header::*;
use axum::http::Request;
use axum::response::IntoResponse;
use axum::routing::{MethodRouter, Router};
use std::sync::Mutex;
use std::time::UNIX_EPOCH;

async fn post_handler(mut req: Request<Body>) -> impl IntoResponse {
    // verify the token is our own token, then we can `unwrap()` everywhere
    let copilot_token = admin::db::get("copilot_token".to_owned()).await;
    let copilot_machineid = admin::db::get("copilot_machineid".to_owned()).await;
    let (Some(copilot_token), Some(copilot_machineid)) = (copilot_token, copilot_machineid) else {
        return Err("please set copilot_token and copilot_machineid in database.");
    };
    let Some(true) = req
        .headers()
        .get(AUTHORIZATION)
        .map(|v| v.as_bytes().ends_with(&copilot_token))
    else {
        // if you use many copilot token on same ip, you're gonna to be banned!
        return Err("only the token in database is allowed.");
    };

    // cache the auth header
    static AUTH_HEADER_CACHE: Mutex<(String, u64)> = Mutex::new((String::new(), 0));
    let now = UNIX_EPOCH.elapsed().unwrap().as_secs();
    if AUTH_HEADER_CACHE.lock().unwrap().1 < now - 120 {
        let mut get_token_auth = b"token ".to_vec();
        get_token_auth.extend(&req.headers()[AUTHORIZATION].as_bytes()["Bearer ".len()..]);
        let req = Request::get("https://api.github.com/copilot_internal/v2/token")
            .header(HOST, "api.github.com")
            .header(AUTHORIZATION, get_token_auth)
            .header(USER_AGENT, "GitHubCopilotChat/0.11.0")
            .body(Body::empty())
            .unwrap();
        let resolved = Some("20.200.245.245:443".to_string());
        let res = CLIENT_NO_SNI.fetch(req, resolved).await.unwrap();
        let body = axum::body::to_bytes(axum::body::Body::new(res), usize::MAX).await;
        let body = serde_json::from_slice::<serde_json::Value>(&body.unwrap()).unwrap();
        let expires_at = body.get("expires_at").unwrap().as_u64().unwrap(); // it's +30 minutes usually
        let auth_header = "Bearer ".to_string() + body.get("token").unwrap().as_str().unwrap();
        *AUTH_HEADER_CACHE.lock().unwrap() = (auth_header, expires_at);
        log!(info: "reissued auth header, expires_at = {expires_at}");
    };
    let auth_header = AUTH_HEADER_CACHE.lock().unwrap().0.to_owned();

    // {"Authorization":"Bearer tid=xxx...","X-Request-Id":"xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx","X-GitHub-Api-Version":"2023-07-07","VScode-SessionId":"xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxxxxxxxxxxxxxxx","VScode-MachineId":"xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx","Editor-Version":"vscode/1.85.1","Editor-Plugin-Version":"copilot-chat/0.11.1","Openai-Organization":"github-copilot","Copilot-Integration-Id":"vscode-chat"}
    let remote_req = Request::post("https://api.githubcopilot.com/chat/completions")
        .header(HOST, "api.githubcopilot.com")
        .header(CONNECTION, "close")
        .header(CONTENT_TYPE, &req.headers()[CONTENT_TYPE])
        .header(ACCEPT, &req.headers()[ACCEPT])
        .header(ACCEPT_ENCODING, &req.headers()[ACCEPT_ENCODING])
        .header(AUTHORIZATION, auth_header)
        .header(USER_AGENT, "GitHubCopilotChat/0.11.0")
        .header("X-GitHub-Api-Version", "2023-07-07")
        .header("X-Request-Id", rand_id(&[8, 4, 4, 4, 12]))
        .header("Vscode-Sessionid", rand_id(&[8, 4, 4, 4, 25]))
        .header("Vscode-Machineid", copilot_machineid.as_ref()) // rand_id(&[8, 4, 4, 4, 12])
        .header("Editor-Version", "vscode/1.85.1")
        .header("Editor-Plugin-Version", "copilot-chat/0.11.0")
        .header("Openai-Organization", "github-copilot")
        .header("Openai-Intent", "conversation-panel")
        .header("Copilot-Integration-Id", "vscode-chat")
        .body(req.into_body())
        .unwrap();
    let mut res = CLIENT.fetch(remote_req, None).await.unwrap();
    let is_sse = res.headers().get(CONTENT_LENGTH).is_none();
    let mut set_header = |k, v| res.headers_mut().insert(k, HeaderValue::from_static(v));
    set_header(ACCESS_CONTROL_ALLOW_ORIGIN, "*");
    if is_sse {
        set_header(CONTENT_TYPE, "text/event-stream; charset=utf-8");
    }
    Ok(res)
}

pub fn service() -> Router {
    Router::new()
        .route(
            "/copilotgpt/v1/models",
            MethodRouter::new().get(r#"{"data":[{"created":1677610602,"id":"gpt-3.5-turbo","object":"model","owned_by":"openai","parent":null,"permission":[{"allow_create_engine":false,"allow_fine_tuning":false,"allow_logprobs":true,"allow_sampling":true,"allow_search_indices":false,"allow_view":true,"created":1677610602,"group":null,"id":"modelperm-0169379b25a2","is_blocking":false,"object":"model_permission","organization":"*"}],"root":"gpt-3.5-turbo"},{"created":1677610602,"id":"gpt-4","object":"model","owned_by":"openai","parent":null,"permission":[{"allow_create_engine":false,"allow_fine_tuning":false,"allow_logprobs":true,"allow_sampling":true,"allow_search_indices":false,"allow_view":true,"created":1677610602,"group":null,"id":"modelperm-346ef4071a8c","is_blocking":false,"object":"model_permission","organization":"*"}],"root":"gpt-4"}],"object":"list"}"#),
        )
        .route(
            "/copilotgpt/v1/chat/completions",
            MethodRouter::new().post(post_handler).options([
                // for Preflight https://developer.mozilla.org/en-US/docs/Glossary/Preflight_request
                (ACCESS_CONTROL_ALLOW_ORIGIN, "*"),
                (ACCESS_CONTROL_ALLOW_HEADERS, "*"),
            ]),
        )
}
