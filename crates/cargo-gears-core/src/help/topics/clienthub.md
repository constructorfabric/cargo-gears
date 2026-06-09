Topic: ClientHub and Plugins

ClientHub provides type-safe client resolution for inter-module
communication. It supports in-process, remote (gRPC), and scoped clients.

Basic usage:
  // Provider registers in init()
  let api: Arc<dyn MyModuleApi> = Arc::new(LocalClient::new(svc));
  ctx.client_hub().register::<dyn MyModuleApi>(api);

  // Consumer resolves
  let api = ctx.client_hub().get::<dyn MyModuleApi>()?;

In-process vs remote:
  In-process   Direct function call, nanosecond latency, shared process
  Remote       gRPC transport, millisecond latency, separate process
  Both implement the same SDK trait; consumers don't know which is used.

Plugin architecture:
  Main module    Exposes public API, registers plugin schema (GTS type)
  Plugin modules Register instances + scoped clients under GTS instance IDs
  Selection      Main module resolves plugin by vendor config + priority

  // Plugin registers scoped client
  let scope = ClientScope::gts_id(&instance_id);
  ctx.client_hub().register_scoped::<dyn PluginClient>(scope, impl);

  // Main module resolves selected plugin
  let plugin = ctx.client_hub().get_scoped::<dyn PluginClient>(&scope)?;

Plugin crate structure:
  modules/<name>/
    <name>-sdk/           API trait (public) + PluginAPI trait (plugins)
    <name>/               Main module (schema registration, plugin routing)
    plugins/
      <vendor>-plugin/    Plugin implementation

Two API traits in SDK:
  - <Module>Client       Public API consumed by other modules
  - <Module>PluginClient Implemented by plugins, called by main module only

Plugin isolation rule: regular modules CANNOT depend on plugin crates.
All plugin functionality is accessed through the main module's public API.

Plugin selection uses choose_plugin_instance() from toolkit::plugins.
Main module depends on types-registry, NOT on plugin crates.

Configuration:
  modules:
    my-module:
      config:
        vendor: "VendorA"
    vendor-a-plugin:
      config:
        vendor: "VendorA"
        priority: 10

See also:
  cargo gears help topic module-layout
  cargo gears help topic architecture
