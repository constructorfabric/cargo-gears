# CyberFabric CLI Design

This folder contains the authoritative design for the CyberFabric CLI, the canonical command-line interface for
building, validating, and deploying applications on the CyberFabric framework.

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

| #  | Document                                                                | Scope                                                               |
|----|-------------------------------------------------------------------------|---------------------------------------------------------------------|
| 01 | [Principles and Philosophy](v1/01-principles-and-philosophy.md)         | Core design principles, standards, and tradeoff guidelines          |
| 02 | [Architecture](v1/02-architecture.md)                                   | Internal crate structure, trait boundaries, extension model         |
| 03 | [Command Surface](v1/03-command-surface.md)                             | Complete command tree, naming conventions, shared argument patterns |
| 04 | [Manifest and Configuration](v1/04-manifest-and-configuration.md)       | Manifest schema, runtime config, validation, migration              |
| 05 | [Scaffolding and Templates](v1/05-scaffolding-and-templates.md)         | Generation commands, template registry, workspace profiles          |
| 06 | [Inspection and Discovery](v1/06-list-and-inspection.md)                | List, docs, help, schema introspection                              |
| 07 | [Documentation and LLM helpers](v1/07-documentation-and-llm-helpers.md) | Documentation and utilities to support LLM loops and dev flow       |
| 08 | [Lint and Test](v1/08-lint-and-test.md)                                 | Lint, test, and coverage orchestration                              |
| 09 | [Build, Run, and Deploy](v1/09-build-and-run.md)                        | Dev loop, build pipeline, Docker, Helm                              |
| 10 | [CI and Automation](v1/10-ci-and-automation.md)                         | Non-interactive mode, CI patterns, LLM integration                  |

## Proposed Command Shape

You can find the surface in [03-command-surface.md](v1/03-command-surface.md).

## Design Principles Summary

These are expanded in [01-principles-and-philosophy.md](v1/01-principles-and-philosophy.md):

1. **Convention over configuration** -- sensible defaults, explicit overrides
2. **Manifest orchestration** -- the manifest defines the policies and the runtime config defines the runtime values
3. **Deterministic outputs** -- same inputs always produce the same artifacts
4. **Fail fast, fail clearly** -- validate early, report structured errors
5. **Orchestrate, don't replace** -- wrap existing Rust tools, never reinvent them
