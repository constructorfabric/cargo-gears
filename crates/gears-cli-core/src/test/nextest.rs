use super::{CONFIG_PATH_ENV_VAR, TestPlan, TestRun};
use anyhow::{Context, bail};
use camino::{Utf8Path, Utf8PathBuf};
use guppy::graph::PackageGraph;
use nextest_filtering::ParseContext;
use nextest_runner::{
    RustcCli,
    cargo_config::{CargoConfigs, EnvironmentMap},
    config::core::{ConfigExperimental, NextestConfig, get_num_cpus},
    double_spawn::DoubleSpawnInfo,
    input::InputHandlerKind,
    list::{BinaryList, RustTestArtifact, TestExecuteContext, TestList},
    platform::{BuildPlatforms, HostPlatform, PlatformLibdir},
    reporter::{
        ReporterBuilder, ReporterOutput, ShowTerminalProgress, events::FinalRunStats,
        structured::StructuredReporter,
    },
    reuse_build::PathMapper,
    run_mode::NextestRunMode,
    runner::{TestRunnerBuilder, configure_handle_inheritance},
    signal::SignalHandlerKind,
    target_runner::TargetRunner,
    test_filter::{FilterBound, RunIgnored, TestFilter},
    test_output::CaptureStrategy,
};
use std::collections::BTreeSet;
use std::io::{Cursor, Write};
use std::path::Path;
use std::process::Stdio;
use std::sync::Arc;

pub(super) fn run(plan: &TestPlan) -> anyhow::Result<()> {
    for run in &plan.runs {
        run_nextest(plan, run)?;
    }

    Ok(())
}

fn run_nextest(plan: &TestPlan, run: &TestRun) -> anyhow::Result<()> {
    let workspace_root = utf8_path(&plan.workspace_root, "workspace root")?;
    let graph_json = cargo_metadata_json(plan, run)?;
    let graph = PackageGraph::from_json(&graph_json).context("failed to parse cargo metadata")?;
    let build_platforms = detect_build_platforms()?;
    let binary_list = Arc::new(build_binary_list(plan, run, &graph, build_platforms)?);
    let cargo_config = cargo_config_with_gears_config(&workspace_root, &plan.config_path)?;
    let cargo_env = EnvironmentMap::new(&cargo_config.configs);

    let pcx = ParseContext::new(&graph);
    let config = NextestConfig::from_sources(
        workspace_root.clone(),
        &pcx,
        None,
        Vec::new().iter(),
        &BTreeSet::<ConfigExperimental>::new(),
    )
    .context("failed to load nextest config")?;
    let profile = config
        .profile(NextestConfig::DEFAULT_PROFILE)
        .context("failed to load nextest default profile")?
        .apply_build_platforms(&binary_list.rust_build_meta.build_platforms);

    let double_spawn = DoubleSpawnInfo::disabled();
    let target_runner = TargetRunner::empty();
    let ctx = TestExecuteContext {
        profile_name: profile.name(),
        double_spawn: &double_spawn,
        target_runner: &target_runner,
    };

    let path_mapper = PathMapper::noop();
    let rust_build_meta = binary_list.rust_build_meta.map_paths(&path_mapper);
    let test_artifacts = RustTestArtifact::from_binary_list(
        &graph,
        Arc::clone(&binary_list),
        &rust_build_meta,
        &path_mapper,
        None,
    )
    .context("failed to create nextest test artifacts")?;
    let test_filter = TestFilter::default_set(NextestRunMode::Test, RunIgnored::Default);
    let test_list = TestList::new(
        &ctx,
        test_artifacts,
        rust_build_meta,
        &test_filter,
        None,
        workspace_root,
        cargo_env,
        &profile,
        FilterBound::DefaultSet,
        get_num_cpus(),
    )
    .context("failed to list tests with nextest")?;

    let mut runner_builder = TestRunnerBuilder::default();
    runner_builder.set_capture_strategy(CaptureStrategy::Split);
    let runner = runner_builder
        .build(
            &test_list,
            &profile,
            nextest_cli_args(run),
            SignalHandlerKind::Standard,
            InputHandlerKind::Standard,
            double_spawn,
            target_runner,
        )
        .context("failed to build nextest runner")?;

    let mut reporter_builder = ReporterBuilder::default();
    reporter_builder.set_colorize(false);
    let mut reporter = reporter_builder.build(
        &test_list,
        &profile,
        ShowTerminalProgress::No,
        ReporterOutput::Terminal,
        StructuredReporter::new(),
    );

    configure_handle_inheritance(false)
        .context("failed to configure nextest handle inheritance")?;
    let run_stats = runner
        .try_execute(|event| reporter.report_event(event))
        .context("nextest failed to execute tests")?;
    reporter.finish();

    match run_stats.summarize_final() {
        FinalRunStats::Success => Ok(()),
        FinalRunStats::NoTestsRun => bail!("nextest found no tests to run"),
        FinalRunStats::Cancelled { kind, .. } => bail!("nextest run cancelled: {kind:?}"),
        FinalRunStats::Failed { kind } => bail!("nextest run failed: {kind:?}"),
    }
}

fn cargo_metadata_json(plan: &TestPlan, run: &TestRun) -> anyhow::Result<String> {
    let mut args = vec!["metadata".to_owned(), "--format-version=1".to_owned()];
    run.append_cargo_metadata_args(&mut args);

    let output = crate::common::cargo_cmd()?
        .args(args)
        .current_dir(&plan.workspace_root)
        .output()
        .context("failed to run cargo metadata for nextest")?;

    if !output.status.success() {
        bail!("cargo metadata for nextest exited with {}", output.status);
    }

    String::from_utf8(output.stdout).context("cargo metadata output was not UTF-8")
}

fn build_binary_list(
    plan: &TestPlan,
    run: &TestRun,
    graph: &PackageGraph,
    build_platforms: BuildPlatforms,
) -> anyhow::Result<BinaryList> {
    let mut args = vec![
        "test".to_owned(),
        "--no-run".to_owned(),
        "--message-format".to_owned(),
        "json-render-diagnostics".to_owned(),
    ];
    run.append_cargo_args(&mut args);

    let output = crate::common::cargo_cmd()?
        .args(args)
        .current_dir(&plan.workspace_root)
        .env(CONFIG_PATH_ENV_VAR, &plan.config_path)
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit())
        .output()
        .context("failed to build test binaries for nextest")?;

    if !output.status.success() {
        bail!(
            "building test binaries for nextest exited with {}",
            output.status
        );
    }

    BinaryList::from_messages(Cursor::new(output.stdout), graph, build_platforms)
        .context("failed to parse nextest binary list from Cargo messages")
}

fn detect_build_platforms() -> anyhow::Result<BuildPlatforms> {
    let host = HostPlatform::detect(PlatformLibdir::from_rustc_stdout(
        RustcCli::print_host_libdir().read(),
    ))?;
    Ok(BuildPlatforms { host, target: None })
}

struct CargoConfigGuard {
    configs: CargoConfigs,
    _file: tempfile::NamedTempFile,
}

fn cargo_config_with_gears_config(
    workspace_root: &Utf8Path,
    config_path: &Path,
) -> anyhow::Result<CargoConfigGuard> {
    let mut file = tempfile::NamedTempFile::new()
        .context("failed to create temporary Cargo config for nextest")?;
    let encoded = toml_edit::Value::from(config_path.to_string_lossy().to_string()).to_string();
    write!(
        file,
        "[env.{CONFIG_PATH_ENV_VAR}]\nvalue = {encoded}\nforce = true\n"
    )
    .context("failed to write temporary Cargo config for nextest")?;

    let config_path = utf8_path(file.path(), "temporary Cargo config")?;
    let configs = CargoConfigs::new_with_isolation(
        [config_path.as_str()],
        workspace_root,
        Utf8Path::new("/"),
        Vec::new(),
    )
    .context("failed to discover Cargo config for nextest")?;
    Ok(CargoConfigGuard {
        configs,
        _file: file,
    })
}

fn utf8_path(path: &Path, label: &str) -> anyhow::Result<Utf8PathBuf> {
    Utf8PathBuf::from_path_buf(path.to_path_buf())
        .map_err(|path| anyhow::anyhow!("{label} is not valid UTF-8: {}", path.display()))
}

fn nextest_cli_args(run: &TestRun) -> Vec<String> {
    let mut args = vec!["cargo".to_owned(), "nextest".to_owned(), "run".to_owned()];
    run.append_cargo_args(&mut args);
    args
}
