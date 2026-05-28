Module Schema

A CyberFabric module is a Rust crate inside the workspace's modules/ directory.
Module metadata is discovered from Cargo.toml and workspace metadata.

Module Cargo.toml structure:
  [package]
  name         string    Crate name (used as the module identifier)
  version      string    Crate version

  [dependencies]
  ...                    Dependencies are promoted to workspace.dependencies
                         during `generate module`

  [package.metadata.cyberware]
  deps           array of string        Module dependencies (other module names)
  capabilities   array of string        Declared capabilities (e.g. "grpc", "http")

Discovery:
  The CLI discovers modules by running `cargo metadata` on the workspace and
  finding packages whose manifest path is under the workspace root.

  Module names in config and manifest reference the Cargo package name.

Example Cargo.toml:
  [package]
  name = "api-gateway"
  version = "0.1.0"
  edition = "2024"

  [dependencies]
  cf-modkit = { workspace = true, features = ["http"] }

  [package.metadata.cyberware]
  deps = ["authn-resolver"]
  capabilities = ["http"]
