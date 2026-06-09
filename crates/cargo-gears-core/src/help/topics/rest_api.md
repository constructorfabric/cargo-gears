Topic: REST API (OperationBuilder)

ToolKit provides type-safe route registration via OperationBuilder,
integrated with OpenAPI, auth, errors, SSE, and OData.

Basic pattern:
  OperationBuilder::get("/my-module/v1/items")
      .operation_id("my_module.list_items")
      .authenticated()
      .require_license_features::<License>([])
      .handler(handlers::list_items)
      .json_response_with_schema::<Page<ItemDto>>(openapi, StatusCode::OK, "Items")
      .with_odata_filter::<ItemDtoFilterField>()
      .with_odata_select()
      .with_odata_orderby::<ItemDtoFilterField>()
      .standard_errors(openapi)
      .register(router, openapi);

Auth posture (required before .register()):
  .authenticated()  Protected; then requires .require_license_features() or
                    .no_license_required()
  .public()         No token required; auto-satisfies license requirement

Content types:
  .json_request::<T>(openapi, desc)            JSON body
  .json_response_with_schema::<T>(openapi,..)  JSON response with schema
  .multipart_file_request("file", desc)        Multipart upload
  .octet_stream_request(desc)                  Raw binary
  .sse_json::<T>(openapi, desc)                Server-Sent Events
  .binary_response(openapi, ..)                Binary download

Error registration:
  .standard_errors(openapi)   Adds 400, 401, 403, 404, 409, 422, 429, 500
  .error_404(openapi)         Individual error codes
  .problem_response(openapi, StatusCode::CONFLICT, "desc")

Handler conventions:
  - Extract SecurityContext: Extension(ctx): Extension<SecurityContext>
  - Inject service: Extension(svc): Extension<Arc<Service>>
  - Return ApiResult<T> with ? operator (DomainError -> Problem via From)
  - Use created_json(), no_content() helpers from toolkit::api::prelude

Key rules:
  - Modules do NOT own the HTTP server; api-gateway owns the Axum router
  - Attach service ONCE after all routes: router.layer(Extension(service))
  - Follow operation_id convention: <crate>.<resource>.<action>
  - All 4xx/5xx errors must be RFC-9457 Problem responses
  - Do not add transport middleware at module level (CORS, timeouts, etc.)
  - Handlers should complete within ~30s; longer work returns 202 Accepted

See also:
  cargo gears help topic errors
  cargo gears help topic security
