//! E166 并行提交的共享状态分法：整体独占、分区、乐观三个候选，在模型层枚举两到三个事务的交错，
//! 数各自让两个事务拿到同一个落点或同一个号的次数（重复分配数），以及乐观那一臂的冲突重算次数。
//!
//! 跑前登记 `research/prompts/e166-preregistration.md`（含第十二节「修订」；树 ID 改号的第 2 次重跑登记 `research/prompts/e166-r2-prereg.md`）；问题单 `research/prompts/m3-concurrency-questions.md` 的 Q4 一行。
//! 独立手写的计数模型，**不与 `crates/` 共用代码**（不是入库装置）：模型里每一条与 `crates/` 对应的规则都写成本地常量或本地函数，
//! 旁边注释写 `crates/` 的文件:行（登记第十一节「停机」逐项对拍过），单测用登记第七节甲组的值回比。
//!
//! 六条臂（A 整体独占、A字面、P区、P设、O整、O读）与它们各自的阳性对照（A去锁、A字面去锁、P区同、P设去锁、O整不、O读不），
//! 三个 Q1 候选（Q1甲 准备并行落盘串行、Q1乙 一次发布装多个事务、Q1丙 多次发布同时在飞）。
//!
//! 这个装置对登记里几处含糊的地方取的读法（逐条写进登记第十二节「修订」，不放宽任何判据）：
//! 1. 用户数据落点一列（U）：各盘空闲表同步变化的臂（A、A字面、P区、O整、O读）按一个起点槽计一次，不乘盘数
//!    （`allocator.rs:1539` 注释「两盘同槽」；登记第七节乙 1 的 U 在默认盘数 2 下就是 4 / 4，只有按槽计才成立）；
//!    P设 各盘表各自变化，按「盘 × 起点槽」计（乙 7 的 U 重复合计按这个口径）。
//! 2. Q1乙 下 G、TXG / RS、T 归批发布者一步发（登记 5.1、Q-2、Q-7 两行），不争用；5.3 末句「Q1乙、Q1丙 下 G 等于两件那一行」
//!    只对 Q1丙 成立，Q1乙 下这几列在阳性对照里也是 0。
//! 3. O读不 与 O整不 同一份代码（提交不校验、整份含 X / J / TXG 的拷贝值写回）：乙 1 要 X 一列 4 / 4，
//!    O读 的 X 在提交点发这件事被「关护栏」一并关掉。
//! 4. 记账 ACC 与 Q-10（丢更新）属第二段，这一版不建模；BASE 只建「有没有文件」位与版本号。
//!
//! 子命令：不带参数 = 第一段（停机对拍、阳性对照与乙 / 丙组锚点、六条臂在默认几何与 D = 1 上的 Q-1 至 Q-9）。
// admission: always 独立手写的纯算术计数模型，每次现算全部调度的枚举与锚点，判的是此刻装置的样子
// run-condition: command python3
use e7_index_bench::Emitter;
use std::collections::HashMap;
use std::fmt::Write as _;

mod anchor_lines_and_cell_lines;
mod arms_shapes_and_cell_specifications;
mod duplicates_metrics_and_exploration;
mod questions_judgements_and_verdict_lines;
mod shared_state_and_step_semantics;
#[cfg(test)]
mod tests;

use self::anchor_lines_and_cell_lines::*;
use self::arms_shapes_and_cell_specifications::*;
use self::duplicates_metrics_and_exploration::*;
use self::questions_judgements_and_verdict_lines::*;
use self::shared_state_and_step_semantics::*;

// ── 与 crates/ 对应的规则：本地常量，旁边写出处（登记第十一节停机条款逐项对拍过） ──────────────

/// 新池第一版的 inode 号水位 = `FIRST_INODE_NUMBER (1) + 1`。`crates/singlefs-core/src/transaction.rs:109`、`:6268`–`6269`。
const INITIAL_INODE_WATERMARK: u8 = 2;
/// 树 ID 的号与水位：树 ID 从 65536 起编，装不进 `u8`，这一族（水位、发号起点、发号个数）用 `u32`。
/// 第 2 次重跑登记 `research/prompts/e166-r2-prereg.md` 第五节臂 E2 ①，取 `u32`（登记第十二节记）。
type TreeIdentifierNumber = u32;
// 两个水位的值出自 D8（核心索引结构） 已定项 11 条文：mkfs 时水位 65536，第一个文件版本那次之后 65548。
// 对应 `crates/singlefs-format/src/lib.rs` 的 `TREE_IDENTIFIER_WATERMARK_AT_MKFS`、`TREE_IDENTIFIER_WATERMARK_AFTER_FIRST_PUBLISH`（按名字找，行号会漂）。
const TREE_IDENTIFIER_WATERMARK_AT_MKFS: TreeIdentifierNumber = 65536;
const TREE_IDENTIFIER_WATERMARK_AFTER_FIRST_PUBLISH: TreeIdentifierNumber = 65548;
/// 第一个文件版本那次发布从水位起连号发的树 ID 个数 = 两个水位之差 = 十二个（D8 已定项 8 ②「十二棵树的号从水位起连号发」）。
/// 对应 `crates/singlefs-core/src/transaction.rs` 的 `FileVersionTreeIdentifiers::issued_from_watermark`（十二项偏移表 `OFFSETS_FROM_THE_WATERMARK_IN_ISSUE_ORDER`）。
const TREE_IDENTIFIERS_ISSUED_AT_FIRST_FILE_VERSION: TreeIdentifierNumber =
    TREE_IDENTIFIER_WATERMARK_AFTER_FIRST_PUBLISH - TREE_IDENTIFIER_WATERMARK_AT_MKFS;
/// 根环区域数 R。`crates/singlefs-format/src/lib.rs:238`。
const ROOT_RING_REGIONS: u32 = 3;
/// journal 一条记录的定长字节数。`crates/singlefs-format/src/lib.rs:191`。
const JOURNAL_RECORD_BYTES: u64 = 4096;
/// journal 环长的 mkfs 默认值（768 MiB）。`crates/singlefs-format/src/lib.rs:213`。
const JOURNAL_RING_DEFAULT_BYTES: u64 = 805_306_368;
/// 默认环槽数 = 默认环长 ÷ 记录长（196608）。登记第四节甲。
const DEFAULT_JOURNAL_RING_SLOTS: u32 = 196_608;
/// 用户数据落点的跨度（槽）。`crates/singlefs-core/src/allocator.rs:1410`（`Placement { slot, span: 2 }`）。
const USER_DATA_SPAN_SLOTS: u32 = 2;

// ── 模型自己的取值（登记 5.1），不抄 crates/ ─────────────────────────────────────

/// 每块盘的用户数据空闲表有 16 个「对」（起点槽 0、2、…、30）。
const USER_DATA_PAIR_COUNT: usize = 16;
/// 提交内生块池：一个 64 槽的开放段。
const COMMIT_GENERATED_POOL_SLOTS: u8 = 64;
/// 每次发布重写的树节点个数（每个节点 1 槽）。
const COMMIT_GENERATED_NODES_PER_PUBLISH: u8 = 2;
/// 初始 checkpoint_txg、jsn 计数器、事务号已用最大值。
const INITIAL_CHECKPOINT_TXG: u8 = 5;
const INITIAL_JSN_COUNTER: u8 = 5;
const INITIAL_HIGHEST_TRANSACTION_NUMBER: u8 = 1;
/// 根环每区槽数的默认值。
const DEFAULT_ROOT_SLOTS_PER_REGION: u32 = 4;
/// 默认盘数与第一个反向取样点。
const DEFAULT_DEVICE_COUNT: usize = 2;
const SAMPLE_DEVICE_COUNT: usize = 1;
/// P设 一次提交内各盘不一致的重算上限（登记乙 7 口径：超过记「截断」）。
const DEVICE_DISAGREEMENT_RETRY_CAP: u8 = 2;

const MAXIMUM_TRANSACTIONS: usize = 4;
const MAXIMUM_DEVICES: usize = 2;
const MAXIMUM_REGIONS: usize = 4;
const NO_PAIR: u8 = u8::MAX;
const NO_SLOT: u8 = u8::MAX;
const FIRST_DEVICE_STEP_INDEX: u8 = 1;

// ── 共享状态分组：事务碰哪几样、锁外锁内怎么分 ───────────────────────────────────

const GROUP_USER: u8 = 1;
const GROUP_COMMIT_SLOTS: u8 = 2;
const GROUP_INODE: u8 = 4;
const GROUP_TREE: u8 = 8;
const GROUP_TRANSACTION_NUMBER: u8 = 16;
const GROUP_JOURNAL: u8 = 32;
const GROUP_ROOT_RING: u8 = 64;
const ALL_GROUPS: u8 = 127;

/// A字面 的锁只罩候选定义点名的四样：分配器（U、G）、根环、journal 写指针、inode 号水位；锁外是事务号与树 ID 水位（记账不建模）。
const LITERAL_LOCK_INSIDE_GROUPS: u8 =
    GROUP_USER | GROUP_COMMIT_SLOTS | GROUP_ROOT_RING | GROUP_JOURNAL | GROUP_INODE;
const LITERAL_LOCK_OUTSIDE_GROUPS: u8 = GROUP_TRANSACTION_NUMBER | GROUP_TREE;
/// O读 在提交那一原子步里按当时共享值发的几样（排序器在提交点上，登记 5.2）。
const OPTIMISTIC_COMMIT_TIME_GROUPS: u8 = GROUP_TRANSACTION_NUMBER | GROUP_JOURNAL | GROUP_ROOT_RING;

fn preflight() {
    let arguments: Vec<String> = std::env::args().skip(1).collect();
    let source = concat!(env!("CARGO_MANIFEST_DIR"), "/e166_parallel_commit_shared_state_partition/main.rs");
    let script = concat!(env!("CARGO_MANIFEST_DIR"), "/../../.claude/singlefs-ai-sop/scripts/preflight.py");
    let mut command = std::process::Command::new("python3");
    command.arg(script).arg("check").arg(source);
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

fn main() {
    preflight();
    let arguments: Vec<String> = std::env::args().skip(1).filter(|argument| argument != "--force").collect();
    if let Some(unknown) = arguments.first() {
        eprintln!("  ✗ 认不出的参数 {unknown}：这一版只有第一段，不带参数跑");
        eprintln!("  → 怎么办：第二段（K = n − 1、多 inode、S = 1、环槽 2）登记标「够判后未跑」，没有实现");
        std::process::exit(2);
    }
    let mut emitter = Emitter::new();
    emit_segment_one(&mut emitter);
}
