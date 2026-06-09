Topic: Architecture Overview

Constructor Fabric Gears is a secure, modular XaaS framework. It provides
reusable toolkit, runtime foundations, and service-level gears that teams
compose into services, applications, and platforms.

Three-tier gear hierarchy:
  - Toolkit (libs/)       Low-level substrate: API middleware, DB access,
                          error definitions, security primitives, macros.
  - System gears          Control-plane gears: API gateway, authn/authz
    (gears/system/)       resolvers, tenant resolver, types registry, etc.
  - Service gears         Business and domain gears: serverless runtime,
    (gears/)              GenAI subsystems, chat engine, file parser, etc.

Architectural principles:
  - Secure-by-default     Every handler enforces auth, tenant isolation,
                          and scoped DB access. No unscoped shortcut exists.
  - DDD-light isolation   Domain logic is free from transport/infra details.
                          Enforced by Dylint lints and #[domain_model] macro.
  - Declarative discovery Gears declare capabilities and deps via
                          #[toolkit::gear(...)]; the runtime discovers them
                          via inventory and wires the system automatically.
  - SDK-first contracts   Public API lives in <gear>-sdk/ crates. Internals
                          never leak to consumers.
  - Compile-time checks   Custom Dylint lints enforce architecture boundaries,
                          DTO placement, versioned REST paths, and more.

Deployment models (same gear code for all):
  - Single-node           All gears in one process (dev, edge, testing)
  - Multi-node            Gears over REST/gRPC without orchestration
  - Kubernetes            Containerized services with full orchestration

Key invariant: gears communicate via typed SDK traits through ClientHub.
In-process calls use direct adapters; out-of-process calls use gRPC clients
implementing the same trait.

See also:
  cargo gears help topic gear-layout
  cargo gears help topic security
  cargo gears help topic gears-catalog
