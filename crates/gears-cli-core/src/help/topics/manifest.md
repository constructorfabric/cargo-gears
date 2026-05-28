Topic: Manifest (Gears.toml)

The Gears.toml manifest is the central configuration for multi-environment
builds. It replaces the need to pass --config to build/run by declaring all
generation inputs in one file.

Key concepts:
  - One manifest per workspace, usually at the workspace root
  - Apps group environments: apps.<app>.<env>
  - Each environment declares its config path, modules, and policies
  - The manifest controls the generated-dir, build profile, watch mode, etc.

Workflow:
  1. Write Gears.toml with your apps and environments
  2. Run: cargo gears build --app myapp --env dev
  3. The CLI resolves the manifest, discovers modules, and generates the server
  4. Use --dry-run to inspect the generated structure without building

Commands that use the manifest:
  cargo gears build --app <APP> --env <ENV>
  cargo gears run   --app <APP> --env <ENV>
  cargo gears manifest validate
  cargo gears manifest ls

See also: cargo gears help schema manifest
