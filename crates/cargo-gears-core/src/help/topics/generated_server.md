Topic: Generated Server

The CLI generates an ephemeral Cargo project that aggregates your modules into
a single runnable binary.

Location:
  <workspace>/<generated-dir>/<name>/
  Default: .gears/<app>-<env>/

Generated files:
  Cargo.toml           Declares dependencies on all selected modules
  .cargo/config.toml   Points target-dir back to workspace target/
  src/main.rs          Bootstraps the modkit server with all modules

The generated server reads its runtime config from the GEARS_CONFIG
environment variable, which the CLI sets automatically during build/run.

Key points:
  - The generated project is ephemeral; regenerated on every build/run
  - Dependencies are rewritten to workspace-relative paths
  - The --name flag overrides the default <app>-<env> project name
  - Use --dry-run to inspect without building
  - The generated-dir is controlled by manifest workspace.generated-dir

Manual execution:
  If you run the compiled binary directly, you must set GEARS_CONFIG:
    GEARS_CONFIG=config/app-dev.yml ./target/debug/app-dev
