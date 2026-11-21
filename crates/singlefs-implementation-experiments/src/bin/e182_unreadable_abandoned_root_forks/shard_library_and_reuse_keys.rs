use super::*;

// ============================================================================
// 十二、片结果库与复用键：键 = 判法入口按 `singlefs-unit-graph` 算的函数单元闭包指纹（去注释）+ 工具链行 + 历史名 + 段号 + 展开方式。
//    一片判完才原子落盘（同目录写临时文件再改名）；被打断后用同一条命令重起，键相同的片直接取库里的，只补没判的。
// ============================================================================

/// 判法入口（闭包的根）：登记第五节「装置源码」那一半由这两个函数的闭包代替，不再用文件级 sha256。
const JUDGING_ENTRY_FUNCTIONS: [&str; 2] = [
    "singlefs_implementation_experiments::record_the_history",
    "singlefs_implementation_experiments::judge_the_shard",
];
const THIS_BINARY_TARGET: &str = "bin:e182_unreadable_abandoned_root_forks";
const THREADS_ENVIRONMENT_VARIABLE: &str = "E182_THREADS";
pub(super) const LIBRARY_ENVIRONMENT_VARIABLE: &str = "E182_VERDICT_LIBRARY";
const SHARD_SHARE_ENVIRONMENT_VARIABLE: &str = "E182_SHARD_SHARE";
const REPOSITORY_ROOT_ENVIRONMENT_VARIABLE: &str = "E182_REPOSITORY_ROOT";

/// 闭包的根：组名（只当索引、不进键）、编译目标（`TargetSelector::parse` 认的写法）与这个目标里的入口 fn（头一个定 crate）。
struct ClosureEntries {
    group_name: &'static str,
    target: &'static str,
    entry_functions: &'static [&'static str],
}

/// 装置跑时取指纹的那一组：本文件两个判法入口。
const JUDGING_ENTRIES: ClosureEntries = ClosureEntries {
    group_name: "e182_single_device_judgement",
    target: THIS_BINARY_TARGET,
    entry_functions: &JUDGING_ENTRY_FUNCTIONS,
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct ClosureFingerprint {
    pub(super) key_hex: String,
    pub(super) toolchain_lines: Vec<String>,
}

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

fn repository_root() -> std::path::PathBuf {
    match std::env::var(REPOSITORY_ROOT_ENVIRONMENT_VARIABLE) {
        Ok(root) if !root.is_empty() => std::path::PathBuf::from(root),
        Ok(_) | Err(_) => std::env::current_dir().expect("当前目录读得出（从仓根起装置）"),
    }
}

pub(super) fn worker_threads() -> usize {
    own_variable_or_default_heavy_threads(
        THREADS_ENVIRONMENT_VARIABLE,
        std::env::var(THREADS_ENVIRONMENT_VARIABLE).ok().as_deref(),
        std::env::var("SINGLEFS_THREAD_CAP").ok().as_deref(),
    )
    .get()
}

/// 本文件两个判法入口的函数单元闭包指纹（去注释）与这一趟的工具链行。闭包里有拒算时交 `Err`（调用方这一趟不复用、不入库）。
pub(super) fn closure_fingerprint() -> Result<ClosureFingerprint, String> {
    closure_fingerprint_of_the_tree(&repository_root(), &JUDGING_ENTRIES)
}

/// 在 `root` 那棵树上算 `entries` 的闭包指纹，依赖包的特性照这一次编译（编进库的判器键表里 `build-feature` 那几行：验证包构建脚本读依赖包经
/// `links` 交出的那一份）。编实验包的测试时，实验包的 dev 依赖替核心层并上 `lookup-work-counters`，这个二进制链进来的
/// 核心层编的是带计数的那一份；只照本包特性算，键就是没并上时那一把，与编出来的代码对不上（D13（验证路线） 已定项 17）。
fn closure_fingerprint_of_the_tree(
    root: &std::path::Path,
    entries: &ClosureEntries,
) -> Result<ClosureFingerprint, String> {
    closure_fingerprint_with_the_features_of_the_build(
        root,
        entries,
        &singlefs_verification::crash_identity::compiled_judging_units_keys()
            .features_enabled_by_the_build,
    )
}

/// 在 `root` 那棵树上、照给定的依赖包特性算 `entries` 的闭包指纹（[`closure_fingerprint_of_the_tree`] 交编进来的那一份）。
/// 闭包的键不带工具链行与 `Cargo.lock` 的依赖行（`singlefs_unit_graph::entry_groups`）；工具链行另记在指纹里（编译档不读、不进键）。
fn closure_fingerprint_with_the_features_of_the_build(
    root: &std::path::Path,
    entries: &ClosureEntries,
    features_enabled_by_the_build: &singlefs_unit_graph::FeaturesEnabledByTheBuild,
) -> Result<ClosureFingerprint, String> {
    use singlefs_unit_graph::build_toolchain::{
        toolchain_lines_of_the_build, BuildToolchainInputs,
    };
    use singlefs_unit_graph::entry_groups::{
        entry_group_key_with_the_features_of_the_build, EntryGroupRequest, EntryGroupVerdict,
    };
    use singlefs_unit_graph::{load_workspace, RuleSwitches, TargetSelector};
    let switches = RuleSwitches::every_rule_intact();
    let threads = std::num::NonZeroUsize::new(worker_threads()).expect("线程数是正数");
    let workspace = load_workspace(root, threads, &switches)
        .map_err(|error| format!("装不起工作区：{error}"))?;
    let rustc = std::env::var("RUSTC").unwrap_or_else(|_| "rustc".to_string());
    let cargo = std::env::var("CARGO").unwrap_or_else(|_| "cargo".to_string());
    let rustc_version_text = output_of(&rustc, "-vV")?;
    let cargo_version_text = output_of(&cargo, "-V")?;
    let target_triple = rustc_version_text
        .lines()
        .find_map(|line| line.strip_prefix("host: "))
        .map(str::trim)
        .ok_or_else(|| "rustc -vV 里没有 host: 那一行".to_string())?
        .to_string();
    let toolchain_lines = toolchain_lines_of_the_build(&BuildToolchainInputs {
        rustc_version_text: &rustc_version_text,
        cargo_version_text: &cargo_version_text,
        target_triple: &target_triple,
        encoded_rustflags: &std::env::var("CARGO_ENCODED_RUSTFLAGS").unwrap_or_default(),
    });
    let request = EntryGroupRequest {
        group_name: entries.group_name.to_string(),
        target: TargetSelector::parse(entries.target)
            .map_err(|text| format!("编译目标写法认不得：{text}"))?,
        entry_paths: entries
            .entry_functions
            .iter()
            .map(|entry| (*entry).to_string())
            .collect(),
        features: Vec::new(),
    };
    let group = entry_group_key_with_the_features_of_the_build(
        &workspace,
        &request,
        features_enabled_by_the_build,
        &target_triple,
        &switches,
    )
    .map_err(|error| format!("入口写错了：{error}"))?;
    match group.verdict {
        EntryGroupVerdict::Keyed { key } => Ok(ClosureFingerprint {
            key_hex: hexadecimal_text(&key.0),
            toolchain_lines,
        }),
        EntryGroupVerdict::Refused { refusal_lines } => {
            Err(format!("闭包里有拒算：{}", refusal_lines.join(" | ")))
        }
        EntryGroupVerdict::NotCompiled { reason } => Err(format!("入口不编进来：{reason}")),
    }
}

/// `single-device` 两个模式没给片结果库目录时的退出码：与用法错同一个码，缺的是调用方必须给的输入。
pub(super) const NO_VERDICT_LIBRARY_EXIT_CODE: i32 = 2;

/// 片结果库目录只认调用方经 `E182_VERDICT_LIBRARY` 给的那一个：判定库只有 prune-server 那一份，装置读写的目录是调用方这一趟的中转目录
/// （`.claude/rules/verification.md`「判定库只有一份」），所以没给就拒，不落本地缺省库。`Err` 里是写给人看的拒绝与出路。
pub(super) fn verdict_library_directory_given_by(
    variable_value: Option<&str>,
) -> Result<std::path::PathBuf, String> {
    match variable_value {
        Some(directory) if !directory.is_empty() => Ok(std::path::PathBuf::from(directory)),
        Some(_) | None => Err(format!(
            "没设 {LIBRARY_ENVIRONMENT_VARIABLE}：装置没有缺省的本地片结果库，片结果只存在 prune-server 里。\n\
             → 怎么办：经长跑脚本起（scripts/experiment-apparatus/e182-single-device-long-run.sh 跑 single-device，\
             scripts/experiment-apparatus/e182-single-device-units-kept-long-run.sh 跑 single-device-units-kept），\
             它每一趟开自己的中转目录、把 {LIBRARY_ENVIRONMENT_VARIABLE} 设成它，跑前经 scripts/gate/admission.py library-files fetch 拉库、\
             跑完经 library-files store 并回服务、再删掉中转目录；手起时照同一套做"
        )),
    }
}

pub(super) fn create_the_verdict_library_directory(
    directory: &std::path::Path,
) -> Result<(), String> {
    std::fs::create_dir_all(directory)
        .map_err(|error| format!("建不了片结果库目录 {}：{error}", directory.display()))
}

fn shard_key_text(
    fingerprint: &ClosureFingerprint,
    history_name: &str,
    shard_index: usize,
    expansion: Expansion,
) -> String {
    let mut text = String::new();
    text.push_str("e182-single-device-shard-v1\n");
    text.push_str(&format!("closure={}\n", fingerprint.key_hex));
    for line in &fingerprint.toolchain_lines {
        text.push_str(&format!("toolchain={line}\n"));
    }
    text.push_str(&format!(
        "history={history_name}\nsegment={shard_index}\nexpansion={}\n",
        expansion.name()
    ));
    text
}

fn shard_file_name(key_text: &str) -> String {
    format!(
        "{}.shard",
        hexadecimal_text(&sha256_digest(key_text.as_bytes()))
    )
}

fn commented_key(key_text: &str) -> String {
    key_text.lines().map(|line| format!("# {line}\n")).collect()
}

fn read_stored_shard(directory: &std::path::Path, key_text: &str) -> Option<Counters> {
    let text = std::fs::read_to_string(directory.join(shard_file_name(key_text))).ok()?;
    let body = text.strip_prefix(&commented_key(key_text))?;
    Counters::from_text(body).ok()
}

/// 一片判完才原子落盘：同目录写临时文件再改名换上。
fn store_shard(
    directory: &std::path::Path,
    key_text: &str,
    counters: &Counters,
) -> Result<(), String> {
    let name = shard_file_name(key_text);
    let temporary = directory.join(format!(".{name}.{}.temporary", std::process::id()));
    let mut text = commented_key(key_text);
    text.push_str(&counters.to_text());
    std::fs::write(&temporary, text)
        .map_err(|error| format!("写不了 {}：{error}", temporary.display()))?;
    std::fs::rename(&temporary, directory.join(&name))
        .map_err(|error| format!("改名不成 {name}：{error}"))
}

pub(super) struct ShardJob<'history> {
    pub(super) history: &'history RecordedHistory,
    pub(super) history_name: String,
    pub(super) shard_index: usize,
    expansion: Expansion,
    /// 全部历史按次序、每条历史的片按段号排下来的片号（多机分片按它对份数取余）。
    global_number: usize,
}

pub(super) fn expansion_of_shard(history: &RecordedHistory, shard_index: usize) -> Expansion {
    match history.segments.get(shard_index) {
        Some(segment) => expansion_of_segment(segment.len()),
        None => Expansion::FinalState,
    }
}

pub(super) fn plan_the_shards(histories: &[RecordedHistory]) -> Vec<ShardJob<'_>> {
    let mut jobs = Vec::new();
    for history in histories {
        for shard_index in 0..=history.segments.len() {
            jobs.push(ShardJob {
                history,
                history_name: history_name(&history.specification),
                shard_index,
                expansion: expansion_of_shard(history, shard_index),
                global_number: jobs.len(),
            });
        }
    }
    jobs
}

/// `E182_SHARD_SHARE=<第几份>/<共几份>`；没设是 `None`。
pub(super) fn shard_share_from_the_environment() -> Result<Option<(usize, usize)>, String> {
    let Ok(text) = std::env::var(SHARD_SHARE_ENVIRONMENT_VARIABLE) else {
        return Ok(None);
    };
    let (which, of) = text.split_once('/').ok_or_else(|| {
        format!("{SHARD_SHARE_ENVIRONMENT_VARIABLE} 要写「第几份/共几份」，给的是 {text}")
    })?;
    let which = which
        .parse::<usize>()
        .map_err(|error| format!("{text}：{error}"))?;
    let of = of
        .parse::<usize>()
        .map_err(|error| format!("{text}：{error}"))?;
    if of == 0 || which >= of {
        return Err(format!(
            "{SHARD_SHARE_ENVIRONMENT_VARIABLE}={text}：第几份要小于共几份、共几份要大于 0"
        ));
    }
    Ok(Some((which, of)))
}

pub(super) struct ShardResult {
    pub(super) counters: Counters,
    pub(super) reused: bool,
}

pub(super) struct RunOptions<'options> {
    pub(super) library: Option<&'options std::path::Path>,
    pub(super) fingerprint: Option<&'options ClosureFingerprint>,
    pub(super) share: Option<(usize, usize)>,
}

pub(super) struct RunReport {
    /// 与 `jobs` 同序；不归这一份的片（多机分片）与停机之后没跑的片是 `None`。
    pub(super) results: Vec<Option<ShardResult>>,
    pub(super) stop: Option<String>,
}

fn key_text_of_the_job(job: &ShardJob<'_>, options: &RunOptions<'_>) -> Option<String> {
    options.fingerprint.map(|fingerprint| {
        shard_key_text(
            fingerprint,
            &job.history_name,
            job.shard_index,
            job.expansion,
        )
    })
}

fn process_one_shard(job: &ShardJob<'_>, options: &RunOptions<'_>) -> Result<ShardResult, String> {
    let key_text = key_text_of_the_job(job, options);
    if let (Some(library), Some(key_text)) = (options.library, &key_text) {
        if let Some(counters) = read_stored_shard(library, key_text) {
            return Ok(ShardResult {
                counters,
                reused: true,
            });
        }
    }
    let started = std::time::Instant::now();
    let counters = judge_the_shard(job.history, job.shard_index)?;
    if let (Some(library), Some(key_text)) = (options.library, &key_text) {
        store_shard(library, key_text, &counters)?;
    }
    eprintln!(
        "progress: shard {} {} segment={} expansion={} states={} seconds={:.1}",
        job.global_number,
        job.history_name,
        job.shard_index,
        job.expansion.name(),
        counters.count_named("states"),
        started.elapsed().as_secs_f64()
    );
    Ok(ShardResult {
        counters,
        reused: false,
    })
}

fn states_the_shard_will_judge(job: &ShardJob<'_>) -> u64 {
    job.history
        .segments
        .get(job.shard_index)
        .map_or(1, |segment| {
            state_count_of_segment(&job.history.writes, segment).unwrap_or(0)
        })
}

/// 把 `jobs` 里归这一份的片分给工作线程判（或从库里取）。出现停机条件（该停下交回主 agent 的）就让各线程做完手头那一片后收手。
pub(super) fn run_the_shards(jobs: &[ShardJob<'_>], options: &RunOptions<'_>) -> RunReport {
    use std::sync::atomic::AtomicBool;
    use std::sync::Mutex;
    let next_job = std::sync::atomic::AtomicUsize::new(0);
    let stop_requested = AtomicBool::new(false);
    let stop_reason: Mutex<Option<String>> = Mutex::new(None);
    let slots: Mutex<Vec<Option<ShardResult>>> =
        Mutex::new((0..jobs.len()).map(|_| None).collect());
    // 状态多的片先派：最大的几片各占一个线程，尾巴上不剩一片慢的拖着整趟（只改派的次序，结果按片号存）。
    let mut dispatch_order: Vec<usize> = (0..jobs.len()).collect();
    dispatch_order
        .sort_by_key(|index| std::cmp::Reverse(states_the_shard_will_judge(&jobs[*index])));
    std::thread::scope(|scope| {
        for _ in 0..worker_threads() {
            scope.spawn(|| loop {
                if stop_requested.load(Ordering::SeqCst) {
                    break;
                }
                let position = next_job.fetch_add(1, Ordering::SeqCst);
                let Some(index) = dispatch_order.get(position).copied() else {
                    break;
                };
                let job = &jobs[index];
                if let Some((which, of)) = options.share {
                    if job.global_number % of != which {
                        continue;
                    }
                }
                match process_one_shard(job, options) {
                    Ok(result) => {
                        slots.lock().expect("片结果表的锁没被毒化")[index] = Some(result);
                    }
                    Err(reason) => {
                        *stop_reason.lock().expect("停机原因的锁没被毒化") = Some(reason);
                        stop_requested.store(true, Ordering::SeqCst);
                        break;
                    }
                }
            });
        }
    });
    let results = slots.into_inner().expect("片结果表的锁没被毒化");
    RunReport {
        results,
        stop: stop_reason.into_inner().expect("停机原因的锁没被毒化"),
    }
}

/// 哨兵（`.claude/rules/test-script-discipline.md`「判定可复用」）：从取自库里的片里挑至多 `SENTINELS_AT_MOST` 片（状态最少的，同数取片号小的）
/// 重新现判，与库里的逐名相同；不同就是键没罩住输入，停机。
const SENTINELS_AT_MOST: usize = 3;

pub(super) fn recheck_the_sentinels(
    jobs: &[ShardJob<'_>],
    report: &RunReport,
) -> Result<usize, String> {
    let mut reused: Vec<(u64, usize)> = report
        .results
        .iter()
        .enumerate()
        .filter_map(|(index, result)| {
            result
                .as_ref()
                .filter(|result| result.reused && result.counters.count_named("states") > 0)
                .map(|result| (result.counters.count_named("states"), index))
        })
        .collect();
    reused.sort_unstable();
    let mut checked = 0;
    for (_, index) in reused.into_iter().take(SENTINELS_AT_MOST) {
        let job = &jobs[index];
        let rejudged = judge_the_shard(job.history, job.shard_index)?;
        let stored = &report.results[index]
            .as_ref()
            .expect("哨兵是取自库里的片，结果在")
            .counters;
        if &rejudged != stored {
            return Err(format!(
                "停机：哨兵 {} 段 {} 现判的计数与库里的不同（复用键没罩住输入）",
                job.history_name, job.shard_index
            ));
        }
        checked += 1;
    }
    Ok(checked)
}

#[cfg(test)]
mod tests;
