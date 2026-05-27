Topic: Module References

Modules are referenced in the manifest and config in two forms:

Local modules:
  Discovered from the workspace via `cargo metadata`. The CLI scans for
  packages whose manifest path is under the workspace root.

  Manifest syntax:
    { source = "local", name = "rest-gateway" }

  The name must match a Cargo package name discoverable in the workspace.
  Optional overrides: version, package.

Remote modules:
  Downloaded from a registry (currently only crates.io). Not present in the
  workspace.

  Manifest syntax:
    { source = "remote", name = "credstore", package = "cf-credstore", version = "0.4" }

  Required fields: name, package, version.

Config module metadata:
  When using config-driven builds (deploy), modules need metadata in the YAML:
    modules:
      rest-gateway:
        metadata:
          package: cf-rest-gateway
          version: "0.4.0"

  With manifest-driven builds (build/run), metadata is resolved automatically
  from the workspace or the remote module reference.
