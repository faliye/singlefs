//! 命令行 `unit-graph`：
//!
//!   unit-graph dump <根> [--threads <N>]
//!   unit-graph closure <根> <入口 fn 路径> [<入口 fn 路径>…] [--target <目标>] [--cfg <谓词>]… [--no-default-features]
//!                      [--toolchain-line <行>…] [--toolchain-of-the-build <profile>]
//!                      [--lock-line <行>… | --lock-file <Cargo.lock>] [--threads <N>]
//!   unit-graph key <根> <入口 fn 路径> [<入口 fn 路径>…] [与 closure 相同的选项]
//!
//! 给几个入口时一起收一个闭包（`compute_closure_of_entries`：几个入口都在 `--target` 点名的那个目标里、同一个 crate）。
//! `--toolchain-of-the-build <profile>` 照这台机器现拼一次构建的工具链行（`build_toolchain::toolchain_lines_of_the_build`，与
//! `singlefs-checker-tier` 的构建脚本同一份拼法）：跑 `$RUSTC -vV`（没设跑 `rustc`）与 `$CARGO -V`（没设跑 `cargo`），目标取
//! `CARGO_BUILD_TARGET`、没设取 `rustc -vV` 的 `host:`，RUSTFLAGS 取 `CARGO_ENCODED_RUSTFLAGS`、没设按空白切 `RUSTFLAGS`，
//! profile 的生效值从 `<根>` 那棵树的根 `Cargo.toml` 算（`CARGO_PROFILE_<名>_<键>` 照盖）；拼出的行接在 `--toolchain-line` 给的后面。
//! 要与构建脚本写进 `OUT_DIR` 的键逐字相同，另给 `--no-default-features` 与构建时开的每个特性（`--cfg 'feature="default"'` 这样逐个写）
//! 和 `--lock-file <根>/Cargo.lock`。
//!
//! `<根>` 是仓根或 `crates/` 目录。目标写 `lib`（不给时）、`bin:<名>`、`test:<名>`、`example:<名>`、`bench:<名>`、`build`、`doctest`；
//! 库里的单测要另给 `--cfg test`。`--cfg` 写 `名字` 或 `键="值"`（`feature="gpu"` 当本 crate 的特性）。
//! 目标平台的 cfg（`target_os`、`target_arch`、`target_pointer_width`、`unix` 这类）从工具链的目标三元组查表：工具链行里
//! `target: <三元组>`（cargo `--target` 给的）优先，没有就取 `host: <三元组>`（`rustc -vV` 那一行原样）；两种都没给是用法错，
//! 表里查不出的三元组拒算（`target-triple-unrecognized`）。
//! 依赖行两种给法，只能取一种：`--lock-line` 是调用方自己收窄好的行；`--lock-file` 给整份 `Cargo.lock`，只取闭包成员真点名的
//! 外部 crate 解析到的包与它们的传递依赖（名、版本、来源、checksum），落不到包的拒算（`lock-dependency-unresolved`）。
//! 输出一行一项、次序确定（行格式见库的 `output` 模块）；单元数、拒算数与用时打在标准错误。
//! `--threads` 不给时照 `.claude/heavy-thread-default` 的取法（`SINGLEFS_THREAD_CAP` 设了照它）。
//!
//! 退出码：0 交了全部行（`dump` 照列拒算行，也是 0）；2 用法错（缺目标三元组、两种依赖行都给了）、根目录或清单读不了、
//! `--lock-file` 读不了、`--toolchain-of-the-build` 拼不出（`rustc`、`cargo` 跑不了，profile 算不出）、入口找不到或不唯一、
//! 几个入口不在同一个 crate、入口在这组 cfg 下不编进来、写标准输出失败；
//! 3 闭包里有拒算（`closure` 列组成清单与拒算行、不打键行，`key` 只打拒算行）。
//! 弄坏开关的环境变量这里不认（只给测试用），规则一律全开。

use std::io::Write as _;
use std::num::NonZeroUsize;
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::time::Instant;

use singlefs_unit_graph::build_toolchain::{
    effective_profile_lines, toolchain_lines_of_the_build, BuildToolchainInputs,
};
use singlefs_unit_graph::closure::{
    compute_closure_of_entries, reuse_key, ClosureRequest, ExternalKeyInputs, RequestedCondition,
};
use singlefs_unit_graph::conditional_compilation::target_triple_of_toolchain_lines;
use singlefs_unit_graph::heavy_threads::default_heavy_threads;
use singlefs_unit_graph::lock_file::{narrowed_lock_lines, parse_lock_file, LockedPackage};
use singlefs_unit_graph::output::{
    closure_lines, dump_lines, key_line, parse_time_refusals, refusal_lines,
};
use singlefs_unit_graph::rule_switches::RuleSwitches;
use singlefs_unit_graph::workspace::{load_workspace, TargetSelector, Workspace};

const USAGE: &str = concat!(
    "用法：unit-graph dump <根> [--threads <N>]\n",
    "      unit-graph closure <根> <入口 fn 路径> [<入口 fn 路径>…] [--target <目标>] [--cfg <谓词>]… [--no-default-features] ",
    "[--toolchain-line <行>…（其中一行 host: <三元组> 或 target: <三元组>）] [--toolchain-of-the-build <profile>] ",
    "[--lock-line <行>… | --lock-file <Cargo.lock>] [--threads <N>]\n",
    "      unit-graph key <根> <入口 fn 路径> [<入口 fn 路径>…] [与 closure 相同的选项]"
);

/// 用法错、读不了、入口找不到。
const EXIT_USAGE: u8 = 2;
/// 闭包里有拒算。
const EXIT_REFUSED: u8 = 3;

#[derive(Clone, Copy, PartialEq, Eq)]
enum Command {
    Dump,
    Closure,
    Key,
}

struct Arguments {
    command: Command,
    root: PathBuf,
    entry_paths: Vec<String>,
    target: TargetSelector,
    conditions: Vec<RequestedCondition>,
    uses_default_features: bool,
    toolchain_lines: Vec<String>,
    lock_lines: Vec<String>,
    lock_file: Option<PathBuf>,
    build_profile: Option<String>,
    threads: Option<NonZeroUsize>,
}

fn parse_arguments(raw: &[String]) -> Result<Arguments, String> {
    let Some(command_text) = raw.first() else {
        return Err("缺子命令".to_string());
    };
    let command = match command_text.as_str() {
        "dump" => Command::Dump,
        "closure" => Command::Closure,
        "key" => Command::Key,
        other => return Err(format!("认不得子命令「{other}」")),
    };
    let mut positional = Vec::new();
    let mut arguments = Arguments {
        command,
        root: PathBuf::new(),
        entry_paths: Vec::new(),
        target: TargetSelector::Library,
        conditions: Vec::new(),
        uses_default_features: true,
        toolchain_lines: Vec::new(),
        lock_lines: Vec::new(),
        lock_file: None,
        build_profile: None,
        threads: None,
    };
    let mut position = 1usize;
    while position < raw.len() {
        let current = raw[position].as_str();
        let value = raw.get(position + 1);
        match current {
            "--threads"
            | "--target"
            | "--cfg"
            | "--toolchain-line"
            | "--lock-line"
            | "--lock-file"
            | "--toolchain-of-the-build" => {
                let Some(value) = value else {
                    return Err(format!("{current} 后面缺值"));
                };
                match current {
                    "--threads" => {
                        arguments.threads = Some(
                            value
                                .parse::<usize>()
                                .ok()
                                .and_then(NonZeroUsize::new)
                                .ok_or_else(|| format!("--threads 要正整数，给的是「{value}」"))?,
                        );
                    }
                    "--target" => {
                        arguments.target = TargetSelector::parse(value)
                            .map_err(|text| format!("--target 认不得「{text}」"))?;
                    }
                    "--cfg" => arguments.conditions.push(
                        RequestedCondition::parse(value)
                            .map_err(|text| format!("--cfg 认不得「{text}」"))?,
                    ),
                    "--toolchain-line" => arguments.toolchain_lines.push(value.clone()),
                    "--lock-file" => arguments.lock_file = Some(PathBuf::from(value)),
                    "--toolchain-of-the-build" => arguments.build_profile = Some(value.clone()),
                    _ => arguments.lock_lines.push(value.clone()),
                }
                position += 2;
            }
            "--no-default-features" => {
                arguments.uses_default_features = false;
                position += 1;
            }
            option if option.starts_with("--") => return Err(format!("认不得选项「{option}」")),
            _ => {
                positional.push(current.to_string());
                position += 1;
            }
        }
    }
    let positional_count_fits = if command == Command::Dump {
        positional.len() == 1
    } else {
        positional.len() >= 2
    };
    if !positional_count_fits {
        return Err(format!(
            "dump 要 1 个位置参数（根），closure 与 key 要根与至少一个入口；给了 {} 个",
            positional.len()
        ));
    }
    if arguments.lock_file.is_some() && !arguments.lock_lines.is_empty() {
        return Err("--lock-line 与 --lock-file 只能给一种".to_string());
    }
    arguments.root = PathBuf::from(&positional[0]);
    arguments.entry_paths = positional[1..].to_vec();
    Ok(arguments)
}

/// 跑一条命令取它的标准输出（`--toolchain-of-the-build` 问 `rustc -vV` 与 `cargo -V` 用）。
fn output_of(program: &str, argument: &str) -> Result<String, String> {
    let output = std::process::Command::new(program)
        .arg(argument)
        .output()
        .map_err(|error| format!("跑不了 {program} {argument}：{error}"))?;
    if !output.status.success() {
        return Err(format!(
            "{program} {argument} 退 {:?}",
            output.status.code()
        ));
    }
    String::from_utf8(output.stdout).map_err(|_| format!("{program} {argument} 的输出不是 UTF-8"))
}

/// `--toolchain-of-the-build <profile>`：照这台机器与 `workspace` 那棵树的根 `Cargo.toml` 现拼工具链行（文件头写了取法）。
fn toolchain_lines_of_this_machine(
    workspace: &Workspace,
    profile: &str,
) -> Result<Vec<String>, String> {
    let environment = |name: &str| std::env::var(name).ok();
    let rustc = environment("RUSTC").unwrap_or_else(|| "rustc".to_string());
    let cargo = environment("CARGO").unwrap_or_else(|| "cargo".to_string());
    let rustc_version_text = output_of(&rustc, "-vV")?;
    let cargo_version_text = output_of(&cargo, "-V")?;
    let host_triple = rustc_version_text
        .lines()
        .find_map(|line| line.strip_prefix("host: "))
        .map(str::trim)
        .ok_or_else(|| format!("{rustc} -vV 里没有 host: 那一行"))?
        .to_string();
    let target_triple = environment("CARGO_BUILD_TARGET").unwrap_or(host_triple);
    let encoded_rustflags = environment("CARGO_ENCODED_RUSTFLAGS").unwrap_or_else(|| {
        environment("RUSTFLAGS")
            .unwrap_or_default()
            .split_whitespace()
            .collect::<Vec<&str>>()
            .join("\u{1f}")
    });
    let root_manifest = workspace.repository_root.join("Cargo.toml");
    let root_manifest_text = std::fs::read_to_string(&root_manifest)
        .map_err(|error| format!("{} 读不了：{error}", root_manifest.display()))?;
    let profile_lines = effective_profile_lines(&root_manifest_text, profile, &environment)?;
    Ok(toolchain_lines_of_the_build(&BuildToolchainInputs {
        rustc_version_text: &rustc_version_text,
        cargo_version_text: &cargo_version_text,
        target_triple: &target_triple,
        encoded_rustflags: &encoded_rustflags,
        profile_lines: &profile_lines,
    }))
}

/// 读并解析 `--lock-file` 给的那份 `Cargo.lock`。
fn locked_packages_of(path: &Path) -> Result<Vec<LockedPackage>, String> {
    let text = std::fs::read_to_string(path)
        .map_err(|error| format!("{} 读不了：{error}", path.display()))?;
    parse_lock_file(&text).map_err(|error| format!("{}：{error}", path.display()))
}

/// 跑一个子命令，交要打的行、标准错误的那一句与退出码。
fn lines_of(
    workspace: &Workspace,
    arguments: &Arguments,
    switches: &RuleSwitches,
) -> (Vec<String>, String, u8) {
    let entry_path = arguments.entry_paths.join(" ");
    match arguments.command {
        Command::Dump => {
            let lines = dump_lines(workspace);
            let summary = format!(
                "{} 个单元（Rust {}、WGSL {}）、{} 份文件、拒算 {} 处",
                workspace.units.len() + workspace.wgsl_units.len(),
                workspace.units.len(),
                workspace.wgsl_units.len(),
                workspace.files.len(),
                parse_time_refusals(workspace).len()
            );
            (lines, summary, 0)
        }
        Command::Closure | Command::Key => {
            let mut toolchain_lines = arguments.toolchain_lines.clone();
            if let Some(profile) = &arguments.build_profile {
                match toolchain_lines_of_this_machine(workspace, profile) {
                    Ok(lines) => toolchain_lines.extend(lines),
                    Err(message) => {
                        return (
                            Vec::new(),
                            format!("--toolchain-of-the-build 拼不出：{message}"),
                            EXIT_USAGE,
                        )
                    }
                }
            }
            let target_triple = match target_triple_of_toolchain_lines(&toolchain_lines) {
                Ok(Some(triple)) => triple,
                Ok(None) => {
                    return (
                        Vec::new(),
                        "缺目标三元组：--toolchain-line 里要有一行 host: <三元组>（rustc -vV）或 target: <三元组>（cargo --target）"
                            .to_string(),
                        EXIT_USAGE,
                    )
                }
                Err(prefix) => {
                    return (
                        Vec::new(),
                        format!("--toolchain-line 里「{prefix}」那一种行给了不止一行"),
                        EXIT_USAGE,
                    )
                }
            };
            let (first_entry, additional_entries) = arguments
                .entry_paths
                .split_first()
                .expect("parse_arguments 核过 closure 与 key 至少一个入口");
            let request = ClosureRequest {
                entry_path: first_entry.clone(),
                target: arguments.target.clone(),
                requested_conditions: arguments.conditions.clone(),
                uses_default_features: arguments.uses_default_features,
                target_triple,
            };
            let mut outcome =
                match compute_closure_of_entries(workspace, &request, additional_entries, switches)
                {
                    Ok(outcome) => outcome,
                    Err(error) => return (Vec::new(), error.to_string(), EXIT_USAGE),
                };
            let lock_lines = match &arguments.lock_file {
                None => arguments.lock_lines.clone(),
                Some(path) => {
                    let packages = match locked_packages_of(path) {
                        Ok(packages) => packages,
                        Err(message) => return (Vec::new(), message, EXIT_USAGE),
                    };
                    match narrowed_lock_lines(workspace, &outcome, &packages) {
                        Ok(lines) => lines,
                        Err(refusals) => {
                            outcome.refusals.extend(refusals);
                            outcome.refusals.sort();
                            outcome.refusals.dedup();
                            Vec::new()
                        }
                    }
                }
            };
            let external = ExternalKeyInputs {
                toolchain_lines,
                lock_lines,
            };
            let key = reuse_key(workspace, &outcome, &external);
            let lines = if arguments.command == Command::Key {
                match &key {
                    Ok(digest) => vec![key_line(digest)],
                    Err(refusals) => refusal_lines(workspace, refusals),
                }
            } else {
                closure_lines(workspace, &outcome, key.as_ref().ok())
            };
            let summary = format!(
                "入口 {entry_path}：闭包 {} 个单元（Rust {}、WGSL {}、数据 {}），拒算 {} 处",
                outcome.units.len() + outcome.wgsl_units.len() + outcome.data_units.len(),
                outcome.units.len(),
                outcome.wgsl_units.len(),
                outcome.data_units.len(),
                outcome.refusals.len()
            );
            let exit_code = if key.is_ok() { 0 } else { EXIT_REFUSED };
            (lines, summary, exit_code)
        }
    }
}

fn main() -> ExitCode {
    let raw: Vec<String> = std::env::args().skip(1).collect();
    let arguments = match parse_arguments(&raw) {
        Ok(arguments) => arguments,
        Err(message) => {
            eprintln!("unit-graph：{message}\n{USAGE}");
            return ExitCode::from(EXIT_USAGE);
        }
    };
    let threads = arguments.threads.unwrap_or_else(default_heavy_threads);
    let started = Instant::now();
    let switches = RuleSwitches::every_rule_intact();
    let workspace = match load_workspace(&arguments.root, threads, &switches) {
        Ok(workspace) => workspace,
        Err(error) => {
            eprintln!("unit-graph：{error}");
            return ExitCode::from(EXIT_USAGE);
        }
    };
    let (lines, summary, exit_code) = lines_of(&workspace, &arguments, &switches);
    let mut text = lines.join("\n");
    if !lines.is_empty() {
        text.push('\n');
    }
    if let Err(error) = std::io::stdout().lock().write_all(text.as_bytes()) {
        eprintln!("unit-graph：写标准输出失败：{error}");
        return ExitCode::from(EXIT_USAGE);
    }
    eprintln!(
        "unit-graph：{summary}，{threads} 线程，{:.2} 秒",
        started.elapsed().as_secs_f64()
    );
    ExitCode::from(exit_code)
}
