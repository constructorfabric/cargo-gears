// simulated_dir=/cyberfabric/modules/some_module/domain/
#[allow(dead_code)]
// Should trigger DE0101 - Serde in domain
#[derive(Debug, Clone, serde::Serialize)]
pub struct WithQualifiedSerialize {
    pub id: String,
}

#[allow(dead_code)]
// Should trigger DE0101 - Serde in domain
#[derive(Debug, serde::Deserialize)]
pub struct WithQualifiedDeserialize {
    pub id: String,
}

#[allow(dead_code)]
// Should trigger DE0101 - Serde in domain
#[derive(serde::Serialize, serde::Deserialize)]
pub struct WithBothQualified {
    pub id: String,
}

fn main() {}
