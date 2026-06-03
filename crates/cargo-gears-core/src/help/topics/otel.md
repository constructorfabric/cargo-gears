Topic: OpenTelemetry (OTel)

OpenTelemetry support enables distributed tracing and metrics collection in
the generated server.

How it works:
  - The CLI passes -F otel to cargo build/run, enabling the otel Cargo feature
  - The modkit framework's otel feature activates tracing and metrics exporters
  - Runtime configuration is in the config YAML under the opentelemetry section

Activation:
  CLI flag:      --otel / --no-otel
  Manifest:      [apps.myapp.dev.run] otel = true
  Priority:      CLI flag > manifest policy > default (false)

Runtime config:
  opentelemetry:
    resource:
      service_name: my-service
    tracing:
      enabled: true
      sampler:
        parent_based_ratio:
          ratio: 0.1
    metrics:
      enabled: true

See also: cargo gears help schema config --section opentelemetry
