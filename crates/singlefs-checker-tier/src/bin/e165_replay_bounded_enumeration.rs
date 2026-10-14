//! E165（恢复重放有界穷举与变异杀伤界）：在小界内穷举输入，直接调 `singlefs_core::recovery::replay_journal`，
//! 照条款文字手写的判据逐条数违例（登记 `research/prompts/e165-preregistration.md`）。
//!
//! 被测对象只有 `replay_journal` 这一个函数的返回值；`records` 表由装置把内存盘上每个记录槽的字节过 `JournalRecord::parse` 建出来。
//! 装置调 core 的这几样（被测对象与它的真编码）：`replay_journal`、`JournalRecord::to_bytes` / `parse`、`back_chain_of`、
//! `crc32_castagnoli`、`record_offset`、`PoolReader`、`RootRecord`、`JournalRecord` 与它们字段的类型。判据一侧只用本地常量
//! 与生成历史时记下的真相（哪条记录属于哪次发布、每次发布的新根段、哪条在盘上），不调 core 的判定函数。
//!
//! 域（登记 5.2）：合法历史一族 L、L6、L9、U、X、E1、E2、R、C、C6；对抗一族 A、A6、AC。界由 `--bounds segment1|segment2|light` 给，
//! `--only <域>`（可写几个，逗号分隔）、`--max-n <N>`、`--family legal|adversarial|all` 只跑一部分。
//! 多盘域 M、M6、MC（登记 `research/prompts/e165-r2-prereg.md` 5.3，第三段）只在 `--profile r2 --family multi_disk` 时跑：每条记录在 0、1 号盘
//! 同一落点各一份、各自落或没落，「完整在盘上」取任一份，直接建表先 0 号盘再 1 号盘；`--family all` 仍只含合法历史与对抗两族。
//! 择根与重放合判一族 K、K9、KW、KA（登记 `research/prompts/e165-r3-prereg.md` 5.3 至 5.5）只在 `--profile r3 --family root_choice` 时跑：真 `choose_root`、`scan_journal`、`replay_journal` 三步串起来，判据 KJ0 至 KJ3 各报各的；`--family all` 不含它。带读故障的择根合判一族 KF、KF3、KF9、KFW、KG、KA（登记 `research/prompts/e165-r4-prereg.md` 5.3 至 5.6）只在 `--profile r4 --family root_choice_faults` 时跑：同样调真三步，多出每块盘两个系统配置槽、挂载开头的读故障、可写挂载读到的更新状态读不出那道闸（实现的取甲）与被判时的读故障，判据加 KJ1T，`--profile r4` 的其余族与 r3 档相同。
//! 线程数取环境变量 `E165_THREADS`，没设取每台默认重活线程数（`SINGLEFS_THREAD_CAP`，再没设取 `.claude/heavy-thread-default` 的登记值）；
//! 内联单测用同一个取法；计数与线程数无关。
//! 产物是 `E7RESULT` 行，收尾 `name=done emitted=N`；耗时另打 `E7TIMING` 行，不进逐字比对。
//! `light` 档的界：L 到 9、U 到 5、A 到 3，其余同第一段（2026-10-01 在 10 线程上量到每态挂钟 0.24–1.2 微秒，每个域取到 3 秒之内）。
//!
//! 同一份 4096 字节只过一次 `JournalRecord::parse`：生成一条记录时解一次、记下结果，之后每个落盘子集建表时取那一次的结果
//! （`parse` 是字节的纯函数）。
//!
//! 对抗域里序号 0 的记录自证校验和成立而 `JournalRecord::parse` 拒收（读到 0 当损坏），进不了表；装置逐态数出来报
//! （`states_holding_a_record_rejected_by_parse`）。停机条款 H4 照登记修订第 9 条：序号不是 0 的每条生成记录断言解得回原记录，
//! 对抗域字母表里序号 0 的记录断言 `parse` 拒收，没拒就停。
//!
//! 单状态重放（登记修订第 12 条）：`--replay-state <域>:<N>:<序号>` 只判枚举次序里的那一个状态，打一行 `name=state`
//! （这个状态让哪几个量加了一、实现的结局、状态的写法）；`--list-states <域>:<N>` 把这个 (域, N) 的每个状态照枚举次序各打一行。
//! 两种都单线程照工作单元的次序走，不跑自检与阳性对照，判据与正常跑是同一段代码。域名用产物里 `domain=` 的写法。
//!
//! 钉住的红例逐行重放：`--pinned-cases <表>` 读 `.claude/gate.d/e165-pinned-red-examples.tsv` 那种表（格式在它的文件头），
//! 按数据行的次序每行判一个状态、打一行 `name=state`（与 `--replay-state` 打的那一行是同一段代码）；按 ②③④ 找到状态之后
//! 拿它重算 ①（`sha256(<域>|<N>|<state= 的值，截到 ;first= 之前>)`），与表里不同、或有行找不到状态、或行本身不对
//! （不是二十一段、⑨ 起抄来的红例账那一行与 ②③④⑦⑧ 对不上），退 1。
//! 逐态行与 ⑦ 比不比得上由读这张表的一方（`scripts/experiments/e165-pin-red-examples.py --replay`）判，装置不判。

use std::cell::Cell;
use std::collections::{btree_map, BTreeMap};
use std::env;
use std::fs;
use std::ops::Range;
use std::panic::{self, AssertUnwindSafe};
use std::sync::atomic::{AtomicBool, AtomicU64, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Instant;

use singlefs_core::address::{
    CheckpointTxg, DeviceIdentity, DeviceOffsetInBytes, InstanceGeneration, SlotNumber,
    TreeIdentifier,
};
use singlefs_core::checksum::crc32_castagnoli;
use singlefs_core::journal::{
    back_chain_of, record_offset, JournalRecord, JournalRecordOrdinalWithinPublish,
    JournalRecordPlaceInPublish, NamedUnit, FIRST_JOURNAL_COUNTER,
};
use singlefs_core::pointer::{BirthSequence, LocationEntry, NodePointer, PointerHead};
use singlefs_core::recovery::{
    choose_root, every_root_ring_slot, read_root_ring_slot, replay_journal, scan_journal,
    BadRootRingSlotReading, PoolReader, RecoveryFailure, RootRingSlotReading,
    CHOSEN_ROOT_CHECKPOINT_TXG_HAS_NO_SUCCESSOR,
};
use singlefs_core::root_record::{RootRecord, UnmountMarker};
use singlefs_core::root_ring::RootRingSlotsPerRegion;
use singlefs_core::system_configuration::{
    journal_in_flight_record_limit, SystemConfiguration, SystemImmutableConfiguration,
    SystemImmutableSizes, SystemMutableConfiguration, SystemRuntimeConfiguration,
    SystemRuntimeQuantities,
};
use singlefs_harness::sha256::sha256_hexadecimal;

// ── 本地常量：值抄自 kb，不从 `crates/` 引；[`cross_check_local_constants_against_the_implementation`] 逐个回比（登记第三节、停机条款 H2）。

/// journal 记录字节数。抄自 `.claude/kb/layout/01-first-txn.md`「journal 环，两盘互为镜像，记录 4096」那一行与
/// D23（journal 的角色与格式） 已定项 12「记录尺寸取 4 KiB」。
const LOCAL_JOURNAL_RECORD_BYTES: u64 = 4096;
/// 在飞上限的安全系数 F。抄自 D23（journal 的角色与格式） 已定项 18「在飞记录数上限的取值与语义」（「在飞记录数上限 = 环槽数 ÷ F」，默认环下 196608 ÷ 3）。
const LOCAL_JOURNAL_SAFETY_FACTOR: u64 = 3;
/// journal 环起点槽号。抄自 `.claude/kb/layout/01-first-txn.md`「journal 环，两盘互为镜像，记录 4096」那一行（环从 16 MiB 起：16 MiB ÷ 16384 = 槽 1024）。
const LOCAL_JOURNAL_RING_START_SLOT: u64 = 1024;
/// 落点粒度：一个槽 16384 字节。抄自 `.claude/kb/layout/01-first-txn.md`「journal 环，两盘互为镜像，记录 4096」那一行的 16 MiB 与槽 1024 之比。
const LOCAL_SLOT_BYTES: u64 = 16384;
/// 第一个 jsn 计数器。抄自 `crates/singlefs-checker/invariants/03-journal.md`「`| I-8.6 |`」那一行（I-8.6：落点式 `(计数器 − 1) mod 槽数 × 4096` 从计数器 1 起）。
const LOCAL_FIRST_JOURNAL_COUNTER: u64 = 1;
/// 索引节点单元字节数。抄自 `.claude/kb/decisions/08-核心索引结构.md` 里的 `format-const: NODE_BYTES = 16384`。
const LOCAL_INDEX_NODE_UNIT_BYTES: u64 = 16384;
/// 点名项里索引节点的类标签（码 2）。抄自 `.claude/kb/decisions/23-journal的角色与格式.md` 已定项 17 的点名项字段表「类标签 1」与
/// D8（核心索引结构） 已定项 2「节点大小 16 KiB，钉成常量」那一条（码 2 是索引节点）。
const LOCAL_UNIT_CLASS_INDEX_NODE: u8 = 2;

/// 大环槽数（登记 5.1：64 槽，在飞上限 21）。
const BIG_RING_SLOTS: u64 = 64;
/// 两个小环（登记 5.1、5.2：6 槽上限 2，9 槽上限 3）。
const SMALL_RING_SIX_SLOTS: u64 = 6;
const SMALL_RING_NINE_SLOTS: u64 = 9;
/// 大环上历史的第一个计数器（登记 5.1：c_0 = 7，不回绕）；小环上取环槽数（历史的第二条记录落回槽 0）。
const BIG_RING_FIRST_HISTORY_COUNTER: u64 = 7;
/// 所选实例 g（登记 5.1）。
const CHOSEN_INSTANCE: u32 = 3;
/// 合法历史第一次发布的 txg（登记 5.1：t_0 = 100，E 域除外）。
const ORDINARY_FIRST_TXG: u64 = 100;
/// 记录头里那 8 字节 fsid（系统配置 fsid 的低 8 字节）；根记录的 16 字节 fsid 低 8 字节取同一个值。
const FILESYSTEM_IDENTIFIER_LOW_EIGHT_BYTES: u64 = 0x4531_3635_5245_504c;
const ROOT_FILESYSTEM_IDENTIFIER: [u8; 16] = [
    0x4c, 0x50, 0x45, 0x52, 0x35, 0x36, 0x31, 0x45, 0x2d, 0x45, 0x31, 0x36, 0x35, 0x2d, 0x30, 0x31,
];
/// 历史的第一条记录前面没有记录：反向链取一个固定的非 0 值（登记 5.1；它的前一条不在盘上，不判）。
const BACK_CHAIN_WHEN_THE_PREVIOUS_SLOT_HOLDS_NOTHING: u32 = 0x5eed_0165;
/// 时间线之外的树 ID 水位从这里起（残留、阳性对照的假根）；时间线上的取 1000 + 发布序号。
const OFF_TIMELINE_WATERMARK_BASE: u64 = 9_000_000;
/// 内存盘每块盘的字节数：只回答 `device_size_in_bytes`，`replay_journal` 不读它。
const MEMORY_DISK_DEVICE_BYTES: u64 = 1 << 40;
/// 点名单元住的槽从这里起（单元区里随便取的一段，两块盘同槽）。
const NAMED_UNIT_FIRST_SLOT: u64 = 60_000;

const THREADS_ENVIRONMENT_VARIABLE: &str = "E165_THREADS";

/// 环几何：只有槽数一个旋钮（登记第八节）。
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
struct RingGeometry {
    ring_slots: u64,
}

impl RingGeometry {
    const BIG: Self = Self {
        ring_slots: BIG_RING_SLOTS,
    };
    const SMALL_SIX: Self = Self {
        ring_slots: SMALL_RING_SIX_SLOTS,
    };
    const SMALL_NINE: Self = Self {
        ring_slots: SMALL_RING_NINE_SLOTS,
    };

    fn ring_bytes(self) -> u64 {
        self.ring_slots * LOCAL_JOURNAL_RECORD_BYTES
    }

    /// 在飞上限，装置自己算：环槽数 ÷ F 取整（I-8.3 补句「N = 环槽数 ÷ F」）。
    fn local_in_flight_limit(self) -> usize {
        usize::try_from(self.ring_slots / LOCAL_JOURNAL_SAFETY_FACTOR)
            .expect("环槽数不超过 64，商装得进 usize")
    }

    fn ring_start_offset_in_bytes() -> u64 {
        LOCAL_JOURNAL_RING_START_SLOT * LOCAL_SLOT_BYTES
    }

    /// 记录落点，装置自己算：环起点 + ((计数器 − 第一个计数器) mod 环槽数) × 记录字节数。
    fn local_record_offset_in_bytes(self, counter: u64) -> u64 {
        Self::ring_start_offset_in_bytes()
            + ((counter - LOCAL_FIRST_JOURNAL_COUNTER) % self.ring_slots)
                * LOCAL_JOURNAL_RECORD_BYTES
    }

    /// 历史的第一个计数器 c_0（登记 5.1）。
    fn first_history_counter(self) -> u64 {
        if self.ring_slots == BIG_RING_SLOTS {
            BIG_RING_FIRST_HISTORY_COUNTER
        } else {
            self.ring_slots
        }
    }
}

// ── 档（登记 r2 5.1）：r1 档的生成器、判据与输出行同 r1 时的装置；r2 档投影之外的根字段随版本变，另判 J6Qn、J7

/// r2 档卸载记号的奇偶（登记 r2 第一节、第八节；`--marker-parity even|odd`，不给取 even）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum MarkerParity {
    /// 偶数版 `WrittenByTheUnmountSequence`、奇数版 `NotWrittenByTheUnmountSequence`（R_0 是 `Written…`）。
    EvenVersionsWrittenByTheUnmountSequence,
    /// 反过来：奇数版 `Written…`、偶数版 `NotWritten…`（R_0 是 `NotWritten…`）。
    OddVersionsWrittenByTheUnmountSequence,
}

impl MarkerParity {
    const fn label(self) -> &'static str {
        match self {
            Self::EvenVersionsWrittenByTheUnmountSequence => "even",
            Self::OddVersionsWrittenByTheUnmountSequence => "odd",
        }
    }
}

/// 时间线上每一版根记录投影（六个字段）之外的四个字段怎么取。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum FieldsBeyondTheProjectionGenerator {
    /// r1 档：全历史相同（fsid 常量、卸载记号 `NotWritten…`、实例表槽 69 000、分配记录树根空）。
    SameForEveryVersion,
    /// r2 档（登记 r2 第一节）：第 j 版实例表指针取槽 69 000 + j、分配记录树根取槽 68 000 + j、fsid 末字节取 j，卸载记号按奇偶。
    VaryWithTheVersion(MarkerParity),
}

/// 判哪几条判据：r1 档只判原登记与 r1 登记的；r2 档另判 J6Qn 两行与 J7 五行（登记 r2 5.4）；r3 档在 r2 档之上，对抗一族的 J6Qn 按 I-8.12 再拆。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum CriteriaSet {
    RegisteredBeforeR2,
    WithJ6QnAndJ7,
    /// r3 档（登记 `research/prompts/e165-r3-prereg.md` 5.1、5.6）：r2 档全部判据原样，对抗一族的 J6Qn 再按 I-8.12 拆行。
    WithJ6QnAndJ7SplitByI812,
    /// r5 档（登记 `research/prompts/e165-r5-prereg.md` 5.1、5.3）：r3 档的判据原样，只把 J7 五行换成 J8 五行与「无出处」一行；
    /// 这一档生成的记录带它那次发布的根的后三项（D23（journal 的角色与格式） 已定项 15 今天的原句）。
    WithJ6QnAndJ8SplitByI812,
}

/// 判据一侧拿哪张记录表去判（登记 r2 5.4 T-表、T-行；`--table direct|scan|both`，不给取 `direct`，只在 r2 档有）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum RecordTableSource {
    /// 装置自己建的表：生成每条记录时解一次 `parse`，按盘次序逐槽收，`(实例, 计数器)` 先到先得（登记 5.1）。
    Direct,
    /// 把状态摆上内存盘，调 `scan_journal` 从盘上字节建表（登记 r2 第三节「由 scan_journal 建表」）。
    Scanned,
    /// 两张表各判一遍逐态比；汇总行取直接建表那一遍，另打 `name=table_comparison` 行。
    Both,
}

impl RecordTableSource {
    const fn label(self) -> &'static str {
        match self {
            Self::Direct => "direct",
            Self::Scanned => "scan",
            Self::Both => "both",
        }
    }
}

/// 内存盘的 `device_identities()` 报哪块盘在前（登记 r2 第八节；`--disk-order normal|reversed`，不给取 `normal`，只在 r2 档有）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ReportedDiskOrder {
    FirstDeviceBeforeSecond,
    SecondDeviceBeforeFirst,
}

impl ReportedDiskOrder {
    const fn label(self) -> &'static str {
        match self {
            Self::FirstDeviceBeforeSecond => "normal",
            Self::SecondDeviceBeforeFirst => "reversed",
        }
    }

    const fn device_numbers(self) -> [u32; 2] {
        match self {
            Self::FirstDeviceBeforeSecond => [0, 1],
            Self::SecondDeviceBeforeFirst => [1, 0],
        }
    }
}

/// 一次跑的档：生成器与判据两样，另有 r2 档第四段加的两个旋钮（记录表怎么建、盘次序）。`r1` 与 `r2` 是命令行给得出的两种；
/// 「r1 档生成器 + r2 判据」只给 J7 的阳性对照用（登记 r2 5.5「同一个对照在 r1 档生成器上 = 0」）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct RunProfile {
    generator: FieldsBeyondTheProjectionGenerator,
    criteria: CriteriaSet,
    table_source: RecordTableSource,
    reported_disk_order: ReportedDiskOrder,
}

/// 不给 `--profile` 时取的档（登记 r2 5.1）。
const DEFAULT_PROFILE_LABEL: &str = "r1";
/// r1 档时间线根的实例表单元指针住的槽；r2 档第 j 版取它加 j。
const INSTANCE_TABLE_SLOT_OF_THE_FIRST_VERSION: u64 = 69_000;
/// r2 档第 j 版分配记录树根住的槽：它加 j。
const ALLOCATION_RECORD_TREE_ROOT_SLOT_OF_THE_FIRST_VERSION: u64 = 68_000;
/// 16 字节 fsid 里 r2 档随版本变的那一个字节：末字节（低 8 字节不动，登记 r2 第一节）。
const FILESYSTEM_IDENTIFIER_BYTE_THAT_VARIES_WITH_THE_VERSION: usize = 15;

impl RunProfile {
    const R1: Self = Self {
        generator: FieldsBeyondTheProjectionGenerator::SameForEveryVersion,
        criteria: CriteriaSet::RegisteredBeforeR2,
        table_source: RecordTableSource::Direct,
        reported_disk_order: ReportedDiskOrder::FirstDeviceBeforeSecond,
    };

    const fn r2(marker_parity: MarkerParity) -> Self {
        Self {
            generator: FieldsBeyondTheProjectionGenerator::VaryWithTheVersion(marker_parity),
            criteria: CriteriaSet::WithJ6QnAndJ7,
            table_source: RecordTableSource::Direct,
            reported_disk_order: ReportedDiskOrder::FirstDeviceBeforeSecond,
        }
    }

    /// r3 档（登记 r3 5.1）：生成器与 r2 档相同，判据是 r2 档的全部再加对抗一族的 I-8.12 拆行。
    const fn r3(marker_parity: MarkerParity) -> Self {
        Self {
            generator: FieldsBeyondTheProjectionGenerator::VaryWithTheVersion(marker_parity),
            criteria: CriteriaSet::WithJ6QnAndJ7SplitByI812,
            table_source: RecordTableSource::Direct,
            reported_disk_order: ReportedDiskOrder::FirstDeviceBeforeSecond,
        }
    }

    /// r5 档（登记 r5 5.1）：时间线根的投影之外四个字段同 r2 / r3 档，生成的记录带它那次发布的根的后三项，判 J8 不判 J7。
    const fn r5(marker_parity: MarkerParity) -> Self {
        Self {
            generator: FieldsBeyondTheProjectionGenerator::VaryWithTheVersion(marker_parity),
            criteria: CriteriaSet::WithJ6QnAndJ8SplitByI812,
            table_source: RecordTableSource::Direct,
            reported_disk_order: ReportedDiskOrder::FirstDeviceBeforeSecond,
        }
    }

    /// 只给 J8 的阳性对照用（登记 r5 5.4 末列「r1 档那种生成器」）：生成器照 r1 档（时间线根与记录的后三项恒为 r1 常量），判据照 r5 档。
    const R1_GENERATOR_WITH_R5_CRITERIA: Self = Self {
        generator: FieldsBeyondTheProjectionGenerator::SameForEveryVersion,
        criteria: CriteriaSet::WithJ6QnAndJ8SplitByI812,
        table_source: RecordTableSource::Direct,
        reported_disk_order: ReportedDiskOrder::FirstDeviceBeforeSecond,
    };

    /// 只给 J7 的阳性对照用：生成器照 r1 档，判据照 r2 档。
    const R1_GENERATOR_WITH_R2_CRITERIA: Self = Self {
        generator: FieldsBeyondTheProjectionGenerator::SameForEveryVersion,
        criteria: CriteriaSet::WithJ6QnAndJ7,
        table_source: RecordTableSource::Direct,
        reported_disk_order: ReportedDiskOrder::FirstDeviceBeforeSecond,
    };

    const fn with_table_source(self, table_source: RecordTableSource) -> Self {
        Self {
            generator: self.generator,
            criteria: self.criteria,
            table_source,
            reported_disk_order: self.reported_disk_order,
        }
    }

    const fn with_reported_disk_order(self, reported_disk_order: ReportedDiskOrder) -> Self {
        Self {
            generator: self.generator,
            criteria: self.criteria,
            table_source: self.table_source,
            reported_disk_order,
        }
    }

    /// J6Qn 两行与多盘一族：r2、r3、r5 档判。
    const fn judges_j6qn(self) -> bool {
        match self.criteria {
            CriteriaSet::RegisteredBeforeR2 => false,
            CriteriaSet::WithJ6QnAndJ7
            | CriteriaSet::WithJ6QnAndJ7SplitByI812
            | CriteriaSet::WithJ6QnAndJ8SplitByI812 => true,
        }
    }

    /// J7 五行（照「投影之外抄所选根」登记的旧规则）：只在 r2、r3 档判（r5 登记 5.1：旧档一个字不改）。
    const fn judges_j7(self) -> bool {
        match self.criteria {
            CriteriaSet::RegisteredBeforeR2 | CriteriaSet::WithJ6QnAndJ8SplitByI812 => false,
            CriteriaSet::WithJ6QnAndJ7 | CriteriaSet::WithJ6QnAndJ7SplitByI812 => true,
        }
    }

    /// J8 五行与「无出处」一行（登记 r5 5.3）：只在 r5 档判；这一档生成的记录同时带它那次发布的根的后三项。
    const fn judges_j8(self) -> bool {
        match self.criteria {
            CriteriaSet::RegisteredBeforeR2
            | CriteriaSet::WithJ6QnAndJ7
            | CriteriaSet::WithJ6QnAndJ7SplitByI812 => false,
            CriteriaSet::WithJ6QnAndJ8SplitByI812 => true,
        }
    }

    /// 对抗一族的 J6Qn 按 I-8.12 再拆（登记 r3 5.6）：r3 档与 r5 档（r5 档 = r3 档原样，只换 J7 → J8）。
    const fn splits_j6qn_by_i812(self) -> bool {
        match self.criteria {
            CriteriaSet::RegisteredBeforeR2 | CriteriaSet::WithJ6QnAndJ7 => false,
            CriteriaSet::WithJ6QnAndJ7SplitByI812 | CriteriaSet::WithJ6QnAndJ8SplitByI812 => true,
        }
    }

    /// `--family root_choice` 认不认这一档：只认 r3 档（`--profile r3`、`--profile r4` 都解成它）；r5 档报用法错（登记 r5 5.1）。
    const fn admits_the_root_choice_family(self) -> bool {
        match self.criteria {
            CriteriaSet::RegisteredBeforeR2
            | CriteriaSet::WithJ6QnAndJ7
            | CriteriaSet::WithJ6QnAndJ8SplitByI812 => false,
            CriteriaSet::WithJ6QnAndJ7SplitByI812 => true,
        }
    }

    /// 两种建表法各判一遍逐态比（`--table both`）。
    const fn compares_the_two_record_tables(self) -> bool {
        matches!(self.table_source, RecordTableSource::Both)
    }

    /// 时间线第 `version` 版 R_j 投影之外的四个字段。
    fn fields_beyond_the_projection_of_version(self, version: usize) -> FieldsBeyondTheProjection {
        match self.generator {
            FieldsBeyondTheProjectionGenerator::SameForEveryVersion => FieldsBeyondTheProjection {
                filesystem_identifier: ROOT_FILESYSTEM_IDENTIFIER,
                unmount_marker: UnmountMarker::NotWrittenByTheUnmountSequence,
                instance_table: node_pointer_at_slot(INSTANCE_TABLE_SLOT_OF_THE_FIRST_VERSION, 1),
                allocation_record_tree_root: NodePointer::empty_root(),
            },
            FieldsBeyondTheProjectionGenerator::VaryWithTheVersion(marker_parity) => {
                let version_in_the_varying_fields = version;
                let version_number =
                    u64::try_from(version_in_the_varying_fields).expect("版本数不超过 12");
                let mut filesystem_identifier = ROOT_FILESYSTEM_IDENTIFIER;
                filesystem_identifier[FILESYSTEM_IDENTIFIER_BYTE_THAT_VARIES_WITH_THE_VERSION] =
                    u8::try_from(version_number).expect("版本数不超过 12，装得进一个字节");
                let version_is_even = version_number % 2 == 0;
                let written_by_the_unmount_sequence = match marker_parity {
                    MarkerParity::EvenVersionsWrittenByTheUnmountSequence => version_is_even,
                    MarkerParity::OddVersionsWrittenByTheUnmountSequence => !version_is_even,
                };
                FieldsBeyondTheProjection {
                    filesystem_identifier,
                    unmount_marker: if written_by_the_unmount_sequence {
                        UnmountMarker::WrittenByTheUnmountSequence
                    } else {
                        UnmountMarker::NotWrittenByTheUnmountSequence
                    },
                    instance_table: node_pointer_at_slot(
                        INSTANCE_TABLE_SLOT_OF_THE_FIRST_VERSION + version_number,
                        1,
                    ),
                    allocation_record_tree_root: node_pointer_at_slot(
                        ALLOCATION_RECORD_TREE_ROOT_SLOT_OF_THE_FIRST_VERSION + version_number,
                        1,
                    ),
                }
            }
        }
    }
}

/// r5 档时间线之外那条记录（对抗一族字母表里的、R 域的残留）的实例表指针住的槽：这个数加区分号 d（登记 r5 5.3 第 2 条 (i)：
/// 对 d 单射，与时间线第 0..12 版的槽 69 000 + j 两两不同）。
const OFF_TIMELINE_RECORD_INSTANCE_TABLE_SLOT_BASE: u64 = 200_000;
/// 同上，分配记录树根住的槽：这个数加 d（与时间线第 0..12 版的槽 68 000 + j 两两不同）。
const OFF_TIMELINE_RECORD_ALLOCATION_RECORD_TREE_ROOT_SLOT_BASE: u64 = 300_000;

/// 一条记录新根段的后三项（D23（journal 的角色与格式） 已定项 15：实例表指针、分配记录树根、卸载记号；fsid 不在新根段里）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct RecordFieldsBeyondTheProjection {
    instance_table: NodePointer,
    allocation_record_tree_root: NodePointer,
    unmount_marker: UnmountMarker,
}

impl RecordFieldsBeyondTheProjection {
    fn of_root_fields(fields: FieldsBeyondTheProjection) -> Self {
        Self {
            instance_table: fields.instance_table,
            allocation_record_tree_root: fields.allocation_record_tree_root,
            unmount_marker: fields.unmount_marker,
        }
    }

    fn of_record(record: &JournalRecord) -> Self {
        Self {
            instance_table: record.new_instance_table,
            allocation_record_tree_root: record.new_allocation_record_tree_root,
            unmount_marker: record.new_unmount_marker,
        }
    }

    fn of_root(root: &RootRecord) -> Self {
        Self {
            instance_table: root.instance_table,
            allocation_record_tree_root: root.allocation_record_tree_root,
            unmount_marker: root.unmount_marker,
        }
    }

    /// 见证里的写法：卸载记号、实例表与分配记录树根住的槽（空根是槽 0）。
    fn describe(self) -> String {
        format!(
            "marker_{}:instance_table_slot{}:allocation_record_tree_root_slot{}",
            match self.unmount_marker {
                UnmountMarker::WrittenByTheUnmountSequence => "written",
                UnmountMarker::NotWrittenByTheUnmountSequence => "not_written",
            },
            self.instance_table.locations[0].slot.0,
            self.allocation_record_tree_root.locations[0].slot.0
        )
    }
}

/// 卸载记号的另一个值（登记 r5 5.3 第 2 条 (ii)：时间线之外的记录取与所选根 R_0 相反的那一值）。
const fn the_other_unmount_marker(marker: UnmountMarker) -> UnmountMarker {
    match marker {
        UnmountMarker::WrittenByTheUnmountSequence => UnmountMarker::NotWrittenByTheUnmountSequence,
        UnmountMarker::NotWrittenByTheUnmountSequence => UnmountMarker::WrittenByTheUnmountSequence,
    }
}

impl RunProfile {
    /// 时间线上第 `publish_ordinal` 次发布（`NewRootSegment::of_timeline_publish` 那一个序号）的每条记录带的后三项（登记 r5 5.3 第 1 条）：
    /// r5 档是时间线第 j 版 R_j 的那三项；别的档一律是 r1 档时间线那一份常量（r1–r4 档一个字不改）。
    fn record_fields_of_the_timeline_publish(
        self,
        publish_ordinal: usize,
    ) -> RecordFieldsBeyondTheProjection {
        if self.judges_j8() {
            RecordFieldsBeyondTheProjection::of_root_fields(
                self.fields_beyond_the_projection_of_version(publish_ordinal),
            )
        } else {
            RecordFieldsBeyondTheProjection::of_root_fields(
                fields_beyond_the_projection_carried_by_every_generated_record(),
            )
        }
    }

    /// 时间线之外区分号为 `distinguishing_number` 的那条记录带的后三项 h(d)（登记 r5 5.3 第 2 条）：r5 档且生成器随版本变时，
    /// 实例表指针、分配记录树根各住 d 的单射槽，卸载记号取与 R_0 相反的那一值；别的档与 r1 档那种生成器上是 r1 常量。
    fn record_fields_off_the_timeline(
        self,
        distinguishing_number: u64,
    ) -> RecordFieldsBeyondTheProjection {
        let r1_constant = RecordFieldsBeyondTheProjection::of_root_fields(
            fields_beyond_the_projection_carried_by_every_generated_record(),
        );
        if !self.judges_j8() {
            return r1_constant;
        }
        match self.generator {
            FieldsBeyondTheProjectionGenerator::SameForEveryVersion => r1_constant,
            FieldsBeyondTheProjectionGenerator::VaryWithTheVersion(_marker_parity) => {
                let marker_of_the_first_version = self
                    .fields_beyond_the_projection_of_version(0)
                    .unmount_marker;
                RecordFieldsBeyondTheProjection {
                    instance_table: node_pointer_at_slot(
                        OFF_TIMELINE_RECORD_INSTANCE_TABLE_SLOT_BASE + distinguishing_number,
                        1,
                    ),
                    allocation_record_tree_root: node_pointer_at_slot(
                        OFF_TIMELINE_RECORD_ALLOCATION_RECORD_TREE_ROOT_SLOT_BASE
                            + distinguishing_number,
                        1,
                    ),
                    unmount_marker: the_other_unmount_marker(marker_of_the_first_version),
                }
            }
        }
    }
}

/// 装置生成的每条记录的新根段后三项（实例表指针、分配记录树根、卸载记号；D23（journal 的角色与格式） 已定项 15，2026-10-02 用户定
/// 「回到甲」之后施加一条记录时这三项取记录里的，不再抄所选根）：r1–r4 档一律带 r1 档时间线那一份常量。r1 档的所选根投影之外的字段就是这几个常量，
/// 施加结果与改之前逐态相同；r2 / r3 档的投影之外字段随版本变，J7 那几行判据照「投影之外抄所选根」登记，与新的施加规则对不上
/// （旧档一个字不改，r5 登记 5.1）。照今天原句重新登记的是 r5 档：记录带它那次发布的根的后三项（[`RunProfile::record_fields_of_the_timeline_publish`]），判 J8。
fn fields_beyond_the_projection_carried_by_every_generated_record() -> FieldsBeyondTheProjection {
    RunProfile::R1.fields_beyond_the_projection_of_version(0)
}

/// 根记录里投影之外的四个字段（登记 r2 第一节「投影之外的根字段」）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct FieldsBeyondTheProjection {
    filesystem_identifier: [u8; 16],
    unmount_marker: UnmountMarker,
    instance_table: NodePointer,
    allocation_record_tree_root: NodePointer,
}

impl FieldsBeyondTheProjection {
    fn of_root(root: &RootRecord) -> Self {
        Self {
            filesystem_identifier: root.filesystem_identifier,
            unmount_marker: root.unmount_marker,
            instance_table: root.instance_table,
            allocation_record_tree_root: root.allocation_record_tree_root,
        }
    }

    /// 见证里的写法：卸载记号、实例表与分配记录树根住的槽（空根是槽 0）、fsid 末字节。
    fn describe(self) -> String {
        format!(
            "marker_{}:instance_table_slot{}:allocation_record_tree_root_slot{}:fsid_last_byte{}",
            match self.unmount_marker {
                UnmountMarker::WrittenByTheUnmountSequence => "written",
                UnmountMarker::NotWrittenByTheUnmountSequence => "not_written",
            },
            self.instance_table.locations[0].slot.0,
            self.allocation_record_tree_root.locations[0].slot.0,
            self.filesystem_identifier[FILESYSTEM_IDENTIFIER_BYTE_THAT_VARIES_WITH_THE_VERSION]
        )
    }
}

/// 系统配置 fsid 低 8 字节的本地取法：16 字节 fsid 的前 8 字节按小端读（登记 r2 第三节回比表；D22 已定项 13「盘上定长字段一律小端」）。
/// [`check_the_fields_beyond_the_projection_of_the_r2_generator`] 拿它与 `singlefs_core::unit::unit_filesystem_identifier` 回比。
fn local_filesystem_identifier_low_eight_bytes(filesystem_identifier: &[u8; 16]) -> u64 {
    let mut low_eight_bytes = [0u8; 8];
    low_eight_bytes.copy_from_slice(&filesystem_identifier[..8]);
    u64::from_le_bytes(low_eight_bytes)
}

// ── 内存盘（读者）

/// 内存盘某个位置上放着什么：读得出的字节，或一份读不出的（`read` 返回 `None`）。
#[derive(Clone, Copy)]
enum StoredBytes<'bytes> {
    Readable(&'bytes [u8]),
    Unreadable,
}

/// `replay_journal` 的读者：两块盘；环里没落盘的记录槽读出来是 4096 个 0（登记 5.1），别处没写过的读不出。
/// `device_identities()` 报哪块盘在前由 `reported_disk_order` 定（登记 r2 第八节「盘次序」）。
struct MemoryDisk<'bytes> {
    ring: RingGeometry,
    reported_disk_order: ReportedDiskOrder,
    contents: Vec<(u32, u64, StoredBytes<'bytes>)>,
}

impl PoolReader for MemoryDisk<'_> {
    fn device_identities(&self) -> Vec<DeviceIdentity> {
        self.reported_disk_order
            .device_numbers()
            .into_iter()
            .map(DeviceIdentity)
            .collect()
    }

    fn device_size_in_bytes(&self, device: DeviceIdentity) -> Option<u64> {
        (device.0 < 2).then_some(MEMORY_DISK_DEVICE_BYTES)
    }

    fn read(
        &self,
        device: DeviceIdentity,
        offset: DeviceOffsetInBytes,
        length: usize,
    ) -> Option<Vec<u8>> {
        for (stored_device, stored_offset, stored) in &self.contents {
            if *stored_device == device.0 && *stored_offset == offset.0 {
                return match stored {
                    StoredBytes::Readable(bytes) => bytes.get(..length).map(<[u8]>::to_vec),
                    StoredBytes::Unreadable => None,
                };
            }
        }
        let ring_start = RingGeometry::ring_start_offset_in_bytes();
        let is_a_whole_record_slot_of_the_ring = device.0 < 2
            && offset.0 >= ring_start
            && offset.0 < ring_start + self.ring.ring_bytes()
            && (offset.0 - ring_start).is_multiple_of(LOCAL_JOURNAL_RECORD_BYTES)
            && u64::try_from(length).ok() == Some(LOCAL_JOURNAL_RECORD_BYTES);
        is_a_whole_record_slot_of_the_ring.then(|| vec![0u8; length])
    }

    fn journal_record_offsets_hint(
        &self,
        _device: DeviceIdentity,
        _ring_start: DeviceOffsetInBytes,
        _ring_bytes: u64,
    ) -> Option<Vec<DeviceOffsetInBytes>> {
        None
    }
}

// ── 新根段与时间线上的一版

/// 记录头里的新根段四个字段（D23（journal 的角色与格式） 已定项 15）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct NewRootSegment {
    tree_table: NodePointer,
    mapping_root: NodePointer,
    tree_identifier_watermark: u64,
    rollback_floor: u64,
}

/// 一条过得了 `to_bytes` / `parse` 往返的节点指针：两条位置条目分在 0、1 号盘同一个槽，槽号取 `slot`。
fn node_pointer_at_slot(slot: u64, birth_txg: u64) -> NodePointer {
    let location_on = |device: u32| LocationEntry {
        device: DeviceIdentity(device),
        slot: SlotNumber(slot),
        unit_checksum: u32::try_from(slot % 0xffff_fff1).expect("取过模，装得进 32 位"),
    };
    NodePointer {
        head: PointerHead {
            birth_tree: TreeIdentifier(11),
            birth_txg: CheckpointTxg(birth_txg),
        },
        locations: [location_on(0), location_on(1)],
        instance: InstanceGeneration(CHOSEN_INSTANCE),
        birth_sequence: BirthSequence(1),
    }
}

impl NewRootSegment {
    /// 时间线上第 `publish_ordinal` 次发布的新根段（登记 5.1：树 ID 水位 1000 + i，F 取 10 + i，两条指针两两不同）。
    fn of_timeline_publish(publish_ordinal: u64, checkpoint_txg: u64) -> Self {
        Self {
            tree_table: node_pointer_at_slot(70_000 + publish_ordinal, checkpoint_txg),
            mapping_root: node_pointer_at_slot(80_000 + publish_ordinal, checkpoint_txg),
            tree_identifier_watermark: 1000 + publish_ordinal,
            rollback_floor: 10 + publish_ordinal,
        }
    }

    /// 时间线之外的新根段：`distinguishing_number` 不同的两份四个字段都不同，与时间线上的任何一版也不同。
    fn off_timeline(distinguishing_number: u64, checkpoint_txg: u64) -> Self {
        Self {
            tree_table: node_pointer_at_slot(90_000 + distinguishing_number, checkpoint_txg),
            mapping_root: node_pointer_at_slot(95_000_000 + distinguishing_number, checkpoint_txg),
            tree_identifier_watermark: OFF_TIMELINE_WATERMARK_BASE + distinguishing_number,
            rollback_floor: 5_000_000 + distinguishing_number,
        }
    }
}

/// 判据比的六个字段（登记 5.3「落点 j」：实例、txg、树表指针、映射根指针、树 ID 水位、F）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct VersionFields {
    instance: u32,
    checkpoint_txg: u64,
    segment: NewRootSegment,
}

impl VersionFields {
    fn of_root(root: &RootRecord) -> Self {
        Self {
            instance: root.instance.0,
            checkpoint_txg: root.checkpoint_txg.0,
            segment: NewRootSegment {
                tree_table: root.tree_table,
                mapping_root: root.mapping_root,
                tree_identifier_watermark: root.tree_identifier_watermark,
                rollback_floor: root.rollback_floor.0,
            },
        }
    }

    fn of_record(record: &JournalRecord) -> Self {
        Self {
            instance: record.instance.0,
            checkpoint_txg: record.checkpoint_txg.0,
            segment: NewRootSegment {
                tree_table: record.new_tree_table,
                mapping_root: record.new_mapping_root,
                tree_identifier_watermark: record.new_tree_identifier_watermark,
                rollback_floor: record.new_rollback_floor.0,
            },
        }
    }

    /// 这六个字段放进一条根记录，投影之外的四个字段取 `beyond`。
    fn to_root_with(self, beyond: FieldsBeyondTheProjection) -> RootRecord {
        RootRecord {
            filesystem_identifier: beyond.filesystem_identifier,
            unmount_marker: beyond.unmount_marker,
            instance: InstanceGeneration(self.instance),
            checkpoint_txg: CheckpointTxg(self.checkpoint_txg),
            tree_table: self.segment.tree_table,
            tree_identifier_watermark: self.segment.tree_identifier_watermark,
            rollback_floor: CheckpointTxg(self.segment.rollback_floor),
            instance_table: beyond.instance_table,
            mapping_root: self.segment.mapping_root,
            allocation_record_tree_root: beyond.allocation_record_tree_root,
        }
    }

    /// 由记录重建的根（登记 r2 第一节，S1 的形态）：投影取这六个字段，实例表指针、分配记录树根、fsid 抄 `chosen_root`，
    /// 卸载记号「不是卸载那一串写下的」。r1 档生成器上它与 r1 时的 `to_root` 逐字段相同（所选根的这四个字段就是那几个常量）。
    fn to_root_rebuilt_over(self, chosen_root: &RootRecord) -> RootRecord {
        self.to_root_with(FieldsBeyondTheProjection {
            filesystem_identifier: chosen_root.filesystem_identifier,
            unmount_marker: UnmountMarker::NotWrittenByTheUnmountSequence,
            instance_table: chosen_root.instance_table,
            allocation_record_tree_root: chosen_root.allocation_record_tree_root,
        })
    }

    /// 照 D23（journal 的角色与格式） 已定项 15 今天的原句由记录重建的根（登记 r5 5.3 J8a、J8b）：投影取这六个字段，实例表指针、
    /// 分配记录树根、卸载记号取施加末条那一份记录新根段里的 `record_fields`，fsid 抄 `chosen_root`。r5 档的理想恢复用它。
    fn to_root_rebuilt_from_the_record_over(
        self,
        record_fields: RecordFieldsBeyondTheProjection,
        chosen_root: &RootRecord,
    ) -> RootRecord {
        self.to_root_with(FieldsBeyondTheProjection {
            filesystem_identifier: chosen_root.filesystem_identifier,
            unmount_marker: record_fields.unmount_marker,
            instance_table: record_fields.instance_table,
            allocation_record_tree_root: record_fields.allocation_record_tree_root,
        })
    }
}

// ── 生成一条记录槽

/// 生成器造的记录过 `parse` 时该是什么结局（登记停机条款 H4）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ParseExpectation {
    /// 合法历史一族：`to_bytes` 再 `parse` 必须等于原记录，不等当场停。
    MustRoundTrip,
    /// 对抗域字母表里的记录（登记修订第 9 条）：序号不是 0 的照 H4 原文必须解得回原记录；序号 0 的 `parse` 必须拒收
    /// （D23 已定项 4：读者当那条记录损坏），没拒就停。
    OrdinalZeroMustBeRejected,
}

/// 内存盘上一个记录槽里的一条记录：字节、`parse` 的结果、它的 487 字节头算出的链值（下一条记录的反向链真值）。
#[derive(Clone, Debug)]
struct GeneratedSlot {
    device: u32,
    offset_in_bytes: u64,
    bytes: Vec<u8>,
    parsed: Option<JournalRecord>,
    chain_value_of_header: u32,
}

fn generate_slot(
    device: u32,
    ring: RingGeometry,
    record: &JournalRecord,
    expectation: ParseExpectation,
) -> GeneratedSlot {
    let bytes = record.to_bytes();
    let parsed = JournalRecord::parse(&bytes, FILESYSTEM_IDENTIFIER_LOW_EIGHT_BYTES);
    match expectation {
        ParseExpectation::MustRoundTrip => assert_eq!(
            parsed.as_ref(),
            Some(record),
            "停机条款 H4：生成的记录过不了 JournalRecord::parse，或解回来不等于原记录"
        ),
        ParseExpectation::OrdinalZeroMustBeRejected => {
            if record.ordinal_within_publish.0 == 0 {
                assert!(
                    parsed.is_none(),
                    "停机条款 H4（登记修订第 9 条）：序号 0 的生成记录 JournalRecord::parse 没有拒收"
                );
            } else {
                assert_eq!(
                    parsed.as_ref(),
                    Some(record),
                    "停机条款 H4：序号不是 0 的生成记录过不了 JournalRecord::parse，或解回来不等于原记录"
                );
            }
        }
    }
    let offset_in_bytes = ring.local_record_offset_in_bytes(record.counter);
    assert_eq!(
        offset_in_bytes,
        record_offset(record.counter, ring.ring_bytes()).0,
        "停机条款 H2：装置自己算的记录落点与 record_offset 不等（计数器 {}，环 {} 槽）",
        record.counter,
        ring.ring_slots
    );
    GeneratedSlot {
        device,
        offset_in_bytes,
        chain_value_of_header: back_chain_of(&bytes),
        bytes,
        parsed,
    }
}

// ── 点名单元（U 域）

/// 一条在盘上的记录点名的那个单元的五态（登记 5.2 的 U 域）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum UnitState {
    BothCopiesGood,
    FirstDeviceCopyUnreadable,
    SecondDeviceCopyUnreadable,
    FirstDeviceCopyChecksumWrong,
    SecondDeviceCopyChecksumWrong,
}

impl UnitState {
    const ALL: [Self; 5] = [
        Self::BothCopiesGood,
        Self::FirstDeviceCopyUnreadable,
        Self::SecondDeviceCopyUnreadable,
        Self::FirstDeviceCopyChecksumWrong,
        Self::SecondDeviceCopyChecksumWrong,
    ];

    const fn label(self) -> &'static str {
        match self {
            Self::BothCopiesGood => "good",
            Self::FirstDeviceCopyUnreadable => "d0_unreadable",
            Self::SecondDeviceCopyUnreadable => "d1_unreadable",
            Self::FirstDeviceCopyChecksumWrong => "d0_checksum_wrong",
            Self::SecondDeviceCopyChecksumWrong => "d1_checksum_wrong",
        }
    }
}

/// 一条记录点名的那个 16384 字节索引节点单元：好的一份与校验和不对的一份（翻了第一个字节）。
#[derive(Clone, Debug)]
struct GeneratedUnit {
    offset_in_bytes: u64,
    good_bytes: Vec<u8>,
    bytes_with_wrong_checksum: Vec<u8>,
    named: NamedUnit,
}

fn generate_unit(record_index: usize, checkpoint_txg: u64) -> GeneratedUnit {
    let unit_bytes = usize::try_from(LOCAL_INDEX_NODE_UNIT_BYTES).expect("16384 装得进 usize");
    let good_bytes: Vec<u8> = (0..unit_bytes)
        .map(|position| {
            u8::try_from((record_index * 31 + position * 7 + 1) % 251).expect("取过模 251")
        })
        .collect();
    let mut bytes_with_wrong_checksum = good_bytes.clone();
    bytes_with_wrong_checksum[0] ^= 0xff;
    let slot = NAMED_UNIT_FIRST_SLOT + u64::try_from(record_index).expect("记录下标装得进 u64");
    let unit_checksum = crc32_castagnoli(&good_bytes);
    let location_on = |device: u32| LocationEntry {
        device: DeviceIdentity(device),
        slot: SlotNumber(slot),
        unit_checksum,
    };
    GeneratedUnit {
        offset_in_bytes: slot * LOCAL_SLOT_BYTES,
        good_bytes,
        bytes_with_wrong_checksum,
        named: NamedUnit {
            locations: [location_on(0), location_on(1)],
            unit_class: LOCAL_UNIT_CLASS_INDEX_NODE,
            birth_tree: TreeIdentifier(11),
            birth_txg: CheckpointTxg(checkpoint_txg),
            key_tail: [u8::try_from(record_index % 251).expect("取过模 251"); 10],
        },
    }
}

// ── 合法历史的生成器（可信件，登记 5.1）

/// 一次发布：几条记录、提交标记怎么摆。`commit_marker_bits` 的第 r 位为 1 表示这次发布的第 r 条（从 0 数，末条之外）带提交标记；末条必带。
#[derive(Clone, Debug, PartialEq, Eq)]
struct PublishShape {
    record_count: usize,
    commit_marker_bits: u32,
}

/// 一段合法历史的计划：所选实例 g 的发布序列，X 域另接实例 g + 1 的发布序列。
#[derive(Clone, Debug)]
struct HistoryPlan {
    ring: RingGeometry,
    first_txg: u64,
    chosen_instance_publishes: Vec<PublishShape>,
    next_instance_publishes: Vec<PublishShape>,
    names_units: bool,
    run_profile: RunProfile,
}

/// `total` 拆成有序的正整数之和的全部拆法，每一份不超过 `largest_part`（没给就不限）。
fn compositions_of(total: usize, largest_part: Option<usize>) -> Vec<Vec<usize>> {
    if total == 0 {
        return vec![Vec::new()];
    }
    let mut compositions = Vec::new();
    for first_part in 1..=total {
        if largest_part.is_some_and(|largest| first_part > largest) {
            break;
        }
        for rest in compositions_of(total - first_part, largest_part) {
            let mut composition = vec![first_part];
            composition.extend(rest);
            compositions.push(composition);
        }
    }
    compositions
}

/// `total_records` 条记录切成发布的全部切法 × 每次发布提交标记的全部摆法（每次发布 2^(k − 1) 种）。
fn publish_sequences_of(
    total_records: usize,
    largest_publish: Option<usize>,
) -> Vec<Vec<PublishShape>> {
    let mut sequences = Vec::new();
    for composition in compositions_of(total_records, largest_publish) {
        let mut partial_sequences: Vec<Vec<PublishShape>> = vec![Vec::new()];
        for record_count in composition {
            let mut extended = Vec::new();
            for partial in &partial_sequences {
                for commit_marker_bits in 0..(1u32 << (record_count - 1)) {
                    let mut sequence = partial.clone();
                    sequence.push(PublishShape {
                        record_count,
                        commit_marker_bits,
                    });
                    extended.push(sequence);
                }
            }
            partial_sequences = extended;
        }
        sequences.extend(partial_sequences);
    }
    sequences
}

/// 生成历史时记下的一次发布的真相。
#[derive(Clone, Debug)]
struct PublishTruth {
    first_record_index: usize,
    record_count: usize,
    fields: VersionFields,
}

/// 生成器记下的真相（登记 r5 5.3 第 3 条）：每条生成的记录的新根段投影（实例代号、txg、树表、映射根、水位、F）对到它的后三项。
/// 合法写路径里同一投影只对一份后三项；记进来时同一投影出现两份不同的后三项就停（停机条款 PH6）。键是投影里的树 ID 水位，
/// 同一个水位下逐份比整个投影。
#[derive(Clone, Debug)]
struct GeneratedRecordTruth {
    by_tree_identifier_watermark:
        BTreeMap<u64, Vec<(VersionFields, RecordFieldsBeyondTheProjection)>>,
}

impl GeneratedRecordTruth {
    fn new() -> Self {
        Self {
            by_tree_identifier_watermark: BTreeMap::new(),
        }
    }

    /// 记下一条生成的记录：它的投影与后三项。
    fn note(&mut self, projection: VersionFields, fields: RecordFieldsBeyondTheProjection) {
        let entries = self
            .by_tree_identifier_watermark
            .entry(projection.segment.tree_identifier_watermark)
            .or_default();
        match entries
            .iter()
            .find(|(noted_projection, _noted_fields)| *noted_projection == projection)
        {
            Some((_noted_projection, noted_fields)) => assert!(
                *noted_fields == fields,
                "停机条款 PH6（登记 r5 5.3 第 3 条）：生成器真相里同一投影（实例 {}、txg {}、水位 {}）对到两份不同的后三项：{} 与 {}",
                projection.instance,
                projection.checkpoint_txg,
                projection.segment.tree_identifier_watermark,
                noted_fields.describe(),
                fields.describe()
            ),
            None => entries.push((projection, fields)),
        }
    }

    fn note_the_record(&mut self, record: &JournalRecord) {
        self.note(
            VersionFields::of_record(record),
            RecordFieldsBeyondTheProjection::of_record(record),
        );
    }

    /// 新根段投影等于 `projection` 的那条生成记录的后三项；生成器一条都没造过这个投影是 `None`（J8「无出处」）。
    fn fields_of(&self, projection: VersionFields) -> Option<RecordFieldsBeyondTheProjection> {
        self.by_tree_identifier_watermark
            .get(&projection.segment.tree_identifier_watermark)
            .and_then(|entries| {
                entries
                    .iter()
                    .find(|(noted_projection, _noted_fields)| *noted_projection == projection)
                    .map(|(_noted_projection, noted_fields)| *noted_fields)
            })
    }
}

/// 一段生成出来的历史：每条记录的槽（都在 0 号盘）、哪条属于哪次发布、每次发布的六个字段、点名单元。
#[derive(Clone, Debug)]
struct History {
    ring: RingGeometry,
    slots: Vec<GeneratedSlot>,
    publishes: Vec<PublishTruth>,
    /// 所选实例 g 的发布数：时间线 R_0 … R_(这个数 − 1)。X 域里实例 g + 1 的发布排在它们之后，不在时间线上。
    chosen_instance_publish_count: usize,
    units: Vec<Option<GeneratedUnit>>,
    shape_description: String,
    run_profile: RunProfile,
    /// 每条生成记录的投影对到它的后三项（登记 r5 5.3 第 3 条）；R 域的残留在 [`tally_of_a_legal_history`] 里补记进来。
    record_truth: GeneratedRecordTruth,
}

fn describe_publish_sequence(publishes: &[PublishShape]) -> String {
    publishes
        .iter()
        .map(|shape| format!("{}c{:b}", shape.record_count, shape.commit_marker_bits))
        .collect::<Vec<String>>()
        .join("+")
}

fn generate_history(plan: &HistoryPlan) -> History {
    let mut slots: Vec<GeneratedSlot> = Vec::new();
    let mut publishes: Vec<PublishTruth> = Vec::new();
    let mut units: Vec<Option<GeneratedUnit>> = Vec::new();
    let mut record_truth = GeneratedRecordTruth::new();
    let mut counter = plan.ring.first_history_counter();
    let mut publish_ordinal: u64 = 0;
    let instance_segments = [
        (CHOSEN_INSTANCE, &plan.chosen_instance_publishes),
        (CHOSEN_INSTANCE + 1, &plan.next_instance_publishes),
    ];
    for (instance, publishes_of_the_instance) in instance_segments {
        // 事务号按实例计数、从 1 起（D23（journal 的角色与格式） 已定项 7）。
        let mut transaction: u64 = 1;
        let mut is_first_record_of_a_later_instance = instance != CHOSEN_INSTANCE;
        for shape in publishes_of_the_instance {
            let checkpoint_txg = plan
                .first_txg
                .checked_add(publish_ordinal)
                .expect("E 域的发布数照登记 5.2 取，txg 不越过 u64");
            let segment_of_this_publish =
                NewRootSegment::of_timeline_publish(publish_ordinal, checkpoint_txg);
            // 新根段后三项：r5 档是这次发布的根 R_j 的那三项，别的档是 r1 常量（登记 r5 5.3 第 1 条）。
            let record_fields_of_this_publish =
                plan.run_profile.record_fields_of_the_timeline_publish(
                    usize::try_from(publish_ordinal).expect("发布序号很小"),
                );
            let first_record_index = slots.len();
            for record_position in 0..shape.record_count {
                let is_last_record_of_the_publish = record_position + 1 == shape.record_count;
                let is_commit = is_last_record_of_the_publish
                    || (shape.commit_marker_bits >> record_position) & 1 == 1;
                // 反向链取真值：全历史里前一条记录的字节算出的链值，不管前一条落没落盘；新实例的首条恒 0（I-8.6）。
                let back_chain = if is_first_record_of_a_later_instance {
                    0
                } else {
                    match slots.last() {
                        Some(previous_slot) => previous_slot.chain_value_of_header,
                        None => BACK_CHAIN_WHEN_THE_PREVIOUS_SLOT_HOLDS_NOTHING,
                    }
                };
                is_first_record_of_a_later_instance = false;
                // 每条记录带它那次发布的新根段（D23（journal 的角色与格式） 已定项 15）。
                let segment = segment_of_this_publish;
                let record_index = slots.len();
                let unit = plan
                    .names_units
                    .then(|| generate_unit(record_index, checkpoint_txg));
                let record = JournalRecord {
                    instance: InstanceGeneration(instance),
                    counter,
                    checkpoint_txg: CheckpointTxg(checkpoint_txg),
                    transaction,
                    is_commit,
                    ordinal_within_publish: JournalRecordOrdinalWithinPublish(
                        u32::try_from(record_position + 1).expect("一次发布的记录条数不超过 10"),
                    ),
                    place_in_publish: if is_last_record_of_the_publish {
                        JournalRecordPlaceInPublish::LastRecordOfThePublish
                    } else {
                        JournalRecordPlaceInPublish::MoreRecordsOfThePublishFollow
                    },
                    back_chain,
                    filesystem_identifier: FILESYSTEM_IDENTIFIER_LOW_EIGHT_BYTES,
                    new_tree_table: segment.tree_table,
                    new_mapping_root: segment.mapping_root,
                    new_tree_identifier_watermark: segment.tree_identifier_watermark,
                    new_rollback_floor: CheckpointTxg(segment.rollback_floor),
                    new_instance_table: record_fields_of_this_publish.instance_table,
                    new_allocation_record_tree_root: record_fields_of_this_publish
                        .allocation_record_tree_root,
                    new_unmount_marker: record_fields_of_this_publish.unmount_marker,
                    named: unit
                        .as_ref()
                        .map(|generated_unit| vec![generated_unit.named])
                        .unwrap_or_default(),
                };
                record_truth.note_the_record(&record);
                slots.push(generate_slot(
                    0,
                    plan.ring,
                    &record,
                    ParseExpectation::MustRoundTrip,
                ));
                units.push(unit);
                if is_commit {
                    transaction += 1;
                }
                counter += 1;
            }
            publishes.push(PublishTruth {
                first_record_index,
                record_count: shape.record_count,
                fields: VersionFields {
                    instance,
                    checkpoint_txg,
                    segment: segment_of_this_publish,
                },
            });
            publish_ordinal += 1;
        }
    }
    let shape_description = if plan.next_instance_publishes.is_empty() {
        describe_publish_sequence(&plan.chosen_instance_publishes)
    } else {
        format!(
            "{}/next_instance:{}",
            describe_publish_sequence(&plan.chosen_instance_publishes),
            describe_publish_sequence(&plan.next_instance_publishes)
        )
    };
    History {
        ring: plan.ring,
        slots,
        publishes,
        chosen_instance_publish_count: plan.chosen_instance_publishes.len(),
        units,
        shape_description,
        run_profile: plan.run_profile,
        record_truth,
    }
}

impl History {
    fn version_fields(&self, version: usize) -> VersionFields {
        self.publishes[version].fields
    }

    /// 时间线第 `version` 版的根 R_j：六个字段照这次发布，投影之外的四个字段照档（登记 r2 第一节）。
    fn timeline_root(&self, version: usize) -> RootRecord {
        self.version_fields(version).to_root_with(
            self.run_profile
                .fields_beyond_the_projection_of_version(version),
        )
    }

    fn record_indexes_of_publish(&self, publish_index: usize) -> Range<usize> {
        let publish = &self.publishes[publish_index];
        publish.first_record_index..publish.first_record_index + publish.record_count
    }

    /// 返回的根逐字段等于时间线上哪一版（登记 5.3「落点 j」）；没有这样的一版是 `None`。
    fn timeline_version_with_fields(&self, fields: VersionFields) -> Option<usize> {
        (0..self.chosen_instance_publish_count)
            .find(|version| self.version_fields(*version) == fields)
    }
}

/// R 域的那一条残留：放在 1 号盘，与 0 号盘上那条记录同 (实例, 计数器)、同 txg、同序号、同标志，新根段取时间线之外的值，
/// 自证过，反向链照真值（登记 5.2）。
#[derive(Clone, Debug)]
struct Residue {
    record_index: usize,
    slot: GeneratedSlot,
    /// 生成器这一侧的那条残留记录（进生成器真相，登记 r5 5.3 第 3 条）。
    record: JournalRecord,
}

fn generate_residue(history: &History, record_index: usize) -> Residue {
    let mut record = history.slots[record_index]
        .parsed
        .clone()
        .expect("合法历史的每条记录都解得出（H4 在生成时断言过）");
    let off_timeline_segment = NewRootSegment::off_timeline(
        u64::try_from(record_index).expect("记录下标装得进 u64"),
        record.checkpoint_txg.0,
    );
    record.new_tree_table = off_timeline_segment.tree_table;
    record.new_mapping_root = off_timeline_segment.mapping_root;
    record.new_tree_identifier_watermark = off_timeline_segment.tree_identifier_watermark;
    record.new_rollback_floor = CheckpointTxg(off_timeline_segment.rollback_floor);
    // 后三项：r5 档是区分号 d = 记录下标的 h(d)，别的档仍是 r1 常量（登记 r5 5.3 第 2 条）。
    let record_fields = history
        .run_profile
        .record_fields_off_the_timeline(u64::try_from(record_index).expect("记录下标装得进 u64"));
    record.new_instance_table = record_fields.instance_table;
    record.new_allocation_record_tree_root = record_fields.allocation_record_tree_root;
    record.new_unmount_marker = record_fields.unmount_marker;
    Residue {
        record_index,
        slot: generate_slot(1, history.ring, &record, ParseExpectation::MustRoundTrip),
        record,
    }
}

// ── 合法历史一族的一个状态

/// 0 号盘上一条记录在不在；在的话它点名的单元是五态里哪一态（不点名单元的域恒取两份都好）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum RecordPresence {
    NotOnDisk,
    OnDisk(UnitState),
}

/// 每条记录在 1 号盘上有没有一份（登记 r2 5.3 多盘域；D22（单元原子性怎么合成） 已定项 8 第 2 条「journal 环在两块盘上互为镜像」）。
#[derive(Clone, Copy)]
enum JournalMirroring<'state> {
    /// 单盘：生成的记录只写 0 号盘（原登记 5.1；R 域的残留另在 1 号盘）。
    FirstDeviceOnly,
    /// 两盘镜像：第 r 条记录在 1 号盘同一落点那一份落没落；0 号盘那一份看 `presence`。
    BothDevices {
        second_device_copy_is_on_disk: &'state [bool],
    },
}

struct LegalState<'state> {
    history: &'state History,
    presence: &'state [RecordPresence],
    mirroring: JournalMirroring<'state>,
    residue: Option<&'state Residue>,
    chosen_version: usize,
    verify_named_units: bool,
}

type RecordTable = BTreeMap<(InstanceGeneration, u64), JournalRecord>;

/// 表里一条记录在盘上那一份的真相：S 的 (d) 要它的头算出的链值，(k) 要它点名的单元两份是不是都读得出、校验和都对。
#[derive(Clone, Copy, Debug)]
struct RecordTruthOnDisk {
    instance: u32,
    counter: u64,
    chain_value_of_header: u32,
    named_units_all_verified: bool,
}

/// 直接建表读盘的次序（登记 5.1「按盘号升序」；r2 5.3「先 0 号盘的全部落了的记录（按记录次序），再 1 号盘的」）。
const DEVICE_ORDER_OF_THE_DIRECT_TABLE: [u32; 2] = [0, 1];

/// 扫盘建表喂给 `scan_journal` 的系统配置里，`scan_journal` 不读的那些字段取的值（它只读 fsid 与 journal 环长，`recovery.rs` 里
/// `scan_journal` 的函数体）：值随便取、只求造得出一份结构体；不借 mkfs 与写路径（登记 r2 第三节「盘上字节由谁造」、停机条款 SH9）。
const SCAN_CONFIGURATION_BLOCK_BYTES_THE_SCAN_DOES_NOT_READ: u32 = 4096;
const SCAN_CONFIGURATION_DEVICE_COUNT: u32 = 2;
const SCAN_CONFIGURATION_ROOT_RING_BASE_SLOT_THE_SCAN_DOES_NOT_READ: u64 = 0;
const SCAN_CONFIGURATION_SLOT_GENERATION_THE_SCAN_DOES_NOT_READ: u64 = 1;
const SCAN_CONFIGURATION_JOURNAL_TAIL_THE_SCAN_DOES_NOT_READ: u64 = 0;
const SCAN_CONFIGURATION_ROLLBACK_FLOOR_THE_SCAN_DOES_NOT_READ: u64 = 0;

/// 一条记录在一块盘上的一份：槽（字节、`parse` 的结果、头算出的链值）、它点名的单元两份摆在哪（跟着 0 号盘那一份摆）、
/// 点名单元是不是两份都好（S 的 (k) 要它）。
struct RecordCopyOnADevice<'state> {
    slot: &'state GeneratedSlot,
    named_unit_copies: Option<(u64, StoredBytes<'state>, StoredBytes<'state>)>,
    named_units_all_verified: bool,
}

/// 一块盘上落了的记录，按记录次序：0 号盘是 `presence` 里在盘上的那几条；1 号盘是镜像那几份（多盘域），之后是 R 域的残留。
fn record_copies_on_the_device<'state>(
    state: &LegalState<'state>,
    device: u32,
) -> Vec<RecordCopyOnADevice<'state>> {
    let mut copies = Vec::new();
    if device == 0 {
        for (record_index, slot) in state.history.slots.iter().enumerate() {
            let RecordPresence::OnDisk(unit_state) = state.presence[record_index] else {
                continue;
            };
            let named_unit_copies = state.history.units[record_index].as_ref().map(|unit| {
                let (first_device_copy, second_device_copy) = match unit_state {
                    UnitState::BothCopiesGood => (
                        StoredBytes::Readable(&unit.good_bytes),
                        StoredBytes::Readable(&unit.good_bytes),
                    ),
                    UnitState::FirstDeviceCopyUnreadable => (
                        StoredBytes::Unreadable,
                        StoredBytes::Readable(&unit.good_bytes),
                    ),
                    UnitState::SecondDeviceCopyUnreadable => (
                        StoredBytes::Readable(&unit.good_bytes),
                        StoredBytes::Unreadable,
                    ),
                    UnitState::FirstDeviceCopyChecksumWrong => (
                        StoredBytes::Readable(&unit.bytes_with_wrong_checksum),
                        StoredBytes::Readable(&unit.good_bytes),
                    ),
                    UnitState::SecondDeviceCopyChecksumWrong => (
                        StoredBytes::Readable(&unit.good_bytes),
                        StoredBytes::Readable(&unit.bytes_with_wrong_checksum),
                    ),
                };
                (unit.offset_in_bytes, first_device_copy, second_device_copy)
            });
            copies.push(RecordCopyOnADevice {
                slot,
                named_unit_copies,
                named_units_all_verified: unit_state == UnitState::BothCopiesGood,
            });
        }
        return copies;
    }
    if let JournalMirroring::BothDevices {
        second_device_copy_is_on_disk,
    } = state.mirroring
    {
        for (record_index, slot) in state.history.slots.iter().enumerate() {
            if !second_device_copy_is_on_disk[record_index] {
                continue;
            }
            assert!(
                state.history.units[record_index].is_none(),
                "多盘域不点名单元（登记 r2 5.3）：1 号盘那一份没有自己的点名单元摆法"
            );
            copies.push(RecordCopyOnADevice {
                slot,
                named_unit_copies: None,
                named_units_all_verified: true,
            });
        }
    }
    if let Some(residue) = state.residue {
        copies.push(RecordCopyOnADevice {
            slot: &residue.slot,
            named_unit_copies: None,
            named_units_all_verified: true,
        });
    }
    copies
}

/// 把一个状态摆上内存盘，并照登记 5.1 建 `records` 表：按 [`DEVICE_ORDER_OF_THE_DIRECT_TABLE`] 的盘次序、逐槽取 `parse` 的结果，
/// 读得出的按 (实例, 计数器) 先到先得；`truths` 记先到的那一份。
fn record_table_and_disk_of<'state>(
    state: &LegalState<'state>,
) -> (RecordTable, MemoryDisk<'state>, Vec<RecordTruthOnDisk>) {
    record_table_and_disk_of_in_device_order(state, DEVICE_ORDER_OF_THE_DIRECT_TABLE)
}

/// 同 [`record_table_and_disk_of`]，直接建表读盘的次序由 `device_order` 给：登记 r2 第八节盘次序那一点上，直接建表分两栏各比一遍，
/// 「盘号升序」栏给 [`DEVICE_ORDER_OF_THE_DIRECT_TABLE`]、「照 `device_identities()` 次序」栏给内存盘报的次序。
fn record_table_and_disk_of_in_device_order<'state>(
    state: &LegalState<'state>,
    device_order: [u32; 2],
) -> (RecordTable, MemoryDisk<'state>, Vec<RecordTruthOnDisk>) {
    let mut records = RecordTable::new();
    let mut truths: Vec<RecordTruthOnDisk> = Vec::new();
    let mut contents: Vec<(u32, u64, StoredBytes<'state>)> = Vec::new();
    for device in device_order {
        for copy in record_copies_on_the_device(state, device) {
            contents.push((
                device,
                copy.slot.offset_in_bytes,
                StoredBytes::Readable(&copy.slot.bytes),
            ));
            if let Some((unit_offset_in_bytes, first_device_copy, second_device_copy)) =
                copy.named_unit_copies
            {
                contents.push((0, unit_offset_in_bytes, first_device_copy));
                contents.push((1, unit_offset_in_bytes, second_device_copy));
            }
            if let Some(parsed) = copy.slot.parsed.as_ref() {
                if let btree_map::Entry::Vacant(vacant_place) =
                    records.entry((parsed.instance, parsed.counter))
                {
                    vacant_place.insert(parsed.clone());
                    truths.push(RecordTruthOnDisk {
                        instance: parsed.instance.0,
                        counter: parsed.counter,
                        chain_value_of_header: copy.slot.chain_value_of_header,
                        named_units_all_verified: copy.named_units_all_verified,
                    });
                }
            }
        }
    }
    (
        records,
        MemoryDisk {
            ring: state.history.ring,
            reported_disk_order: state.history.run_profile.reported_disk_order,
            contents,
        },
        truths,
    )
}

// ── 扫盘建表（登记 r2 第三节「由 scan_journal 建表」、5.4 的 T-表与 T-行）

/// 喂给 `scan_journal` 的系统配置：fsid 取时间线根的那个 16 字节常量（低 8 字节就是记录头里那 8 字节），journal 环长取这个环几何的字节数，
/// 其余字段 `scan_journal` 不读（[`SCAN_CONFIGURATION_BLOCK_BYTES_THE_SCAN_DOES_NOT_READ`] 一组常量）。
fn system_configuration_for_the_scan(ring: RingGeometry) -> SystemConfiguration {
    SystemConfiguration {
        immutable: SystemImmutableConfiguration {
            filesystem_identifier: ROOT_FILESYSTEM_IDENTIFIER,
            this_device: DeviceIdentity(0),
            device_count: SCAN_CONFIGURATION_DEVICE_COUNT,
            region_devices: [DeviceIdentity(0), DeviceIdentity(1), DeviceIdentity(0)],
            sizes: SystemImmutableSizes {
                physical_block_size: SCAN_CONFIGURATION_BLOCK_BYTES_THE_SCAN_DOES_NOT_READ,
                minimum_input_output_bytes: SCAN_CONFIGURATION_BLOCK_BYTES_THE_SCAN_DOES_NOT_READ,
                fixed_structure_slot_spacing: SCAN_CONFIGURATION_BLOCK_BYTES_THE_SCAN_DOES_NOT_READ,
                journal_ring_bytes: ring.ring_bytes(),
                root_ring_slots_per_region: RootRingSlotsPerRegion::AT_MAKE_FILESYSTEM,
            },
            journal_ring_start_slot: SlotNumber(LOCAL_JOURNAL_RING_START_SLOT),
            root_ring_base_slot: SlotNumber(
                SCAN_CONFIGURATION_ROOT_RING_BASE_SLOT_THE_SCAN_DOES_NOT_READ,
            ),
        },
        mutable: SystemMutableConfiguration,
        runtime: SystemRuntimeConfiguration,
        quantities: SystemRuntimeQuantities {
            slot_generation: SCAN_CONFIGURATION_SLOT_GENERATION_THE_SCAN_DOES_NOT_READ,
            journal_tail: SCAN_CONFIGURATION_JOURNAL_TAIL_THE_SCAN_DOES_NOT_READ,
            journal_instance: InstanceGeneration(CHOSEN_INSTANCE),
            rollback_floor: CheckpointTxg(SCAN_CONFIGURATION_ROLLBACK_FLOOR_THE_SCAN_DOES_NOT_READ),
        },
    }
}

/// 扫盘建表：对摆好的内存盘调 `scan_journal`。`implementation` 是 T-表那条对照时，建完把键最大的那条记录删掉（假结局：扫盘建的表少一条）。
fn scanned_record_table_of(disk: &MemoryDisk<'_>, implementation: Implementation) -> RecordTable {
    let mut table = scan_journal(disk, &system_configuration_for_the_scan(disk.ring));
    if implementation == Implementation::Control(ControlKind::ScanTableLosesItsLargestKeyRecord) {
        table.pop_last();
    }
    table
}

/// T-表：同一个状态上两张表逐键逐值比，有一处不同即不同（登记 r2 5.4）。
fn the_two_record_tables_differ(direct_table: &RecordTable, scanned_table: &RecordTable) -> bool {
    direct_table != scanned_table
}

/// 两张表差在哪：只在直接建的表里的键、只在扫盘建的表里的键、两边都有而记录不同的键（键写成 `实例:计数器`）。
fn describe_how_the_two_record_tables_differ(
    direct_table: &RecordTable,
    scanned_table: &RecordTable,
) -> String {
    let key_text = |key: &(InstanceGeneration, u64)| format!("{}:{}", key.0 .0, key.1);
    let keys_of = |keys: Vec<String>| {
        if keys.is_empty() {
            "none".to_string()
        } else {
            keys.join("+")
        }
    };
    let only_in_the_direct_table = keys_of(
        direct_table
            .keys()
            .filter(|key| !scanned_table.contains_key(*key))
            .map(key_text)
            .collect(),
    );
    let only_in_the_scanned_table = keys_of(
        scanned_table
            .keys()
            .filter(|key| !direct_table.contains_key(*key))
            .map(key_text)
            .collect(),
    );
    let with_different_records = keys_of(
        direct_table
            .iter()
            .filter(|(key, record)| {
                scanned_table
                    .get(*key)
                    .is_some_and(|scanned_record| scanned_record != *record)
            })
            .map(|(key, _record)| key_text(key))
            .collect(),
    );
    format!(
        "direct_table_entries={};scanned_table_entries={};only_in_the_direct_table={only_in_the_direct_table};only_in_the_scanned_table={only_in_the_scanned_table};same_key_different_record={with_different_records}",
        direct_table.len(),
        scanned_table.len()
    )
}

/// 一个状态在一种建表法下判完的小结（登记 r2 5.4 T-行要比的两样）：第一次调用的结局，这个状态让哪几个计数器加了一。
struct JudgedRowSummary {
    first_outcome: ReplayOutcome,
    counter_labels: String,
}

/// T-行：两遍的实现结局（`describe_outcome` 那一串）与这个状态加了一的计数器标签集合，有一项不同即不同（登记 r2 5.4）。
fn the_two_judged_rows_differ(direct: &JudgedRowSummary, scanned: &JudgedRowSummary) -> bool {
    describe_outcome(&direct.first_outcome) != describe_outcome(&scanned.first_outcome)
        || direct.counter_labels != scanned.counter_labels
}

/// 两张表各判一遍之后逐态比，比出来不同的记进 `tally` 的 `tables_differ_counter`、`rows_differ_counter`，各记第一个见证（状态的写法 + 差在哪）。
fn add_the_comparison_of_two_judged_tables(
    tally: &mut Tally,
    (tables_differ_counter, rows_differ_counter): (Counter, Counter),
    direct: (&RecordTable, &JudgedRowSummary),
    scanned: (&RecordTable, &JudgedRowSummary),
    describe_state: &dyn Fn() -> String,
) {
    let (direct_table, direct_summary) = direct;
    let (scanned_table, scanned_summary) = scanned;
    if the_two_record_tables_differ(direct_table, scanned_table) {
        tally.add_violation(tables_differ_counter, &|| {
            format!(
                "{};{}",
                describe_state(),
                describe_how_the_two_record_tables_differ(direct_table, scanned_table)
            )
        });
    }
    if the_two_judged_rows_differ(direct_summary, scanned_summary) {
        tally.add_violation(rows_differ_counter, &|| {
            format!(
                "{};direct_outcome={};scanned_outcome={};direct_counters={};scanned_counters={}",
                describe_state(),
                describe_outcome(&direct_summary.first_outcome),
                describe_outcome(&scanned_summary.first_outcome),
                direct_summary.counter_labels,
                scanned_summary.counter_labels
            )
        });
    }
}

/// 扫盘那一遍用哪种实现、算不算指定状态：T-行那条对照把扫盘那一遍的结局换成 panic（表不动），别的情形与直接建表那一遍相同。
fn implementation_and_designation_of_the_scan_pass(
    implementation: Implementation,
    is_the_designated_state: bool,
) -> (Implementation, bool) {
    match implementation {
        Implementation::Control(ControlKind::ScanPassOutcomeBecomesAPanic) => (
            Implementation::Control(ControlKind::PanicsOnTheDesignatedState),
            true,
        ),
        Implementation::TodayReplayJournal | Implementation::Control(_) => {
            (implementation, is_the_designated_state)
        }
    }
}

/// 一个状态判完之后「这个状态让哪几个计数器加了一」：对着判之前的计数。
fn counter_labels_a_state_added_to(tally: &Tally, counts_before: &[u64]) -> String {
    labels_of_the_counters_a_state_added(counts_before, &tally.counts)
}

/// 每条记录在每块盘上都落了（登记 r2 第七节甲类「全部落盘那一态」）：0 号盘上都在，多盘域 1 号盘上的镜像也都在。
fn every_record_is_landed_on_every_device(state: &LegalState<'_>) -> bool {
    state
        .presence
        .iter()
        .all(|record_presence| *record_presence != RecordPresence::NotOnDisk)
        && (0..state.presence.len()).all(|record_index| match state.mirroring {
            JournalMirroring::FirstDeviceOnly => true,
            JournalMirroring::BothDevices {
                second_device_copy_is_on_disk,
            } => second_device_copy_is_on_disk[record_index],
        })
}

/// 登记 r2 第七节甲类：每条记录在每块盘上都落了的那一态，扫盘表的条数等于这一态生成的记录条数（R 域的残留与 0 号盘那一份同键，后到不进表）；
/// 不符走 SF9（停下，两边都查）。只在今天的实现上断言：T-表那条对照故意删一条。
fn assert_the_scan_table_of_a_fully_landed_state(
    state: &LegalState<'_>,
    scanned_table: &RecordTable,
    implementation: Implementation,
    tally: &mut Tally,
) {
    if implementation != Implementation::TodayReplayJournal
        || !every_record_is_landed_on_every_device(state)
    {
        return;
    }
    tally.add(Counter::FullyLandedScanTableSizeChecked);
    assert_eq!(
        scanned_table.len(),
        state.history.slots.len(),
        "停机条款 SF9：每条记录在每块盘上都落了的那一态，扫盘表的条数不等于生成的记录条数（形状 {}）",
        state.history.shape_description
    );
}

/// 一个状态判完的结果：返回给 C、MC 域状态对要用的施加区间，加上第一次调用的结局与它落到的最远版本（逐态行的写法要）。
struct LegalJudgement {
    interval: Option<AppliedInterval>,
    first_outcome: ReplayOutcome,
    farthest_version: usize,
}

/// 一条记录「完整在盘上」：在 0 号盘上读得出，且（U 域）它点名的单元两份都读得出、校验和都对（登记 5.3）；
/// 多盘域任一份在即在（登记 r2 5.3，D23（journal 的角色与格式） 已定项 14「两份镜像何时算「记录在」：任一份自证校验和过即在」）。
fn record_is_completely_on_disk(state: &LegalState<'_>, record_index: usize) -> bool {
    let first_device_copy_is_complete = match state.presence[record_index] {
        RecordPresence::NotOnDisk => false,
        RecordPresence::OnDisk(unit_state) => unit_state == UnitState::BothCopiesGood,
    };
    first_device_copy_is_complete || second_device_copy_is_on_disk(state, record_index)
}

/// 1 号盘上那一份落没落；单盘域恒为假。
fn second_device_copy_is_on_disk(state: &LegalState<'_>, record_index: usize) -> bool {
    match state.mirroring {
        JournalMirroring::FirstDeviceOnly => false,
        JournalMirroring::BothDevices {
            second_device_copy_is_on_disk,
        } => second_device_copy_is_on_disk[record_index],
    }
}

/// 这条记录有没有一份落了盘（不看点名单元）。
fn record_has_a_copy_on_disk(state: &LegalState<'_>, record_index: usize) -> bool {
    state.presence[record_index] != RecordPresence::NotOnDisk
        || second_device_copy_is_on_disk(state, record_index)
}

fn publish_is_completely_on_disk(state: &LegalState<'_>, publish_index: usize) -> bool {
    state
        .history
        .record_indexes_of_publish(publish_index)
        .all(|record_index| record_is_completely_on_disk(state, record_index))
}

fn publish_has_a_record_on_disk(state: &LegalState<'_>, publish_index: usize) -> bool {
    state
        .history
        .record_indexes_of_publish(publish_index)
        .any(|record_index| record_has_a_copy_on_disk(state, record_index))
}

/// j\*（登记 5.3 的 J4）：最大的 j ≥ c，使 (c, j] 里每次发布都完整在盘上、且这些发布的记录条数之和不超过在飞上限；
/// 只数所选实例的发布。`in_flight_limit` 为 `None` 时不看上限（只给阳性对照「不看上限」用）。
fn farthest_version_of(state: &LegalState<'_>, in_flight_limit: Option<usize>) -> usize {
    let mut farthest_version = state.chosen_version;
    let mut record_count_so_far = 0usize;
    for version in state.chosen_version + 1..state.history.chosen_instance_publish_count {
        if !publish_is_completely_on_disk(state, version) {
            break;
        }
        record_count_so_far += state.history.publishes[version].record_count;
        if in_flight_limit.is_some_and(|limit| record_count_so_far > limit) {
            break;
        }
        farthest_version = version;
    }
    farthest_version
}

/// (chosen_version, version] 里各次发布的记录条数之和。
fn record_count_of_versions_after_the_chosen_one(state: &LegalState<'_>, version: usize) -> usize {
    (state.chosen_version + 1..=version)
        .map(|applied_version| state.history.publishes[applied_version].record_count)
        .sum()
}

// ── 一次调用的结局

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum FailureKind {
    /// `UnitMalformed`，`what` 是 `CHOSEN_ROOT_CHECKPOINT_TXG_HAS_NO_SUCCESSOR`。
    ChosenRootTxgHasNoSuccessor,
    /// `RootPublishCarriesMoreThanOneLastRecordFlagWhoseAnchorIsUndecided`。
    RootPublishCarriesMoreThanOneLastRecordFlag,
    Other,
}

impl FailureKind {
    fn of(failure: &RecoveryFailure) -> Self {
        if matches!(failure, RecoveryFailure::UnitMalformed { what } if *what == CHOSEN_ROOT_CHECKPOINT_TXG_HAS_NO_SUCCESSOR)
        {
            Self::ChosenRootTxgHasNoSuccessor
        } else if matches!(
            failure,
            RecoveryFailure::RootPublishCarriesMoreThanOneLastRecordFlagWhoseAnchorIsUndecided { .. }
        ) {
            Self::RootPublishCarriesMoreThanOneLastRecordFlag
        } else {
            Self::Other
        }
    }

    const fn label(self) -> &'static str {
        match self {
            Self::ChosenRootTxgHasNoSuccessor => "chosen_root_txg_has_no_successor",
            Self::RootPublishCarriesMoreThanOneLastRecordFlag => "more_than_one_last_record_flag",
            Self::Other => "other_failure",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct ReturnedReplay {
    root: RootRecord,
    prefix_applied: usize,
    verification_passed: usize,
}

#[allow(
    clippy::large_enum_variant,
    reason = "每个状态一两个结局、按值比较拷来拷去要 Copy；装箱就丢了 Copy，每个状态还多一次分配"
)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ReplayOutcome {
    Panicked,
    Failed(FailureKind),
    Returned(ReturnedReplay),
}

fn describe_outcome(outcome: &ReplayOutcome) -> String {
    match outcome {
        ReplayOutcome::Panicked => "panicked".to_string(),
        ReplayOutcome::Failed(kind) => format!("failed:{}", kind.label()),
        ReplayOutcome::Returned(returned) => format!(
            "returned:txg{}:watermark{}:applied{}",
            returned.root.checkpoint_txg.0,
            returned.root.tree_identifier_watermark,
            returned.prefix_applied
        ),
    }
}

thread_local! {
    /// 这条线程此刻是不是在 `replay_journal` 里：panic 钩子只在这时不打印（被变异的实现可以 panic 几百万次）。
    static IS_INSIDE_THE_REPLAY_CALL: Cell<bool> = const { Cell::new(false) };
}

fn install_the_panic_hook_that_stays_quiet_inside_the_replay_call() {
    let default_hook = panic::take_hook();
    panic::set_hook(Box::new(move |panic_information| {
        if !IS_INSIDE_THE_REPLAY_CALL.with(Cell::get) {
            default_hook(panic_information);
        }
    }));
}

/// 被测对象：今天的 `replay_journal`，不改一个字。panic 用 `catch_unwind` 接住（J0、A0）。
fn call_replay_journal(
    disk: &MemoryDisk<'_>,
    chosen_root: &RootRecord,
    records: &RecordTable,
    verify_named_units: bool,
) -> ReplayOutcome {
    IS_INSIDE_THE_REPLAY_CALL.with(|is_inside| is_inside.set(true));
    let result = panic::catch_unwind(AssertUnwindSafe(|| {
        replay_journal(
            disk,
            chosen_root,
            disk.ring.ring_bytes(),
            records,
            verify_named_units,
        )
    }));
    IS_INSIDE_THE_REPLAY_CALL.with(|is_inside| is_inside.set(false));
    match result {
        Err(_panic_payload) => ReplayOutcome::Panicked,
        Ok(Err(failure)) => ReplayOutcome::Failed(FailureKind::of(&failure)),
        Ok(Ok((report, root))) => ReplayOutcome::Returned(ReturnedReplay {
            root,
            prefix_applied: report.prefix_applied,
            verification_passed: report.verification_passed,
        }),
    }
}

// ── 照条款写的全函数 S（登记 5.3「对抗一族的判据」）

/// (j) 的读法（岔路单 K9）。`NeverJudges` 只给 A-互 的阳性对照用。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum FollowReading {
    WholeTable,
    ConsecutiveGapContinues,
    ConsecutiveGapStops,
    NeverJudges,
}

impl FollowReading {
    /// 登记的三种读法，次序是 AS-全、AS-连甲、AS-连乙。
    const REGISTERED: [Self; 3] = [
        Self::WholeTable,
        Self::ConsecutiveGapContinues,
        Self::ConsecutiveGapStops,
    ];

    const fn label(self) -> &'static str {
        match self {
            Self::WholeTable => "whole_table",
            Self::ConsecutiveGapContinues => "consecutive_gap_continues",
            Self::ConsecutiveGapStops => "consecutive_gap_stops",
            Self::NeverJudges => "never_judges",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum SpecificationExpectation {
    /// S 第 1 条：报错、一条都不施加。
    FailsWithoutApplying,
    /// 期望返回的根的六个字段。
    ReturnsRoot(VersionFields),
}

struct SpecificationView<'view> {
    chosen_root: &'view RootRecord,
    records: &'view RecordTable,
    in_flight_limit: usize,
    verify_named_units: bool,
    truths: &'view [RecordTruthOnDisk],
}

fn record_carries_the_last_record_flag(record: &JournalRecord) -> bool {
    record.place_in_publish == JournalRecordPlaceInPublish::LastRecordOfThePublish
}

/// 全表看：表里任何位置有同实例、同 txg、计数器更大的记录。
fn a_same_publish_record_follows_anywhere_in_the_table(
    record: &JournalRecord,
    records: &RecordTable,
) -> bool {
    records.values().any(|later| {
        later.instance == record.instance
            && later.checkpoint_txg == record.checkpoint_txg
            && later.counter > record.counter
    })
}

/// 连着看·缺号不停：从下一个计数器起按表里同实例记录的计数器升序看，看到 txg 越过这一条的就停；停之前见到同 txg 的为真。
fn a_same_publish_record_follows_before_a_later_txg(
    record: &JournalRecord,
    records: &RecordTable,
) -> bool {
    for later in records.values() {
        if later.instance != record.instance || later.counter <= record.counter {
            continue;
        }
        if later.checkpoint_txg > record.checkpoint_txg {
            return false;
        }
        if later.checkpoint_txg == record.checkpoint_txg {
            return true;
        }
    }
    false
}

/// 连着看·缺号即停：只看计数器一个接一个都在表里的那几条，缺号或 txg 越过就停；停之前见到同 txg 的为真。
fn a_same_publish_record_follows_without_a_gap(
    record: &JournalRecord,
    records: &RecordTable,
) -> bool {
    let largest_counter_in_the_table = records
        .keys()
        .filter(|(instance, _counter)| *instance == record.instance)
        .map(|(_instance, counter)| *counter)
        .max()
        .unwrap_or(record.counter);
    for counter in record.counter + 1..=largest_counter_in_the_table {
        let Some(later) = records.get(&(record.instance, counter)) else {
            return false;
        };
        if later.checkpoint_txg > record.checkpoint_txg {
            return false;
        }
        if later.checkpoint_txg == record.checkpoint_txg {
            return true;
        }
    }
    false
}

fn a_same_publish_record_follows(
    reading: FollowReading,
    record: &JournalRecord,
    records: &RecordTable,
) -> bool {
    match reading {
        FollowReading::WholeTable => {
            a_same_publish_record_follows_anywhere_in_the_table(record, records)
        }
        FollowReading::ConsecutiveGapContinues => {
            a_same_publish_record_follows_before_a_later_txg(record, records)
        }
        FollowReading::ConsecutiveGapStops => {
            a_same_publish_record_follows_without_a_gap(record, records)
        }
        FollowReading::NeverJudges => false,
    }
}

/// S：照 D23（journal 的角色与格式） 已定项 14 五条与注 1、已定项 4 读者规则、已定项 7、I-8.3、I-8.6 的字面，对任意一张表给出期望的结局。
/// 分四条（登记 5.3）：1. 所选根那次发布带末条标志的多于一条 ⇒ 报错；2. 链首；3. 前缀逐条取、遇 (a)–(k) 任一情形停；
/// 4. 施加前缀里走到了末条的发布，期望的根 = 所选根把六个字段换成最后一次被施加的发布里带末条标志那一条的。
fn specification_expectation(
    view: &SpecificationView<'_>,
    reading: FollowReading,
) -> SpecificationExpectation {
    let instance = view.chosen_root.instance;
    let chosen_txg = view.chosen_root.checkpoint_txg;
    let is_above_water = |record: &JournalRecord| record.checkpoint_txg > chosen_txg;
    // 第 1 条与第 2 条的锚点。
    let counters_of_the_root_publish_carrying_the_last_record_flag: Vec<u64> = view
        .records
        .values()
        .filter(|record| {
            record.instance == instance
                && record.checkpoint_txg == chosen_txg
                && record_carries_the_last_record_flag(record)
        })
        .map(|record| record.counter)
        .collect();
    if counters_of_the_root_publish_carrying_the_last_record_flag.len() > 1 {
        return SpecificationExpectation::FailsWithoutApplying;
    }
    let nothing_applied =
        SpecificationExpectation::ReturnsRoot(VersionFields::of_root(view.chosen_root));
    let chain_head_counter =
        match counters_of_the_root_publish_carrying_the_last_record_flag.first() {
            Some(anchor_counter) => anchor_counter + 1,
            None => {
                // 注 1：锚点读不出时链首接水位之上第一条可读记录，前提是它的序号为 1、txg = 所选根的 txg + 1。
                let Some(first_record_above_water) = view
                    .records
                    .values()
                    .filter(|record| record.instance == instance && is_above_water(record))
                    .min_by_key(|record| record.counter)
                else {
                    return nothing_applied;
                };
                let follows_the_chosen_root_directly = chosen_txg.0.checked_add(1)
                    == Some(first_record_above_water.checkpoint_txg.0)
                    && first_record_above_water.ordinal_within_publish.0 == 1;
                if !follows_the_chosen_root_directly {
                    return nothing_applied;
                }
                first_record_above_water.counter
            }
        };
    // 第 3 条：前缀。迭代次数的上界是在飞上限（(c)）；跨轮携带的是已进前缀的记录，每一轮至多加一条。
    let mut prefix: Vec<&JournalRecord> = Vec::new();
    let mut counter = chain_head_counter;
    while let Some(record) = view.records.get(&(instance, counter)) {
        // (a) 计数器缺号：`get` 取不到就出循环。(b) 该条不在水位之上：取「停」（条款没逐字写，主 agent 2026-10-01 认的，推的）。
        if !is_above_water(record) {
            break;
        }
        // (c) 已取满在飞上限条。
        if prefix.len() == view.in_flight_limit {
            break;
        }
        // (d) 反向链不等于盘上前一条的头算出的链值；前一条不在盘上不判（I-8.6）。
        let previous_on_disk = view.truths.iter().find(|truth| {
            truth.instance == instance.0 && Some(truth.counter) == counter.checked_sub(1)
        });
        if previous_on_disk.is_some_and(|truth| truth.chain_value_of_header != record.back_chain) {
            break;
        }
        // (e) 序号 0。
        if record.ordinal_within_publish.0 == 0 {
            break;
        }
        let previous_record_of_the_open_publish = prefix
            .last()
            .filter(|previous| !record_carries_the_last_record_flag(previous));
        match previous_record_of_the_open_publish {
            // (f) 一次发布的首条（链首，或紧跟在带末条标志的那条之后）序号不是 1。
            None => {
                if record.ordinal_within_publish.0 != 1 {
                    break;
                }
            }
            Some(previous) => {
                // (g) 不是首条，而 txg 与本次发布前一条不同、或序号不是前一条加一。
                if record.checkpoint_txg != previous.checkpoint_txg
                    || u64::from(record.ordinal_within_publish.0)
                        != u64::from(previous.ordinal_within_publish.0) + 1
                {
                    break;
                }
                // (h) 本次发布前一条不带提交标记而该条换了事务号。
                if !previous.is_commit && record.transaction != previous.transaction {
                    break;
                }
            }
        }
        if record_carries_the_last_record_flag(record) {
            // (i) 带末条标志而不带提交标记。
            if !record.is_commit {
                break;
            }
            // (j) 带末条标志，且按所取读法之后还有同 (实例, txg) 的记录。
            if a_same_publish_record_follows(reading, record, view.records) {
                break;
            }
        }
        // (k) 验点名单元开着而该条有一份点名单元读不出或校验和不对。
        if view.verify_named_units {
            let named_units_all_verified = view
                .truths
                .iter()
                .find(|truth| truth.instance == instance.0 && truth.counter == counter)
                .is_some_and(|truth| truth.named_units_all_verified);
            if !named_units_all_verified {
                break;
            }
        }
        prefix.push(record);
        let Some(next_counter) = counter.checked_add(1) else {
            break;
        };
        counter = next_counter;
    }
    // 第 4 条：末尾没走到末条的那一段不施加。
    match prefix
        .iter()
        .rev()
        .find(|record| record_carries_the_last_record_flag(record))
    {
        Some(last_applied_record) => {
            SpecificationExpectation::ReturnsRoot(VersionFields::of_record(last_applied_record))
        }
        None => nothing_applied,
    }
}

/// 实现的结局与 S 的期望同不同（AS 三条）：比「报错 / 返回」的种类与返回根的六个字段。
fn outcome_matches_the_expectation(
    outcome: &ReplayOutcome,
    expectation: SpecificationExpectation,
) -> bool {
    match (outcome, expectation) {
        (ReplayOutcome::Failed(_kind), SpecificationExpectation::FailsWithoutApplying) => true,
        (ReplayOutcome::Returned(returned), SpecificationExpectation::ReturnsRoot(fields)) => {
            VersionFields::of_root(&returned.root) == fields
        }
        (ReplayOutcome::Panicked, _)
        | (ReplayOutcome::Failed(_), SpecificationExpectation::ReturnsRoot(_))
        | (ReplayOutcome::Returned(_), SpecificationExpectation::FailsWithoutApplying) => false,
    }
}

/// 把 S 的期望当成一次调用的结局（AS 三条与 A-互 的阳性对照：拿另一种读法的 S 当实现）。
fn outcome_of_the_expectation(
    chosen_root: &RootRecord,
    expectation: SpecificationExpectation,
) -> ReplayOutcome {
    match expectation {
        SpecificationExpectation::FailsWithoutApplying => {
            ReplayOutcome::Failed(FailureKind::RootPublishCarriesMoreThanOneLastRecordFlag)
        }
        SpecificationExpectation::ReturnsRoot(fields) => {
            let root = if fields == VersionFields::of_root(chosen_root) {
                *chosen_root
            } else {
                fields.to_root_rebuilt_over(chosen_root)
            };
            ReplayOutcome::Returned(ReturnedReplay {
                root,
                prefix_applied: 0,
                verification_passed: 0,
            })
        }
    }
}

// ── 计数

macro_rules! labelled_enumeration {
    ($(#[$attribute:meta])* enum $name:ident { $($variant:ident => $label:literal,)+ }) => {
        $(#[$attribute])*
        #[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
        enum $name {
            $($variant,)+
        }

        impl $name {
            const ALL: &'static [Self] = &[$(Self::$variant,)+];

            const fn label(self) -> &'static str {
                match self {
                    $(Self::$variant => $label,)+
                }
            }

            #[allow(dead_code, reason = "不是每个枚举都拿下标去索引计数数组")]
            fn index(self) -> usize {
                Self::ALL
                    .iter()
                    .position(|member| *member == self)
                    .expect("ALL 列了这个枚举的每一个成员")
            }
        }
    };
}

labelled_enumeration! {
    /// 每个 (域, N) 上数的量。违例类的量在第一次加一时记一个见证。
    enum Counter {
        States => "states",
        Panicked => "panicked",
        Failed => "failed",
        J1Violations => "J1",
        J2Violations => "J2",
        J3Violations => "J3",
        J4Violations => "J4",
        J6PremiseHoldsJudged => "j6_premise_holds_judged",
        J6PremiseHoldsViolations => "J6_premise_holds",
        J6PremiseFailsJudged => "j6_premise_fails_judged",
        J6PremiseFailsViolations => "J6_premise_fails",
        J6FirstCallFailed => "j6_first_call_failed",
        J6ThirdCallEqualsSecond => "j6_third_call_equals_second",
        J6ThirdCallDiffersOrIsMissing => "j6_third_call_differs_or_is_missing",
        Ia1Judged => "ia1_judged",
        Ia1Violations => "IA1",
        Ia2Judged => "ia2_judged",
        Ia2Violations => "IA2",
        AppliedAtLeastOne => "applied_ge1",
        AppliedAtLeastTwo => "applied_ge2",
        FarthestAtLeastOne => "farthest_ge1",
        FarthestAtLeastTwo => "farthest_ge2",
        AboveWaterOverLimit => "above_water_over_limit",
        AboveWaterAtLimit => "above_water_at_limit",
        SpecificationWholeTableMismatch => "S_whole_table",
        SpecificationGapContinuesMismatch => "S_consecutive_gap_continues",
        SpecificationGapStopsMismatch => "S_consecutive_gap_stops",
        ChosenRootIsALaterVersion => "chosen_root_is_a_later_version",
        OffTimelineLandings => "off_timeline_landings",
        LegalCrashStatesWhenTheRootWriteIsItsOwnSegment => "legal_crash_states_cut_own",
        LegalCrashStatesWhenTheRootWriteSharesTheRecordSegment => "legal_crash_states_cut_with",
        PairStatesWithoutAnInterval => "pair_states_without_an_interval",
        AsWholeTableViolations => "AS_whole_table",
        AsGapContinuesViolations => "AS_consecutive_gap_continues",
        AsGapStopsViolations => "AS_consecutive_gap_stops",
        MutualWholeTableAndGapContinues => "whole_table_vs_consecutive_gap_continues",
        MutualWholeTableAndGapStops => "whole_table_vs_consecutive_gap_stops",
        MutualGapContinuesAndGapStops => "consecutive_gap_continues_vs_consecutive_gap_stops",
        StructureViolations => "A_structure",
        LimitViolations => "A_limit",
        SpecificationExpectsFailure => "specification_expects_failure",
        StatesHoldingARecordRejectedByParse => "states_holding_a_record_rejected_by_parse",
        J6QnPremiseHoldsJudged => "j6qn_premise_holds_judged",
        J6QnPremiseHoldsViolations => "J6Qn_premise_holds",
        J6QnPremiseFailsJudged => "j6qn_premise_fails_judged",
        J6QnPremiseFailsViolations => "J6Qn_premise_fails",
        FirstReturnedRootTxgIsTheLargest => "first_returned_root_txg_is_the_largest",
        J7Judged => "j7_judged",
        J7aInstanceTableViolations => "J7a_instance_table",
        J7aAllocationRecordTreeRootViolations => "J7a_allocation_record_tree_root",
        J7aFilesystemIdentifierViolations => "J7a_filesystem_identifier",
        J7bJudged => "j7b_judged",
        J7bViolations => "J7b",
        J7cJudged => "j7c_judged",
        J7cViolations => "J7c",
        TablesDiffer => "tables_differ",
        RowsDiffer => "rows_differ",
        TablesDifferInReportedOrder => "tables_differ_in_reported_order",
        RowsDifferInReportedOrder => "rows_differ_in_reported_order",
        DirectTableHoldsARecord => "direct_table_holds_a_record",
        FullyLandedScanTableSizeChecked => "fully_landed_scan_table_size_checked",
        TablesSatisfyI812 => "i812_states_hold",
        TablesViolateI812 => "i812_states_fail",
        J6QnPremiseHoldsI812HoldsJudged => "j6qn_premise_holds_i812_holds_judged",
        J6QnPremiseHoldsI812HoldsViolations => "J6Qn_premise_holds_i812_holds",
        J6QnPremiseHoldsI812FailsJudged => "j6qn_premise_holds_i812_fails_judged",
        J6QnPremiseHoldsI812FailsViolations => "J6Qn_premise_holds_i812_fails",
        J6QnPremiseFailsI812HoldsJudged => "j6qn_premise_fails_i812_holds_judged",
        J6QnPremiseFailsI812HoldsViolations => "J6Qn_premise_fails_i812_holds",
        J6QnPremiseFailsI812FailsJudged => "j6qn_premise_fails_i812_fails_judged",
        J6QnPremiseFailsI812FailsViolations => "J6Qn_premise_fails_i812_fails",
        J8Judged => "j8_judged",
        J8aJudged => "j8a_judged",
        J8aInstanceTableViolations => "J8a_instance_table",
        J8aAllocationRecordTreeRootViolations => "J8a_allocation_record_tree_root",
        J8aUnmountMarkerViolations => "J8a_unmount_marker",
        J8bViolations => "J8b",
        J8cJudged => "j8c_judged",
        J8cViolations => "J8c",
        J8WithoutASource => "J8_without_a_source",
    }
}

labelled_enumeration! {
    /// J5 的状态对集合（登记 5.3 五个集合），另按「同段的那一段是哪一种」分出的行（登记第六节 Q6）。
    enum PairSet {
        Wide => "wide",
        CutOwnLegal => "cut_own_legal",
        CutOwnSameSegment => "cut_own_same_segment",
        CutOwnSameRecordSegment => "cut_own_same_segment_stopped_in_a_record_segment",
        CutOwnSameRootSegment => "cut_own_same_segment_stopped_in_a_root_segment",
        CutWithLegal => "cut_with_legal",
        CutWithSameSegment => "cut_with_same_segment",
        CutWithSameSegmentRootWriteDiffers => "cut_with_same_segment_root_write_differs",
        CutWithSameSegmentRootWriteAgrees => "cut_with_same_segment_root_write_agrees",
    }
}

labelled_enumeration! {
    /// 每个状态对集合上数的五个量：对数，今天的实现与理想恢复各自的 J5a、J5b 违例对数。
    enum PairColumn {
        Pairs => "pairs",
        TodayAppliedSetViolations => "today_j5a",
        TodayLandedVersionViolations => "today_j5b",
        IdealAppliedSetViolations => "ideal_j5a",
        IdealLandedVersionViolations => "ideal_j5b",
    }
}

labelled_enumeration! {
    /// 多盘 M、M6 一个状态两块盘落盘集合的关系（登记 r2 5.3「落后类别」；SQ17 把落后按方向拆两行）。
    enum LaggingClass {
        Equal => "equal",
        FirstDeviceBehind => "first_device_behind",
        SecondDeviceBehind => "second_device_behind",
        Crossed => "crossed",
    }
}

/// 落后类别各自数的量（登记 r2 SQ9、SQ17：状态数与 J1、J2、J4、J6、J6Qn、J7 的违例数，各行照原样分开）。
/// r5 档另记 J8 五格（登记 r5 5.3：J7 五行换成 J8 五行）；产物只打这一档判的那几列（[`lagging_class_columns_shown_under`]）。
const LAGGING_CLASS_COLUMNS: [Counter; 18] = [
    Counter::States,
    Counter::J1Violations,
    Counter::J2Violations,
    Counter::J4Violations,
    Counter::J6PremiseHoldsViolations,
    Counter::J6PremiseFailsViolations,
    Counter::J6QnPremiseHoldsViolations,
    Counter::J6QnPremiseFailsViolations,
    Counter::J7aInstanceTableViolations,
    Counter::J7aAllocationRecordTreeRootViolations,
    Counter::J7aFilesystemIdentifierViolations,
    Counter::J7bViolations,
    Counter::J7cViolations,
    Counter::J8aInstanceTableViolations,
    Counter::J8aAllocationRecordTreeRootViolations,
    Counter::J8aUnmountMarkerViolations,
    Counter::J8bViolations,
    Counter::J8cViolations,
];

/// 落后类别的行里打哪几列：J7 五列只在判 J7 的档（r2、r3）打，J8 五列只在 r5 档打，其余列各档都打
/// （r2 档的落后类别行因此与加 J8 之前逐字节相同）。
fn lagging_class_columns_shown_under(run_profile: RunProfile) -> Vec<Counter> {
    LAGGING_CLASS_COLUMNS
        .iter()
        .copied()
        .filter(|counter| {
            let is_a_j7_column = matches!(
                counter,
                Counter::J7aInstanceTableViolations
                    | Counter::J7aAllocationRecordTreeRootViolations
                    | Counter::J7aFilesystemIdentifierViolations
                    | Counter::J7bViolations
                    | Counter::J7cViolations
            );
            let is_a_j8_column = matches!(
                counter,
                Counter::J8aInstanceTableViolations
                    | Counter::J8aAllocationRecordTreeRootViolations
                    | Counter::J8aUnmountMarkerViolations
                    | Counter::J8bViolations
                    | Counter::J8cViolations
            );
            (!is_a_j7_column || run_profile.judges_j7())
                && (!is_a_j8_column || run_profile.judges_j8())
        })
        .collect()
}

/// 两块盘的落盘集合（按记录下标置位）：相等；0 号盘的真包含于 1 号盘的是 0 号盘落后；反过来是 1 号盘落后；其余互不包含。
fn lagging_class_of(first_device_mask: u32, second_device_mask: u32) -> LaggingClass {
    if first_device_mask == second_device_mask {
        LaggingClass::Equal
    } else if first_device_mask & second_device_mask == first_device_mask {
        LaggingClass::FirstDeviceBehind
    } else if first_device_mask & second_device_mask == second_device_mask {
        LaggingClass::SecondDeviceBehind
    } else {
        LaggingClass::Crossed
    }
}

#[derive(Clone, Debug)]
struct Tally {
    counts: Vec<u64>,
    witnesses: Vec<Option<String>>,
    pair_counts: Vec<u64>,
    above_water_maximum: usize,
    /// 下标 = 落后类别的下标 × [`LAGGING_CLASS_COLUMNS`] 的列数 + 列下标；单盘域恒为 0。
    lagging_class_counts: Vec<u64>,
}

impl Tally {
    fn new() -> Self {
        Self {
            counts: vec![0; Counter::ALL.len()],
            witnesses: vec![None; Counter::ALL.len()],
            pair_counts: vec![0; PairSet::ALL.len() * PairColumn::ALL.len()],
            above_water_maximum: 0,
            lagging_class_counts: vec![0; LaggingClass::ALL.len() * LAGGING_CLASS_COLUMNS.len()],
        }
    }

    /// 一个多盘状态判完：把它让 [`LAGGING_CLASS_COLUMNS`] 各加的数记进它的落后类别（`counts_before` 是判这个状态之前的计数）。
    fn add_to_the_lagging_class(&mut self, class: LaggingClass, counts_before: &[u64]) {
        for (column_index, counter) in LAGGING_CLASS_COLUMNS.iter().enumerate() {
            let added = self.counts[counter.index()] - counts_before[counter.index()];
            self.lagging_class_counts
                [class.index() * LAGGING_CLASS_COLUMNS.len() + column_index] += added;
        }
    }

    fn lagging_class_count(&self, class: LaggingClass, counter: Counter) -> u64 {
        let column_index = LAGGING_CLASS_COLUMNS
            .iter()
            .position(|column| *column == counter)
            .expect("只问 LAGGING_CLASS_COLUMNS 里列了的量");
        self.lagging_class_counts[class.index() * LAGGING_CLASS_COLUMNS.len() + column_index]
    }

    fn add(&mut self, counter: Counter) {
        self.counts[counter.index()] += 1;
    }

    /// 违例加一；这个量的第一条违例记一个见证（枚举次序里最先的那个，合并时按工作单元的次序保留最先的）。
    fn add_violation(&mut self, counter: Counter, describe_witness: &dyn Fn() -> String) {
        let index = counter.index();
        self.counts[index] += 1;
        if self.witnesses[index].is_none() {
            self.witnesses[index] = Some(describe_witness());
        }
    }

    fn get(&self, counter: Counter) -> u64 {
        self.counts[counter.index()]
    }

    fn witness(&self, counter: Counter) -> Option<&str> {
        self.witnesses[counter.index()].as_deref()
    }

    fn add_pair(&mut self, set: PairSet, column: PairColumn) {
        self.pair_counts[set.index() * PairColumn::ALL.len() + column.index()] += 1;
    }

    fn pair(&self, set: PairSet, column: PairColumn) -> u64 {
        self.pair_counts[set.index() * PairColumn::ALL.len() + column.index()]
    }

    /// 把后一个工作单元的计数并进来：数相加，见证保留先到的，最大值取大。
    fn merge(&mut self, later: &Self) {
        for (count, later_count) in self.counts.iter_mut().zip(&later.counts) {
            *count += later_count;
        }
        for (count, later_count) in self.pair_counts.iter_mut().zip(&later.pair_counts) {
            *count += later_count;
        }
        for (count, later_count) in self
            .lagging_class_counts
            .iter_mut()
            .zip(&later.lagging_class_counts)
        {
            *count += later_count;
        }
        for (witness, later_witness) in self.witnesses.iter_mut().zip(&later.witnesses) {
            if witness.is_none() {
                witness.clone_from(later_witness);
            }
        }
        self.above_water_maximum = self.above_water_maximum.max(later.above_water_maximum);
    }
}

// ── 单状态重放与逐态判定行（登记修订第 12 条）：只挑哪些状态要判、把判出来的量打出来，不改任何判据

/// 这一次跑只看哪些状态。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum StateProbe {
    /// 只判枚举次序里序号等于它的那一个状态，别的状态跳过不判；打出那一行就收尾退出。
    ReplayOneState { ordinal: u64 },
    /// 每个状态都判，各打一行。
    ListEveryState,
    /// 只判序号等于它的那一个状态，别的状态跳过不判；那一行记进 [`CAPTURED_STATE_ROW`]，不打、不退（钉住的红例逐行重放用）。
    CaptureOneStateRow { ordinal: u64 },
}

/// 探针开没开：正常跑时每个状态只读这一个原子量，不碰 [`STATE_PROBE`] 那把锁。
static STATE_PROBE_IS_SET: AtomicBool = AtomicBool::new(false);
/// 探针要的 (域, N) 与模式；没设就是正常跑，两个判定函数里的探针什么都不做。钉住的红例逐行重放时每一行换一次。
static STATE_PROBE: Mutex<Option<(Domain, usize, StateProbe)>> = Mutex::new(None);
/// 探针模式下已经走过的状态数：单线程照工作单元的次序走，它就是状态在枚举次序里的序号。
static NEXT_STATE_ORDINAL: AtomicU64 = AtomicU64::new(0);
/// 探针模式下打出去的 `name=state` 行数。
static STATE_ROWS_EMITTED: AtomicU64 = AtomicU64::new(0);
/// `CaptureOneStateRow` 记下的那一行（不带换行）；设探针时清空。
static CAPTURED_STATE_ROW: Mutex<Option<String>> = Mutex::new(None);

/// 设探针：(域, N) 与模式换成新的，走过的序号、打出去的行数、记下的那一行都从头算。
fn set_the_state_probe(domain: Domain, size: usize, probe: StateProbe) {
    *STATE_PROBE.lock().expect("拿着探针那把锁的地方都不 panic") = Some((domain, size, probe));
    NEXT_STATE_ORDINAL.store(0, Ordering::Relaxed);
    STATE_ROWS_EMITTED.store(0, Ordering::Relaxed);
    *CAPTURED_STATE_ROW
        .lock()
        .expect("拿着记下那一行的锁的地方都不 panic") = None;
    STATE_PROBE_IS_SET.store(true, Ordering::Release);
}

/// 此刻的探针；没设交 `None`。
fn current_state_probe() -> Option<(Domain, usize, StateProbe)> {
    if !STATE_PROBE_IS_SET.load(Ordering::Acquire) {
        return None;
    }
    *STATE_PROBE.lock().expect("拿着探针那把锁的地方都不 panic")
}

/// 一个状态开判之前探针的决定。
enum ProbeDecision {
    NotProbing,
    SkipThisState,
    ReportThisState {
        ordinal: u64,
        counts_before: Vec<u64>,
    },
}

fn begin_probing_a_state(tally: &Tally) -> ProbeDecision {
    let Some((_domain, _size, probe)) = current_state_probe() else {
        return ProbeDecision::NotProbing;
    };
    let ordinal = NEXT_STATE_ORDINAL.fetch_add(1, Ordering::Relaxed);
    match probe {
        StateProbe::ReplayOneState { ordinal: wanted }
        | StateProbe::CaptureOneStateRow { ordinal: wanted }
            if wanted != ordinal =>
        {
            ProbeDecision::SkipThisState
        }
        StateProbe::ReplayOneState { .. }
        | StateProbe::ListEveryState
        | StateProbe::CaptureOneStateRow { .. } => ProbeDecision::ReportThisState {
            ordinal,
            counts_before: tally.counts.clone(),
        },
    }
}

/// 一个状态让哪几个量加了一：照 [`Counter::ALL`] 的次序列名字，加了不止一的写 `名*次数`；一个都没加写 `none`。
fn labels_of_the_counters_a_state_added(counts_before: &[u64], counts_after: &[u64]) -> String {
    let labels: Vec<String> = Counter::ALL
        .iter()
        .zip(counts_before.iter().zip(counts_after))
        .filter_map(|(counter, (before, after))| match after - before {
            0 => None,
            1 => Some(counter.label().to_string()),
            added => Some(format!("{}*{added}", counter.label())),
        })
        .collect();
    if labels.is_empty() {
        "none".to_string()
    } else {
        labels.join(",")
    }
}

/// 一个状态判完之后：探针要报它的，打一行 `name=state`（记下模式只记不打）。重放模式下报完这一个就收尾退出。
fn finish_probing_a_state(
    decision: ProbeDecision,
    tally: &Tally,
    describe_state: &dyn Fn() -> String,
) {
    let ProbeDecision::ReportThisState {
        ordinal,
        counts_before,
    } = decision
    else {
        return;
    };
    let (domain, size, probe) = current_state_probe().expect("探针报一个状态时探针必然设过");
    let state_row = format!(
        "E7RESULT name=state domain={} ring_slots={} n={size} ordinal={ordinal} counters={} state={}",
        domain.label(),
        domain.ring().ring_slots,
        labels_of_the_counters_a_state_added(&counts_before, &tally.counts),
        describe_state()
    );
    STATE_ROWS_EMITTED.fetch_add(1, Ordering::Relaxed);
    match probe {
        StateProbe::ReplayOneState { .. } => {
            println!("{state_row}");
            // 配置行一行、状态行一行、收尾行一行。
            println!("E7RESULT name=done emitted=3");
            std::process::exit(0);
        }
        StateProbe::ListEveryState => println!("{state_row}"),
        StateProbe::CaptureOneStateRow { .. } => {
            let earlier_row = CAPTURED_STATE_ROW
                .lock()
                .expect("拿着记下那一行的锁的地方都不 panic")
                .replace(state_row);
            assert!(
                earlier_row.is_none(),
                "一个序号只对应枚举次序里的一个状态，记下模式一次设探针只记一行"
            );
        }
    }
}

/// 域名与 N 的原文 → (域, N)：域名用产物里 `domain=` 的写法，N 要在这个域最小的 N 到 12 之间。
fn domain_and_size_of(domain_text: &str, size_text: &str) -> Result<(Domain, usize), String> {
    let domain = Domain::ALL
        .iter()
        .copied()
        .find(|domain| domain.label() == domain_text)
        .ok_or_else(|| format!("不认识的域：{domain_text}"))?;
    let size = size_text
        .parse::<usize>()
        .map_err(|error| format!("N 要一个非负整数：{error}"))?;
    if size < domain.smallest_size() || size > 12 {
        return Err(format!(
            "{} 域的 N 要在 {} 到 12 之间，收到的是 {size}",
            domain.label(),
            domain.smallest_size()
        ));
    }
    Ok((domain, size))
}

/// `<域>:<N>:<序号>` 或 `<域>:<N>`：域名用产物里 `domain=` 的写法（U 域是 `U_verify_on`、`U_verify_off`）。
fn parse_the_state_probe(
    arguments: &[String],
) -> Result<Option<(Domain, usize, StateProbe)>, String> {
    let [option, value] = arguments else {
        return Ok(None);
    };
    let wants_one_state = match option.as_str() {
        "--replay-state" => true,
        "--list-states" => false,
        _other_option => return Ok(None),
    };
    let parts: Vec<&str> = value.split(':').collect();
    let expected_parts = if wants_one_state { 3 } else { 2 };
    if parts.len() != expected_parts {
        return Err(format!(
            "{option} 要 {expected_parts} 段（冒号分隔），收到的是「{value}」"
        ));
    }
    let (domain, size) = domain_and_size_of(parts[0], parts[1])?;
    let probe = if wants_one_state {
        StateProbe::ReplayOneState {
            ordinal: parts[2]
                .parse::<u64>()
                .map_err(|error| format!("序号要一个非负整数：{error}"))?,
        }
    } else {
        StateProbe::ListEveryState
    };
    Ok(Some((domain, size, probe)))
}

/// 探针模式的主流程：不跑自检与阳性对照，单线程把这个 (域, N) 照工作单元的次序走一遍。
fn run_the_state_probe(domain: Domain, size: usize, probe: StateProbe, run_profile: RunProfile) {
    install_the_panic_hook_that_stays_quiet_inside_the_replay_call();
    println!(
        "E7RESULT name=config experiment=E165 mode={} domain={} ring_slots={} n={size} self_checks=not_run positive_controls=not_run threads=1{}",
        match probe {
            StateProbe::ReplayOneState { .. } => "replay_state",
            StateProbe::ListEveryState => "list_states",
            StateProbe::CaptureOneStateRow { .. } => "capture_state",
        },
        domain.label(),
        domain.ring().ring_slots,
        profile_fields_of_the_run_rows(run_profile)
    );
    set_the_state_probe(domain, size, probe);
    let tally = tally_of_a_domain_size(
        domain,
        size,
        Implementation::TodayReplayJournal,
        run_profile,
        1,
    );
    let walked = NEXT_STATE_ORDINAL.load(Ordering::Relaxed);
    let captured_state_row = CAPTURED_STATE_ROW
        .lock()
        .expect("拿着记下那一行的锁的地方都不 panic")
        .take();
    match (probe, captured_state_row) {
        (StateProbe::CaptureOneStateRow { .. }, Some(state_row)) => {
            // 记下模式走完整个 (域, N) 才回到这里；与重放模式打同样的三行。
            println!("{state_row}");
            println!("E7RESULT name=done emitted=3");
        }
        (
            StateProbe::ReplayOneState { ordinal } | StateProbe::CaptureOneStateRow { ordinal },
            None,
        ) => {
            // 走到这里说明那个序号一直没遇到（重放模式遇到的那一刻已经退出）。
            eprintln!(
                "  ✗ {} 域 N = {size} 只有 {walked} 个状态，序号 {ordinal} 不在里面",
                domain.label()
            );
            eprintln!(
                "  → 怎么办：序号从 0 数；用 --list-states {}:{size} 看这个 (域, N) 的全部状态",
                domain.label()
            );
            std::process::exit(2);
        }
        (StateProbe::ReplayOneState { .. } | StateProbe::ListEveryState, Some(_state_row)) => {
            unreachable!("只有记下模式往 CAPTURED_STATE_ROW 里放行，设探针时清空过")
        }
        (StateProbe::ListEveryState, None) => {
            let rows = STATE_ROWS_EMITTED.load(Ordering::Relaxed);
            assert_eq!(rows, walked, "每个状态恰好一行");
            assert_eq!(
                tally.get(Counter::States),
                walked,
                "探针走过的状态数等于这个 (域, N) 数出来的状态数"
            );
            println!("E7RESULT name=done emitted={}", rows + 2);
        }
    }
}

// ── 钉住的红例逐行重放（`.claude/gate.d/e165-pinned-red-examples.tsv` 文件头的约定）：只按 ②③④ 找状态、重算 ①，不改任何判据

/// 钉住的表自己的段数（① 输入指纹、② 域、③ N、④ 序号、⑤ 出处、⑥ 变异、⑦ 原实现的逐态行、⑧ 变体的逐态行）。
const PINNED_TABLE_OWN_COLUMN_COUNT: usize = 8;

/// 红例账（`research/results/e165-*-red-examples.tsv`）一行的段数，列定义在账的表头。
const RED_EXAMPLE_LEDGER_COLUMN_COUNT: usize = 13;

/// 钉住的表一行有几段：自己的八段后面接红例账出红的那一行原样（⑨–㉑），⑨ 起各段用制表符接起来与账那一行逐字节相同。
const PINNED_TABLE_COLUMN_COUNT: usize =
    PINNED_TABLE_OWN_COLUMN_COUNT + RED_EXAMPLE_LEDGER_COLUMN_COUNT;

/// 钉住的表的段号写法，下标是段号减一。
const PINNED_TABLE_COLUMN_MARKS: [&str; PINNED_TABLE_COLUMN_COUNT] = [
    "①", "②", "③", "④", "⑤", "⑥", "⑦", "⑧", "⑨", "⑩", "⑪", "⑫", "⑬", "⑭", "⑮", "⑯", "⑰", "⑱", "⑲",
    "⑳", "㉑",
];

/// 抄来的账行里的一段与钉住的表自己的一段必须逐字相同：段号都从 1 数，账的段号按账自己数（钉表里是它加八）。
struct LedgerColumnThatMustEqualAPinnedColumn {
    ledger_column_number: usize,
    pinned_column_number: usize,
}

/// 按红例账表头的列定义：第 3、4、5 段是域、N、枚举次序里的序号，对 ②③④；第 12、13 段是原实现与变体的逐态行，对 ⑦⑧。
const LEDGER_COLUMNS_THAT_MUST_EQUAL_A_PINNED_COLUMN: [LedgerColumnThatMustEqualAPinnedColumn; 5] = [
    LedgerColumnThatMustEqualAPinnedColumn {
        ledger_column_number: 3,
        pinned_column_number: 2,
    },
    LedgerColumnThatMustEqualAPinnedColumn {
        ledger_column_number: 4,
        pinned_column_number: 3,
    },
    LedgerColumnThatMustEqualAPinnedColumn {
        ledger_column_number: 5,
        pinned_column_number: 4,
    },
    LedgerColumnThatMustEqualAPinnedColumn {
        ledger_column_number: 12,
        pinned_column_number: 7,
    },
    LedgerColumnThatMustEqualAPinnedColumn {
        ledger_column_number: 13,
        pinned_column_number: 8,
    },
];

/// 段数已经是 [`PINNED_TABLE_COLUMN_COUNT`] 的一行：抄来的账行（⑨ 起）逐对对着钉表自己的段核，不同的每一对都报出来、点名是哪一段。
fn check_that_the_copied_ledger_row_agrees_with_the_pinned_columns(
    columns: &[&str],
) -> Result<(), String> {
    assert_eq!(
        columns.len(),
        PINNED_TABLE_COLUMN_COUNT,
        "调用方 parse_a_pinned_red_example 先核过段数"
    );
    let disagreements: Vec<String> = LEDGER_COLUMNS_THAT_MUST_EQUAL_A_PINNED_COLUMN
        .iter()
        .filter_map(|pair| {
            let ledger_column_index_in_the_table =
                PINNED_TABLE_OWN_COLUMN_COUNT + pair.ledger_column_number - 1;
            let pinned_column_index = pair.pinned_column_number - 1;
            let ledger_value = columns[ledger_column_index_in_the_table];
            let pinned_value = columns[pinned_column_index];
            (ledger_value != pinned_value).then(|| {
                format!(
                    "{}（红例账第 {} 段）与 {} 不同：{} 是「{ledger_value}」，{} 是「{pinned_value}」",
                    PINNED_TABLE_COLUMN_MARKS[ledger_column_index_in_the_table],
                    pair.ledger_column_number,
                    PINNED_TABLE_COLUMN_MARKS[pinned_column_index],
                    PINNED_TABLE_COLUMN_MARKS[ledger_column_index_in_the_table],
                    PINNED_TABLE_COLUMN_MARKS[pinned_column_index],
                )
            })
        })
        .collect();
    if disagreements.is_empty() {
        Ok(())
    } else {
        Err(disagreements.join("；"))
    }
}

/// 钉住的表里的一行数据：① 与 ②③ 留原文（重算 ① 用原文），②③④ 另解出值来找状态。
#[derive(Clone, Debug, PartialEq, Eq)]
struct PinnedRedExample {
    line_number_in_the_table: usize,
    input_fingerprint: String,
    domain_text: String,
    size_text: String,
    domain: Domain,
    size: usize,
    ordinal: u64,
}

/// 表里一行数据 → [`PinnedRedExample`]：段数不是二十一（八段的旧行另报「没整行抄红例账」）、抄来的账行与 ②③④⑦⑧ 对不上、
/// 域不认识、N 不在这个域的范围、序号不是非负整数，都拒。
fn parse_a_pinned_red_example(
    line_number_in_the_table: usize,
    line: &str,
) -> Result<PinnedRedExample, String> {
    let columns: Vec<&str> = line.split('\t').collect();
    if columns.len() == PINNED_TABLE_OWN_COLUMN_COUNT {
        return Err(format!(
            "这一行没整行抄红例账，用 scripts/experiments/e165-pin-red-examples.py 补（这一行是 {PINNED_TABLE_OWN_COLUMN_COUNT} 段的旧格式，要 {PINNED_TABLE_COLUMN_COUNT} 段）"
        ));
    }
    if columns.len() != PINNED_TABLE_COLUMN_COUNT {
        return Err(format!(
            "要 {PINNED_TABLE_COLUMN_COUNT} 段（制表符分隔：自己的 {PINNED_TABLE_OWN_COLUMN_COUNT} 段后面接红例账一行的 {RED_EXAMPLE_LEDGER_COLUMN_COUNT} 段），这一行是 {} 段",
            columns.len()
        ));
    }
    check_that_the_copied_ledger_row_agrees_with_the_pinned_columns(&columns)?;
    let (domain, size) = domain_and_size_of(columns[1], columns[2])?;
    let ordinal = columns[3]
        .parse::<u64>()
        .map_err(|error| format!("④ 序号要一个非负整数：{error}"))?;
    Ok(PinnedRedExample {
        line_number_in_the_table,
        input_fingerprint: columns[0].to_string(),
        domain_text: columns[1].to_string(),
        size_text: columns[2].to_string(),
        domain,
        size,
        ordinal,
    })
}

/// 输入指纹：`sha256(<域>|<N>|<逐态行里 state= 的值，截到 ;first= 之前>)` 的十六进制，与
/// `scripts/experiments/e165-pin-red-examples.py` 的 `input_fingerprint` 同一个算法。逐态行里认不出 ` state=<不带空白的值>` 收尾交 `None`。
fn input_fingerprint_of(domain_text: &str, size_text: &str, state_row: &str) -> Option<String> {
    let (_row_before_the_state, state_text) = state_row.rsplit_once(" state=")?;
    if state_text.is_empty() || state_text.chars().any(char::is_whitespace) {
        return None;
    }
    let state_before_the_outcome = state_text
        .split_once(";first=")
        .map_or(state_text, |(before_the_outcome, _outcome)| {
            before_the_outcome
        });
    Some(sha256_hexadecimal(
        format!("{domain_text}|{size_text}|{state_before_the_outcome}").as_bytes(),
    ))
}

/// 一行钉住的红例在今天的装置上重放出来的样子。
#[derive(Clone, Debug, PartialEq, Eq)]
enum PinnedRedExampleReplay {
    /// 找到了状态，拿它的逐态行重算的指纹与 ① 相同。
    FingerprintMatches { state_row: String },
    /// 找到了状态，重算的指纹与 ① 不同：枚举次序或状态编码漂了。
    FingerprintDrifted {
        state_row: String,
        recomputed_fingerprint: String,
    },
    /// 找到了状态，逐态行里认不出 `state=` 那一段，算不出指纹。
    StateRowUnrecognized { state_row: String },
    /// 这个 (域, N) 走完了也没遇到这个序号。
    OrdinalOutsideTheDomainSize { states_walked: u64 },
}

/// 重放出来的逐态行（没遇到序号是 `None`）对着表里那一行判：纯函数，不碰探针。
fn judge_a_pinned_red_example_replay(
    example: &PinnedRedExample,
    captured_state_row: Option<String>,
    states_walked: u64,
) -> PinnedRedExampleReplay {
    let Some(state_row) = captured_state_row else {
        return PinnedRedExampleReplay::OrdinalOutsideTheDomainSize { states_walked };
    };
    match input_fingerprint_of(&example.domain_text, &example.size_text, &state_row) {
        None => PinnedRedExampleReplay::StateRowUnrecognized { state_row },
        Some(recomputed_fingerprint) if recomputed_fingerprint == example.input_fingerprint => {
            PinnedRedExampleReplay::FingerprintMatches { state_row }
        }
        Some(recomputed_fingerprint) => PinnedRedExampleReplay::FingerprintDrifted {
            state_row,
            recomputed_fingerprint,
        },
    }
}

/// 在今天的实现上重放一行：设记下模式的探针，单线程把这个 (域, N) 走一遍，交回记下的那一行与走过的状态数。
fn replay_a_pinned_red_example(
    example: &PinnedRedExample,
    run_profile: RunProfile,
) -> (Option<String>, u64) {
    set_the_state_probe(
        example.domain,
        example.size,
        StateProbe::CaptureOneStateRow {
            ordinal: example.ordinal,
        },
    );
    let _tally_of_the_one_judged_state = tally_of_a_domain_size(
        example.domain,
        example.size,
        Implementation::TodayReplayJournal,
        run_profile,
        1,
    );
    let states_walked = NEXT_STATE_ORDINAL.load(Ordering::Relaxed);
    let captured_state_row = CAPTURED_STATE_ROW
        .lock()
        .expect("拿着记下那一行的锁的地方都不 panic")
        .take();
    (captured_state_row, states_walked)
}

/// 钉住的表全文解出来的样子：数据行（非空、不以 `#` 起头）有几行，解得出的各行（按表里的次序），解不出的各行的问题（带行号）。
#[derive(Debug)]
struct PinnedTableParse {
    data_line_count: usize,
    examples: Vec<PinnedRedExample>,
    problems_of_the_rows_that_do_not_parse: Vec<String>,
}

/// 钉住的表全文 → [`PinnedTableParse`]：纯函数，不碰探针、不重放。
fn parse_the_pinned_table(table_text: &str) -> PinnedTableParse {
    let mut parse = PinnedTableParse {
        data_line_count: 0,
        examples: Vec::new(),
        problems_of_the_rows_that_do_not_parse: Vec::new(),
    };
    let data_lines = table_text
        .lines()
        .enumerate()
        .map(|(line_index, line)| (line_index + 1, line))
        .filter(|(_line_number, line)| !line.trim().is_empty() && !line.starts_with('#'));
    for (line_number, line) in data_lines {
        parse.data_line_count += 1;
        match parse_a_pinned_red_example(line_number, line) {
            Ok(example) => parse.examples.push(example),
            Err(problem) => parse
                .problems_of_the_rows_that_do_not_parse
                .push(format!("第 {line_number} 行：{problem}")),
        }
    }
    parse
}

/// `--pinned-cases <表>` 的主流程：按数据行的次序每行打一行 `name=state`；有行不对、找不到状态或指纹漂了，全部报完退 1。
fn replay_every_pinned_red_example(table_path: &str, run_profile: RunProfile) {
    let table_text = match fs::read_to_string(table_path) {
        Ok(table_text) => table_text,
        Err(error) => {
            eprintln!("  ✗ 读不了钉住的表 {table_path}：{error}");
            eprintln!("  → 怎么办：{USAGE}");
            std::process::exit(2);
        }
    };
    install_the_panic_hook_that_stays_quiet_inside_the_replay_call();
    let PinnedTableParse {
        data_line_count,
        examples,
        problems_of_the_rows_that_do_not_parse,
    } = parse_the_pinned_table(&table_text);
    println!(
        "E7RESULT name=config experiment=E165 mode=pinned_cases rows={data_line_count} self_checks=not_run positive_controls=not_run threads=1"
    );
    let mut problems: Vec<String> = Vec::new();
    if data_line_count == 0 {
        problems.push(format!("{table_path} 一行数据都没有"));
    }
    problems.extend(problems_of_the_rows_that_do_not_parse);
    let mut state_rows_printed: u64 = 0;
    for example in examples {
        let (captured_state_row, states_walked) =
            replay_a_pinned_red_example(&example, run_profile);
        let place = format!(
            "第 {} 行（{}:{}:{}）",
            example.line_number_in_the_table,
            example.domain_text,
            example.size_text,
            example.ordinal
        );
        match judge_a_pinned_red_example_replay(&example, captured_state_row, states_walked) {
            PinnedRedExampleReplay::FingerprintMatches { state_row } => {
                println!("{state_row}");
                state_rows_printed += 1;
            }
            PinnedRedExampleReplay::FingerprintDrifted {
                state_row,
                recomputed_fingerprint,
            } => {
                println!("{state_row}");
                state_rows_printed += 1;
                problems.push(format!(
                    "{place}：拿这个状态重算的指纹是 {recomputed_fingerprint}，表里 ① 是 {}",
                    example.input_fingerprint
                ));
            }
            PinnedRedExampleReplay::StateRowUnrecognized { state_row } => {
                println!("{state_row}");
                state_rows_printed += 1;
                problems.push(format!("{place}：逐态行里认不出 state= 那一段"));
            }
            PinnedRedExampleReplay::OrdinalOutsideTheDomainSize { states_walked } => {
                problems.push(format!(
                    "{place}：这个 (域, N) 只有 {states_walked} 个状态，序号不在里面"
                ));
            }
        }
    }
    // 配置行一行、逐态行若干、收尾行一行。
    println!("E7RESULT name=done emitted={}", state_rows_printed + 2);
    if !problems.is_empty() {
        for problem in &problems {
            eprintln!("  ✗ {problem}");
        }
        eprintln!(
            "  → 怎么办：指纹漂了先查装置的枚举次序与状态编码变没变；行不对的照表头的二十一段格式（自己的八段后面接红例账出红的那一行原样）找回原行，八段的旧行用 scripts/experiments/e165-pin-red-examples.py 补整行（表只由它写）"
        );
        std::process::exit(1);
    }
}

/// 把工作单元分给几条线程跑，再按工作单元的次序合并：计数是加法，见证取次序里最先的，结果与线程数、线程次序无关。
fn tally_of_work_units<Unit: Sync>(
    units: &[Unit],
    thread_count: usize,
    work: &(dyn Fn(usize, &Unit) -> Tally + Sync),
) -> Tally {
    let next_unit_index = AtomicUsize::new(0);
    let finished_units: Mutex<Vec<(usize, Tally)>> = Mutex::new(Vec::new());
    std::thread::scope(|scope| {
        for _worker in 0..thread_count.clamp(1, units.len().max(1)) {
            scope.spawn(|| loop {
                let unit_index = next_unit_index.fetch_add(1, Ordering::Relaxed);
                let Some(unit) = units.get(unit_index) else {
                    break;
                };
                let tally = work(unit_index, unit);
                finished_units
                    .lock()
                    .expect("别的线程没有在拿着这把锁的时候 panic")
                    .push((unit_index, tally));
            });
        }
    });
    let mut finished_units = finished_units
        .into_inner()
        .expect("线程都已经收回，没有谁在拿着这把锁的时候 panic");
    finished_units.sort_by_key(|(unit_index, _tally)| *unit_index);
    assert_eq!(finished_units.len(), units.len(), "每个工作单元恰好跑一遍");
    let mut total = Tally::new();
    for (_unit_index, tally) in &finished_units {
        total.merge(tally);
    }
    total
}

// ── 被判的实现：今天的 `replay_journal`，或阳性对照的一种假结局（登记 5.4）

/// 阳性对照的假结局。每一种说它把真结局或理想结局改坏在哪里。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ControlKind {
    /// J0、A0：在指定的一态上 panic，别的态照真结局。
    PanicsOnTheDesignatedState,
    /// J1：理想恢复，凡施加过的，返回根的新根段改成时间线之外的值。
    OffTimelineRootWheneverApplied,
    /// J2：理想恢复，下一次发布有记录在盘上但不完整时多落一版。
    LandsOnePublishTooFarOverAnIncompletePublish,
    /// J3：不看上限，落到连续完整的最远一版。
    IgnoresTheInFlightLimit,
    /// J4：永远返回所选根。
    AlwaysReturnsTheChosenRoot,
    /// J5a、J5b：记录全在盘上时返回所选根、施加集为空，其余照理想恢复。
    ReturnsTheChosenRootWhenEveryRecordIsOnDisk,
    /// J5a 与 J5b 分得开：理想恢复，但施加集报成累计的 (0, j\*]。
    IdealWithTheCumulativeAppliedSet,
    /// J6：第一次照真结局，第二次调用的结局换成「再多落一版」（没有下一版时换成时间线之外的根）。
    SecondCallMovesOn,
    /// AS 三条与 A-互：拿 S 的某一种读法当实现。
    SpecificationWithReading(FollowReading),
    /// A-结构：返回根的新根段取表里一条同实例、水位之上、带提交标记而不带末条标志的记录的。
    RootFromARecordWithoutTheLastRecordFlag,
    /// A-限：报告的 `prefix_applied` 改成上限加一。
    ReportsOneRecordOverTheLimit,
    /// IA1：所选根 txg 是最大值时返回「没报错」。
    ReportsNoFailureOnTheLargestTxg,
    /// IA2：关掉点名验证时 `verification_passed` 报成 1。
    ReportsOneVerificationWithoutVerifying,
    /// J6Qn「(i)(ii) 之外都算违例」：第一次照真结局，第二次换成「报 `RootPublishCarriesMoreThanOneLastRecordFlag…`」。
    SecondCallReportsMoreThanOneLastRecordFlag,
    /// J6Qn「(ii) 要核 txg 顶格」：第一次照真结局，第二次换成「报 `CHOSEN_ROOT_CHECKPOINT_TXG_HAS_NO_SUCCESSOR`」。
    SecondCallReportsTheChosenRootTxgHasNoSuccessor,
    /// J6 与 J6Qn 分得开：第一次取理想恢复（落到 j\*），第二次照真调用。
    IdealFirstCallThenTodaysSecondCall,
    /// J7a：理想恢复，施加过时把投影之外的一个字段换成落到的那一版 R_j 的。
    IdealWithAFieldOfTheLandedVersion(FieldOfTheRebuiltRoot),
    /// J7b：理想恢复，施加过时卸载记号抄所选根的。
    IdealCopyingTheChosenUnmountMarkerWhenApplied,
    /// J7c：理想恢复，一条没施加时卸载记号写成 `NotWritten…`；r5 档的 C8d 也是它（登记 r5 5.4）。
    IdealWritingNotWrittenWhenNothingIsApplied,
    /// C8a（登记 r5 5.4）：理想恢复，施加过时把实例表指针、分配记录树根、卸载记号三项都抄所选根的。
    IdealCopyingTheChosenRecordFieldsWhenApplied,
    /// C8b（登记 r5 5.4）：理想恢复，施加过时卸载记号一律写 [`UNMOUNT_MARKER_THE_C8B_CONTROL_WRITES_WHEN_APPLIED`]。
    IdealWritingOneUnmountMarkerWhenApplied,
    /// 多盘「任一份在即在」（登记 r2 5.5）：理想恢复，但 j\* 只数 0 号盘那一份。
    IdealCountingOnlyTheFirstDeviceCopies,
    /// T-表（登记 r2 5.5）：扫盘建表之后删掉表里键最大的那条记录，直接建的表不动。
    ScanTableLosesItsLargestKeyRecord,
    /// T-行（登记 r2 5.5）：扫盘建表那一遍的实现结局换成 `Panicked`，表不动。
    ScanPassOutcomeBecomesAPanic,
    /// 对抗一族 J6Qn 按 I-8.12 拆开的四行（登记 r3 5.7 的 KF3）：第一次调用返回所选根，第二次调用报
    /// `RootPublishCarriesMoreThanOneLastRecordFlag…`；第二次既不是返回根、也不是 txg 顶格，每个判到的态都是 J6Qn 违例。
    FirstCallReturnsTheChosenRootAndSecondCallReportsMoreThanOneLastRecordFlag,
}

/// J7a 判的三个字段（登记 r2 第一节）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum FieldOfTheRebuiltRoot {
    InstanceTable,
    AllocationRecordTreeRoot,
    FilesystemIdentifier,
}

impl FieldOfTheRebuiltRoot {
    const fn label(self) -> &'static str {
        match self {
            Self::InstanceTable => "instance_table",
            Self::AllocationRecordTreeRoot => "allocation_record_tree_root",
            Self::FilesystemIdentifier => "filesystem_identifier",
        }
    }

    const ALL: [Self; 3] = [
        Self::InstanceTable,
        Self::AllocationRecordTreeRoot,
        Self::FilesystemIdentifier,
    ];

    const fn violations_counter(self) -> Counter {
        match self {
            Self::InstanceTable => Counter::J7aInstanceTableViolations,
            Self::AllocationRecordTreeRoot => Counter::J7aAllocationRecordTreeRootViolations,
            Self::FilesystemIdentifier => Counter::J7aFilesystemIdentifierViolations,
        }
    }

    /// 把 `root` 的这个字段换成 `source` 的。
    fn copy_from(self, root: &mut RootRecord, source: &RootRecord) {
        match self {
            Self::InstanceTable => root.instance_table = source.instance_table,
            Self::AllocationRecordTreeRoot => {
                root.allocation_record_tree_root = source.allocation_record_tree_root;
            }
            Self::FilesystemIdentifier => root.filesystem_identifier = source.filesystem_identifier,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Implementation {
    TodayReplayJournal,
    Control(ControlKind),
}

/// C8b 的假结局施加过时写的卸载记号（登记 r5 5.4：旧规则的第三项「由记录施加的一律不是卸载写的」）。
const UNMOUNT_MARKER_THE_C8B_CONTROL_WRITES_WHEN_APPLIED: UnmountMarker =
    UnmountMarker::NotWrittenByTheUnmountSequence;

/// 一次重放施加的发布集合 (low, high]，high 同时是落到的版本。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct AppliedInterval {
    low: usize,
    high: usize,
}

/// J5a（施加集读法）：(inner.low, inner.high] ⊆ (outer.low, outer.high]。空集包含于任何集合。
fn applied_set_is_contained(inner: AppliedInterval, outer: AppliedInterval) -> bool {
    inner.low == inner.high || (outer.low <= inner.low && inner.high <= outer.high)
}

/// J5b（落到的版本读法）为假：子状态落到的版本晚于父状态落到的版本。
fn landed_version_is_later(inner: AppliedInterval, outer: AppliedInterval) -> bool {
    inner.high > outer.high
}

/// 合法历史一族一个状态上、照条款算出来的量（不读实现的任何输出）。
struct LegalTruth {
    in_flight_limit: usize,
    farthest_version: usize,
    above_water_record_count: usize,
}

fn legal_truth_of(state: &LegalState<'_>, records: &RecordTable) -> LegalTruth {
    let in_flight_limit = state.history.ring.local_in_flight_limit();
    let chosen_fields = state.history.version_fields(state.chosen_version);
    let above_water_record_count = records
        .values()
        .filter(|record| {
            record.instance.0 == chosen_fields.instance
                && record.checkpoint_txg.0 > chosen_fields.checkpoint_txg
        })
        .count();
    LegalTruth {
        in_flight_limit,
        farthest_version: farthest_version_of(state, Some(in_flight_limit)),
        above_water_record_count,
    }
}

/// 落到时间线第 `version` 版的一次返回：没施加时原样返回所选根。
fn returned_at_version(
    state: &LegalState<'_>,
    chosen_root: &RootRecord,
    version: usize,
) -> ReturnedReplay {
    let prefix_applied = record_count_of_versions_after_the_chosen_one(state, version);
    ReturnedReplay {
        root: if version == state.chosen_version {
            *chosen_root
        } else {
            rebuilt_root_of_the_ideal_recovery(state.history, version, chosen_root)
        },
        prefix_applied,
        verification_passed: if state.verify_named_units {
            prefix_applied
        } else {
            0
        },
    }
}

/// 理想恢复落到时间线第 `version` 版时由记录重建的根：r5 档照 D23（journal 的角色与格式） 已定项 15 今天的原句（后三项取那次发布的
/// 记录里的、fsid 抄所选根，登记 r5 5.3），别的档照各自登记时的「投影之外抄所选根」（r1–r4 档一个字不改）。
fn rebuilt_root_of_the_ideal_recovery(
    history: &History,
    version: usize,
    chosen_root: &RootRecord,
) -> RootRecord {
    let landed_fields = history.version_fields(version);
    if history.run_profile.judges_j8() {
        landed_fields.to_root_rebuilt_from_the_record_over(
            history
                .run_profile
                .record_fields_of_the_timeline_publish(version),
            chosen_root,
        )
    } else {
        landed_fields.to_root_rebuilt_over(chosen_root)
    }
}

/// 一条根，六个字段都不等于时间线上任何一版；投影之外照「由记录重建」抄 `rebuilt_over`。
fn off_timeline_root(
    checkpoint_txg: u64,
    distinguishing_number: u64,
    rebuilt_over: &RootRecord,
) -> RootRecord {
    VersionFields {
        instance: CHOSEN_INSTANCE,
        checkpoint_txg,
        segment: NewRootSegment::off_timeline(distinguishing_number, checkpoint_txg),
    }
    .to_root_rebuilt_over(rebuilt_over)
}

/// 合法历史一族里第一次调用的结局：今天的实现，或阳性对照的假结局。
fn first_outcome_on_a_legal_state(
    implementation: Implementation,
    state: &LegalState<'_>,
    truth: &LegalTruth,
    chosen_root: &RootRecord,
    records: &RecordTable,
    disk: &MemoryDisk<'_>,
    is_the_designated_state: bool,
) -> ReplayOutcome {
    let today = || call_replay_journal(disk, chosen_root, records, state.verify_named_units);
    let ideal = || returned_at_version(state, chosen_root, truth.farthest_version);
    let control = match implementation {
        Implementation::TodayReplayJournal => return today(),
        Implementation::Control(control) => control,
    };
    match control {
        ControlKind::PanicsOnTheDesignatedState => {
            if is_the_designated_state {
                ReplayOutcome::Panicked
            } else {
                today()
            }
        }
        ControlKind::OffTimelineRootWheneverApplied => {
            let mut returned = ideal();
            if truth.farthest_version > state.chosen_version {
                returned.root = off_timeline_root(returned.root.checkpoint_txg.0, 1, chosen_root);
            }
            ReplayOutcome::Returned(returned)
        }
        ControlKind::LandsOnePublishTooFarOverAnIncompletePublish => {
            let next_version = truth.farthest_version + 1;
            let lands_one_too_far = next_version < state.history.chosen_instance_publish_count
                && publish_has_a_record_on_disk(state, next_version)
                && !publish_is_completely_on_disk(state, next_version);
            ReplayOutcome::Returned(returned_at_version(
                state,
                chosen_root,
                if lands_one_too_far {
                    next_version
                } else {
                    truth.farthest_version
                },
            ))
        }
        ControlKind::IgnoresTheInFlightLimit => ReplayOutcome::Returned(returned_at_version(
            state,
            chosen_root,
            farthest_version_of(state, None),
        )),
        ControlKind::AlwaysReturnsTheChosenRoot => ReplayOutcome::Returned(returned_at_version(
            state,
            chosen_root,
            state.chosen_version,
        )),
        ControlKind::ReturnsTheChosenRootWhenEveryRecordIsOnDisk => {
            let every_record_is_on_disk = (0..state.presence.len())
                .all(|record_index| record_has_a_copy_on_disk(state, record_index));
            ReplayOutcome::Returned(returned_at_version(
                state,
                chosen_root,
                if every_record_is_on_disk {
                    state.chosen_version
                } else {
                    truth.farthest_version
                },
            ))
        }
        ControlKind::IdealWithTheCumulativeAppliedSet => ReplayOutcome::Returned(ideal()),
        ControlKind::ReportsNoFailureOnTheLargestTxg => match today() {
            ReplayOutcome::Failed(_kind) => ReplayOutcome::Returned(returned_at_version(
                state,
                chosen_root,
                state.chosen_version,
            )),
            outcome @ (ReplayOutcome::Panicked | ReplayOutcome::Returned(_)) => outcome,
        },
        ControlKind::ReportsOneVerificationWithoutVerifying => match today() {
            ReplayOutcome::Returned(mut returned) => {
                returned.verification_passed = 1;
                ReplayOutcome::Returned(returned)
            }
            outcome @ (ReplayOutcome::Panicked | ReplayOutcome::Failed(_)) => outcome,
        },
        ControlKind::IdealFirstCallThenTodaysSecondCall => ReplayOutcome::Returned(ideal()),
        ControlKind::IdealWithAFieldOfTheLandedVersion(field) => {
            let mut returned = ideal();
            if truth.farthest_version > state.chosen_version {
                field.copy_from(
                    &mut returned.root,
                    &state.history.timeline_root(truth.farthest_version),
                );
            }
            ReplayOutcome::Returned(returned)
        }
        ControlKind::IdealCopyingTheChosenUnmountMarkerWhenApplied => {
            let mut returned = ideal();
            if truth.farthest_version > state.chosen_version {
                returned.root.unmount_marker = chosen_root.unmount_marker;
            }
            ReplayOutcome::Returned(returned)
        }
        ControlKind::IdealWritingNotWrittenWhenNothingIsApplied => {
            let mut returned = ideal();
            if truth.farthest_version == state.chosen_version {
                returned.root.unmount_marker = UnmountMarker::NotWrittenByTheUnmountSequence;
            }
            ReplayOutcome::Returned(returned)
        }
        ControlKind::IdealCopyingTheChosenRecordFieldsWhenApplied => {
            let mut returned = ideal();
            if truth.farthest_version > state.chosen_version {
                returned.root.instance_table = chosen_root.instance_table;
                returned.root.allocation_record_tree_root = chosen_root.allocation_record_tree_root;
                returned.root.unmount_marker = chosen_root.unmount_marker;
            }
            ReplayOutcome::Returned(returned)
        }
        ControlKind::IdealWritingOneUnmountMarkerWhenApplied => {
            let mut returned = ideal();
            if truth.farthest_version > state.chosen_version {
                returned.root.unmount_marker = UNMOUNT_MARKER_THE_C8B_CONTROL_WRITES_WHEN_APPLIED;
            }
            ReplayOutcome::Returned(returned)
        }
        ControlKind::IdealCountingOnlyTheFirstDeviceCopies => {
            let first_device_view = LegalState {
                history: state.history,
                presence: state.presence,
                mirroring: JournalMirroring::FirstDeviceOnly,
                residue: state.residue,
                chosen_version: state.chosen_version,
                verify_named_units: state.verify_named_units,
            };
            ReplayOutcome::Returned(returned_at_version(
                state,
                chosen_root,
                farthest_version_of(&first_device_view, Some(truth.in_flight_limit)),
            ))
        }
        ControlKind::SecondCallMovesOn
        | ControlKind::SpecificationWithReading(_)
        | ControlKind::RootFromARecordWithoutTheLastRecordFlag
        | ControlKind::ReportsOneRecordOverTheLimit
        | ControlKind::SecondCallReportsMoreThanOneLastRecordFlag
        | ControlKind::SecondCallReportsTheChosenRootTxgHasNoSuccessor
        | ControlKind::FirstCallReturnsTheChosenRootAndSecondCallReportsMoreThanOneLastRecordFlag
        | ControlKind::ScanTableLosesItsLargestKeyRecord
        | ControlKind::ScanPassOutcomeBecomesAPanic => today(),
    }
}

/// J6 的第二次调用：拿第一次返回的根当所选根、同一张表、同一个读者、同一个开关再调一次。
fn second_outcome(
    implementation: Implementation,
    first_root: &RootRecord,
    next_timeline_version_root: Option<RootRecord>,
    records: &RecordTable,
    disk: &MemoryDisk<'_>,
    verify_named_units: bool,
) -> ReplayOutcome {
    match implementation {
        Implementation::Control(ControlKind::SecondCallMovesOn) => {
            ReplayOutcome::Returned(ReturnedReplay {
                root: next_timeline_version_root.unwrap_or_else(|| {
                    off_timeline_root(first_root.checkpoint_txg.0, 2, first_root)
                }),
                prefix_applied: 1,
                verification_passed: 0,
            })
        }
        Implementation::Control(
            ControlKind::SecondCallReportsMoreThanOneLastRecordFlag
            | ControlKind::FirstCallReturnsTheChosenRootAndSecondCallReportsMoreThanOneLastRecordFlag,
        ) => ReplayOutcome::Failed(FailureKind::RootPublishCarriesMoreThanOneLastRecordFlag),
        Implementation::Control(ControlKind::SecondCallReportsTheChosenRootTxgHasNoSuccessor) => {
            ReplayOutcome::Failed(FailureKind::ChosenRootTxgHasNoSuccessor)
        }
        Implementation::TodayReplayJournal | Implementation::Control(_) => {
            call_replay_journal(disk, first_root, records, verify_named_units)
        }
    }
}

/// J6（不动点）：判第二次是不是「返回根且与第一次的根全字段相等」，按前提真假分两行记；为假的态上另看第三遍与第二遍相不相等（只报不判）。
#[allow(
    clippy::too_many_arguments,
    reason = "每个参数都是这一判要用的一样东西，凑不成一个带不变量的概念"
)]
fn judge_the_fixed_point(
    implementation: Implementation,
    run_profile: RunProfile,
    first: &ReturnedReplay,
    next_timeline_version_root: Option<RootRecord>,
    premise_holds: bool,
    table_satisfies_i812: Option<bool>,
    records: &RecordTable,
    disk: &MemoryDisk<'_>,
    verify_named_units: bool,
    describe_witness: &dyn Fn() -> String,
    tally: &mut Tally,
) {
    let second = second_outcome(
        implementation,
        &first.root,
        next_timeline_version_root,
        records,
        disk,
        verify_named_units,
    );
    let fixed_point_holds = matches!(&second, ReplayOutcome::Returned(second_returned) if second_returned.root == first.root);
    tally.add(if premise_holds {
        Counter::J6PremiseHoldsJudged
    } else {
        Counter::J6PremiseFailsJudged
    });
    if run_profile.judges_j6qn() {
        if first.root.checkpoint_txg.0 == u64::MAX {
            tally.add(Counter::FirstReturnedRootTxgIsTheLargest);
        }
        judge_the_fixed_point_under_qn(
            &first.root,
            &second,
            fixed_point_holds,
            premise_holds,
            table_satisfies_i812,
            describe_witness,
            tally,
        );
    }
    if fixed_point_holds {
        return;
    }
    let describe_with_the_second_call = || {
        format!(
            "{};second={}",
            describe_witness(),
            describe_outcome(&second)
        )
    };
    tally.add_violation(
        if premise_holds {
            Counter::J6PremiseHoldsViolations
        } else {
            Counter::J6PremiseFailsViolations
        },
        &describe_with_the_second_call,
    );
    let third_equals_second = match &second {
        ReplayOutcome::Returned(second_returned) => {
            let third =
                call_replay_journal(disk, &second_returned.root, records, verify_named_units);
            matches!(&third, ReplayOutcome::Returned(third_returned) if third_returned.root == second_returned.root)
        }
        ReplayOutcome::Panicked | ReplayOutcome::Failed(_) => false,
    };
    tally.add(if third_equals_second {
        Counter::J6ThirdCallEqualsSecond
    } else {
        Counter::J6ThirdCallDiffersOrIsMissing
    });
}

/// J6Qn (ii)（登记 r2 第一节）：第二次报 `CHOSEN_ROOT_CHECKPOINT_TXG_HAS_NO_SUCCESSOR`，且判据自己看到 ρ_1 的 txg 是 u64 最大值
/// （「因所选根 txg 顶格」由判据核，不只认报错的字样；不取 Qw）。
fn second_call_failed_because_the_first_root_txg_is_the_largest(
    second: &ReplayOutcome,
    first_root: &RootRecord,
) -> bool {
    let first_root_txg_is_the_largest = first_root.checkpoint_txg.0 == u64::MAX;
    let second_call_reports_no_successor = matches!(
        second,
        ReplayOutcome::Failed(FailureKind::ChosenRootTxgHasNoSuccessor)
    );
    second_call_reports_no_successor && first_root_txg_is_the_largest
}

/// I-8.12（同一实例里 journal 记录的 txg 随计数器不减；`research/prompts/m3-inv-b-r1-main-verification.md` 第二节第 9 行）
/// 在「这一态交给 `replay_journal` 的那张记录表」上成不成立（登记 r3 第一节「Q3 的拆法」）：表按（实例，计数器）升序摊开，同一实例里
/// 相邻两条记录后一条的 txg 小于前一条即不成立（相邻不减与任两条不减等价）。`parse` 拒收的序号 0 进不了表，判在表上就不看它。
/// 本地写法，不调 `crates/` 里 Verus 前件 `journal_txg_does_not_decrease_with_the_counter_within_an_instance`。
fn record_table_satisfies_i812(records: &RecordTable) -> bool {
    let mut previous_record_of_the_same_instance: Option<&JournalRecord> = None;
    for ((instance, _counter), record) in records {
        if let Some(previous_record) = previous_record_of_the_same_instance {
            if previous_record.instance == *instance
                && record.checkpoint_txg < previous_record.checkpoint_txg
            {
                return false;
            }
        }
        previous_record_of_the_same_instance = Some(record);
    }
    true
}

/// J6Qn 的前提真 / 假与 I-8.12 成不成立，四个格各自的「判了几态」与「违例几态」两个量（登记 r3 5.6）。
const fn j6qn_cell_counters(premise_holds: bool, i812_holds: bool) -> (Counter, Counter) {
    match (premise_holds, i812_holds) {
        (true, true) => (
            Counter::J6QnPremiseHoldsI812HoldsJudged,
            Counter::J6QnPremiseHoldsI812HoldsViolations,
        ),
        (true, false) => (
            Counter::J6QnPremiseHoldsI812FailsJudged,
            Counter::J6QnPremiseHoldsI812FailsViolations,
        ),
        (false, true) => (
            Counter::J6QnPremiseFailsI812HoldsJudged,
            Counter::J6QnPremiseFailsI812HoldsViolations,
        ),
        (false, false) => (
            Counter::J6QnPremiseFailsI812FailsJudged,
            Counter::J6QnPremiseFailsI812FailsViolations,
        ),
    }
}

/// J6Qn（登记 r2 第一节，判决 S6 取 Qn）：第二次 (i) 返回根且与 ρ_1 全字段相等，或 (ii) 因 ρ_1 的 txg 顶格报
/// `CHOSEN_ROOT_CHECKPOINT_TXG_HAS_NO_SUCCESSOR`，记相等；别的报错与 panic 一律违例。前提真 / 假两行照原 J6 分。
/// `table_satisfies_i812` 给了（r3 档的对抗一族）时，同一次判再按这张表满不满足 I-8.12 记进四格里的一格（登记 r3 5.6），
/// 两行原有的计数一个不动。
fn judge_the_fixed_point_under_qn(
    first_root: &RootRecord,
    second: &ReplayOutcome,
    fixed_point_holds: bool,
    premise_holds: bool,
    table_satisfies_i812: Option<bool>,
    describe_witness: &dyn Fn() -> String,
    tally: &mut Tally,
) {
    tally.add(if premise_holds {
        Counter::J6QnPremiseHoldsJudged
    } else {
        Counter::J6QnPremiseFailsJudged
    });
    let split_cell =
        table_satisfies_i812.map(|i812_holds| j6qn_cell_counters(premise_holds, i812_holds));
    if let Some((judged_counter, _violations_counter)) = split_cell {
        tally.add(judged_counter);
    }
    let counts_as_equal_under_qn = fixed_point_holds
        || second_call_failed_because_the_first_root_txg_is_the_largest(second, first_root);
    if counts_as_equal_under_qn {
        return;
    }
    let describe_with_the_second_call = || {
        format!(
            "{};first_root_txg={};second={}",
            describe_witness(),
            first_root.checkpoint_txg.0,
            describe_outcome(second)
        )
    };
    tally.add_violation(
        if premise_holds {
            Counter::J6QnPremiseHoldsViolations
        } else {
            Counter::J6QnPremiseFailsViolations
        },
        &describe_with_the_second_call,
    );
    if let Some((_judged_counter, violations_counter)) = split_cell {
        tally.add_violation(violations_counter, &describe_with_the_second_call);
    }
}

/// J7（登记 r2 第一节，判决 S1 后半）：凡第一次调用返回根 ρ 的态都判。J7a：实例表指针、分配记录树根、fsid 各等于所选根的；
/// 投影不等于所选根的（由记录重建过）J7b：卸载记号是 `NotWritten…`；投影等于所选根的（一条没施加）J7c：卸载记号等于所选根的。
/// 「重建过没有」由判据比投影定，不读报告的 `prefix_applied`。
fn judge_the_fields_beyond_the_projection(
    chosen_root: &RootRecord,
    returned_root: &RootRecord,
    describe_witness: &dyn Fn() -> String,
    tally: &mut Tally,
) {
    let projection_equals_the_chosen_one =
        VersionFields::of_root(returned_root) == VersionFields::of_root(chosen_root);
    let describe_with_both_roots = || {
        format!(
            "{};chosen_root={};returned_root={};projection_equals_the_chosen_one={projection_equals_the_chosen_one}",
            describe_witness(),
            FieldsBeyondTheProjection::of_root(chosen_root).describe(),
            FieldsBeyondTheProjection::of_root(returned_root).describe()
        )
    };
    tally.add(Counter::J7Judged);
    if returned_root.instance_table != chosen_root.instance_table {
        tally.add_violation(
            Counter::J7aInstanceTableViolations,
            &describe_with_both_roots,
        );
    }
    if returned_root.allocation_record_tree_root != chosen_root.allocation_record_tree_root {
        tally.add_violation(
            Counter::J7aAllocationRecordTreeRootViolations,
            &describe_with_both_roots,
        );
    }
    if returned_root.filesystem_identifier != chosen_root.filesystem_identifier {
        tally.add_violation(
            Counter::J7aFilesystemIdentifierViolations,
            &describe_with_both_roots,
        );
    }
    let expected_unmount_marker = if projection_equals_the_chosen_one {
        chosen_root.unmount_marker
    } else {
        UnmountMarker::NotWrittenByTheUnmountSequence
    };
    let (judged_counter, violations_counter) = if projection_equals_the_chosen_one {
        (Counter::J7cJudged, Counter::J7cViolations)
    } else {
        (Counter::J7bJudged, Counter::J7bViolations)
    };
    tally.add(judged_counter);
    if returned_root.unmount_marker != expected_unmount_marker {
        tally.add_violation(violations_counter, &describe_with_both_roots);
    }
}

/// J8（登记 r5 5.3，照 D23（journal 的角色与格式） 已定项 15 今天的原句重新登记）：凡第一次调用返回根 ρ 的态都判。
/// J8b：ρ 的 fsid 等于所选根的。投影等于所选根的（一条没施加）J8c：fsid、实例表指针、分配记录树根、卸载记号四项都等于所选根的。
/// 投影不等于所选根的（由记录重建过）J8a 三格：实例表指针、分配记录树根、卸载记号各等于生成器真相里新根段投影等于 ρ 的投影的那条记录的；
/// 生成器真相里没有这样一条记录的不判 J8a，记「无出处」（只报）。「重建过没有」由判据比投影定，不读报告的 `prefix_applied`。
/// `landed_version_root` 是 ρ 落到的那一版时间线根（对抗一族与落点不在时间线上的给 `None`），只进见证。
fn judge_the_fields_beyond_the_projection_against_the_generated_records(
    chosen_root: &RootRecord,
    returned_root: &RootRecord,
    record_truth: &GeneratedRecordTruth,
    landed_version_root: Option<RootRecord>,
    describe_witness: &dyn Fn() -> String,
    tally: &mut Tally,
) {
    let returned_projection = VersionFields::of_root(returned_root);
    let projection_equals_the_chosen_one =
        returned_projection == VersionFields::of_root(chosen_root);
    let source_of_the_returned_projection = record_truth.fields_of(returned_projection);
    let describe_with_both_roots = || {
        format!(
            "{};chosen_root={};returned_root={};record_of_the_returned_projection={};landed_version_root={};projection_equals_the_chosen_one={projection_equals_the_chosen_one}",
            describe_witness(),
            FieldsBeyondTheProjection::of_root(chosen_root).describe(),
            FieldsBeyondTheProjection::of_root(returned_root).describe(),
            source_of_the_returned_projection
                .map_or_else(|| "none".to_string(), RecordFieldsBeyondTheProjection::describe),
            landed_version_root.map_or_else(
                || "none".to_string(),
                |landed_root| FieldsBeyondTheProjection::of_root(&landed_root).describe()
            )
        )
    };
    tally.add(Counter::J8Judged);
    let expected_filesystem_identifier = chosen_root.filesystem_identifier;
    if returned_root.filesystem_identifier != expected_filesystem_identifier {
        tally.add_violation(Counter::J8bViolations, &describe_with_both_roots);
    }
    if projection_equals_the_chosen_one {
        tally.add(Counter::J8cJudged);
        let returned_equals_the_chosen_root_beyond_the_projection =
            FieldsBeyondTheProjection::of_root(returned_root)
                == FieldsBeyondTheProjection::of_root(chosen_root);
        if !returned_equals_the_chosen_root_beyond_the_projection {
            tally.add_violation(Counter::J8cViolations, &describe_with_both_roots);
        }
        return;
    }
    let Some(fields_from_the_new_root_segment) = source_of_the_returned_projection else {
        tally.add_violation(Counter::J8WithoutASource, &describe_with_both_roots);
        return;
    };
    tally.add(Counter::J8aJudged);
    let expected_fields_of_the_rebuilt_root = fields_from_the_new_root_segment;
    let returned_fields = RecordFieldsBeyondTheProjection::of_root(returned_root);
    for (field_differs, violations_counter) in [
        (
            returned_fields.instance_table != expected_fields_of_the_rebuilt_root.instance_table,
            Counter::J8aInstanceTableViolations,
        ),
        (
            returned_fields.allocation_record_tree_root
                != expected_fields_of_the_rebuilt_root.allocation_record_tree_root,
            Counter::J8aAllocationRecordTreeRootViolations,
        ),
        (
            returned_fields.unmount_marker != expected_fields_of_the_rebuilt_root.unmount_marker,
            Counter::J8aUnmountMarkerViolations,
        ),
    ] {
        if field_differs {
            tally.add_violation(violations_counter, &describe_with_both_roots);
        }
    }
}

/// 这个域的状态进不进「S 自己的对照」（登记 5.4：L、L 小环、U 开着验证、X、E1）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum SpecificationSelfCheck {
    Judged,
    NotJudged,
}

/// 一个合法历史状态的写法（逐态行的 `state=` 与每条违例的见证）：形状、每条记录在盘上与否、1 号盘那几份、残留、所选版本、j\*、第一次调用的结局。
fn describe_a_legal_state_with_its_first_outcome(
    state: &LegalState<'_>,
    farthest_version: usize,
    first: &ReplayOutcome,
) -> String {
    let presence: Vec<String> = state
        .presence
        .iter()
        .map(|record_presence| match record_presence {
            RecordPresence::NotOnDisk => "-".to_string(),
            RecordPresence::OnDisk(unit_state) => unit_state.label().to_string(),
        })
        .collect();
    let second_device_presence = match state.mirroring {
        JournalMirroring::FirstDeviceOnly => String::new(),
        JournalMirroring::BothDevices {
            second_device_copy_is_on_disk,
        } => format!(
            ";on_second_device={}",
            second_device_copy_is_on_disk
                .iter()
                .map(|is_on_disk| if *is_on_disk { "good" } else { "-" })
                .collect::<Vec<&str>>()
                .join(",")
        ),
    };
    format!(
        "shape={};on_disk={}{second_device_presence};residue_at={};chosen={};farthest={};verify={};first={}",
        state.history.shape_description,
        presence.join(","),
        state.residue.map_or_else(
            || "none".to_string(),
            |residue| residue.record_index.to_string()
        ),
        state.chosen_version,
        farthest_version,
        state.verify_named_units,
        describe_outcome(first)
    )
}

/// 判合法历史一族的一个状态：J0–J4、J6、IA1、IA2、域非空见证、在飞上限的分布、S 自己的对照。记录表怎么建由档的 `table_source` 定
/// （登记 r2 5.4）：`Both` 时两张表各判一遍、逐态比，汇总的计数取直接建表那一遍。
/// 返回这一次重放施加的发布区间（C 域的状态对要它）；panic 或落点不在时间线上时是 `None`。
fn judge_a_legal_state(
    state: &LegalState<'_>,
    implementation: Implementation,
    specification_self_check: SpecificationSelfCheck,
    is_the_designated_state: bool,
    tally: &mut Tally,
) -> Option<AppliedInterval> {
    let probe_decision = begin_probing_a_state(tally);
    if matches!(probe_decision, ProbeDecision::SkipThisState) {
        return None;
    }
    let run_profile = state.history.run_profile;
    let (direct_table, disk, truths) = record_table_and_disk_of(state);
    let judge_on = |table: &RecordTable,
                    implementation_of_the_pass: Implementation,
                    designated_in_the_pass: bool,
                    tally_of_the_pass: &mut Tally| {
        judge_a_legal_state_on_a_table(
            state,
            implementation_of_the_pass,
            specification_self_check,
            designated_in_the_pass,
            table,
            &disk,
            &truths,
            tally_of_the_pass,
        )
    };
    let judgement = match run_profile.table_source {
        RecordTableSource::Direct => judge_on(
            &direct_table,
            implementation,
            is_the_designated_state,
            tally,
        ),
        RecordTableSource::Scanned => {
            let scanned_table = scanned_record_table_of(&disk, implementation);
            assert_the_scan_table_of_a_fully_landed_state(
                state,
                &scanned_table,
                implementation,
                tally,
            );
            judge_on(
                &scanned_table,
                implementation,
                is_the_designated_state,
                tally,
            )
        }
        RecordTableSource::Both => {
            let scanned_table = scanned_record_table_of(&disk, implementation);
            let counts_before_the_state = tally.counts.clone();
            let (scan_pass_implementation, scan_pass_is_designated) =
                implementation_and_designation_of_the_scan_pass(
                    implementation,
                    is_the_designated_state,
                );
            let mut scanned_tally = Tally::new();
            let scanned_judgement = judge_on(
                &scanned_table,
                scan_pass_implementation,
                scan_pass_is_designated,
                &mut scanned_tally,
            );
            let scanned_summary = JudgedRowSummary {
                first_outcome: scanned_judgement.first_outcome,
                counter_labels: counter_labels_a_state_added_to(
                    &scanned_tally,
                    &Tally::new().counts,
                ),
            };
            let direct_judgement = judge_on(
                &direct_table,
                implementation,
                is_the_designated_state,
                tally,
            );
            let direct_summary = JudgedRowSummary {
                first_outcome: direct_judgement.first_outcome,
                counter_labels: counter_labels_a_state_added_to(tally, &counts_before_the_state),
            };
            let describe_state = || {
                describe_a_legal_state_with_its_first_outcome(
                    state,
                    direct_judgement.farthest_version,
                    &direct_judgement.first_outcome,
                )
            };
            add_the_comparison_of_two_judged_tables(
                tally,
                (Counter::TablesDiffer, Counter::RowsDiffer),
                (&direct_table, &direct_summary),
                (&scanned_table, &scanned_summary),
                &describe_state,
            );
            if !direct_table.is_empty() {
                tally.add(Counter::DirectTableHoldsARecord);
            }
            assert_the_scan_table_of_a_fully_landed_state(
                state,
                &scanned_table,
                implementation,
                tally,
            );
            if run_profile.reported_disk_order == ReportedDiskOrder::SecondDeviceBeforeFirst {
                let (reported_order_table, _reported_order_disk, reported_order_truths) =
                    record_table_and_disk_of_in_device_order(
                        state,
                        run_profile.reported_disk_order.device_numbers(),
                    );
                let mut reported_order_tally = Tally::new();
                let reported_order_judgement = judge_a_legal_state_on_a_table(
                    state,
                    implementation,
                    specification_self_check,
                    is_the_designated_state,
                    &reported_order_table,
                    &disk,
                    &reported_order_truths,
                    &mut reported_order_tally,
                );
                let reported_order_summary = JudgedRowSummary {
                    first_outcome: reported_order_judgement.first_outcome,
                    counter_labels: counter_labels_a_state_added_to(
                        &reported_order_tally,
                        &Tally::new().counts,
                    ),
                };
                add_the_comparison_of_two_judged_tables(
                    tally,
                    (
                        Counter::TablesDifferInReportedOrder,
                        Counter::RowsDifferInReportedOrder,
                    ),
                    (&reported_order_table, &reported_order_summary),
                    (&scanned_table, &scanned_summary),
                    &describe_state,
                );
            }
            direct_judgement
        }
    };
    finish_probing_a_state(probe_decision, tally, &|| {
        describe_a_legal_state_with_its_first_outcome(
            state,
            judgement.farthest_version,
            &judgement.first_outcome,
        )
    });
    judgement.interval
}

/// [`judge_a_legal_state`] 在一张给定的记录表上判一个状态：不管探针、不管两种建表法怎么比，只数这一遍判出来的量。
#[allow(
    clippy::too_many_arguments,
    reason = "每个参数都是这一判要用的一样东西，凑不成一个带不变量的概念"
)]
fn judge_a_legal_state_on_a_table(
    state: &LegalState<'_>,
    implementation: Implementation,
    specification_self_check: SpecificationSelfCheck,
    is_the_designated_state: bool,
    records: &RecordTable,
    disk: &MemoryDisk<'_>,
    truths: &[RecordTruthOnDisk],
    tally: &mut Tally,
) -> LegalJudgement {
    let truth = legal_truth_of(state, records);
    let history = state.history;
    let chosen_version = state.chosen_version;
    let chosen_root = history.timeline_root(chosen_version);
    let first = first_outcome_on_a_legal_state(
        implementation,
        state,
        &truth,
        &chosen_root,
        records,
        disk,
        is_the_designated_state,
    );
    let describe_witness =
        || describe_a_legal_state_with_its_first_outcome(state, truth.farthest_version, &first);

    tally.add(Counter::States);
    if chosen_version > 0 {
        tally.add(Counter::ChosenRootIsALaterVersion);
    }
    tally.above_water_maximum = tally
        .above_water_maximum
        .max(truth.above_water_record_count);
    if truth.above_water_record_count > truth.in_flight_limit {
        tally.add(Counter::AboveWaterOverLimit);
    }
    if truth.above_water_record_count == truth.in_flight_limit {
        tally.add(Counter::AboveWaterAtLimit);
    }
    if truth.farthest_version > chosen_version {
        tally.add(Counter::FarthestAtLeastOne);
    }
    if truth.farthest_version > chosen_version + 1 {
        tally.add(Counter::FarthestAtLeastTwo);
    }

    if specification_self_check == SpecificationSelfCheck::Judged {
        let view = SpecificationView {
            chosen_root: &chosen_root,
            records,
            in_flight_limit: truth.in_flight_limit,
            verify_named_units: state.verify_named_units,
            truths,
        };
        let expected_by_the_farthest_version =
            SpecificationExpectation::ReturnsRoot(history.version_fields(truth.farthest_version));
        let mismatch_counters = [
            Counter::SpecificationWholeTableMismatch,
            Counter::SpecificationGapContinuesMismatch,
            Counter::SpecificationGapStopsMismatch,
        ];
        for (reading, mismatch_counter) in
            FollowReading::REGISTERED.into_iter().zip(mismatch_counters)
        {
            if specification_expectation(&view, reading) != expected_by_the_farthest_version {
                tally.add_violation(mismatch_counter, &describe_witness);
            }
        }
    }

    // IA1、IA2 是实现锚点，不是条款判据（登记 5.3 末段）。
    if chosen_root.checkpoint_txg.0 == u64::MAX {
        tally.add(Counter::Ia1Judged);
        if first != ReplayOutcome::Failed(FailureKind::ChosenRootTxgHasNoSuccessor) {
            tally.add_violation(Counter::Ia1Violations, &describe_witness);
        }
    }
    if !state.verify_named_units {
        tally.add(Counter::Ia2Judged);
        if matches!(&first, ReplayOutcome::Returned(returned) if returned.verification_passed != 0)
        {
            tally.add_violation(Counter::Ia2Violations, &describe_witness);
        }
    }

    // 落点 j：报错记 j = c；返回的根六个字段等于 R_j 的那个 j；没有这样的 j 是「不在时间线上」。
    let landing: Option<usize> = match &first {
        ReplayOutcome::Panicked => {
            tally.add_violation(Counter::Panicked, &describe_witness);
            None
        }
        ReplayOutcome::Failed(_kind) => {
            tally.add_violation(Counter::Failed, &describe_witness);
            tally.add(Counter::J6FirstCallFailed);
            Some(chosen_version)
        }
        ReplayOutcome::Returned(returned) => {
            let landing =
                history.timeline_version_with_fields(VersionFields::of_root(&returned.root));
            if landing.is_none() {
                tally.add(Counter::OffTimelineLandings);
            }
            let next_timeline_version_root = landing
                .map(|version| version + 1)
                .filter(|next_version| *next_version < history.chosen_instance_publish_count)
                .map(|next_version| {
                    history
                        .version_fields(next_version)
                        .to_root_rebuilt_over(&returned.root)
                });
            judge_the_fixed_point(
                implementation,
                history.run_profile,
                returned,
                next_timeline_version_root,
                truth.above_water_record_count <= truth.in_flight_limit,
                None,
                records,
                disk,
                state.verify_named_units,
                &describe_witness,
                tally,
            );
            if history.run_profile.judges_j7() {
                judge_the_fields_beyond_the_projection(
                    &chosen_root,
                    &returned.root,
                    &describe_witness,
                    tally,
                );
            }
            if history.run_profile.judges_j8() {
                judge_the_fields_beyond_the_projection_against_the_generated_records(
                    &chosen_root,
                    &returned.root,
                    &history.record_truth,
                    landing.map(|version| history.timeline_root(version)),
                    &describe_witness,
                    tally,
                );
            }
            landing
        }
    };

    // J1 返回的根：落点存在且不早于所选根。
    let returned_root_holds = landing.is_some_and(|version| version >= chosen_version);
    if !returned_root_holds {
        tally.add_violation(Counter::J1Violations, &describe_witness);
    }
    // J2 根加盘：J1 为真，且 (c, j] 里每次发布都完整在盘上。
    let root_plus_disk_holds = returned_root_holds
        && landing.is_some_and(|version| {
            (chosen_version + 1..=version)
                .all(|applied_version| publish_is_completely_on_disk(state, applied_version))
        });
    if !root_plus_disk_holds {
        tally.add_violation(Counter::J2Violations, &describe_witness);
    }
    // J3 在飞上限：J1 为真，且 (c, j] 里各次发布的记录条数之和不超过在飞上限。
    let within_the_in_flight_limit = returned_root_holds
        && landing.is_some_and(|version| {
            let applied_record_count =
                record_count_of_versions_after_the_chosen_one(state, version);
            applied_record_count <= truth.in_flight_limit
        });
    if !within_the_in_flight_limit {
        tally.add_violation(Counter::J3Violations, &describe_witness);
    }
    // J4 最远一版：j = j*。
    if landing != Some(truth.farthest_version) {
        tally.add_violation(Counter::J4Violations, &describe_witness);
    }
    // 域非空见证：实现施加过至少一次、两次以上发布。
    if landing.is_some_and(|version| version > chosen_version) {
        tally.add(Counter::AppliedAtLeastOne);
    }
    if landing.is_some_and(|version| version > chosen_version + 1) {
        tally.add(Counter::AppliedAtLeastTwo);
    }

    let interval = landing
        .filter(|version| *version >= chosen_version)
        .map(|version| AppliedInterval {
            low: if implementation
                == Implementation::Control(ControlKind::IdealWithTheCumulativeAppliedSet)
            {
                0
            } else {
                chosen_version
            },
            high: version,
        });
    LegalJudgement {
        interval,
        first_outcome: first,
        farthest_version: truth.farthest_version,
    }
}

// ── 域（登记 5.2）

labelled_enumeration! {
    /// 登记 5.2 的域；U 域验证开、关各算一个。
    enum Domain {
        LegalBigRing => "L",
        LegalSmallRingSix => "L6",
        LegalSmallRingNine => "L9",
        NamedUnitsVerified => "U_verify_on",
        NamedUnitsNotVerified => "U_verify_off",
        MonotoneBigRing => "C",
        MonotoneSmallRingSix => "C6",
        AdversarialBigRing => "A",
        AdversarialSmallRingSix => "A6",
        AdversarialBackChain => "AC",
        CrossInstance => "X",
        TxgOneBelowTheLargest => "E1",
        TxgAtTheLargest => "E2",
        Residue => "R",
        MirroredBigRing => "M",
        MirroredSmallRingSix => "M6",
        MirroredMonotoneBigRing => "MC",
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Family {
    LegalHistory,
    Adversarial,
    /// 登记 r2 5.3 的多盘域 M、M6、MC：只在 `--profile r2 --family multi_disk` 时跑。
    MultiDisk,
}

impl Domain {
    const fn ring(self) -> RingGeometry {
        match self {
            Self::LegalBigRing
            | Self::NamedUnitsVerified
            | Self::NamedUnitsNotVerified
            | Self::MonotoneBigRing
            | Self::AdversarialBigRing
            | Self::AdversarialBackChain
            | Self::CrossInstance
            | Self::TxgOneBelowTheLargest
            | Self::TxgAtTheLargest
            | Self::Residue
            | Self::MirroredBigRing
            | Self::MirroredMonotoneBigRing => RingGeometry::BIG,
            Self::LegalSmallRingSix
            | Self::MonotoneSmallRingSix
            | Self::AdversarialSmallRingSix
            | Self::MirroredSmallRingSix => RingGeometry::SMALL_SIX,
            Self::LegalSmallRingNine => RingGeometry::SMALL_NINE,
        }
    }

    const fn family(self) -> Family {
        match self {
            Self::LegalBigRing
            | Self::LegalSmallRingSix
            | Self::LegalSmallRingNine
            | Self::NamedUnitsVerified
            | Self::NamedUnitsNotVerified
            | Self::MonotoneBigRing
            | Self::MonotoneSmallRingSix
            | Self::CrossInstance
            | Self::TxgOneBelowTheLargest
            | Self::TxgAtTheLargest
            | Self::Residue => Family::LegalHistory,
            Self::AdversarialBigRing
            | Self::AdversarialSmallRingSix
            | Self::AdversarialBackChain => Family::Adversarial,
            Self::MirroredBigRing | Self::MirroredSmallRingSix | Self::MirroredMonotoneBigRing => {
                Family::MultiDisk
            }
        }
    }

    /// `--only` 认的名字：U 同时选中验证开、关两遍。
    const fn selection_name(self) -> &'static str {
        match self {
            Self::NamedUnitsVerified | Self::NamedUnitsNotVerified => "U",
            Self::LegalBigRing
            | Self::LegalSmallRingSix
            | Self::LegalSmallRingNine
            | Self::MonotoneBigRing
            | Self::MonotoneSmallRingSix
            | Self::AdversarialBigRing
            | Self::AdversarialSmallRingSix
            | Self::AdversarialBackChain
            | Self::CrossInstance
            | Self::TxgOneBelowTheLargest
            | Self::TxgAtTheLargest
            | Self::Residue
            | Self::MirroredBigRing
            | Self::MirroredSmallRingSix
            | Self::MirroredMonotoneBigRing => self.label(),
        }
    }

    /// 这个域里最小的 N（或 Nc、n）。
    const fn smallest_size(self) -> usize {
        match self {
            Self::CrossInstance | Self::Residue => 2,
            Self::LegalBigRing
            | Self::LegalSmallRingSix
            | Self::LegalSmallRingNine
            | Self::NamedUnitsVerified
            | Self::NamedUnitsNotVerified
            | Self::MonotoneBigRing
            | Self::MonotoneSmallRingSix
            | Self::AdversarialBigRing
            | Self::AdversarialSmallRingSix
            | Self::AdversarialBackChain
            | Self::TxgOneBelowTheLargest
            | Self::TxgAtTheLargest
            | Self::MirroredBigRing
            | Self::MirroredSmallRingSix
            | Self::MirroredMonotoneBigRing => 1,
        }
    }

    /// 这个域的状态进不进「S 自己的对照」（登记 5.4：L、L 小环、U 开着验证、X、E1）。
    const fn specification_self_check(self) -> SpecificationSelfCheck {
        match self {
            Self::LegalBigRing
            | Self::LegalSmallRingSix
            | Self::LegalSmallRingNine
            | Self::NamedUnitsVerified
            | Self::CrossInstance
            | Self::TxgOneBelowTheLargest => SpecificationSelfCheck::Judged,
            Self::NamedUnitsNotVerified
            | Self::MonotoneBigRing
            | Self::MonotoneSmallRingSix
            | Self::AdversarialBigRing
            | Self::AdversarialSmallRingSix
            | Self::AdversarialBackChain
            | Self::TxgAtTheLargest
            | Self::Residue
            | Self::MirroredBigRing
            | Self::MirroredSmallRingSix
            | Self::MirroredMonotoneBigRing => SpecificationSelfCheck::NotJudged,
        }
    }
}

/// 每个域每个 N 的状态数（登记第七节乙类锚点，出自登记里那份脚本的闭式与暴力枚举；下标 0 对应这个域最小的 N）。
const fn pinned_state_counts(domain: Domain) -> &'static [u64] {
    match domain {
        Domain::LegalBigRing => &[
            2, 12, 72, 432, 2_592, 15_552, 93_312, 559_872, 3_359_232, 20_155_392,
        ],
        Domain::LegalSmallRingSix => &[2, 12, 40, 176, 672, 2_752],
        Domain::LegalSmallRingNine => &[2, 12, 72, 304, 1_568, 7_872, 38_016, 189_184, 934_400],
        Domain::NamedUnitsVerified | Domain::NamedUnitsNotVerified => {
            &[6, 108, 1_944, 34_992, 629_856, 11_337_408]
        }
        Domain::MonotoneBigRing => &[4, 32, 256, 2_048, 16_384, 131_072],
        Domain::MonotoneSmallRingSix => &[4, 32, 192, 1_280, 8_192],
        Domain::AdversarialBigRing => &[146, 10_658, 778_034, 56_796_482],
        Domain::AdversarialSmallRingSix => &[146, 10_658, 778_034],
        Domain::AdversarialBackChain => &[290, 42_050, 6_097_250],
        Domain::CrossInstance => &[4, 48, 432, 3_456, 25_920, 186_624],
        Domain::TxgOneBelowTheLargest => &[2, 12, 64, 320],
        Domain::TxgAtTheLargest => &[2, 8, 32],
        Domain::Residue => &[4, 64, 688, 6_208],
        // 登记 r2 第七节乙类：M 是 4 × 12^(N − 1)；M6 每次发布至多 2 条；MC 是 Nc = 1..4。
        Domain::MirroredBigRing => &[4, 48, 576, 6_912, 82_944, 995_328],
        Domain::MirroredSmallRingSix => &[4, 48, 320, 2_816, 21_504, 176_128],
        Domain::MirroredMonotoneBigRing => &[8, 128, 2_048, 32_768],
    }
}

/// L 域的见证（j\* ≥ 1 的态数, j\* ≥ 2 的态数），N = 1..7（登记第七节乙类锚点：照条款写的 j\* 的分布，不是实现的结果）。
const PINNED_FARTHEST_VERSION_WITNESSES_OF_THE_BIG_RING: [(u64, u64); 7] = [
    (0, 0),
    (2, 0),
    (16, 2),
    (104, 20),
    (640, 144),
    (3_872, 928),
    (23_296, 5_728),
];

/// C 域每个 Nc 的锚点（登记第七节乙类；小环的「甲合法」「乙合法」两组是登记第十二节补的）。
struct PinnedMonotoneCounts {
    wide_pairs: u64,
    cut_own_legal_states: u64,
    cut_own_legal_pairs: u64,
    cut_own_same_segment_pairs: u64,
    cut_with_legal_states: u64,
    cut_with_legal_pairs: u64,
    cut_with_same_segment_pairs: u64,
}

const fn pinned_monotone_counts(domain: Domain) -> &'static [PinnedMonotoneCounts] {
    const fn row(
        wide_pairs: u64,
        cut_own: (u64, u64, u64),
        cut_with: (u64, u64, u64),
    ) -> PinnedMonotoneCounts {
        PinnedMonotoneCounts {
            wide_pairs,
            cut_own_legal_states: cut_own.0,
            cut_own_legal_pairs: cut_own.1,
            cut_own_same_segment_pairs: cut_own.2,
            cut_with_legal_states: cut_with.0,
            cut_with_legal_pairs: cut_with.1,
            cut_with_same_segment_pairs: cut_with.2,
        }
    }
    const BIG_RING: [PinnedMonotoneCounts; 6] = [
        row(9, (3, 6, 5), (4, 9, 9)),
        row(135, (15, 43, 31), (23, 80, 71)),
        row(2_025, (71, 280, 189), (118, 600, 489)),
        row(30_375, (323, 1_721, 1_143), (565, 4_075, 3_159)),
        row(455_625, (1_427, 10_210, 6_885), (2_584, 25_995, 19_737)),
        row(
            6_834_375,
            (6_167, 59_319, 41_391),
            (11_443, 159_266, 121_095),
        ),
    ];
    const SMALL_RING_SIX: [PinnedMonotoneCounts; 5] = [
        row(9, (3, 6, 5), (4, 9, 9)),
        row(135, (15, 43, 31), (23, 80, 71)),
        row(1_701, (35, 136, 73), (54, 276, 165)),
        row(22_599, (99, 481, 215), (157, 1_059, 503)),
        row(295_245, (231, 1_354, 505), (368, 3_083, 1_181)),
    ];
    match domain {
        Domain::MonotoneBigRing => &BIG_RING,
        Domain::MonotoneSmallRingSix => &SMALL_RING_SIX,
        Domain::LegalBigRing
        | Domain::LegalSmallRingSix
        | Domain::LegalSmallRingNine
        | Domain::NamedUnitsVerified
        | Domain::NamedUnitsNotVerified
        | Domain::AdversarialBigRing
        | Domain::AdversarialSmallRingSix
        | Domain::AdversarialBackChain
        | Domain::CrossInstance
        | Domain::TxgOneBelowTheLargest
        | Domain::TxgAtTheLargest
        | Domain::Residue
        | Domain::MirroredBigRing
        | Domain::MirroredSmallRingSix
        | Domain::MirroredMonotoneBigRing => &[],
    }
}

/// M、M6 每个 N 的锚点（登记 r2 第七节乙类，`anchors_r2.py`、`anchors_r2b.py`、`anchors_r2c.py`）：三类（相等, 落后, 互不包含）、
/// 落后的一个方向（0 号盘落后与 1 号盘落后两行相等）、见证（j\* ≥ 1, j\* ≥ 2；记录任一份在即在）。
struct PinnedMirroredCounts {
    equal_states: u64,
    lagging_states: u64,
    crossed_states: u64,
    lagging_states_in_one_direction: u64,
    farthest_at_least_one: u64,
    farthest_at_least_two: u64,
}

const fn pinned_mirrored_counts(domain: Domain) -> &'static [PinnedMirroredCounts] {
    const fn row(
        classes: (u64, u64, u64),
        lagging_states_in_one_direction: u64,
        witnesses: (u64, u64),
    ) -> PinnedMirroredCounts {
        PinnedMirroredCounts {
            equal_states: classes.0,
            lagging_states: classes.1,
            crossed_states: classes.2,
            lagging_states_in_one_direction,
            farthest_at_least_one: witnesses.0,
            farthest_at_least_two: witnesses.1,
        }
    }
    const MIRRORED_BIG_RING: [PinnedMirroredCounts; 6] = [
        row((2, 2, 0), 1, (0, 0)),
        row((12, 30, 6), 15, (12, 0)),
        row((72, 342, 162), 171, (216, 36)),
        row((432, 3_510, 2_970), 1_755, (3_024, 864)),
        row((2_592, 34_182, 46_170), 17_091, (38_880, 14_256)),
        row((15_552, 323_190, 656_586), 161_595, (482_112, 202_176)),
    ];
    const MIRRORED_SMALL_RING_SIX: [PinnedMirroredCounts; 6] = [
        row((2, 2, 0), 1, (0, 0)),
        row((12, 30, 6), 15, (12, 0)),
        row((40, 190, 90), 95, (216, 36)),
        row((176, 1_430, 1_210), 715, (1_824, 432)),
        row((672, 8_862, 11_970), 4_431, (14_208, 2_880)),
        row((2_752, 57_190, 116_186), 28_595, (115_200, 25_344)),
    ];
    match domain {
        Domain::MirroredBigRing => &MIRRORED_BIG_RING,
        Domain::MirroredSmallRingSix => &MIRRORED_SMALL_RING_SIX,
        Domain::LegalBigRing
        | Domain::LegalSmallRingSix
        | Domain::LegalSmallRingNine
        | Domain::NamedUnitsVerified
        | Domain::NamedUnitsNotVerified
        | Domain::MonotoneBigRing
        | Domain::MonotoneSmallRingSix
        | Domain::AdversarialBigRing
        | Domain::AdversarialSmallRingSix
        | Domain::AdversarialBackChain
        | Domain::CrossInstance
        | Domain::TxgOneBelowTheLargest
        | Domain::TxgAtTheLargest
        | Domain::Residue
        | Domain::MirroredMonotoneBigRing => &[],
    }
}

/// MC 每个 Nc 的锚点（登记 r2 第七节乙类）：宽对数；见证（所选根不是 R_0 的态数, j\* − c ≥ 1, j\* − c ≥ 2）。
struct PinnedMirroredMonotoneCounts {
    wide_pairs: u64,
    chosen_root_is_a_later_version: u64,
    farthest_at_least_one: u64,
    farthest_at_least_two: u64,
}

const fn pinned_mirrored_monotone_counts(
    domain: Domain,
) -> &'static [PinnedMirroredMonotoneCounts] {
    const fn row(wide_pairs: u64, witnesses: (u64, u64, u64)) -> PinnedMirroredMonotoneCounts {
        PinnedMirroredMonotoneCounts {
            wide_pairs,
            chosen_root_is_a_later_version: witnesses.0,
            farthest_at_least_one: witnesses.1,
            farthest_at_least_two: witnesses.2,
        }
    }
    const MIRRORED_MONOTONE_BIG_RING: [PinnedMirroredMonotoneCounts; 4] = [
        row(27, (4, 3, 0)),
        row(1_215, (80, 42, 9)),
        row(54_675, (1_472, 636, 180)),
        row(2_460_375, (25_856, 9_960, 2_988)),
    ];
    match domain {
        Domain::MirroredMonotoneBigRing => &MIRRORED_MONOTONE_BIG_RING,
        Domain::LegalBigRing
        | Domain::LegalSmallRingSix
        | Domain::LegalSmallRingNine
        | Domain::NamedUnitsVerified
        | Domain::NamedUnitsNotVerified
        | Domain::MonotoneBigRing
        | Domain::MonotoneSmallRingSix
        | Domain::AdversarialBigRing
        | Domain::AdversarialSmallRingSix
        | Domain::AdversarialBackChain
        | Domain::CrossInstance
        | Domain::TxgOneBelowTheLargest
        | Domain::TxgAtTheLargest
        | Domain::Residue
        | Domain::MirroredBigRing
        | Domain::MirroredSmallRingSix => &[],
    }
}

/// 位数为 `bit_count` 的全部子集掩码，空集在内。
fn every_subset_mask(bit_count: usize) -> Range<u32> {
    0..(1u32 << bit_count)
}

fn bit_is_set(mask: u32, bit: usize) -> bool {
    (mask >> bit) & 1 == 1
}

// ── 合法历史一族的工作单元

/// 一段历史就是一个工作单元；R 域另带残留落在哪一条记录上。
#[derive(Clone, Debug)]
struct LegalWorkUnit {
    plan: HistoryPlan,
    residue_record_index: Option<usize>,
}

fn history_plan(
    domain: Domain,
    first_txg: u64,
    chosen_instance_publishes: Vec<PublishShape>,
    next_instance_publishes: Vec<PublishShape>,
    run_profile: RunProfile,
) -> HistoryPlan {
    HistoryPlan {
        run_profile,
        ring: domain.ring(),
        first_txg,
        chosen_instance_publishes,
        next_instance_publishes,
        names_units: match domain {
            Domain::NamedUnitsVerified | Domain::NamedUnitsNotVerified => true,
            Domain::LegalBigRing
            | Domain::LegalSmallRingSix
            | Domain::LegalSmallRingNine
            | Domain::MonotoneBigRing
            | Domain::MonotoneSmallRingSix
            | Domain::AdversarialBigRing
            | Domain::AdversarialSmallRingSix
            | Domain::AdversarialBackChain
            | Domain::CrossInstance
            | Domain::TxgOneBelowTheLargest
            | Domain::TxgAtTheLargest
            | Domain::Residue
            | Domain::MirroredBigRing
            | Domain::MirroredSmallRingSix
            | Domain::MirroredMonotoneBigRing => false,
        },
    }
}

/// 一个合法历史域在记录总数 `size` 上的全部工作单元（登记 5.2 每个域的定义）。
fn legal_work_units(domain: Domain, size: usize, run_profile: RunProfile) -> Vec<LegalWorkUnit> {
    let plain = |first_txg: u64, sequences: Vec<Vec<PublishShape>>| -> Vec<LegalWorkUnit> {
        sequences
            .into_iter()
            .map(|sequence| LegalWorkUnit {
                plan: history_plan(domain, first_txg, sequence, Vec::new(), run_profile),
                residue_record_index: None,
            })
            .collect()
    };
    let in_flight_limit = domain.ring().local_in_flight_limit();
    match domain {
        // M：同 L，每条记录在两块盘上各一份（登记 r2 5.3）；落盘子集在 [`tally_of_a_mirrored_history`] 里两盘各取。
        Domain::LegalBigRing
        | Domain::NamedUnitsVerified
        | Domain::NamedUnitsNotVerified
        | Domain::MirroredBigRing => plain(ORDINARY_FIRST_TXG, publish_sequences_of(size, None)),
        // 小环：每次发布不超过上限条（合法写侧一次发布最多切上限条）；M6 同 L6。
        Domain::LegalSmallRingSix | Domain::LegalSmallRingNine | Domain::MirroredSmallRingSix => {
            plain(
                ORDINARY_FIRST_TXG,
                publish_sequences_of(size, Some(in_flight_limit)),
            )
        }
        // X：实例 g 的一段合法历史之后接实例 g + 1 的一段合法历史，两段各至少 1 条。
        Domain::CrossInstance => {
            let mut units = Vec::new();
            for chosen_instance_records in 1..size {
                for chosen_sequence in publish_sequences_of(chosen_instance_records, None) {
                    for next_sequence in publish_sequences_of(size - chosen_instance_records, None)
                    {
                        units.push(LegalWorkUnit {
                            plan: history_plan(
                                domain,
                                ORDINARY_FIRST_TXG,
                                chosen_sequence.clone(),
                                next_sequence,
                                run_profile,
                            ),
                            residue_record_index: None,
                        });
                    }
                }
            }
            units
        }
        // E1：t_0 = u64 最大值 − 1、至多两次发布。
        Domain::TxgOneBelowTheLargest => plain(
            u64::MAX - 1,
            publish_sequences_of(size, None)
                .into_iter()
                .filter(|sequence| sequence.len() <= 2)
                .collect(),
        ),
        // E2：t_0 = u64 最大值、只有 P_0。
        Domain::TxgAtTheLargest => plain(
            u64::MAX,
            publish_sequences_of(size, None)
                .into_iter()
                .filter(|sequence| sequence.len() == 1)
                .collect(),
        ),
        // R：至少两次发布；残留落在 P_0 之后的哪一条全扫。
        Domain::Residue => {
            let mut units = Vec::new();
            for sequence in publish_sequences_of(size, None) {
                if sequence.len() < 2 {
                    continue;
                }
                for residue_record_index in sequence[0].record_count..size {
                    units.push(LegalWorkUnit {
                        plan: history_plan(
                            domain,
                            ORDINARY_FIRST_TXG,
                            sequence.clone(),
                            Vec::new(),
                            run_profile,
                        ),
                        residue_record_index: Some(residue_record_index),
                    });
                }
            }
            units
        }
        // C：P_0 一条记录，之后 Nc 条记录；小环每次发布至多上限条。MC 同 C 大环（登记 r2 5.3）。
        Domain::MonotoneBigRing
        | Domain::MonotoneSmallRingSix
        | Domain::MirroredMonotoneBigRing => {
            let largest_publish =
                (domain == Domain::MonotoneSmallRingSix).then_some(in_flight_limit);
            publish_sequences_of(size, largest_publish)
                .into_iter()
                .map(|sequence| {
                    let mut publishes = vec![PublishShape {
                        record_count: 1,
                        commit_marker_bits: 0,
                    }];
                    publishes.extend(sequence);
                    LegalWorkUnit {
                        plan: history_plan(
                            domain,
                            ORDINARY_FIRST_TXG,
                            publishes,
                            Vec::new(),
                            run_profile,
                        ),
                        residue_record_index: None,
                    }
                })
                .collect()
        }
        Domain::AdversarialBigRing
        | Domain::AdversarialSmallRingSix
        | Domain::AdversarialBackChain => {
            unreachable!("对抗一族不走合法历史的生成器")
        }
    }
}

/// 一段历史的全部落盘子集（U 域：每条在盘上的记录另取点名单元五态），逐个判。所选根是 R_0。
fn tally_of_a_legal_history(
    domain: Domain,
    unit: &LegalWorkUnit,
    unit_index: usize,
    implementation: Implementation,
) -> Tally {
    let mut history = generate_history(&unit.plan);
    let residue = unit
        .residue_record_index
        .map(|record_index| generate_residue(&history, record_index));
    if let Some(generated_residue) = &residue {
        history
            .record_truth
            .note_the_record(&generated_residue.record);
    }
    let record_count = history.slots.len();
    let verify_named_units = domain != Domain::NamedUnitsNotVerified;
    let mut tally = Tally::new();
    let mut presence = vec![RecordPresence::NotOnDisk; record_count];
    let judge = |presence_of_the_state: &[RecordPresence],
                 is_the_first_state: bool,
                 tally_of_the_unit: &mut Tally| {
        let state = LegalState {
            history: &history,
            presence: presence_of_the_state,
            mirroring: JournalMirroring::FirstDeviceOnly,
            residue: residue.as_ref(),
            chosen_version: 0,
            verify_named_units,
        };
        judge_a_legal_state(
            &state,
            implementation,
            domain.specification_self_check(),
            unit_index == 0 && is_the_first_state && record_count == 1,
            tally_of_the_unit,
        );
    };
    if unit.plan.names_units {
        // 每条记录六态：不在，或在而点名单元取五态之一。按六进制数数过去。
        let mut codes = vec![0usize; record_count];
        let mut is_the_first_state = true;
        loop {
            for (record_index, code) in codes.iter().enumerate() {
                presence[record_index] = match code.checked_sub(1) {
                    None => RecordPresence::NotOnDisk,
                    Some(unit_state_index) => {
                        RecordPresence::OnDisk(UnitState::ALL[unit_state_index])
                    }
                };
            }
            judge(&presence, is_the_first_state, &mut tally);
            is_the_first_state = false;
            // 进位：迭代次数的上界是记录条数；每一位都回到 0 时数完了。
            let mut position = 0;
            while position < record_count && codes[position] == UnitState::ALL.len() {
                codes[position] = 0;
                position += 1;
            }
            if position == record_count {
                break;
            }
            codes[position] += 1;
        }
    } else {
        for records_on_the_first_device in every_subset_mask(record_count) {
            for (record_index, record_presence) in presence.iter_mut().enumerate() {
                *record_presence = if bit_is_set(records_on_the_first_device, record_index) {
                    RecordPresence::OnDisk(UnitState::BothCopiesGood)
                } else {
                    RecordPresence::NotOnDisk
                };
            }
            judge(&presence, records_on_the_first_device == 0, &mut tally);
        }
    }
    tally
}

// ── C 域：状态 = (记录子集, 根槽子集)，所选根 = 落了盘的根里最新的那条；状态对上判 J5a、J5b

/// 一个状态是哪几段的「停在这一段」的状态：前面各段全落、之后各段全没落、这一段任意。返回按段号置位的掩码。
fn stop_segment_mask(state_mask: u32, segments: &[u32]) -> u32 {
    let mut stop_mask = 0u32;
    let mut earlier_segments = 0u32;
    for (segment_index, segment) in segments.iter().enumerate() {
        let every_earlier_write_is_on_disk = state_mask & earlier_segments == earlier_segments;
        let no_later_write_is_on_disk = state_mask & !(earlier_segments | segment) == 0;
        if every_earlier_write_is_on_disk && no_later_write_is_on_disk {
            stop_mask |= 1 << segment_index;
        }
        earlier_segments |= segment;
    }
    stop_mask
}

/// C 域一段历史：先逐状态判（今天的实现与理想恢复各得一个施加区间），再在全部 s ⊆ u 的状态对上数五个集合的 J5a、J5b。
fn tally_of_a_monotone_history(
    unit: &LegalWorkUnit,
    unit_index: usize,
    implementation: Implementation,
) -> Tally {
    let history = generate_history(&unit.plan);
    let later_record_count = history.slots.len() - 1;
    let root_write_count = history.chosen_instance_publish_count - 1;
    let state_bit_count = later_record_count + root_write_count;
    let mut tally = Tally::new();
    let mut today_intervals: Vec<Option<AppliedInterval>> = Vec::new();
    let mut ideal_intervals: Vec<AppliedInterval> = Vec::new();
    let mut presence = vec![RecordPresence::OnDisk(UnitState::BothCopiesGood); history.slots.len()];
    // 状态掩码：低 Nc 位是 P_0 之后各条记录在不在盘上，高 m 位是各次发布的根槽写落没落。
    for state_mask in every_subset_mask(state_bit_count) {
        for (later_record, record_presence) in presence.iter_mut().skip(1).enumerate() {
            *record_presence = if bit_is_set(state_mask, later_record) {
                RecordPresence::OnDisk(UnitState::BothCopiesGood)
            } else {
                RecordPresence::NotOnDisk
            };
        }
        let chosen_version = (1..=root_write_count)
            .rev()
            .find(|version| bit_is_set(state_mask, later_record_count + version - 1))
            .unwrap_or(0);
        let state = LegalState {
            history: &history,
            presence: &presence,
            mirroring: JournalMirroring::FirstDeviceOnly,
            residue: None,
            chosen_version,
            verify_named_units: true,
        };
        let today_interval = judge_a_legal_state(
            &state,
            implementation,
            SpecificationSelfCheck::NotJudged,
            unit_index == 0 && state_mask == 0 && later_record_count == 1,
            &mut tally,
        );
        if today_interval.is_none() {
            tally.add(Counter::PairStatesWithoutAnInterval);
        }
        today_intervals.push(today_interval);
        ideal_intervals.push(AppliedInterval {
            low: chosen_version,
            high: farthest_version_of(&state, Some(history.ring.local_in_flight_limit())),
        });
    }

    // 两种切法的段（登记 5.3）。甲：根槽那次写自成一段；乙：根槽那次写与它那次发布的记录同段。
    let mut segments_when_the_root_write_is_its_own_segment: Vec<u32> = Vec::new();
    let mut segments_when_the_root_write_shares_the_record_segment: Vec<u32> = Vec::new();
    let mut root_write_bits = 0u32;
    for version in 1..=root_write_count {
        let record_bits = history
            .record_indexes_of_publish(version)
            .fold(0u32, |bits, record_index| bits | (1 << (record_index - 1)));
        let root_bit = 1u32 << (later_record_count + version - 1);
        root_write_bits |= root_bit;
        segments_when_the_root_write_is_its_own_segment.push(record_bits);
        segments_when_the_root_write_is_its_own_segment.push(root_bit);
        segments_when_the_root_write_shares_the_record_segment.push(record_bits | root_bit);
    }
    // 甲切法里偶数号段是记录段、奇数号段是根槽段。
    let record_segment_bits_of_cut_own =
        (0..root_write_count).fold(0u32, |bits, version| bits | (1 << (2 * version)));
    let root_segment_bits_of_cut_own = record_segment_bits_of_cut_own << 1;
    let stop_masks_of_cut_own: Vec<u32> = every_subset_mask(state_bit_count)
        .map(|state_mask| {
            stop_segment_mask(state_mask, &segments_when_the_root_write_is_its_own_segment)
        })
        .collect();
    let stop_masks_of_cut_with: Vec<u32> = every_subset_mask(state_bit_count)
        .map(|state_mask| {
            stop_segment_mask(
                state_mask,
                &segments_when_the_root_write_shares_the_record_segment,
            )
        })
        .collect();
    for state_mask in every_subset_mask(state_bit_count) {
        let state_index = usize::try_from(state_mask).expect("状态掩码不超过 12 位");
        if stop_masks_of_cut_own[state_index] != 0 {
            tally.add(Counter::LegalCrashStatesWhenTheRootWriteIsItsOwnSegment);
        }
        if stop_masks_of_cut_with[state_index] != 0 {
            tally.add(Counter::LegalCrashStatesWhenTheRootWriteSharesTheRecordSegment);
        }
    }

    // 全部 s ⊆ u 的有序对（含 s = u）：对每个 u 枚举它的子掩码。迭代次数合计 3^(Nc + m)。
    for outer_mask in every_subset_mask(state_bit_count) {
        let outer_index = usize::try_from(outer_mask).expect("状态掩码不超过 12 位");
        let mut inner_mask = outer_mask;
        loop {
            let inner_index = usize::try_from(inner_mask).expect("状态掩码不超过 12 位");
            let shared_segments_of_cut_own =
                stop_masks_of_cut_own[inner_index] & stop_masks_of_cut_own[outer_index];
            let shared_segments_of_cut_with =
                stop_masks_of_cut_with[inner_index] & stop_masks_of_cut_with[outer_index];
            let root_write_differs = (inner_mask ^ outer_mask) & root_write_bits != 0;
            let memberships = [
                (PairSet::Wide, true),
                (
                    PairSet::CutOwnLegal,
                    stop_masks_of_cut_own[inner_index] != 0
                        && stop_masks_of_cut_own[outer_index] != 0,
                ),
                (PairSet::CutOwnSameSegment, shared_segments_of_cut_own != 0),
                (
                    PairSet::CutOwnSameRecordSegment,
                    shared_segments_of_cut_own & record_segment_bits_of_cut_own != 0,
                ),
                (
                    PairSet::CutOwnSameRootSegment,
                    shared_segments_of_cut_own & root_segment_bits_of_cut_own != 0,
                ),
                (
                    PairSet::CutWithLegal,
                    stop_masks_of_cut_with[inner_index] != 0
                        && stop_masks_of_cut_with[outer_index] != 0,
                ),
                (
                    PairSet::CutWithSameSegment,
                    shared_segments_of_cut_with != 0,
                ),
                (
                    PairSet::CutWithSameSegmentRootWriteDiffers,
                    shared_segments_of_cut_with != 0 && root_write_differs,
                ),
                (
                    PairSet::CutWithSameSegmentRootWriteAgrees,
                    shared_segments_of_cut_with != 0 && !root_write_differs,
                ),
            ];
            let today_pair = today_intervals[inner_index].zip(today_intervals[outer_index]);
            let ideal_inner = ideal_intervals[inner_index];
            let ideal_outer = ideal_intervals[outer_index];
            for (set, is_member) in memberships {
                if !is_member {
                    continue;
                }
                tally.add_pair(set, PairColumn::Pairs);
                if let Some((today_inner, today_outer)) = today_pair {
                    if !applied_set_is_contained(today_inner, today_outer) {
                        tally.add_pair(set, PairColumn::TodayAppliedSetViolations);
                    }
                    if landed_version_is_later(today_inner, today_outer) {
                        tally.add_pair(set, PairColumn::TodayLandedVersionViolations);
                    }
                }
                if !applied_set_is_contained(ideal_inner, ideal_outer) {
                    tally.add_pair(set, PairColumn::IdealAppliedSetViolations);
                }
                if landed_version_is_later(ideal_inner, ideal_outer) {
                    tally.add_pair(set, PairColumn::IdealLandedVersionViolations);
                }
            }
            if inner_mask == 0 {
                break;
            }
            inner_mask = (inner_mask - 1) & outer_mask;
        }
    }
    tally
}

// ── 多盘域（登记 r2 5.3）：每条记录在 0、1 号盘同一落点各一份，各自落或没落；没有点名单元

/// M、M6 一段历史：每条记录四态（0 号盘那一份落没落 × 1 号盘那一份落没落），所选根 R_0；每个状态另按两盘落盘集合的关系记进落后类别。
fn tally_of_a_mirrored_history(
    unit: &LegalWorkUnit,
    unit_index: usize,
    implementation: Implementation,
) -> Tally {
    let history = generate_history(&unit.plan);
    let record_count = history.slots.len();
    let mut tally = Tally::new();
    let mut presence = vec![RecordPresence::NotOnDisk; record_count];
    let mut second_device_copy_is_on_disk = vec![false; record_count];
    // 状态掩码：低 N 位是各条记录 0 号盘那一份，高 N 位是 1 号盘那一份。
    for state_mask in every_subset_mask(2 * record_count) {
        let first_device_mask = state_mask & ((1u32 << record_count) - 1);
        let second_device_mask = state_mask >> record_count;
        for (record_index, (record_presence, second_device_copy)) in presence
            .iter_mut()
            .zip(second_device_copy_is_on_disk.iter_mut())
            .enumerate()
        {
            *record_presence = if bit_is_set(first_device_mask, record_index) {
                RecordPresence::OnDisk(UnitState::BothCopiesGood)
            } else {
                RecordPresence::NotOnDisk
            };
            *second_device_copy = bit_is_set(second_device_mask, record_index);
        }
        let state = LegalState {
            history: &history,
            presence: &presence,
            mirroring: JournalMirroring::BothDevices {
                second_device_copy_is_on_disk: &second_device_copy_is_on_disk,
            },
            residue: None,
            chosen_version: 0,
            verify_named_units: true,
        };
        let counts_before = tally.counts.clone();
        judge_a_legal_state(
            &state,
            implementation,
            SpecificationSelfCheck::NotJudged,
            unit_index == 0 && state_mask == 0 && record_count == 1,
            &mut tally,
        );
        // 探针跳过的状态不判、不进落后类别。
        if tally.get(Counter::States) > counts_before[Counter::States.index()] {
            tally.add_to_the_lagging_class(
                lagging_class_of(first_device_mask, second_device_mask),
                &counts_before,
            );
        }
    }
    tally
}

/// MC 一段历史（登记 r2 5.3）：P_0 一条两盘都在；其后 Nc 条记录各四态；P_1… 每次发布一个根槽写落或没落；所选根 = 落了的根里
/// 最新那条（R_0 恒在）。先逐状态判，再在「宽」集合上数 J5a、J5b（甲合法、乙合法与同段那几个集合这一次不建）。
fn tally_of_a_mirrored_monotone_history(
    unit: &LegalWorkUnit,
    unit_index: usize,
    implementation: Implementation,
) -> Tally {
    let history = generate_history(&unit.plan);
    let later_record_count = history.slots.len() - 1;
    let root_write_count = history.chosen_instance_publish_count - 1;
    let state_bit_count = 2 * later_record_count + root_write_count;
    let mut tally = Tally::new();
    let mut today_intervals: Vec<Option<AppliedInterval>> = Vec::new();
    let mut ideal_intervals: Vec<AppliedInterval> = Vec::new();
    let mut presence = vec![RecordPresence::OnDisk(UnitState::BothCopiesGood); history.slots.len()];
    let mut second_device_copy_is_on_disk = vec![true; history.slots.len()];
    // 状态掩码：低 Nc 位是 P_0 之后各条记录 0 号盘那一份，其上 Nc 位是 1 号盘那一份，最高 m 位是各次发布的根槽写落没落。
    for state_mask in every_subset_mask(state_bit_count) {
        for (later_record, (record_presence, second_device_copy)) in presence
            .iter_mut()
            .zip(second_device_copy_is_on_disk.iter_mut())
            .skip(1)
            .enumerate()
        {
            *record_presence = if bit_is_set(state_mask, later_record) {
                RecordPresence::OnDisk(UnitState::BothCopiesGood)
            } else {
                RecordPresence::NotOnDisk
            };
            *second_device_copy = bit_is_set(state_mask, later_record_count + later_record);
        }
        let chosen_version = (1..=root_write_count)
            .rev()
            .find(|version| bit_is_set(state_mask, 2 * later_record_count + version - 1))
            .unwrap_or(0);
        let state = LegalState {
            history: &history,
            presence: &presence,
            mirroring: JournalMirroring::BothDevices {
                second_device_copy_is_on_disk: &second_device_copy_is_on_disk,
            },
            residue: None,
            chosen_version,
            verify_named_units: true,
        };
        let today_interval = judge_a_legal_state(
            &state,
            implementation,
            SpecificationSelfCheck::NotJudged,
            unit_index == 0 && state_mask == 0 && later_record_count == 1,
            &mut tally,
        );
        if today_interval.is_none() {
            tally.add(Counter::PairStatesWithoutAnInterval);
        }
        today_intervals.push(today_interval);
        ideal_intervals.push(AppliedInterval {
            low: chosen_version,
            high: farthest_version_of(&state, Some(history.ring.local_in_flight_limit())),
        });
    }
    add_the_wide_pairs(
        &mut tally,
        state_bit_count,
        &today_intervals,
        &ideal_intervals,
    );
    tally
}

/// 「宽」集合（登记 r2 5.3：两个状态的落盘集合与根槽集合都是包含关系，含相等）上数今天的实现与理想恢复各自的 J5a、J5b：
/// 对每个 u 枚举它的子掩码 s，迭代次数合计 3^(状态位数)。
fn add_the_wide_pairs(
    tally: &mut Tally,
    state_bit_count: usize,
    today_intervals: &[Option<AppliedInterval>],
    ideal_intervals: &[AppliedInterval],
) {
    for outer_mask in every_subset_mask(state_bit_count) {
        let outer_index = usize::try_from(outer_mask).expect("状态掩码不超过 12 位");
        let mut inner_mask = outer_mask;
        loop {
            let inner_index = usize::try_from(inner_mask).expect("状态掩码不超过 12 位");
            if inner_mask & outer_mask == inner_mask {
                tally.add_pair(PairSet::Wide, PairColumn::Pairs);
                if let Some((today_inner, today_outer)) =
                    today_intervals[inner_index].zip(today_intervals[outer_index])
                {
                    if !applied_set_is_contained(today_inner, today_outer) {
                        tally.add_pair(PairSet::Wide, PairColumn::TodayAppliedSetViolations);
                    }
                    if landed_version_is_later(today_inner, today_outer) {
                        tally.add_pair(PairSet::Wide, PairColumn::TodayLandedVersionViolations);
                    }
                }
                let (ideal_inner, ideal_outer) =
                    (ideal_intervals[inner_index], ideal_intervals[outer_index]);
                if !applied_set_is_contained(ideal_inner, ideal_outer) {
                    tally.add_pair(PairSet::Wide, PairColumn::IdealAppliedSetViolations);
                }
                if landed_version_is_later(ideal_inner, ideal_outer) {
                    tally.add_pair(PairSet::Wide, PairColumn::IdealLandedVersionViolations);
                }
            }
            if inner_mask == 0 {
                break;
            }
            inner_mask = (inner_mask - 1) & outer_mask;
        }
    }
}

// ── 对抗一族（登记 5.2 的 A、A 小环、A 链）

/// 对抗域里一个槽上那条记录的自由字段（登记 5.2：txg ∈ {t, t + 1, t + 2} × 序号 ∈ {0, 1, 2} × 末条标志 × 提交标记 × 事务号 ∈ {1, 2}）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct AdversarialRecordFields {
    txg_above_the_chosen_root: u64,
    ordinal_within_publish: u32,
    carries_the_last_record_flag: bool,
    is_commit: bool,
    transaction: u64,
}

/// 72 种自由字段的取法，次序写死（枚举次序进见证的先后）。
fn adversarial_record_alphabet() -> Vec<AdversarialRecordFields> {
    let mut alphabet = Vec::new();
    for txg_above_the_chosen_root in 0..3u64 {
        for ordinal_within_publish in 0..3u32 {
            for carries_the_last_record_flag in [false, true] {
                for is_commit in [false, true] {
                    for transaction in 1..=2u64 {
                        alphabet.push(AdversarialRecordFields {
                            txg_above_the_chosen_root,
                            ordinal_within_publish,
                            carries_the_last_record_flag,
                            is_commit,
                            transaction,
                        });
                    }
                }
            }
        }
    }
    alphabet
}

/// 一个槽的取法：不在，或字母表里第几种记录；A 链另取反向链对 / 错。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum AdversarialSlotChoice {
    HoldsNothing,
    Record {
        alphabet_index: usize,
        back_chain_is_wrong: bool,
    },
}

/// 一个槽的全部取法：A、A 小环 73 种，A 链 145 种（每条在的记录的反向链另取对 / 错）。
fn adversarial_slot_choices(domain: Domain, alphabet_length: usize) -> Vec<AdversarialSlotChoice> {
    let mut choices = vec![AdversarialSlotChoice::HoldsNothing];
    for alphabet_index in 0..alphabet_length {
        choices.push(AdversarialSlotChoice::Record {
            alphabet_index,
            back_chain_is_wrong: false,
        });
        if domain == Domain::AdversarialBackChain {
            choices.push(AdversarialSlotChoice::Record {
                alphabet_index,
                back_chain_is_wrong: true,
            });
        }
    }
    choices
}

/// 对抗一族的所选根：时间线 R_0 那一份（登记 r2 第一节：投影之外的四个字段照档取第 0 版的）。
fn adversarial_chosen_root(run_profile: RunProfile) -> RootRecord {
    VersionFields {
        instance: CHOSEN_INSTANCE,
        checkpoint_txg: ORDINARY_FIRST_TXG,
        segment: NewRootSegment::of_timeline_publish(0, ORDINARY_FIRST_TXG),
    }
    .to_root_with(run_profile.fields_beyond_the_projection_of_version(0))
}

/// 对抗一族锚点之后第 `position` 个槽上字母表第 `alphabet_index` 种记录的新根段投影与后三项：区分号 d = 槽位 × 1000 + 字母表下标，
/// 新根段取时间线之外的值，后三项取 h(d)（登记 r5 5.3 第 2 条；r1–r4 档是 r1 常量）。生成器与生成器真相都从这里取。
fn adversarial_record_projection_and_fields(
    chosen_root: &RootRecord,
    alphabet_fields: AdversarialRecordFields,
    position: usize,
    alphabet_index: usize,
    run_profile: RunProfile,
) -> (VersionFields, RecordFieldsBeyondTheProjection) {
    let checkpoint_txg = chosen_root.checkpoint_txg.0 + alphabet_fields.txg_above_the_chosen_root;
    let distinguishing_number =
        u64::try_from(position * 1000 + alphabet_index).expect("槽位与字母表下标都很小");
    (
        VersionFields {
            instance: CHOSEN_INSTANCE,
            checkpoint_txg,
            segment: NewRootSegment::off_timeline(distinguishing_number, checkpoint_txg),
        },
        run_profile.record_fields_off_the_timeline(distinguishing_number),
    )
}

/// 对抗一族一个域在 `slot_count` 个槽上能造出的每条记录（锚点与每个槽位上字母表的每一种）的投影对到后三项（登记 r5 5.3 第 3 条）。
fn adversarial_record_truth(
    slot_count: usize,
    alphabet: &[AdversarialRecordFields],
    chosen_root: &RootRecord,
    run_profile: RunProfile,
) -> GeneratedRecordTruth {
    let mut truth = GeneratedRecordTruth::new();
    truth.note(
        VersionFields::of_root(chosen_root),
        run_profile.record_fields_of_the_timeline_publish(0),
    );
    for position in 1..=slot_count {
        for (alphabet_index, alphabet_fields) in alphabet.iter().enumerate() {
            let (projection, record_fields) = adversarial_record_projection_and_fields(
                chosen_root,
                *alphabet_fields,
                position,
                alphabet_index,
                run_profile,
            );
            truth.note(projection, record_fields);
        }
    }
    truth
}

/// 对抗域枚举到一半的状态：锚点槽与其后各槽上放着什么、由它们建出来的表。
struct AdversarialEnumeration<'run> {
    domain: Domain,
    slot_count: usize,
    alphabet: &'run [AdversarialRecordFields],
    choices: &'run [AdversarialSlotChoice],
    chosen_root: RootRecord,
    implementation: Implementation,
    run_profile: RunProfile,
    /// 这个域这一档字母表能造出的每条记录（与锚点）的投影对到后三项（登记 r5 5.3 第 3 条），J8a 拿它定期望。
    record_truth: &'run GeneratedRecordTruth,
    /// 下标 0 是锚点槽，之后是第 1..n 个槽；`None` 是不在。
    slots: Vec<Option<GeneratedSlot>>,
    choice_trail: Vec<AdversarialSlotChoice>,
    is_the_designated_state: bool,
}

impl AdversarialEnumeration<'_> {
    fn anchor_counter(&self) -> u64 {
        self.domain.ring().first_history_counter()
    }

    /// 锚点：计数器 a，txg = 所选根的 txg、序号 1、带末条标志与提交标记（登记 5.2）。
    fn generate_the_anchor_slot(&self) -> GeneratedSlot {
        let chosen_fields = VersionFields::of_root(&self.chosen_root);
        // 锚点是所选根 R_0 那次发布的记录：r5 档带 R_0 的后三项（登记 r5 5.3 第 1 条），别的档 r1 常量。
        let anchor_record_fields = self.run_profile.record_fields_of_the_timeline_publish(0);
        let record = JournalRecord {
            instance: InstanceGeneration(CHOSEN_INSTANCE),
            counter: self.anchor_counter(),
            checkpoint_txg: CheckpointTxg(chosen_fields.checkpoint_txg),
            transaction: 1,
            is_commit: true,
            ordinal_within_publish: JournalRecordOrdinalWithinPublish::FIRST,
            place_in_publish: JournalRecordPlaceInPublish::LastRecordOfThePublish,
            back_chain: BACK_CHAIN_WHEN_THE_PREVIOUS_SLOT_HOLDS_NOTHING,
            filesystem_identifier: FILESYSTEM_IDENTIFIER_LOW_EIGHT_BYTES,
            new_tree_table: chosen_fields.segment.tree_table,
            new_mapping_root: chosen_fields.segment.mapping_root,
            new_tree_identifier_watermark: chosen_fields.segment.tree_identifier_watermark,
            new_rollback_floor: CheckpointTxg(chosen_fields.segment.rollback_floor),
            new_instance_table: anchor_record_fields.instance_table,
            new_allocation_record_tree_root: anchor_record_fields.allocation_record_tree_root,
            new_unmount_marker: anchor_record_fields.unmount_marker,
            named: Vec::new(),
        };
        generate_slot(
            0,
            self.domain.ring(),
            &record,
            ParseExpectation::MustRoundTrip,
        )
    }

    /// 锚点之后第 `position` 个槽（从 1 数）上的一条记录：字段照字母表，新根段每条各不相同，
    /// 反向链照盘上前一槽的真值（前一槽不在时取固定值），A 链取错时把真值异或 1。
    fn generate_a_record_slot(
        &self,
        position: usize,
        alphabet_index: usize,
        back_chain_is_wrong: bool,
    ) -> GeneratedSlot {
        let fields = self.alphabet[alphabet_index];
        let true_back_chain = match self.slots[position - 1].as_ref() {
            Some(slot_before) => slot_before.chain_value_of_header,
            None => BACK_CHAIN_WHEN_THE_PREVIOUS_SLOT_HOLDS_NOTHING,
        };
        let (projection, record_fields) = adversarial_record_projection_and_fields(
            &self.chosen_root,
            fields,
            position,
            alphabet_index,
            self.run_profile,
        );
        let checkpoint_txg = projection.checkpoint_txg;
        let segment = projection.segment;
        let record = JournalRecord {
            instance: InstanceGeneration(CHOSEN_INSTANCE),
            counter: self.anchor_counter() + u64::try_from(position).expect("槽位不超过 4"),
            checkpoint_txg: CheckpointTxg(checkpoint_txg),
            transaction: fields.transaction,
            is_commit: fields.is_commit,
            ordinal_within_publish: JournalRecordOrdinalWithinPublish(
                fields.ordinal_within_publish,
            ),
            place_in_publish: if fields.carries_the_last_record_flag {
                JournalRecordPlaceInPublish::LastRecordOfThePublish
            } else {
                JournalRecordPlaceInPublish::MoreRecordsOfThePublishFollow
            },
            back_chain: if back_chain_is_wrong {
                true_back_chain ^ 1
            } else {
                true_back_chain
            },
            filesystem_identifier: FILESYSTEM_IDENTIFIER_LOW_EIGHT_BYTES,
            new_tree_table: segment.tree_table,
            new_mapping_root: segment.mapping_root,
            new_tree_identifier_watermark: segment.tree_identifier_watermark,
            new_rollback_floor: CheckpointTxg(segment.rollback_floor),
            new_instance_table: record_fields.instance_table,
            new_allocation_record_tree_root: record_fields.allocation_record_tree_root,
            new_unmount_marker: record_fields.unmount_marker,
            named: Vec::new(),
        };
        generate_slot(
            0,
            self.domain.ring(),
            &record,
            ParseExpectation::OrdinalZeroMustBeRejected,
        )
    }

    fn describe_the_state(&self) -> String {
        let anchor = if self.slots[0].is_some() {
            "anchor=present"
        } else {
            "anchor=absent"
        };
        let slots: Vec<String> = self
            .choice_trail
            .iter()
            .map(|choice| match choice {
                AdversarialSlotChoice::HoldsNothing => "-".to_string(),
                AdversarialSlotChoice::Record {
                    alphabet_index,
                    back_chain_is_wrong,
                } => {
                    let fields = self.alphabet[*alphabet_index];
                    format!(
                        "t+{}:o{}:L{}:C{}:x{}:chain_{}",
                        fields.txg_above_the_chosen_root,
                        fields.ordinal_within_publish,
                        u8::from(fields.carries_the_last_record_flag),
                        u8::from(fields.is_commit),
                        fields.transaction,
                        if *back_chain_is_wrong {
                            "wrong"
                        } else {
                            "true"
                        }
                    )
                }
            })
            .collect();
        format!("{anchor};slots={}", slots.join(","))
    }

    /// 从第 `position` 个槽起把剩下的槽取遍；取满 n 个槽就判这一个状态。
    fn descend(&mut self, position: usize, tally: &mut Tally) {
        if position > self.slot_count {
            self.judge_the_state(tally);
            self.is_the_designated_state = false;
            return;
        }
        for choice_index in 0..self.choices.len() {
            self.put_the_choice_and_descend(position, self.choices[choice_index], tally);
        }
    }

    fn put_the_choice_and_descend(
        &mut self,
        position: usize,
        choice: AdversarialSlotChoice,
        tally: &mut Tally,
    ) {
        let slot = match choice {
            AdversarialSlotChoice::HoldsNothing => None,
            AdversarialSlotChoice::Record {
                alphabet_index,
                back_chain_is_wrong,
            } => Some(self.generate_a_record_slot(position, alphabet_index, back_chain_is_wrong)),
        };
        self.slots.push(slot);
        self.choice_trail.push(choice);
        self.descend(position + 1, tally);
        self.choice_trail.pop();
        self.slots.pop();
    }

    /// 一个对抗状态的写法（逐态行的 `state=` 与每条违例的见证）：锚点与各槽、第一次调用的结局、S 三种读法的期望。
    fn describe_the_state_with_its_first_outcome(
        &self,
        first: &ReplayOutcome,
        expectations: &[SpecificationExpectation; 3],
    ) -> String {
        format!(
            "{};first={};expected={}",
            self.describe_the_state(),
            describe_outcome(first),
            expectations
                .iter()
                .map(|expectation| match expectation {
                    SpecificationExpectation::FailsWithoutApplying => "fails".to_string(),
                    SpecificationExpectation::ReturnsRoot(fields) => format!(
                        "txg{}:watermark{}",
                        fields.checkpoint_txg, fields.segment.tree_identifier_watermark
                    ),
                })
                .collect::<Vec<String>>()
                .join("/")
        )
    }

    /// 判对抗域的一个状态：A0、AS 三条、A-互 三个数、A-结构、A-限、J6。记录表怎么建由档的 `table_source` 定（登记 r2 5.4）：
    /// `Both` 时两张表各判一遍、逐态比，汇总的计数取直接建表那一遍。对抗域的槽都在 0 号盘，盘次序不改直接建的表，不另判「照盘报的次序」那一栏。
    fn judge_the_state(&self, tally: &mut Tally) {
        let probe_decision = begin_probing_a_state(tally);
        if matches!(probe_decision, ProbeDecision::SkipThisState) {
            return;
        }
        let ring = self.domain.ring();
        let mut direct_table = RecordTable::new();
        let mut truths: Vec<RecordTruthOnDisk> = Vec::new();
        let mut contents: Vec<(u32, u64, StoredBytes<'_>)> = Vec::new();
        let mut holds_a_record_rejected_by_parse = false;
        for slot in self.slots.iter().flatten() {
            contents.push((
                slot.device,
                slot.offset_in_bytes,
                StoredBytes::Readable(&slot.bytes),
            ));
            match slot.parsed.as_ref() {
                Some(parsed) => {
                    direct_table.insert((parsed.instance, parsed.counter), parsed.clone());
                    truths.push(RecordTruthOnDisk {
                        instance: parsed.instance.0,
                        counter: parsed.counter,
                        chain_value_of_header: slot.chain_value_of_header,
                        named_units_all_verified: true,
                    });
                }
                None => holds_a_record_rejected_by_parse = true,
            }
        }
        let disk = MemoryDisk {
            ring,
            reported_disk_order: self.run_profile.reported_disk_order,
            contents,
        };
        let judge_on = |table: &RecordTable,
                        implementation_of_the_pass: Implementation,
                        designated_in_the_pass: bool,
                        tally_of_the_pass: &mut Tally| {
            self.judge_the_state_on_a_table(
                table,
                &disk,
                &truths,
                holds_a_record_rejected_by_parse,
                implementation_of_the_pass,
                designated_in_the_pass,
                tally_of_the_pass,
            )
        };
        let judgement = match self.run_profile.table_source {
            RecordTableSource::Direct => judge_on(
                &direct_table,
                self.implementation,
                self.is_the_designated_state,
                tally,
            ),
            RecordTableSource::Scanned => {
                let scanned_table = scanned_record_table_of(&disk, self.implementation);
                judge_on(
                    &scanned_table,
                    self.implementation,
                    self.is_the_designated_state,
                    tally,
                )
            }
            RecordTableSource::Both => {
                let scanned_table = scanned_record_table_of(&disk, self.implementation);
                let counts_before_the_state = tally.counts.clone();
                let (scan_pass_implementation, scan_pass_is_designated) =
                    implementation_and_designation_of_the_scan_pass(
                        self.implementation,
                        self.is_the_designated_state,
                    );
                let mut scanned_tally = Tally::new();
                let scanned_judgement = judge_on(
                    &scanned_table,
                    scan_pass_implementation,
                    scan_pass_is_designated,
                    &mut scanned_tally,
                );
                let scanned_summary = JudgedRowSummary {
                    first_outcome: scanned_judgement.first_outcome,
                    counter_labels: counter_labels_a_state_added_to(
                        &scanned_tally,
                        &Tally::new().counts,
                    ),
                };
                let direct_judgement = judge_on(
                    &direct_table,
                    self.implementation,
                    self.is_the_designated_state,
                    tally,
                );
                let direct_summary = JudgedRowSummary {
                    first_outcome: direct_judgement.first_outcome,
                    counter_labels: counter_labels_a_state_added_to(
                        tally,
                        &counts_before_the_state,
                    ),
                };
                add_the_comparison_of_two_judged_tables(
                    tally,
                    (Counter::TablesDiffer, Counter::RowsDiffer),
                    (&direct_table, &direct_summary),
                    (&scanned_table, &scanned_summary),
                    &|| {
                        self.describe_the_state_with_its_first_outcome(
                            &direct_judgement.first_outcome,
                            &direct_judgement.expectations,
                        )
                    },
                );
                if !direct_table.is_empty() {
                    tally.add(Counter::DirectTableHoldsARecord);
                }
                direct_judgement
            }
        };
        finish_probing_a_state(probe_decision, tally, &|| {
            self.describe_the_state_with_its_first_outcome(
                &judgement.first_outcome,
                &judgement.expectations,
            )
        });
    }

    /// [`Self::judge_the_state`] 在一张给定的记录表上判一个状态：不管探针、不管两种建表法怎么比，只数这一遍判出来的量。
    #[allow(
        clippy::too_many_arguments,
        reason = "每个参数都是这一判要用的一样东西，凑不成一个带不变量的概念"
    )]
    fn judge_the_state_on_a_table(
        &self,
        records: &RecordTable,
        disk: &MemoryDisk<'_>,
        truths: &[RecordTruthOnDisk],
        holds_a_record_rejected_by_parse: bool,
        implementation: Implementation,
        is_the_designated_state: bool,
        tally: &mut Tally,
    ) -> AdversarialJudgement {
        let in_flight_limit = self.domain.ring().local_in_flight_limit();
        let view = SpecificationView {
            chosen_root: &self.chosen_root,
            records,
            in_flight_limit,
            verify_named_units: true,
            truths,
        };
        let expectations =
            FollowReading::REGISTERED.map(|reading| specification_expectation(&view, reading));
        let today = || call_replay_journal(disk, &self.chosen_root, records, true);
        let first = match implementation {
            Implementation::TodayReplayJournal => today(),
            Implementation::Control(control) => match control {
                ControlKind::PanicsOnTheDesignatedState => {
                    if is_the_designated_state {
                        ReplayOutcome::Panicked
                    } else {
                        today()
                    }
                }
                ControlKind::SpecificationWithReading(reading) => outcome_of_the_expectation(
                    &self.chosen_root,
                    specification_expectation(&view, reading),
                ),
                ControlKind::RootFromARecordWithoutTheLastRecordFlag => {
                    let record_without_the_flag = records.values().find(|record| {
                        record.instance == self.chosen_root.instance
                            && record.checkpoint_txg > self.chosen_root.checkpoint_txg
                            && record.is_commit
                            && !record_carries_the_last_record_flag(record)
                    });
                    match record_without_the_flag {
                        Some(record) => ReplayOutcome::Returned(ReturnedReplay {
                            root: VersionFields::of_record(record)
                                .to_root_rebuilt_over(&self.chosen_root),
                            prefix_applied: 1,
                            verification_passed: 1,
                        }),
                        None => today(),
                    }
                }
                ControlKind::FirstCallReturnsTheChosenRootAndSecondCallReportsMoreThanOneLastRecordFlag => {
                    ReplayOutcome::Returned(ReturnedReplay {
                        root: self.chosen_root,
                        prefix_applied: 0,
                        verification_passed: 0,
                    })
                }
                ControlKind::ReportsOneRecordOverTheLimit => match today() {
                    ReplayOutcome::Returned(mut returned) => {
                        returned.prefix_applied = in_flight_limit + 1;
                        ReplayOutcome::Returned(returned)
                    }
                    ReplayOutcome::Panicked | ReplayOutcome::Failed(_) => {
                        ReplayOutcome::Returned(ReturnedReplay {
                            root: self.chosen_root,
                            prefix_applied: in_flight_limit + 1,
                            verification_passed: 0,
                        })
                    }
                },
                ControlKind::OffTimelineRootWheneverApplied
                | ControlKind::LandsOnePublishTooFarOverAnIncompletePublish
                | ControlKind::IgnoresTheInFlightLimit
                | ControlKind::AlwaysReturnsTheChosenRoot
                | ControlKind::ReturnsTheChosenRootWhenEveryRecordIsOnDisk
                | ControlKind::IdealWithTheCumulativeAppliedSet
                | ControlKind::SecondCallMovesOn
                | ControlKind::ReportsNoFailureOnTheLargestTxg
                | ControlKind::ReportsOneVerificationWithoutVerifying
                | ControlKind::SecondCallReportsMoreThanOneLastRecordFlag
                | ControlKind::SecondCallReportsTheChosenRootTxgHasNoSuccessor
                | ControlKind::IdealFirstCallThenTodaysSecondCall
                | ControlKind::IdealWithAFieldOfTheLandedVersion(_)
                | ControlKind::IdealCopyingTheChosenUnmountMarkerWhenApplied
                | ControlKind::IdealWritingNotWrittenWhenNothingIsApplied
                | ControlKind::IdealCopyingTheChosenRecordFieldsWhenApplied
                | ControlKind::IdealWritingOneUnmountMarkerWhenApplied
                | ControlKind::IdealCountingOnlyTheFirstDeviceCopies
                | ControlKind::ScanTableLosesItsLargestKeyRecord
                | ControlKind::ScanPassOutcomeBecomesAPanic => today(),
            },
        };
        let describe_witness =
            || self.describe_the_state_with_its_first_outcome(&first, &expectations);

        tally.add(Counter::States);
        // I-8.12 成不成立（登记 r3 5.6）：对抗一族每个状态各数一次，不看实现的结局；只有 r3 档数。
        let table_satisfies_i812 = self
            .run_profile
            .splits_j6qn_by_i812()
            .then(|| record_table_satisfies_i812(records));
        match table_satisfies_i812 {
            Some(true) => tally.add(Counter::TablesSatisfyI812),
            Some(false) => tally.add(Counter::TablesViolateI812),
            None => {}
        }
        if holds_a_record_rejected_by_parse {
            tally.add(Counter::StatesHoldingARecordRejectedByParse);
        }
        if expectations[0] == SpecificationExpectation::FailsWithoutApplying {
            tally.add(Counter::SpecificationExpectsFailure);
        }
        let above_water_record_count = records
            .values()
            .filter(|record| {
                record.instance == self.chosen_root.instance
                    && record.checkpoint_txg > self.chosen_root.checkpoint_txg
            })
            .count();
        tally.above_water_maximum = tally.above_water_maximum.max(above_water_record_count);
        if above_water_record_count > in_flight_limit {
            tally.add(Counter::AboveWaterOverLimit);
        }
        if above_water_record_count == in_flight_limit {
            tally.add(Counter::AboveWaterAtLimit);
        }

        // AS-全、AS-连甲、AS-连乙：实现的结局与 S 在那种读法下的期望不同即一条违例。
        let specification_violation_counters = [
            Counter::AsWholeTableViolations,
            Counter::AsGapContinuesViolations,
            Counter::AsGapStopsViolations,
        ];
        for (expectation, violation_counter) in expectations
            .into_iter()
            .zip(specification_violation_counters)
        {
            if !outcome_matches_the_expectation(&first, expectation) {
                tally.add_violation(violation_counter, &describe_witness);
            }
        }
        // A-互：三种读法两两之间期望不同的态数。
        if expectations[0] != expectations[1] {
            tally.add_violation(Counter::MutualWholeTableAndGapContinues, &describe_witness);
        }
        if expectations[0] != expectations[2] {
            tally.add_violation(Counter::MutualWholeTableAndGapStops, &describe_witness);
        }
        if expectations[1] != expectations[2] {
            tally.add_violation(Counter::MutualGapContinuesAndGapStops, &describe_witness);
        }

        match &first {
            ReplayOutcome::Panicked => tally.add_violation(Counter::Panicked, &describe_witness),
            ReplayOutcome::Failed(_kind) => {
                tally.add(Counter::Failed);
                tally.add(Counter::J6FirstCallFailed);
            }
            ReplayOutcome::Returned(returned) => {
                let returned_fields = VersionFields::of_root(&returned.root);
                let chosen_fields = VersionFields::of_root(&self.chosen_root);
                if returned_fields != chosen_fields {
                    tally.add(Counter::AppliedAtLeastOne);
                }
                // A-结构（M2 字面，不靠 S）：返回的根要么六个字段等于所选根，要么它的新根段四个字段与 txg 等于表里某一条
                // 「同实例、txg 大于所选根、带末条标志、带提交标记」的记录的。
                let comes_from_a_committed_last_record = records.values().any(|record| {
                    record.instance == self.chosen_root.instance
                        && record.checkpoint_txg > self.chosen_root.checkpoint_txg
                        && record.is_commit
                        && record_carries_the_last_record_flag(record)
                        && VersionFields::of_record(record).checkpoint_txg
                            == returned_fields.checkpoint_txg
                        && VersionFields::of_record(record).segment == returned_fields.segment
                });
                if returned_fields != chosen_fields && !comes_from_a_committed_last_record {
                    tally.add_violation(Counter::StructureViolations, &describe_witness);
                }
                // A-限（I-8.3）：报告的 prefix_applied 不超过在飞上限。
                if returned.prefix_applied > in_flight_limit {
                    tally.add_violation(Counter::LimitViolations, &describe_witness);
                }
                judge_the_fixed_point(
                    implementation,
                    self.run_profile,
                    returned,
                    None,
                    above_water_record_count <= in_flight_limit,
                    table_satisfies_i812,
                    records,
                    disk,
                    true,
                    &describe_witness,
                    tally,
                );
                if self.run_profile.judges_j7() {
                    judge_the_fields_beyond_the_projection(
                        &self.chosen_root,
                        &returned.root,
                        &describe_witness,
                        tally,
                    );
                }
                if self.run_profile.judges_j8() {
                    judge_the_fields_beyond_the_projection_against_the_generated_records(
                        &self.chosen_root,
                        &returned.root,
                        self.record_truth,
                        None,
                        &describe_witness,
                        tally,
                    );
                }
            }
        }
        AdversarialJudgement {
            first_outcome: first,
            expectations,
        }
    }
}

/// 一个对抗状态判完的结果：第一次调用的结局与 S 三种读法的期望（逐态行的写法要）。
struct AdversarialJudgement {
    first_outcome: ReplayOutcome,
    expectations: [SpecificationExpectation; 3],
}

/// 对抗域的一个工作单元：锚点在不在、第一个槽取哪一种，其余的槽在单元里取遍。
#[derive(Clone, Copy, Debug)]
struct AdversarialWorkUnit {
    anchor_is_on_disk: bool,
    first_choice: AdversarialSlotChoice,
}

fn tally_of_an_adversarial_work_unit(
    domain: Domain,
    slot_count: usize,
    unit: &AdversarialWorkUnit,
    unit_index: usize,
    implementation: Implementation,
    run_profile: RunProfile,
) -> Tally {
    let alphabet = adversarial_record_alphabet();
    let choices = adversarial_slot_choices(domain, alphabet.len());
    let chosen_root = adversarial_chosen_root(run_profile);
    let record_truth = adversarial_record_truth(slot_count, &alphabet, &chosen_root, run_profile);
    let mut enumeration = AdversarialEnumeration {
        domain,
        slot_count,
        alphabet: &alphabet,
        choices: &choices,
        chosen_root,
        implementation,
        run_profile,
        record_truth: &record_truth,
        slots: Vec::new(),
        choice_trail: Vec::new(),
        is_the_designated_state: unit_index == 0 && slot_count == 1,
    };
    let anchor_slot = unit
        .anchor_is_on_disk
        .then(|| enumeration.generate_the_anchor_slot());
    enumeration.slots.push(anchor_slot);
    let mut tally = Tally::new();
    enumeration.put_the_choice_and_descend(1, unit.first_choice, &mut tally);
    tally
}

/// 一个域在一个 N（或 Nc、n）上数出来的全部量。
fn tally_of_a_domain_size(
    domain: Domain,
    size: usize,
    implementation: Implementation,
    run_profile: RunProfile,
    thread_count: usize,
) -> Tally {
    match domain.family() {
        Family::LegalHistory => {
            let units = legal_work_units(domain, size, run_profile);
            match domain {
                Domain::MonotoneBigRing | Domain::MonotoneSmallRingSix => {
                    tally_of_work_units(&units, thread_count, &|unit_index, unit| {
                        tally_of_a_monotone_history(unit, unit_index, implementation)
                    })
                }
                Domain::LegalBigRing
                | Domain::LegalSmallRingSix
                | Domain::LegalSmallRingNine
                | Domain::NamedUnitsVerified
                | Domain::NamedUnitsNotVerified
                | Domain::AdversarialBigRing
                | Domain::AdversarialSmallRingSix
                | Domain::AdversarialBackChain
                | Domain::CrossInstance
                | Domain::TxgOneBelowTheLargest
                | Domain::TxgAtTheLargest
                | Domain::Residue => {
                    tally_of_work_units(&units, thread_count, &|unit_index, unit| {
                        tally_of_a_legal_history(domain, unit, unit_index, implementation)
                    })
                }
                Domain::MirroredBigRing
                | Domain::MirroredSmallRingSix
                | Domain::MirroredMonotoneBigRing => {
                    unreachable!("多盘域走 Family::MultiDisk 那一支")
                }
            }
        }
        Family::MultiDisk => {
            let units = legal_work_units(domain, size, run_profile);
            if domain == Domain::MirroredMonotoneBigRing {
                tally_of_work_units(&units, thread_count, &|unit_index, unit| {
                    tally_of_a_mirrored_monotone_history(unit, unit_index, implementation)
                })
            } else {
                tally_of_work_units(&units, thread_count, &|unit_index, unit| {
                    tally_of_a_mirrored_history(unit, unit_index, implementation)
                })
            }
        }
        Family::Adversarial => {
            let choices = adversarial_slot_choices(domain, adversarial_record_alphabet().len());
            let mut units = Vec::new();
            for anchor_is_on_disk in [false, true] {
                for first_choice in &choices {
                    units.push(AdversarialWorkUnit {
                        anchor_is_on_disk,
                        first_choice: *first_choice,
                    });
                }
            }
            tally_of_work_units(&units, thread_count, &|unit_index, unit| {
                tally_of_an_adversarial_work_unit(
                    domain,
                    size,
                    unit,
                    unit_index,
                    implementation,
                    run_profile,
                )
            })
        }
    }
}

// ── 界（登记 5.2、5.6；`light` 是用户 2026-10-01 定的轻量档）

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum BoundsTier {
    SegmentOne,
    SegmentTwo,
    Light,
}

impl BoundsTier {
    const fn label(self) -> &'static str {
        match self {
            Self::SegmentOne => "segment1",
            Self::SegmentTwo => "segment2",
            Self::Light => "light",
        }
    }

    /// 这一档里这个域扫到的最大 N（或 Nc、n）；`None` 是这一档不跑这个域。
    const fn largest_size(self, domain: Domain) -> Option<usize> {
        match self {
            Self::SegmentOne => Some(match domain {
                Domain::LegalBigRing => 10,
                Domain::LegalSmallRingSix => 6,
                Domain::LegalSmallRingNine => 9,
                Domain::NamedUnitsVerified | Domain::NamedUnitsNotVerified => 6,
                Domain::MonotoneBigRing => 6,
                Domain::MonotoneSmallRingSix => 5,
                Domain::AdversarialBigRing => 4,
                Domain::AdversarialSmallRingSix | Domain::AdversarialBackChain => 3,
                Domain::CrossInstance => 7,
                Domain::TxgOneBelowTheLargest => 4,
                Domain::TxgAtTheLargest => 3,
                Domain::Residue => 5,
                // 登记 r2 5.3：M、M6 到 N = 6，MC 到 Nc = 4。
                Domain::MirroredBigRing | Domain::MirroredSmallRingSix => 6,
                Domain::MirroredMonotoneBigRing => 4,
            }),
            Self::SegmentTwo => match domain {
                Domain::LegalBigRing => Some(8),
                Domain::LegalSmallRingSix => Some(6),
                Domain::LegalSmallRingNine => None,
                Domain::NamedUnitsVerified | Domain::NamedUnitsNotVerified => Some(5),
                Domain::MonotoneBigRing | Domain::MonotoneSmallRingSix => Some(5),
                Domain::AdversarialBigRing
                | Domain::AdversarialSmallRingSix
                | Domain::AdversarialBackChain => Some(3),
                Domain::CrossInstance => Some(6),
                Domain::TxgOneBelowTheLargest => Some(4),
                Domain::TxgAtTheLargest => Some(3),
                Domain::Residue => Some(5),
                // 第二段的变体不跑多盘域（登记 r2 5.6：多盘是第三段的事）。
                Domain::MirroredBigRing
                | Domain::MirroredSmallRingSix
                | Domain::MirroredMonotoneBigRing => None,
            },
            Self::Light => match domain {
                Domain::LegalBigRing => Some(9),
                Domain::LegalSmallRingSix => Some(6),
                Domain::LegalSmallRingNine => Some(9),
                Domain::NamedUnitsVerified | Domain::NamedUnitsNotVerified => Some(5),
                Domain::MonotoneBigRing => Some(6),
                Domain::MonotoneSmallRingSix => Some(5),
                Domain::AdversarialBigRing
                | Domain::AdversarialSmallRingSix
                | Domain::AdversarialBackChain => Some(3),
                Domain::CrossInstance => Some(7),
                Domain::TxgOneBelowTheLargest => Some(4),
                Domain::TxgAtTheLargest => Some(3),
                Domain::Residue => Some(5),
                // `light` 是 r1 档的轻量档（用户 2026-10-01 定），不跑多盘域。
                Domain::MirroredBigRing
                | Domain::MirroredSmallRingSix
                | Domain::MirroredMonotoneBigRing => None,
            },
        }
    }
}

// ── 钉绝对值的断言（登记第七节；不符 ⇒ 作废条款 V1）

fn assert_the_pinned_absolute_values(domain: Domain, size: usize, tally: &Tally) {
    let size_index = size - domain.smallest_size();
    assert_eq!(
        tally.get(Counter::States),
        pinned_state_counts(domain)[size_index],
        "作废条款 V1：{} 域 N = {size} 的状态数与登记第七节不等",
        domain.label()
    );
    if domain == Domain::LegalBigRing {
        if let Some((farthest_at_least_one, farthest_at_least_two)) =
            PINNED_FARTHEST_VERSION_WITNESSES_OF_THE_BIG_RING.get(size_index)
        {
            assert_eq!(
                (
                    tally.get(Counter::FarthestAtLeastOne),
                    tally.get(Counter::FarthestAtLeastTwo)
                ),
                (*farthest_at_least_one, *farthest_at_least_two),
                "作废条款 V1：L 域 N = {size} 的见证数（j* ≥ 1, j* ≥ 2）与登记第七节不等"
            );
        }
    }
    if let Some(pinned) = pinned_monotone_counts(domain).get(size_index) {
        let counted = [
            tally.pair(PairSet::Wide, PairColumn::Pairs),
            tally.get(Counter::LegalCrashStatesWhenTheRootWriteIsItsOwnSegment),
            tally.pair(PairSet::CutOwnLegal, PairColumn::Pairs),
            tally.pair(PairSet::CutOwnSameSegment, PairColumn::Pairs),
            tally.get(Counter::LegalCrashStatesWhenTheRootWriteSharesTheRecordSegment),
            tally.pair(PairSet::CutWithLegal, PairColumn::Pairs),
            tally.pair(PairSet::CutWithSameSegment, PairColumn::Pairs),
        ];
        let expected = [
            pinned.wide_pairs,
            pinned.cut_own_legal_states,
            pinned.cut_own_legal_pairs,
            pinned.cut_own_same_segment_pairs,
            pinned.cut_with_legal_states,
            pinned.cut_with_legal_pairs,
            pinned.cut_with_same_segment_pairs,
        ];
        assert_eq!(
            counted,
            expected,
            "作废条款 V1：{} 域 Nc = {size} 的状态对数或合法崩溃状态数与登记第七节、第十二节不等（次序：宽对数、甲合法态数、甲合法对数、甲同段对数、乙合法态数、乙合法对数、乙同段对数）",
            domain.label()
        );
    }
    if let Some(pinned) = pinned_mirrored_counts(domain).get(size_index) {
        let class_states = |class: LaggingClass| tally.lagging_class_count(class, Counter::States);
        assert_eq!(
            [
                class_states(LaggingClass::Equal),
                class_states(LaggingClass::FirstDeviceBehind)
                    + class_states(LaggingClass::SecondDeviceBehind),
                class_states(LaggingClass::Crossed),
                class_states(LaggingClass::FirstDeviceBehind),
                class_states(LaggingClass::SecondDeviceBehind),
                tally.get(Counter::FarthestAtLeastOne),
                tally.get(Counter::FarthestAtLeastTwo),
            ],
            [
                pinned.equal_states,
                pinned.lagging_states,
                pinned.crossed_states,
                pinned.lagging_states_in_one_direction,
                pinned.lagging_states_in_one_direction,
                pinned.farthest_at_least_one,
                pinned.farthest_at_least_two,
            ],
            "作废条款 SV7：{} 域 N = {size} 的落后类别或见证数与登记 r2 第七节乙类不等（次序：相等、落后、互不包含、0 号盘落后、1 号盘落后、j* ≥ 1、j* ≥ 2）",
            domain.label()
        );
        assert_eq!(
            LaggingClass::ALL
                .iter()
                .map(|class| class_states(*class))
                .sum::<u64>(),
            tally.get(Counter::States),
            "作废条款 SV7：{} 域 N = {size} 四类状态数之和不等于状态数（登记 r2 5.5「落后类别」那一行）",
            domain.label()
        );
    }
    if let Some(pinned) = pinned_mirrored_monotone_counts(domain).get(size_index) {
        assert_eq!(
            [
                tally.pair(PairSet::Wide, PairColumn::Pairs),
                tally.get(Counter::ChosenRootIsALaterVersion),
                tally.get(Counter::FarthestAtLeastOne),
                tally.get(Counter::FarthestAtLeastTwo),
            ],
            [
                pinned.wide_pairs,
                pinned.chosen_root_is_a_later_version,
                pinned.farthest_at_least_one,
                pinned.farthest_at_least_two,
            ],
            "作废条款 SV7：MC 域 Nc = {size} 的宽对数或见证数与登记 r2 第七节乙类不等（次序：宽对数、所选根不是 R_0、j* − c ≥ 1、j* − c ≥ 2）"
        );
    }
}

// ── 阳性对照（登记 5.4）：判据表的每一行一条假结局，必须红

labelled_enumeration! {
    /// 判据表。加一条判据就多一个成员，[`control_readings_of`] 的穷举 `match` 逼着给它配一条对照。
    enum Criterion {
        J0Defined => "J0",
        J1ReturnedRoot => "J1",
        J2RootPlusDisk => "J2",
        J3WithinTheInFlightLimit => "J3",
        J4FarthestVersion => "J4",
        J5aAppliedSet => "J5a",
        J5bLandedVersion => "J5b",
        J5aAndJ5bAreSeparable => "J5a_vs_J5b",
        J6FixedPoint => "J6",
        Ia1LargestTxgFails => "IA1",
        Ia2NoVerificationCounted => "IA2",
        A0Defined => "A0",
        AsWholeTable => "AS_whole_table",
        AsGapContinues => "AS_consecutive_gap_continues",
        AsGapStops => "AS_consecutive_gap_stops",
        AMutual => "A_mutual",
        AStructure => "A_structure",
        ALimit => "A_limit",
        J6QnFixedPoint => "J6Qn",
        J7aInstanceTable => "J7a_instance_table",
        J7aAllocationRecordTreeRoot => "J7a_allocation_record_tree_root",
        J7aFilesystemIdentifier => "J7a_filesystem_identifier",
        J7bUnmountMarkerOfARebuiltRoot => "J7b",
        J7cUnmountMarkerWhenNothingIsApplied => "J7c",
        J8aFieldsFromTheNewRootSegment => "J8a",
        J8aUnmountMarkerFromTheNewRootSegment => "J8a_unmount_marker",
        J8bFilesystemIdentifierOfTheChosenRoot => "J8b",
        J8cNothingAppliedReturnsTheChosenRoot => "J8c",
        MultiDiskAnyCopyIsOnDisk => "multi_disk_any_copy",
        MultiDiskReturnedRoot => "J1_multi_disk",
        MultiDiskFarthestVersion => "J4_multi_disk",
        MultiDiskAppliedSetAndLandedVersion => "J5_multi_disk",
        TableComparisonSeesADifferentTable => "T_table",
        TableComparisonSeesADifferentRow => "T_row",
        J6QnSplitByI812 => "J6Qn_i812_split",
    }
}

impl Criterion {
    const fn family(self) -> Family {
        match self {
            Self::J0Defined
            | Self::J1ReturnedRoot
            | Self::J2RootPlusDisk
            | Self::J3WithinTheInFlightLimit
            | Self::J4FarthestVersion
            | Self::J5aAppliedSet
            | Self::J5bLandedVersion
            | Self::J5aAndJ5bAreSeparable
            | Self::J6FixedPoint
            | Self::Ia1LargestTxgFails
            | Self::Ia2NoVerificationCounted
            | Self::J6QnFixedPoint
            | Self::J7aInstanceTable
            | Self::J7aAllocationRecordTreeRoot
            | Self::J7aFilesystemIdentifier
            | Self::J7bUnmountMarkerOfARebuiltRoot
            | Self::J7cUnmountMarkerWhenNothingIsApplied
            | Self::J8aFieldsFromTheNewRootSegment
            | Self::J8aUnmountMarkerFromTheNewRootSegment
            | Self::J8bFilesystemIdentifierOfTheChosenRoot
            | Self::J8cNothingAppliedReturnsTheChosenRoot
            | Self::TableComparisonSeesADifferentTable
            | Self::TableComparisonSeesADifferentRow => Family::LegalHistory,
            Self::A0Defined
            | Self::AsWholeTable
            | Self::AsGapContinues
            | Self::AsGapStops
            | Self::AMutual
            | Self::AStructure
            | Self::ALimit
            | Self::J6QnSplitByI812 => Family::Adversarial,
            Self::MultiDiskAnyCopyIsOnDisk
            | Self::MultiDiskReturnedRoot
            | Self::MultiDiskFarthestVersion
            | Self::MultiDiskAppliedSetAndLandedVersion => Family::MultiDisk,
        }
    }

    /// T-表与 T-行判的是每个域上两种建表法的逐态比，对照只拿 L 域跑一遍；每个带 `--table both` 的跑法都带这两条对照，不管选的是哪一族。
    const fn is_judged_in_every_family(self) -> bool {
        matches!(
            self,
            Self::TableComparisonSeesADifferentTable | Self::TableComparisonSeesADifferentRow
        )
    }

    /// 这一档判不判这条判据：J6Qn 与多盘四条在 r2、r3、r5 档判，J7 只在 r2、r3 档判（登记 r2 5.4），J8 只在 r5 档判（登记 r5 5.3），
    /// J6Qn 的 I-8.12 拆行在 r3、r5 档判（登记 r3 5.6），T-表与 T-行只在 `--table both` 时判，别的各档都判。
    const fn is_judged_under(self, run_profile: RunProfile) -> bool {
        match self {
            Self::J6QnSplitByI812 => run_profile.splits_j6qn_by_i812(),
            Self::TableComparisonSeesADifferentTable | Self::TableComparisonSeesADifferentRow => {
                run_profile.compares_the_two_record_tables()
            }
            Self::J7aInstanceTable
            | Self::J7aAllocationRecordTreeRoot
            | Self::J7aFilesystemIdentifier
            | Self::J7bUnmountMarkerOfARebuiltRoot
            | Self::J7cUnmountMarkerWhenNothingIsApplied => run_profile.judges_j7(),
            Self::J8aFieldsFromTheNewRootSegment
            | Self::J8aUnmountMarkerFromTheNewRootSegment
            | Self::J8bFilesystemIdentifierOfTheChosenRoot
            | Self::J8cNothingAppliedReturnsTheChosenRoot => run_profile.judges_j8(),
            Self::J6QnFixedPoint
            | Self::MultiDiskAnyCopyIsOnDisk
            | Self::MultiDiskReturnedRoot
            | Self::MultiDiskFarthestVersion
            | Self::MultiDiskAppliedSetAndLandedVersion => run_profile.judges_j6qn(),
            Self::J0Defined
            | Self::J1ReturnedRoot
            | Self::J2RootPlusDisk
            | Self::J3WithinTheInFlightLimit
            | Self::J4FarthestVersion
            | Self::J5aAppliedSet
            | Self::J5bLandedVersion
            | Self::J5aAndJ5bAreSeparable
            | Self::J6FixedPoint
            | Self::Ia1LargestTxgFails
            | Self::Ia2NoVerificationCounted
            | Self::A0Defined
            | Self::AsWholeTable
            | Self::AsGapContinues
            | Self::AsGapStops
            | Self::AMutual
            | Self::AStructure
            | Self::ALimit => true,
        }
    }
}

/// 阳性对照的一个读数：哪条判据、在哪里量的什么、量到多少、必须看到什么、看到了没有。
#[derive(Clone, Debug)]
struct ControlReading {
    criterion: Criterion,
    measured_where: String,
    value: u64,
    must_see: String,
    is_red_as_required: bool,
}

/// 对照的界取第一段各域的界减半，小环照原界（登记 5.4）。
const CONTROL_LARGEST_SIZE_OF_THE_BIG_RING_LEGAL_DOMAIN: usize = 5;
const CONTROL_LARGEST_SIZE_OF_THE_NAMED_UNITS_DOMAIN: usize = 3;
const CONTROL_LARGEST_SIZE_OF_THE_MONOTONE_BIG_RING_DOMAIN: usize = 3;
const CONTROL_LARGEST_SIZE_OF_THE_ADVERSARIAL_DOMAIN: usize = 2;
/// 对抗一族的 6 槽小环在 I-8.12 拆行的对照里扫到 n = 3（登记 r3 5.7：A6 n ≤ 3，那里前提为假的态才出现）。
const CONTROL_LARGEST_SIZE_OF_THE_I812_SMALL_RING_DOMAIN: usize = 3;
/// 对照界里 L 域 j\* ≥ 1 的态数：登记第七节见证数 N = 1..5 的和，0 + 2 + 16 + 104 + 640。
const FARTHEST_AT_LEAST_ONE_STATES_WITHIN_THE_CONTROL_BOUND: u64 = 762;
/// 对照界里 L 域的态数：登记 r2 第七节乙类「L 大环对照界 N ≤ 5：态数 3 110」（2 + 12 + 72 + 432 + 2 592）。
const STATES_WITHIN_THE_CONTROL_BOUND_OF_THE_BIG_RING_LEGAL_DOMAIN: u64 = 3_110;
/// 对照界里 L 域 j\* = 0 的态数：登记 r2 第七节乙类「j\* = 0 2 348」。
const FARTHEST_IS_THE_CHOSEN_VERSION_STATES_WITHIN_THE_CONTROL_BOUND: u64 = 2_348;
/// 对照界里 L 域 j\* 为奇数的态数：登记 r5 第七节乙类「j\* 为奇数 620」（逐 N 0 + 2 + 14 + 86 + 518，`anchors_r5.py`）。
const FARTHEST_IS_ODD_STATES_WITHIN_THE_CONTROL_BOUND: u64 = 620;
/// 对照界里 L 域 j\* 为偶数且 ≥ 2 的态数：登记 r5 第七节乙类「j\* 为偶数且 ≥ 2 的 142」（620 + 142 = 762）。
const FARTHEST_IS_EVEN_AND_AT_LEAST_TWO_STATES_WITHIN_THE_CONTROL_BOUND: u64 = 142;
/// E1 全界（N ≤ 4）上 j\* = 1 的态数：登记 r2 第七节乙类「E1 上 j\* = 1 的态数 0、2、12、56，合 70」。
const E1_STATES_WHOSE_FARTHEST_VERSION_TXG_IS_THE_LARGEST: u64 = 70;
/// 多盘对照的界（登记 r2 5.5 末段）：M 到 N = 3，M6 到 N = 4，MC 到 Nc = 2。
const CONTROL_LARGEST_SIZE_OF_THE_MIRRORED_BIG_RING_DOMAIN: usize = 3;
const CONTROL_LARGEST_SIZE_OF_THE_MIRRORED_SMALL_RING_SIX_DOMAIN: usize = 4;
const CONTROL_LARGEST_SIZE_OF_THE_MIRRORED_MONOTONE_DOMAIN: usize = 2;
/// 登记 r2 第七节「多盘「任一份在即在」」：只数 0 号盘的 j\* 与任一份的 j\* 不等的态数，M 域 N ≤ 3 是 0 + 4 + 96，
/// M6 域 N ≤ 4 是 0 + 4 + 96 + 896（`anchors_r2c.py` 的 `jstar_disk0_only_differs`）。
const MIRRORED_BIG_RING_STATES_WHERE_THE_FIRST_DEVICE_ALONE_GIVES_ANOTHER_FARTHEST_VERSION: u64 =
    100;
const MIRRORED_SMALL_RING_SIX_STATES_WHERE_THE_FIRST_DEVICE_ALONE_GIVES_ANOTHER_FARTHEST_VERSION:
    u64 = 996;
/// 原登记 J1、J4 两条对照在多盘域上的绝对值（登记 r2 第七节）：j\* − c ≥ 1 的态数，M 域 N ≤ 3 是 0 + 12 + 216，
/// M6 域 N ≤ 4 是 0 + 12 + 216 + 1 824，MC 域 Nc ≤ 2 是 3 + 42。
const MIRRORED_BIG_RING_FARTHEST_AT_LEAST_ONE_WITHIN_THE_CONTROL_BOUND: u64 = 228;
const MIRRORED_SMALL_RING_SIX_FARTHEST_AT_LEAST_ONE_WITHIN_THE_CONTROL_BOUND: u64 = 2_052;
const MIRRORED_MONOTONE_FARTHEST_AT_LEAST_ONE_WITHIN_THE_CONTROL_BOUND: u64 = 45;

/// 一条假结局跑在哪个档上：T-表、T-行两条对照拿 `--table both`，别的对照一律拿直接建表、盘次序照常（对照证的是判据分得出差别，
/// 与这一次跑选了哪张表、盘报什么次序无关；这样 `--table both` 的跑法里别的对照读数与默认跑法逐字相同）。
fn run_profile_of_a_control(control: ControlKind, run_profile: RunProfile) -> RunProfile {
    let without_the_record_table_knobs = run_profile
        .with_table_source(RecordTableSource::Direct)
        .with_reported_disk_order(ReportedDiskOrder::FirstDeviceBeforeSecond);
    if matches!(
        control,
        ControlKind::ScanTableLosesItsLargestKeyRecord | ControlKind::ScanPassOutcomeBecomesAPanic
    ) {
        without_the_record_table_knobs.with_table_source(RecordTableSource::Both)
    } else {
        without_the_record_table_knobs
    }
}

/// 一条假结局在一个域上、从最小的 N 扫到 `largest_size` 的合计。
fn control_tally(
    domain: Domain,
    largest_size: usize,
    control: ControlKind,
    run_profile: RunProfile,
    thread_count: usize,
) -> Tally {
    let run_profile = run_profile_of_a_control(control, run_profile);
    let mut total = Tally::new();
    for size in domain.smallest_size()..=largest_size {
        total.merge(&tally_of_a_domain_size(
            domain,
            size,
            Implementation::Control(control),
            run_profile,
            thread_count,
        ));
    }
    total
}

/// AS 三条的对照要 A-互 三个数都不为 0；对照界 n ≤ 2 里三个数全为 0 时照登记 5.4 放宽 n，一档一档放到第一段的界为止。
fn smallest_adversarial_size_where_the_three_readings_part(
    run_profile: RunProfile,
    thread_count: usize,
) -> usize {
    let largest_registered_size = BoundsTier::SegmentOne
        .largest_size(Domain::AdversarialBigRing)
        .expect("第一段跑 A 域");
    for largest_size in CONTROL_LARGEST_SIZE_OF_THE_ADVERSARIAL_DOMAIN..=largest_registered_size {
        let tally = control_tally(
            Domain::AdversarialBigRing,
            largest_size,
            ControlKind::SpecificationWithReading(FollowReading::WholeTable),
            run_profile,
            thread_count,
        );
        if tally.get(Counter::MutualWholeTableAndGapContinues) > 0
            && tally.get(Counter::MutualWholeTableAndGapStops) > 0
            && tally.get(Counter::MutualGapContinuesAndGapStops) > 0
        {
            return largest_size;
        }
    }
    largest_registered_size
}

fn reading_that_must_be_positive(
    criterion: Criterion,
    measured_where: String,
    value: u64,
) -> ControlReading {
    ControlReading {
        criterion,
        measured_where,
        value,
        must_see: "greater_than_0".to_string(),
        is_red_as_required: value > 0,
    }
}

fn reading_that_must_equal(
    criterion: Criterion,
    measured_where: String,
    value: u64,
    expected: u64,
) -> ControlReading {
    ControlReading {
        criterion,
        measured_where,
        value,
        must_see: format!("equals_{expected}"),
        is_red_as_required: value == expected,
    }
}

/// AS 三条各自的对照：拿另一种读法的 S 当实现，违例数必须等于 A-互 里那两种读法之间的那个数，且不为 0。
fn specification_control_readings(
    criterion: Criterion,
    other_reading: FollowReading,
    violation_counter: Counter,
    mutual_counter: Counter,
    run_profile: RunProfile,
    thread_count: usize,
) -> Vec<ControlReading> {
    let largest_size =
        smallest_adversarial_size_where_the_three_readings_part(run_profile, thread_count);
    let tally = control_tally(
        Domain::AdversarialBigRing,
        largest_size,
        ControlKind::SpecificationWithReading(other_reading),
        run_profile,
        thread_count,
    );
    vec![
        reading_that_must_equal(
            criterion,
            format!(
                "domain=A,max_n={largest_size},implementation=S_{},counter={}",
                other_reading.label(),
                violation_counter.label()
            ),
            tally.get(violation_counter),
            tally.get(mutual_counter),
        ),
        reading_that_must_be_positive(
            criterion,
            format!(
                "domain=A,max_n={largest_size},counter={}",
                mutual_counter.label()
            ),
            tally.get(mutual_counter),
        ),
    ]
}

/// 判据表一行的阳性对照（登记 5.4 那张表的一行）。
fn control_readings_of(
    criterion: Criterion,
    run_profile: RunProfile,
    thread_count: usize,
) -> Vec<ControlReading> {
    let big_ring_legal = |control: ControlKind| {
        control_tally(
            Domain::LegalBigRing,
            CONTROL_LARGEST_SIZE_OF_THE_BIG_RING_LEGAL_DOMAIN,
            control,
            run_profile,
            thread_count,
        )
    };
    let adversarial = |largest_size: usize, control: ControlKind| {
        control_tally(
            Domain::AdversarialBigRing,
            largest_size,
            control,
            run_profile,
            thread_count,
        )
    };
    let legal_where = |counter: Counter| {
        format!(
            "domain=L,max_n={CONTROL_LARGEST_SIZE_OF_THE_BIG_RING_LEGAL_DOMAIN},counter={}",
            counter.label()
        )
    };
    match criterion {
        Criterion::J0Defined => {
            let tally = big_ring_legal(ControlKind::PanicsOnTheDesignatedState);
            vec![reading_that_must_equal(
                criterion,
                legal_where(Counter::Panicked),
                tally.get(Counter::Panicked),
                1,
            )]
        }
        Criterion::J1ReturnedRoot => {
            let tally = big_ring_legal(ControlKind::OffTimelineRootWheneverApplied);
            vec![
                reading_that_must_equal(
                    criterion,
                    legal_where(Counter::J1Violations),
                    tally.get(Counter::J1Violations),
                    FARTHEST_AT_LEAST_ONE_STATES_WITHIN_THE_CONTROL_BOUND,
                ),
                reading_that_must_equal(
                    criterion,
                    legal_where(Counter::FarthestAtLeastOne),
                    tally.get(Counter::FarthestAtLeastOne),
                    FARTHEST_AT_LEAST_ONE_STATES_WITHIN_THE_CONTROL_BOUND,
                ),
            ]
        }
        Criterion::J2RootPlusDisk => {
            let tally = big_ring_legal(ControlKind::LandsOnePublishTooFarOverAnIncompletePublish);
            let named_units_tally = control_tally(
                Domain::NamedUnitsVerified,
                CONTROL_LARGEST_SIZE_OF_THE_NAMED_UNITS_DOMAIN,
                ControlKind::LandsOnePublishTooFarOverAnIncompletePublish,
                run_profile,
                thread_count,
            );
            vec![
                reading_that_must_be_positive(
                    criterion,
                    legal_where(Counter::J2Violations),
                    tally.get(Counter::J2Violations),
                ),
                reading_that_must_equal(
                    criterion,
                    legal_where(Counter::J1Violations),
                    tally.get(Counter::J1Violations),
                    0,
                ),
                reading_that_must_be_positive(
                    criterion,
                    format!(
                        "domain=U_verify_on,max_n={CONTROL_LARGEST_SIZE_OF_THE_NAMED_UNITS_DOMAIN},counter=J2"
                    ),
                    named_units_tally.get(Counter::J2Violations),
                ),
            ]
        }
        Criterion::J3WithinTheInFlightLimit => {
            let small_ring_largest_size = BoundsTier::SegmentOne
                .largest_size(Domain::LegalSmallRingSix)
                .expect("第一段跑 L 小环 6");
            let small_ring_tally = control_tally(
                Domain::LegalSmallRingSix,
                small_ring_largest_size,
                ControlKind::IgnoresTheInFlightLimit,
                run_profile,
                thread_count,
            );
            let big_ring_tally = big_ring_legal(ControlKind::IgnoresTheInFlightLimit);
            vec![
                reading_that_must_be_positive(
                    criterion,
                    format!("domain=L6,max_n={small_ring_largest_size},counter=J3"),
                    small_ring_tally.get(Counter::J3Violations),
                ),
                reading_that_must_equal(
                    criterion,
                    legal_where(Counter::J3Violations),
                    big_ring_tally.get(Counter::J3Violations),
                    0,
                ),
            ]
        }
        Criterion::J4FarthestVersion => {
            let tally = big_ring_legal(ControlKind::AlwaysReturnsTheChosenRoot);
            vec![
                reading_that_must_equal(
                    criterion,
                    legal_where(Counter::J4Violations),
                    tally.get(Counter::J4Violations),
                    FARTHEST_AT_LEAST_ONE_STATES_WITHIN_THE_CONTROL_BOUND,
                ),
                reading_that_must_equal(
                    criterion,
                    legal_where(Counter::J1Violations),
                    tally.get(Counter::J1Violations),
                    0,
                ),
                reading_that_must_equal(
                    criterion,
                    legal_where(Counter::J2Violations),
                    tally.get(Counter::J2Violations),
                    0,
                ),
                reading_that_must_equal(
                    criterion,
                    legal_where(Counter::J3Violations),
                    tally.get(Counter::J3Violations),
                    0,
                ),
            ]
        }
        Criterion::J5aAppliedSet | Criterion::J5bLandedVersion => {
            let tally = control_tally(
                Domain::MonotoneBigRing,
                CONTROL_LARGEST_SIZE_OF_THE_MONOTONE_BIG_RING_DOMAIN,
                ControlKind::ReturnsTheChosenRootWhenEveryRecordIsOnDisk,
                run_profile,
                thread_count,
            );
            let column = if criterion == Criterion::J5aAppliedSet {
                PairColumn::TodayAppliedSetViolations
            } else {
                PairColumn::TodayLandedVersionViolations
            };
            vec![reading_that_must_be_positive(
                criterion,
                format!(
                    "domain=C,max_n={CONTROL_LARGEST_SIZE_OF_THE_MONOTONE_BIG_RING_DOMAIN},set=wide,column={}",
                    column.label()
                ),
                tally.pair(PairSet::Wide, column),
            )]
        }
        Criterion::J5aAndJ5bAreSeparable => {
            let small_ring_largest_size = BoundsTier::SegmentOne
                .largest_size(Domain::MonotoneSmallRingSix)
                .expect("第一段跑 C 小环 6");
            let mut readings = Vec::new();
            for (domain, largest_size) in [
                (
                    Domain::MonotoneBigRing,
                    CONTROL_LARGEST_SIZE_OF_THE_MONOTONE_BIG_RING_DOMAIN,
                ),
                (Domain::MonotoneSmallRingSix, small_ring_largest_size),
            ] {
                let tally = control_tally(
                    domain,
                    largest_size,
                    ControlKind::IdealWithTheCumulativeAppliedSet,
                    run_profile,
                    thread_count,
                );
                for set in PairSet::ALL {
                    readings.push(reading_that_must_equal(
                        criterion,
                        format!(
                            "domain={},max_n={largest_size},set={},column=today_j5a_must_equal_today_j5b",
                            domain.label(),
                            set.label()
                        ),
                        tally.pair(*set, PairColumn::TodayAppliedSetViolations),
                        tally.pair(*set, PairColumn::TodayLandedVersionViolations),
                    ));
                }
            }
            readings
        }
        Criterion::J6FixedPoint => {
            let tally = big_ring_legal(ControlKind::SecondCallMovesOn);
            vec![reading_that_must_be_positive(
                criterion,
                legal_where(Counter::J6PremiseHoldsViolations),
                tally.get(Counter::J6PremiseHoldsViolations)
                    + tally.get(Counter::J6PremiseFailsViolations),
            )]
        }
        Criterion::Ia1LargestTxgFails => {
            let largest_size = BoundsTier::SegmentOne
                .largest_size(Domain::TxgAtTheLargest)
                .expect("第一段跑 E2");
            let tally = control_tally(
                Domain::TxgAtTheLargest,
                largest_size,
                ControlKind::ReportsNoFailureOnTheLargestTxg,
                run_profile,
                thread_count,
            );
            vec![reading_that_must_be_positive(
                criterion,
                format!("domain=E2,max_n={largest_size},counter=IA1"),
                tally.get(Counter::Ia1Violations),
            )]
        }
        Criterion::Ia2NoVerificationCounted => {
            let tally = control_tally(
                Domain::NamedUnitsNotVerified,
                CONTROL_LARGEST_SIZE_OF_THE_NAMED_UNITS_DOMAIN,
                ControlKind::ReportsOneVerificationWithoutVerifying,
                run_profile,
                thread_count,
            );
            vec![reading_that_must_be_positive(
                criterion,
                format!(
                    "domain=U_verify_off,max_n={CONTROL_LARGEST_SIZE_OF_THE_NAMED_UNITS_DOMAIN},counter=IA2"
                ),
                tally.get(Counter::Ia2Violations),
            )]
        }
        Criterion::A0Defined => {
            let tally = adversarial(
                CONTROL_LARGEST_SIZE_OF_THE_ADVERSARIAL_DOMAIN,
                ControlKind::PanicsOnTheDesignatedState,
            );
            vec![reading_that_must_equal(
                criterion,
                format!(
                    "domain=A,max_n={CONTROL_LARGEST_SIZE_OF_THE_ADVERSARIAL_DOMAIN},counter=panicked"
                ),
                tally.get(Counter::Panicked),
                1,
            )]
        }
        Criterion::AsWholeTable => specification_control_readings(
            criterion,
            FollowReading::ConsecutiveGapContinues,
            Counter::AsWholeTableViolations,
            Counter::MutualWholeTableAndGapContinues,
            run_profile,
            thread_count,
        ),
        Criterion::AsGapContinues => specification_control_readings(
            criterion,
            FollowReading::ConsecutiveGapStops,
            Counter::AsGapContinuesViolations,
            Counter::MutualGapContinuesAndGapStops,
            run_profile,
            thread_count,
        ),
        Criterion::AsGapStops => specification_control_readings(
            criterion,
            FollowReading::WholeTable,
            Counter::AsGapStopsViolations,
            Counter::MutualWholeTableAndGapStops,
            run_profile,
            thread_count,
        ),
        Criterion::AMutual => {
            let largest_size =
                smallest_adversarial_size_where_the_three_readings_part(run_profile, thread_count);
            let tally = adversarial(
                largest_size,
                ControlKind::SpecificationWithReading(FollowReading::NeverJudges),
            );
            vec![reading_that_must_be_positive(
                criterion,
                format!(
                    "domain=A,max_n={largest_size},implementation=S_never_judges,counter=AS_whole_table"
                ),
                tally.get(Counter::AsWholeTableViolations),
            )]
        }
        Criterion::AStructure => {
            let tally = adversarial(
                CONTROL_LARGEST_SIZE_OF_THE_ADVERSARIAL_DOMAIN,
                ControlKind::RootFromARecordWithoutTheLastRecordFlag,
            );
            vec![reading_that_must_be_positive(
                criterion,
                format!(
                    "domain=A,max_n={CONTROL_LARGEST_SIZE_OF_THE_ADVERSARIAL_DOMAIN},counter=A_structure"
                ),
                tally.get(Counter::StructureViolations),
            )]
        }
        Criterion::ALimit => {
            let tally = adversarial(
                CONTROL_LARGEST_SIZE_OF_THE_ADVERSARIAL_DOMAIN,
                ControlKind::ReportsOneRecordOverTheLimit,
            );
            vec![reading_that_must_equal(
                criterion,
                format!(
                    "domain=A,max_n={CONTROL_LARGEST_SIZE_OF_THE_ADVERSARIAL_DOMAIN},counter=A_limit_must_equal_states"
                ),
                tally.get(Counter::LimitViolations),
                tally.get(Counter::States),
            )]
        }
        Criterion::J6QnFixedPoint => j6qn_control_readings(run_profile, thread_count),
        Criterion::J7aInstanceTable => j7a_control_readings(
            criterion,
            FieldOfTheRebuiltRoot::InstanceTable,
            run_profile,
            thread_count,
        ),
        Criterion::J7aAllocationRecordTreeRoot => j7a_control_readings(
            criterion,
            FieldOfTheRebuiltRoot::AllocationRecordTreeRoot,
            run_profile,
            thread_count,
        ),
        Criterion::J7aFilesystemIdentifier => j7a_control_readings(
            criterion,
            FieldOfTheRebuiltRoot::FilesystemIdentifier,
            run_profile,
            thread_count,
        ),
        Criterion::J7bUnmountMarkerOfARebuiltRoot => unmount_marker_control_readings(
            criterion,
            ControlKind::IdealCopyingTheChosenUnmountMarkerWhenApplied,
            Counter::J7bViolations,
            FARTHEST_AT_LEAST_ONE_STATES_WITHIN_THE_CONTROL_BOUND,
            run_profile,
            thread_count,
        ),
        Criterion::J7cUnmountMarkerWhenNothingIsApplied => unmount_marker_control_readings(
            criterion,
            ControlKind::IdealWritingNotWrittenWhenNothingIsApplied,
            Counter::J7cViolations,
            FARTHEST_IS_THE_CHOSEN_VERSION_STATES_WITHIN_THE_CONTROL_BOUND,
            run_profile,
            thread_count,
        ),
        Criterion::J8aFieldsFromTheNewRootSegment => {
            c8a_control_readings(run_profile, thread_count)
        }
        Criterion::J8aUnmountMarkerFromTheNewRootSegment => {
            c8b_control_readings(run_profile, thread_count)
        }
        Criterion::J8bFilesystemIdentifierOfTheChosenRoot => {
            c8c_control_readings(run_profile, thread_count)
        }
        Criterion::J8cNothingAppliedReturnsTheChosenRoot => {
            c8d_control_readings(run_profile, thread_count)
        }
        Criterion::MultiDiskAnyCopyIsOnDisk => any_copy_control_readings(run_profile, thread_count),
        Criterion::MultiDiskReturnedRoot => {
            mirrored_returned_root_control_readings(run_profile, thread_count)
        }
        Criterion::MultiDiskFarthestVersion => {
            mirrored_farthest_version_control_readings(run_profile, thread_count)
        }
        Criterion::MultiDiskAppliedSetAndLandedVersion => {
            mirrored_monotone_control_readings(run_profile, thread_count)
        }
        Criterion::TableComparisonSeesADifferentTable => {
            table_comparison_control_readings_on_tables(criterion, run_profile, thread_count)
        }
        Criterion::TableComparisonSeesADifferentRow => {
            table_comparison_control_readings_on_rows(criterion, run_profile, thread_count)
        }
        Criterion::J6QnSplitByI812 => i812_split_control_readings(run_profile, thread_count),
    }
}

// ── J6Qn 按 I-8.12 拆行（登记 r3 5.6、5.7、第七节乙类）

/// 对抗一族每个 (域, n) 上，表满足 / 不满足 I-8.12 的态数（登记 r3 第七节乙类「Q3 的分类计数」，`anchors_r3_i812.py` 动态规划与暴力枚举互核）：
/// (域, n, 成立, 不成立)。三个域各 n 的态数之和等于原登记 5.7 的 57 585 320、788 838、6 139 590。
const PINNED_I812_STATE_COUNTS: [(Domain, usize, u64, u64); 10] = [
    (Domain::AdversarialBigRing, 1, 146, 0),
    (Domain::AdversarialBigRing, 2, 9_122, 1_536),
    (Domain::AdversarialBigRing, 3, 523_570, 254_464),
    (Domain::AdversarialBigRing, 4, 28_459_330, 28_337_152),
    (Domain::AdversarialSmallRingSix, 1, 146, 0),
    (Domain::AdversarialSmallRingSix, 2, 9_122, 1_536),
    (Domain::AdversarialSmallRingSix, 3, 523_570, 254_464),
    (Domain::AdversarialBackChain, 1, 290, 0),
    (Domain::AdversarialBackChain, 2, 35_906, 6_144),
    (Domain::AdversarialBackChain, 3, 4_079_970, 2_017_280),
];

/// KF3 假结局（第一次返回所选根、第二次报「不止一条末条标志」）在对照界里必须读到的四格（登记 r3 第七节乙类，`anchors_r3_i812_premise.py`）：
/// (域, n, 前提真·I-8.12 成立, 前提真·不成立, 前提假·成立, 前提假·不成立)。每格的违例数与 judged 都等于它（对照界内第一次调用都「返回」）。
const PINNED_KF3_CELLS: [(Domain, usize, [u64; 4]); 7] = [
    (Domain::AdversarialBigRing, 1, [146, 0, 0, 0]),
    (Domain::AdversarialBigRing, 2, [9_122, 1_536, 0, 0]),
    (Domain::AdversarialSmallRingSix, 1, [146, 0, 0, 0]),
    (Domain::AdversarialSmallRingSix, 2, [9_122, 1_536, 0, 0]),
    (
        Domain::AdversarialSmallRingSix,
        3,
        [490_802, 221_696, 32_768, 32_768],
    ),
    (Domain::AdversarialBackChain, 1, [290, 0, 0, 0]),
    (Domain::AdversarialBackChain, 2, [35_906, 6_144, 0, 0]),
];

/// 四格的次序（前提真·成立、前提真·不成立、前提假·成立、前提假·不成立），与 [`PINNED_KF3_CELLS`] 里四个数的次序一致。
const I812_SPLIT_CELLS: [(bool, bool); 4] =
    [(true, true), (true, false), (false, true), (false, false)];

fn pinned_i812_state_counts(domain: Domain, size: usize) -> Option<(u64, u64)> {
    PINNED_I812_STATE_COUNTS
        .iter()
        .find(|(pinned_domain, pinned_size, _, _)| *pinned_domain == domain && *pinned_size == size)
        .map(|(_, _, holds, fails)| (*holds, *fails))
}

fn pinned_kf3_cells(domain: Domain, size: usize) -> Option<[u64; 4]> {
    PINNED_KF3_CELLS
        .iter()
        .find(|(pinned_domain, pinned_size, _)| *pinned_domain == domain && *pinned_size == size)
        .map(|(_, _, cells)| *cells)
}

/// 对抗一族这一个 (域, n) 上，表满足 / 不满足 I-8.12 的态数等于登记的分类计数；不符 ⇒ 作废条款 TV4（装置数错了）。
fn assert_the_pinned_i812_state_counts(domain: Domain, size: usize, tally: &Tally) {
    let (pinned_holding, pinned_failing) = pinned_i812_state_counts(domain, size).unwrap_or_else(|| {
        panic!(
            "作废条款 TV4：{} 域 n = {size} 没有登记 r3 第七节乙类的 I-8.12 分类计数，这一格没被钉住",
            domain.label()
        )
    });
    assert_eq!(
        (
            tally.get(Counter::TablesSatisfyI812),
            tally.get(Counter::TablesViolateI812)
        ),
        (pinned_holding, pinned_failing),
        "作废条款 TV4：{} 域 n = {size} 的 I-8.12 成立 / 不成立态数与登记 r3 第七节乙类不等",
        domain.label()
    );
    assert_eq!(
        tally.get(Counter::TablesSatisfyI812) + tally.get(Counter::TablesViolateI812),
        tally.get(Counter::States),
        "作废条款 TV4：{} 域 n = {size} 的 I-8.12 成立与不成立两类态数之和不等于状态数",
        domain.label()
    );
}

/// 拆开的四行对回未拆的两行（登记 r3 作废条款 TV6）：前提真那两格的违例数之和与 judged 之和各等于 `J6Qn_premise_holds` 一行的，前提假同理。
/// 对得上交回 `None`，对不上交回一句说明。
fn the_split_rows_differ_from_the_unsplit_rows(tally: &Tally) -> Option<String> {
    let sum_of_cells = |premise_holds: bool, pick: fn((Counter, Counter)) -> Counter| {
        I812_SPLIT_CELLS
            .iter()
            .filter(|(cell_premise_holds, _)| *cell_premise_holds == premise_holds)
            .map(|(cell_premise_holds, i812_holds)| {
                tally.get(pick(j6qn_cell_counters(*cell_premise_holds, *i812_holds)))
            })
            .sum::<u64>()
    };
    let judged_of = |cells: (Counter, Counter)| cells.0;
    let violations_of = |cells: (Counter, Counter)| cells.1;
    let split_and_unsplit = [
        (
            "前提真的违例数",
            sum_of_cells(true, violations_of),
            tally.get(Counter::J6QnPremiseHoldsViolations),
        ),
        (
            "前提真的 judged",
            sum_of_cells(true, judged_of),
            tally.get(Counter::J6QnPremiseHoldsJudged),
        ),
        (
            "前提假的违例数",
            sum_of_cells(false, violations_of),
            tally.get(Counter::J6QnPremiseFailsViolations),
        ),
        (
            "前提假的 judged",
            sum_of_cells(false, judged_of),
            tally.get(Counter::J6QnPremiseFailsJudged),
        ),
    ];
    let differing: Vec<String> = split_and_unsplit
        .iter()
        .filter(|(_, split_sum, unsplit)| split_sum != unsplit)
        .map(|(what, split_sum, unsplit)| {
            format!("{what}：拆开之后之和 {split_sum}、未拆那一行 {unsplit}")
        })
        .collect();
    (!differing.is_empty()).then(|| differing.join("；"))
}

/// 对抗一族一个 (域, n) 的 I-8.12 拆行（登记 r3 5.6）：四格各一行（judged、违例数、违例里最先的见证），另一行报全部状态里表成立 / 不成立的态数。
/// 不是 r3 档、或不是对抗一族，一行都不出。
fn rows_of_the_i812_split(
    run_profile: RunProfile,
    domain: Domain,
    place: &str,
    tally: &Tally,
) -> Vec<String> {
    if !run_profile.splits_j6qn_by_i812() || domain.family() != Family::Adversarial {
        return Vec::new();
    }
    let mut rows: Vec<String> = I812_SPLIT_CELLS
        .iter()
        .map(|(premise_holds, i812_holds)| {
            let (judged_counter, violations_counter) =
                j6qn_cell_counters(*premise_holds, *i812_holds);
            format!(
                "name=violations {place} criterion={} violations={} judged={} witness={}",
                violations_counter.label(),
                tally.get(violations_counter),
                tally.get(judged_counter),
                witness_field(tally, violations_counter)
            )
        })
        .collect();
    rows.push(format!(
        "name=i812_states {place} i812_holds={} i812_fails={} states={}",
        tally.get(Counter::TablesSatisfyI812),
        tally.get(Counter::TablesViolateI812),
        tally.get(Counter::States)
    ));
    rows
}

/// 对抗一族一个域在全部 n 上的 I-8.12 拆行合计（登记 r3 5.6「每个域收尾的 criteria=r2 判决行后面另起一行 criteria=r3_i812」）。
fn verdict_row_of_the_i812_split(place: &str, total: &Tally) -> String {
    let cell = |premise_holds: bool, i812_holds: bool| {
        let (judged_counter, violations_counter) = j6qn_cell_counters(premise_holds, i812_holds);
        (total.get(violations_counter), total.get(judged_counter))
    };
    let (holds_holds_violations, holds_holds_judged) = cell(true, true);
    let (holds_fails_violations, holds_fails_judged) = cell(true, false);
    let (fails_holds_violations, fails_holds_judged) = cell(false, true);
    let (fails_fails_violations, fails_fails_judged) = cell(false, false);
    format!(
        "name=verdict {place} criteria=r3_i812 j6qn_premise_holds_i812_holds_violations={holds_holds_violations} j6qn_premise_holds_i812_holds_judged={holds_holds_judged} j6qn_premise_holds_i812_fails_violations={holds_fails_violations} j6qn_premise_holds_i812_fails_judged={holds_fails_judged} j6qn_premise_fails_i812_holds_violations={fails_holds_violations} j6qn_premise_fails_i812_holds_judged={fails_holds_judged} j6qn_premise_fails_i812_fails_violations={fails_fails_violations} j6qn_premise_fails_i812_fails_judged={fails_fails_judged} i812_holds={} i812_fails={} split_rows_equal_the_unsplit_rows={}",
        total.get(Counter::TablesSatisfyI812),
        total.get(Counter::TablesViolateI812),
        the_split_rows_differ_from_the_unsplit_rows(total).is_none()
    )
}

/// KF3 的对照（登记 r3 5.7、第七节乙类）：对抗一族对照界（A n ≤ 2、A6 n ≤ 3、AC n ≤ 2）每个 n 上，第一次返回所选根、第二次报「不止一条末条标志」，
/// 四格各自的违例数与 judged 都等于钉的数；全部状态里表成立 / 不成立的态数也等于钉的分类计数。
fn i812_split_control_readings(
    run_profile: RunProfile,
    thread_count: usize,
) -> Vec<ControlReading> {
    let criterion = Criterion::J6QnSplitByI812;
    let control =
        ControlKind::FirstCallReturnsTheChosenRootAndSecondCallReportsMoreThanOneLastRecordFlag;
    let mut readings = Vec::new();
    for (domain, largest_size) in [
        (
            Domain::AdversarialBigRing,
            CONTROL_LARGEST_SIZE_OF_THE_ADVERSARIAL_DOMAIN,
        ),
        (
            Domain::AdversarialSmallRingSix,
            CONTROL_LARGEST_SIZE_OF_THE_I812_SMALL_RING_DOMAIN,
        ),
        (
            Domain::AdversarialBackChain,
            CONTROL_LARGEST_SIZE_OF_THE_ADVERSARIAL_DOMAIN,
        ),
    ] {
        for size in domain.smallest_size()..=largest_size {
            let tally = tally_of_a_domain_size(
                domain,
                size,
                Implementation::Control(control),
                run_profile_of_a_control(control, run_profile),
                thread_count,
            );
            let pinned_cells = pinned_kf3_cells(domain, size)
                .expect("对照界里每个 (域, n) 都有登记 r3 第七节乙类 KF3 的四格");
            let (pinned_holding, pinned_failing) = pinned_i812_state_counts(domain, size)
                .expect("对照界里每个 (域, n) 都有登记 r3 第七节乙类的分类计数");
            let place = format!(
                "domain={},n={size},implementation=first_call_returns_the_chosen_root_second_call_reports_more_than_one_last_record_flag",
                domain.label()
            );
            for ((premise_holds, i812_holds), pinned) in I812_SPLIT_CELLS.iter().zip(pinned_cells) {
                let (judged_counter, violations_counter) =
                    j6qn_cell_counters(*premise_holds, *i812_holds);
                readings.push(reading_that_must_equal(
                    criterion,
                    format!("{place},counter={}", violations_counter.label()),
                    tally.get(violations_counter),
                    pinned,
                ));
                readings.push(reading_that_must_equal(
                    criterion,
                    format!("{place},counter={}", judged_counter.label()),
                    tally.get(judged_counter),
                    pinned,
                ));
            }
            readings.push(reading_that_must_equal(
                criterion,
                format!("{place},counter={}", Counter::TablesSatisfyI812.label()),
                tally.get(Counter::TablesSatisfyI812),
                pinned_holding,
            ));
            readings.push(reading_that_must_equal(
                criterion,
                format!("{place},counter={}", Counter::TablesViolateI812.label()),
                tally.get(Counter::TablesViolateI812),
                pinned_failing,
            ));
        }
    }
    readings
}

/// L 域 N ≤ 5 上直接建的表非空的态数（至少一条记录落了盘的态数）：登记 r2 第七节乙类「L 大环对照界 N ≤ 5：……至少一条记录落盘的态数 2 989」
/// （每段历史的 2^N 个落盘子集里去掉空集：1 + 9 + 63 + 405 + 2 511）。
const STATES_WITH_A_RECORD_ON_DISK_WITHIN_THE_CONTROL_BOUND_OF_THE_BIG_RING_LEGAL_DOMAIN: u64 =
    2_989;

/// T-表的对照（登记 r2 5.5、第七节）：扫盘建表之后删掉键最大的那条记录、直接建的表不动。L 域 N ≤ 5 上 `tables_differ` 等于直接建的表非空的态数
/// （装置另数一遍，钉 2 989），`rows_differ` 大于 0。
fn table_comparison_control_readings_on_tables(
    criterion: Criterion,
    run_profile: RunProfile,
    thread_count: usize,
) -> Vec<ControlReading> {
    let tally = control_tally(
        Domain::LegalBigRing,
        CONTROL_LARGEST_SIZE_OF_THE_BIG_RING_LEGAL_DOMAIN,
        ControlKind::ScanTableLosesItsLargestKeyRecord,
        run_profile,
        thread_count,
    );
    let place = |counter: Counter| {
        control_place(
            Domain::LegalBigRing,
            CONTROL_LARGEST_SIZE_OF_THE_BIG_RING_LEGAL_DOMAIN,
            "scan_table_loses_its_largest_key_record",
            counter,
        )
    };
    vec![
        reading_that_must_equal(
            criterion,
            place(Counter::TablesDiffer),
            tally.get(Counter::TablesDiffer),
            tally.get(Counter::DirectTableHoldsARecord),
        ),
        reading_that_must_equal(
            criterion,
            place(Counter::DirectTableHoldsARecord),
            tally.get(Counter::DirectTableHoldsARecord),
            STATES_WITH_A_RECORD_ON_DISK_WITHIN_THE_CONTROL_BOUND_OF_THE_BIG_RING_LEGAL_DOMAIN,
        ),
        reading_that_must_be_positive(
            criterion,
            place(Counter::RowsDiffer),
            tally.get(Counter::RowsDiffer),
        ),
    ]
}

/// T-行的对照（登记 r2 5.5、第七节）：扫盘建表那一遍的实现结局换成 `Panicked`、表不动。L 域 N ≤ 5 上 `rows_differ` 等于状态数（3 110）、
/// `tables_differ` 是 0（两格分得开）。
fn table_comparison_control_readings_on_rows(
    criterion: Criterion,
    run_profile: RunProfile,
    thread_count: usize,
) -> Vec<ControlReading> {
    let tally = control_tally(
        Domain::LegalBigRing,
        CONTROL_LARGEST_SIZE_OF_THE_BIG_RING_LEGAL_DOMAIN,
        ControlKind::ScanPassOutcomeBecomesAPanic,
        run_profile,
        thread_count,
    );
    let place = |counter: Counter| {
        control_place(
            Domain::LegalBigRing,
            CONTROL_LARGEST_SIZE_OF_THE_BIG_RING_LEGAL_DOMAIN,
            "scan_pass_outcome_becomes_a_panic",
            counter,
        )
    };
    vec![
        reading_that_must_equal(
            criterion,
            place(Counter::RowsDiffer),
            tally.get(Counter::RowsDiffer),
            STATES_WITHIN_THE_CONTROL_BOUND_OF_THE_BIG_RING_LEGAL_DOMAIN,
        ),
        reading_that_must_equal(
            criterion,
            place(Counter::States),
            tally.get(Counter::States),
            STATES_WITHIN_THE_CONTROL_BOUND_OF_THE_BIG_RING_LEGAL_DOMAIN,
        ),
        reading_that_must_equal(
            criterion,
            place(Counter::TablesDiffer),
            tally.get(Counter::TablesDiffer),
            0,
        ),
    ]
}

/// 多盘三个域与各自对照的界、j\* − c ≥ 1 的态数（登记 r2 第七节）。
const MIRRORED_DOMAINS_WITH_THE_CONTROL_BOUND: [(Domain, usize, u64); 3] = [
    (
        Domain::MirroredBigRing,
        CONTROL_LARGEST_SIZE_OF_THE_MIRRORED_BIG_RING_DOMAIN,
        MIRRORED_BIG_RING_FARTHEST_AT_LEAST_ONE_WITHIN_THE_CONTROL_BOUND,
    ),
    (
        Domain::MirroredSmallRingSix,
        CONTROL_LARGEST_SIZE_OF_THE_MIRRORED_SMALL_RING_SIX_DOMAIN,
        MIRRORED_SMALL_RING_SIX_FARTHEST_AT_LEAST_ONE_WITHIN_THE_CONTROL_BOUND,
    ),
    (
        Domain::MirroredMonotoneBigRing,
        CONTROL_LARGEST_SIZE_OF_THE_MIRRORED_MONOTONE_DOMAIN,
        MIRRORED_MONOTONE_FARTHEST_AT_LEAST_ONE_WITHIN_THE_CONTROL_BOUND,
    ),
];

fn control_place(
    domain: Domain,
    largest_size: usize,
    implementation_label: &str,
    counter: Counter,
) -> String {
    format!(
        "domain={},max_n={largest_size},implementation={implementation_label},counter={}",
        domain.label(),
        counter.label()
    )
}

/// 多盘「任一份在即在」（登记 r2 5.5、第七节）：理想恢复但 j\* 只数 0 号盘那一份。M 域 N ≤ 3 的 J4 = 100、M6 域 N ≤ 4 = 996，
/// L 域 N ≤ 5（单盘，0 号盘那一份就是全部）= 0。
fn any_copy_control_readings(run_profile: RunProfile, thread_count: usize) -> Vec<ControlReading> {
    let control = ControlKind::IdealCountingOnlyTheFirstDeviceCopies;
    [
        (
            Domain::MirroredBigRing,
            CONTROL_LARGEST_SIZE_OF_THE_MIRRORED_BIG_RING_DOMAIN,
            MIRRORED_BIG_RING_STATES_WHERE_THE_FIRST_DEVICE_ALONE_GIVES_ANOTHER_FARTHEST_VERSION,
        ),
        (
            Domain::MirroredSmallRingSix,
            CONTROL_LARGEST_SIZE_OF_THE_MIRRORED_SMALL_RING_SIX_DOMAIN,
            MIRRORED_SMALL_RING_SIX_STATES_WHERE_THE_FIRST_DEVICE_ALONE_GIVES_ANOTHER_FARTHEST_VERSION,
        ),
        (
            Domain::LegalBigRing,
            CONTROL_LARGEST_SIZE_OF_THE_BIG_RING_LEGAL_DOMAIN,
            0,
        ),
    ]
    .into_iter()
    .map(|(domain, largest_size, expected)| {
        let tally = control_tally(domain, largest_size, control, run_profile, thread_count);
        reading_that_must_equal(
            Criterion::MultiDiskAnyCopyIsOnDisk,
            control_place(domain, largest_size, "ideal_counting_only_the_first_device_copies", Counter::J4Violations),
            tally.get(Counter::J4Violations),
            expected,
        )
    })
    .collect()
}

/// 原登记 J1 那条对照（施加过就返回时间线之外的根）在多盘三个域上：J1 违例数与 j\* − c ≥ 1 的态数都等于第七节的值。
fn mirrored_returned_root_control_readings(
    run_profile: RunProfile,
    thread_count: usize,
) -> Vec<ControlReading> {
    let mut readings = Vec::new();
    for (domain, largest_size, farthest_at_least_one) in MIRRORED_DOMAINS_WITH_THE_CONTROL_BOUND {
        let tally = control_tally(
            domain,
            largest_size,
            ControlKind::OffTimelineRootWheneverApplied,
            run_profile,
            thread_count,
        );
        for counter in [Counter::J1Violations, Counter::FarthestAtLeastOne] {
            readings.push(reading_that_must_equal(
                Criterion::MultiDiskReturnedRoot,
                control_place(
                    domain,
                    largest_size,
                    "off_timeline_root_whenever_applied",
                    counter,
                ),
                tally.get(counter),
                farthest_at_least_one,
            ));
        }
    }
    readings
}

/// 原登记 J4 那条对照（永远返回所选根）在多盘三个域上：J4 违例数等于第七节的值，J1、J2、J3 是 0。
fn mirrored_farthest_version_control_readings(
    run_profile: RunProfile,
    thread_count: usize,
) -> Vec<ControlReading> {
    let mut readings = Vec::new();
    for (domain, largest_size, farthest_at_least_one) in MIRRORED_DOMAINS_WITH_THE_CONTROL_BOUND {
        let tally = control_tally(
            domain,
            largest_size,
            ControlKind::AlwaysReturnsTheChosenRoot,
            run_profile,
            thread_count,
        );
        for (counter, expected) in [
            (Counter::J4Violations, farthest_at_least_one),
            (Counter::J1Violations, 0),
            (Counter::J2Violations, 0),
            (Counter::J3Violations, 0),
        ] {
            readings.push(reading_that_must_equal(
                Criterion::MultiDiskFarthestVersion,
                control_place(
                    domain,
                    largest_size,
                    "always_returns_the_chosen_root",
                    counter,
                ),
                tally.get(counter),
                expected,
            ));
        }
    }
    readings
}

/// MC 宽对上 J5a、J5b 的对照（登记 r2 第十二节修订 3 补的；原登记 5.4 J5 那条假结局搬到多盘）：记录每条都有一份在盘上时返回所选根，
/// 其余照理想恢复；MC 域 Nc ≤ 2 宽对上今天那两列都大于 0。
fn mirrored_monotone_control_readings(
    run_profile: RunProfile,
    thread_count: usize,
) -> Vec<ControlReading> {
    let tally = control_tally(
        Domain::MirroredMonotoneBigRing,
        CONTROL_LARGEST_SIZE_OF_THE_MIRRORED_MONOTONE_DOMAIN,
        ControlKind::ReturnsTheChosenRootWhenEveryRecordIsOnDisk,
        run_profile,
        thread_count,
    );
    [PairColumn::TodayAppliedSetViolations, PairColumn::TodayLandedVersionViolations]
        .into_iter()
        .map(|column| {
            reading_that_must_be_positive(
                Criterion::MultiDiskAppliedSetAndLandedVersion,
                format!(
                    "domain=MC,max_n={CONTROL_LARGEST_SIZE_OF_THE_MIRRORED_MONOTONE_DOMAIN},implementation=returns_the_chosen_root_when_every_record_is_on_disk,set=wide,column={}",
                    column.label()
                ),
                tally.pair(PairSet::Wide, column),
            )
        })
        .collect()
}

/// J6Qn 的三条对照（登记 r2 5.5；第七节把它们钉成等式：L 域 N ≤ 5 两条各 3 110，E1 全界原 J6 70、J6Qn 0）。
fn j6qn_control_readings(run_profile: RunProfile, thread_count: usize) -> Vec<ControlReading> {
    let criterion = Criterion::J6QnFixedPoint;
    let sum_of_both_premises = |tally: &Tally, premise_holds: Counter, premise_fails: Counter| {
        tally.get(premise_holds) + tally.get(premise_fails)
    };
    let mut readings = Vec::new();
    for (control, implementation_label) in [
        (
            ControlKind::SecondCallReportsMoreThanOneLastRecordFlag,
            "second_call_reports_more_than_one_last_record_flag",
        ),
        (
            ControlKind::SecondCallReportsTheChosenRootTxgHasNoSuccessor,
            "second_call_reports_chosen_root_txg_has_no_successor",
        ),
    ] {
        let tally = control_tally(
            Domain::LegalBigRing,
            CONTROL_LARGEST_SIZE_OF_THE_BIG_RING_LEGAL_DOMAIN,
            control,
            run_profile,
            thread_count,
        );
        let place = format!(
            "domain=L,max_n={CONTROL_LARGEST_SIZE_OF_THE_BIG_RING_LEGAL_DOMAIN},implementation={implementation_label}"
        );
        readings.push(reading_that_must_equal(
            criterion,
            format!("{place},counter=J6Qn_premise_holds_plus_J6Qn_premise_fails"),
            sum_of_both_premises(
                &tally,
                Counter::J6QnPremiseHoldsViolations,
                Counter::J6QnPremiseFailsViolations,
            ),
            STATES_WITHIN_THE_CONTROL_BOUND_OF_THE_BIG_RING_LEGAL_DOMAIN,
        ));
        readings.push(reading_that_must_equal(
            criterion,
            format!("{place},counter=j6qn_premise_holds_judged_plus_j6qn_premise_fails_judged"),
            sum_of_both_premises(
                &tally,
                Counter::J6QnPremiseHoldsJudged,
                Counter::J6QnPremiseFailsJudged,
            ),
            STATES_WITHIN_THE_CONTROL_BOUND_OF_THE_BIG_RING_LEGAL_DOMAIN,
        ));
        if control == ControlKind::SecondCallReportsTheChosenRootTxgHasNoSuccessor {
            readings.push(reading_that_must_equal(
                criterion,
                format!("{place},counter=J6_premise_holds_plus_J6_premise_fails"),
                sum_of_both_premises(
                    &tally,
                    Counter::J6PremiseHoldsViolations,
                    Counter::J6PremiseFailsViolations,
                ),
                STATES_WITHIN_THE_CONTROL_BOUND_OF_THE_BIG_RING_LEGAL_DOMAIN,
            ));
        }
    }
    let e1_largest_size = BoundsTier::SegmentOne
        .largest_size(Domain::TxgOneBelowTheLargest)
        .expect("第一段跑 E1");
    let tally = control_tally(
        Domain::TxgOneBelowTheLargest,
        e1_largest_size,
        ControlKind::IdealFirstCallThenTodaysSecondCall,
        run_profile,
        thread_count,
    );
    let place = format!(
        "domain=E1,max_n={e1_largest_size},implementation=ideal_first_call_then_todays_second_call"
    );
    readings.push(reading_that_must_equal(
        criterion,
        format!("{place},counter=J6_premise_holds_plus_J6_premise_fails"),
        sum_of_both_premises(
            &tally,
            Counter::J6PremiseHoldsViolations,
            Counter::J6PremiseFailsViolations,
        ),
        E1_STATES_WHOSE_FARTHEST_VERSION_TXG_IS_THE_LARGEST,
    ));
    readings.push(reading_that_must_equal(
        criterion,
        format!("{place},counter=J6Qn_premise_holds_plus_J6Qn_premise_fails"),
        sum_of_both_premises(
            &tally,
            Counter::J6QnPremiseHoldsViolations,
            Counter::J6QnPremiseFailsViolations,
        ),
        0,
    ));
    readings
}

/// J7a 一格的对照（登记 r2 5.5、第七节）：理想恢复、施加过时把这个字段换成落到的那一版的。L 域 N ≤ 5 上这一格 = 762、另两格 = 0；
/// 同一个对照在 r1 档生成器上这一格 = 0（判别力自证：看得见是生成器换了的结果）。
fn j7a_control_readings(
    criterion: Criterion,
    field: FieldOfTheRebuiltRoot,
    run_profile: RunProfile,
    thread_count: usize,
) -> Vec<ControlReading> {
    let control = ControlKind::IdealWithAFieldOfTheLandedVersion(field);
    let place = format!(
        "domain=L,max_n={CONTROL_LARGEST_SIZE_OF_THE_BIG_RING_LEGAL_DOMAIN},implementation=ideal_with_the_landed_versions_{}",
        field.label()
    );
    let tally = control_tally(
        Domain::LegalBigRing,
        CONTROL_LARGEST_SIZE_OF_THE_BIG_RING_LEGAL_DOMAIN,
        control,
        run_profile,
        thread_count,
    );
    let mut readings = Vec::new();
    for judged_field in FieldOfTheRebuiltRoot::ALL {
        let counter = judged_field.violations_counter();
        readings.push(reading_that_must_equal(
            criterion,
            format!("{place},counter={}", counter.label()),
            tally.get(counter),
            if judged_field == field {
                FARTHEST_AT_LEAST_ONE_STATES_WITHIN_THE_CONTROL_BOUND
            } else {
                0
            },
        ));
    }
    let r1_generator_tally = control_tally(
        Domain::LegalBigRing,
        CONTROL_LARGEST_SIZE_OF_THE_BIG_RING_LEGAL_DOMAIN,
        control,
        RunProfile::R1_GENERATOR_WITH_R2_CRITERIA,
        thread_count,
    );
    readings.push(reading_that_must_equal(
        criterion,
        format!(
            "{place},generator=r1,counter={}",
            field.violations_counter().label()
        ),
        r1_generator_tally.get(field.violations_counter()),
        0,
    ));
    readings
}

/// r5 档跑的是哪一种卸载记号奇偶（J8 的对照只在 r5 档跑，r5 档的生成器随版本变）。
fn marker_parity_of_the_r5_profile(run_profile: RunProfile) -> MarkerParity {
    match run_profile.generator {
        FieldsBeyondTheProjectionGenerator::VaryWithTheVersion(marker_parity) => marker_parity,
        FieldsBeyondTheProjectionGenerator::SameForEveryVersion => {
            unreachable!(
                "J8 的对照只在 r5 档跑（Criterion::is_judged_under），r5 档的生成器随版本变"
            )
        }
    }
}

/// 一条 J8 对照（登记 r5 5.4）在 L 域 N ≤ 5 上逐格钉值、在 r1 档那种生成器上逐格必须是 0、反奇偶另在 C 域 Nc ≤ 3 上逐格必须大于 0。
/// `pinned_cells` 是 (量, r5 生成器上的钉值)；`cells_that_must_be_zero_on_the_r1_generator` 与 `cells_above_zero_on_the_monotone_domain_under_odd`
/// 各列一组量。
#[allow(
    clippy::too_many_arguments,
    reason = "每个参数都是这一条对照要用的一样东西，凑不成一个带不变量的概念"
)]
fn c8_control_readings(
    criterion: Criterion,
    control_label: &str,
    control: ControlKind,
    pinned_cells: &[(Counter, u64)],
    cells_that_must_be_zero_on_the_r1_generator: &[Counter],
    cells_above_zero_on_the_monotone_domain_under_odd: &[Counter],
    run_profile: RunProfile,
    thread_count: usize,
) -> Vec<ControlReading> {
    let marker_parity = marker_parity_of_the_r5_profile(run_profile);
    let place = format!(
        "domain=L,max_n={CONTROL_LARGEST_SIZE_OF_THE_BIG_RING_LEGAL_DOMAIN},implementation={control_label}"
    );
    let big_ring_tally = control_tally(
        Domain::LegalBigRing,
        CONTROL_LARGEST_SIZE_OF_THE_BIG_RING_LEGAL_DOMAIN,
        control,
        run_profile,
        thread_count,
    );
    let mut readings: Vec<ControlReading> = pinned_cells
        .iter()
        .map(|(counter, pinned_value)| {
            reading_that_must_equal(
                criterion,
                format!(
                    "{place},counter={},marker_parity={}",
                    counter.label(),
                    marker_parity.label()
                ),
                big_ring_tally.get(*counter),
                *pinned_value,
            )
        })
        .collect();
    let r1_generator_tally = control_tally(
        Domain::LegalBigRing,
        CONTROL_LARGEST_SIZE_OF_THE_BIG_RING_LEGAL_DOMAIN,
        control,
        RunProfile::R1_GENERATOR_WITH_R5_CRITERIA,
        thread_count,
    );
    for counter in cells_that_must_be_zero_on_the_r1_generator {
        readings.push(reading_that_must_equal(
            criterion,
            format!("{place},generator=r1,counter={}", counter.label()),
            r1_generator_tally.get(*counter),
            0,
        ));
    }
    match marker_parity {
        MarkerParity::EvenVersionsWrittenByTheUnmountSequence => {}
        MarkerParity::OddVersionsWrittenByTheUnmountSequence => {
            let monotone_tally = control_tally(
                Domain::MonotoneBigRing,
                CONTROL_LARGEST_SIZE_OF_THE_MONOTONE_BIG_RING_DOMAIN,
                control,
                run_profile,
                thread_count,
            );
            for counter in cells_above_zero_on_the_monotone_domain_under_odd {
                readings.push(reading_that_must_be_positive(
                    criterion,
                    format!(
                        "domain=C,max_n={CONTROL_LARGEST_SIZE_OF_THE_MONOTONE_BIG_RING_DOMAIN},implementation={control_label},counter={},marker_parity=odd",
                        counter.label()
                    ),
                    monotone_tally.get(*counter),
                ));
            }
        }
    }
    readings
}

/// C8a（登记 r5 5.4、第七节乙类）：理想恢复、施加过时三项都抄所选根的。L 域 N ≤ 5 上 J8a 实例表指针 762、分配记录树根 762
/// （第 j 版的两个槽与第 0 版的不同 ⟺ j ≥ 1）、卸载记号 620（第 j 版与第 0 版的记号不同 ⟺ j 为奇数，两种奇偶都如此），J8b、J8c 0。
fn c8a_control_readings(run_profile: RunProfile, thread_count: usize) -> Vec<ControlReading> {
    c8_control_readings(
        Criterion::J8aFieldsFromTheNewRootSegment,
        "C8a",
        ControlKind::IdealCopyingTheChosenRecordFieldsWhenApplied,
        &[
            (
                Counter::J8aInstanceTableViolations,
                FARTHEST_AT_LEAST_ONE_STATES_WITHIN_THE_CONTROL_BOUND,
            ),
            (
                Counter::J8aAllocationRecordTreeRootViolations,
                FARTHEST_AT_LEAST_ONE_STATES_WITHIN_THE_CONTROL_BOUND,
            ),
            (
                Counter::J8aUnmountMarkerViolations,
                FARTHEST_IS_ODD_STATES_WITHIN_THE_CONTROL_BOUND,
            ),
            (Counter::J8bViolations, 0),
            (Counter::J8cViolations, 0),
        ],
        &[
            Counter::J8aInstanceTableViolations,
            Counter::J8aAllocationRecordTreeRootViolations,
            Counter::J8aUnmountMarkerViolations,
        ],
        &[
            Counter::J8aInstanceTableViolations,
            Counter::J8aAllocationRecordTreeRootViolations,
        ],
        run_profile,
        thread_count,
    )
}

/// C8b（登记 r5 5.4、第七节乙类）：理想恢复、施加过时卸载记号一律写 `NotWritten…`。落到的那一版是 `Written…` ⟺ `even` 下 j 为偶数、
/// `odd` 下 j 为奇数，施加过的态里 j ≥ 1：L 域 N ≤ 5 上 `even` 142、`odd` 620。
fn c8b_control_readings(run_profile: RunProfile, thread_count: usize) -> Vec<ControlReading> {
    let pinned_unmount_marker_violations = match marker_parity_of_the_r5_profile(run_profile) {
        MarkerParity::EvenVersionsWrittenByTheUnmountSequence => {
            FARTHEST_IS_EVEN_AND_AT_LEAST_TWO_STATES_WITHIN_THE_CONTROL_BOUND
        }
        MarkerParity::OddVersionsWrittenByTheUnmountSequence => {
            FARTHEST_IS_ODD_STATES_WITHIN_THE_CONTROL_BOUND
        }
    };
    c8_control_readings(
        Criterion::J8aUnmountMarkerFromTheNewRootSegment,
        "C8b",
        ControlKind::IdealWritingOneUnmountMarkerWhenApplied,
        &[(
            Counter::J8aUnmountMarkerViolations,
            pinned_unmount_marker_violations,
        )],
        &[Counter::J8aUnmountMarkerViolations],
        &[],
        run_profile,
        thread_count,
    )
}

/// C8c（登记 r5 5.4、第七节乙类）：理想恢复、施加过时 fsid 取落到的那一版 R_j 的。fsid 末字节取 j，第 j 版与第 0 版不同 ⟺ j ≥ 1：
/// L 域 N ≤ 5 上 J8b 762，J8a 三格 0。
fn c8c_control_readings(run_profile: RunProfile, thread_count: usize) -> Vec<ControlReading> {
    c8_control_readings(
        Criterion::J8bFilesystemIdentifierOfTheChosenRoot,
        "C8c",
        ControlKind::IdealWithAFieldOfTheLandedVersion(FieldOfTheRebuiltRoot::FilesystemIdentifier),
        &[
            (
                Counter::J8bViolations,
                FARTHEST_AT_LEAST_ONE_STATES_WITHIN_THE_CONTROL_BOUND,
            ),
            (Counter::J8aInstanceTableViolations, 0),
            (Counter::J8aAllocationRecordTreeRootViolations, 0),
            (Counter::J8aUnmountMarkerViolations, 0),
        ],
        &[Counter::J8bViolations],
        &[Counter::J8bViolations],
        run_profile,
        thread_count,
    )
}

/// C8d（登记 r5 5.4、第七节乙类）：理想恢复、一条没施加时卸载记号写成 `NotWritten…`。一条没施加 ⟺ j* = 0；R_0 的记号 `even` 下
/// `Written…`、`odd` 下 `NotWritten…`：L 域 N ≤ 5 上 `even` 2 348、`odd` 0（`odd` 另在 C 域 Nc ≤ 3 上大于 0）。
fn c8d_control_readings(run_profile: RunProfile, thread_count: usize) -> Vec<ControlReading> {
    let pinned_nothing_applied_violations = match marker_parity_of_the_r5_profile(run_profile) {
        MarkerParity::EvenVersionsWrittenByTheUnmountSequence => {
            FARTHEST_IS_THE_CHOSEN_VERSION_STATES_WITHIN_THE_CONTROL_BOUND
        }
        MarkerParity::OddVersionsWrittenByTheUnmountSequence => 0,
    };
    c8_control_readings(
        Criterion::J8cNothingAppliedReturnsTheChosenRoot,
        "C8d",
        ControlKind::IdealWritingNotWrittenWhenNothingIsApplied,
        &[(Counter::J8cViolations, pinned_nothing_applied_violations)],
        &[Counter::J8cViolations],
        &[Counter::J8cViolations],
        run_profile,
        thread_count,
    )
}

/// J7b、J7c 的对照（登记 r2 5.5、第七节、第八节奇偶的判别力自证）。偶数版带记号（R_0 是 `Written…`）：L 域 N ≤ 5 上
/// 等于 `count_when_the_first_version_is_written`，r1 档生成器上 0。奇数版带记号（R_0 是 `NotWritten…`）：L 域 0（假结局与期望同值），
/// C 域 Nc ≤ 3 大于 0（所选根是第几版取决于哪些根槽落了，两种记号的所选根都有）。
fn unmount_marker_control_readings(
    criterion: Criterion,
    control: ControlKind,
    counter: Counter,
    count_when_the_first_version_is_written: u64,
    run_profile: RunProfile,
    thread_count: usize,
) -> Vec<ControlReading> {
    let big_ring_place = format!(
        "domain=L,max_n={CONTROL_LARGEST_SIZE_OF_THE_BIG_RING_LEGAL_DOMAIN},implementation={},counter={}",
        criterion.label(),
        counter.label()
    );
    let big_ring_tally = |profile: RunProfile| {
        control_tally(
            Domain::LegalBigRing,
            CONTROL_LARGEST_SIZE_OF_THE_BIG_RING_LEGAL_DOMAIN,
            control,
            profile,
            thread_count,
        )
        .get(counter)
    };
    let r1_generator_reading = reading_that_must_equal(
        criterion,
        format!("{big_ring_place},generator=r1"),
        big_ring_tally(RunProfile::R1_GENERATOR_WITH_R2_CRITERIA),
        0,
    );
    match run_profile.generator {
        FieldsBeyondTheProjectionGenerator::VaryWithTheVersion(
            MarkerParity::EvenVersionsWrittenByTheUnmountSequence,
        ) => vec![
            reading_that_must_equal(
                criterion,
                format!("{big_ring_place},marker_parity=even"),
                big_ring_tally(run_profile),
                count_when_the_first_version_is_written,
            ),
            r1_generator_reading,
        ],
        FieldsBeyondTheProjectionGenerator::VaryWithTheVersion(
            MarkerParity::OddVersionsWrittenByTheUnmountSequence,
        ) => {
            let monotone_tally = control_tally(
                Domain::MonotoneBigRing,
                CONTROL_LARGEST_SIZE_OF_THE_MONOTONE_BIG_RING_DOMAIN,
                control,
                run_profile,
                thread_count,
            );
            vec![
                reading_that_must_equal(
                    criterion,
                    format!("{big_ring_place},marker_parity=odd"),
                    big_ring_tally(run_profile),
                    0,
                ),
                reading_that_must_be_positive(
                    criterion,
                    format!(
                        "domain=C,max_n={CONTROL_LARGEST_SIZE_OF_THE_MONOTONE_BIG_RING_DOMAIN},implementation={},counter={},marker_parity=odd",
                        criterion.label(),
                        counter.label()
                    ),
                    monotone_tally.get(counter),
                ),
                r1_generator_reading,
            ]
        }
        FieldsBeyondTheProjectionGenerator::SameForEveryVersion => {
            unreachable!(
                "J7 的对照只在 r2 档跑（Criterion::is_judged_under），r2 档的生成器随版本变"
            )
        }
    }
}

/// 阳性对照那段循环：遍历判据表的每一行（只跑选中的那一族、这一档判的判据）。
fn positive_control_readings(
    families: &[Family],
    run_profile: RunProfile,
    thread_count: usize,
) -> Vec<ControlReading> {
    let mut readings = Vec::new();
    for criterion in Criterion::ALL {
        if !families.contains(&criterion.family()) && !criterion.is_judged_in_every_family() {
            continue;
        }
        if !criterion.is_judged_under(run_profile) {
            continue;
        }
        readings.extend(control_readings_of(*criterion, run_profile, thread_count));
    }
    readings
}

// ── 产物行

/// 逐行打 `E7RESULT`，数着打了几行（收尾行 `name=done emitted=N` 把自己也算进去）。
struct Emitter {
    emitted: usize,
}

impl Emitter {
    fn result(&mut self, text: &str) {
        println!("E7RESULT {text}");
        self.emitted += 1;
    }

    fn done(&mut self) {
        println!("E7RESULT name=done emitted={}", self.emitted + 1);
        self.emitted += 1;
    }
}

/// 跑完要交代的两类事：作废（登记第十一节 V 条）与停机（H 条、F11）。有一条，进程以非 0 退出。
#[derive(Default)]
struct RunFindings {
    void_conditions: Vec<String>,
    halt_conditions: Vec<String>,
}

fn witness_field(tally: &Tally, counter: Counter) -> String {
    tally.witness(counter).unwrap_or("none").to_string()
}

/// 落后类别的五行（登记 r2 SQ9、SQ17）：相等、落后（两个方向之和）、互不包含、0 号盘落后、1 号盘落后；每行列出
/// [`LAGGING_CLASS_COLUMNS`] 各量。`row_name` 是 `lagging_class`（每个 (域, N)）或 `verdict`（域的合计）。
fn lagging_class_rows(
    row_name: &str,
    place: &str,
    tally: &Tally,
    run_profile: RunProfile,
) -> Vec<String> {
    let shown_columns = lagging_class_columns_shown_under(run_profile);
    let groups: [(&str, &[LaggingClass]); 5] = [
        (LaggingClass::Equal.label(), &[LaggingClass::Equal]),
        (
            "lagging",
            &[
                LaggingClass::FirstDeviceBehind,
                LaggingClass::SecondDeviceBehind,
            ],
        ),
        (LaggingClass::Crossed.label(), &[LaggingClass::Crossed]),
        (
            LaggingClass::FirstDeviceBehind.label(),
            &[LaggingClass::FirstDeviceBehind],
        ),
        (
            LaggingClass::SecondDeviceBehind.label(),
            &[LaggingClass::SecondDeviceBehind],
        ),
    ];
    groups
        .iter()
        .map(|(group_label, classes)| {
            let columns: Vec<String> = shown_columns
                .iter()
                .map(|counter| {
                    let count: u64 = classes
                        .iter()
                        .map(|class| tally.lagging_class_count(*class, *counter))
                        .sum();
                    format!("{}={count}", counter.label())
                })
                .collect();
            format!(
                "name={row_name} {place} lagging_class={group_label} {}",
                columns.join(" ")
            )
        })
        .collect()
}

/// 一个 (域, N) 的全部行：状态数与见证、每条判据一行违例数、S 自己的对照、C 域的状态对、对抗域的 A-互。
fn emit_the_rows_of_a_domain_size(
    emitter: &mut Emitter,
    domain: Domain,
    size: usize,
    tally: &Tally,
    run_profile: RunProfile,
) {
    let place = format!(
        "domain={} ring_slots={} n={size}",
        domain.label(),
        domain.ring().ring_slots
    );
    emitter.result(&format!(
        "name=states {place} states={} panicked={} failed={} applied_ge1={} applied_ge2={} farthest_ge1={} farthest_ge2={} in_flight_limit={} above_water_max={} above_water_over_limit={} above_water_at_limit={} chosen_root_is_a_later_version={} off_timeline_landings={} states_holding_a_record_rejected_by_parse={}",
        tally.get(Counter::States),
        tally.get(Counter::Panicked),
        tally.get(Counter::Failed),
        tally.get(Counter::AppliedAtLeastOne),
        tally.get(Counter::AppliedAtLeastTwo),
        tally.get(Counter::FarthestAtLeastOne),
        tally.get(Counter::FarthestAtLeastTwo),
        domain.ring().local_in_flight_limit(),
        tally.above_water_maximum,
        tally.get(Counter::AboveWaterOverLimit),
        tally.get(Counter::AboveWaterAtLimit),
        tally.get(Counter::ChosenRootIsALaterVersion),
        tally.get(Counter::OffTimelineLandings),
        tally.get(Counter::StatesHoldingARecordRejectedByParse),
    ));
    let mut violations_row = |criterion: &str, violations: Counter, judged: u64| {
        emitter.result(&format!(
            "name=violations {place} criterion={criterion} violations={} judged={judged} witness={}",
            tally.get(violations),
            witness_field(tally, violations)
        ));
    };
    let states = tally.get(Counter::States);
    match domain.family() {
        Family::LegalHistory | Family::MultiDisk => {
            violations_row("J0", Counter::Panicked, states);
            violations_row("J1", Counter::J1Violations, states);
            violations_row("J2", Counter::J2Violations, states);
            violations_row("J3", Counter::J3Violations, states);
            violations_row("J4", Counter::J4Violations, states);
        }
        Family::Adversarial => {
            violations_row("A0", Counter::Panicked, states);
            violations_row("AS_whole_table", Counter::AsWholeTableViolations, states);
            violations_row(
                "AS_consecutive_gap_continues",
                Counter::AsGapContinuesViolations,
                states,
            );
            violations_row(
                "AS_consecutive_gap_stops",
                Counter::AsGapStopsViolations,
                states,
            );
            violations_row(
                "A_structure",
                Counter::StructureViolations,
                states - tally.get(Counter::Failed) - tally.get(Counter::Panicked),
            );
            violations_row(
                "A_limit",
                Counter::LimitViolations,
                states - tally.get(Counter::Failed) - tally.get(Counter::Panicked),
            );
        }
    }
    violations_row(
        "J6_premise_holds",
        Counter::J6PremiseHoldsViolations,
        tally.get(Counter::J6PremiseHoldsJudged),
    );
    violations_row(
        "J6_premise_fails",
        Counter::J6PremiseFailsViolations,
        tally.get(Counter::J6PremiseFailsJudged),
    );
    if run_profile.judges_j6qn() {
        violations_row(
            "J6Qn_premise_holds",
            Counter::J6QnPremiseHoldsViolations,
            tally.get(Counter::J6QnPremiseHoldsJudged),
        );
        violations_row(
            "J6Qn_premise_fails",
            Counter::J6QnPremiseFailsViolations,
            tally.get(Counter::J6QnPremiseFailsJudged),
        );
    }
    if run_profile.judges_j7() {
        for field in FieldOfTheRebuiltRoot::ALL {
            violations_row(
                field.violations_counter().label(),
                field.violations_counter(),
                tally.get(Counter::J7Judged),
            );
        }
        violations_row("J7b", Counter::J7bViolations, tally.get(Counter::J7bJudged));
        violations_row("J7c", Counter::J7cViolations, tally.get(Counter::J7cJudged));
    }
    // r5 档（登记 r5 5.3）：J7 五行换成 J8 五行与「无出处」一行；「无出处」只报，它的 judged 是投影不等于所选根的态数。
    if run_profile.judges_j8() {
        for counter in [
            Counter::J8aInstanceTableViolations,
            Counter::J8aAllocationRecordTreeRootViolations,
            Counter::J8aUnmountMarkerViolations,
        ] {
            violations_row(counter.label(), counter, tally.get(Counter::J8aJudged));
        }
        violations_row("J8b", Counter::J8bViolations, tally.get(Counter::J8Judged));
        violations_row("J8c", Counter::J8cViolations, tally.get(Counter::J8cJudged));
        violations_row(
            "J8_without_a_source",
            Counter::J8WithoutASource,
            tally.get(Counter::J8aJudged) + tally.get(Counter::J8WithoutASource),
        );
    }
    if tally.get(Counter::Ia1Judged) > 0 {
        violations_row("IA1", Counter::Ia1Violations, tally.get(Counter::Ia1Judged));
    }
    if tally.get(Counter::Ia2Judged) > 0 {
        violations_row("IA2", Counter::Ia2Violations, tally.get(Counter::Ia2Judged));
    }
    emitter.result(&format!(
        "name=fixed_point {place} first_call_failed={} third_call_equals_second={} third_call_differs_or_is_missing={}",
        tally.get(Counter::J6FirstCallFailed),
        tally.get(Counter::J6ThirdCallEqualsSecond),
        tally.get(Counter::J6ThirdCallDiffersOrIsMissing),
    ));
    if run_profile.judges_j6qn() {
        emitter.result(&format!(
            "name=first_returned_root_txg {place} first_returned_root_txg_is_the_largest={}",
            tally.get(Counter::FirstReturnedRootTxgIsTheLargest),
        ));
    }
    if domain.specification_self_check() == SpecificationSelfCheck::Judged {
        for counter in [
            Counter::SpecificationWholeTableMismatch,
            Counter::SpecificationGapContinuesMismatch,
            Counter::SpecificationGapStopsMismatch,
        ] {
            emitter.result(&format!(
                "name=specification_self_check {place} reading={} mismatches={} witness={}",
                counter.label(),
                tally.get(counter),
                witness_field(tally, counter)
            ));
        }
    }
    if !pinned_monotone_counts(domain).is_empty() {
        emitter.result(&format!(
            "name=legal_crash_states {place} cut_own={} cut_with={} pair_states_without_an_interval={}",
            tally.get(Counter::LegalCrashStatesWhenTheRootWriteIsItsOwnSegment),
            tally.get(Counter::LegalCrashStatesWhenTheRootWriteSharesTheRecordSegment),
            tally.get(Counter::PairStatesWithoutAnInterval),
        ));
        for set in PairSet::ALL {
            emitter.result(&format!(
                "name=pairs {place} set={} pairs={} today_j5a={} today_j5b={} ideal_j5a={} ideal_j5b={}",
                set.label(),
                tally.pair(*set, PairColumn::Pairs),
                tally.pair(*set, PairColumn::TodayAppliedSetViolations),
                tally.pair(*set, PairColumn::TodayLandedVersionViolations),
                tally.pair(*set, PairColumn::IdealAppliedSetViolations),
                tally.pair(*set, PairColumn::IdealLandedVersionViolations),
            ));
        }
    }
    if !pinned_mirrored_counts(domain).is_empty() {
        for row in lagging_class_rows("lagging_class", &place, tally, run_profile) {
            emitter.result(&row);
        }
    }
    if !pinned_mirrored_monotone_counts(domain).is_empty() {
        emitter.result(&format!(
            "name=pair_states {place} pair_states_without_an_interval={}",
            tally.get(Counter::PairStatesWithoutAnInterval),
        ));
        emitter.result(&format!(
            "name=pairs {place} set={} pairs={} today_j5a={} today_j5b={} ideal_j5a={} ideal_j5b={}",
            PairSet::Wide.label(),
            tally.pair(PairSet::Wide, PairColumn::Pairs),
            tally.pair(PairSet::Wide, PairColumn::TodayAppliedSetViolations),
            tally.pair(PairSet::Wide, PairColumn::TodayLandedVersionViolations),
            tally.pair(PairSet::Wide, PairColumn::IdealAppliedSetViolations),
            tally.pair(PairSet::Wide, PairColumn::IdealLandedVersionViolations),
        ));
    }
    if run_profile.compares_the_two_record_tables() {
        let reported_order_fields = match run_profile.reported_disk_order {
            ReportedDiskOrder::FirstDeviceBeforeSecond => String::new(),
            ReportedDiskOrder::SecondDeviceBeforeFirst => format!(
                " tables_differ_in_reported_order={} rows_differ_in_reported_order={} witness_in_reported_order={} rows_witness_in_reported_order={}",
                tally.get(Counter::TablesDifferInReportedOrder),
                tally.get(Counter::RowsDifferInReportedOrder),
                witness_field(tally, Counter::TablesDifferInReportedOrder),
                witness_field(tally, Counter::RowsDifferInReportedOrder),
            ),
        };
        emitter.result(&format!(
            "name=table_comparison {place} states={} tables_differ={} rows_differ={} direct_table_holds_a_record={} fully_landed_scan_table_size_checked={} witness={} rows_witness={}{reported_order_fields}",
            tally.get(Counter::States),
            tally.get(Counter::TablesDiffer),
            tally.get(Counter::RowsDiffer),
            tally.get(Counter::DirectTableHoldsARecord),
            tally.get(Counter::FullyLandedScanTableSizeChecked),
            witness_field(tally, Counter::TablesDiffer),
            witness_field(tally, Counter::RowsDiffer),
        ));
    }
    for row in rows_of_the_i812_split(run_profile, domain, &place, tally) {
        emitter.result(&row);
    }
    if domain.family() == Family::Adversarial {
        emitter.result(&format!(
            "name=specification {place} expects_failure={}",
            tally.get(Counter::SpecificationExpectsFailure)
        ));
        for counter in [
            Counter::MutualWholeTableAndGapContinues,
            Counter::MutualWholeTableAndGapStops,
            Counter::MutualGapContinuesAndGapStops,
        ] {
            emitter.result(&format!(
                "name=mutual {place} readings={} differing={} witness={}",
                counter.label(),
                tally.get(counter),
                witness_field(tally, counter)
            ));
        }
    }
}

/// 一个域扫完之后的判决行：各判据在这个域全部 N 上的合计。也在这里记作废与停机。
fn emit_the_verdict_of_a_domain(
    emitter: &mut Emitter,
    findings: &mut RunFindings,
    domain: Domain,
    largest_size: usize,
    total: &Tally,
    run_profile: RunProfile,
) {
    let place = format!(
        "domain={} ring_slots={} smallest_n={} largest_n={largest_size}",
        domain.label(),
        domain.ring().ring_slots,
        domain.smallest_size()
    );
    // 域非空见证（登记 Q12、作废条款 V4）：实现施加过至少一次发布的态数必须大于 0，E2 除外。
    let domain_is_nonempty =
        total.get(Counter::AppliedAtLeastOne) > 0 || domain == Domain::TxgAtTheLargest;
    if !domain_is_nonempty {
        findings.void_conditions.push(format!(
            "V4 {} 域没有一个态施加过发布，这个域的 0 违例不算数",
            domain.label()
        ));
    }
    let specification_mismatches = total.get(Counter::SpecificationWholeTableMismatch)
        + total.get(Counter::SpecificationGapContinuesMismatch)
        + total.get(Counter::SpecificationGapStopsMismatch);
    if specification_mismatches > 0 {
        findings.void_conditions.push(format!(
            "V3 {} 域里 S 的期望落点不等于 j* 的态数（三种读法合计）{specification_mismatches}",
            domain.label()
        ));
    }
    match domain.family() {
        Family::LegalHistory | Family::MultiDisk => {
            // 登记 r2 第七节甲类、SF9：M、M6、MC 里今天的实现第一次调用报错的态数是 0；不符停下，两边都查。
            if domain.family() == Family::MultiDisk && total.get(Counter::Failed) > 0 {
                findings.halt_conditions.push(format!(
                    "SF9 {} 域里今天的实现第一次调用报错的态数 {}（登记 r2 第七节甲类锚点是 0）",
                    domain.label(),
                    total.get(Counter::Failed)
                ));
            }
            // 甲类锚点（登记第七节）：L、L 小环、U 开着验证、X、E1 里原实现报错的态数是 0；不符走 F11，停下两边都查。
            let failed_states_are_pinned_to_zero =
                domain.specification_self_check() == SpecificationSelfCheck::Judged;
            if failed_states_are_pinned_to_zero && total.get(Counter::Failed) > 0 {
                findings.halt_conditions.push(format!(
                    "F11 {} 域里原实现报错的态数 {}（登记第七节甲类锚点是 0）",
                    domain.label(),
                    total.get(Counter::Failed)
                ));
            }
            emitter.result(&format!(
                "name=verdict {place} states={} j0_violations={} failed_states={} j1_violations={} j2_violations={} j3_violations={} j4_violations={} j6_premise_holds_violations={} j6_premise_fails_violations={} j6_premise_fails_judged={} ia1_violations={} ia2_violations={} specification_self_check_mismatches={specification_mismatches} applied_ge1={} applied_ge2={} domain_is_nonempty={domain_is_nonempty}",
                total.get(Counter::States),
                total.get(Counter::Panicked),
                total.get(Counter::Failed),
                total.get(Counter::J1Violations),
                total.get(Counter::J2Violations),
                total.get(Counter::J3Violations),
                total.get(Counter::J4Violations),
                total.get(Counter::J6PremiseHoldsViolations),
                total.get(Counter::J6PremiseFailsViolations),
                total.get(Counter::J6PremiseFailsJudged),
                total.get(Counter::Ia1Violations),
                total.get(Counter::Ia2Violations),
                total.get(Counter::AppliedAtLeastOne),
                total.get(Counter::AppliedAtLeastTwo),
            ));
            if !pinned_monotone_counts(domain).is_empty() {
                for set in PairSet::ALL {
                    emitter.result(&format!(
                        "name=verdict {place} set={} pairs={} today_j5a_violations={} today_j5b_violations={} ideal_j5a_violations={} ideal_j5b_violations={}",
                        set.label(),
                        total.pair(*set, PairColumn::Pairs),
                        total.pair(*set, PairColumn::TodayAppliedSetViolations),
                        total.pair(*set, PairColumn::TodayLandedVersionViolations),
                        total.pair(*set, PairColumn::IdealAppliedSetViolations),
                        total.pair(*set, PairColumn::IdealLandedVersionViolations),
                    ));
                }
            }
            if !pinned_mirrored_counts(domain).is_empty() {
                for row in lagging_class_rows("verdict", &place, total, run_profile) {
                    emitter.result(&row);
                }
            }
            if !pinned_mirrored_monotone_counts(domain).is_empty() {
                emitter.result(&format!(
                    "name=verdict {place} set={} pairs={} today_j5a_violations={} today_j5b_violations={} ideal_j5a_violations={} ideal_j5b_violations={}",
                    PairSet::Wide.label(),
                    total.pair(PairSet::Wide, PairColumn::Pairs),
                    total.pair(PairSet::Wide, PairColumn::TodayAppliedSetViolations),
                    total.pair(PairSet::Wide, PairColumn::TodayLandedVersionViolations),
                    total.pair(PairSet::Wide, PairColumn::IdealAppliedSetViolations),
                    total.pair(PairSet::Wide, PairColumn::IdealLandedVersionViolations),
                ));
            }
        }
        Family::Adversarial => {
            emitter.result(&format!(
                "name=verdict {place} states={} a0_violations={} failed_states={} as_whole_table_violations={} as_consecutive_gap_continues_violations={} as_consecutive_gap_stops_violations={} readings_differing_whole_table_vs_consecutive_gap_continues={} readings_differing_whole_table_vs_consecutive_gap_stops={} readings_differing_consecutive_gap_continues_vs_consecutive_gap_stops={} a_structure_violations={} a_limit_violations={} j6_premise_holds_violations={} j6_premise_fails_violations={} j6_premise_fails_judged={} applied_ge1={} states_holding_a_record_rejected_by_parse={} domain_is_nonempty={domain_is_nonempty}",
                total.get(Counter::States),
                total.get(Counter::Panicked),
                total.get(Counter::Failed),
                total.get(Counter::AsWholeTableViolations),
                total.get(Counter::AsGapContinuesViolations),
                total.get(Counter::AsGapStopsViolations),
                total.get(Counter::MutualWholeTableAndGapContinues),
                total.get(Counter::MutualWholeTableAndGapStops),
                total.get(Counter::MutualGapContinuesAndGapStops),
                total.get(Counter::StructureViolations),
                total.get(Counter::LimitViolations),
                total.get(Counter::J6PremiseHoldsViolations),
                total.get(Counter::J6PremiseFailsViolations),
                total.get(Counter::J6PremiseFailsJudged),
                total.get(Counter::AppliedAtLeastOne),
                total.get(Counter::StatesHoldingARecordRejectedByParse),
            ));
        }
    }
    if run_profile.judges_j6qn() {
        let first_returned_root_txg_is_the_largest =
            total.get(Counter::FirstReturnedRootTxgIsTheLargest);
        // 登记 r2 作废条款 SV4：E1 上 ρ_1 的 txg 是 u64 最大值的态数为 0，J6Qn 在 E1 上的 0 不算「Qn 管住了顶格」。
        if domain == Domain::TxgOneBelowTheLargest && first_returned_root_txg_is_the_largest == 0 {
            findings.void_conditions.push(
                "SV4 E1 域上第一次返回的根 txg 是 u64 最大值的态数为 0，J6Qn 在 E1 上的 0 不算数"
                    .to_string(),
            );
        }
    }
    if run_profile.judges_j7() {
        let first_returned_root_txg_is_the_largest =
            total.get(Counter::FirstReturnedRootTxgIsTheLargest);
        emitter.result(&format!(
            "name=verdict {place} criteria=r2 j6qn_premise_holds_violations={} j6qn_premise_holds_judged={} j6qn_premise_fails_violations={} j6qn_premise_fails_judged={} j7_judged={} j7a_instance_table_violations={} j7a_allocation_record_tree_root_violations={} j7a_filesystem_identifier_violations={} j7b_violations={} j7b_judged={} j7c_violations={} j7c_judged={} first_returned_root_txg_is_the_largest={first_returned_root_txg_is_the_largest}",
            total.get(Counter::J6QnPremiseHoldsViolations),
            total.get(Counter::J6QnPremiseHoldsJudged),
            total.get(Counter::J6QnPremiseFailsViolations),
            total.get(Counter::J6QnPremiseFailsJudged),
            total.get(Counter::J7Judged),
            total.get(Counter::J7aInstanceTableViolations),
            total.get(Counter::J7aAllocationRecordTreeRootViolations),
            total.get(Counter::J7aFilesystemIdentifierViolations),
            total.get(Counter::J7bViolations),
            total.get(Counter::J7bJudged),
            total.get(Counter::J7cViolations),
            total.get(Counter::J7cJudged),
        ));
    }
    if run_profile.judges_j8() {
        emitter.result(&format!(
            "name=verdict {place} criteria=r5 j6qn_premise_holds_violations={} j6qn_premise_holds_judged={} j6qn_premise_fails_violations={} j6qn_premise_fails_judged={} j8_judged={} j8a_judged={} j8a_instance_table_violations={} j8a_allocation_record_tree_root_violations={} j8a_unmount_marker_violations={} j8b_violations={} j8b_judged={} j8c_violations={} j8c_judged={} j8_without_a_source={} first_returned_root_txg_is_the_largest={}",
            total.get(Counter::J6QnPremiseHoldsViolations),
            total.get(Counter::J6QnPremiseHoldsJudged),
            total.get(Counter::J6QnPremiseFailsViolations),
            total.get(Counter::J6QnPremiseFailsJudged),
            total.get(Counter::J8Judged),
            total.get(Counter::J8aJudged),
            total.get(Counter::J8aInstanceTableViolations),
            total.get(Counter::J8aAllocationRecordTreeRootViolations),
            total.get(Counter::J8aUnmountMarkerViolations),
            total.get(Counter::J8bViolations),
            total.get(Counter::J8Judged),
            total.get(Counter::J8cViolations),
            total.get(Counter::J8cJudged),
            total.get(Counter::J8WithoutASource),
            total.get(Counter::FirstReturnedRootTxgIsTheLargest),
        ));
    }
    if run_profile.splits_j6qn_by_i812() && domain.family() == Family::Adversarial {
        emitter.result(&verdict_row_of_the_i812_split(&place, total));
    }
    if run_profile.compares_the_two_record_tables() {
        let reported_order_fields = match run_profile.reported_disk_order {
            ReportedDiskOrder::FirstDeviceBeforeSecond => String::new(),
            ReportedDiskOrder::SecondDeviceBeforeFirst => format!(
                " tables_differ_in_reported_order={} rows_differ_in_reported_order={}",
                total.get(Counter::TablesDifferInReportedOrder),
                total.get(Counter::RowsDifferInReportedOrder),
            ),
        };
        emitter.result(&format!(
            "name=verdict {place} criteria=r2_record_tables tables_differ={} rows_differ={}{reported_order_fields}",
            total.get(Counter::TablesDiffer),
            total.get(Counter::RowsDiffer),
        ));
    }
}

// ── 常量回比（登记第三节、停机条款 H2）与装置自检

/// 本地常量逐个对着实现侧的同名常量回比；在飞上限与记录落点在装置取的每个环长上对着实现的函数回比。
fn cross_check_local_constants_against_the_implementation() {
    assert_eq!(
        LOCAL_JOURNAL_RECORD_BYTES,
        singlefs_format::JOURNAL_RECORD_BYTES,
        "H2：记录字节数"
    );
    assert_eq!(
        LOCAL_JOURNAL_SAFETY_FACTOR,
        singlefs_format::JOURNAL_SAFETY_FACTOR,
        "H2：安全系数 F"
    );
    assert_eq!(
        LOCAL_JOURNAL_RING_START_SLOT,
        singlefs_format::JOURNAL_RING_START_SLOT,
        "H2：环起点槽号"
    );
    assert_eq!(
        LOCAL_SLOT_BYTES,
        singlefs_format::SLOT_BYTES,
        "H2：槽字节数"
    );
    assert_eq!(
        LOCAL_FIRST_JOURNAL_COUNTER, FIRST_JOURNAL_COUNTER,
        "H2：第一个计数器"
    );
    assert_eq!(
        LOCAL_INDEX_NODE_UNIT_BYTES,
        singlefs_format::NODE_BYTES,
        "H2：索引节点单元字节数"
    );
    assert_eq!(
        LOCAL_UNIT_CLASS_INDEX_NODE,
        singlefs_core::unit::UNIT_CLASS_INDEX_NODE,
        "H2：索引节点的类标签"
    );
    assert_eq!(
        SlotNumber(NAMED_UNIT_FIRST_SLOT).to_device_offset().0,
        NAMED_UNIT_FIRST_SLOT * LOCAL_SLOT_BYTES,
        "H2：槽号到设备内偏移的换算"
    );
    // 甲类锚点（登记第七节）：64 槽 ⇒ 21，9 槽 ⇒ 3，6 槽 ⇒ 2；不符走 F11。
    for (ring, pinned_limit) in [
        (RingGeometry::BIG, 21usize),
        (RingGeometry::SMALL_NINE, 3),
        (RingGeometry::SMALL_SIX, 2),
    ] {
        assert_eq!(
            ring.local_in_flight_limit(),
            pinned_limit,
            "F11：装置自己算的在飞上限（环 {} 槽）",
            ring.ring_slots
        );
        assert_eq!(
            u64::try_from(ring.local_in_flight_limit()).expect("上限不超过 21"),
            journal_in_flight_record_limit(ring.ring_bytes()),
            "F11 / H2：装置自己算的在飞上限与 journal_in_flight_record_limit 不等（环 {} 槽）",
            ring.ring_slots
        );
        for counter in LOCAL_FIRST_JOURNAL_COUNTER..=200 {
            assert_eq!(
                ring.local_record_offset_in_bytes(counter),
                record_offset(counter, ring.ring_bytes()).0,
                "H2：记录落点（计数器 {counter}，环 {} 槽）",
                ring.ring_slots
            );
        }
    }
}

fn hand_made_record(
    counter: u64,
    checkpoint_txg: u64,
    ordinal_within_publish: u32,
    carries_the_last_record_flag: bool,
) -> JournalRecord {
    let segment = NewRootSegment::off_timeline(counter, checkpoint_txg);
    JournalRecord {
        instance: InstanceGeneration(CHOSEN_INSTANCE),
        counter,
        checkpoint_txg: CheckpointTxg(checkpoint_txg),
        transaction: 1,
        is_commit: true,
        ordinal_within_publish: JournalRecordOrdinalWithinPublish(ordinal_within_publish),
        place_in_publish: if carries_the_last_record_flag {
            JournalRecordPlaceInPublish::LastRecordOfThePublish
        } else {
            JournalRecordPlaceInPublish::MoreRecordsOfThePublishFollow
        },
        back_chain: 0,
        filesystem_identifier: FILESYSTEM_IDENTIFIER_LOW_EIGHT_BYTES,
        new_tree_table: segment.tree_table,
        new_mapping_root: segment.mapping_root,
        new_tree_identifier_watermark: segment.tree_identifier_watermark,
        new_rollback_floor: CheckpointTxg(segment.rollback_floor),
        new_instance_table: fields_beyond_the_projection_carried_by_every_generated_record()
            .instance_table,
        new_allocation_record_tree_root:
            fields_beyond_the_projection_carried_by_every_generated_record()
                .allocation_record_tree_root,
        new_unmount_marker: fields_beyond_the_projection_carried_by_every_generated_record()
            .unmount_marker,
        named: Vec::new(),
    }
}

fn table_of(records: Vec<JournalRecord>) -> RecordTable {
    records
        .into_iter()
        .map(|record| ((record.instance, record.counter), record))
        .collect()
}

/// S 在一张手写的表上、三种读法各自的期望（所选根取对抗域那一条，上限取大环的，表里的前一条一律当不在盘上）。
fn expectations_on_a_hand_made_table(records: &RecordTable) -> [SpecificationExpectation; 3] {
    let chosen_root = adversarial_chosen_root(RunProfile::R1);
    let view = SpecificationView {
        chosen_root: &chosen_root,
        records,
        in_flight_limit: RingGeometry::BIG.local_in_flight_limit(),
        verify_named_units: false,
        truths: &[],
    };
    FollowReading::REGISTERED.map(|reading| specification_expectation(&view, reading))
}

fn check_the_back_chain_of_every_generated_record() {
    for ring_domain in [Domain::LegalBigRing, Domain::LegalSmallRingSix] {
        for unit in legal_work_units(ring_domain, 4, RunProfile::R1) {
            let history = generate_history(&unit.plan);
            for (record_index, slot) in history.slots.iter().enumerate() {
                let expected_back_chain = match record_index.checked_sub(1) {
                    Some(index_before) => back_chain_of(&history.slots[index_before].bytes),
                    None => BACK_CHAIN_WHEN_THE_PREVIOUS_SLOT_HOLDS_NOTHING,
                };
                let parsed = slot.parsed.as_ref().expect("合法历史的每条记录都解得出");
                assert_eq!(
                    parsed.back_chain, expected_back_chain,
                    "生成的每条记录的反向链等于 back_chain_of(前一条的字节)（历史 {}，第 {record_index} 条）",
                    history.shape_description
                );
            }
        }
    }
}

fn check_the_new_root_segment_of_every_generated_record() {
    for unit in legal_work_units(Domain::LegalBigRing, 4, RunProfile::R1) {
        let history = generate_history(&unit.plan);
        for publish_index in 0..history.publishes.len() {
            for record_index in history.record_indexes_of_publish(publish_index) {
                let parsed = history.slots[record_index]
                    .parsed
                    .as_ref()
                    .expect("合法历史的每条记录都解得出");
                assert_eq!(
                    VersionFields::of_record(parsed),
                    history.version_fields(publish_index),
                    "一次发布的每条记录带同一份新根段（历史 {}，第 {record_index} 条）",
                    history.shape_description
                );
            }
        }
    }
}

fn check_the_state_counts_of_the_smallest_sizes() {
    assert_eq!(
        every_subset_mask(3).collect::<Vec<u32>>(),
        vec![0, 1, 2, 3, 4, 5, 6, 7],
        "三位的子集掩码从空集起共 8 个"
    );
    for domain in Domain::ALL {
        // 多盘域只在 r2 档有，它们的最小几档在 [`check_the_multi_disk_state_counts_of_the_smallest_sizes`] 里照 r2 档核。
        if domain.family() == Family::MultiDisk {
            continue;
        }
        let largest_checked_size = (domain.smallest_size() + 1).min(2);
        for size in domain.smallest_size()..=largest_checked_size {
            let tally = tally_of_a_domain_size(
                *domain,
                size,
                Implementation::TodayReplayJournal,
                RunProfile::R1,
                1,
            );
            assert_the_pinned_absolute_values(*domain, size, &tally);
        }
    }
}

fn check_the_three_readings_on_hand_made_tables() {
    // 带标志的一条（txg 101）、txg 越过的一条（102）、同 txg 的一条（101）：全表看为真，连着看两种都在 txg 越过处停、为假。
    let flagged_then_later_txg_then_same_txg = table_of(vec![
        hand_made_record(8, 101, 1, true),
        hand_made_record(9, 102, 1, true),
        hand_made_record(10, 101, 2, false),
    ]);
    let flagged = &flagged_then_later_txg_then_same_txg[&(InstanceGeneration(CHOSEN_INSTANCE), 8)];
    assert_eq!(
        FollowReading::REGISTERED.map(|reading| a_same_publish_record_follows(
            reading,
            flagged,
            &flagged_then_later_txg_then_same_txg
        )),
        [true, false, false],
        "带标志一条、txg 越过一条、同 txg 一条：全表看 / 连着看·缺号不停 / 连着看·缺号即停"
    );
    let nothing_applied = SpecificationExpectation::ReturnsRoot(VersionFields::of_root(
        &adversarial_chosen_root(RunProfile::R1),
    ));
    let lands_on_counter_nine = SpecificationExpectation::ReturnsRoot(VersionFields::of_record(
        &hand_made_record(9, 102, 1, true),
    ));
    assert_eq!(
        expectations_on_a_hand_made_table(&flagged_then_later_txg_then_same_txg),
        [
            nothing_applied,
            lands_on_counter_nine,
            lands_on_counter_nine
        ],
        "同一张表上 S 三种读法的期望：全表看一条都不施加，连着看两种都施加到计数器 9 那一条"
    );
    // 带标志的一条、缺一号、同 txg 的一条：全表看与缺号不停为真，缺号即停为假。
    let flagged_then_gap_then_same_txg = table_of(vec![
        hand_made_record(8, 101, 1, true),
        hand_made_record(10, 101, 2, false),
    ]);
    let flagged_before_the_gap =
        &flagged_then_gap_then_same_txg[&(InstanceGeneration(CHOSEN_INSTANCE), 8)];
    assert_eq!(
        FollowReading::REGISTERED.map(|reading| a_same_publish_record_follows(
            reading,
            flagged_before_the_gap,
            &flagged_then_gap_then_same_txg
        )),
        [true, true, false],
        "带标志一条、缺一号、同 txg 一条：全表看 / 连着看·缺号不停 / 连着看·缺号即停"
    );
    let lands_on_counter_eight = SpecificationExpectation::ReturnsRoot(VersionFields::of_record(
        &hand_made_record(8, 101, 1, true),
    ));
    assert_eq!(
        expectations_on_a_hand_made_table(&flagged_then_gap_then_same_txg),
        [nothing_applied, nothing_applied, lands_on_counter_eight],
        "同一张表上 S 三种读法的期望：只有缺号即停施加计数器 8 那一条"
    );
}

/// U 域、两次发布各一条记录：第二条在盘上、它点名的单元 0 号盘那份校验和不对。
fn state_with_one_bad_unit_copy_judged_under(implementation: Implementation) -> Tally {
    let one_record_publish = || PublishShape {
        record_count: 1,
        commit_marker_bits: 0,
    };
    let plan = history_plan(
        Domain::NamedUnitsVerified,
        ORDINARY_FIRST_TXG,
        vec![one_record_publish(), one_record_publish()],
        Vec::new(),
        RunProfile::R1,
    );
    let history = generate_history(&plan);
    let presence = [
        RecordPresence::OnDisk(UnitState::BothCopiesGood),
        RecordPresence::OnDisk(UnitState::FirstDeviceCopyChecksumWrong),
    ];
    let state = LegalState {
        history: &history,
        presence: &presence,
        mirroring: JournalMirroring::FirstDeviceOnly,
        residue: None,
        chosen_version: 0,
        verify_named_units: true,
    };
    let mut tally = Tally::new();
    judge_a_legal_state(
        &state,
        implementation,
        SpecificationSelfCheck::Judged,
        false,
        &mut tally,
    );
    tally
}

fn check_root_plus_disk_on_a_bad_unit_copy() {
    let tally = state_with_one_bad_unit_copy_judged_under(Implementation::Control(
        ControlKind::LandsOnePublishTooFarOverAnIncompletePublish,
    ));
    assert_eq!(
        (
            tally.get(Counter::J1Violations),
            tally.get(Counter::J2Violations),
            tally.get(Counter::FarthestAtLeastOne)
        ),
        (0, 1, 0),
        "记录在、点名单元一份坏：多落一版的假结局过 J1、不过 J2，j* 停在所选根"
    );
}

fn check_the_in_flight_limit_control_on_both_rings() {
    let readings = control_readings_of(Criterion::J3WithinTheInFlightLimit, RunProfile::R1, 1);
    assert_eq!(readings.len(), 2, "J3 的对照两个读数：小环、大环");
    assert!(
        readings[0].value > 0 && readings[0].is_red_as_required,
        "不看上限的假结局在小环上 J3 违例数大于 0：{readings:?}"
    );
    assert!(
        readings[1].value == 0 && readings[1].is_red_as_required,
        "不看上限的假结局在大环上 J3 违例数是 0：{readings:?}"
    );
}

fn check_the_farthest_version_on_the_small_ring() {
    // 小环 6（上限 2）：P_0 一条、P_1 两条、P_2 一条全在盘上 ⇒ j* = 1（再加 P_2 是 3 条，过上限）；不看上限是 2。
    let shape = |record_count: usize| PublishShape {
        record_count,
        commit_marker_bits: 0,
    };
    let plan = history_plan(
        Domain::LegalSmallRingSix,
        ORDINARY_FIRST_TXG,
        vec![shape(1), shape(2), shape(1)],
        Vec::new(),
        RunProfile::R1,
    );
    let history = generate_history(&plan);
    let presence = [RecordPresence::OnDisk(UnitState::BothCopiesGood); 4];
    let state = LegalState {
        history: &history,
        presence: &presence,
        mirroring: JournalMirroring::FirstDeviceOnly,
        residue: None,
        chosen_version: 0,
        verify_named_units: true,
    };
    let in_flight_limit = history.ring.local_in_flight_limit();
    assert_eq!(
        (
            farthest_version_of(&state, Some(in_flight_limit)),
            farthest_version_of(&state, None)
        ),
        (1, 2),
        "小环 6 上 1 + 2 + 1 全在盘上：j* 看上限是 1，不看上限是 2"
    );
}

fn check_the_fixed_point_control() {
    let tally = control_tally(
        Domain::LegalBigRing,
        3,
        ControlKind::SecondCallMovesOn,
        RunProfile::R1,
        1,
    );
    assert!(
        tally.get(Counter::J6PremiseHoldsViolations) > 0,
        "第二次调用换成「再多落一版」的假结局，J6 违例数大于 0"
    );
}

fn check_the_two_monotone_readings_on_one_later_record() {
    let tally = tally_of_a_domain_size(
        Domain::MonotoneBigRing,
        1,
        Implementation::TodayReplayJournal,
        RunProfile::R1,
        1,
    );
    assert_eq!(
        (
            tally.pair(PairSet::Wide, PairColumn::Pairs),
            tally.pair(PairSet::Wide, PairColumn::IdealAppliedSetViolations),
            tally.pair(PairSet::Wide, PairColumn::IdealLandedVersionViolations)
        ),
        (9, 1, 0),
        "C 域 Nc = 1 的「宽」集合 9 对：理想恢复在施加集读法上恰 1 对违例（记录落了、根槽落没落那一对），落到的版本读法 0 对"
    );
}

fn check_the_structure_control() {
    let tally = control_tally(
        Domain::AdversarialBigRing,
        1,
        ControlKind::RootFromARecordWithoutTheLastRecordFlag,
        RunProfile::R1,
        1,
    );
    assert!(
        tally.get(Counter::StructureViolations) > 0,
        "返回根取自不带末条标志的记录的假结局，A-结构 违例数大于 0"
    );
}

fn check_a_residue_does_not_complete_a_publish() {
    // R 域 N = 2：P_1 那一条 0 号盘没落，1 号盘上有残留 ⇒ 它进了表，但 P_1 不算完整在盘上，j* 是 0。
    let units = legal_work_units(Domain::Residue, 2, RunProfile::R1);
    assert_eq!(units.len(), 1, "R 域 N = 2 只有一段历史、残留只有一个位置");
    let history = generate_history(&units[0].plan);
    let residue = generate_residue(&history, 1);
    let presence = [
        RecordPresence::OnDisk(UnitState::BothCopiesGood),
        RecordPresence::NotOnDisk,
    ];
    let state = LegalState {
        history: &history,
        presence: &presence,
        mirroring: JournalMirroring::FirstDeviceOnly,
        residue: Some(&residue),
        chosen_version: 0,
        verify_named_units: true,
    };
    let (records, _disk, _truths) = record_table_and_disk_of(&state);
    assert_eq!(records.len(), 2, "0 号盘那一份没落时残留进表");
    assert_eq!(
        farthest_version_of(&state, Some(history.ring.local_in_flight_limit())),
        0,
        "残留不算时间线上的记录：P_1 不完整，j* 停在所选根"
    );
}

fn check_parse_rejects_ordinal_zero_whose_checksums_hold() {
    let ordinal_zero = hand_made_record(8, 101, 0, true);
    let ordinal_one = hand_made_record(8, 101, 1, true);
    let slot = generate_slot(
        0,
        RingGeometry::BIG,
        &ordinal_zero,
        ParseExpectation::OrdinalZeroMustBeRejected,
    );
    assert!(
        slot.parsed.is_none(),
        "序号 0 的记录 JournalRecord::parse 拒收（读到 0 当损坏）"
    );
    // 只把序号那 4 字节改回 1、重算自证校验和，就解得出：被拒的原因只是序号。
    let readable = generate_slot(
        0,
        RingGeometry::BIG,
        &ordinal_one,
        ParseExpectation::MustRoundTrip,
    );
    assert_eq!(readable.parsed.as_ref(), Some(&ordinal_one));
    assert_ne!(slot.bytes, readable.bytes, "两条记录只差序号，字节不同");
}

/// 装置开跑先做的自检（不含阳性对照，那一段在 [`main`] 里单独跑、逐行进产物）。每一项都有一条同名的单测。
const SELF_CHECKS: &[(&str, fn())] = &[
    (
        "local_constants_equal_the_implementation",
        cross_check_local_constants_against_the_implementation,
    ),
    (
        "back_chain_of_every_generated_record",
        check_the_back_chain_of_every_generated_record,
    ),
    (
        "new_root_segment_of_every_generated_record",
        check_the_new_root_segment_of_every_generated_record,
    ),
    (
        "state_counts_of_the_smallest_sizes",
        check_the_state_counts_of_the_smallest_sizes,
    ),
    (
        "three_readings_on_hand_made_tables",
        check_the_three_readings_on_hand_made_tables,
    ),
    (
        "root_plus_disk_on_a_bad_unit_copy",
        check_root_plus_disk_on_a_bad_unit_copy,
    ),
    (
        "farthest_version_on_the_small_ring",
        check_the_farthest_version_on_the_small_ring,
    ),
    (
        "two_monotone_readings_on_one_later_record",
        check_the_two_monotone_readings_on_one_later_record,
    ),
    (
        "residue_does_not_complete_a_publish",
        check_a_residue_does_not_complete_a_publish,
    ),
    (
        "parse_rejects_ordinal_zero_whose_checksums_hold",
        check_parse_rejects_ordinal_zero_whose_checksums_hold,
    ),
    (
        "in_flight_limit_control_on_both_rings",
        check_the_in_flight_limit_control_on_both_rings,
    ),
    ("fixed_point_control", check_the_fixed_point_control),
    ("structure_control", check_the_structure_control),
];

/// r2 档生成器的四个字段（登记 r2 第一节、第八节）：两种奇偶各取第 0–3 版逐字段比；fsid 低 8 字节不随版本变、本地取法与
/// `unit_filesystem_identifier` 回比（登记 r2 第三节）；由记录重建的根取 S1 的形态；r1 档生成器上「重建」与时间线根逐字段相同。
fn check_the_fields_beyond_the_projection_of_the_r2_generator() {
    for version in 0..4usize {
        assert_eq!(
            RunProfile::R1.fields_beyond_the_projection_of_version(version),
            FieldsBeyondTheProjection {
                filesystem_identifier: ROOT_FILESYSTEM_IDENTIFIER,
                unmount_marker: UnmountMarker::NotWrittenByTheUnmountSequence,
                instance_table: node_pointer_at_slot(69_000, 1),
                allocation_record_tree_root: NodePointer::empty_root(),
            },
            "r1 档：第 {version} 版投影之外的四个字段与 r1 时的装置相同"
        );
    }
    for (marker_parity, versions_written_by_the_unmount_sequence) in [
        (
            MarkerParity::EvenVersionsWrittenByTheUnmountSequence,
            [0usize, 2],
        ),
        (MarkerParity::OddVersionsWrittenByTheUnmountSequence, [1, 3]),
    ] {
        for version in 0..4usize {
            let fields =
                RunProfile::r2(marker_parity).fields_beyond_the_projection_of_version(version);
            let version_number = u64::try_from(version).expect("版本数很小");
            let mut expected_filesystem_identifier = ROOT_FILESYSTEM_IDENTIFIER;
            expected_filesystem_identifier[15] = u8::try_from(version).expect("版本数很小");
            assert_eq!(
                (
                    fields.instance_table,
                    fields.allocation_record_tree_root,
                    fields.filesystem_identifier
                ),
                (
                    node_pointer_at_slot(69_000 + version_number, 1),
                    node_pointer_at_slot(68_000 + version_number, 1),
                    expected_filesystem_identifier
                ),
                "r2 档（{}）第 {version} 版：实例表槽 69 000 + j、分配记录树根槽 68 000 + j、fsid 末字节 j",
                marker_parity.label()
            );
            assert_eq!(
                fields.unmount_marker,
                if versions_written_by_the_unmount_sequence.contains(&version) {
                    UnmountMarker::WrittenByTheUnmountSequence
                } else {
                    UnmountMarker::NotWrittenByTheUnmountSequence
                },
                "r2 档（{}）第 {version} 版的卸载记号",
                marker_parity.label()
            );
            assert_eq!(
                local_filesystem_identifier_low_eight_bytes(&fields.filesystem_identifier),
                FILESYSTEM_IDENTIFIER_LOW_EIGHT_BYTES,
                "fsid 低 8 字节不随版本变，等于记录头那 8 字节"
            );
            assert_eq!(
                local_filesystem_identifier_low_eight_bytes(&fields.filesystem_identifier),
                singlefs_core::unit::unit_filesystem_identifier(&fields.filesystem_identifier),
                "登记 r2 第三节：fsid 低 8 字节的本地取法与 unit_filesystem_identifier 不等"
            );
        }
    }
    let one_record_publish = || PublishShape {
        record_count: 1,
        commit_marker_bits: 0,
    };
    let history_under = |run_profile: RunProfile| {
        generate_history(&history_plan(
            Domain::LegalBigRing,
            ORDINARY_FIRST_TXG,
            vec![one_record_publish(), one_record_publish()],
            Vec::new(),
            run_profile,
        ))
    };
    let r2_profile = RunProfile::r2(MarkerParity::EvenVersionsWrittenByTheUnmountSequence);
    let r2_history = history_under(r2_profile);
    let first_root = r2_history.timeline_root(0);
    let second_root = r2_history.timeline_root(1);
    assert_eq!(
        VersionFields::of_root(&second_root),
        r2_history.version_fields(1)
    );
    assert_eq!(
        FieldsBeyondTheProjection::of_root(&second_root),
        r2_profile.fields_beyond_the_projection_of_version(1),
        "时间线根 R_1 的四个字段照 r2 档第 1 版"
    );
    assert_eq!(
        adversarial_chosen_root(r2_profile),
        first_root,
        "对抗一族的所选根就是时间线 R_0"
    );
    let rebuilt = r2_history
        .version_fields(1)
        .to_root_rebuilt_over(&first_root);
    assert_eq!(
        (
            VersionFields::of_root(&rebuilt),
            rebuilt.instance_table,
            rebuilt.allocation_record_tree_root,
            rebuilt.filesystem_identifier,
            rebuilt.unmount_marker
        ),
        (
            r2_history.version_fields(1),
            first_root.instance_table,
            first_root.allocation_record_tree_root,
            first_root.filesystem_identifier,
            UnmountMarker::NotWrittenByTheUnmountSequence
        ),
        "由记录重建的根：投影取记录的，实例表指针、分配记录树根、fsid 抄所选根，卸载记号 NotWritten"
    );
    let r1_history = history_under(RunProfile::R1);
    assert_eq!(
        r1_history
            .version_fields(1)
            .to_root_rebuilt_over(&r1_history.timeline_root(0)),
        r1_history.timeline_root(1),
        "r1 档生成器上由记录重建的根与时间线根逐字段相同（r1 档输出不变的前提）"
    );
}

/// `--profile` 与 `--marker-parity`（登记 r2 5.1、第八节）：不给 `--profile` 时 `replay.sh` 三份 r1 产物的那三条命令仍是 r1 档、
/// 参数原样留下；两个参数写在哪都行；`--marker-parity` 只在 r2 档有。
fn check_the_profile_options() {
    let split = |words: &[&str]| {
        split_off_the_profile_options(
            &words
                .iter()
                .map(|word| (*word).to_string())
                .collect::<Vec<String>>(),
        )
    };
    let owned = |words: &[&str]| {
        words
            .iter()
            .map(|word| (*word).to_string())
            .collect::<Vec<String>>()
    };
    for r1_product_command in [
        ["--bounds", "segment1", "--family", "legal"],
        ["--bounds", "light", "--family", "legal"],
        ["--bounds", "segment1", "--family", "adversarial"],
    ] {
        assert_eq!(
            split(&r1_product_command),
            Ok((RunProfile::R1, owned(&r1_product_command))),
            "不给 --profile 时是 r1 档、参数原样留下：{r1_product_command:?}"
        );
    }
    assert_eq!(
        split(&["--profile", "r2", "--bounds", "segment1"]),
        Ok((
            RunProfile::r2(MarkerParity::EvenVersionsWrittenByTheUnmountSequence),
            owned(&["--bounds", "segment1"])
        ))
    );
    assert_eq!(
        split(&[
            "--bounds",
            "segment1",
            "--profile",
            "r2",
            "--marker-parity",
            "odd"
        ]),
        Ok((
            RunProfile::r2(MarkerParity::OddVersionsWrittenByTheUnmountSequence),
            owned(&["--bounds", "segment1"])
        ))
    );
    assert_eq!(
        split(&["--profile", "r2", "--list-states", "L:2"]),
        Ok((
            RunProfile::r2(MarkerParity::EvenVersionsWrittenByTheUnmountSequence),
            owned(&["--list-states", "L:2"])
        )),
        "逐态判定行同样认 --profile"
    );
    assert!(split(&["--marker-parity", "odd"]).is_err(), "r1 档不认奇偶");
    assert_eq!(
        split(&["--profile", "r3", "--bounds", "segment1"]),
        Ok((
            RunProfile::r3(MarkerParity::EvenVersionsWrittenByTheUnmountSequence),
            owned(&["--bounds", "segment1"])
        )),
        "r3 档认得，参数原样留下"
    );
    assert_ne!(
        RunProfile::r3(MarkerParity::EvenVersionsWrittenByTheUnmountSequence),
        RunProfile::r2(MarkerParity::EvenVersionsWrittenByTheUnmountSequence),
        "r3 档与 r2 档是两个档"
    );
    assert_eq!(
        split(&[
            "--profile",
            "r5",
            "--marker-parity",
            "odd",
            "--bounds",
            "segment1"
        ]),
        Ok((
            RunProfile::r5(MarkerParity::OddVersionsWrittenByTheUnmountSequence),
            owned(&["--bounds", "segment1"])
        )),
        "r5 档认得（登记 r5 5.1），奇偶照 r2 档的意思认，参数原样留下"
    );
    let (r5_profile, _remaining_arguments) =
        split(&["--profile", "r5"]).expect("r5 档认得（登记 r5 5.1）");
    assert!(
        r5_profile.judges_j8() && !r5_profile.judges_j7() && r5_profile.judges_j6qn(),
        "r5 档判 J8、J6Qn，不判 J7（登记 r5 5.1、5.3）"
    );
    assert!(
        !r5_profile.admits_the_root_choice_family(),
        "r5 档不认根环择根一族（登记 r5 5.1）"
    );
    assert_ne!(
        RunProfile::r5(MarkerParity::EvenVersionsWrittenByTheUnmountSequence),
        RunProfile::r3(MarkerParity::EvenVersionsWrittenByTheUnmountSequence),
        "r5 档与 r3 档是两个档"
    );
    assert!(split(&["--profile", "r6"]).is_err());
    assert!(split(&["--profile"]).is_err());
    assert!(split(&["--profile", "r2", "--profile", "r1"]).is_err());
}

/// J6Qn 三条对照的读数（登记 r2 第七节「5.5 的对照钉成等式」）：L 域 N ≤ 5 两种第二次报错各 3 110（违例与 judged），报
/// `CHOSEN_ROOT_…` 那一条原 J6 同值；E1 全界第一次取理想恢复时原 J6 70、J6Qn 0。
#[cfg(test)]
fn check_the_j6qn_control_readings() {
    let readings = control_readings_of(
        Criterion::J6QnFixedPoint,
        RunProfile::r2(MarkerParity::EvenVersionsWrittenByTheUnmountSequence),
        1,
    );
    assert_eq!(
        readings
            .iter()
            .map(|reading| reading.value)
            .collect::<Vec<u64>>(),
        vec![3_110, 3_110, 3_110, 3_110, 3_110, 70, 0],
        "J6Qn 三条对照的读数：{readings:?}"
    );
    assert!(
        readings.iter().all(|reading| reading.is_red_as_required),
        "J6Qn 的对照都照登记红：{readings:?}"
    );
}

/// I-8.12 在手写的表上（登记 r3 第一节「Q3 的拆法」、9.2 甲）：同一实例里 txg 随计数器不减才成立；相等算成立（「严格增」那一读法判不成立）；
/// 换实例的两条不比；计数器断号不影响。
#[cfg(test)]
fn check_the_i812_judgement_on_hand_made_tables() {
    let in_another_instance = |mut record: JournalRecord| {
        record.instance = InstanceGeneration(CHOSEN_INSTANCE + 1);
        record
    };
    assert!(
        record_table_satisfies_i812(&table_of(Vec::new())),
        "空表：成立"
    );
    assert!(
        record_table_satisfies_i812(&table_of(vec![hand_made_record(7, 100, 1, true)])),
        "一条：成立"
    );
    assert!(
        record_table_satisfies_i812(&table_of(vec![
            hand_made_record(7, 100, 1, true),
            hand_made_record(8, 100, 1, true)
        ])),
        "同一实例里相邻两条 txg 相等：不减，成立"
    );
    assert!(
        !record_table_satisfies_i812(&table_of(vec![
            hand_made_record(7, 101, 1, true),
            hand_made_record(8, 100, 1, true)
        ])),
        "同一实例里后一条的 txg 比前一条小：不成立"
    );
    assert!(
        record_table_satisfies_i812(&table_of(vec![
            hand_made_record(7, 100, 1, true),
            hand_made_record(9, 102, 1, true)
        ])),
        "计数器 7、9 之间断号、txg 上升：成立"
    );
    assert!(
        !record_table_satisfies_i812(&table_of(vec![
            hand_made_record(7, 102, 1, true),
            hand_made_record(9, 100, 1, true)
        ])),
        "计数器 7、9 之间断号、txg 下降：不成立（相邻不减与任两条不减等价，断号照比）"
    );
    assert!(
        record_table_satisfies_i812(&table_of(vec![
            hand_made_record(7, 101, 1, true),
            in_another_instance(hand_made_record(8, 100, 1, true))
        ])),
        "两条属于不同实例：I-8.12 只管同一实例里，换实例那一半不判"
    );
}

/// 对抗一族最小几档上，表成立 / 不成立 I-8.12 的态数等于登记的分类计数（登记 r3 第七节乙类；9.2 甲前两条变异靠它红）。
#[cfg(test)]
fn check_the_i812_state_counts_of_the_smallest_sizes() {
    let run_profile = RunProfile::r3(MarkerParity::EvenVersionsWrittenByTheUnmountSequence);
    for (domain, largest_size) in [
        (Domain::AdversarialBigRing, 3),
        (Domain::AdversarialSmallRingSix, 3),
        (Domain::AdversarialBackChain, 2),
    ] {
        for size in domain.smallest_size()..=largest_size {
            let tally = tally_of_a_domain_size(
                domain,
                size,
                Implementation::TodayReplayJournal,
                run_profile,
                1,
            );
            assert_the_pinned_i812_state_counts(domain, size, &tally);
            assert_eq!(
                the_split_rows_differ_from_the_unsplit_rows(&tally),
                None,
                "{} 域 n = {size}：拆开的四行对得回未拆的两行",
                domain.label()
            );
        }
    }
}

/// KF3 的四格读数（登记 r3 5.7、第七节乙类）：对照界里七个 (域, n)，每个 n 上四格的违例数与 judged、成立 / 不成立的态数共十个读数，都红。
#[cfg(test)]
fn check_the_i812_split_control_readings() {
    let readings = control_readings_of(
        Criterion::J6QnSplitByI812,
        RunProfile::r3(MarkerParity::EvenVersionsWrittenByTheUnmountSequence),
        1,
    );
    assert_eq!(
        readings.len(),
        PINNED_KF3_CELLS.len() * 10,
        "每个 (域, n) 十个读数：四格各违例数与 judged 八个、成立与不成立的态数两个：{readings:?}"
    );
    assert!(
        readings.iter().all(|reading| reading.is_red_as_required),
        "KF3 的读数都等于钉的数：{readings:?}"
    );
}

/// 拆出来的行（登记 r3 5.6、9.2 甲）：r3 档对抗一族每个 (域, n) 五行（四格各一行、`i812_states` 一行）；r2 档与合法历史一族一行都没有。
#[cfg(test)]
fn check_the_rows_of_the_i812_split() {
    let r3 = RunProfile::r3(MarkerParity::EvenVersionsWrittenByTheUnmountSequence);
    let r2 = RunProfile::r2(MarkerParity::EvenVersionsWrittenByTheUnmountSequence);
    let tally = tally_of_a_domain_size(
        Domain::AdversarialBigRing,
        2,
        Implementation::TodayReplayJournal,
        r3,
        1,
    );
    let place = "domain=A ring_slots=64 n=2";
    let rows = rows_of_the_i812_split(r3, Domain::AdversarialBigRing, place, &tally);
    assert_eq!(rows.len(), 5, "四格各一行、i812_states 一行：{rows:?}");
    for criterion in [
        "J6Qn_premise_holds_i812_holds",
        "J6Qn_premise_holds_i812_fails",
        "J6Qn_premise_fails_i812_holds",
        "J6Qn_premise_fails_i812_fails",
    ] {
        assert!(
            rows.iter()
                .any(|row| row.contains(&format!(" criterion={criterion} violations="))),
            "缺 {criterion} 那一行：{rows:?}"
        );
    }
    assert_eq!(
        rows[4],
        format!("name=i812_states {place} i812_holds=9122 i812_fails=1536 states=10658"),
        "A n = 2 的分类计数（登记 r3 第七节乙类）"
    );
    assert!(
        rows_of_the_i812_split(r2, Domain::AdversarialBigRing, place, &tally).is_empty(),
        "r2 档不出这几行"
    );
    assert!(
        rows_of_the_i812_split(r3, Domain::LegalBigRing, place, &tally).is_empty(),
        "合法历史一族不出这几行"
    );
}

/// J7 五行对照的读数（登记 r2 第七节）：r2 档 J7a 各自那一格 762、另两格 0，J7b 762，J7c 2 348；r1 档生成器上都是 0。
#[cfg(test)]
fn check_the_j7_control_readings() {
    let run_profile = RunProfile::r2(MarkerParity::EvenVersionsWrittenByTheUnmountSequence);
    for (criterion, expected_values) in [
        (Criterion::J7aInstanceTable, vec![762, 0, 0, 0]),
        (Criterion::J7aAllocationRecordTreeRoot, vec![0, 762, 0, 0]),
        (Criterion::J7aFilesystemIdentifier, vec![0, 0, 762, 0]),
        (Criterion::J7bUnmountMarkerOfARebuiltRoot, vec![762, 0]),
        (
            Criterion::J7cUnmountMarkerWhenNothingIsApplied,
            vec![2_348, 0],
        ),
    ] {
        let readings = control_readings_of(criterion, run_profile, 1);
        assert_eq!(
            readings
                .iter()
                .map(|reading| reading.value)
                .collect::<Vec<u64>>(),
            expected_values,
            "{} 的对照读数：{readings:?}",
            criterion.label()
        );
        assert!(
            readings.iter().all(|reading| reading.is_red_as_required),
            "{} 的对照都照登记红：{readings:?}",
            criterion.label()
        );
    }
}

/// 第八节奇偶的判别力自证：`--marker-parity odd` 解出来的档上，J7b、J7c 两条对照在 L 域 N ≤ 5 都是 0（R_0 是 `NotWritten…`，
/// 假结局与期望同值），在 C 域 Nc ≤ 3 都大于 0。
#[cfg(test)]
fn check_the_odd_marker_parity_turns_the_unmount_marker_controls() {
    let arguments: Vec<String> = [
        "--profile",
        "r2",
        "--marker-parity",
        "odd",
        "--bounds",
        "segment1",
    ]
    .iter()
    .map(|word| (*word).to_string())
    .collect();
    let (run_profile, _remaining_arguments) =
        split_off_the_profile_options(&arguments).expect("这几个参数都认得");
    for (control, counter) in [
        (
            ControlKind::IdealCopyingTheChosenUnmountMarkerWhenApplied,
            Counter::J7bViolations,
        ),
        (
            ControlKind::IdealWritingNotWrittenWhenNothingIsApplied,
            Counter::J7cViolations,
        ),
    ] {
        let big_ring = control_tally(
            Domain::LegalBigRing,
            CONTROL_LARGEST_SIZE_OF_THE_BIG_RING_LEGAL_DOMAIN,
            control,
            run_profile,
            1,
        );
        let monotone = control_tally(
            Domain::MonotoneBigRing,
            CONTROL_LARGEST_SIZE_OF_THE_MONOTONE_BIG_RING_DOMAIN,
            control,
            run_profile,
            1,
        );
        assert_eq!(
            big_ring.get(counter),
            0,
            "反奇偶：L 域 {} 对照是 0",
            counter.label()
        );
        assert!(
            monotone.get(counter) > 0,
            "反奇偶：C 域 {} 对照大于 0",
            counter.label()
        );
    }
}

/// r2 档另跑的启动自检（r1 档不跑、不进产物，r1 档的输出因此与 r1 时的装置逐字节相同）。每一项都有一条同名的单测。
///
/// 启动自检只放不读实现结局的：第二段把装置交给被变异的实现跑，实现一变，读它结局的断言在启动时红、一个域都没跑
/// （segment2b：登记行 406、414、448、518、1003、1004 五个变体在 [`check_the_j6qn_control_readings`] 上 panic）。
/// 逐条判（登记 r2 修订 4）：
/// - `fields_beyond_the_projection_of_the_r2_generator`：只读生成器、`VersionFields` 与 `unit_filesystem_identifier`，不调 `replay_journal`，留下。
/// - `profile_options`：纯参数解析，留下。
/// - `j6qn_control_readings`、`j7_control_readings`、`odd_marker_parity_turns_the_unmount_marker_controls`：三条都经 [`control_readings_of`] /
///   [`control_tally`] 跑假结局，假结局里「第一次照真结局」「第二次照真调用」都调 `replay_journal`（J6Qn 的读数直接取决于它返回什么，
///   J7 与奇偶两条的读数由理想恢复定、但同样调它）；挪成单测（`tests` 模块里同名的三条，断言一个字没动，三个检查函数只在 `cfg(test)` 下编）。产物里它们的读数
///   仍由 [`positive_control_readings`] 逐行打成 `name=control`，不红记 V2。
const R2_SELF_CHECKS: &[(&str, fn())] = &[
    (
        "fields_beyond_the_projection_of_the_r2_generator",
        check_the_fields_beyond_the_projection_of_the_r2_generator,
    ),
    ("profile_options", check_the_profile_options),
];

/// 多盘三个域最小的两档（登记 r2 第七节乙类）：状态数、M 与 M6 的落后类别与见证、MC 的宽对与见证，照 r2 档逐格比钉的值。
fn check_the_multi_disk_state_counts_of_the_smallest_sizes() {
    for domain in [
        Domain::MirroredBigRing,
        Domain::MirroredSmallRingSix,
        Domain::MirroredMonotoneBigRing,
    ] {
        for size in 1..=2 {
            let tally = tally_of_a_domain_size(
                domain,
                size,
                Implementation::TodayReplayJournal,
                RunProfile::r2(MarkerParity::EvenVersionsWrittenByTheUnmountSequence),
                1,
            );
            assert_the_pinned_absolute_values(domain, size, &tally);
        }
    }
}

/// 落后的方向（登记 r2 SQ17、9.5「落后方向两行对调」）：0 号盘什么都没落、1 号盘落了第一条 ⇒ 0 号盘落后；反过来 1 号盘落后。
fn check_the_lagging_direction_on_hand_made_masks() {
    assert_eq!(
        lagging_class_of(0b0, 0b1),
        LaggingClass::FirstDeviceBehind,
        "0 号盘 {{}}、1 号盘 {{r_1}}：0 号盘的落盘集合真包含于 1 号盘的，是 0 号盘落后"
    );
    assert_eq!(
        lagging_class_of(0b1, 0b0),
        LaggingClass::SecondDeviceBehind,
        "0 号盘 {{r_1}}、1 号盘 {{}}：1 号盘落后"
    );
    assert_eq!(lagging_class_of(0b11, 0b11), LaggingClass::Equal);
    assert_eq!(lagging_class_of(0b01, 0b10), LaggingClass::Crossed);
}

/// 多盘四条判据的对照读数（登记 r2 5.5、第七节）：只数 0 号盘的 j\* 在 M、M6、L 上 100、996、0；原登记 J1、J4 两条对照在 M、M6、MC 上
/// 228、2 052、45（J4 那条的 J1、J2、J3 是 0）；MC 宽对上 J5 那条对照两列都大于 0。
fn check_the_multi_disk_control_readings() {
    let run_profile = RunProfile::r2(MarkerParity::EvenVersionsWrittenByTheUnmountSequence);
    for (criterion, expected_values) in [
        (Criterion::MultiDiskAnyCopyIsOnDisk, vec![100, 996, 0]),
        (
            Criterion::MultiDiskReturnedRoot,
            vec![228, 228, 2_052, 2_052, 45, 45],
        ),
        (
            Criterion::MultiDiskFarthestVersion,
            vec![228, 0, 0, 0, 2_052, 0, 0, 0, 45, 0, 0, 0],
        ),
    ] {
        let readings = control_readings_of(criterion, run_profile, 1);
        assert_eq!(
            readings
                .iter()
                .map(|reading| reading.value)
                .collect::<Vec<u64>>(),
            expected_values,
            "{} 的对照读数：{readings:?}",
            criterion.label()
        );
    }
    let readings = control_readings_of(
        Criterion::MultiDiskAppliedSetAndLandedVersion,
        run_profile,
        1,
    );
    assert_eq!(readings.len(), 2, "J5 那条对照两个读数：J5a、J5b");
    assert!(
        readings
            .iter()
            .all(|reading| reading.value > 0 && reading.is_red_as_required),
        "MC 宽对上 J5 那条对照两列都大于 0：{readings:?}"
    );
}

/// 直接建表的盘次序（登记 5.1、r2 5.3、9.5「多盘直接建表先读 1 号盘」）：R 域 N = 2，P_1 那条在 0 号盘落了时表里取 0 号盘那一份、
/// 残留不进表；0 号盘那一份没落时残留进表。M 域两条记录、第二条只有 1 号盘那一份：它进表，P_1 完整在盘上。
fn check_the_direct_table_reads_the_first_device_before_the_second() {
    let units = legal_work_units(Domain::Residue, 2, RunProfile::R1);
    let history = generate_history(&units[0].plan);
    let residue = generate_residue(&history, 1);
    let second_record = history.slots[1]
        .parsed
        .clone()
        .expect("合法历史的每条记录都解得出");
    let residue_record = residue.slot.parsed.clone().expect("残留解得出");
    let key = (second_record.instance, second_record.counter);
    for (second_record_presence, expected_record) in [
        (
            RecordPresence::OnDisk(UnitState::BothCopiesGood),
            &second_record,
        ),
        (RecordPresence::NotOnDisk, &residue_record),
    ] {
        let presence = [
            RecordPresence::OnDisk(UnitState::BothCopiesGood),
            second_record_presence,
        ];
        let state = LegalState {
            history: &history,
            presence: &presence,
            mirroring: JournalMirroring::FirstDeviceOnly,
            residue: Some(&residue),
            chosen_version: 0,
            verify_named_units: true,
        };
        let (records, _disk, _truths) = record_table_and_disk_of(&state);
        assert_eq!(
            records.get(&key),
            Some(expected_record),
            "残留只在 0 号盘那一份没落时进表（P_1 那条 0 号盘：{second_record_presence:?}）"
        );
    }
    let one_record_publish = || PublishShape {
        record_count: 1,
        commit_marker_bits: 0,
    };
    let mirrored_history = generate_history(&history_plan(
        Domain::MirroredBigRing,
        ORDINARY_FIRST_TXG,
        vec![one_record_publish(), one_record_publish()],
        Vec::new(),
        RunProfile::r2(MarkerParity::EvenVersionsWrittenByTheUnmountSequence),
    ));
    let presence = [
        RecordPresence::OnDisk(UnitState::BothCopiesGood),
        RecordPresence::NotOnDisk,
    ];
    let second_device_copy_is_on_disk = [false, true];
    let state = LegalState {
        history: &mirrored_history,
        presence: &presence,
        mirroring: JournalMirroring::BothDevices {
            second_device_copy_is_on_disk: &second_device_copy_is_on_disk,
        },
        residue: None,
        chosen_version: 0,
        verify_named_units: true,
    };
    let (records, _disk, _truths) = record_table_and_disk_of(&state);
    assert_eq!(records.len(), 2, "只有 1 号盘那一份的记录也进表");
    assert_eq!(
        farthest_version_of(&state, Some(mirrored_history.ring.local_in_flight_limit())),
        1,
        "多盘：记录任一份在即在，P_1 完整在盘上，j* = 1"
    );
}

/// 多盘一族另跑的自检（只在 `--family multi_disk` 时跑、进那一份产物）。每一项都有一条同名的单测。
const MULTI_DISK_SELF_CHECKS: &[(&str, fn())] = &[
    (
        "multi_disk_state_counts_of_the_smallest_sizes",
        check_the_multi_disk_state_counts_of_the_smallest_sizes,
    ),
    (
        "lagging_direction_on_hand_made_masks",
        check_the_lagging_direction_on_hand_made_masks,
    ),
    (
        "multi_disk_control_readings",
        check_the_multi_disk_control_readings,
    ),
    (
        "direct_table_reads_the_first_device_before_the_second",
        check_the_direct_table_reads_the_first_device_before_the_second,
    ),
];

/// 扫盘建表喂的系统配置与生成器对得上（登记 r2 第三节回比表的 fsid 一行、第七节甲类）：fsid 低 8 字节的本地取法等于记录头里那 8 字节、
/// 等于 `unit_filesystem_identifier`，journal 环长等于这个环几何的字节数。不调 `scan_journal`（它是被测对象），可以放进启动自检。
fn check_the_scan_system_configuration_matches_the_generator() {
    for ring in [
        RingGeometry::BIG,
        RingGeometry::SMALL_SIX,
        RingGeometry::SMALL_NINE,
    ] {
        let configuration = system_configuration_for_the_scan(ring);
        let filesystem_identifier = configuration.immutable.filesystem_identifier;
        assert_eq!(
            local_filesystem_identifier_low_eight_bytes(&filesystem_identifier),
            FILESYSTEM_IDENTIFIER_LOW_EIGHT_BYTES,
            "扫盘建表的系统配置 fsid 低 8 字节要等于记录头里那 8 字节（环 {} 槽）",
            ring.ring_slots
        );
        assert_eq!(
            local_filesystem_identifier_low_eight_bytes(&filesystem_identifier),
            singlefs_core::unit::unit_filesystem_identifier(&filesystem_identifier),
            "登记 r2 第三节：fsid 低 8 字节的本地取法与 unit_filesystem_identifier 不等（环 {} 槽）",
            ring.ring_slots
        );
        assert_eq!(
            configuration.immutable.sizes.journal_ring_bytes,
            ring.ring_bytes(),
            "扫盘建表的系统配置里 journal 环长要等于这个环几何的字节数（环 {} 槽）",
            ring.ring_slots
        );
    }
}

/// `--table` 与 `--disk-order`（登记 r2 5.4、第八节）：不给取 `direct` 与 `normal`；写在哪都行；r1 档不认。
fn check_the_table_and_disk_order_options() {
    let owned = |words: &[&str]| {
        words
            .iter()
            .map(|word| (*word).to_string())
            .collect::<Vec<String>>()
    };
    let split = |words: &[&str]| split_off_the_profile_options(&owned(words));
    let even_r2 = RunProfile::r2(MarkerParity::EvenVersionsWrittenByTheUnmountSequence);
    assert_eq!(
        split(&["--profile", "r2", "--bounds", "segment1"]),
        Ok((even_r2, owned(&["--bounds", "segment1"]))),
        "不给 --table 与 --disk-order 时取 direct 与 normal"
    );
    assert_eq!(
        split(&[
            "--bounds",
            "segment1",
            "--table",
            "both",
            "--profile",
            "r2",
            "--disk-order",
            "reversed"
        ]),
        Ok((
            even_r2
                .with_table_source(RecordTableSource::Both)
                .with_reported_disk_order(ReportedDiskOrder::SecondDeviceBeforeFirst),
            owned(&["--bounds", "segment1"])
        ))
    );
    assert_eq!(
        split(&["--profile", "r2", "--table", "scan"]),
        Ok((
            even_r2.with_table_source(RecordTableSource::Scanned),
            Vec::new()
        ))
    );
    assert_eq!(
        split(&[
            "--profile",
            "r2",
            "--table",
            "direct",
            "--disk-order",
            "normal"
        ]),
        Ok((even_r2, Vec::new()))
    );
    assert!(split(&["--table", "both"]).is_err(), "r1 档不认 --table");
    assert!(
        split(&["--disk-order", "reversed"]).is_err(),
        "r1 档不认 --disk-order"
    );
    assert!(split(&["--profile", "r2", "--table", "scanned"]).is_err());
    assert!(split(&["--profile", "r2", "--disk-order", "backwards"]).is_err());
    assert!(split(&["--profile", "r2", "--table"]).is_err());
    assert!(
        split(&["--profile", "r2", "--table", "scan", "--table", "both"]).is_err(),
        "同一个参数给了不止一次"
    );
}

/// 盘次序（登记 r2 第八节「判别力自证」前一半）：`--disk-order reversed` 的内存盘 `device_identities()` 报 1 号盘在前，默认的报 0 号盘在前。
fn check_a_reversed_memory_disk_reports_the_second_device_first() {
    let device_numbers_reported_by = |reported_disk_order: ReportedDiskOrder| -> Vec<u32> {
        MemoryDisk {
            ring: RingGeometry::BIG,
            reported_disk_order,
            contents: Vec::new(),
        }
        .device_identities()
        .iter()
        .map(|device| device.0)
        .collect()
    };
    assert_eq!(
        device_numbers_reported_by(ReportedDiskOrder::SecondDeviceBeforeFirst),
        vec![1, 0],
        "--disk-order reversed：device_identities() 要报 1 号盘在前"
    );
    assert_eq!(
        device_numbers_reported_by(ReportedDiskOrder::FirstDeviceBeforeSecond),
        vec![0, 1],
        "默认的盘次序：0 号盘在前"
    );
}

/// 记录表怎么建、盘次序有一个不是默认值时另跑的启动自检（只在 r2 档能设、进那一份产物）。每一项都有一条同名的单测。
/// 都不调 `scan_journal` 与 `replay_journal`：它们是被测对象，读它们结局的断言在单测里。
const RECORD_TABLE_SELF_CHECKS: &[(&str, fn())] = &[
    (
        "scan_system_configuration_matches_the_generator",
        check_the_scan_system_configuration_matches_the_generator,
    ),
    (
        "table_and_disk_order_options",
        check_the_table_and_disk_order_options,
    ),
    (
        "reversed_memory_disk_reports_the_second_device_first",
        check_a_reversed_memory_disk_reports_the_second_device_first,
    ),
];

// ── 择根与重放合判一族（登记 `research/prompts/e165-r3-prereg.md` 5.3 至 5.5：K、K9、KW、KA；只在 `--profile r3 --family root_choice` 时跑）
//
// 被测对象是 `recover` 的 `Consult` 支里串起来的三步：`choose_root`、`scan_journal`、`replay_journal(…, true)`。
// 一个状态 = 内存盘上同时摆好根环（三个区域、每区 S 个槽）与 journal 环（两盘镜像）的一份盘；根与记录由合法历史生成，按持久顺序取崩溃点。
// 判据一侧只用本地常量与生成时记下的真相（哪个根槽里是哪条根、哪条记录有哪几份在盘上、哪几次用户发布已确认），不调 core 的判定函数。

/// 根环区域数、区域起点三个常量、区域归属、根槽宽、固定结构槽距、mkfs 的第 0 代根（登记第三节的回比表；
/// [`cross_check_root_choice_local_constants_against_the_implementation`] 逐个回比）。
const ROOT_CHOICE_REGION_COUNT: u64 = 3;
const ROOT_CHOICE_BASE_SLOT: u64 = 64;
const ROOT_CHOICE_PRIME_STEP: u64 = 3;
const ROOT_CHOICE_CHUNK_BYTES: u64 = 1 << 20;
const ROOT_CHOICE_REGION_DEVICES: [u32; 3] = [0, 1, 0];
const ROOT_CHOICE_ROOT_SLOT_BYTES: usize = 512;
const ROOT_CHOICE_FIXED_STRUCTURE_SLOT_SPACING_BYTES: u32 = 4096;
const ROOT_CHOICE_MKFS_INSTANCE: u32 = 0;
const ROOT_CHOICE_MKFS_CHECKPOINT_TXG: u64 = 0;
/// 暖机至多推几次空发布（D16 已定项 8「至多 R = 3 次」）；第一版实付 2 次是几何的结果，不是这里的常量。
const ROOT_CHOICE_WARM_UP_EMPTY_PUBLISHES_AT_MOST: usize = 3;
/// 每区槽数：K、K9、KA 取 8，KW 取 4（登记 5.3、第三节回比表）。
const ROOT_CHOICE_SLOTS_PER_REGION_WIDE: u64 = 8;
const ROOT_CHOICE_SLOTS_PER_REGION_NARROW: u64 = 4;
/// 内存盘根槽数组的长度：三个区域 × 每区至多 8 个槽。
const ROOT_CHOICE_ROOT_SLOTS_AT_MOST: usize = 24;
/// 内存盘 journal 记录数组的长度：环至多 64 槽。
const ROOT_CHOICE_RING_SLOTS_AT_MOST: usize = 64;

labelled_enumeration! {
    /// 判据表（登记 5.5）；阳性对照那段循环遍历它。
    enum RootChoiceCriterion {
        Kj0Defined => "KJ0",
        Kj1NotEarlierThanTheLastConfirmedPublish => "KJ1",
        Kj2NotEarlierThanAnySelfVerifiedRoot => "KJ2",
        Kj3LandsOnTheIdealRecovery => "KJ3",
    }
}

labelled_enumeration! {
    /// 登记 5.4 的四个域：K、K9、KW 是合法历史，KA 是不合法的对抗根环。
    enum RootChoiceDomain {
        BigRing => "K",
        SmallRingNine => "K9",
        WrappingRootRing => "KW",
        AdversarialRootRing => "KA",
    }
}

impl RootChoiceDomain {
    const fn ring(self) -> RingGeometry {
        match self {
            Self::BigRing | Self::WrappingRootRing | Self::AdversarialRootRing => RingGeometry::BIG,
            Self::SmallRingNine => RingGeometry::SMALL_NINE,
        }
    }

    const fn slots_per_region(self) -> u64 {
        match self {
            Self::BigRing | Self::SmallRingNine | Self::AdversarialRootRing => {
                ROOT_CHOICE_SLOTS_PER_REGION_WIDE
            }
            Self::WrappingRootRing => ROOT_CHOICE_SLOTS_PER_REGION_NARROW,
        }
    }

    /// 用户发布每次的记录条数（登记 5.3：k ∈ {1, 2}，KW 只取 1）。KA 没有历史。
    const fn user_publish_record_counts(self) -> &'static [usize] {
        match self {
            Self::BigRing | Self::SmallRingNine => &[1, 2],
            Self::WrappingRootRing => &[1],
            Self::AdversarialRootRing => &[],
        }
    }

    /// 合法历史一族判 KJ1、KJ3；KA 不是合法历史，只判 KJ0、KJ2（登记 5.5 最后一列、第一节读法）。
    const fn is_a_legal_history(self) -> bool {
        match self {
            Self::BigRing | Self::SmallRingNine | Self::WrappingRootRing => true,
            Self::AdversarialRootRing => false,
        }
    }

    /// 全界（登记 5.4：K M ≤ 3、U ≤ 4；K9 M ≤ 2、U ≤ 2；KW M = 1、U ≤ 14）与轻档（调试用，不是登记的界）。
    const fn largest_mounts(self, tier: BoundsTier) -> usize {
        match (self, tier) {
            (Self::BigRing, BoundsTier::SegmentOne | BoundsTier::SegmentTwo) => 3,
            (Self::BigRing, BoundsTier::Light) => 2,
            (Self::SmallRingNine, BoundsTier::SegmentOne | BoundsTier::SegmentTwo) => 2,
            (Self::SmallRingNine | Self::WrappingRootRing, BoundsTier::Light) => 1,
            (Self::WrappingRootRing, BoundsTier::SegmentOne | BoundsTier::SegmentTwo) => 1,
            (
                Self::AdversarialRootRing,
                BoundsTier::SegmentOne | BoundsTier::SegmentTwo | BoundsTier::Light,
            ) => 0,
        }
    }

    const fn largest_user_publishes(self, tier: BoundsTier) -> usize {
        match (self, tier) {
            (Self::BigRing, BoundsTier::SegmentOne | BoundsTier::SegmentTwo) => 4,
            (Self::BigRing | Self::SmallRingNine, BoundsTier::Light) => 2,
            (Self::SmallRingNine, BoundsTier::SegmentOne | BoundsTier::SegmentTwo) => 2,
            (Self::WrappingRootRing, BoundsTier::SegmentOne | BoundsTier::SegmentTwo) => 14,
            (Self::WrappingRootRing, BoundsTier::Light) => 6,
            (
                Self::AdversarialRootRing,
                BoundsTier::SegmentOne | BoundsTier::SegmentTwo | BoundsTier::Light,
            ) => 0,
        }
    }
}

/// 一个 (M, U) 格：M 次挂载、U 次用户发布（跨挂载的总数）。KA 没有挂载与发布，取 (0, 0)。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct RootChoiceCell {
    mounts: usize,
    user_publishes: usize,
}

/// 一个域在这一档、这一次命令行下要跑的格，按 (M 升序, U 升序)。`largest_user_publishes_cap` 是 `--max-u`（只给调试用）。
fn root_choice_cells_of(
    domain: RootChoiceDomain,
    tier: BoundsTier,
    largest_user_publishes_cap: Option<usize>,
) -> Vec<RootChoiceCell> {
    if !domain.is_a_legal_history() {
        return vec![RootChoiceCell {
            mounts: 0,
            user_publishes: 0,
        }];
    }
    let largest_user_publishes = largest_user_publishes_cap
        .map_or(domain.largest_user_publishes(tier), |cap| {
            cap.min(domain.largest_user_publishes(tier))
        });
    let mut cells = Vec::new();
    for mounts in 1..=domain.largest_mounts(tier) {
        for user_publishes in 0..=largest_user_publishes {
            cells.push(RootChoiceCell {
                mounts,
                user_publishes,
            });
        }
    }
    cells
}

/// 版本 = (`checkpoint_txg`, 实例代号)，txg 为主、实例代号破平局（D13 已定项 1 M3；与 D22 已定项 7 的择新序同一个序）。
/// 字段次序就是比较次序。
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
struct RootVersion {
    checkpoint_txg: u64,
    instance: u32,
}

impl RootVersion {
    fn of_root(root: &RootRecord) -> Self {
        Self {
            checkpoint_txg: root.checkpoint_txg.0,
            instance: root.instance.0,
        }
    }

    fn describe(self) -> String {
        format!("txg{}/instance{}", self.checkpoint_txg, self.instance)
    }
}

// ── 根环落点与暖机计划（本地算，不调 core；回比在 [`cross_check_root_choice_local_constants_against_the_implementation`]）

/// 本地算的区域起点：基槽 × 槽字节 + 区域号 × 步长 × chunk（D22 已定项 2；登记第三节回比表第 1 行）。
const fn root_choice_region_start_in_bytes(region: u64) -> u64 {
    ROOT_CHOICE_BASE_SLOT * LOCAL_SLOT_BYTES
        + region * ROOT_CHOICE_PRIME_STEP * ROOT_CHOICE_CHUNK_BYTES
}

/// 本地算的根槽落点 (区域, 槽)：区域 = txg mod 3、槽 = (txg div 3) mod S（登记第三节回比表第 2 行）。
const fn root_choice_target_of_publish(checkpoint_txg: u64, slots_per_region: u64) -> (u64, u64) {
    (
        checkpoint_txg % ROOT_CHOICE_REGION_COUNT,
        (checkpoint_txg / ROOT_CHOICE_REGION_COUNT) % slots_per_region,
    )
}

fn root_choice_slot_index_of(region: u64, slot: u64, slots_per_region: u64) -> usize {
    usize::try_from(region * slots_per_region + slot).expect("根槽数不超过 24")
}

/// 这次发布的根写进根槽数组的哪一格（区域号主序）。
fn root_choice_slot_index_of_publish(domain: RootChoiceDomain, checkpoint_txg: u64) -> usize {
    let slots_per_region = domain.slots_per_region();
    let (region, slot) = root_choice_target_of_publish(checkpoint_txg, slots_per_region);
    root_choice_slot_index_of(region, slot, slots_per_region)
}

fn root_choice_device_of_publish(checkpoint_txg: u64) -> usize {
    let region =
        usize::try_from(checkpoint_txg % ROOT_CHOICE_REGION_COUNT).expect("区域号不超过 2");
    usize::try_from(ROOT_CHOICE_REGION_DEVICES[region]).expect("盘号是 0 或 1")
}

/// 暖机计划（D16 已定项 8、登记 5.3 第 2 点）：写行那次发布取 `row_write_txg`，之后从它起逐个加一、看那个 txg 的根落哪块盘，
/// 推到本实例的根覆盖两块盘为止，空发布至多 [`ROOT_CHOICE_WARM_UP_EMPTY_PUBLISHES_AT_MOST`] 次。返回的第一个是写行那次发布。
fn root_choice_warm_up_plan(row_write_txg: u64) -> Vec<u64> {
    let mut covered = [false; 2];
    covered[root_choice_device_of_publish(row_write_txg)] = true;
    let mut plan = vec![row_write_txg];
    let mut checkpoint_txg = row_write_txg;
    while !(covered[0] && covered[1]) && plan.len() <= ROOT_CHOICE_WARM_UP_EMPTY_PUBLISHES_AT_MOST {
        checkpoint_txg += 1;
        plan.push(checkpoint_txg);
        covered[root_choice_device_of_publish(checkpoint_txg)] = true;
    }
    plan
}

/// 每个生成出来的根槽，本地算的 (区域, 槽)、偏移与 `root_ring::target_for_publish`、`root_ring::slot_offset` 相等（登记第三节回比表第 2 行；不符走 TH2）。
fn assert_the_local_root_slot_arithmetic_against_the_implementation(
    domain: RootChoiceDomain,
    checkpoint_txg: u64,
) {
    let slots_per_region = domain.slots_per_region();
    let (region, slot) = root_choice_target_of_publish(checkpoint_txg, slots_per_region);
    let implementation_target = singlefs_core::root_ring::target_for_publish(
        CheckpointTxg(checkpoint_txg),
        RootRingSlotsPerRegion::from_system_configuration_field(slots_per_region)
            .expect("每区槽数 4 或 8 在格式承诺的区间里"),
    );
    assert_eq!(
        (region, slot),
        (implementation_target.region, implementation_target.slot),
        "TH2：本地算的根槽 (区域, 槽) 与 root_ring::target_for_publish 不等（txg {checkpoint_txg}，每区 {slots_per_region} 槽）"
    );
    assert_eq!(
        root_choice_region_start_in_bytes(region)
            + slot * u64::from(ROOT_CHOICE_FIXED_STRUCTURE_SLOT_SPACING_BYTES),
        singlefs_core::root_ring::slot_offset(
            implementation_target,
            ROOT_CHOICE_FIXED_STRUCTURE_SLOT_SPACING_BYTES
        )
        .0,
        "TH2：本地算的根槽偏移与 root_ring::slot_offset 不等（txg {checkpoint_txg}）"
    );
}

// ── 一次发布的材料：它的根（字节与自证结果）与它的记录（每条一个槽）

/// 投影之外的四个字段：全历史取 r1 档那一组常量（登记 5.3 第 4 点）。
fn root_choice_fields_beyond_the_projection() -> FieldsBeyondTheProjection {
    RunProfile::R1.fields_beyond_the_projection_of_version(0)
}

/// 一次发布（含 mkfs 的第 0 代根与 KA 的根）的材料：生成一次、之后只借用。
#[derive(Debug)]
struct RootChoicePublishMaterial {
    version: RootVersion,
    fields: VersionFields,
    root_bytes: Vec<u8>,
    records: Vec<GeneratedSlot>,
    is_user_publish: bool,
}

/// 只有根、没有记录的材料：根过 `to_slot(512)` 再 `parse_slot` 必须等于原根，不等当场停（登记停机条款 TH9）。
fn root_material_without_records(
    instance: u32,
    checkpoint_txg: u64,
    segment: NewRootSegment,
) -> RootChoicePublishMaterial {
    let fields = VersionFields {
        instance,
        checkpoint_txg,
        segment,
    };
    let root = fields.to_root_with(root_choice_fields_beyond_the_projection());
    let root_bytes = root.to_slot(ROOT_CHOICE_ROOT_SLOT_BYTES);
    assert_eq!(
        RootRecord::parse_slot(&root_bytes, &ROOT_FILESYSTEM_IDENTIFIER),
        Some(root),
        "停机条款 TH9：生成的根过不了 to_slot(512) 再 parse_slot（实例 {instance}，txg {checkpoint_txg}）"
    );
    RootChoicePublishMaterial {
        version: RootVersion {
            checkpoint_txg,
            instance,
        },
        fields,
        root_bytes,
        records: Vec::new(),
        is_user_publish: false,
    }
}

/// mkfs 的第 0 代根：实例代号 0、txg 0，种进三个区域各自的槽 0（D22 已定项 8 第 3 条；登记第三节回比表）。
fn mkfs_root_material() -> RootChoicePublishMaterial {
    root_material_without_records(
        ROOT_CHOICE_MKFS_INSTANCE,
        ROOT_CHOICE_MKFS_CHECKPOINT_TXG,
        NewRootSegment::off_timeline(0, ROOT_CHOICE_MKFS_CHECKPOINT_TXG),
    )
}

struct RootChoicePublishPlan {
    domain: RootChoiceDomain,
    instance: u32,
    checkpoint_txg: u64,
    publish_ordinal: u64,
    first_counter: u64,
    new_pool_file_creation: u64,
    record_count: usize,
    commit_marker_bits: u32,
    /// 本实例写的上一条记录的 487 字节头算出的链值；本实例第一条记录没有，取 0（I-8.6）。
    previous_chain_value: Option<u32>,
    is_user_publish: bool,
}

/// 一次发布的材料：k 条记录（写行与暖机的空发布一条：事务号 0、提交标记 1、序号 1、带末条标志），新根段按全历史的发布序号取。
fn root_choice_publish_material(plan: &RootChoicePublishPlan) -> RootChoicePublishMaterial {
    assert_the_local_root_slot_arithmetic_against_the_implementation(
        plan.domain,
        plan.checkpoint_txg,
    );
    let segment = NewRootSegment::of_timeline_publish(plan.publish_ordinal, plan.checkpoint_txg);
    let mut material = root_material_without_records(plan.instance, plan.checkpoint_txg, segment);
    material.is_user_publish = plan.is_user_publish;
    let mut transaction = plan.new_pool_file_creation;
    let mut back_chain = plan.previous_chain_value.unwrap_or(0);
    for record_position in 0..plan.record_count {
        let is_last_record_of_the_publish = record_position + 1 == plan.record_count;
        let is_commit =
            is_last_record_of_the_publish || (plan.commit_marker_bits >> record_position) & 1 == 1;
        let counter = plan.first_counter
            + u64::try_from(record_position).expect("一次发布的记录条数不超过 2");
        assert!(
            counter <= plan.domain.ring().ring_slots,
            "停机条款 TH9：界内最大计数器 {counter} 超过环槽数 {}，journal 环回绕了",
            plan.domain.ring().ring_slots
        );
        let record = JournalRecord {
            instance: InstanceGeneration(plan.instance),
            counter,
            checkpoint_txg: CheckpointTxg(plan.checkpoint_txg),
            transaction: if plan.is_user_publish { transaction } else { 0 },
            is_commit,
            ordinal_within_publish: JournalRecordOrdinalWithinPublish(
                u32::try_from(record_position + 1).expect("一次发布的记录条数不超过 2"),
            ),
            place_in_publish: if is_last_record_of_the_publish {
                JournalRecordPlaceInPublish::LastRecordOfThePublish
            } else {
                JournalRecordPlaceInPublish::MoreRecordsOfThePublishFollow
            },
            back_chain,
            filesystem_identifier: FILESYSTEM_IDENTIFIER_LOW_EIGHT_BYTES,
            new_tree_table: segment.tree_table,
            new_mapping_root: segment.mapping_root,
            new_tree_identifier_watermark: segment.tree_identifier_watermark,
            new_rollback_floor: CheckpointTxg(segment.rollback_floor),
            new_instance_table: fields_beyond_the_projection_carried_by_every_generated_record()
                .instance_table,
            new_allocation_record_tree_root:
                fields_beyond_the_projection_carried_by_every_generated_record()
                    .allocation_record_tree_root,
            new_unmount_marker: fields_beyond_the_projection_carried_by_every_generated_record()
                .unmount_marker,
            named: Vec::new(),
        };
        let slot = generate_slot(
            0,
            plan.domain.ring(),
            &record,
            ParseExpectation::MustRoundTrip,
        );
        assert_eq!(
            VersionFields::of_record(&record),
            material.fields,
            "停机条款 TH9：记录头里的新根段与这次发布的根不是同一组六个字段"
        );
        back_chain = slot.chain_value_of_header;
        if plan.is_user_publish && is_commit {
            transaction += 1;
        }
        material.records.push(slot);
    }
    material
}

// ── 一段历史走到某一刻的盘（世界）、崩溃点、判据看到的盘面

#[derive(Clone, Debug)]
struct RootChoiceLandedRecord {
    material: Arc<RootChoicePublishMaterial>,
    record_index: usize,
    counter: u64,
    /// 0 号盘、1 号盘上这条记录各有没有一份（任一份自证过即算在，D23 已定项 14）。
    copies_on_devices: [bool; 2],
}

#[derive(Clone, Debug)]
struct RootChoiceWorld {
    domain: RootChoiceDomain,
    /// 根环三个区域的槽，区域号主序、区域内槽号升序（与 `every_root_ring_slot` 同一个次序）；没写过的槽是 `None`（读成 512 个 0）。
    root_slots: Vec<Option<Arc<RootChoicePublishMaterial>>>,
    /// 盘上至少有一份的记录，按写的次序。
    landed_records: Vec<RootChoiceLandedRecord>,
    /// 崩溃点所在那次发布之前、fsync 已返回的用户发布，按写的次序（D16 已定项 7；写行与暖机的空发布不是用户的 fsync）。
    confirmed_user_publishes: Vec<RootVersion>,
    publishes_started: u64,
    /// 前面各次挂载的写法（见证用）。
    history: Vec<String>,
}

impl RootChoiceWorld {
    /// mkfs 之后、一次挂载都还没有：第 0 代根在三个区域的槽 0，别处全 0，journal 环空（登记 5.3「mkfs」）。
    fn right_after_mkfs(domain: RootChoiceDomain) -> Self {
        let genesis_root = Arc::new(mkfs_root_material());
        let slots_per_region = domain.slots_per_region();
        let mut root_slots = vec![
            None;
            usize::try_from(ROOT_CHOICE_REGION_COUNT * slots_per_region)
                .expect("根槽数不超过 24")
        ];
        for region in 0..ROOT_CHOICE_REGION_COUNT {
            root_slots[root_choice_slot_index_of(region, 0, slots_per_region)] =
                Some(Arc::clone(&genesis_root));
        }
        Self {
            domain,
            root_slots,
            landed_records: Vec::new(),
            confirmed_user_publishes: Vec::new(),
            publishes_started: 0,
            history: Vec::new(),
        }
    }

    /// 根环里现存的根与盘上读得出的记录里最大的 txg（新实例写行那次发布的 txg 取它加一，D23 已定项 14 注 3）。
    fn highest_checkpoint_txg_on_disk(&self) -> u64 {
        let highest_root_txg = self
            .root_slots
            .iter()
            .flatten()
            .map(|material| material.version.checkpoint_txg)
            .max()
            .unwrap_or(0);
        let highest_record_txg = self
            .landed_records
            .iter()
            .map(|landed| landed.material.version.checkpoint_txg)
            .max()
            .unwrap_or(0);
        highest_root_txg.max(highest_record_txg)
    }

    /// 盘上读得出的最大计数器（新实例第一条记录的计数器取它加一；没有记录时取 0，于是第一条是 1）。
    fn highest_landed_counter(&self) -> u64 {
        self.landed_records
            .iter()
            .map(|landed| landed.counter)
            .max()
            .unwrap_or(0)
    }

    fn landed_record_of(
        material: &Arc<RootChoicePublishMaterial>,
        record_index: usize,
        copies_on_devices: [bool; 2],
    ) -> RootChoiceLandedRecord {
        RootChoiceLandedRecord {
            material: Arc::clone(material),
            record_index,
            counter: material.records[record_index]
                .parsed
                .as_ref()
                .expect("合法历史的每条记录都解得出（生成时断言过）")
                .counter,
            copies_on_devices,
        }
    }

    /// 这次发布整次落了：每条记录两盘各一份，根写进它的落点（盖掉那个槽原来的根）。`is_confirmed` 是它之后 fsync 返回了（用户发布才会）。
    fn with_publish_applied(
        &self,
        material: &Arc<RootChoicePublishMaterial>,
        is_confirmed: bool,
    ) -> Self {
        let mut world = self.clone();
        for record_index in 0..material.records.len() {
            world
                .landed_records
                .push(Self::landed_record_of(material, record_index, [true, true]));
        }
        world.root_slots
            [root_choice_slot_index_of_publish(self.domain, material.version.checkpoint_txg)] =
            Some(Arc::clone(material));
        if is_confirmed {
            world.confirmed_user_publishes.push(material.version);
        }
        world.publishes_started += 1;
        world
    }

    /// 这次发布崩在 `crashed.crash_point` 上之后的盘；`trace` 是这次挂载到这里为止的写法，连同崩溃点记进历史（见证用）。
    fn with_crash_applied(&self, crashed: &RootChoiceCrashedPublish<'_>, trace: &str) -> Self {
        let mut world = self.clone();
        match crashed.crash_point {
            RootChoiceCrashPoint::RecordCopiesLanded { copy_mask } => {
                for record_index in 0..crashed.material.records.len() {
                    let copies_on_devices = copies_of_a_record_in_a_mask(copy_mask, record_index);
                    if copies_on_devices[0] || copies_on_devices[1] {
                        world.landed_records.push(Self::landed_record_of(
                            crashed.material,
                            record_index,
                            copies_on_devices,
                        ));
                    }
                }
            }
            RootChoiceCrashPoint::RootLanded => {
                for record_index in 0..crashed.material.records.len() {
                    world.landed_records.push(Self::landed_record_of(
                        crashed.material,
                        record_index,
                        [true, true],
                    ));
                }
                world.root_slots[root_choice_slot_index_of_publish(
                    self.domain,
                    crashed.material.version.checkpoint_txg,
                )] = Some(Arc::clone(crashed.material));
            }
        }
        world.publishes_started += 1;
        world
            .history
            .push(format!("{trace};{}", crashed.describe()));
        world
    }
}

/// 一次发布的崩溃点（登记 5.3 第 6 点，D13 已定项 4 按设备切段）：记录那一段 2k 份拷贝的任一子集落了、根没落（4^k 个，位 2j 是第 j 条的
/// 0 号盘那一份、位 2j + 1 是 1 号盘那一份），或根落了（1 个：系统配置轮换那一段的各个子集在三步眼里是同一块盘，并成这一个）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum RootChoiceCrashPoint {
    RecordCopiesLanded { copy_mask: u32 },
    RootLanded,
}

fn copies_of_a_record_in_a_mask(copy_mask: u32, record_index: usize) -> [bool; 2] {
    [
        (copy_mask >> (2 * record_index)) & 1 == 1,
        (copy_mask >> (2 * record_index + 1)) & 1 == 1,
    ]
}

fn root_choice_crash_points_of(record_count: usize) -> Vec<RootChoiceCrashPoint> {
    let mut crash_points: Vec<RootChoiceCrashPoint> = (0..(1u32 << (2 * record_count)))
        .map(|copy_mask| RootChoiceCrashPoint::RecordCopiesLanded { copy_mask })
        .collect();
    crash_points.push(RootChoiceCrashPoint::RootLanded);
    crash_points
}

struct RootChoiceCrashedPublish<'material> {
    material: &'material Arc<RootChoicePublishMaterial>,
    crash_point: RootChoiceCrashPoint,
}

impl RootChoiceCrashedPublish<'_> {
    fn describe(&self) -> String {
        let crash_point_text = match self.crash_point {
            RootChoiceCrashPoint::RecordCopiesLanded { copy_mask } => {
                format!("record_copies_mask_{copy_mask:b}_root_not_landed")
            }
            RootChoiceCrashPoint::RootLanded => "root_landed".to_string(),
        };
        format!(
            "crash_in_publish_txg{}_instance{}_{}records_{crash_point_text}",
            self.material.version.checkpoint_txg,
            self.material.version.instance,
            self.material.records.len()
        )
    }
}

/// 判据与三步调用看到的盘面：每个根槽里是哪条根（读根环次序），盘上至少有一份的每条记录与它在哪几块盘上。
struct RootChoiceLandedRecordRef<'world> {
    material: &'world RootChoicePublishMaterial,
    record_index: usize,
    copies_on_devices: [bool; 2],
}

struct RootChoiceLanding<'world> {
    roots_in_ring_order: Vec<Option<&'world RootChoicePublishMaterial>>,
    records: Vec<RootChoiceLandedRecordRef<'world>>,
}

/// 世界加上崩溃在途的那次发布（没有就是世界本身）落在盘上的样子。
fn root_choice_landing_of<'world>(
    world: &'world RootChoiceWorld,
    crashed: Option<&'world RootChoiceCrashedPublish<'world>>,
) -> RootChoiceLanding<'world> {
    let mut roots_in_ring_order: Vec<Option<&RootChoicePublishMaterial>> = world
        .root_slots
        .iter()
        .map(|slot| slot.as_deref())
        .collect();
    let mut records: Vec<RootChoiceLandedRecordRef<'_>> = world
        .landed_records
        .iter()
        .map(|landed| RootChoiceLandedRecordRef {
            material: &landed.material,
            record_index: landed.record_index,
            copies_on_devices: landed.copies_on_devices,
        })
        .collect();
    if let Some(crashed) = crashed {
        match crashed.crash_point {
            RootChoiceCrashPoint::RecordCopiesLanded { copy_mask } => {
                for record_index in 0..crashed.material.records.len() {
                    let copies_on_devices = copies_of_a_record_in_a_mask(copy_mask, record_index);
                    if copies_on_devices[0] || copies_on_devices[1] {
                        records.push(RootChoiceLandedRecordRef {
                            material: crashed.material,
                            record_index,
                            copies_on_devices,
                        });
                    }
                }
            }
            RootChoiceCrashPoint::RootLanded => {
                for record_index in 0..crashed.material.records.len() {
                    records.push(RootChoiceLandedRecordRef {
                        material: crashed.material,
                        record_index,
                        copies_on_devices: [true, true],
                    });
                }
                roots_in_ring_order[root_choice_slot_index_of_publish(
                    world.domain,
                    crashed.material.version.checkpoint_txg,
                )] = Some(crashed.material);
            }
        }
    }
    RootChoiceLanding {
        roots_in_ring_order,
        records,
    }
}

// ── 内存盘：根槽（512 字节）与 journal 记录槽（4096 字节）摆在各自的落点上

/// 三步调用的读者：两块盘；根环没写过的槽读成 512 个 0（mkfs 清零，登记 5.5 第一段）；journal 环没落盘的记录槽读成 4096 个 0；别处读不出。
struct RootChoiceDisk<'landing> {
    ring: RingGeometry,
    slots_per_region: u64,
    root_slot_bytes: [Option<&'landing [u8]>; ROOT_CHOICE_ROOT_SLOTS_AT_MOST],
    record_bytes: [[Option<&'landing [u8]>; ROOT_CHOICE_RING_SLOTS_AT_MOST]; 2],
}

impl<'landing> RootChoiceDisk<'landing> {
    fn of_the_landing(domain: RootChoiceDomain, landing: &RootChoiceLanding<'landing>) -> Self {
        let mut root_slot_bytes: [Option<&[u8]>; ROOT_CHOICE_ROOT_SLOTS_AT_MOST] =
            [None; ROOT_CHOICE_ROOT_SLOTS_AT_MOST];
        for (slot_index, root) in landing.roots_in_ring_order.iter().enumerate() {
            root_slot_bytes[slot_index] = root.map(|material| material.root_bytes.as_slice());
        }
        let mut record_bytes: [[Option<&[u8]>; ROOT_CHOICE_RING_SLOTS_AT_MOST]; 2] =
            [[None; ROOT_CHOICE_RING_SLOTS_AT_MOST]; 2];
        for record in &landing.records {
            let slot = &record.material.records[record.record_index];
            let ring_index = usize::try_from(
                (slot
                    .parsed
                    .as_ref()
                    .expect("合法历史的每条记录都解得出（生成时断言过）")
                    .counter
                    - LOCAL_FIRST_JOURNAL_COUNTER)
                    % domain.ring().ring_slots,
            )
            .expect("环槽数不超过 64");
            for (device_index, has_a_copy) in record.copies_on_devices.iter().enumerate() {
                if *has_a_copy {
                    assert!(
                        record_bytes[device_index][ring_index].is_none(),
                        "停机条款 TH9：两条记录落在同一个环槽里，journal 环回绕了"
                    );
                    record_bytes[device_index][ring_index] = Some(slot.bytes.as_slice());
                }
            }
        }
        Self {
            ring: domain.ring(),
            slots_per_region: domain.slots_per_region(),
            root_slot_bytes,
            record_bytes,
        }
    }
}

impl PoolReader for RootChoiceDisk<'_> {
    fn device_identities(&self) -> Vec<DeviceIdentity> {
        vec![DeviceIdentity(0), DeviceIdentity(1)]
    }

    fn device_size_in_bytes(&self, device: DeviceIdentity) -> Option<u64> {
        (device.0 < 2).then_some(MEMORY_DISK_DEVICE_BYTES)
    }

    fn read(
        &self,
        device: DeviceIdentity,
        offset: DeviceOffsetInBytes,
        length: usize,
    ) -> Option<Vec<u8>> {
        for region in 0..ROOT_CHOICE_REGION_COUNT {
            let region_start = root_choice_region_start_in_bytes(region);
            let spacing = u64::from(ROOT_CHOICE_FIXED_STRUCTURE_SLOT_SPACING_BYTES);
            if offset.0 < region_start || offset.0 >= region_start + self.slots_per_region * spacing
            {
                continue;
            }
            let offset_in_region = offset.0 - region_start;
            let region_device =
                ROOT_CHOICE_REGION_DEVICES[usize::try_from(region).expect("区域号不超过 2")];
            if !offset_in_region.is_multiple_of(spacing)
                || device.0 != region_device
                || length != ROOT_CHOICE_ROOT_SLOT_BYTES
            {
                return None;
            }
            let slot_index = root_choice_slot_index_of(
                region,
                offset_in_region / spacing,
                self.slots_per_region,
            );
            return Some(match self.root_slot_bytes[slot_index] {
                Some(bytes) => bytes.get(..length)?.to_vec(),
                None => vec![0u8; length],
            });
        }
        let ring_start = RingGeometry::ring_start_offset_in_bytes();
        let is_a_whole_record_slot_of_the_ring = device.0 < 2
            && offset.0 >= ring_start
            && offset.0 < ring_start + self.ring.ring_bytes()
            && (offset.0 - ring_start).is_multiple_of(LOCAL_JOURNAL_RECORD_BYTES)
            && u64::try_from(length).ok() == Some(LOCAL_JOURNAL_RECORD_BYTES);
        if !is_a_whole_record_slot_of_the_ring {
            return None;
        }
        let ring_index = usize::try_from((offset.0 - ring_start) / LOCAL_JOURNAL_RECORD_BYTES)
            .expect("环槽数不超过 64");
        let device_index = usize::try_from(device.0).expect("盘号是 0 或 1");
        Some(match self.record_bytes[device_index][ring_index] {
            Some(bytes) => bytes.get(..length)?.to_vec(),
            None => vec![0u8; length],
        })
    }

    fn journal_record_offsets_hint(
        &self,
        _device: DeviceIdentity,
        _ring_start: DeviceOffsetInBytes,
        _ring_bytes: u64,
    ) -> Option<Vec<DeviceOffsetInBytes>> {
        None
    }
}

/// 喂给三步的系统配置：fsid 取时间线根的那个 16 字节常量，区域归属 0 / 1 / 0，根槽宽 512、槽距 4096、每区 S 槽、journal 环长与起点 1024
/// （`choose_root`、`scan_journal` 不读的字段同 r2 修订 4 那一份的取法）。
fn system_configuration_for_the_root_choice(domain: RootChoiceDomain) -> SystemConfiguration {
    SystemConfiguration {
        immutable: SystemImmutableConfiguration {
            filesystem_identifier: ROOT_FILESYSTEM_IDENTIFIER,
            this_device: DeviceIdentity(0),
            device_count: SCAN_CONFIGURATION_DEVICE_COUNT,
            region_devices: [DeviceIdentity(0), DeviceIdentity(1), DeviceIdentity(0)],
            sizes: SystemImmutableSizes {
                physical_block_size: u32::try_from(ROOT_CHOICE_ROOT_SLOT_BYTES)
                    .expect("512 装得进 32 位"),
                minimum_input_output_bytes: u32::try_from(ROOT_CHOICE_ROOT_SLOT_BYTES)
                    .expect("512 装得进 32 位"),
                fixed_structure_slot_spacing: ROOT_CHOICE_FIXED_STRUCTURE_SLOT_SPACING_BYTES,
                journal_ring_bytes: domain.ring().ring_bytes(),
                root_ring_slots_per_region:
                    RootRingSlotsPerRegion::from_system_configuration_field(
                        domain.slots_per_region(),
                    )
                    .expect("每区槽数 4 或 8 在格式承诺的区间里"),
            },
            journal_ring_start_slot: SlotNumber(LOCAL_JOURNAL_RING_START_SLOT),
            root_ring_base_slot: SlotNumber(
                SCAN_CONFIGURATION_ROOT_RING_BASE_SLOT_THE_SCAN_DOES_NOT_READ,
            ),
        },
        mutable: SystemMutableConfiguration,
        runtime: SystemRuntimeConfiguration,
        quantities: SystemRuntimeQuantities {
            slot_generation: SCAN_CONFIGURATION_SLOT_GENERATION_THE_SCAN_DOES_NOT_READ,
            journal_tail: SCAN_CONFIGURATION_JOURNAL_TAIL_THE_SCAN_DOES_NOT_READ,
            journal_instance: InstanceGeneration(CHOSEN_INSTANCE),
            rollback_floor: CheckpointTxg(SCAN_CONFIGURATION_ROLLBACK_FLOOR_THE_SCAN_DOES_NOT_READ),
        },
    }
}

// ── 理想恢复与真相（登记 5.5 KJ3；判据一侧不调 core）

/// 理想恢复（D13 已定项 1 M4「恰是 j*」，所选根换成给定的那条根）：从它起沿它的实例逐次往后，每次发布的每条记录至少一份在盘上、
/// 条数累计不超在飞上限、第一次的 txg 是它的 txg + 1（D23 已定项 14 注 1 链首），走得最远的那一版。
struct RootChoiceIdealRecovery {
    landed_version: RootVersion,
    landed_fields: VersionFields,
    applied_publishes: usize,
}

fn root_choice_ideal_recovery_from(
    start: &RootChoicePublishMaterial,
    records: &[RootChoiceLandedRecordRef<'_>],
    in_flight_limit: usize,
) -> RootChoiceIdealRecovery {
    let mut recovery = RootChoiceIdealRecovery {
        landed_version: start.version,
        landed_fields: start.fields,
        applied_publishes: 0,
    };
    let mut records_applied = 0usize;
    let mut next_checkpoint_txg = start.version.checkpoint_txg;
    while let Some(following_txg) = next_checkpoint_txg.checked_add(1) {
        let mut publish: Option<&RootChoicePublishMaterial> = None;
        let mut landed_records_of_the_publish = 0usize;
        for record in records {
            if record.material.version.instance == start.version.instance
                && record.material.version.checkpoint_txg == following_txg
            {
                publish = Some(record.material);
                landed_records_of_the_publish += 1;
            }
        }
        let Some(publish) = publish else {
            break;
        };
        if landed_records_of_the_publish != publish.records.len()
            || records_applied + landed_records_of_the_publish > in_flight_limit
        {
            break;
        }
        records_applied += landed_records_of_the_publish;
        recovery.landed_version = publish.version;
        recovery.landed_fields = publish.fields;
        recovery.applied_publishes += 1;
        next_checkpoint_txg = following_txg;
    }
    recovery
}

/// 真相里根环每个槽现存的根（读根环次序），自证过的才有。
fn root_choice_versions_in_ring_order(landing: &RootChoiceLanding<'_>) -> Vec<Option<RootVersion>> {
    landing
        .roots_in_ring_order
        .iter()
        .map(|root| root.map(|material| material.version))
        .collect()
}

/// 版本最大的那一条根（真相里；同版本的几条取读根环次序里最前的，版本相同的根字节相同、哪条都一样）。
fn root_choice_newest_root<'world>(
    landing: &RootChoiceLanding<'world>,
) -> Option<&'world RootChoicePublishMaterial> {
    let mut newest: Option<&RootChoicePublishMaterial> = None;
    for root in landing.roots_in_ring_order.iter().copied().flatten() {
        if newest.is_none_or(|current| root.version > current.version) {
            newest = Some(root);
        }
    }
    newest
}

fn root_choice_oldest_root<'world>(
    landing: &RootChoiceLanding<'world>,
) -> Option<&'world RootChoicePublishMaterial> {
    let mut oldest: Option<&RootChoicePublishMaterial> = None;
    for root in landing.roots_in_ring_order.iter().copied().flatten() {
        if oldest.is_none_or(|current| root.version < current.version) {
            oldest = Some(root);
        }
    }
    oldest
}

// ── 数的量与见证

labelled_enumeration! {
    /// 每个 (域, M, U) 格上数的量（登记 5.5 末段）。违例类的量在第一次加一时记一个见证。
    enum RootChoiceCounter {
        States => "states",
        ConfirmedStates => "confirmed_states",
        FirstSelfVerifiedRootIsNotTheNewest => "first_self_verified_root_is_not_the_newest",
        SameTxgDifferentInstanceRoots => "same_txg_different_instance_roots",
        HigherInstanceLowerTxgRoots => "higher_instance_lower_txg_roots",
        TwoOrMoreInstancesInTheRing => "two_or_more_instances",
        IdealRecoveryApplied => "ideal_recovery_applied",
        AboveNewestRootOverLimit => "above_newest_root_over_limit",
        AboveNewestRootAtLimit => "above_newest_root_at_limit",
        AboveChosenRootOverLimit => "above_chosen_root_over_limit",
        AboveChosenRootAtLimit => "above_chosen_root_at_limit",
        StatesWithoutASelfVerifiedRoot => "states_without_a_self_verified_root",
        PanickedStates => "panicked",
        ChoseNothingStates => "choose_root_returned_none",
        ReplayFailedStates => "replay_failed",
        ReplayAppliedStates => "replay_applied",
        Kj0Violations => "KJ0",
        Kj1Judged => "kj1_judged",
        Kj1Violations => "KJ1",
        Kj2Judged => "kj2_judged",
        Kj2Violations => "KJ2",
        Kj3Judged => "kj3_judged",
        Kj3Violations => "KJ3",
        SlotReadingsChecked => "slot_readings_checked",
        SlotReadingsDifferingFromTheTruth => "slot_readings_differing_from_the_truth",
    }
}

#[derive(Clone, Debug)]
struct RootChoiceTally {
    counts: Vec<u64>,
    witnesses: Vec<Option<String>>,
    /// 真相里版本最大的根之上、同实例的记录条数的最大值（登记第八节）。
    above_newest_root_maximum: usize,
    /// 实现所选的根之上、同实例的记录条数的最大值（登记第八节：变体择到更旧的根时在飞上限可能咬到）。
    above_chosen_root_maximum: usize,
    /// 已确认的用户发布条数的最大值（登记第八节）。
    confirmed_publishes_maximum: usize,
    /// 盘上读得出的最大计数器。
    largest_counter: u64,
}

impl RootChoiceTally {
    fn new() -> Self {
        Self {
            counts: vec![0; RootChoiceCounter::ALL.len()],
            witnesses: vec![None; RootChoiceCounter::ALL.len()],
            above_newest_root_maximum: 0,
            above_chosen_root_maximum: 0,
            confirmed_publishes_maximum: 0,
            largest_counter: 0,
        }
    }

    fn add(&mut self, counter: RootChoiceCounter) {
        self.counts[counter.index()] += 1;
    }

    /// 违例加一；这个量的第一条违例记一个见证（枚举次序里最先的那个，合并时按工作单元的次序保留最先的）。
    fn add_violation(&mut self, counter: RootChoiceCounter, describe_witness: &dyn Fn() -> String) {
        let index = counter.index();
        self.counts[index] += 1;
        if self.witnesses[index].is_none() {
            self.witnesses[index] = Some(describe_witness());
        }
    }

    fn get(&self, counter: RootChoiceCounter) -> u64 {
        self.counts[counter.index()]
    }

    fn witness(&self, counter: RootChoiceCounter) -> &str {
        self.witnesses[counter.index()].as_deref().unwrap_or("none")
    }

    fn merge(&mut self, later: &Self) {
        for (count, later_count) in self.counts.iter_mut().zip(&later.counts) {
            *count += later_count;
        }
        for (witness, later_witness) in self.witnesses.iter_mut().zip(&later.witnesses) {
            if witness.is_none() {
                witness.clone_from(later_witness);
            }
        }
        self.above_newest_root_maximum = self
            .above_newest_root_maximum
            .max(later.above_newest_root_maximum);
        self.above_chosen_root_maximum = self
            .above_chosen_root_maximum
            .max(later.above_chosen_root_maximum);
        self.confirmed_publishes_maximum = self
            .confirmed_publishes_maximum
            .max(later.confirmed_publishes_maximum);
        self.largest_counter = self.largest_counter.max(later.largest_counter);
    }
}

/// 把工作单元分给几条线程跑，再按工作单元的次序合并（同 [`tally_of_work_units`]，计数的种类不同）。
fn root_choice_tally_of_work_units<Unit: Sync>(
    units: &[Unit],
    thread_count: usize,
    work: &(dyn Fn(usize, &Unit) -> RootChoiceTally + Sync),
) -> RootChoiceTally {
    let next_unit_index = AtomicUsize::new(0);
    let finished_units: Mutex<Vec<(usize, RootChoiceTally)>> = Mutex::new(Vec::new());
    std::thread::scope(|scope| {
        for _worker in 0..thread_count.clamp(1, units.len().max(1)) {
            scope.spawn(|| loop {
                let unit_index = next_unit_index.fetch_add(1, Ordering::Relaxed);
                let Some(unit) = units.get(unit_index) else {
                    break;
                };
                let tally = work(unit_index, unit);
                finished_units
                    .lock()
                    .expect("别的线程没有在拿着这把锁的时候 panic")
                    .push((unit_index, tally));
            });
        }
    });
    let mut finished_units = finished_units
        .into_inner()
        .expect("线程都已经收回，没有谁在拿着这把锁的时候 panic");
    finished_units.sort_by_key(|(unit_index, _tally)| *unit_index);
    assert_eq!(finished_units.len(), units.len(), "每个工作单元恰好跑一遍");
    let mut total = RootChoiceTally::new();
    for (_unit_index, tally) in &finished_units {
        total.merge(tally);
    }
    total
}

// ── 被判的一方：今天的三步，或阳性对照的假结局（登记 5.7）

/// KA 上本地写的几种择根序（登记 5.7 第四行、9.1 四行替换文的比较式各写一份；落点 = 择到的根，journal 环空）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum LocalRootChoiceRule {
    /// 今天的序：txg 大者赢，txg 相等实例代号大者赢，键完全相同取读根环次序里前面的。
    NewestWins,
    /// 变异 1825：txg 平局时实例代号低者赢。
    TieTakesTheLowerInstance,
    /// 变异 1826：实例代号为主、txg 只破平局。
    InstanceFirstThenTxg,
    /// 变异 1823：比较不看实例代号。
    IgnoresTheInstance,
    /// 变异 1824：取第一个自证过的槽、不再换。
    FirstSelfVerifiedWins,
}

impl LocalRootChoiceRule {
    const ALL: [Self; 5] = [
        Self::NewestWins,
        Self::TieTakesTheLowerInstance,
        Self::InstanceFirstThenTxg,
        Self::IgnoresTheInstance,
        Self::FirstSelfVerifiedWins,
    ];

    const fn label(self) -> &'static str {
        match self {
            Self::NewestWins => "newest_wins",
            Self::TieTakesTheLowerInstance => "tie_takes_the_lower_instance",
            Self::InstanceFirstThenTxg => "instance_first_then_txg",
            Self::IgnoresTheInstance => "ignores_the_instance",
            Self::FirstSelfVerifiedWins => "first_self_verified_wins",
        }
    }

    /// 读根环次序里往后走，`candidate` 换掉已择的 `chosen` 吗。
    fn replaces(self, candidate: RootVersion, chosen: RootVersion) -> bool {
        match self {
            Self::NewestWins => {
                candidate.checkpoint_txg > chosen.checkpoint_txg
                    || (candidate.checkpoint_txg == chosen.checkpoint_txg
                        && candidate.instance > chosen.instance)
            }
            Self::TieTakesTheLowerInstance => {
                candidate.checkpoint_txg > chosen.checkpoint_txg
                    || (candidate.checkpoint_txg == chosen.checkpoint_txg
                        && candidate.instance < chosen.instance)
            }
            Self::InstanceFirstThenTxg => {
                candidate.instance > chosen.instance
                    || (candidate.instance == chosen.instance
                        && candidate.checkpoint_txg > chosen.checkpoint_txg)
            }
            Self::IgnoresTheInstance => candidate.checkpoint_txg > chosen.checkpoint_txg,
            Self::FirstSelfVerifiedWins => false,
        }
    }

    fn chooses_among(self, versions_in_ring_order: &[Option<RootVersion>]) -> Option<RootVersion> {
        let mut chosen: Option<RootVersion> = None;
        for candidate in versions_in_ring_order.iter().copied().flatten() {
            if chosen.is_none_or(|already_chosen| self.replaces(candidate, already_chosen)) {
                chosen = Some(candidate);
            }
        }
        chosen
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum RootChoiceImplementation {
    /// 被测对象：今天的 `choose_root`、`scan_journal`、`replay_journal(…, true)`，不改一个字。
    TodayThreeSteps,
    /// KJ0 的对照：在指定的一态上 panic，别的态照真结局。
    PanicsOnTheDesignatedState,
    /// KF1：所选根取真相里版本最小的根，落点照理想恢复从它接（不调实现）。
    ChoosesTheOldestRoot,
    /// KF2：有已确认发布的态，落点取最后一次已确认那一版的前一版（同实例、txg 减一）；没有已确认的态照理想恢复。
    LandsOneVersionBeforeTheLastConfirmed,
    /// KA 上本地写的一种择根序，不调实现。
    LocalChoice(LocalRootChoiceRule),
}

#[derive(Debug)]
enum RootChoiceOutcome {
    Panicked {
        step: &'static str,
    },
    ChoseNothing,
    Landed {
        chosen: RootVersion,
        landed_version: RootVersion,
        landed_fields: VersionFields,
        /// `replay_journal` 交回 `Err`：落到的版本取所选根的（登记第一节读法）。
        replay_failed: bool,
    },
}

// ── 判一个状态（登记 5.5：KJ0、KJ1、KJ2、KJ3 各报各的）

/// 一个状态判之前先算好的真相：根环里每个槽的版本、版本最大的根、最后一次已确认的版本、理想恢复。
struct RootChoiceStateView<'view, 'world> {
    landing: &'view RootChoiceLanding<'world>,
    confirmed_user_publishes: &'view [RootVersion],
    versions_in_ring_order: Vec<Option<RootVersion>>,
    newest_root: Option<&'world RootChoicePublishMaterial>,
    last_confirmed: Option<RootVersion>,
    ideal_recovery_from_the_newest_root: Option<RootChoiceIdealRecovery>,
}

fn counter_of_the_landed_record(record: &RootChoiceLandedRecordRef<'_>) -> u64 {
    record.material.records[record.record_index]
        .parsed
        .as_ref()
        .expect("合法历史的每条记录都解得出（生成时断言过）")
        .counter
}

fn describe_the_roots_in_ring_order(
    versions_in_ring_order: &[Option<RootVersion>],
    slots_per_region: u64,
) -> String {
    let mut parts = Vec::new();
    for (slot_index, version) in versions_in_ring_order.iter().enumerate() {
        if let Some(version) = version {
            let slot_number = u64::try_from(slot_index).expect("根槽数不超过 24");
            parts.push(format!(
                "region{}slot{}={}",
                slot_number / slots_per_region,
                slot_number % slots_per_region,
                version.describe()
            ));
        }
    }
    parts.join("+")
}

struct RootChoiceJudgement {
    domain: RootChoiceDomain,
    implementation: RootChoiceImplementation,
    system_configuration: SystemConfiguration,
    in_flight_limit: usize,
}

impl RootChoiceJudgement {
    fn new(domain: RootChoiceDomain, implementation: RootChoiceImplementation) -> Self {
        Self {
            domain,
            implementation,
            system_configuration: system_configuration_for_the_root_choice(domain),
            in_flight_limit: domain.ring().local_in_flight_limit(),
        }
    }

    fn judge(
        &self,
        landing: &RootChoiceLanding<'_>,
        confirmed_user_publishes: &[RootVersion],
        is_the_designated_state: bool,
        tally: &mut RootChoiceTally,
        describe_state: &dyn Fn() -> String,
    ) {
        tally.add(RootChoiceCounter::States);
        let newest_root = root_choice_newest_root(landing);
        let view = RootChoiceStateView {
            landing,
            confirmed_user_publishes,
            versions_in_ring_order: root_choice_versions_in_ring_order(landing),
            newest_root,
            last_confirmed: confirmed_user_publishes.iter().copied().max(),
            ideal_recovery_from_the_newest_root: newest_root.map(|newest| {
                root_choice_ideal_recovery_from(newest, &landing.records, self.in_flight_limit)
            }),
        };
        self.count_the_shape_of_the_state(&view, tally);
        let outcome = self.outcome_of(&view, is_the_designated_state, tally, describe_state);
        self.judge_the_outcome(&view, &outcome, tally, describe_state);
    }

    fn count_the_shape_of_the_state(
        &self,
        view: &RootChoiceStateView<'_, '_>,
        tally: &mut RootChoiceTally,
    ) {
        let present_versions: Vec<RootVersion> = view
            .versions_in_ring_order
            .iter()
            .copied()
            .flatten()
            .collect();
        if let (Some(first_self_verified), Some(newest)) =
            (present_versions.first(), view.newest_root)
        {
            if *first_self_verified != newest.version {
                tally.add(RootChoiceCounter::FirstSelfVerifiedRootIsNotTheNewest);
            }
        }
        let mut has_same_txg_different_instance = false;
        let mut has_higher_instance_lower_txg = false;
        for earlier in &present_versions {
            for later in &present_versions {
                has_same_txg_different_instance |= earlier.checkpoint_txg == later.checkpoint_txg
                    && earlier.instance != later.instance;
                has_higher_instance_lower_txg |= earlier.instance > later.instance
                    && earlier.checkpoint_txg < later.checkpoint_txg;
            }
        }
        if has_same_txg_different_instance {
            tally.add(RootChoiceCounter::SameTxgDifferentInstanceRoots);
        }
        if has_higher_instance_lower_txg {
            tally.add(RootChoiceCounter::HigherInstanceLowerTxgRoots);
        }
        let mut instances: Vec<u32> = present_versions
            .iter()
            .map(|version| version.instance)
            .collect();
        instances.sort_unstable();
        instances.dedup();
        if instances.len() >= 2 {
            tally.add(RootChoiceCounter::TwoOrMoreInstancesInTheRing);
        }
        if view.newest_root.is_none() {
            tally.add(RootChoiceCounter::StatesWithoutASelfVerifiedRoot);
        }
        if view.last_confirmed.is_some() {
            tally.add(RootChoiceCounter::ConfirmedStates);
        }
        tally.confirmed_publishes_maximum = tally
            .confirmed_publishes_maximum
            .max(view.confirmed_user_publishes.len());
        if view
            .ideal_recovery_from_the_newest_root
            .as_ref()
            .is_some_and(|ideal| ideal.applied_publishes > 0)
        {
            tally.add(RootChoiceCounter::IdealRecoveryApplied);
        }
        for record in &view.landing.records {
            tally.largest_counter = tally
                .largest_counter
                .max(counter_of_the_landed_record(record));
        }
        if let Some(newest) = view.newest_root {
            let above_newest_root = view
                .landing
                .records
                .iter()
                .filter(|record| {
                    record.material.version.instance == newest.version.instance
                        && record.material.version.checkpoint_txg > newest.version.checkpoint_txg
                })
                .count();
            tally.above_newest_root_maximum =
                tally.above_newest_root_maximum.max(above_newest_root);
            if above_newest_root > self.in_flight_limit {
                tally.add(RootChoiceCounter::AboveNewestRootOverLimit);
            } else if above_newest_root == self.in_flight_limit {
                tally.add(RootChoiceCounter::AboveNewestRootAtLimit);
            }
        }
    }

    fn outcome_of(
        &self,
        view: &RootChoiceStateView<'_, '_>,
        is_the_designated_state: bool,
        tally: &mut RootChoiceTally,
        describe_state: &dyn Fn() -> String,
    ) -> RootChoiceOutcome {
        match self.implementation {
            RootChoiceImplementation::TodayThreeSteps => {
                self.outcome_of_the_three_steps(view, tally, describe_state)
            }
            RootChoiceImplementation::PanicsOnTheDesignatedState => {
                if is_the_designated_state {
                    RootChoiceOutcome::Panicked {
                        step: "designated_state",
                    }
                } else {
                    self.outcome_of_the_three_steps(view, tally, describe_state)
                }
            }
            RootChoiceImplementation::ChoosesTheOldestRoot => {
                let Some(oldest) = root_choice_oldest_root(view.landing) else {
                    return RootChoiceOutcome::ChoseNothing;
                };
                let ideal = root_choice_ideal_recovery_from(
                    oldest,
                    &view.landing.records,
                    self.in_flight_limit,
                );
                RootChoiceOutcome::Landed {
                    chosen: oldest.version,
                    landed_version: ideal.landed_version,
                    landed_fields: ideal.landed_fields,
                    replay_failed: false,
                }
            }
            RootChoiceImplementation::LandsOneVersionBeforeTheLastConfirmed => {
                let Some(newest) = view.newest_root else {
                    return RootChoiceOutcome::ChoseNothing;
                };
                if let Some(last_confirmed) = view.confirmed_user_publishes.iter().copied().max() {
                    let one_version_before = RootVersion {
                        checkpoint_txg: last_confirmed.checkpoint_txg - 1,
                        instance: last_confirmed.instance,
                    };
                    let landed_fields = view
                        .landing
                        .records
                        .iter()
                        .find(|record| record.material.version == one_version_before)
                        .map(|record| record.material.fields)
                        .expect("最后一次已确认的用户发布前面一次发布的记录整次落过（生成时按这个次序写）");
                    RootChoiceOutcome::Landed {
                        chosen: newest.version,
                        landed_version: one_version_before,
                        landed_fields,
                        replay_failed: false,
                    }
                } else {
                    let ideal = root_choice_ideal_recovery_from(
                        newest,
                        &view.landing.records,
                        self.in_flight_limit,
                    );
                    RootChoiceOutcome::Landed {
                        chosen: newest.version,
                        landed_version: ideal.landed_version,
                        landed_fields: ideal.landed_fields,
                        replay_failed: false,
                    }
                }
            }
            RootChoiceImplementation::LocalChoice(rule) => {
                let Some(chosen) = rule.chooses_among(&view.versions_in_ring_order) else {
                    return RootChoiceOutcome::ChoseNothing;
                };
                let landed_fields = view
                    .landing
                    .roots_in_ring_order
                    .iter()
                    .copied()
                    .flatten()
                    .find(|material| material.version == chosen)
                    .map(|material| material.fields)
                    .expect("择到的版本来自根环里的某个槽");
                RootChoiceOutcome::Landed {
                    chosen,
                    landed_version: chosen,
                    landed_fields,
                    replay_failed: false,
                }
            }
        }
    }
}

impl RootChoiceJudgement {
    /// 被测对象：`recover` 的 `Consult` 支里串起来的三步（`choose_root`、`scan_journal`、`replay_journal(…, true)`），各包 `catch_unwind`；
    /// 另逐槽调 `read_root_ring_slot` 与真相对拍（登记 5.5 末段、停机条款 TH4）。
    fn outcome_of_the_three_steps(
        &self,
        view: &RootChoiceStateView<'_, '_>,
        tally: &mut RootChoiceTally,
        describe_state: &dyn Fn() -> String,
    ) -> RootChoiceOutcome {
        let disk = RootChoiceDisk::of_the_landing(self.domain, view.landing);
        self.check_the_slot_readings_against_the_truth(&disk, view, tally, describe_state);
        IS_INSIDE_THE_REPLAY_CALL.with(|is_inside| is_inside.set(true));
        let outcome = self.outcome_of_the_three_steps_with_the_panic_hook_quiet(&disk);
        IS_INSIDE_THE_REPLAY_CALL.with(|is_inside| is_inside.set(false));
        outcome
    }

    fn outcome_of_the_three_steps_with_the_panic_hook_quiet(
        &self,
        disk: &RootChoiceDisk<'_>,
    ) -> RootChoiceOutcome {
        let chosen_root = match panic::catch_unwind(AssertUnwindSafe(|| {
            choose_root(disk, &self.system_configuration)
        })) {
            Err(_panic_payload) => {
                return RootChoiceOutcome::Panicked {
                    step: "choose_root",
                }
            }
            Ok(None) => return RootChoiceOutcome::ChoseNothing,
            Ok(Some(root)) => root,
        };
        let records = match panic::catch_unwind(AssertUnwindSafe(|| {
            scan_journal(disk, &self.system_configuration)
        })) {
            Err(_panic_payload) => {
                return RootChoiceOutcome::Panicked {
                    step: "scan_journal",
                }
            }
            Ok(records) => records,
        };
        let replay = panic::catch_unwind(AssertUnwindSafe(|| {
            replay_journal(
                disk,
                &chosen_root,
                self.system_configuration.immutable.sizes.journal_ring_bytes,
                &records,
                true,
            )
        }));
        match replay {
            Err(_panic_payload) => RootChoiceOutcome::Panicked {
                step: "replay_journal",
            },
            Ok(Err(_failure)) => RootChoiceOutcome::Landed {
                chosen: RootVersion::of_root(&chosen_root),
                landed_version: RootVersion::of_root(&chosen_root),
                landed_fields: VersionFields::of_root(&chosen_root),
                replay_failed: true,
            },
            Ok(Ok((_report, returned_root))) => RootChoiceOutcome::Landed {
                chosen: RootVersion::of_root(&chosen_root),
                landed_version: RootVersion::of_root(&returned_root),
                landed_fields: VersionFields::of_root(&returned_root),
                replay_failed: false,
            },
        }
    }

    /// 读槽对拍：逐槽调 `read_root_ring_slot`，自证过的槽集合与每个槽交回的根必须等于生成时的真相（没写过的槽读成全 0、自证不过）。
    /// 不一致不当 KJ 的违例记，单独数、记见证，收尾走停机条款 TH4。
    fn check_the_slot_readings_against_the_truth(
        &self,
        disk: &RootChoiceDisk<'_>,
        view: &RootChoiceStateView<'_, '_>,
        tally: &mut RootChoiceTally,
        describe_state: &dyn Fn() -> String,
    ) {
        tally.add(RootChoiceCounter::SlotReadingsChecked);
        let immutable = &self.system_configuration.immutable;
        for ring_slot in every_root_ring_slot(&immutable.sizes) {
            let reading = read_root_ring_slot(
                disk,
                &immutable.region_devices,
                &immutable.sizes,
                &immutable.filesystem_identifier,
                ring_slot,
            );
            let slot_index = root_choice_slot_index_of(
                ring_slot.region,
                ring_slot.slot,
                self.domain.slots_per_region(),
            );
            let truth = view.landing.roots_in_ring_order[slot_index];
            if !root_slot_reading_agrees_with_the_truth(reading, truth) {
                tally.add_violation(
                    RootChoiceCounter::SlotReadingsDifferingFromTheTruth,
                    &|| {
                        format!(
                            "region{}_slot{}_read_as={reading:?}_truth={};{}",
                            ring_slot.region,
                            ring_slot.slot,
                            truth.map_or_else(
                                || "empty".to_string(),
                                |material| material.version.describe()
                            ),
                            describe_state()
                        )
                    },
                );
            }
        }
    }

    /// 一个状态的结局落进四条判据（各报各的，不合成一个真假）。
    fn judge_the_outcome(
        &self,
        view: &RootChoiceStateView<'_, '_>,
        outcome: &RootChoiceOutcome,
        tally: &mut RootChoiceTally,
        describe_state: &dyn Fn() -> String,
    ) {
        match outcome {
            RootChoiceOutcome::Panicked { step } => {
                tally.add(RootChoiceCounter::PanickedStates);
                tally.add_violation(RootChoiceCounter::Kj0Violations, &|| {
                    format!("panicked_in={step};{}", describe_state())
                });
            }
            RootChoiceOutcome::ChoseNothing => {
                tally.add(RootChoiceCounter::ChoseNothingStates);
                tally.add_violation(RootChoiceCounter::Kj0Violations, &|| {
                    format!("choose_root_returned_none;{}", describe_state())
                });
            }
            RootChoiceOutcome::Landed {
                chosen,
                landed_version,
                landed_fields,
                replay_failed,
            } => {
                if *replay_failed {
                    tally.add(RootChoiceCounter::ReplayFailedStates);
                }
                let above_chosen_root = view
                    .landing
                    .records
                    .iter()
                    .filter(|record| {
                        record.material.version.instance == chosen.instance
                            && record.material.version.checkpoint_txg > chosen.checkpoint_txg
                    })
                    .count();
                tally.above_chosen_root_maximum =
                    tally.above_chosen_root_maximum.max(above_chosen_root);
                if above_chosen_root > self.in_flight_limit {
                    tally.add(RootChoiceCounter::AboveChosenRootOverLimit);
                } else if above_chosen_root == self.in_flight_limit {
                    tally.add(RootChoiceCounter::AboveChosenRootAtLimit);
                }
                if landed_version != chosen {
                    tally.add(RootChoiceCounter::ReplayAppliedStates);
                }
                let facts = || {
                    format!(
                        "chosen={};landed={};newest_root={};last_confirmed={};roots_in_ring_order={};{}",
                        chosen.describe(),
                        landed_version.describe(),
                        view.newest_root
                            .map_or_else(|| "none".to_string(), |newest| newest.version.describe()),
                        view.last_confirmed
                            .map_or_else(|| "none".to_string(), RootVersion::describe),
                        describe_the_roots_in_ring_order(
                            &view.versions_in_ring_order,
                            self.domain.slots_per_region()
                        ),
                        describe_state()
                    )
                };
                if self.domain.is_a_legal_history() {
                    if let Some(last_confirmed) = view.last_confirmed {
                        tally.add(RootChoiceCounter::Kj1Judged);
                        if *landed_version < last_confirmed {
                            tally.add_violation(RootChoiceCounter::Kj1Violations, &|| {
                                format!("KJ1;{}", facts())
                            });
                        }
                    }
                }
                if let Some(newest) = view.newest_root {
                    tally.add(RootChoiceCounter::Kj2Judged);
                    if *landed_version < newest.version {
                        tally.add_violation(RootChoiceCounter::Kj2Violations, &|| {
                            format!("KJ2;{}", facts())
                        });
                    }
                }
                if self.domain.is_a_legal_history() {
                    if let Some(ideal) = &view.ideal_recovery_from_the_newest_root {
                        tally.add(RootChoiceCounter::Kj3Judged);
                        if *landed_fields != ideal.landed_fields {
                            tally.add_violation(RootChoiceCounter::Kj3Violations, &|| {
                                format!(
                                    "KJ3;ideal_landed={};{}",
                                    ideal.landed_version.describe(),
                                    facts()
                                )
                            });
                        }
                    }
                }
            }
        }
    }
}

/// 一个根环槽的读数对真相：读成自证过的，它的 (txg, 实例代号) 必须等于真相里这个槽的根；读成自证不过的（全 0 的槽），真相里这个槽必须是空的；
/// 读不出一律不合（内存盘对根环的槽只回 512 字节或全 0）。
fn root_slot_reading_agrees_with_the_truth(
    reading: RootRingSlotReading,
    truth: Option<&RootChoicePublishMaterial>,
) -> bool {
    match reading {
        RootRingSlotReading::SelfVerified(root) => truth.is_some_and(|material| {
            root.checkpoint_txg.0 == material.version.checkpoint_txg
                && root.instance.0 == material.version.instance
        }),
        RootRingSlotReading::Bad(BadRootRingSlotReading::NotSelfVerified) => truth.is_none(),
        RootRingSlotReading::Bad(BadRootRingSlotReading::Unreadable) => false,
    }
}

// ── 枚举一个 (域, M, U) 格里的全部状态

/// 一次挂载里写到哪了：下一条记录的计数器、下一个事务号、本实例上一条记录的链值、下一次发布的 txg。
#[derive(Clone, Copy, Debug)]
struct RootChoiceMountCursor {
    instance: u32,
    next_counter: u64,
    next_transaction: u64,
    previous_chain_value: Option<u32>,
    next_checkpoint_txg: u64,
}

impl RootChoiceMountCursor {
    fn after_the_publish_of(self, material: &RootChoicePublishMaterial) -> Self {
        let last_slot = material.records.last().expect("每次发布至少一条记录");
        let commit_records = material
            .records
            .iter()
            .filter(|slot| {
                slot.parsed
                    .as_ref()
                    .expect("合法历史的每条记录都解得出（生成时断言过）")
                    .is_commit
            })
            .count();
        Self {
            instance: self.instance,
            next_counter: self.next_counter
                + u64::try_from(material.records.len()).expect("一次发布的记录条数不超过 2"),
            next_transaction: if material.is_user_publish {
                self.next_transaction
                    + u64::try_from(commit_records).expect("一次发布的记录条数不超过 2")
            } else {
                self.next_transaction
            },
            previous_chain_value: Some(last_slot.chain_value_of_header),
            next_checkpoint_txg: material.version.checkpoint_txg + 1,
        }
    }

    fn plan_of_the_next_publish(
        self,
        domain: RootChoiceDomain,
        publishes_started: u64,
        record_count: usize,
        commit_marker_bits: u32,
        is_user_publish: bool,
    ) -> RootChoicePublishPlan {
        RootChoicePublishPlan {
            domain,
            instance: self.instance,
            checkpoint_txg: self.next_checkpoint_txg,
            publish_ordinal: publishes_started,
            first_counter: self.next_counter,
            new_pool_file_creation: self.next_transaction,
            record_count,
            commit_marker_bits,
            previous_chain_value: self.previous_chain_value,
            is_user_publish,
        }
    }
}

/// 走枚举的一方：收工作单元的起点，或判最后一次挂载崩溃点上的每个状态。
trait RootChoiceVisitor {
    /// 走到「第 `mount_number` 次挂载开始之前」（只在 `stop_before_mount` 点名的那一次挂载上触发，之后不再往下走）。
    fn reach_the_start_of_a_mount(
        &mut self,
        world: &RootChoiceWorld,
        mount_number: usize,
        user_publishes_left: usize,
    );

    /// 最后一次挂载的崩溃点：`world` 是崩溃在途那次发布之前的盘，`crashed` 是它的崩溃点，`trace` 是这次挂载到这里为止的写法。
    fn visit_a_final_state(
        &mut self,
        world: &RootChoiceWorld,
        crashed: &RootChoiceCrashedPublish<'_>,
        trace: &str,
    );
}

struct RootChoiceEnumeration {
    domain: RootChoiceDomain,
    mounts: usize,
    stop_before_mount: Option<usize>,
}

impl RootChoiceEnumeration {
    /// 第 `mount_number` 次挂载（实例代号 = 它）：写行那次发布、暖机的空发布、用户发布 u_m 次，崩溃点在第 u_m 次用户发布里；
    /// u_m = 0 时在写行或任一次暖机发布里。非最后一次挂载 u_m 取 0..=剩余的用户发布数，最后一次用完剩余的（登记 5.3）。
    fn enumerate_a_mount(
        &self,
        world: &RootChoiceWorld,
        mount_number: usize,
        user_publishes_left: usize,
        visitor: &mut dyn RootChoiceVisitor,
    ) {
        if self.stop_before_mount == Some(mount_number) {
            visitor.reach_the_start_of_a_mount(world, mount_number, user_publishes_left);
            return;
        }
        let is_the_last_mount = mount_number == self.mounts;
        let overhead_txgs = root_choice_warm_up_plan(world.highest_checkpoint_txg_on_disk() + 1);
        let first_user_txg = overhead_txgs.last().expect("写行那次发布至少有一次") + 1;
        let user_publish_counts: Vec<usize> = if is_the_last_mount {
            vec![user_publishes_left]
        } else {
            (0..=user_publishes_left).collect()
        };
        for user_publish_count in user_publish_counts {
            let user_publishes_after_this_mount = user_publishes_left - user_publish_count;
            let mut running_world = world.clone();
            let mut cursor = RootChoiceMountCursor {
                instance: u32::try_from(mount_number).expect("挂载次数不超过 3"),
                next_counter: world.highest_landed_counter() + 1,
                next_transaction: 1,
                previous_chain_value: None,
                next_checkpoint_txg: overhead_txgs[0],
            };
            let mut trace = format!(
                "mount{mount_number}:overhead_txgs={}",
                overhead_txgs
                    .iter()
                    .map(u64::to_string)
                    .collect::<Vec<String>>()
                    .join(",")
            );
            for overhead_txg in &overhead_txgs {
                assert_eq!(
                    cursor.next_checkpoint_txg, *overhead_txg,
                    "开销发布的 txg 逐个加一"
                );
                let material = Arc::new(root_choice_publish_material(
                    &cursor.plan_of_the_next_publish(
                        self.domain,
                        running_world.publishes_started,
                        1,
                        0,
                        false,
                    ),
                ));
                if user_publish_count == 0 {
                    self.enumerate_the_crash_points(
                        &running_world,
                        &material,
                        &trace,
                        (mount_number, user_publishes_after_this_mount),
                        visitor,
                    );
                }
                running_world = running_world.with_publish_applied(&material, false);
                cursor = cursor.after_the_publish_of(&material);
            }
            if user_publish_count > 0 {
                assert_eq!(cursor.next_checkpoint_txg, first_user_txg);
                trace.push_str(";user_publishes");
                self.enumerate_user_publishes(
                    &running_world,
                    cursor,
                    (
                        mount_number,
                        user_publish_count,
                        user_publishes_after_this_mount,
                    ),
                    &trace,
                    visitor,
                );
            }
        }
    }

    /// 第 `user_publishes_in_the_mount`（≥ 1）次用户发布起，每次的 k 与提交标记摆法各一种；崩溃点在这一挂载的最后一次用户发布里。
    fn enumerate_user_publishes(
        &self,
        running_world: &RootChoiceWorld,
        cursor: RootChoiceMountCursor,
        (mount_number, user_publishes_in_the_mount, user_publishes_after_this_mount): (
            usize,
            usize,
            usize,
        ),
        trace: &str,
        visitor: &mut dyn RootChoiceVisitor,
    ) {
        for record_count in self.domain.user_publish_record_counts() {
            for commit_marker_bits in 0..(1u32 << (record_count - 1)) {
                let material = Arc::new(root_choice_publish_material(
                    &cursor.plan_of_the_next_publish(
                        self.domain,
                        running_world.publishes_started,
                        *record_count,
                        commit_marker_bits,
                        true,
                    ),
                ));
                let trace_with_this_publish = format!(
                    "{trace}|txg{}k{record_count}b{commit_marker_bits}",
                    material.version.checkpoint_txg
                );
                if user_publishes_in_the_mount == 1 {
                    self.enumerate_the_crash_points(
                        running_world,
                        &material,
                        &trace_with_this_publish,
                        (mount_number, user_publishes_after_this_mount),
                        visitor,
                    );
                } else {
                    let applied_world = running_world.with_publish_applied(&material, true);
                    self.enumerate_user_publishes(
                        &applied_world,
                        cursor.after_the_publish_of(&material),
                        (
                            mount_number,
                            user_publishes_in_the_mount - 1,
                            user_publishes_after_this_mount,
                        ),
                        &trace_with_this_publish,
                        visitor,
                    );
                }
            }
        }
    }

    /// 崩在 `material` 这次发布的每个崩溃点上：最后一次挂载交给 `visitor` 判；之前的挂载接着枚举下一次挂载。
    fn enumerate_the_crash_points(
        &self,
        running_world: &RootChoiceWorld,
        material: &Arc<RootChoicePublishMaterial>,
        trace: &str,
        (mount_number, user_publishes_after_this_mount): (usize, usize),
        visitor: &mut dyn RootChoiceVisitor,
    ) {
        for crash_point in root_choice_crash_points_of(material.records.len()) {
            let crashed = RootChoiceCrashedPublish {
                material,
                crash_point,
            };
            if mount_number == self.mounts {
                visitor.visit_a_final_state(running_world, &crashed, trace);
            } else {
                let crashed_world = running_world.with_crash_applied(&crashed, trace);
                self.enumerate_a_mount(
                    &crashed_world,
                    mount_number + 1,
                    user_publishes_after_this_mount,
                    visitor,
                );
            }
        }
    }
}

// ── 一个格的计数：枚举、判、按工作单元分给几条线程

struct RootChoiceUnit {
    world: RootChoiceWorld,
    mount_number: usize,
    user_publishes_left: usize,
}

/// 工作单元：前面几次挂载（最多两次）的全部走法在崩溃之后得到的盘，每个单元接着枚举剩下的挂载。
struct RootChoiceUnitCollector {
    units: Vec<RootChoiceUnit>,
}

impl RootChoiceVisitor for RootChoiceUnitCollector {
    fn reach_the_start_of_a_mount(
        &mut self,
        world: &RootChoiceWorld,
        mount_number: usize,
        user_publishes_left: usize,
    ) {
        self.units.push(RootChoiceUnit {
            world: world.clone(),
            mount_number,
            user_publishes_left,
        });
    }

    fn visit_a_final_state(
        &mut self,
        _world: &RootChoiceWorld,
        _crashed: &RootChoiceCrashedPublish<'_>,
        _trace: &str,
    ) {
        panic!("收工作单元的那一遍停在第 min(M, 3) 次挂载开始之前，走不到最后一次挂载的崩溃点");
    }
}

struct RootChoiceStateJudge<'run> {
    judgement: &'run RootChoiceJudgement,
    tally: RootChoiceTally,
    states_seen_in_this_unit: u64,
    is_the_first_unit: bool,
}

impl RootChoiceVisitor for RootChoiceStateJudge<'_> {
    fn reach_the_start_of_a_mount(
        &mut self,
        _world: &RootChoiceWorld,
        _mount_number: usize,
        _user_publishes_left: usize,
    ) {
        panic!("判状态的那一遍不设 stop_before_mount");
    }

    fn visit_a_final_state(
        &mut self,
        world: &RootChoiceWorld,
        crashed: &RootChoiceCrashedPublish<'_>,
        trace: &str,
    ) {
        let landing = root_choice_landing_of(world, Some(crashed));
        let describe_state =
            || format!("{}|{trace};{}", world.history.join("/"), crashed.describe());
        let is_the_designated_state = self.is_the_first_unit && self.states_seen_in_this_unit == 0;
        self.states_seen_in_this_unit += 1;
        self.judgement.judge(
            &landing,
            &world.confirmed_user_publishes,
            is_the_designated_state,
            &mut self.tally,
            &describe_state,
        );
    }
}

/// KA（登记 5.4）：根环读根环次序的前三个槽（区域 0 的槽 0、1、2）各取空或四条根之一，其余根槽全 0，journal 环全 0。
const ROOT_CHOICE_ADVERSARIAL_ROOT_VERSIONS: [RootVersion; 4] = [
    RootVersion {
        checkpoint_txg: 100,
        instance: 1,
    },
    RootVersion {
        checkpoint_txg: 100,
        instance: 2,
    },
    RootVersion {
        checkpoint_txg: 101,
        instance: 1,
    },
    RootVersion {
        checkpoint_txg: 101,
        instance: 2,
    },
];
const ROOT_CHOICE_ADVERSARIAL_FILLED_SLOTS: usize = 3;

fn root_choice_tally_of_the_adversarial_root_rings(
    implementation: RootChoiceImplementation,
) -> RootChoiceTally {
    let domain = RootChoiceDomain::AdversarialRootRing;
    let judgement = RootChoiceJudgement::new(domain, implementation);
    let root_materials: Vec<Arc<RootChoicePublishMaterial>> = ROOT_CHOICE_ADVERSARIAL_ROOT_VERSIONS
        .iter()
        .enumerate()
        .map(|(version_index, version)| {
            Arc::new(root_material_without_records(
                version.instance,
                version.checkpoint_txg,
                NewRootSegment::off_timeline(
                    1 + u64::try_from(version_index).expect("四条根"),
                    version.checkpoint_txg,
                ),
            ))
        })
        .collect();
    let choices_per_slot = root_materials.len() + 1;
    let mut tally = RootChoiceTally::new();
    let state_count =
        choices_per_slot.pow(u32::try_from(ROOT_CHOICE_ADVERSARIAL_FILLED_SLOTS).expect("三个槽"));
    for state_number in 0..state_count {
        let mut world = RootChoiceWorld {
            domain,
            root_slots: vec![
                None;
                usize::try_from(ROOT_CHOICE_REGION_COUNT * domain.slots_per_region())
                    .expect("根槽数不超过 24")
            ],
            landed_records: Vec::new(),
            confirmed_user_publishes: Vec::new(),
            publishes_started: 0,
            history: Vec::new(),
        };
        let mut remaining_choices = state_number;
        let mut choice_texts = Vec::new();
        for slot_index in (0..ROOT_CHOICE_ADVERSARIAL_FILLED_SLOTS).rev() {
            let choice = remaining_choices % choices_per_slot;
            remaining_choices /= choices_per_slot;
            if choice > 0 {
                world.root_slots[slot_index] = Some(Arc::clone(&root_materials[choice - 1]));
                choice_texts.push(root_materials[choice - 1].version.describe());
            } else {
                choice_texts.push("empty".to_string());
            }
        }
        choice_texts.reverse();
        let landing = root_choice_landing_of(&world, None);
        let describe_state = || format!("ka_region0_slots012=[{}]", choice_texts.join(","));
        judgement.judge(&landing, &[], false, &mut tally, &describe_state);
    }
    tally
}

/// 一个 (域, M, U) 格上、一种实现下全部状态的计数。KA 单线程走 125 个状态；合法历史的三个域按工作单元分给几条线程。
fn root_choice_tally_of_a_cell(
    domain: RootChoiceDomain,
    cell: RootChoiceCell,
    implementation: RootChoiceImplementation,
    thread_count: usize,
) -> RootChoiceTally {
    if !domain.is_a_legal_history() {
        return root_choice_tally_of_the_adversarial_root_rings(implementation);
    }
    let collecting = RootChoiceEnumeration {
        domain,
        mounts: cell.mounts,
        stop_before_mount: Some(cell.mounts.min(3)),
    };
    let mut collector = RootChoiceUnitCollector { units: Vec::new() };
    collecting.enumerate_a_mount(
        &RootChoiceWorld::right_after_mkfs(domain),
        1,
        cell.user_publishes,
        &mut collector,
    );
    let judgement = RootChoiceJudgement::new(domain, implementation);
    let continuing = RootChoiceEnumeration {
        domain,
        mounts: cell.mounts,
        stop_before_mount: None,
    };
    root_choice_tally_of_work_units(&collector.units, thread_count, &|unit_index, unit| {
        let mut judge = RootChoiceStateJudge {
            judgement: &judgement,
            tally: RootChoiceTally::new(),
            states_seen_in_this_unit: 0,
            is_the_first_unit: unit_index == 0,
        };
        continuing.enumerate_a_mount(
            &unit.world,
            unit.mount_number,
            unit.user_publishes_left,
            &mut judge,
        );
        judge.tally
    })
}

// ── 钉绝对值（登记第七节乙类：出自 `anchors_r3_k.py` 独立生成器，常量写在断言右边；不符走 TV4）

#[derive(Clone, Copy, Debug)]
struct PinnedRootChoiceCell {
    domain: RootChoiceDomain,
    mounts: usize,
    user_publishes: usize,
    states: u64,
    confirmed_states: u64,
    first_self_verified_root_is_not_the_newest: u64,
    same_txg_different_instance_roots: u64,
    higher_instance_lower_txg_roots: u64,
    two_or_more_instances: u64,
    ideal_recovery_applied: u64,
    above_chosen_root_maximum: usize,
    largest_counter: u64,
    /// KF1（所选根取版本最小的根）下 KJ1、KJ2 必须读到的数。
    oldest_root_kj1: u64,
    oldest_root_kj2: u64,
}

fn pinned_root_choice_cell(
    domain: RootChoiceDomain,
    cell: RootChoiceCell,
) -> Option<&'static PinnedRootChoiceCell> {
    PINNED_ROOT_CHOICE_CELLS.iter().find(|pinned| {
        pinned.domain == domain
            && pinned.mounts == cell.mounts
            && pinned.user_publishes == cell.user_publishes
    })
}

/// KA 上各种本地择根序的 KJ2 违例数（登记第七节乙类 KA：今天的序 0、平局取低 36、实例代号为主 18、比较不看实例 18、取第一个有效者 52），
/// 出自 `anchors_r3_ka.py`。
const fn pinned_root_choice_adversarial_kj2_violations(rule: LocalRootChoiceRule) -> u64 {
    match rule {
        LocalRootChoiceRule::NewestWins => 0,
        LocalRootChoiceRule::TieTakesTheLowerInstance => 36,
        LocalRootChoiceRule::InstanceFirstThenTxg => 18,
        LocalRootChoiceRule::IgnoresTheInstance => 18,
        LocalRootChoiceRule::FirstSelfVerifiedWins => 52,
    }
}

/// KA 的状态数 125 与其中没有一条自证过的根的 1 态（登记第七节乙类 KA：`choose_root` 交回空，KJ0 在这一态记一条）。
const PINNED_ROOT_CHOICE_ADVERSARIAL_STATES: u64 = 125;

/// 每个跑到的合法历史格，装置数出的状态数与几样计数等于独立生成器的数；不符走作废条款 TV4（这些量只看生成器与真相，与被测实现无关）。
fn assert_the_pinned_root_choice_cell(
    domain: RootChoiceDomain,
    cell: RootChoiceCell,
    tally: &RootChoiceTally,
) {
    let pinned = pinned_root_choice_cell(domain, cell)
        .expect("每个跑到的合法历史格都在登记第七节乙类的表里");
    let place = format!(
        "域 {} M = {} U = {}",
        domain.label(),
        cell.mounts,
        cell.user_publishes
    );
    assert_eq!(
        tally.get(RootChoiceCounter::States),
        pinned.states,
        "TV4：{place} 状态数"
    );
    assert_eq!(
        tally.get(RootChoiceCounter::ConfirmedStates),
        pinned.confirmed_states,
        "TV4：{place} 有已确认发布的态数"
    );
    assert_eq!(
        tally.get(RootChoiceCounter::FirstSelfVerifiedRootIsNotTheNewest),
        pinned.first_self_verified_root_is_not_the_newest,
        "TV4：{place} 读根环次序里第一个自证过的根不是版本最大那条的态数"
    );
    assert_eq!(
        tally.get(RootChoiceCounter::SameTxgDifferentInstanceRoots),
        pinned.same_txg_different_instance_roots,
        "TV4：{place} 根环里同 txg 不同实例同在的态数"
    );
    assert_eq!(
        tally.get(RootChoiceCounter::HigherInstanceLowerTxgRoots),
        pinned.higher_instance_lower_txg_roots,
        "TV4：{place} 实例代号更高而 txg 更低的一对根同在的态数"
    );
    assert_eq!(
        tally.get(RootChoiceCounter::TwoOrMoreInstancesInTheRing),
        pinned.two_or_more_instances,
        "TV4：{place} 根环里有两个以上实例的根的态数"
    );
    assert_eq!(
        tally.get(RootChoiceCounter::IdealRecoveryApplied),
        pinned.ideal_recovery_applied,
        "TV4：{place} 理想恢复施加过至少一次发布的态数"
    );
    assert_eq!(
        tally.get(RootChoiceCounter::StatesWithoutASelfVerifiedRoot),
        0,
        "TV4：{place} 根环里没有一条自证过的根的态数（合法历史里第 0 代根至少还有一份）"
    );
    assert_eq!(
        tally.above_newest_root_maximum, pinned.above_chosen_root_maximum,
        "TV4：{place} 真相里版本最大的根之上同实例的记录条数的最大值"
    );
    assert_eq!(
        tally.largest_counter, pinned.largest_counter,
        "TV4：{place} 盘上读得出的最大计数器"
    );
}

/// 本地常量逐个对着实现侧的同名常量与函数回比（登记第三节回比表；不符走停机条款 TH2）。
fn cross_check_root_choice_local_constants_against_the_implementation() {
    assert_eq!(
        ROOT_CHOICE_REGION_COUNT,
        singlefs_format::ROOT_RING_REGIONS,
        "TH2：根环区域数"
    );
    assert_eq!(
        ROOT_CHOICE_BASE_SLOT,
        singlefs_format::ROOT_RING_BASE_SLOT,
        "TH2：根环基槽"
    );
    assert_eq!(
        ROOT_CHOICE_PRIME_STEP,
        singlefs_format::ROOT_RING_PRIME_STEP,
        "TH2：区域步长"
    );
    assert_eq!(
        ROOT_CHOICE_CHUNK_BYTES,
        singlefs_format::ROOT_RING_CHUNK_BYTES,
        "TH2：区域 chunk 字节数"
    );
    for region in 0..ROOT_CHOICE_REGION_COUNT {
        assert_eq!(
            root_choice_region_start_in_bytes(region),
            singlefs_core::root_ring::region_start(region).0,
            "TH2：本地算的区域起点与 root_ring::region_start 不等（区域 {region}）"
        );
    }
    assert_eq!(
        ROOT_CHOICE_REGION_DEVICES,
        singlefs_format::ROOT_RING_REGION_DEVICES,
        "TH2：根环区域的设备归属（格式常量）"
    );
    assert_eq!(
        ROOT_CHOICE_REGION_DEVICES.map(DeviceIdentity),
        singlefs_core::make_filesystem::FIRST_VERSION_REGION_DEVICES,
        "TH2：根环区域的设备归属（mkfs 第一版）"
    );
    assert_eq!(
        ROOT_CHOICE_SLOTS_PER_REGION_WIDE,
        RootRingSlotsPerRegion::AT_MAKE_FILESYSTEM.count(),
        "TH2：mkfs 取的每区槽数"
    );
    assert_eq!(
        ROOT_CHOICE_SLOTS_PER_REGION_NARROW,
        singlefs_format::ROOT_RING_SLOTS_PER_REGION_MINIMUM,
        "TH2：每区槽数下限"
    );
    assert_eq!(
        ROOT_CHOICE_MKFS_INSTANCE,
        singlefs_core::make_filesystem::MKFS_INSTANCE_GENERATION.0,
        "TH2：mkfs 第 0 代根的实例代号"
    );
    assert!(
        u64::try_from(ROOT_CHOICE_ROOT_SLOT_BYTES).expect("512 装得进 u64")
            >= singlefs_format::ROOT_RECORD_BYTES,
        "TH2：根槽宽装得下根记录"
    );
    assert_eq!(
        u64::from(ROOT_CHOICE_FIXED_STRUCTURE_SLOT_SPACING_BYTES),
        singlefs_format::FIXED_STRUCTURE_SLOT_SPACING_MINIMUM_BYTES,
        "TH2：固定结构槽距"
    );
    for domain in RootChoiceDomain::ALL {
        for checkpoint_txg in 0..=40 {
            assert_the_local_root_slot_arithmetic_against_the_implementation(
                *domain,
                checkpoint_txg,
            );
        }
    }
    // 新池第一次挂载：写行 txg 1、暖机 txg 2（开销发布两次），第一个用户发布 txg 3（登记第七节甲类、第三节回比表第 5 行）。
    let first_mount_plan = root_choice_warm_up_plan(1);
    assert_eq!(
        u64::try_from(first_mount_plan.len()).expect("开销发布不超过 4 次"),
        singlefs_format::WARM_UP_EMPTY_PUBLISHES,
        "TH2 / TF9：新池第一次挂载的开销发布次数与 WARM_UP_EMPTY_PUBLISHES 不等"
    );
    assert_eq!(
        first_mount_plan,
        [1, 2],
        "TF9：新池第一次挂载写行 txg 1、暖机 txg 2"
    );
    assert_eq!(
        first_mount_plan.last().expect("至少一次开销发布") + 1,
        singlefs_format::NEW_POOL_FILE_CREATION_TXG,
        "TH2 / TF9：新池第一个用户发布的 txg 与 NEW_POOL_FILE_CREATION_TXG 不等"
    );
}
const PINNED_ROOT_CHOICE_CELLS: [PinnedRootChoiceCell; 36] = [
    PinnedRootChoiceCell {
        domain: RootChoiceDomain::BigRing,
        mounts: 1,
        user_publishes: 0,
        states: 10,
        confirmed_states: 0,
        first_self_verified_root_is_not_the_newest: 6,
        same_txg_different_instance_roots: 0,
        higher_instance_lower_txg_roots: 0,
        two_or_more_instances: 6,
        ideal_recovery_applied: 3,
        above_chosen_root_maximum: 1,
        largest_counter: 2,
        oldest_root_kj1: 0,
        oldest_root_kj2: 6,
    },
    PinnedRootChoiceCell {
        domain: RootChoiceDomain::BigRing,
        mounts: 1,
        user_publishes: 1,
        states: 39,
        confirmed_states: 0,
        first_self_verified_root_is_not_the_newest: 39,
        same_txg_different_instance_roots: 0,
        higher_instance_lower_txg_roots: 0,
        two_or_more_instances: 39,
        ideal_recovery_applied: 21,
        above_chosen_root_maximum: 2,
        largest_counter: 4,
        oldest_root_kj1: 0,
        oldest_root_kj2: 39,
    },
    PinnedRootChoiceCell {
        domain: RootChoiceDomain::BigRing,
        mounts: 1,
        user_publishes: 2,
        states: 117,
        confirmed_states: 117,
        first_self_verified_root_is_not_the_newest: 117,
        same_txg_different_instance_roots: 0,
        higher_instance_lower_txg_roots: 0,
        two_or_more_instances: 117,
        ideal_recovery_applied: 63,
        above_chosen_root_maximum: 2,
        largest_counter: 6,
        oldest_root_kj1: 117,
        oldest_root_kj2: 117,
    },
    PinnedRootChoiceCell {
        domain: RootChoiceDomain::BigRing,
        mounts: 1,
        user_publishes: 3,
        states: 351,
        confirmed_states: 351,
        first_self_verified_root_is_not_the_newest: 351,
        same_txg_different_instance_roots: 0,
        higher_instance_lower_txg_roots: 0,
        two_or_more_instances: 351,
        ideal_recovery_applied: 189,
        above_chosen_root_maximum: 2,
        largest_counter: 8,
        oldest_root_kj1: 351,
        oldest_root_kj2: 351,
    },
    PinnedRootChoiceCell {
        domain: RootChoiceDomain::BigRing,
        mounts: 1,
        user_publishes: 4,
        states: 1053,
        confirmed_states: 1053,
        first_self_verified_root_is_not_the_newest: 1053,
        same_txg_different_instance_roots: 0,
        higher_instance_lower_txg_roots: 0,
        two_or_more_instances: 1053,
        ideal_recovery_applied: 567,
        above_chosen_root_maximum: 2,
        largest_counter: 10,
        oldest_root_kj1: 1053,
        oldest_root_kj2: 1053,
    },
    PinnedRootChoiceCell {
        domain: RootChoiceDomain::BigRing,
        mounts: 2,
        user_publishes: 0,
        states: 125,
        confirmed_states: 0,
        first_self_verified_root_is_not_the_newest: 109,
        same_txg_different_instance_roots: 0,
        higher_instance_lower_txg_roots: 0,
        two_or_more_instances: 109,
        ideal_recovery_applied: 57,
        above_chosen_root_maximum: 1,
        largest_counter: 4,
        oldest_root_kj1: 0,
        oldest_root_kj2: 109,
    },
    PinnedRootChoiceCell {
        domain: RootChoiceDomain::BigRing,
        mounts: 2,
        user_publishes: 1,
        states: 780,
        confirmed_states: 0,
        first_self_verified_root_is_not_the_newest: 780,
        same_txg_different_instance_roots: 0,
        higher_instance_lower_txg_roots: 0,
        two_or_more_instances: 780,
        ideal_recovery_applied: 411,
        above_chosen_root_maximum: 2,
        largest_counter: 6,
        oldest_root_kj1: 0,
        oldest_root_kj2: 780,
    },
    PinnedRootChoiceCell {
        domain: RootChoiceDomain::BigRing,
        mounts: 2,
        user_publishes: 2,
        states: 4401,
        confirmed_states: 2880,
        first_self_verified_root_is_not_the_newest: 4401,
        same_txg_different_instance_roots: 0,
        higher_instance_lower_txg_roots: 0,
        two_or_more_instances: 4401,
        ideal_recovery_applied: 2376,
        above_chosen_root_maximum: 2,
        largest_counter: 9,
        oldest_root_kj1: 2880,
        oldest_root_kj2: 4401,
    },
    PinnedRootChoiceCell {
        domain: RootChoiceDomain::BigRing,
        mounts: 2,
        user_publishes: 3,
        states: 16281,
        confirmed_states: 16281,
        first_self_verified_root_is_not_the_newest: 16281,
        same_txg_different_instance_roots: 0,
        higher_instance_lower_txg_roots: 0,
        two_or_more_instances: 16281,
        ideal_recovery_applied: 8694,
        above_chosen_root_maximum: 2,
        largest_counter: 11,
        oldest_root_kj1: 16281,
        oldest_root_kj2: 16281,
    },
    PinnedRootChoiceCell {
        domain: RootChoiceDomain::BigRing,
        mounts: 2,
        user_publishes: 4,
        states: 62127,
        confirmed_states: 62127,
        first_self_verified_root_is_not_the_newest: 62127,
        same_txg_different_instance_roots: 0,
        higher_instance_lower_txg_roots: 0,
        two_or_more_instances: 62127,
        ideal_recovery_applied: 33210,
        above_chosen_root_maximum: 2,
        largest_counter: 13,
        oldest_root_kj1: 62127,
        oldest_root_kj2: 62127,
    },
    PinnedRootChoiceCell {
        domain: RootChoiceDomain::BigRing,
        mounts: 3,
        user_publishes: 0,
        states: 1480,
        confirmed_states: 0,
        first_self_verified_root_is_not_the_newest: 1416,
        same_txg_different_instance_roots: 0,
        higher_instance_lower_txg_roots: 0,
        two_or_more_instances: 1416,
        ideal_recovery_applied: 741,
        above_chosen_root_maximum: 1,
        largest_counter: 7,
        oldest_root_kj1: 0,
        oldest_root_kj2: 1416,
    },
    PinnedRootChoiceCell {
        domain: RootChoiceDomain::BigRing,
        mounts: 3,
        user_publishes: 1,
        states: 13770,
        confirmed_states: 0,
        first_self_verified_root_is_not_the_newest: 13770,
        same_txg_different_instance_roots: 0,
        higher_instance_lower_txg_roots: 0,
        two_or_more_instances: 13770,
        ideal_recovery_applied: 7266,
        above_chosen_root_maximum: 2,
        largest_counter: 9,
        oldest_root_kj1: 0,
        oldest_root_kj2: 13770,
    },
    PinnedRootChoiceCell {
        domain: RootChoiceDomain::BigRing,
        mounts: 3,
        user_publishes: 2,
        states: 92565,
        confirmed_states: 46890,
        first_self_verified_root_is_not_the_newest: 92565,
        same_txg_different_instance_roots: 0,
        higher_instance_lower_txg_roots: 0,
        two_or_more_instances: 92565,
        ideal_recovery_applied: 49068,
        above_chosen_root_maximum: 2,
        largest_counter: 12,
        oldest_root_kj1: 46890,
        oldest_root_kj2: 92565,
    },
    PinnedRootChoiceCell {
        domain: RootChoiceDomain::BigRing,
        mounts: 3,
        user_publishes: 3,
        states: 512514,
        confirmed_states: 453195,
        first_self_verified_root_is_not_the_newest: 512514,
        same_txg_different_instance_roots: 0,
        higher_instance_lower_txg_roots: 0,
        two_or_more_instances: 512514,
        ideal_recovery_applied: 274563,
        above_chosen_root_maximum: 2,
        largest_counter: 14,
        oldest_root_kj1: 453195,
        oldest_root_kj2: 512514,
    },
    PinnedRootChoiceCell {
        domain: RootChoiceDomain::BigRing,
        mounts: 3,
        user_publishes: 4,
        states: 2225961,
        confirmed_states: 2225961,
        first_self_verified_root_is_not_the_newest: 2225961,
        same_txg_different_instance_roots: 0,
        higher_instance_lower_txg_roots: 0,
        two_or_more_instances: 2225961,
        ideal_recovery_applied: 1186326,
        above_chosen_root_maximum: 2,
        largest_counter: 16,
        oldest_root_kj1: 2225961,
        oldest_root_kj2: 2225961,
    },
    PinnedRootChoiceCell {
        domain: RootChoiceDomain::SmallRingNine,
        mounts: 1,
        user_publishes: 0,
        states: 10,
        confirmed_states: 0,
        first_self_verified_root_is_not_the_newest: 6,
        same_txg_different_instance_roots: 0,
        higher_instance_lower_txg_roots: 0,
        two_or_more_instances: 6,
        ideal_recovery_applied: 3,
        above_chosen_root_maximum: 1,
        largest_counter: 2,
        oldest_root_kj1: 0,
        oldest_root_kj2: 6,
    },
    PinnedRootChoiceCell {
        domain: RootChoiceDomain::SmallRingNine,
        mounts: 1,
        user_publishes: 1,
        states: 39,
        confirmed_states: 0,
        first_self_verified_root_is_not_the_newest: 39,
        same_txg_different_instance_roots: 0,
        higher_instance_lower_txg_roots: 0,
        two_or_more_instances: 39,
        ideal_recovery_applied: 21,
        above_chosen_root_maximum: 2,
        largest_counter: 4,
        oldest_root_kj1: 0,
        oldest_root_kj2: 39,
    },
    PinnedRootChoiceCell {
        domain: RootChoiceDomain::SmallRingNine,
        mounts: 1,
        user_publishes: 2,
        states: 117,
        confirmed_states: 117,
        first_self_verified_root_is_not_the_newest: 117,
        same_txg_different_instance_roots: 0,
        higher_instance_lower_txg_roots: 0,
        two_or_more_instances: 117,
        ideal_recovery_applied: 63,
        above_chosen_root_maximum: 2,
        largest_counter: 6,
        oldest_root_kj1: 117,
        oldest_root_kj2: 117,
    },
    PinnedRootChoiceCell {
        domain: RootChoiceDomain::SmallRingNine,
        mounts: 2,
        user_publishes: 0,
        states: 125,
        confirmed_states: 0,
        first_self_verified_root_is_not_the_newest: 109,
        same_txg_different_instance_roots: 0,
        higher_instance_lower_txg_roots: 0,
        two_or_more_instances: 109,
        ideal_recovery_applied: 57,
        above_chosen_root_maximum: 1,
        largest_counter: 4,
        oldest_root_kj1: 0,
        oldest_root_kj2: 109,
    },
    PinnedRootChoiceCell {
        domain: RootChoiceDomain::SmallRingNine,
        mounts: 2,
        user_publishes: 1,
        states: 780,
        confirmed_states: 0,
        first_self_verified_root_is_not_the_newest: 780,
        same_txg_different_instance_roots: 0,
        higher_instance_lower_txg_roots: 0,
        two_or_more_instances: 780,
        ideal_recovery_applied: 411,
        above_chosen_root_maximum: 2,
        largest_counter: 6,
        oldest_root_kj1: 0,
        oldest_root_kj2: 780,
    },
    PinnedRootChoiceCell {
        domain: RootChoiceDomain::SmallRingNine,
        mounts: 2,
        user_publishes: 2,
        states: 4401,
        confirmed_states: 2880,
        first_self_verified_root_is_not_the_newest: 4401,
        same_txg_different_instance_roots: 0,
        higher_instance_lower_txg_roots: 0,
        two_or_more_instances: 4401,
        ideal_recovery_applied: 2376,
        above_chosen_root_maximum: 2,
        largest_counter: 9,
        oldest_root_kj1: 2880,
        oldest_root_kj2: 4401,
    },
    PinnedRootChoiceCell {
        domain: RootChoiceDomain::WrappingRootRing,
        mounts: 1,
        user_publishes: 0,
        states: 10,
        confirmed_states: 0,
        first_self_verified_root_is_not_the_newest: 6,
        same_txg_different_instance_roots: 0,
        higher_instance_lower_txg_roots: 0,
        two_or_more_instances: 6,
        ideal_recovery_applied: 3,
        above_chosen_root_maximum: 1,
        largest_counter: 2,
        oldest_root_kj1: 0,
        oldest_root_kj2: 6,
    },
    PinnedRootChoiceCell {
        domain: RootChoiceDomain::WrappingRootRing,
        mounts: 1,
        user_publishes: 1,
        states: 5,
        confirmed_states: 0,
        first_self_verified_root_is_not_the_newest: 5,
        same_txg_different_instance_roots: 0,
        higher_instance_lower_txg_roots: 0,
        two_or_more_instances: 5,
        ideal_recovery_applied: 3,
        above_chosen_root_maximum: 1,
        largest_counter: 3,
        oldest_root_kj1: 0,
        oldest_root_kj2: 5,
    },
    PinnedRootChoiceCell {
        domain: RootChoiceDomain::WrappingRootRing,
        mounts: 1,
        user_publishes: 2,
        states: 5,
        confirmed_states: 5,
        first_self_verified_root_is_not_the_newest: 5,
        same_txg_different_instance_roots: 0,
        higher_instance_lower_txg_roots: 0,
        two_or_more_instances: 5,
        ideal_recovery_applied: 3,
        above_chosen_root_maximum: 1,
        largest_counter: 4,
        oldest_root_kj1: 5,
        oldest_root_kj2: 5,
    },
    PinnedRootChoiceCell {
        domain: RootChoiceDomain::WrappingRootRing,
        mounts: 1,
        user_publishes: 3,
        states: 5,
        confirmed_states: 5,
        first_self_verified_root_is_not_the_newest: 5,
        same_txg_different_instance_roots: 0,
        higher_instance_lower_txg_roots: 0,
        two_or_more_instances: 5,
        ideal_recovery_applied: 3,
        above_chosen_root_maximum: 1,
        largest_counter: 5,
        oldest_root_kj1: 5,
        oldest_root_kj2: 5,
    },
    PinnedRootChoiceCell {
        domain: RootChoiceDomain::WrappingRootRing,
        mounts: 1,
        user_publishes: 4,
        states: 5,
        confirmed_states: 5,
        first_self_verified_root_is_not_the_newest: 5,
        same_txg_different_instance_roots: 0,
        higher_instance_lower_txg_roots: 0,
        two_or_more_instances: 5,
        ideal_recovery_applied: 3,
        above_chosen_root_maximum: 1,
        largest_counter: 6,
        oldest_root_kj1: 5,
        oldest_root_kj2: 5,
    },
    PinnedRootChoiceCell {
        domain: RootChoiceDomain::WrappingRootRing,
        mounts: 1,
        user_publishes: 5,
        states: 5,
        confirmed_states: 5,
        first_self_verified_root_is_not_the_newest: 5,
        same_txg_different_instance_roots: 0,
        higher_instance_lower_txg_roots: 0,
        two_or_more_instances: 5,
        ideal_recovery_applied: 3,
        above_chosen_root_maximum: 1,
        largest_counter: 7,
        oldest_root_kj1: 5,
        oldest_root_kj2: 5,
    },
    PinnedRootChoiceCell {
        domain: RootChoiceDomain::WrappingRootRing,
        mounts: 1,
        user_publishes: 6,
        states: 5,
        confirmed_states: 5,
        first_self_verified_root_is_not_the_newest: 5,
        same_txg_different_instance_roots: 0,
        higher_instance_lower_txg_roots: 0,
        two_or_more_instances: 5,
        ideal_recovery_applied: 3,
        above_chosen_root_maximum: 1,
        largest_counter: 8,
        oldest_root_kj1: 5,
        oldest_root_kj2: 5,
    },
    PinnedRootChoiceCell {
        domain: RootChoiceDomain::WrappingRootRing,
        mounts: 1,
        user_publishes: 7,
        states: 5,
        confirmed_states: 5,
        first_self_verified_root_is_not_the_newest: 5,
        same_txg_different_instance_roots: 0,
        higher_instance_lower_txg_roots: 0,
        two_or_more_instances: 5,
        ideal_recovery_applied: 3,
        above_chosen_root_maximum: 1,
        largest_counter: 9,
        oldest_root_kj1: 5,
        oldest_root_kj2: 5,
    },
    PinnedRootChoiceCell {
        domain: RootChoiceDomain::WrappingRootRing,
        mounts: 1,
        user_publishes: 8,
        states: 5,
        confirmed_states: 5,
        first_self_verified_root_is_not_the_newest: 5,
        same_txg_different_instance_roots: 0,
        higher_instance_lower_txg_roots: 0,
        two_or_more_instances: 5,
        ideal_recovery_applied: 3,
        above_chosen_root_maximum: 1,
        largest_counter: 10,
        oldest_root_kj1: 5,
        oldest_root_kj2: 5,
    },
    PinnedRootChoiceCell {
        domain: RootChoiceDomain::WrappingRootRing,
        mounts: 1,
        user_publishes: 9,
        states: 5,
        confirmed_states: 5,
        first_self_verified_root_is_not_the_newest: 5,
        same_txg_different_instance_roots: 0,
        higher_instance_lower_txg_roots: 0,
        two_or_more_instances: 5,
        ideal_recovery_applied: 3,
        above_chosen_root_maximum: 1,
        largest_counter: 11,
        oldest_root_kj1: 5,
        oldest_root_kj2: 5,
    },
    PinnedRootChoiceCell {
        domain: RootChoiceDomain::WrappingRootRing,
        mounts: 1,
        user_publishes: 10,
        states: 5,
        confirmed_states: 5,
        first_self_verified_root_is_not_the_newest: 4,
        same_txg_different_instance_roots: 0,
        higher_instance_lower_txg_roots: 0,
        two_or_more_instances: 4,
        ideal_recovery_applied: 3,
        above_chosen_root_maximum: 1,
        largest_counter: 12,
        oldest_root_kj1: 4,
        oldest_root_kj2: 4,
    },
    PinnedRootChoiceCell {
        domain: RootChoiceDomain::WrappingRootRing,
        mounts: 1,
        user_publishes: 11,
        states: 5,
        confirmed_states: 5,
        first_self_verified_root_is_not_the_newest: 1,
        same_txg_different_instance_roots: 0,
        higher_instance_lower_txg_roots: 0,
        two_or_more_instances: 0,
        ideal_recovery_applied: 3,
        above_chosen_root_maximum: 1,
        largest_counter: 13,
        oldest_root_kj1: 0,
        oldest_root_kj2: 0,
    },
    PinnedRootChoiceCell {
        domain: RootChoiceDomain::WrappingRootRing,
        mounts: 1,
        user_publishes: 12,
        states: 5,
        confirmed_states: 5,
        first_self_verified_root_is_not_the_newest: 5,
        same_txg_different_instance_roots: 0,
        higher_instance_lower_txg_roots: 0,
        two_or_more_instances: 0,
        ideal_recovery_applied: 3,
        above_chosen_root_maximum: 1,
        largest_counter: 14,
        oldest_root_kj1: 0,
        oldest_root_kj2: 0,
    },
    PinnedRootChoiceCell {
        domain: RootChoiceDomain::WrappingRootRing,
        mounts: 1,
        user_publishes: 13,
        states: 5,
        confirmed_states: 5,
        first_self_verified_root_is_not_the_newest: 5,
        same_txg_different_instance_roots: 0,
        higher_instance_lower_txg_roots: 0,
        two_or_more_instances: 0,
        ideal_recovery_applied: 3,
        above_chosen_root_maximum: 1,
        largest_counter: 15,
        oldest_root_kj1: 0,
        oldest_root_kj2: 0,
    },
    PinnedRootChoiceCell {
        domain: RootChoiceDomain::WrappingRootRing,
        mounts: 1,
        user_publishes: 14,
        states: 5,
        confirmed_states: 5,
        first_self_verified_root_is_not_the_newest: 5,
        same_txg_different_instance_roots: 0,
        higher_instance_lower_txg_roots: 0,
        two_or_more_instances: 0,
        ideal_recovery_applied: 3,
        above_chosen_root_maximum: 1,
        largest_counter: 16,
        oldest_root_kj1: 0,
        oldest_root_kj2: 0,
    },
];

// ── 阳性对照（登记 5.7；新的几条读实现的结局或调三步，不进启动自检，放进单测，产物里打 `name=control` 行）

#[derive(Clone, Debug)]
struct RootChoiceControlReading {
    criterion: RootChoiceCriterion,
    measured_where: String,
    value: u64,
    must_see: String,
    is_red_as_required: bool,
}

fn root_choice_reading_that_must_equal(
    criterion: RootChoiceCriterion,
    measured_where: String,
    value: u64,
    expected: u64,
) -> RootChoiceControlReading {
    RootChoiceControlReading {
        criterion,
        measured_where,
        value,
        must_see: format!("equals_{expected}"),
        is_red_as_required: value == expected,
    }
}

fn root_choice_place_of(domain: RootChoiceDomain, cell: RootChoiceCell) -> String {
    format!(
        "domain={} ring_slots={} slots_per_region={} mounts={} user_publishes={}",
        domain.label(),
        domain.ring().ring_slots,
        domain.slots_per_region(),
        cell.mounts,
        cell.user_publishes
    )
}

/// 每条判据在每个 (域, M, U) 格上的对照读数：KJ0 在 K 域 M = 1、U = 0 的指定一态上 panic（违例数 1）；KJ1、KJ2 在 KF1 下等于第七节乙类 KF1 那两列；
/// KJ1、KJ2、KJ3 在 KF2 下各等于「有已确认发布的态数」；KJ2 在 KA 上本地写的几种择根序下等于 36、18、18、52（今天的序 0）。
fn root_choice_control_readings(
    selected_cells: &[(RootChoiceDomain, Vec<RootChoiceCell>)],
    thread_count: usize,
) -> Vec<RootChoiceControlReading> {
    let mut readings = Vec::new();
    for (domain, cells) in selected_cells {
        for cell in cells {
            let place = root_choice_place_of(*domain, *cell);
            if !domain.is_a_legal_history() {
                for rule in LocalRootChoiceRule::ALL {
                    let tally = root_choice_tally_of_a_cell(
                        *domain,
                        *cell,
                        RootChoiceImplementation::LocalChoice(rule),
                        thread_count,
                    );
                    readings.push(root_choice_reading_that_must_equal(
                        RootChoiceCriterion::Kj2NotEarlierThanAnySelfVerifiedRoot,
                        format!("{place} local_choice={}", rule.label()),
                        tally.get(RootChoiceCounter::Kj2Violations),
                        pinned_root_choice_adversarial_kj2_violations(rule),
                    ));
                }
                continue;
            }
            let pinned = pinned_root_choice_cell(*domain, *cell)
                .expect("每个跑到的合法历史格都在登记第七节乙类的表里");
            if *domain == RootChoiceDomain::BigRing && cell.mounts == 1 && cell.user_publishes == 0
            {
                let tally = root_choice_tally_of_a_cell(
                    *domain,
                    *cell,
                    RootChoiceImplementation::PanicsOnTheDesignatedState,
                    thread_count,
                );
                readings.push(root_choice_reading_that_must_equal(
                    RootChoiceCriterion::Kj0Defined,
                    format!("{place} fake_outcome=panics_on_the_designated_state"),
                    tally.get(RootChoiceCounter::Kj0Violations),
                    1,
                ));
            }
            let kf1 = root_choice_tally_of_a_cell(
                *domain,
                *cell,
                RootChoiceImplementation::ChoosesTheOldestRoot,
                thread_count,
            );
            for (criterion, counter, expected) in [
                (
                    RootChoiceCriterion::Kj1NotEarlierThanTheLastConfirmedPublish,
                    RootChoiceCounter::Kj1Violations,
                    pinned.oldest_root_kj1,
                ),
                (
                    RootChoiceCriterion::Kj2NotEarlierThanAnySelfVerifiedRoot,
                    RootChoiceCounter::Kj2Violations,
                    pinned.oldest_root_kj2,
                ),
            ] {
                readings.push(root_choice_reading_that_must_equal(
                    criterion,
                    format!("{place} fake_outcome=KF1_chooses_the_oldest_root"),
                    kf1.get(counter),
                    expected,
                ));
            }
            let kf2 = root_choice_tally_of_a_cell(
                *domain,
                *cell,
                RootChoiceImplementation::LandsOneVersionBeforeTheLastConfirmed,
                thread_count,
            );
            for (criterion, counter) in [
                (
                    RootChoiceCriterion::Kj1NotEarlierThanTheLastConfirmedPublish,
                    RootChoiceCounter::Kj1Violations,
                ),
                (
                    RootChoiceCriterion::Kj2NotEarlierThanAnySelfVerifiedRoot,
                    RootChoiceCounter::Kj2Violations,
                ),
                (
                    RootChoiceCriterion::Kj3LandsOnTheIdealRecovery,
                    RootChoiceCounter::Kj3Violations,
                ),
            ] {
                readings.push(root_choice_reading_that_must_equal(
                    criterion,
                    format!("{place} fake_outcome=KF2_lands_one_version_before_the_last_confirmed"),
                    kf2.get(counter),
                    pinned.confirmed_states,
                ));
            }
        }
    }
    readings
}

// ── 产物行

/// `name=root_choice_states` 行里数的量，按这个次序逐个写 `名字=数`。
const ROOT_CHOICE_COUNTERS_OF_THE_STATES_ROW: [RootChoiceCounter; 18] = [
    RootChoiceCounter::States,
    RootChoiceCounter::ConfirmedStates,
    RootChoiceCounter::FirstSelfVerifiedRootIsNotTheNewest,
    RootChoiceCounter::SameTxgDifferentInstanceRoots,
    RootChoiceCounter::HigherInstanceLowerTxgRoots,
    RootChoiceCounter::TwoOrMoreInstancesInTheRing,
    RootChoiceCounter::IdealRecoveryApplied,
    RootChoiceCounter::AboveNewestRootOverLimit,
    RootChoiceCounter::AboveNewestRootAtLimit,
    RootChoiceCounter::AboveChosenRootOverLimit,
    RootChoiceCounter::AboveChosenRootAtLimit,
    RootChoiceCounter::StatesWithoutASelfVerifiedRoot,
    RootChoiceCounter::PanickedStates,
    RootChoiceCounter::ChoseNothingStates,
    RootChoiceCounter::ReplayFailedStates,
    RootChoiceCounter::ReplayAppliedStates,
    RootChoiceCounter::SlotReadingsChecked,
    RootChoiceCounter::SlotReadingsDifferingFromTheTruth,
];

/// 一个 (域, M, U) 格的全部行：状态数与几样计数、每条判据一行违例数（KA 只判 KJ0、KJ2，另两条不出行）。
fn emit_the_rows_of_a_root_choice_cell(
    emitter: &mut Emitter,
    domain: RootChoiceDomain,
    cell: RootChoiceCell,
    tally: &RootChoiceTally,
) {
    let place = root_choice_place_of(domain, cell);
    let counter_fields: Vec<String> = ROOT_CHOICE_COUNTERS_OF_THE_STATES_ROW
        .iter()
        .map(|counter| format!("{}={}", counter.label(), tally.get(*counter)))
        .collect();
    emitter.result(&format!(
        "name=root_choice_states {place} {} in_flight_limit={} above_newest_root_max={} above_chosen_root_max={} confirmed_publishes_max={} largest_counter={} slot_reading_witness={}",
        counter_fields.join(" "),
        domain.ring().local_in_flight_limit(),
        tally.above_newest_root_maximum,
        tally.above_chosen_root_maximum,
        tally.confirmed_publishes_maximum,
        tally.largest_counter,
        tally.witness(RootChoiceCounter::SlotReadingsDifferingFromTheTruth),
    ));
    let mut violations_row =
        |criterion: RootChoiceCriterion, violations: RootChoiceCounter, judged: u64| {
            emitter.result(&format!(
                "name=violations {place} criterion={} violations={} judged={judged} witness={}",
                criterion.label(),
                tally.get(violations),
                tally.witness(violations)
            ));
        };
    violations_row(
        RootChoiceCriterion::Kj0Defined,
        RootChoiceCounter::Kj0Violations,
        tally.get(RootChoiceCounter::States),
    );
    if domain.is_a_legal_history() {
        violations_row(
            RootChoiceCriterion::Kj1NotEarlierThanTheLastConfirmedPublish,
            RootChoiceCounter::Kj1Violations,
            tally.get(RootChoiceCounter::Kj1Judged),
        );
    }
    violations_row(
        RootChoiceCriterion::Kj2NotEarlierThanAnySelfVerifiedRoot,
        RootChoiceCounter::Kj2Violations,
        tally.get(RootChoiceCounter::Kj2Judged),
    );
    if domain.is_a_legal_history() {
        violations_row(
            RootChoiceCriterion::Kj3LandsOnTheIdealRecovery,
            RootChoiceCounter::Kj3Violations,
            tally.get(RootChoiceCounter::Kj3Judged),
        );
    }
}

/// 一个域扫完之后的判决行（各判据在这个域全部格上的合计），并记作废与停机。
fn emit_the_verdict_of_a_root_choice_domain(
    emitter: &mut Emitter,
    findings: &mut RunFindings,
    domain: RootChoiceDomain,
    cells: &[RootChoiceCell],
    total: &RootChoiceTally,
) {
    let largest_mounts = cells.iter().map(|cell| cell.mounts).max().unwrap_or(0);
    let largest_user_publishes = cells
        .iter()
        .map(|cell| cell.user_publishes)
        .max()
        .unwrap_or(0);
    // 登记作废条款 TV10：域是空的（合法历史三个域合起来没有一个已确认的态，KJ1 一态都没判；KA 上同 txg 不同实例同在为 0）。
    let domain_is_nonempty = if domain.is_a_legal_history() {
        total.get(RootChoiceCounter::ConfirmedStates) > 0
    } else {
        total.get(RootChoiceCounter::SameTxgDifferentInstanceRoots) > 0
            && total.get(RootChoiceCounter::HigherInstanceLowerTxgRoots) > 0
    };
    if !domain_is_nonempty {
        findings.void_conditions.push(format!(
            "TV10 {} 域是空的（合法历史域没有一个已确认发布的态；KA 上同 txg 不同实例同在与实例代号更高而 txg 更低同在有一样为 0），这个域的 0 违例不算数",
            domain.label()
        ));
    }
    // 登记停机条款 TH4：读槽对拍不一致，不当 KJ 的违例记。
    if total.get(RootChoiceCounter::SlotReadingsDifferingFromTheTruth) > 0 {
        findings.halt_conditions.push(format!(
            "TH4 {} 域里 read_root_ring_slot 交回的与生成时的真相不同的槽数 {}（见证 {}）",
            domain.label(),
            total.get(RootChoiceCounter::SlotReadingsDifferingFromTheTruth),
            total.witness(RootChoiceCounter::SlotReadingsDifferingFromTheTruth)
        ));
    }
    // 登记第七节甲类、失败条款 TF9：合法历史三个域里今天的 choose_root 交回空、replay_journal 报错的态数是 0。
    if domain.is_a_legal_history()
        && total.get(RootChoiceCounter::ChoseNothingStates)
            + total.get(RootChoiceCounter::ReplayFailedStates)
            > 0
    {
        findings.halt_conditions.push(format!(
            "TF9 {} 域里 choose_root 交回空的态数 {}、replay_journal 报错的态数 {}（登记第七节甲类锚点都是 0）",
            domain.label(),
            total.get(RootChoiceCounter::ChoseNothingStates),
            total.get(RootChoiceCounter::ReplayFailedStates)
        ));
    }
    emitter.result(&format!(
        "name=verdict domain={} ring_slots={} slots_per_region={} largest_mounts={largest_mounts} largest_user_publishes={largest_user_publishes} criteria=root_choice states={} confirmed_states={} kj0_violations={} panicked={} choose_root_returned_none={} replay_failed={} kj1_violations={} kj1_judged={} kj2_violations={} kj2_judged={} kj3_violations={} kj3_judged={} slot_readings_differing_from_the_truth={} domain_is_nonempty={domain_is_nonempty}",
        domain.label(),
        domain.ring().ring_slots,
        domain.slots_per_region(),
        total.get(RootChoiceCounter::States),
        total.get(RootChoiceCounter::ConfirmedStates),
        total.get(RootChoiceCounter::Kj0Violations),
        total.get(RootChoiceCounter::PanickedStates),
        total.get(RootChoiceCounter::ChoseNothingStates),
        total.get(RootChoiceCounter::ReplayFailedStates),
        total.get(RootChoiceCounter::Kj1Violations),
        total.get(RootChoiceCounter::Kj1Judged),
        total.get(RootChoiceCounter::Kj2Violations),
        total.get(RootChoiceCounter::Kj2Judged),
        total.get(RootChoiceCounter::Kj3Violations),
        total.get(RootChoiceCounter::Kj3Judged),
        total.get(RootChoiceCounter::SlotReadingsDifferingFromTheTruth),
    ));
}

// ── 命令行与主流程（`--family root_choice`，只在 `--profile r3`）

struct RootChoiceSelection {
    bounds: BoundsTier,
    only: Option<Vec<String>>,
    largest_user_publishes_cap: Option<usize>,
}

const ROOT_CHOICE_FAMILY_LABEL: &str = "root_choice";

fn arguments_name_the_root_choice_family(arguments: &[String]) -> bool {
    arguments
        .windows(2)
        .any(|pair| pair[0] == "--family" && pair[1] == ROOT_CHOICE_FAMILY_LABEL)
}

/// `--bounds segment1|segment2|light`（这一族 segment1 与 segment2 同界、`light` 是调试用的小界）、`--family root_choice`、
/// `--only K,K9,KW,KA`、`--max-u <U>`（每个格的 U 封顶，只给调试用）。
fn parse_the_root_choice_command_line(arguments: &[String]) -> Result<RootChoiceSelection, String> {
    let mut bounds: Option<BoundsTier> = None;
    let mut only: Option<Vec<String>> = None;
    let mut largest_user_publishes_cap: Option<usize> = None;
    let mut position = 0;
    while position < arguments.len() {
        let option = arguments[position].as_str();
        let value = arguments
            .get(position + 1)
            .ok_or_else(|| format!("{option} 后面缺一个值"))?
            .as_str();
        match option {
            "--bounds" => {
                bounds = Some(match value {
                    "segment1" => BoundsTier::SegmentOne,
                    "segment2" => BoundsTier::SegmentTwo,
                    "light" => BoundsTier::Light,
                    unknown_tier => return Err(format!("不认识的界：{unknown_tier}")),
                });
            }
            "--family" => {
                if value != ROOT_CHOICE_FAMILY_LABEL {
                    return Err(format!("不认识的族：{value}"));
                }
            }
            "--only" => {
                let names: Vec<String> = value.split(',').map(str::to_string).collect();
                for name in &names {
                    if !RootChoiceDomain::ALL
                        .iter()
                        .any(|domain| domain.label() == name)
                    {
                        return Err(format!("不认识的域：{name}（这一族的域是 K、K9、KW、KA）"));
                    }
                }
                only = Some(names);
            }
            "--max-u" => {
                largest_user_publishes_cap = Some(
                    value
                        .parse::<usize>()
                        .map_err(|error| format!("--max-u 要一个非负整数：{error}"))?,
                );
            }
            unknown_option => return Err(format!("不认识的参数：{unknown_option}")),
        }
        position += 2;
    }
    Ok(RootChoiceSelection {
        bounds: bounds.ok_or_else(|| "缺 --bounds".to_string())?,
        only,
        largest_user_publishes_cap,
    })
}

fn run_the_root_choice_family(arguments: &[String], run_profile: RunProfile) {
    let selection = match parse_the_root_choice_command_line(arguments) {
        Ok(selection) => selection,
        Err(problem) => {
            eprintln!("  ✗ {problem}");
            eprintln!("  → 怎么办：{USAGE}");
            std::process::exit(2);
        }
    };
    let thread_count = thread_count_from_the_environment();
    install_the_panic_hook_that_stays_quiet_inside_the_replay_call();
    let run_started = Instant::now();
    let mut emitter = Emitter { emitted: 0 };
    let mut findings = RunFindings::default();
    let selected_cells: Vec<(RootChoiceDomain, Vec<RootChoiceCell>)> = RootChoiceDomain::ALL
        .iter()
        .copied()
        .filter(|domain| {
            selection
                .only
                .as_ref()
                .is_none_or(|names| names.iter().any(|name| name == domain.label()))
        })
        .map(|domain| {
            (
                domain,
                root_choice_cells_of(
                    domain,
                    selection.bounds,
                    selection.largest_user_publishes_cap,
                ),
            )
        })
        .collect();
    emitter.result(&format!(
        "name=config experiment=E165 bounds={} family={ROOT_CHOICE_FAMILY_LABEL} only={} max_u={} root_slot_bytes={ROOT_CHOICE_ROOT_SLOT_BYTES} region_devices=0,1,0 slots_per_region_wide={ROOT_CHOICE_SLOTS_PER_REGION_WIDE} slots_per_region_narrow={ROOT_CHOICE_SLOTS_PER_REGION_NARROW}{}",
        selection.bounds.label(),
        selection
            .only
            .as_ref()
            .map_or_else(|| "every_domain".to_string(), |names| names.join(",")),
        selection
            .largest_user_publishes_cap
            .map_or_else(|| "none".to_string(), |cap| cap.to_string()),
        profile_fields_of_the_run_rows(run_profile),
    ));

    // ① 常量回比断言（不过当场 panic，进程以非 0 退出，一行结果都不算）。
    cross_check_root_choice_local_constants_against_the_implementation();
    emitter.result(
        "name=self_check check=root_choice_local_constants_equal_the_implementation passed=true",
    );
    let self_checks_passed = 1;
    for ring in [RingGeometry::BIG, RingGeometry::SMALL_NINE] {
        emitter.result(&format!(
            "name=in_flight_limit ring_slots={} local={} implementation={}",
            ring.ring_slots,
            ring.local_in_flight_limit(),
            journal_in_flight_record_limit(ring.ring_bytes())
        ));
    }

    // ② 阳性对照：每条判据都跑；有一行没红，这一段作废（V2）。
    let controls_started = Instant::now();
    let control_readings = root_choice_control_readings(&selected_cells, thread_count);
    for reading in &control_readings {
        emitter.result(&format!(
            "name=control criterion={} measured={} value={} must_see={} red_as_required={}",
            reading.criterion.label(),
            reading.measured_where.replace(' ', ","),
            reading.value,
            reading.must_see,
            reading.is_red_as_required
        ));
        if !reading.is_red_as_required {
            findings.void_conditions.push(format!(
                "V2 判据 {} 的阳性对照没红：{} 量到 {}，必须 {}",
                reading.criterion.label(),
                reading.measured_where,
                reading.value,
                reading.must_see
            ));
        }
    }
    println!(
        "E7TIMING name=elapsed scope=positive_controls elapsed_seconds={:.3} threads={thread_count}",
        controls_started.elapsed().as_secs_f64()
    );

    // ③ 各域各格：今天的三步。
    let mut total_states: u64 = 0;
    for (domain, cells) in &selected_cells {
        let mut domain_total = RootChoiceTally::new();
        for cell in cells {
            let cell_started = Instant::now();
            let tally = root_choice_tally_of_a_cell(
                *domain,
                *cell,
                RootChoiceImplementation::TodayThreeSteps,
                thread_count,
            );
            if domain.is_a_legal_history() {
                assert_the_pinned_root_choice_cell(*domain, *cell, &tally);
            } else {
                assert_eq!(
                    tally.get(RootChoiceCounter::States),
                    PINNED_ROOT_CHOICE_ADVERSARIAL_STATES,
                    "TV4：KA 状态数"
                );
                assert_eq!(
                    tally.get(RootChoiceCounter::StatesWithoutASelfVerifiedRoot),
                    1,
                    "TV4：KA 里根环全空的态数（登记第七节乙类 KA：没有一条自证过的根的 1 态）"
                );
            }
            emit_the_rows_of_a_root_choice_cell(&mut emitter, *domain, *cell, &tally);
            println!(
                "E7TIMING name=elapsed scope=cell domain={} mounts={} user_publishes={} states={} elapsed_seconds={:.3} threads={thread_count}",
                domain.label(),
                cell.mounts,
                cell.user_publishes,
                tally.get(RootChoiceCounter::States),
                cell_started.elapsed().as_secs_f64()
            );
            domain_total.merge(&tally);
        }
        total_states += domain_total.get(RootChoiceCounter::States);
        emit_the_verdict_of_a_root_choice_domain(
            &mut emitter,
            &mut findings,
            *domain,
            cells,
            &domain_total,
        );
    }

    for condition in &findings.void_conditions {
        emitter.result(&format!(
            "name=void condition={}",
            condition.replace(' ', "_")
        ));
    }
    for condition in &findings.halt_conditions {
        emitter.result(&format!(
            "name=halt condition={}",
            condition.replace(' ', "_")
        ));
    }
    emitter.result(&format!(
        "name=verdict scope=run bounds={} family={ROOT_CHOICE_FAMILY_LABEL} states={total_states} self_checks_passed={self_checks_passed} control_readings={} controls_not_red={} void_conditions={} halt_conditions={}{}",
        selection.bounds.label(),
        control_readings.len(),
        control_readings
            .iter()
            .filter(|reading| !reading.is_red_as_required)
            .count(),
        findings.void_conditions.len(),
        findings.halt_conditions.len(),
        profile_fields_of_the_run_rows(run_profile),
    ));
    println!(
        "E7TIMING name=elapsed scope=run states={total_states} elapsed_seconds={:.3} threads={thread_count}",
        run_started.elapsed().as_secs_f64()
    );
    emitter.done();
    if !findings.void_conditions.is_empty() || !findings.halt_conditions.is_empty() {
        std::process::exit(1);
    }
}

// ── 带读故障的择根合判一族（登记 `research/prompts/e165-r4-prereg.md` 5.3 至 5.6、第七节：KF、KF3、KF9、KFW、KG、KA；只在 `--profile r4 --family root_choice_faults` 时跑）
//
// 被测对象与 r3 的择根合判一族相同：`recover` 的 `Consult` 支里串起来的三步（`choose_root`、`scan_journal`、`replay_journal(…, true)`）。
// 这一族多出来的东西全在生成器一侧（可信件，按登记 5.3 与 D16 已定项 7、8，D22 已定项 7、8、16，D23 已定项 14、18 的字面写，不调 `mount.rs`、`transaction.rs`）：
// 每块盘两个系统配置槽、挂载开头的读故障（根槽、系统配置槽的一时读不出）、可写挂载读到的更新状态读不出那道闸（不给 `--gate` 是 r4 登记修订 1 的取甲；
// `--gate r2` 是 r5 登记修订 1 的 R2）、取号那一写与发布末尾的系统配置轮换。
// 被判的那一次恢复另有 5 种读故障（无、RN、RI、D0、D1）；故障只落在读者一侧，盘上的字节不变。

const FAULT_FAMILY_LABEL: &str = "root_choice_faults";
/// 每块盘的系统配置槽数、mkfs 写下的世代号、可写挂载的设备数下限：本地常量，抄自 D22 已定项 16「每盘恒 2 个槽……槽世代号从 1 起」与 D2 已定项 13
/// 「可写设备数 ≥ w 的下限」（w = 2）；[`cross_check_root_choice_fault_local_constants_against_the_implementation`] 逐个回比。
const FAULT_SYSTEM_SLOTS_PER_DEVICE: usize = 2;
const FAULT_MKFS_SYSTEM_GENERATION: u64 = 1;
const FAULT_WRITABLE_DEVICE_COUNT_LOWER_BOUND: usize = 2;
const FAULT_DEVICE_COUNT: usize = 2;
/// 整池的系统配置槽数：可写挂载读到的自证过的槽不足这个数就判不出（实现的取甲，登记修订 1）。
const FAULT_SYSTEM_SLOT_COUNT_OF_THE_POOL: usize =
    FAULT_DEVICE_COUNT * FAULT_SYSTEM_SLOTS_PER_DEVICE;
/// 去掉在飞上限的理想恢复用（登记 5.5「上限截掉」：去掉上限之后成立）。
const FAULT_IN_FLIGHT_LIMIT_REMOVED: usize = 1_000_000_000;
/// 挂载开头故障菜单的项数（登记 5.3 第 5 条：无，RN、SN0、SN1、SB、RI、RNSB 六种落点各配 T1 / TP 两种时机）。
const MOUNT_FAULT_MENU_LENGTH: usize = 13;
const MOUNT_FAULT_MENU_LABELS: [&str; MOUNT_FAULT_MENU_LENGTH] = [
    "none", "RN-T1", "RN-TP", "SN0-T1", "SN0-TP", "SN1-T1", "SN1-TP", "SB-T1", "SB-TP", "RI-T1",
    "RI-TP", "RNSB-T1", "RNSB-TP",
];

labelled_enumeration! {
    /// 登记 5.4 的六个域：KF、KF3、KF9、KFW 是带读故障的合法历史，KG 是把条款的两道保护拿掉的对照，KA 是 r3 原样的对抗根环。
    enum FaultDomain {
        GateOn => "KF",
        GateOnThreeMounts => "KF3",
        GateOnSmallRing => "KF9",
        GateOnWrappingRootRing => "KFW",
        GateOff => "KG",
        AdversarialRootRing => "KA",
    }
}

impl FaultDomain {
    /// 这个域的几何（环槽数、每区槽数）与 r3 的哪个域相同。
    const fn geometry(self) -> RootChoiceDomain {
        match self {
            Self::GateOn | Self::GateOnThreeMounts | Self::GateOff => RootChoiceDomain::BigRing,
            Self::GateOnSmallRing => RootChoiceDomain::SmallRingNine,
            Self::GateOnWrappingRootRing => RootChoiceDomain::WrappingRootRing,
            Self::AdversarialRootRing => RootChoiceDomain::AdversarialRootRing,
        }
    }

    /// 用户发布每次的记录条数（登记 5.4：KF、KF9、KG 取 {1, 2}，KF3、KFW 取 {1}）。
    const fn user_publish_record_counts(self) -> &'static [usize] {
        match self {
            Self::GateOn | Self::GateOnSmallRing | Self::GateOff => &[1, 2],
            Self::GateOnThreeMounts | Self::GateOnWrappingRootRing => &[1],
            Self::AdversarialRootRing => &[],
        }
    }

    /// 条款的两道保护开着：可写挂载判 N-配置（实现的取甲）与取号那一刻、写行 txg 看记录（登记 5.3 第 6 条）。KG 关掉。
    const fn gate_is_on(self) -> bool {
        match self {
            Self::GateOn
            | Self::GateOnThreeMounts
            | Self::GateOnSmallRing
            | Self::GateOnWrappingRootRing => true,
            Self::GateOff | Self::AdversarialRootRing => false,
        }
    }

    /// 合法历史：Q1、Q2 的域。KG 与 KA 不是。
    const fn is_a_legal_history(self) -> bool {
        self.gate_is_on()
    }

    const fn has_histories(self) -> bool {
        match self {
            Self::AdversarialRootRing => false,
            Self::GateOn
            | Self::GateOnThreeMounts
            | Self::GateOnSmallRing
            | Self::GateOnWrappingRootRing
            | Self::GateOff => true,
        }
    }

    /// 全界（登记 5.4：KF M ≤ 2、U ≤ 3；KF3 M ≤ 3、U ≤ 3；KF9 M ≤ 2、U ≤ 2；KFW M = 1、U ≤ 14；KG M ≤ 2、U ≤ 3）与轻档（调试用，不是登记的界）。
    const fn largest_mounts(self, tier: BoundsTier) -> usize {
        match (self, tier) {
            (Self::AdversarialRootRing, _) => 0,
            (Self::GateOnThreeMounts, BoundsTier::SegmentOne | BoundsTier::SegmentTwo) => 3,
            (
                Self::GateOn | Self::GateOnSmallRing | Self::GateOff,
                BoundsTier::SegmentOne | BoundsTier::SegmentTwo,
            ) => 2,
            (Self::GateOnWrappingRootRing, BoundsTier::SegmentOne | BoundsTier::SegmentTwo) => 1,
            (
                Self::GateOn | Self::GateOnThreeMounts | Self::GateOnSmallRing | Self::GateOff,
                BoundsTier::Light,
            ) => 2,
            (Self::GateOnWrappingRootRing, BoundsTier::Light) => 1,
        }
    }

    const fn largest_user_publishes(self, tier: BoundsTier) -> usize {
        match (self, tier) {
            (Self::AdversarialRootRing, _) => 0,
            (
                Self::GateOn | Self::GateOnThreeMounts | Self::GateOff,
                BoundsTier::SegmentOne | BoundsTier::SegmentTwo,
            ) => 3,
            (Self::GateOnSmallRing, BoundsTier::SegmentOne | BoundsTier::SegmentTwo) => 2,
            (Self::GateOnWrappingRootRing, BoundsTier::SegmentOne | BoundsTier::SegmentTwo) => 14,
            (
                Self::GateOn | Self::GateOnThreeMounts | Self::GateOnSmallRing | Self::GateOff,
                BoundsTier::Light,
            ) => 1,
            (Self::GateOnWrappingRootRing, BoundsTier::Light) => 4,
        }
    }
}

/// 一个域在这一档、这一次命令行下要跑的格，按 (M 升序, U 升序)。`largest_user_publishes_cap` 是 `--max-u`（只给调试用）。
fn fault_cells_of(
    domain: FaultDomain,
    tier: BoundsTier,
    largest_user_publishes_cap: Option<usize>,
) -> Vec<RootChoiceCell> {
    if !domain.has_histories() {
        return vec![RootChoiceCell {
            mounts: 0,
            user_publishes: 0,
        }];
    }
    let largest_user_publishes = largest_user_publishes_cap
        .map_or(domain.largest_user_publishes(tier), |cap| {
            cap.min(domain.largest_user_publishes(tier))
        });
    let mut cells = Vec::new();
    for mounts in 1..=domain.largest_mounts(tier) {
        for user_publishes in 0..=largest_user_publishes {
            cells.push(RootChoiceCell {
                mounts,
                user_publishes,
            });
        }
    }
    cells
}

// ── 读故障的落点（登记 5.3 第 5 条）：根槽、系统配置槽、整块盘（盘不在）；journal 记录槽不单独加读故障，盘不在时那块盘上的记录拷贝随之读不出

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
struct FaultLocations {
    /// 位 = 根槽下标（区域号主序）。
    root_slot_bits: u32,
    /// 位 = 盘号 × 每盘槽数 + 槽号。
    system_slot_bits: u8,
    /// 位 = 盘号。
    dead_device_bits: u8,
}

impl FaultLocations {
    const NOWHERE: Self = Self {
        root_slot_bits: 0,
        system_slot_bits: 0,
        dead_device_bits: 0,
    };

    fn with_root_slot(self, root_slot_index: usize) -> Self {
        Self {
            root_slot_bits: self.root_slot_bits | (1 << root_slot_index),
            ..self
        }
    }

    fn with_system_slot(self, device: usize, slot: usize) -> Self {
        Self {
            system_slot_bits: self.system_slot_bits
                | (1 << (device * FAULT_SYSTEM_SLOTS_PER_DEVICE + slot)),
            ..self
        }
    }

    fn with_dead_device(self, device: usize) -> Self {
        Self {
            dead_device_bits: self.dead_device_bits | (1 << device),
            ..self
        }
    }

    fn union(self, other: Self) -> Self {
        Self {
            root_slot_bits: self.root_slot_bits | other.root_slot_bits,
            system_slot_bits: self.system_slot_bits | other.system_slot_bits,
            dead_device_bits: self.dead_device_bits | other.dead_device_bits,
        }
    }

    fn device_is_dead(self, device: usize) -> bool {
        (self.dead_device_bits >> device) & 1 == 1
    }

    /// 这个根槽读不出：它自己在故障里，或它所在的盘不在。
    fn root_slot_is_unreadable(self, root_slot_index: usize, slots_per_region: u64) -> bool {
        (self.root_slot_bits >> root_slot_index) & 1 == 1
            || self.device_is_dead(root_choice_device_of_root_slot(
                root_slot_index,
                slots_per_region,
            ))
    }

    fn system_slot_is_unreadable(self, device: usize, slot: usize) -> bool {
        (self.system_slot_bits >> (device * FAULT_SYSTEM_SLOTS_PER_DEVICE + slot)) & 1 == 1
            || self.device_is_dead(device)
    }
}

/// 根槽下标（区域号主序）所在的盘：区域归属 0 / 1 / 0。
fn root_choice_device_of_root_slot(root_slot_index: usize, slots_per_region: u64) -> usize {
    let region = root_slot_index / usize::try_from(slots_per_region).expect("每区槽数 4 或 8");
    usize::try_from(ROOT_CHOICE_REGION_DEVICES[region]).expect("盘号是 0 或 1")
}

/// 读者收到的 (盘, 偏移) 落在哪个根槽上（区域归属与槽距照 r3 的内存盘）；不是根槽的落点交回空。
fn root_choice_root_slot_index_at(
    device: DeviceIdentity,
    offset: DeviceOffsetInBytes,
    slots_per_region: u64,
) -> Option<usize> {
    let spacing = u64::from(ROOT_CHOICE_FIXED_STRUCTURE_SLOT_SPACING_BYTES);
    for region in 0..ROOT_CHOICE_REGION_COUNT {
        let region_start = root_choice_region_start_in_bytes(region);
        if offset.0 < region_start || offset.0 >= region_start + slots_per_region * spacing {
            continue;
        }
        let offset_in_region = offset.0 - region_start;
        let region_device =
            ROOT_CHOICE_REGION_DEVICES[usize::try_from(region).expect("区域号不超过 2")];
        if !offset_in_region.is_multiple_of(spacing) || device.0 != region_device {
            return None;
        }
        return Some(root_choice_slot_index_of(
            region,
            offset_in_region / spacing,
            slots_per_region,
        ));
    }
    None
}

// ── 系统配置槽：每块盘两个，每槽 (世代号, tail, 实例代号) 或「撕裂」（新旧都读不出，D13 已定项 4）

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum SystemSlotContent {
    Torn,
    Verified {
        generation: u64,
        tail: u64,
        instance: u32,
    },
}

type SystemSlots = [[SystemSlotContent; FAULT_SYSTEM_SLOTS_PER_DEVICE]; FAULT_DEVICE_COUNT];

/// mkfs 之后：每块盘两槽都是 (世代号 1, tail 0, 实例 0)（登记 5.3 第 2 条）。
fn system_slots_right_after_mkfs() -> SystemSlots {
    [[SystemSlotContent::Verified {
        generation: FAULT_MKFS_SYSTEM_GENERATION,
        tail: 0,
        instance: ROOT_CHOICE_MKFS_INSTANCE,
    }; FAULT_SYSTEM_SLOTS_PER_DEVICE]; FAULT_DEVICE_COUNT]
}

/// 一次系统配置写（取号与发布末尾的轮换都是它，登记 5.3 第 3 条）：世代号 = 这块盘两槽里这一刻读得出（不是撕裂、也不在这次挂载一直读不出的落点里）的
/// 最大世代号 + 1（一槽都读不出时 1），写在槽 世代号 mod 槽数。
fn system_slot_write_of(
    slots: &SystemSlots,
    device: usize,
    tail: u64,
    instance: u32,
    unreadable_all_mount_long: FaultLocations,
) -> (usize, SystemSlotContent) {
    let mut largest_generation = 0;
    for (slot, content) in slots[device].iter().enumerate() {
        if let SystemSlotContent::Verified { generation, .. } = content {
            if !unreadable_all_mount_long.system_slot_is_unreadable(device, slot) {
                largest_generation = largest_generation.max(*generation);
            }
        }
    }
    let generation = largest_generation + 1;
    let slot =
        usize::try_from(generation % u64::try_from(FAULT_SYSTEM_SLOTS_PER_DEVICE).expect("2"))
            .expect("槽号是 0 或 1");
    (
        slot,
        SystemSlotContent::Verified {
            generation,
            tail,
            instance,
        },
    )
}

/// 这个读视图里读得出的系统配置槽的 tail：个数与最大值（没有时 0）。
fn readable_system_slots_of(slots: &SystemSlots, locations: FaultLocations) -> (usize, u64) {
    let mut count = 0;
    let mut largest_tail = 0;
    for (device, device_slots) in slots.iter().enumerate() {
        for (slot, content) in device_slots.iter().enumerate() {
            if let SystemSlotContent::Verified { tail, .. } = content {
                if !locations.system_slot_is_unreadable(device, slot) {
                    count += 1;
                    largest_tail = largest_tail.max(*tail);
                }
            }
        }
    }
    (count, largest_tail)
}

/// 可写挂载读到的更新状态读不出那道闸、与取号那一刻的「判不出」取法（登记 r5 修订 1：`--gate r2` 只在 `--profile r4 --family root_choice_faults`
/// 时认；不给是取甲，r4 档输出逐字节不变）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum WitnessGateRule {
    /// 取甲（r4 登记修订 1，2026-09-28 那一版实现）：读得出且没撕裂的系统配置槽不足整池槽数就判不出；取号那一刻同判。
    OneUnreadableOrTornSlotMeansUndecidable,
    /// R2（D22（单元原子性怎么合成） 已定项 8 射程，今天的 `recovery.rs` 的 `system_configuration_witness_is_undecidable`）：判不出 ⟺ 有槽设备报读错，
    /// 且没有一块盘两槽都自证过；取号那一刻判不出、或某块盘两槽里一份自证过的都没有，就拒。撕裂（读得出、自证不过）不报读错（S-辨）。
    SomeSlotReportsAReadErrorAndNoDeviceSelfVerifiedBothSlots,
}

impl WitnessGateRule {
    /// config 行与收尾判决行里闸那一段。取甲那一份与 r4 档的 config 行逐字相同。
    const fn label(self) -> &'static str {
        match self {
            Self::OneUnreadableOrTornSlotMeansUndecidable => {
                "the_implementation_choice_one_unreadable_or_torn_slot_means_undecidable"
            }
            Self::SomeSlotReportsAReadErrorAndNoDeviceSelfVerifiedBothSlots => {
                "r2_some_slot_reports_a_read_error_and_no_device_self_verified_both_slots"
            }
        }
    }
}

labelled_enumeration! {
    /// 一次挂载被拒在哪一步（登记 r5 修订 1 修1.五第 4 条「拒因分计」：闸那一栏 = 重读那一遍仍为真；取号那一刻的拒按 R2 的两道分；
    /// 取甲下取号那一刻只有一种拒，另记一栏，不进 R2 的三栏）。走了第二遍又在取号那一刻被拒的挂载记取号那一栏。
    enum MountRefusalReason {
        TheGateAfterTheOneReread => "gate",
        AcquisitionDeviceWithoutASelfVerifiedSlot => "acquisition_device_without_a_self_verified_slot",
        AcquisitionWitnessUndecidable => "acquisition_witness_undecidable",
        AcquisitionUnderTheImplementationChoice => "acquisition_under_the_implementation_choice",
    }
}

/// R2 读法：这个系统配置槽是不是「设备报读错」——读故障落点与不在的盘报读错；撕裂的槽读得出、自证不过，设备不报错
/// （D22 已定项 8 射程 S-辨；`recovery.rs` 的 `system_configuration_slots_of_one_device_read_once` 只在读不出时记读错）。
fn system_slot_reports_a_read_error(
    content: SystemSlotContent,
    is_unreadable_in_the_view: bool,
) -> bool {
    match content {
        SystemSlotContent::Torn => is_unreadable_in_the_view,
        SystemSlotContent::Verified { .. } => is_unreadable_in_the_view,
    }
}

/// R2 的判不出（D22 已定项 8 射程「判不出 ⟺ 有槽设备报读错，且没有一块盘两槽都自证过」）：在 `locations` 那一遍读视图里判。
fn system_configuration_witness_is_undecidable_under_r2(
    slots: &SystemSlots,
    locations: FaultLocations,
) -> bool {
    let some_slot_reports_a_read_error = slots.iter().enumerate().any(|(device, device_slots)| {
        device_slots.iter().enumerate().any(|(slot, content)| {
            system_slot_reports_a_read_error(
                *content,
                locations.system_slot_is_unreadable(device, slot),
            )
        })
    });
    let some_device_self_verified_both_slots =
        slots.iter().enumerate().any(|(device, device_slots)| {
            device_slots.iter().enumerate().all(|(slot, content)| {
                *content != SystemSlotContent::Torn
                    && !locations.system_slot_is_unreadable(device, slot)
            })
        });
    some_slot_reports_a_read_error && !some_device_self_verified_both_slots
}

/// 某块盘两槽里一份读得出且没撕裂的都没有（D22 已定项 8 射程「某块盘一份自证过的系统配置都没有时，取号那一步也拒」；
/// 与 `mount.rs` 逐盘核第一支同一判、同一视图）。
fn some_device_has_no_self_verified_system_slot(
    slots: &SystemSlots,
    locations: FaultLocations,
) -> bool {
    slots.iter().enumerate().any(|(device, device_slots)| {
        !device_slots.iter().enumerate().any(|(slot, content)| {
            matches!(content, SystemSlotContent::Verified { .. })
                && !locations.system_slot_is_unreadable(device, slot)
        })
    })
}

/// 取号那一刻（读视图 = 盘减去 TP 的落点，登记 r4 5.3 第 6 条、r5 修订 1 修1.五第 3 条）拒不拒、拒在哪一道：取甲下四槽有一槽读不出或撕裂就拒；
/// R2 下 (a) 判不出拒，(b) 某块盘一份自证过的都没有拒。
fn acquisition_refusal_of(
    world: &FaultWorld,
    locations: FaultLocations,
) -> Option<MountRefusalReason> {
    match world.witness_gate_rule {
        WitnessGateRule::OneUnreadableOrTornSlotMeansUndecidable => {
            let (readable_slot_count, _largest_tail) =
                readable_system_slots_of(&world.system_slots, locations);
            (readable_slot_count < FAULT_SYSTEM_SLOT_COUNT_OF_THE_POOL)
                .then_some(MountRefusalReason::AcquisitionUnderTheImplementationChoice)
        }
        WitnessGateRule::SomeSlotReportsAReadErrorAndNoDeviceSelfVerifiedBothSlots => {
            if system_configuration_witness_is_undecidable_under_r2(&world.system_slots, locations)
            {
                return Some(MountRefusalReason::AcquisitionWitnessUndecidable);
            }
            if some_device_has_no_self_verified_system_slot(&world.system_slots, locations) {
                return Some(MountRefusalReason::AcquisitionDeviceWithoutASelfVerifiedSlot);
            }
            None
        }
    }
}

/// 挂载开头故障菜单的一项（登记 5.3 第 5 条）：时机 T1 = 这次挂载里对那个落点的第一次读读不出、之后每一次读都读得出（D23 已定项 14「重读一次」的 R = 1）；
/// TP = 这次挂载里对那个落点的每一次读都读不出；两种都在下一次挂载开头恢复。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct MountFaultMenuEntry {
    menu_index: usize,
    /// 读阶段第一遍的读视图里读不出的落点（T1 与 TP 相同）。
    first_read_unreadable: FaultLocations,
    /// 重读那一遍、取号那一刻、写侧的读里仍读不出的落点：T1 是空，TP 同第一遍。
    every_read_unreadable: FaultLocations,
}

impl MountFaultMenuEntry {
    fn label(self) -> &'static str {
        MOUNT_FAULT_MENU_LABELS[self.menu_index]
    }

    /// 历史标签（登记第一节「承诺内」第 1 条）：RNSB 是 D22 已定项 8 写明不罩的双故障形，别的落点罩着。
    const fn is_inside_the_single_fault_promise(self) -> bool {
        self.menu_index < RNSB_FIRST_MENU_INDEX
    }
}

const RNSB_FIRST_MENU_INDEX: usize = 11;

/// 菜单里六种落点的名字次序：RN、SN0、SN1、SB、RI、RNSB（每种配 T1、TP 两项，前面是「无」）。
const MOUNT_FAULT_SITE_COUNT: usize = 6;

fn locations_of_a_mount_fault_site(world: &FaultWorld, site_number: usize) -> FaultLocations {
    let newest_root = world.newest_root_locations();
    let newest_system_slot_of_device_0 = world.newest_system_slot_locations(0);
    let newest_system_slot_of_device_1 = world.newest_system_slot_locations(1);
    match site_number {
        0 => newest_root,
        1 => newest_system_slot_of_device_0,
        2 => newest_system_slot_of_device_1,
        3 => newest_system_slot_of_device_0.union(newest_system_slot_of_device_1),
        4 => world.newest_instance_root_locations(),
        5 => newest_root
            .union(newest_system_slot_of_device_0)
            .union(newest_system_slot_of_device_1),
        unknown_site => panic!("故障落点只有 6 种，收到 {unknown_site}"),
    }
}

/// 挂载 `mount_number` 开头的故障菜单：挂载 1 没有故障（只有「无」一项），挂载 2 起 13 项（登记 5.3 第 5 条）。
fn mount_fault_menu(world: &FaultWorld, mount_number: usize) -> Vec<MountFaultMenuEntry> {
    let mut menu = vec![MountFaultMenuEntry {
        menu_index: 0,
        first_read_unreadable: FaultLocations::NOWHERE,
        every_read_unreadable: FaultLocations::NOWHERE,
    }];
    if mount_number < 2 {
        return menu;
    }
    for site_number in 0..MOUNT_FAULT_SITE_COUNT {
        let locations = locations_of_a_mount_fault_site(world, site_number);
        menu.push(MountFaultMenuEntry {
            menu_index: menu.len(),
            first_read_unreadable: locations,
            every_read_unreadable: FaultLocations::NOWHERE,
        });
        menu.push(MountFaultMenuEntry {
            menu_index: menu.len(),
            first_read_unreadable: locations,
            every_read_unreadable: locations,
        });
    }
    assert_eq!(menu.len(), MOUNT_FAULT_MENU_LENGTH, "故障菜单 13 项");
    menu
}

labelled_enumeration! {
    /// 被判的那一次恢复的读故障（登记 5.3 第 11 条）：无、根环里键最大的那个根槽读不出、最新实例的全部根读不出、某块盘不在。
    enum JudgedFault {
        Nothing => "none",
        NewestRoot => "RN",
        NewestInstanceRoots => "RI",
        DeviceZeroGone => "D0",
        DeviceOneGone => "D1",
    }
}

impl JudgedFault {
    /// 被判时标签（登记第一节「承诺内」第 2 条）：RI 一次让一个实例的全部根读不出，超出单故障，不罩；别的罩着。
    const fn is_inside_the_single_fault_promise(self) -> bool {
        match self {
            Self::Nothing | Self::NewestRoot | Self::DeviceZeroGone | Self::DeviceOneGone => true,
            Self::NewestInstanceRoots => false,
        }
    }

    fn locations(self, world: &FaultWorld) -> FaultLocations {
        match self {
            Self::Nothing => FaultLocations::NOWHERE,
            Self::NewestRoot => world.newest_root_locations(),
            Self::NewestInstanceRoots => world.newest_instance_root_locations(),
            Self::DeviceZeroGone => FaultLocations::NOWHERE.with_dead_device(0),
            Self::DeviceOneGone => FaultLocations::NOWHERE.with_dead_device(1),
        }
    }
}

// ── 一段带读故障的历史走到某一刻的盘（世界）

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum MountOutcome {
    /// 可写挂载读到的更新状态读不出（重读一次仍读不出），或取号那一刻有槽读不出：在取号之前拒，盘上不变，不取号。
    Refused,
    WentDownOnTheFirstRead,
    WentDownOnTheSecondRead,
}

#[derive(Clone, Copy, Debug)]
struct MountLogEntry {
    fault: MountFaultMenuEntry,
    outcome: MountOutcome,
    /// 往下走的挂载：写行那次发布的 txg；被拒的挂载 0。
    row_write_txg: u64,
}

/// 系统配置轮换那一段（登记 5.3 第 8 条）里两块盘各自写成什么样：没写、写了、撕裂（新旧都读不出）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum SystemWriteOutcome {
    Unwritten,
    Written,
    Torn,
}

/// 一次发布的崩溃点（登记 5.3 第 8 条，D13 已定项 4 按设备切段）：记录那一段 2k 份拷贝的任一子集落了、根没落（4^k 个）；
/// 根落了之后，两块盘的系统配置写各取三态之一（最后一次挂载并成「都没写」一个：被判的三步不读系统配置槽）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum FaultCrashPoint {
    RecordCopiesLanded {
        copy_mask: u32,
    },
    RootLanded {
        system_writes: [SystemWriteOutcome; FAULT_DEVICE_COUNT],
    },
}

fn fault_crash_points_of(record_count: usize, is_the_last_mount: bool) -> Vec<FaultCrashPoint> {
    let mut crash_points: Vec<FaultCrashPoint> = (0..(1u32 << (2 * record_count)))
        .map(|copy_mask| FaultCrashPoint::RecordCopiesLanded { copy_mask })
        .collect();
    if is_the_last_mount {
        crash_points.push(FaultCrashPoint::RootLanded {
            system_writes: [SystemWriteOutcome::Unwritten; FAULT_DEVICE_COUNT],
        });
        return crash_points;
    }
    let outcomes = [
        SystemWriteOutcome::Unwritten,
        SystemWriteOutcome::Written,
        SystemWriteOutcome::Torn,
    ];
    for first_device in outcomes {
        for second_device in outcomes {
            crash_points.push(FaultCrashPoint::RootLanded {
                system_writes: [first_device, second_device],
            });
        }
    }
    crash_points
}

#[derive(Clone, Copy, Debug)]
struct FaultCrashLabel {
    version: RootVersion,
    record_count: usize,
    point: FaultCrashPoint,
}

#[derive(Clone, Debug)]
struct FaultWorld {
    geometry: RootChoiceDomain,
    /// 根环三个区域的槽，区域号主序、区域内槽号升序；没写过的槽是 `None`（读成 512 个 0）。
    root_slots: Vec<Option<Arc<RootChoicePublishMaterial>>>,
    /// 盘上至少有一份的记录，按写的次序。
    landed_records: Vec<RootChoiceLandedRecord>,
    system_slots: SystemSlots,
    /// 崩溃点所在那次发布之前、fsync 已返回的用户发布，按写的次序。
    confirmed_user_publishes: Vec<RootVersion>,
    /// 版本的父链（登记 5.3 第 7 条）：每次挂载里每次发布的父是上一次发布，一次挂载第一次发布的父是这次挂载往下走所用的那一版。
    parents: Vec<(RootVersion, RootVersion)>,
    /// 开始过的每次发布（含崩在途中的）与 mkfs 的第 0 代根，按次序；第 0 个是第 0 代根。
    started_publishes: Vec<Arc<RootChoicePublishMaterial>>,
    mount_log: Vec<MountLogEntry>,
    next_instance: u32,
    refused_mounts: u64,
    reread_mounts: u64,
    final_crash: Option<FaultCrashLabel>,
    /// 闸与取号那一刻的判不出取法（登记 r5 修订 1）；mkfs 之后取甲，`--gate r2` 时换成 R2。
    witness_gate_rule: WitnessGateRule,
    /// 被拒的挂载按拒因分（下标是 [`MountRefusalReason`] 的次序；各栏之和等于 `refused_mounts`）。
    refusals_by_reason: [u64; MOUNT_REFUSAL_REASON_COUNT],
}

/// [`MountRefusalReason`] 的成员数。
const MOUNT_REFUSAL_REASON_COUNT: usize = MountRefusalReason::ALL.len();

impl FaultWorld {
    /// mkfs 之后、一次挂载都还没有：第 0 代根在三个区域的槽 0，别处全 0，journal 环空，每块盘两个系统配置槽都是 (世代号 1, tail 0, 实例 0)。
    fn right_after_mkfs(geometry: RootChoiceDomain) -> Self {
        let genesis_root = Arc::new(mkfs_root_material());
        let slots_per_region = geometry.slots_per_region();
        let mut root_slots = vec![
            None;
            usize::try_from(ROOT_CHOICE_REGION_COUNT * slots_per_region)
                .expect("根槽数不超过 24")
        ];
        for region in 0..ROOT_CHOICE_REGION_COUNT {
            root_slots[root_choice_slot_index_of(region, 0, slots_per_region)] =
                Some(Arc::clone(&genesis_root));
        }
        Self {
            geometry,
            root_slots,
            landed_records: Vec::new(),
            system_slots: system_slots_right_after_mkfs(),
            confirmed_user_publishes: Vec::new(),
            parents: Vec::new(),
            started_publishes: vec![genesis_root],
            mount_log: Vec::new(),
            next_instance: 1,
            refused_mounts: 0,
            reread_mounts: 0,
            final_crash: None,
            witness_gate_rule: WitnessGateRule::OneUnreadableOrTornSlotMeansUndecidable,
            refusals_by_reason: [0; MOUNT_REFUSAL_REASON_COUNT],
        }
    }

    /// mkfs 之后、闸取 `witness_gate_rule` 的那一份（登记 r5 修订 1：`--gate r2`）。
    fn right_after_mkfs_under_the_gate(
        geometry: RootChoiceDomain,
        witness_gate_rule: WitnessGateRule,
    ) -> Self {
        let mut world = Self::right_after_mkfs(geometry);
        world.witness_gate_rule = witness_gate_rule;
        world
    }

    fn slots_per_region(&self) -> u64 {
        self.geometry.slots_per_region()
    }

    /// 开始过几次发布（第 0 代根不算）：新根段按全历史的发布序号取。
    fn publishes_started(&self) -> u64 {
        u64::try_from(self.started_publishes.len() - 1).expect("发布次数不超过 u64")
    }

    fn parent_of(&self, version: RootVersion) -> Option<RootVersion> {
        self.parents
            .iter()
            .find(|(child, _parent)| *child == version)
            .map(|(_child, parent)| *parent)
    }

    /// `version` 是 `ancestor` 本身，或在它的后代上（沿父链往上走能走到它）。
    fn is_the_version_or_a_descendant_of(
        &self,
        version: RootVersion,
        ancestor: RootVersion,
    ) -> bool {
        let mut current = version;
        for _step in 0..=self.parents.len() {
            if current == ancestor {
                return true;
            }
            match self.parent_of(current) {
                Some(parent) => current = parent,
                None => return false,
            }
        }
        false
    }

    fn fields_of_a_version(&self, version: RootVersion) -> Option<VersionFields> {
        self.started_publishes
            .iter()
            .find(|material| material.version == version)
            .map(|material| material.fields)
    }

    /// 根环里键 (txg, 实例代号) 最大的那个根槽（几个槽同键取读根环次序里第一个）。
    fn newest_root_locations(&self) -> FaultLocations {
        let mut newest: Option<(usize, RootVersion)> = None;
        for (root_slot_index, root) in self.root_slots.iter().enumerate() {
            if let Some(material) = root {
                if newest.is_none_or(|(_index, version)| material.version > version) {
                    newest = Some((root_slot_index, material.version));
                }
            }
        }
        newest.map_or(FaultLocations::NOWHERE, |(root_slot_index, _version)| {
            FaultLocations::NOWHERE.with_root_slot(root_slot_index)
        })
    }

    /// 「RN 那条根的实例」的全部根所在的槽；那个实例是 0（mkfs 的第 0 代）时没有落点。
    fn newest_instance_root_locations(&self) -> FaultLocations {
        let newest_instance = self
            .root_slots
            .iter()
            .flatten()
            .map(|material| material.version)
            .max()
            .map_or(ROOT_CHOICE_MKFS_INSTANCE, |version| version.instance);
        let mut locations = FaultLocations::NOWHERE;
        if newest_instance == ROOT_CHOICE_MKFS_INSTANCE {
            return locations;
        }
        for (root_slot_index, root) in self.root_slots.iter().enumerate() {
            if root
                .as_ref()
                .is_some_and(|material| material.version.instance == newest_instance)
            {
                locations = locations.with_root_slot(root_slot_index);
            }
        }
        locations
    }

    /// 这块盘两槽里没撕裂、世代号较大的那一槽，世代号相同取槽号大的（两槽都撕了时没有落点）。
    fn newest_system_slot_locations(&self, device: usize) -> FaultLocations {
        let mut newest: Option<(u64, usize)> = None;
        for (slot, content) in self.system_slots[device].iter().enumerate() {
            if let SystemSlotContent::Verified { generation, .. } = content {
                if newest.is_none_or(|(largest, _slot)| *generation >= largest) {
                    newest = Some((*generation, slot));
                }
            }
        }
        newest.map_or(FaultLocations::NOWHERE, |(_generation, slot)| {
            FaultLocations::NOWHERE.with_system_slot(device, slot)
        })
    }
}

/// 这次发布最后一条记录的计数器：系统配置轮换写的 tail（登记 5.3 第 4 条：tail = c + k − 1）。
fn last_counter_of_a_publish(material: &RootChoicePublishMaterial) -> u64 {
    material
        .records
        .last()
        .expect("每次发布至少一条记录")
        .parsed
        .as_ref()
        .expect("合法历史的每条记录都解得出（生成时断言过）")
        .counter
}

impl FaultWorld {
    /// 这次发布开始了：记下它的父（登记 5.3 第 7 条）与它的材料。崩溃点与整次落盘都在这之后。
    fn register_a_started_publish(
        &mut self,
        material: &Arc<RootChoicePublishMaterial>,
        parent: RootVersion,
    ) {
        self.parents.push((material.version, parent));
        self.started_publishes.push(Arc::clone(material));
    }

    fn put_every_record_of_a_publish_on_both_devices(
        &mut self,
        material: &Arc<RootChoicePublishMaterial>,
    ) {
        for record_index in 0..material.records.len() {
            self.landed_records.push(RootChoiceWorld::landed_record_of(
                material,
                record_index,
                [true, true],
            ));
        }
    }

    fn put_the_root_of_a_publish_in_its_slot(&mut self, material: &Arc<RootChoicePublishMaterial>) {
        let slots_per_region = self.slots_per_region();
        let (region, slot) =
            root_choice_target_of_publish(material.version.checkpoint_txg, slots_per_region);
        self.root_slots[root_choice_slot_index_of(region, slot, slots_per_region)] =
            Some(Arc::clone(material));
    }

    /// 每块盘一次系统配置写（取号与每次发布末尾的轮换），写进它的落槽。`unreadable_all_mount_long` 是这次挂载里一直读不出的落点（TP 故障）。
    fn write_the_system_configuration_of_every_device(
        &mut self,
        tail: u64,
        instance: u32,
        unreadable_all_mount_long: FaultLocations,
    ) {
        for device in 0..FAULT_DEVICE_COUNT {
            let (slot, content) = system_slot_write_of(
                &self.system_slots,
                device,
                tail,
                instance,
                unreadable_all_mount_long,
            );
            self.system_slots[device][slot] = content;
        }
    }

    /// 这次发布整次落了：每条记录两盘各一份，根写进它的落点，两块盘各一次系统配置写（登记 5.3 第 4 条）。
    /// `is_confirmed`：它之后 fsync 返回了（只有用户发布、而且后面还有用户发布时才是）。
    fn apply_a_publish_in_full(
        &mut self,
        material: &Arc<RootChoicePublishMaterial>,
        is_confirmed: bool,
        unreadable_all_mount_long: FaultLocations,
    ) {
        self.put_every_record_of_a_publish_on_both_devices(material);
        self.put_the_root_of_a_publish_in_its_slot(material);
        self.write_the_system_configuration_of_every_device(
            last_counter_of_a_publish(material),
            material.version.instance,
            unreadable_all_mount_long,
        );
        if is_confirmed {
            self.confirmed_user_publishes.push(material.version);
        }
    }

    /// 这次发布崩在 `point` 上之后的盘。
    fn with_crash_applied(
        &self,
        material: &Arc<RootChoicePublishMaterial>,
        point: FaultCrashPoint,
        unreadable_all_mount_long: FaultLocations,
    ) -> Self {
        let mut world = self.clone();
        match point {
            FaultCrashPoint::RecordCopiesLanded { copy_mask } => {
                for record_index in 0..material.records.len() {
                    let copies_on_devices = copies_of_a_record_in_a_mask(copy_mask, record_index);
                    if copies_on_devices[0] || copies_on_devices[1] {
                        world.landed_records.push(RootChoiceWorld::landed_record_of(
                            material,
                            record_index,
                            copies_on_devices,
                        ));
                    }
                }
            }
            FaultCrashPoint::RootLanded { system_writes } => {
                world.put_every_record_of_a_publish_on_both_devices(material);
                world.put_the_root_of_a_publish_in_its_slot(material);
                for (device, system_write) in system_writes.iter().enumerate() {
                    let (slot, content) = system_slot_write_of(
                        &self.system_slots,
                        device,
                        last_counter_of_a_publish(material),
                        material.version.instance,
                        unreadable_all_mount_long,
                    );
                    match system_write {
                        SystemWriteOutcome::Unwritten => {}
                        SystemWriteOutcome::Written => world.system_slots[device][slot] = content,
                        SystemWriteOutcome::Torn => {
                            world.system_slots[device][slot] = SystemSlotContent::Torn;
                        }
                    }
                }
            }
        }
        world.final_crash = Some(FaultCrashLabel {
            version: material.version,
            record_count: material.records.len(),
            point,
        });
        world
    }
}

// ── 读视图：盘减去读不出的落点

/// 世界在 `locations` 这个读视图里的样子：读不出的根槽当没有根，盘不在时那块盘上的记录拷贝读不出（只有一份在那块盘上的记录整条读不出）。
fn fault_landing_of_a_view(world: &FaultWorld, locations: FaultLocations) -> RootChoiceLanding<'_> {
    let slots_per_region = world.slots_per_region();
    let roots_in_ring_order: Vec<Option<&RootChoicePublishMaterial>> = world
        .root_slots
        .iter()
        .enumerate()
        .map(|(root_slot_index, root)| {
            if locations.root_slot_is_unreadable(root_slot_index, slots_per_region) {
                None
            } else {
                root.as_deref()
            }
        })
        .collect();
    let mut records = Vec::with_capacity(world.landed_records.len());
    for landed in &world.landed_records {
        let copies_on_devices = [
            landed.copies_on_devices[0] && !locations.device_is_dead(0),
            landed.copies_on_devices[1] && !locations.device_is_dead(1),
        ];
        if copies_on_devices[0] || copies_on_devices[1] {
            records.push(RootChoiceLandedRecordRef {
                material: &landed.material,
                record_index: landed.record_index,
                copies_on_devices,
            });
        }
    }
    RootChoiceLanding {
        roots_in_ring_order,
        records,
    }
}

/// 条款的理想恢复在这个读视图上落到哪一版（登记 5.5 KJ3 的写法：读视图里版本最大的根起，同实例逐次往后）；读视图里一条根都没有时 `None`。
fn ideal_landed_version_of_a_view(
    view: &RootChoiceLanding<'_>,
    in_flight_limit: usize,
) -> Option<RootVersion> {
    root_choice_newest_root(view).map(|newest| {
        root_choice_ideal_recovery_from(newest, &view.records, in_flight_limit).landed_version
    })
}

/// 可写挂载读到的更新状态读不出那道闸（D23 已定项 14）：`landed` 是读阶段落到的那一版，`locations` 是这一遍读视图里读不出的落点。
/// 为真 = 判不出或见证到比所选那一版更新的发布。判不出的取法跟 `world.witness_gate_rule`：取甲（r4 登记修订 1）或 R2（r5 登记修订 1）。
/// 写法照 `mount.rs` 的 `newer_publish_witness` 那一段条款文字，不调它。
fn newer_publish_is_witnessed(
    world: &FaultWorld,
    locations: FaultLocations,
    landed: RootVersion,
) -> bool {
    let (readable_slot_count, largest_tail) =
        readable_system_slots_of(&world.system_slots, locations);
    match world.witness_gate_rule {
        WitnessGateRule::OneUnreadableOrTornSlotMeansUndecidable => {}
        WitnessGateRule::SomeSlotReportsAReadErrorAndNoDeviceSelfVerifiedBothSlots => {
            return newer_publish_is_witnessed_under_r2(world, locations, landed, largest_tail);
        }
    }
    if readable_slot_count < FAULT_SYSTEM_SLOT_COUNT_OF_THE_POOL {
        return true;
    }
    if largest_tail == 0 {
        return false;
    }
    newer_publish_is_witnessed_against_the_witnessed_tail(world, locations, landed, largest_tail)
}

/// 闸的 R2 支（登记 r5 修订 1 修1.五第 2 条；D22（单元原子性怎么合成） 已定项 8 射程）：判不出 ⟺ 这一遍读视图里有系统配置槽报读错，
/// 且没有一块盘两槽都读得出且没撕裂 ⇒ 真；c_见证（读得出且没撕裂的槽 tail 的最大值）= 0 ⇒ 假；其余与取甲同一段比法。
fn newer_publish_is_witnessed_under_r2(
    world: &FaultWorld,
    locations: FaultLocations,
    landed: RootVersion,
    largest_tail: u64,
) -> bool {
    if system_configuration_witness_is_undecidable_under_r2(&world.system_slots, locations) {
        return true;
    }
    let nothing_is_witnessed = largest_tail == 0;
    if nothing_is_witnessed {
        return false;
    }
    newer_publish_is_witnessed_against_the_witnessed_tail(world, locations, landed, largest_tail)
}

/// 闸判得出、c_见证 > 0 之后的比法（`mount.rs` 的 `newer_publish_witness` 那三支的条款文字）：所选那一版末条读得出 ⇒ c_见证 > 它的计数器；
/// 否则计数器等于 c_见证 的记录读得出 ⇒ 键最大那条的 (实例代号, txg) > 所选那一版；都读不出 ⇒ 真。
fn newer_publish_is_witnessed_against_the_witnessed_tail(
    world: &FaultWorld,
    locations: FaultLocations,
    landed: RootVersion,
    largest_tail: u64,
) -> bool {
    let view = fault_landing_of_a_view(world, locations);
    let last_record_counter_of_the_landed_publish = view
        .records
        .iter()
        .filter(|record| {
            record.material.version == landed
                && record.record_index + 1 == record.material.records.len()
        })
        .map(counter_of_the_landed_record)
        .max();
    if let Some(counter) = last_record_counter_of_the_landed_publish {
        return largest_tail > counter;
    }
    let newest_of_the_record_at_the_tail = view
        .records
        .iter()
        .filter(|record| counter_of_the_landed_record(record) == largest_tail)
        .map(|record| {
            (
                record.material.version.instance,
                record.material.version.checkpoint_txg,
            )
        })
        .max();
    match newest_of_the_record_at_the_tail {
        Some(instance_and_txg) => instance_and_txg > (landed.instance, landed.checkpoint_txg),
        None => true,
    }
}

// ── 被判的三步看到的盘：真相的内存盘加上读故障

/// 三步调用的读者：真相的内存盘（根槽、记录槽摆在各自的落点上），再减去读不出的落点：根槽交读错，盘不在时那块盘上的读全交读错。
struct FaultedRootChoiceDisk<'disk, 'landing> {
    inner: &'disk RootChoiceDisk<'landing>,
    locations: FaultLocations,
    slots_per_region: u64,
}

impl PoolReader for FaultedRootChoiceDisk<'_, '_> {
    fn device_identities(&self) -> Vec<DeviceIdentity> {
        self.inner.device_identities()
    }

    fn device_size_in_bytes(&self, device: DeviceIdentity) -> Option<u64> {
        self.inner.device_size_in_bytes(device)
    }

    fn read(
        &self,
        device: DeviceIdentity,
        offset: DeviceOffsetInBytes,
        length: usize,
    ) -> Option<Vec<u8>> {
        if let Ok(device_index) = usize::try_from(device.0) {
            if device_index < FAULT_DEVICE_COUNT && self.locations.device_is_dead(device_index) {
                return None;
            }
        }
        if let Some(root_slot_index) =
            root_choice_root_slot_index_at(device, offset, self.slots_per_region)
        {
            if self
                .locations
                .root_slot_is_unreadable(root_slot_index, self.slots_per_region)
            {
                return None;
            }
        }
        self.inner.read(device, offset, length)
    }

    fn journal_record_offsets_hint(
        &self,
        _device: DeviceIdentity,
        _ring_start: DeviceOffsetInBytes,
        _ring_bytes: u64,
    ) -> Option<Vec<DeviceOffsetInBytes>> {
        None
    }
}

// ── 枚举一个 (域, M, U) 格里的全部历史（登记 5.3 第 6 至 10 条）

/// 走枚举的一方：收工作单元的起点，或判最后一次挂载崩溃点上的每个状态。
trait FaultVisitor {
    /// 走到「第 `mount_number` 次挂载开始之前」（只在 `stop_before_mount` 点名的那一次挂载上触发，之后不再往下走）。
    fn reach_the_start_of_a_mount(
        &mut self,
        world: &FaultWorld,
        mount_number: usize,
        user_publishes_left: usize,
    );

    /// 最后一次挂载走完的一个状态：崩溃点已落在 `world` 上；最后一次挂载被拒时是被拒之后的盘（没有崩溃点）。
    fn visit_a_final_state(&mut self, world: &FaultWorld);
}

struct FaultEnumeration {
    domain: FaultDomain,
    mounts: usize,
    stop_before_mount: Option<usize>,
}

/// 一次往下走的挂载的几样不变的设定。
#[derive(Clone, Copy, Debug)]
struct MountSetting {
    mount_number: usize,
    user_publishes_after_this_mount: usize,
    /// 这次挂载里一直读不出的落点（TP 故障）：系统配置写算世代号时不看它们。
    unreadable_all_mount_long: FaultLocations,
    is_the_last_mount: bool,
}

impl FaultEnumeration {
    fn in_flight_limit(&self) -> usize {
        self.domain.geometry().ring().local_in_flight_limit()
    }

    /// 第 `mount_number` 次挂载：挂载 2 起开头先取一项故障（13 项），读阶段、闸、取号、写行与暖机的空发布、用户发布 u_m 次，崩溃点在第 u_m 次用户发布里；
    /// u_m = 0 时在写行或任一次暖机发布里。非最后一次挂载 u_m 取 0..=剩余的用户发布数，最后一次用完剩余的。`only_menu_index` 只走故障菜单的那一项。
    fn enumerate_a_mount(
        &self,
        world: &FaultWorld,
        mount_number: usize,
        user_publishes_left: usize,
        only_menu_index: Option<usize>,
        visitor: &mut dyn FaultVisitor,
    ) {
        if self.stop_before_mount == Some(mount_number) {
            visitor.reach_the_start_of_a_mount(world, mount_number, user_publishes_left);
            return;
        }
        for entry in mount_fault_menu(world, mount_number) {
            if only_menu_index.is_some_and(|only| only != entry.menu_index) {
                continue;
            }
            self.enumerate_a_mount_under_a_fault(
                world,
                mount_number,
                user_publishes_left,
                entry,
                visitor,
            );
        }
    }

    /// 读阶段：第一遍读视图 = 盘减去故障落点，在它上面做条款的理想恢复；闸为真就用第二遍读视图（盘减去 TP 的落点）重做，仍为真拒；
    /// 往下走之前再判取号那一刻（取甲：两块盘的四个系统配置槽在 TP 视图里有一槽读不出或撕裂就拒；R2：判不出、或某块盘一份自证过的都没有就拒，
    /// 登记 r5 修订 1）。KG 关掉闸与取号那一刻（登记 5.3 第 6 条）。
    fn enumerate_a_mount_under_a_fault(
        &self,
        world: &FaultWorld,
        mount_number: usize,
        user_publishes_left: usize,
        entry: MountFaultMenuEntry,
        visitor: &mut dyn FaultVisitor,
    ) {
        let limit = self.in_flight_limit();
        let first_view = fault_landing_of_a_view(world, entry.first_read_unreadable);
        let mut landed = ideal_landed_version_of_a_view(&first_view, limit)
            .expect("停机条款 TH9：读阶段第一遍的读视图里一条根都没有");
        let mut settled_unreadable = entry.first_read_unreadable;
        let mut outcome = MountOutcome::WentDownOnTheFirstRead;
        let mut refusal: Option<MountRefusalReason> = None;
        if self.domain.gate_is_on() {
            if newer_publish_is_witnessed(world, entry.first_read_unreadable, landed) {
                let second_view = fault_landing_of_a_view(world, entry.every_read_unreadable);
                let second_landed = ideal_landed_version_of_a_view(&second_view, limit)
                    .expect("停机条款 TH9：读阶段第二遍的读视图里一条根都没有");
                if newer_publish_is_witnessed(world, entry.every_read_unreadable, second_landed) {
                    refusal = Some(MountRefusalReason::TheGateAfterTheOneReread);
                } else {
                    settled_unreadable = entry.every_read_unreadable;
                    landed = second_landed;
                    outcome = MountOutcome::WentDownOnTheSecondRead;
                }
            }
            if refusal.is_none() {
                refusal = acquisition_refusal_of(world, entry.every_read_unreadable);
            }
        }
        if let Some(reason) = refusal {
            let mut refused_world = world.clone();
            refused_world.refused_mounts += 1;
            refused_world.refusals_by_reason[reason.index()] += 1;
            refused_world.mount_log.push(MountLogEntry {
                fault: entry,
                outcome: MountOutcome::Refused,
                row_write_txg: 0,
            });
            if mount_number == self.mounts {
                visitor.visit_a_final_state(&refused_world);
            } else {
                self.enumerate_a_mount(
                    &refused_world,
                    mount_number + 1,
                    user_publishes_left,
                    None,
                    visitor,
                );
            }
            return;
        }
        self.go_down_and_write(
            world,
            (mount_number, user_publishes_left),
            (entry, outcome, settled_unreadable, landed),
            visitor,
        );
    }
}

impl FaultEnumeration {
    /// 往下走：实例代号 = 之前取过号的挂载数 + 1；写行 txg = 往下走那一遍读得出的根与（闸开时）记录的最大 txg + 1；第一条记录的计数器 = 那一遍读得出的最大计数器 + 1；
    /// 取号那一写每块盘一次系统配置写，tail = 取号那一刻读得出的槽 tail 的最大值（不取崩溃点）；之后写行、暖机、用户发布（登记 5.3 第 6 条）。
    fn go_down_and_write(
        &self,
        world: &FaultWorld,
        (mount_number, user_publishes_left): (usize, usize),
        (entry, outcome, settled_unreadable, landed): (
            MountFaultMenuEntry,
            MountOutcome,
            FaultLocations,
            RootVersion,
        ),
        visitor: &mut dyn FaultVisitor,
    ) {
        let settled_view = fault_landing_of_a_view(world, settled_unreadable);
        let highest_root_txg = settled_view
            .roots_in_ring_order
            .iter()
            .flatten()
            .map(|material| material.version.checkpoint_txg)
            .max()
            .expect("停机条款 TH9：往下走那一遍的读视图里一条根都没有");
        let highest_record_txg = settled_view
            .records
            .iter()
            .map(|record| record.material.version.checkpoint_txg)
            .max()
            .unwrap_or(0);
        let row_write_txg = if self.domain.gate_is_on() {
            highest_root_txg.max(highest_record_txg) + 1
        } else {
            highest_root_txg + 1
        };
        let first_counter = settled_view
            .records
            .iter()
            .map(counter_of_the_landed_record)
            .max()
            .unwrap_or(0)
            + 1;
        let instance = world.next_instance;
        let mut acquired = world.clone();
        let (_readable_slot_count, largest_tail_at_the_acquisition) =
            readable_system_slots_of(&world.system_slots, entry.every_read_unreadable);
        acquired.write_the_system_configuration_of_every_device(
            largest_tail_at_the_acquisition,
            instance,
            entry.every_read_unreadable,
        );
        acquired.next_instance += 1;
        if outcome == MountOutcome::WentDownOnTheSecondRead {
            acquired.reread_mounts += 1;
        }
        acquired.mount_log.push(MountLogEntry {
            fault: entry,
            outcome,
            row_write_txg,
        });
        let overhead_txgs = root_choice_warm_up_plan(row_write_txg);
        let is_the_last_mount = mount_number == self.mounts;
        let user_publish_counts: Vec<usize> = if is_the_last_mount {
            vec![user_publishes_left]
        } else {
            (0..=user_publishes_left).collect()
        };
        for user_publish_count in user_publish_counts {
            let setting = MountSetting {
                mount_number,
                user_publishes_after_this_mount: user_publishes_left - user_publish_count,
                unreadable_all_mount_long: entry.every_read_unreadable,
                is_the_last_mount,
            };
            let mut running = acquired.clone();
            let mut cursor = RootChoiceMountCursor {
                instance,
                next_counter: first_counter,
                next_transaction: 1,
                previous_chain_value: None,
                next_checkpoint_txg: overhead_txgs[0],
            };
            let mut previous_version = landed;
            for overhead_txg in &overhead_txgs {
                assert_eq!(
                    cursor.next_checkpoint_txg, *overhead_txg,
                    "开销发布的 txg 逐个加一"
                );
                let material = Arc::new(root_choice_publish_material(
                    &cursor.plan_of_the_next_publish(
                        self.domain.geometry(),
                        running.publishes_started(),
                        1,
                        0,
                        false,
                    ),
                ));
                running.register_a_started_publish(&material, previous_version);
                if user_publish_count == 0 {
                    self.enumerate_the_crash_points(&running, &material, setting, visitor);
                }
                running.apply_a_publish_in_full(
                    &material,
                    false,
                    setting.unreadable_all_mount_long,
                );
                cursor = cursor.after_the_publish_of(&material);
                previous_version = material.version;
            }
            if user_publish_count > 0 {
                self.enumerate_user_publishes(
                    &running,
                    cursor,
                    (previous_version, user_publish_count),
                    setting,
                    visitor,
                );
            }
        }
    }

    /// 第 `user_publishes_in_the_mount`（≥ 1）次用户发布起，每次的 k 与提交标记摆法各一种；崩溃点在这一挂载的最后一次用户发布里。
    fn enumerate_user_publishes(
        &self,
        running: &FaultWorld,
        cursor: RootChoiceMountCursor,
        (previous_version, user_publishes_in_the_mount): (RootVersion, usize),
        setting: MountSetting,
        visitor: &mut dyn FaultVisitor,
    ) {
        for record_count in self.domain.user_publish_record_counts() {
            for commit_marker_bits in 0..(1u32 << (record_count - 1)) {
                let material = Arc::new(root_choice_publish_material(
                    &cursor.plan_of_the_next_publish(
                        self.domain.geometry(),
                        running.publishes_started(),
                        *record_count,
                        commit_marker_bits,
                        true,
                    ),
                ));
                let mut branch = running.clone();
                branch.register_a_started_publish(&material, previous_version);
                if user_publishes_in_the_mount == 1 {
                    self.enumerate_the_crash_points(&branch, &material, setting, visitor);
                } else {
                    branch.apply_a_publish_in_full(
                        &material,
                        true,
                        setting.unreadable_all_mount_long,
                    );
                    self.enumerate_user_publishes(
                        &branch,
                        cursor.after_the_publish_of(&material),
                        (material.version, user_publishes_in_the_mount - 1),
                        setting,
                        visitor,
                    );
                }
            }
        }
    }

    /// 崩在 `material` 这次发布的每个崩溃点上：最后一次挂载交给 `visitor` 判；之前的挂载接着枚举下一次挂载。
    fn enumerate_the_crash_points(
        &self,
        running: &FaultWorld,
        material: &Arc<RootChoicePublishMaterial>,
        setting: MountSetting,
        visitor: &mut dyn FaultVisitor,
    ) {
        for point in fault_crash_points_of(material.records.len(), setting.is_the_last_mount) {
            let crashed =
                running.with_crash_applied(material, point, setting.unreadable_all_mount_long);
            if setting.is_the_last_mount {
                visitor.visit_a_final_state(&crashed);
            } else {
                self.enumerate_a_mount(
                    &crashed,
                    setting.mount_number + 1,
                    setting.user_publishes_after_this_mount,
                    None,
                    visitor,
                );
            }
        }
    }
}

// ── 判据与计数（登记 5.5）

labelled_enumeration! {
    /// 判据表（登记 5.5）；阳性对照那段循环遍历它。KJ1T 是这一族多出来的一条（登记第一节）。
    enum FaultCriterion {
        Kj0Defined => "KJ0",
        Kj1NotEarlierThanTheLastConfirmedPublish => "KJ1",
        Kj1tDescendsFromTheLastConfirmedPublish => "KJ1T",
        Kj2NotEarlierThanAnyReadableRoot => "KJ2",
        Kj3LandsOnTheIdealRecovery => "KJ3",
    }
}

const FAULT_CRITERION_COUNT: usize = 5;
const KJ1_INDEX: usize = 1;
const KJ1T_INDEX: usize = 2;
const KJ2_INDEX: usize = 3;
const KJ3_INDEX: usize = 4;
/// 本地择根序的个数：登记 9.1 的五种加平局取后者赢（v1822），再加 KF1 用的「取键最小的根」。
const FAULT_LOCAL_RULE_COUNT: usize = 7;
const FAULT_OLDEST_RULE_INDEX: usize = 6;
/// 每个 (域, M, U) 格里按「被判时故障 × 历史标签」分的格数。
const FAULT_CELL_COUNT: usize = 10;

/// 一个 (被判时故障, 历史标签) 格里数的量：状态数、有已确认的态数、上限截掉的态数，与每条判据的判数、违例数（上限截掉的态里的违例另记）。
#[derive(Clone, Copy, Debug, Default)]
struct FaultCellTally {
    states: u64,
    confirmed_states: u64,
    cut_states: u64,
    judged: [u64; FAULT_CRITERION_COUNT],
    violations: [u64; FAULT_CRITERION_COUNT],
    cut_violations: [u64; FAULT_CRITERION_COUNT],
    /// 条款的理想恢复在这一格里 KJ1、KJ1T 各违例几次（TQ3 把实现的数与它并列）。
    ideal_violations: [u64; 2],
}

impl FaultCellTally {
    fn merge(&mut self, later: &Self) {
        self.states += later.states;
        self.confirmed_states += later.confirmed_states;
        self.cut_states += later.cut_states;
        for criterion in 0..FAULT_CRITERION_COUNT {
            self.judged[criterion] += later.judged[criterion];
            self.violations[criterion] += later.violations[criterion];
            self.cut_violations[criterion] += later.cut_violations[criterion];
        }
        for criterion in 0..2 {
            self.ideal_violations[criterion] += later.ideal_violations[criterion];
        }
    }
}

/// 格的下标：被判时故障的次序 × 2 + 历史标签（0 罩着、1 不罩）。
fn fault_cell_index_of(judged_fault: JudgedFault, history_is_covered: bool) -> usize {
    judged_fault.index() * 2 + usize::from(!history_is_covered)
}

/// M = 2 的格按挂载 2 开头那项故障报的三样。
#[derive(Clone, Copy, Debug, Default)]
struct MountTwoFaultCount {
    histories: u64,
    refused: u64,
    reread: u64,
}

/// 一个 (域, M, U) 格的全部计数。违例类的量在第一次加一时记一个见证。
#[derive(Clone, Debug)]
struct FaultTally {
    cells: [FaultCellTally; FAULT_CELL_COUNT],
    /// 每个格每条判据的第一个见证。
    witnesses: Vec<Option<String>>,
    /// 承诺内（历史罩着 ∧ 被判时罩着 ∧ 不是上限截掉）的态数与 KJ1、KJ1T 的违例数，装置直接数（登记 5.5 末段）。
    promised_states: u64,
    promised_violations: [u64; 2],
    promised_witnesses: [Option<String>; 2],
    histories: u64,
    histories_with_an_uncovered_fault: u64,
    judged_states: u64,
    confirmed_states: u64,
    refused_mounts: u64,
    /// 被拒的挂载按拒因分（登记 r5 修订 1 修1.五第 4 条；只在 `--gate r2` 时打进产物）。
    refusals_by_reason: [u64; MOUNT_REFUSAL_REASON_COUNT],
    reread_mounts: u64,
    states_with_a_torn_system_slot: u64,
    states_with_same_txg_different_instance_roots: u64,
    states_with_higher_instance_lower_txg_roots: u64,
    states_without_a_readable_root: u64,
    above_newest_root_maximum: usize,
    confirmed_publishes_maximum: usize,
    largest_counter: u64,
    ideal_loss_by_limit: u64,
    ideal_loss_covered_other: u64,
    ideal_loss_uncovered: u64,
    mount_two: [MountTwoFaultCount; MOUNT_FAULT_MENU_LENGTH],
    panicked_states: u64,
    chose_nothing_with_a_readable_root: u64,
    chose_nothing_without_a_readable_root: u64,
    replay_failed_states: u64,
    replay_applied_states: u64,
    slot_readings_checked: u64,
    slot_readings_differing_from_the_truth: u64,
    slot_reading_witness: Option<String>,
    /// KJ0 的对照：在指定的一态上 panic，别的态不算。
    kj0_control_violations: u64,
    /// 本地择根序加理想重放读到的 KJ1、KJ1T、KJ2 违例数（登记第七节「本地择根序」那张表；下标 6 是 KF1 的「取键最小的根」）。
    local_rules: [[u64; 3]; FAULT_LOCAL_RULE_COUNT],
    /// 本地择根序的 KJ1T 违例数按被判时故障分（C331 的判别力自证：今天的序在被判时无故障的态上的合计）。
    local_kj1t_by_judged_fault: [[u64; 5]; FAULT_LOCAL_RULE_COUNT],
    /// KF2 的 KJ1、KJ1T、KJ2、KJ3 读数；KF4 的 KJ1、KJ1T 读数（登记 5.6）。
    kf2: [u64; 4],
    kf4: [u64; 2],
    kf4_states_without_a_root_off_the_confirmed_chain: u64,
    /// KJ1 与 KJ1T 对同一态判得不同的态数（TQ15）。
    kj1_and_kj1t_differing_states: u64,
    kj1_and_kj1t_differing_witness: Option<String>,
}

impl FaultTally {
    fn new() -> Self {
        Self {
            cells: [FaultCellTally::default(); FAULT_CELL_COUNT],
            witnesses: vec![None; FAULT_CELL_COUNT * FAULT_CRITERION_COUNT],
            promised_states: 0,
            promised_violations: [0; 2],
            promised_witnesses: [None, None],
            histories: 0,
            histories_with_an_uncovered_fault: 0,
            judged_states: 0,
            confirmed_states: 0,
            refused_mounts: 0,
            refusals_by_reason: [0; MOUNT_REFUSAL_REASON_COUNT],
            reread_mounts: 0,
            states_with_a_torn_system_slot: 0,
            states_with_same_txg_different_instance_roots: 0,
            states_with_higher_instance_lower_txg_roots: 0,
            states_without_a_readable_root: 0,
            above_newest_root_maximum: 0,
            confirmed_publishes_maximum: 0,
            largest_counter: 0,
            ideal_loss_by_limit: 0,
            ideal_loss_covered_other: 0,
            ideal_loss_uncovered: 0,
            mount_two: [MountTwoFaultCount::default(); MOUNT_FAULT_MENU_LENGTH],
            panicked_states: 0,
            chose_nothing_with_a_readable_root: 0,
            chose_nothing_without_a_readable_root: 0,
            replay_failed_states: 0,
            replay_applied_states: 0,
            slot_readings_checked: 0,
            slot_readings_differing_from_the_truth: 0,
            slot_reading_witness: None,
            kj0_control_violations: 0,
            local_rules: [[0; 3]; FAULT_LOCAL_RULE_COUNT],
            local_kj1t_by_judged_fault: [[0; 5]; FAULT_LOCAL_RULE_COUNT],
            kf2: [0; 4],
            kf4: [0; 2],
            kf4_states_without_a_root_off_the_confirmed_chain: 0,
            kj1_and_kj1t_differing_states: 0,
            kj1_and_kj1t_differing_witness: None,
        }
    }
}

fn merge_the_first_witness(earlier: &mut Option<String>, later: &Option<String>) {
    if earlier.is_none() {
        earlier.clone_from(later);
    }
}

impl FaultTally {
    /// 违例加一，并在这个格这条判据的第一条违例上记一个见证（枚举次序里最先的那个，合并时按工作单元的次序保留最先的）。
    fn add_cell_violation(
        &mut self,
        cell_index: usize,
        criterion_index: usize,
        describe_witness: &dyn Fn() -> String,
    ) {
        self.cells[cell_index].violations[criterion_index] += 1;
        let witness = &mut self.witnesses[cell_index * FAULT_CRITERION_COUNT + criterion_index];
        if witness.is_none() {
            *witness = Some(describe_witness());
        }
    }

    fn witness_of_a_cell(&self, cell_index: usize, criterion_index: usize) -> &str {
        self.witnesses[cell_index * FAULT_CRITERION_COUNT + criterion_index]
            .as_deref()
            .unwrap_or("none")
    }

    /// 一条判据在全部格上的违例合计。
    fn violations_of_a_criterion(&self, criterion_index: usize) -> u64 {
        self.cells
            .iter()
            .map(|cell| cell.violations[criterion_index])
            .sum()
    }

    fn judged_of_a_criterion(&self, criterion_index: usize) -> u64 {
        self.cells
            .iter()
            .map(|cell| cell.judged[criterion_index])
            .sum()
    }

    fn merge(&mut self, later: &Self) {
        for (cell, later_cell) in self.cells.iter_mut().zip(&later.cells) {
            cell.merge(later_cell);
        }
        for (witness, later_witness) in self.witnesses.iter_mut().zip(&later.witnesses) {
            merge_the_first_witness(witness, later_witness);
        }
        self.promised_states += later.promised_states;
        for criterion in 0..2 {
            self.promised_violations[criterion] += later.promised_violations[criterion];
            merge_the_first_witness(
                &mut self.promised_witnesses[criterion],
                &later.promised_witnesses[criterion],
            );
        }
        self.histories += later.histories;
        self.histories_with_an_uncovered_fault += later.histories_with_an_uncovered_fault;
        self.judged_states += later.judged_states;
        self.confirmed_states += later.confirmed_states;
        self.refused_mounts += later.refused_mounts;
        for (count, later_count) in self
            .refusals_by_reason
            .iter_mut()
            .zip(&later.refusals_by_reason)
        {
            *count += later_count;
        }
        self.reread_mounts += later.reread_mounts;
        self.states_with_a_torn_system_slot += later.states_with_a_torn_system_slot;
        self.states_with_same_txg_different_instance_roots +=
            later.states_with_same_txg_different_instance_roots;
        self.states_with_higher_instance_lower_txg_roots +=
            later.states_with_higher_instance_lower_txg_roots;
        self.states_without_a_readable_root += later.states_without_a_readable_root;
        self.above_newest_root_maximum = self
            .above_newest_root_maximum
            .max(later.above_newest_root_maximum);
        self.confirmed_publishes_maximum = self
            .confirmed_publishes_maximum
            .max(later.confirmed_publishes_maximum);
        self.largest_counter = self.largest_counter.max(later.largest_counter);
        self.ideal_loss_by_limit += later.ideal_loss_by_limit;
        self.ideal_loss_covered_other += later.ideal_loss_covered_other;
        self.ideal_loss_uncovered += later.ideal_loss_uncovered;
        for (count, later_count) in self.mount_two.iter_mut().zip(&later.mount_two) {
            count.histories += later_count.histories;
            count.refused += later_count.refused;
            count.reread += later_count.reread;
        }
        self.panicked_states += later.panicked_states;
        self.chose_nothing_with_a_readable_root += later.chose_nothing_with_a_readable_root;
        self.chose_nothing_without_a_readable_root += later.chose_nothing_without_a_readable_root;
        self.replay_failed_states += later.replay_failed_states;
        self.replay_applied_states += later.replay_applied_states;
        self.slot_readings_checked += later.slot_readings_checked;
        self.slot_readings_differing_from_the_truth += later.slot_readings_differing_from_the_truth;
        merge_the_first_witness(&mut self.slot_reading_witness, &later.slot_reading_witness);
        self.kj0_control_violations += later.kj0_control_violations;
        for rule in 0..FAULT_LOCAL_RULE_COUNT {
            for criterion in 0..3 {
                self.local_rules[rule][criterion] += later.local_rules[rule][criterion];
            }
            for fault in 0..5 {
                self.local_kj1t_by_judged_fault[rule][fault] +=
                    later.local_kj1t_by_judged_fault[rule][fault];
            }
        }
        for criterion in 0..4 {
            self.kf2[criterion] += later.kf2[criterion];
        }
        for criterion in 0..2 {
            self.kf4[criterion] += later.kf4[criterion];
        }
        self.kf4_states_without_a_root_off_the_confirmed_chain +=
            later.kf4_states_without_a_root_off_the_confirmed_chain;
        self.kj1_and_kj1t_differing_states += later.kj1_and_kj1t_differing_states;
        merge_the_first_witness(
            &mut self.kj1_and_kj1t_differing_witness,
            &later.kj1_and_kj1t_differing_witness,
        );
    }
}

/// 把工作单元分给几条线程跑，再按工作单元的次序合并（同 [`root_choice_tally_of_work_units`]，计数的种类不同）。
fn fault_tally_of_work_units<Unit: Sync>(
    units: &[Unit],
    thread_count: usize,
    work: &(dyn Fn(usize, &Unit) -> FaultTally + Sync),
) -> FaultTally {
    let next_unit_index = AtomicUsize::new(0);
    let finished_units: Mutex<Vec<(usize, FaultTally)>> = Mutex::new(Vec::new());
    std::thread::scope(|scope| {
        for _worker in 0..thread_count.clamp(1, units.len().max(1)) {
            scope.spawn(|| loop {
                let unit_index = next_unit_index.fetch_add(1, Ordering::Relaxed);
                let Some(unit) = units.get(unit_index) else {
                    break;
                };
                let tally = work(unit_index, unit);
                finished_units
                    .lock()
                    .expect("别的线程没有在拿着这把锁的时候 panic")
                    .push((unit_index, tally));
            });
        }
    });
    let mut finished_units = finished_units
        .into_inner()
        .expect("线程都已经收回，没有谁在拿着这把锁的时候 panic");
    finished_units.sort_by_key(|(unit_index, _tally)| *unit_index);
    assert_eq!(finished_units.len(), units.len(), "每个工作单元恰好跑一遍");
    let mut total = FaultTally::new();
    for (_unit_index, tally) in &finished_units {
        total.merge(tally);
    }
    total
}

labelled_enumeration! {
    /// 本地写的择根序（登记第七节「本地择根序」那张表、9.1 六行替换文的比较式各写一份；落点 = 择到的根加理想重放，不调实现）。
    /// 次序与那张表的列一致：今天的序，v1823 至 v1826，v1822；最后一个是 KF1 用的「取键最小的根」。
    enum FaultLocalRule {
        Today => "today",
        IgnoresTheInstance => "v1823",
        FirstSelfVerifiedWins => "v1824",
        TieTakesTheLowerInstance => "v1825",
        InstanceFirstThenTxg => "v1826",
        TieTakesTheLater => "v1822",
        OldestWins => "oldest",
    }
}

impl FaultLocalRule {
    /// 读根环次序里往后走，`candidate` 换掉已择的 `chosen` 吗。
    fn replaces(self, candidate: RootVersion, chosen: RootVersion) -> bool {
        match self {
            Self::Today => {
                candidate.checkpoint_txg > chosen.checkpoint_txg
                    || (candidate.checkpoint_txg == chosen.checkpoint_txg
                        && candidate.instance > chosen.instance)
            }
            Self::IgnoresTheInstance => candidate.checkpoint_txg > chosen.checkpoint_txg,
            Self::FirstSelfVerifiedWins => false,
            Self::TieTakesTheLowerInstance => {
                candidate.checkpoint_txg > chosen.checkpoint_txg
                    || (candidate.checkpoint_txg == chosen.checkpoint_txg
                        && candidate.instance < chosen.instance)
            }
            Self::InstanceFirstThenTxg => {
                candidate.instance > chosen.instance
                    || (candidate.instance == chosen.instance
                        && candidate.checkpoint_txg > chosen.checkpoint_txg)
            }
            Self::TieTakesTheLater => {
                candidate.checkpoint_txg > chosen.checkpoint_txg
                    || (candidate.checkpoint_txg == chosen.checkpoint_txg
                        && candidate.instance >= chosen.instance)
            }
            Self::OldestWins => candidate < chosen,
        }
    }

    fn chooses_among<'world>(
        self,
        roots_in_ring_order: &[Option<&'world RootChoicePublishMaterial>],
    ) -> Option<&'world RootChoicePublishMaterial> {
        let mut chosen: Option<&RootChoicePublishMaterial> = None;
        for candidate in roots_in_ring_order.iter().copied().flatten() {
            if chosen.is_none_or(|already_chosen| {
                self.replaces(candidate.version, already_chosen.version)
            }) {
                chosen = Some(candidate);
            }
        }
        chosen
    }
}

/// 一个被判态判之前先算好的真相（判据一侧只用这些，不调 core 的判定函数）。
struct FaultStateFacts {
    last_confirmed: Option<RootVersion>,
    newest_readable: Option<RootVersion>,
    /// 条款的理想恢复（读视图里版本最大的根起）落到的六个字段。
    ideal_fields: Option<VersionFields>,
}

/// 一条落点按各条判据读出来的结果；`Some(true)` 是违例，`Some(false)` 是判过没违例，`None` 是这一态不判这一条。
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
struct FaultCriteriaReading {
    kj1: Option<bool>,
    kj1t: Option<bool>,
    kj2: Option<bool>,
    kj3: Option<bool>,
}

/// 落点 `landed_version` 按登记 5.5 的四条判据读：KJ1 版本序不早于最后一次已确认的发布；KJ1T 落在它本身或后代上（父链，登记 5.3 第 7 条）；
/// KJ2 不早于读视图里任一读得出的根；KJ3 六个字段等于条款的理想恢复。
fn fault_criteria_of_a_landing(
    world: &FaultWorld,
    facts: &FaultStateFacts,
    landed_version: RootVersion,
    landed_fields: Option<VersionFields>,
) -> FaultCriteriaReading {
    FaultCriteriaReading {
        kj1: facts.last_confirmed.map(|last| landed_version < last),
        kj1t: facts
            .last_confirmed
            .map(|last| !world.is_the_version_or_a_descendant_of(landed_version, last)),
        kj2: facts.newest_readable.map(|newest| landed_version < newest),
        kj3: match (facts.ideal_fields, landed_fields) {
            (Some(ideal), Some(fields)) => Some(fields != ideal),
            (None, _) | (_, None) => None,
        },
    }
}

/// 一个被判态的结局在每条判据上的读数：真实现的结局或阳性对照的假结局都走这一段。
fn count_a_criteria_reading_into_a_cell(
    reading: FaultCriteriaReading,
    is_cut_by_the_limit: bool,
    cell: &mut FaultCellTally,
) -> [bool; FAULT_CRITERION_COUNT] {
    let mut violated = [false; FAULT_CRITERION_COUNT];
    for (criterion_index, verdict) in [
        (KJ1_INDEX, reading.kj1),
        (KJ1T_INDEX, reading.kj1t),
        (KJ2_INDEX, reading.kj2),
        (KJ3_INDEX, reading.kj3),
    ] {
        if let Some(is_violation) = verdict {
            cell.judged[criterion_index] += 1;
            if is_violation {
                cell.violations[criterion_index] += 1;
                if is_cut_by_the_limit {
                    cell.cut_violations[criterion_index] += 1;
                }
                violated[criterion_index] = true;
            }
        }
    }
    violated
}

/// 一个被判态上三步的结局（只在 `run_the_three_steps` 时算；单测里为省时可以关掉）。
fn outcome_of_the_three_steps_on_a_faulted_disk(
    disk: &FaultedRootChoiceDisk<'_, '_>,
    system_configuration: &SystemConfiguration,
) -> RootChoiceOutcome {
    IS_INSIDE_THE_REPLAY_CALL.with(|is_inside| is_inside.set(true));
    let outcome = outcome_of_the_three_steps_with_the_panic_hook_quiet(disk, system_configuration);
    IS_INSIDE_THE_REPLAY_CALL.with(|is_inside| is_inside.set(false));
    outcome
}

fn outcome_of_the_three_steps_with_the_panic_hook_quiet(
    disk: &FaultedRootChoiceDisk<'_, '_>,
    system_configuration: &SystemConfiguration,
) -> RootChoiceOutcome {
    let chosen_root =
        match panic::catch_unwind(AssertUnwindSafe(|| choose_root(disk, system_configuration))) {
            Err(_panic_payload) => {
                return RootChoiceOutcome::Panicked {
                    step: "choose_root",
                }
            }
            Ok(None) => return RootChoiceOutcome::ChoseNothing,
            Ok(Some(root)) => root,
        };
    let records = match panic::catch_unwind(AssertUnwindSafe(|| {
        scan_journal(disk, system_configuration)
    })) {
        Err(_panic_payload) => {
            return RootChoiceOutcome::Panicked {
                step: "scan_journal",
            }
        }
        Ok(records) => records,
    };
    let replay = panic::catch_unwind(AssertUnwindSafe(|| {
        replay_journal(
            disk,
            &chosen_root,
            system_configuration.immutable.sizes.journal_ring_bytes,
            &records,
            true,
        )
    }));
    match replay {
        Err(_panic_payload) => RootChoiceOutcome::Panicked {
            step: "replay_journal",
        },
        Ok(Err(_failure)) => RootChoiceOutcome::Landed {
            chosen: RootVersion::of_root(&chosen_root),
            landed_version: RootVersion::of_root(&chosen_root),
            landed_fields: VersionFields::of_root(&chosen_root),
            replay_failed: true,
        },
        Ok(Ok((_report, returned_root))) => RootChoiceOutcome::Landed {
            chosen: RootVersion::of_root(&chosen_root),
            landed_version: RootVersion::of_root(&returned_root),
            landed_fields: VersionFields::of_root(&returned_root),
            replay_failed: false,
        },
    }
}

/// 读槽对拍（停机 TH4 的输入，登记 5.5 末段）：被判时故障的落点必须读成 `Bad(Unreadable)`；别的槽读成自证过的，它的 (txg, 实例代号) 等于真相；
/// 全 0 的槽读成自证不过、而且真相里这个槽是空的。
fn fault_slot_reading_agrees_with_the_truth(
    reading: RootRingSlotReading,
    truth: Option<&RootChoicePublishMaterial>,
    is_unreadable_in_this_view: bool,
) -> bool {
    if is_unreadable_in_this_view {
        return matches!(
            reading,
            RootRingSlotReading::Bad(BadRootRingSlotReading::Unreadable)
        );
    }
    root_slot_reading_agrees_with_the_truth(reading, truth)
}

/// 逐槽调 `read_root_ring_slot`，与真相对拍；不一致数一笔、记第一个见证，收尾走停机条款 TH4。
fn check_the_fault_slot_readings_against_the_truth(
    disk: &FaultedRootChoiceDisk<'_, '_>,
    system_configuration: &SystemConfiguration,
    world: &FaultWorld,
    tally: &mut FaultTally,
    describe_state: &dyn Fn() -> String,
) {
    tally.slot_readings_checked += 1;
    let immutable = &system_configuration.immutable;
    let slots_per_region = world.slots_per_region();
    for ring_slot in every_root_ring_slot(&immutable.sizes) {
        let reading = read_root_ring_slot(
            disk,
            &immutable.region_devices,
            &immutable.sizes,
            &immutable.filesystem_identifier,
            ring_slot,
        );
        let slot_index =
            root_choice_slot_index_of(ring_slot.region, ring_slot.slot, slots_per_region);
        let truth = world.root_slots[slot_index].as_deref();
        let is_unreadable = disk
            .locations
            .root_slot_is_unreadable(slot_index, slots_per_region);
        if !fault_slot_reading_agrees_with_the_truth(reading, truth, is_unreadable) {
            tally.slot_readings_differing_from_the_truth += 1;
            if tally.slot_reading_witness.is_none() {
                tally.slot_reading_witness = Some(format!(
                    "region{}_slot{}_read_as={reading:?}_truth={}_unreadable_in_the_view={is_unreadable};{}",
                    ring_slot.region,
                    ring_slot.slot,
                    truth.map_or_else(|| "empty".to_string(), |material| material.version.describe()),
                    describe_state()
                ));
            }
        }
    }
}

fn describe_a_system_slot(content: SystemSlotContent) -> String {
    match content {
        SystemSlotContent::Torn => "torn".to_string(),
        SystemSlotContent::Verified {
            generation,
            tail,
            instance,
        } => format!("g{generation}t{tail}i{instance}"),
    }
}

/// 一段历史的写法（见证用）：每次挂载开头的故障与走了哪一遍、写行 txg，开始过的每次发布，系统配置槽，最后一次的崩溃点。
fn describe_a_fault_history(world: &FaultWorld) -> String {
    let mounts: Vec<String> = world
        .mount_log
        .iter()
        .enumerate()
        .map(|(position, entry)| {
            format!(
                "mount{}:fault={}:{:?}:row_write_txg={}",
                position + 1,
                entry.fault.label(),
                entry.outcome,
                entry.row_write_txg
            )
        })
        .collect();
    let publishes: Vec<String> = world
        .started_publishes
        .iter()
        .skip(1)
        .map(|material| {
            format!(
                "txg{}i{}k{}{}",
                material.version.checkpoint_txg,
                material.version.instance,
                material.records.len(),
                if material.is_user_publish { "u" } else { "o" }
            )
        })
        .collect();
    let system_slots: Vec<String> = world
        .system_slots
        .iter()
        .map(|slots| {
            slots
                .iter()
                .map(|content| describe_a_system_slot(*content))
                .collect::<Vec<String>>()
                .join("/")
        })
        .collect();
    let slots_per_region = usize::try_from(world.slots_per_region()).expect("每区槽数 4 或 8");
    let roots: Vec<String> = world
        .root_slots
        .iter()
        .enumerate()
        .filter_map(|(root_slot_index, root)| {
            root.as_ref().map(|material| {
                format!(
                    "region{}slot{}={}",
                    root_slot_index / slots_per_region,
                    root_slot_index % slots_per_region,
                    material.version.describe()
                )
            })
        })
        .collect();
    let confirmed: Vec<String> = world
        .confirmed_user_publishes
        .iter()
        .map(|version| version.describe())
        .collect();
    let mut chain_of_the_last_confirmed = Vec::new();
    let mut current = world.confirmed_user_publishes.iter().copied().max();
    while let Some(version) = current {
        chain_of_the_last_confirmed.push(version.describe());
        current = world.parent_of(version);
        if chain_of_the_last_confirmed.len() > world.parents.len() {
            break;
        }
    }
    format!(
        "{};publishes={};roots_in_ring_order={};confirmed={};parent_chain_of_the_last_confirmed={};system_slots_d0|d1={};{}",
        mounts.join("+"),
        publishes.join(","),
        roots.join("+"),
        confirmed.join(","),
        chain_of_the_last_confirmed.join("<-"),
        system_slots.join("|"),
        describe_a_fault_crash(world.final_crash)
    )
}

fn describe_a_fault_crash(crash: Option<FaultCrashLabel>) -> String {
    let Some(label) = crash else {
        return "no_crash_the_last_mount_was_refused".to_string();
    };
    let point_text = match label.point {
        FaultCrashPoint::RecordCopiesLanded { copy_mask } => {
            format!("record_copies_mask_{copy_mask:b}_root_not_landed")
        }
        FaultCrashPoint::RootLanded { system_writes } => {
            format!("root_landed_system_writes_{system_writes:?}")
        }
    };
    format!(
        "crash_in_publish_txg{}_instance{}_{}records_{point_text}",
        label.version.checkpoint_txg, label.version.instance, label.record_count
    )
}

// ── 判一个最后一次挂载走完的历史：5 种被判时故障各判一次（登记 5.3 第 11 条）

struct FaultJudgement {
    domain: FaultDomain,
    /// 这个格的 M（M = 2 的格按挂载 2 开头那项故障另报）。
    mounts: usize,
    system_configuration: SystemConfiguration,
    in_flight_limit: usize,
    /// 关掉时不调三步（单测里为省时；产物里恒开）。
    run_the_three_steps: bool,
}

/// 一段历史上与被判时故障无关的真相：每个被判态共用。
struct FaultHistoryFacts {
    is_inside_the_single_fault_promise: bool,
    last_confirmed: Option<RootVersion>,
    parent_of_the_last_confirmed: Option<RootVersion>,
    /// 根环真相里不是最后一次已确认那一版、也不是它后代的根里键最大的那一条（KF4 的落点）。
    kf4_landing: Option<RootVersion>,
    has_same_txg_different_instance_roots: bool,
    has_higher_instance_lower_txg_roots: bool,
    has_a_torn_system_slot: bool,
}

fn facts_of_a_fault_history(world: &FaultWorld, ring_slots: u64) -> FaultHistoryFacts {
    let versions_in_the_true_ring: Vec<RootVersion> = world
        .root_slots
        .iter()
        .flatten()
        .map(|material| material.version)
        .collect();
    let last_confirmed = world.confirmed_user_publishes.iter().copied().max();
    if let Some(last) = last_confirmed {
        for confirmed in &world.confirmed_user_publishes {
            assert!(
                world.is_the_version_or_a_descendant_of(last, *confirmed),
                "停机条款 TH9：已确认的几次发布不在一条父链上（{}、{}）",
                last.describe(),
                confirmed.describe()
            );
        }
    }
    let largest_counter = world
        .landed_records
        .iter()
        .map(|landed| landed.counter)
        .max()
        .unwrap_or(0);
    assert!(
        largest_counter <= ring_slots,
        "停机条款 TH9：界内最大计数器 {largest_counter} 超过环槽数 {ring_slots}，journal 环回绕了"
    );
    let mut has_same_txg_different_instance_roots = false;
    let mut has_higher_instance_lower_txg_roots = false;
    for earlier in &versions_in_the_true_ring {
        for later in &versions_in_the_true_ring {
            has_same_txg_different_instance_roots |= earlier.checkpoint_txg == later.checkpoint_txg
                && earlier.instance != later.instance;
            has_higher_instance_lower_txg_roots |=
                earlier.instance > later.instance && earlier.checkpoint_txg < later.checkpoint_txg;
        }
    }
    FaultHistoryFacts {
        is_inside_the_single_fault_promise: world
            .mount_log
            .iter()
            .all(|entry| entry.fault.is_inside_the_single_fault_promise()),
        last_confirmed,
        parent_of_the_last_confirmed: last_confirmed.and_then(|last| world.parent_of(last)),
        kf4_landing: last_confirmed.and_then(|last| {
            versions_in_the_true_ring
                .iter()
                .copied()
                .filter(|version| !world.is_the_version_or_a_descendant_of(*version, last))
                .max()
        }),
        has_same_txg_different_instance_roots,
        has_higher_instance_lower_txg_roots,
        has_a_torn_system_slot: world
            .system_slots
            .iter()
            .flatten()
            .any(|content| *content == SystemSlotContent::Torn),
    }
}

impl FaultJudgement {
    fn new(domain: FaultDomain, mounts: usize, run_the_three_steps: bool) -> Self {
        Self {
            domain,
            mounts,
            system_configuration: system_configuration_for_the_root_choice(domain.geometry()),
            in_flight_limit: domain.geometry().ring().local_in_flight_limit(),
            run_the_three_steps,
        }
    }

    /// 一段历史：先记与被判时故障无关的计数，再对 5 种被判时故障各判一个状态。
    fn judge_a_history(
        &self,
        world: &FaultWorld,
        is_the_designated_history: bool,
        tally: &mut FaultTally,
    ) {
        let history = facts_of_a_fault_history(world, self.domain.geometry().ring().ring_slots);
        tally.histories += 1;
        tally.refused_mounts += world.refused_mounts;
        for (count, world_count) in tally
            .refusals_by_reason
            .iter_mut()
            .zip(&world.refusals_by_reason)
        {
            *count += world_count;
        }
        tally.reread_mounts += world.reread_mounts;
        tally.largest_counter = tally.largest_counter.max(
            world
                .landed_records
                .iter()
                .map(|landed| landed.counter)
                .max()
                .unwrap_or(0),
        );
        tally.confirmed_publishes_maximum = tally
            .confirmed_publishes_maximum
            .max(world.confirmed_user_publishes.len());
        if !history.is_inside_the_single_fault_promise {
            tally.histories_with_an_uncovered_fault += 1;
        }
        if self.mounts == 2 {
            assert_eq!(world.mount_log.len(), 2, "M = 2 的历史恰好开始过两次挂载");
            let count = &mut tally.mount_two[world.mount_log[1].fault.menu_index];
            count.histories += 1;
            count.refused += world.refused_mounts;
            count.reread += world.reread_mounts;
        }
        let truth = fault_landing_of_a_view(world, FaultLocations::NOWHERE);
        let disk = RootChoiceDisk::of_the_landing(self.domain.geometry(), &truth);
        for judged_fault in JudgedFault::ALL {
            let is_the_designated_state =
                is_the_designated_history && *judged_fault == JudgedFault::Nothing;
            self.judge_a_state(
                world,
                &history,
                &disk,
                (*judged_fault, is_the_designated_state),
                tally,
            );
        }
    }
}

impl FaultJudgement {
    /// 一个被判态：读视图 = 真相的盘减去被判时故障的落点。先记与实现无关的计数与分类，再算本地择根序与 KF2、KF4 的假结局读数，最后调三步判四条判据。
    fn judge_a_state(
        &self,
        world: &FaultWorld,
        history: &FaultHistoryFacts,
        disk: &RootChoiceDisk<'_>,
        (judged_fault, is_the_designated_state): (JudgedFault, bool),
        tally: &mut FaultTally,
    ) {
        let locations = judged_fault.locations(world);
        let view = fault_landing_of_a_view(world, locations);
        let describe_state = || {
            format!(
                "judged_fault={};history_inside_the_promise={};{}",
                judged_fault.label(),
                history.is_inside_the_single_fault_promise,
                describe_a_fault_history(world)
            )
        };
        tally.judged_states += 1;
        tally.states_with_a_torn_system_slot += u64::from(history.has_a_torn_system_slot);
        tally.states_with_same_txg_different_instance_roots +=
            u64::from(history.has_same_txg_different_instance_roots);
        tally.states_with_higher_instance_lower_txg_roots +=
            u64::from(history.has_higher_instance_lower_txg_roots);
        let newest_root = root_choice_newest_root(&view);
        match newest_root {
            None => tally.states_without_a_readable_root += 1,
            Some(newest) => {
                let above_newest_root = view
                    .records
                    .iter()
                    .filter(|record| {
                        record.material.version.instance == newest.version.instance
                            && record.material.version.checkpoint_txg
                                > newest.version.checkpoint_txg
                    })
                    .count();
                tally.above_newest_root_maximum =
                    tally.above_newest_root_maximum.max(above_newest_root);
            }
        }
        let ideal = newest_root.map(|newest| {
            root_choice_ideal_recovery_from(newest, &view.records, self.in_flight_limit)
        });
        let facts = FaultStateFacts {
            last_confirmed: history.last_confirmed,
            newest_readable: newest_root.map(|newest| newest.version),
            ideal_fields: ideal.as_ref().map(|recovery| recovery.landed_fields),
        };
        let cell_index =
            fault_cell_index_of(judged_fault, history.is_inside_the_single_fault_promise);
        tally.cells[cell_index].states += 1;
        let is_cut_by_the_limit = self.classify_the_ideal_recovery(
            world,
            history,
            &view,
            newest_root,
            &ideal,
            judged_fault,
            cell_index,
            tally,
        );
        if is_cut_by_the_limit {
            tally.cells[cell_index].cut_states += 1;
        }
        let is_inside_the_promise = history.is_inside_the_single_fault_promise
            && judged_fault.is_inside_the_single_fault_promise()
            && !is_cut_by_the_limit;
        if is_inside_the_promise {
            tally.promised_states += 1;
        }
        self.read_the_local_rules_and_the_fake_outcomes(
            world,
            history,
            &view,
            &facts,
            judged_fault,
            tally,
        );
        if !self.run_the_three_steps {
            return;
        }
        if is_the_designated_state {
            let mut scratch = FaultTally::new();
            self.count_the_outcome(
                world,
                &facts,
                &RootChoiceOutcome::Panicked {
                    step: "designated_state",
                },
                (cell_index, false, false),
                &describe_state,
                &mut scratch,
            );
            tally.kj0_control_violations += scratch.violations_of_a_criterion(0);
        }
        let faulted_disk = FaultedRootChoiceDisk {
            inner: disk,
            locations,
            slots_per_region: world.slots_per_region(),
        };
        check_the_fault_slot_readings_against_the_truth(
            &faulted_disk,
            &self.system_configuration,
            world,
            tally,
            &describe_state,
        );
        let outcome =
            outcome_of_the_three_steps_on_a_faulted_disk(&faulted_disk, &self.system_configuration);
        let violated = self.count_the_outcome(
            world,
            &facts,
            &outcome,
            (cell_index, is_cut_by_the_limit, newest_root.is_some()),
            &describe_state,
            tally,
        );
        if is_inside_the_promise {
            for (position, criterion_index) in [KJ1_INDEX, KJ1T_INDEX].into_iter().enumerate() {
                if violated[criterion_index] {
                    tally.promised_violations[position] += 1;
                    if tally.promised_witnesses[position].is_none() {
                        tally.promised_witnesses[position] = Some(describe_state());
                    }
                }
            }
        }
    }
}

impl FaultJudgement {
    /// 有已确认的被判态：数一笔，再分类条款的理想恢复（读视图里版本最大的根起）丢没丢最后一次已确认的发布：
    /// 去掉在飞上限就不丢 ⇒ 上限截掉；否则被判时与历史都罩着 ⇒ 罩着的其余；否则不罩。交回「是上限截掉」。
    #[allow(
        clippy::too_many_arguments,
        reason = "一个被判态的分类要同时看历史、读视图、理想恢复与两个标签，拆开只是把同一组量再传一遍"
    )]
    fn classify_the_ideal_recovery(
        &self,
        world: &FaultWorld,
        history: &FaultHistoryFacts,
        view: &RootChoiceLanding<'_>,
        newest_root: Option<&RootChoicePublishMaterial>,
        ideal: &Option<RootChoiceIdealRecovery>,
        judged_fault: JudgedFault,
        cell_index: usize,
        tally: &mut FaultTally,
    ) -> bool {
        let Some(last) = history.last_confirmed else {
            return false;
        };
        tally.confirmed_states += 1;
        tally.cells[cell_index].confirmed_states += 1;
        let (Some(newest), Some(ideal_recovery)) = (newest_root, ideal) else {
            return false;
        };
        let ideal_version = ideal_recovery.landed_version;
        let ideal_is_earlier = ideal_version < last;
        let ideal_is_off_the_chain = !world.is_the_version_or_a_descendant_of(ideal_version, last);
        tally.cells[cell_index].ideal_violations[0] += u64::from(ideal_is_earlier);
        tally.cells[cell_index].ideal_violations[1] += u64::from(ideal_is_off_the_chain);
        if !(ideal_is_earlier || ideal_is_off_the_chain) {
            return false;
        }
        let without_the_limit =
            root_choice_ideal_recovery_from(newest, &view.records, FAULT_IN_FLIGHT_LIMIT_REMOVED)
                .landed_version;
        let is_cut_by_the_limit = without_the_limit >= last
            && world.is_the_version_or_a_descendant_of(without_the_limit, last);
        if is_cut_by_the_limit {
            tally.ideal_loss_by_limit += 1;
        } else if history.is_inside_the_single_fault_promise
            && judged_fault.is_inside_the_single_fault_promise()
        {
            tally.ideal_loss_covered_other += 1;
        } else {
            tally.ideal_loss_uncovered += 1;
        }
        is_cut_by_the_limit
    }

    /// 本地择根序加理想重放的七种读数（登记第七节「本地择根序」那张表；下标 6 是 KF1），KF2（落在最后一次已确认那一版的父上）与 KF4（落在根环真相里
    /// 不在已确认那条链上的根中键最大的那条）的假结局读数。全部走 [`fault_criteria_of_a_landing`]，与真实现的结局读同一段判据。
    fn read_the_local_rules_and_the_fake_outcomes(
        &self,
        world: &FaultWorld,
        history: &FaultHistoryFacts,
        view: &RootChoiceLanding<'_>,
        facts: &FaultStateFacts,
        judged_fault: JudgedFault,
        tally: &mut FaultTally,
    ) {
        for (rule_index, rule) in FaultLocalRule::ALL.iter().enumerate() {
            let Some(chosen) = rule.chooses_among(&view.roots_in_ring_order) else {
                continue;
            };
            let landed_version =
                root_choice_ideal_recovery_from(chosen, &view.records, self.in_flight_limit)
                    .landed_version;
            let reading = fault_criteria_of_a_landing(world, facts, landed_version, None);
            for (position, verdict) in [reading.kj1, reading.kj1t, reading.kj2]
                .into_iter()
                .enumerate()
            {
                tally.local_rules[rule_index][position] += u64::from(verdict == Some(true));
            }
            tally.local_kj1t_by_judged_fault[rule_index][judged_fault.index()] +=
                u64::from(reading.kj1t == Some(true));
        }
        if history.last_confirmed.is_none() {
            return;
        }
        if let Some(parent) = history.parent_of_the_last_confirmed {
            let reading = fault_criteria_of_a_landing(
                world,
                facts,
                parent,
                world.fields_of_a_version(parent),
            );
            tally.kf2[0] += u64::from(reading.kj1 == Some(true));
            tally.kf2[1] += u64::from(reading.kj1t == Some(true));
            tally.kf2[2] += u64::from(reading.kj2 == Some(true));
            // 读视图里一条根都没有时没有理想恢复可比：假结局落在一版上，照 KJ3 的字面算违例。
            tally.kf2[3] += u64::from(reading.kj3.unwrap_or(facts.ideal_fields.is_none()));
        }
        match history.kf4_landing {
            Some(landing) => {
                let reading = fault_criteria_of_a_landing(world, facts, landing, None);
                tally.kf4[0] += u64::from(reading.kj1 == Some(true));
                tally.kf4[1] += u64::from(reading.kj1t == Some(true));
            }
            None => tally.kf4_states_without_a_root_off_the_confirmed_chain += 1,
        }
    }
}

impl FaultJudgement {
    /// 一个被判态的结局落进判据（各报各的，不合成一个真假）：真实现的结局与阳性对照的假结局走这一段。
    /// `(cell_index, is_cut_by_the_limit, a_root_is_readable)`：这一态落在哪个格、是不是上限截掉、读视图里有没有读得出的根。
    /// 交回每条判据这一态是不是违例（承诺内那一行用）。
    fn count_the_outcome(
        &self,
        world: &FaultWorld,
        facts: &FaultStateFacts,
        outcome: &RootChoiceOutcome,
        (cell_index, is_cut_by_the_limit, a_root_is_readable): (usize, bool, bool),
        describe_state: &dyn Fn() -> String,
        tally: &mut FaultTally,
    ) -> [bool; FAULT_CRITERION_COUNT] {
        let mut violated = [false; FAULT_CRITERION_COUNT];
        tally.cells[cell_index].judged[0] += 1;
        match outcome {
            RootChoiceOutcome::Panicked { step } => {
                tally.panicked_states += 1;
                self.add_a_kj0_violation(cell_index, is_cut_by_the_limit, tally, &|| {
                    format!("panicked_in={step};{}", describe_state())
                });
                violated[0] = true;
            }
            RootChoiceOutcome::ChoseNothing => {
                if a_root_is_readable {
                    tally.chose_nothing_with_a_readable_root += 1;
                    self.add_a_kj0_violation(cell_index, is_cut_by_the_limit, tally, &|| {
                        format!(
                            "choose_root_returned_none_with_a_readable_root;{}",
                            describe_state()
                        )
                    });
                    violated[0] = true;
                } else {
                    tally.chose_nothing_without_a_readable_root += 1;
                }
            }
            RootChoiceOutcome::Landed {
                chosen,
                landed_version,
                landed_fields,
                replay_failed,
            } => {
                tally.replay_failed_states += u64::from(*replay_failed);
                tally.replay_applied_states += u64::from(landed_version != chosen);
                let reading = fault_criteria_of_a_landing(
                    world,
                    facts,
                    *landed_version,
                    Some(*landed_fields),
                );
                let counted = count_a_criteria_reading_into_a_cell(
                    reading,
                    is_cut_by_the_limit,
                    &mut tally.cells[cell_index],
                );
                for criterion in FaultCriterion::ALL {
                    let criterion_index = criterion.index();
                    if criterion_index == 0 || !counted[criterion_index] {
                        continue;
                    }
                    violated[criterion_index] = true;
                    let witness =
                        &mut tally.witnesses[cell_index * FAULT_CRITERION_COUNT + criterion_index];
                    if witness.is_none() {
                        *witness = Some(format!(
                            "{};chosen={};landed={};last_confirmed={};newest_readable={};{}",
                            criterion.label(),
                            chosen.describe(),
                            landed_version.describe(),
                            facts
                                .last_confirmed
                                .map_or_else(|| "none".to_string(), RootVersion::describe),
                            facts
                                .newest_readable
                                .map_or_else(|| "none".to_string(), RootVersion::describe),
                            describe_state()
                        ));
                    }
                }
                if reading.kj1.is_some() && reading.kj1 != reading.kj1t {
                    tally.kj1_and_kj1t_differing_states += 1;
                    if tally.kj1_and_kj1t_differing_witness.is_none() {
                        tally.kj1_and_kj1t_differing_witness = Some(format!(
                            "kj1_violated={:?};kj1t_violated={:?};landed={};{}",
                            reading.kj1,
                            reading.kj1t,
                            landed_version.describe(),
                            describe_state()
                        ));
                    }
                }
            }
        }
        violated
    }

    fn add_a_kj0_violation(
        &self,
        cell_index: usize,
        is_cut_by_the_limit: bool,
        tally: &mut FaultTally,
        describe_witness: &dyn Fn() -> String,
    ) {
        tally.add_cell_violation(cell_index, 0, describe_witness);
        if is_cut_by_the_limit {
            tally.cells[cell_index].cut_violations[0] += 1;
        }
    }
}

// ── 一个格的计数：枚举、判、按工作单元分给几条线程

/// 工作单元：挂载 M 开始之前的盘，再加故障菜单里的一项。前面几次挂载的全部走法在崩溃之后得到的盘，每个单元接着枚举最后一次挂载的一项故障。
struct FaultUnit {
    world: FaultWorld,
    mount_number: usize,
    user_publishes_left: usize,
    menu_index: usize,
}

struct FaultUnitCollector {
    units: Vec<FaultUnit>,
}

impl FaultVisitor for FaultUnitCollector {
    fn reach_the_start_of_a_mount(
        &mut self,
        world: &FaultWorld,
        mount_number: usize,
        user_publishes_left: usize,
    ) {
        for entry in mount_fault_menu(world, mount_number) {
            self.units.push(FaultUnit {
                world: world.clone(),
                mount_number,
                user_publishes_left,
                menu_index: entry.menu_index,
            });
        }
    }

    fn visit_a_final_state(&mut self, _world: &FaultWorld) {
        panic!("收工作单元的那一遍停在最后一次挂载开始之前，走不到最后一次挂载的崩溃点");
    }
}

struct FaultStateJudge<'run> {
    judgement: &'run FaultJudgement,
    tally: FaultTally,
    histories_seen_in_this_unit: u64,
    is_the_first_unit: bool,
}

impl FaultVisitor for FaultStateJudge<'_> {
    fn reach_the_start_of_a_mount(
        &mut self,
        _world: &FaultWorld,
        _mount_number: usize,
        _user_publishes_left: usize,
    ) {
        panic!("判状态的那一遍不设 stop_before_mount");
    }

    fn visit_a_final_state(&mut self, world: &FaultWorld) {
        let is_the_designated_history =
            self.is_the_first_unit && self.histories_seen_in_this_unit == 0;
        self.histories_seen_in_this_unit += 1;
        self.judgement
            .judge_a_history(world, is_the_designated_history, &mut self.tally);
    }
}

/// 一个 (域, M, U) 格上全部历史的计数，闸取甲（r4 档；只给单测用，跑法里一律带闸的取法）。
#[cfg(test)]
fn fault_tally_of_a_cell(
    domain: FaultDomain,
    cell: RootChoiceCell,
    run_the_three_steps: bool,
    thread_count: usize,
) -> FaultTally {
    fault_tally_of_a_cell_under_the_gate(
        domain,
        cell,
        WitnessGateRule::OneUnreadableOrTornSlotMeansUndecidable,
        run_the_three_steps,
        thread_count,
    )
}

/// 一个 (域, M, U) 格上全部历史的计数，闸取 `witness_gate_rule`。合法历史与 KG 按工作单元分给几条线程；KA 不走这里。
fn fault_tally_of_a_cell_under_the_gate(
    domain: FaultDomain,
    cell: RootChoiceCell,
    witness_gate_rule: WitnessGateRule,
    run_the_three_steps: bool,
    thread_count: usize,
) -> FaultTally {
    let collecting = FaultEnumeration {
        domain,
        mounts: cell.mounts,
        stop_before_mount: Some(cell.mounts),
    };
    let mut collector = FaultUnitCollector { units: Vec::new() };
    collecting.enumerate_a_mount(
        &FaultWorld::right_after_mkfs_under_the_gate(domain.geometry(), witness_gate_rule),
        1,
        cell.user_publishes,
        None,
        &mut collector,
    );
    let judgement = FaultJudgement::new(domain, cell.mounts, run_the_three_steps);
    let continuing = FaultEnumeration {
        domain,
        mounts: cell.mounts,
        stop_before_mount: None,
    };
    fault_tally_of_work_units(&collector.units, thread_count, &|unit_index, unit| {
        let mut judge = FaultStateJudge {
            judgement: &judgement,
            tally: FaultTally::new(),
            histories_seen_in_this_unit: 0,
            is_the_first_unit: unit_index == 0,
        };
        continuing.enumerate_a_mount(
            &unit.world,
            unit.mount_number,
            unit.user_publishes_left,
            Some(unit.menu_index),
            &mut judge,
        );
        judge.tally
    })
}

/// 登记第七节乙类（出自设计员的独立生成器 `anchors_r4.py`，输出 `anchors_r4.out` sha256 5a36b72237f84c52e6901ae5affcfc5431fd4c8b123f5b9e476907b749ba1fc7）：
/// 每行 = 域 M U，然后依次是：历史数、历史里有不罩故障的历史数、被判态数、有已确认的被判态数、被拒的挂载数、走了重读的挂载数、带撕裂系统配置槽的被判态数、
/// 同 txg 不同实例同在的被判态数、实例高而 txg 低同在的被判态数、视图里没有根的被判态数、视图里最大根之上的记录条数最大值、
/// 条款理想恢复丢的被判态按因在飞上限 / 罩着的其余 / 不罩三类的数；KF1 的 KJ1 KJ1T KJ2；KF2 的 KJ1 KJ1T KJ2 KJ3；KF4 的 KJ1 KJ1T；
/// 本地择根序 today v1823 v1824 v1825 v1826 v1822 各自的 KJ1 KJ1T KJ2。
const PINNED_FAULT_CELLS_TEXT: &str = "\
KF 1 0 10 0 50 0 0 0 0 0 0 0 1 0 0 0 0 0 8 0 0 0 0 0 0 0 0 0 0 0 0 0 0 8 0 0 0 0 0 0 0 0 0\n\
KF 1 1 39 0 195 0 0 0 0 0 0 0 3 0 0 0 0 0 117 0 0 0 0 0 0 0 0 0 0 0 0 0 0 117 0 0 0 0 0 0 0 0 0\n\
KF 1 2 117 0 585 585 0 0 0 0 0 0 5 0 0 117 468 468 351 585 585 252 585 585 585 117 117 0 117 117 0 468 468 351 117 117 0 117 117 0 117 117 0\n\
KF 1 3 351 0 1755 1755 0 0 0 0 0 0 4 0 0 351 1404 1404 1053 1755 1755 756 1755 1755 1755 351 351 0 351 351 0 1404 1404 1053 351 351 0 351 351 0 351 351 0\n\
KF 2 0 1792 236 8960 0 212 1020 650 0 0 0 2 0 0 0 0 0 4827 0 0 0 0 0 0 0 0 0 0 0 0 0 0 4827 0 0 0 0 0 0 0 0 0\n\
KF 2 1 9401 1218 47005 0 647 5598 1625 0 0 0 3 0 0 0 0 0 40494 0 0 0 0 0 0 0 0 0 0 0 0 0 0 40494 0 0 0 0 0 0 0 0 0\n\
KF 2 2 48590 6207 242950 164835 1952 29529 4550 0 0 0 5 0 0 20655 145557 145557 213495 164835 164835 99999 164835 164835 164835 20655 20655 0 20655 20655 0 145557 145557 213495 20655 20655 0 20655 20655 0 20655 20655 0\n\
KF 2 3 175589 22353 877945 874710 5867 107217 13325 0 0 0 6 0 0 108198 785763 785763 804138 874710 874710 576648 874710 874710 874710 108198 108198 0 108198 108198 0 785763 785763 804138 108198 108198 0 108198 108198 0 108198 108198 0\n\
KF3 1 0 10 0 50 0 0 0 0 0 0 0 1 0 0 0 0 0 8 0 0 0 0 0 0 0 0 0 0 0 0 0 0 8 0 0 0 0 0 0 0 0 0\n\
KF3 1 1 5 0 25 0 0 0 0 0 0 0 2 0 0 0 0 0 15 0 0 0 0 0 0 0 0 0 0 0 0 0 0 15 0 0 0 0 0 0 0 0 0\n\
KF3 1 2 5 0 25 25 0 0 0 0 0 0 3 0 0 5 20 20 15 25 25 12 25 25 25 5 5 0 5 5 0 20 20 15 5 5 0 5 5 0 5 5 0\n\
KF3 1 3 5 0 25 25 0 0 0 0 0 0 2 0 0 5 20 20 15 25 25 12 25 25 25 5 5 0 5 5 0 20 20 15 5 5 0 5 5 0 5 5 0\n\
KF3 2 0 1792 236 8960 0 212 1020 650 0 0 0 2 0 0 0 0 0 4827 0 0 0 0 0 0 0 0 0 0 0 0 0 0 4827 0 0 0 0 0 0 0 0 0\n\
KF3 2 1 1587 214 7935 0 317 810 975 0 0 0 2 0 0 0 0 0 6254 0 0 0 0 0 0 0 0 0 0 0 0 0 0 6254 0 0 0 0 0 0 0 0 0\n\
KF3 2 2 2292 307 11460 8275 422 1185 1300 0 0 0 3 0 0 991 7205 7205 9049 8275 8275 5623 8275 8275 8275 991 991 0 991 991 0 7205 7205 9049 991 991 0 991 991 0 991 991 0\n\
KF3 2 3 2477 335 12385 10800 527 1235 1625 0 0 0 3 0 0 1416 9545 9545 9964 10800 10800 7520 10800 10800 10800 1416 1416 0 1416 1416 0 9545 9545 9964 1416 1416 0 1416 1416 0 1416 1416 0\n\
KF3 3 0 275939 67845 1379695 0 46096 334980 111150 0 0 0 2 0 0 0 0 0 1091126 0 0 0 0 0 0 0 0 0 0 0 0 0 0 1091126 0 0 0 0 0 0 0 0 0\n\
KF3 3 1 350403 86331 1752015 0 73710 411175 197925 0 0 0 2 0 0 0 0 0 1633188 0 0 0 0 0 0 0 0 0 0 0 0 0 0 1633188 0 0 0 0 0 0 0 0 0\n\
KF3 3 2 549582 134997 2747910 1782245 118454 634670 323700 0 0 0 3 0 0 157713 1612806 1612806 2553380 1782245 1782245 1335924 1782245 1782245 1782245 157713 157713 0 157713 157713 0 1612806 1612806 2553380 157713 157713 0 157713 157713 0 157713 157713 0\n\
KF3 3 3 765601 187840 3828005 3371140 162278 884730 454675 0 0 0 3 0 0 277332 3110727 3110727 3647308 3371140 3371140 2653847 3371140 3371140 3371140 277332 277332 0 277332 277332 0 3110727 3110727 3647308 277332 277332 0 277332 277332 0 277332 277332 0\n\
KF9 1 0 10 0 50 0 0 0 0 0 0 0 1 0 0 0 0 0 8 0 0 0 0 0 0 0 0 0 0 0 0 0 0 8 0 0 0 0 0 0 0 0 0\n\
KF9 1 1 39 0 195 0 0 0 0 0 0 0 3 0 0 0 0 0 117 0 0 0 0 0 0 0 0 0 0 0 0 0 0 117 0 0 0 0 0 0 0 0 0\n\
KF9 1 2 117 0 585 585 0 0 0 0 0 0 5 0 0 117 468 468 359 585 585 252 585 585 585 117 117 0 117 117 0 468 468 359 117 117 0 117 117 0 117 117 0\n\
KF9 2 0 1792 236 8960 0 212 1020 650 0 0 0 2 0 0 0 0 0 4827 0 0 0 0 0 0 0 0 0 0 0 0 0 0 4827 0 0 0 0 0 0 0 0 0\n\
KF9 2 1 9401 1218 47005 0 647 5598 1625 0 0 0 3 0 0 0 0 0 40494 0 0 0 0 0 0 0 0 0 0 0 0 0 0 40494 0 0 0 0 0 0 0 0 0\n\
KF9 2 2 48590 6207 242950 164835 1952 29529 4550 0 0 0 5 0 0 20655 145557 145557 217831 164835 164835 99999 164835 164835 164835 20655 20655 0 20655 20655 0 145557 145557 217831 20655 20655 0 20655 20655 0 20655 20655 0\n\
KFW 1 0 10 0 50 0 0 0 0 0 0 0 1 0 0 0 0 0 8 0 0 0 0 0 0 0 0 0 0 0 0 0 0 8 0 0 0 0 0 0 0 0 0\n\
KFW 1 1 5 0 25 0 0 0 0 0 0 0 2 0 0 0 0 0 15 0 0 0 0 0 0 0 0 0 0 0 0 0 0 15 0 0 0 0 0 0 0 0 0\n\
KFW 1 2 5 0 25 25 0 0 0 0 0 0 3 0 0 5 20 20 15 25 25 12 25 25 25 5 5 0 5 5 0 20 20 15 5 5 0 5 5 0 5 5 0\n\
KFW 1 3 5 0 25 25 0 0 0 0 0 0 2 0 0 5 20 20 15 25 25 12 25 25 25 5 5 0 5 5 0 20 20 15 5 5 0 5 5 0 5 5 0\n\
KFW 1 4 5 0 25 25 0 0 0 0 0 0 2 0 0 5 20 20 15 25 25 11 25 25 25 5 5 0 5 5 0 20 20 15 5 5 0 5 5 0 5 5 0\n\
KFW 1 5 5 0 25 25 0 0 0 0 0 0 3 0 0 5 20 20 15 25 25 12 25 25 25 5 5 0 5 5 0 20 20 15 5 5 0 5 5 0 5 5 0\n\
KFW 1 6 5 0 25 25 0 0 0 0 0 0 2 0 0 5 20 20 15 25 25 12 25 25 25 5 5 0 5 5 0 20 20 15 5 5 0 5 5 0 5 5 0\n\
KFW 1 7 5 0 25 25 0 0 0 0 0 0 2 0 0 5 20 20 15 25 25 11 25 25 25 5 5 0 5 5 0 20 20 15 5 5 0 5 5 0 5 5 0\n\
KFW 1 8 5 0 25 25 0 0 0 0 0 0 3 0 0 5 20 20 15 25 25 12 25 25 25 5 5 0 5 5 0 20 20 15 5 5 0 5 5 0 5 5 0\n\
KFW 1 9 5 0 25 25 0 0 0 0 0 0 2 0 0 5 20 20 15 25 25 12 25 25 25 5 5 0 5 5 0 20 20 15 5 5 0 5 5 0 5 5 0\n\
KFW 1 10 5 0 25 25 0 0 0 0 0 1 2 0 0 4 16 16 12 25 25 11 25 25 25 4 4 0 4 4 0 16 16 12 4 4 0 4 4 0 4 4 0\n\
KFW 1 11 5 0 25 25 0 0 0 0 0 5 3 0 0 0 0 0 0 25 25 12 25 25 25 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0\n\
KFW 1 12 5 0 25 25 0 0 0 0 0 5 2 0 0 0 0 0 0 25 25 12 25 25 25 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0\n\
KFW 1 13 5 0 25 25 0 0 0 0 0 5 2 0 0 0 0 0 0 25 25 11 25 25 25 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0\n\
KFW 1 14 5 0 25 25 0 0 0 0 0 5 3 0 0 0 0 0 0 25 25 12 25 25 25 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0\n\
KG 1 0 10 0 50 0 0 0 0 0 0 0 1 0 0 0 0 0 8 0 0 0 0 0 0 0 0 0 0 0 0 0 0 8 0 0 0 0 0 0 0 0 0\n\
KG 1 1 39 0 195 0 0 0 0 0 0 0 3 0 0 0 0 0 117 0 0 0 0 0 0 0 0 0 0 0 0 0 0 117 0 0 0 0 0 0 0 0 0\n\
KG 1 2 117 0 585 585 0 0 0 0 0 0 5 0 0 117 468 468 351 585 585 252 585 585 585 117 117 0 117 117 0 468 468 351 117 117 0 117 117 0 117 117 0\n\
KG 1 3 351 0 1755 1755 0 0 0 0 0 0 4 0 0 351 1404 1404 1053 1755 1755 756 1755 1755 1755 351 351 0 351 351 0 1404 1404 1053 351 351 0 351 351 0 351 351 0\n\
KG 2 0 4015 610 20075 0 0 0 925 0 450 0 2 0 0 0 0 0 10824 0 0 0 0 0 0 0 0 0 0 0 0 0 0 10824 0 0 0 0 0 36 0 0 0\n\
KG 2 1 22092 3648 110460 0 0 0 3120 0 3420 0 3 0 0 0 0 0 88296 0 0 0 0 0 0 0 0 0 0 0 0 0 0 88296 0 0 0 0 0 630 0 0 0\n\
KG 2 2 98892 14778 494460 334755 0 0 19035 0 21060 0 5 0 1512 51642 292302 292302 423162 334755 334755 188253 334755 334755 334755 53154 53154 0 53154 53154 0 292302 292302 423162 53154 53154 0 58824 58824 7236 53154 53154 0\n\
KG 2 3 400194 61506 2000970 2000970 0 0 56160 0 104490 0 6 0 19116 265491 1765179 1770093 1756485 2000970 2000970 1261629 2000970 1995300 2000970 265923 284607 0 265923 284607 0 1765179 1770093 1756485 265923 284607 0 309609 346005 49680 265923 284607 0\n\
";

/// 登记第七节「挂载 2 开头那次故障」表（M = 2 的各格合起来；每行 = 域、故障名、历史数、被拒数、走了重读的数）。
const PINNED_MOUNT_TWO_FAULT_TEXT: &str = "\
KF none 28776 310 0\n\
KF RN-T1 28776 310 3639\n\
KF RN-TP 25165 338 0\n\
KF SN0-T1 28776 310 28466\n\
KF SN0-TP 1238 1238 0\n\
KF SN1-T1 28776 310 28466\n\
KF SN1-TP 1238 1238 0\n\
KF SB-T1 28776 310 28466\n\
KF SB-TP 1238 1238 0\n\
KF RI-T1 28776 310 25861\n\
KF RI-TP 3823 1218 0\n\
KF RNSB-T1 28776 310 28466\n\
KF RNSB-TP 1238 1238 0\n\
KF3 none 910 70 0\n\
KF3 RN-T1 910 70 195\n\
KF3 RN-TP 743 98 0\n\
KF3 SN0-T1 910 70 840\n\
KF3 SN0-TP 182 182 0\n\
KF3 SN1-T1 910 70 840\n\
KF3 SN1-TP 182 182 0\n\
KF3 SB-T1 910 70 840\n\
KF3 SB-TP 182 182 0\n\
KF3 RI-T1 910 70 695\n\
KF3 RI-TP 307 162 0\n\
KF3 RNSB-T1 910 70 840\n\
KF3 RNSB-TP 182 182 0\n\
KF9 none 7268 105 0\n\
KF9 RN-T1 7268 105 1182\n\
KF9 RN-TP 6107 126 0\n\
KF9 SN0-T1 7268 105 7163\n\
KF9 SN0-TP 393 393 0\n\
KF9 SN1-T1 7268 105 7163\n\
KF9 SN1-TP 393 393 0\n\
KF9 SB-T1 7268 105 7163\n\
KF9 SB-TP 393 393 0\n\
KF9 RI-T1 7268 105 6313\n\
KF9 RI-TP 1228 378 0\n\
KF9 RNSB-T1 7268 105 7163\n\
KF9 RNSB-TP 393 393 0\n\
KG none 40921 0 0\n\
KG RN-T1 40271 0 0\n\
KG RN-TP 40271 0 0\n\
KG SN0-T1 40921 0 0\n\
KG SN0-TP 40921 0 0\n\
KG SN1-T1 40921 0 0\n\
KG SN1-TP 40921 0 0\n\
KG SB-T1 40921 0 0\n\
KG SB-TP 40921 0 0\n\
KG RI-T1 38831 0 0\n\
KG RI-TP 38831 0 0\n\
KG RNSB-T1 40271 0 0\n\
KG RNSB-TP 40271 0 0\n\
";

/// 登记 r5 修订 1 修1.七乙类（闸取 R2；出自设计员的独立生成器 `anchors_s5.py`，sha256 133e56358b88a284e9a5afcbb19197be3e6d2b6ce571c2f87549809144b83af7，
/// 输出 `anchors_s5.out` sha256 4578b08579af7e0198e23326b61ea4fe4711464533c72da930bfdfe8a42230c1，由第三段执行员的 `to_pinned_text.py cells` 只换形摊成这一张；
/// 同一份脚本对 `E165_S5_GATE=A` 那一遍的输出摊出来与 [`PINNED_FAULT_CELLS_TEXT`] 逐行相同）：列同 [`PINNED_FAULT_CELLS_TEXT`]。
/// 与取甲那一张不同的 15 行是 KF、KF3、KF9 的 M ≥ 2 各格，其余 34 行逐行相同。
const PINNED_FAULT_CELLS_TEXT_UNDER_R2: &str = "\
KF 1 0 10 0 50 0 0 0 0 0 0 0 1 0 0 0 0 0 8 0 0 0 0 0 0 0 0 0 0 0 0 0 0 8 0 0 0 0 0 0 0 0 0\n\
KF 1 1 39 0 195 0 0 0 0 0 0 0 3 0 0 0 0 0 117 0 0 0 0 0 0 0 0 0 0 0 0 0 0 117 0 0 0 0 0 0 0 0 0\n\
KF 1 2 117 0 585 585 0 0 0 0 0 0 5 0 0 117 468 468 351 585 585 252 585 585 585 117 117 0 117 117 0 468 468 351 117 117 0 117 117 0 117 117 0\n\
KF 1 3 351 0 1755 1755 0 0 0 0 0 0 4 0 0 351 1404 1404 1053 1755 1755 756 1755 1755 1755 351 351 0 351 351 0 1404 1404 1053 351 351 0 351 351 0 351 351 0\n\
KF 2 0 3094 351 15470 0 99 1130 245 0 0 0 2 0 0 0 0 0 9265 0 0 0 0 0 0 0 0 0 0 0 0 0 0 9265 0 0 0 0 0 0 0 0 0\n\
KF 2 1 15639 1733 78195 0 318 5619 620 0 0 0 3 0 0 0 0 0 70332 0 0 0 0 0 0 0 0 0 0 0 0 0 0 70332 0 0 0 0 0 0 0 0 0\n\
KF 2 2 78888 8567 394440 275850 975 27900 1745 0 0 0 5 0 0 35820 247356 247356 354423 275850 275850 167166 275850 275850 275850 35820 35820 0 35820 35820 0 247356 247356 354423 35820 35820 0 35820 35820 0 35820 35820 0\n\
KF 2 3 282585 30518 1412925 1411335 2946 99414 5120 0 0 0 6 0 0 178317 1279296 1279296 1316046 1411335 1411335 919764 1411335 1411335 1411335 178317 178317 0 178317 178317 0 1279296 1279296 1316046 178317 178317 0 178317 178317 0 178317 178317 0\n\
KF3 1 0 10 0 50 0 0 0 0 0 0 0 1 0 0 0 0 0 8 0 0 0 0 0 0 0 0 0 0 0 0 0 0 8 0 0 0 0 0 0 0 0 0\n\
KF3 1 1 5 0 25 0 0 0 0 0 0 0 2 0 0 0 0 0 15 0 0 0 0 0 0 0 0 0 0 0 0 0 0 15 0 0 0 0 0 0 0 0 0\n\
KF3 1 2 5 0 25 25 0 0 0 0 0 0 3 0 0 5 20 20 15 25 25 12 25 25 25 5 5 0 5 5 0 20 20 15 5 5 0 5 5 0 5 5 0\n\
KF3 1 3 5 0 25 25 0 0 0 0 0 0 2 0 0 5 20 20 15 25 25 12 25 25 25 5 5 0 5 5 0 20 20 15 5 5 0 5 5 0 5 5 0\n\
KF3 2 0 3094 351 15470 0 99 1130 245 0 0 0 2 0 0 0 0 0 9265 0 0 0 0 0 0 0 0 0 0 0 0 0 0 9265 0 0 0 0 0 0 0 0 0\n\
KF3 2 1 2543 299 12715 0 148 905 370 0 0 0 2 0 0 0 0 0 11002 0 0 0 0 0 0 0 0 0 0 0 0 0 0 11002 0 0 0 0 0 0 0 0 0\n\
KF3 2 2 3742 437 18710 14970 197 1340 495 0 0 0 3 0 0 1724 13116 13116 15849 14970 14970 10462 14970 14970 14970 1724 1724 0 1724 1724 0 13116 13116 15849 1724 1724 0 1724 1724 0 1724 1724 0\n\
KF3 2 3 3891 460 19455 18715 246 1370 620 0 0 0 3 0 0 2373 16712 16712 16946 18715 18715 13092 18715 18715 18715 2373 2373 0 2373 2373 0 16712 16712 16946 2373 2373 0 2373 2373 0 2373 2373 0\n\
KF3 3 0 885618 188613 4428090 0 42196 668541 74905 0 0 0 2 0 0 0 0 0 3783158 0 0 0 0 0 0 0 0 0 0 0 0 0 0 3783158 0 0 0 0 0 0 0 0 0\n\
KF3 3 1 1068016 228892 5340080 0 64571 796720 134805 0 0 0 2 0 0 0 0 0 5173977 0 0 0 0 0 0 0 0 0 0 0 0 0 0 5173977 0 0 0 0 0 0 0 0 0\n\
KF3 3 2 1665004 357011 8325020 5864165 104556 1233377 223455 0 0 0 3 0 0 507003 5333364 5333364 7951618 5864165 5864165 4481658 5864165 5864165 5864165 507003 507003 0 507003 507003 0 5333364 5333364 7951618 507003 507003 0 507003 507003 0 507003 507003 0\n\
KF3 3 3 2326572 498392 11632860 10967505 141571 1726053 314605 0 0 0 3 0 0 870431 10186388 10186388 11363419 10967505 10967505 8745782 10967505 10967505 10967505 870431 870431 0 870431 870431 0 10186388 10186388 11363419 870431 870431 0 870431 870431 0 870431 870431 0\n\
KF9 1 0 10 0 50 0 0 0 0 0 0 0 1 0 0 0 0 0 8 0 0 0 0 0 0 0 0 0 0 0 0 0 0 8 0 0 0 0 0 0 0 0 0\n\
KF9 1 1 39 0 195 0 0 0 0 0 0 0 3 0 0 0 0 0 117 0 0 0 0 0 0 0 0 0 0 0 0 0 0 117 0 0 0 0 0 0 0 0 0\n\
KF9 1 2 117 0 585 585 0 0 0 0 0 0 5 0 0 117 468 468 359 585 585 252 585 585 585 117 117 0 117 117 0 468 468 359 117 117 0 117 117 0 117 117 0\n\
KF9 2 0 3094 351 15470 0 99 1130 245 0 0 0 2 0 0 0 0 0 9265 0 0 0 0 0 0 0 0 0 0 0 0 0 0 9265 0 0 0 0 0 0 0 0 0\n\
KF9 2 1 15639 1733 78195 0 318 5619 620 0 0 0 3 0 0 0 0 0 70332 0 0 0 0 0 0 0 0 0 0 0 0 0 0 70332 0 0 0 0 0 0 0 0 0\n\
KF9 2 2 78888 8567 394440 275850 975 27900 1745 0 0 0 5 0 0 35820 247356 247356 363767 275850 275850 167166 275850 275850 275850 35820 35820 0 35820 35820 0 247356 247356 363767 35820 35820 0 35820 35820 0 35820 35820 0\n\
KFW 1 0 10 0 50 0 0 0 0 0 0 0 1 0 0 0 0 0 8 0 0 0 0 0 0 0 0 0 0 0 0 0 0 8 0 0 0 0 0 0 0 0 0\n\
KFW 1 1 5 0 25 0 0 0 0 0 0 0 2 0 0 0 0 0 15 0 0 0 0 0 0 0 0 0 0 0 0 0 0 15 0 0 0 0 0 0 0 0 0\n\
KFW 1 2 5 0 25 25 0 0 0 0 0 0 3 0 0 5 20 20 15 25 25 12 25 25 25 5 5 0 5 5 0 20 20 15 5 5 0 5 5 0 5 5 0\n\
KFW 1 3 5 0 25 25 0 0 0 0 0 0 2 0 0 5 20 20 15 25 25 12 25 25 25 5 5 0 5 5 0 20 20 15 5 5 0 5 5 0 5 5 0\n\
KFW 1 4 5 0 25 25 0 0 0 0 0 0 2 0 0 5 20 20 15 25 25 11 25 25 25 5 5 0 5 5 0 20 20 15 5 5 0 5 5 0 5 5 0\n\
KFW 1 5 5 0 25 25 0 0 0 0 0 0 3 0 0 5 20 20 15 25 25 12 25 25 25 5 5 0 5 5 0 20 20 15 5 5 0 5 5 0 5 5 0\n\
KFW 1 6 5 0 25 25 0 0 0 0 0 0 2 0 0 5 20 20 15 25 25 12 25 25 25 5 5 0 5 5 0 20 20 15 5 5 0 5 5 0 5 5 0\n\
KFW 1 7 5 0 25 25 0 0 0 0 0 0 2 0 0 5 20 20 15 25 25 11 25 25 25 5 5 0 5 5 0 20 20 15 5 5 0 5 5 0 5 5 0\n\
KFW 1 8 5 0 25 25 0 0 0 0 0 0 3 0 0 5 20 20 15 25 25 12 25 25 25 5 5 0 5 5 0 20 20 15 5 5 0 5 5 0 5 5 0\n\
KFW 1 9 5 0 25 25 0 0 0 0 0 0 2 0 0 5 20 20 15 25 25 12 25 25 25 5 5 0 5 5 0 20 20 15 5 5 0 5 5 0 5 5 0\n\
KFW 1 10 5 0 25 25 0 0 0 0 0 1 2 0 0 4 16 16 12 25 25 11 25 25 25 4 4 0 4 4 0 16 16 12 4 4 0 4 4 0 4 4 0\n\
KFW 1 11 5 0 25 25 0 0 0 0 0 5 3 0 0 0 0 0 0 25 25 12 25 25 25 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0\n\
KFW 1 12 5 0 25 25 0 0 0 0 0 5 2 0 0 0 0 0 0 25 25 12 25 25 25 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0\n\
KFW 1 13 5 0 25 25 0 0 0 0 0 5 2 0 0 0 0 0 0 25 25 11 25 25 25 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0\n\
KFW 1 14 5 0 25 25 0 0 0 0 0 5 3 0 0 0 0 0 0 25 25 12 25 25 25 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0\n\
KG 1 0 10 0 50 0 0 0 0 0 0 0 1 0 0 0 0 0 8 0 0 0 0 0 0 0 0 0 0 0 0 0 0 8 0 0 0 0 0 0 0 0 0\n\
KG 1 1 39 0 195 0 0 0 0 0 0 0 3 0 0 0 0 0 117 0 0 0 0 0 0 0 0 0 0 0 0 0 0 117 0 0 0 0 0 0 0 0 0\n\
KG 1 2 117 0 585 585 0 0 0 0 0 0 5 0 0 117 468 468 351 585 585 252 585 585 585 117 117 0 117 117 0 468 468 351 117 117 0 117 117 0 117 117 0\n\
KG 1 3 351 0 1755 1755 0 0 0 0 0 0 4 0 0 351 1404 1404 1053 1755 1755 756 1755 1755 1755 351 351 0 351 351 0 1404 1404 1053 351 351 0 351 351 0 351 351 0\n\
KG 2 0 4015 610 20075 0 0 0 925 0 450 0 2 0 0 0 0 0 10824 0 0 0 0 0 0 0 0 0 0 0 0 0 0 10824 0 0 0 0 0 36 0 0 0\n\
KG 2 1 22092 3648 110460 0 0 0 3120 0 3420 0 3 0 0 0 0 0 88296 0 0 0 0 0 0 0 0 0 0 0 0 0 0 88296 0 0 0 0 0 630 0 0 0\n\
KG 2 2 98892 14778 494460 334755 0 0 19035 0 21060 0 5 0 1512 51642 292302 292302 423162 334755 334755 188253 334755 334755 334755 53154 53154 0 53154 53154 0 292302 292302 423162 53154 53154 0 58824 58824 7236 53154 53154 0\n\
KG 2 3 400194 61506 2000970 2000970 0 0 56160 0 104490 0 6 0 19116 265491 1765179 1770093 1756485 2000970 2000970 1261629 2000970 1995300 2000970 265923 284607 0 265923 284607 0 1765179 1770093 1756485 265923 284607 0 309609 346005 49680 265923 284607 0\n\
";

/// 登记 r5 修订 1 修1.七乙类「M = 2 的格按挂载 2 开头那一项故障合起来」（闸取 R2；同一份 `anchors_s5.out`，`to_pinned_text.py mount_two` 摊成）：
/// 列同 [`PINNED_MOUNT_TWO_FAULT_TEXT`]。
const PINNED_MOUNT_TWO_FAULT_TEXT_UNDER_R2: &str = "\
KF none 39931 0 0\n\
KF RN-T1 39931 0 4683\n\
KF RN-TP 35284 36 0\n\
KF SN0-T1 39931 0 6879\n\
KF SN0-TP 28776 310 0\n\
KF SN1-T1 39931 0 6879\n\
KF SN1-TP 28776 310 0\n\
KF SB-T1 39931 0 39931\n\
KF SB-TP 1238 1238 0\n\
KF RI-T1 39931 0 35760\n\
KF RI-TP 5377 1206 0\n\
KF RNSB-T1 39931 0 39931\n\
KF RNSB-TP 1238 1238 0\n\
KF3 none 1365 0 0\n\
KF3 RN-T1 1365 0 255\n\
KF3 RN-TP 1146 36 0\n\
KF3 SN0-T1 1365 0 315\n\
KF3 SN0-TP 910 70 0\n\
KF3 SN1-T1 1365 0 315\n\
KF3 SN1-TP 910 70 0\n\
KF3 SB-T1 1365 0 1365\n\
KF3 SB-TP 182 182 0\n\
KF3 RI-T1 1365 0 1130\n\
KF3 RI-TP 385 150 0\n\
KF3 RNSB-T1 1365 0 1365\n\
KF3 RNSB-TP 182 182 0\n\
KF9 none 10258 0 0\n\
KF9 RN-T1 10258 0 1524\n\
KF9 RN-TP 8761 27 0\n\
KF9 SN0-T1 10258 0 1857\n\
KF9 SN0-TP 7268 105 0\n\
KF9 SN1-T1 10258 0 1857\n\
KF9 SN1-TP 7268 105 0\n\
KF9 SB-T1 10258 0 10258\n\
KF9 SB-TP 393 393 0\n\
KF9 RI-T1 10258 0 8895\n\
KF9 RI-TP 1732 369 0\n\
KF9 RNSB-T1 10258 0 10258\n\
KF9 RNSB-TP 393 393 0\n\
KG none 40921 0 0\n\
KG RN-T1 40271 0 0\n\
KG RN-TP 40271 0 0\n\
KG SN0-T1 40921 0 0\n\
KG SN0-TP 40921 0 0\n\
KG SN1-T1 40921 0 0\n\
KG SN1-TP 40921 0 0\n\
KG SB-T1 40921 0 0\n\
KG SB-TP 40921 0 0\n\
KG RI-T1 38831 0 0\n\
KG RI-TP 38831 0 0\n\
KG RNSB-T1 40271 0 0\n\
KG RNSB-TP 40271 0 0\n\
";

/// 登记 r5 修订 1 修1.七乙类「拒因分计」（闸取 R2；同一份 `anchors_s5.out` 的 `refused_gate`、`refused_acq_device_without_self_verified`、
/// `refused_acq_undecidable` 三个键，`to_pinned_text.py refusals` 摊成）：每行 = 域 M U，然后是闸拒（重读那一遍仍真）、取号那一刻拒（某块盘一份自证过的都没有）、
/// 取号那一刻拒（判不出）。
const PINNED_REFUSALS_BY_REASON_TEXT_UNDER_R2: &str = "\
KF 1 0 0 0 0\n\
KF 1 1 0 0 0\n\
KF 1 2 0 0 0\n\
KF 1 3 0 0 0\n\
KF 2 0 91 8 0\n\
KF 2 1 298 20 0\n\
KF 2 2 919 56 0\n\
KF 2 3 2782 164 0\n\
KF3 1 0 0 0 0\n\
KF3 1 1 0 0 0\n\
KF3 1 2 0 0 0\n\
KF3 1 3 0 0 0\n\
KF3 2 0 91 8 0\n\
KF3 2 1 136 12 0\n\
KF3 2 2 181 16 0\n\
KF3 2 3 226 20 0\n\
KF3 3 0 38920 3276 0\n\
KF3 3 1 59499 5072 0\n\
KF3 3 2 96288 8268 0\n\
KF3 3 3 130347 11224 0\n\
KF9 1 0 0 0 0\n\
KF9 1 1 0 0 0\n\
KF9 1 2 0 0 0\n\
KF9 2 0 91 8 0\n\
KF9 2 1 298 20 0\n\
KF9 2 2 919 56 0\n\
KFW 1 0 0 0 0\n\
KFW 1 1 0 0 0\n\
KFW 1 2 0 0 0\n\
KFW 1 3 0 0 0\n\
KFW 1 4 0 0 0\n\
KFW 1 5 0 0 0\n\
KFW 1 6 0 0 0\n\
KFW 1 7 0 0 0\n\
KFW 1 8 0 0 0\n\
KFW 1 9 0 0 0\n\
KFW 1 10 0 0 0\n\
KFW 1 11 0 0 0\n\
KFW 1 12 0 0 0\n\
KFW 1 13 0 0 0\n\
KFW 1 14 0 0 0\n\
KG 1 0 0 0 0\n\
KG 1 1 0 0 0\n\
KG 1 2 0 0 0\n\
KG 1 3 0 0 0\n\
KG 2 0 0 0 0\n\
KG 2 1 0 0 0\n\
KG 2 2 0 0 0\n\
KG 2 3 0 0 0\n\
";

/// 一行钉值（`PINNED_FAULT_CELLS_TEXT` 里的一行）：一个 (域, M, U) 格上生成器一侧的计数与假结局、本地择根序的读数。
#[derive(Clone, Debug)]
struct PinnedFaultCell {
    domain: FaultDomain,
    mounts: usize,
    user_publishes: usize,
    histories: u64,
    histories_with_an_uncovered_fault: u64,
    judged_states: u64,
    confirmed_states: u64,
    refused_mounts: u64,
    reread_mounts: u64,
    states_with_a_torn_system_slot: u64,
    states_with_same_txg_different_instance_roots: u64,
    states_with_higher_instance_lower_txg_roots: u64,
    states_without_a_readable_root: u64,
    above_newest_root_maximum: usize,
    /// 条款的理想恢复丢了的被判态：因在飞上限、罩着的其余、不罩。
    ideal_loss: [u64; 3],
    kf1: [u64; 3],
    kf2: [u64; 4],
    kf4: [u64; 2],
    /// 本地择根序 today、v1823、v1824、v1825、v1826、v1822 各自的 KJ1、KJ1T、KJ2。
    local_rules: [[u64; 3]; 6],
}

fn parse_the_pinned_fault_cells(text: &str) -> Vec<PinnedFaultCell> {
    text.lines()
        .map(|line| {
            let fields: Vec<&str> = line.split_whitespace().collect();
            let domain = *FaultDomain::ALL
                .iter()
                .find(|domain| domain.label() == fields[0])
                .expect("钉值表里的域名在六个域之内");
            let numbers: Vec<u64> = fields[3..]
                .iter()
                .map(|field| field.parse::<u64>().expect("钉值表里的数是非负整数"))
                .collect();
            assert_eq!(numbers.len(), 41, "钉值表每行 41 个数：{line}");
            let mut local_rules = [[0u64; 3]; 6];
            for (rule_index, rule) in local_rules.iter_mut().enumerate() {
                rule.copy_from_slice(&numbers[23 + 3 * rule_index..26 + 3 * rule_index]);
            }
            PinnedFaultCell {
                domain,
                mounts: fields[1].parse().expect("M 是非负整数"),
                user_publishes: fields[2].parse().expect("U 是非负整数"),
                histories: numbers[0],
                histories_with_an_uncovered_fault: numbers[1],
                judged_states: numbers[2],
                confirmed_states: numbers[3],
                refused_mounts: numbers[4],
                reread_mounts: numbers[5],
                states_with_a_torn_system_slot: numbers[6],
                states_with_same_txg_different_instance_roots: numbers[7],
                states_with_higher_instance_lower_txg_roots: numbers[8],
                states_without_a_readable_root: numbers[9],
                above_newest_root_maximum: usize::try_from(numbers[10])
                    .expect("记录条数装得进 usize"),
                ideal_loss: [numbers[11], numbers[12], numbers[13]],
                kf1: [numbers[14], numbers[15], numbers[16]],
                kf2: [numbers[17], numbers[18], numbers[19], numbers[20]],
                kf4: [numbers[21], numbers[22]],
                local_rules,
            }
        })
        .collect()
}

fn pinned_fault_cells() -> &'static Vec<PinnedFaultCell> {
    static PINNED: std::sync::OnceLock<Vec<PinnedFaultCell>> = std::sync::OnceLock::new();
    PINNED.get_or_init(|| parse_the_pinned_fault_cells(PINNED_FAULT_CELLS_TEXT))
}

/// 闸取 `witness_gate_rule` 时那一张钉值表：取甲是 [`PINNED_FAULT_CELLS_TEXT`]，R2 是 [`PINNED_FAULT_CELLS_TEXT_UNDER_R2`]。
fn pinned_fault_cells_under_the_gate(
    witness_gate_rule: WitnessGateRule,
) -> &'static Vec<PinnedFaultCell> {
    match witness_gate_rule {
        WitnessGateRule::OneUnreadableOrTornSlotMeansUndecidable => pinned_fault_cells(),
        WitnessGateRule::SomeSlotReportsAReadErrorAndNoDeviceSelfVerifiedBothSlots => {
            static PINNED_UNDER_R2: std::sync::OnceLock<Vec<PinnedFaultCell>> =
                std::sync::OnceLock::new();
            PINNED_UNDER_R2
                .get_or_init(|| parse_the_pinned_fault_cells(PINNED_FAULT_CELLS_TEXT_UNDER_R2))
        }
    }
}

fn pinned_fault_cell_under_the_gate(
    witness_gate_rule: WitnessGateRule,
    domain: FaultDomain,
    cell: RootChoiceCell,
) -> Option<&'static PinnedFaultCell> {
    pinned_fault_cells_under_the_gate(witness_gate_rule)
        .iter()
        .find(|pinned| {
            pinned.domain == domain
                && pinned.mounts == cell.mounts
                && pinned.user_publishes == cell.user_publishes
        })
}

/// R2 那一张拒因钉值表里 (域, M, U) 那一格的三栏：闸拒、取号那一刻拒（某块盘一份自证过的都没有）、取号那一刻拒（判不出）。
fn pinned_refusals_by_reason_under_r2(
    domain: FaultDomain,
    cell: RootChoiceCell,
) -> Option<[u64; 3]> {
    PINNED_REFUSALS_BY_REASON_TEXT_UNDER_R2
        .lines()
        .map(|line| line.split_whitespace().collect::<Vec<&str>>())
        .find(|fields| {
            fields[0] == domain.label()
                && fields[1].parse::<usize>().ok() == Some(cell.mounts)
                && fields[2].parse::<usize>().ok() == Some(cell.user_publishes)
        })
        .map(|fields| {
            let numbers: Vec<u64> = fields[3..]
                .iter()
                .map(|field| field.parse::<u64>().expect("拒因钉值表里的数是非负整数"))
                .collect();
            assert_eq!(numbers.len(), 3, "拒因钉值表每行三栏：{fields:?}");
            [numbers[0], numbers[1], numbers[2]]
        })
}

/// 本地择根序 today 在 KG 上、被判时无故障的态里的 KJ1T 合计（登记 5.6「C331 的判别力自证」：本地理想恢复算出的数是 648）。
const PINNED_KG_TODAY_KJ1T_WITHOUT_A_JUDGED_FAULT: u64 = 648;

/// 每个跑到的格，装置数出的生成器一侧的计数与本地择根序的读数等于独立生成器的数；不符走作废条款 TV4（这些量只看生成器与真相，与被测实现无关）。
/// 条款的理想恢复在合法域里丢了的被判态（因在飞上限、罩着的其余）不是 0 时不当 TV4 停，交回登记的 TF9（条款或实现可能错）。
/// 闸取甲（只给单测用，跑法里一律带闸的取法）。
#[cfg(test)]
fn assert_the_pinned_fault_cell(
    domain: FaultDomain,
    cell: RootChoiceCell,
    tally: &FaultTally,
) -> Vec<String> {
    assert_the_pinned_fault_cell_under_the_gate(
        WitnessGateRule::OneUnreadableOrTornSlotMeansUndecidable,
        domain,
        cell,
        tally,
    )
}

/// 每个跑到的格的钉值比对（同上一个函数的说法），按闸的取法选表：取甲照 r4 登记第七节乙类，R2 照 r5 登记修订 1 修1.七乙类，另核拒因三栏（各栏之和等于被拒的挂载数）。
fn assert_the_pinned_fault_cell_under_the_gate(
    witness_gate_rule: WitnessGateRule,
    domain: FaultDomain,
    cell: RootChoiceCell,
    tally: &FaultTally,
) -> Vec<String> {
    assert_eq!(
        tally.refusals_by_reason.iter().sum::<u64>(),
        tally.refused_mounts,
        "拒因各栏之和等于被拒的挂载数（{} M = {} U = {}）",
        domain.label(),
        cell.mounts,
        cell.user_publishes
    );
    match witness_gate_rule {
        WitnessGateRule::OneUnreadableOrTornSlotMeansUndecidable => {}
        WitnessGateRule::SomeSlotReportsAReadErrorAndNoDeviceSelfVerifiedBothSlots => {
            let pinned_refusals = pinned_refusals_by_reason_under_r2(domain, cell)
                .expect("每个跑到的格都在登记 r5 修订 1 修1.七乙类的拒因表里");
            assert_eq!(
                [
                    tally.refusals_by_reason[MountRefusalReason::TheGateAfterTheOneReread.index()],
                    tally.refusals_by_reason
                        [MountRefusalReason::AcquisitionDeviceWithoutASelfVerifiedSlot.index()],
                    tally.refusals_by_reason
                        [MountRefusalReason::AcquisitionWitnessUndecidable.index()],
                ],
                pinned_refusals,
                "PV12：{} M = {} U = {} 拒因三栏（闸、取号那一刻 (b)、取号那一刻 (a)）",
                domain.label(),
                cell.mounts,
                cell.user_publishes
            );
        }
    }
    let pinned = pinned_fault_cell_under_the_gate(witness_gate_rule, domain, cell)
        .expect("每个跑到的格都在登记第七节乙类的表里");
    let place = format!(
        "域 {} M = {} U = {}",
        domain.label(),
        cell.mounts,
        cell.user_publishes
    );
    assert_eq!(tally.histories, pinned.histories, "TV4：{place} 历史数");
    assert_eq!(
        tally.histories_with_an_uncovered_fault, pinned.histories_with_an_uncovered_fault,
        "TV4：{place} 历史里有不罩故障的历史数"
    );
    assert_eq!(
        tally.judged_states, pinned.judged_states,
        "TV4：{place} 被判态数"
    );
    assert_eq!(
        tally.confirmed_states, pinned.confirmed_states,
        "TV4：{place} 有已确认的被判态数"
    );
    assert_eq!(
        tally.refused_mounts, pinned.refused_mounts,
        "TV4：{place} 被拒的挂载数"
    );
    assert_eq!(
        tally.reread_mounts, pinned.reread_mounts,
        "TV4：{place} 走了重读那一遍的挂载数"
    );
    assert_eq!(
        tally.states_with_a_torn_system_slot, pinned.states_with_a_torn_system_slot,
        "TV4：{place} 带撕裂系统配置槽的被判态数"
    );
    assert_eq!(
        tally.states_with_same_txg_different_instance_roots,
        pinned.states_with_same_txg_different_instance_roots,
        "TV4：{place} 根环里同 txg 不同实例同在的被判态数"
    );
    assert_eq!(
        tally.states_with_higher_instance_lower_txg_roots,
        pinned.states_with_higher_instance_lower_txg_roots,
        "TV4：{place} 实例代号更高而 txg 更低同在的被判态数"
    );
    assert_eq!(
        tally.states_without_a_readable_root, pinned.states_without_a_readable_root,
        "TV4：{place} 读视图里一条根都没有的被判态数"
    );
    assert_eq!(
        tally.above_newest_root_maximum, pinned.above_newest_root_maximum,
        "TV4：{place} 读视图里版本最大的根之上同实例的记录条数的最大值"
    );
    assert_eq!(
        tally.ideal_loss_uncovered, pinned.ideal_loss[2],
        "TV4：{place} 条款的理想恢复丢了的被判态里「不罩」那一类"
    );
    let mut findings = Vec::new();
    if domain.is_a_legal_history() {
        if tally.ideal_loss_by_limit != pinned.ideal_loss[0]
            || tally.ideal_loss_covered_other != pinned.ideal_loss[1]
        {
            findings.push(format!(
                "TF9 {place} 条款的理想恢复丢了已确认发布的被判态：因在飞上限 {}、罩着的其余 {}（登记第七节甲类锚点都是 0）",
                tally.ideal_loss_by_limit, tally.ideal_loss_covered_other
            ));
        }
    } else {
        assert_eq!(
            tally.ideal_loss_by_limit, pinned.ideal_loss[0],
            "TV4：{place} 条款的理想恢复丢了的被判态里「因在飞上限」那一类"
        );
        assert_eq!(
            tally.ideal_loss_covered_other, pinned.ideal_loss[1],
            "TV4：{place} 条款的理想恢复丢了的被判态里「罩着的其余」那一类"
        );
    }
    for (rule_index, rule) in FaultLocalRule::ALL.iter().take(6).enumerate() {
        assert_eq!(
            tally.local_rules[rule_index],
            pinned.local_rules[rule_index],
            "TV4：{place} 本地择根序 {} 的 KJ1 / KJ1T / KJ2 读数",
            rule.label()
        );
    }
    findings
}

/// M = 2 的格合起来、按挂载 2 开头那项故障报的历史数、被拒数、重读数等于登记第七节那张表（TV4）；按闸的取法选表：取甲照 r4 登记第七节，
/// R2 照 r5 登记修订 1 修1.七乙类。
fn assert_the_pinned_mount_two_table_under_the_gate(
    witness_gate_rule: WitnessGateRule,
    domain: FaultDomain,
    merged_over_the_mount_two_cells: &FaultTally,
) {
    let pinned_text = match witness_gate_rule {
        WitnessGateRule::OneUnreadableOrTornSlotMeansUndecidable => PINNED_MOUNT_TWO_FAULT_TEXT,
        WitnessGateRule::SomeSlotReportsAReadErrorAndNoDeviceSelfVerifiedBothSlots => {
            PINNED_MOUNT_TWO_FAULT_TEXT_UNDER_R2
        }
    };
    let mut checked_rows = 0;
    for line in pinned_text.lines() {
        let fields: Vec<&str> = line.split_whitespace().collect();
        if fields[0] != domain.label() {
            continue;
        }
        let menu_index = MOUNT_FAULT_MENU_LABELS
            .iter()
            .position(|label| *label == fields[1])
            .expect("钉值表里的故障名在菜单之内");
        let pinned: Vec<u64> = fields[2..]
            .iter()
            .map(|field| field.parse::<u64>().expect("钉值表里的数是非负整数"))
            .collect();
        let count = merged_over_the_mount_two_cells.mount_two[menu_index];
        assert_eq!(
            [count.histories, count.refused, count.reread],
            [pinned[0], pinned[1], pinned[2]],
            "TV4：域 {} M = 2 挂载 2 开头故障 {} 的历史数 / 被拒数 / 重读数",
            domain.label(),
            fields[1]
        );
        checked_rows += 1;
    }
    assert_eq!(
        checked_rows, MOUNT_FAULT_MENU_LENGTH,
        "每个域的 M = 2 表 13 行"
    );
}

// ── 阳性对照（登记 5.6）：假结局读数与钉值比，走同一段判据；产物里打 `name=control` 行

#[derive(Clone, Debug)]
struct FaultControlReading {
    criterion: FaultCriterion,
    measured_where: String,
    value: u64,
    must_see: String,
    is_red_as_required: bool,
}

fn fault_reading_that_must_equal(
    criterion: FaultCriterion,
    measured_where: String,
    value: u64,
    expected: u64,
) -> FaultControlReading {
    FaultControlReading {
        criterion,
        measured_where,
        value,
        must_see: format!("equals_{expected}"),
        is_red_as_required: value == expected,
    }
}

/// R2 的拒因行（登记 r5 修订 1 修1.五第 4 条、修1.七乙类「拒因分计」）：被拒的挂载按闸（重读那一遍仍真）、取号那一刻 (b)（某块盘一份自证过的都没有）、
/// 取号那一刻 (a)（判不出）三栏分；三栏之和等于被拒的挂载数（[`assert_the_pinned_fault_cell_under_the_gate`] 断言）。
fn refusal_row_of_a_fault_cell(
    domain: FaultDomain,
    cell: RootChoiceCell,
    tally: &FaultTally,
) -> String {
    let columns: Vec<String> = [
        MountRefusalReason::TheGateAfterTheOneReread,
        MountRefusalReason::AcquisitionDeviceWithoutASelfVerifiedSlot,
        MountRefusalReason::AcquisitionWitnessUndecidable,
    ]
    .iter()
    .map(|reason| {
        format!(
            "refused_{}={}",
            reason.label(),
            tally.refusals_by_reason[reason.index()]
        )
    })
    .collect();
    format!(
        "name=root_choice_fault_refusals {} refused_mounts={} {}",
        fault_place_of(domain, cell),
        tally.refused_mounts,
        columns.join(" ")
    )
}

fn fault_place_of(domain: FaultDomain, cell: RootChoiceCell) -> String {
    format!(
        "domain={} ring_slots={} slots_per_region={} mounts={} user_publishes={}",
        domain.label(),
        domain.geometry().ring().ring_slots,
        domain.geometry().slots_per_region(),
        cell.mounts,
        cell.user_publishes
    )
}

/// 一个合法历史或 KG 格上每条判据的对照读数（登记 5.6 表）：KJ0 在 KF M = 1、U = 0 指定一态上 panic（违例数 1）；KJ1、KJ1T、KJ2 在 KF1 下等于第七节 KF1 三列；
/// KJ1、KJ1T、KJ3 在 KF2 下各等于「有已确认的被判态数」，KJ2 等于第七节 KF2 KJ2 一列；KJ1、KJ1T 在 KF4 下等于第七节 KF4 两列。
/// 闸取甲（只给单测用，跑法里一律带闸的取法）。
#[cfg(test)]
fn fault_control_readings_of_a_cell(
    domain: FaultDomain,
    cell: RootChoiceCell,
    tally: &FaultTally,
) -> Vec<FaultControlReading> {
    fault_control_readings_of_a_cell_under_the_gate(
        WitnessGateRule::OneUnreadableOrTornSlotMeansUndecidable,
        domain,
        cell,
        tally,
    )
}

/// 一个合法历史或 KG 格上每条判据的对照读数，按闸的取法取钉值：取甲照 r4 登记第七节，R2 照 r5 登记修订 1 修1.七乙类（判据那一套同 r4 登记 5.6）。
fn fault_control_readings_of_a_cell_under_the_gate(
    witness_gate_rule: WitnessGateRule,
    domain: FaultDomain,
    cell: RootChoiceCell,
    tally: &FaultTally,
) -> Vec<FaultControlReading> {
    let place = fault_place_of(domain, cell);
    let pinned = pinned_fault_cell_under_the_gate(witness_gate_rule, domain, cell)
        .expect("每个跑到的格都在登记第七节乙类的表里");
    let mut readings = Vec::new();
    if domain == FaultDomain::GateOn && cell.mounts == 1 && cell.user_publishes == 0 {
        readings.push(fault_reading_that_must_equal(
            FaultCriterion::Kj0Defined,
            format!("{place} fake_outcome=panics_on_the_designated_state"),
            tally.kj0_control_violations,
            1,
        ));
    }
    let oldest = tally.local_rules[FAULT_OLDEST_RULE_INDEX];
    for (criterion, position) in [
        (FaultCriterion::Kj1NotEarlierThanTheLastConfirmedPublish, 0),
        (FaultCriterion::Kj1tDescendsFromTheLastConfirmedPublish, 1),
        (FaultCriterion::Kj2NotEarlierThanAnyReadableRoot, 2),
    ] {
        readings.push(fault_reading_that_must_equal(
            criterion,
            format!("{place} fake_outcome=KF1_chooses_the_oldest_root"),
            oldest[position],
            pinned.kf1[position],
        ));
    }
    for (criterion, position, expected) in [
        (
            FaultCriterion::Kj1NotEarlierThanTheLastConfirmedPublish,
            0,
            pinned.confirmed_states,
        ),
        (
            FaultCriterion::Kj1tDescendsFromTheLastConfirmedPublish,
            1,
            pinned.confirmed_states,
        ),
        (
            FaultCriterion::Kj2NotEarlierThanAnyReadableRoot,
            2,
            pinned.kf2[2],
        ),
        (
            FaultCriterion::Kj3LandsOnTheIdealRecovery,
            3,
            pinned.confirmed_states,
        ),
    ] {
        readings.push(fault_reading_that_must_equal(
            criterion,
            format!("{place} fake_outcome=KF2_lands_on_the_parent_of_the_last_confirmed"),
            tally.kf2[position],
            expected,
        ));
    }
    for (criterion, position) in [
        (FaultCriterion::Kj1NotEarlierThanTheLastConfirmedPublish, 0),
        (FaultCriterion::Kj1tDescendsFromTheLastConfirmedPublish, 1),
    ] {
        readings.push(fault_reading_that_must_equal(
            criterion,
            format!("{place} fake_outcome=KF4_lands_on_the_newest_root_off_the_confirmed_chain"),
            tally.kf4[position],
            pinned.kf4[position],
        ));
    }
    readings
}

/// KA 上本地写的六种择根序（五种加平局取后者赢）的 KJ2 违例数必须等于 36、18、18、52、0 与 0（登记第七节 KA；今天的序 0）。
/// 次序：今天的序、平局取低、实例代号为主、比较不看实例、取第一个有效者、平局取后者赢。
const PINNED_KA_KJ2_OF_THE_LOCAL_RULES: [(FaultLocalRule, u64); 6] = [
    (FaultLocalRule::Today, 0),
    (FaultLocalRule::TieTakesTheLowerInstance, 36),
    (FaultLocalRule::InstanceFirstThenTxg, 18),
    (FaultLocalRule::IgnoresTheInstance, 18),
    (FaultLocalRule::FirstSelfVerifiedWins, 52),
    (FaultLocalRule::TieTakesTheLater, 0),
];

/// KA（登记 5.4，同 r3）：根环读根环次序的前三个槽各取空或四条根之一，别的槽全 0，journal 环全 0。本地写的一种择根序在这 125 个态上 KJ2 违例几次。
fn ka_kj2_violations_of_a_local_rule(rule: FaultLocalRule) -> u64 {
    let materials: Vec<Arc<RootChoicePublishMaterial>> = ROOT_CHOICE_ADVERSARIAL_ROOT_VERSIONS
        .iter()
        .enumerate()
        .map(|(version_index, version)| {
            Arc::new(root_material_without_records(
                version.instance,
                version.checkpoint_txg,
                NewRootSegment::off_timeline(
                    1 + u64::try_from(version_index).expect("四条根"),
                    version.checkpoint_txg,
                ),
            ))
        })
        .collect();
    let choices_per_slot = materials.len() + 1;
    let mut violations = 0;
    for state_number in 0..choices_per_slot
        .pow(u32::try_from(ROOT_CHOICE_ADVERSARIAL_FILLED_SLOTS).expect("三个槽"))
    {
        let mut remaining_choices = state_number;
        let mut roots_in_ring_order: Vec<Option<&RootChoicePublishMaterial>> =
            vec![None; ROOT_CHOICE_ADVERSARIAL_FILLED_SLOTS];
        for slot_index in (0..ROOT_CHOICE_ADVERSARIAL_FILLED_SLOTS).rev() {
            let choice = remaining_choices % choices_per_slot;
            remaining_choices /= choices_per_slot;
            if choice > 0 {
                roots_in_ring_order[slot_index] = Some(materials[choice - 1].as_ref());
            }
        }
        let newest_version = roots_in_ring_order
            .iter()
            .copied()
            .flatten()
            .map(|material| material.version)
            .max();
        if let (Some(chosen), Some(newest)) =
            (rule.chooses_among(&roots_in_ring_order), newest_version)
        {
            violations += u64::from(chosen.version < newest);
        }
    }
    violations
}

/// KA 上每种本地择根序的对照读数（KJ2）：必须等于 [`PINNED_KA_KJ2_OF_THE_LOCAL_RULES`]。
fn fault_ka_control_readings() -> Vec<FaultControlReading> {
    let place = fault_place_of(
        FaultDomain::AdversarialRootRing,
        RootChoiceCell {
            mounts: 0,
            user_publishes: 0,
        },
    );
    PINNED_KA_KJ2_OF_THE_LOCAL_RULES
        .iter()
        .map(|(rule, expected)| {
            fault_reading_that_must_equal(
                FaultCriterion::Kj2NotEarlierThanAnyReadableRoot,
                format!("{place} local_choice={}", rule.label()),
                ka_kj2_violations_of_a_local_rule(*rule),
                *expected,
            )
        })
        .collect()
}

// ── 产物行

/// 承诺内（历史罩着 ∧ 被判时罩着 ∧ 不是上限截掉）的违例数，从分格的行里算：罩着的格的违例数减去其中上限截掉那部分（登记 5.5 末段）。
fn promised_violations_from_the_split_rows(
    cells: &[FaultCellTally; FAULT_CELL_COUNT],
    criterion_index: usize,
) -> u64 {
    let mut promised = 0;
    for judged_fault in JudgedFault::ALL {
        if !judged_fault.is_inside_the_single_fault_promise() {
            continue;
        }
        let cell = &cells[fault_cell_index_of(*judged_fault, true)];
        promised += cell.violations[criterion_index] - cell.cut_violations[criterion_index];
    }
    promised
}

fn emit_the_rows_of_a_fault_cell(
    emitter: &mut Emitter,
    domain: FaultDomain,
    cell: RootChoiceCell,
    tally: &FaultTally,
) {
    let place = fault_place_of(domain, cell);
    emitter.result(&format!(
        "name=root_choice_fault_states {place} histories={} histories_with_an_uncovered_fault={} judged_states={} confirmed_states={} refused_mounts={} reread_mounts={} states_with_a_torn_system_slot={} same_txg_different_instance_roots={} higher_instance_lower_txg_roots={} states_without_a_readable_root={} above_newest_root_max={} in_flight_limit={} ideal_loss_by_limit={} ideal_loss_covered_other={} ideal_loss_uncovered={} promised_states={} confirmed_publishes_max={} largest_counter={} panicked={} choose_root_returned_none_with_a_readable_root={} choose_root_returned_none_without_a_readable_root={} replay_failed={} replay_applied={} kj1_and_kj1t_differing_states={} kj1_and_kj1t_differing_witness={} slot_readings_checked={} slot_readings_differing_from_the_truth={} slot_reading_witness={}",
        tally.histories,
        tally.histories_with_an_uncovered_fault,
        tally.judged_states,
        tally.confirmed_states,
        tally.refused_mounts,
        tally.reread_mounts,
        tally.states_with_a_torn_system_slot,
        tally.states_with_same_txg_different_instance_roots,
        tally.states_with_higher_instance_lower_txg_roots,
        tally.states_without_a_readable_root,
        tally.above_newest_root_maximum,
        domain.geometry().ring().local_in_flight_limit(),
        tally.ideal_loss_by_limit,
        tally.ideal_loss_covered_other,
        tally.ideal_loss_uncovered,
        tally.promised_states,
        tally.confirmed_publishes_maximum,
        tally.largest_counter,
        tally.panicked_states,
        tally.chose_nothing_with_a_readable_root,
        tally.chose_nothing_without_a_readable_root,
        tally.replay_failed_states,
        tally.replay_applied_states,
        tally.kj1_and_kj1t_differing_states,
        tally.kj1_and_kj1t_differing_witness.as_deref().unwrap_or("none"),
        tally.slot_readings_checked,
        tally.slot_readings_differing_from_the_truth,
        tally.slot_reading_witness.as_deref().unwrap_or("none"),
    ));
    let local_rule_fields: Vec<String> = FaultLocalRule::ALL
        .iter()
        .enumerate()
        .map(|(rule_index, rule)| {
            let reading = tally.local_rules[rule_index];
            format!(
                "{}={}/{}/{}",
                rule.label(),
                reading[0],
                reading[1],
                reading[2]
            )
        })
        .collect();
    emitter.result(&format!(
        "name=fake_outcomes {place} local_rules_kj1_kj1t_kj2:{} kf2_kj1_kj1t_kj2_kj3={}/{}/{}/{} kf4_kj1_kj1t={}/{} kf4_states_without_a_root_off_the_confirmed_chain={}",
        local_rule_fields.join(","),
        tally.kf2[0],
        tally.kf2[1],
        tally.kf2[2],
        tally.kf2[3],
        tally.kf4[0],
        tally.kf4[1],
        tally.kf4_states_without_a_root_off_the_confirmed_chain,
    ));
    emit_the_split_and_promised_rows(emitter, &place, tally);
    if cell.mounts == 2 {
        for (menu_index, label) in MOUNT_FAULT_MENU_LABELS.iter().enumerate() {
            let count = tally.mount_two[menu_index];
            emitter.result(&format!(
                "name=mount_two_fault {place} mount_two_fault={label} histories={} refused={} reread={}",
                count.histories, count.refused, count.reread
            ));
        }
    }
}

/// 每个 (被判时故障, 历史标签) 格每条判据一行，再加承诺内两行（KJ1、KJ1T，装置直接数的与从分格的行算的各报一遍）。
fn emit_the_split_and_promised_rows(emitter: &mut Emitter, place: &str, tally: &FaultTally) {
    for judged_fault in JudgedFault::ALL {
        for history_is_covered in [true, false] {
            let cell_index = fault_cell_index_of(*judged_fault, history_is_covered);
            let cell = &tally.cells[cell_index];
            for criterion in FaultCriterion::ALL {
                let criterion_index = criterion.index();
                let ideal_violations = if criterion_index == KJ1_INDEX {
                    cell.ideal_violations[0].to_string()
                } else if criterion_index == KJ1T_INDEX {
                    cell.ideal_violations[1].to_string()
                } else {
                    "not_applicable".to_string()
                };
                emitter.result(&format!(
                    "name=violations {place} judged_fault={} history={} criterion={} states={} confirmed_states={} cut_by_the_limit_states={} judged={} violations={} cut_by_the_limit_violations={} ideal_clause_violations={ideal_violations} witness={}",
                    judged_fault.label(),
                    if history_is_covered { "inside_the_promise" } else { "outside_the_promise" },
                    criterion.label(),
                    cell.states,
                    cell.confirmed_states,
                    cell.cut_states,
                    cell.judged[criterion_index],
                    cell.violations[criterion_index],
                    cell.cut_violations[criterion_index],
                    tally.witness_of_a_cell(cell_index, criterion_index)
                ));
            }
        }
    }
    for (position, criterion_index) in [KJ1_INDEX, KJ1T_INDEX].into_iter().enumerate() {
        emitter.result(&format!(
            "name=promised {place} criterion={} promised_states={} violations={} violations_from_the_split_rows={} witness={}",
            FaultCriterion::ALL[criterion_index].label(),
            tally.promised_states,
            tally.promised_violations[position],
            promised_violations_from_the_split_rows(&tally.cells, criterion_index),
            tally.promised_witnesses[position].as_deref().unwrap_or("none")
        ));
    }
}

/// 一个域扫完之后的判决行（各判据在这个域全部格上的合计），并记作废与停机。
fn emit_the_verdict_of_a_fault_domain(
    emitter: &mut Emitter,
    findings: &mut RunFindings,
    domain: FaultDomain,
    cells: &[RootChoiceCell],
    total: &FaultTally,
    tier: BoundsTier,
) {
    let largest_mounts = cells.iter().map(|cell| cell.mounts).max().unwrap_or(0);
    let largest_user_publishes = cells
        .iter()
        .map(|cell| cell.user_publishes)
        .max()
        .unwrap_or(0);
    // 登记作废条款 TV10：域是空的（没有一个已确认发布的态；KG 上实例代号更高而 txg 更低同在为 0）。
    let domain_is_nonempty = total.confirmed_states > 0
        && (domain != FaultDomain::GateOff
            || total.states_with_higher_instance_lower_txg_roots > 0);
    if !domain_is_nonempty {
        findings.void_conditions.push(format!(
            "TV10 {} 域是空的（没有一个已确认发布的态，或 KG 上实例代号更高而 txg 更低同在为 0），这个域的 0 违例不算数",
            domain.label()
        ));
    }
    // 登记作废条款 TV6：承诺内那一行对不回分格的行。
    let promised_rows_equal_the_split_rows =
        [KJ1_INDEX, KJ1T_INDEX]
            .into_iter()
            .enumerate()
            .all(|(position, criterion_index)| {
                total.promised_violations[position]
                    == promised_violations_from_the_split_rows(&total.cells, criterion_index)
            });
    if !promised_rows_equal_the_split_rows {
        findings.void_conditions.push(format!(
            "TV6 {} 域里「承诺内」那一行的违例数对不回分格的行",
            domain.label()
        ));
    }
    // 登记停机条款 TH4：读槽对拍不一致，不当 KJ 的违例记。
    if total.slot_readings_differing_from_the_truth > 0 {
        findings.halt_conditions.push(format!(
            "TH4 {} 域里 read_root_ring_slot 交回的与生成时的真相不同的槽数 {}（见证 {}）",
            domain.label(),
            total.slot_readings_differing_from_the_truth,
            total.slot_reading_witness.as_deref().unwrap_or("none")
        ));
    }
    // 登记第七节甲类、失败条款 TF9：合法历史的域里今天的 replay_journal 报错的被判态数是 0。
    if domain.is_a_legal_history() && total.replay_failed_states > 0 {
        findings.halt_conditions.push(format!(
            "TF9 {} 域里 replay_journal 报错的被判态数 {}（登记第七节甲类锚点是 0）",
            domain.label(),
            total.replay_failed_states
        ));
    }
    emitter.result(&format!(
        "name=verdict domain={} ring_slots={} slots_per_region={} largest_mounts={largest_mounts} largest_user_publishes={largest_user_publishes} bounds={} criteria=root_choice_faults histories={} states={} confirmed_states={} kj0_violations={} panicked={} choose_root_returned_none_with_a_readable_root={} replay_failed={} kj1_violations={} kj1_judged={} kj1t_violations={} kj1t_judged={} kj2_violations={} kj2_judged={} kj3_violations={} kj3_judged={} promised_states={} promised_kj1_violations={} promised_kj1t_violations={} promised_rows_equal_the_split_rows={promised_rows_equal_the_split_rows} kj1_and_kj1t_differing_states={} slot_readings_differing_from_the_truth={} domain_is_nonempty={domain_is_nonempty}",
        domain.label(),
        domain.geometry().ring().ring_slots,
        domain.geometry().slots_per_region(),
        tier.label(),
        total.histories,
        total.judged_states,
        total.confirmed_states,
        total.violations_of_a_criterion(0),
        total.panicked_states,
        total.chose_nothing_with_a_readable_root,
        total.replay_failed_states,
        total.violations_of_a_criterion(KJ1_INDEX),
        total.judged_of_a_criterion(KJ1_INDEX),
        total.violations_of_a_criterion(KJ1T_INDEX),
        total.judged_of_a_criterion(KJ1T_INDEX),
        total.violations_of_a_criterion(KJ2_INDEX),
        total.judged_of_a_criterion(KJ2_INDEX),
        total.violations_of_a_criterion(KJ3_INDEX),
        total.judged_of_a_criterion(KJ3_INDEX),
        total.promised_states,
        total.promised_violations[0],
        total.promised_violations[1],
        total.kj1_and_kj1t_differing_states,
        total.slot_readings_differing_from_the_truth,
    ));
}

// ── 本地常量回比（登记第三节回比表，r4 新读的三样；不符走停机条款 TH2）

fn cross_check_root_choice_fault_local_constants_against_the_implementation() {
    cross_check_root_choice_local_constants_against_the_implementation();
    assert_eq!(
        u64::try_from(FAULT_SYSTEM_SLOTS_PER_DEVICE).expect("2 装得进 u64"),
        singlefs_format::SYSTEM_CONFIGURATION_SLOTS_PER_DEVICE,
        "TH2：每块盘的系统配置槽数"
    );
    assert_eq!(
        FAULT_MKFS_SYSTEM_GENERATION,
        singlefs_core::make_filesystem::SYSTEM_CONFIGURATION_GENERATION_AT_MKFS,
        "TH2：mkfs 写下的系统配置世代号"
    );
    assert_eq!(
        FAULT_WRITABLE_DEVICE_COUNT_LOWER_BOUND,
        singlefs_core::mount::STRIPE_WIDTH_LOWER_BOUND.0,
        "TH2：可写挂载的设备数下限"
    );
    const {
        assert!(
            FAULT_DEVICE_COUNT >= FAULT_WRITABLE_DEVICE_COUNT_LOWER_BOUND,
            "TH2：两块盘装得下可写挂载的设备数下限（历史里的可写挂载在盘上都不是空操作）"
        );
    }
    assert_eq!(
        FAULT_SYSTEM_SLOT_COUNT_OF_THE_POOL,
        FAULT_DEVICE_COUNT * FAULT_SYSTEM_SLOTS_PER_DEVICE,
        "整池的系统配置槽数"
    );
}

// ── 命令行与主流程（`--family root_choice_faults`，只在 `--profile r4`）

struct FaultSelection {
    bounds: BoundsTier,
    only: Option<Vec<String>>,
    largest_user_publishes_cap: Option<usize>,
    /// `--gate r2` 时 R2，不给时取甲（登记 r5 修订 1）。
    witness_gate_rule: WitnessGateRule,
}

fn arguments_name_the_root_choice_faults_family(arguments: &[String]) -> bool {
    arguments
        .windows(2)
        .any(|pair| pair[0] == "--family" && pair[1] == FAULT_FAMILY_LABEL)
}

/// `--bounds segment1|segment2|light`（这一族 segment1 与 segment2 同界、`light` 是调试用的小界）、`--family root_choice_faults`、
/// `--only KF,KF3,KF9,KFW,KG,KA`、`--max-u <U>`（每个格的 U 封顶，只给调试用）。
fn parse_the_root_choice_faults_command_line(
    arguments: &[String],
) -> Result<FaultSelection, String> {
    let mut bounds: Option<BoundsTier> = None;
    let mut only: Option<Vec<String>> = None;
    let mut largest_user_publishes_cap: Option<usize> = None;
    let mut witness_gate_rule = WitnessGateRule::OneUnreadableOrTornSlotMeansUndecidable;
    let mut position = 0;
    while position < arguments.len() {
        let option = arguments[position].as_str();
        let value = arguments
            .get(position + 1)
            .ok_or_else(|| format!("{option} 后面缺一个值"))?
            .as_str();
        match option {
            "--bounds" => {
                bounds = Some(match value {
                    "segment1" => BoundsTier::SegmentOne,
                    "segment2" => BoundsTier::SegmentTwo,
                    "light" => BoundsTier::Light,
                    unknown_tier => return Err(format!("不认识的界：{unknown_tier}")),
                });
            }
            "--gate" => {
                witness_gate_rule = match value {
                    "r2" => {
                        WitnessGateRule::SomeSlotReportsAReadErrorAndNoDeviceSelfVerifiedBothSlots
                    }
                    unknown_gate => {
                        return Err(format!(
                        "不认识的闸：{unknown_gate}（只认 r2；不给 --gate 是取甲，登记 r5 修订 1）"
                    ))
                    }
                };
            }
            "--family" => {
                if value != FAULT_FAMILY_LABEL {
                    return Err(format!("不认识的族：{value}"));
                }
            }
            "--only" => {
                let names: Vec<String> = value.split(',').map(str::to_string).collect();
                for name in &names {
                    if !FaultDomain::ALL.iter().any(|domain| domain.label() == name) {
                        return Err(format!(
                            "不认识的域：{name}（这一族的域是 KF、KF3、KF9、KFW、KG、KA）"
                        ));
                    }
                }
                only = Some(names);
            }
            "--max-u" => {
                largest_user_publishes_cap = Some(
                    value
                        .parse::<usize>()
                        .map_err(|error| format!("--max-u 要一个非负整数：{error}"))?,
                );
            }
            unknown_option => return Err(format!("不认识的参数：{unknown_option}")),
        }
        position += 2;
    }
    Ok(FaultSelection {
        bounds: bounds.ok_or_else(|| "缺 --bounds".to_string())?,
        only,
        largest_user_publishes_cap,
        witness_gate_rule,
    })
}

/// 跑中途收集的、收尾要用的几样读数（判别力自证、C331 的判别力自证）。
#[derive(Default)]
struct FaultRunWitnesses {
    mount_two_by_domain: Vec<(FaultDomain, FaultTally)>,
    /// KF 的 M = 2 每一格（U, 取号那一刻因「某块盘一份自证过的都没有」被拒的挂载数），R2 的判别力自证第 3 条用。
    acquisition_refusals_without_a_self_verified_slot_on_kf_two_mounts: Vec<(usize, u64)>,
    torn_states_of_the_gate_on_domain_at_two_mounts_without_user_publishes: Option<u64>,
    kg_kf4_at_two_mounts_and_three_user_publishes: Option<[u64; 2]>,
    kg_local_today_kj1t_without_a_judged_fault: u64,
    kg_implementation_kj1t_without_a_judged_fault: u64,
    kg_implementation_kj3_without_a_judged_fault: u64,
    kg_was_run: bool,
}

fn emit_the_control_rows(
    emitter: &mut Emitter,
    findings: &mut RunFindings,
    readings: &[FaultControlReading],
) {
    for reading in readings {
        emitter.result(&format!(
            "name=control criterion={} measured={} value={} must_see={} red_as_required={}",
            reading.criterion.label(),
            reading.measured_where.replace(' ', ","),
            reading.value,
            reading.must_see,
            reading.is_red_as_required
        ));
        if !reading.is_red_as_required {
            findings.void_conditions.push(format!(
                "TV3 判据 {} 的阳性对照没红：{} 量到 {}，必须 {}",
                reading.criterion.label(),
                reading.measured_where,
                reading.value,
                reading.must_see
            ));
        }
    }
}

/// 登记第八节「判别力自证」四条与 C331 的判别力自证（5.6）：读数都来自这一次跑出的格，不另跑。
/// 挂载 2 开头那项故障在菜单里的下标。
fn mount_fault_menu_index_of(label: &str) -> usize {
    MOUNT_FAULT_MENU_LABELS
        .iter()
        .position(|candidate| *candidate == label)
        .expect("故障名在菜单之内")
}

/// 取甲的判别力自证第 1 条（r4 登记 5.6）：KF 的 M = 2 各格合起来，SN0-TP、SN1-TP、SB-TP、RNSB-TP 四项每一项的历史全被拒，KG 同四项一个不拒。
fn gate_refusal_proof_under_the_implementation_choice(
    gate_on: &FaultTally,
    gate_off: &FaultTally,
) -> (String, bool, String) {
    let mut gate_refusal_proof_passed = true;
    let mut gate_refusal_proof_detail = Vec::new();
    for label in ["SN0-TP", "SN1-TP", "SB-TP", "RNSB-TP"] {
        let with_the_gate = gate_on.mount_two[mount_fault_menu_index_of(label)];
        let without_the_gate = gate_off.mount_two[mount_fault_menu_index_of(label)];
        gate_refusal_proof_passed &= with_the_gate.histories > 0
            && with_the_gate.refused == with_the_gate.histories
            && without_the_gate.refused == 0;
        gate_refusal_proof_detail.push(format!(
            "{label}:KF_histories={}_KF_refused={}_KG_refused={}",
            with_the_gate.histories, with_the_gate.refused, without_the_gate.refused
        ));
    }
    (
        "gate_refuses_every_history_of_the_four_tp_faults_and_the_gate_off_domain_refuses_none"
            .to_string(),
        gate_refusal_proof_passed,
        gate_refusal_proof_detail.join(","),
    )
}

/// R2 的判别力自证第 1 条（r5 登记修订 1 修1.八）：KF 的 M = 2 各格合起来，SN0-TP、SN1-TP 两项有历史、有被拒、而不是全拒（取甲下全拒）；
/// SB-TP、RNSB-TP 两项历史全被拒；KG 同四项一个不拒。把门槛挪到两点之间（「SN0-TP 的历史全被拒」）：取甲那一点为真、R2 这一点为假。
fn gate_refusal_proof_under_r2(
    gate_on: &FaultTally,
    gate_off: &FaultTally,
) -> (String, bool, String) {
    let mut gate_refusal_proof_passed = true;
    let mut gate_refusal_proof_detail = Vec::new();
    for (label, every_history_is_refused) in [
        ("SN0-TP", false),
        ("SN1-TP", false),
        ("SB-TP", true),
        ("RNSB-TP", true),
    ] {
        let with_the_gate = gate_on.mount_two[mount_fault_menu_index_of(label)];
        let without_the_gate = gate_off.mount_two[mount_fault_menu_index_of(label)];
        let refusal_matches_the_rule = if every_history_is_refused {
            with_the_gate.refused == with_the_gate.histories
        } else {
            with_the_gate.refused > 0 && with_the_gate.refused < with_the_gate.histories
        };
        gate_refusal_proof_passed &= with_the_gate.histories > 0
            && refusal_matches_the_rule
            && without_the_gate.refused == 0;
        gate_refusal_proof_detail.push(format!(
            "{label}:KF_histories={}_KF_refused={}_KG_refused={}",
            with_the_gate.histories, with_the_gate.refused, without_the_gate.refused
        ));
    }
    (
        "r2_gate_lets_part_of_the_sn_tp_histories_through_refuses_every_sb_and_rnsb_tp_history_and_the_gate_off_domain_refuses_none"
            .to_string(),
        gate_refusal_proof_passed,
        gate_refusal_proof_detail.join(","),
    )
}

/// 判别力自证第 1 条照闸的取法选（取甲那一条与 R2 那一条判的是相反的两点）。
fn gate_refusal_proof_of(
    witness_gate_rule: WitnessGateRule,
    gate_on: &FaultTally,
    gate_off: &FaultTally,
) -> (String, bool, String) {
    match witness_gate_rule {
        WitnessGateRule::OneUnreadableOrTornSlotMeansUndecidable => {
            gate_refusal_proof_under_the_implementation_choice(gate_on, gate_off)
        }
        WitnessGateRule::SomeSlotReportsAReadErrorAndNoDeviceSelfVerifiedBothSlots => {
            gate_refusal_proof_under_r2(gate_on, gate_off)
        }
    }
}

/// 判别力自证第 2 条（T1 生效）：取甲照 r4 登记 5.6（SN0-T1、SN1-T1、SB-T1 走第二遍的挂载数都不是 0）；R2 照 r5 登记修订 1 修1.八
/// （SB-T1、RNSB-T1 走第二遍的挂载数等于那一项的历史数，SN0-T1、SN1-T1 不是 0）。
fn second_read_proof_of(
    witness_gate_rule: WitnessGateRule,
    gate_on: &FaultTally,
) -> (String, bool, String) {
    let mut second_read_proof_passed = true;
    let mut second_read_proof_detail = Vec::new();
    match witness_gate_rule {
        WitnessGateRule::OneUnreadableOrTornSlotMeansUndecidable => {
            for label in ["SN0-T1", "SN1-T1", "SB-T1"] {
                let with_the_gate = gate_on.mount_two[mount_fault_menu_index_of(label)];
                second_read_proof_passed &= with_the_gate.reread > 0;
                second_read_proof_detail
                    .push(format!("{label}:KF_reread={}", with_the_gate.reread));
            }
            (
                "t1_takes_the_second_read_in_the_gate_on_domain".to_string(),
                second_read_proof_passed,
                second_read_proof_detail.join(","),
            )
        }
        WitnessGateRule::SomeSlotReportsAReadErrorAndNoDeviceSelfVerifiedBothSlots => {
            for (label, every_history_takes_the_second_read) in [
                ("SN0-T1", false),
                ("SN1-T1", false),
                ("SB-T1", true),
                ("RNSB-T1", true),
            ] {
                let with_the_gate = gate_on.mount_two[mount_fault_menu_index_of(label)];
                second_read_proof_passed &= with_the_gate.reread > 0
                    && (!every_history_takes_the_second_read
                        || with_the_gate.reread == with_the_gate.histories);
                second_read_proof_detail.push(format!(
                    "{label}:KF_histories={}_KF_reread={}",
                    with_the_gate.histories, with_the_gate.reread
                ));
            }
            (
                "r2_t1_takes_the_second_read_on_every_sb_and_rnsb_t1_history_and_on_some_sn_t1_histories"
                    .to_string(),
                second_read_proof_passed,
                second_read_proof_detail.join(","),
            )
        }
    }
}

fn emit_the_self_proofs(
    emitter: &mut Emitter,
    findings: &mut RunFindings,
    witnesses: &FaultRunWitnesses,
    witness_gate_rule: WitnessGateRule,
) {
    let mount_two_of = |domain: FaultDomain| {
        witnesses
            .mount_two_by_domain
            .iter()
            .find(|(candidate, _tally)| *candidate == domain)
            .map(|(_domain, tally)| tally)
    };
    let mut proofs: Vec<(String, bool, String)> = Vec::new();
    if let (Some(gate_on), Some(gate_off)) = (
        mount_two_of(FaultDomain::GateOn),
        mount_two_of(FaultDomain::GateOff),
    ) {
        proofs.push(gate_refusal_proof_of(witness_gate_rule, gate_on, gate_off));
        proofs.push(second_read_proof_of(witness_gate_rule, gate_on));
    }
    // R2 的判别力自证第 3 条（r5 登记修订 1 修1.八）：取号那一刻第二道拒（某块盘一份自证过的都没有）在 KF 的 M = 2 每一格都拒过挂载。
    match witness_gate_rule {
        WitnessGateRule::OneUnreadableOrTornSlotMeansUndecidable => {}
        WitnessGateRule::SomeSlotReportsAReadErrorAndNoDeviceSelfVerifiedBothSlots => {
            let cells =
                &witnesses.acquisition_refusals_without_a_self_verified_slot_on_kf_two_mounts;
            if !cells.is_empty() {
                proofs.push((
                    "r2_acquisition_refuses_a_device_without_a_self_verified_slot_on_every_kf_two_mount_cell"
                        .to_string(),
                    cells.iter().all(|(_user_publishes, refused)| *refused > 0),
                    cells
                        .iter()
                        .map(|(user_publishes, refused)| format!("KF_M2_U{user_publishes}:refused={refused}"))
                        .collect::<Vec<String>>()
                        .join(","),
                ));
            }
        }
    }
    if let Some([kj1, kj1t]) = witnesses.kg_kf4_at_two_mounts_and_three_user_publishes {
        proofs.push((
            "kf4_parts_kj1_from_kj1t_on_the_gate_off_domain".to_string(),
            kj1 != kj1t,
            format!("KF4_kj1={kj1},KF4_kj1t={kj1t}"),
        ));
    }
    if let Some(torn) =
        witnesses.torn_states_of_the_gate_on_domain_at_two_mounts_without_user_publishes
    {
        proofs.push((
            "torn_third_state_is_enumerated".to_string(),
            torn > 0,
            format!("KF_M2_U0_states_with_a_torn_system_slot={torn}"),
        ));
    }
    for (item, passed, detail) in &proofs {
        emitter.result(&format!(
            "name=self_proof item={item} passed={passed} detail={detail}"
        ));
        if !passed {
            findings
                .void_conditions
                .push(format!("TV3 判别力自证没过：{item}（{detail}）"));
        }
    }
    if witnesses.kg_was_run {
        emitter.result(&format!(
            "name=c331_proof domain=KG local_today_kj1t_without_a_judged_fault={} pinned={PINNED_KG_TODAY_KJ1T_WITHOUT_A_JUDGED_FAULT} implementation_kj1t_without_a_judged_fault={} implementation_kj3_without_a_judged_fault={}",
            witnesses.kg_local_today_kj1t_without_a_judged_fault,
            witnesses.kg_implementation_kj1t_without_a_judged_fault,
            witnesses.kg_implementation_kj3_without_a_judged_fault,
        ));
        if witnesses
            .kg_kf4_at_two_mounts_and_three_user_publishes
            .is_some()
            && witnesses.kg_local_today_kj1t_without_a_judged_fault == 0
        {
            findings.void_conditions.push(
                "TV3 KG 没造出 C331 那一形：本地理想恢复在被判时无故障的态上 KJ1T 合计为 0（只作废 KG 那一行与依它的「拿掉保护会丢」）"
                    .to_string(),
            );
        }
    }
}

/// KA（同 r3）：三步在 125 个对抗根环上，行与判决用 r3 的写法；对照是本地写的六种择根序。
fn run_the_adversarial_domain_of_the_faults_family(
    emitter: &mut Emitter,
    findings: &mut RunFindings,
    control_readings: &mut Vec<FaultControlReading>,
) -> u64 {
    let geometry = RootChoiceDomain::AdversarialRootRing;
    let cell = RootChoiceCell {
        mounts: 0,
        user_publishes: 0,
    };
    let tally =
        root_choice_tally_of_the_adversarial_root_rings(RootChoiceImplementation::TodayThreeSteps);
    assert_eq!(
        tally.get(RootChoiceCounter::States),
        PINNED_ROOT_CHOICE_ADVERSARIAL_STATES,
        "TV4：KA 状态数"
    );
    assert_eq!(
        tally.get(RootChoiceCounter::StatesWithoutASelfVerifiedRoot),
        1,
        "TV4：KA 里根环全空的态数（登记第七节乙类 KA：没有一条自证过的根的 1 态）"
    );
    emit_the_rows_of_a_root_choice_cell(emitter, geometry, cell, &tally);
    let readings = fault_ka_control_readings();
    emit_the_control_rows(emitter, findings, &readings);
    control_readings.extend(readings);
    emit_the_verdict_of_a_root_choice_domain(emitter, findings, geometry, &[cell], &tally);
    tally.get(RootChoiceCounter::States)
}

fn run_the_root_choice_faults_family(arguments: &[String]) {
    let selection = match parse_the_root_choice_faults_command_line(arguments) {
        Ok(selection) => selection,
        Err(problem) => {
            eprintln!("  ✗ {problem}");
            eprintln!("  → 怎么办：{USAGE}");
            std::process::exit(2);
        }
    };
    let thread_count = thread_count_from_the_environment();
    install_the_panic_hook_that_stays_quiet_inside_the_replay_call();
    let run_started = Instant::now();
    let mut emitter = Emitter { emitted: 0 };
    let mut findings = RunFindings::default();
    let selected_cells: Vec<(FaultDomain, Vec<RootChoiceCell>)> = FaultDomain::ALL
        .iter()
        .copied()
        .filter(|domain| {
            selection
                .only
                .as_ref()
                .is_none_or(|names| names.iter().any(|name| name == domain.label()))
        })
        .map(|domain| {
            (
                domain,
                fault_cells_of(
                    domain,
                    selection.bounds,
                    selection.largest_user_publishes_cap,
                ),
            )
        })
        .collect();
    emitter.result(&format!(
        "name=config experiment=E165 bounds={} family={FAULT_FAMILY_LABEL} only={} max_u={} root_slot_bytes={ROOT_CHOICE_ROOT_SLOT_BYTES} region_devices=0,1,0 system_slots_per_device={FAULT_SYSTEM_SLOTS_PER_DEVICE} witness_gate={} profile=r4",
        selection.bounds.label(),
        selection
            .only
            .as_ref()
            .map_or_else(|| "every_domain".to_string(), |names| names.join(",")),
        selection
            .largest_user_publishes_cap
            .map_or_else(|| "none".to_string(), |cap| cap.to_string()),
        selection.witness_gate_rule.label(),
    ));
    // ① 常量回比断言（不过当场 panic，进程以非 0 退出，一行结果都不算）。
    cross_check_root_choice_fault_local_constants_against_the_implementation();
    emitter.result(
        "name=self_check check=root_choice_fault_local_constants_equal_the_implementation passed=true",
    );
    let self_checks_passed = 1;
    for ring in [RingGeometry::BIG, RingGeometry::SMALL_NINE] {
        emitter.result(&format!(
            "name=in_flight_limit ring_slots={} local={} implementation={}",
            ring.ring_slots,
            ring.local_in_flight_limit(),
            journal_in_flight_record_limit(ring.ring_bytes())
        ));
    }
    let mut witnesses = FaultRunWitnesses::default();
    let mut control_readings: Vec<FaultControlReading> = Vec::new();
    let mut total_states: u64 = 0;
    let the_whole_registered_bound_is_run = selection.largest_user_publishes_cap.is_none()
        && matches!(
            selection.bounds,
            BoundsTier::SegmentOne | BoundsTier::SegmentTwo
        );
    for (domain, cells) in &selected_cells {
        if !domain.has_histories() {
            total_states += run_the_adversarial_domain_of_the_faults_family(
                &mut emitter,
                &mut findings,
                &mut control_readings,
            );
            continue;
        }
        let mut domain_total = FaultTally::new();
        let mut mount_two_merged = FaultTally::new();
        for cell in cells {
            let cell_started = Instant::now();
            let tally = fault_tally_of_a_cell_under_the_gate(
                *domain,
                *cell,
                selection.witness_gate_rule,
                true,
                thread_count,
            );
            findings
                .halt_conditions
                .extend(assert_the_pinned_fault_cell_under_the_gate(
                    selection.witness_gate_rule,
                    *domain,
                    *cell,
                    &tally,
                ));
            assert_eq!(
                tally.chose_nothing_without_a_readable_root,
                pinned_fault_cell_under_the_gate(selection.witness_gate_rule, *domain, *cell)
                    .expect("每个跑到的格都在登记第七节乙类的表里")
                    .states_without_a_readable_root,
                "TV4：{} 读视图里一条根都没有的被判态里 choose_root 交回空的态数",
                fault_place_of(*domain, *cell)
            );
            emit_the_rows_of_a_fault_cell(&mut emitter, *domain, *cell, &tally);
            match selection.witness_gate_rule {
                WitnessGateRule::OneUnreadableOrTornSlotMeansUndecidable => {}
                WitnessGateRule::SomeSlotReportsAReadErrorAndNoDeviceSelfVerifiedBothSlots => {
                    emitter.result(&refusal_row_of_a_fault_cell(*domain, *cell, &tally));
                }
            }
            if *domain == FaultDomain::GateOn && cell.mounts == 2 {
                witnesses
                    .acquisition_refusals_without_a_self_verified_slot_on_kf_two_mounts
                    .push((
                        cell.user_publishes,
                        tally.refusals_by_reason
                            [MountRefusalReason::AcquisitionDeviceWithoutASelfVerifiedSlot.index()],
                    ));
            }
            let readings = fault_control_readings_of_a_cell_under_the_gate(
                selection.witness_gate_rule,
                *domain,
                *cell,
                &tally,
            );
            emit_the_control_rows(&mut emitter, &mut findings, &readings);
            control_readings.extend(readings);
            println!(
                "E7TIMING name=elapsed scope=cell domain={} mounts={} user_publishes={} states={} elapsed_seconds={:.3} threads={thread_count}",
                domain.label(),
                cell.mounts,
                cell.user_publishes,
                tally.judged_states,
                cell_started.elapsed().as_secs_f64()
            );
            if *domain == FaultDomain::GateOn && cell.mounts == 2 && cell.user_publishes == 0 {
                witnesses.torn_states_of_the_gate_on_domain_at_two_mounts_without_user_publishes =
                    Some(tally.states_with_a_torn_system_slot);
            }
            if *domain == FaultDomain::GateOff {
                witnesses.kg_was_run = true;
                witnesses.kg_local_today_kj1t_without_a_judged_fault +=
                    tally.local_kj1t_by_judged_fault[0][JudgedFault::Nothing.index()];
                for history_is_covered in [true, false] {
                    let none_cell =
                        &tally.cells[fault_cell_index_of(JudgedFault::Nothing, history_is_covered)];
                    witnesses.kg_implementation_kj1t_without_a_judged_fault +=
                        none_cell.violations[KJ1T_INDEX];
                    witnesses.kg_implementation_kj3_without_a_judged_fault +=
                        none_cell.violations[KJ3_INDEX];
                }
                if cell.mounts == 2 && cell.user_publishes == 3 {
                    witnesses.kg_kf4_at_two_mounts_and_three_user_publishes = Some(tally.kf4);
                }
            }
            domain_total.merge(&tally);
            if cell.mounts == 2 {
                mount_two_merged.merge(&tally);
            }
        }
        if the_whole_registered_bound_is_run && domain.largest_mounts(selection.bounds) >= 2 {
            assert_the_pinned_mount_two_table_under_the_gate(
                selection.witness_gate_rule,
                *domain,
                &mount_two_merged,
            );
        }
        if cells.iter().any(|cell| cell.mounts == 2) {
            witnesses
                .mount_two_by_domain
                .push((*domain, mount_two_merged));
        }
        total_states += domain_total.judged_states;
        emit_the_verdict_of_a_fault_domain(
            &mut emitter,
            &mut findings,
            *domain,
            cells,
            &domain_total,
            selection.bounds,
        );
    }
    emit_the_self_proofs(
        &mut emitter,
        &mut findings,
        &witnesses,
        selection.witness_gate_rule,
    );
    for condition in &findings.void_conditions {
        emitter.result(&format!(
            "name=void condition={}",
            condition.replace(' ', "_")
        ));
    }
    for condition in &findings.halt_conditions {
        emitter.result(&format!(
            "name=halt condition={}",
            condition.replace(' ', "_")
        ));
    }
    emitter.result(&format!(
        "name=verdict scope=run bounds={} family={FAULT_FAMILY_LABEL} states={total_states} self_checks_passed={self_checks_passed} control_readings={} controls_not_red={} void_conditions={} halt_conditions={} profile=r4{}",
        selection.bounds.label(),
        control_readings.len(),
        control_readings
            .iter()
            .filter(|reading| !reading.is_red_as_required)
            .count(),
        findings.void_conditions.len(),
        findings.halt_conditions.len(),
        match selection.witness_gate_rule {
            WitnessGateRule::OneUnreadableOrTornSlotMeansUndecidable => String::new(),
            WitnessGateRule::SomeSlotReportsAReadErrorAndNoDeviceSelfVerifiedBothSlots => {
                format!(" witness_gate={}", selection.witness_gate_rule.label())
            }
        },
    ));
    println!(
        "E7TIMING name=elapsed scope=run states={total_states} elapsed_seconds={:.3} threads={thread_count}",
        run_started.elapsed().as_secs_f64()
    );
    emitter.done();
    if !findings.void_conditions.is_empty() || !findings.halt_conditions.is_empty() {
        std::process::exit(1);
    }
}

// ── 命令行与主流程

struct RunSelection {
    bounds: BoundsTier,
    only: Option<Vec<String>>,
    largest_size_cap: Option<usize>,
    families: Vec<Family>,
    family_label: &'static str,
}

const USAGE: &str = "用法：e165_replay_bounded_enumeration --bounds segment1|segment2|light [--family legal|adversarial|all|multi_disk|root_choice|root_choice_faults] [--only <域>[,<域>…]] [--max-n <N>] [--profile r1|r2|r3|r4|r5] [--marker-parity even|odd] [--table direct|scan|both] [--disk-order normal|reversed]（档不给取 r1；奇偶、建表法、盘次序只在 r2、r3、r5 档有，不给分别取 even、direct、normal；这几个参数写在哪都行，单状态重放、逐态判定行、钉住的红例逐行重放同样认）；域名：L L6 L9 U C C6 A A6 AC X E1 E2 R M M6 MC（多盘三个域只在 --family multi_disk 时跑、要 --profile r2；all 只含合法历史与对抗两族；--family root_choice 只在 --profile r3 时跑，域 K K9 KW KA，认 --bounds、--only、--max-u；--family root_choice_faults 只在 --profile r4 时跑，域 KF KF3 KF9 KFW KG KA，认 --bounds、--only、--max-u、--gate r2（不给是取甲，r5 登记修订 1）；--profile r4 其余族与 r3 档相同；--profile r5 认 legal、adversarial、all、multi_disk，不认 root_choice、root_choice_faults，其余同 r3 档、只把 J7 换成 J8）。单状态重放：--replay-state <域>:<N>:<序号>；逐态判定行：--list-states <域>:<N>（这两种的域名用产物里 domain= 的写法，U 域是 U_verify_on、U_verify_off）；钉住的红例逐行重放：--pinned-cases <表>（格式见 .claude/gate.d/e165-pinned-red-examples.tsv 文件头）";

fn parse_the_command_line(arguments: &[String]) -> Result<RunSelection, String> {
    let mut bounds: Option<BoundsTier> = None;
    let mut only: Option<Vec<String>> = None;
    let mut largest_size_cap: Option<usize> = None;
    let mut families = vec![Family::LegalHistory, Family::Adversarial];
    let mut family_label = "all";
    let mut position = 0;
    while position < arguments.len() {
        let option = arguments[position].as_str();
        let value = arguments
            .get(position + 1)
            .ok_or_else(|| format!("{option} 后面缺一个值"))?
            .as_str();
        match option {
            "--bounds" => {
                bounds = Some(match value {
                    "segment1" => BoundsTier::SegmentOne,
                    "segment2" => BoundsTier::SegmentTwo,
                    "light" => BoundsTier::Light,
                    unknown_tier => return Err(format!("不认识的界：{unknown_tier}")),
                });
            }
            "--family" => match value {
                "legal" => {
                    families = vec![Family::LegalHistory];
                    family_label = "legal";
                }
                "adversarial" => {
                    families = vec![Family::Adversarial];
                    family_label = "adversarial";
                }
                "multi_disk" => {
                    families = vec![Family::MultiDisk];
                    family_label = "multi_disk";
                }
                "all" => {}
                unknown_family => return Err(format!("不认识的族：{unknown_family}")),
            },
            "--only" => {
                let names: Vec<String> = value.split(',').map(str::to_string).collect();
                for name in &names {
                    if !Domain::ALL
                        .iter()
                        .any(|domain| domain.selection_name() == name)
                    {
                        return Err(format!("不认识的域：{name}"));
                    }
                }
                only = Some(names);
            }
            "--max-n" => {
                largest_size_cap = Some(
                    value
                        .parse::<usize>()
                        .map_err(|error| format!("--max-n 要一个非负整数：{error}"))?,
                );
            }
            unknown_option => return Err(format!("不认识的参数：{unknown_option}")),
        }
        position += 2;
    }
    Ok(RunSelection {
        bounds: bounds.ok_or_else(|| "缺 --bounds".to_string())?,
        only,
        largest_size_cap,
        families,
        family_label,
    })
}

/// 从参数里取出 `--profile` 与 `--marker-parity`（写在哪都行），交回档与其余参数（登记 r2 5.1、第八节）。
/// 档不给取 [`DEFAULT_PROFILE_LABEL`]；`--marker-parity` 只在 r2 档有。
fn split_off_the_profile_options(
    arguments: &[String],
) -> Result<(RunProfile, Vec<String>), String> {
    let mut profile_label: Option<String> = None;
    let mut marker_parity_label: Option<String> = None;
    let mut table_source_label: Option<String> = None;
    let mut disk_order_label: Option<String> = None;
    let mut remaining_arguments = Vec::new();
    let mut position = 0;
    while position < arguments.len() {
        let argument = &arguments[position];
        let destination = match argument.as_str() {
            "--profile" => Some(&mut profile_label),
            "--marker-parity" => Some(&mut marker_parity_label),
            "--table" => Some(&mut table_source_label),
            "--disk-order" => Some(&mut disk_order_label),
            _other_argument => None,
        };
        let Some(destination) = destination else {
            remaining_arguments.push(argument.clone());
            position += 1;
            continue;
        };
        let value = arguments
            .get(position + 1)
            .ok_or_else(|| format!("{argument} 后面缺一个值"))?;
        if destination.replace(value.clone()).is_some() {
            return Err(format!("{argument} 给了不止一次"));
        }
        position += 2;
    }
    let marker_parity = match marker_parity_label.as_deref() {
        None | Some("even") => MarkerParity::EvenVersionsWrittenByTheUnmountSequence,
        Some("odd") => MarkerParity::OddVersionsWrittenByTheUnmountSequence,
        Some(unknown_parity) => {
            return Err(format!(
                "不认识的卸载记号奇偶：{unknown_parity}（只认 even、odd）"
            ))
        }
    };
    let table_source = match table_source_label.as_deref() {
        None | Some("direct") => RecordTableSource::Direct,
        Some("scan") => RecordTableSource::Scanned,
        Some("both") => RecordTableSource::Both,
        Some(unknown_table_source) => {
            return Err(format!(
                "不认识的建表法：{unknown_table_source}（只认 direct、scan、both）"
            ))
        }
    };
    let reported_disk_order = match disk_order_label.as_deref() {
        None | Some("normal") => ReportedDiskOrder::FirstDeviceBeforeSecond,
        Some("reversed") => ReportedDiskOrder::SecondDeviceBeforeFirst,
        Some(unknown_disk_order) => {
            return Err(format!(
                "不认识的盘次序：{unknown_disk_order}（只认 normal、reversed）"
            ))
        }
    };
    match profile_label.as_deref().unwrap_or(DEFAULT_PROFILE_LABEL) {
        "r1" if marker_parity_label.is_some() => {
            Err("--marker-parity 只在 --profile r2、r3 或 r5 时有".to_string())
        }
        "r1" if table_source_label.is_some() || disk_order_label.is_some() => {
            Err("--table 与 --disk-order 只在 --profile r2、r3 或 r5 时有".to_string())
        }
        "r1" => Ok((RunProfile::R1, remaining_arguments)),
        "r2" => Ok((
            RunProfile::r2(marker_parity)
                .with_table_source(table_source)
                .with_reported_disk_order(reported_disk_order),
            remaining_arguments,
        )),
        "r3" | "r4" => Ok((
            RunProfile::r3(marker_parity)
                .with_table_source(table_source)
                .with_reported_disk_order(reported_disk_order),
            remaining_arguments,
        )),
        "r5" => Ok((
            RunProfile::r5(marker_parity)
                .with_table_source(table_source)
                .with_reported_disk_order(reported_disk_order),
            remaining_arguments,
        )),
        unknown_profile => Err(format!(
            "不认识的档：{unknown_profile}（只认 r1、r2、r3、r4、r5）"
        )),
    }
}

/// 配置行与收尾判决行里档的那几段：r1 档什么都不加（同 r1 时的装置逐字节）；r2 档的建表法与盘次序只在不是默认值时才写
/// （默认的 r2 档输出因此不因第四段加了这两个旋钮而多一个字）。
fn profile_fields_of_the_run_rows(run_profile: RunProfile) -> String {
    let table_source_field = match run_profile.table_source {
        RecordTableSource::Direct => String::new(),
        RecordTableSource::Scanned | RecordTableSource::Both => {
            format!(" table={}", run_profile.table_source.label())
        }
    };
    let disk_order_field = match run_profile.reported_disk_order {
        ReportedDiskOrder::FirstDeviceBeforeSecond => String::new(),
        ReportedDiskOrder::SecondDeviceBeforeFirst => {
            format!(" disk_order={}", run_profile.reported_disk_order.label())
        }
    };
    format!(
        "{}{table_source_field}{disk_order_field}",
        generator_and_criteria_fields_of_the_run_rows(run_profile)
    )
}

/// [`profile_fields_of_the_run_rows`] 里生成器与判据那两样的几段。
fn generator_and_criteria_fields_of_the_run_rows(run_profile: RunProfile) -> String {
    match (run_profile.generator, run_profile.criteria) {
        (
            FieldsBeyondTheProjectionGenerator::SameForEveryVersion,
            CriteriaSet::RegisteredBeforeR2,
        ) => String::new(),
        (
            FieldsBeyondTheProjectionGenerator::VaryWithTheVersion(marker_parity),
            CriteriaSet::WithJ6QnAndJ7,
        ) => format!(" profile=r2 marker_parity={}", marker_parity.label()),
        (
            FieldsBeyondTheProjectionGenerator::VaryWithTheVersion(marker_parity),
            CriteriaSet::WithJ6QnAndJ7SplitByI812,
        ) => format!(" profile=r3 marker_parity={}", marker_parity.label()),
        (FieldsBeyondTheProjectionGenerator::SameForEveryVersion, CriteriaSet::WithJ6QnAndJ7) => {
            " profile=r1_generator_with_r2_criteria".to_string()
        }
        (
            FieldsBeyondTheProjectionGenerator::SameForEveryVersion,
            CriteriaSet::WithJ6QnAndJ7SplitByI812,
        ) => " profile=r1_generator_with_r3_criteria".to_string(),
        (
            FieldsBeyondTheProjectionGenerator::VaryWithTheVersion(marker_parity),
            CriteriaSet::RegisteredBeforeR2,
        ) => format!(
            " profile=r2_generator_with_r1_criteria marker_parity={}",
            marker_parity.label()
        ),
        (
            FieldsBeyondTheProjectionGenerator::VaryWithTheVersion(marker_parity),
            CriteriaSet::WithJ6QnAndJ8SplitByI812,
        ) => format!(" profile=r5 marker_parity={}", marker_parity.label()),
        (
            FieldsBeyondTheProjectionGenerator::SameForEveryVersion,
            CriteriaSet::WithJ6QnAndJ8SplitByI812,
        ) => " profile=r1_generator_with_r5_criteria".to_string(),
    }
}

/// 线程数：`E165_THREADS`，没设取每台默认重活线程数（`SINGLEFS_THREAD_CAP`，再没设取 `.claude/heavy-thread-default` 的登记值）。
fn thread_count_from_the_environment() -> usize {
    thread_count_of_the_variable_values(
        env::var(THREADS_ENVIRONMENT_VARIABLE).ok().as_deref(),
        singlefs_harness::numeric_environment_variable::process_environment_variable(
            singlefs_harness::heavy_threads::THREAD_CAP_ENVIRONMENT_VARIABLE,
        )
        .as_deref(),
    )
}

/// [`thread_count_from_the_environment`] 的判法，两个变量的值由调用方给（`None` 是没设）：`E165_THREADS` 设了照它，
/// 没设经 `singlefs_harness::heavy_threads` 取每台默认重活线程数（`SINGLEFS_THREAD_CAP` 那一级在那里，这里不另留一份）。
///
/// # Panics
/// `E165_THREADS` 设了而不是正整数；落到默认时 `SINGLEFS_THREAD_CAP` 设了而不是正整数。
fn thread_count_of_the_variable_values(
    e165_threads_value: Option<&str>,
    thread_cap_value: Option<&str>,
) -> usize {
    match e165_threads_value {
        Some(text) => text
            .parse::<usize>()
            .ok()
            .filter(|thread_count| *thread_count > 0)
            .unwrap_or_else(|| {
                panic!("{THREADS_ENVIRONMENT_VARIABLE} 要是正整数，收到的是「{text}」")
            }),
        None => singlefs_harness::heavy_threads::default_heavy_threads_of_the_cap_value(
            thread_cap_value,
        )
        .get(),
    }
}

fn main() {
    let arguments_as_given: Vec<String> = env::args().skip(1).collect();
    let (run_profile, arguments) = match split_off_the_profile_options(&arguments_as_given) {
        Ok(split) => split,
        Err(problem) => {
            eprintln!("  ✗ {problem}");
            eprintln!("  → 怎么办：{USAGE}");
            std::process::exit(2);
        }
    };
    if let [option, table_path] = arguments.as_slice() {
        if option == "--pinned-cases" {
            replay_every_pinned_red_example(table_path, run_profile);
            return;
        }
    }
    if arguments_name_the_root_choice_faults_family(&arguments) {
        let profile_is_r4 = arguments_as_given
            .windows(2)
            .any(|pair| pair[0] == "--profile" && pair[1] == "r4");
        if !profile_is_r4 {
            eprintln!(
                "  ✗ --family root_choice_faults 只在 --profile r4 时有（登记 r4 5.1：这一族只在 r4 档）"
            );
            eprintln!("  → 怎么办：{USAGE}");
            std::process::exit(2);
        }
        run_the_root_choice_faults_family(&arguments);
        return;
    }
    if arguments_name_the_root_choice_family(&arguments) {
        if !run_profile.admits_the_root_choice_family() {
            eprintln!(
                "  ✗ --family root_choice 只在 --profile r3（或 r4）时有（登记 r3 5.1：这一族只在 r3 档；登记 r5 5.1：r5 档不认它）"
            );
            eprintln!("  → 怎么办：{USAGE}");
            std::process::exit(2);
        }
        run_the_root_choice_family(&arguments, run_profile);
        return;
    }
    match parse_the_state_probe(&arguments) {
        Ok(Some((domain, _size, _probe)))
            if domain.family() == Family::MultiDisk && !run_profile.judges_j6qn() =>
        {
            eprintln!(
                "  ✗ 多盘域 {} 只在 --profile r2、r3、r5 时有",
                domain.label()
            );
            eprintln!("  → 怎么办：{USAGE}");
            std::process::exit(2);
        }
        Ok(Some((domain, size, probe))) => {
            run_the_state_probe(domain, size, probe, run_profile);
            return;
        }
        Ok(None) => {}
        Err(problem) => {
            eprintln!("  ✗ {problem}");
            eprintln!("  → 怎么办：{USAGE}");
            std::process::exit(2);
        }
    }
    let selection = match parse_the_command_line(&arguments) {
        Ok(selection) => selection,
        Err(problem) => {
            eprintln!("  ✗ {problem}");
            eprintln!("  → 怎么办：{USAGE}");
            std::process::exit(2);
        }
    };
    if selection.families.contains(&Family::MultiDisk) && !run_profile.judges_j6qn() {
        eprintln!(
            "  ✗ --family multi_disk 只在 --profile r2、r3、r5 时有（登记 r2 5.1：多盘域不在 r1 档；登记 r5 5.1：r5 档认它）"
        );
        eprintln!("  → 怎么办：{USAGE}");
        std::process::exit(2);
    }
    let thread_count = thread_count_from_the_environment();
    install_the_panic_hook_that_stays_quiet_inside_the_replay_call();
    let run_started = Instant::now();
    let mut emitter = Emitter { emitted: 0 };
    let mut findings = RunFindings::default();
    emitter.result(&format!(
        "name=config experiment=E165 bounds={} family={} only={} max_n={} chosen_instance={CHOSEN_INSTANCE} first_txg={ORDINARY_FIRST_TXG} big_ring_slots={BIG_RING_SLOTS} big_ring_first_counter={BIG_RING_FIRST_HISTORY_COUNTER} small_ring_slots={SMALL_RING_SIX_SLOTS},{SMALL_RING_NINE_SLOTS}{}",
        selection.bounds.label(),
        selection.family_label,
        selection
            .only
            .as_ref()
            .map_or_else(|| "every_domain".to_string(), |names| names.join(",")),
        selection
            .largest_size_cap
            .map_or_else(|| "none".to_string(), |cap| cap.to_string()),
        profile_fields_of_the_run_rows(run_profile),
    ));

    // ① 常量回比断言与装置自检（断言不过当场 panic，进程以非 0 退出，一行结果都不算）；r2 档另跑 r2 那几项。
    let self_checks_of_the_profile: &[(&str, fn())] = if run_profile.judges_j6qn() {
        R2_SELF_CHECKS
    } else {
        &[]
    };
    // 多盘一族另跑多盘那几项（只在 --family multi_disk 时）。
    let self_checks_of_the_multi_disk_family: &[(&str, fn())] =
        if selection.families.contains(&Family::MultiDisk) {
            MULTI_DISK_SELF_CHECKS
        } else {
            &[]
        };
    // 记录表怎么建、盘次序有一个不是默认值时另跑这几项（只在 r2 档能设）。
    let self_checks_of_the_record_table_knobs: &[(&str, fn())] = if run_profile.table_source
        != RecordTableSource::Direct
        || run_profile.reported_disk_order != ReportedDiskOrder::FirstDeviceBeforeSecond
    {
        RECORD_TABLE_SELF_CHECKS
    } else {
        &[]
    };
    for (check_name, check) in SELF_CHECKS
        .iter()
        .chain(self_checks_of_the_profile)
        .chain(self_checks_of_the_multi_disk_family)
        .chain(self_checks_of_the_record_table_knobs)
    {
        check();
        emitter.result(&format!("name=self_check check={check_name} passed=true"));
    }
    for ring in [
        RingGeometry::BIG,
        RingGeometry::SMALL_NINE,
        RingGeometry::SMALL_SIX,
    ] {
        emitter.result(&format!(
            "name=in_flight_limit ring_slots={} local={} implementation={}",
            ring.ring_slots,
            ring.local_in_flight_limit(),
            journal_in_flight_record_limit(ring.ring_bytes())
        ));
    }

    // ② 阳性对照：遍历判据表的每一行；有一行没红，这一段作废（V2）。
    let controls_started = Instant::now();
    let control_readings =
        positive_control_readings(&selection.families, run_profile, thread_count);
    for reading in &control_readings {
        emitter.result(&format!(
            "name=control criterion={} measured={} value={} must_see={} red_as_required={}",
            reading.criterion.label(),
            reading.measured_where,
            reading.value,
            reading.must_see,
            reading.is_red_as_required
        ));
        if !reading.is_red_as_required {
            findings.void_conditions.push(format!(
                "V2 判据 {} 的阳性对照没红：{} 量到 {}，必须 {}",
                reading.criterion.label(),
                reading.measured_where,
                reading.value,
                reading.must_see
            ));
        }
    }
    println!(
        "E7TIMING name=elapsed scope=positive_controls elapsed_seconds={:.3} threads={thread_count}",
        controls_started.elapsed().as_secs_f64()
    );

    // ③–⑥ 各域照登记 5.5 的次序：L、L 小环、U；C；A、A 小环、A 链；X、E、R。
    let mut total_states: u64 = 0;
    for domain in Domain::ALL {
        if !selection.families.contains(&domain.family()) {
            continue;
        }
        if selection.only.as_ref().is_some_and(|names| {
            !names
                .iter()
                .any(|name| name.as_str() == domain.selection_name())
        }) {
            continue;
        }
        let Some(registered_largest_size) = selection.bounds.largest_size(*domain) else {
            continue;
        };
        let largest_size = selection
            .largest_size_cap
            .map_or(registered_largest_size, |cap| {
                cap.min(registered_largest_size)
            });
        if largest_size < domain.smallest_size() {
            continue;
        }
        let mut domain_total = Tally::new();
        for size in domain.smallest_size()..=largest_size {
            let size_started = Instant::now();
            let tally = tally_of_a_domain_size(
                *domain,
                size,
                Implementation::TodayReplayJournal,
                run_profile,
                thread_count,
            );
            assert_the_pinned_absolute_values(*domain, size, &tally);
            if run_profile.splits_j6qn_by_i812() && domain.family() == Family::Adversarial {
                assert_the_pinned_i812_state_counts(*domain, size, &tally);
                // 登记 r3 作废条款 TV6：拆开的四行对不回未拆的两行。
                if let Some(difference) = the_split_rows_differ_from_the_unsplit_rows(&tally) {
                    findings.void_conditions.push(format!(
                        "TV6 {} 域 n = {size} 拆开的 J6Qn 四行对不回未拆的两行：{difference}",
                        domain.label()
                    ));
                }
            }
            // 登记 r2 第七节甲类、SF9：每条记录在每块盘上都落了的那一态扫盘表的条数断言要真跑到过（一个都没跑到，这条断言是空的）。
            if run_profile.table_source != RecordTableSource::Direct
                && domain.family() != Family::Adversarial
                && tally.get(Counter::FullyLandedScanTableSizeChecked) == 0
            {
                findings.halt_conditions.push(format!(
                    "SF9 {} 域 N = {size} 没有一个「每条记录在每块盘上都落了」的态被断言过扫盘表的条数",
                    domain.label()
                ));
            }
            // 登记 r2 作废条款 SV6：两种建表法的表相同、判出来的行却不同（同一张表同一套判据不该有两种结局）。
            if run_profile.compares_the_two_record_tables()
                && tally.get(Counter::TablesDiffer) == 0
                && tally.get(Counter::RowsDiffer) > 0
            {
                findings.void_conditions.push(format!(
                    "SV6 {} 域 N = {size} 两种建表法的表逐态相同、判出来的行却有 {} 个态不同",
                    domain.label(),
                    tally.get(Counter::RowsDiffer)
                ));
            }
            emit_the_rows_of_a_domain_size(&mut emitter, *domain, size, &tally, run_profile);
            println!(
                "E7TIMING name=elapsed scope=domain domain={} n={size} states={} elapsed_seconds={:.3} threads={thread_count}",
                domain.label(),
                tally.get(Counter::States),
                size_started.elapsed().as_secs_f64()
            );
            domain_total.merge(&tally);
        }
        total_states += domain_total.get(Counter::States);
        emit_the_verdict_of_a_domain(
            &mut emitter,
            &mut findings,
            *domain,
            largest_size,
            &domain_total,
            run_profile,
        );
    }

    for condition in &findings.void_conditions {
        emitter.result(&format!(
            "name=void condition={}",
            condition.replace(' ', "_")
        ));
    }
    for condition in &findings.halt_conditions {
        emitter.result(&format!(
            "name=halt condition={}",
            condition.replace(' ', "_")
        ));
    }
    emitter.result(&format!(
        "name=verdict scope=run bounds={} family={} states={total_states} self_checks_passed={} control_readings={} controls_not_red={} void_conditions={} halt_conditions={}{}",
        selection.bounds.label(),
        selection.family_label,
        SELF_CHECKS.len()
            + self_checks_of_the_profile.len()
            + self_checks_of_the_multi_disk_family.len()
            + self_checks_of_the_record_table_knobs.len(),
        control_readings.len(),
        control_readings
            .iter()
            .filter(|reading| !reading.is_red_as_required)
            .count(),
        findings.void_conditions.len(),
        findings.halt_conditions.len(),
        profile_fields_of_the_run_rows(run_profile),
    ));
    println!(
        "E7TIMING name=elapsed scope=run states={total_states} elapsed_seconds={:.3} threads={thread_count}",
        run_started.elapsed().as_secs_f64()
    );
    emitter.done();
    if !findings.void_conditions.is_empty() || !findings.halt_conditions.is_empty() {
        std::process::exit(1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 线程数：`E165_THREADS` 先读；没设读 `SINGLEFS_THREAD_CAP`；都没设取登记的每台默认重活线程数，不取本机核数。
    #[test]
    fn thread_count_reads_e165_threads_first_then_the_heavy_thread_default() {
        assert_eq!(thread_count_of_the_variable_values(Some("3"), Some("5")), 3);
        assert_eq!(thread_count_of_the_variable_values(None, Some("5")), 5);
        assert_eq!(
            thread_count_of_the_variable_values(None, None),
            singlefs_harness::heavy_threads::registered_heavy_threads().get(),
            "都没设取 .claude/heavy-thread-default 的登记值"
        );
    }

    #[test]
    fn local_constants_equal_the_implementation_constants_and_the_in_flight_limits_are_21_3_2() {
        cross_check_local_constants_against_the_implementation();
    }

    #[test]
    fn every_generated_record_carries_the_back_chain_of_the_record_before_it() {
        check_the_back_chain_of_every_generated_record();
    }

    #[test]
    fn every_record_of_a_publish_carries_the_same_new_root_segment() {
        check_the_new_root_segment_of_every_generated_record();
    }

    #[test]
    fn the_state_counts_of_the_smallest_sizes_equal_the_pinned_values_and_the_empty_subset_is_counted(
    ) {
        check_the_state_counts_of_the_smallest_sizes();
    }

    #[test]
    fn the_three_readings_part_on_a_later_txg_and_on_a_missing_counter() {
        check_the_three_readings_on_hand_made_tables();
    }

    #[test]
    fn root_plus_disk_is_violated_when_a_named_unit_copy_is_bad() {
        check_root_plus_disk_on_a_bad_unit_copy();
    }

    #[test]
    fn the_in_flight_limit_control_is_red_on_the_small_ring_and_green_on_the_big_ring() {
        check_the_in_flight_limit_control_on_both_rings();
    }

    #[test]
    fn the_farthest_version_stops_at_the_in_flight_limit_on_the_small_ring() {
        check_the_farthest_version_on_the_small_ring();
    }

    #[test]
    fn the_fixed_point_control_is_red() {
        check_the_fixed_point_control();
    }

    #[test]
    fn ideal_recovery_on_one_later_record_breaks_the_applied_set_reading_once_and_the_landed_version_reading_never(
    ) {
        check_the_two_monotone_readings_on_one_later_record();
    }

    #[test]
    fn the_structure_control_is_red() {
        check_the_structure_control();
    }

    #[test]
    fn a_residue_on_the_second_device_does_not_complete_a_publish() {
        check_a_residue_does_not_complete_a_publish();
    }

    #[test]
    fn parse_rejects_a_record_with_ordinal_zero_whose_checksums_hold() {
        check_parse_rejects_ordinal_zero_whose_checksums_hold();
    }

    #[test]
    fn the_r2_generator_varies_the_four_fields_beyond_the_projection_with_the_version() {
        check_the_fields_beyond_the_projection_of_the_r2_generator();
    }

    #[test]
    fn without_profile_the_three_r1_product_commands_stay_on_the_r1_profile_and_the_marker_parity_needs_r2(
    ) {
        check_the_profile_options();
    }

    #[test]
    fn the_j6qn_controls_read_3110_on_the_big_ring_and_70_against_0_on_e1() {
        check_the_j6qn_control_readings();
    }

    #[test]
    fn the_j7_controls_read_762_and_2348_on_the_r2_generator_and_0_on_the_r1_generator() {
        check_the_j7_control_readings();
    }

    #[test]
    fn i812_holds_when_the_txg_does_not_fall_with_the_counter_inside_one_instance_and_equal_txgs_count_as_holding(
    ) {
        check_the_i812_judgement_on_hand_made_tables();
    }

    #[test]
    fn the_i812_state_counts_of_the_adversarial_domains_equal_the_pinned_values() {
        check_the_i812_state_counts_of_the_smallest_sizes();
    }

    #[test]
    fn the_i812_split_controls_read_the_pinned_cells_of_the_adversarial_domains() {
        check_the_i812_split_control_readings();
    }

    #[test]
    fn every_i812_cell_has_a_row_on_the_r3_profile_and_the_other_profiles_have_none() {
        check_the_rows_of_the_i812_split();
    }

    #[test]
    fn the_odd_marker_parity_turns_the_unmount_marker_controls_to_0_on_l_and_above_0_on_c() {
        check_the_odd_marker_parity_turns_the_unmount_marker_controls();
    }

    #[test]
    fn the_multi_disk_state_counts_lagging_classes_and_witnesses_of_the_smallest_sizes_equal_the_pinned_values(
    ) {
        check_the_multi_disk_state_counts_of_the_smallest_sizes();
    }

    #[test]
    fn a_state_whose_first_device_holds_nothing_and_whose_second_holds_the_first_record_lags_on_the_first_device(
    ) {
        check_the_lagging_direction_on_hand_made_masks();
    }

    #[test]
    fn the_multi_disk_controls_read_100_996_0_on_the_first_device_alone_and_228_2052_45_on_j1_and_j4(
    ) {
        check_the_multi_disk_control_readings();
    }

    #[test]
    fn a_residue_enters_the_direct_table_only_when_the_first_device_copy_is_not_on_disk() {
        check_the_direct_table_reads_the_first_device_before_the_second();
    }

    #[test]
    fn the_scan_configuration_carries_the_generators_filesystem_identifier_and_ring_length() {
        check_the_scan_system_configuration_matches_the_generator();
    }

    #[test]
    fn the_table_and_disk_order_options_default_to_direct_and_normal_and_the_r1_profile_refuses_them(
    ) {
        check_the_table_and_disk_order_options();
    }

    #[test]
    fn a_reversed_memory_disk_reports_the_second_device_before_the_first() {
        check_a_reversed_memory_disk_reports_the_second_device_first();
    }

    /// 一张表键相同、其中一条记录的新根段不同：必须报「两张表不同」（登记 r2 9.5「T-表只比键集合、不比值」）。
    #[test]
    fn two_record_tables_with_the_same_keys_and_a_different_new_root_segment_are_reported_as_different(
    ) {
        let record = hand_made_record(7, 101, 1, true);
        let mut other_record = record.clone();
        other_record.new_tree_identifier_watermark += 1;
        let direct_table = table_of(vec![record]);
        let scanned_table = table_of(vec![other_record]);
        assert!(
            direct_table.keys().eq(scanned_table.keys()),
            "两张表的键相同"
        );
        assert!(the_two_record_tables_differ(&direct_table, &scanned_table));
        assert!(!the_two_record_tables_differ(
            &direct_table,
            &direct_table.clone()
        ));
    }

    /// 两行的计数器标签集合相同、结局一边「报错」一边「返回根」：必须报「两行不同」（登记 r2 9.5「T-行只比计数器标签、不比实现结局」）。
    #[test]
    fn two_judged_rows_with_the_same_counter_labels_and_a_failure_against_a_returned_root_are_reported_as_different(
    ) {
        let returned = ReplayOutcome::Returned(ReturnedReplay {
            root: adversarial_chosen_root(RunProfile::R1),
            prefix_applied: 0,
            verification_passed: 0,
        });
        let failed = ReplayOutcome::Failed(FailureKind::Other);
        let summary = |first_outcome: ReplayOutcome, counter_labels: &str| JudgedRowSummary {
            first_outcome,
            counter_labels: counter_labels.to_string(),
        };
        assert!(the_two_judged_rows_differ(
            &summary(returned, "states,J4"),
            &summary(failed, "states,J4")
        ));
        assert!(the_two_judged_rows_differ(
            &summary(returned, "states,J4"),
            &summary(returned, "states")
        ));
        assert!(!the_two_judged_rows_differ(
            &summary(returned, "states,J4"),
            &summary(returned, "states,J4")
        ));
    }

    /// 两种建表法在最小的几档上逐态相同（登记 r2 第七节甲类「全部落盘那一态的扫盘表条数等于生成的条数」由装置在每个这样的态上断言，
    /// 断言没跑到的域这里也报红）：合法历史一族、多盘一族、对抗一族各取最小的两三档。
    #[test]
    fn the_two_table_sources_agree_on_the_smallest_sizes_and_the_scan_table_of_every_fully_landed_state_holds_every_record(
    ) {
        let run_profile = RunProfile::r2(MarkerParity::EvenVersionsWrittenByTheUnmountSequence)
            .with_table_source(RecordTableSource::Both);
        for (domain, sizes) in [
            (Domain::LegalBigRing, 1..=3),
            (Domain::LegalSmallRingSix, 1..=3),
            (Domain::NamedUnitsVerified, 1..=2),
            (Domain::MonotoneBigRing, 1..=2),
            (Domain::CrossInstance, 2..=3),
            (Domain::TxgOneBelowTheLargest, 1..=3),
            (Domain::Residue, 2..=3),
            (Domain::MirroredBigRing, 1..=2),
            (Domain::MirroredSmallRingSix, 1..=2),
            (Domain::MirroredMonotoneBigRing, 1..=2),
            (Domain::AdversarialBigRing, 1..=1),
        ] {
            for size in sizes {
                let tally = tally_of_a_domain_size(
                    domain,
                    size,
                    Implementation::TodayReplayJournal,
                    run_profile,
                    1,
                );
                assert_eq!(
                    (
                        tally.get(Counter::TablesDiffer),
                        tally.get(Counter::RowsDiffer)
                    ),
                    (0, 0),
                    "{} 域 N = {size}：两种建表法逐态相同",
                    domain.label()
                );
                if domain.family() != Family::Adversarial {
                    assert!(
                        tally.get(Counter::FullyLandedScanTableSizeChecked) > 0,
                        "{} 域 N = {size}：全部落盘那一态的扫盘表条数断言要真跑到",
                        domain.label()
                    );
                    assert!(tally.get(Counter::DirectTableHoldsARecord) > 0);
                }
            }
        }
    }

    /// 盘次序反过来（登记 r2 第八节「判别力自证」后一半）：R 域 N = 2，残留在 1 号盘，扫盘先读到残留；「盘号升序」那一栏的直接表在
    /// 0 号盘那一份落了的态上与扫盘表不同，必须大于 0。「照盘报的次序」那一栏是推 s7 的预期（0），不是判别力自证，不在这里断言。
    #[test]
    fn with_the_disk_order_reversed_the_scan_takes_the_residue_first_so_the_device_number_order_direct_table_differs(
    ) {
        let run_profile = RunProfile::r2(MarkerParity::EvenVersionsWrittenByTheUnmountSequence)
            .with_table_source(RecordTableSource::Both)
            .with_reported_disk_order(ReportedDiskOrder::SecondDeviceBeforeFirst);
        let tally = tally_of_a_domain_size(
            Domain::Residue,
            2,
            Implementation::TodayReplayJournal,
            run_profile,
            1,
        );
        assert!(
            tally.get(Counter::TablesDiffer) > 0,
            "R 域 N = 2：盘号升序一栏的直接表在残留所在那条记录 0 号盘那份落了的态上要与扫盘表不同"
        );
    }

    /// T-表与 T-行两条对照的读数（登记 r2 5.5、第七节）：T-表 `tables_differ` = `direct_table_holds_a_record` = 2 989、`rows_differ` > 0；
    /// T-行 `rows_differ` = 状态数 = 3 110、`states` = 3 110、`tables_differ` = 0。
    #[test]
    fn the_table_comparison_controls_read_2989_on_the_tables_and_3110_on_the_rows() {
        let run_profile = RunProfile::r2(MarkerParity::EvenVersionsWrittenByTheUnmountSequence)
            .with_table_source(RecordTableSource::Both);
        for (criterion, expected_values) in [
            (
                Criterion::TableComparisonSeesADifferentTable,
                [Some(2_989), Some(2_989), None],
            ),
            (
                Criterion::TableComparisonSeesADifferentRow,
                [Some(3_110), Some(3_110), Some(0)],
            ),
        ] {
            let readings = control_readings_of(criterion, run_profile, 1);
            assert_eq!(readings.len(), expected_values.len(), "{readings:?}");
            for (reading, expected) in readings.iter().zip(expected_values) {
                assert!(reading.is_red_as_required, "对照没红：{reading:?}");
                match expected {
                    Some(value) => assert_eq!(reading.value, value, "{reading:?}"),
                    None => assert!(reading.value > 0, "{reading:?}"),
                }
            }
        }
    }

    /// 逐态判定行里「这个状态让哪几个量加了一」：照 `Counter::ALL` 的次序，加了不止一的带次数，一个都没加写 none。
    #[test]
    fn the_counters_a_state_added_are_listed_in_order_with_their_multiplicity() {
        let before = vec![0u64; Counter::ALL.len()];
        let mut after = before.clone();
        assert_eq!(
            labels_of_the_counters_a_state_added(&before, &after),
            "none"
        );
        after[Counter::J4Violations.index()] += 1;
        after[Counter::States.index()] += 1;
        after[Counter::Failed.index()] += 2;
        assert_eq!(
            labels_of_the_counters_a_state_added(&before, &after),
            "states,failed*2,J4"
        );
    }

    /// 单状态重放的参数：三段是重放一个状态，两段是逐态列；段数不对、域不认识、N 小于这个域最小的 N 都拒。
    #[test]
    fn the_state_probe_argument_names_a_domain_a_size_and_an_ordinal() {
        let arguments = |option: &str, value: &str| vec![option.to_string(), value.to_string()];
        assert_eq!(
            parse_the_state_probe(&arguments("--replay-state", "U_verify_off:2:17")),
            Ok(Some((
                Domain::NamedUnitsNotVerified,
                2,
                StateProbe::ReplayOneState { ordinal: 17 }
            )))
        );
        assert_eq!(
            parse_the_state_probe(&arguments("--list-states", "AC:3")),
            Ok(Some((
                Domain::AdversarialBackChain,
                3,
                StateProbe::ListEveryState
            )))
        );
        assert_eq!(
            parse_the_state_probe(&arguments("--bounds", "light")),
            Ok(None)
        );
        assert!(parse_the_state_probe(&arguments("--replay-state", "L:2")).is_err());
        assert!(parse_the_state_probe(&arguments("--list-states", "L:2:0")).is_err());
        assert!(parse_the_state_probe(&arguments("--replay-state", "U:2:0")).is_err());
        assert!(parse_the_state_probe(&arguments("--replay-state", "X:1:0")).is_err());
    }

    /// 判据表每一行都有一条对照且都红（登记 5.4、作废条款 V2；r2 档加的 J6Qn、J7 五行同样，登记 r2 5.5；r5 档加的 J8 五行，登记 r5 5.4；
    /// T_table、T_row，登记 r2 5.5）。逐个核正式跑的单元：`scripts/experiments/replay.sh` 起这个装置的那几条驱动，加上第 5 次重跑登记
    /// 比对对象表里没有复跑行的四条（r2 直接建表，两族、两种标记奇偶），每条一族、一档、
    /// 一种建表法、一种盘次序、一种标记奇偶；每个单元跑出对照的判据恰好是那条驱动产物里 `name=control` 的那一份（写死成字面，
    /// 不从 `is_judged_under`、`Criterion::family` 现算，那是被核的东西），且读数全红；全部单元并起来罩全表。不把几档、几族并起来判：
    /// 并起来时一个单元少跑的对照会被别的单元的读数补上（判决 `research/prompts/m3-formal-code-b37-r1-main-verification.md` B1-甲、
    /// `research/prompts/m3-formal-code-b37-r2-main-verification.md` C3-甲）。对抗一族那几行要扫到 n = 3，线程数照装置的取法（[`thread_count_from_the_environment`]）。
    #[test]
    fn every_criterion_has_a_positive_control_and_every_control_is_red() {
        let thread_count = thread_count_from_the_environment();
        let even = MarkerParity::EvenVersionsWrittenByTheUnmountSequence;
        let odd = MarkerParity::OddVersionsWrittenByTheUnmountSequence;
        let labels: std::collections::BTreeSet<&str> = Criterion::ALL
            .iter()
            .map(|criterion| criterion.label())
            .collect();
        assert_eq!(labels.len(), Criterion::ALL.len(), "判据的标签互不相同");
        let formal_run_units: [(&str, Family, RunProfile, &[Criterion]); 19] = [
            // `scripts/experiments/replay.sh` 的 `driver_e165`、`driver_e165_light`；产物 `e165-replay-bounded-enumeration-2026-10-04-r5-segment1-r1-legal-family.out`，11 条
            (
                "r1 合法",
                Family::LegalHistory,
                RunProfile::R1,
                &[
                    Criterion::J0Defined,
                    Criterion::J1ReturnedRoot,
                    Criterion::J2RootPlusDisk,
                    Criterion::J3WithinTheInFlightLimit,
                    Criterion::J4FarthestVersion,
                    Criterion::J5aAppliedSet,
                    Criterion::J5bLandedVersion,
                    Criterion::J5aAndJ5bAreSeparable,
                    Criterion::J6FixedPoint,
                    Criterion::Ia1LargestTxgFails,
                    Criterion::Ia2NoVerificationCounted,
                ],
            ),
            // `scripts/experiments/replay.sh` 的 `driver_e165_adversarial`；产物 `e165-replay-bounded-enumeration-2026-10-04-r5-segment1-r1-adversarial-family.out`，7 条
            (
                "r1 对抗",
                Family::Adversarial,
                RunProfile::R1,
                &[
                    Criterion::A0Defined,
                    Criterion::AsWholeTable,
                    Criterion::AsGapContinues,
                    Criterion::AsGapStops,
                    Criterion::AMutual,
                    Criterion::AStructure,
                    Criterion::ALimit,
                ],
            ),
            // `scripts/experiments/replay.sh` 的 `driver_e165_r2_segment3_multi_disk`；产物 `e165-replay-bounded-enumeration-2026-10-04-r5-segment1-r2-multi-disk.out`，4 条
            (
                "r2 多盘",
                Family::MultiDisk,
                RunProfile::r2(even),
                &[
                    Criterion::MultiDiskAnyCopyIsOnDisk,
                    Criterion::MultiDiskReturnedRoot,
                    Criterion::MultiDiskFarthestVersion,
                    Criterion::MultiDiskAppliedSetAndLandedVersion,
                ],
            ),
            // `research/prompts/e165-r5-prereg.md` 比对对象表第 4 行（`replay.sh` 里没有复跑行）；产物 `e165-replay-bounded-enumeration-2026-10-04-r5-segment1-r2-legal-family.out`，17 条
            (
                "r2 合法",
                Family::LegalHistory,
                RunProfile::r2(even),
                &[
                    Criterion::J0Defined,
                    Criterion::J1ReturnedRoot,
                    Criterion::J2RootPlusDisk,
                    Criterion::J3WithinTheInFlightLimit,
                    Criterion::J4FarthestVersion,
                    Criterion::J5aAppliedSet,
                    Criterion::J5bLandedVersion,
                    Criterion::J5aAndJ5bAreSeparable,
                    Criterion::J6FixedPoint,
                    Criterion::Ia1LargestTxgFails,
                    Criterion::Ia2NoVerificationCounted,
                    Criterion::J6QnFixedPoint,
                    Criterion::J7aInstanceTable,
                    Criterion::J7aAllocationRecordTreeRoot,
                    Criterion::J7aFilesystemIdentifier,
                    Criterion::J7bUnmountMarkerOfARebuiltRoot,
                    Criterion::J7cUnmountMarkerWhenNothingIsApplied,
                ],
            ),
            // `research/prompts/e165-r5-prereg.md` 比对对象表第 5 行（`replay.sh` 里没有复跑行）；产物 `e165-replay-bounded-enumeration-2026-10-04-r5-segment1-r2-adversarial-family.out`，7 条
            (
                "r2 对抗",
                Family::Adversarial,
                RunProfile::r2(even),
                &[
                    Criterion::A0Defined,
                    Criterion::AsWholeTable,
                    Criterion::AsGapContinues,
                    Criterion::AsGapStops,
                    Criterion::AMutual,
                    Criterion::AStructure,
                    Criterion::ALimit,
                ],
            ),
            // `research/prompts/e165-r5-prereg.md` 比对对象表第 6 行（`replay.sh` 里没有复跑行）；产物 `e165-replay-bounded-enumeration-2026-10-04-r5-segment1-r2-odd-marker-legal-family.out`，17 条
            (
                "奇数版本的 r2 合法",
                Family::LegalHistory,
                RunProfile::r2(odd),
                &[
                    Criterion::J0Defined,
                    Criterion::J1ReturnedRoot,
                    Criterion::J2RootPlusDisk,
                    Criterion::J3WithinTheInFlightLimit,
                    Criterion::J4FarthestVersion,
                    Criterion::J5aAppliedSet,
                    Criterion::J5bLandedVersion,
                    Criterion::J5aAndJ5bAreSeparable,
                    Criterion::J6FixedPoint,
                    Criterion::Ia1LargestTxgFails,
                    Criterion::Ia2NoVerificationCounted,
                    Criterion::J6QnFixedPoint,
                    Criterion::J7aInstanceTable,
                    Criterion::J7aAllocationRecordTreeRoot,
                    Criterion::J7aFilesystemIdentifier,
                    Criterion::J7bUnmountMarkerOfARebuiltRoot,
                    Criterion::J7cUnmountMarkerWhenNothingIsApplied,
                ],
            ),
            // `research/prompts/e165-r5-prereg.md` 比对对象表第 7 行（`replay.sh` 里没有复跑行）；产物 `e165-replay-bounded-enumeration-2026-10-04-r5-segment1-r2-odd-marker-adversarial-family.out`，7 条
            (
                "奇数版本的 r2 对抗",
                Family::Adversarial,
                RunProfile::r2(odd),
                &[
                    Criterion::A0Defined,
                    Criterion::AsWholeTable,
                    Criterion::AsGapContinues,
                    Criterion::AsGapStops,
                    Criterion::AMutual,
                    Criterion::AStructure,
                    Criterion::ALimit,
                ],
            ),
            // `scripts/experiments/replay.sh` 的 `driver_e165_r2_segment4_table_both_legal`；产物 `e165-replay-bounded-enumeration-2026-10-04-r5-segment1-r2-table-both-legal-family.out`，19 条
            (
                "两份表都取的 r2 合法",
                Family::LegalHistory,
                RunProfile::r2(even).with_table_source(RecordTableSource::Both),
                &[
                    Criterion::J0Defined,
                    Criterion::J1ReturnedRoot,
                    Criterion::J2RootPlusDisk,
                    Criterion::J3WithinTheInFlightLimit,
                    Criterion::J4FarthestVersion,
                    Criterion::J5aAppliedSet,
                    Criterion::J5bLandedVersion,
                    Criterion::J5aAndJ5bAreSeparable,
                    Criterion::J6FixedPoint,
                    Criterion::Ia1LargestTxgFails,
                    Criterion::Ia2NoVerificationCounted,
                    Criterion::J6QnFixedPoint,
                    Criterion::J7aInstanceTable,
                    Criterion::J7aAllocationRecordTreeRoot,
                    Criterion::J7aFilesystemIdentifier,
                    Criterion::J7bUnmountMarkerOfARebuiltRoot,
                    Criterion::J7cUnmountMarkerWhenNothingIsApplied,
                    Criterion::TableComparisonSeesADifferentTable,
                    Criterion::TableComparisonSeesADifferentRow,
                ],
            ),
            // `scripts/experiments/replay.sh` 的 `driver_e165_r2_segment4_table_both_adversarial`；产物 `e165-replay-bounded-enumeration-2026-10-04-r5-segment1-r2-table-both-adversarial-family.out`，9 条
            (
                "两份表都取的 r2 对抗",
                Family::Adversarial,
                RunProfile::r2(even).with_table_source(RecordTableSource::Both),
                &[
                    Criterion::A0Defined,
                    Criterion::AsWholeTable,
                    Criterion::AsGapContinues,
                    Criterion::AsGapStops,
                    Criterion::AMutual,
                    Criterion::AStructure,
                    Criterion::ALimit,
                    Criterion::TableComparisonSeesADifferentTable,
                    Criterion::TableComparisonSeesADifferentRow,
                ],
            ),
            // `scripts/experiments/replay.sh` 的 `driver_e165_r2_segment4_table_both_multi_disk`；产物 `e165-replay-bounded-enumeration-2026-10-04-r5-segment1-r2-table-both-multi-disk.out`，6 条
            (
                "两份表都取的 r2 多盘",
                Family::MultiDisk,
                RunProfile::r2(even).with_table_source(RecordTableSource::Both),
                &[
                    Criterion::MultiDiskAnyCopyIsOnDisk,
                    Criterion::MultiDiskReturnedRoot,
                    Criterion::MultiDiskFarthestVersion,
                    Criterion::MultiDiskAppliedSetAndLandedVersion,
                    Criterion::TableComparisonSeesADifferentTable,
                    Criterion::TableComparisonSeesADifferentRow,
                ],
            ),
            // `scripts/experiments/replay.sh` 的 `driver_e165_r2_segment4_reversed_disk_order_legal`；产物 `e165-replay-bounded-enumeration-2026-10-04-r5-segment1-r2-reversed-disk-order-legal-family.out`，19 条
            (
                "两份表都取、盘次序反报的 r2 合法",
                Family::LegalHistory,
                RunProfile::r2(even)
                    .with_table_source(RecordTableSource::Both)
                    .with_reported_disk_order(ReportedDiskOrder::SecondDeviceBeforeFirst),
                &[
                    Criterion::J0Defined,
                    Criterion::J1ReturnedRoot,
                    Criterion::J2RootPlusDisk,
                    Criterion::J3WithinTheInFlightLimit,
                    Criterion::J4FarthestVersion,
                    Criterion::J5aAppliedSet,
                    Criterion::J5bLandedVersion,
                    Criterion::J5aAndJ5bAreSeparable,
                    Criterion::J6FixedPoint,
                    Criterion::Ia1LargestTxgFails,
                    Criterion::Ia2NoVerificationCounted,
                    Criterion::J6QnFixedPoint,
                    Criterion::J7aInstanceTable,
                    Criterion::J7aAllocationRecordTreeRoot,
                    Criterion::J7aFilesystemIdentifier,
                    Criterion::J7bUnmountMarkerOfARebuiltRoot,
                    Criterion::J7cUnmountMarkerWhenNothingIsApplied,
                    Criterion::TableComparisonSeesADifferentTable,
                    Criterion::TableComparisonSeesADifferentRow,
                ],
            ),
            // `scripts/experiments/replay.sh` 的 `driver_e165_r2_segment4_reversed_disk_order_multi_disk`；产物 `e165-replay-bounded-enumeration-2026-10-04-r5-segment1-r2-reversed-disk-order-multi-disk.out`，6 条
            (
                "两份表都取、盘次序反报的 r2 多盘",
                Family::MultiDisk,
                RunProfile::r2(even)
                    .with_table_source(RecordTableSource::Both)
                    .with_reported_disk_order(ReportedDiskOrder::SecondDeviceBeforeFirst),
                &[
                    Criterion::MultiDiskAnyCopyIsOnDisk,
                    Criterion::MultiDiskReturnedRoot,
                    Criterion::MultiDiskFarthestVersion,
                    Criterion::MultiDiskAppliedSetAndLandedVersion,
                    Criterion::TableComparisonSeesADifferentTable,
                    Criterion::TableComparisonSeesADifferentRow,
                ],
            ),
            // `scripts/experiments/replay.sh` 的 `driver_e165_r3_segment1_adversarial`；产物 `e165-replay-bounded-enumeration-2026-10-04-r5-segment1-r3-adversarial-family.out`，8 条
            (
                "r3 对抗",
                Family::Adversarial,
                RunProfile::r3(even),
                &[
                    Criterion::A0Defined,
                    Criterion::AsWholeTable,
                    Criterion::AsGapContinues,
                    Criterion::AsGapStops,
                    Criterion::AMutual,
                    Criterion::AStructure,
                    Criterion::ALimit,
                    Criterion::J6QnSplitByI812,
                ],
            ),
            // `scripts/experiments/replay.sh` 的 `driver_e165_r6_r3_legal`；产物 `e165-replay-bounded-enumeration-2026-10-09-r6-r3-legal-family.out`，17 条
            (
                "r3 合法",
                Family::LegalHistory,
                RunProfile::r3(even),
                &[
                    Criterion::J0Defined,
                    Criterion::J1ReturnedRoot,
                    Criterion::J2RootPlusDisk,
                    Criterion::J3WithinTheInFlightLimit,
                    Criterion::J4FarthestVersion,
                    Criterion::J5aAppliedSet,
                    Criterion::J5bLandedVersion,
                    Criterion::J5aAndJ5bAreSeparable,
                    Criterion::J6FixedPoint,
                    Criterion::Ia1LargestTxgFails,
                    Criterion::Ia2NoVerificationCounted,
                    Criterion::J6QnFixedPoint,
                    Criterion::J7aInstanceTable,
                    Criterion::J7aAllocationRecordTreeRoot,
                    Criterion::J7aFilesystemIdentifier,
                    Criterion::J7bUnmountMarkerOfARebuiltRoot,
                    Criterion::J7cUnmountMarkerWhenNothingIsApplied,
                ],
            ),
            // `scripts/experiments/replay.sh` 的 `driver_e165_r5_segment3_legal`；产物 `e165-replay-bounded-enumeration-2026-10-04-r5-segment3-r5-legal-family.out`，16 条
            (
                "r5 合法",
                Family::LegalHistory,
                RunProfile::r5(even),
                &[
                    Criterion::J0Defined,
                    Criterion::J1ReturnedRoot,
                    Criterion::J2RootPlusDisk,
                    Criterion::J3WithinTheInFlightLimit,
                    Criterion::J4FarthestVersion,
                    Criterion::J5aAppliedSet,
                    Criterion::J5bLandedVersion,
                    Criterion::J5aAndJ5bAreSeparable,
                    Criterion::J6FixedPoint,
                    Criterion::Ia1LargestTxgFails,
                    Criterion::Ia2NoVerificationCounted,
                    Criterion::J6QnFixedPoint,
                    Criterion::J8aFieldsFromTheNewRootSegment,
                    Criterion::J8aUnmountMarkerFromTheNewRootSegment,
                    Criterion::J8bFilesystemIdentifierOfTheChosenRoot,
                    Criterion::J8cNothingAppliedReturnsTheChosenRoot,
                ],
            ),
            // `scripts/experiments/replay.sh` 的 `driver_e165_r5_segment3_adversarial`；产物 `e165-replay-bounded-enumeration-2026-10-04-r5-segment3-r5-adversarial-family.out`，8 条
            (
                "r5 对抗",
                Family::Adversarial,
                RunProfile::r5(even),
                &[
                    Criterion::A0Defined,
                    Criterion::AsWholeTable,
                    Criterion::AsGapContinues,
                    Criterion::AsGapStops,
                    Criterion::AMutual,
                    Criterion::AStructure,
                    Criterion::ALimit,
                    Criterion::J6QnSplitByI812,
                ],
            ),
            // `scripts/experiments/replay.sh` 的 `driver_e165_r5_segment3_multi_disk`；产物 `e165-replay-bounded-enumeration-2026-10-04-r5-segment3-r5-multi-disk.out`，4 条
            (
                "r5 多盘",
                Family::MultiDisk,
                RunProfile::r5(even),
                &[
                    Criterion::MultiDiskAnyCopyIsOnDisk,
                    Criterion::MultiDiskReturnedRoot,
                    Criterion::MultiDiskFarthestVersion,
                    Criterion::MultiDiskAppliedSetAndLandedVersion,
                ],
            ),
            // `scripts/experiments/replay.sh` 的 `driver_e165_r5_segment3_odd_marker_legal`；产物 `e165-replay-bounded-enumeration-2026-10-04-r5-segment3-r5-odd-marker-legal-family.out`，16 条
            (
                "奇数版本的 r5 合法",
                Family::LegalHistory,
                RunProfile::r5(odd),
                &[
                    Criterion::J0Defined,
                    Criterion::J1ReturnedRoot,
                    Criterion::J2RootPlusDisk,
                    Criterion::J3WithinTheInFlightLimit,
                    Criterion::J4FarthestVersion,
                    Criterion::J5aAppliedSet,
                    Criterion::J5bLandedVersion,
                    Criterion::J5aAndJ5bAreSeparable,
                    Criterion::J6FixedPoint,
                    Criterion::Ia1LargestTxgFails,
                    Criterion::Ia2NoVerificationCounted,
                    Criterion::J6QnFixedPoint,
                    Criterion::J8aFieldsFromTheNewRootSegment,
                    Criterion::J8aUnmountMarkerFromTheNewRootSegment,
                    Criterion::J8bFilesystemIdentifierOfTheChosenRoot,
                    Criterion::J8cNothingAppliedReturnsTheChosenRoot,
                ],
            ),
            // `scripts/experiments/replay.sh` 的 `driver_e165_r5_segment3_odd_marker_adversarial`；产物 `e165-replay-bounded-enumeration-2026-10-04-r5-segment3-r5-odd-marker-adversarial-family.out`，8 条
            (
                "奇数版本的 r5 对抗",
                Family::Adversarial,
                RunProfile::r5(odd),
                &[
                    Criterion::A0Defined,
                    Criterion::AsWholeTable,
                    Criterion::AsGapContinues,
                    Criterion::AsGapStops,
                    Criterion::AMutual,
                    Criterion::AStructure,
                    Criterion::ALimit,
                    Criterion::J6QnSplitByI812,
                ],
            ),
        ];
        let mut controlled_in_some_unit = std::collections::BTreeSet::new();
        for (unit_name, family, run_profile, registered_criteria) in formal_run_units {
            let readings = positive_control_readings(&[family], run_profile, thread_count);
            let controlled: std::collections::BTreeSet<Criterion> =
                readings.iter().map(|reading| reading.criterion).collect();
            let registered: std::collections::BTreeSet<Criterion> =
                registered_criteria.iter().copied().collect();
            assert_eq!(
                controlled, registered,
                "{unit_name} 那个单元跑出对照的判据与那条正式跑产物里的那一份不同"
            );
            for reading in &readings {
                assert!(
                    reading.is_red_as_required,
                    "{unit_name} 那个单元对照没红：{reading:?}"
                );
            }
            controlled_in_some_unit.extend(controlled);
        }
        for criterion in Criterion::ALL.iter() {
            assert!(
                controlled_in_some_unit.contains(criterion),
                "判据 {} 在正式跑的单元里都没有对照的读数",
                criterion.label()
            );
        }
    }

    /// 钉住的表 `.claude/gate.d/e165-pinned-red-examples.tsv` 里 ⑤ 指 2026-10-01 第二段红例账、① 为下面两个指纹的行，①②③⑦ 原样抄来
    /// （A 域那样的行有两行，⑥ 不同、①②③⑦ 相同）：拿 ⑦ 重算的指纹要等于 ①。
    /// A 域那一行的 state= 在 ;first= 之后还有 ;expected=，截掉的是 ;first= 起的整段。
    #[test]
    fn the_input_fingerprint_recomputed_from_a_pinned_state_row_equals_the_one_the_table_carries() {
        let cross_instance_state_row = "E7RESULT name=state domain=X ring_slots=64 n=2 ordinal=2 counters=states,j6_premise_holds_judged state=shape=1c0/next_instance:1c0;on_disk=-,good;residue_at=none;chosen=0;farthest=0;verify=true;first=returned:txg100:watermark1000:applied0";
        assert_eq!(
            input_fingerprint_of("X", "2", cross_instance_state_row).as_deref(),
            Some("371a71190dce62e090cbf0a361b11069fb68cc63ab2a44e88398bb2f0570f2f0"),
            "X 域那一行：sha256(域|N|state= 截到 ;first= 之前) 要等于表里的 ①"
        );
        let adversarial_state_row = "E7RESULT name=state domain=A ring_slots=64 n=1 ordinal=112 counters=states,j6_premise_holds_judged,applied_ge1 state=anchor=present;slots=t+1:o1:L1:C1:x1:chain_true;first=returned:txg101:watermark9001038:applied1;expected=txg101:watermark9001038/txg101:watermark9001038/txg101:watermark9001038";
        assert_eq!(
            input_fingerprint_of("A", "1", adversarial_state_row).as_deref(),
            Some("aacef16d8fa9f38237852f113de9db647ac0655bfe4192f8cdcce7c5e8a002b3"),
            "A 域那一行：;first= 之后的 ;expected= 也不进指纹"
        );
        assert_eq!(
            input_fingerprint_of("A", "1", "E7RESULT name=done emitted=3"),
            None,
            "没有 state= 那一段的行算不出指纹"
        );
    }

    /// 钉住的表一行：二十一段、抄来的账行与 ②③④⑦⑧ 相同、②③④ 解得出才收；段数不对、域不认识、N 越界、序号不是数都拒。
    #[test]
    fn a_pinned_red_example_row_needs_twenty_one_columns_a_known_domain_a_size_in_range_and_an_ordinal(
    ) {
        let row = |domain: &str, size: &str, ordinal: &str| {
            format!("abc\t{domain}\t{size}\t{ordinal}\tledger.tsv:2\tcrates/mutations.tsv#name\tE7RESULT original\tE7RESULT variant\ttsv-1\t1\t{domain}\t{size}\t{ordinal}\tcounters\t+J1\ttrue\ttrue\ttrue\tcommand\tE7RESULT original\tE7RESULT variant")
        };
        assert_eq!(
            parse_a_pinned_red_example(20, &row("U_verify_off", "3", "36")),
            Ok(PinnedRedExample {
                line_number_in_the_table: 20,
                input_fingerprint: "abc".to_string(),
                domain_text: "U_verify_off".to_string(),
                size_text: "3".to_string(),
                domain: Domain::NamedUnitsNotVerified,
                size: 3,
                ordinal: 36,
            })
        );
        let twenty_columns = row("L", "2", "3")
            .rsplit_once('\t')
            .map(|(first_twenty_columns, _twenty_first_column)| first_twenty_columns.to_string())
            .expect("造的行有二十一段");
        assert!(parse_a_pinned_red_example(20, &twenty_columns).is_err());
        assert!(parse_a_pinned_red_example(20, &row("U", "2", "3")).is_err());
        assert!(parse_a_pinned_red_example(20, &row("X", "1", "3")).is_err());
        assert!(parse_a_pinned_red_example(20, &row("L", "2", "three")).is_err());
    }

    /// 今天钉住的表里 X:2:2 那一行自己的八段（原样抄来）。
    const OWN_COLUMNS_OF_THE_PINNED_CROSS_INSTANCE_ROW: [&str; 8] = [
        "371a71190dce62e090cbf0a361b11069fb68cc63ab2a44e88398bb2f0570f2f0",
        "X",
        "2",
        "2",
        "e165-replay-bounded-enumeration-2026-10-01-segment2-red-examples.tsv:2",
        "crates/mutations.tsv#步 3：前缀跨实例边界（把别的实例的记录也接上）",
        "E7RESULT name=state domain=X ring_slots=64 n=2 ordinal=2 counters=states,j6_premise_holds_judged state=shape=1c0/next_instance:1c0;on_disk=-,good;residue_at=none;chosen=0;farthest=0;verify=true;first=returned:txg100:watermark1000:applied0",
        "E7RESULT name=state domain=X ring_slots=64 n=2 ordinal=2 counters=states,J1,J2,J3,J4,j6_premise_holds_judged,off_timeline_landings state=shape=1c0/next_instance:1c0;on_disk=-,good;residue_at=none;chosen=0;farthest=0;verify=true;first=returned:txg101:watermark1001:applied1",
    ];

    /// 那一行 ⑤ 指的红例账 `research/results/e165-replay-bounded-enumeration-2026-10-01-segment2-red-examples.tsv` 里 `tsv-23` 那一行的十三段（原样抄来）。
    const LEDGER_COLUMNS_OF_THE_PINNED_CROSS_INSTANCE_ROW: [&str; 13] = [
        "tsv-23",
        "23+355",
        "X",
        "2",
        "2",
        "counters",
        "+J1,+J2,+J3,+J4,+off_timeline_landings",
        "true",
        "true",
        "true",
        "target/release/e165_replay_bounded_enumeration --replay-state X:2:2",
        "E7RESULT name=state domain=X ring_slots=64 n=2 ordinal=2 counters=states,j6_premise_holds_judged state=shape=1c0/next_instance:1c0;on_disk=-,good;residue_at=none;chosen=0;farthest=0;verify=true;first=returned:txg100:watermark1000:applied0",
        "E7RESULT name=state domain=X ring_slots=64 n=2 ordinal=2 counters=states,J1,J2,J3,J4,j6_premise_holds_judged,off_timeline_landings state=shape=1c0/next_instance:1c0;on_disk=-,good;residue_at=none;chosen=0;farthest=0;verify=true;first=returned:txg101:watermark1001:applied1",
    ];

    /// 一张表：两行表头注释，第 3 行是 X:2:2 那一行自己的八段接上给的账行。
    fn pinned_table_of_the_cross_instance_row_with_ledger_columns(
        ledger_columns: &[&str],
    ) -> String {
        let mut row_columns: Vec<&str> = OWN_COLUMNS_OF_THE_PINNED_CROSS_INSTANCE_ROW.to_vec();
        row_columns.extend_from_slice(ledger_columns);
        format!(
            "# 钉住的红例\n# 二十一段，制表符分隔\n{}\n",
            row_columns.join("\t")
        )
    }

    /// X:2:2 那一行的账行把第 `ledger_column_number` 段（从 1 数）换成 `replacement`，整张表解出来的问题。
    fn problems_when_one_ledger_column_of_the_cross_instance_row_is_replaced(
        ledger_column_number: usize,
        replacement: &str,
    ) -> Vec<String> {
        let mut ledger_columns = LEDGER_COLUMNS_OF_THE_PINNED_CROSS_INSTANCE_ROW;
        ledger_columns[ledger_column_number - 1] = replacement;
        let parse = parse_the_pinned_table(
            &pinned_table_of_the_cross_instance_row_with_ledger_columns(&ledger_columns),
        );
        assert_eq!(parse.data_line_count, 1, "表里只有一行数据");
        assert!(
            parse.examples.is_empty(),
            "账行第 {ledger_column_number} 段与钉表自己的段不同，这一行不该收：{parse:?}"
        );
        parse.problems_of_the_rows_that_do_not_parse
    }

    /// 二十一段、⑨ 起是红例账那一行原样的表：每行都收，②③④ 与 ① 照表里的解出来。
    #[test]
    fn a_pinned_table_whose_rows_carry_the_whole_ledger_row_parses_every_row() {
        let parse =
            parse_the_pinned_table(&pinned_table_of_the_cross_instance_row_with_ledger_columns(
                &LEDGER_COLUMNS_OF_THE_PINNED_CROSS_INSTANCE_ROW,
            ));
        assert_eq!(parse.data_line_count, 1, "两行注释不算数据行");
        assert_eq!(
            parse.problems_of_the_rows_that_do_not_parse,
            Vec::<String>::new(),
            "合法的二十一段行不该有问题"
        );
        assert_eq!(
            parse.examples,
            vec![PinnedRedExample {
                line_number_in_the_table: 3,
                input_fingerprint:
                    "371a71190dce62e090cbf0a361b11069fb68cc63ab2a44e88398bb2f0570f2f0".to_string(),
                domain_text: "X".to_string(),
                size_text: "2".to_string(),
                domain: Domain::CrossInstance,
                size: 2,
                ordinal: 2,
            }]
        );
    }

    /// 只有八段的旧行：报「没整行抄红例账」并给补的脚本，这一行不收。
    #[test]
    fn a_pinned_table_row_with_only_the_old_eight_columns_is_rejected_for_not_carrying_the_ledger_row(
    ) {
        let parse = parse_the_pinned_table(
            &pinned_table_of_the_cross_instance_row_with_ledger_columns(&[]),
        );
        assert_eq!(parse.data_line_count, 1, "表里只有一行数据");
        assert!(parse.examples.is_empty(), "八段的旧行不该收：{parse:?}");
        assert_eq!(parse.problems_of_the_rows_that_do_not_parse.len(), 1);
        assert!(
            parse.problems_of_the_rows_that_do_not_parse[0].starts_with(
                "第 3 行：这一行没整行抄红例账，用 scripts/experiments/e165-pin-red-examples.py 补"
            ),
            "要点名行号、说没整行抄、给补的脚本：{:?}",
            parse.problems_of_the_rows_that_do_not_parse
        );
    }

    /// 账行第 12 段（原实现的逐态行）与 ⑦ 不同：拒，点名第 3 行的 ⑳ 与 ⑦。
    #[test]
    fn a_pinned_table_row_whose_ledger_original_state_row_differs_from_column_seven_is_rejected() {
        let problems = problems_when_one_ledger_column_of_the_cross_instance_row_is_replaced(
            12,
            "E7RESULT name=state domain=X ring_slots=64 n=2 ordinal=2 counters=states state=shape=1c0;first=returned:txg100:watermark1000:applied0",
        );
        assert_eq!(problems.len(), 1, "{problems:?}");
        assert!(
            problems[0].starts_with("第 3 行：⑳（红例账第 12 段）与 ⑦ 不同"),
            "{problems:?}"
        );
    }

    /// 账行第 13 段（变体的逐态行）与 ⑧ 不同：拒，点名第 3 行的 ㉑ 与 ⑧。
    #[test]
    fn a_pinned_table_row_whose_ledger_variant_state_row_differs_from_column_eight_is_rejected() {
        let problems = problems_when_one_ledger_column_of_the_cross_instance_row_is_replaced(
            13,
            "E7RESULT name=state domain=X ring_slots=64 n=2 ordinal=2 counters=states state=shape=1c0;first=returned:txg101:watermark1001:applied1",
        );
        assert_eq!(problems.len(), 1, "{problems:?}");
        assert!(
            problems[0].starts_with("第 3 行：㉑（红例账第 13 段）与 ⑧ 不同"),
            "{problems:?}"
        );
    }

    /// 账行第 3 段（域）与 ② 不同：拒，点名第 3 行的 ⑪ 与 ②。
    #[test]
    fn a_pinned_table_row_whose_ledger_domain_differs_from_column_two_is_rejected() {
        let problems =
            problems_when_one_ledger_column_of_the_cross_instance_row_is_replaced(3, "L");
        assert_eq!(problems.len(), 1, "{problems:?}");
        assert!(
            problems[0].starts_with("第 3 行：⑪（红例账第 3 段）与 ② 不同"),
            "{problems:?}"
        );
    }

    /// 账行第 4 段（N）与 ③ 不同：拒，点名第 3 行的 ⑫ 与 ③。
    #[test]
    fn a_pinned_table_row_whose_ledger_size_differs_from_column_three_is_rejected() {
        let problems =
            problems_when_one_ledger_column_of_the_cross_instance_row_is_replaced(4, "3");
        assert_eq!(problems.len(), 1, "{problems:?}");
        assert!(
            problems[0].starts_with("第 3 行：⑫（红例账第 4 段）与 ③ 不同"),
            "{problems:?}"
        );
    }

    /// 账行第 5 段（序号）与 ④ 不同：拒，点名第 3 行的 ⑬ 与 ④。
    #[test]
    fn a_pinned_table_row_whose_ledger_ordinal_differs_from_column_four_is_rejected() {
        let problems =
            problems_when_one_ledger_column_of_the_cross_instance_row_is_replaced(5, "7");
        assert_eq!(problems.len(), 1, "{problems:?}");
        assert!(
            problems[0].starts_with("第 3 行：⑬（红例账第 5 段）与 ④ 不同"),
            "{problems:?}"
        );
    }

    /// 重放出来的逐态行对着表里那一行判：指纹相同、指纹漂了、序号不在这个 (域, N) 里，三种各判各的。
    #[test]
    fn a_pinned_red_example_whose_recomputed_fingerprint_differs_is_reported_as_drifted() {
        let state_row = "E7RESULT name=state domain=X ring_slots=64 n=2 ordinal=2 counters=states,j6_premise_holds_judged state=shape=1c0/next_instance:1c0;on_disk=-,good;residue_at=none;chosen=0;farthest=0;verify=true;first=returned:txg100:watermark1000:applied0";
        let example = PinnedRedExample {
            line_number_in_the_table: 20,
            input_fingerprint: "371a71190dce62e090cbf0a361b11069fb68cc63ab2a44e88398bb2f0570f2f0"
                .to_string(),
            domain_text: "X".to_string(),
            size_text: "2".to_string(),
            domain: Domain::CrossInstance,
            size: 2,
            ordinal: 2,
        };
        assert_eq!(
            judge_a_pinned_red_example_replay(&example, Some(state_row.to_string()), 10),
            PinnedRedExampleReplay::FingerprintMatches {
                state_row: state_row.to_string()
            }
        );
        let drifted_state_row = state_row.replace("on_disk=-,good", "on_disk=good,-");
        assert!(
            matches!(
                judge_a_pinned_red_example_replay(&example, Some(drifted_state_row), 10),
                PinnedRedExampleReplay::FingerprintDrifted { .. }
            ),
            "状态编码变了，重算的指纹与 ① 不同，要报漂了"
        );
        assert_eq!(
            judge_a_pinned_red_example_replay(&example, None, 2),
            PinnedRedExampleReplay::OrdinalOutsideTheDomainSize { states_walked: 2 }
        );
    }

    // ── 择根与重放合判一族（登记 r3 5.3 至 5.7、9.2 乙）

    #[test]
    fn root_choice_local_constants_equal_the_implementation_and_the_first_mount_of_a_new_pool_pushes_two_overhead_publishes(
    ) {
        cross_check_root_choice_local_constants_against_the_implementation();
    }

    #[test]
    fn root_versions_compare_the_txg_first_and_the_instance_breaks_ties() {
        let version = |checkpoint_txg: u64, instance: u32| RootVersion {
            checkpoint_txg,
            instance,
        };
        assert!(
            version(100, 1) > version(99, 9),
            "txg 为主：txg 大的版本大，不看实例代号"
        );
        assert!(
            version(100, 2) > version(100, 1),
            "txg 相等时实例代号大的版本大"
        );
        assert_eq!(version(7, 3), version(7, 3));
    }

    #[test]
    fn the_warm_up_plan_covers_both_devices_with_at_most_two_empty_publishes_after_the_row_write() {
        assert_eq!(root_choice_warm_up_plan(1), vec![1, 2]);
        assert_eq!(root_choice_warm_up_plan(2), vec![2, 3, 4]);
        assert_eq!(root_choice_warm_up_plan(3), vec![3, 4]);
        assert_eq!(root_choice_warm_up_plan(4), vec![4, 5]);
        for row_write_txg in 1..=30 {
            let plan = root_choice_warm_up_plan(row_write_txg);
            let mut covered = [false; 2];
            for checkpoint_txg in &plan {
                covered[root_choice_device_of_publish(*checkpoint_txg)] = true;
            }
            assert!(
                covered[0] && covered[1],
                "写行 txg {row_write_txg} 的计划 {plan:?} 没覆盖两块盘"
            );
            assert!(
                plan.len() <= 3,
                "写行之后至多两次空发布（几何的结果）：{plan:?}"
            );
        }
    }

    #[test]
    fn a_root_slot_reading_with_the_same_txg_and_the_other_instance_disagrees_with_the_truth() {
        let material_of = |checkpoint_txg: u64, instance: u32| {
            root_material_without_records(
                instance,
                checkpoint_txg,
                NewRootSegment::off_timeline(u64::from(instance), checkpoint_txg),
            )
        };
        let root_of = |material: &RootChoicePublishMaterial| {
            RootRecord::parse_slot(&material.root_bytes, &ROOT_FILESYSTEM_IDENTIFIER)
                .expect("生成的根自证过")
        };
        let instance_one = material_of(7, 1);
        let instance_two = material_of(7, 2);
        assert!(root_slot_reading_agrees_with_the_truth(
            RootRingSlotReading::SelfVerified(root_of(&instance_one)),
            Some(&instance_one)
        ));
        assert!(
            !root_slot_reading_agrees_with_the_truth(
                RootRingSlotReading::SelfVerified(root_of(&instance_one)),
                Some(&instance_two)
            ),
            "同 txg、实例代号互换：对拍必须报不一致"
        );
        assert!(!root_slot_reading_agrees_with_the_truth(
            RootRingSlotReading::SelfVerified(root_of(&instance_one)),
            None
        ));
        assert!(root_slot_reading_agrees_with_the_truth(
            RootRingSlotReading::Bad(BadRootRingSlotReading::NotSelfVerified),
            None
        ));
        assert!(!root_slot_reading_agrees_with_the_truth(
            RootRingSlotReading::Bad(BadRootRingSlotReading::NotSelfVerified),
            Some(&instance_one)
        ));
        assert!(!root_slot_reading_agrees_with_the_truth(
            RootRingSlotReading::Bad(BadRootRingSlotReading::Unreadable),
            None
        ));
    }

    #[test]
    fn the_local_root_choice_rules_pick_as_the_registered_comparisons_on_hand_made_rings() {
        let version = |checkpoint_txg: u64, instance: u32| {
            Some(RootVersion {
                checkpoint_txg,
                instance,
            })
        };
        let ring = [version(100, 2), version(100, 1), version(101, 1)];
        let picked = |rule: LocalRootChoiceRule| rule.chooses_among(&ring).expect("环里有根");
        assert_eq!(
            picked(LocalRootChoiceRule::NewestWins).describe(),
            "txg101/instance1"
        );
        assert_eq!(
            picked(LocalRootChoiceRule::InstanceFirstThenTxg).describe(),
            "txg100/instance2"
        );
        assert_eq!(
            picked(LocalRootChoiceRule::FirstSelfVerifiedWins).describe(),
            "txg100/instance2"
        );
        assert_eq!(
            picked(LocalRootChoiceRule::IgnoresTheInstance).describe(),
            "txg101/instance1"
        );
        let tie = [version(100, 2), version(100, 1)];
        assert_eq!(
            LocalRootChoiceRule::TieTakesTheLowerInstance
                .chooses_among(&tie)
                .expect("环里有根")
                .describe(),
            "txg100/instance1"
        );
        assert_eq!(
            LocalRootChoiceRule::NewestWins
                .chooses_among(&tie)
                .expect("环里有根")
                .describe(),
            "txg100/instance2"
        );
        assert_eq!(
            LocalRootChoiceRule::NewestWins.chooses_among(&[None, None]),
            None
        );
    }

    #[test]
    fn the_root_choice_state_counts_of_the_smallest_cells_equal_the_pinned_values_and_the_kw_wrap_cells_read_four_and_one(
    ) {
        let thread_count = thread_count_from_the_environment();
        for (domain, mounts, user_publishes) in [
            (RootChoiceDomain::BigRing, 1, 0),
            (RootChoiceDomain::BigRing, 1, 1),
            (RootChoiceDomain::BigRing, 1, 2),
            (RootChoiceDomain::BigRing, 2, 0),
            (RootChoiceDomain::BigRing, 2, 1),
            (RootChoiceDomain::SmallRingNine, 1, 0),
            (RootChoiceDomain::SmallRingNine, 2, 1),
            (RootChoiceDomain::WrappingRootRing, 1, 0),
            (RootChoiceDomain::WrappingRootRing, 1, 10),
            (RootChoiceDomain::WrappingRootRing, 1, 11),
        ] {
            let cell = RootChoiceCell {
                mounts,
                user_publishes,
            };
            let tally = root_choice_tally_of_a_cell(
                domain,
                cell,
                RootChoiceImplementation::TodayThreeSteps,
                thread_count,
            );
            assert_the_pinned_root_choice_cell(domain, cell, &tally);
            assert_eq!(
                tally.get(RootChoiceCounter::SlotReadingsDifferingFromTheTruth),
                0,
                "读槽对拍与真相一致：{domain:?} {cell:?}"
            );
            assert_eq!(tally.get(RootChoiceCounter::ChoseNothingStates), 0);
            assert_eq!(tally.get(RootChoiceCounter::PanickedStates), 0);
        }
    }

    #[test]
    fn the_adversarial_root_rings_have_125_states_one_of_them_without_a_self_verified_root_and_zero_kj2_violations_today(
    ) {
        let today = root_choice_tally_of_a_cell(
            RootChoiceDomain::AdversarialRootRing,
            RootChoiceCell {
                mounts: 0,
                user_publishes: 0,
            },
            RootChoiceImplementation::TodayThreeSteps,
            1,
        );
        assert_eq!(
            today.get(RootChoiceCounter::States),
            PINNED_ROOT_CHOICE_ADVERSARIAL_STATES
        );
        assert_eq!(today.get(RootChoiceCounter::ChoseNothingStates), 1);
        assert_eq!(today.get(RootChoiceCounter::Kj0Violations), 1);
        assert_eq!(today.get(RootChoiceCounter::Kj2Judged), 124);
        assert_eq!(today.get(RootChoiceCounter::Kj2Violations), 0);
        assert_eq!(
            today.get(RootChoiceCounter::SlotReadingsDifferingFromTheTruth),
            0
        );
        assert!(today.get(RootChoiceCounter::SameTxgDifferentInstanceRoots) > 0);
        assert!(today.get(RootChoiceCounter::HigherInstanceLowerTxgRoots) > 0);
    }

    /// 每条判据在每个被判的格上各有对照读数、都红；读数条数照格数逐条判据数（删掉某条判据的某一行对照，条数对不上）。
    #[test]
    fn every_root_choice_criterion_has_a_positive_control_and_every_control_is_red() {
        let thread_count = thread_count_from_the_environment();
        let big_ring_cells: Vec<RootChoiceCell> = [(1, 0), (1, 2), (1, 3)]
            .into_iter()
            .map(|(mounts, user_publishes)| RootChoiceCell {
                mounts,
                user_publishes,
            })
            .collect();
        let selected_cells = vec![
            (RootChoiceDomain::BigRing, big_ring_cells.clone()),
            (
                RootChoiceDomain::AdversarialRootRing,
                vec![RootChoiceCell {
                    mounts: 0,
                    user_publishes: 0,
                }],
            ),
        ];
        let readings = root_choice_control_readings(&selected_cells, thread_count);
        for reading in &readings {
            assert!(reading.is_red_as_required, "对照没红：{reading:?}");
        }
        let readings_of = |criterion: RootChoiceCriterion| {
            readings
                .iter()
                .filter(|reading| reading.criterion == criterion)
                .count()
        };
        let cell_count = big_ring_cells.len();
        let local_rule_count = LocalRootChoiceRule::ALL.len();
        assert_eq!(readings_of(RootChoiceCriterion::Kj0Defined), 1);
        assert_eq!(
            readings_of(RootChoiceCriterion::Kj1NotEarlierThanTheLastConfirmedPublish),
            2 * cell_count,
            "KJ1 在每个格上 KF1、KF2 各一条对照"
        );
        assert_eq!(
            readings_of(RootChoiceCriterion::Kj2NotEarlierThanAnySelfVerifiedRoot),
            2 * cell_count + local_rule_count,
            "KJ2 在每个格上 KF1、KF2 各一条，在 KA 上每种本地择根序一条"
        );
        assert_eq!(
            readings_of(RootChoiceCriterion::Kj3LandsOnTheIdealRecovery),
            cell_count,
            "KJ3 在每个格上 KF2 一条"
        );
        for criterion in RootChoiceCriterion::ALL {
            assert!(
                readings
                    .iter()
                    .any(|reading| reading.criterion == *criterion),
                "判据 {} 没有对照的读数",
                criterion.label()
            );
        }
    }

    #[test]
    fn the_root_choice_command_line_takes_the_family_and_rejects_unknown_domains_and_options() {
        let arguments =
            |text: &str| -> Vec<String> { text.split_whitespace().map(str::to_string).collect() };
        assert!(arguments_name_the_root_choice_family(&arguments(
            "--bounds segment1 --family root_choice"
        )));
        assert!(!arguments_name_the_root_choice_family(&arguments(
            "--bounds segment1 --family legal"
        )));
        let selection = parse_the_root_choice_command_line(&arguments(
            "--bounds light --family root_choice --only K9,KA --max-u 1",
        ))
        .expect("认得的参数");
        assert_eq!(selection.bounds, BoundsTier::Light);
        assert_eq!(
            selection.only,
            Some(vec!["K9".to_string(), "KA".to_string()])
        );
        assert_eq!(selection.largest_user_publishes_cap, Some(1));
        assert!(parse_the_root_choice_command_line(&arguments(
            "--bounds segment1 --family root_choice --only L"
        ))
        .is_err());
        assert!(parse_the_root_choice_command_line(&arguments(
            "--bounds segment1 --family root_choice --max-n 3"
        ))
        .is_err());
        assert!(parse_the_root_choice_command_line(&arguments("--family root_choice")).is_err());
    }
}

#[cfg(test)]
mod root_choice_fault_tests {
    use super::*;

    fn cell_of(mounts: usize, user_publishes: usize) -> RootChoiceCell {
        RootChoiceCell {
            mounts,
            user_publishes,
        }
    }

    #[test]
    fn the_mount_fault_menu_has_thirteen_entries_in_the_registered_order_and_only_rnsb_is_outside_the_promise(
    ) {
        let world = FaultWorld::right_after_mkfs(RootChoiceDomain::BigRing);
        let menu = mount_fault_menu(&world, 2);
        let labels: Vec<&str> = menu.iter().map(|entry| entry.label()).collect();
        assert_eq!(labels, MOUNT_FAULT_MENU_LABELS);
        let outside_the_promise: Vec<&str> = menu
            .iter()
            .filter(|entry| !entry.is_inside_the_single_fault_promise())
            .map(|entry| entry.label())
            .collect();
        assert_eq!(outside_the_promise, ["RNSB-T1", "RNSB-TP"]);
        assert_eq!(mount_fault_menu(&world, 1).len(), 1, "挂载 1 开头没有故障");
        for entry in &menu {
            if entry.label().ends_with("-TP") {
                assert_eq!(
                    entry.every_read_unreadable,
                    entry.first_read_unreadable,
                    "TP 每一遍都读不出：{}",
                    entry.label()
                );
            } else {
                assert_eq!(
                    entry.every_read_unreadable,
                    FaultLocations::NOWHERE,
                    "T1 只有第一遍读不出（「无」那一项也没有落点）：{}",
                    entry.label()
                );
            }
        }
    }

    #[test]
    fn the_pinned_text_tables_hold_the_registered_number_of_rows() {
        assert_eq!(pinned_fault_cells().len(), 49);
        assert_eq!(
            PINNED_MOUNT_TWO_FAULT_TEXT.lines().count(),
            4 * MOUNT_FAULT_MENU_LENGTH
        );
    }

    /// 登记 9.2 里生成器一侧那几条变异的取样点：闸恒假、T1 当 TP、写行 txg 只看根、不枚举撕裂第三态、最后一次挂载不并系统配置那 9 个组合、c_见证 = 0 判真、
    /// 已确认算进崩着的那一次、RI 算不罩、盘不在时两块盘的记录都读不出、KJ2 比真相里的根、RN 取键最小的根，都落在这几个格的计数上。
    #[test]
    fn the_generator_counts_of_the_sampling_cells_equal_the_pinned_values() {
        for (domain, mounts, user_publishes) in [
            (FaultDomain::GateOn, 1, 0),
            (FaultDomain::GateOn, 1, 1),
            (FaultDomain::GateOn, 1, 2),
            (FaultDomain::GateOn, 2, 0),
            (FaultDomain::GateOn, 2, 1),
            (FaultDomain::GateOnWrappingRootRing, 1, 11),
            (FaultDomain::GateOff, 1, 2),
        ] {
            let cell = cell_of(mounts, user_publishes);
            let tally = fault_tally_of_a_cell(domain, cell, false, 1);
            let findings = assert_the_pinned_fault_cell(domain, cell, &tally);
            assert!(findings.is_empty(), "{findings:?}");
        }
    }

    #[test]
    fn the_promised_rows_subtract_the_states_the_in_flight_limit_cut() {
        let mut cells = [FaultCellTally::default(); FAULT_CELL_COUNT];
        let covered_without_a_judged_fault = fault_cell_index_of(JudgedFault::Nothing, true);
        cells[covered_without_a_judged_fault].violations[KJ1_INDEX] = 3;
        cells[covered_without_a_judged_fault].cut_violations[KJ1_INDEX] = 3;
        let device_gone = fault_cell_index_of(JudgedFault::DeviceZeroGone, true);
        cells[device_gone].violations[KJ1_INDEX] = 5;
        cells[device_gone].cut_violations[KJ1_INDEX] = 1;
        let instance_roots_gone = fault_cell_index_of(JudgedFault::NewestInstanceRoots, true);
        cells[instance_roots_gone].violations[KJ1_INDEX] = 7;
        assert_eq!(
            promised_violations_from_the_split_rows(&cells, KJ1_INDEX),
            4,
            "罩着的格 3 − 3 加 5 − 1；被判时 RI 不罩，不进承诺内"
        );
    }

    #[test]
    fn a_root_slot_reading_of_a_slot_unreadable_in_the_view_agrees_with_the_truth_only_as_unreadable(
    ) {
        let material = mkfs_root_material();
        let unreadable = RootRingSlotReading::Bad(BadRootRingSlotReading::Unreadable);
        assert!(fault_slot_reading_agrees_with_the_truth(
            unreadable,
            Some(&material),
            true
        ));
        assert!(!fault_slot_reading_agrees_with_the_truth(
            unreadable,
            Some(&material),
            false
        ));
        let zeros = RootRingSlotReading::Bad(BadRootRingSlotReading::NotSelfVerified);
        assert!(!fault_slot_reading_agrees_with_the_truth(zeros, None, true));
        assert!(fault_slot_reading_agrees_with_the_truth(zeros, None, false));
    }

    #[test]
    fn the_local_choice_rules_pick_as_the_registered_comparisons_on_the_adversarial_rings() {
        for (rule, expected) in PINNED_KA_KJ2_OF_THE_LOCAL_RULES {
            assert_eq!(
                ka_kj2_violations_of_a_local_rule(rule),
                expected,
                "{}",
                rule.label()
            );
        }
    }

    #[test]
    fn a_system_configuration_write_takes_the_generation_after_the_largest_one_it_can_read() {
        let mut slots = system_slots_right_after_mkfs();
        let (first_write_slot, first_write_content) =
            system_slot_write_of(&slots, 0, 7, 3, FaultLocations::NOWHERE);
        assert_eq!(first_write_slot, 0);
        assert_eq!(
            first_write_content,
            SystemSlotContent::Verified {
                generation: 2,
                tail: 7,
                instance: 3
            }
        );
        slots[0][0] = first_write_content;
        let (second_write_slot, second_write_content) =
            system_slot_write_of(&slots, 0, 8, 3, FaultLocations::NOWHERE);
        assert_eq!(
            (second_write_slot, second_write_content),
            (
                1,
                SystemSlotContent::Verified {
                    generation: 3,
                    tail: 8,
                    instance: 3
                }
            )
        );
        let unreadable_slot_zero = FaultLocations::NOWHERE.with_system_slot(0, 0);
        let (write_slot_beside_the_unreadable_one, write_content_beside_the_unreadable_one) =
            system_slot_write_of(&slots, 0, 9, 4, unreadable_slot_zero);
        assert_eq!(
            (
                write_slot_beside_the_unreadable_one,
                write_content_beside_the_unreadable_one
            ),
            (
                0,
                SystemSlotContent::Verified {
                    generation: 2,
                    tail: 9,
                    instance: 4
                }
            ),
            "一直读不出的槽不算进最大世代号：只剩世代号 1 的那一槽，写 2"
        );
        slots[1][1] = SystemSlotContent::Torn;
        slots[1][0] = SystemSlotContent::Torn;
        let (_slot, write_content_when_no_slot_reads) =
            system_slot_write_of(&slots, 1, 1, 1, FaultLocations::NOWHERE);
        assert_eq!(
            write_content_when_no_slot_reads,
            SystemSlotContent::Verified {
                generation: 1,
                tail: 1,
                instance: 1
            },
            "一槽都读不出时世代号从 1 起"
        );
    }

    #[test]
    fn the_r4_profile_is_accepted_and_the_faults_family_name_is_recognised() {
        assert!(arguments_name_the_root_choice_faults_family(&[
            "--family".to_string(),
            "root_choice_faults".to_string()
        ]));
        assert!(!arguments_name_the_root_choice_faults_family(&[
            "--family".to_string(),
            "root_choice".to_string()
        ]));
        let selection = parse_the_root_choice_faults_command_line(&[
            "--bounds".to_string(),
            "segment1".to_string(),
            "--family".to_string(),
            "root_choice_faults".to_string(),
            "--only".to_string(),
            "KF,KA".to_string(),
        ])
        .expect("登记的命令行");
        assert_eq!(
            selection.only,
            Some(vec!["KF".to_string(), "KA".to_string()])
        );
        let as_given = |profile: &str| {
            split_off_the_profile_options(&[
                "--profile".to_string(),
                profile.to_string(),
                "--bounds".to_string(),
                "segment1".to_string(),
            ])
        };
        assert_eq!(
            as_given("r4"),
            as_given("r3"),
            "r4 档的其余族与 r3 档相同：只多一族 root_choice_faults"
        );
    }

    /// 登记 9.2 里「一次挂载第一次发布的父取最后一次已确认那一版」与「KJ1T 写成按版本序比」两条变异的取样点：KG 的 M = 2、U = 3 里，挂载 2 开头最新实例的全部根读不出（RI-T1）、
    /// 而挂载 1 有已确认发布的那些单元。挂载 2 往下走的版本比最后一次已确认的更旧，它的发布不在已确认那一版的后代上，KJ1T 与 KJ1 在那里分得开。
    #[test]
    fn the_confirmed_chain_parts_kj1t_from_kj1_when_a_mount_goes_down_from_an_older_version_on_the_gate_off_domain(
    ) {
        let domain = FaultDomain::GateOff;
        let collecting = FaultEnumeration {
            domain,
            mounts: 2,
            stop_before_mount: Some(2),
        };
        let mut collector = FaultUnitCollector { units: Vec::new() };
        collecting.enumerate_a_mount(
            &FaultWorld::right_after_mkfs(domain.geometry()),
            1,
            3,
            None,
            &mut collector,
        );
        let continuing = FaultEnumeration {
            domain,
            mounts: 2,
            stop_before_mount: None,
        };
        let judgement = FaultJudgement::new(domain, 2, false);
        let first_menu_index_of_ri = MOUNT_FAULT_MENU_LABELS
            .iter()
            .position(|label| *label == "RI-T1")
            .expect("RI-T1 在菜单里");
        let mut tally = FaultTally::new();
        for unit in collector.units.iter().filter(|unit| {
            !unit.world.confirmed_user_publishes.is_empty()
                && unit.menu_index == first_menu_index_of_ri
        }) {
            let mut judge = FaultStateJudge {
                judgement: &judgement,
                tally: FaultTally::new(),
                histories_seen_in_this_unit: 0,
                is_the_first_unit: false,
            };
            continuing.enumerate_a_mount(
                &unit.world,
                unit.mount_number,
                unit.user_publishes_left,
                Some(unit.menu_index),
                &mut judge,
            );
            tally.merge(&judge.tally);
        }
        assert!(
            tally.confirmed_states > 0,
            "这些单元的最终态都有已确认的发布"
        );
        assert_eq!(
            tally.kf4[1], tally.confirmed_states,
            "KF4 落在不在已确认那条链上的根里：每个有已确认的态 KJ1T 都违例"
        );
        assert!(
            tally.kf4[0] < tally.kf4[1],
            "挂载 2 的根（版本比已确认的那一版新）不在它的后代上：KJ1 不违例而 KJ1T 违例，两条分得开（KJ1 {}、KJ1T {}）",
            tally.kf4[0],
            tally.kf4[1]
        );
    }

    /// 每条判据在每个被判的格上各有对照读数、都红；读数条数照格数逐条判据数（删掉某条判据的某一行对照，条数对不上）。
    #[test]
    fn every_fault_criterion_has_a_positive_control_and_every_control_is_red() {
        let cells = [
            (FaultDomain::GateOn, cell_of(1, 0)),
            (FaultDomain::GateOn, cell_of(1, 2)),
            (FaultDomain::GateOff, cell_of(1, 2)),
        ];
        let mut readings = Vec::new();
        for (domain, cell) in cells {
            let tally = fault_tally_of_a_cell(domain, cell, true, 1);
            readings.extend(fault_control_readings_of_a_cell(domain, cell, &tally));
        }
        readings.extend(fault_ka_control_readings());
        for reading in &readings {
            assert!(reading.is_red_as_required, "对照没红：{reading:?}");
        }
        let readings_of = |criterion: FaultCriterion| {
            readings
                .iter()
                .filter(|reading| reading.criterion == criterion)
                .count()
        };
        assert_eq!(readings_of(FaultCriterion::Kj0Defined), 1);
        assert_eq!(
            readings_of(FaultCriterion::Kj1NotEarlierThanTheLastConfirmedPublish),
            3 * cells.len(),
            "KJ1 在每个格上 KF1、KF2、KF4 各一条对照"
        );
        assert_eq!(
            readings_of(FaultCriterion::Kj1tDescendsFromTheLastConfirmedPublish),
            3 * cells.len(),
            "KJ1T 在每个格上 KF1、KF2、KF4 各一条对照"
        );
        assert_eq!(
            readings_of(FaultCriterion::Kj2NotEarlierThanAnyReadableRoot),
            2 * cells.len() + PINNED_KA_KJ2_OF_THE_LOCAL_RULES.len(),
            "KJ2 在每个格上 KF1、KF2 各一条，在 KA 上每种本地择根序一条"
        );
        assert_eq!(
            readings_of(FaultCriterion::Kj3LandsOnTheIdealRecovery),
            cells.len(),
            "KJ3 在每个格上 KF2 一条对照"
        );
    }

    // ── r5 档（登记 `research/prompts/e165-r5-prereg.md` 5.3、5.4、第七节乙类、第八节、9.2）

    /// 登记 r5 5.3 第 1 条：第 j 次发布的每条记录带时间线第 j 版 R_j 的后三项，生成器真相里那条记录的投影对到同一份；
    /// r1 档那种生成器上记录的后三项仍是 r1 常量。9.2「r5 档时间线记录的后三项写回 r1 常量」点名这一条。
    #[test]
    fn the_r5_generator_puts_the_three_fields_of_each_publishs_root_into_every_record_of_that_publish(
    ) {
        let shapes = || {
            vec![
                PublishShape {
                    record_count: 1,
                    commit_marker_bits: 0,
                },
                PublishShape {
                    record_count: 2,
                    commit_marker_bits: 0,
                },
                PublishShape {
                    record_count: 1,
                    commit_marker_bits: 0,
                },
            ]
        };
        for marker_parity in [
            MarkerParity::EvenVersionsWrittenByTheUnmountSequence,
            MarkerParity::OddVersionsWrittenByTheUnmountSequence,
        ] {
            let r5_profile = RunProfile::r5(marker_parity);
            let history = generate_history(&history_plan(
                Domain::LegalBigRing,
                ORDINARY_FIRST_TXG,
                shapes(),
                Vec::new(),
                r5_profile,
            ));
            for publish_index in 0..history.publishes.len() {
                let fields_of_the_version =
                    RecordFieldsBeyondTheProjection::of_root(&history.timeline_root(publish_index));
                for record_index in history.record_indexes_of_publish(publish_index) {
                    let parsed = history.slots[record_index]
                        .parsed
                        .as_ref()
                        .expect("合法历史的每条记录都解得出");
                    assert_eq!(
                        RecordFieldsBeyondTheProjection::of_record(parsed),
                        fields_of_the_version,
                        "r5 档（{}）第 {publish_index} 次发布的第 {record_index} 条记录带 R_{publish_index} 的后三项",
                        marker_parity.label()
                    );
                    assert_eq!(
                        history
                            .record_truth
                            .fields_of(VersionFields::of_record(parsed)),
                        Some(fields_of_the_version),
                        "生成器真相里这条记录的投影对到同一份后三项"
                    );
                }
            }
            let first = RecordFieldsBeyondTheProjection::of_root(&history.timeline_root(0));
            let second = RecordFieldsBeyondTheProjection::of_root(&history.timeline_root(1));
            assert!(
                first.instance_table != second.instance_table
                    && first.allocation_record_tree_root != second.allocation_record_tree_root
                    && first.unmount_marker != second.unmount_marker,
                "R_0 与 R_1 的后三项逐项不同（C8a 的钉值靠这一条）"
            );
        }
        let r1_constant = RecordFieldsBeyondTheProjection::of_root_fields(
            fields_beyond_the_projection_carried_by_every_generated_record(),
        );
        let r1_generator_history = generate_history(&history_plan(
            Domain::LegalBigRing,
            ORDINARY_FIRST_TXG,
            shapes(),
            Vec::new(),
            RunProfile::R1_GENERATOR_WITH_R5_CRITERIA,
        ));
        for slot in &r1_generator_history.slots {
            let parsed = slot.parsed.as_ref().expect("合法历史的每条记录都解得出");
            assert_eq!(
                RecordFieldsBeyondTheProjection::of_record(parsed),
                r1_constant,
                "r1 档那种生成器上记录的后三项恒为 r1 常量"
            );
        }
    }

    /// 登记 r5 5.3 第 2 条：时间线之外的记录（对抗一族字母表里的、R 域的残留）后三项 h(d) 的 (i) 实例表指针与分配记录树根各是 d 的单射、
    /// 与时间线第 0..12 版的同一字段两两不同；(ii) 卸载记号取与 R_0 相反的那一值（A 域 n = 1 第一个字母那条记录也如此）；(iii) 过 H4 往返
    /// （A 域 n = 1、R 域 n = 2 整格生成一遍，生成时断言往返）。9.2「h(d) 的卸载记号取与 R_0 相同的值」点名这一条。
    #[test]
    fn the_records_off_the_timeline_carry_tails_distinct_from_every_version_and_the_unmount_marker_opposite_to_the_first_version(
    ) {
        let alphabet = adversarial_record_alphabet();
        let mut distinguishing_numbers: Vec<u64> = (0..12).collect();
        for position in 1..=4usize {
            for alphabet_index in 0..alphabet.len() {
                distinguishing_numbers
                    .push(u64::try_from(position * 1000 + alphabet_index).expect("很小"));
            }
        }
        for marker_parity in [
            MarkerParity::EvenVersionsWrittenByTheUnmountSequence,
            MarkerParity::OddVersionsWrittenByTheUnmountSequence,
        ] {
            let r5_profile = RunProfile::r5(marker_parity);
            let first_version_marker = r5_profile
                .fields_beyond_the_projection_of_version(0)
                .unmount_marker;
            let mut instance_table_slots = std::collections::BTreeSet::new();
            let mut allocation_record_tree_root_slots = std::collections::BTreeSet::new();
            for distinguishing_number in &distinguishing_numbers {
                let tail = r5_profile.record_fields_off_the_timeline(*distinguishing_number);
                assert_ne!(
                    tail.unmount_marker,
                    first_version_marker,
                    "（{}）d = {distinguishing_number} 的卸载记号与 R_0 相反",
                    marker_parity.label()
                );
                for version in 0..=12usize {
                    let fields_of_the_version =
                        r5_profile.fields_beyond_the_projection_of_version(version);
                    assert!(
                        tail.instance_table != fields_of_the_version.instance_table
                            && tail.allocation_record_tree_root
                                != fields_of_the_version.allocation_record_tree_root,
                        "d = {distinguishing_number} 的两条指针与时间线第 {version} 版的同一字段不同"
                    );
                }
                instance_table_slots.insert(tail.instance_table.locations[0].slot.0);
                allocation_record_tree_root_slots
                    .insert(tail.allocation_record_tree_root.locations[0].slot.0);
            }
            assert_eq!(
                instance_table_slots.len(),
                distinguishing_numbers.len(),
                "实例表指针对 d 单射"
            );
            assert_eq!(
                allocation_record_tree_root_slots.len(),
                distinguishing_numbers.len(),
                "分配记录树根对 d 单射"
            );
            let chosen_root = adversarial_chosen_root(r5_profile);
            let (_projection, first_letter_tail) = adversarial_record_projection_and_fields(
                &chosen_root,
                alphabet[0],
                1,
                0,
                r5_profile,
            );
            assert_ne!(
                first_letter_tail.unmount_marker, chosen_root.unmount_marker,
                "A 域 n = 1 第一个字母那条记录的卸载记号与所选根 R_0 相反"
            );
            for (domain, size) in [(Domain::AdversarialBigRing, 1), (Domain::Residue, 2)] {
                let tally = tally_of_a_domain_size(
                    domain,
                    size,
                    Implementation::TodayReplayJournal,
                    r5_profile,
                    1,
                );
                assert!(
                    tally.get(Counter::States) > 0,
                    "{} 域 n = {size} 生成过",
                    domain.label()
                );
            }
        }
        let r1_constant = RecordFieldsBeyondTheProjection::of_root_fields(
            fields_beyond_the_projection_carried_by_every_generated_record(),
        );
        assert_eq!(
            RunProfile::R1_GENERATOR_WITH_R5_CRITERIA.record_fields_off_the_timeline(1000),
            r1_constant,
            "r1 档那种生成器上时间线之外的记录也带 r1 常量"
        );
        assert_eq!(
            RunProfile::r3(MarkerParity::EvenVersionsWrittenByTheUnmountSequence)
                .record_fields_off_the_timeline(1000),
            r1_constant,
            "r1–r4 档一个字不改：时间线之外的记录带 r1 常量"
        );
    }

    /// C8a 的钉值（登记 r5 5.4、第七节乙类）：`even` 下 L 域 N ≤ 5 上 J8a 三格 762、762、620，J8b、J8c 0，r1 档那种生成器上三格 0；
    /// `odd` 下 L 域同值（第 j 版与第 0 版的记号不同 ⟺ j 为奇数，两种奇偶都如此），C 域 Nc ≤ 3 两条指针那两格大于 0。
    /// 9.2「r5 档时间线记录的后三项写回 r1 常量」点名这一条。
    #[test]
    fn the_c8a_control_reads_762_762_620_on_the_r5_generator_and_0_on_the_r1_generator() {
        assert_eq!(
            FARTHEST_AT_LEAST_ONE_STATES_WITHIN_THE_CONTROL_BOUND,
            FARTHEST_IS_ODD_STATES_WITHIN_THE_CONTROL_BOUND
                + FARTHEST_IS_EVEN_AND_AT_LEAST_TWO_STATES_WITHIN_THE_CONTROL_BOUND,
            "登记 r5 第七节乙类：620 + 142 = 762"
        );
        for marker_parity in [
            MarkerParity::EvenVersionsWrittenByTheUnmountSequence,
            MarkerParity::OddVersionsWrittenByTheUnmountSequence,
        ] {
            let readings = control_readings_of(
                Criterion::J8aFieldsFromTheNewRootSegment,
                RunProfile::r5(marker_parity),
                1,
            );
            assert_eq!(
                readings
                    .iter()
                    .take(8)
                    .map(|reading| reading.value)
                    .collect::<Vec<u64>>(),
                vec![762, 762, 620, 0, 0, 0, 0, 0],
                "C8a（{}）的对照读数：{readings:?}",
                marker_parity.label()
            );
            assert!(
                readings.iter().all(|reading| reading.is_red_as_required),
                "C8a（{}）的对照都照登记红：{readings:?}",
                marker_parity.label()
            );
        }
    }

    /// C8b 的钉值与第八节奇偶的判别力自证（登记 r5 5.4、第七节乙类、第八节）：`even` 142、`odd` 620，r1 档那种生成器上 0；把门槛挪到
    /// 142 与 620 之间（判「卸载记号那一格 ≥ 400」），`even` 那一点由绿转红、`odd` 那一点不动。9.2「C8b 的卸载记号写 Written（写反）」点名这一条。
    #[test]
    fn the_c8b_control_reads_142_on_even_and_620_on_odd_and_a_threshold_of_400_parts_the_two_parities(
    ) {
        let readings_on = |marker_parity: MarkerParity| {
            control_readings_of(
                Criterion::J8aUnmountMarkerFromTheNewRootSegment,
                RunProfile::r5(marker_parity),
                1,
            )
        };
        let even = readings_on(MarkerParity::EvenVersionsWrittenByTheUnmountSequence);
        let odd = readings_on(MarkerParity::OddVersionsWrittenByTheUnmountSequence);
        assert_eq!(
            even.iter()
                .map(|reading| reading.value)
                .collect::<Vec<u64>>(),
            vec![142, 0],
            "C8b even：{even:?}"
        );
        assert_eq!(
            odd.iter()
                .map(|reading| reading.value)
                .collect::<Vec<u64>>(),
            vec![620, 0],
            "C8b odd：{odd:?}"
        );
        assert!(
            even.iter()
                .chain(&odd)
                .all(|reading| reading.is_red_as_required),
            "C8b 两种奇偶都照登记红"
        );
        let threshold_between_the_two_points = 400;
        assert!(
            even[0].value < threshold_between_the_two_points
                && odd[0].value >= threshold_between_the_two_points,
            "判别力自证：门槛 400 上 even 那一点红、odd 那一点不动（{} 与 {}）",
            even[0].value,
            odd[0].value
        );
    }

    /// C8c 的钉值（登记 r5 5.4、第七节乙类）：L 域 N ≤ 5 上 J8b 762、J8a 三格 0，r1 档那种生成器上 J8b 0；`odd` 下 C 域 Nc ≤ 3 上 J8b 大于 0。
    /// 9.2「J8b 拿落到的那一版的 fsid 当期望」点名这一条。
    #[test]
    fn the_c8c_control_reads_762_on_j8b_and_0_on_the_three_j8a_cells_and_on_the_r1_generator() {
        for marker_parity in [
            MarkerParity::EvenVersionsWrittenByTheUnmountSequence,
            MarkerParity::OddVersionsWrittenByTheUnmountSequence,
        ] {
            let readings = control_readings_of(
                Criterion::J8bFilesystemIdentifierOfTheChosenRoot,
                RunProfile::r5(marker_parity),
                1,
            );
            assert_eq!(
                readings
                    .iter()
                    .take(5)
                    .map(|reading| reading.value)
                    .collect::<Vec<u64>>(),
                vec![762, 0, 0, 0, 0],
                "C8c（{}）：{readings:?}",
                marker_parity.label()
            );
            assert!(
                readings.iter().all(|reading| reading.is_red_as_required),
                "C8c（{}）都照登记红：{readings:?}",
                marker_parity.label()
            );
        }
    }

    /// C8d 的钉值（登记 r5 5.4、第七节乙类）：`even` 下 L 域 N ≤ 5 上 J8c 2 348、`odd` 下 0，r1 档那种生成器上 0；`odd` 下 C 域 Nc ≤ 3 上大于 0。
    #[test]
    fn the_c8d_control_reads_2348_on_even_0_on_odd_and_above_0_on_the_monotone_domain_under_odd() {
        let even = control_readings_of(
            Criterion::J8cNothingAppliedReturnsTheChosenRoot,
            RunProfile::r5(MarkerParity::EvenVersionsWrittenByTheUnmountSequence),
            1,
        );
        let odd = control_readings_of(
            Criterion::J8cNothingAppliedReturnsTheChosenRoot,
            RunProfile::r5(MarkerParity::OddVersionsWrittenByTheUnmountSequence),
            1,
        );
        assert_eq!(
            even.iter()
                .map(|reading| reading.value)
                .collect::<Vec<u64>>(),
            vec![2_348, 0],
            "C8d even：{even:?}"
        );
        assert_eq!(
            odd.iter()
                .take(2)
                .map(|reading| reading.value)
                .collect::<Vec<u64>>(),
            vec![0, 0],
            "C8d odd：{odd:?}"
        );
        assert_eq!(odd.len(), 3, "odd 另有 C 域那一条：{odd:?}");
        assert!(
            even.iter()
                .chain(&odd)
                .all(|reading| reading.is_red_as_required),
            "C8d 两种奇偶都照登记红"
        );
    }

    /// 今天的 `replay_journal` 在 r5 档 L 域 N ≤ 2 上 J8 每格 0、施加过的态有判到、「无出处」0（调真实现，同 r2 登记修订 4 挪进单测）。
    /// 9.2「J8a 拿所选根当期望（回到旧规则）」点名这一条：那一变异在 N = 2、P_1 落了那一态上 J8a 两格各 1。
    #[test]
    fn todays_replay_journal_breaks_no_j8_cell_on_the_big_ring_up_to_two_records() {
        let mut judged_rebuilt_states = 0;
        for size in 1..=2 {
            let tally = tally_of_a_domain_size(
                Domain::LegalBigRing,
                size,
                Implementation::TodayReplayJournal,
                RunProfile::r5(MarkerParity::EvenVersionsWrittenByTheUnmountSequence),
                1,
            );
            for counter in [
                Counter::J8aInstanceTableViolations,
                Counter::J8aAllocationRecordTreeRootViolations,
                Counter::J8aUnmountMarkerViolations,
                Counter::J8bViolations,
                Counter::J8cViolations,
                Counter::J8WithoutASource,
            ] {
                assert_eq!(
                    tally.get(counter),
                    0,
                    "L 域 N = {size} 上 {} 是 0：{:?}",
                    counter.label(),
                    tally.witness(counter)
                );
            }
            judged_rebuilt_states += tally.get(Counter::J8aJudged);
        }
        assert!(
            judged_rebuilt_states > 0,
            "L 域 N ≤ 2 里有施加过、判到 J8a 的态"
        );
    }

    /// J8c 手造（登记 r5 9.2「J8c 只比卸载记号」那一行的取样点）：一条没施加、返回根的实例表指针被改过，J8c 记一条违例；原样交回的不记。
    /// 另一格：投影不在生成器真相里的返回根记「无出处」、不判 J8a。
    #[test]
    fn j8c_counts_a_returned_root_that_kept_the_projection_but_changed_the_instance_table() {
        let r5_profile = RunProfile::r5(MarkerParity::EvenVersionsWrittenByTheUnmountSequence);
        let chosen_root = adversarial_chosen_root(r5_profile);
        let truth =
            adversarial_record_truth(1, &adversarial_record_alphabet(), &chosen_root, r5_profile);
        let judge = |returned_root: &RootRecord| {
            let mut tally = Tally::new();
            judge_the_fields_beyond_the_projection_against_the_generated_records(
                &chosen_root,
                returned_root,
                &truth,
                None,
                &|| "hand_made".to_string(),
                &mut tally,
            );
            tally
        };
        let untouched = judge(&chosen_root);
        assert_eq!(
            (
                untouched.get(Counter::J8cJudged),
                untouched.get(Counter::J8cViolations),
                untouched.get(Counter::J8bViolations)
            ),
            (1, 0, 0),
            "原样交回所选根：J8c 判到、不违例"
        );
        let mut changed_instance_table = chosen_root;
        changed_instance_table.instance_table =
            node_pointer_at_slot(INSTANCE_TABLE_SLOT_OF_THE_FIRST_VERSION + 5, 1);
        let changed = judge(&changed_instance_table);
        assert_eq!(
            (
                changed.get(Counter::J8cJudged),
                changed.get(Counter::J8cViolations),
                changed.get(Counter::J8aJudged)
            ),
            (1, 1, 0),
            "一条没施加而实例表指针变了：J8c 记一条"
        );
        let mut without_a_source = chosen_root;
        without_a_source.tree_identifier_watermark = OFF_TIMELINE_WATERMARK_BASE + 999_999;
        let unsourced = judge(&without_a_source);
        assert_eq!(
            (
                unsourced.get(Counter::J8WithoutASource),
                unsourced.get(Counter::J8aJudged),
                unsourced.get(Counter::J8cJudged)
            ),
            (1, 0, 0),
            "投影不在生成器真相里：记无出处，J8a、J8c 都不判"
        );
    }

    /// C8e（登记 r5 5.4、停机条款 PH6）：手造一份真相，同一投影两份不同的后三项，必须停在 PH6。
    /// 9.2「真相表同一投影两份后三项时不停、取先到的那一份」点名这一条。
    #[test]
    #[should_panic(expected = "停机条款 PH6")]
    fn a_generated_record_truth_given_two_tails_for_one_projection_halts_on_ph6() {
        let r5_profile = RunProfile::r5(MarkerParity::EvenVersionsWrittenByTheUnmountSequence);
        let projection = VersionFields::of_root(&adversarial_chosen_root(r5_profile));
        let mut truth = GeneratedRecordTruth::new();
        truth.note(
            projection,
            r5_profile.record_fields_of_the_timeline_publish(0),
        );
        truth.note(
            projection,
            r5_profile.record_fields_of_the_timeline_publish(1),
        );
    }

    // ── r5 修订 1 第五段（`--gate r2`；登记 `research/prompts/e165-r5-prereg.md` 修1.七甲类、修1.八、修1.九甲表）

    /// 一份手造的系统配置槽：`torn` 里列的 (盘, 槽) 撕裂，其余是 mkfs 那一份自证过的。
    fn system_slots_with_torn(torn: &[(usize, usize)]) -> SystemSlots {
        let mut slots = system_slots_right_after_mkfs();
        for (device, slot) in torn {
            slots[*device][*slot] = SystemSlotContent::Torn;
        }
        slots
    }

    /// 读视图里报读错的 (盘, 槽)。
    fn locations_with_read_errors(read_errors: &[(usize, usize)]) -> FaultLocations {
        read_errors
            .iter()
            .fold(FaultLocations::NOWHERE, |locations, (device, slot)| {
                locations.with_system_slot(*device, *slot)
            })
    }

    /// 修1.七甲类「判不出的真值」（D22（单元原子性怎么合成） 已定项 8 射程 R2 原句与 S-辨那一句）：一槽撕裂而没有槽报读错 ⇒ 判得出；
    /// 一块盘报读错而另一块盘两槽都自证过 ⇒ 判得出；两块盘各有一槽报读错 ⇒ 判不出；一块盘报读错、另一块盘一槽撕裂 ⇒ 判不出。
    /// 9.2「撕裂也算报读错」「R2 后一半拿掉」「两槽都自证过不看撕裂」点名这一条。
    #[test]
    fn the_r2_undecidability_holds_only_when_a_slot_reports_a_read_error_and_no_device_self_verified_both(
    ) {
        for (torn, read_errors, undecidable, case) in [
            (
                vec![(0, 0), (1, 0)],
                vec![],
                false,
                "两块盘各一槽撕裂、没有槽报读错",
            ),
            (
                vec![],
                vec![(0, 0)],
                false,
                "0 号盘一槽报读错、1 号盘两槽都自证过",
            ),
            (vec![], vec![(0, 0), (1, 1)], true, "两块盘各一槽报读错"),
            (
                vec![(1, 0)],
                vec![(0, 0)],
                true,
                "0 号盘一槽报读错、1 号盘一槽撕裂",
            ),
            (vec![], vec![], false, "全都自证过"),
        ] {
            assert_eq!(
                system_configuration_witness_is_undecidable_under_r2(
                    &system_slots_with_torn(&torn),
                    locations_with_read_errors(&read_errors)
                ),
                undecidable,
                "{case}"
            );
        }
    }

    /// 修1.七甲类「取号那一刻：某块盘两槽一份自证过的都没有 ⇒ 拒（即使判得出）」：0 号盘一槽报读错、另一槽撕裂，1 号盘两槽都自证过。
    /// 9.2「取号那一刻不判 (b)」点名这一条。
    #[test]
    fn acquisition_refuses_a_device_without_a_self_verified_slot_even_when_the_witness_is_decidable(
    ) {
        let mut world = FaultWorld::right_after_mkfs_under_the_gate(
            RootChoiceDomain::BigRing,
            WitnessGateRule::SomeSlotReportsAReadErrorAndNoDeviceSelfVerifiedBothSlots,
        );
        world.system_slots = system_slots_with_torn(&[(0, 1)]);
        let locations = locations_with_read_errors(&[(0, 0)]);
        assert!(!system_configuration_witness_is_undecidable_under_r2(
            &world.system_slots,
            locations
        ));
        assert_eq!(
            acquisition_refusal_of(&world, locations),
            Some(MountRefusalReason::AcquisitionDeviceWithoutASelfVerifiedSlot)
        );
    }

    /// 修1.九甲表「取号那一刻不判 (a)」的手造单测（界内没有取样点）：跳过闸，直接对「两块盘各一槽报读错」的读视图判取号那一刻，必须拒在判不出那一道。
    #[test]
    fn acquisition_refuses_an_undecidable_witness_when_each_device_reports_a_read_error_on_one_slot(
    ) {
        let world = FaultWorld::right_after_mkfs_under_the_gate(
            RootChoiceDomain::BigRing,
            WitnessGateRule::SomeSlotReportsAReadErrorAndNoDeviceSelfVerifiedBothSlots,
        );
        assert_eq!(
            acquisition_refusal_of(&world, locations_with_read_errors(&[(0, 0), (1, 0)])),
            Some(MountRefusalReason::AcquisitionWitnessUndecidable)
        );
        assert_eq!(
            acquisition_refusal_of(&world, FaultLocations::NOWHERE),
            None,
            "四槽都自证过时取号那一刻不拒"
        );
    }

    /// `--gate r2` 解析出的闸（登记 r5 修订 1）上，KF M = 2、U = 0 那一格的生成器计数与拒因三栏等于修1.七乙类（历史 3 094、被拒 99、
    /// 走了重读 1 130、带撕裂 245；拒因 91、8、0）。9.2「R2 后一半拿掉」「两槽都自证过不看撕裂」「取号那一刻不判 (b)」「c_见证 = 0 判真」
    /// 「--gate r2 认了但闸照取甲」点名这一条。
    #[test]
    fn the_r2_generator_counts_of_the_kf_two_mount_cell_equal_the_r2_pinned_values() {
        let selection = parse_the_root_choice_faults_command_line(
            &[
                "--bounds",
                "segment1",
                "--family",
                "root_choice_faults",
                "--gate",
                "r2",
            ]
            .iter()
            .map(|word| (*word).to_string())
            .collect::<Vec<String>>(),
        )
        .expect("登记的命令行");
        let cell = cell_of(2, 0);
        let tally = fault_tally_of_a_cell_under_the_gate(
            FaultDomain::GateOn,
            cell,
            selection.witness_gate_rule,
            false,
            1,
        );
        assert_eq!(
            (tally.histories, tally.refused_mounts, tally.reread_mounts),
            (3_094, 99, 1_130),
            "KF M = 2、U = 0 在 R2 下的历史数、被拒数、走了重读的数"
        );
        let findings = assert_the_pinned_fault_cell_under_the_gate(
            selection.witness_gate_rule,
            FaultDomain::GateOn,
            cell,
            &tally,
        );
        assert!(findings.is_empty(), "{findings:?}");
    }

    /// 修1.七乙类「拒因分计」KF M = 2、U = 0 那一格：闸拒 91、取号那一刻 (b) 8、取号那一刻 (a) 0，三栏之和等于被拒的 99。
    /// 9.2「拒因记错：取号那一刻 (b) 的拒记进闸那一栏」点名这一条。
    #[test]
    fn the_r2_refusals_of_the_kf_two_mount_cell_split_into_91_at_the_gate_8_without_a_self_verified_slot_and_0_undecidable(
    ) {
        let tally = fault_tally_of_a_cell_under_the_gate(
            FaultDomain::GateOn,
            cell_of(2, 0),
            WitnessGateRule::SomeSlotReportsAReadErrorAndNoDeviceSelfVerifiedBothSlots,
            false,
            1,
        );
        assert_eq!(
            [
                tally.refusals_by_reason[MountRefusalReason::TheGateAfterTheOneReread.index()],
                tally.refusals_by_reason
                    [MountRefusalReason::AcquisitionDeviceWithoutASelfVerifiedSlot.index()],
                tally.refusals_by_reason[MountRefusalReason::AcquisitionWitnessUndecidable.index()],
                tally.refusals_by_reason
                    [MountRefusalReason::AcquisitionUnderTheImplementationChoice.index()],
            ],
            [91, 8, 0, 0]
        );
        assert_eq!(tally.refused_mounts, 91 + 8);
    }

    /// 修1.八判别力自证第 1 条（门槛挪到两点之间：「SN0-TP 的历史全被拒」）：KF、KG 的 M = 2、U = 0 一格上，R2 的闸下 R2 那一条为真、取甲那一条为假；
    /// 取甲的闸下反过来。9.2「R2 档的判别力自证仍用取甲那四条」点名这一条。
    #[test]
    fn the_r2_gate_refusal_proof_holds_under_r2_and_the_implementation_choice_proof_holds_only_under_the_implementation_choice(
    ) {
        for (witness_gate_rule, rule_label) in [
            (
                WitnessGateRule::SomeSlotReportsAReadErrorAndNoDeviceSelfVerifiedBothSlots,
                "R2",
            ),
            (
                WitnessGateRule::OneUnreadableOrTornSlotMeansUndecidable,
                "取甲",
            ),
        ] {
            let gate_on = fault_tally_of_a_cell_under_the_gate(
                FaultDomain::GateOn,
                cell_of(2, 0),
                witness_gate_rule,
                false,
                1,
            );
            let gate_off = fault_tally_of_a_cell_under_the_gate(
                FaultDomain::GateOff,
                cell_of(2, 0),
                witness_gate_rule,
                false,
                1,
            );
            let (_item, own_proof_holds, own_detail) =
                gate_refusal_proof_of(witness_gate_rule, &gate_on, &gate_off);
            assert!(
                own_proof_holds,
                "{rule_label} 的闸下它自己那一条为真：{own_detail}"
            );
            let other_proof_holds = match witness_gate_rule {
                WitnessGateRule::SomeSlotReportsAReadErrorAndNoDeviceSelfVerifiedBothSlots => {
                    gate_refusal_proof_under_the_implementation_choice(&gate_on, &gate_off).1
                }
                WitnessGateRule::OneUnreadableOrTornSlotMeansUndecidable => {
                    gate_refusal_proof_under_r2(&gate_on, &gate_off).1
                }
            };
            assert!(
                !other_proof_holds,
                "{rule_label} 的闸下另一种取法那一条为假"
            );
        }
    }

    /// `--gate r2` 只在 `--family root_choice_faults` 那一支认（别的族的参数解析报用法错），解出 R2；不给是取甲，config 行的闸字段与 r4 档逐字相同；
    /// 别的值报错。9.2「--gate r2 认了但闸照取甲」点名这一条（config 行带 R2 的闸字段）。
    #[test]
    fn the_gate_option_names_the_r2_gate_in_the_config_row_only_on_the_faults_family() {
        let words = |list: &[&str]| {
            list.iter()
                .map(|word| (*word).to_string())
                .collect::<Vec<String>>()
        };
        let with_r2 = parse_the_root_choice_faults_command_line(&words(&[
            "--bounds",
            "segment1",
            "--family",
            "root_choice_faults",
            "--gate",
            "r2",
        ]))
        .expect("登记的命令行");
        assert_eq!(
            with_r2.witness_gate_rule.label(),
            "r2_some_slot_reports_a_read_error_and_no_device_self_verified_both_slots",
            "--gate r2 的 config 行带 R2 的闸字段"
        );
        let without = parse_the_root_choice_faults_command_line(&words(&[
            "--bounds",
            "segment1",
            "--family",
            "root_choice_faults",
        ]))
        .expect("登记的命令行");
        assert_eq!(
            without.witness_gate_rule.label(),
            "the_implementation_choice_one_unreadable_or_torn_slot_means_undecidable",
            "不给 --gate 是取甲，config 行与 r4 档逐字相同"
        );
        assert!(parse_the_root_choice_faults_command_line(&words(&[
            "--bounds", "segment1", "--gate", "a"
        ]))
        .is_err());
        assert!(
            parse_the_command_line(&words(&[
                "--bounds", "segment1", "--family", "legal", "--gate", "r2"
            ]))
            .is_err(),
            "合法历史一族不认 --gate"
        );
    }
}
