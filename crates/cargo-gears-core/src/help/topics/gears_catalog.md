Topic: Gears Catalog (Gear Categories)

Gears are organized into categories. Each gear encapsulates business logic
and exposes versioned contracts via SDK traits, REST, or gRPC.

Gear categories:
  API Ingress           Single public entry point for external traffic.
                        API gateway handles routing, auth, rate limiting,
                        versioned REST surface, and OpenAPI publication.

  Business Logic        User-facing SaaS capabilities built on ToolKit.
                        Compose GenAI, Serverless, and Core gears into
                        domain-specific workflows.

  Gen AI                Foundational generative AI: chat engine, LLM gateway,
                        model management, agents, memory, search, crawling,
                        scheduling, MCP integration.

  Serverless            Functions, workflows, runtimes, durable state,
                        settings, and cluster coordination gears.

  Core Functionality    Shared platform capabilities: audit, usage collection,
                        jobs, registries, file handling, quotas,
                        notifications, analytics, approvals.

  Core Platform         Interfaces and adapters for platform services:
  Integration           tenancy, access policies, licensing, credentials,
                        outbound egress control.

  Core Platform         External services implementing platform functionality.
  Services              Can be replaced with vendor-specific implementations
                        via the plugin system.

Dependency rules:
  - All external HTTP traffic goes through api-gateway
  - Business gears MAY depend on GenAI, Serverless, Core gears
  - GenAI gears MAY depend on Serverless and Core gears
  - Only integration/adapter gears talk to external components
  - No cross-category sideways deps except through SDK contracts
  - No circular dependencies allowed
  - In-process calls must propagate SecurityContext

Extension points:
  - Global Type System (GTS) for schema-validated type extensibility
  - Plugin architecture for swappable implementations
  - API hooks and serverless callbacks for domain-specific logic

Repository structure:
  libs/              ToolKit libraries (substrate)
  gears/system/      System gears (control plane)
  gears/             Service gears (business logic)
  apps/              Executable applications composing gears

Use `cargo gears ls modules` to list available system and local gears.

See also:
  cargo gears help topic architecture
  cargo gears help topic clienthub
