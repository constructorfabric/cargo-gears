# CyberFabric CLI Design

This folder contains the authoritative design for the CyberFabric CLI, the canonical command-line interface for
building, validating, and deploying applications on the CyberFabric framework.

> **Supersedes:** The original v1 design documents in `v1/` are retained for historical reference. The top-level
> documents below represent the current design.

## Purpose

The CyberFabric CLI exists to maximize developer productivity, consistency, and correctness. It acts as a
**deterministic enforcement layer** so that applications scaffolded, built, or deployed through it always follow
CyberFabric standards, architectural patterns, and approved practices.

The CLI orchestrates existing Rust ecosystem tooling rather than replacing it. Developers focus on writing modules and
business logic; Modkit libraries provide the framework runtime; system modules provide generic functionality; and this
CLI provides the development lifecycle tooling that ties everything together.

## Core Goals

- **Enforce consistency** across generated code, configuration, project structure, and developer workflows
- **Reduce ambiguity** in how developers use the CyberFabric framework
- **Improve productivity** by automating scaffolding, validation, and framework-specific tasks
- **Ensure determinism** so outputs are predictable, repeatable, and aligned with organizational standards
- **Serve as canonical interface** for applying CyberFabric best practices in day-to-day development
- **Support automation** with structured output that LLMs, CI systems, and scripts can consume reliably

## Key Decision

The **manifest** (`Cyberfabric.toml`) becomes the source of truth for what the CLI generates and orchestrates. Runtime
configuration remains the source of truth for runtime settings.

| Question                                                                                                   | Source             |
|------------------------------------------------------------------------------------------------------------|--------------------|
| Which app, environment, modules, feature sets, test strategy, lint policy, runner mode, and build outputs? | **Manifest**       |
| What values should the generated server and modules read at runtime?                                       | **Runtime config** |

## Design Documents

| #  | Document                                                                         | Scope                                                               |
|----|----------------------------------------------------------------------------------|---------------------------------------------------------------------|
| 01 | [Principles and Philosophy](v1_proposal_a/01-principles-and-philosophy.md)       | Core design principles, standards, and tradeoff guidelines          |
| 02 | [Architecture](v1_proposal_a/02-architecture.md)                                 | Internal crate structure, trait boundaries, extension model         |
| 03 | [Command Surface](v1_proposal_a/03-command-surface.md)                           | Complete command tree, naming conventions, shared argument patterns |
| 04 | [Manifest and Configuration](v1_proposal_a/04-manifest-and-configuration.md)     | Manifest schema, runtime config, validation, migration              |
| 05 | [Scaffolding and Templates](v1_proposal_a/05-scaffolding-and-templates.md)       | Generation commands, template registry, workspace profiles          |
| 06 | [Inspection and Discovery](v1_proposal_a/06-inspection-and-discovery.md)         | List, docs, help, schema introspection                              |
| 07 | [Quality Gates](v1_proposal_a/07-quality-gates.md)                               | Lint, test, and coverage orchestration                              |
| 08 | [Build, Run, and Deploy](v1_proposal_a/08-build-run-deploy.md)                   | Dev loop, build pipeline, Docker, Helm                              |
| 09 | [Developer Experience](v1_proposal_a/09-developer-experience.md)                 | Output formatting, error handling, exit codes, UX conventions       |
| 10 | [Security](v1_proposal_a/10-security.md)                                         | Secrets, credentials, secure defaults                               |
| 11 | [CI and Automation](v1_proposal_a/11-ci-and-automation.md)                       | Non-interactive mode, CI patterns, LLM integration                  |
| 12 | [Versioning and Compatibility](v1_proposal_a/12-versioning-and-compatibility.md) | CLI versioning, manifest versioning, migration                      |
| 13 | [Implementation Plan](v1_proposal_a/13-implementation-plan.md)                   | Phased rollout with success criteria                                |
| -- | [Glossary](v1_proposal_a/glossary.md)                                            | Precise definitions for all domain terms                            |

## Proposed Command Shape

```text
cargo cyberfabric
├── init                          # Alias for generate workspace
├── generate
│   ├── workspace
│   ├── module
│   ├── config
│   ├── manifest
│   ├── build
│   ├── ci
│   ├── agents
│   └── skill
├── manifest
│   ├── add
│   ├── edit
│   ├── rm
│   ├── validate
│   ├── render
│   └── migrate
├── list
│   ├── modules
│   ├── system-modules
│   ├── local-modules
│   ├── configs
│   └── apps
├── help
│   ├── schema
│   ├── docs
│   └── topic
├── config
│   ├── mod
│   │   ├── list
│   │   ├── add
│   │   ├── rm
│   │   └── db { add | edit | rm }
│   └── db { add | edit | rm }
├── lint
├── test
├── run
├── build
├── deploy
├── completions
└── man
```

The tree preserves existing commands (`init`, `mod add`, `config`, `docs`, `lint`, `run`, `build`, `deploy`) while
adding the manifest-first model, normalized generation namespace, and structured inspection commands.

## Design Principles Summary

These are expanded in [01-principles-and-philosophy.md](v1_proposal_a/01-principles-and-philosophy.md):

1. **Convention over configuration** -- sensible defaults, explicit overrides
2. **Manifest-first orchestration** -- the manifest drives all generation and tooling
3. **Deterministic outputs** -- same inputs always produce the same artifacts
4. **Fail fast, fail clearly** -- validate early, report structured errors
5. **Orchestrate, don't replace** -- wrap existing Rust tools, never reinvent them
6. **Machine-readable by default** -- `--format json` everywhere, stable exit codes
7. **Secure by default** -- no secrets in generated files, env-var expansion for credentials
8. **Backward compatible** -- existing workflows keep working across upgrades
