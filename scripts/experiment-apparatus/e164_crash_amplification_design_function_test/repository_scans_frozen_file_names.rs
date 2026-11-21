//! E164 W3：登记时冻结文件名在全部 git 历史上撞不撞（跑前登记 5.7「W3」）。只读 git。

use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

use crate::repository_scans::{git_text, ProductSuffix};

struct RenameEvent {
    commit: String,
    commit_time: u64,
    old_path: String,
    new_path: String,
}

/// `git log --all -M --name-status --diff-filter=<过滤> --format=commit:%H:%ct` 按提交拆成（提交，时刻，名字状态行）。
fn name_status_by_commit(root: &Path, diff_filter: &str) -> Vec<(String, u64, Vec<Vec<String>>)> {
    let filter_argument = format!("--diff-filter={diff_filter}");
    let text = git_text(root, &["log", "--all", "-M", "--name-status", &filter_argument, "--format=commit:%H:%ct"]);
    let mut commits: Vec<(String, u64, Vec<Vec<String>>)> = Vec::new();
    for line in text.lines() {
        if let Some(rest) = line.strip_prefix("commit:") {
            let (hash, time) = rest.split_once(':').expect("commit:<哈希>:<时刻>");
            commits.push((hash.to_string(), time.parse().expect("时刻是数"), Vec::new()));
        } else if !line.is_empty() {
            let fields: Vec<String> = line.split('\t').map(str::to_string).collect();
            commits.last_mut().expect("名字状态行前面有提交行").2.push(fields);
        }
    }
    commits
}

fn has_test_attribute_at(root: &Path, commit: &str, path: &str) -> bool {
    if !path.ends_with(".rs") {
        return false;
    }
    let specification = format!("{commit}:{path}");
    git_text(root, &["show", &specification]).contains("#[test]")
}

/// 一次提交的树里 `#[test]` 函数：（仓内路径，函数名）。取 `#[test]` 之后同一份文件里第一行带 `fn ` 的。
fn test_functions_at(root: &Path, commit: &str) -> Vec<(String, String)> {
    let arguments = crate::common::strings(&["grep", "-z", "-n", "-A", "6", "-e", "#[test]", "-F", commit, "--", "*.rs"]);
    let output = crate::common::run_allowed("git", &arguments, root, &[]);
    let no_match = output.status.code() == Some(1) && output.stderr.is_empty();
    assert!(output.status.success() || no_match, "git grep 在 {commit} 上失败：{}", String::from_utf8_lossy(&output.stderr));
    let text = String::from_utf8_lossy(&output.stdout).into_owned();
    let mut functions = Vec::new();
    let mut waiting_file: Option<String> = None;
    for line in text.lines() {
        if line == "--" {
            continue;
        }
        let without_commit = line.strip_prefix(commit).and_then(|rest| rest.strip_prefix(':')).unwrap_or(line);
        let mut parts = without_commit.splitn(3, '\0');
        let (Some(path), Some(_line_number), Some(content)) = (parts.next(), parts.next(), parts.next()) else { continue };
        if content.trim_start().starts_with("#[test]") {
            waiting_file = Some(path.to_string());
            continue;
        }
        if waiting_file.as_deref() == Some(path) {
            if let Some(after) = content.split("fn ").nth(1) {
                let name: String = after.chars().take_while(|character| character.is_alphanumeric() || *character == '_').collect();
                if !name.is_empty() {
                    functions.push((path.to_string(), name));
                    waiting_file = None;
                }
            }
        }
    }
    functions
}

/// W3 的域：全部引用可达的提交、全部路径。
pub fn all_commits(root: &Path) -> Vec<String> {
    git_text(root, &["rev-list", "--all"]).lines().map(str::to_string).collect()
}

pub fn scan_frozen_file_names(root: &Path, suffix: &ProductSuffix) {
    let commits = all_commits(root);
    let mut renames = Vec::new();
    for (commit, commit_time, entries) in name_status_by_commit(root, "R") {
        for fields in entries {
            if fields.first().is_some_and(|status| status.starts_with('R')) && fields.len() == 3 {
                renames.push(RenameEvent { commit: commit.clone(), commit_time, old_path: fields[1].clone(), new_path: fields[2].clone() });
            }
        }
    }
    let mut creations: Vec<(String, u64, String, Option<String>)> = Vec::new();
    for (commit, commit_time, entries) in name_status_by_commit(root, "AR") {
        for fields in entries {
            match (fields.first().map(String::as_str), fields.len()) {
                (Some("A"), 2) => creations.push((commit.clone(), commit_time, fields[1].clone(), None)),
                (Some(status), 3) if status.starts_with('R') => creations.push((commit.clone(), commit_time, fields[2].clone(), Some(fields[1].clone()))),
                _ => {}
            }
        }
    }
    let mut reoccupied_all = 0usize;
    let mut reoccupied_test_files = 0usize;
    let mut test_file_renames = 0usize;
    for rename in &renames {
        let is_test_file = has_test_attribute_at(root, &rename.commit, &rename.new_path);
        test_file_renames += usize::from(is_test_file);
        let reoccupied = creations.iter().any(|(commit, commit_time, path, source)| {
            *commit != rename.commit
                && *commit_time > rename.commit_time
                && *path == rename.old_path
                && source.as_deref() != Some(rename.new_path.as_str())
        });
        if reoccupied {
            reoccupied_all += 1;
            reoccupied_test_files += usize::from(is_test_file);
            suffix.emit(&format!("E7RESULT name=w3_reoccupied commit={} old={} new={} test_file={is_test_file}", rename.commit, rename.old_path, rename.new_path));
        }
    }
    suffix.emit(&format!(
        "E7RESULT name=w3_scan commits={} renames={} test_file_renames={test_file_renames}",
        commits.len(),
        renames.len()
    ));
    suffix.emit(&format!("E7RESULT name=w3_reoccupied_summary all_paths={reoccupied_all} test_rs_files={reoccupied_test_files}"));
    let mut commits_with_duplicate_by_file_name = 0usize;
    let mut commits_with_duplicate_by_path = 0usize;
    let mut most_duplicates_by_file_name = 0usize;
    let mut most_duplicates_by_path = 0usize;
    let mut example_by_path: Option<String> = None;
    let mut distinct_duplicates_by_path: BTreeSet<String> = BTreeSet::new();
    for commit in &commits {
        let functions = test_functions_at(root, commit);
        let mut by_file_name: BTreeMap<String, usize> = BTreeMap::new();
        let mut by_path: BTreeMap<String, usize> = BTreeMap::new();
        for (path, name) in &functions {
            let file_name = path.rsplit('/').next().unwrap_or(path);
            *by_file_name.entry(format!("{file_name}::{name}")).or_default() += 1;
            *by_path.entry(format!("{path}::{name}")).or_default() += 1;
        }
        let duplicated_file_names = by_file_name.values().filter(|count| **count > 1).count();
        let duplicated_paths: BTreeSet<&String> = by_path.iter().filter(|(_, count)| **count > 1).map(|(identity, _)| identity).collect();
        commits_with_duplicate_by_file_name += usize::from(duplicated_file_names > 0);
        commits_with_duplicate_by_path += usize::from(!duplicated_paths.is_empty());
        most_duplicates_by_file_name = most_duplicates_by_file_name.max(duplicated_file_names);
        most_duplicates_by_path = most_duplicates_by_path.max(duplicated_paths.len());
        distinct_duplicates_by_path.extend(duplicated_paths.iter().map(|identity| (*identity).clone()));
        if example_by_path.is_none() {
            example_by_path = duplicated_paths.iter().next().map(|identity| format!("{commit}:{identity}"));
        }
    }
    suffix.emit(&format!(
        "E7RESULT name=w3_uniqueness commits_scanned={} commits_with_duplicate_by_file_name={commits_with_duplicate_by_file_name} most_duplicates_in_one_commit_by_file_name={most_duplicates_by_file_name} commits_with_duplicate_by_path={commits_with_duplicate_by_path} most_duplicates_in_one_commit_by_path={most_duplicates_by_path} first_duplicate_by_path={}",
        commits.len(),
        example_by_path.unwrap_or_else(|| "none".to_string())
    ));
    for identity in &distinct_duplicates_by_path {
        suffix.emit(&format!("E7RESULT name=w3_duplicate_identity_by_path identity={identity}"));
    }
    suffix.emit(&format!("E7RESULT name=w3_duplicate_identities_by_path distinct={}", distinct_duplicates_by_path.len()));
}
