//! `cargo xtask` verification, hygiene, and CI runner for `gs` (Godseat).
//!
//! Enforces estate best practices:
//! - Host throttling (`-j 2`)
//! - Wave-0 purity of `gs-domain` (`graph-audit` + `--break-check`)
//! - Repository hygiene (`rust-toolchain.toml`, `rustfmt.toml`, `clippy.toml`, `deny.toml`, `.cargo/config.toml`)
//! - Full verification pipeline (`xwork`, `check`, `fmt`, `clippy`, `test`, `deny`, `build`, `smoke`)

use std::collections::BTreeSet;
use std::env;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode};

/// Forbidden runtime/network/filesystem crates in Wave-0 `gs-domain`.
const FORBIDDEN_DOMAIN_DEPS: &[&str] = &[
    "tokio",
    "reqwest",
    "hyper",
    "axum",
    "sqlx",
    "rusqlite",
    "std::fs",
    "std::net",
    "std::process",
];

fn main() -> ExitCode {
    let args: Vec<String> = env::args().skip(1).collect();
    let root = match workspace_root() {
        Ok(r) => r,
        Err(e) => {
            eprintln!("xtask error: {e}");
            return ExitCode::FAILURE;
        }
    };
    let jobs = env::var("JOBS")
        .ok()
        .filter(|s| !s.trim().is_empty())
        .unwrap_or_else(|| "2".to_string());

    let cmd = args.first().map(String::as_str).unwrap_or("help");
    let break_check = args.iter().any(|a| a == "--break-check");

    let result = match cmd {
        "xwork" => run_xwork(&root, &jobs),
        "check" => run_check(&root, &jobs, break_check),
        "hygiene" => run_hygiene(&root),
        "graph-audit" => run_graph_audit(&root, break_check),
        "fmt" => run_cargo(&root, &["fmt", "--all"]),
        "fmt-check" => run_cargo(&root, &["fmt", "--all", "--", "--check"]),
        "clippy" => run_cargo(
            &root,
            &[
                "clippy",
                "--workspace",
                "--all-targets",
                "--locked",
                "-j",
                &jobs,
                "--",
                "-D",
                "warnings",
            ],
        ),
        "test" => run_cargo(&root, &["test", "--workspace", "--locked", "-j", &jobs]),
        "doc-test" => run_cargo(
            &root,
            &["test", "--doc", "--workspace", "--locked", "-j", &jobs],
        ),
        "deny" => run_cargo(&root, &["deny", "check"]),
        "build" => run_cargo(&root, &["build", "--workspace", "--locked", "-j", &jobs]),
        "smoke" => run_cargo(
            &root,
            &["build", "--package", "gs", "--locked", "-j", &jobs],
        )
        .and_then(|()| {
            run_cargo(
                &root,
                &[
                    "run",
                    "--package",
                    "gs",
                    "--locked",
                    "-j",
                    &jobs,
                    "--",
                    "status",
                ],
            )
        }),
        "help" | "--help" | "-h" => {
            print_usage();
            Ok(())
        }
        other => Err(format!("unknown xtask command `{other}`")),
    };

    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("xtask error: {e}");
            ExitCode::FAILURE
        }
    }
}

fn print_usage() {
    println!("Usage: cargo xtask <command> [--break-check]");
    println!();
    println!("Commands:");
    println!(
        "  xwork        Execute full verification DAG (hygiene, fmt-check, clippy, test, graph-audit)"
    );
    println!("  check        Run verification gates (supports --break-check)");
    println!("  hygiene      Verify estate configuration files and absence of Justfile");
    println!(
        "  graph-audit  Verify workspace DAG and Wave-0 gs-domain purity (supports --break-check)"
    );
    println!("  fmt          Format all workspace Rust code");
    println!("  fmt-check    Check Rust formatting without modifying files");
    println!("  clippy       Run clippy across workspace with -D warnings");
    println!("  test         Run unit and integration tests (-j 2)");
    println!("  doc-test     Run documentation tests (-j 2)");
    println!("  deny         Run cargo-deny supply-chain check");
    println!("  build        Build workspace (-j 2)");
    println!("  smoke        Build and smoke-test the gs CLI");
}

fn workspace_root() -> Result<PathBuf, String> {
    let manifest_dir = env::var("CARGO_MANIFEST_DIR")
        .map(PathBuf::from)
        .map_err(|_| "CARGO_MANIFEST_DIR not set".to_string())?;
    manifest_dir
        .parent()
        .and_then(Path::parent)
        .map(Path::to_path_buf)
        .ok_or_else(|| "cannot resolve workspace root from CARGO_MANIFEST_DIR".to_string())
}

fn run_cargo(root: &Path, args: &[&str]) -> Result<(), String> {
    println!("==> cargo {}", args.join(" "));
    let status = Command::new("cargo")
        .args(args)
        .current_dir(root)
        .status()
        .map_err(|e| format!("failed to spawn cargo: {e}"))?;
    if status.success() {
        Ok(())
    } else {
        Err(format!("`cargo {}` exited with {status}", args.join(" ")))
    }
}

fn run_hygiene(root: &Path) -> Result<(), String> {
    println!("==> xtask hygiene");
    let required = [
        "rust-toolchain.toml",
        "rustfmt.toml",
        "clippy.toml",
        "deny.toml",
        ".cargo/config.toml",
        "Makefile",
        "Cargo.toml",
    ];
    for rel in required {
        if !root.join(rel).is_file() {
            return Err(format!("missing required estate file: {rel}"));
        }
    }
    if root.join("Justfile").exists() || root.join("justfile").exists() {
        return Err("Justfile is retired by estate SOP; use Makefile + cargo xtask".to_string());
    }
    let cargo_cfg = std::fs::read_to_string(root.join(".cargo/config.toml"))
        .map_err(|e| format!("read .cargo/config.toml: {e}"))?;
    if !cargo_cfg.contains("jobs = 2") {
        return Err(
            ".cargo/config.toml must enforce `jobs = 2` for host ceres throttling".to_string(),
        );
    }
    let root_toml = std::fs::read_to_string(root.join("Cargo.toml"))
        .map_err(|e| format!("read Cargo.toml: {e}"))?;
    if !root_toml.contains("line-tables-only") {
        return Err("Cargo.toml must set `[profile.dev] debug = \"line-tables-only\"`".to_string());
    }
    println!("xtask hygiene: ok");
    Ok(())
}

/// Evaluates whether `gs-domain` dependency list and source imports satisfy Wave-0 purity.
pub fn check_domain_purity(deps: &[&str], source_snippets: &[&str]) -> Result<(), String> {
    for dep in deps {
        if FORBIDDEN_DOMAIN_DEPS.contains(dep) {
            return Err(format!(
                "Wave-0 purity violation: `gs-domain` depends on forbidden crate `{dep}`"
            ));
        }
        if *dep == "gs" || *dep == "xtask" {
            return Err(format!(
                "DAG cycle violation: Wave-0 `gs-domain` cannot depend on higher-wave crate `{dep}`"
            ));
        }
    }
    for src in source_snippets {
        for forbidden in ["std::fs", "std::net", "std::process"] {
            if src.contains(forbidden) {
                return Err(format!(
                    "Wave-0 purity violation: `gs-domain` source references `{forbidden}`"
                ));
            }
        }
    }
    Ok(())
}

fn run_graph_audit(root: &Path, break_check: bool) -> Result<(), String> {
    println!("==> xtask graph-audit (break_check={break_check})");
    let domain_toml = std::fs::read_to_string(root.join("crates/gs-domain/Cargo.toml"))
        .map_err(|e| format!("read crates/gs-domain/Cargo.toml: {e}"))?;

    let mut deps = BTreeSet::new();
    let mut in_deps = false;
    for line in domain_toml.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with('[') {
            in_deps = trimmed == "[dependencies]" || trimmed.starts_with("[dependencies.");
            continue;
        }
        if in_deps && !trimmed.is_empty() && !trimmed.starts_with('#') {
            if let Some((name, _)) = trimmed.split_once('=') {
                let dep_name = name.trim().split('.').next().unwrap_or("").trim();
                if !dep_name.is_empty() {
                    deps.insert(dep_name.to_string());
                }
            }
        }
    }

    let mut sources = Vec::new();
    let src_dir = root.join("crates/gs-domain/src");
    let entries =
        std::fs::read_dir(&src_dir).map_err(|e| format!("read_dir {}: {e}", src_dir.display()))?;
    for entry in entries {
        let entry = entry.map_err(|e| e.to_string())?;
        let path = entry.path();
        if path.extension().and_then(|s| s.to_str()) == Some("rs") {
            let content = std::fs::read_to_string(&path)
                .map_err(|e| format!("read {}: {e}", path.display()))?;
            sources.push(content);
        }
    }

    let dep_refs: Vec<&str> = deps.iter().map(String::as_str).collect();
    let src_refs: Vec<&str> = sources.iter().map(String::as_str).collect();
    check_domain_purity(&dep_refs, &src_refs)?;

    if break_check {
        let mut sabotaged = dep_refs.clone();
        sabotaged.push("tokio");
        if check_domain_purity(&sabotaged, &src_refs).is_ok() {
            return Err("break-check failed: injected `tokio` into gs-domain did not fail check_domain_purity".to_string());
        }
        let sabotaged_src = vec!["use std::fs::File;"];
        if check_domain_purity(&dep_refs, &sabotaged_src).is_ok() {
            return Err("break-check failed: injected `std::fs` into gs-domain did not fail check_domain_purity".to_string());
        }
        println!("xtask graph-audit: break-check verified (sabotaged inputs rejected)");
    }

    println!("xtask graph-audit: ok (gs-domain Wave-0 pure)");
    Ok(())
}

fn run_check(root: &Path, jobs: &str, break_check: bool) -> Result<(), String> {
    run_hygiene(root)?;
    run_graph_audit(root, break_check)?;
    run_cargo(root, &["fmt", "--all", "--", "--check"])?;
    run_cargo(
        root,
        &[
            "clippy",
            "--workspace",
            "--all-targets",
            "--locked",
            "-j",
            jobs,
            "--",
            "-D",
            "warnings",
        ],
    )?;
    run_cargo(root, &["test", "--workspace", "--locked", "-j", jobs])?;
    run_cargo(
        root,
        &["test", "--doc", "--workspace", "--locked", "-j", jobs],
    )?;
    Ok(())
}

fn run_xwork(root: &Path, jobs: &str) -> Result<(), String> {
    run_check(root, jobs, true)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wave0_domain_purity_accepts_clean_domain_and_rejects_io() {
        assert!(
            check_domain_purity(&["serde", "serde_json", "thiserror"], &["pub struct Foo;"])
                .is_ok()
        );
        assert!(check_domain_purity(&["serde", "tokio"], &["pub struct Foo;"]).is_err());
        assert!(check_domain_purity(&["gs"], &["pub struct Foo;"]).is_err());
        assert!(check_domain_purity(&["serde"], &["use std::net::TcpStream;"]).is_err());
    }
}
