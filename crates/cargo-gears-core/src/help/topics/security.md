Topic: Security (AuthN, AuthZ, SecureConn)

Gears enforce a secure-by-default data path. Module developers get
tenant-scoped, authorized database access without implementing security.

Security data flow:
  Request -> API Gateway (AuthN) -> SecurityContext
    -> Module Handler (PEP)
    -> PolicyEnforcer -> AuthZ Resolver (PDP) -> AccessScope
    -> SecureConn (automatic WHERE clauses) -> Database

Three components:
  1. AuthN Resolver    Validates tokens, produces SecurityContext
                       (subject_id, tenant_id, token_scopes)
  2. AuthZ Resolver    Evaluates policies, returns decision + constraints
  3. Module (PEP)      Calls PolicyEnforcer, passes AccessScope to SecureConn

Route auth posture (compile-time enforced):
  OperationBuilder::get("/my-module/v1/items")
      .authenticated()                        // protected route
      .require_license_features::<License>([]) // license gate
  OperationBuilder::get("/my-module/v1/health")
      .public()                               // no token required

PolicyEnforcer usage:
  // In init(): resolve AuthZ client
  let authz = ctx.client_hub().get::<dyn AuthZResolverClient>()?;
  let enforcer = PolicyEnforcer::new(authz);

  // In service methods: get AccessScope from PDP
  let scope = enforcer.access_scope(ctx, &RESOURCE, action, id).await?;

SecureConn usage (all DB access must be scoped):
  let conn = db.sea_secure();
  let users = conn.find::<user::Entity>(&scope).all(&conn).await?;

Scopable entities (SeaORM):
  #[derive(Scopable)]
  #[secure(tenant_col = "tenant_id", resource_col = "id", no_owner, no_type)]
  pub struct Model { ... }

CRUD patterns:
  LIST:   scope from PolicyEnforcer, pass to SecureConn
  GET:    prefetch with allow_all(), then narrow PDP call with owner_tenant_id
  CREATE: pass owner_tenant_id as resource property to PDP
  UPDATE: prefetch, PDP call, scoped write (TOCTOU-safe via WHERE clause)

Key rules:
  - Modules never parse tokens or construct AccessScope manually
  - No raw DB connections; use SecureConn everywhere
  - Fail-closed: denied PDP, unreachable PDP, or missing constraints -> 403
  - tenant_id is immutable in updates
  - Map EnforcerError to domain errors, never expose PDP internals

See also:
  cargo gears help topic database
  cargo gears help topic rest-api
