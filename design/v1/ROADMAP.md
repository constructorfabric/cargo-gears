# Gears CLI v1 Roadmap

This roadmap converts the v1 design documents into implementation work. It is
organized by dependency order rather than release date: later phases assume the
core manifest pipeline, output model, and tool orchestration are already in place.

## Milestones

| Phase | Theme                   | Outcome                                                                      | Source                                                                         |
|-------|-------------------------|------------------------------------------------------------------------------|--------------------------------------------------------------------------------|
| 0     | Architecture foundation | Testable crates, shared command/output plumbing, tool runner                 | [02](./02-architecture.md), [03](./03-command-surface.md)                      |
| 1     | Manifest-first core     | `Gears.toml` can be validated, rendered, and used as orchestration input | [04](./04-manifest-and-configuration.md)                                       |
| 2     | Build and run           | Deterministic `.gears/<app>-<env>/` generation plus `run` and `build`    | [09](./09-build-and-run.md)                                                    |
| 3     | Scaffolding             | Workspace, module, config, manifest, build, agent, and skill templates       | [05](./05-scaffolding-and-templates.md)                                        |
| 4     | Inspection and help     | Machine-readable list/help surfaces for humans, CI, and LLMs                 | [06](./06-list-and-inspection.md), [07](./07-documentation-and-llm-helpers.md) |
| 5     | Quality gates           | Manifest-aware linting, testing, coverage, and tool bootstrap                | [08](./08-lint-and-test.md)                                                    |
| 6     | CI and automation       | `ci` alias, dry-run contracts, GitHub Actions patterns, LLM workflows        | [10](./10-ci-and-automation.md)                                                |

It's expected that, except for phase 0 that we need to reach an agreement with the team before starting the implementation,
the other phases can be implemented in parallel.

Some of the work is already done in the codebase, however, it will need a refactor to align with the architecture foundation.

## Phase 0: Architecture Foundation

- [ ] Split responsibilities into focused crates or internal modules:
  `cli`, `core`, `codegen`, `templates`, `module-parser`, and `tools`.
- [ ] Introduce a manifest pipeline abstraction with independently testable
  steps: discover, select app/env, validate, resolve modules, generate, execute.
- [ ] Add a shared `ToolRunner` abstraction for Cargo, cargo-generate,
  nextest, llvm-cov, dylint, cargo-deny, and rustup invocations.
- [ ] Add a shared output pipeline that renders the same domain result as `table`, `json`, `yaml`, or `toml`.
- [ ] Standardize structured diagnostics and JSON error output.
- [ ] Convert closed flag sets to enums with `clap::ValueEnum`.
- [ ] Add shared argument handling for `--workspace`, `--config`,
  `--manifest`, `--app`, `--env`, `--dry-run`, `--verbose`, and `--install-missing-tools`.
- [ ] Generate shell completions for `bash`, `zsh`, `fish`, and `powershell`.

## Phase 1: Manifest-First Core

- [ ] Implement `Gears.toml` parsing from the schema in [manifest_schema.rs](./manifest_schema.rs).
- [ ] Preserve comments and formatting for manifest edits using `toml_edit`.
- [ ] Implement manifest discovery with `--manifest` override.
- [ ] Implement app/environment selection with automatic selection when the
  manifest has exactly one app and one environment.
- [ ] Implement module references: `local` from workspace metadata and `remote` from the configured registry.
- [ ] Validate unknown apps/environments, unknown module reference kinds,
  unresolved local modules, unresolved remote modules, duplicate packages,
  dependency conflicts, unsupported policies, and incompatible FIPS requests.
- [ ] Implement `cargo gears manifest validate`.
- [ ] Implement `cargo gears manifest render` as the canonical resolved
  generation model for build, run, CI, and LLM context.
- [ ] Implement manifest mutation commands: `add`, `edit`, `rm`, and `migrate`.
- [ ] Keep the existing `--config` flow working during migration.
- [ ] Add `generate manifest --from-config` as the migration path from config-centric projects.

## Phase 2: Build and Run

- [ ] Generate deterministic server projects under `.gears/<app>-<env>/`
  or `.gears/<name>/` when `--name` is provided.
- [ ] Generate `.gears/<name>/Cargo.toml` from resolved module references,
  features, versions, and path dependencies.
- [ ] Generate `.gears/<name>/src/main.rs` with the configured module list and runtime config loading.
- [ ] Generate `.gears/<name>/.cargo/config.toml` to reuse the workspace target directory.
- [ ] Add `cargo gears run` with manifest selection, config override,
  `--release`, feature flags, `--clean`, and `--dry-run`.
- [ ] Add `cargo gears build` with the same pipeline and output selection.
- [ ] Implement CLI/manifest/default precedence for run and build settings.
- [ ] Implement `--otel`, `--no-otel`, `--fips`, and `--no-fips`.
- [ ] Warn when FIPS is enabled outside production environments.
- [ ] Fail validation when selected modules declare known FIPS incompatibility.
- [ ] Implement `--dry-run --format json` without generating files or invoking Cargo.
- [ ] Preserve existing watch behavior while adding manifest app/env selection.
- [ ] Add watch path customization from manifest policy after the basic watch flow is stable.
- [ ] Ensure `.gears/` is ignored, derived, reproducible, and safe to delete.

## Phase 3: Scaffolding and Templates

- [ ] Implement the template registry with manifest-defined sources and default fallback templates.
- [ ] Resolve template sources from Git, local paths, and embedded fallback files.
- [ ] Use cargo-generate bindings for template execution.
- [ ] Pass normalized template variables for project names, module names, and feature selections.
- [ ] Add `cargo gears generate workspace <path>` and keep `new <path>` as an alias.
- [ ] Generate canonical workspaces with `Cargo.toml`, `modules/`, `config/`,
  `Gears.toml`, `.gitignore`, and optional AI/build assets.
- [ ] Add `cargo gears generate module --template <template> --name <name>`.
- [ ] Support initial module templates: `background-worker`, `api-db-handler`,
  `api-gateway`, `grpc-service`, and `oop-module`.
- [ ] Wire generated modules and SDK crates into workspace members.
- [ ] Normalize generated dependencies to workspace dependencies.
- [ ] Inherit workspace lints in generated modules.
- [ ] Optionally add generated modules to manifest apps and runtime configs.
- [ ] Add `generate config`, `generate manifest`, `generate build`,
  `generate ai --agents`, and `generate ai --skill`.
- [ ] Keep template repositories organized by scaffold family:
  `Init/`, `Modules/`, `Config/`, `Manifest/`, `Build/`, `Agents/`, `Skills/`.
- [ ] Validate templates by generating output, running clippy on generated
  projects, and running tests on generated projects.

## Phase 4: Inspection and Help

- [ ] Implement `cargo gears list modules`.
- [ ] Implement `cargo gears list system-modules`.
- [ ] Implement `cargo gears list local-modules`.
- [ ] Implement `cargo gears list configs`.
- [ ] Implement `cargo gears list apps`.
- [ ] Reuse existing module discovery from Cargo metadata and gears module annotations in `src/*.rs`.
- [ ] Add a simple system module registry until remote registry resolution is available.
- [ ] Include resolved versions, capabilities, dependencies, features, and app/env usage in list output.
- [ ] Add `cargo gears help schema <manifest|config|module>`.
- [ ] Generate schema help from the same Rust types used by parsing where possible.
- [ ] Add `cargo gears help docs <rust-path>` as an alias for the existing`docs` command.
- [ ] Add `cargo gears help topic <topic>` for `manifest`, `module-refs`, `generated-server`, `fips`, and `otel`.

## Phase 5: Lint, Test, and Coverage

- [ ] Make `cargo gears lint` manifest-aware with `--app` and `--env`.
- [ ] Preserve existing lint modes for fmt, clippy, dylint, strict, and workspace checks.
- [ ] Apply manifest lint policy for clippy, fmt, feature-set testing, and dylint skips.
- [ ] Add `cargo gears test` with runner resolution from CLI, app policy, workspace default, then `cargo-test`.
- [ ] Support runner enum values `cargo-test` and `nextest`. If possible, by including nextest as a dependency instead
  of relying on the system having it installed.
- [ ] Print exact "install" commands for missing optional tools.
- [ ] Implement `--install-missing-tools` for approved missing tools.
- [ ] Implement manifest-defined test matrices and named test sets.
- [ ] Support custom test commands from manifest policy.
- [ ] Add coverage mode with `cargo llvm-cov`.
- [ ] Support coverage formats: `summary`, `lcov`, `html`, and `json`.
- [ ] Label initial workspace-only coverage limitations clearly when app/module filtering is not yet available.

## Phase 6: CI and Automation

- [ ] Add `cargo gears ci` as an alias for manifest validation, lint, test, and build.
- [ ] Make `ci` read app/env lint, test, and build policy from the manifest.
- [ ] Support strict CI behavior that turns warnings into failures.
- [ ] Provide generated or documented GitHub Actions workflow examples.
- [ ] Ensure dry-run JSON output is stable enough for CI pre-flight checks.
- [ ] Ensure `manifest render --format json` provides a complete project state snapshot for automation and LLMs.
- [ ] Generate `AGENTS.md`, `CLAUDE.md`, and `SKILL.md` assets with command
  references, workflow examples, and generated path warnings.
- [ ] Keep read-only commands idempotent and make mutating commands predictable
  with explicit flags such as `--force` where needed.

## Cross-Cutting Requirements

- [ ] Keep generated artifacts deterministic: no timestamps, random IDs, or unsorted dependency/module lists.
- [ ] Validate before side effects, network calls, and subprocess invocation.
- [ ] Redact secret-like values from render and diagnostic output.
- [ ] Never embed credentials in generated files.
- [ ] Keep table output human-readable and JSON output stable for tools.
- [ ] Surface wrapped tool stderr with enough context to reproduce failures.
- [ ] Preserve backward-compatible commands and flags across v1 migration.
- [ ] Add tests around manifest parsing, validation, rendering, command planning,
  generated files, output formats, and tool argument construction.
