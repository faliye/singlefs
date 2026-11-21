//! 开跑之前的三样：crates 摘要、CPU 钟分辨率自检、登记的线程数。

use super::*;

// ---- 开跑之前的三样（停机条款第 1、2 条，开跑自检）----

/// crates 摘要从哪几处起走（从仓根起的路径；目录逐层走到每个文件，文件就是它自己）：装置这份二进制经普通路径依赖链进来的工作区包的 `src/`
/// （产品 GPU 包、`singlefs-mount-runtime`、`singlefs-replay-prefix`、`singlefs-format`、核心层），装置自己的源文件，与它经 `#[path]` 编进来的两份共用模块。
/// 这一趟在计时窗里跑的 `crates/` 下的代码就是这几处，停机条款第 2 条（`crates/` 中途被改）要看得见它们每一处的改动。
/// 验证包不进：它只管按需领卡与判定库，不在计时窗里决定读数；只经它链进来的包同样不进。
/// 包有没有漏，测试 `every_package_linked_into_the_apparatus_by_path_has_its_sources_in_the_crates_digest` 照各包 `Cargo.toml` 的路径依赖核。
const STARTING_PATHS_OF_THE_CRATES_DIGEST: [&str; 8] = [
    "crates/singlefs-background-gpu/src",
    "crates/singlefs-mount-runtime/src",
    "crates/singlefs-replay-prefix/src",
    "crates/singlefs-format/src",
    "crates/singlefs-core/src",
    "crates/singlefs-background-gpu-tests/src/bin/e202_gpu_cpu_gc_comparison",
    "crates/singlefs-background-gpu-tests/tests/common_discrete_gpus_of_this_host/mod.rs",
    "crates/singlefs-background-gpu-tests/tests/common_pairing_input_through_the_library/mod.rs",
];

/// crates 摘要走到的一处是什么：目录（底下各项的名字）还是文件。
enum PathWalkedForTheCratesDigest {
    Directory { entry_names: Vec<String> },
    File,
}

/// [`STARTING_PATHS_OF_THE_CRATES_DIGEST`] 每一处的每个文件（路径相对仓根，按路径排）与它的 sha256；装置从仓根起，仓根是当前目录。
fn source_files_and_digests() -> Result<Vec<(String, String)>, String> {
    source_files_and_digests_walked_with(
        &STARTING_PATHS_OF_THE_CRATES_DIGEST,
        &what_the_path_is_on_the_disk,
        &|path: &str| std::fs::read(path).map_err(|error| format!("读 {path}：{error}")),
    )
}

/// 盘上的这一处（从仓根起的路径）是目录还是文件；目录交回底下各项的名字。
///
/// # Errors
/// 这一处不在、读不了。
fn what_the_path_is_on_the_disk(path: &str) -> Result<PathWalkedForTheCratesDigest, String> {
    let metadata =
        std::fs::metadata(path).map_err(|error| format!("读 {path}（装置要从仓根起）：{error}"))?;
    if !metadata.is_dir() {
        return Ok(PathWalkedForTheCratesDigest::File);
    }
    std::fs::read_dir(path)
        .map_err(|error| format!("读 {path}（装置要从仓根起）：{error}"))?
        .map(|entry_read| {
            entry_read
                .map(|directory_entry| directory_entry.file_name().to_string_lossy().into_owned())
                .map_err(|error| format!("读 {path} 的一项：{error}"))
        })
        .collect::<Result<Vec<String>, String>>()
        .map(|entry_names| PathWalkedForTheCratesDigest::Directory { entry_names })
}

/// 从 `starting_paths` 起走（`what_the_path_is` 答每一处是目录还是文件，`bytes_of_the_file` 交文件的字节）：目录逐层走到每个文件，
/// 文件就是它自己。交回每个文件的路径（按路径的各段排）与它的 sha256。
///
/// # Errors
/// 起走的一处或走到的一处答不出、文件读不了。
fn source_files_and_digests_walked_with(
    starting_paths: &[&str],
    what_the_path_is: &dyn Fn(&str) -> Result<PathWalkedForTheCratesDigest, String>,
    bytes_of_the_file: &dyn Fn(&str) -> Result<Vec<u8>, String>,
) -> Result<Vec<(String, String)>, String> {
    let mut files: Vec<String> = Vec::new();
    let mut paths_to_walk: Vec<String> = starting_paths
        .iter()
        .map(|starting_path| (*starting_path).to_string())
        .collect();
    while let Some(path) = paths_to_walk.pop() {
        match what_the_path_is(&path)? {
            PathWalkedForTheCratesDigest::File => files.push(path),
            PathWalkedForTheCratesDigest::Directory { entry_names } => paths_to_walk.extend(
                entry_names
                    .into_iter()
                    .map(|entry_name| format!("{path}/{entry_name}")),
            ),
        }
    }
    files.sort_by(|left, right| Path::new(left).cmp(Path::new(right)));
    files
        .into_iter()
        .map(|path| {
            let bytes = bytes_of_the_file(&path)?;
            let mut digest = Sha256Digest::new();
            digest.feed(&bytes);
            Ok((path, digest.finish_as_hex()))
        })
        .collect()
}

/// 每个文件的摘要连同路径按次序喂成一个合起来的摘要（`name=crates_version` 的 `combined_sha256`）。
fn combined_digest_of_the_sources(files: &[(String, String)]) -> String {
    let mut digest = Sha256Digest::new();
    for (path, file_digest) in files {
        digest.feed(path.as_bytes());
        digest.feed(&[0]);
        digest.feed(file_digest.as_bytes());
        digest.feed(&[0]);
    }
    digest.finish_as_hex()
}

/// 打 `name=crates_file` 每个文件一行与 `name=crates_version` 一行，交回合起来的摘要。
pub(super) fn emit_the_crates_version(product: &mut ProductWriter) -> Result<String, String> {
    let files = source_files_and_digests()?;
    for (path, digest) in &files {
        product.emit(
            "crates_file",
            &format!("path={} sha256={digest}", field_value(path)),
        );
    }
    let combined = combined_digest_of_the_sources(&files);
    product.emit(
        "crates_version",
        &format!("files={} combined_sha256={combined}", files.len()),
    );
    Ok(combined)
}

/// 开跑之前的三样：常量回比（停机条款第 1 条）、本线程 CPU 钟的分辨率（自检）、读登记的每台默认重活线程数。
/// 交回登记的线程数；常量对不上交回 `Err(EXIT_STOPPED)`，钟或线程数不行交回 `Err(EXIT_RUN_CONDITION_NOT_MET)`。
pub(super) fn checks_before_the_run(product: &mut ProductWriter) -> Result<usize, i32> {
    let mismatches = local_constants_disagreeing_with_the_core();
    for mismatch in &mismatches {
        product.emit(
            "constant_mismatch",
            &format!(
                "constant={} local={} of_the_crates={}",
                mismatch.name, mismatch.local, mismatch.of_the_crates
            ),
        );
    }
    product.emit(
        "constants",
        &format!(
            "compared_against_the_crates=true mismatches={}",
            mismatches.len()
        ),
    );
    if !mismatches.is_empty() {
        return Err(EXIT_STOPPED);
    }
    let advances = advances_of_the_thread_cpu_clock_in_a_millisecond();
    let is_fine_enough = the_thread_cpu_clock_is_fine_enough(advances);
    product.emit(
        "cpu_clock_check",
        &format!("advances_in_a_millisecond={advances} fine_enough={is_fine_enough}"),
    );
    if !is_fine_enough {
        return Err(EXIT_RUN_CONDITION_NOT_MET);
    }
    registered_heavy_threads().map_err(|reason| {
        product.emit(
            "heavy_threads",
            &format!("readable=false reason={}", field_value(&reason)),
        );
        EXIT_RUN_CONDITION_NOT_MET
    })
}

/// 开跑自检的门槛：1 ms 里至少前进 100 次（登记第十一节、第四节第 3a 步第 2 项）。
fn the_thread_cpu_clock_is_fine_enough(advances_in_a_millisecond: u64) -> bool {
    advances_in_a_millisecond >= CPU_CLOCK_ADVANCES_PER_MILLISECOND_AT_LEAST
}

/// 产物里已有的 `name=crates_version` 与这一次合起来的摘要有一份不同（停机条款第 2 条：`crates/` 在同一段的几条命令之间被改了）。
pub(super) fn the_crates_changed_since_the_earlier_commands(
    lines_already_in_the_product: &[KindAndFieldsOfAResultLine],
    combined_digest_now: &str,
) -> bool {
    lines_already_in_the_product
        .iter()
        .filter(|(name, _fields)| name == "crates_version")
        .filter_map(|(_name, fields)| fields.get("combined_sha256"))
        .any(|earlier| earlier != combined_digest_now)
}

/// 登记的每台默认重活线程数（`.claude/heavy-thread-default` 第一条非注释行的那个正整数；登记第十四节「线程」）。
pub(super) fn registered_heavy_threads() -> Result<usize, String> {
    let text = std::fs::read_to_string(".claude/heavy-thread-default")
        .map_err(|error| format!("读 .claude/heavy-thread-default（装置要从仓根起）：{error}"))?;
    text.lines()
        .map(str::trim)
        .find(|line| !line.is_empty() && !line.starts_with('#'))
        .and_then(|line| line.split_whitespace().next())
        .and_then(|number| number.parse::<usize>().ok())
        .filter(|threads| *threads > 0)
        .ok_or_else(|| ".claude/heavy-thread-default 第一条非注释行不是正整数".to_string())
}

#[cfg(test)]
mod tests;
