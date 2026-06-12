Topic: ClientHub and Plugins

ClientHub provides type-safe client resolution for inter-gear
communication. It supports in-process, remote (gRPC), and scoped clients.

Basic usage:
  // Provider registers in init()
  let api: Arc<dyn MyGearApi> = Arc::new(LocalClient::new(svc));
  ctx.client_hub().register::<dyn MyGearApi>(api);

  // Consumer resolves
  let api = ctx.client_hub().get::<dyn MyGearApi>()?;

In-process vs remote:
  In-process   Direct function call, nanosecond latency, shared process
  Remote       gRPC transport, millisecond latency, separate process
  Both implement the same SDK trait; consumers don't know which is used.

Plugin architecture:
  Main gear      Exposes public API, registers plugin schema (GTS type)
  Plugin gears   Register instances + scoped clients under GTS instance IDs
  Selection      Main gear resolves plugin by vendor config + priority

  // Plugin registers scoped client
  let scope = ClientScope::gts_id(&instance_id);
  ctx.client_hub().register_scoped::<dyn PluginClient>(scope, impl);

  // Main gear resolves selected plugin
  let plugin = ctx.client_hub().get_scoped::<dyn PluginClient>(&scope)?;

Plugin crate structure:
  gears/<name>/
    <name>-sdk/           API trait (public) + PluginAPI trait (plugins)
    <name>/               Main gear (schema registration, plugin routing)
    plugins/
      <vendor>-plugin/    Plugin implementation

Two API traits in SDK:
  - <Gear>Client       Public API consumed by other gears
  - <Gear>PluginClient Implemented by plugins, called by main gear only

Plugin isolation rule: regular gears CANNOT depend on plugin crates.
All plugin functionality is accessed through the main gear's public API.

Plugin selection uses choose_plugin_instance() from toolkit::plugins.
Main gear depends on types-registry, NOT on plugin crates.

Configuration:
  gears:
    my-gear:
      config:
        vendor: "VendorA"
    vendor-a-plugin:
      config:
        vendor: "VendorA"
        priority: 10

See also:
  cargo gears help topic gear-layout
  cargo gears help topic architecture
