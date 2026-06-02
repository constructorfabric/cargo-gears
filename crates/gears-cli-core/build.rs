#[cfg(feature = "dylint-rules")]
use anyhow::{Context, bail};
#[cfg(feature = "dylint-rules")]
use std::fs;
#[cfg(feature = "dylint-rules")]
use std::path::{Path, PathBuf};
#[cfg(feature = "dylint-rules")]
use std::process::Command;

#[cfg(feature = "dylint-rules")]
const LINTS_REPO_URL: &str = "git@github.com:constructorfabric/cyberware-rust.git";

#[cfg(feature = "dylint-rules")]
const LINTS_REPO_REVISION: &str = "cfc6b674fdbbaad89baa338424b6e35b56a8cf8f";

#[cfg(feature = "dylint-rules")]
mod ensure_toolchain_installed_shared {
    include!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/shared/ensure_toolchain_installed.rs"
    ));
}

#[cfg(feature = "dylint-rules")]
use ensure_toolchain_installed_shared::{
    ensure_toolchain_components_installed, ensure_toolchain_installed,
};

#[cfg(feature = "dylint-rules")]
fn build_dylint_rules() -> anyhow::Result<()> {
    use std::env;
    use std::fmt::Write as _;

    let out_dir = PathBuf::from(env::var("OUT_DIR")?);
    let lints_dir = ensure_lints_dir(&out_dir)?;
    let lint_build_dir = out_dir.join("lint_build");

    emit_rerun_markers(&lints_dir);

    // -- Toolchain detection ------------------------------------------------
    let channel = read_toolchain_channel(&lints_dir)?;

    ensure_toolchain_installed(&channel)?;
    let components = read_toolchain_components(&lints_dir)?;
    ensure_toolchain_components_installed(&channel, &components)?;

    // Get the host triple for the installed nightly toolchain.
    let rustc_vv = Command::new("rustup")
        .args(["run", &channel, "rustc", "-vV"])
        .output()
        .with_context(|| format!("failed to run `rustup run {channel} rustc -vV`"))?;

    if !rustc_vv.status.success() {
        bail!(
            "rustc -vV failed for toolchain `{channel}`: {}",
            String::from_utf8_lossy(&rustc_vv.stderr)
        );
    }

    let rustc_info = String::from_utf8(rustc_vv.stdout)?;
    let host = rustc_info
        .lines()
        .find(|l| l.starts_with("host:"))
        .context("no `host:` line in rustc -vV output")?
        .trim_start_matches("host:")
        .trim()
        .to_owned();

    // Full versioned toolchain name used in the dylib filename convention.
    let versioned_toolchain = format!("{channel}-{host}");

    // -- Build the lint workspace -------------------------------------------
    // Use `rustup run` so the toolchain is explicit, and strip every env var
    // that the outer stable `cargo build` injects — in particular `RUSTC`,
    // `CARGO`, `RUSTFLAGS`, and `RUSTUP_TOOLCHAIN` — so the inner build
    // cannot accidentally inherit a stable toolchain.
    if !lint_build_dir.exists() {
        let status = Command::new("rustup")
            .args([
                "run",
                &channel,
                "cargo",
                "build",
                "--release",
                "--workspace",
                "--manifest-path",
            ])
            .arg(lints_dir.join("Cargo.toml"))
            .arg("--target-dir")
            .arg(&lint_build_dir)
            .env_remove("RUSTUP_TOOLCHAIN")
            .env_remove("RUSTC")
            .env_remove("RUSTC_WRAPPER")
            .env_remove("RUSTC_WORKSPACE_WRAPPER")
            .env_remove("RUSTDOC")
            .env_remove("CARGO")
            .env_remove("RUSTFLAGS")
            .env_remove("CARGO_ENCODED_RUSTFLAGS")
            .status()
            .context("failed to spawn cargo build for lint workspace")?;

        if !status.success() {
            bail!("cargo build failed for lint workspace");
        }
    }

    // -- Copy dylibs with versioned names -----------------------------------
    let release_dir = lint_build_dir.join("release");
    let libs_dir = out_dir.join("dylint_libs");
    fs::create_dir_all(&libs_dir)?;

    let (dll_prefix, dll_suffix) = if cfg!(target_os = "macos") {
        ("lib", "dylib")
    } else if cfg!(target_os = "windows") {
        ("", "dll")
    } else {
        ("lib", "so")
    };

    for entry in fs::read_dir(&release_dir).context("could not read release dir")? {
        let entry = entry?;
        let path = entry.path();
        let filename = match path.file_name() {
            Some(f) => f.to_string_lossy().into_owned(),
            None => continue,
        };

        // Only consider shared library files that don't already have the versioned name.
        if !filename.starts_with(dll_prefix)
            || !filename.ends_with(dll_suffix)
            || filename.contains('@')
        {
            continue;
        }

        let stem = filename
            .strip_prefix(dll_prefix)
            .context("wrong library prefix")?
            .strip_suffix(&format!(".{dll_suffix}"))
            .context("wrong library suffix")?;

        let versioned = format!("{dll_prefix}{stem}@{versioned_toolchain}.{dll_suffix}");
        let dest = libs_dir.join(&versioned);
        fs::copy(&path, &dest)
            .with_context(|| format!("failed to copy {filename} -> {versioned}"))?;
        println!("cargo:warning=dylint lint installed: {versioned}");
    }

    // -- Generate embedded-libs source file --------------------------------
    // Build a `generated_libs.rs` that hard-codes every versioned dylib as
    // raw bytes via `include_bytes!`.  `crates/gears-cli-core/src/lint/mod.rs`
    // includes this file and writes the bytes to a temp directory at runtime, so the
    // binary is fully self-contained
    let mut src = String::from("/// Dylib files embedded at compile time.\n");
    src.push_str("pub const LIBS: &[(&str, &[u8])] = &[\n");

    for entry in fs::read_dir(&libs_dir).context("could not read libs_dir for embedding")? {
        let entry = entry?;
        let path = entry.path();
        let filename = match path.file_name() {
            Some(f) => f.to_string_lossy().into_owned(),
            None => continue,
        };
        // Only embed the versioned dylib files we just placed there.
        if !filename.contains('@') {
            continue;
        }
        // Use forward slashes so the literal is valid on all platforms.
        let abs_path = path.to_string_lossy().replace('\\', "/");
        writeln!(src, "    (\"{filename}\", include_bytes!(\"{abs_path}\")),")?;
    }

    src.push_str("];\n");

    let generated_path = out_dir.join("generated_libs.rs");
    fs::write(&generated_path, &src).context("could not write generated_libs.rs")?;

    Ok(())
}

#[cfg_attr(not(feature = "dylint-rules"), allow(clippy::unnecessary_wraps))]
fn main() -> anyhow::Result<()> {
    #[cfg(feature = "dylint-rules")]
    build_dylint_rules()?;
    Ok(())
}

#[cfg(feature = "dylint-rules")]
fn clone_lints_repo(repo_dir: &Path) -> anyhow::Result<()> {
    let status = Command::new("git")
        .args(["clone", "--no-checkout", LINTS_REPO_URL])
        .arg(repo_dir)
        .status()
        .context("failed to clone gears-core")?;

    if !status.success() {
        bail!("git clone failed for {LINTS_REPO_URL}");
    }

    let status = Command::new("git")
        .arg("-C")
        .arg(repo_dir)
        .args(["fetch", "--depth", "1", "origin", LINTS_REPO_REVISION])
        .status()
        .with_context(|| format!("failed to fetch pinned revision {LINTS_REPO_REVISION}"))?;

    if !status.success() {
        bail!("git fetch failed for revision {LINTS_REPO_REVISION}");
    }

    let status = Command::new("git")
        .arg("-C")
        .arg(repo_dir)
        .args(["checkout", "--detach", "FETCH_HEAD"])
        .status()
        .with_context(|| format!("failed to checkout pinned revision {LINTS_REPO_REVISION}"))?;

    if !status.success() {
        bail!("git checkout failed for revision {LINTS_REPO_REVISION}");
    }

    Ok(())
}

#[cfg(feature = "dylint-rules")]
fn ensure_lints_dir(out_dir: &Path) -> anyhow::Result<PathBuf> {
    let repo_dir = out_dir.join("gears-core");
    let lints_dir = repo_dir.join("tools").join("dylint_lints");

    if !repo_dir.exists() {
        clone_lints_repo(&repo_dir)?;
    }

    if repo_dir.join(".git").is_dir() && lints_dir.join("Cargo.toml").is_file() {
        return Ok(lints_dir);
    }

    if repo_dir.exists() {
        fs::remove_dir_all(&repo_dir).with_context(|| {
            format!(
                "failed to remove invalid cached repo {}",
                repo_dir.display()
            )
        })?;
    }

    clone_lints_repo(&repo_dir)?;

    if !repo_dir.join(".git").is_dir() || !lints_dir.join("Cargo.toml").is_file() {
        bail!(
            "dylint workspace manifest not found in cached repo: {}",
            lints_dir.display()
        );
    }

    Ok(lints_dir)
}

#[cfg(feature = "dylint-rules")]
fn read_toolchain_channel(lints_dir: &Path) -> anyhow::Result<String> {
    read_toolchain(lints_dir)?
        .get("toolchain")
        .and_then(toml::Value::as_table)
        .and_then(|toolchain| toolchain.get("channel"))
        .and_then(toml::Value::as_str)
        .map(str::to_owned)
        .context("no `toolchain.channel` field found in rust-toolchain.toml")
}

#[cfg(feature = "dylint-rules")]
fn read_toolchain_components(lints_dir: &Path) -> anyhow::Result<Vec<String>> {
    let mut components: Vec<String> = read_toolchain(lints_dir)?
        .get("toolchain")
        .and_then(toml::Value::as_table)
        .and_then(|toolchain| toolchain.get("components"))
        .and_then(toml::Value::as_array)
        .map(|components| {
            components
                .iter()
                .filter_map(toml::Value::as_str)
                .map(str::to_owned)
                .collect()
        })
        .unwrap_or_default();

    if !components.iter().any(|component| component == "rust-src") {
        components.push("rust-src".to_owned());
    }

    components.sort();
    components.dedup();
    Ok(components)
}

#[cfg(feature = "dylint-rules")]
fn read_toolchain(lints_dir: &Path) -> anyhow::Result<toml::Value> {
    let toolchain_file = lints_dir.join("rust-toolchain.toml");
    let toolchain_content = fs::read_to_string(&toolchain_file)
        .context("could not read rust-toolchain.toml from lint workspace")?;

    toml::from_str(&toolchain_content)
        .context("could not parse rust-toolchain.toml from lint workspace")
}

#[cfg(feature = "dylint-rules")]
fn emit_rerun_markers(lints_dir: &Path) {
    println!(
        "cargo:rerun-if-changed={}",
        lints_dir.join("Cargo.toml").display()
    );
    println!(
        "cargo:rerun-if-changed={}",
        lints_dir.join("rust-toolchain.toml").display()
    );
}
