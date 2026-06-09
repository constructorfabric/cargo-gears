Topic: CLI Reference (cargo gears)

cargo gears is the CLI for managing Constructor Fabric Gears development.
It handles workspace scaffolding, config management, server generation,
building, running, linting, testing, deploying, and source inspection.

General guidelines for LLMs:
  - When adding dependencies use `cargo add`, do not edit Cargo.toml manually
  - When linting, use `cargo gears lint`, not cargo check/clippy/fmt directly
  - Always verify that the application runs successfully after modifying code
  - Prefer system modules over implementing custom ones (check available
    modules with `cargo gears ls modules --system`)
  - Do not create modules from scratch; use `cargo gears generate module`
  - Pay attention to the "deps" section of system modules

Invocation: cargo gears <command>

Command tree:
  generate workspace  Initialize a Gears workspace from a template
  generate module     Generate a module from a template (background-worker,
                      api-db-handler, api-gateway)
  generate config     Generate a runtime YAML config (dev, prod, db)
  new                 Alias for generate workspace
  config mod add/rm   Add or remove a module in the YAML config
  config mod db       Manage module-level database config
  config db           Manage global database server config
  src                 Resolve Rust source for crate/module/item queries
  help schema         Print schema for manifest, config, or module formats
  help topic          Print operational documentation for a topic
  lint                Run workspace linting (fmt, clippy, dylint)
  ls modules          List system and workspace modules
  manifest validate   Validate Gears.toml
  manifest ls         List app/environment pairs
  test                Run manifest-driven tests (cargo test or nextest)
  build               Generate server project and build it
  run                 Generate server project and run it
  deploy              Build a Docker image with the workspace Dockerfile
  tools               Install or upgrade Rust tooling (rustup, clippy, fmt)

Shared argument patterns:
  -p, --path <PATH>      Workspace root (default: current directory)
  -c, --config <PATH>    Config file path (required for config/deploy)
  --manifest <PATH>      Gears.toml path (default: Gears.toml)
  --app <APP> --env <ENV> Select app/environment from manifest
  --dry-run              Generate without building/running

Quick start flow:
  cargo gears new /tmp/my-app
  cargo gears generate module --template background-worker -p /tmp/my-app
  cargo gears generate config --template dev --app app1 --env dev -p /tmp/my-app
  cargo gears config mod add background-worker -p /tmp/my-app \
      -c /tmp/my-app/config/app1-dev.yml
  cargo gears run -p /tmp/my-app --app app1 --env dev

For detailed command documentation:
  cargo gears <command> --help
  cargo gears help schema <manifest|config|module>

Available help topics:
  architecture      Framework architecture and principles
  module-layout     Module directory structure and SDK pattern
  security          AuthN, AuthZ, SecureConn, AccessScope
  rest-api          OperationBuilder, OpenAPI, SSE, OData
  clienthub         Typed ClientHub, plugins, GTS
  errors            RFC-9457 Problem error handling
  database          SecureConn, transactions, migrations
  lifecycle         Module lifecycle, cancellation, background tasks
  gears-catalog     Module categories and dependency rules
  manifest          Gears.toml and manifest-driven workflows
  module-refs       How local and remote modules are referenced
  generated-server  How the generated server project works
  fips              FIPS mode activation and usage
  otel              OpenTelemetry activation and configuration
  cli               This topic
