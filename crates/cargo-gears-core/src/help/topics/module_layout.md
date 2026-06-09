Topic: Module Layout and SDK Pattern

Every module follows a DDD-light layout with an SDK crate and an
implementation crate. Module names MUST be kebab-case.

Canonical directory structure:
  modules/<name>/
    <name>-sdk/                   Public API for consumers
      src/
        lib.rs                    Re-exports main types
        api.rs                    ClientHub trait(s) with SecurityContext
        models.rs                 Transport-agnostic domain models
        errors.rs                 Transport-agnostic errors
    <name>/                       Module implementation
      src/
        lib.rs                    Re-exports SDK types + module struct
        module.rs                 #[toolkit::module(...)] + capabilities
        config.rs                 Typed config (optional)
        api/rest/
          dto.rs                  REST DTOs (serde/utoipa/ODataFilterable)
          handlers.rs             Axum handlers
          routes.rs               OperationBuilder route registration
        domain/
          service.rs              Business logic
          error.rs                DomainError enum
          local_client.rs         SDK trait impl for in-process calls
        infra/storage/
          entity.rs               SeaORM entities with #[derive(Scopable)]
          mapper.rs               Entity <-> SDK model conversions
          migrations/              SeaORM migrations (raw SQL allowed here)

Module registration:
  #[toolkit::module(
      name = "my-module",
      deps = ["foo", "bar"],
      capabilities = [db, rest],
      client = my_module_sdk::MyModuleApi,
  )]
  pub struct MyModule { ... }

Key rules:
  - SDK trait methods take &SecurityContext as first param
  - All domain/ types must have #[domain_model] (enforced by lint DE0309)
  - Register client in init(): ctx.client_hub().register::<dyn Api>(impl)
  - SeaORM entities derive Scopable with #[secure(tenant_col, ...)]
  - No infrastructure types allowed in domain/ (DB, HTTP, IO types rejected)

Data type naming matrix:
  DB layer       Domain/SDK      API Request         API Response
  ActiveModel    NewUser         CreateUserRequest   UserResponse
  UserEntity     User            Path(id)            UserResponse
  UserEntity     User (Vec)      ListUsersQuery      Page<UserView>

See also:
  cargo gears help topic architecture
  cargo gears help topic security
  cargo gears help topic errors
