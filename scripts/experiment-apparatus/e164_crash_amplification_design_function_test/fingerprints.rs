//! E164 F13：用例指纹的两臂（跑前登记 5.7「F13」）与「会不会改编出的代码」的重编对照。
//! 不做臂 = `admission.py crash-case-manifest --judging-digest --toolchain --build-environment` 今天的算法；
//! 做臂 = admission 今天收的源文件清单（带 --judging-digest，不带后两样）+ 只收会改编出代码的配置键。

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use crate::common::{run_allowed, sha256_hexadecimal, strings};
use crate::repository_scans::{dependency_graph, ProductSuffix};

const CASE_KEY: &str = "crash-case:layer0-first-stream";
const CASE_PACKAGE: &str = "singlefs-verification";

/// 做臂收的配置表与键：`build` 表里这几个键，`target.*` 下这几个键，`profile` 整张。`[env]` 另按闭包判。只给变异 M19 加键。
const CONFIGURATION_KEYS_THAT_CHANGE_CODE: &[&str] = &["build.rustflags", "build.rustc", "build.rustc-wrapper", "build.rustc-workspace-wrapper", "build.target"];
const TARGET_KEYS_THAT_CHANGE_CODE: &[&str] = &["rustflags", "runner", "linker"];
/// 做臂收的环境变量（名字相等或以 `*` 前的一段起头、以 `*` 后的一段收尾）。只给变异 M20 删键。
const ENVIRONMENT_VARIABLES_THAT_CHANGE_CODE: &[&str] = &[
    "RUSTFLAGS", "CARGO_ENCODED_RUSTFLAGS", "CARGO_PROFILE_*", "CARGO_TARGET_*_RUSTFLAGS", "CARGO_TARGET_*_RUNNER",
    "CARGO_TARGET_*_LINKER", "RUSTC", "RUSTC_WRAPPER", "RUSTC_WORKSPACE_WRAPPER", "CARGO_BUILD_RUSTFLAGS", "CARGO_BUILD_TARGET",
];
/// 闭包里有带 build 脚本的包时另收的环境变量（`[env]` 表里的键另外全收）。
const BUILD_SCRIPT_ENVIRONMENT_VARIABLES: &[&str] = &["CC", "CXX", "CFLAGS", "CXXFLAGS", "LDFLAGS", "BINDGEN_EXTRA_CLANG_ARGS", "LIBCLANG_PATH", "PKG_CONFIG_PATH"];

fn variable_matches(pattern: &str, name: &str) -> bool {
    match pattern.split_once('*') {
        Some((head, tail)) => name.starts_with(head) && name.ends_with(tail) && name.len() >= head.len() + tail.len(),
        None => pattern == name,
    }
}

/// 拷一份仓（带 .git，跳过名叫 target 的目录）；不带旧修改时刻。
pub fn copy_tree(source: &Path, destination: &Path) {
    std::fs::create_dir_all(destination).expect("建得了目标目录");
    for entry in std::fs::read_dir(source).expect("读得了源目录").flatten() {
        let file_type = entry.file_type().expect("读得了文件类型");
        let target = destination.join(entry.file_name());
        if file_type.is_dir() {
            if entry.file_name() != "target" {
                copy_tree(&entry.path(), &target);
            }
        } else if file_type.is_symlink() {
            let link = std::fs::read_link(entry.path()).expect("读得了链接");
            std::os::unix::fs::symlink(link, &target).expect("建得了链接");
        } else {
            std::fs::copy(entry.path(), &target).expect("拷得了文件");
        }
    }
}

/// 一份 cargo 配置文件里的（表.键，值原文）。只认单行的 `键 = 值` 与 `[表]`，够判这几种设置。
fn configuration_entries(text: &str) -> Vec<(String, String)> {
    let mut table = String::new();
    let mut entries = Vec::new();
    for line in text.lines().map(str::trim) {
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if let Some(header) = line.strip_prefix('[').and_then(|rest| rest.strip_suffix(']')) {
            table = header.trim().to_string();
            continue;
        }
        if let Some((key, value)) = line.split_once('=') {
            entries.push((format!("{table}.{}", key.trim()), value.trim().to_string()));
        }
    }
    entries
}

/// cargo 读配置的层：从 `directory` 往上每一层的 `.cargo/config.toml` 与 `.cargo/config`，再加 CARGO_HOME 的。
fn configuration_layers(directory: &Path, environment: &BTreeMap<String, String>) -> Vec<String> {
    let mut texts = Vec::new();
    let mut current = Some(directory.to_path_buf());
    while let Some(level) = current {
        for name in ["config.toml", "config"] {
            if let Ok(text) = std::fs::read_to_string(level.join(".cargo").join(name)) {
                texts.push(text);
            }
        }
        current = level.parent().map(Path::to_path_buf);
    }
    let cargo_home = environment.get("CARGO_HOME").map(PathBuf::from).unwrap_or_else(|| PathBuf::from(environment.get("HOME").cloned().unwrap_or_default()).join(".cargo"));
    for name in ["config.toml", "config"] {
        if let Ok(text) = std::fs::read_to_string(cargo_home.join(name)) {
            texts.push(text);
        }
    }
    texts
}

fn program_content_line(value: &str, environment: &BTreeMap<String, String>) -> String {
    let program = value.trim_matches(|character| character == '"' || character == '[' || character == ']').split([' ', ',']).next().unwrap_or("").trim_matches('"');
    let candidates: Vec<PathBuf> = if program.contains('/') {
        vec![PathBuf::from(program)]
    } else {
        environment.get("PATH").map(|path| path.split(':').map(|directory| Path::new(directory).join(program)).collect()).unwrap_or_default()
    };
    match candidates.iter().find_map(|candidate| std::fs::read(candidate).ok()) {
        Some(bytes) => format!("program {program} {}", sha256_hexadecimal(&bytes)),
        None => format!("program {program} missing"),
    }
}

/// 做臂里配置那一半：只收会改编出代码的键；闭包里有带 build 脚本的包时另收 `[env]` 与那几个环境变量。
pub fn code_changing_configuration_lines(directory: &Path, environment: &BTreeMap<String, String>, closure_has_build_script: bool) -> Vec<String> {
    let mut lines = BTreeSet::new();
    for text in configuration_layers(directory, environment) {
        for (key, value) in configuration_entries(&text) {
            let is_target_key = key.starts_with("target.") && TARGET_KEYS_THAT_CHANGE_CODE.iter().any(|name| key.ends_with(&format!(".{name}")));
            let takes = CONFIGURATION_KEYS_THAT_CHANGE_CODE.contains(&key.as_str())
                || is_target_key
                || key.starts_with("profile.")
                || (closure_has_build_script && key.starts_with("env."));
            if takes {
                if key.ends_with("runner") || key.ends_with("wrapper") {
                    lines.insert(program_content_line(&value, environment));
                }
                lines.insert(format!("config {key}={value}"));
            }
            if closure_has_build_script && key.starts_with("env.") {
                let variable = key.trim_start_matches("env.");
                if let Some(environment_value) = environment.get(variable) {
                    lines.insert(format!("environment {variable}={environment_value}"));
                }
            }
        }
    }
    for (name, value) in environment {
        let changes_code = ENVIRONMENT_VARIABLES_THAT_CHANGE_CODE.iter().any(|pattern| variable_matches(pattern, name))
            || (closure_has_build_script && BUILD_SCRIPT_ENVIRONMENT_VARIABLES.contains(&name.as_str()));
        if changes_code {
            if name.ends_with("RUNNER") || name.ends_with("WRAPPER") {
                lines.insert(program_content_line(value, environment));
            }
            lines.insert(format!("environment {name}={value}"));
        }
    }
    lines.into_iter().collect()
}

fn environment_list(environment: &BTreeMap<String, String>) -> Vec<(String, Option<String>)> {
    let current: BTreeMap<String, String> = std::env::vars().collect();
    let mut list: Vec<(String, Option<String>)> = current.keys().filter(|name| !environment.contains_key(*name)).map(|name| (name.clone(), None)).collect();
    list.extend(environment.iter().map(|(name, value)| (name.clone(), Some(value.clone()))));
    list
}

fn rustc_version_text(directory: &Path, environment: &BTreeMap<String, String>) -> String {
    let output = run_allowed("rustc", &strings(&["-vV"]), directory, &environment_list(environment));
    String::from_utf8_lossy(&output.stdout).into_owned()
}

/// admission 的用例清单：交回（指纹，清单文件的行）。
fn admission_manifest(copy_root: &Path, environment: &BTreeMap<String, String>, with_environment_lines: bool, scratch: &Path) -> (String, Vec<String>) {
    let manifest_path = scratch.join(format!("manifest-{}.txt", u8::from(with_environment_lines)));
    let mut arguments = vec![
        copy_root.join("scripts/gate/admission.py").display().to_string(),
        "crash-case-manifest".to_string(),
        copy_root.display().to_string(),
        CASE_KEY.to_string(),
        manifest_path.display().to_string(),
        "--judging-digest".to_string(),
    ];
    if with_environment_lines {
        arguments.extend(strings(&["--toolchain", "--build-environment"]));
    }
    let output = run_allowed("python3", &arguments, copy_root, &environment_list(environment));
    assert!(output.status.success(), "admission crash-case-manifest 失败：{}", String::from_utf8_lossy(&output.stderr));
    let fingerprint = String::from_utf8_lossy(&output.stdout).split_whitespace().next().expect("admission 打出指纹").to_string();
    let lines = std::fs::read_to_string(&manifest_path).expect("读得了清单").lines().map(str::to_string).collect();
    (fingerprint, lines)
}

fn closure_has_build_script(directory: &Path, package: &str, extra_arguments: &[&str]) -> Result<bool, String> {
    let graph = dependency_graph(directory, extra_arguments)?;
    let identifier = graph.identifier_of(package).ok_or_else(|| format!("{package} 不是工作区成员"))?;
    Ok(graph.closure(&identifier, true).iter().any(|member| graph.packages[member].has_build_script))
}

struct ArmFingerprints {
    not_doing: String,
    doing: String,
}

fn both_fingerprints(copy_root: &Path, environment: &BTreeMap<String, String>, build_script_in_closure: bool, scratch: &Path) -> ArmFingerprints {
    let (not_doing, _) = admission_manifest(copy_root, environment, true, scratch);
    let (_, source_lines) = admission_manifest(copy_root, environment, false, scratch);
    let mut doing_text = source_lines.join("\n");
    doing_text.push('\n');
    doing_text.push_str(&rustc_version_text(copy_root, environment));
    for line in code_changing_configuration_lines(copy_root, environment, build_script_in_closure) {
        doing_text.push_str(&line);
        doing_text.push('\n');
    }
    ArmFingerprints { not_doing, doing: sha256_hexadecimal(doing_text.as_bytes()) }
}

/// 重编对照：同一份副本、同一个编译目录里编一次，交回 cargo 对工作区包报了 Compiling / Dirty 没有。
fn builds_workspace_packages(copy_root: &Path, environment: &BTreeMap<String, String>) -> bool {
    let arguments = strings(&["build", "--release", "--offline", "-p", "singlefs-verification", "--lib", "-v"]);
    let output = run_allowed("cargo", &arguments, copy_root, &environment_list(environment));
    assert!(output.status.success(), "重编对照的 cargo build 失败：{}", String::from_utf8_lossy(&output.stderr).lines().rev().take(5).collect::<Vec<&str>>().join(" / "));
    String::from_utf8_lossy(&output.stderr).lines().any(|line| {
        let trimmed = line.trim_start();
        trimmed.starts_with("Compiling singlefs-") || trimmed.starts_with("Dirty singlefs-")
    })
}

struct Setting {
    name: &'static str,
    /// 这一种设置改了编译环境之外什么：记在产物里，只为读的人。
    description: &'static str,
}

const SETTINGS: [Setting; 6] = [
    Setting { name: "S1", description: "personal_cargo_home_with_term_color" },
    Setting { name: "S2", description: "upper_directory_cargo_config_with_net_offline" },
    Setting { name: "S3", description: "repository_cargo_config_gcc_13_to_14" },
    Setting { name: "S4", description: "rustflags_target_cpu_native" },
    Setting { name: "S5", description: "profile_release_overflow_checks_environment" },
    Setting { name: "S6", description: "other_path_other_target_directory_other_jobs" },
];

const GCC_THIRTEEN_INCLUDE: &str = "gcc/x86_64-linux-gnu/13/include";
const GCC_FOURTEEN_INCLUDE: &str = "gcc/x86_64-linux-gnu/14/include";

fn replace_in_file(path: &Path, old: &str, new: &str) {
    let text = std::fs::read_to_string(path).expect("读得了要改的文件");
    assert_eq!(text.matches(old).count(), 1, "{} 里 {old} 应恰好命中一次", path.display());
    std::fs::write(path, text.replace(old, new)).expect("写得回去");
}

/// 施加一种设置：交回这一种设置下的环境；改了文件的由 `revert_setting` 换回。
fn apply_setting(name: &str, copy_root: &Path, upper: &Path, scratch: &Path, base: &BTreeMap<String, String>) -> BTreeMap<String, String> {
    let mut environment = base.clone();
    match name {
        "S1" => {
            let real_home = base.get("CARGO_HOME").map(PathBuf::from).unwrap_or_else(|| PathBuf::from(base.get("HOME").cloned().unwrap_or_default()).join(".cargo"));
            let personal = scratch.join("cargo-home-s1");
            std::fs::create_dir_all(&personal).expect("建得了个人 CARGO_HOME");
            for shared in ["registry", "git", "bin"] {
                let link = personal.join(shared);
                if !link.exists() {
                    std::os::unix::fs::symlink(real_home.join(shared), link).expect("链得上共用的缓存");
                }
            }
            std::fs::write(personal.join("config.toml"), "[term]\ncolor = \"always\"\n").expect("写得了个人配置");
            environment.insert("CARGO_HOME".to_string(), personal.display().to_string());
        }
        "S2" => {
            std::fs::create_dir_all(upper.join(".cargo")).expect("建得了上层 .cargo");
            std::fs::write(upper.join(".cargo/config.toml"), "[net]\noffline = true\n").expect("写得了上层配置");
        }
        "S3" => replace_in_file(&copy_root.join(".cargo/config.toml"), GCC_THIRTEEN_INCLUDE, GCC_FOURTEEN_INCLUDE),
        "S4" => {
            environment.insert("RUSTFLAGS".to_string(), "-C target-cpu=native".to_string());
        }
        "S5" => {
            environment.insert("CARGO_PROFILE_RELEASE_OVERFLOW_CHECKS".to_string(), "true".to_string());
        }
        other => panic!("apply_setting 不管 {other}"),
    }
    environment
}

fn revert_setting(name: &str, copy_root: &Path, upper: &Path) {
    match name {
        "S2" => std::fs::remove_file(upper.join(".cargo/config.toml")).expect("删得了上层配置"),
        "S3" => replace_in_file(&copy_root.join(".cargo/config.toml"), GCC_FOURTEEN_INCLUDE, GCC_THIRTEEN_INCLUDE),
        _ => {}
    }
}

pub fn scan_fingerprints(root: &Path, suffix: &ProductSuffix) {
    let scratch = crate::scratch_directory().join("f13");
    if scratch.exists() {
        std::fs::remove_dir_all(&scratch).expect("清得掉上一趟的 F13 草稿");
    }
    let upper = scratch.join("upper");
    let copy_root = upper.join("repository");
    copy_tree(root, &copy_root);
    let base: BTreeMap<String, String> = std::env::vars().collect();
    let layer0_build_script = match closure_has_build_script(&copy_root, CASE_PACKAGE, &[]) {
        Ok(answer) => answer,
        Err(message) => {
            suffix.emit(&format!("E7RESULT name=q13_offline_failure closure=layer0 message={}", message.replace(' ', "_")));
            return;
        }
    };
    suffix.emit(&format!("E7RESULT name=q13_closure case={CASE_KEY} closure_has_build_script={layer0_build_script}"));
    let default_fingerprints = both_fingerprints(&copy_root, &base, layer0_build_script, &scratch);
    builds_workspace_packages(&copy_root, &base);
    let mut rows: Vec<(String, bool, bool, String)> = Vec::new();
    for setting in &SETTINGS {
        let (fingerprints, rebuild) = if setting.name == "S6" {
            let other_root = scratch.join("elsewhere/level/repository");
            copy_tree(root, &other_root);
            let mut environment = base.clone();
            environment.insert("CARGO_TARGET_DIR".to_string(), scratch.join("target-s6").display().to_string());
            environment.insert("CARGO_BUILD_JOBS".to_string(), "3".to_string());
            (both_fingerprints(&other_root, &environment, layer0_build_script, &scratch), "not_applicable".to_string())
        } else {
            let environment = apply_setting(setting.name, &copy_root, &upper, &scratch, &base);
            let fingerprints = both_fingerprints(&copy_root, &environment, layer0_build_script, &scratch);
            let rebuild = if builds_workspace_packages(&copy_root, &environment) { "rebuilds" } else { "does_not_rebuild" };
            revert_setting(setting.name, &copy_root, &upper);
            builds_workspace_packages(&copy_root, &base);
            (fingerprints, rebuild.to_string())
        };
        let not_doing_changed = fingerprints.not_doing != default_fingerprints.not_doing;
        let doing_changed = fingerprints.doing != default_fingerprints.doing;
        suffix.emit(&format!(
            "E7RESULT name=q13_setting setting={} description={} not_doing_fingerprint_changed={not_doing_changed} doing_fingerprint_changed={doing_changed} rebuild_control={rebuild}",
            setting.name, setting.description
        ));
        rows.push((setting.name.to_string(), not_doing_changed, doing_changed, rebuild));
    }
    let doing_missed = rows.iter().filter(|(_, _, doing, rebuild)| rebuild == "rebuilds" && !doing).count();
    let doing_false = rows.iter().filter(|(_, _, doing, rebuild)| rebuild == "does_not_rebuild" && *doing).count();
    let not_doing_false = rows.iter().filter(|(_, not_doing, _, rebuild)| rebuild == "does_not_rebuild" && *not_doing).count();
    let not_doing_positive = rows.iter().filter(|(name, not_doing, _, _)| ["S1", "S2", "S3"].contains(&name.as_str()) && *not_doing).count();
    let doing_positive = rows.iter().filter(|(name, _, doing, _)| ["S4", "S5"].contains(&name.as_str()) && *doing).count();
    suffix.emit(&format!(
        "E7RESULT name=q13_summary doing_missed={doing_missed} doing_false={doing_false} not_doing_false={not_doing_false} not_doing_positive_settings_among_s1_to_s3={not_doing_positive} doing_positive_settings_among_s4_s5={doing_positive}"
    ));
    let research = copy_root.join("scripts").join("experiments");
    match closure_has_build_script(&research, "e7-index-bench", &["--features", "e162-block-stores"]) {
        Ok(e162_build_script) => {
            let before = code_changing_configuration_lines(&research, &base, e162_build_script);
            replace_in_file(&copy_root.join(".cargo/config.toml"), GCC_THIRTEEN_INCLUDE, GCC_FOURTEEN_INCLUDE);
            let after = code_changing_configuration_lines(&research, &base, e162_build_script);
            replace_in_file(&copy_root.join(".cargo/config.toml"), GCC_FOURTEEN_INCLUDE, GCC_THIRTEEN_INCLUDE);
            suffix.emit(&format!("E7RESULT name=q13_geometry closure=e162_block_stores closure_has_build_script={e162_build_script} doing_configuration_changed_by_s3={}", before != after));
        }
        Err(message) => suffix.emit(&format!("E7RESULT name=q13_offline_failure closure=e162 message={}", message.replace(' ', "_"))),
    }
}
