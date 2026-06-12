// simulated_dir=/cyberfabric/modules/some_module/api/rest/
#![feature(register_tool)]
#![register_tool(cf_gears_toolkit_macros)]
#![allow(dead_code)]

// Should not trigger DE0104 - api_dto in domain
#[cf_gears_toolkit_macros::api_dto(request, response)]
pub struct UserDto {
    pub id: String,
    pub name: String,
}

// Should not trigger DE0104 - api_dto in domain
pub struct PlainStruct {
    pub field: String,
}

fn main() {}
