// simulated_dir=/cyberfabric/modules/some_module/domain/
use utoipa::ToSchema;

#[allow(dead_code)]
// Should trigger DE0102 - ToSchema in domain
#[derive(Debug, Clone, ToSchema)]
pub struct Product {
    pub id: String,
    pub name: String,
    pub price: f64,
}

#[allow(dead_code)]
// Should trigger DE0102 - ToSchema in domain
#[derive(Debug, Clone, ToSchema)]
pub struct Order {
    pub id: String,
    pub total: f64,
}

#[allow(dead_code)]
// Should trigger DE0102 - ToSchema in domain
#[derive(Debug, Clone, ToSchema)]
pub enum Status {
    Active,
    Inactive,
    Pending,
}

fn main() {}
