use crate::units::admin::db_get;
use axum::body::HttpBody;
use once_cell::sync::Lazy;
use tower_http::auth::require_authorization::{Basic, RequireAuthorizationLayer};

pub static AUTH_KEY: Lazy<String> = Lazy::new(|| match db_get("auth_key") {
    Some((v,)) => String::from_utf8(v).unwrap(),
    None => format!("{:x}", rand::random::<u64>()),
});

pub fn auth_layer<T: HttpBody + Default>() -> RequireAuthorizationLayer<Basic<T>> {
    RequireAuthorizationLayer::basic("", &AUTH_KEY)
}
