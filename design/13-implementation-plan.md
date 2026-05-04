# 13. Implementation Plan

## Table of Contents

1. [Principles](#principles)
2. [Phase 0: Foundation](#phase-0-foundation)
3. [Phase 1: Manifest Model](#phase-1-manifest-model)
4. [Phase 2: Manifest-Aware Build and Run](#phase-2-manifest-aware-build-and-run)
5. [Phase 3: Inspection and Help](#phase-3-inspection-and-help)
6. [Phase 4: Template Generation](#phase-4-template-generation)
7. [Phase 5: Test Orchestration](#phase-5-test-orchestration)
8. [Phase 6: Build Outputs](#phase-6-build-outputs)
9. [Phase 7: Polish and Stabilization](#phase-7-polish-and-stabilization)
10. [Resolved Design Decisions](#resolved-design-decisions)

## Principles

- Keep existing commands working during migration. No regressions.
- Add typed models before adding new flags.
- Prefer dry-run/render commands before commands that mutate files.
- Use JSON output as the stable machine contract from day one.
- Keep runtime config backward compatible until a clear migration path exists.
- Extract crates incrementally: start with internal boundaries, promote when stable.
- Write tests before implementing each phase. Snapshot tests for generation, integration tests for commands.

## Phase 0: Foundation

**Goal**: establish the architectural base for all subsequent phases without changing user-facing behavior.

### Deliverables

- [ ] Extract shared argument structs (`WorkspaceArgs`, `OutputArgs`) into a common module.
- [ ] Add `OutputFormat` enum and `--format` flag plumbing to existing list-like commands (`config mod list`).
- [ ] Add structured error types (`CliError`, `Diagnostic`) and wire them into the error handling path.
- [ ] Add exit code constants matching the defined taxonomy.
- [ ] Add `--dry-run` plumbing (flag accepted, no-op initially) to `run`, `build`, `deploy`.
- [ ] Add `CF_CLI_LOG` environment variable support for debug logging.
- [ ] Set up snapshot test infrastructure for generated files.

### Success Criteria

- Existing commands pass all current tests.
- `config mod list --format json` produces valid JSON.
- Errors print diagnostic codes.
- `--dry-run` is accepted without error on action commands.

### Estimated Effort

2-3 weeks.

## Phase 1: Manifest Model

**Goal**: introduce `Cyberfabric.toml` as a typed, validated domain model.

### Deliverables

- [ ] Define manifest Rust structs with `serde::Deserialize` and `serde::Serialize`.
- [ ] Implement `Cyberfabric.toml` discovery (walk up from workspace root).
- [ ] Implement `manifest validate` with all rules from [04-manifest-and-configuration.md](./04-manifest-and-configuration.md#validation-rules).
- [ ] Implement `manifest render` with JSON/table output.
- [ ] Implement `generate manifest` (including `--from-config` migration).
- [ ] Add `schema_version` handling (parse, validate, refuse unknown versions).
- [ ] Add `manifest add`, `manifest edit`, `manifest rm` for basic CRUD.

### Success Criteria

- Existing configs can be converted into a manifest with `generate manifest --from-config`.
- `manifest render` produces the same dependency model currently derived from config metadata.
- Invalid module references fail with precise error codes and suggestions.
- `manifest validate --format json` produces an array of typed diagnostics.

### Estimated Effort

3-4 weeks.

## Phase 2: Manifest-Aware Build and Run

**Goal**: make `run` and `build` work with the manifest while preserving the `--config` flow.

### Deliverables

- [ ] Add `--manifest`, `--env`, `--app` flags to `run` and `build`.
- [ ] Implement manifest-based module resolution in the generation pipeline.
- [ ] Generate project path as `.cyberfabric/<env>-<app>/` when using manifest.
- [ ] Keep `--config` flow working when no manifest exists.
- [ ] Implement `--dry-run` for `run` and `build` (print resolved model, exit 0).
- [ ] Implement `--name` override for generated project path.
- [ ] Wire `deploy` to accept `--env`/`--app` alongside existing `--config`.

### Success Criteria

- `run --env dev --app app1` works without `--config` when the manifest declares it.
- `build --env dev --app app1` generates `.cyberfabric/dev-app1/`.
- `run -c config/quickstart.yml` keeps working (backward compatible).
- `run --dry-run --format json` prints the resolved model.

### Estimated Effort

3-4 weeks.

## Phase 3: Inspection and Help

**Goal**: add read-only inspection commands for workspace state and schema documentation.

### Deliverables

- [ ] Implement `list modules`, `list local-modules`, `list system-modules`.
- [ ] Implement `list apps` and `list configs`.
- [ ] Add `--format table|json|yaml|toml` to all list commands.
- [ ] Implement `help schema` (manifest, config, module) with format output.
- [ ] Implement `help topic` with embedded topic documentation.
- [ ] Add `help docs` as alias for `docs`.
- [ ] Implement auto-format detection (TTY -> table, pipe -> json).

### Success Criteria

- `list modules --format json` produces stable, typed JSON.
- `list apps` shows all manifest-defined apps with policy summaries.
- `help schema manifest --format json` produces machine-readable schema documentation.
- Schema help is generated from or checked against the actual Rust parser types.

### Estimated Effort

2-3 weeks.

## Phase 4: Template Generation

**Goal**: unify all scaffolding under the `generate` command namespace.

### Deliverables

- [ ] Implement `generate workspace` with `--profile` (minimal, service, platform).
- [ ] Implement `generate module` with `--name` and `--add-to-manifest`.
- [ ] Implement `generate config` (dev, prod, test kinds).
- [ ] Implement `generate build` (docker, compose, helm, ci-github).
- [ ] Implement `generate agents` and `generate skill`.
- [ ] Wire `init` as alias for `generate workspace`.
- [ ] Wire `mod add` as alias for `generate module`.
- [ ] Implement template registry from manifest (`[[templates.*]]`).
- [ ] Add post-generation hooks (format, validate, summary, next steps).

### Success Criteria

- `generate workspace ./my-app --profile service` produces a workspace with manifest, config, module, and Docker
  template.
- `generate module background-worker --name jobs --add-to-manifest env.dev.app1` creates the module and wires it into
  the manifest.
- `init` and `mod add` work as before (alias behavior).
- Template registry resolution works (local -> manifest -> git -> embedded).

### Estimated Effort

3-4 weeks.

## Phase 5: Test Orchestration

**Goal**: implement the `test` command that is currently a placeholder.

### Deliverables

- [ ] Implement `test` with default behavior (unit + integration).
- [ ] Add runner selection (`--runner cargo-test|nextest`).
- [ ] Add test set selection (`--set unit|integration|e2e|module|all`).
- [ ] Add module scoping (`--module <name>`).
- [ ] Add manifest test policy reading (`--env`/`--app`).
- [ ] Add coverage mode (`--coverage`, `--coverage-format`).
- [ ] Add e2e test flow (generate app, set env vars, run tests).
- [ ] Add tool detection (nextest, cargo-llvm-cov) with install suggestions.

### Success Criteria

- `test` runs workspace unit + integration tests by default.
- `test --module background-worker` scopes to the correct package.
- `test --env dev --app app1 --set e2e` generates the app and runs e2e tests.
- `test --coverage --coverage-format lcov` produces LCOV output.
- Missing tools produce clear install instructions.

### Estimated Effort

3-4 weeks.

## Phase 6: Build Outputs

**Goal**: add Docker and Helm as build outputs alongside the existing binary build.

### Deliverables

- [ ] Add `--output binary|docker|helm|all` to `build`.
- [ ] Refactor Docker build logic from `deploy` into a shared `DockerBuilder`.
- [ ] Wire `build --output docker` through the shared builder.
- [ ] Implement `build --output helm` (chart validation, values rendering, `helm package`).
- [ ] Add build summary output (table and JSON).
- [ ] Add `--tag` flag for Docker image tagging.
- [ ] Document `deploy` as compatibility alias for `build --output docker`.

### Success Criteria

- `build --output binary` matches current behavior.
- `build --output docker` works identically to `deploy`.
- `build --output helm` validates chart inputs and packages.
- Build summary prints all outputs with paths/tags.

### Estimated Effort

2-3 weeks.

## Phase 7: Polish and Stabilization

**Goal**: finalize developer experience, documentation, and prepare for 1.0.

### Deliverables

- [ ] Audit all error messages for consistency, diagnostic codes, and suggestions.
- [ ] Add shell completion generation (`completions --shell`).
- [ ] Add man page generation (`man --output-dir`).
- [ ] Update `SKILL.md` to cover all new commands.
- [ ] Update `README.md` with manifest-first workflows.
- [ ] Write migration guide from config-centric to manifest-first.
- [ ] Performance audit: startup time, generation time, resolution time.
- [ ] Security audit: no secrets in generated files, proper redaction.
- [ ] CI pipeline template with all quality gates.

### Success Criteria

- All commands have consistent error messages with codes and suggestions.
- Shell completions work for bash, zsh, and fish.
- `SKILL.md` is accurate and complete.
- Startup time is under 100ms for non-network commands.

### Estimated Effort

2-3 weeks.

## Resolved Design Decisions

These questions from the v1 design are resolved:

| Question | Decision | Rationale |
|---|---|---|
| Manifest format: TOML only or also YAML? | **TOML only.** | `toml_edit` preserves comments on in-place edits. Single format reduces ambiguity. |
| `local:<name>` resolution: module name only or fallback? | **Module name first, then package name fallback.** | Documented in verbose output. Matches current behavior. |
| Production FIPS policy: advisory or hard-fail? | **Warning by default, hard-fail with `--strict`.** | Avoids blocking development; CI uses `--strict`. |
| `deploy` vs `build --output docker`? | **Both.** `deploy` is a compatibility alias. | New users guided to `build --output docker`. |
| Generated `AGENTS.md`: workspace-level only? | **Workspace-level only, initially.** | Module-level is a future enhancement. |
| `SKILL.md`: replace root or separate file? | **Same root `SKILL.md`.** | Generated from template, updated by `generate skill`. |
