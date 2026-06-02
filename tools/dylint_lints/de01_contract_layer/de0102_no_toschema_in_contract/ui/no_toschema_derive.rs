// simulated_dir=/cyberfabric/modules/some_module/domain/
#[allow(dead_code)]
// Should not trigger DE0102 - ToSchema in domain
#[derive(Debug, Clone, PartialEq)]
pub struct Product {
    pub id: String,
    pub name: String,
    pub price: f64,
}

#[allow(dead_code)]
// Should not trigger DE0102 - ToSchema in domain
#[derive(Clone, PartialEq)]
pub enum Status {
    Active,
    Inactive,
    Pending,
}

fn main() {}
