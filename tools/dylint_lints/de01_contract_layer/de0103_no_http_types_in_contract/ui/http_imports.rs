// simulated_dir=/cyberfabric/modules/some_module/domain/
// Should trigger DE0103 - HTTP types in domain
use http::StatusCode;
// Should trigger DE0103 - HTTP types in domain
use http::Method;
// Should trigger DE0103 - HTTP types in domain
use axum::http::HeaderMap;

#[allow(dead_code)]
pub struct OrderResult {
    pub status: StatusCode,
}

#[allow(dead_code)]
pub struct RequestInfo {
    pub method: Method,
    pub headers: HeaderMap,
}

fn main() {}
