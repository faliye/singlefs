//! E198（取号跳过认出生代号的前提）：入库装置。跑前登记 `research/prompts/e198-preregistration.md`。
//!
//! 回答问题单 `research/prompts/m3-close-b19-c699-forks.md` 两行：第 1 行（加第四条件「R 的实例表指针出生 txg 等于 R.txg」之后，
//! 「行 + R 的实例表指针出生代号」能不能把取号跳过与存在过的版本分全）与第 2 行（C698 两组镜像 nrb、rs1 上改后的 I-3.5 一半与今天的 I-3.9）。
//!
//! 只读、只驱动、只观测：只调 `crates/` 的公开入口造历史（`singlefs_core` 的 mkfs、可写挂载、各条发布与回退）与读盘
//! （`singlefs_core::recovery` 的读根环、读实例表、扫环，`singlefs_checker` 的系统配置槽、根环读法与 `check_pool_image`），
//! 不改 `crates/singlefs-core`、`crates/singlefs-checker`。两条判法（今天只认行的 `today`、加第四条件的 `with_pointer_birth`）、
//! 候选集、洞对与真值账本都写在本文件里，不调、不抄 `walk.rs` 的私有函数（它们没有公开入口，登记第三节）。
//!
//! 臂的名字（产物里写这些，登记里的叫法对应如下）：H1-A = `today`；甲 = `with_pointer_birth`。
//!
//! 几何、工作量与判据门槛一律用本文件的本地常量，值抄自跑前登记第五节并注明出处；`constant_checks` 逐个与实现侧回比，
//! 不等就是停机条款 S4（登记第十一节），整趟不出结论。
//!
//! 模式（第一个参数）：
//!   `checks`                 本地常量回比（S4）与不建历史的锚点（B1–B6 的闭式张数、B8），不建历史
//!   `segment-one`            第一段（登记第五节「分段」）：nrb、rs1 与几何敏感性 G4，逐张判两条臂的分类、跑 `check_pool_image` 取 I-3.9 与 I-3.5
//!   `segment-two`            第二段：ss、skr、sw-crash、sw-read 与几何敏感性 G1–G3，算前提 P1–P3、E1 / E2、清扫触发计数与洞对覆盖
//!   `segment-three`          第三段（附带）：sw-none（问题单两行都够判时不跑，登记第五节）
//!   `stop-clauses <段名>`    只建那一段的历史、只判停机条款 S1、S2、S3、S5 与 S4（登记 3b：停机条款在产物之前跑），不打分类
//!   `list-items <段名>`      逐行列出那一段的历史名，不建历史
//!
//! 环境变量：`E198_THREADS`（工作线程数，没设取每台默认重活线程数：task-server 起任务时设好的 `SINGLEFS_THREAD_CAP`，
//! 再没设取 `.claude/heavy-thread-default` 的登记值）。
//!
//! 产物按 `E7RESULT` 行打到标准输出：历史全部建完、判完之后按历史名的次序一次打出（与线程数无关，逐字节可比）；进度与段用时打到标准错误。
//!
//! 准入写 `always`：复跑登记（`replay.sh` 的 `driver_e198` 两行）要在输入不变时再跑同一段、核产物逐字节相同。

// admission: always 复跑登记要在输入不变时再跑同一段、比逐字节相同的产物，上一次跑完不替这一次作保
// run-condition: command python3

#![forbid(unsafe_code)]

use std::collections::{BTreeMap, BTreeSet};
use std::io::Write as _;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};

use singlefs_checker::image::{verified_system_configuration_slots, InvariantVerdict};
use singlefs_checker::walk::{
    check_pool_image, journal_records_as_the_recovery_reads_them,
    root_ring_the_ring_judgements_read,
};
use singlefs_core::address::{CheckpointTxg, DeviceIdentity, InstanceGeneration};
use singlefs_core::admission::SpaceAdmission;
use singlefs_core::allocator::PoolAllocator;
use singlefs_core::block_device::PhysicalBlockSizeInBytes;
use singlefs_core::instance_table_sweep::{
    InstanceTableRowCountThreshold, InstanceTableSweepAtTheEndOfTheMount,
    InstanceTableSweepConfiguration, InstanceTableSweepOutcome, InstanceTableSweepSwitch,
    SweepPublishPlacements,
};
use singlefs_core::journal::{back_chain_of, JournalRecordPlaceInPublish};
use singlefs_core::make_filesystem::{make_filesystem, MakeFilesystemParameters};
use singlefs_core::mount::{
    mount_writable, mount_writable_with_instance_table_sweep_configuration,
    roll_back_by_a_forward_publish, MountOutput, Mounted, RollbackTarget, ShadowLedger,
};
use singlefs_core::pointer::NodePointer;
use singlefs_core::recovery::{
    choose_system_configuration, instance_table_of_root, readable_roots_with_ring_slots,
    scan_journal,
};
use singlefs_core::root_record::RootRecord;
use singlefs_core::root_ring::{
    slot_offset, target_for_publish, RootRingSlot, RootRingSlotsPerRegion,
};
use singlefs_core::system_configuration::SystemImmutableSizes;
use singlefs_core::transaction::{
    publish_first_file, publish_new_inodes, publish_overwrite, publish_sequential_write,
    publish_version, FirstFile, InstanceTablePlan, PoolVersion, PoolWriter, PublishPlan,
    TransactionOutput,
};
use singlefs_verification::heavy_threads::own_variable_or_default_heavy_threads;
use singlefs_verification::memory_pool::{MemoryPool, SparseBlockDevice};
use singlefs_verification::segments::{FixedGeometry, StepKind};
use singlefs_verification::{
    RecordedOperationKind, RecordingBlockDevice, RetainedOperation, SharedStream,
};

// ============================================================================
// 一、本地常量：值抄自 `research/prompts/e198-preregistration.md` 第五节「本地常量表」与「几何」一段（登记修订见第十二节）。
//    回比在 `constant_checks`（停机条款 S4）。
// ============================================================================

/// 根记录里实例表指针的偏移：登记第五节本地常量表第 1 行（`crates/singlefs-core/src/root_record.rs` 里 `spec_decode_at(bytes, 170)` 那一处）。
const ROOT_RECORD_INSTANCE_TABLE_POINTER_OFFSET: usize = 170;
/// 指针里出生 txg 的偏移：登记第五节本地常量表第 2 行（`crates/singlefs-checker/src/image.rs` 里 `read_u64(bytes, 42)` 那一处）。
const POINTER_BIRTH_TXG_OFFSET: usize = 42;
/// 出生 txg 字段的字节数（小端 u64）：登记第七节 B7「[212, 220)」。
const POINTER_BIRTH_TXG_BYTES: usize = 8;
/// 根环区域数：登记第五节本地常量表第 3 行（`singlefs_format::ROOT_RING_REGIONS`）。
const ROOT_RING_REGIONS: u64 = 3;
/// 区域到盘：登记第五节「几何」一段「区域到盘 [0, 1, 0]」。
const REGION_DEVICES: [u32; 3] = [0, 1, 0];
/// 清扫阈值的产品默认：365（登记第五节本地常量表第 4 行、登记第十二节修订），
/// 与 `InstanceTableSweepConfiguration::PRODUCT` 的阈值回比，不等即 S4 停机（C690（清扫触发阈值的默认值没定））。
const SWEEP_THRESHOLD_PRODUCT_DEFAULT: u64 = 365;
/// 清扫阈值「有一行就扫」：登记第五节本地常量表第 5 行（`InstanceTableRowCountThreshold::ANY_ROW`）。
const SWEEP_THRESHOLD_ANY_ROW: u64 = 0;
/// 一片实例表的记录数：登记第五节本地常量表第 6 行（`singlefs_format::INSTANCE_TABLE_PAGE_RECORDS`）。
const INSTANCE_TABLE_PAGE_RECORDS: u64 = 370;
/// mkfs 那一版的 txg：登记第五节本地常量表第 7 行（`GENESIS_CHECKPOINT_TXG`）。
const GENESIS_TXG: u64 = 0;
/// 读坏一个根槽时取反的字节：登记第五节本地常量表第 8 行「槽内第 40 字节取反」。
const CORRUPTED_BYTE_IN_THE_ROOT_SLOT: u64 = 40;

/// 几何（登记第五节「几何」一段）：两块盘、每盘 256 MiB、物理块与最小 I/O 512 字节、固定结构槽距 4096。
const DEVICE_BYTES: u64 = 256 << 20;
const PHYSICAL_BLOCK_BYTES: u32 = 512;
const FIXED_STRUCTURE_SLOT_SPACING: u32 = 4096;
/// journal 环长：主取样点 1 MiB，几何敏感性 G2 改成 4 MiB（登记第五节取样域表与第八节）。
const SHORT_JOURNAL_RING_BYTES: u64 = 1 << 20;
const LONG_JOURNAL_RING_BYTES: u64 = 4 << 20;
/// 内容：默认 3000 字节覆盖写；顺序写内容长 c × 524 288 字节（登记第五节「几何」一段）；nrb 顺序写那一组 2 MiB（第五节取样域表）。
const OVERWRITE_CONTENT_BYTES: usize = 3000;
const SEQUENTIAL_WRITE_CHUNK_BYTES: usize = 524_288;
const NEWEST_ROW_BAD_SEQUENTIAL_CONTENT_BYTES: usize = 2 * 1024 * 1024;
/// 写时刻固定（登记第五节「几何」一段「写时刻固定」）；fsid 自取 16 字节。
const WRITE_TIME_SECONDS: u64 = 1_788_000_000;
const FILESYSTEM_IDENTIFIER: [u8; 16] = *b"E198-birth-skip!";
/// G1 攒行时实例数的上限：阈值 365 要 366 行（实例 367 的挂载才扫），留余量；到上限还没扫就照实报「这一形没造出来」。
const SWEEP_ROW_ACCUMULATION_INSTANCES_AT_MOST: u32 = 400;

// ============================================================================
// 二、取样域：组、几何点与每一段历史的参数（登记第五节取样域表、第八节几何敏感性表）。一段历史 = 一个组的一个参数格，
//    从 mkfs 起自己造、自己判（登记第十四节「独立单位」）。
// ============================================================================

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
enum Group {
    ShortRing,
    SkippedRow,
    SweepCrash,
    SweepRead,
    SweepNone,
    NewestRowBad,
    RollbackSlotBad,
}

impl Group {
    /// 产物里的组名（登记第五节取样域表第一列）。
    fn name(self) -> &'static str {
        match self {
            Group::ShortRing => "ss",
            Group::SkippedRow => "skr",
            Group::SweepCrash => "sw-crash",
            Group::SweepRead => "sw-read",
            Group::SweepNone => "sw-none",
            Group::NewestRowBad => "nrb",
            Group::RollbackSlotBad => "rs1",
        }
    }
}

/// 取样点：主取样点与第八节的四个几何敏感性点。
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
enum SamplingPoint {
    Main,
    SweepThresholdProductDefault,
    LongJournalRing,
    SkippedRowWiderRing,
    NewestRowAndRollbackWiderRing,
}

impl SamplingPoint {
    fn name(self) -> &'static str {
        match self {
            SamplingPoint::Main => "main",
            SamplingPoint::SweepThresholdProductDefault => "G1",
            SamplingPoint::LongJournalRing => "G2",
            SamplingPoint::SkippedRowWiderRing => "G3",
            SamplingPoint::NewestRowAndRollbackWiderRing => "G4",
        }
    }
}

/// 用户改动的写法：3000 字节覆盖写（`publish_overwrite`），或给定长度的顺序写（`publish_sequential_write`）。
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
enum UserWrite {
    Overwrite,
    SequentialWrite { content_bytes: usize },
}

/// rs1 回退之后的动作（登记第五节「动作的写法照线索模型 `rollback_hole.rs`」）。
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
enum RollbackAction {
    Overwrite,
    LongerSequentialWrite,
    NewInode,
    EmptyPublish,
}

impl RollbackAction {
    fn name(self) -> &'static str {
        match self {
            RollbackAction::Overwrite => "over",
            RollbackAction::LongerSequentialWrite => "sequential",
            RollbackAction::NewInode => "inodes",
            RollbackAction::EmptyPublish => "empty",
        }
    }
}

const ROLLBACK_ACTIONS: [RollbackAction; 4] = [
    RollbackAction::Overwrite,
    RollbackAction::LongerSequentialWrite,
    RollbackAction::NewInode,
    RollbackAction::EmptyPublish,
];

/// sw 三组崩不崩：sw-crash 在 b + 3S 那一次覆盖写崩，sw-none 不崩；sw-read 不崩、在实例 3 的每一版上取镜像。
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
enum SweepVariant {
    CrashBeforeTheNextInstance,
    NoCrash,
    ReadFaultOnTheSweepInstance,
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum HistoryShape {
    /// ss：实例 1 覆盖写到 last → last + 1 覆盖写崩 → 实例 2 产品挂载 → 顺序写 k 次（内容 c × 524 288 字节），k = 0 … 3S 每个 k 一张。
    ShortRing { last: u64, chunks: u64 },
    /// skr：实例 1 到 last → 实例 2 产品挂载崩在写行那次根槽写之前 → 实例 3 产品挂载 → 覆盖写 k 次，k = 0 … 14。
    SkippedRow { last: u64 },
    /// sw 三组与 G1：每次挂载走带清扫配置的入口。实例 1 到 a → 实例 2 覆盖写到 b = a + 3S + d（G1：实例 2 起每次挂载只写行与暖机、
    /// 攒到清扫发生为止，b 取清扫那次挂载之前的最后一版）→ 清扫那个实例覆盖写到 b + 3S − 1 → 崩或不崩 → 下一个实例挂载 → 覆盖写 k 次。
    Sweep {
        first_instance_last: u64,
        extra_overwrites: u64,
        threshold: u64,
        accumulate_rows_by_mounts: bool,
        variant: SweepVariant,
    },
    /// nrb：实例 1 到 last → 实例 2 产品挂载（写行 r）→ 用户写 k 次 → 读坏 r … r + bad − 1 的根槽。
    NewestRowBad {
        last: u64,
        steps_after_the_mount: u64,
        bad_slots: u64,
        user_write: UserWrite,
    },
    /// rs1：实例 1 到 10 → 实例 2 产品挂载（写行 r）→ 回退到 (实例 1, target)（那一版 v）→ 动作串 → 读坏 r … R − 1 的根槽，R ∈ {v, …, v + 动作数}。
    RollbackSlotBad {
        target: u64,
        actions: Vec<RollbackAction>,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct HistorySpecification {
    group: Group,
    point: SamplingPoint,
    slots_per_region: u64,
    journal_ring_bytes: u64,
    shape: HistoryShape,
}

/// rs1 的回退目标、nrb 那一组实例 1 写到哪一版：登记第五节取样域表。
const ROLLBACK_FIRST_INSTANCE_LAST: u64 = 10;

fn user_write_name(user_write: UserWrite) -> String {
    match user_write {
        UserWrite::Overwrite => "overwrite".to_string(),
        UserWrite::SequentialWrite { content_bytes } => format!("sequential{content_bytes}"),
    }
}

fn sweep_variant_group(variant: SweepVariant) -> Group {
    match variant {
        SweepVariant::CrashBeforeTheNextInstance => Group::SweepCrash,
        SweepVariant::NoCrash => Group::SweepNone,
        SweepVariant::ReadFaultOnTheSweepInstance => Group::SweepRead,
    }
}

/// 历史名：组、取样点、S、环长与参数格，产物里每一行带着它；按它排序就是产物的次序。
fn history_name(specification: &HistorySpecification) -> String {
    let prefix = format!(
        "{}.{}.S{}.ring{}",
        specification.group.name(),
        specification.point.name(),
        specification.slots_per_region,
        specification.journal_ring_bytes
    );
    match &specification.shape {
        HistoryShape::ShortRing { last, chunks } => format!("{prefix}.last{last}.c{chunks}"),
        HistoryShape::SkippedRow { last } => format!("{prefix}.last{last}"),
        HistoryShape::Sweep {
            first_instance_last,
            extra_overwrites,
            threshold,
            accumulate_rows_by_mounts,
            variant: _,
        } => format!(
            "{prefix}.a{first_instance_last}.d{extra_overwrites}.threshold{threshold}.accumulate{accumulate_rows_by_mounts}"
        ),
        HistoryShape::NewestRowBad { last, steps_after_the_mount, bad_slots, user_write } => format!(
            "{prefix}.last{last}.k{steps_after_the_mount}.bad{bad_slots}.{}",
            user_write_name(*user_write)
        ),
        HistoryShape::RollbackSlotBad { target, actions } => {
            let names: Vec<&str> = actions.iter().map(|action| action.name()).collect();
            let actions_text = if names.is_empty() { "-".to_string() } else { names.join("+") };
            format!("{prefix}.target{target}.actions{actions_text}")
        }
    }
}

/// 一段历史尝试造的张数（登记第七节 B1–B6 数的是「尝试」，含写者返回 `Err` 的格）。sw-read 的张数由清扫那一版的 txg 定，
/// 跑之前不知道（推的 315 不进锚点，登记第四节第 3 条），交 `None`。
fn attempted_images(specification: &HistorySpecification) -> Option<u64> {
    match &specification.shape {
        HistoryShape::ShortRing { .. } => Some(3 * specification.slots_per_region + 1),
        HistoryShape::SkippedRow { .. } => Some(15),
        HistoryShape::Sweep { variant, .. } => match variant {
            SweepVariant::CrashBeforeTheNextInstance | SweepVariant::NoCrash => {
                Some(3 * specification.slots_per_region + 1)
            }
            SweepVariant::ReadFaultOnTheSweepInstance => None,
        },
        HistoryShape::NewestRowBad { .. } => Some(1),
        HistoryShape::RollbackSlotBad { actions, .. } => {
            Some(1 + u64::try_from(actions.len()).expect("动作数装得进 u64"))
        }
    }
}

fn newest_row_bad_specifications(
    point: SamplingPoint,
    slots_per_region: u64,
) -> Vec<HistorySpecification> {
    let mut specifications = Vec::new();
    for bad_slots in [1, 2] {
        specifications.push(HistorySpecification {
            group: Group::NewestRowBad,
            point,
            slots_per_region,
            journal_ring_bytes: SHORT_JOURNAL_RING_BYTES,
            shape: HistoryShape::NewestRowBad {
                last: 10,
                steps_after_the_mount: 3,
                bad_slots,
                user_write: UserWrite::Overwrite,
            },
        });
    }
    for steps_after_the_mount in 3..=6 {
        specifications.push(HistorySpecification {
            group: Group::NewestRowBad,
            point,
            slots_per_region,
            journal_ring_bytes: SHORT_JOURNAL_RING_BYTES,
            shape: HistoryShape::NewestRowBad {
                last: 4,
                steps_after_the_mount,
                bad_slots: 1,
                user_write: UserWrite::SequentialWrite {
                    content_bytes: NEWEST_ROW_BAD_SEQUENTIAL_CONTENT_BYTES,
                },
            },
        });
    }
    specifications
}

/// 动作串：{不做} ∪ {一个} ∪ {两个}，按线索模型 `rollback-sweep.sh` 的次序。
fn rollback_action_strings() -> Vec<Vec<RollbackAction>> {
    let mut strings = vec![Vec::new()];
    for first in ROLLBACK_ACTIONS {
        strings.push(vec![first]);
        for second in ROLLBACK_ACTIONS {
            strings.push(vec![first, second]);
        }
    }
    strings
}

fn rollback_specifications(
    point: SamplingPoint,
    slots_per_region: u64,
    targets: &[u64],
) -> Vec<HistorySpecification> {
    let mut specifications = Vec::new();
    for target in targets {
        for actions in rollback_action_strings() {
            specifications.push(HistorySpecification {
                group: Group::RollbackSlotBad,
                point,
                slots_per_region,
                journal_ring_bytes: SHORT_JOURNAL_RING_BYTES,
                shape: HistoryShape::RollbackSlotBad {
                    target: *target,
                    actions,
                },
            });
        }
    }
    specifications
}

fn short_ring_specifications(
    point: SamplingPoint,
    slots_per_region_list: &[u64],
    journal_ring_bytes: u64,
) -> Vec<HistorySpecification> {
    let mut specifications = Vec::new();
    for slots_per_region in slots_per_region_list {
        for last in [4, 6, 8] {
            for chunks in [2, 3, 4] {
                specifications.push(HistorySpecification {
                    group: Group::ShortRing,
                    point,
                    slots_per_region: *slots_per_region,
                    journal_ring_bytes,
                    shape: HistoryShape::ShortRing { last, chunks },
                });
            }
        }
    }
    specifications
}

fn skipped_row_specifications(
    point: SamplingPoint,
    slots_per_region: u64,
) -> Vec<HistorySpecification> {
    [6, 8, 10]
        .into_iter()
        .map(|last| HistorySpecification {
            group: Group::SkippedRow,
            point,
            slots_per_region,
            journal_ring_bytes: SHORT_JOURNAL_RING_BYTES,
            shape: HistoryShape::SkippedRow { last },
        })
        .collect()
}

fn sweep_specifications(variant: SweepVariant) -> Vec<HistorySpecification> {
    let mut specifications = Vec::new();
    for slots_per_region in [4, 5, 6] {
        for first_instance_last in [4, 6, 8] {
            for extra_overwrites in [0, 1, 2] {
                specifications.push(HistorySpecification {
                    group: sweep_variant_group(variant),
                    point: SamplingPoint::Main,
                    slots_per_region,
                    journal_ring_bytes: SHORT_JOURNAL_RING_BYTES,
                    shape: HistoryShape::Sweep {
                        first_instance_last,
                        extra_overwrites,
                        threshold: SWEEP_THRESHOLD_ANY_ROW,
                        accumulate_rows_by_mounts: false,
                        variant,
                    },
                });
            }
        }
    }
    specifications
}

/// G1：sw-crash 的清扫阈值从 0 改成产品默认（修订后 365），S = 4、a = 4，实例 2 起每次挂载只写行与暖机、攒到清扫发生（登记第八节 G1）。
fn sweep_threshold_product_default_specification() -> HistorySpecification {
    HistorySpecification {
        group: Group::SweepCrash,
        point: SamplingPoint::SweepThresholdProductDefault,
        slots_per_region: 4,
        journal_ring_bytes: SHORT_JOURNAL_RING_BYTES,
        shape: HistoryShape::Sweep {
            first_instance_last: 4,
            extra_overwrites: 0,
            threshold: SWEEP_THRESHOLD_PRODUCT_DEFAULT,
            accumulate_rows_by_mounts: true,
            variant: SweepVariant::CrashBeforeTheNextInstance,
        },
    }
}

/// 一段（登记第五节「分段」）跑哪几组。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Segment {
    One,
    Two,
    Three,
}

impl Segment {
    fn name(self) -> &'static str {
        match self {
            Segment::One => "segment-one",
            Segment::Two => "segment-two",
            Segment::Three => "segment-three",
        }
    }

    fn from_name(name: &str) -> Option<Segment> {
        [Segment::One, Segment::Two, Segment::Three]
            .into_iter()
            .find(|segment| segment.name() == name)
    }

    /// 这一段的全部历史，按历史名排序（产物的次序）。
    fn history_specifications(self) -> Vec<HistorySpecification> {
        let mut specifications = match self {
            Segment::One => {
                let mut specifications = newest_row_bad_specifications(SamplingPoint::Main, 4);
                specifications.extend(rollback_specifications(
                    SamplingPoint::Main,
                    4,
                    &[5, 6, 7, 8, 9],
                ));
                specifications.extend(newest_row_bad_specifications(
                    SamplingPoint::NewestRowAndRollbackWiderRing,
                    6,
                ));
                specifications.extend(rollback_specifications(
                    SamplingPoint::NewestRowAndRollbackWiderRing,
                    6,
                    &[5, 9],
                ));
                specifications
            }
            Segment::Two => {
                let mut specifications = short_ring_specifications(
                    SamplingPoint::Main,
                    &[4, 5, 6],
                    SHORT_JOURNAL_RING_BYTES,
                );
                specifications.extend(skipped_row_specifications(SamplingPoint::Main, 4));
                specifications.extend(sweep_specifications(
                    SweepVariant::CrashBeforeTheNextInstance,
                ));
                specifications.extend(sweep_specifications(
                    SweepVariant::ReadFaultOnTheSweepInstance,
                ));
                specifications.push(sweep_threshold_product_default_specification());
                specifications.extend(short_ring_specifications(
                    SamplingPoint::LongJournalRing,
                    &[4],
                    LONG_JOURNAL_RING_BYTES,
                ));
                specifications.extend(skipped_row_specifications(
                    SamplingPoint::SkippedRowWiderRing,
                    6,
                ));
                specifications
            }
            Segment::Three => sweep_specifications(SweepVariant::NoCrash),
        };
        specifications.sort_by_key(history_name);
        specifications
    }
}

// ============================================================================
// 三、建池的几样共用件：参数、录制盘、确定性内容。
// ============================================================================

fn pool_parameters(slots_per_region: u64, journal_ring_bytes: u64) -> MakeFilesystemParameters {
    MakeFilesystemParameters {
        filesystem_identifier: FILESYSTEM_IDENTIFIER,
        region_devices: REGION_DEVICES.map(DeviceIdentity),
        geometry: SystemImmutableSizes {
            physical_block_size: PHYSICAL_BLOCK_BYTES,
            minimum_input_output_bytes: PHYSICAL_BLOCK_BYTES,
            fixed_structure_slot_spacing: FIXED_STRUCTURE_SLOT_SPACING,
            journal_ring_bytes,
            root_ring_slots_per_region: RootRingSlotsPerRegion::from_system_configuration_field(
                slots_per_region,
            )
            .expect("登记的 S 都在 [4, 16] 区间内"),
        },
    }
}

fn fixed_geometry_of(parameters: &MakeFilesystemParameters) -> FixedGeometry {
    FixedGeometry {
        fixed_structure_slot_spacing: parameters.geometry.fixed_structure_slot_spacing,
        journal_ring_bytes: parameters.geometry.journal_ring_bytes,
        root_ring_slots_per_region: parameters.geometry.root_ring_slots_per_region,
    }
}

/// 确定性内容（写法同线索模型 `lib.rs` 的 `content`）：同一个盐同一个长度两次逐字节相同。
fn deterministic_content(salt: u64, length: usize) -> Vec<u8> {
    let salt_term = usize::try_from(salt % 251).expect("模 251 装得进 usize");
    (0..length)
        .map(|index| u8::try_from((index + salt_term * 7 + 3) % 251).expect("模 251 装得进 u8"))
        .collect()
}

fn device_identities() -> [DeviceIdentity; 2] {
    [DeviceIdentity(0), DeviceIdentity(1)]
}

type RecordedDevices = Vec<(DeviceIdentity, RecordingBlockDevice<SparseBlockDevice>)>;

/// 以 `image` 为起点的两块录制盘，共用一条录制流。每一步（一次挂载、一次发布）各起一份：镜像在这一步之前与盘上字节逐字节相同。
fn recording_devices_of(image: &MemoryPool, stream: &SharedStream) -> RecordedDevices {
    device_identities()
        .into_iter()
        .map(|identity| {
            let mut device = SparseBlockDevice::new(
                DEVICE_BYTES,
                PhysicalBlockSizeInBytes(PHYSICAL_BLOCK_BYTES),
            );
            if let Some(sparse) = image.devices.get(&identity) {
                device.image = sparse.clone();
            }
            (
                identity,
                RecordingBlockDevice::with_shared_stream(identity, device, stream.clone()),
            )
        })
        .collect()
}

// ============================================================================
// 四、真值账本（只从历史取，不读镜像；登记第一节「真值」「祖先链」「存在过的版本」）。
//    每个发布过的版本记父版本、发布路径、写者交回的实例表指针出生 txg、根槽落没落、有没有被某次挂载施加过、账本推的实例表行集。
// ============================================================================

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct VersionKey {
    instance: u32,
    txg: u64,
}

impl VersionKey {
    fn of_root(root: &RootRecord) -> VersionKey {
        VersionKey {
            instance: root.instance.0,
            txg: root.checkpoint_txg.0,
        }
    }

    fn text(self) -> String {
        format!("{}@{}", self.instance, self.txg)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
enum PublishKind {
    MakeFilesystem,
    RowPublish,
    WarmUp,
    Sweep,
    OtherMountPublish,
    FirstFile,
    Overwrite,
    SequentialWrite,
    NewInode,
    EmptyPublish,
    Rollback,
}

impl PublishKind {
    fn name(self) -> &'static str {
        match self {
            PublishKind::MakeFilesystem => "mkfs",
            PublishKind::RowPublish => "row_publish",
            PublishKind::WarmUp => "warm_up",
            PublishKind::Sweep => "sweep",
            PublishKind::OtherMountPublish => "other_mount_publish",
            PublishKind::FirstFile => "first_file",
            PublishKind::Overwrite => "overwrite",
            PublishKind::SequentialWrite => "sequential_write",
            PublishKind::NewInode => "new_inode",
            PublishKind::EmptyPublish => "empty_publish",
            PublishKind::Rollback => "rollback",
        }
    }
}

/// 实例表的一行：(实例代号, 所选根的 txg, 最大已施加事务号)。
type RowTriple = (u32, u64, u64);

#[derive(Clone, Debug)]
struct LedgerVersion {
    /// 父版本（登记第一节「祖先链」）：普通发布、暖机、清扫、回退发布是发它之前那个会话的现行那一版；写行那次发布是那次挂载恢复之后的有效根。
    parent: Option<VersionKey>,
    kind: PublishKind,
    /// 写者交回的根记录（崩在根槽写之前的那一版取它录制流里那次根槽写的内容）里实例表指针的出生 txg。
    pointer_birth_txg: u64,
    root_landed: bool,
    applied_by_a_mount: bool,
    /// 发布的先后（账本里的第几次发布），P1、P2 按它取「第一版」「相邻两版」。
    publish_order: usize,
    /// 账本按每次挂载写的行与清扫删的行推出的这一版实例表行集（停机条款 S3 比它）。
    rows: Arc<BTreeSet<RowTriple>>,
}

impl LedgerVersion {
    /// 存在过（登记第一节）：根槽写落进了镜像，或之后某一次可写挂载的恢复把它从 journal 记录里施加了出来。
    fn existed(&self) -> bool {
        self.root_landed || self.applied_by_a_mount
    }
}

#[derive(Clone, Debug, Default)]
struct Ledger {
    versions: BTreeMap<VersionKey, LedgerVersion>,
    /// 根环每个 (区域, 槽) 此刻住着哪一版（按根槽写落下的次序，后写的盖前写的；停机条款 S1 比它）。
    ring_occupants: BTreeMap<(u64, u64), VersionKey>,
    /// 镜像上被读坏的根环槽（S1 里这几槽不该再被读出）。
    corrupted_ring_slots: BTreeSet<(u64, u64)>,
    /// S2：落下的根槽写里 [212, 220) 字节与写者交回的出生 txg 比过几条、对不上的逐条、没有交回值可比的几条。
    root_slot_writes_compared: u64,
    root_slot_write_mismatches: Vec<String>,
    root_slot_writes_without_a_returned_root: u64,
    /// 落点公式（本地公式、`target_for_publish`、录制流里那次写的位置）三方对不上的逐条（S4 的「落点公式」那一格）。
    ring_formula_mismatches: Vec<String>,
    /// 账本自己认不出的东西（根槽写解不出根、有效根不在账本里……），非空时这段历史不可信。
    consistency_problems: Vec<String>,
}

impl Ledger {
    fn rows_of(&self, key: Option<VersionKey>) -> Arc<BTreeSet<RowTriple>> {
        key.and_then(|key| self.versions.get(&key))
            .map(|version| Arc::clone(&version.rows))
            .unwrap_or_default()
    }

    /// R 的祖先链（不含 R 自己），沿父版本一路往前；链上出现环（不该有）就停。
    fn ancestors_of(&self, key: VersionKey) -> Vec<VersionKey> {
        let mut chain = Vec::new();
        let mut seen = BTreeSet::new();
        let mut cursor = self.versions.get(&key).and_then(|version| version.parent);
        while let Some(ancestor) = cursor {
            if !seen.insert(ancestor) {
                break;
            }
            chain.push(ancestor);
            cursor = self
                .versions
                .get(&ancestor)
                .and_then(|version| version.parent);
        }
        chain
    }
}

/// 一个洞对的真值（登记第一节「真值」）：直接接上 ⟺ R 的父版本是 R⁻ 那一组里的某一版；否则看 R⁻ 那一组有没有一版在 R 的祖先链上。
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
enum Truth {
    DirectlyBridged,
    ExistedVersionsInBetween,
    PreviousNotOnTheChain,
    /// R 不在账本里（S1、S5 本该先拦下）：这一对不进 E1 / E2，单列。
    VersionMissingFromTheLedger,
}

impl Truth {
    fn name(self) -> &'static str {
        match self {
            Truth::DirectlyBridged => "directly_bridged",
            Truth::ExistedVersionsInBetween => "existed_versions_in_between",
            Truth::PreviousNotOnTheChain => "previous_not_on_the_chain",
            Truth::VersionMissingFromTheLedger => "version_missing_from_the_ledger",
        }
    }
}

/// 洞对 (R⁻ 那一组, R) 的真值：只读账本里的父版本链，链上只算存在过的版本（纯函数，单测喂已知答案）。
fn truth_of_the_hole_pair(
    ledger: &Ledger,
    previous_group: &[VersionKey],
    version: VersionKey,
) -> Truth {
    if !ledger.versions.contains_key(&version) {
        return Truth::VersionMissingFromTheLedger;
    }
    let ancestors: Vec<VersionKey> = ledger
        .ancestors_of(version)
        .into_iter()
        .filter(|ancestor| {
            ledger
                .versions
                .get(ancestor)
                .is_some_and(LedgerVersion::existed)
        })
        .collect();
    // 往前第一个存在过的祖先（合法历史里父版本都存在过，它就是父版本）。
    if ancestors
        .first()
        .is_some_and(|parent| previous_group.contains(parent))
    {
        return Truth::DirectlyBridged;
    }
    if previous_group
        .iter()
        .any(|previous| ancestors.contains(previous))
    {
        Truth::ExistedVersionsInBetween
    } else {
        Truth::PreviousNotOnTheChain
    }
}

// ============================================================================
// 五、造历史：每一步（mkfs、一次挂载、一次发布）在镜像上各起一份录制盘，录下这一步的写流；落下的写按次序施加到镜像上，
//    根槽写同时记进账本。「崩」= 那一步的写流截在它第一个根槽写（`StepKind::RootRecordFua`）之前：记录已落、根槽没落（登记第五节）。
// ============================================================================

/// 一次挂载（或一串发布）落下的根槽写怎么记进账本：每一版的发布路径、写行那次发布写的行、清扫删的行。
struct LandingContext<'kinds> {
    kind_of: &'kinds dyn Fn(VersionKey) -> PublishKind,
    rows_written_by_the_row_publish: Vec<RowTriple>,
    rows_deleted_by_the_sweep: Vec<RowTriple>,
}

/// 一个实例挂着时手里的东西：分配器、现行那一版、账本里最后一版（下一次发布的父版本）。
struct Session {
    instance: InstanceGeneration,
    allocator: PoolAllocator,
    current: PoolVersion,
    last_key: VersionKey,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum MountConfiguration {
    /// 产品挂载 `mount_writable`（ss、skr、nrb、rs1）。
    Product,
    /// 每次挂载走 `mount_writable_with_instance_table_sweep_configuration`（开关关、给定阈值、落点取挂载的分配器；sw 三组与 G1）。
    SweepAtTheEndAboveTheThreshold { threshold: u64 },
}

/// 一次挂载交回的东西里这条实验要的那几样。
#[derive(Clone, Debug, Default)]
struct MountSummary {
    instance: u32,
    chosen: Option<VersionKey>,
    effective: Option<VersionKey>,
    row_publish: Option<VersionKey>,
    sweep_outcome: String,
    sweep_version: Option<VersionKey>,
}

fn row_triple_of(row: &singlefs_core::instance_table::InstanceRow) -> RowTriple {
    (
        row.instance.0,
        row.selected_root_txg.0,
        row.applied_transaction_high_water,
    )
}

/// 清扫结局的名字（只取成员名，带的值另报）。
fn sweep_outcome_name(sweep: &InstanceTableSweepAtTheEndOfTheMount) -> String {
    match sweep {
        InstanceTableSweepAtTheEndOfTheMount::NotSweptAtOrBelowTheThreshold { rows, threshold } => {
            format!("not_swept_rows{rows}_threshold{}", threshold.0)
        }
        InstanceTableSweepAtTheEndOfTheMount::HandedToTheBackgroundSweepWhileMounted => {
            "handed_to_the_background_sweep".to_string()
        }
        InstanceTableSweepAtTheEndOfTheMount::Swept(outcome) => match outcome {
            InstanceTableSweepOutcome::NoRowDeleted(reason) => {
                let text = format!("{reason:?}");
                let variant: String = text
                    .chars()
                    .take_while(|character| character.is_alphanumeric())
                    .collect();
                format!("no_row_deleted_{variant}")
            }
            InstanceTableSweepOutcome::RowsDeleted { rows_deleted, .. } => {
                format!("rows_deleted{}", rows_deleted.len())
            }
            InstanceTableSweepOutcome::SweepPublishRefusedForSpace { .. } => {
                "sweep_publish_refused_for_space".to_string()
            }
            InstanceTableSweepOutcome::StoppedByABlockDeviceErrorWhileErasing { .. } => {
                "stopped_by_a_block_device_error_while_erasing".to_string()
            }
            InstanceTableSweepOutcome::SweepPublishFailed { .. } => {
                "sweep_publish_failed".to_string()
            }
        },
    }
}

fn rows_deleted_by(sweep: &InstanceTableSweepAtTheEndOfTheMount) -> Option<Vec<RowTriple>> {
    match sweep {
        InstanceTableSweepAtTheEndOfTheMount::Swept(InstanceTableSweepOutcome::RowsDeleted {
            rows_deleted,
            ..
        }) => Some(rows_deleted.iter().map(row_triple_of).collect()),
        InstanceTableSweepAtTheEndOfTheMount::NotSweptAtOrBelowTheThreshold { .. }
        | InstanceTableSweepAtTheEndOfTheMount::HandedToTheBackgroundSweepWhileMounted
        | InstanceTableSweepAtTheEndOfTheMount::Swept(
            InstanceTableSweepOutcome::NoRowDeleted(_)
            | InstanceTableSweepOutcome::SweepPublishRefusedForSpace { .. }
            | InstanceTableSweepOutcome::StoppedByABlockDeviceErrorWhileErasing { .. }
            | InstanceTableSweepOutcome::SweepPublishFailed { .. },
        ) => None,
    }
}

/// 一次挂载交回的每一版根（写者交回的那一份，S2 拿它比镜像上的字节）。
fn roots_returned_by_the_mount(output: &MountOutput, current: &PoolVersion) -> Vec<RootRecord> {
    let mut roots = vec![*output.row_publish.root(), *current.root()];
    roots.extend(
        output
            .warm_up_publishes
            .iter()
            .map(|version| *version.root()),
    );
    for raised in output.space_admission.floor_raises() {
        roots.extend(raised.publishes.iter().map(|publish| publish.root));
    }
    for raised in &output.floor_raises_pushed_by_the_continuation {
        roots.extend(raised.publishes.iter().map(|publish| publish.root));
    }
    roots.extend(
        output
            .deletion_queue_continuation
            .iter()
            .map(|step| step.root),
    );
    roots
}

struct HistoryBuilder {
    parameters: MakeFilesystemParameters,
    fixed: FixedGeometry,
    image: MemoryPool,
    ledger: Ledger,
    /// 根环每个槽在哪块盘、哪个偏移（`root_ring::slot_offset` 按区域与槽现算），反查根槽写落在哪个 (区域, 槽)。
    ring_slot_of_location: BTreeMap<(u32, u64), (u64, u64)>,
    /// 写者交回的每一版根里实例表指针的出生 txg（S2 的一边）。
    returned_pointer_births: BTreeMap<VersionKey, u64>,
    publishes_so_far: usize,
}

fn is_a_root_slot_write(operation: &RetainedOperation, fixed: &FixedGeometry) -> bool {
    matches!(
        operation.operation.kind,
        RecordedOperationKind::Write | RecordedOperationKind::WriteForceUnitAccess
    ) && fixed.classify(&operation.operation) == StepKind::RootRecordFua
}

fn root_written_by(operation: &RetainedOperation) -> Option<RootRecord> {
    RootRecord::parse_slot(operation.contents.as_ref()?, &FILESYSTEM_IDENTIFIER)
}

/// 根槽字节里实例表指针的出生 txg（本地偏移 170 + 42，小端 8 字节；登记第七节 B7）。
fn pointer_birth_txg_in_root_slot_bytes(slot_bytes: &[u8]) -> Option<u64> {
    let start = ROOT_RECORD_INSTANCE_TABLE_POINTER_OFFSET + POINTER_BIRTH_TXG_OFFSET;
    let bytes: [u8; POINTER_BIRTH_TXG_BYTES] = slot_bytes
        .get(start..start + POINTER_BIRTH_TXG_BYTES)?
        .try_into()
        .ok()?;
    Some(u64::from_le_bytes(bytes))
}

/// 本地落点公式（登记第五节本地常量表第 3 行、`mount.rs` 写行那段读法）：区域 = txg mod 3，槽 = (txg ÷ 3) mod S。
fn ring_slot_by_the_local_formula(txg: u64, slots_per_region: u64) -> (u64, u64) {
    (
        txg % ROOT_RING_REGIONS,
        (txg / ROOT_RING_REGIONS) % slots_per_region,
    )
}

impl HistoryBuilder {
    fn new(specification: &HistorySpecification) -> HistoryBuilder {
        let parameters = pool_parameters(
            specification.slots_per_region,
            specification.journal_ring_bytes,
        );
        let fixed = fixed_geometry_of(&parameters);
        let mut ring_slot_of_location = BTreeMap::new();
        for region in 0..ROOT_RING_REGIONS {
            for slot in 0..specification.slots_per_region {
                let device = REGION_DEVICES[usize::try_from(region).expect("区域号装得进 usize")];
                let offset = slot_offset(
                    RootRingSlot { region, slot },
                    parameters.geometry.fixed_structure_slot_spacing,
                );
                ring_slot_of_location.insert((device, offset.0), (region, slot));
            }
        }
        HistoryBuilder {
            parameters,
            fixed,
            image: MemoryPool::with_devices(&device_identities(), DEVICE_BYTES),
            ledger: Ledger::default(),
            ring_slot_of_location,
            returned_pointer_births: BTreeMap::new(),
            publishes_so_far: 0,
        }
    }

    fn slots_per_region(&self) -> u64 {
        self.parameters.geometry.root_ring_slots_per_region.count()
    }

    /// 在此刻的镜像上起一份录制盘，做 `body`，交回它的结果与这一步录下的写流。
    fn recorded<Value>(
        &self,
        body: impl FnOnce(&mut RecordedDevices) -> Value,
    ) -> (Value, Vec<RetainedOperation>) {
        let stream = SharedStream::retaining_contents();
        let mut devices = recording_devices_of(&self.image, &stream);
        let value = body(&mut devices);
        drop(devices);
        (value, stream.retained_operations())
    }

    fn note_returned_roots(&mut self, roots: &[RootRecord]) {
        for root in roots {
            self.returned_pointer_births.insert(
                VersionKey::of_root(root),
                root.instance_table.head.birth_txg.0,
            );
        }
    }

    /// 账本里加一版（已有的只补「根槽落了」）。行集按发布路径推：写行那次发布 = 父版本的行并上这次写的行；清扫 = 父版本的行减去删的行；别的照抄父版本。
    fn register_version(
        &mut self,
        root: &RootRecord,
        parent: Option<VersionKey>,
        context: &LandingContext<'_>,
        root_landed: bool,
    ) -> VersionKey {
        let key = VersionKey::of_root(root);
        if let Some(existing) = self.ledger.versions.get_mut(&key) {
            existing.root_landed |= root_landed;
            return key;
        }
        let kind = (context.kind_of)(key);
        let parent_rows = self.ledger.rows_of(parent);
        let rows = match kind {
            PublishKind::RowPublish => {
                let mut rows = (*parent_rows).clone();
                rows.extend(context.rows_written_by_the_row_publish.iter().copied());
                Arc::new(rows)
            }
            PublishKind::Sweep => {
                let mut rows = (*parent_rows).clone();
                for deleted in &context.rows_deleted_by_the_sweep {
                    rows.remove(deleted);
                }
                Arc::new(rows)
            }
            PublishKind::MakeFilesystem
            | PublishKind::WarmUp
            | PublishKind::OtherMountPublish
            | PublishKind::FirstFile
            | PublishKind::Overwrite
            | PublishKind::SequentialWrite
            | PublishKind::NewInode
            | PublishKind::EmptyPublish
            | PublishKind::Rollback => parent_rows,
        };
        self.ledger.versions.insert(
            key,
            LedgerVersion {
                parent,
                kind,
                pointer_birth_txg: root.instance_table.head.birth_txg.0,
                root_landed,
                applied_by_a_mount: false,
                publish_order: self.publishes_so_far,
                rows,
            },
        );
        self.publishes_so_far += 1;
        key
    }

    /// 根槽写落下那一刻：核落点公式（本地公式、`target_for_publish`、写的位置三方相同）与 S2（[212, 220) 字节对写者交回的出生 txg），记占位。
    fn note_a_landed_root_slot_write(&mut self, operation: &RetainedOperation, root: &RootRecord) {
        let key = VersionKey::of_root(root);
        let location = (operation.operation.device.0, operation.operation.offset.0);
        let Some(&ring_slot) = self.ring_slot_of_location.get(&location) else {
            self.ledger.consistency_problems.push(format!(
                "版本 {} 的根槽写落在盘 {} 偏移 {}，不是根环里的槽",
                key.text(),
                location.0,
                location.1
            ));
            return;
        };
        if key.txg != GENESIS_TXG {
            let local = ring_slot_by_the_local_formula(key.txg, self.slots_per_region());
            let crates_target = target_for_publish(
                CheckpointTxg(key.txg),
                self.parameters.geometry.root_ring_slots_per_region,
            );
            if local != ring_slot || (crates_target.region, crates_target.slot) != ring_slot {
                self.ledger.ring_formula_mismatches.push(format!(
                    "版本 {}：写在 {ring_slot:?}，本地公式 {local:?}，target_for_publish ({}, {})",
                    key.text(),
                    crates_target.region,
                    crates_target.slot
                ));
            }
        } else if ring_slot.1 != 0 {
            self.ledger
                .ring_formula_mismatches
                .push(format!("第 0 代根写在 {ring_slot:?}，不是某个区域的槽 0"));
        }
        let in_bytes = operation
            .contents
            .as_deref()
            .and_then(pointer_birth_txg_in_root_slot_bytes);
        match self.returned_pointer_births.get(&key) {
            Some(returned) => {
                self.ledger.root_slot_writes_compared += 1;
                if in_bytes != Some(*returned) {
                    self.ledger.root_slot_write_mismatches.push(format!(
                        "版本 {}：根槽 [212, 220) 读出 {in_bytes:?}，写者交回 {returned}",
                        key.text()
                    ));
                }
            }
            None => self.ledger.root_slot_writes_without_a_returned_root += 1,
        }
        self.ledger.ring_occupants.insert(ring_slot, key);
        self.ledger.corrupted_ring_slots.remove(&ring_slot);
    }

    /// 把一步的写流按次序施加到镜像上；根槽写逐条记进账本，父版本接 `chain_from` 往下串。交回这一串里最后一版。
    fn land_operations(
        &mut self,
        operations: &[RetainedOperation],
        context: &LandingContext<'_>,
        chain_from: Option<VersionKey>,
    ) -> Option<VersionKey> {
        let mut last = chain_from;
        for operation in operations {
            if is_a_root_slot_write(operation, &self.fixed) {
                match root_written_by(operation) {
                    Some(root) => {
                        // 已在账本里的版本（mkfs 把第 0 代根种进三个区域）只补「根槽落了」，父版本不改（`register_version`）。
                        let key = self.register_version(&root, last, context, true);
                        self.note_a_landed_root_slot_write(operation, &root);
                        last = Some(key);
                    }
                    None => self
                        .ledger
                        .consistency_problems
                        .push("一次根槽写的内容解不出根记录".to_string()),
                }
            }
            self.image.apply(std::slice::from_ref(operation));
        }
        last
    }

    /// 「崩」：施加第一个根槽写之前的那一截，那一版记成「发了、根槽没落」。交回那一版。
    fn land_up_to_the_first_root_slot_write(
        &mut self,
        operations: &[RetainedOperation],
        context: &LandingContext<'_>,
        parent: Option<VersionKey>,
    ) -> Result<VersionKey, String> {
        let cut = operations
            .iter()
            .position(|operation| is_a_root_slot_write(operation, &self.fixed))
            .ok_or_else(|| "要崩的那一步写流里没有根槽写".to_string())?;
        let root = root_written_by(&operations[cut])
            .ok_or_else(|| "要崩的那一步的根槽写解不出根记录".to_string())?;
        self.image.apply(&operations[..cut]);
        Ok(self.register_version(&root, parent, context, false))
    }

    /// 一次挂载的恢复施加出来的那几版（有效根沿父版本往回到所选根为止，不含所选根）记成「被某次挂载施加过」。
    fn mark_the_versions_the_mount_applied(&mut self, chosen: VersionKey, effective: VersionKey) {
        let mut cursor = Some(effective);
        while let Some(key) = cursor {
            if key == chosen {
                return;
            }
            let Some(version) = self.ledger.versions.get_mut(&key) else {
                self.ledger.consistency_problems.push(format!(
                    "挂载的有效根 {} 沿父版本往回走到 {}，账本里没有这一版",
                    effective.text(),
                    key.text()
                ));
                return;
            };
            version.applied_by_a_mount = true;
            cursor = version.parent;
        }
        self.ledger.consistency_problems.push(format!(
            "挂载的有效根 {} 沿父版本往回走不到所选根 {}",
            effective.text(),
            chosen.text()
        ));
    }

    /// mkfs：录下 mkfs 的写流、施加，第 0 代根记进账本（父版本无）。
    fn make_filesystem(&mut self) -> Result<(), String> {
        let parameters = self.parameters.clone();
        let (result, operations) =
            self.recorded(|devices| make_filesystem(&parameters, devices.as_mut_slice()));
        let _made = result.map_err(|error| format!("mkfs：{error:?}"))?;
        let kind_of = |_key: VersionKey| PublishKind::MakeFilesystem;
        let context = LandingContext {
            kind_of: &kind_of,
            rows_written_by_the_row_publish: Vec::new(),
            rows_deleted_by_the_sweep: Vec::new(),
        };
        self.land_operations(&operations, &context, None);
        Ok(())
    }

    fn mount_on(
        parameters: &MakeFilesystemParameters,
        devices: &mut RecordedDevices,
        configuration: MountConfiguration,
    ) -> Result<Mounted, singlefs_core::mount::MountError> {
        match configuration {
            MountConfiguration::Product => mount_writable(parameters, devices),
            MountConfiguration::SweepAtTheEndAboveTheThreshold { threshold } => {
                mount_writable_with_instance_table_sweep_configuration(
                    parameters,
                    devices,
                    SpaceAdmission::JudgedByTheFormula,
                    ShadowLedger::On,
                    InstanceTableSweepConfiguration {
                        switch: InstanceTableSweepSwitch::SweptAtTheEndOfTheMountAboveTheThreshold,
                        row_count_threshold: InstanceTableRowCountThreshold(threshold),
                        sweep_publish_placements:
                            SweepPublishPlacements::TakenFromTheAllocatorOfTheMount,
                    },
                )
            }
        }
    }

    /// 一次可写挂载的发布路径：写行那次、暖机、清扫（发成了的那一版就是 `Mounted::current`），别的记「挂载里别的发布」。
    fn summary_of_the_mount(mounted: &Mounted) -> MountSummary {
        let output = &mounted.output;
        let rows_deleted = rows_deleted_by(&output.instance_table_sweep);
        MountSummary {
            instance: output.instance.0,
            chosen: Some(VersionKey::of_root(&output.chosen_root)),
            effective: Some(VersionKey::of_root(&output.effective_root)),
            row_publish: Some(VersionKey::of_root(output.row_publish.root())),
            sweep_outcome: sweep_outcome_name(&output.instance_table_sweep),
            sweep_version: rows_deleted.map(|_| VersionKey::of_root(mounted.current.root())),
        }
    }

    /// 可写挂载做成：写流施加到镜像、根槽写记进账本（写行那次发布的父版本是有效根）、恢复施加出来的那几版记成被施加过。
    /// 挂载返回 `Err` 是合法结局：镜像不动，交回错误，这段历史从这里停。
    fn mount(
        &mut self,
        configuration: MountConfiguration,
    ) -> Result<(Session, MountSummary), String> {
        let parameters = self.parameters.clone();
        let (result, operations) =
            self.recorded(|devices| HistoryBuilder::mount_on(&parameters, devices, configuration));
        let mounted = result.map_err(|error| format!("可写挂载：{error:?}"))?;
        let summary = HistoryBuilder::summary_of_the_mount(&mounted);
        self.note_returned_roots(&roots_returned_by_the_mount(
            &mounted.output,
            &mounted.current,
        ));
        let warm_up_keys: BTreeSet<VersionKey> = mounted
            .output
            .warm_up_publishes
            .iter()
            .map(|version| VersionKey::of_root(version.root()))
            .collect();
        let row_publish = summary.row_publish;
        let sweep_version = summary.sweep_version;
        let kind_of = |key: VersionKey| {
            if Some(key) == row_publish {
                PublishKind::RowPublish
            } else if warm_up_keys.contains(&key) {
                PublishKind::WarmUp
            } else if Some(key) == sweep_version {
                PublishKind::Sweep
            } else {
                PublishKind::OtherMountPublish
            }
        };
        let context = LandingContext {
            kind_of: &kind_of,
            rows_written_by_the_row_publish: mounted
                .output
                .rows_written
                .iter()
                .map(row_triple_of)
                .collect(),
            rows_deleted_by_the_sweep: rows_deleted_by(&mounted.output.instance_table_sweep)
                .unwrap_or_default(),
        };
        let effective = summary.effective.expect("挂载交回有效根");
        let chosen = summary.chosen.expect("挂载交回所选根");
        if !self.ledger.versions.contains_key(&effective) {
            self.ledger.consistency_problems.push(format!(
                "实例 {} 挂载的有效根 {} 不在账本里",
                summary.instance,
                effective.text()
            ));
        }
        self.mark_the_versions_the_mount_applied(chosen, effective);
        let last = self
            .land_operations(&operations, &context, Some(effective))
            .unwrap_or(effective);
        let Mounted {
            output,
            allocator,
            current,
        } = mounted;
        let session = Session {
            instance: output.instance,
            allocator,
            current,
            last_key: last,
        };
        Ok((session, summary))
    }

    /// skr 里实例 2 那一次挂载：挂载在内存里做完，写流截在它第一个根槽写（写行那次发布）之前；那一版记成「发了、根槽没落」。
    fn mount_crashing_at_the_first_root_slot_write(
        &mut self,
        configuration: MountConfiguration,
    ) -> Result<(MountSummary, VersionKey), String> {
        let parameters = self.parameters.clone();
        let (result, operations) =
            self.recorded(|devices| HistoryBuilder::mount_on(&parameters, devices, configuration));
        let mounted = result.map_err(|error| format!("要崩的那次可写挂载：{error:?}"))?;
        let summary = HistoryBuilder::summary_of_the_mount(&mounted);
        self.note_returned_roots(&roots_returned_by_the_mount(
            &mounted.output,
            &mounted.current,
        ));
        let row_publish = summary.row_publish;
        let kind_of = |key: VersionKey| {
            if Some(key) == row_publish {
                PublishKind::RowPublish
            } else {
                PublishKind::OtherMountPublish
            }
        };
        let context = LandingContext {
            kind_of: &kind_of,
            rows_written_by_the_row_publish: mounted
                .output
                .rows_written
                .iter()
                .map(row_triple_of)
                .collect(),
            rows_deleted_by_the_sweep: Vec::new(),
        };
        let crashed =
            self.land_up_to_the_first_root_slot_write(&operations, &context, summary.effective)?;
        Ok((summary, crashed))
    }

    fn run_the_publish(
        parameters: &MakeFilesystemParameters,
        devices: &mut RecordedDevices,
        allocator: &mut PoolAllocator,
        current: &PoolVersion,
        publish: Publish,
        instance: InstanceGeneration,
    ) -> Result<TransactionOutput, String> {
        let mut writer = PoolWriter::new(parameters, devices.as_mut_slice());
        let salt = current.root().checkpoint_txg.0 + 1;
        let write_time_seconds = WRITE_TIME_SECONDS + salt;
        if publish == Publish::FirstFile {
            let content = deterministic_content(0, OVERWRITE_CONTENT_BYTES);
            return publish_first_file(
                &mut writer,
                allocator,
                current.root(),
                FirstFile {
                    content: &content,
                    write_time_seconds,
                },
                instance,
                current.record_bytes(),
            )
            .map_err(|error| format!("第一个文件：{error:?}"));
        }
        let previous = current
            .file_version()
            .ok_or_else(|| "现行那一版没有文件，接不上用户改动".to_string())?;
        let result = match publish {
            Publish::FirstFile => unreachable!("第一个文件在进 match 之前已交回"),
            Publish::UserWrite(UserWrite::Overwrite)
            | Publish::RollbackAction {
                action: RollbackAction::Overwrite,
                ..
            } => {
                let content = deterministic_content(salt, OVERWRITE_CONTENT_BYTES);
                publish_overwrite(
                    &mut writer,
                    allocator,
                    previous,
                    FirstFile {
                        content: &content,
                        write_time_seconds,
                    },
                    instance,
                )
            }
            Publish::UserWrite(UserWrite::SequentialWrite { content_bytes }) => {
                let content = deterministic_content(salt, content_bytes);
                publish_sequential_write(
                    &mut writer,
                    allocator,
                    previous,
                    FirstFile {
                        content: &content,
                        write_time_seconds,
                    },
                    instance,
                )
            }
            Publish::RollbackAction {
                action: RollbackAction::LongerSequentialWrite,
                target,
            } => {
                let mut content = deterministic_content(target, OVERWRITE_CONTENT_BYTES);
                content.extend(deterministic_content(salt, OVERWRITE_CONTENT_BYTES));
                publish_sequential_write(
                    &mut writer,
                    allocator,
                    previous,
                    FirstFile {
                        content: &content,
                        write_time_seconds,
                    },
                    instance,
                )
            }
            Publish::RollbackAction {
                action: RollbackAction::NewInode,
                ..
            } => publish_new_inodes(
                &mut writer,
                allocator,
                previous,
                1,
                write_time_seconds,
                instance,
            ),
            Publish::RollbackAction {
                action: RollbackAction::EmptyPublish,
                ..
            } => publish_version(
                &mut writer,
                allocator,
                PublishPlan {
                    txg: CheckpointTxg(previous.root.checkpoint_txg.0 + 1),
                    counter: previous.record.counter + 1,
                    transaction: 0,
                    highest_transaction_number_before_this_publish: previous
                        .highest_transaction_number_in_this_instance,
                    instance,
                    back_chain: back_chain_of(&previous.record_bytes),
                    file: None,
                    new_inode_records: &[],
                    instance_table: InstanceTablePlan::Carry(previous.root.instance_table),
                    tree_birth_txg: previous.tree_birth_txg(),
                    tree_identifier_watermark: previous.root.tree_identifier_watermark,
                    rollback_floor: previous.root.rollback_floor,
                },
                Some(previous),
            ),
        };
        result.map_err(|error| format!("{}：{error:?}", publish.kind().name()))
    }

    /// 一次发布（第一个文件、用户写、rs1 回退之后的动作），落下的根槽写记进账本，父版本是会话里最后一版。
    fn publish(&mut self, session: &mut Session, publish: Publish) -> Result<VersionKey, String> {
        let parameters = self.parameters.clone();
        let instance = session.instance;
        let current = session.current.clone();
        let allocator = &mut session.allocator;
        let (result, operations) = self.recorded(|devices| {
            HistoryBuilder::run_the_publish(
                &parameters,
                devices,
                allocator,
                &current,
                publish,
                instance,
            )
        });
        let output = result?;
        self.note_returned_roots(&[output.root]);
        let kind = publish.kind();
        let kind_of = move |_key: VersionKey| kind;
        let context = LandingContext {
            kind_of: &kind_of,
            rows_written_by_the_row_publish: Vec::new(),
            rows_deleted_by_the_sweep: Vec::new(),
        };
        let last = self
            .land_operations(&operations, &context, Some(session.last_key))
            .ok_or_else(|| format!("{} 那一次发布的写流里没有根槽写", kind.name()))?;
        session.current = PoolVersion::of_a_version_of_trees(output);
        session.last_key = last;
        Ok(last)
    }

    /// 「崩」的那一次发布：写流截在它的根槽写之前，那一版记成「发了、根槽没落」，会话到此为止。
    fn publish_crashing(
        &mut self,
        mut session: Session,
        publish: Publish,
    ) -> Result<VersionKey, String> {
        let parameters = self.parameters.clone();
        let instance = session.instance;
        let current = session.current.clone();
        let allocator = &mut session.allocator;
        let (result, operations) = self.recorded(|devices| {
            HistoryBuilder::run_the_publish(
                &parameters,
                devices,
                allocator,
                &current,
                publish,
                instance,
            )
        });
        let output = result?;
        self.note_returned_roots(&[output.root]);
        let kind = publish.kind();
        let kind_of = move |_key: VersionKey| kind;
        let context = LandingContext {
            kind_of: &kind_of,
            rows_written_by_the_row_publish: Vec::new(),
            rows_deleted_by_the_sweep: Vec::new(),
        };
        self.land_up_to_the_first_root_slot_write(&operations, &context, Some(session.last_key))
    }

    /// rs1：管理员回退（一次向前发布）到 (实例 1, target)。
    fn roll_back(&mut self, session: &mut Session, target: u64) -> Result<VersionKey, String> {
        let parameters = self.parameters.clone();
        let mut current = session
            .current
            .file_version()
            .cloned()
            .ok_or_else(|| "回退之前现行那一版没有文件".to_string())?;
        let allocator = &mut session.allocator;
        let (result, operations) = self.recorded(|devices| {
            roll_back_by_a_forward_publish(
                &parameters,
                devices,
                allocator,
                &mut current,
                RollbackTarget {
                    instance: InstanceGeneration(1),
                    checkpoint_txg: CheckpointTxg(target),
                },
            )
        });
        let _rolled_back =
            result.map_err(|error| format!("回退到 (实例 1, txg {target})：{error:?}"))?;
        self.note_returned_roots(&[current.root]);
        let kind_of = |_key: VersionKey| PublishKind::Rollback;
        let context = LandingContext {
            kind_of: &kind_of,
            rows_written_by_the_row_publish: Vec::new(),
            rows_deleted_by_the_sweep: Vec::new(),
        };
        let last = self
            .land_operations(&operations, &context, Some(session.last_key))
            .ok_or_else(|| "回退那一次发布的写流里没有根槽写".to_string())?;
        session.current = PoolVersion::of_a_version_of_trees(current);
        session.last_key = last;
        Ok(last)
    }

    /// 此刻镜像与账本的一份拷贝，把点名的几版的根槽各读坏（槽内第 40 字节取反）。槽里已经住着别的版本的不翻、记进交回的第三样。
    fn copy_with_corrupted_root_slots(
        &self,
        versions: &[VersionKey],
    ) -> (MemoryPool, Ledger, Vec<String>) {
        let mut image = self.image.clone();
        let mut ledger = self.ledger.clone();
        let mut skipped = Vec::new();
        for version in versions {
            let ring_slot = ring_slot_by_the_local_formula(version.txg, self.slots_per_region());
            if version.txg == GENESIS_TXG || ledger.ring_occupants.get(&ring_slot) != Some(version)
            {
                skipped.push(format!("{} 的根槽此刻不住着它", version.text()));
                continue;
            }
            let device = REGION_DEVICES[usize::try_from(ring_slot.0).expect("区域号装得进 usize")];
            let offset = slot_offset(
                RootRingSlot {
                    region: ring_slot.0,
                    slot: ring_slot.1,
                },
                self.parameters.geometry.fixed_structure_slot_spacing,
            );
            image.flip_byte(
                DeviceIdentity(device),
                offset,
                CORRUPTED_BYTE_IN_THE_ROOT_SLOT,
            );
            ledger.corrupted_ring_slots.insert(ring_slot);
        }
        (image, ledger, skipped)
    }
}

/// 一次发布是哪一种（`HistoryBuilder::publish` 的参数）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Publish {
    FirstFile,
    UserWrite(UserWrite),
    RollbackAction { action: RollbackAction, target: u64 },
}

impl Publish {
    fn kind(self) -> PublishKind {
        match self {
            Publish::FirstFile => PublishKind::FirstFile,
            Publish::UserWrite(UserWrite::Overwrite)
            | Publish::RollbackAction {
                action: RollbackAction::Overwrite,
                ..
            } => PublishKind::Overwrite,
            Publish::UserWrite(UserWrite::SequentialWrite { .. })
            | Publish::RollbackAction {
                action: RollbackAction::LongerSequentialWrite,
                ..
            } => PublishKind::SequentialWrite,
            Publish::RollbackAction {
                action: RollbackAction::NewInode,
                ..
            } => PublishKind::NewInode,
            Publish::RollbackAction {
                action: RollbackAction::EmptyPublish,
                ..
            } => PublishKind::EmptyPublish,
        }
    }
}

// ============================================================================
// 六、判法这一侧（只读镜像）：候选集、由记录施加出来的版本、相邻对与洞对、两条臂。
//    取法照 `walk.rs` 今天的读法另写一份（登记第一节「候选集与相邻对」、第三节；逐句对照表在执行员报告里）。
// ============================================================================

/// R 的实例表指针出生 txg：读得出的那个数，或指针全零、读不出（甲把这一对按「不认作取号跳过」处理，另计一格）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum PointerBirth {
    Readable(u64),
    AllZeroOrUnreadable,
}

/// 镜像上的一版：候选集里的一条根，或由记录施加出来、根槽从没落盘的一版（两样分别放在 `ImageReadingOfTheJudgements` 的两个字段里）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct VersionOnTheImage {
    key: VersionKey,
    pointer_birth: PointerBirth,
}

/// 根环里一条自证过的根：住的区域、它的版本、带的 F、实例表指针出生 txg。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct RootOnTheRing {
    region: u64,
    key: VersionKey,
    rollback_floor: u64,
    pointer_birth: PointerBirth,
}

/// 环里一条记录（扫环的读法，每个 (实例, 计数器) 一条）：判「由记录施加出来」要的几样。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct RecordOnTheRing {
    key: VersionKey,
    is_commit: bool,
    is_the_last_record_of_the_publish: bool,
    pointer_birth: PointerBirth,
}

/// 最新根实例表的一行。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct RowOfTheNewestTable {
    instance: u32,
    published_checkpoint_txg: u64,
    applied_transaction_high_water_mark: u64,
}

/// 相邻两组之间有没有洞、今天的判法（H1-A，`walk.rs` 的 `hole_in_the_candidate_set_between` 三条件）认不认作行接得上。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum HoleClass {
    NoHole,
    BridgedByTheRows,
    NotBridgedByTheRows,
}

/// H1-A（今天）：最新根的实例表里有一行 (R⁻ 的某个实例 i, T = R⁻.txg)；R 的实例 v > i；i 与 v 之间每个实例 j 都有一行、都是 (j, 0, 0)。
fn hole_class_today(
    previous_txg: u64,
    previous_instances: &BTreeSet<u32>,
    version_txg: u64,
    version_instance: u32,
    rows: &[RowOfTheNewestTable],
) -> HoleClass {
    if previous_txg.checked_add(1) == Some(version_txg) {
        return HoleClass::NoHole;
    }
    let bridged_from = |previous_instance: u32| {
        if version_instance <= previous_instance {
            return false;
        }
        let row_of_the_previous_version = rows.iter().any(|row| {
            row.instance == previous_instance && row.published_checkpoint_txg == previous_txg
        });
        let rows_between: Vec<&RowOfTheNewestTable> = rows
            .iter()
            .filter(|row| row.instance > previous_instance && row.instance < version_instance)
            .collect();
        let instances_between: BTreeSet<u32> =
            rows_between.iter().map(|row| row.instance).collect();
        let every_instance_between_has_a_row = u64::try_from(instances_between.len())
            .expect("行数装得进 u64")
            == u64::from(version_instance - previous_instance - 1);
        let every_row_between_published_nothing = rows_between.iter().all(|row| {
            row.published_checkpoint_txg == 0 && row.applied_transaction_high_water_mark == 0
        });
        row_of_the_previous_version
            && every_instance_between_has_a_row
            && every_row_between_published_nothing
    };
    if previous_instances
        .iter()
        .any(|previous_instance| bridged_from(*previous_instance))
    {
        HoleClass::BridgedByTheRows
    } else {
        HoleClass::NotBridgedByTheRows
    }
}

/// 甲（被测）：H1-A 的三条件，再加第四条件「R 的实例表指针出生 txg 等于 R.txg」；指针全零或读不出按不认作取号跳过。
/// R⁻ 那一组只用来立断言（相邻对里它们都比 R 早），第四条件只读 R 自己的指针。
fn bridged_with_the_pointer_birth(
    hole: HoleClass,
    previous_versions: &[VersionOnTheImage],
    version: &VersionOnTheImage,
) -> bool {
    debug_assert!(previous_versions
        .iter()
        .all(|previous| previous.key.txg < version.key.txg));
    hole == HoleClass::BridgedByTheRows
        && version.pointer_birth == PointerBirth::Readable(version.key.txg)
}

/// 按最新根的实例表判抛弃：有行 (i, Ti) 且 t > Ti。
fn is_abandoned_by_the_rows(key: VersionKey, rows: &[RowOfTheNewestTable]) -> bool {
    rows.iter()
        .any(|row| row.instance == key.instance && key.txg > row.published_checkpoint_txg)
}

/// 择最新根：按 (txg, 实例) 取最大，相同时取读根环次序里最前的那条（只在严格更大时才换）。
fn newest_root_index(roots: &[RootOnTheRing]) -> Option<usize> {
    let mut newest: Option<(usize, (u64, u32))> = None;
    for (index, root) in roots.iter().enumerate() {
        let choice_key = (root.key.txg, root.key.instance);
        match newest {
            Some((_, newest_key)) if choice_key <= newest_key => {}
            Some(_) | None => newest = Some((index, choice_key)),
        }
    }
    newest.map(|(index, _)| index)
}

/// F 生效值：max(每块盘上没被抛弃、(txg, 实例) 最大那条根带的 F, 被抛弃的根带的 F, 自证过、fsid 是本池的系统配置槽带的 F)。
fn effective_rollback_floor(
    roots: &[RootOnTheRing],
    rows: &[RowOfTheNewestTable],
    configuration_floors: &[u64],
) -> u64 {
    let mut newest_valid_on_each_device: BTreeMap<u32, &RootOnTheRing> = BTreeMap::new();
    let mut highest_floor_on_the_abandoned_roots = 0;
    for root in roots {
        if is_abandoned_by_the_rows(root.key, rows) {
            highest_floor_on_the_abandoned_roots =
                highest_floor_on_the_abandoned_roots.max(root.rollback_floor);
            continue;
        }
        let device = REGION_DEVICES[usize::try_from(root.region).expect("区域号装得进 usize")];
        let newest_on_this_device = newest_valid_on_each_device.entry(device).or_insert(root);
        if (root.key.txg, root.key.instance)
            > (
                newest_on_this_device.key.txg,
                newest_on_this_device.key.instance,
            )
        {
            *newest_on_this_device = root;
        }
    }
    let on_the_roots = newest_valid_on_each_device
        .values()
        .map(|root| root.rollback_floor)
        .max()
        .unwrap_or(0)
        .max(highest_floor_on_the_abandoned_roots);
    configuration_floors
        .iter()
        .copied()
        .fold(on_the_roots, u64::max)
}

/// 回退候选集里的根：最新根恒收；别的根不被最新根的实例表判抛弃、txg ≥ F 生效值。按读根环的次序。
fn candidate_root_indexes(
    roots: &[RootOnTheRing],
    rows: &[RowOfTheNewestTable],
    floor: u64,
    newest: usize,
) -> Vec<usize> {
    roots
        .iter()
        .enumerate()
        .filter(|(index, root)| {
            *index == newest || (!is_abandoned_by_the_rows(root.key, rows) && root.key.txg >= floor)
        })
        .map(|(index, _)| index)
        .collect()
}

/// 由记录施加出来、根槽从没落盘的那几版（`crates/singlefs-checker/src/walk.rs` 第 12419 行 `versions_applied_only_by_records`
/// 的文档注释 ①–④ 四条同时成立）：① 带提交标记、最新根的实例表里有这个实例的行
/// 且 txg ≤ Ti；② 根环里自证过的根没有一条是这一版；③ txg ≥ F 生效值；④ 根环里有一条没被抛弃的根 txg 比它小。
/// 同一版几条记录时取带末条标志的那一条的新根段，读不到末条就取先到的那一条（`records` 按 (实例, 计数器) 升序）。
fn versions_applied_only_by_records(
    records: &[RecordOnTheRing],
    roots: &[RootOnTheRing],
    rows: &[RowOfTheNewestTable],
    floor: u64,
) -> Vec<VersionOnTheImage> {
    let Some(oldest_valid_root_txg) = roots
        .iter()
        .filter(|root| !is_abandoned_by_the_rows(root.key, rows))
        .map(|root| root.key.txg)
        .min()
    else {
        return Vec::new();
    };
    let mut taken: BTreeMap<VersionKey, (bool, PointerBirth)> = BTreeMap::new();
    for record in records {
        let applied_by_a_recovery = record.is_commit
            && rows.iter().any(|row| {
                row.instance == record.key.instance
                    && record.key.txg <= row.published_checkpoint_txg
            });
        let root_slot_is_readable = roots.iter().any(|root| root.key == record.key);
        let at_or_above_the_floor = record.key.txg >= floor;
        let its_units_are_not_reclaimable_yet = oldest_valid_root_txg < record.key.txg;
        if !(applied_by_a_recovery
            && !root_slot_is_readable
            && at_or_above_the_floor
            && its_units_are_not_reclaimable_yet)
        {
            continue;
        }
        let takes_this_record = match taken.get(&record.key) {
            None => true,
            Some((from_the_last_record, _)) => {
                !from_the_last_record && record.is_the_last_record_of_the_publish
            }
        };
        if takes_this_record {
            taken.insert(
                record.key,
                (
                    record.is_the_last_record_of_the_publish,
                    record.pointer_birth,
                ),
            );
        }
    }
    taken
        .into_iter()
        .map(|(key, (_, pointer_birth))| VersionOnTheImage { key, pointer_birth })
        .collect()
}

/// 一个洞对 (R⁻ 那一组, R) 与两条臂的分类。
#[derive(Clone, Debug, PartialEq, Eq)]
struct HolePair {
    previous_txg: u64,
    previous_group: Vec<VersionKey>,
    version: VersionOnTheImage,
    bridged_today: bool,
    bridged_with_the_pointer_birth: bool,
}

/// 候选版本按 txg 升序、同 txg 并成一组，相邻两组 (R⁻, R) 为一对，R 侧逐条各算一对；只交回洞对（R.txg > R⁻.txg + 1）。
fn hole_pairs_of(versions: &[VersionOnTheImage], rows: &[RowOfTheNewestTable]) -> Vec<HolePair> {
    let mut groups: BTreeMap<u64, Vec<VersionOnTheImage>> = BTreeMap::new();
    for version in versions {
        groups.entry(version.key.txg).or_default().push(*version);
    }
    let groups: Vec<(u64, Vec<VersionOnTheImage>)> = groups.into_iter().collect();
    let mut pairs = Vec::new();
    for adjacent in groups.windows(2) {
        let (previous_txg, previous_versions) = (adjacent[0].0, &adjacent[0].1);
        let previous_instances: BTreeSet<u32> = previous_versions
            .iter()
            .map(|version| version.key.instance)
            .collect();
        let previous_group: Vec<VersionKey> = previous_versions
            .iter()
            .map(|version| version.key)
            .collect();
        for version in &adjacent[1].1 {
            let hole = hole_class_today(
                previous_txg,
                &previous_instances,
                version.key.txg,
                version.key.instance,
                rows,
            );
            if hole == HoleClass::NoHole {
                continue;
            }
            pairs.push(HolePair {
                previous_txg,
                previous_group: previous_group.clone(),
                version: *version,
                bridged_today: hole == HoleClass::BridgedByTheRows,
                bridged_with_the_pointer_birth: bridged_with_the_pointer_birth(
                    hole,
                    previous_versions,
                    version,
                ),
            });
        }
    }
    pairs
}

/// 一张镜像上判法读到的东西。
#[derive(Clone, Debug)]
struct ImageReadingOfTheJudgements {
    newest: VersionKey,
    rows: Vec<RowOfTheNewestTable>,
    effective_floor: u64,
    candidates: Vec<VersionOnTheImage>,
    applied_only_by_records: Vec<VersionOnTheImage>,
    hole_pairs: Vec<HolePair>,
}

fn pointer_birth_of(pointer: &NodePointer, birth_from_the_bytes: Option<u64>) -> PointerBirth {
    if *pointer == NodePointer::empty_root() {
        return PointerBirth::AllZeroOrUnreadable;
    }
    birth_from_the_bytes.map_or(PointerBirth::AllZeroOrUnreadable, PointerBirth::Readable)
}

/// 只读镜像：择系统配置、读根环（每条根连同槽，槽里 [212, 220) 字节取出生 txg）、读最新根的实例表、算 F 生效值、候选集、扫环认由记录施加出来的版本、
/// 取洞对。系统配置择不出、根环一条根都没有、最新根的实例表读不出时交 `Err`（这一张不进 E1 / E2，单列；登记第十一节 Z6）。
fn read_the_image(image: &MemoryPool) -> Result<ImageReadingOfTheJudgements, String> {
    let system_configuration = choose_system_configuration(image)
        .map_err(|failure| format!("系统配置择不出：{failure:?}"))?;
    let immutable = &system_configuration.immutable;
    let ring = readable_roots_with_ring_slots(
        image,
        &immutable.region_devices,
        &immutable.sizes,
        &immutable.filesystem_identifier,
    );
    let mut roots = Vec::new();
    for (ring_slot, root) in &ring {
        let ring_slot = *ring_slot;
        let device = immutable.region_devices
            [usize::try_from(ring_slot.region).expect("区域号装得进 usize")];
        let offset = slot_offset(ring_slot, immutable.sizes.fixed_structure_slot_spacing);
        let slot_bytes = singlefs_core::recovery::PoolReader::read(
            image,
            device,
            offset,
            usize::try_from(PHYSICAL_BLOCK_BYTES).expect("512 装得进 usize"),
        );
        let birth_in_bytes = slot_bytes
            .as_deref()
            .and_then(pointer_birth_txg_in_root_slot_bytes);
        roots.push(RootOnTheRing {
            region: ring_slot.region,
            key: VersionKey::of_root(root),
            rollback_floor: root.rollback_floor.0,
            pointer_birth: pointer_birth_of(&root.instance_table, birth_in_bytes),
        });
    }
    let newest =
        newest_root_index(&roots).ok_or_else(|| "根环里一条自证过的根都没有".to_string())?;
    let newest_key = roots[newest].key;
    let newest_root = ring[newest].1;
    let rows: Vec<RowOfTheNewestTable> = instance_table_of_root(image, &newest_root)
        .ok_or_else(|| format!("最新根 {} 的实例表读不出", newest_key.text()))?
        .rows
        .iter()
        .map(|row| RowOfTheNewestTable {
            instance: row.instance.0,
            published_checkpoint_txg: row.selected_root_txg.0,
            applied_transaction_high_water_mark: row.applied_transaction_high_water,
        })
        .collect();
    let configuration_floors: Vec<u64> = verified_system_configuration_slots(image)
        .into_iter()
        .flat_map(|(_, slots)| slots.into_iter().map(|(view, _)| view))
        .filter(|view| view.filesystem_identifier == immutable.filesystem_identifier)
        .map(|view| view.rollback_floor)
        .collect();
    let effective_floor = effective_rollback_floor(&roots, &rows, &configuration_floors);
    let candidates: Vec<VersionOnTheImage> =
        candidate_root_indexes(&roots, &rows, effective_floor, newest)
            .into_iter()
            .map(|index| VersionOnTheImage {
                key: roots[index].key,
                pointer_birth: roots[index].pointer_birth,
            })
            .collect();
    let records: Vec<RecordOnTheRing> = scan_journal(image, &system_configuration)
        .into_iter()
        .map(|((instance, _counter), record)| RecordOnTheRing {
            key: VersionKey {
                instance: instance.0,
                txg: record.checkpoint_txg.0,
            },
            is_commit: record.is_commit,
            is_the_last_record_of_the_publish: record.place_in_publish
                == JournalRecordPlaceInPublish::LastRecordOfThePublish,
            pointer_birth: pointer_birth_of(
                &record.new_instance_table,
                Some(record.new_instance_table.head.birth_txg.0),
            ),
        })
        .collect();
    let applied_only_by_records =
        versions_applied_only_by_records(&records, &roots, &rows, effective_floor);
    let mut versions = candidates.clone();
    versions.extend(applied_only_by_records.iter().copied());
    let hole_pairs = hole_pairs_of(&versions, &rows);
    Ok(ImageReadingOfTheJudgements {
        newest: newest_key,
        rows,
        effective_floor,
        candidates,
        applied_only_by_records,
        hole_pairs,
    })
}

// ============================================================================
// 七、一张镜像：两条臂的分类对上账本的真值、停机条款 S1、S3、S5 与（nrb、rs1 两组）今天的池级 checker。
// ============================================================================

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
struct ArmCounts {
    today_e1: u64,
    today_e2: u64,
    with_pointer_birth_e1: u64,
    with_pointer_birth_e2: u64,
}

impl ArmCounts {
    fn add(&mut self, other: ArmCounts) {
        self.today_e1 += other.today_e1;
        self.today_e2 += other.today_e2;
        self.with_pointer_birth_e1 += other.with_pointer_birth_e1;
        self.with_pointer_birth_e2 += other.with_pointer_birth_e2;
    }
}

/// 一个洞对上两条臂各错了哪一样：E1 = 认作取号跳过而真值不是直接接上；E2 = 真值直接接上而不认作取号跳过（登记第一节）。
/// 真值是「R 不在账本里」的不计。
fn arm_counts_of_the_pair(pair: &HolePair, truth: Truth) -> ArmCounts {
    let directly_bridged = match truth {
        Truth::DirectlyBridged => true,
        Truth::ExistedVersionsInBetween | Truth::PreviousNotOnTheChain => false,
        Truth::VersionMissingFromTheLedger => return ArmCounts::default(),
    };
    ArmCounts {
        today_e1: u64::from(pair.bridged_today && !directly_bridged),
        today_e2: u64::from(directly_bridged && !pair.bridged_today),
        with_pointer_birth_e1: u64::from(pair.bridged_with_the_pointer_birth && !directly_bridged),
        with_pointer_birth_e2: u64::from(directly_bridged && !pair.bridged_with_the_pointer_birth),
    }
}

/// 停机条款 S5 的读数：装置按 ①–④ 认出的「由记录施加出来」的版本，对账本里两支（被某次挂载施加过、根槽没落；根槽落过、后来被环盖掉或读坏，登记修订 3），
/// 两支都要账本推的行与根环占位让 ① ② ④ 成立（登记修订 4）、记录此刻还读得出的那几版。装置多认的按账本里那一版的根槽落没落过分两格
/// （落过的那一格是账本两支都没收进的形，单列给主 agent 看）。
#[derive(Clone, Debug, Default)]
struct RecordsAppliedComparison {
    apparatus_extra_with_a_root_that_landed_once: Vec<VersionKey>,
    apparatus_extra_otherwise: Vec<VersionKey>,
    apparatus_missing: Vec<VersionKey>,
}

impl RecordsAppliedComparison {
    fn mismatch_count(&self) -> usize {
        self.apparatus_extra_with_a_root_that_landed_once.len()
            + self.apparatus_extra_otherwise.len()
            + self.apparatus_missing.len()
    }
}

#[derive(Clone, Debug)]
struct ImageResult {
    label: String,
    step: u64,
    reading: Result<ImageReadingOfTheJudgements, String>,
    truths: Vec<Truth>,
    counts: ArmCounts,
    pointer_unreadable_pairs: u64,
    ring_slot_mismatches: Vec<String>,
    row_mismatch: Option<String>,
    records_applied: RecordsAppliedComparison,
    checker_verdicts: Option<(String, String)>,
    corrupted_skipped: Vec<String>,
}

fn verdict_name(verdict: &InvariantVerdict) -> &'static str {
    match verdict {
        InvariantVerdict::Holds => "holds",
        InvariantVerdict::Violated(_) => "violated",
        InvariantVerdict::NotApplicable(_) => "not_applicable",
        InvariantVerdict::PartiallyJudged(_) => "partially_judged",
    }
}

/// S1：账本推的每个根环槽（写过的、没读坏的）住哪一版，对 `root_ring_the_ring_judgements_read` 逐槽交回的。
fn ring_slot_mismatches(image: &MemoryPool, ledger: &Ledger) -> Vec<String> {
    let predicted: BTreeMap<(u64, u64), (u32, u64)> = ledger
        .ring_occupants
        .iter()
        .filter(|(ring_slot, _)| !ledger.corrupted_ring_slots.contains(*ring_slot))
        .map(|(ring_slot, key)| (*ring_slot, (key.instance, key.txg)))
        .collect();
    let Some(read) = root_ring_the_ring_judgements_read(image) else {
        return vec!["池级 checker 择不到系统配置，根环没读".to_string()];
    };
    let actual: BTreeMap<(u64, u64), (u32, u64)> = read
        .iter()
        .map(|slot| {
            (
                (slot.region, slot.slot_in_region),
                (slot.instance, slot.checkpoint_txg),
            )
        })
        .collect();
    let every_slot: BTreeSet<(u64, u64)> = predicted.keys().chain(actual.keys()).copied().collect();
    every_slot
        .into_iter()
        .filter(|ring_slot| predicted.get(ring_slot) != actual.get(ring_slot))
        .map(|ring_slot| {
            format!(
                "槽 {ring_slot:?}：账本 {:?}，池级 checker {:?}",
                predicted.get(&ring_slot),
                actual.get(&ring_slot)
            )
        })
        .collect()
}

/// S3：账本推的最新根那张实例表的行集，对镜像上读出的。
fn row_mismatch(ledger: &Ledger, reading: &ImageReadingOfTheJudgements) -> Option<String> {
    let on_the_image: BTreeSet<RowTriple> = reading
        .rows
        .iter()
        .map(|row| {
            (
                row.instance,
                row.published_checkpoint_txg,
                row.applied_transaction_high_water_mark,
            )
        })
        .collect();
    let Some(version) = ledger.versions.get(&reading.newest) else {
        return Some(format!("最新根 {} 不在账本里", reading.newest.text()));
    };
    (*version.rows != on_the_image).then(|| {
        format!(
            "最新根 {}：账本推的行 {:?}，镜像上读出 {:?}",
            reading.newest.text(),
            version.rows,
            on_the_image
        )
    })
}

/// S5 账本这一边（登记修订 3、修订 4）：记录此刻还读得出、账本按自己推的行与根环占位认 ① ② ④ 成立的版本，分两支：
/// 被某次挂载施加过、根槽没落的；根槽落过、没被某次挂载施加过、后来被环盖掉或根槽读坏的。
/// ① 最新根的实例表（账本推的行集）里有这个实例的行 (i, Ti) 且 txg ≤ Ti；② 此刻读得出的根环槽（账本推的占位，减去读坏的槽）里没有这一版；
/// ④ 此刻读得出的根环槽里有一条没被最新根的实例表判抛弃的根，txg 比它小。③（不低于回退下界）在 ss、skr、sw 各组恒成立（这几组不抬 F），账本不判。
/// 两支都要过 ④：环转过去、最旧的有效根比这一版新之后，这一版换下的单元已可回收，装置按 ④ 不认，账本也不收（修订 4：sw-crash 的 k = 9 那一张，实例 3 崩掉的那一版）。
fn version_the_records_alone_show(
    ledger: &Ledger,
    rows_of_the_newest: &BTreeSet<RowTriple>,
    key: VersionKey,
    version: &LedgerVersion,
) -> bool {
    let applied_by_a_mount_without_a_root = version.applied_by_a_mount && !version.root_landed;
    let covered_after_its_root_landed = version.root_landed && !version.applied_by_a_mount;
    if !(applied_by_a_mount_without_a_root || covered_after_its_root_landed) {
        return false;
    }
    let readable_on_the_ring: Vec<VersionKey> = ledger
        .ring_occupants
        .iter()
        .filter(|(ring_slot, _)| !ledger.corrupted_ring_slots.contains(*ring_slot))
        .map(|(_, occupant)| *occupant)
        .collect();
    let applied_by_a_recovery =
        rows_of_the_newest
            .iter()
            .any(|(instance, published_checkpoint_txg, _)| {
                *instance == key.instance && key.txg <= *published_checkpoint_txg
            });
    let not_on_the_ring = !readable_on_the_ring.contains(&key);
    let abandoned_by_the_rows = |root: &VersionKey| {
        rows_of_the_newest
            .iter()
            .any(|(instance, published_checkpoint_txg, _)| {
                *instance == root.instance && root.txg > *published_checkpoint_txg
            })
    };
    let an_older_valid_root = readable_on_the_ring
        .iter()
        .any(|root| !abandoned_by_the_rows(root) && root.txg < key.txg);
    applied_by_a_recovery && not_on_the_ring && an_older_valid_root
}

/// S5：装置认出的由记录施加出来的版本，对账本里 `version_the_records_alone_show` 收的版本（两支，各过 ① ② ④，登记修订 3、修订 4）减去此刻记录已读不出的
/// （`journal_records_as_the_recovery_reads_them` 读，按 (实例, txg) 认）。
fn records_applied_comparison(
    image: &MemoryPool,
    ledger: &Ledger,
    reading: &ImageReadingOfTheJudgements,
) -> RecordsAppliedComparison {
    let readable: BTreeSet<VersionKey> = journal_records_as_the_recovery_reads_them(image)
        .unwrap_or_default()
        .iter()
        .map(|record| VersionKey {
            instance: record.instance,
            txg: record.checkpoint_txg,
        })
        .collect();
    let rows_of_the_newest = ledger.rows_of(Some(reading.newest));
    let in_the_ledger: BTreeSet<VersionKey> = ledger
        .versions
        .iter()
        .filter(|(key, version)| {
            readable.contains(*key)
                && version_the_records_alone_show(ledger, &rows_of_the_newest, **key, version)
        })
        .map(|(key, _)| *key)
        .collect();
    let by_the_apparatus: BTreeSet<VersionKey> = reading
        .applied_only_by_records
        .iter()
        .map(|version| version.key)
        .collect();
    let mut comparison = RecordsAppliedComparison::default();
    for extra in by_the_apparatus.difference(&in_the_ledger) {
        if ledger
            .versions
            .get(extra)
            .is_some_and(|version| version.root_landed)
        {
            comparison
                .apparatus_extra_with_a_root_that_landed_once
                .push(*extra);
        } else {
            comparison.apparatus_extra_otherwise.push(*extra);
        }
    }
    comparison.apparatus_missing = in_the_ledger
        .difference(&by_the_apparatus)
        .copied()
        .collect();
    comparison
}

/// 判一张镜像：两条臂的洞对分类、真值、停机条款 S1、S3、S5；`run_the_pool_checker` 时另跑今天的 `check_pool_image` 取 I-3.9 与 I-3.5。
// gpu-exempt: experiment-measures-the-cpu-judge-itself
fn judge_the_image(
    image: &MemoryPool,
    ledger: &Ledger,
    label: String,
    step: u64,
    run_the_pool_checker: bool,
    corrupted_skipped: Vec<String>,
) -> ImageResult {
    let reading = read_the_image(image);
    let mut truths = Vec::new();
    let mut counts = ArmCounts::default();
    let mut pointer_unreadable_pairs = 0;
    let mut row_mismatch_text = None;
    let mut records_applied = RecordsAppliedComparison::default();
    if let Ok(reading) = &reading {
        for pair in &reading.hole_pairs {
            let truth = truth_of_the_hole_pair(ledger, &pair.previous_group, pair.version.key);
            truths.push(truth);
            counts.add(arm_counts_of_the_pair(pair, truth));
            if pair.version.pointer_birth == PointerBirth::AllZeroOrUnreadable {
                pointer_unreadable_pairs += 1;
            }
        }
        row_mismatch_text = row_mismatch(ledger, reading);
        records_applied = records_applied_comparison(image, ledger, reading);
    }
    let checker_verdicts = run_the_pool_checker.then(|| {
        let verdicts = check_pool_image(image);
        let name_of = |invariant: &str| {
            verdicts
                .iter()
                .find(|(name, _)| *name == invariant)
                .map_or("missing", |(_, verdict)| verdict_name(verdict))
                .to_string()
        };
        (name_of("I-3.9"), name_of("I-3.5"))
    });
    ImageResult {
        label,
        step,
        reading,
        truths,
        counts,
        pointer_unreadable_pairs,
        ring_slot_mismatches: ring_slot_mismatches(image, ledger),
        row_mismatch: row_mismatch_text,
        records_applied,
        checker_verdicts,
        corrupted_skipped,
    }
}

// ============================================================================
// 八、前提 P1–P3（只读账本，登记第一节「前提」、第六节 Q1–Q3）。
// ============================================================================

#[derive(Clone, Debug, Default)]
struct PremiseCounts {
    /// P1：写过行的实例（实例代号 ≥ 2）第一版的指针出生 txg 不等于它自己的 txg。
    first_version_violations: Vec<String>,
    /// PC-P1：实例 1 的第一版报不成立的次数（不计入 Q2）。
    instance_one_first_version_mismatches: u64,
    /// P2：同实例相邻两个存在过的版本，后一版的出生 txg 既不等于前一版的、又不是「清扫发布、等于自己的 txg」。
    adjacent_version_violations: Vec<String>,
    /// PC-P2：清扫换指针的次数。
    sweep_pointer_changes: u64,
    /// P3：出生 txg 大于自己 txg 的版本（实例 1 也算）。
    birth_after_the_txg_violations: Vec<String>,
}

fn premise_counts(ledger: &Ledger) -> PremiseCounts {
    let mut counts = PremiseCounts::default();
    let mut by_instance: BTreeMap<u32, Vec<(VersionKey, &LedgerVersion)>> = BTreeMap::new();
    for (key, version) in &ledger.versions {
        by_instance
            .entry(key.instance)
            .or_default()
            .push((*key, version));
        if version.pointer_birth_txg > key.txg {
            counts.birth_after_the_txg_violations.push(format!(
                "{}（{}）出生 txg {}",
                key.text(),
                version.kind.name(),
                version.pointer_birth_txg
            ));
        }
    }
    for (instance, mut versions) in by_instance {
        versions.sort_by_key(|(_, version)| version.publish_order);
        if instance == 0 {
            continue;
        }
        if let Some((first_key, first)) = versions.first() {
            let mismatch = first.pointer_birth_txg != first_key.txg;
            if instance == 1 {
                counts.instance_one_first_version_mismatches += u64::from(mismatch);
            } else if mismatch {
                counts.first_version_violations.push(format!(
                    "{}（{}）出生 txg {}",
                    first_key.text(),
                    first.kind.name(),
                    first.pointer_birth_txg
                ));
            }
        }
        if instance == 1 {
            continue;
        }
        let existed: Vec<&(VersionKey, &LedgerVersion)> = versions
            .iter()
            .filter(|(_, version)| version.existed())
            .collect();
        for adjacent in existed.windows(2) {
            let (previous_key, previous) = adjacent[0];
            let (next_key, next) = adjacent[1];
            if next.kind == PublishKind::Sweep && next.pointer_birth_txg == next_key.txg {
                counts.sweep_pointer_changes += 1;
            } else if next.pointer_birth_txg != previous.pointer_birth_txg {
                counts.adjacent_version_violations.push(format!(
                    "{} → {}（{}）出生 txg {} → {}",
                    previous_key.text(),
                    next_key.text(),
                    next.kind.name(),
                    previous.pointer_birth_txg,
                    next.pointer_birth_txg
                ));
            }
        }
    }
    counts
}

// ============================================================================
// 九、每一组怎么造（登记第五节取样域表），每一版之后按组取镜像、当场判（不留整份镜像，一段历史只留读数）。
// ============================================================================

/// 清扫那一形（sw 三组与 G1）的读数：清扫那个实例的挂载怎么扫的、清扫那一版、按读码推的 b + 2 + w。
#[derive(Clone, Debug, Default)]
struct SweepReading {
    sweep_instance: u32,
    outcome: String,
    sweep_version: Option<VersionKey>,
    last_txg_before_the_sweep_instance: u64,
    predicted_sweep_txg: u64,
    instances_mounted_before_the_sweep: u32,
}

#[derive(Clone, Debug)]
struct HistoryResult {
    specification: HistorySpecification,
    name: String,
    images: Vec<ImageResult>,
    /// 写者在哪一步返回 `Err`（合法结局、之后没有镜像）；没停是 `None`。
    stopped: Option<String>,
    sweep: Option<SweepReading>,
    premise: PremiseCounts,
    ledger: Ledger,
}

/// 暖机次数的读码推法（`anchors.py` 的 `w_of`，登记第四节第 4 条）：写行那一版落在哪块盘，补到两块盘都有本实例的根，至多 3 次。
fn predicted_warm_up_publishes(row_publish_txg: u64) -> u64 {
    let device_of = |txg: u64| {
        REGION_DEVICES[usize::try_from(txg % ROOT_RING_REGIONS).expect("区域号装得进 usize")]
    };
    let mut covered = BTreeSet::from([device_of(row_publish_txg)]);
    let mut warm_ups = 0;
    let mut txg = row_publish_txg;
    while covered.len() < 2 && warm_ups < 3 {
        txg += 1;
        covered.insert(device_of(txg));
        warm_ups += 1;
    }
    warm_ups
}

/// mkfs → 实例 1 挂载 → 第一个文件 → 用户写到 txg `last`。
fn build_the_first_instance(
    builder: &mut HistoryBuilder,
    configuration: MountConfiguration,
    last: u64,
    user_write: UserWrite,
) -> Result<Session, String> {
    builder.make_filesystem()?;
    let (mut session, _) = builder.mount(configuration)?;
    builder.publish(&mut session, Publish::FirstFile)?;
    while session.last_key.txg < last {
        builder.publish(&mut session, Publish::UserWrite(user_write))?;
    }
    Ok(session)
}

fn write_until(
    builder: &mut HistoryBuilder,
    session: &mut Session,
    last: u64,
    user_write: UserWrite,
) -> Result<(), String> {
    while session.last_key.txg < last {
        builder.publish(session, Publish::UserWrite(user_write))?;
    }
    Ok(())
}

struct ImageSink<'images> {
    images: &'images mut Vec<ImageResult>,
    run_the_pool_checker: bool,
}

impl ImageSink<'_> {
    fn judge_now(&mut self, builder: &HistoryBuilder, label: String, step: u64) {
        self.images.push(judge_the_image(
            &builder.image,
            &builder.ledger,
            label,
            step,
            self.run_the_pool_checker,
            Vec::new(),
        ));
    }

    fn judge_with_corrupted_root_slots(
        &mut self,
        builder: &HistoryBuilder,
        versions: &[VersionKey],
        label: String,
        step: u64,
    ) {
        let (image, ledger, skipped) = builder.copy_with_corrupted_root_slots(versions);
        self.images.push(judge_the_image(
            &image,
            &ledger,
            label,
            step,
            self.run_the_pool_checker,
            skipped,
        ));
    }
}

/// ss：实例 1 覆盖写到 last → last + 1 覆盖写崩 → 实例 2 产品挂载 → 顺序写 k 次，k = 0 … 3S 每个 k 一张。
fn build_the_short_ring(
    builder: &mut HistoryBuilder,
    sink: &mut ImageSink<'_>,
    last: u64,
    chunks: u64,
) -> Result<(), String> {
    let first_instance = build_the_first_instance(
        builder,
        MountConfiguration::Product,
        last,
        UserWrite::Overwrite,
    )?;
    builder.publish_crashing(first_instance, Publish::UserWrite(UserWrite::Overwrite))?;
    let (mut session, _) = builder.mount(MountConfiguration::Product)?;
    sink.judge_now(builder, "publishes_after_the_mount=0".to_string(), 0);
    let content_bytes =
        usize::try_from(chunks).expect("块数装得进 usize") * SEQUENTIAL_WRITE_CHUNK_BYTES;
    for step in 1..=3 * builder.slots_per_region() {
        builder.publish(
            &mut session,
            Publish::UserWrite(UserWrite::SequentialWrite { content_bytes }),
        )?;
        sink.judge_now(builder, format!("publishes_after_the_mount={step}"), step);
    }
    Ok(())
}

/// skr：实例 1 到 last → 实例 2 产品挂载崩在写行那次根槽写之前 → 实例 3 产品挂载 → 覆盖写 k 次，k = 0 … 14。
fn build_the_skipped_row(
    builder: &mut HistoryBuilder,
    sink: &mut ImageSink<'_>,
    last: u64,
) -> Result<(), String> {
    build_the_first_instance(
        builder,
        MountConfiguration::Product,
        last,
        UserWrite::Overwrite,
    )?;
    builder.mount_crashing_at_the_first_root_slot_write(MountConfiguration::Product)?;
    let (mut session, _) = builder.mount(MountConfiguration::Product)?;
    sink.judge_now(builder, "publishes_after_the_mount=0".to_string(), 0);
    for step in 1..=14 {
        builder.publish(&mut session, Publish::UserWrite(UserWrite::Overwrite))?;
        sink.judge_now(builder, format!("publishes_after_the_mount={step}"), step);
    }
    Ok(())
}

/// nrb：实例 1 到 last → 实例 2 产品挂载（写行 r）→ 用户写 k 次 → 读坏 r … r + bad − 1 的根槽；实例 1 的覆盖写与挂载之后的写同一种写法
/// （线索模型 `rerun-all.sh` 的 nrb 段：顺序写那一组 `PROTO_SEQ=1` 管到实例 1 的每次覆盖写）。
fn build_the_newest_row_bad(
    builder: &mut HistoryBuilder,
    sink: &mut ImageSink<'_>,
    last: u64,
    steps_after_the_mount: u64,
    bad_slots: u64,
    user_write: UserWrite,
) -> Result<(), String> {
    build_the_first_instance(builder, MountConfiguration::Product, last, user_write)?;
    let (mut session, summary) = builder.mount(MountConfiguration::Product)?;
    let row_publish = summary
        .row_publish
        .ok_or_else(|| "实例 2 的挂载没有写行那次发布".to_string())?;
    for _ in 0..steps_after_the_mount {
        builder.publish(&mut session, Publish::UserWrite(user_write))?;
    }
    let bad: Vec<VersionKey> = (row_publish.txg..row_publish.txg + bad_slots)
        .map(|txg| VersionKey {
            instance: row_publish.instance,
            txg,
        })
        .collect();
    let label = format!(
        "bad={}",
        bad.iter()
            .map(|key| key.text())
            .collect::<Vec<_>>()
            .join(",")
    );
    sink.judge_with_corrupted_root_slots(builder, &bad, label, 0);
    Ok(())
}

/// rs1：实例 1 到 10 → 实例 2 产品挂载（写行 r）→ 回退到 (实例 1, target)（那一版 v）→ 动作串 → R ∈ {v, …, v + 动作数} 每个一张，
/// 读坏实例 2 在 [r, R) 里的每一版的根槽（R 成了洞右边第一条读得出的根）。
fn build_the_rollback_slot_bad(
    builder: &mut HistoryBuilder,
    sink: &mut ImageSink<'_>,
    target: u64,
    actions: &[RollbackAction],
) -> Result<(), String> {
    build_the_first_instance(
        builder,
        MountConfiguration::Product,
        ROLLBACK_FIRST_INSTANCE_LAST,
        UserWrite::Overwrite,
    )?;
    let (mut session, summary) = builder.mount(MountConfiguration::Product)?;
    let row_publish = summary
        .row_publish
        .ok_or_else(|| "实例 2 的挂载没有写行那次发布".to_string())?;
    let rollback = builder.roll_back(&mut session, target)?;
    for action in actions {
        builder.publish(
            &mut session,
            Publish::RollbackAction {
                action: *action,
                target,
            },
        )?;
    }
    let last_txg = session.last_key.txg;
    for (step, first_readable_txg) in (rollback.txg..=last_txg).enumerate() {
        let bad: Vec<VersionKey> = builder
            .ledger
            .versions
            .keys()
            .filter(|key| {
                key.instance == row_publish.instance
                    && key.txg >= row_publish.txg
                    && key.txg < first_readable_txg
            })
            .copied()
            .collect();
        sink.judge_with_corrupted_root_slots(
            builder,
            &bad,
            format!("R={first_readable_txg}"),
            u64::try_from(step).expect("步号装得进 u64"),
        );
    }
    Ok(())
}

/// sw 三组与 G1（登记第五节取样域表、第八节 G1）。
fn build_the_sweep(
    builder: &mut HistoryBuilder,
    sink: &mut ImageSink<'_>,
    first_instance_last: u64,
    extra_overwrites: u64,
    threshold: u64,
    accumulate_rows_by_mounts: bool,
    variant: SweepVariant,
) -> Result<SweepReading, String> {
    let configuration = MountConfiguration::SweepAtTheEndAboveTheThreshold { threshold };
    let ring_slots = 3 * builder.slots_per_region();
    build_the_first_instance(
        builder,
        configuration,
        first_instance_last,
        UserWrite::Overwrite,
    )?;
    let (
        mut sweep_session,
        sweep_summary,
        last_txg_before_the_sweep_instance,
        instances_mounted_before_the_sweep,
    ) = if accumulate_rows_by_mounts {
        let mut mounted_without_a_sweep = 0;
        loop {
            let before = builder
                .ledger
                .versions
                .keys()
                .max_by_key(|key| (key.txg, key.instance))
                .copied();
            let (session, summary) = builder.mount(configuration)?;
            if summary.sweep_version.is_some() {
                let last_version_before_the_sweep_mount =
                    before.ok_or_else(|| "清扫那次挂载之前账本是空的".to_string())?;
                break (
                    session,
                    summary,
                    last_version_before_the_sweep_mount.txg,
                    mounted_without_a_sweep,
                );
            }
            mounted_without_a_sweep += 1;
            if summary.instance >= SWEEP_ROW_ACCUMULATION_INSTANCES_AT_MOST {
                return Err(format!(
                    "挂到实例 {} 还没扫（阈值 {threshold}）：清扫那一形没造出来",
                    summary.instance
                ));
            }
        }
    } else {
        let (mut second, _) = builder.mount(configuration)?;
        let last_txg_before_the_sweep_instance =
            first_instance_last + ring_slots + extra_overwrites;
        write_until(
            builder,
            &mut second,
            last_txg_before_the_sweep_instance,
            UserWrite::Overwrite,
        )?;
        drop(second);
        let (session, summary) = builder.mount(configuration)?;
        (session, summary, last_txg_before_the_sweep_instance, 1)
    };
    let reading = SweepReading {
        sweep_instance: sweep_summary.instance,
        outcome: sweep_summary.sweep_outcome.clone(),
        sweep_version: sweep_summary.sweep_version,
        last_txg_before_the_sweep_instance,
        predicted_sweep_txg: last_txg_before_the_sweep_instance
            + 2
            + predicted_warm_up_publishes(last_txg_before_the_sweep_instance + 1),
        instances_mounted_before_the_sweep,
    };
    let last_before_the_crash = last_txg_before_the_sweep_instance + ring_slots - 1;
    match variant {
        SweepVariant::ReadFaultOnTheSweepInstance => {
            let first_version_on_the_image = sweep_session.last_key.txg;
            let bad: Vec<VersionKey> = builder
                .ledger
                .versions
                .keys()
                .filter(|key| {
                    key.instance == sweep_summary.instance && key.txg < first_version_on_the_image
                })
                .copied()
                .collect();
            sink.judge_with_corrupted_root_slots(
                builder,
                &bad,
                format!("version_txg={first_version_on_the_image}"),
                0,
            );
            let mut step = 0;
            while sweep_session.last_key.txg < last_before_the_crash {
                builder.publish(&mut sweep_session, Publish::UserWrite(UserWrite::Overwrite))?;
                step += 1;
                sink.judge_with_corrupted_root_slots(
                    builder,
                    &bad,
                    format!("version_txg={}", sweep_session.last_key.txg),
                    step,
                );
            }
        }
        SweepVariant::CrashBeforeTheNextInstance | SweepVariant::NoCrash => {
            write_until(
                builder,
                &mut sweep_session,
                last_before_the_crash,
                UserWrite::Overwrite,
            )?;
            if variant == SweepVariant::CrashBeforeTheNextInstance {
                builder
                    .publish_crashing(sweep_session, Publish::UserWrite(UserWrite::Overwrite))?;
            } else {
                builder.publish(&mut sweep_session, Publish::UserWrite(UserWrite::Overwrite))?;
                drop(sweep_session);
            }
            let (mut next_session, _) = builder.mount(configuration)?;
            sink.judge_now(builder, "publishes_after_the_mount=0".to_string(), 0);
            for step in 1..=ring_slots {
                builder.publish(&mut next_session, Publish::UserWrite(UserWrite::Overwrite))?;
                sink.judge_now(builder, format!("publishes_after_the_mount={step}"), step);
            }
        }
    }
    Ok(reading)
}

/// 建一段历史、当场判每一张镜像；写者返回 `Err` 的那一步之后没有镜像，`stopped` 记原因（合法结局，登记第一节待认项 4）。
fn run_the_history(specification: &HistorySpecification) -> HistoryResult {
    let mut builder = HistoryBuilder::new(specification);
    let mut images = Vec::new();
    let run_the_pool_checker = matches!(
        specification.group,
        Group::NewestRowBad | Group::RollbackSlotBad
    );
    let mut sweep = None;
    let outcome = {
        let mut sink = ImageSink {
            images: &mut images,
            run_the_pool_checker,
        };
        match &specification.shape {
            HistoryShape::ShortRing { last, chunks } => {
                build_the_short_ring(&mut builder, &mut sink, *last, *chunks)
            }
            HistoryShape::SkippedRow { last } => {
                build_the_skipped_row(&mut builder, &mut sink, *last)
            }
            HistoryShape::NewestRowBad {
                last,
                steps_after_the_mount,
                bad_slots,
                user_write,
            } => build_the_newest_row_bad(
                &mut builder,
                &mut sink,
                *last,
                *steps_after_the_mount,
                *bad_slots,
                *user_write,
            ),
            HistoryShape::RollbackSlotBad { target, actions } => {
                build_the_rollback_slot_bad(&mut builder, &mut sink, *target, actions)
            }
            HistoryShape::Sweep {
                first_instance_last,
                extra_overwrites,
                threshold,
                accumulate_rows_by_mounts,
                variant,
            } => build_the_sweep(
                &mut builder,
                &mut sink,
                *first_instance_last,
                *extra_overwrites,
                *threshold,
                *accumulate_rows_by_mounts,
                *variant,
            )
            .map(|reading| {
                sweep = Some(reading);
            }),
        }
    };
    HistoryResult {
        specification: specification.clone(),
        name: history_name(specification),
        images,
        stopped: outcome.err(),
        sweep,
        premise: premise_counts(&builder.ledger),
        ledger: builder.ledger,
    }
}

// ============================================================================
// 十、并行：一段历史一件，`E198_THREADS` 个线程从共享的件号取活（登记第十四节）；结果按历史名的次序合并，与线程数无关。
//    G1 那一段最长（段内一次次挂载攒行，只能串着做），排在最前取。
// ============================================================================

/// 一处自己的线程变量从进程环境取：设了照它，没设取 `SINGLEFS_THREAD_CAP`，再没设取登记的每台默认重活线程数
/// （判法在 `singlefs_verification::heavy_threads::own_variable_or_default_heavy_threads`；设了而不是正整数就 panic）。
fn own_variable_or_default_heavy_threads_from_the_process_environment(
    own_variable: &str,
) -> std::num::NonZeroUsize {
    own_variable_or_default_heavy_threads(
        own_variable,
        std::env::var(own_variable).ok().as_deref(),
        std::env::var(singlefs_verification::heavy_threads::THREAD_CAP_ENVIRONMENT_VARIABLE)
            .ok()
            .as_deref(),
    )
}

/// 工作线程数：`E198_THREADS`（变量名写在这一条语句里：门禁 code-tooling 的 heavy-thread-defaults 格按读线程变量的那一条语句认它有没有回退到登记值）。
fn worker_threads() -> usize {
    own_variable_or_default_heavy_threads_from_the_process_environment("E198_THREADS").get()
}

fn run_the_histories(
    specifications: &[HistorySpecification],
    threads: usize,
) -> Vec<HistoryResult> {
    let mut order: Vec<usize> = (0..specifications.len()).collect();
    order.sort_by_key(|index| !specifications[*index].shape_accumulates_rows());
    let next_job = AtomicUsize::new(0);
    let results: Mutex<Vec<(usize, HistoryResult)>> = Mutex::new(Vec::new());
    let started = std::time::Instant::now();
    std::thread::scope(|scope| {
        for _ in 0..threads.max(1).min(specifications.len().max(1)) {
            scope.spawn(|| loop {
                let position = next_job.fetch_add(1, Ordering::SeqCst);
                let Some(&index) = order.get(position) else {
                    return;
                };
                let result = run_the_history(&specifications[index]);
                eprintln!(
                    "  进度：{} 判完 {} 张，已用 {:.0} 秒",
                    result.name,
                    result.images.len(),
                    started.elapsed().as_secs_f64()
                );
                results
                    .lock()
                    .expect("结果表的锁没被毒")
                    .push((index, result));
            });
        }
    });
    let mut results = results.into_inner().expect("结果表的锁没被毒");
    results.sort_by_key(|(index, _)| *index);
    results.into_iter().map(|(_, result)| result).collect()
}

impl HistorySpecification {
    fn shape_accumulates_rows(&self) -> bool {
        matches!(
            self.shape,
            HistoryShape::Sweep {
                accumulate_rows_by_mounts: true,
                ..
            }
        )
    }
}

// ============================================================================
// 十一、本地常量回比（停机条款 S4、锚点 B7、B8）与不建历史的锚点（B1–B6 的闭式张数）。
// ============================================================================

#[derive(Clone, Debug, PartialEq, Eq)]
struct ConstantCheck {
    name: &'static str,
    local: u64,
    implementation: u64,
}

impl ConstantCheck {
    fn holds(&self) -> bool {
        self.local == self.implementation
    }
}

/// 出生 txg 的回比用的显眼值：写进一条真根的实例表指针，`to_slot` 之后按本地偏移读回来要等于它。
const DISTINCTIVE_POINTER_BIRTH_TXG: u64 = 0x0102_0304_0506_0708;

/// 偏移 170 + 42 的回比：mkfs、挂载、写第一个文件得到一条真根，把它实例表指针的出生 txg 换成显眼值，`RootRecord::to_slot` 编码，
/// 按本地偏移读回的 8 字节要等于显眼值。建不出那条根时交回 `None`（照 S4 停机，不当 0）。
fn pointer_birth_offset_read_back() -> Option<u64> {
    let specification = HistorySpecification {
        group: Group::NewestRowBad,
        point: SamplingPoint::Main,
        slots_per_region: 4,
        journal_ring_bytes: SHORT_JOURNAL_RING_BYTES,
        shape: HistoryShape::SkippedRow { last: 3 },
    };
    let mut builder = HistoryBuilder::new(&specification);
    let session = build_the_first_instance(
        &mut builder,
        MountConfiguration::Product,
        3,
        UserWrite::Overwrite,
    )
    .ok()?;
    let mut root = *session.current.root();
    root.instance_table.head.birth_txg = CheckpointTxg(DISTINCTIVE_POINTER_BIRTH_TXG);
    pointer_birth_txg_in_root_slot_bytes(
        &root.to_slot(usize::try_from(FIXED_STRUCTURE_SLOT_SPACING).expect("4096 装得进 usize")),
    )
}

/// 第五节本地常量表逐个对 `crates/` 那一处（S4）；B8 修订后比的是产品默认阈值。
fn constant_checks() -> Vec<ConstantCheck> {
    vec![
        ConstantCheck {
            name: "root_record_bytes_212_to_220_hold_the_instance_table_pointer_birth_txg",
            local: DISTINCTIVE_POINTER_BIRTH_TXG,
            implementation: pointer_birth_offset_read_back().unwrap_or(0),
        },
        ConstantCheck {
            name: "root_ring_regions",
            local: ROOT_RING_REGIONS,
            implementation: singlefs_format::ROOT_RING_REGIONS,
        },
        ConstantCheck {
            name: "sweep_threshold_product_default",
            local: SWEEP_THRESHOLD_PRODUCT_DEFAULT,
            implementation: InstanceTableSweepConfiguration::PRODUCT
                .row_count_threshold
                .0,
        },
        ConstantCheck {
            name: "sweep_threshold_any_row",
            local: SWEEP_THRESHOLD_ANY_ROW,
            implementation: InstanceTableRowCountThreshold::ANY_ROW.0,
        },
        ConstantCheck {
            name: "instance_table_page_records",
            local: INSTANCE_TABLE_PAGE_RECORDS,
            implementation: singlefs_format::INSTANCE_TABLE_PAGE_RECORDS,
        },
        ConstantCheck {
            name: "genesis_checkpoint_txg",
            local: GENESIS_TXG,
            implementation: singlefs_core::make_filesystem::GENESIS_CHECKPOINT_TXG.0,
        },
    ]
}

/// 第七节「独立算出」那一类里的张数锚点（B1–B6）：(锚名, 组, 取样点, 登记写的尝试张数)。
const ATTEMPTED_IMAGE_ANCHORS: [(&str, Group, SamplingPoint, u64); 10] = [
    ("B1", Group::ShortRing, SamplingPoint::Main, 432),
    ("B2", Group::SkippedRow, SamplingPoint::Main, 45),
    ("B3", Group::NewestRowBad, SamplingPoint::Main, 6),
    ("B4", Group::RollbackSlotBad, SamplingPoint::Main, 285),
    ("B5", Group::SweepCrash, SamplingPoint::Main, 432),
    (
        "B6",
        Group::RollbackSlotBad,
        SamplingPoint::NewestRowAndRollbackWiderRing,
        114,
    ),
    ("B6", Group::ShortRing, SamplingPoint::LongJournalRing, 117),
    (
        "B6",
        Group::SkippedRow,
        SamplingPoint::SkippedRowWiderRing,
        45,
    ),
    (
        "B6",
        Group::NewestRowBad,
        SamplingPoint::NewestRowAndRollbackWiderRing,
        6,
    ),
    (
        "B6",
        Group::SweepCrash,
        SamplingPoint::SweepThresholdProductDefault,
        13,
    ),
];

/// 第七节「出自被测条款本身」的张数锚点（A1–A4，不符走 F5，不作废）。
const CLAUSE_IMAGE_ANCHORS: [(&str, Group, u64); 4] = [
    ("A1", Group::ShortRing, 432),
    ("A2", Group::SkippedRow, 45),
    ("A3", Group::NewestRowBad, 6),
    ("A4", Group::RollbackSlotBad, 285),
];

fn attempted_images_of(
    specifications: &[HistorySpecification],
    group: Group,
    point: SamplingPoint,
) -> Option<u64> {
    specifications
        .iter()
        .filter(|specification| specification.group == group && specification.point == point)
        .map(attempted_images)
        .sum()
}

// ============================================================================
// 十二、合计与产物行（登记第六节「判定写法」：每组每个量各报一行；第八节轨迹三样；第十节、第十一节逐条结论）。
// ============================================================================

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ArmError {
    TodayE1,
    TodayE2,
    WithPointerBirthE1,
    WithPointerBirthE2,
}

const ARM_ERRORS: [ArmError; 4] = [
    ArmError::TodayE1,
    ArmError::TodayE2,
    ArmError::WithPointerBirthE1,
    ArmError::WithPointerBirthE2,
];

impl ArmError {
    fn arm(self) -> &'static str {
        match self {
            ArmError::TodayE1 | ArmError::TodayE2 => "today",
            ArmError::WithPointerBirthE1 | ArmError::WithPointerBirthE2 => "with_pointer_birth",
        }
    }

    fn error(self) -> &'static str {
        match self {
            ArmError::TodayE1 | ArmError::WithPointerBirthE1 => "E1",
            ArmError::TodayE2 | ArmError::WithPointerBirthE2 => "E2",
        }
    }

    fn of(self, counts: &ArmCounts) -> u64 {
        match self {
            ArmError::TodayE1 => counts.today_e1,
            ArmError::TodayE2 => counts.today_e2,
            ArmError::WithPointerBirthE1 => counts.with_pointer_birth_e1,
            ArmError::WithPointerBirthE2 => counts.with_pointer_birth_e2,
        }
    }
}

/// 一组（组 × 取样点）的合计。`only_swept` 时只合计清扫发成了的那几段（Q6、Q9、Q10、Q11 的读法，登记 F6）。
#[derive(Clone, Debug, Default)]
struct GroupTotals {
    histories: u64,
    produced_images: u64,
    stopped_histories: u64,
    judged_images: u64,
    unreadable_images: u64,
    hole_pairs: u64,
    truth_counts: BTreeMap<Truth, u64>,
    pointer_unreadable_pairs: u64,
    pairs: ArmCounts,
    images_with_a_positive_count: ArmCounts,
    first_image_with_a_positive_count: BTreeMap<&'static str, String>,
    release_generation_verdicts: BTreeMap<String, u64>,
    reference_interval_verdicts: BTreeMap<String, u64>,
    swept_histories: u64,
    sweep_txg_matches_the_prediction: u64,
    first_version_violations: u64,
    adjacent_version_violations: u64,
    birth_after_the_txg_violations: u64,
    instance_one_first_version_mismatches: u64,
    sweep_pointer_changes: u64,
}

fn history_swept(result: &HistoryResult) -> bool {
    result
        .sweep
        .as_ref()
        .is_some_and(|sweep| sweep.sweep_version.is_some())
}

fn totals_of(results: &[&HistoryResult], only_swept: bool) -> GroupTotals {
    let mut totals = GroupTotals::default();
    for result in results {
        if only_swept && !history_swept(result) {
            continue;
        }
        totals.histories += 1;
        totals.stopped_histories += u64::from(result.stopped.is_some());
        totals.first_version_violations +=
            u64::try_from(result.premise.first_version_violations.len()).expect("条数装得进 u64");
        totals.adjacent_version_violations +=
            u64::try_from(result.premise.adjacent_version_violations.len())
                .expect("条数装得进 u64");
        totals.birth_after_the_txg_violations +=
            u64::try_from(result.premise.birth_after_the_txg_violations.len())
                .expect("条数装得进 u64");
        totals.instance_one_first_version_mismatches +=
            result.premise.instance_one_first_version_mismatches;
        totals.sweep_pointer_changes += result.premise.sweep_pointer_changes;
        if let Some(sweep) = &result.sweep {
            if let Some(version) = sweep.sweep_version {
                totals.swept_histories += 1;
                totals.sweep_txg_matches_the_prediction +=
                    u64::from(version.txg == sweep.predicted_sweep_txg);
            }
        }
        for image in &result.images {
            totals.produced_images += 1;
            if image.reading.is_err() {
                totals.unreadable_images += 1;
                continue;
            }
            totals.judged_images += 1;
            totals.hole_pairs += u64::try_from(image.truths.len()).expect("对数装得进 u64");
            for truth in &image.truths {
                *totals.truth_counts.entry(*truth).or_insert(0) += 1;
            }
            totals.pointer_unreadable_pairs += image.pointer_unreadable_pairs;
            totals.pairs.add(image.counts);
            for arm_error in ARM_ERRORS {
                if arm_error.of(&image.counts) > 0 {
                    let mut one = ArmCounts::default();
                    match arm_error {
                        ArmError::TodayE1 => one.today_e1 = 1,
                        ArmError::TodayE2 => one.today_e2 = 1,
                        ArmError::WithPointerBirthE1 => one.with_pointer_birth_e1 = 1,
                        ArmError::WithPointerBirthE2 => one.with_pointer_birth_e2 = 1,
                    }
                    totals.images_with_a_positive_count.add(one);
                    let key = match arm_error {
                        ArmError::TodayE1 => "today_E1",
                        ArmError::TodayE2 => "today_E2",
                        ArmError::WithPointerBirthE1 => "with_pointer_birth_E1",
                        ArmError::WithPointerBirthE2 => "with_pointer_birth_E2",
                    };
                    totals
                        .first_image_with_a_positive_count
                        .entry(key)
                        .or_insert_with(|| format!("{}/{}", result.name, image.label));
                }
            }
            if let Some((release_generation_verdict, reference_interval_verdict)) =
                &image.checker_verdicts
            {
                *totals
                    .release_generation_verdicts
                    .entry(release_generation_verdict.clone())
                    .or_insert(0) += 1;
                *totals
                    .reference_interval_verdicts
                    .entry(reference_interval_verdict.clone())
                    .or_insert(0) += 1;
            }
        }
    }
    totals
}

fn count_text(counts: &BTreeMap<String, u64>) -> String {
    if counts.is_empty() {
        return "none".to_string();
    }
    counts
        .iter()
        .map(|(name, count)| format!("{name}:{count}"))
        .collect::<Vec<_>>()
        .join(",")
}

fn truth_count(counts: &BTreeMap<Truth, u64>, truth: Truth) -> u64 {
    counts.get(&truth).copied().unwrap_or(0)
}

fn keys_text(keys: impl Iterator<Item = VersionKey>) -> String {
    let texts: Vec<String> = keys.map(VersionKey::text).collect();
    if texts.is_empty() {
        "none".to_string()
    } else {
        texts.join(",")
    }
}

fn pointer_birth_text(pointer_birth: PointerBirth) -> String {
    match pointer_birth {
        PointerBirth::Readable(txg) => txg.to_string(),
        PointerBirth::AllZeroOrUnreadable => "unreadable".to_string(),
    }
}

/// 一张镜像一行：最新根、F 生效值、候选集、由记录施加出来的版本、每个洞对（R⁻ 那一组 > R：真值：两条臂：R 的出生 txg）与四个计数。
fn image_line(result: &HistoryResult, image: &ImageResult) -> String {
    let prefix = format!(
        "name=image group={} point={} history={} image={} step={}",
        result.specification.group.name(),
        result.specification.point.name(),
        result.name,
        image.label,
        image.step
    );
    let checker = image
        .checker_verdicts
        .as_ref()
        .map_or("release_generation_verdict=not_run reference_interval_verdict=not_run".to_string(), |(release_generation_verdict, reference_interval_verdict)| {
            format!("release_generation_verdict={release_generation_verdict} reference_interval_verdict={reference_interval_verdict}")
        });
    match &image.reading {
        Err(reason) => format!("{prefix} unreadable={} {checker}", reason.replace(' ', "_")),
        Ok(reading) => {
            let pairs: Vec<String> = reading
                .hole_pairs
                .iter()
                .zip(&image.truths)
                .map(|(pair, truth)| {
                    format!(
                        "{}>{}:{}:today{}:with_pointer_birth{}:birth{}",
                        keys_text(pair.previous_group.iter().copied()),
                        pair.version.key.text(),
                        truth.name(),
                        u8::from(pair.bridged_today),
                        u8::from(pair.bridged_with_the_pointer_birth),
                        pointer_birth_text(pair.version.pointer_birth)
                    )
                })
                .collect();
            format!(
                "{prefix} newest={} floor={} candidates={} applied_only_by_records={} hole_pairs={} pairs={} today_e1={} today_e2={} with_pointer_birth_e1={} with_pointer_birth_e2={} pointer_unreadable_pairs={} corrupted_skipped={} {checker}",
                reading.newest.text(),
                reading.effective_floor,
                keys_text(reading.candidates.iter().map(|version| version.key)),
                keys_text(reading.applied_only_by_records.iter().map(|version| version.key)),
                reading.hole_pairs.len(),
                if pairs.is_empty() { "none".to_string() } else { pairs.join(";") },
                image.counts.today_e1,
                image.counts.today_e2,
                image.counts.with_pointer_birth_e1,
                image.counts.with_pointer_birth_e2,
                image.pointer_unreadable_pairs,
                image.corrupted_skipped.len()
            )
        }
    }
}

/// 一段历史一行：尝试与造出的张数、写者停在哪、清扫的读数、前提 P1–P3 的计数与逐条。
fn history_lines(result: &HistoryResult) -> Vec<String> {
    let attempted = attempted_images(&result.specification)
        .map_or("unknown_before_the_run".to_string(), |count| {
            count.to_string()
        });
    let sweep = result.sweep.as_ref().map_or("not_a_sweep_history".to_string(), |sweep| {
        format!(
            "sweep_instance={} sweep_outcome={} sweep_txg={} predicted_sweep_txg={} last_txg_before_the_sweep_instance={} instances_mounted_before_the_sweep={}",
            sweep.sweep_instance,
            sweep.outcome,
            sweep.sweep_version.map_or("none".to_string(), |version| version.txg.to_string()),
            sweep.predicted_sweep_txg,
            sweep.last_txg_before_the_sweep_instance,
            sweep.instances_mounted_before_the_sweep
        )
    });
    let premise = &result.premise;
    let mut lines = vec![
        format!(
            "name=history group={} point={} history={} attempted_images={attempted} produced_images={} stopped={} versions={} {sweep}",
            result.specification.group.name(),
            result.specification.point.name(),
            result.name,
            result.images.len(),
            result.stopped.as_deref().map_or("none".to_string(), |reason| reason.replace(' ', "_")),
            result.ledger.versions.len()
        ),
        format!(
            "name=premise history={} first_version_violations={} adjacent_version_violations={} birth_after_the_txg_violations={} instance_one_first_version_mismatches={} sweep_pointer_changes={}",
            result.name,
            premise.first_version_violations.len(),
            premise.adjacent_version_violations.len(),
            premise.birth_after_the_txg_violations.len(),
            premise.instance_one_first_version_mismatches,
            premise.sweep_pointer_changes
        ),
    ];
    for (clause, details) in [
        ("P1", &premise.first_version_violations),
        ("P2", &premise.adjacent_version_violations),
        ("P3", &premise.birth_after_the_txg_violations),
    ] {
        for detail in details {
            lines.push(format!(
                "name=premise_violation history={} clause={clause} detail={}",
                result.name,
                detail.replace(' ', "_")
            ));
        }
    }
    lines
}

/// 每段历史的轨迹三样（登记第八节）：按步，每条臂每种错各报峰值、为正的步数、期末值。
fn trajectory_of(result: &HistoryResult, arm_error: ArmError) -> (u64, u64, u64) {
    let values: Vec<u64> = result
        .images
        .iter()
        .filter(|image| image.reading.is_ok())
        .map(|image| arm_error.of(&image.counts))
        .collect();
    let peak = values.iter().copied().max().unwrap_or(0);
    let positive_steps =
        u64::try_from(values.iter().filter(|value| **value > 0).count()).expect("步数装得进 u64");
    let final_value = values.last().copied().unwrap_or(0);
    (peak, positive_steps, final_value)
}

fn results_of(
    results: &[HistoryResult],
    group: Group,
    point: SamplingPoint,
) -> Vec<&HistoryResult> {
    results
        .iter()
        .filter(|result| result.specification.group == group && result.specification.point == point)
        .collect()
}

/// 一组一行合计、四条轨迹行；sw 三组另报只合计清扫发成了的那几段的一行（`scope=swept_histories`）。
fn group_lines(results: &[HistoryResult], group: Group, point: SamplingPoint) -> Vec<String> {
    let members = results_of(results, group, point);
    if members.is_empty() {
        return Vec::new();
    }
    let mut lines = Vec::new();
    let scopes: Vec<(&str, bool)> = if matches!(
        group,
        Group::SweepCrash | Group::SweepRead | Group::SweepNone
    ) {
        vec![("all_histories", false), ("swept_histories", true)]
    } else {
        vec![("all_histories", false)]
    };
    for (scope, only_swept) in scopes {
        let totals = totals_of(&members, only_swept);
        lines.push(format!(
            "name=group group={} point={} scope={scope} histories={} stopped_histories={} produced_images={} judged_images={} unreadable_images={} hole_pairs={} truth_directly_bridged={} truth_existed_versions_in_between={} truth_previous_not_on_the_chain={} truth_version_missing_from_the_ledger={} pointer_unreadable_pairs={} today_e1_pairs={} today_e1_images={} today_e2_pairs={} today_e2_images={} with_pointer_birth_e1_pairs={} with_pointer_birth_e1_images={} with_pointer_birth_e2_pairs={} with_pointer_birth_e2_images={} swept_histories={} sweep_txg_matches_the_prediction={} first_version_violations={} adjacent_version_violations={} birth_after_the_txg_violations={} instance_one_first_version_mismatches={} sweep_pointer_changes={} release_generation_verdict={} reference_interval_verdict={}",
            group.name(),
            point.name(),
            totals.histories,
            totals.stopped_histories,
            totals.produced_images,
            totals.judged_images,
            totals.unreadable_images,
            totals.hole_pairs,
            truth_count(&totals.truth_counts, Truth::DirectlyBridged),
            truth_count(&totals.truth_counts, Truth::ExistedVersionsInBetween),
            truth_count(&totals.truth_counts, Truth::PreviousNotOnTheChain),
            truth_count(&totals.truth_counts, Truth::VersionMissingFromTheLedger),
            totals.pointer_unreadable_pairs,
            totals.pairs.today_e1,
            totals.images_with_a_positive_count.today_e1,
            totals.pairs.today_e2,
            totals.images_with_a_positive_count.today_e2,
            totals.pairs.with_pointer_birth_e1,
            totals.images_with_a_positive_count.with_pointer_birth_e1,
            totals.pairs.with_pointer_birth_e2,
            totals.images_with_a_positive_count.with_pointer_birth_e2,
            totals.swept_histories,
            totals.sweep_txg_matches_the_prediction,
            totals.first_version_violations,
            totals.adjacent_version_violations,
            totals.birth_after_the_txg_violations,
            totals.instance_one_first_version_mismatches,
            totals.sweep_pointer_changes,
            count_text(&totals.release_generation_verdicts),
            count_text(&totals.reference_interval_verdicts)
        ));
        for (key, first) in &totals.first_image_with_a_positive_count {
            lines.push(format!(
                "name=first_positive_image group={} point={} scope={scope} count={key} image={first}",
                group.name(),
                point.name()
            ));
        }
        for arm_error in ARM_ERRORS {
            let mut peak_history = ("none".to_string(), 0);
            let mut positive_steps_total = 0;
            let mut histories_with_a_positive_final_value = 0;
            for result in members
                .iter()
                .filter(|result| !only_swept || history_swept(result))
            {
                let (peak, positive_steps, final_value) = trajectory_of(result, arm_error);
                if peak > peak_history.1 {
                    peak_history = (result.name.clone(), peak);
                }
                positive_steps_total += positive_steps;
                histories_with_a_positive_final_value += u64::from(final_value > 0);
            }
            lines.push(format!(
                "name=trajectory group={} point={} scope={scope} arm={} error={} peak={} peak_history={} positive_steps_total={positive_steps_total} histories_with_a_positive_final_value={histories_with_a_positive_final_value}",
                group.name(),
                point.name(),
                arm_error.arm(),
                arm_error.error(),
                peak_history.1,
                peak_history.0
            ));
        }
    }
    lines
}

/// 每段历史每条臂每种错的轨迹三样各一行。
fn history_trajectory_lines(result: &HistoryResult) -> Vec<String> {
    ARM_ERRORS
        .iter()
        .map(|arm_error| {
            let (peak, positive_steps, final_value) = trajectory_of(result, *arm_error);
            format!(
                "name=history_trajectory history={} arm={} error={} peak={peak} positive_steps={positive_steps} final={final_value}",
                result.name,
                arm_error.arm(),
                arm_error.error()
            )
        })
        .collect()
}

const GROUP_ORDER: [Group; 7] = [
    Group::NewestRowBad,
    Group::RollbackSlotBad,
    Group::ShortRing,
    Group::SkippedRow,
    Group::SweepCrash,
    Group::SweepRead,
    Group::SweepNone,
];

const POINT_ORDER: [SamplingPoint; 5] = [
    SamplingPoint::Main,
    SamplingPoint::SweepThresholdProductDefault,
    SamplingPoint::LongJournalRing,
    SamplingPoint::SkippedRowWiderRing,
    SamplingPoint::NewestRowAndRollbackWiderRing,
];

/// 停机条款（登记第十一节「停机」）：装置与 `crates/` 对不上。触发的那一段不出结论，交主 agent。
struct StopClauses {
    lines: Vec<String>,
    triggered: Vec<&'static str>,
}

fn first_detail(details: impl Iterator<Item = String>) -> String {
    details
        .into_iter()
        .next()
        .map_or("none".to_string(), |detail| detail.replace(' ', "_"))
}

/// S5 的一格在全部镜像上的合计。
fn sum_over_the_images(
    results: &[HistoryResult],
    pick: fn(&RecordsAppliedComparison) -> usize,
) -> usize {
    results
        .iter()
        .flat_map(|result| result.images.iter())
        .map(|image| pick(&image.records_applied))
        .sum()
}

fn stop_clauses(results: &[HistoryResult], constants: &[ConstantCheck]) -> StopClauses {
    let mut lines = Vec::new();
    let mut triggered = Vec::new();
    let images = || {
        results
            .iter()
            .flat_map(|result| result.images.iter().map(move |image| (result, image)))
    };
    let ring_slot_mismatch_images = images()
        .filter(|(_, image)| !image.ring_slot_mismatches.is_empty())
        .count();
    lines.push(format!(
        "name=stop clause=S1 checked_images={} triggered_images={ring_slot_mismatch_images} first={}",
        images().count(),
        first_detail(images().flat_map(|(result, image)| {
            image
                .ring_slot_mismatches
                .iter()
                .map(move |detail| format!("{}/{}:{detail}", result.name, image.label))
        }))
    ));
    if ring_slot_mismatch_images > 0 {
        triggered.push("S1");
    }
    let compared: u64 = results
        .iter()
        .map(|result| result.ledger.root_slot_writes_compared)
        .sum();
    let without: u64 = results
        .iter()
        .map(|result| result.ledger.root_slot_writes_without_a_returned_root)
        .sum();
    let root_slot_write_mismatch_count: usize = results
        .iter()
        .map(|result| result.ledger.root_slot_write_mismatches.len())
        .sum();
    lines.push(format!(
        "name=stop clause=S2 compared_root_slot_writes={compared} root_slot_writes_without_a_returned_root={without} mismatches={root_slot_write_mismatch_count} first={}",
        first_detail(results.iter().flat_map(|result| result.ledger.root_slot_write_mismatches.iter().cloned()))
    ));
    if root_slot_write_mismatch_count > 0 || compared == 0 {
        triggered.push("S2");
    }
    let row_mismatch_images = images()
        .filter(|(_, image)| image.row_mismatch.is_some())
        .count();
    lines.push(format!(
        "name=stop clause=S3 checked_images={} triggered_images={row_mismatch_images} first={}",
        images().filter(|(_, image)| image.reading.is_ok()).count(),
        first_detail(images().filter_map(|(result, image)| {
            image
                .row_mismatch
                .as_ref()
                .map(|detail| format!("{}/{}:{detail}", result.name, image.label))
        }))
    ));
    if row_mismatch_images > 0 {
        triggered.push("S3");
    }
    let constant_failures = constants.iter().filter(|check| !check.holds()).count();
    let formula_mismatches: usize = results
        .iter()
        .map(|result| result.ledger.ring_formula_mismatches.len())
        .sum();
    lines.push(format!(
        "name=stop clause=S4 constants_checked={} constants_unequal={constant_failures} ring_formula_mismatches={formula_mismatches} first={}",
        constants.len(),
        first_detail(results.iter().flat_map(|result| result.ledger.ring_formula_mismatches.iter().cloned()))
    ));
    if constant_failures > 0 || formula_mismatches > 0 {
        triggered.push("S4");
    }
    let records_applied_mismatch_images = images()
        .filter(|(_, image)| image.records_applied.mismatch_count() > 0)
        .count();
    lines.push(format!(
        "name=stop clause=S5 checked_images={} triggered_images={records_applied_mismatch_images} apparatus_extra_with_a_root_that_landed_once={} apparatus_extra_otherwise={} apparatus_missing={} first={}",
        images().filter(|(_, image)| image.reading.is_ok()).count(),
        sum_over_the_images(results, |comparison| comparison.apparatus_extra_with_a_root_that_landed_once.len()),
        sum_over_the_images(results, |comparison| comparison.apparatus_extra_otherwise.len()),
        sum_over_the_images(results, |comparison| comparison.apparatus_missing.len()),
        first_detail(images().filter(|(_, image)| image.records_applied.mismatch_count() > 0).map(|(result, image)| {
            format!(
                "{}/{}:extra_landed_once={}:extra_otherwise={}:missing={}",
                result.name,
                image.label,
                keys_text(image.records_applied.apparatus_extra_with_a_root_that_landed_once.iter().copied()),
                keys_text(image.records_applied.apparatus_extra_otherwise.iter().copied()),
                keys_text(image.records_applied.apparatus_missing.iter().copied())
            )
        }))
    ));
    if records_applied_mismatch_images > 0 {
        triggered.push("S5");
    }
    let consistency: usize = results
        .iter()
        .map(|result| result.ledger.consistency_problems.len())
        .sum();
    lines.push(format!(
        "name=stop clause=ledger_consistency problems={consistency} first={}",
        first_detail(
            results
                .iter()
                .flat_map(|result| result.ledger.consistency_problems.iter().cloned())
        )
    ));
    if consistency > 0 {
        triggered.push("ledger_consistency");
    }
    StopClauses { lines, triggered }
}

// ============================================================================
// 十三、判定：每个量一行（登记第六节），阳性对照、锚点、几何敏感性、失败与作废条款逐条，两行岔路各一行（不合成一个真假）。
// ============================================================================

/// 一个量在一组上的读数：对数、张数、洞对总数、判了的张数。
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
struct QuestionReading {
    pairs: u64,
    images: u64,
    hole_pairs: u64,
    judged_images: u64,
    unreadable_images: u64,
}

fn question_reading(
    results: &[HistoryResult],
    group: Group,
    point: SamplingPoint,
    only_swept: bool,
    arm_error: ArmError,
) -> QuestionReading {
    let totals = totals_of(&results_of(results, group, point), only_swept);
    QuestionReading {
        pairs: arm_error.of(&totals.pairs),
        images: arm_error.of(&totals.images_with_a_positive_count),
        hole_pairs: totals.hole_pairs,
        judged_images: totals.judged_images,
        unreadable_images: totals.unreadable_images,
    }
}

/// 错分类量的判定词：洞对一对都没有记「没有对象」（不记分对，登记 Q18）；0 记分对；≥ 1 记乙。
fn misclassification_judgement(reading: QuestionReading) -> &'static str {
    if reading.judged_images == 0 {
        "not_run"
    } else if reading.hole_pairs == 0 {
        "no_object"
    } else if reading.pairs == 0 {
        "classified_right"
    } else {
        "branch_two"
    }
}

fn question_line(
    question: &str,
    group: Group,
    point: SamplingPoint,
    scope: &str,
    arm_error: ArmError,
    reading: QuestionReading,
) -> String {
    format!(
        "name=verdict question={question} group={} point={} scope={scope} arm={} error={} value_pairs={} value_images={} hole_pairs={} judged_images={} unreadable_images={} judgement={}",
        group.name(),
        point.name(),
        arm_error.arm(),
        arm_error.error(),
        reading.pairs,
        reading.images,
        reading.hole_pairs,
        reading.judged_images,
        reading.unreadable_images,
        misclassification_judgement(reading)
    )
}

/// 几何敏感性一行（登记第八节、`.claude/rules/mutation-sampling.md` 第六类）：判定同向记两点、反向记不稳定；判别力自证把门槛挪到两值之间。
fn sensitivity_line(
    point: SamplingPoint,
    question: &str,
    main_value: u64,
    sensitive_value: u64,
) -> String {
    let record = if (main_value == 0) == (sensitive_value == 0) {
        "two_points"
    } else {
        "unstable"
    };
    let discriminating = if main_value != sensitive_value {
        "proven_threshold_between_the_two_values_splits_them"
    } else if main_value == 0 {
        "both_zero_threshold_cannot_move_between"
    } else {
        "equal_values_threshold_at_one_half_judges_both_the_same_not_proven"
    };
    format!(
        "name=sensitivity point={} question={question} main={main_value} sensitive={sensitive_value} record={record} discriminating={discriminating}",
        point.name()
    )
}

fn anchor_line(anchor: &str, kind: &str, subject: &str, expected: &str, observed: &str) -> String {
    format!(
        "name=anchor anchor={anchor} kind={kind} subject={subject} expected={expected} observed={observed} holds={}",
        expected == observed
    )
}

/// C1、C2 这类钉一张的洞对表：交回 (洞对数, 第一对的 R⁻.txg, R.txg, R 的出生 txg, 真值, 两条臂认不认, 两条臂的 E1、E2)。
fn pinned_image_text(result: Option<&HistoryResult>, step: u64) -> String {
    let Some(image) =
        result.and_then(|result| result.images.iter().find(|image| image.step == step))
    else {
        return "image_missing".to_string();
    };
    let Ok(reading) = &image.reading else {
        return "image_unreadable".to_string();
    };
    let pairs: Vec<String> = reading
        .hole_pairs
        .iter()
        .zip(&image.truths)
        .map(|(pair, truth)| {
            format!(
                "({},{}):birth{}:{}:today{}:with_pointer_birth{}",
                pair.previous_txg,
                pair.version.key.txg,
                pointer_birth_text(pair.version.pointer_birth),
                truth.name(),
                u8::from(pair.bridged_today),
                u8::from(pair.bridged_with_the_pointer_birth)
            )
        })
        .collect();
    format!(
        "pairs{}:{}:today_e1_{}:today_e2_{}:with_pointer_birth_e1_{}:with_pointer_birth_e2_{}",
        reading.hole_pairs.len(),
        pairs.join(";"),
        image.counts.today_e1,
        image.counts.today_e2,
        image.counts.with_pointer_birth_e1,
        image.counts.with_pointer_birth_e2
    )
}

/// 登记第七节 C1：nrb（last 10、k 3、bad 1）那一张。期望的写法与 `pinned_image_text` 同形，数抄自登记第七节与第十三节 `anchors.py` 的输出。
const NEWEST_ROW_BAD_PIN_EXPECTED: &str = "pairs1:(10,12):birth11:existed_versions_in_between:today1:with_pointer_birth0:today_e1_1:today_e2_0:with_pointer_birth_e1_0:with_pointer_birth_e2_0";
/// 登记第七节 C2：skr（last 6、k 0）那一张。
const SKIPPED_ROW_PIN_EXPECTED: &str = "pairs1:(6,8):birth8:directly_bridged:today1:with_pointer_birth1:today_e1_0:today_e2_0:with_pointer_birth_e1_0:with_pointer_birth_e2_0";

fn find_history<'results>(
    results: &'results [HistoryResult],
    specification: &HistorySpecification,
) -> Option<&'results HistoryResult> {
    results
        .iter()
        .find(|result| &result.specification == specification)
}

fn newest_row_bad_pin_specification() -> HistorySpecification {
    HistorySpecification {
        group: Group::NewestRowBad,
        point: SamplingPoint::Main,
        slots_per_region: 4,
        journal_ring_bytes: SHORT_JOURNAL_RING_BYTES,
        shape: HistoryShape::NewestRowBad {
            last: 10,
            steps_after_the_mount: 3,
            bad_slots: 1,
            user_write: UserWrite::Overwrite,
        },
    }
}

fn skipped_row_pin_specification() -> HistorySpecification {
    HistorySpecification {
        group: Group::SkippedRow,
        point: SamplingPoint::Main,
        slots_per_region: 4,
        journal_ring_bytes: SHORT_JOURNAL_RING_BYTES,
        shape: HistoryShape::SkippedRow { last: 6 },
    }
}

fn joined_or_none(items: &[String]) -> String {
    if items.is_empty() {
        "none".to_string()
    } else {
        items.join(",")
    }
}

fn yes_or_no(value: bool) -> &'static str {
    if value {
        "yes"
    } else {
        "no"
    }
}

fn clause_line(clause: &str, triggered: bool, observation: &str) -> String {
    format!(
        "name=clause clause={clause} triggered={} observation={}",
        yes_or_no(triggered),
        observation.replace(' ', "_")
    )
}

/// 今天的池级 checker 在这条实验里取的两格。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum CheckedInvariant {
    ReleaseGenerationInterval,
    ReferenceIntervalPrecision,
}

/// 一组镜像上那一格判违例的张数与各判定的张数。
fn checker_violations(
    results: &[HistoryResult],
    group: Group,
    point: SamplingPoint,
    invariant: CheckedInvariant,
) -> (u64, String) {
    let totals = totals_of(&results_of(results, group, point), false);
    let counts = match invariant {
        CheckedInvariant::ReleaseGenerationInterval => &totals.release_generation_verdicts,
        CheckedInvariant::ReferenceIntervalPrecision => &totals.reference_interval_verdicts,
    };
    (
        counts.get("violated").copied().unwrap_or(0),
        count_text(counts),
    )
}

/// 张数锚点：(A 或 B 的行, 张数对不上的锚名)。
fn image_count_anchor_lines(
    specifications: &[HistorySpecification],
    results: &[HistoryResult],
    segment_groups: &[Group],
) -> (Vec<String>, Vec<String>, Vec<String>) {
    let mut lines = Vec::new();
    let mut independent_failures = Vec::new();
    let mut clause_failures = Vec::new();
    for (anchor, group, point, expected) in ATTEMPTED_IMAGE_ANCHORS {
        if !segment_groups.contains(&group)
            || !specifications
                .iter()
                .any(|specification| specification.group == group && specification.point == point)
        {
            continue;
        }
        let observed = attempted_images_of(specifications, group, point)
            .map_or("unknown".to_string(), |count| count.to_string());
        let line = anchor_line(
            anchor,
            "independent",
            &format!("attempted_images.{}.{}", group.name(), point.name()),
            &expected.to_string(),
            &observed,
        );
        if observed != expected.to_string() {
            independent_failures.push(format!("{anchor}.{}.{}", group.name(), point.name()));
        }
        lines.push(line);
    }
    for (anchor, group, expected) in CLAUSE_IMAGE_ANCHORS {
        if !segment_groups.contains(&group) {
            continue;
        }
        let produced =
            totals_of(&results_of(results, group, SamplingPoint::Main), false).produced_images;
        if produced != expected {
            clause_failures.push(format!("{anchor}.{}", group.name()));
        }
        lines.push(anchor_line(
            anchor,
            "clause",
            &format!("produced_images.{}.main", group.name()),
            &expected.to_string(),
            &produced.to_string(),
        ));
    }
    (lines, independent_failures, clause_failures)
}

fn premise_question_lines(results: &[HistoryResult], scope: &str) -> Vec<String> {
    let all: Vec<&HistoryResult> = results.iter().collect();
    let totals = totals_of(&all, false);
    let judge = |value: u64| {
        if value == 0 {
            "premise_holds_on_these_groups"
        } else {
            "branch_two"
        }
    };
    vec![
        format!("name=verdict question=Q1 scope={scope} value={} sweep_pointer_changes={} judgement={}", totals.adjacent_version_violations, totals.sweep_pointer_changes, judge(totals.adjacent_version_violations)),
        format!("name=verdict question=Q2 scope={scope} value={} instance_one_reported_separately={} judgement={}", totals.first_version_violations, totals.instance_one_first_version_mismatches, judge(totals.first_version_violations)),
        format!("name=verdict question=Q3 scope={scope} value={} judgement={}", totals.birth_after_the_txg_violations, judge(totals.birth_after_the_txg_violations)),
    ]
}

/// PC-P1：每段历史实例 1 的第一版都报一处不成立（指针出生 txg = 0）。交回 (行, 没中的段数)。
fn instance_one_control_line(results: &[HistoryResult]) -> (String, usize) {
    let missed = results
        .iter()
        .filter(|result| result.premise.instance_one_first_version_mismatches != 1)
        .count();
    (
        format!("name=control control=PC-P1 histories={} histories_not_reporting_exactly_one={missed} hit={}", results.len(), yes_or_no(missed == 0)),
        missed,
    )
}

/// 第一段（补第 2 行）：nrb、rs1 与 G4。
fn judgement_lines_of_segment_one(
    specifications: &[HistorySpecification],
    results: &[HistoryResult],
    stops: &StopClauses,
) -> Vec<String> {
    let mut lines = Vec::new();
    let main = SamplingPoint::Main;
    let wider = SamplingPoint::NewestRowAndRollbackWiderRing;
    let (anchor_lines, mut independent_failures, clause_failures) = image_count_anchor_lines(
        specifications,
        results,
        &[Group::NewestRowBad, Group::RollbackSlotBad],
    );
    lines.extend(anchor_lines);
    let newest_row_bad_pin_observed = pinned_image_text(
        find_history(results, &newest_row_bad_pin_specification()),
        0,
    );
    lines.push(anchor_line(
        "C1",
        "independent",
        "nrb.main.S4.last10.k3.bad1",
        NEWEST_ROW_BAD_PIN_EXPECTED,
        &newest_row_bad_pin_observed,
    ));
    if newest_row_bad_pin_observed != NEWEST_ROW_BAD_PIN_EXPECTED {
        independent_failures.push("C1".to_string());
    }
    let nrb_today = question_reading(results, Group::NewestRowBad, main, false, ArmError::TodayE1);
    let rs1_today = question_reading(
        results,
        Group::RollbackSlotBad,
        main,
        false,
        ArmError::TodayE1,
    );
    let nrb_with = question_reading(
        results,
        Group::NewestRowBad,
        main,
        false,
        ArmError::WithPointerBirthE1,
    );
    let rs1_with = question_reading(
        results,
        Group::RollbackSlotBad,
        main,
        false,
        ArmError::WithPointerBirthE1,
    );
    let (rollback_release_generation_violations, rollback_release_generation_verdicts) =
        checker_violations(
            results,
            Group::RollbackSlotBad,
            main,
            CheckedInvariant::ReleaseGenerationInterval,
        );
    let (newest_row_release_generation_violations, newest_row_release_generation_verdicts) =
        checker_violations(
            results,
            Group::NewestRowBad,
            main,
            CheckedInvariant::ReleaseGenerationInterval,
        );
    let clause_number_mismatches: Vec<&str> = [
        ("A5", nrb_today.images != 6),
        ("A6", rs1_today.images != 285),
        ("A7", rollback_release_generation_violations != 150),
    ]
    .into_iter()
    .filter(|(_, differs)| *differs)
    .map(|(anchor, _)| anchor)
    .collect();
    lines.push(anchor_line(
        "A5",
        "clause",
        "today_e1_images.nrb.main",
        "6",
        &nrb_today.images.to_string(),
    ));
    lines.push(anchor_line(
        "A6",
        "clause",
        "today_e1_images.rs1.main",
        "285",
        &rs1_today.images.to_string(),
    ));
    lines.push(anchor_line(
        "A7",
        "clause",
        "i39_violated_images.rs1.main",
        "150",
        &rollback_release_generation_violations.to_string(),
    ));
    let nrb_produced =
        totals_of(&results_of(results, Group::NewestRowBad, main), false).produced_images;
    lines.push(format!(
        "name=control control=PC-H1A nrb_images={nrb_produced} nrb_images_with_today_e1={} c1_today_e1_is_one={} hit={}",
        nrb_today.images,
        yes_or_no(newest_row_bad_pin_observed.contains(":today_e1_1:")),
        yes_or_no(nrb_today.images >= 1 && newest_row_bad_pin_observed.contains(":today_e1_1:"))
    ));
    let (instance_one_control_text, instance_one_control_misses) =
        instance_one_control_line(results);
    lines.push(instance_one_control_text);
    lines.push(question_line(
        "Q13",
        Group::NewestRowBad,
        main,
        "all_histories",
        ArmError::TodayE1,
        nrb_today,
    ));
    lines.push(question_line(
        "Q13",
        Group::RollbackSlotBad,
        main,
        "all_histories",
        ArmError::TodayE1,
        rs1_today,
    ));
    lines.push(question_line(
        "Q14",
        Group::NewestRowBad,
        main,
        "all_histories",
        ArmError::WithPointerBirthE1,
        nrb_with,
    ));
    lines.push(question_line(
        "Q15",
        Group::RollbackSlotBad,
        main,
        "all_histories",
        ArmError::WithPointerBirthE1,
        rs1_with,
    ));
    let release_generation_judgement = |violated: u64| {
        if violated == 0 {
            "release_generation_half_not_red"
        } else {
            "branch_two"
        }
    };
    lines.push(format!("name=verdict question=Q16 group=rs1 point=main value_images={rollback_release_generation_violations} verdicts={rollback_release_generation_verdicts} judgement={}", release_generation_judgement(rollback_release_generation_violations)));
    lines.push(format!("name=verdict question=Q17 group=nrb point=main value_images={newest_row_release_generation_violations} verdicts={newest_row_release_generation_verdicts} judgement={}", release_generation_judgement(newest_row_release_generation_violations)));
    for group in [Group::NewestRowBad, Group::RollbackSlotBad] {
        let totals = totals_of(&results_of(results, group, main), false);
        lines.push(format!(
            "name=verdict question=Q18 group={} point=main hole_pairs={} directly_bridged={} existed_versions_in_between={} previous_not_on_the_chain={} version_missing_from_the_ledger={} pointer_unreadable_pairs={} judgement=coverage_only",
            group.name(),
            totals.hole_pairs,
            truth_count(&totals.truth_counts, Truth::DirectlyBridged),
            truth_count(&totals.truth_counts, Truth::ExistedVersionsInBetween),
            truth_count(&totals.truth_counts, Truth::PreviousNotOnTheChain),
            truth_count(&totals.truth_counts, Truth::VersionMissingFromTheLedger),
            totals.pointer_unreadable_pairs
        ));
        let (_, reference_interval_verdicts_text) = checker_violations(
            results,
            group,
            main,
            CheckedInvariant::ReferenceIntervalPrecision,
        );
        lines.push(format!("name=verdict question=Q19 group={} point=main reference_interval_verdicts={reference_interval_verdicts_text} judgement=attached_not_judged", group.name()));
    }
    lines.extend(premise_question_lines(results, "segment_one_groups"));
    for (question, group, arm_error) in [
        ("Q13.nrb", Group::NewestRowBad, ArmError::TodayE1),
        ("Q13.rs1", Group::RollbackSlotBad, ArmError::TodayE1),
        ("Q14", Group::NewestRowBad, ArmError::WithPointerBirthE1),
        ("Q15", Group::RollbackSlotBad, ArmError::WithPointerBirthE1),
    ] {
        let main_value = question_reading(results, group, main, false, arm_error).pairs;
        let wider_value = question_reading(results, group, wider, false, arm_error).pairs;
        lines.push(sensitivity_line(wider, question, main_value, wider_value));
    }
    for (question, group) in [
        ("Q16", Group::RollbackSlotBad),
        ("Q17", Group::NewestRowBad),
    ] {
        let (main_value, _) = checker_violations(
            results,
            group,
            main,
            CheckedInvariant::ReleaseGenerationInterval,
        );
        let (wider_value, _) = checker_violations(
            results,
            group,
            wider,
            CheckedInvariant::ReleaseGenerationInterval,
        );
        lines.push(sensitivity_line(wider, question, main_value, wider_value));
    }
    let unreadable: u64 = [Group::NewestRowBad, Group::RollbackSlotBad]
        .iter()
        .flat_map(|group| POINT_ORDER.iter().map(move |point| (*group, *point)))
        .map(|(group, point)| {
            totals_of(&results_of(results, group, point), false).unreadable_images
        })
        .sum();
    let positive_control_of_today_missed =
        nrb_today.images == 0 || !newest_row_bad_pin_observed.contains(":today_e1_1:");
    let premise_controls_missed = instance_one_control_misses > 0;
    let independent_anchor_failed = !independent_failures.is_empty();
    lines.push(clause_line(
        "Z1",
        positive_control_of_today_missed,
        &format!(
            "nrb_today_e1_images={} c1={newest_row_bad_pin_observed}",
            nrb_today.images
        ),
    ));
    lines.push(clause_line(
        "Z3",
        premise_controls_missed,
        &format!("instance_one_control_histories_missed={instance_one_control_misses}"),
    ));
    lines.push(clause_line(
        "Z5",
        independent_anchor_failed,
        &format!(
            "failed_independent_anchors={}",
            if independent_failures.is_empty() {
                "none".to_string()
            } else {
                independent_failures.join(",")
            }
        ),
    ));
    lines.push(clause_line(
        "Z6",
        false,
        &format!("unreadable_images_listed_separately_not_counted={unreadable}"),
    ));
    lines.push(clause_line(
        "F3",
        nrb_with.pairs + rs1_with.pairs > 0,
        &format!(
            "nrb_with_pointer_birth_pairs={} rs1_with_pointer_birth_pairs={}",
            nrb_with.pairs, rs1_with.pairs
        ),
    ));
    lines.push(clause_line(
        "F4",
        rollback_release_generation_violations + newest_row_release_generation_violations > 0,
        &format!("rs1_release_generation_violated_images={rollback_release_generation_violations} nrb_release_generation_violated_images={newest_row_release_generation_violations}"),
    ));
    lines.push(clause_line(
        "F5",
        !clause_failures.is_empty(),
        &format!(
            "clause_image_anchors_unequal={}",
            if clause_failures.is_empty() {
                "none".to_string()
            } else {
                clause_failures.join(",")
            }
        ),
    ));
    lines.push(clause_line(
        "F7",
        rs1_today.images == 0,
        &format!("rs1_today_e1_images={}", rs1_today.images),
    ));
    lines.push(clause_line(
        "F8",
        !clause_number_mismatches.is_empty(),
        &format!(
            "clause_numbers_unequal={}",
            if clause_number_mismatches.is_empty() {
                "none".to_string()
            } else {
                clause_number_mismatches.join(",")
            }
        ),
    ));
    let void =
        positive_control_of_today_missed || premise_controls_missed || independent_anchor_failed;
    let all_made = nrb_produced == 6
        && totals_of(&results_of(results, Group::RollbackSlotBad, main), false).produced_images
            == 285;
    let status = row_status(stops, void, all_made);
    lines.push(format!("name=verdict row=2 half=reference_interval_precision value_pairs={} judgement={} status={status}", nrb_with.pairs + rs1_with.pairs, if nrb_with.pairs + rs1_with.pairs == 0 { "branch_one" } else { "branch_two" }));
    lines.push(format!("name=verdict row=2 half=release_generation_interval value_images={} judgement={} status={status}", rollback_release_generation_violations + newest_row_release_generation_violations, if rollback_release_generation_violations + newest_row_release_generation_violations == 0 { "branch_one" } else { "branch_two" }));
    lines
}

/// 一行岔路的状态：停机（装置与 `crates/` 对不上）> 作废（阳性对照或独立锚点不中）> 造全了够判 / 还差。
fn row_status(stops: &StopClauses, void: bool, all_made: bool) -> String {
    if !stops.triggered.is_empty() {
        format!("stopped_by_{}", stops.triggered.join("_"))
    } else if void {
        "void".to_string()
    } else if all_made {
        "enough_to_judge".to_string()
    } else {
        "enough_to_judge_on_the_images_made_short_of_the_clause_count".to_string()
    }
}

/// 第二段（补第 1 行）：ss、skr、sw-crash、sw-read 与 G1–G3。
fn judgement_lines_of_segment_two(
    specifications: &[HistorySpecification],
    results: &[HistoryResult],
    stops: &StopClauses,
) -> Vec<String> {
    let mut lines = Vec::new();
    let main = SamplingPoint::Main;
    let (anchor_lines, mut independent_failures, clause_failures) = image_count_anchor_lines(
        specifications,
        results,
        &[
            Group::ShortRing,
            Group::SkippedRow,
            Group::SweepCrash,
            Group::SweepRead,
        ],
    );
    lines.extend(anchor_lines);
    let skipped_row_pin_observed =
        pinned_image_text(find_history(results, &skipped_row_pin_specification()), 0);
    lines.push(anchor_line(
        "C2",
        "independent",
        "skr.main.S4.last6.k0",
        SKIPPED_ROW_PIN_EXPECTED,
        &skipped_row_pin_observed,
    ));
    if skipped_row_pin_observed != SKIPPED_ROW_PIN_EXPECTED {
        independent_failures.push("C2".to_string());
    }
    let questions: [(&str, Group, bool, ArmError); 8] = [
        ("Q4", Group::ShortRing, false, ArmError::WithPointerBirthE1),
        ("Q5", Group::SkippedRow, false, ArmError::WithPointerBirthE1),
        ("Q6", Group::SweepCrash, true, ArmError::WithPointerBirthE1),
        ("Q7", Group::ShortRing, false, ArmError::WithPointerBirthE2),
        ("Q8", Group::SkippedRow, false, ArmError::WithPointerBirthE2),
        ("Q9", Group::SweepCrash, true, ArmError::WithPointerBirthE2),
        ("Q10", Group::SweepRead, true, ArmError::WithPointerBirthE1),
        ("Q11", Group::SweepRead, true, ArmError::WithPointerBirthE2),
    ];
    let mut misclassified_questions = Vec::new();
    for (question, group, only_swept, arm_error) in questions {
        let reading = question_reading(results, group, main, only_swept, arm_error);
        if reading.pairs > 0 {
            misclassified_questions.push(question);
        }
        let scope = if only_swept {
            "swept_histories"
        } else {
            "all_histories"
        };
        lines.push(question_line(
            question, group, main, scope, arm_error, reading,
        ));
    }
    for group in [
        Group::ShortRing,
        Group::SkippedRow,
        Group::SweepCrash,
        Group::SweepRead,
    ] {
        for arm_error in [ArmError::TodayE1, ArmError::TodayE2] {
            let reading = question_reading(results, group, main, false, arm_error);
            lines.push(question_line(
                "Q19",
                group,
                main,
                "all_histories_attached_not_judged",
                arm_error,
                reading,
            ));
        }
    }
    let mut swept_counts = Vec::new();
    for group in [Group::SweepCrash, Group::SweepRead] {
        let totals = totals_of(&results_of(results, group, main), false);
        swept_counts.push((group, totals.histories, totals.swept_histories));
        lines.push(format!(
            "name=verdict question=Q12 group={} point=main histories={} swept_histories={} sweep_txg_matches_the_prediction={} judgement=validity_gate_{}",
            group.name(),
            totals.histories,
            totals.swept_histories,
            totals.sweep_txg_matches_the_prediction,
            if totals.swept_histories == totals.histories { "every_history_swept" } else { "some_history_did_not_sweep_see_F6" }
        ));
    }
    for group in [
        Group::ShortRing,
        Group::SkippedRow,
        Group::SweepCrash,
        Group::SweepRead,
    ] {
        let totals = totals_of(&results_of(results, group, main), false);
        lines.push(format!(
            "name=verdict question=Q18 group={} point=main hole_pairs={} directly_bridged={} existed_versions_in_between={} previous_not_on_the_chain={} version_missing_from_the_ledger={} pointer_unreadable_pairs={} judgement={}",
            group.name(),
            totals.hole_pairs,
            truth_count(&totals.truth_counts, Truth::DirectlyBridged),
            truth_count(&totals.truth_counts, Truth::ExistedVersionsInBetween),
            truth_count(&totals.truth_counts, Truth::PreviousNotOnTheChain),
            truth_count(&totals.truth_counts, Truth::VersionMissingFromTheLedger),
            totals.pointer_unreadable_pairs,
            if totals.hole_pairs == 0 { "no_object_in_this_group" } else { "coverage_only" }
        ));
    }
    let premise_lines = premise_question_lines(results, "segment_two_groups");
    let all: Vec<&HistoryResult> = results.iter().collect();
    let premise_totals = totals_of(&all, false);
    lines.extend(premise_lines);
    let skr = totals_of(&results_of(results, Group::SkippedRow, main), false);
    let ss = totals_of(&results_of(results, Group::ShortRing, main), false);
    let skr_direct = truth_count(&skr.truth_counts, Truth::DirectlyBridged);
    let ss_in_between = truth_count(&ss.truth_counts, Truth::ExistedVersionsInBetween);
    lines.push(format!("name=control control=PC-truth skr_directly_bridged={skr_direct} ss_existed_versions_in_between={ss_in_between} hit={}", yes_or_no(skr_direct >= 1 && ss_in_between >= 1)));
    let sweep_read_swept = totals_of(&results_of(results, Group::SweepRead, main), true);
    lines.push(format!(
        "name=control control=PC-with-pointer-birth swept_histories={} with_pointer_birth_e1_pairs={} hit={}",
        sweep_read_swept.histories,
        sweep_read_swept.pairs.with_pointer_birth_e1,
        yes_or_no(sweep_read_swept.histories > 0 && sweep_read_swept.pairs.with_pointer_birth_e1 >= 1)
    ));
    let sweep_swept: Vec<&HistoryResult> = results
        .iter()
        .filter(|result| history_swept(result))
        .collect();
    let sweep_changes: u64 = sweep_swept
        .iter()
        .map(|result| result.premise.sweep_pointer_changes)
        .sum();
    lines.push(format!("name=control control=PC-P2 swept_histories={} sweep_pointer_changes={sweep_changes} hit={}", sweep_swept.len(), yes_or_no(!sweep_swept.is_empty() && sweep_changes >= 1)));
    let (instance_one_control_text, instance_one_control_misses) =
        instance_one_control_line(results);
    lines.push(instance_one_control_text);
    let sensitivity_cases: [(SamplingPoint, &str, Group, bool, ArmError); 8] = [
        (
            SamplingPoint::SweepThresholdProductDefault,
            "Q6",
            Group::SweepCrash,
            true,
            ArmError::WithPointerBirthE1,
        ),
        (
            SamplingPoint::SweepThresholdProductDefault,
            "Q9",
            Group::SweepCrash,
            true,
            ArmError::WithPointerBirthE2,
        ),
        (
            SamplingPoint::LongJournalRing,
            "Q4",
            Group::ShortRing,
            false,
            ArmError::WithPointerBirthE1,
        ),
        (
            SamplingPoint::LongJournalRing,
            "Q7",
            Group::ShortRing,
            false,
            ArmError::WithPointerBirthE2,
        ),
        (
            SamplingPoint::LongJournalRing,
            "Q19.today_E1",
            Group::ShortRing,
            false,
            ArmError::TodayE1,
        ),
        (
            SamplingPoint::LongJournalRing,
            "Q19.today_E2",
            Group::ShortRing,
            false,
            ArmError::TodayE2,
        ),
        (
            SamplingPoint::SkippedRowWiderRing,
            "Q5",
            Group::SkippedRow,
            false,
            ArmError::WithPointerBirthE1,
        ),
        (
            SamplingPoint::SkippedRowWiderRing,
            "Q8",
            Group::SkippedRow,
            false,
            ArmError::WithPointerBirthE2,
        ),
    ];
    for (point, question, group, only_swept, arm_error) in sensitivity_cases {
        let main_value = question_reading(results, group, main, only_swept, arm_error).pairs;
        let sensitive_value = question_reading(results, group, point, only_swept, arm_error).pairs;
        lines.push(sensitivity_line(
            point,
            question,
            main_value,
            sensitive_value,
        ));
    }
    let sweep_threshold_point = SamplingPoint::SweepThresholdProductDefault;
    let main_sweep = totals_of(&results_of(results, Group::SweepCrash, main), false);
    let sweep_threshold_point_totals = totals_of(
        &results_of(results, Group::SweepCrash, sweep_threshold_point),
        false,
    );
    lines.push(sensitivity_line(
        sweep_threshold_point,
        "Q1",
        main_sweep.adjacent_version_violations,
        sweep_threshold_point_totals.adjacent_version_violations,
    ));
    lines.push(sensitivity_line(
        sweep_threshold_point,
        "Q12.histories_without_a_sweep",
        main_sweep.histories - main_sweep.swept_histories,
        sweep_threshold_point_totals.histories - sweep_threshold_point_totals.swept_histories,
    ));
    let positive_control_of_the_pointer_birth_missed =
        sweep_read_swept.histories > 0 && sweep_read_swept.pairs.with_pointer_birth_e1 == 0;
    let premise_controls_missed =
        instance_one_control_misses > 0 || (!sweep_swept.is_empty() && sweep_changes == 0);
    let truth_control_missed = skr_direct == 0 || ss_in_between == 0;
    let independent_anchor_failed = !independent_failures.is_empty();
    let void = positive_control_of_the_pointer_birth_missed
        || premise_controls_missed
        || truth_control_missed
        || independent_anchor_failed;
    lines.push(clause_line(
        "Z2",
        positive_control_of_the_pointer_birth_missed,
        &format!(
            "swept_sweep_read_histories={} with_pointer_birth_e1_pairs={}",
            sweep_read_swept.histories, sweep_read_swept.pairs.with_pointer_birth_e1
        ),
    ));
    lines.push(clause_line("Z3", premise_controls_missed, &format!("instance_one_control_histories_missed={instance_one_control_misses} swept_histories={} sweep_pointer_changes={sweep_changes}", sweep_swept.len())));
    lines.push(clause_line(
        "Z4",
        truth_control_missed,
        &format!(
            "skr_directly_bridged={skr_direct} ss_existed_versions_in_between={ss_in_between}"
        ),
    ));
    lines.push(clause_line(
        "Z5",
        independent_anchor_failed,
        &format!(
            "failed_independent_anchors={}",
            joined_or_none(&independent_failures)
        ),
    ));
    let unreadable: u64 = [
        Group::ShortRing,
        Group::SkippedRow,
        Group::SweepCrash,
        Group::SweepRead,
    ]
    .iter()
    .flat_map(|group| POINT_ORDER.iter().map(move |point| (*group, *point)))
    .map(|(group, point)| totals_of(&results_of(results, group, point), false).unreadable_images)
    .sum();
    lines.push(clause_line(
        "Z6",
        false,
        &format!("unreadable_images_listed_separately_not_counted={unreadable}"),
    ));
    let misclassified: Vec<String> = misclassified_questions
        .iter()
        .map(|question| (*question).to_string())
        .collect();
    lines.push(clause_line(
        "F1",
        !misclassified.is_empty(),
        &format!(
            "questions_with_misclassified_pairs={}",
            joined_or_none(&misclassified)
        ),
    ));
    let premise_broken = premise_totals.first_version_violations
        + premise_totals.adjacent_version_violations
        + premise_totals.birth_after_the_txg_violations
        > 0;
    lines.push(clause_line(
        "F2",
        premise_broken,
        &format!(
            "first_version_violations={} adjacent_version_violations={} birth_after_the_txg_violations={}",
            premise_totals.first_version_violations,
            premise_totals.adjacent_version_violations,
            premise_totals.birth_after_the_txg_violations
        ),
    ));
    lines.push(clause_line(
        "F5",
        !clause_failures.is_empty(),
        &format!(
            "clause_image_anchors_unequal={}",
            joined_or_none(&clause_failures)
        ),
    ));
    let unswept: Vec<String> = swept_counts
        .iter()
        .filter(|(_, histories, swept)| swept < histories)
        .map(|(group, histories, swept)| format!("{}:{swept}/{histories}", group.name()))
        .collect();
    lines.push(clause_line(
        "F6",
        !unswept.is_empty(),
        &format!("groups_with_unswept_histories={}", joined_or_none(&unswept)),
    ));
    let made = [
        Group::ShortRing,
        Group::SkippedRow,
        Group::SweepCrash,
        Group::SweepRead,
    ]
    .iter()
    .all(|group| totals_of(&results_of(results, *group, main), false).judged_images > 0)
        && [
            SamplingPoint::SweepThresholdProductDefault,
            SamplingPoint::LongJournalRing,
            SamplingPoint::SkippedRowWiderRing,
        ]
        .iter()
        .all(|point| {
            results
                .iter()
                .any(|result| result.specification.point == *point && !result.images.is_empty())
        });
    let status = row_status(stops, void, made && clause_failures.is_empty());
    lines.push(format!(
        "name=verdict row=1 questions_with_misclassified_pairs={} premise_violations={} judgement={} status={status}",
        joined_or_none(&misclassified),
        premise_totals.first_version_violations + premise_totals.adjacent_version_violations + premise_totals.birth_after_the_txg_violations,
        if misclassified.is_empty() && !premise_broken { "branch_one" } else { "branch_two" }
    ));
    lines
}

/// 第三段（附带，不判岔路）：sw-none 只报合计与轨迹。
fn judgement_lines_of_segment_three(results: &[HistoryResult], stops: &StopClauses) -> Vec<String> {
    let mut lines = Vec::new();
    for arm_error in ARM_ERRORS {
        let reading = question_reading(
            results,
            Group::SweepNone,
            SamplingPoint::Main,
            false,
            arm_error,
        );
        lines.push(question_line(
            "Q19",
            Group::SweepNone,
            SamplingPoint::Main,
            "all_histories_attached_not_judged",
            arm_error,
            reading,
        ));
    }
    lines.push(format!(
        "name=verdict row=attached status={}",
        row_status(stops, false, true)
    ));
    lines
}

// ============================================================================
// 十四、入口：参数、准入、打印。
// ============================================================================

/// 跑一段：建全部历史、判停机条款；`print_the_classification` 时另打每段历史、每张镜像、合计、轨迹与判定（产物），否则只打停机条款（登记 3b）。
fn run_the_segment(segment: Segment, print_the_classification: bool) -> Vec<String> {
    let specifications = segment.history_specifications();
    let threads = worker_threads();
    let constants = constant_checks();
    let started = std::time::Instant::now();
    eprintln!(
        "  {}：{} 段历史，{threads} 个线程",
        segment.name(),
        specifications.len()
    );
    let results = run_the_histories(&specifications, threads);
    eprintln!(
        "  {}：历史全部判完，用了 {:.0} 秒",
        segment.name(),
        started.elapsed().as_secs_f64()
    );
    let mut lines = vec![format!(
        "name=config experiment=e198 segment={} mode={} histories={} history_threads={threads} judging_threads={threads} output_threads=1",
        segment.name(),
        if print_the_classification { "products" } else { "stop_clauses" },
        specifications.len()
    )];
    for check in &constants {
        lines.push(format!(
            "name=constant constant={} local={} implementation={} equal={}",
            check.name,
            check.local,
            check.implementation,
            yes_or_no(check.holds())
        ));
    }
    let stops = stop_clauses(&results, &constants);
    lines.extend(stops.lines.iter().cloned());
    if print_the_classification {
        for result in &results {
            lines.extend(history_lines(result));
            lines.extend(history_trajectory_lines(result));
            for image in &result.images {
                lines.push(image_line(result, image));
            }
        }
        for group in GROUP_ORDER {
            for point in POINT_ORDER {
                lines.extend(group_lines(&results, group, point));
            }
        }
        lines.extend(match segment {
            Segment::One => judgement_lines_of_segment_one(&specifications, &results, &stops),
            Segment::Two => judgement_lines_of_segment_two(&specifications, &results, &stops),
            Segment::Three => judgement_lines_of_segment_three(&results, &stops),
        });
    } else {
        lines.push(format!(
            "name=verdict stop_clauses_triggered={} status={}",
            if stops.triggered.is_empty() {
                "none".to_string()
            } else {
                stops.triggered.join(",")
            },
            if stops.triggered.is_empty() {
                "products_may_run"
            } else {
                "stopped_hand_to_the_main_agent"
            }
        ));
    }
    lines
}

/// `checks`：常量回比与闭式张数锚点，不建历史（编完先跑它，S4 不过就不跑产物）。
fn run_the_checks() -> Vec<String> {
    let mut lines = Vec::new();
    for check in constant_checks() {
        lines.push(format!(
            "name=constant constant={} local={} implementation={} equal={}",
            check.name,
            check.local,
            check.implementation,
            yes_or_no(check.holds())
        ));
    }
    let mut every_specification = Segment::One.history_specifications();
    every_specification.extend(Segment::Two.history_specifications());
    for (anchor, group, point, expected) in ATTEMPTED_IMAGE_ANCHORS {
        let observed = attempted_images_of(&every_specification, group, point)
            .map_or("unknown".to_string(), |count| count.to_string());
        lines.push(anchor_line(
            anchor,
            "independent",
            &format!("attempted_images.{}.{}", group.name(), point.name()),
            &expected.to_string(),
            &observed,
        ));
    }
    lines
}

/// 准入与运行条件（`.claude/singlefs-ai-sop/rules/preflight-discipline.md`「开头先判」Rust 那一行）：起 `preflight.py check`，
/// 退出码不是 0 就原样退出；强制跑（`--force`）时交回摘要，`main` 把它作为头一行打到标准输出。
fn preflight() -> Option<String> {
    let manifest_directory = env!("CARGO_MANIFEST_DIR");
    let source =
        format!("{manifest_directory}/src/bin/e198_instance_table_pointer_birth_skip_premise.rs");
    let script = format!("{manifest_directory}/../../.claude/singlefs-ai-sop/scripts/preflight.py");
    let arguments: Vec<String> = std::env::args().skip(1).collect();
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
    let first_line = String::from_utf8_lossy(&output.stdout)
        .lines()
        .next()
        .unwrap_or("")
        .to_string();
    first_line
        .strip_prefix("forced")
        .map(|summary| summary.trim().replace(' ', "_"))
}

fn usage() {
    eprintln!("用法：e198_instance_table_pointer_birth_skip_premise <checks | segment-one | segment-two | segment-three | stop-clauses <段名> | list-items <段名>> [--force]");
}

fn print_result_lines(lines: &[String]) {
    let stdout = std::io::stdout();
    let mut handle = stdout.lock();
    for line in lines {
        writeln!(handle, "E7RESULT {line}").expect("标准输出写得进");
    }
    writeln!(handle, "E7RESULT name=done result_lines={}", lines.len()).expect("标准输出写得进");
}

fn main() {
    let forced_summary = preflight();
    let arguments: Vec<String> = std::env::args()
        .skip(1)
        .filter(|argument| argument != "--force")
        .collect();
    let words: Vec<&str> = arguments.iter().map(String::as_str).collect();
    if let Some(summary) = forced_summary {
        if words.first() == Some(&"list-items") {
            eprintln!("name=preflight forced={summary}");
        } else {
            println!("E7RESULT name=preflight forced={summary}");
        }
    }
    let lines = match words.as_slice() {
        ["checks"] => run_the_checks(),
        [segment_name] if Segment::from_name(segment_name).is_some() => run_the_segment(
            Segment::from_name(segment_name).expect("match 守卫判过认得出"),
            true,
        ),
        ["stop-clauses", segment_name] if Segment::from_name(segment_name).is_some() => {
            run_the_segment(
                Segment::from_name(segment_name).expect("match 守卫判过认得出"),
                false,
            )
        }
        ["list-items", segment_name] if Segment::from_name(segment_name).is_some() => {
            for specification in Segment::from_name(segment_name)
                .expect("match 守卫判过认得出")
                .history_specifications()
            {
                println!("{}", history_name(&specification));
            }
            return;
        }
        _ => {
            usage();
            std::process::exit(2);
        }
    };
    print_result_lines(&lines);
}

// ============================================================================
// 十五、单测（登记第五节 PC-单测、第七节 C1 / C2、第九节变异的必红断言）。纯判法各喂一张已知答案表；
//     造历史的几条各建一段真历史（nrb、skr、ss、sw-crash、sw-read、rs1 各一段，G1 那一段要攒 366 行、不进单测）。
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    fn key(instance: u32, txg: u64) -> VersionKey {
        VersionKey { instance, txg }
    }

    fn row(
        instance: u32,
        published_checkpoint_txg: u64,
        applied_transaction_high_water_mark: u64,
    ) -> RowOfTheNewestTable {
        RowOfTheNewestTable {
            instance,
            published_checkpoint_txg,
            applied_transaction_high_water_mark,
        }
    }

    fn ring_version(instance: u32, txg: u64, birth: Option<u64>) -> VersionOnTheImage {
        VersionOnTheImage {
            key: key(instance, txg),
            pointer_birth: birth.map_or(PointerBirth::AllZeroOrUnreadable, PointerBirth::Readable),
        }
    }

    #[test]
    fn today_arm_answers_its_known_answer_table() {
        let one = BTreeSet::from([1]);
        let bridged_rows = [row(1, 6, 5), row(2, 0, 0)];
        assert_eq!(
            hole_class_today(6, &one, 8, 3, &bridged_rows),
            HoleClass::BridgedByTheRows,
            "行 (1, 6) 加中间实例 (2, 0, 0) 接得上"
        );
        assert_eq!(
            hole_class_today(6, &one, 8, 3, &[row(1, 6, 5)]),
            HoleClass::NotBridgedByTheRows,
            "中间实例 2 没有行"
        );
        assert_eq!(
            hole_class_today(6, &one, 8, 3, &[row(1, 6, 5), row(2, 7, 0)]),
            HoleClass::NotBridgedByTheRows,
            "中间实例发过东西"
        );
        assert_eq!(
            hole_class_today(6, &one, 8, 3, &[row(1, 5, 5), row(2, 0, 0)]),
            HoleClass::NotBridgedByTheRows,
            "行的 T 不等于 R⁻.txg"
        );
        assert_eq!(
            hole_class_today(6, &one, 8, 1, &[row(1, 6, 5)]),
            HoleClass::NotBridgedByTheRows,
            "R 的实例不比 R⁻ 的大"
        );
        assert_eq!(
            hole_class_today(6, &one, 7, 3, &bridged_rows),
            HoleClass::NoHole,
            "R.txg = R⁻.txg + 1 没有洞"
        );
        assert_eq!(
            hole_class_today(6, &BTreeSet::from([1, 2]), 9, 3, &[row(2, 6, 3)]),
            HoleClass::BridgedByTheRows,
            "R⁻ 那一组里任一个实例接得上就算"
        );
    }

    #[test]
    fn with_pointer_birth_arm_answers_its_known_answer_table() {
        let previous = [ring_version(1, 6, Some(0))];
        assert!(
            bridged_with_the_pointer_birth(
                HoleClass::BridgedByTheRows,
                &previous,
                &ring_version(3, 8, Some(8))
            ),
            "出生 txg 等于 R.txg"
        );
        assert!(
            !bridged_with_the_pointer_birth(
                HoleClass::BridgedByTheRows,
                &previous,
                &ring_version(3, 8, Some(7))
            ),
            "出生 txg 小于 R.txg"
        );
        assert!(
            !bridged_with_the_pointer_birth(
                HoleClass::BridgedByTheRows,
                &previous,
                &ring_version(3, 8, None)
            ),
            "指针读不出按不认作取号跳过"
        );
        assert!(
            !bridged_with_the_pointer_birth(
                HoleClass::NotBridgedByTheRows,
                &previous,
                &ring_version(3, 8, Some(8))
            ),
            "行接不上"
        );
    }

    #[test]
    fn each_arm_counts_e1_and_e2_exactly_on_the_four_kinds_of_pairs() {
        let pair = |bridged_today: bool, bridged_with_the_pointer_birth: bool| HolePair {
            previous_txg: 6,
            previous_group: vec![key(1, 6)],
            version: ring_version(2, 8, Some(7)),
            bridged_today,
            bridged_with_the_pointer_birth,
        };
        let counts = |today_e1, today_e2, with_pointer_birth_e1, with_pointer_birth_e2| ArmCounts {
            today_e1,
            today_e2,
            with_pointer_birth_e1,
            with_pointer_birth_e2,
        };
        assert_eq!(
            arm_counts_of_the_pair(&pair(true, true), Truth::ExistedVersionsInBetween),
            counts(1, 0, 1, 0),
            "两臂都把存在过的认成取号跳过：E1"
        );
        assert_eq!(
            arm_counts_of_the_pair(&pair(false, false), Truth::DirectlyBridged),
            counts(0, 1, 0, 1),
            "两臂都把取号跳过认成存在过：E2"
        );
        assert_eq!(
            arm_counts_of_the_pair(&pair(true, false), Truth::PreviousNotOnTheChain),
            counts(1, 0, 0, 0),
            "只有今天的臂认错"
        );
        assert_eq!(
            arm_counts_of_the_pair(&pair(true, true), Truth::DirectlyBridged),
            counts(0, 0, 0, 0),
            "两臂都认对取号跳过"
        );
        assert_eq!(
            arm_counts_of_the_pair(&pair(false, false), Truth::ExistedVersionsInBetween),
            counts(0, 0, 0, 0),
            "两臂都认对存在过"
        );
        assert_eq!(
            arm_counts_of_the_pair(&pair(true, true), Truth::VersionMissingFromTheLedger),
            counts(0, 0, 0, 0),
            "R 不在账本里的不计"
        );
    }

    fn ledger_version(
        parent: Option<VersionKey>,
        kind: PublishKind,
        pointer_birth_txg: u64,
        root_landed: bool,
        applied_by_a_mount: bool,
        publish_order: usize,
    ) -> LedgerVersion {
        LedgerVersion {
            parent,
            kind,
            pointer_birth_txg,
            root_landed,
            applied_by_a_mount,
            publish_order,
            rows: Arc::default(),
        }
    }

    /// 0@0 → 1@4 → 1@5（被施加、根槽没落）→ 2@6 → 2@7 → 2@8（崩、没被施加）；3@9 的父版本是 2@7（取号越过 2@8）。
    fn chain_ledger() -> Ledger {
        let mut ledger = Ledger::default();
        let entries = [
            (key(0, 0), None, PublishKind::MakeFilesystem, true, false),
            (
                key(1, 4),
                Some(key(0, 0)),
                PublishKind::Overwrite,
                true,
                false,
            ),
            (
                key(1, 5),
                Some(key(1, 4)),
                PublishKind::Overwrite,
                false,
                true,
            ),
            (
                key(2, 6),
                Some(key(1, 5)),
                PublishKind::RowPublish,
                true,
                false,
            ),
            (key(2, 7), Some(key(2, 6)), PublishKind::WarmUp, true, false),
            (
                key(2, 8),
                Some(key(2, 7)),
                PublishKind::Overwrite,
                false,
                false,
            ),
            (
                key(3, 9),
                Some(key(2, 7)),
                PublishKind::RowPublish,
                true,
                false,
            ),
        ];
        for (order, (version, parent, kind, landed, applied)) in entries.into_iter().enumerate() {
            ledger.versions.insert(
                version,
                ledger_version(parent, kind, 0, landed, applied, order),
            );
        }
        ledger
    }

    #[test]
    fn truth_reads_the_parent_chain_over_versions_that_existed() {
        let ledger = chain_ledger();
        assert_eq!(
            truth_of_the_hole_pair(&ledger, &[key(1, 5)], key(2, 6)),
            Truth::DirectlyBridged,
            "父版本就在 R⁻ 那一组里"
        );
        assert_eq!(
            truth_of_the_hole_pair(&ledger, &[key(1, 4)], key(2, 6)),
            Truth::ExistedVersionsInBetween,
            "中间隔着被施加出来的 1@5"
        );
        assert_eq!(
            truth_of_the_hole_pair(&ledger, &[key(2, 8)], key(3, 9)),
            Truth::PreviousNotOnTheChain,
            "崩了的 2@8 不在 3@9 的祖先链上"
        );
        assert_eq!(
            truth_of_the_hole_pair(&ledger, &[key(2, 7)], key(3, 9)),
            Truth::DirectlyBridged,
            "取号越过之后直接接在 2@7 上"
        );
        assert_eq!(
            truth_of_the_hole_pair(&ledger, &[key(2, 7)], key(4, 10)),
            Truth::VersionMissingFromTheLedger,
            "不在账本里的版本单列"
        );
    }

    fn ring_root(region: u64, instance: u32, txg: u64, rollback_floor: u64) -> RootOnTheRing {
        RootOnTheRing {
            region,
            key: key(instance, txg),
            rollback_floor,
            pointer_birth: PointerBirth::Readable(0),
        }
    }

    fn record(
        instance: u32,
        txg: u64,
        is_commit: bool,
        is_the_last_record_of_the_publish: bool,
        birth: u64,
    ) -> RecordOnTheRing {
        RecordOnTheRing {
            key: key(instance, txg),
            is_commit,
            is_the_last_record_of_the_publish,
            pointer_birth: PointerBirth::Readable(birth),
        }
    }

    #[test]
    fn versions_applied_only_by_records_need_all_four_conditions() {
        let rows = [row(1, 10, 9)];
        let roots = [ring_root(0, 1, 6, 0), ring_root(2, 1, 8, 0)];
        let records = [
            record(1, 5, true, true, 5),
            record(1, 7, true, true, 7),
            record(1, 8, true, true, 8),
            record(1, 9, true, false, 3),
            record(1, 9, true, true, 9),
            record(1, 10, false, true, 10),
            record(1, 11, true, true, 11),
            record(2, 9, true, true, 9),
        ];
        let applied = versions_applied_only_by_records(&records, &roots, &rows, 0);
        let expected = vec![
            VersionOnTheImage {
                key: key(1, 7),
                pointer_birth: PointerBirth::Readable(7),
            },
            VersionOnTheImage {
                key: key(1, 9),
                pointer_birth: PointerBirth::Readable(9),
            },
        ];
        assert_eq!(applied, expected, "1@5 环里最旧有效根不比它旧、1@8 根槽读得出、1@10 没提交、1@11 超过行的 T、实例 2 没有行；1@9 取带末条标志那一条");
        let applied_above_the_floor = versions_applied_only_by_records(&records, &roots, &rows, 8);
        assert_eq!(
            applied_above_the_floor
                .iter()
                .map(|version| version.key)
                .collect::<Vec<_>>(),
            vec![key(1, 9)],
            "F = 8 时 1@7 在 F 之下"
        );
    }

    #[test]
    fn candidate_set_keeps_the_newest_and_drops_abandoned_and_below_the_floor() {
        let rows = [row(1, 6, 0), row(2, 7, 0)];
        let roots = [
            ring_root(0, 1, 3, 0),
            ring_root(1, 1, 4, 0),
            ring_root(0, 1, 6, 0),
            ring_root(2, 2, 8, 0),
            ring_root(0, 3, 9, 0),
        ];
        assert_eq!(newest_root_index(&roots), Some(4));
        assert_eq!(
            candidate_root_indexes(&roots, &rows, 4, 4),
            vec![1, 2, 4],
            "1@3 在 F 之下、2@8 被行 (2, 7) 判抛弃"
        );
        let abandoned_newest = [ring_root(0, 1, 3, 0), ring_root(1, 2, 8, 0)];
        assert_eq!(
            candidate_root_indexes(&abandoned_newest, &rows, 0, 1),
            vec![0, 1],
            "最新根恒收"
        );
    }

    #[test]
    fn effective_floor_takes_the_highest_of_the_roots_and_the_configuration() {
        let rows = [row(2, 7, 0)];
        let roots = [
            ring_root(0, 1, 6, 2),
            ring_root(1, 1, 5, 3),
            ring_root(2, 2, 8, 5),
        ];
        assert_eq!(
            effective_rollback_floor(&roots, &rows, &[4]),
            5,
            "被抛弃的 2@8 带的 F 也算进来"
        );
        assert_eq!(
            effective_rollback_floor(&roots, &rows, &[7]),
            7,
            "系统配置带的 F 更高时取它"
        );
        assert_eq!(
            effective_rollback_floor(&roots[..2], &rows, &[]),
            3,
            "每块盘最新有效根带的 F 取最大"
        );
    }

    #[test]
    fn hole_pairs_pair_every_version_of_a_txg_with_the_whole_group_before_it() {
        let rows = [row(1, 4, 0), row(2, 0, 0), row(3, 6, 1)];
        let versions = [
            ring_version(0, 0, Some(0)),
            ring_version(1, 4, Some(0)),
            ring_version(2, 6, Some(6)),
            ring_version(3, 6, Some(5)),
            ring_version(4, 9, Some(9)),
            ring_version(4, 10, Some(9)),
        ];
        let pairs = hole_pairs_of(&versions, &rows);
        let summary: Vec<(u64, Vec<VersionKey>, VersionKey, bool, bool)> = pairs
            .iter()
            .map(|pair| {
                (
                    pair.previous_txg,
                    pair.previous_group.clone(),
                    pair.version.key,
                    pair.bridged_today,
                    pair.bridged_with_the_pointer_birth,
                )
            })
            .collect();
        assert_eq!(
            summary,
            vec![
                (0, vec![key(0, 0)], key(1, 4), false, false),
                (4, vec![key(1, 4)], key(2, 6), true, true),
                (4, vec![key(1, 4)], key(3, 6), true, false),
                (6, vec![key(2, 6), key(3, 6)], key(4, 9), true, true),
            ],
            "同 txg 两版并成一组、R 侧逐条各算一对；9 → 10 没有洞"
        );
    }

    #[test]
    fn premise_counts_each_clause_and_reports_instance_one_separately() {
        let mut ledger = Ledger::default();
        let entries = [
            (key(0, 0), PublishKind::MakeFilesystem, 0),
            (key(1, 1), PublishKind::RowPublish, 0),
            (key(1, 2), PublishKind::WarmUp, 0),
            (key(1, 3), PublishKind::FirstFile, 0),
            (key(2, 4), PublishKind::RowPublish, 4),
            (key(2, 5), PublishKind::WarmUp, 4),
            (key(2, 6), PublishKind::Sweep, 6),
            (key(2, 7), PublishKind::Overwrite, 6),
            (key(3, 8), PublishKind::RowPublish, 8),
            (key(3, 9), PublishKind::Overwrite, 9),
            (key(3, 10), PublishKind::Overwrite, 11),
        ];
        for (order, (version, kind, birth)) in entries.into_iter().enumerate() {
            ledger.versions.insert(
                version,
                ledger_version(None, kind, birth, true, false, order),
            );
        }
        let counts = premise_counts(&ledger);
        assert_eq!(
            counts.instance_one_first_version_mismatches, 1,
            "实例 1 的第一版照抄 mkfs 那一份指针（PC-P1）"
        );
        assert_eq!(
            counts.first_version_violations.len(),
            0,
            "实例 2、3 的写行那一版出生 txg 等于自己的 txg"
        );
        assert_eq!(counts.sweep_pointer_changes, 1, "2@6 是清扫换指针（PC-P2）");
        assert_eq!(
            counts.adjacent_version_violations.len(),
            2,
            "3@9 与 3@10 两次非清扫换指针"
        );
        assert_eq!(
            counts.birth_after_the_txg_violations.len(),
            1,
            "3@10 的出生 txg 11 大于自己的 txg"
        );
    }

    #[test]
    fn predicted_warm_ups_follow_the_region_to_device_map() {
        assert_eq!(
            predicted_warm_up_publishes(17),
            2,
            "登记第十三节 anchors.py：b = 16 时 w₃ = 2"
        );
        assert_eq!(predicted_warm_up_publishes(18), 1, "b = 17 时 w₃ = 1");
        assert_eq!(predicted_warm_up_publishes(11), 2, "C1：写行 11、暖机 2 次");
        assert_eq!(predicted_warm_up_publishes(8), 2, "C2：写行 8、暖机 2 次");
    }

    #[test]
    fn closed_form_attempted_images_match_the_registered_anchors() {
        let mut every_specification = Segment::One.history_specifications();
        every_specification.extend(Segment::Two.history_specifications());
        for (anchor, group, point, expected) in ATTEMPTED_IMAGE_ANCHORS {
            assert_eq!(
                attempted_images_of(&every_specification, group, point),
                Some(expected),
                "{anchor} {} {}",
                group.name(),
                point.name()
            );
        }
        assert_eq!(
            Segment::One.history_specifications().len(),
            6 + 105 + 6 + 42,
            "第一段：nrb 6、rs1 105、G4 的 nrb 6 与 rs1 42"
        );
        assert_eq!(
            Segment::Two.history_specifications().len(),
            27 + 3 + 27 + 27 + 1 + 9 + 3,
            "第二段：ss、skr、sw-crash、sw-read、G1、G2、G3"
        );
        assert_eq!(
            rollback_action_strings().len(),
            1 + 4 + 16,
            "动作串：不做、一个、两个"
        );
        let names: BTreeSet<String> = every_specification.iter().map(history_name).collect();
        assert_eq!(names.len(), every_specification.len(), "历史名两两不同");
    }

    #[test]
    fn constants_match_the_implementation() {
        for check in constant_checks() {
            assert!(
                check.holds(),
                "{} 本地 {} 实现 {}",
                check.name,
                check.local,
                check.implementation
            );
        }
    }

    /// 造历史的几条共用的核对：写者一步没停、每张都读得出，停机条款 S1、S2、S3 与账本自检都没触发（S5 另报，见登记修订）。
    fn assert_the_history_is_consistent(result: &HistoryResult, expected_images: usize) {
        assert_eq!(result.stopped, None, "{} 的写者没停", result.name);
        assert_eq!(
            result.images.len(),
            expected_images,
            "{} 的张数",
            result.name
        );
        assert!(
            result.ledger.consistency_problems.is_empty(),
            "{:?}",
            result.ledger.consistency_problems
        );
        assert!(
            result.ledger.ring_formula_mismatches.is_empty(),
            "{:?}",
            result.ledger.ring_formula_mismatches
        );
        assert!(
            result.ledger.root_slot_write_mismatches.is_empty(),
            "{:?}",
            result.ledger.root_slot_write_mismatches
        );
        assert!(
            result.ledger.root_slot_writes_compared > 0,
            "S2 一条都没比到"
        );
        for image in &result.images {
            assert!(
                image.reading.is_ok(),
                "{}/{}：{:?}",
                result.name,
                image.label,
                image.reading.as_ref().err()
            );
            assert!(
                image.ring_slot_mismatches.is_empty(),
                "{}/{}：{:?}",
                result.name,
                image.label,
                image.ring_slot_mismatches
            );
            assert_eq!(image.row_mismatch, None, "{}/{}", result.name, image.label);
        }
    }

    #[test]
    fn newest_row_bad_with_the_row_publish_slot_read_bad_pins_the_registered_hole_pair_table() {
        let result = run_the_history(&newest_row_bad_pin_specification());
        assert_the_history_is_consistent(&result, 1);
        assert_eq!(
            pinned_image_text(Some(&result), 0),
            NEWEST_ROW_BAD_PIN_EXPECTED,
            "登记第七节 C1"
        );
    }

    #[test]
    fn skipped_row_first_image_pins_the_registered_hole_pair_table() {
        let result = run_the_history(&skipped_row_pin_specification());
        assert_the_history_is_consistent(&result, 15);
        assert_eq!(
            pinned_image_text(Some(&result), 0),
            SKIPPED_ROW_PIN_EXPECTED,
            "登记第七节 C2"
        );
    }

    fn short_ring_specification(chunks: u64) -> HistorySpecification {
        HistorySpecification {
            group: Group::ShortRing,
            point: SamplingPoint::Main,
            slots_per_region: 4,
            journal_ring_bytes: SHORT_JOURNAL_RING_BYTES,
            shape: HistoryShape::ShortRing { last: 4, chunks },
        }
    }

    /// ss（S 4、last 4、c 2）挂载之后那一张：last + 1 = 5 那一版由记录施加出来（根槽从没落盘、记录还读得出），候选集连成一串、没有洞；
    /// 它也是账本里「被施加过、根槽没落」的那一版（S5 两边相同）。
    #[test]
    fn short_ring_first_image_pins_the_applied_version_and_has_no_hole_pair() {
        let result = run_the_history(&short_ring_specification(2));
        assert_the_history_is_consistent(&result, 13);
        let first = &result.images[0];
        let reading = first.reading.as_ref().expect("第一张读得出");
        assert_eq!(
            reading
                .applied_only_by_records
                .iter()
                .map(|version| version.key)
                .collect::<Vec<_>>(),
            vec![key(1, 5)],
            "由记录施加出来的只有 1@5"
        );
        assert_eq!(reading.hole_pairs, Vec::new(), "0 1 2 3 4 5 6 7 连成一串");
        assert_eq!(
            first.records_applied.mismatch_count(),
            0,
            "S5：装置认的与账本推的相同"
        );
    }

    /// ss（S 4、last 4、c 4）：顺序写把 1 MiB 的 journal 环转一圈之后 1@5 的记录被盖掉，(4, 6) 成了洞对，中间那一版 1@5 存在过
    /// （实例 2 的挂载施加了它）；今天的臂与甲都不认作取号跳过（行是 (1, 5)，不是 (1, 4)）。
    #[test]
    fn short_ring_hole_left_by_an_overwritten_record_is_an_existed_version_in_between() {
        let result = run_the_history(&short_ring_specification(4));
        assert_the_history_is_consistent(&result, 13);
        let found: Vec<(Truth, bool, bool)> = result
            .images
            .iter()
            .filter_map(|image| image.reading.as_ref().ok().map(|reading| (image, reading)))
            .flat_map(|(image, reading)| reading.hole_pairs.iter().zip(image.truths.iter()))
            .filter(|(pair, _)| pair.previous_txg == 4 && pair.version.key == key(2, 6))
            .map(|(pair, truth)| {
                (
                    *truth,
                    pair.bridged_today,
                    pair.bridged_with_the_pointer_birth,
                )
            })
            .collect();
        assert!(
            !found.is_empty(),
            "(4, 6) 这个洞对在 k = 0 … 12 里一次都没出现"
        );
        for observation in found {
            assert_eq!(
                observation,
                (Truth::ExistedVersionsInBetween, false, false),
                "真值是中间有存在过的版本，两臂都不认"
            );
        }
    }

    fn sweep_specification(variant: SweepVariant) -> HistorySpecification {
        HistorySpecification {
            group: sweep_variant_group(variant),
            point: SamplingPoint::Main,
            slots_per_region: 4,
            journal_ring_bytes: SHORT_JOURNAL_RING_BYTES,
            shape: HistoryShape::Sweep {
                first_instance_last: 4,
                extra_overwrites: 0,
                threshold: SWEEP_THRESHOLD_ANY_ROW,
                accumulate_rows_by_mounts: false,
                variant,
            },
        }
    }

    #[test]
    fn sweep_crash_history_sweeps_once_and_changes_the_pointer_only_there() {
        let result = run_the_history(&sweep_specification(
            SweepVariant::CrashBeforeTheNextInstance,
        ));
        assert_the_history_is_consistent(&result, 13);
        assert!(
            history_swept(&result),
            "实例 3 的挂载末尾发了清扫：{:?}",
            result.sweep
        );
        assert_eq!(result.premise.sweep_pointer_changes, 1, "清扫换指针一次");
        assert!(
            result.premise.first_version_violations.is_empty()
                && result.premise.adjacent_version_violations.is_empty()
                && result.premise.birth_after_the_txg_violations.is_empty(),
            "{:?}",
            result.premise
        );
    }

    /// 登记修订 3：sw-crash 里实例 4 的挂载盖掉实例 3 写行那一版起的几个根槽，那几版的记录还读得出、b 那一槽还住着实例 2 的 b，
    /// 装置按 ①–④ 认它们由记录施加出来；账本收被盖掉的版本那一支收下它们，S5 两边相同。
    /// 登记修订 4：实例 3 崩掉的那一版（根槽没落、被实例 4 的挂载施加）在环里最旧的有效根还比它老的那几张上装置认、账本收；
    /// 环转过去、最旧的有效根比它新之后（k = 9 起）装置按 ④ 不认，账本那一支也要过 ④、不收，S5 两边仍相同。
    #[test]
    fn records_applied_comparison_takes_in_the_covered_versions_whose_records_are_still_readable() {
        let result = run_the_history(&sweep_specification(
            SweepVariant::CrashBeforeTheNextInstance,
        ));
        assert_the_history_is_consistent(&result, 13);
        let mut recognised_with_a_root_that_landed = 0;
        let crashed_versions: Vec<VersionKey> = result
            .ledger
            .versions
            .iter()
            .filter(|(_, version)| version.applied_by_a_mount && !version.root_landed)
            .map(|(key, _)| *key)
            .collect();
        assert!(
            !crashed_versions.is_empty(),
            "sw-crash 里有崩掉、被下一个实例的挂载施加的版本"
        );
        let mut images_recognising_a_crashed_version = 0;
        let mut images_not_recognising_any_crashed_version = 0;
        for image in &result.images {
            assert_eq!(
                image.records_applied.mismatch_count(),
                0,
                "S5：{} 装置认的与账本推的相同：{:?}",
                image.label,
                image.records_applied
            );
            let reading = image.reading.as_ref().expect("读得出");
            if reading
                .applied_only_by_records
                .iter()
                .any(|version| crashed_versions.contains(&version.key))
            {
                images_recognising_a_crashed_version += 1;
            } else {
                images_not_recognising_any_crashed_version += 1;
            }
            recognised_with_a_root_that_landed += reading
                .applied_only_by_records
                .iter()
                .filter(|version| {
                    result
                        .ledger
                        .versions
                        .get(&version.key)
                        .is_some_and(|ledger_version| ledger_version.root_landed)
                })
                .count();
        }
        assert!(
            recognised_with_a_root_that_landed >= 1,
            "被环盖掉、根槽落过的版本里有装置按 ①–④ 认作由记录施加出来的"
        );
        assert!(
            images_recognising_a_crashed_version >= 1
                && images_not_recognising_any_crashed_version >= 1,
            "崩掉的那一版：环里最旧的有效根还比它老的张上认（{images_recognising_a_crashed_version} 张），环转过去之后不认（{images_not_recognising_any_crashed_version} 张）"
        );
    }

    #[test]
    fn sweep_read_history_makes_one_image_per_version_of_the_sweep_instance_from_the_sweep_on() {
        let result = run_the_history(&sweep_specification(
            SweepVariant::ReadFaultOnTheSweepInstance,
        ));
        let sweep = result.sweep.clone().expect("sw-read 有清扫读数");
        let sweep_txg = sweep.sweep_version.expect("实例 3 的挂载末尾发了清扫").txg;
        let expected = usize::try_from(sweep.last_txg_before_the_sweep_instance + 12 - sweep_txg)
            .expect("张数装得进 usize");
        assert_the_history_is_consistent(&result, expected);
        let with_pointer_birth_e1: u64 = result
            .images
            .iter()
            .map(|image| image.counts.with_pointer_birth_e1)
            .sum();
        assert!(
            with_pointer_birth_e1 >= 1,
            "PC-甲：R 是清扫那一版、它前面同实例的版本读坏，甲按字面认错（登记第四节第 3 条）"
        );
    }

    #[test]
    fn rollback_history_makes_one_image_per_first_readable_root() {
        let specification = HistorySpecification {
            group: Group::RollbackSlotBad,
            point: SamplingPoint::Main,
            slots_per_region: 4,
            journal_ring_bytes: SHORT_JOURNAL_RING_BYTES,
            shape: HistoryShape::RollbackSlotBad {
                target: 5,
                actions: vec![
                    RollbackAction::LongerSequentialWrite,
                    RollbackAction::EmptyPublish,
                ],
            },
        };
        let result = run_the_history(&specification);
        assert_the_history_is_consistent(&result, 3);
        for image in &result.images {
            assert!(
                image.checker_verdicts.is_some(),
                "rs1 每张都跑今天的池级 checker"
            );
        }
    }

    #[test]
    fn sweep_none_history_makes_one_image_per_step_after_the_next_mount() {
        let result = run_the_history(&sweep_specification(SweepVariant::NoCrash));
        assert_the_history_is_consistent(&result, 13);
        assert!(
            history_swept(&result),
            "实例 3 的挂载末尾发了清扫：{:?}",
            result.sweep
        );
    }

    /// G1：阈值 365 要攒到 366 行，实例 367 的挂载才扫（推的）；这一条要挂三百多次，慢，照样进单测（产物的 G1 那一段就是它）。
    #[test]
    fn sweep_at_the_product_default_threshold_waits_until_the_rows_exceed_it() {
        let result = run_the_history(&sweep_threshold_product_default_specification());
        assert_the_history_is_consistent(&result, 13);
        let sweep = result.sweep.clone().expect("G1 有清扫读数");
        assert!(
            sweep.sweep_version.is_some(),
            "攒满之后那次挂载发了清扫：{sweep:?}"
        );
        assert!(
            u64::from(sweep.sweep_instance) > SWEEP_THRESHOLD_PRODUCT_DEFAULT,
            "行数超过阈值才扫：清扫那个实例的代号 {} 要大于 365",
            sweep.sweep_instance
        );
    }

    fn synthetic_image(
        step: u64,
        counts: ArmCounts,
        truths: Vec<Truth>,
        checker_verdicts: Option<(&str, &str)>,
    ) -> ImageResult {
        let hole_pairs = truths
            .iter()
            .map(|_| HolePair {
                previous_txg: 4,
                previous_group: vec![key(1, 4)],
                version: ring_version(2, 6, Some(6)),
                bridged_today: false,
                bridged_with_the_pointer_birth: false,
            })
            .collect();
        ImageResult {
            label: format!("publishes_after_the_mount={step}"),
            step,
            reading: Ok(ImageReadingOfTheJudgements {
                newest: key(2, 6),
                rows: Vec::new(),
                effective_floor: 0,
                candidates: Vec::new(),
                applied_only_by_records: Vec::new(),
                hole_pairs,
            }),
            truths,
            counts,
            pointer_unreadable_pairs: 0,
            ring_slot_mismatches: Vec::new(),
            row_mismatch: None,
            records_applied: RecordsAppliedComparison::default(),
            checker_verdicts: checker_verdicts
                .map(|(release, reference)| (release.to_string(), reference.to_string())),
            corrupted_skipped: Vec::new(),
        }
    }

    fn unreadable_image(step: u64) -> ImageResult {
        let mut image = synthetic_image(step, ArmCounts::default(), Vec::new(), None);
        image.reading = Err("最新根的实例表读不出".to_string());
        image
    }

    fn synthetic_history(
        group: Group,
        name: &str,
        swept: bool,
        images: Vec<ImageResult>,
    ) -> HistoryResult {
        HistoryResult {
            specification: HistorySpecification {
                group,
                point: SamplingPoint::Main,
                slots_per_region: 4,
                journal_ring_bytes: SHORT_JOURNAL_RING_BYTES,
                shape: HistoryShape::SkippedRow { last: 6 },
            },
            name: name.to_string(),
            images,
            stopped: None,
            sweep: Some(SweepReading {
                sweep_instance: 3,
                outcome: "synthetic".to_string(),
                sweep_version: swept.then_some(key(3, 20)),
                last_txg_before_the_sweep_instance: 16,
                predicted_sweep_txg: 20,
                instances_mounted_before_the_sweep: 1,
            }),
            premise: PremiseCounts::default(),
            ledger: Ledger::default(),
        }
    }

    fn with_pointer_birth_e1(count: u64) -> ArmCounts {
        ArmCounts {
            today_e1: 0,
            today_e2: 0,
            with_pointer_birth_e1: count,
            with_pointer_birth_e2: 0,
        }
    }

    #[test]
    fn group_totals_and_question_readings_count_only_what_was_judged() {
        let swept = synthetic_history(
            Group::SweepCrash,
            "swept",
            true,
            vec![
                synthetic_image(
                    0,
                    with_pointer_birth_e1(1),
                    vec![Truth::ExistedVersionsInBetween],
                    None,
                ),
                synthetic_image(1, ArmCounts::default(), vec![Truth::DirectlyBridged], None),
            ],
        );
        let unswept = synthetic_history(
            Group::SweepCrash,
            "unswept",
            false,
            vec![
                synthetic_image(
                    0,
                    with_pointer_birth_e1(2),
                    vec![Truth::ExistedVersionsInBetween; 2],
                    None,
                ),
                unreadable_image(1),
            ],
        );
        let results = vec![swept, unswept];
        let every: Vec<&HistoryResult> = results.iter().collect();
        let totals = totals_of(&every, false);
        assert_eq!(
            (
                totals.histories,
                totals.produced_images,
                totals.judged_images,
                totals.unreadable_images,
                totals.hole_pairs
            ),
            (2, 4, 3, 1, 4),
            "读不出的那一张单列、不进对数"
        );
        assert_eq!(
            (
                totals.pairs.with_pointer_birth_e1,
                totals.images_with_a_positive_count.with_pointer_birth_e1
            ),
            (3, 2)
        );
        assert_eq!(
            truth_count(&totals.truth_counts, Truth::ExistedVersionsInBetween),
            3
        );
        let swept_only = totals_of(&every, true);
        assert_eq!(
            (
                swept_only.histories,
                swept_only.swept_histories,
                swept_only.sweep_txg_matches_the_prediction
            ),
            (1, 1, 1)
        );
        let reading = question_reading(
            &results,
            Group::SweepCrash,
            SamplingPoint::Main,
            true,
            ArmError::WithPointerBirthE1,
        );
        assert_eq!(
            reading,
            QuestionReading {
                pairs: 1,
                images: 1,
                hole_pairs: 2,
                judged_images: 2,
                unreadable_images: 0
            }
        );
        assert_eq!(misclassification_judgement(reading), "branch_two");
        assert_eq!(
            misclassification_judgement(QuestionReading::default()),
            "not_run"
        );
        assert_eq!(
            misclassification_judgement(QuestionReading {
                pairs: 0,
                images: 0,
                hole_pairs: 0,
                judged_images: 1,
                unreadable_images: 0,
            }),
            "no_object"
        );
        assert_eq!(
            misclassification_judgement(QuestionReading {
                pairs: 0,
                images: 0,
                hole_pairs: 2,
                judged_images: 1,
                unreadable_images: 0,
            }),
            "classified_right"
        );
        assert_eq!(
            trajectory_of(&results[0], ArmError::WithPointerBirthE1),
            (1, 1, 0),
            "峰值 1、为正 1 步、期末 0"
        );
        assert!(sensitivity_line(SamplingPoint::LongJournalRing, "Q4", 0, 0)
            .contains("record=two_points discriminating=both_zero"));
        assert!(sensitivity_line(SamplingPoint::LongJournalRing, "Q4", 0, 2)
            .contains("record=unstable discriminating=proven"));
        assert!(sensitivity_line(SamplingPoint::LongJournalRing, "Q4", 3, 3)
            .contains("record=two_points discriminating=equal_values"));
        let checked = synthetic_history(
            Group::NewestRowBad,
            "checked",
            false,
            vec![
                synthetic_image(
                    0,
                    ArmCounts::default(),
                    Vec::new(),
                    Some(("violated", "holds")),
                ),
                synthetic_image(
                    1,
                    ArmCounts::default(),
                    Vec::new(),
                    Some(("holds", "holds")),
                ),
            ],
        );
        assert_eq!(
            checker_violations(
                &[checked],
                Group::NewestRowBad,
                SamplingPoint::Main,
                CheckedInvariant::ReleaseGenerationInterval
            ),
            (1, "holds:1,violated:1".to_string())
        );
        let stopped = StopClauses {
            lines: Vec::new(),
            triggered: vec!["S5"],
        };
        let clear = StopClauses {
            lines: Vec::new(),
            triggered: Vec::new(),
        };
        assert_eq!(row_status(&stopped, false, true), "stopped_by_S5");
        assert_eq!(row_status(&clear, true, true), "void");
        assert_eq!(row_status(&clear, false, true), "enough_to_judge");
    }

    #[test]
    fn segment_two_clause_lines_follow_the_readings() {
        let results = vec![
            synthetic_history(
                Group::ShortRing,
                "ss",
                false,
                vec![synthetic_image(
                    0,
                    with_pointer_birth_e1(1),
                    vec![Truth::ExistedVersionsInBetween],
                    None,
                )],
            ),
            synthetic_history(
                Group::SweepCrash,
                "unswept",
                false,
                vec![synthetic_image(0, ArmCounts::default(), Vec::new(), None)],
            ),
        ];
        let clear = StopClauses {
            lines: Vec::new(),
            triggered: Vec::new(),
        };
        let lines = judgement_lines_of_segment_two(&[], &results, &clear);
        let has = |text: &str| lines.iter().any(|line| line.contains(text));
        assert!(has("name=verdict question=Q4 group=ss point=main scope=all_histories arm=with_pointer_birth error=E1 value_pairs=1"), "{lines:#?}");
        assert!(has("name=clause clause=F1 triggered=yes"), "Q4 有错分的对");
        assert!(
            has("name=clause clause=F6 triggered=yes"),
            "sw-crash 有一段没扫"
        );
        assert!(
            has("name=clause clause=Z4 triggered=yes"),
            "skr 一对直接接上的都没有"
        );
        assert!(has("name=clause clause=Z5 triggered=yes"), "C2 那一张不在");
        assert!(
            has("name=verdict row=1") && has("status=void"),
            "{lines:#?}"
        );
    }

    #[test]
    fn segment_one_clause_lines_follow_the_readings() {
        let counts = ArmCounts {
            today_e1: 1,
            today_e2: 0,
            with_pointer_birth_e1: 0,
            with_pointer_birth_e2: 0,
        };
        let results = vec![synthetic_history(
            Group::NewestRowBad,
            "nrb",
            false,
            vec![synthetic_image(
                0,
                counts,
                vec![Truth::ExistedVersionsInBetween],
                Some(("violated", "holds")),
            )],
        )];
        let clear = StopClauses {
            lines: Vec::new(),
            triggered: Vec::new(),
        };
        let lines = judgement_lines_of_segment_one(&[], &results, &clear);
        let has = |text: &str| lines.iter().any(|line| line.contains(text));
        assert!(has("name=verdict question=Q13 group=nrb point=main scope=all_histories arm=today error=E1 value_pairs=1 value_images=1"), "{lines:#?}");
        assert!(
            has("name=verdict question=Q17 group=nrb point=main value_images=1"),
            "{lines:#?}"
        );
        assert!(
            has("name=clause clause=F4 triggered=yes"),
            "I-3.9 判违例一张"
        );
        assert!(has("name=clause clause=F3 triggered=no"), "甲没有认错");
        assert!(
            has("name=clause clause=Z1 triggered=yes"),
            "C1 那一张不在，PC-H1A 不算中"
        );
        assert!(has("name=verdict row=2 half=release_generation_interval value_images=1 judgement=branch_two status=void"), "{lines:#?}");
        // 登记修订 5：读数里的键名是全词，不是题号缩写。
        assert!(
            has("clause=F3 triggered=no observation=nrb_with_pointer_birth_pairs=0_rs1_with_pointer_birth_pairs=0"),
            "{lines:#?}"
        );
        assert!(
            has("clause=F4 triggered=yes observation=rs1_release_generation_violated_images=")
                && has("_nrb_release_generation_violated_images=1"),
            "{lines:#?}"
        );
        assert!(
            has("clause=F7 triggered=yes observation=rs1_today_e1_images="),
            "{lines:#?}"
        );
        assert!(
            has("observation=instance_one_control_histories_missed="),
            "{lines:#?}"
        );
        assert!(
            has("name=verdict question=Q19 group=nrb point=main reference_interval_verdicts="),
            "{lines:#?}"
        );
    }

    /// 多线程与单线程的对拍（`.claude/rules/test-script-discipline.md`「编译与算法都要快」）：同两段历史用 1 个与 2 个线程各判一遍，
    /// 交回的次序与每一行产物逐字相同。
    #[test]
    fn histories_judged_with_one_or_two_threads_come_back_in_the_same_order_with_the_same_lines() {
        let specifications = vec![
            skipped_row_pin_specification(),
            newest_row_bad_pin_specification(),
        ];
        let lines_of = |results: &[HistoryResult]| -> Vec<String> {
            results
                .iter()
                .flat_map(|result| {
                    let mut lines = history_lines(result);
                    lines.extend(result.images.iter().map(|image| image_line(result, image)));
                    lines
                })
                .collect()
        };
        let one_thread = lines_of(&run_the_histories(&specifications, 1));
        let two_threads = lines_of(&run_the_histories(&specifications, 2));
        assert!(!one_thread.is_empty(), "两段历史都有产物行");
        assert_eq!(one_thread, two_threads, "线程数不改产物");
    }
}
