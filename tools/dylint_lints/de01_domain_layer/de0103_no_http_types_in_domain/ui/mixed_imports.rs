// simulated_dir=/cyberfabric/modules/some_module/domain/
// Should not trigger DE0103 - HTTP types in domain
use std::collections::HashMap;
// Should trigger DE0103 - HTTP types in domain
use http::StatusCode;

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub enum OrderStatus {
    Pending,
    Confirmed,
}

#[allow(dead_code)]
pub struct OrderResult {
    pub status: StatusCode,
    pub metadata: HashMap<String, String>,
}

fn main() {}
