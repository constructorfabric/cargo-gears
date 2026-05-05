# 09. Implementation Plan

## Table of Contents

1. [Principles](#principles)
2. [Phase 1: Manifest Model](#phase-1-manifest-model)
3. [Phase 2: Manifest-Aware Build and Run](#phase-2-manifest-aware-build-and-run)
4. [Phase 3: Listing and Help](#phase-3-listing-and-help)
5. [Phase 4: Template Generation](#phase-4-template-generation)
6. [Phase 5: Test Orchestration](#phase-5-test-orchestration)
7. [Phase 6: Build Outputs](#phase-6-build-outputs)
8. [Open Questions](#open-questions)

## Principles

- Keep existing commands working during migration.
- Add typed models before adding many flags.
- Prefer dry-run/render commands before commands that mutate files.
- Use JSON output as the stable machine contract.
- Keep runtime config backward compatible until a clear migration path exists.

## Phase 1: Manifest Model

Scope:

- Add typed manifest structs.
- Add `Cyberfabric.toml` discovery.
- Add `manifest validate`.
- Add `manifest render`.
- Add `manifest init --from-config`.

Success criteria:

- Existing configs can be converted into a manifest without changing runtime
  behavior.
- `manifest render` produces the same dependency model currently produced from
  config metadata.
- Invalid module refs fail with precise errors.

## Phase 2: Manifest-Aware Build and Run

Scope:

- Add `--manifest`, `--env`, and `--app` to `run` and `build`.
- Keep `--config` supported.
- Resolve module composition from manifest.
- Keep `CF_CLI_CONFIG` runtime behavior unchanged.

Success criteria:

- `run --env dev --app app1` works without `--config` when the manifest declares
  the config.
- `build --env dev --app app1` generates `.cyberfabric/dev-app1`.
- Existing `run -c config/quickstart.yml` and `build -c ...` keep working.

## Phase 3: Listing and Help

Scope:

- Add `list modules`, `list local-modules`, `list system-modules`, `list apps`,
  and `list configs`.
- Add `--format table|json|yaml`.
- Add `help schema`.
- Add `help docs` alias for `docs`.

Success criteria:

- Humans can inspect module/app/config state with table output.
- LLMs can consume deterministic JSON.
- Schema help is generated from or checked against parser types.

## Phase 4: Template Generation

Scope:

- Add `generate` command namespace.
- Keep `init` and `mod add` as aliases.
- Add manifest/config/build/agents/skill template commands.
- Add explicit Docker generation.

Success criteria:

- A new workspace can be generated with manifest and config separated.
- A module can be generated and optionally added to a manifest app.
- `AGENTS.md` and `SKILL.md` can be generated from maintained templates.

## Phase 5: Test Orchestration

Scope:

- Implement existing `test`.
- Add runner enum: `cargo-test`, `nextest`.
- Add test set enum.
- Add coverage mode.
- Read manifest test policy.

Success criteria:

- Default `test` runs useful workspace tests.
- `test --module <name>` resolves local module package.
- `test --env dev --app app1 --set e2e` validates/generates the app before
  running e2e tests.
- Coverage has a clear first implementation and clear unsupported cases.

## Phase 6: Build Outputs

Scope:

- Add `build --output binary|docker|helm|all`.
- Share Docker build logic with `deploy`.
- Add Helm chart validation/package support.
- Add build summary output.

Success criteria:

- Binary build matches current behavior.
- Docker build works through both `deploy` and `build --output docker`.
- Helm output validates chart inputs and packages when Helm is installed.

## Open Questions

- Should the manifest file be TOML only, or should YAML be accepted from the
  beginning?
- Should `local:<name>` resolve by module name only, or module name then package
  name fallback?
- Should production FIPS policy be advisory or hard-fail by default?
- Should `deploy` eventually become an alias for `build --output docker`, or
  remain a separate verb?
- Should generated `AGENTS.md` be workspace-specific only, or also support
  module-level files?
- Should `SKILL.md` replace the current root skill guide or be generated as a
  separate LLM reference file?

