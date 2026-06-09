Topic: Error Handling (RFC-9457)

Gears use RFC-9457 Problem Details for all HTTP error responses.
Errors flow through three layers: Domain -> SDK -> REST Problem.

Error flow:
  DomainError (business logic, in domain/error.rs)
    -> From impl -> <Module>Error (SDK, transport-agnostic)
    -> From impl -> Problem (RFC-9457, implements IntoResponse)
    -> ApiResult<T> (handler return type)

DomainError (domain/error.rs):
  #[domain_model]
  #[derive(Error, Debug)]
  pub enum DomainError {
      #[error("User not found: {id}")]
      UserNotFound { id: Uuid },
      #[error("Email already exists")]
      EmailAlreadyExists { email: String },
      #[error("Permission denied")]
      PermissionDenied,
      #[error("Database error: {0}")]
      Database(#[from] sea_orm::DbErr),
  }

REST mapping (api/rest/error.rs):
  impl From<DomainError> for Problem {
      fn from(err: DomainError) -> Self {
          match err {
              DomainError::UserNotFound { id } => Problem::builder()
                  .type_url(ProblemType::NotFound)
                  .title("User not found")
                  .detail(format!("User with id {} not found", id))
                  .build(),
              DomainError::PermissionDenied => Problem::builder()
                  .type_url(ProblemType::Forbidden)
                  .title("Permission denied")
                  .build(),
              // ...
          }
      }
  }

Available ProblemType constants:
  BadRequest, Unauthorized, Forbidden, NotFound, Conflict,
  UnprocessableEntity, TooManyRequests, InternalServerError

Handler pattern:
  pub async fn get_user(...) -> ApiResult<JsonBody<UserDto>> {
      let user = svc.get_user(&ctx, id).await?;  // DomainError -> Problem
      Ok(Json(UserDto::from(user)))
  }

OperationBuilder error registration:
  .standard_errors(openapi)  // adds 400, 401, 403, 404, 409, 422, 429, 500
  .error_404(openapi)        // or individual codes

Key rules:
  - Use ApiResult<T> and ? for error propagation in handlers
  - Define DomainError with thiserror, impl From<DomainError> for Problem
  - SDK errors are transport-agnostic (no serde derives)
  - Map EnforcerError (AuthZ) to domain errors, not directly to Problem
  - Do not use ProblemResponse (does not exist)

See also:
  cargo gears help topic rest-api
  cargo gears help topic security
