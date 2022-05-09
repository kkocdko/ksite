use axum::body::HttpBody;
use once_cell::sync::Lazy;
use tower_http::auth::require_authorization::{Basic, RequireAuthorizationLayer};

pub static TOKEN: Lazy<String> = Lazy::new(|| format!("{:x}", rand::random::<u64>()));

pub fn auth_layer<T: HttpBody + Default>() -> RequireAuthorizationLayer<Basic<T>> {
    RequireAuthorizationLayer::basic("", &TOKEN)
}
