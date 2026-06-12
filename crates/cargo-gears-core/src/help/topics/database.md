Topic: Database Patterns

Gears use SecureConn for all database access, with gear-owned migrations
and a repository pattern based on SeaORM.

Executors:
  SecureConn   Scoped DB connection (from db.sea_secure())
  SecureTx     Scoped DB connection inside a transaction
  DBRunner     Trait implemented by both; use in repository signatures

Repository pattern:
  pub async fn find_by_id(
      runner: &impl DBRunner,       // works with SecureConn or SecureTx
      scope: &AccessScope,
      id: Uuid,
  ) -> Result<Option<Model>, ScopeError> {
      user::Entity::find_by_id(id)
          .secure()
          .scope_with(scope)
          .one(runner)
          .await
  }

Transactions:
  let (conn, result) = secure_conn
      .in_transaction_mapped(DomainError::db_error, |tx| {
          Box::pin(async move {
              // tx is &SecureTx, use as runner for repo calls
              repo.create(tx, &scope, model).await?;
              Ok(())
          })
      })
      .await;
  result?;

Scoped queries:
  // Auto-scoped
  conn.find::<Entity>(&scope).all(&conn).await?;
  conn.find_by_id::<Entity>(&scope, id)?.one(&conn).await?;

  // Manual scoping (complex queries)
  Entity::find()
      .filter(Column::Email.eq(email))
      .secure()
      .scope_with(&scope)
      .one(&conn).await?;

Scoped mutations:
  // Insert (validates tenant_id is within scope)
  secure_insert::<Entity>(active_model, &scope, &conn).await?;

  // Update one (checks row exists in scope, tenant_id is immutable)
  conn.update_with_ctx::<Entity>(&scope, id, active_model).await?;

Migrations (raw SQL allowed ONLY here):
  impl DatabaseCapability for MyGear {
      fn migrations(&self) -> Vec<Box<dyn MigrationTrait>> {
          migrations::Migrator::migrations()
      }
  }
  // Each gear gets its own migration history table

Key rules:
  - No plain SQL in handlers/services/repos, only in migrations
  - Repository methods accept &impl DBRunner, not &SecureConn
  - Use in_transaction_mapped for multi-step mutations
  - SecureConn applies AccessScope as automatic WHERE clauses
  - Add indexes on security columns (tenant_id, resource_id)
  - tenant_id is immutable in updates (enforced at runtime)

See also:
  cargo gears help topic security
  cargo gears help topic gear-layout
