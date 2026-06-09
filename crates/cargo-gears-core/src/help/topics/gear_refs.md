Topic: Gear References

Gears are referenced in the manifest and config in two forms:

Local gears:
  Discovered from the workspace via `cargo metadata`. The CLI scans for
  packages whose manifest path is under the workspace root.

  Manifest syntax:
    { source = "local", name = "api-gateway" }

  The name must match a Cargo package name discoverable in the workspace.
  Optional overrides: version, package.

Remote gears:
  Downloaded from a registry (currently only crates.io). Not present in the
  workspace.

  Manifest syntax:
    { source = "remote", name = "credstore", package = "cf-credstore", version = "0.4" }

  Required fields: name, package, version.

Config gear metadata:
  When using config-driven builds (deploy), gears need metadata in the YAML:
    modules:
      api-gateway:
        metadata:
          package: cf-api-gateway
          version: "0.4.0"

  With manifest-driven builds (build/run), metadata is resolved automatically
  from the workspace or the remote gear reference.
