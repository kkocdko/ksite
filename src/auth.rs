use axum::body::HttpBody;
use once_cell::sync::Lazy;
use tower_http::auth::require_authorization::{Basic, RequireAuthorizationLayer};

pub static TOKEN: Lazy<String> = Lazy::new(|| format!("{:x}", rand::random::<u64>()));

pub fn auth_layer<T: HttpBody + Default>() -> RequireAuthorizationLayer<Basic<T>> {
    RequireAuthorizationLayer::basic("", &TOKEN)
}

// MethodRouter::new().get(
//     |u: WebSocketUpgrade, c: ConnectInfo<SocketAddr>| async move {
//         if c.0.ip() != IpAddr::V4(Ipv4Addr::LOCALHOST) {
//             return "only allowed for localhost".into_response();
//         }
//         u.on_upgrade(ws_handler)
//     },
// )
