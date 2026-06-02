use super::{CONFIG_PATH_ENV_VAR, TestPlan, TestRun, nextest};
use crate::common::cargo_cmd;
use crate::manifest::TestRunner;
use anyhow::{Context, bail};
use std::process::Command;

pub(super) fn run(plan: &TestPlan, runner: TestRunner) -> anyhow::Result<()> {
    match runner {
        TestRunner::Cargo => run_cargo_llvm_cov(plan),
        TestRunner::Nextest => run_nextest_llvm_cov(plan),
    }
}

fn run_cargo_llvm_cov(plan: &TestPlan) -> anyhow::Result<()> {
    for run in &plan.runs {
        let mut cmd = llvm_cov_command(plan, run)?;
        let status = cmd.status().context("failed to run llvm-cov test runner")?;
        if !status.success() {
            bail!("llvm-cov test runner exited with {status}");
        }
    }

    Ok(())
}

fn run_nextest_llvm_cov(plan: &TestPlan) -> anyhow::Result<()> {
    let coverage_env = llvm_cov_env(plan)?;
    for run in &plan.runs {
        run_llvm_cov_clean(plan, &coverage_env)?;
        nextest::run_nextest(plan, run, &coverage_env)?;
        run_llvm_cov_report(plan, run, &coverage_env)?;
    }

    Ok(())
}

fn llvm_cov_command(plan: &TestPlan, run: &TestRun) -> anyhow::Result<Command> {
    let mut cmd = cargo_cmd()?;
    cmd.arg("llvm-cov");

    let mut args = Vec::new();
    run.append_cargo_args(&mut args);
    cmd.args(args);
    cmd.current_dir(&plan.workspace_root);
    cmd.env(CONFIG_PATH_ENV_VAR, &plan.config_path);

    Ok(cmd)
}

fn llvm_cov_env(plan: &TestPlan) -> anyhow::Result<Vec<(String, String)>> {
    let output = cargo_cmd()?
        .args(["llvm-cov", "show-env", "--sh"])
        .current_dir(&plan.workspace_root)
        .output()
        .context("failed to run llvm-cov show-env")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        bail!("llvm-cov show-env exited with {}: {stderr}", output.status);
    }

    let stdout = String::from_utf8(output.stdout).context("llvm-cov show-env was not UTF-8")?;
    parse_llvm_cov_env(&stdout)
}

fn run_llvm_cov_clean(plan: &TestPlan, coverage_env: &[(String, String)]) -> anyhow::Result<()> {
    let status = cargo_cmd()?
        .args(["llvm-cov", "clean", "--workspace"])
        .current_dir(&plan.workspace_root)
        .envs(coverage_env.iter().map(|(key, value)| (key, value)))
        .status()
        .context("failed to run llvm-cov clean")?;

    if !status.success() {
        bail!("llvm-cov clean exited with {status}");
    }

    Ok(())
}

fn run_llvm_cov_report(
    plan: &TestPlan,
    run: &TestRun,
    coverage_env: &[(String, String)],
) -> anyhow::Result<()> {
    let mut args = vec!["llvm-cov".to_owned(), "report".to_owned()];
    run.append_cargo_args(&mut args);

    let status = cargo_cmd()?
        .args(args)
        .current_dir(&plan.workspace_root)
        .envs(coverage_env.iter().map(|(key, value)| (key, value)))
        .status()
        .context("failed to run llvm-cov report")?;

    if !status.success() {
        bail!("llvm-cov report exited with {status}");
    }

    Ok(())
}

fn parse_llvm_cov_env(output: &str) -> anyhow::Result<Vec<(String, String)>> {
    output
        .lines()
        .filter(|line| !line.trim().is_empty())
        .map(parse_llvm_cov_env_line)
        .collect()
}

fn parse_llvm_cov_env_line(line: &str) -> anyhow::Result<(String, String)> {
    let parts = shell_words::split(line)
        .with_context(|| format!("failed to parse llvm-cov show-env line `{line}`"))?;
    let assignment = match parts.as_slice() {
        [assignment] => assignment.as_str(),
        [export, assignment] if export == "export" => assignment.as_str(),
        _ => bail!("unexpected llvm-cov show-env line `{line}`"),
    };

    let (key, value) = assignment
        .split_once('=')
        .with_context(|| format!("llvm-cov show-env line `{line}` is not KEY=VALUE"))?;

    if key.is_empty() {
        bail!("llvm-cov show-env line `{line}` has an empty key");
    }

    Ok((key.to_owned(), value.to_owned()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test::FeatureSelection;
    use std::path::PathBuf;

    #[test]
    fn llvm_cov_uses_package_and_feature_args() {
        let plan = TestPlan {
            workspace_root: PathBuf::from("/workspace"),
            config_path: PathBuf::from("/workspace/config/app-dev.yml"),
            runs: vec![],
        };
        let run = TestRun {
            package: Some("cf-module".to_owned()),
            features: FeatureSelection::Features(vec!["sqlite".to_owned(), "otel".to_owned()]),
        };

        let command =
            llvm_cov_command(&plan, &run).expect("CARGO env var should exist under tests");
        let args = command
            .get_args()
            .map(|arg| arg.to_string_lossy().into_owned())
            .collect::<Vec<_>>();

        assert_eq!(
            args,
            vec![
                "llvm-cov",
                "-p",
                "cf-module",
                "--no-default-features",
                "--features",
                "sqlite,otel"
            ]
        );
        assert_eq!(
            command.get_current_dir(),
            Some(plan.workspace_root.as_path())
        );
        assert_eq!(
            command
                .get_envs()
                .find(|(key, _)| key == &CONFIG_PATH_ENV_VAR),
            Some((
                CONFIG_PATH_ENV_VAR.as_ref(),
                Some(plan.config_path.as_os_str())
            ))
        );
    }

    #[test]
    fn parses_exported_llvm_cov_env() {
        let env = parse_llvm_cov_env(
            "export LLVM_PROFILE_FILE='target/llvm cov/%p-%m.profraw'\n\
             export RUSTFLAGS=-Cinstrument-coverage\n",
        )
        .expect("show-env output should parse");

        assert_eq!(
            env,
            vec![
                (
                    "LLVM_PROFILE_FILE".to_owned(),
                    "target/llvm cov/%p-%m.profraw".to_owned()
                ),
                ("RUSTFLAGS".to_owned(), "-Cinstrument-coverage".to_owned()),
            ]
        );
    }
}
