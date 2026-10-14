//! 编译时算三种结论的代码摘要表（崩溃点重放合并第三轮判决 J8、第二轮判决 L3-1）：GPU 判器一行，`tests/` 下每份候选文件各一行
//! CPU 判器（连同它当设点文件）与一行观察者代码，写进 `OUT_DIR` 里的表，库经 `include_str!(concat!(env!("OUT_DIR"), "/…"))` 读回；
//! 跑时库从盘上再算一份比（`crash_identity::conclusion_version_checked_against_the_build`），不等由调用方整趟不写库、退 78。
//! 算法只有一份，在 `src/judging_closure.rs`，这里经 `#[path]` 引；摘要经 `#[path]` 引 harness 的 SHA-256 与十六进制两份源文件。

#[path = "../singlefs-harness/src/hexadecimal.rs"]
mod hexadecimal;
#[allow(
    dead_code,
    reason = "build.rs 只算结论代码摘要表；今天的判法摘要、闭包明细与表的读回是库里用的"
)]
#[path = "src/judging_closure.rs"]
mod judging_closure;
#[allow(
    dead_code,
    reason = "build.rs 只用 sha256_digest 与 DIGEST_BYTES，sha256_hexadecimal 是库里用的"
)]
#[path = "../singlefs-harness/src/sha256.rs"]
mod sha256;

use std::path::{Path, PathBuf};

use hexadecimal::hexadecimal_text;
use sha256::{sha256_digest, DIGEST_BYTES};

fn main() {
    let manifest_directory = PathBuf::from(
        std::env::var_os("CARGO_MANIFEST_DIR").expect("cargo 起 build.rs 时设 CARGO_MANIFEST_DIR"),
    );
    // 包目录往上两层就是仓根（crates/<包>）：用 parent() 取，不写往上的相对路径字面量（拷贝范围判的是那种字面量）
    let repository_root = manifest_directory
        .parent()
        .and_then(Path::parent)
        .expect("包目录在 crates/ 下，往上两层是仓根")
        .to_path_buf();
    let output_directory =
        PathBuf::from(std::env::var_os("OUT_DIR").expect("cargo 起 build.rs 时设 OUT_DIR"));
    let mut reader = judging_closure::ClosureTreeReader::new(&repository_root)
        .unwrap_or_else(|error| panic!("读不了这棵树的 crate 清单：{}", error.reason));
    let table = judging_closure::ConclusionCodeDigestTable::of_the_tree(&mut reader)
        .unwrap_or_else(|error| panic!("算不出结论代码摘要表：{}", error.reason));
    let table_path = output_directory.join(judging_closure::CONCLUSION_CODE_DIGEST_TABLE_FILE_NAME);
    std::fs::write(&table_path, table.to_table_text())
        .unwrap_or_else(|error| panic!("写不了 {}：{error}", table_path.display()));
    for watched in watched_paths(&repository_root, &reader) {
        println!("cargo:rerun-if-changed={}", watched.display());
    }
}

/// 改了哪些就要重算：仓根的 `Cargo.lock` 与 `Cargo.toml`、每个 crate 的 `Cargo.toml` 与 `src/`（`impl` 头的扫描与模块路径都在这里落）、
/// 候选目录（加一份测试文件就多一行）、这一趟读过的每份文件与列过的每个目录（经 `#[path]`、`include_str!` 读到别处的也在里面）。
fn watched_paths(
    repository_root: &Path,
    reader: &judging_closure::ClosureTreeReader,
) -> Vec<PathBuf> {
    let mut watched: Vec<String> = vec![
        "Cargo.lock".to_string(),
        "Cargo.toml".to_string(),
        judging_closure::CONCLUSION_CANDIDATE_DIRECTORY.to_string(),
        judging_closure::CONCLUSION_CANDIDATE_BINARY_DIRECTORY.to_string(),
    ];
    watched.extend(
        judging_closure::CONCLUSION_CANDIDATE_LIBRARY_FILES
            .iter()
            .map(|file| (*file).to_string()),
    );
    for crate_directory in reader.crate_directories() {
        watched.push(format!("{crate_directory}/Cargo.toml"));
        watched.push(format!("{crate_directory}/src"));
    }
    watched.extend(reader.directories_listed());
    watched.extend(reader.files_read());
    watched.sort();
    watched.dedup();
    watched
        .into_iter()
        .map(|relative| repository_root.join(relative))
        .collect()
}
