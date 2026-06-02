// simulated_dir=/cyberfabric/modules/some_module/domain/
use serde::Deserialize;

#[allow(dead_code)]
// Should trigger DE0101 - Serde in domain
#[derive(Debug, Clone, Deserialize)]
pub struct Order {
    pub id: String,
    pub total: f64,
}

#[allow(dead_code)]
// Should trigger DE0101 - Serde in domain
#[derive(Debug, Clone, Deserialize)]
pub enum UserRole {
    Admin,
    User,
    Guest,
}

fn main() {}
