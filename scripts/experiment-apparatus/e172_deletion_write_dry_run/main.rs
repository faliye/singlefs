//! E172（删除写出的字节的干跑）：第 1 次跑前登记 `research/prompts/e172-preregistration.md`，
//! 第 2 次重跑登记 `research/prompts/e172-r2-prereg.md`（按它第五节 5.8 改1–改15 改过），
//! 第 3 次重跑登记 `research/prompts/e172-r3-prereg.md`（按它第五节 5.8 改16–改30 加了退役项：标签 6 式子的退役份额、
//! 退役容器 tr@0、退役记录、退役树根 tr、树表第 12 条、水位 65549，与 harness 钉值层、判别臂 R-ret、PC-J/K/L）。
//!
//! 研究侧独立解码器：不 `use` 任何 `singlefs-*` crate、不 include E142 的源码。读 `crates/` 侧只读导出
//! `deletion_write_dump` 的两份输出文件（不带参数的 H1、H2 一份，`--history h3` 一份），把窗口里每个写区域按
//! layout/01、layout/03（连 D18 已定项 7 / 11 / 18 / 24 的带偏移字段表）逐字段解出来，与登记取值、跑前登记第七节的锚点比；
//! 外加覆盖率（每个字节归到哪一格）。不手写独立模型：期望值只来自条款与锚点。
//!
//!   e172-deletion-write-dry-run <H1H2 导出文件> <H3 导出文件>
//!
//! 退出码：0 = 跑完（判定写在产物行里，不进退出码）；2 = 命令行不对或导出读不了。

// admission: always 输入变没变由 main 里的 admit_experiment_run_or_exit 照 stage-inputs.tsv 的 E172 行判（输入没变退 77），这一层每次放行
// run-condition: command python3
use e7_index_bench::Emitter;
use std::collections::{BTreeMap, BTreeSet};

mod cell_judging;
mod export_parsing_and_checksums;
mod history_analysis_attribution_and_judgement;
mod positive_controls_and_anchor_lines;
mod region_decoding;
mod report_rows;
mod roles_and_segment_analysis;
mod version_readings_and_expectations;

use self::cell_judging::*;
use self::export_parsing_and_checksums::*;
use self::history_analysis_attribution_and_judgement::*;
use self::positive_controls_and_anchor_lines::*;
use self::region_decoding::*;
use self::report_rows::*;
use self::roles_and_segment_analysis::*;
use self::version_readings_and_expectations::*;

// ---------------------------------------------------------------------------------------------
// 偏移与宽度常量：每一个都写出处（文件与行号），值从 kb 抄，不从 `crates/` 引。
// ---------------------------------------------------------------------------------------------

/// `.claude/kb/layout/01-first-txn.md` 第 40–41 行：系统配置两槽，槽宽 4096，槽 1 在偏移 4096。
const SYSTEM_CONFIGURATION_SLOT_BYTES: usize = 4096;
/// layout/01 第 156 行：系统配置字段合计 489 字节，其后到槽末是补齐（第 99 行：整槽校验和覆盖补齐）。
const SYSTEM_CONFIGURATION_FIELD_BYTES: usize = 489;
/// layout/01 第 42–43 行：根环基址 1 MiB、区域 r 在 基址 + r × P × chunk（P = 3、chunk = 1 MiB），每区 8 槽、槽距 4096、槽宽 512。
const ROOT_RING_BASE_BYTES: u64 = 1_048_576;
const ROOT_RING_REGION_STRIDE_BYTES: u64 = 3 * 1_048_576;
const ROOT_RING_SLOTS_PER_REGION: u64 = 8;
const ROOT_RING_SLOT_SPACING_BYTES: u64 = 4096;
const ROOT_RECORD_SLOT_BYTES: usize = 512;
/// layout/01 第 44、162 行：区域归属 0 / 1 / 0。
const ROOT_RING_REGION_DEVICES: [u32; 3] = [0, 1, 0];
/// layout/01 第 372–388 行按行累加：根记录 466 字节（回收门槛 t 428、钉槽 p 436 在分配记录树根指针之后），其后到 512 槽末是补齐（第 380 行）。
const ROOT_RECORD_FIELD_BYTES: usize = 466;
/// layout/01 第 45 行：journal 环从 16 MiB 起、长 768 MiB、记录 4096；第 319 行：记录 n 落 `(计数器 − 1) × 4096`。
const JOURNAL_RING_START_BYTES: u64 = 16 * 1_048_576;
const JOURNAL_RING_BYTES: u64 = 768 * 1_048_576;
const JOURNAL_RECORD_BYTES: usize = 4096;
/// layout/01 第 360 行：记录头合计 496（第 362 行逐个偏移：回收门槛 t 463、钉槽 p 471、fsid 472、MAC 480，到 496 为止）；第 364 行：点名项 56。
const JOURNAL_HEADER_BYTES: usize = 496;
const JOURNAL_NAMED_ITEM_BYTES: usize = 56;
/// layout/01 第 46 行：单元区起始槽号 50176；落点粒度 16384（第 34–35 行）。
const UNIT_AREA_START_SLOT: u64 = 50176;
const SLOT_BYTES: u64 = 16384;
/// layout/01 第 174 行与 D18 已定项 9：码 1 / 码 3 单元 32768，码 2 节点 16384。
const NODE_BYTES: usize = 16384;
/// layout/01 第 175–180 行：单元共同前缀（magic 0、格式版本 4、单元类型标签 6、flag 7、声明长度 8、头校验和 10 起 32）。
const UNIT_DECLARED_LENGTH_OFFSET: usize = 8;
const UNIT_HEADER_CHECKSUM_OFFSET: usize = 10;
const UNIT_COMMON_PREFIX_BYTES: usize = 42;
/// D18 已定项 18（登记第二节抄录）：码 2 头 = 86 + 2k，含 29 字节预留位 115 + 2k。
const INDEX_NODE_HEADER_BYTES_WITHOUT_KEY_RANGE: usize = 86;
const RESERVED_NONCE_MAC_ALGORITHM_BYTES: usize = 29;
/// D18 已定项 11 码 3 类身份段表（登记第二节第 276–293 行）：明文头 107，含预留位 136。
const PACKED_UNIT_PLAINTEXT_HEADER_BYTES: usize = 107;
const PACKED_UNIT_RECORDS_OFFSET: usize = 136;
/// layout/01 第 189–194 行：码 1 明文头 105，含预留位 134。
const DATA_UNIT_PLAINTEXT_HEADER_BYTES: usize = 105;
const DATA_UNIT_PAYLOAD_OFFSET: usize = 134;
const DATA_UNIT_BYTES: usize = 32768;
/// layout/01 第 214 行：指向码 2 / 码 3 的指针 86，指向码 1 的指针 88；第 216 行：位置条目 14，第一条在 50、第二条在 64。
const NODE_POINTER_BYTES: usize = 86;
const DATA_POINTER_BYTES: usize = 88;
const LOCATION_ENTRY_BYTES: usize = 14;
const POINTER_FIRST_LOCATION_OFFSET: usize = 50;
/// layout/01 第 285–290 行：extent 上段叶条目 113 = key 24 + 标签 1 + 载荷 88；下段叶条目 112 = key 24 + 数据指针 88。
const EXTENT_UPPER_LEAF_ENTRY_BYTES: usize = 113;
const EXTENT_LOWER_LEAF_ENTRY_BYTES: usize = 112;
/// D8 已定项 14（`.claude/kb/decisions/08-核心索引结构.md` 第 402 行）：下段叶罩 144 个单元。
const EXTENT_LOWER_LEAF_DATA_UNITS: u64 = 144;
/// layout/01 第 191 行：单元净荷容量 32768 − 134。
const DATA_UNIT_PAYLOAD_CAPACITY: u64 = 32768 - 134;
/// layout/01 第 260–262 行：inode 树内部节点条目 120 = 分隔 key 8 + 身份引用 26 + 子指针 86。
const INODE_INTERNAL_ENTRY_BYTES: usize = 120;
/// layout/01 第 265–273 行：inode 记录 140。
const INODE_RECORD_BYTES: usize = 140;
/// layout/01 第 297–299 行：分配记录 20 = key 10 + value 10；内部条目 96 = key 10 + 子指针 86；叶罩 812 槽，层级 1 罩 137228 槽。
const ALLOCATION_RECORD_BYTES: usize = 20;
const ALLOCATION_INTERNAL_ENTRY_BYTES: usize = 96;
const ALLOCATION_LEAF_SLOTS: u64 = 812;
const ALLOCATION_LEVEL_ONE_SLOTS: u64 = 812 * 169;
/// layout/01 第 303–305、311 行：记账条目 34 = key 22 + value 8 + seq 4。
const ACCOUNTING_ENTRY_BYTES: usize = 34;
/// layout/01 第 228–236 行：映射条目 55 = key 27 + value 28。
const MAPPING_ENTRY_BYTES: usize = 55;
/// layout/01 第 387 行：树表条目 200。
const TREE_TABLE_ENTRY_BYTES: usize = 200;
/// layout/03 第 10–11 行：`deleted_inodes` 条目 8；第 14、30–36 行：墓碑记录 40；D18 已定项 11 第 302–307 行：实例表记录 88。
const DELETED_INODES_ENTRY_BYTES: usize = 8;
const TOMBSTONE_RECORD_BYTES: usize = 40;
const INSTANCE_TABLE_RECORD_BYTES: usize = 88;
/// layout/03 第 21 行（「退役容器」那一行）与第 66–80 行（「五、退役记录」）：退役容器的打包记录类型 3、退役记录 34 字节。
const RETIREMENT_PACKED_RECORD_TYPE: u64 = 3;
const RETIREMENT_RECORD_BYTES: usize = 34;

/// layout/01 第 48 行与 layout/03 第 19 行：树 ID。
const EXTENT_TREE: u64 = 65536;
const INODE_TREE: u64 = 65537;
const ALLOCATION_RECORDS_TREE: u64 = 65538;
const ACCOUNTING_TREE: u64 = 65539;
const MAPPING_TREE: u64 = 65540;
const DELETED_INODES_TREE: u64 = 65544;
const TOMBSTONE_TREE: u64 = 65545;
const LINEAGE_TREE: u64 = 65546;
const LOGGED_OPERATIONS_TREE: u64 = 65547;
/// layout/03 第 23 行（「树表退役树条目」那一行）：退役树的树 ID 65548、种类码 13，头 ID 写 inode 树的树 ID（同墓碑树）。
const RETIREMENT_TREE: u64 = 65548;
const RETIREMENT_TREE_KIND_CODE: u64 = 13;
/// layout/01 第 378 行（「树 ID 水位」那一行）与 layout/03 第 22–23 行：第一个文件版本那次发布之后水位 65549（退役树占到 65548）。
const TREE_IDENTIFIER_WATERMARK_AFTER_THE_FIRST_FILE: u64 = 65549;
/// layout/01 第 80、392 行：树表第 1 版 12 条（layout/03 第 22 行「树表 7 → 11 条（再加退役树一条为 12）」）。
const TREE_TABLE_ENTRIES_AFTER_THE_FIRST_FILE: usize = 12;

/// layout/01 第 109、175、338、369 行：四种 magic。
const SYSTEM_CONFIGURATION_MAGIC: &[u8; 4] = b"SFSB";
const UNIT_MAGIC: &[u8; 4] = b"SFSU";
const JOURNAL_MAGIC: &[u8; 4] = b"SFSJ";
const ROOT_RECORD_MAGIC: &[u8; 4] = b"SFSR";

/// 跑前登记第六节 L03-2 与第七节 A-content：被删文件 = 两个满净荷加半个。
const DELETED_INODE: u64 = 1;
const EXPECTED_FILE_BYTES: u64 = 2 * DATA_UNIT_PAYLOAD_CAPACITY + DATA_UNIT_PAYLOAD_CAPACITY / 2;
/// layout/03 第 24 行（「记账标签 6 的值」那一行）：标签 6 式子里的副本数与 41（⌈2 × 16384 ÷ 815⌉）。
const ACCOUNTING_COPIES: u64 = 2;
const PER_LIVE_OBJECT_TOMBSTONE_BYTES: u64 = 41;
/// layout/03 第 24 行：每条 inode 记录（含 nlink = 0 的）的退役份额 35 = ⌈32768 ÷ 959⌉（每份副本）。
const PER_INODE_RECORD_RETIREMENT_BYTES: u64 = 35;
/// `crates/singlefs-verification/tests/unlink_idempotence_criteria_and_rollback_to_the_emptied_version.rs` 第 198 行钉的五个值，只抄数、不引 `crates/`：
/// 删除之前、意图、3 → 2、2 → 1、最后一批（跑前登记 e172-r3 第一节 1.2 那张表）。
const HARNESS_PINNED_TAG_SIX_VALUES: [u64; 5] = [196_760, 196_678, 196_678, 196_678, 196_608];

/// layout/01 第 158 行的偏移表：整槽校验和 155（宽 32，CRC32C 4 + 28 零，第 99、118 行）、T_time 433（宽 4，第 149 行）。
const SYSTEM_CONFIGURATION_CHECKSUM_OFFSET: usize = 155;
const SYSTEM_CONFIGURATION_TIME_THRESHOLD_OFFSET: usize = 433;
/// layout/01 第 149 行：T_time「5000（单位毫秒，即 5 秒）」（跑前登记 e172-r2 第七节 A-T_time；不引 `crates/` 的常量）。
const EXPECTED_TIME_THRESHOLD_MILLISECONDS: u64 = 5000;

/// R8.c 比 R8.b 多要求的两样（跑前登记 e172-r2 第六节 6.4，各写成一个能单独改的常量，第九节 M24、M25、M26 的锚点）：
/// 先判取号（6.4 ①在 ②之前）；轮换一支另要系统配置里的实例代号 = 那条 journal 记录 jsn 里的实例代号（layout/01 第 344 行）。
const ATTRIBUTION_JUDGES_ACQUISITION_BEFORE_ROTATION: bool = true;
const ROTATION_ATTRIBUTION_REQUIRES_THE_SAME_INSTANCE: bool = true;

const SHA256_ROUND_CONSTANTS: [u32; 64] = [
    0x428a2f98, 0x71374491, 0xb5c0fbcf, 0xe9b5dba5, 0x3956c25b, 0x59f111f1, 0x923f82a4, 0xab1c5ed5,
    0xd807aa98, 0x12835b01, 0x243185be, 0x550c7dc3, 0x72be5d74, 0x80deb1fe, 0x9bdc06a7, 0xc19bf174,
    0xe49b69c1, 0xefbe4786, 0x0fc19dc6, 0x240ca1cc, 0x2de92c6f, 0x4a7484aa, 0x5cb0a9dc, 0x76f988da,
    0x983e5152, 0xa831c66d, 0xb00327c8, 0xbf597fc7, 0xc6e00bf3, 0xd5a79147, 0x06ca6351, 0x14292967,
    0x27b70a85, 0x2e1b2138, 0x4d2c6dfc, 0x53380d13, 0x650a7354, 0x766a0abb, 0x81c2c92e, 0x92722c85,
    0xa2bfe8a1, 0xa81a664b, 0xc24b8b70, 0xc76c51a3, 0xd192e819, 0xd6990624, 0xf40e3585, 0x106aa070,
    0x19a4c116, 0x1e376c08, 0x2748774c, 0x34b0bcb5, 0x391c0cb3, 0x4ed8aa4a, 0x5b9cca4f, 0x682e6ff3,
    0x748f82ee, 0x78a5636f, 0x84c87814, 0x8cc70208, 0x90befffa, 0xa4506ceb, 0xbef9a3f7, 0xc67178f2,
];

fn main() {
    preflight();
    // 准入（`.claude/gate.d/stage-inputs.tsv` 的 E172 行，判法在 `scripts/gate/admission.py`；跑前登记 e172-r3 第十一节 11.3）：
    // 输入自上次产物以来没变就不重跑（退 77）；放行时输入指纹打在产物最前面。
    for admission_header_line in e7_index_bench::admit_experiment_run_or_exit("E172") {
        println!("{admission_header_line}");
    }
    let arguments: Vec<String> = std::env::args().skip(1).collect();
    let [first_export_path, second_export_path] = arguments.as_slice() else {
        eprintln!("用法：e172-deletion-write-dry-run <H1H2 导出文件> <H3 导出文件>，收到 {arguments:?}");
        std::process::exit(2);
    };
    let read_export = |export_path: &String| match std::fs::read_to_string(export_path) {
        Ok(text) => text,
        Err(error) => {
            eprintln!("读不了导出文件 {export_path}：{error}");
            std::process::exit(2);
        }
    };
    let first_export_text = read_export(first_export_path);
    let second_export_text = read_export(second_export_path);
    let mut emitter = Emitter::new();
    for body in report_lines(&[first_export_text.as_str(), second_export_text.as_str()]) {
        println!("{}", emitter.emit_raw(&body));
    }
    println!("{}", emitter.finish());
}

/// A-容量：layout/03 第 16 行 ⌊(32768 − 136) ÷ 40⌋、第 11 行 ⌊(16384 − 131) ÷ 8⌋、第 17 行 8 + 26 + 86，按头宽现算。
fn tombstone_container_capacity() -> usize {
    (DATA_UNIT_BYTES - PACKED_UNIT_RECORDS_OFFSET) / TOMBSTONE_RECORD_BYTES
}

fn deleted_inodes_root_capacity() -> usize {
    (NODE_BYTES - index_node_header_bytes(8) - RESERVED_NONCE_MAC_ALGORITHM_BYTES) / DELETED_INODES_ENTRY_BYTES
}

fn tombstone_root_entry_bytes() -> usize {
    8 + 26 + NODE_POINTER_BYTES
}

/// A-容量：layout/03 第 21 行 ⌊(32768 − 136) ÷ 34⌋ = 959（退役容器装的记录条数）。
fn retirement_container_capacity() -> usize {
    (DATA_UNIT_BYTES - PACKED_UNIT_RECORDS_OFFSET) / RETIREMENT_RECORD_BYTES
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
    let source = root.join("scripts/experiment-apparatus/e172_deletion_write_dry_run/main.rs");
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
