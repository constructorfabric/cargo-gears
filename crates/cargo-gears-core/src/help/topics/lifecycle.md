Topic: Module Lifecycle

The platform defines an explicit lifecycle with ordered phases and
dependency-aware teardown. Modules integrate through capabilities.

Lifecycle phases (executed by HostRuntime):
  pre_init -> DB migrations -> init -> post_init
    -> REST wiring -> gRPC wiring -> start -> ... -> stop

  - System gears run first where required
  - post_init is a barrier: begins only after all init hooks complete
  - Shutdown runs in reverse dependency order with a graceful deadline

Module lifecycle declaration:
  #[toolkit::module(
      name = "my-module",
      capabilities = [db, rest, stateful],
      lifecycle(entry = "serve", stop_timeout = "30s", await_ready)
  )]
  pub struct MyModule { ... }

Lifecycle entry method:
  async fn serve(
      self: Arc<Self>,
      cancel: CancellationToken,
      ready: ReadySignal,
  ) -> anyhow::Result<()> {
      // Setup resources
      let listener = TcpListener::bind("0.0.0.0:8080").await?;
      ready.notify();  // signal that module is ready

      // Run until cancelled
      axum::serve(listener, app)
          .with_graceful_shutdown(cancel.cancelled())
          .await?;
      Ok(())
  }

CancellationToken patterns:
  // Root token propagates to all modules automatically
  // Create child tokens for background tasks
  let child = cancel.child_token();
  tokio::spawn(async move {
      loop {
          tokio::select! {
              _ = child.cancelled() => break,
              _ = interval.tick() => { do_work().await; }
          }
      }
  });

WithLifecycle states:
  Stopped -> start() -> Starting -> (ready.notify()) -> Running
  Running -> stop()/cancel -> Stopped

Key rules:
  - Use CancellationToken for coordinated shutdown
  - Pass child tokens to all background tasks
  - Call ready.notify() after setup when using await_ready
  - Use tokio::select! for cooperative shutdown in loops
  - Graceful shutdown should have a timeout fallback

See also:
  cargo gears help topic architecture
  cargo gears help topic module-layout
