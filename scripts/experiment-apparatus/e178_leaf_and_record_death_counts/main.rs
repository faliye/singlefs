//! E178（叶容器与记录死亡的计数）装置：跑前登记 `research/prompts/e178-preregistration.md`（第一至十三节）。
//!
//! 独立手写计数模型，不与 `crates/` 共用代码、不依赖 `crates/`（`.claude/rules/implementation-first.md` 第 4 条）。
//! 写路径：多对象带数据、叶容器末尾分裂、删空不落盘、合并策略 M(θ) 的借与吸收（借分 N1、N2 两条臂）、退役记录（底座）、
//! 五条墓碑写流（X2 的 Z / T 与 X3 的 E-I / E-L-w / E-L-r 配出来的）、谱系、实例表、管理员回退、意图到最后一批之间崩溃与恢复续做。
//! 读者一个，按臂取参数（登记 5.4）：主路径按组索引择新，参照路径整扫一遍择新，两路对拍（第十一节 V3）。
//!
//! 用法（在 research/ 下）：
//!   cargo run --release --bin e178-leaf-and-record-death-counts -- segment-one     第一段：X4（坏镜像全表、只跑写路径）
//!   cargo run --release --bin e178-leaf-and-record-death-counts -- segment-two     第二段：X2 与 X3 的漏活（每次扫描都读）
//!   cargo run --release --bin e178-leaf-and-record-death-counts -- segment-three   第三段：X3 的崩溃那一半（F12 全部崩溃支）
//!   python3 scripts/experiment-apparatus/device-code-key.py run . --binary e178-leaf-and-record-death-counts --entry e7_index_bench::segment_four_item_lines \
//!       -- segment-four                                                           第四段：登记修订 2 的两条新底座臂（B-stub、B-tsz），读写项库
//! 线程数取环境变量 E178_THREADS（没设取 SINGLEFS_THREAD_CAP，都没设取登记的重活默认并行度 .claude/heavy-thread-default），产物按工作项的次序输出，与线程数无关。
//! 第四段的分片、续跑与多机（登记修订 2 之十三、修订 4 之五）：997 个工作项，每项一把项键 = sha256（入口 `segment_four_item_lines` 的代码键
//! 与这一项 `{item:?}` 的全部字段值）；代码键由取键脚本 `scripts/experiment-apparatus/device-code-key.py` 算（单元闭包、工具链行、闭包点到的依赖行），
//! 经 `--code-key-file` 交进来。项库是 prune-server 里的那一份判定库（.claude/rules/verification.md「判定库只有一份」）：装置不开本地库目录，
//! 读写经 `device-code-key.py run` 开的两根管道转给服务（`crates/singlefs-device-library/src/lib.rs` 文件头；族与种类在 device-code-key.py 文件头「判定库的转交」），
//! 服务连不上照转回的退出码退（75），不退回本地文件。每项算完写进 `items/<项键>.out` 那一条（键行、parameters、meta、产物行、done 行），
//! 组成清单存 `codes/<代码键>.txt`，不分片的那一趟拼完重写 `index.tsv`（项名 → 项键，只当索引）。同一条命令重起，项键相同的项读回不重算；
//! 项名、序号、标签都不进键。补算缺的项，997 项齐了按项的次序拼出产物打到 stdout；要两台分判，由统一任务服务分片。
//! 产物是 `E7RESULT` 行，收尾 `E7RESULT name=done emitted=N`（N 把 config 行与 done 行都算进去）。
//!
//! 登记没写死、装置取了一个读法的几格（报告与实验页逐条列，交主 agent 认）：
//! 1. 容器名字空间只数打包记录类型 2（inode 叶容器）的身份；墓碑容器、退役容器不进三个名字空间。
//! 2. X2 的判活（Z 的 size、T 与 G0 的对象门控）用施行整对象墓碑之前的那棵树；inode 读出集合是那棵树去掉被整对象墓碑施行的
//!    (o, b)。登记第九节 M8（E-I 不丢数据单元时途中 extent m 由 300、156、12 变 0）只在这个读法下成立。
//! 3. T 的区间规则只认区间起 > 0 的记录（变短那次写的）；区间起 = 0 的整对象墓碑归 X3 臂。登记第四节第 6 条
//!    「E-L 两臂与 Z、T 配对时 unlink 途中漏活 0」只在这个读法下成立。
//! 4. 回退那次发布不重写墓碑树、退役树的开放容器（D18 已定项 24 第 3 条「有新记录的那次发布才重写」）；缺口里写的那几版容器
//!    诞生 txg 不在新父链上，读者在「候选」那一步就把它们滤掉，回退之后第一次往那棵树追加才重新读得到缺口里的记录。
//! 5. 每次用户改动先把删除队列走空（D8 已定项 16 第 6 条）；只在回退回到意图那一版、崩溃恢复之后才走得到。
//! 6. F11 的「续按升序删」= 删号最小的活对象（号 ≥ 2），回退带回来的对象先被再删一次。
//! 7. F12 的「再建 5 个空 inode」= 一次发布建 5 个；P-m 前缀按升序删时不做 F10 的覆盖写规则（F12 那一行没写）。
//! 8. Q-X4-1 的「这段历史全部写单元」= 数据单元 + 叶容器 + inode 树根 + 退役容器 + 退役树根 + 今天那条墓碑写流
//!    （Z × E-I，意图那次写整对象墓碑）的墓碑容器与墓碑树根，每份计。
//! 9. 植入的「写序最大」= (扫描那一版的 txg, 现行实例, 现行事务号)，同写序按单元序号破平，植入的排在盘上全部单元之后。
//!
//! 第四段（登记修订 2，问法已定：N2-none、P-C）另取的读法：
//! 10. B-stub 写路径里「新号只落最右叶」的最右叶 = 最右那片活叶（记录数 ≥ 1），桩不收新记录；活叶装不下时新片接在叶序末尾。
//!     依据是修订 2 之五「活叶的身份、记录、分裂、M(θ) 的借与吸收，逐版与同格原写路径相同」；叶序里桩的位置不进任何计数。
//! 11. 改动计数（记录偏移 88）：新建、覆盖写、顺序写、意图、每一批写本次 txg；挪动、连带重写、回退照抄。植入的记录取植入写序的诞生 txg。
//! 12. PC-Bs1 按 W 两档各取一次扫描（「在这一档 W 下读得出」），只对同一档 W 的 B-stub 读者判期望。
//! 13. B-tsz 的「(o, b) 的记录在所有被择中的版本里都不收」与读出那一步施行整对象墓碑在这个模型里同值（同一个 inode 号只有一个对象
//!     出生代），装置只在读出那一步施行；数据单元门用供出之后、施行整对象墓碑之前的那棵树（修订 2 之五数据单元门）。

// admission: always 每次照此刻的仓与自己的常量现算，上一次跑出的产物不替这一次作保
// run-condition: command python3
use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};

mod arms_streams_and_readers;
mod crash_prefix_runs_and_verdicts;
mod history_runs_and_pair_statistics;
mod item_library_and_reuse;
mod leaf_shapes_and_their_predicates;
mod missed_deaths_runs_and_verdicts;
mod new_base_arm_aggregates_and_verdicts;
mod new_base_arm_runs_and_lines;
mod positive_controls_and_deltas;
mod publish_records_and_tallies;
mod publish_work;
mod reader_scan_and_judgement;
mod simulation_steps;
mod stub_lineage_probes;
#[cfg(test)]
mod test_support_for_the_runs;
mod trajectories_and_histories;
mod units_disks_and_versions;
mod write_path_runs_and_stop_checks;

use self::arms_streams_and_readers::*;
use self::crash_prefix_runs_and_verdicts::*;
use self::history_runs_and_pair_statistics::*;
use self::item_library_and_reuse::*;
use self::leaf_shapes_and_their_predicates::*;
use self::missed_deaths_runs_and_verdicts::*;
use self::new_base_arm_aggregates_and_verdicts::*;
use self::new_base_arm_runs_and_lines::*;
use self::positive_controls_and_deltas::*;
use self::publish_records_and_tallies::*;
use self::publish_work::*;
use self::reader_scan_and_judgement::*;
use self::stub_lineage_probes::*;
#[cfg(test)]
use self::test_support_for_the_runs::*;
use self::trajectories_and_histories::*;
use self::units_disks_and_versions::*;
use self::write_path_runs_and_stop_checks::*;

// ---------------------------------------------------------------------------------------------
// 格式常量（本地常量，值抄自 kb 的登记位；第七节 K1–K3、A2 钉它们）
// ---------------------------------------------------------------------------------------------

/// 打包记录单元（码 3）32 KiB，记录区起点 136（含 nonce / MAC 预留位的头，D18 已定项 11 / 已定项 16）。
const PACKED_CONTAINER_BYTES: u64 = 32_768;
const PACKED_RECORDS_OFFSET: u64 = 136;
/// inode 记录定长 140（D8 已定项 6）。
const INODE_RECORD_BYTES: u64 = 140;
/// 墓碑记录 40 字节（D18 已定项 24 第 2 条）。
const TOMBSTONE_RECORD_BYTES: u64 = 40;
/// 退役记录 = 出生树 8 + 打包记录类型 2 + 容器号 8 + 容器出生代 8 + 死亡 txg 8（登记第一节读法 9；宽度是本实验自定，条款没定）。
const RETIREMENT_RECORD_BYTES: u64 = 34;
const INODE_LEAF_RECORDS: u64 = 233;
const TOMBSTONE_CONTAINER_RECORDS: u64 = 815;
const RETIREMENT_CONTAINER_RECORDS: u64 = 959;
// 整数字面量给门禁 format-constants 读；推导式在编译期核，两处对不上编不过
const _: () = assert!(RETIREMENT_RECORD_BYTES == 8 + 2 + 8 + 8 + 8);
const _: () = assert!(INODE_LEAF_RECORDS == (PACKED_CONTAINER_BYTES - PACKED_RECORDS_OFFSET) / INODE_RECORD_BYTES);
const _: () = assert!(TOMBSTONE_CONTAINER_RECORDS == (PACKED_CONTAINER_BYTES - PACKED_RECORDS_OFFSET) / TOMBSTONE_RECORD_BYTES);
const _: () = assert!(RETIREMENT_CONTAINER_RECORDS == (PACKED_CONTAINER_BYTES - PACKED_RECORDS_OFFSET) / RETIREMENT_RECORD_BYTES);
/// 码 2 根：节点 16 KiB，头 = 86 + 2 × key 宽 8 + 预留 29，条目 120（D8 已定项 6「扇出 135」）。
const NODE_BYTES: u64 = 16_384;
const INDEX_NODE_HEADER_BYTES_WITHOUT_KEY_RANGE: u64 = 86;
const INDEX_NODE_RESERVED_BYTES: u64 = 29;
const INODE_KEY_BYTES: u64 = 8;
const INODE_INTERNAL_ENTRY_BYTES: u64 = 120;
const ROOT_NODE_LEAF_CONTAINERS: u64 = (NODE_BYTES
    - (INDEX_NODE_HEADER_BYTES_WITHOUT_KEY_RANGE + 2 * INODE_KEY_BYTES + INDEX_NODE_RESERVED_BYTES))
    / INODE_INTERNAL_ENTRY_BYTES;
/// 数据单元净荷容量 32634（`crates/singlefs-core/src/unit.rs:233-236`；单元序号 = 字节偏移 ÷ 它，登记第一节读法 1）。
const DATA_UNIT_PAYLOAD_BYTES: u64 = 32_634;
/// 删除一批的单元数：产品取 extent 树下段一片叶的单元数 144（D8 已定项 16 第 2 条）。
const PRODUCT_UNLINK_BATCH_DATA_UNITS: u64 = 144;
/// 新池新建文件：inode 1、出生代 3、size 3000（D8 已定项 6「新池新建文件写」与 `blocks` 那段）。
const FIRST_FILE_INODE: u64 = 1;
const NEW_POOL_FILE_CREATION_TXG: u64 = 3;
const NEW_POOL_FILE_SIZE_BYTES: u64 = 3_000;
/// mkfs 那个进程取实例 1，可写挂载取实例 2（E177 装置第 102–103 行同一读法）。
const MAKE_FILESYSTEM_PROCESS_INSTANCE: u64 = 1;
const MOUNT_INSTANCE: u64 = 2;
/// 树 ID（E177 装置第 85–90 行抄的值）；退役树是本实验模型自定的一棵，条款没有它的号。数据单元只有一棵出生树，模型不记它。
const TREE_IDENTIFIER_INODE: u64 = 65_537;
const TREE_IDENTIFIER_TOMBSTONE: u64 = 65_545;
const TREE_IDENTIFIER_RETIREMENT_IN_THIS_MODEL: u64 = 65_549;
/// 阳性对照植入的「全新身份」用的出生树，盘上任何真单元都不用它。
const PLANTED_BIRTH_TREE: u64 = 0xE178;
const PACKED_TYPE_TOMBSTONE: u64 = 1;
const PACKED_TYPE_INODE: u64 = 2;
const PACKED_TYPE_RETIREMENT: u64 = 3;
const WHOLE_OBJECT_RANGE_START_BYTES: u64 = 0;
/// 旋钮 W 的有限那一档 23（登记 5.1：E177 装置 `RING_WINDOW_IN_PUBLISHES`，第 39 行）。
const READABLE_WINDOW_RING_PUBLISHES: u64 = 23;
/// F11 回退距离的上限（登记 5.2）。
const ROLLBACK_DISTANCE_LIMIT_PUBLISHES: u64 = 20;
/// 叶容器容量的只供测试那一档（登记 5.1）。
const TEST_LEAF_CAPACITY: u64 = 16;
/// 历史族的常量（登记 5.2；F1、F3 的两个抄 E177 装置第 4611、4615 行）。
const NEW_INODES_PER_STEP: u64 = 50;
const MIXED_RANDOM_STEPS: u64 = 300;
const MANY_OBJECTS_OVERWRITE_EVERY_UNLINKS: u64 = 10;
const MANY_OBJECTS_SEQUENTIAL_WRITE_EVERY_UNLINKS: u64 = 25;
const MANY_OBJECTS_SEQUENTIAL_WRITE_DATA_UNITS: u64 = 5;
const ROLLBACK_OVER_A_GAP_INODE_COUNT: u64 = 300;
const ROLLBACK_OVER_A_GAP_CHANGES_BEFORE_ROLLBACK: u64 = 10;
const ROLLBACK_OVER_A_GAP_CHANGES_AFTER_ROLLBACK: u64 = 20;
const INTENT_CRASH_INODE_COUNT: u64 = 300;
const INTENT_CRASH_NEW_INODES_AFTER_RECOVERY: u64 = 5;

fn length_as_u64(length: usize) -> u64 {
    u64::try_from(length).expect("长度装得进 u64")
}

fn u64_as_index(value: u64) -> usize {
    usize::try_from(value).expect("下标装得进 usize")
}

/// 两个计数相减，减数大于被减数时给 u64::MAX（产物里一眼看得出不对，不静默回绕）。
fn difference_or_marker(larger: u64, smaller: u64) -> u64 {
    larger.checked_sub(smaller).unwrap_or(u64::MAX)
}

/// 64 位 FNV-1a，给逐发布的叶容器形状做摘要（A5 比 N1、N2 两条写流逐单元相同）。
fn fold_into_digest(digest: u64, value: u64) -> u64 {
    let mut folded = digest;
    for byte in value.to_le_bytes() {
        folded ^= u64::from(byte);
        folded = folded.wrapping_mul(0x0000_0100_0000_01B3);
    }
    folded
}

const DIGEST_START: u64 = 0xCBF2_9CE4_8422_2325;

/// 第四段参数解析要读的环境（单测换成假的，不动进程的环境变量）：转交判定库的口子（device-code-key.py run 放进环境的两根管道接得上是 Ok，
/// 不然是为什么）。
struct SegmentFourEnvironment {
    store: Result<DeviceLibrary, String>,
}

impl SegmentFourEnvironment {
    fn of_this_process() -> SegmentFourEnvironment {
        SegmentFourEnvironment { store: DeviceLibrary::from_environment() }
    }
}

/// `segment-four` 的参数（登记修订 4 之六第 5 条）：`--code-key-file <组成清单>`（必给）。
/// 要接得上转交判定库的管道（经 device-code-key.py run 起）。
fn segment_four_options_from(arguments: &[String], environment: &SegmentFourEnvironment) -> Result<SegmentFourOptions, String> {
    let mut code_key_file: Option<std::path::PathBuf> = None;
    let mut remaining = arguments.iter();
    while let Some(argument) = remaining.next() {
        match argument.as_str() {
            "--code-key-file" => code_key_file = Some(std::path::PathBuf::from(remaining.next().ok_or("--code-key-file 后面缺文件")?)),
            other => return Err(format!("认不得的参数：{other}")),
        }
    }
    let Some(path) = &code_key_file else {
        return Err("没给 --code-key-file：读写项库要经 scripts/experiment-apparatus/device-code-key.py run 起".to_string());
    };
    let composition = std::fs::read_to_string(path).map_err(|error| format!("读不了 --code-key-file {}：{error}", path.display()))?;
    let library = segment_four_library_from_composition(environment.store.clone()?, &composition)?;
    Ok(SegmentFourOptions { library })
}

// 写成 `#[path] mod` 而不是条目级 `include!`：`unit-graph` 把条目级 `include!` 记成拒算，入口闭包的键就算不出（登记修订 4 之六第 2 条）。
#[path = "../heavy_threads_default/threads.rs"]
mod heavy_threads_default;
use heavy_threads_default::default_heavy_threads;
// 判定库的口子（文件头「第四段的分片、续跑与多机」）：包 `singlefs-device-library`（crates/singlefs-device-library），scripts/experiment-apparatus/Cargo.toml 里按 path 依赖引
use singlefs_device_library::DeviceLibrary;

/// 线程数：`E178_THREADS`，没设取 `SINGLEFS_THREAD_CAP`，都没设取登记的重活默认并行度（`.claude/heavy-thread-default`）。
fn threads_from_the_environment() -> usize {
    std::env::var("E178_THREADS")
        .ok()
        .and_then(|value| value.parse::<usize>().ok())
        .filter(|threads| *threads >= 1)
        .unwrap_or_else(|| default_heavy_threads(include_str!("../../../.claude/heavy-thread-default")))
}

fn main() {
    preflight();
    let arguments: Vec<String> = std::env::args().collect();
    let mode = arguments.get(1).cloned().unwrap_or_default();
    let threads = threads_from_the_environment();
    let usage = "用法：e178-leaf-and-record-death-counts segment-one | segment-two | segment-three \
                 | segment-four --code-key-file <组成清单>（读写项库经 scripts/experiment-apparatus/device-code-key.py run 起）";
    let emitted_lines = match mode.as_str() {
        "segment-one" => run_segment_one(threads),
        "segment-two" => run_segment_two(threads),
        "segment-three" => run_segment_three(threads),
        "segment-four" => match segment_four_options_from(&arguments[2..], &SegmentFourEnvironment::of_this_process()) {
            Ok(options) => run_segment_four(threads, &options),
            Err(problem) => {
                eprintln!("{problem}\n{usage}");
                std::process::exit(2);
            }
        },
        _ => {
            eprintln!("{usage}");
            std::process::exit(2);
        }
    };
    let emitted = emitted_lines.len() + 1;
    for line in &emitted_lines {
        println!("E7RESULT {line}");
    }
    println!("E7RESULT name=done emitted={emitted}");
}

/// 准入用的仓根：环境变量 SINGLEFS_REPOSITORY_ROOT，没设取当前目录（replay.sh 与实验页的命令都在仓根起）；不取「清单目录/../..」（单元键把它当运行期路径拒算）。
fn repository_root() -> std::path::PathBuf {
    let root = std::env::var_os("SINGLEFS_REPOSITORY_ROOT")
        .map(std::path::PathBuf::from)
        .or_else(|| std::env::current_dir().ok())
        .unwrap_or_default();
    if !root.join(".claude/singlefs-ai-sop/scripts/preflight.py").is_file() {
        eprintln!("  ✗ {} 不是仓根（下面没有 .claude/singlefs-ai-sop/scripts/preflight.py），判不了准入", root.display());
        eprintln!("  → 怎么办：在仓根起这个装置，或设 SINGLEFS_REPOSITORY_ROOT=<仓根>");
        std::process::exit(78);
    }
    root
}

fn preflight() {
    let arguments: Vec<String> = std::env::args().skip(1).collect();
    let root = repository_root();
    let source = root.join("scripts/experiment-apparatus/e178_leaf_and_record_death_counts/main.rs");
    let script = root.join(".claude/singlefs-ai-sop/scripts/preflight.py");
    let mut command = std::process::Command::new("python3");
    command.arg(&script).arg("check").arg(&source);
    if arguments.iter().any(|argument| argument == "--force") {
        command.arg("--force");
    }
    command.arg("--");
    command.args(arguments.iter().filter(|argument| *argument != "--force"));
    let output = command.output().unwrap_or_else(|error| {
        eprintln!("  ✗ 起不了 python3 判准入：{error}");
        eprintln!("  → 怎么办：装上 python3，或在有 python3 的机器上跑");
        std::process::exit(78)
    });
    if !output.status.success() {
        eprint!("{}", String::from_utf8_lossy(&output.stderr));
        print!("{}", String::from_utf8_lossy(&output.stdout));
        std::process::exit(output.status.code().unwrap_or(1));
    }
}

#[cfg(test)]
mod tests;
