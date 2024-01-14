//! Use Copilot as normal GPT, this is a lightweight alternative to https://github.com/aaamoon/copilot-gpt4-service .

use crate::log;
use crate::units::admin;
use crate::utils::{CLIENT, CLIENT_NO_SNI};
use axum::body::Body;
use axum::http::header::*;
use axum::http::Request;
use axum::response::IntoResponse;
use axum::routing::{MethodRouter, Router};
use std::sync::Mutex;
use std::time::UNIX_EPOCH;

fn rand_id(sections: &[usize]) -> Vec<u8> {
    let mut ret = Vec::new();
    for section in sections {
        for _ in 0..*section {
            ret.push(match rand::random::<u8>() >> 4 {
                d @ 0..=9 => d + b'0',
                d @ 10..=255 => d - 10 + b'a',
            });
        }
        ret.push(b'-');
    }
    if ret.last() == Some(&b'-') {
        ret.pop();
    }
    ret
}

async fn post_handler(mut req: Request<Body>) -> impl IntoResponse {
    // verify the token is our own token, then we can `unwrap()` everywhere
    let Ok(Some((copilot_token, copilot_machineid))) = tokio::task::spawn_blocking(|| {
        Some((
            admin::db::get("copilot_token")?,
            admin::db::get("copilot_machineid")?,
        ))
    })
    .await
    else {
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

    if let Some(Ok(v)) = req.headers().get(CONTENT_LENGTH).map(|v| v.to_str()) {
        log!(INFO: "request content-length = {v}")
    }

    // cache the auth header
    static AUTH_HEADER_CACHE: Mutex<(String, u64)> = Mutex::new((String::new(), 0));
    let now = UNIX_EPOCH.elapsed().unwrap().as_secs();
    if AUTH_HEADER_CACHE.lock().unwrap().1 < now - 60 {
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
        let expires_at = body.pointer("/expires_at").unwrap().as_u64().unwrap(); // it's +30 minutes usually
        let auth_header = "Bearer ".to_string() + body.pointer("/token").unwrap().as_str().unwrap();
        *AUTH_HEADER_CACHE.lock().unwrap() = (auth_header, expires_at);
        log!(INFO: "reissued auth header, expires_at = {expires_at}");
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
        .header("Vscode-Machineid", copilot_machineid) // rand_id(&[8, 4, 4, 4, 12])
        .header("Editor-Version", "vscode/1.85.1")
        .header("Editor-Plugin-Version", "copilot-chat/0.11.0")
        .header("Openai-Organization", "github-copilot")
        .header("Openai-Intent", "conversation-panel")
        .header("Copilot-Integration-Id", "vscode-chat")
        .body(req.into_body())
        .unwrap();
    let mut res = CLIENT.fetch(remote_req, None).await.unwrap();
    let mut set_header = |k, v| res.headers_mut().insert(k, HeaderValue::from_static(v));
    set_header(ACCESS_CONTROL_ALLOW_ORIGIN, "*");
    set_header(CONTENT_TYPE, "text/event-stream; charset=utf-8");
    Ok(res)
}

pub fn service() -> Router {
    Router::new().route(
        "/copilotgpt/v1/chat/completions",
        MethodRouter::new().post(post_handler).options([
            (ACCESS_CONTROL_ALLOW_ORIGIN, "*"),
            (ACCESS_CONTROL_ALLOW_HEADERS, "*"),
        ]),
    )
}
