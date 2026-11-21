//! E164 第二段：仓副本、施加变体（跑前登记 5.3）、验证包探针与点名测试，读回录制流。

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use crate::common::{run_allowed, sha256_hexadecimal, strings};
use crate::fingerprints::copy_tree;
use crate::lexical::combined_digest;
use crate::repository_scans::files_under;
use crate::write_tables::{OperationKind, RecordedOperation};

pub const PROBE_SOURCE: &str = include_str!("probe_recording.rs");
const VARIANT_PATCHES: &str = include_str!("variant_patches.tsv");
pub const PROBE_TEST_FILE: &str = "crates/singlefs-verification/tests/e164_probe.rs";

/// 变体表：名字 → 怎么施加（`crates/mutations.tsv` 的变异名，或 `variant_patches.tsv` 的一行，或不改）。
pub fn variant_sources() -> Vec<(&'static str, Option<VariantSource>)> {
    vec![
        ("V0", None),
        ("V0b", None),
        ("V1", Some(VariantSource::MutationRow("步 3：记录与根之间少一道屏障（实二二三起三条发布路径共用 persist_publish_writes，零单元发布那一段也少了这一道）"))),
        ("V2", Some(VariantSource::Patch("V2"))),
        ("V3", Some(VariantSource::MutationRow("步 3：前缀跨实例边界（把别的实例的记录也接上）"))),
        ("V4", Some(VariantSource::Patch("V4"))),
        ("V5", Some(VariantSource::MutationRow("记录头 311：池级 checker 的载荷校验和偏移没跟着后挪 4 字节"))),
        ("V6", Some(VariantSource::Patch("V6"))),
        ("V6b", Some(VariantSource::Patch("V6b"))),
        ("V7", Some(VariantSource::Patch("V7"))),
        ("V8", Some(VariantSource::Patch("V8"))),
        ("V11", Some(VariantSource::MutationRow("格式常量字面量改掉一个数：DATA_POINTER_BYTES"))),
    ]
}

#[derive(Clone, Copy)]
pub enum VariantSource {
    MutationRow(&'static str),
    Patch(&'static str),
}

pub struct Replacement {
    pub file: String,
    pub original: String,
    pub replacement: String,
}

fn unescape(text: &str) -> String {
    text.replace("\\n", "\n")
}

pub fn replacement_of(root: &Path, source: VariantSource) -> Replacement {
    let (table, key) = match source {
        VariantSource::MutationRow(name) => (std::fs::read_to_string(root.join("crates/mutations.tsv")).expect("读得了 crates/mutations.tsv"), name),
        VariantSource::Patch(name) => (VARIANT_PATCHES.to_string(), name),
    };
    let row = table
        .lines()
        .filter(|line| !line.starts_with('#'))
        .map(|line| line.split('\t').collect::<Vec<&str>>())
        .find(|columns| columns[0] == key)
        .unwrap_or_else(|| stop(&format!("变体 {key} 在表里找不到（S-anchor）")));
    Replacement { file: row[1].to_string(), original: unescape(row[2]), replacement: unescape(row[3]) }
}

pub fn stop(reason: &str) -> ! {
    eprintln!("  ✗ 停机：{reason}");
    eprintln!("  → 怎么办：两边都查（跑前登记第十一节），装置与被测代码都不默认对；写进报告交主 agent");
    std::process::exit(4)
}

/// 副本 `crates/` 的原文摘要；`replacement` 给了就在内存里把它施加到主仓原文上再算（「原样 + 这一个变体」）。
pub fn crates_raw_digest_of(root: &Path, replacement: Option<&Replacement>) -> String {
    let entries: Vec<(String, String)> = files_under(root, "crates")
        .into_iter()
        .filter(|path| path != PROBE_TEST_FILE)
        .map(|path| {
            let mut bytes = std::fs::read(root.join(&path)).expect("读得了文件");
            if let Some(change) = replacement.filter(|change| change.file == path) {
                let text = String::from_utf8(bytes).expect("源码是 UTF-8");
                bytes = text.replacen(&change.original, &change.replacement, 1).into_bytes();
            }
            (path, sha256_hexadecimal(&bytes))
        })
        .collect();
    combined_digest(&entries)
}

/// 建一份变体副本：拷 `crates/`、`Cargo.toml`、`Cargo.lock`、`.cargo`（不带 .git、不带 target、不带旧修改时刻），施加变体，放探针。
pub fn prepare_copy(root: &Path, directory: &Path, replacement: Option<&Replacement>) {
    if directory.exists() {
        std::fs::remove_dir_all(directory).expect("清得掉上一趟的副本");
    }
    std::fs::create_dir_all(directory).expect("建得了副本目录");
    copy_tree(&root.join("crates"), &directory.join("crates"));
    copy_tree(&root.join(".cargo"), &directory.join(".cargo"));
    for file in ["Cargo.toml", "Cargo.lock"] {
        std::fs::copy(root.join(file), directory.join(file)).expect("拷得了工作区文件");
    }
    if let Some(change) = replacement {
        let path = directory.join(&change.file);
        let text = std::fs::read_to_string(&path).expect("读得了要改的文件");
        if text.matches(&change.original).count() != 1 {
            stop(&format!("{} 里变体原文不是恰好命中一次（S-anchor）", change.file));
        }
        std::fs::write(&path, text.replacen(&change.original, &change.replacement, 1)).expect("写得回去");
    }
    let expected = crates_raw_digest_of(root, replacement);
    let actual = crates_raw_digest_of(directory, None);
    if expected != actual {
        eprintln!("  ✗ 副本 crates/ 的原文摘要不等于「原样 + 这一个变体」（V-build）");
        eprintln!("  → 怎么办：这一格作废；查副本是不是拷全了、变体是不是施加对了");
        std::process::exit(5);
    }
    for forbidden in ["CrashImage", "enumerate_layer0", "check_pool_image"] {
        if PROBE_SOURCE.contains(forbidden) {
            eprintln!("  ✗ 探针源码里出现 {forbidden}（V-tier）");
            eprintln!("  → 怎么办：探针只录制，不造崩溃状态；改探针");
            std::process::exit(5);
        }
    }
    std::fs::write(directory.join(PROBE_TEST_FILE), PROBE_SOURCE).expect("放得下探针");
}

pub struct TestRun {
    pub exit_code: i32,
    pub stdout: String,
}

pub fn cargo_test(directory: &Path, arguments: &[&str]) -> TestRun {
    let mut full = strings(&["test", "--offline"]);
    full.extend(strings(arguments));
    let target_directory = directory.join("target").display().to_string();
    let output = run_allowed("cargo", &full, directory, &[("CARGO_TARGET_DIR".to_string(), Some(target_directory))]);
    TestRun { exit_code: output.status.code().unwrap_or(-1), stdout: String::from_utf8_lossy(&output.stdout).into_owned() }
}

/// 探针读回的一条历史。
#[derive(Default)]
pub struct ProbeHistory {
    pub steps: Vec<(String, usize, usize)>,
    pub operations: Vec<RecordedOperation>,
    pub mkfs_sectors: BTreeMap<(u32, u64), String>,
    pub harness_segments: BTreeMap<String, (String, String)>,
}

fn result_fields(line: &str) -> Option<BTreeMap<String, String>> {
    line.find("E7RESULT ").map(|start| crate::common::parse_result_fields(&line[start..]))
}

/// 从探针输出里读一条历史（`probe` 取 h_short 或 h_long）；完整性闸 V-count 不过就交回 Err。
pub fn parse_history(stdout: &str, probe: &str) -> Result<ProbeHistory, String> {
    let mut history = ProbeHistory::default();
    let mut counted_lines = 0usize;
    let mut end_fields: Option<BTreeMap<String, String>> = None;
    for fields in stdout.lines().filter_map(result_fields).filter(|fields| fields.get("probe").map(String::as_str) == Some(probe)) {
        let field = |name: &str| fields.get(name).cloned().unwrap_or_default();
        match field("name").as_str() {
            "step" => history.steps.push((field("step"), field("start").parse().expect("start"), field("end").parse().expect("end"))),
            "op" => {
                let kind = OperationKind::from_serialized_name(&field("kind")).expect("认得的种类");
                let sectors = field("sectors");
                history.operations.push(RecordedOperation {
                    device: field("device").parse().expect("device"),
                    kind,
                    offset_in_bytes: field("offset").parse().expect("offset"),
                    length_in_bytes: field("length").parse().expect("length"),
                    content_sha256: field("sha256"),
                    sector_sha256s: if sectors == "none" { Vec::new() } else { sectors.split(',').map(str::to_string).collect() },
                });
            }
            "mkfs_sector" => {
                history.mkfs_sectors.insert((field("device").parse().expect("device"), field("sector").parse().expect("sector")), field("sha256"));
            }
            "harness_segments" => {
                history.harness_segments.insert(field("step"), (field("sizes"), field("closed_form")));
            }
            "probe_end" => {
                end_fields = Some(fields.clone());
                continue;
            }
            _ => {}
        }
        counted_lines += 1;
    }
    let end = end_fields.ok_or_else(|| format!("{probe} 没有 probe_end 行"))?;
    let emitted: usize = end["emitted_before_this_line"].parse().expect("emitted");
    let operations: usize = end["operations"].parse().expect("operations");
    if emitted != counted_lines || operations != history.operations.len() {
        return Err(format!("{probe} 完整性闸：读到 {counted_lines} 行 / {} 条操作，探针报 {emitted} 行 / {operations} 条（V-count）", history.operations.len()));
    }
    Ok(history)
}

pub fn variant_directory(scratch: &Path, name: &str) -> PathBuf {
    scratch.join("variants").join(name)
}

pub fn command_variants(scratch: &Path) {
    if scratch.as_os_str().is_empty() {
        eprintln!("  ✗ variants 要一个草稿目录参数");
        eprintln!("  → 怎么办：variants <草稿目录>，放得下十二份 crates/ 副本与各自的编译目录");
        std::process::exit(2);
    }
    crate::arms::run_segment_two(&crate::repository_root(), scratch);
}
