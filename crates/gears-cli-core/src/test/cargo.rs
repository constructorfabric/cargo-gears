use super::{CONFIG_PATH_ENV_VAR, TestPlan, TestRun};
use crate::common::cargo_cmd;
use anyhow::{Context, bail};
use std::process::Command;

pub(super) fn run(plan: &TestPlan) -> anyhow::Result<()> {
    for run in &plan.runs {
        let mut cmd = cargo_command(plan, run)?;
        let status = cmd.status().context("failed to run cargo test runner")?;
        if !status.success() {
            bail!("cargo test runner exited with {status}");
        }
    }

    Ok(())
}

fn cargo_command(plan: &TestPlan, run: &TestRun) -> anyhow::Result<Command> {
    let mut cmd = cargo_cmd()?;
    cmd.arg("test");

    let mut args = Vec::new();
    run.append_cargo_args(&mut args);
    cmd.args(args);
    cmd.current_dir(&plan.workspace_root);
    cmd.env(CONFIG_PATH_ENV_VAR, &plan.config_path);

    Ok(cmd)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test::FeatureSelection;
    use std::path::PathBuf;

    #[test]
    fn cargo_test_uses_workspace_and_config_env() {
        let plan = TestPlan {
            workspace_root: PathBuf::from("/workspace"),
            config_path: PathBuf::from("/workspace/config/app-dev.yml"),
            runs: vec![],
        };
        let run = TestRun {
            package: None,
            features: FeatureSelection::Default,
        };

        let command = cargo_command(&plan, &run).expect("CARGO env var should exist under tests");
        let args = command
            .get_args()
            .map(|arg| arg.to_string_lossy().into_owned())
            .collect::<Vec<_>>();

        assert_eq!(args, vec!["test", "--workspace"]);
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
}
