// simulated_dir=/cyberfabric/modules/some_module/domain/
#[derive(Debug, Clone)]
#[allow(dead_code)]
// Should not trigger DE0103 - HTTP types in domain
pub enum OrderStatus {
    Pending,
    Confirmed,
    Shipped,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
// Should not trigger DE0103 - HTTP types in domain
pub struct OrderResult {
    pub status: OrderStatus,
}

fn main() {}
