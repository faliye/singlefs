//! E164 W5：节点代码按 crate 闭包取时，一次改动让多少节点重跑（跑前登记 5.7「W5」）。只读 git 与 `cargo metadata --offline`。

use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

use crate::repository_scans::{dependency_graph, git_text, DependencyGraph, ProductSuffix};

/// 闭包算不算 dev-dependencies：节点是测试目标，dev 依赖进它的编译。只给变异 M22 翻。
const CLOSURE_INCLUDES_DEVELOPMENT_DEPENDENCIES: bool = true;

struct Node {
    name: String,
    is_crash_enumeration: bool,
    package: String,
    /// 节点自己的目标文件（仓内路径）；库节点没有单独的目标文件，它就是 `src/`。
    target_file: Option<String>,
}

fn package_directory(graph: &DependencyGraph, root: &Path, name: &str) -> String {
    let identifier = graph.identifier_of(name).unwrap_or_else(|| panic!("{name} 是工作区成员"));
    graph.packages[&identifier].manifest_directory.strip_prefix(root).expect("工作区成员在仓里").display().to_string()
}

fn nodes_of(root: &Path, graph: &DependencyGraph) -> Vec<Node> {
    let mut nodes = Vec::new();
    let stage_inputs = std::fs::read_to_string(root.join(".claude/gate.d/stage-inputs.tsv")).expect("读得了 stage-inputs.tsv");
    for line in stage_inputs.lines().filter(|line| line.starts_with("crash-case:")) {
        let columns: Vec<&str> = line.split('\t').collect();
        let test_column = columns.iter().find_map(|column| column.strip_prefix("test=")).expect("crash-case 行有 test= 列");
        let mut parts = test_column.split(':');
        let package = parts.next().expect("test= 的包").to_string();
        let target = parts.next().expect("test= 的测试目标").to_string();
        let directory = package_directory(graph, root, &package);
        nodes.push(Node {
            name: columns[0].to_string(),
            is_crash_enumeration: true,
            target_file: Some(format!("{directory}/tests/{target}.rs")),
            package,
        });
    }
    for package in ["singlefs-verification", "singlefs-checker", "singlefs-core", "singlefs-format"] {
        let directory = package_directory(graph, root, package);
        nodes.push(Node { name: format!("{package}:--lib"), is_crash_enumeration: false, package: package.to_string(), target_file: None });
        let mut test_files: Vec<String> = std::fs::read_dir(root.join(&directory).join("tests"))
            .map(|entries| {
                entries
                    .flatten()
                    .filter(|entry| entry.file_type().is_ok_and(|kind| kind.is_file()))
                    .map(|entry| entry.file_name().to_string_lossy().into_owned())
                    .filter(|name| name.ends_with(".rs"))
                    .collect()
            })
            .unwrap_or_default();
        test_files.sort();
        for file in test_files {
            nodes.push(Node {
                name: format!("{package}:{}", file.trim_end_matches(".rs")),
                is_crash_enumeration: false,
                package: package.to_string(),
                target_file: Some(format!("{directory}/tests/{file}")),
            });
        }
    }
    nodes
}

/// 节点要重跑当且仅当它的闭包里有包的 `src/` 被改了，或这次提交改了它自己的目标文件。
pub fn node_reruns(closure: &BTreeSet<String>, touched_packages: &BTreeSet<String>, target_file: Option<&str>, changed_files: &[String]) -> bool {
    !closure.is_disjoint(touched_packages) || target_file.is_some_and(|file| changed_files.iter().any(|changed| changed == file))
}

fn commit_class(touched: &BTreeSet<String>) -> String {
    match touched.iter().collect::<Vec<&String>>().as_slice() {
        [only] => only.trim_start_matches("singlefs-").to_string(),
        _ => "multi".to_string(),
    }
}

fn ratio_summary(values: &mut [f64]) -> String {
    if values.is_empty() {
        return "none".to_string();
    }
    values.sort_by(|left, right| left.partial_cmp(right).expect("比例不是 NaN"));
    let median = values[values.len() / 2];
    format!("{:.3}/{median:.3}/{:.3}", values[0], values[values.len() - 1])
}

pub fn scan_rerun_surface(root: &Path, suffix: &ProductSuffix) {
    let graph = match dependency_graph(root, &[]) {
        Ok(graph) => graph,
        Err(message) => {
            suffix.emit(&format!("E7RESULT name=w5_offline_failure message={}", message.replace(' ', "_")));
            return;
        }
    };
    let nodes = nodes_of(root, &graph);
    let directory_to_package: BTreeMap<String, String> = graph
        .packages
        .values()
        .filter(|package| package.is_workspace_member)
        .filter_map(|package| package.manifest_directory.strip_prefix(root).ok().map(|directory| (directory.display().to_string(), package.name.clone())))
        .collect();
    let closures: Vec<BTreeSet<String>> = nodes
        .iter()
        .map(|node| {
            let identifier = graph.identifier_of(&node.package).expect("节点的包是工作区成员");
            graph.member_names(&graph.closure(&identifier, CLOSURE_INCLUDES_DEVELOPMENT_DEPENDENCIES))
        })
        .collect();
    let nodes_whose_closure_needs_development_dependencies = nodes
        .iter()
        .zip(&closures)
        .filter(|(node, closure)| {
            let identifier = graph.identifier_of(&node.package).expect("节点的包是工作区成员");
            graph.member_names(&graph.closure(&identifier, false)) != **closure
        })
        .count();
    let crash_total = nodes.iter().filter(|node| node.is_crash_enumeration).count();
    let test_total = nodes.len() - crash_total;
    for (node, closure) in nodes.iter().zip(&closures) {
        suffix.emit(&format!("E7RESULT name=w5_node node={} crash_enumeration={} closure={}", node.name, node.is_crash_enumeration, closure.iter().cloned().collect::<Vec<String>>().join(",")));
    }
    suffix.emit(&format!("E7RESULT name=w5_nodes total={} crash_enumeration={crash_total} test={test_total} nodes_whose_closure_needs_development_dependencies={nodes_whose_closure_needs_development_dependencies}", nodes.len()));
    for window in [10usize, 30] {
        let count_argument = format!("-n{window}");
        let commits: Vec<String> = git_text(root, &["log", &count_argument, "--format=%H", "--", "crates/*/src/*"]).lines().map(str::to_string).collect();
        let mut by_class: BTreeMap<String, (Vec<f64>, Vec<f64>)> = BTreeMap::new();
        for commit in &commits {
            let changed: Vec<String> = git_text(root, &["show", "--name-only", "--format=", commit]).lines().filter(|line| !line.is_empty()).map(str::to_string).collect();
            let touched: BTreeSet<String> = changed
                .iter()
                .filter_map(|path| {
                    directory_to_package.iter().find(|(directory, _)| path.starts_with(&format!("{directory}/src/"))).map(|(_, package)| package.clone())
                })
                .collect();
            let mut crash_rerun = 0usize;
            let mut test_rerun = 0usize;
            for (node, closure) in nodes.iter().zip(&closures) {
                let reruns = node_reruns(closure, &touched, node.target_file.as_deref(), &changed);
                if reruns {
                    if node.is_crash_enumeration {
                        crash_rerun += 1;
                    } else {
                        test_rerun += 1;
                    }
                }
            }
            let class = commit_class(&touched);
            let crash_ratio = crash_rerun as f64 / crash_total.max(1) as f64;
            let test_ratio = test_rerun as f64 / test_total.max(1) as f64;
            let entry = by_class.entry(class.clone()).or_default();
            entry.0.push(crash_ratio);
            entry.1.push(test_ratio);
            if window == 30 {
                suffix.emit(&format!(
                    "E7RESULT name=w5_commit commit={commit} class={class} touched_packages={} crash_enumeration_rerun={crash_rerun}/{crash_total} test_rerun={test_rerun}/{test_total}",
                    touched.iter().cloned().collect::<Vec<String>>().join(",")
                ));
            }
        }
        for (class, (crash_ratios, test_ratios)) in &mut by_class {
            suffix.emit(&format!(
                "E7RESULT name=w5_class_summary window={window} commits_in_window={} class={class} commits={} crash_enumeration_rerun_ratio_min_median_max={} test_rerun_ratio_min_median_max={}",
                commits.len(),
                crash_ratios.len(),
                ratio_summary(crash_ratios),
                ratio_summary(test_ratios)
            ));
        }
    }
}
