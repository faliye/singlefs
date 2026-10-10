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
//!   cargo run --release --bin e178-leaf-and-record-death-counts -- segment-four [--shard i/k] [--no-reuse]
//!                                                                                  第四段：登记修订 2 的两条新底座臂（B-stub、B-tsz）
//! 线程数取环境变量 E178_THREADS（默认 6），产物按工作项的次序输出，与线程数无关。
//! 第四段的分片、续跑与多机（登记修订 2 之十三）：997 个工作项，每项算完把产物行原子写进项目录（环境变量
//! E178_SEGMENT_FOUR_ITEMS，默认 ~/.local/share/singlefs/work/e178-segment-four/items）下的 `<项名>.out`，第一行是键（项名、装置源
//! sha256、几何参数）；同一条命令重起，键相同的项读回不重算。`--shard i/k` 只算序号 mod k = i 的项、只落项文件、不拼产物；
//! 不带 `--shard` 时补算缺的项，997 项齐了按项的次序拼出产物打到 stdout。`--no-reuse` 不读项目录（项文件照写）。
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

use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};

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
const RETIREMENT_RECORD_BYTES: u64 = 8 + 2 + 8 + 8 + 8;
const INODE_LEAF_RECORDS: u64 = (PACKED_CONTAINER_BYTES - PACKED_RECORDS_OFFSET) / INODE_RECORD_BYTES;
const TOMBSTONE_CONTAINER_RECORDS: u64 = (PACKED_CONTAINER_BYTES - PACKED_RECORDS_OFFSET) / TOMBSTONE_RECORD_BYTES;
const RETIREMENT_CONTAINER_RECORDS: u64 = (PACKED_CONTAINER_BYTES - PACKED_RECORDS_OFFSET) / RETIREMENT_RECORD_BYTES;
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
const F10_OVERWRITE_EVERY_UNLINKS: u64 = 10;
const F10_SEQUENTIAL_WRITE_EVERY_UNLINKS: u64 = 25;
const F10_SEQUENTIAL_WRITE_DATA_UNITS: u64 = 5;
const F11_INODE_COUNT: u64 = 300;
const F11_CHANGES_BEFORE_ROLLBACK: u64 = 10;
const F11_CHANGES_AFTER_ROLLBACK: u64 = 20;
const F12_INODE_COUNT: u64 = 300;
const F12_NEW_INODES_AFTER_RECOVERY: u64 = 5;
const DEFAULT_THREADS: usize = 6;

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

// ---------------------------------------------------------------------------------------------
// 臂（登记 5.3）与写路径格、读者
// ---------------------------------------------------------------------------------------------

/// X4 的两条臂：借时右片怎么处理身份。
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
enum BorrowArm {
    /// N1：右片保留身份，分隔 key 抬到借之后右片的最小 key。
    RaiseSeparator,
    /// N2：右片换身份（容器号 = 借之后的最小 key、出生代 = 本次 txg），旧身份写退役记录。
    RenewIdentity,
}

const BORROW_ARMS: [BorrowArm; 2] = [BorrowArm::RaiseSeparator, BorrowArm::RenewIdentity];

impl BorrowArm {
    const fn label(self) -> &'static str {
        match self {
            BorrowArm::RaiseSeparator => "N1",
            BorrowArm::RenewIdentity => "N2",
        }
    }
}

/// 一个写路径格：N × 叶容器容量 × 合并门槛 θ（登记 5.1 几何旋钮、5.2 末段）。
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct WritePathCell {
    borrow_arm: BorrowArm,
    leaf_capacity: u64,
    merge_threshold: u64,
}

impl WritePathCell {
    fn label(self) -> String {
        format!(
            "borrow_arm={} capacity={} theta={}",
            self.borrow_arm.label(),
            self.leaf_capacity,
            self.merge_threshold
        )
    }
}

const LEAF_CAPACITIES: [u64; 2] = [INODE_LEAF_RECORDS, TEST_LEAF_CAPACITY];

/// θ ∈ {0, ⌊容量 ÷ 4⌋, ⌊容量 ÷ 2⌋}（登记 5.1）。
const fn merge_thresholds_of(leaf_capacity: u64) -> [u64; 3] {
    [0, leaf_capacity / 4, leaf_capacity / 2]
}

/// 底座三臂（登记 5.3）。
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
enum BaseArm {
    /// B-on：退役记录在谱系上才施行。
    RetirementOnLineage,
    /// B-off-ret：读者不认退役记录（今天的 `crates/` 不写它）。
    WithoutRetirement,
    /// B-off-lin：读者施行退役记录时不看谱系。
    RetirementIgnoringLineage,
    /// B-stub（修订 2）：消失的身份落一个零条目版本、一直留在叶序里；读者不丢版本，择中零条目版本的身份不出记录。
    ZeroRecordStubs,
    /// B-stub-lin-off（只给 PC-Bs2）：同 B-stub，读者对类型 2 版本不看「诞生 txg 在父链上」。
    ZeroRecordStubsIgnoringLineage,
    /// B-tsz（修订 2）：盘上零新增；读者不丢版本，记录按改动计数供出（P-C，平局按写序），整对象墓碑照 E-I。
    TombstonesOnly,
}

impl BaseArm {
    const fn label(self) -> &'static str {
        match self {
            BaseArm::RetirementOnLineage => "B-on",
            BaseArm::WithoutRetirement => "B-off-ret",
            BaseArm::RetirementIgnoringLineage => "B-off-lin",
            BaseArm::ZeroRecordStubs => "B-stub",
            BaseArm::ZeroRecordStubsIgnoringLineage => "B-stub-lin-off",
            BaseArm::TombstonesOnly => "B-tsz",
        }
    }
}

/// 同一个 inode 号在几个被择中的类型 2 版本里都有记录时由哪一条供出（登记 5.4 第 4 条；修订 2 问法二）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Provenance {
    /// P-W：所在版本写序最大的那一条。
    NewestWriteOrder,
    /// P-C：记录偏移 88 的改动计数最大的那一条，平局按 P-W。
    LargestChangeCounter,
    /// PC-Bt2 的倒置：所在版本写序最小的那一条。
    OldestWriteOrder,
}

/// 读者用哪条供出规则：B-tsz 取 P-C（修订 2 问法二），别的底座照原登记取 P-W。
const fn provenance_of(base: BaseArm) -> Provenance {
    match base {
        BaseArm::TombstonesOnly => Provenance::LargestChangeCounter,
        BaseArm::RetirementOnLineage
        | BaseArm::WithoutRetirement
        | BaseArm::RetirementIgnoringLineage
        | BaseArm::ZeroRecordStubs
        | BaseArm::ZeroRecordStubsIgnoringLineage => Provenance::NewestWriteOrder,
    }
}

/// X2 的三条臂（两个候选与阳性对照 G0）。
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
enum ShrinkArm {
    /// Z：按重建出的 inode 记录的对象出生代与 size 折成的单元数判死活。
    RecordSize,
    /// T：变短那次往墓碑树写区间墓碑。
    RangeTombstone,
    /// G0：只按对象门控、不看 size（阳性对照）。
    ObjectGateOnly,
}

impl ShrinkArm {
    const fn label(self) -> &'static str {
        match self {
            ShrinkArm::RecordSize => "Z",
            ShrinkArm::RangeTombstone => "T",
            ShrinkArm::ObjectGateOnly => "G0",
        }
    }
}

/// X3 的三条臂：unlink 的整对象墓碑从哪一次发布起生效。
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
enum UnlinkEffectArm {
    /// E-I：意图那次写、那次起生效（今天的 `crates/`）。
    AtIntent,
    /// E-L-w：最后一批那次写。
    AtLastBatchWritten,
    /// E-L-r：意图那次写，读者只在树里已经没有那条记录时才施行。
    AtLastBatchRead,
}

impl UnlinkEffectArm {
    const fn label(self) -> &'static str {
        match self {
            UnlinkEffectArm::AtIntent => "E-I",
            UnlinkEffectArm::AtLastBatchWritten => "E-L-w",
            UnlinkEffectArm::AtLastBatchRead => "E-L-r",
        }
    }
}

const UNLINK_EFFECT_ARMS: [UnlinkEffectArm; 3] =
    [UnlinkEffectArm::AtIntent, UnlinkEffectArm::AtLastBatchWritten, UnlinkEffectArm::AtLastBatchRead];

/// 旋钮 W：换下的单元在之后几次发布的扫描里还读得出（登记 5.1）。
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
enum ReadableWindow {
    Forever,
    RingWindow,
}

const READABLE_WINDOWS: [ReadableWindow; 2] = [ReadableWindow::Forever, ReadableWindow::RingWindow];

impl ReadableWindow {
    const fn label(self) -> &'static str {
        match self {
            ReadableWindow::Forever => "inf",
            ReadableWindow::RingWindow => "23",
        }
    }

    /// 在发布序号 p_r 那次被换下的单元，在发布序号 p 的扫描里读得出当且仅当 p − p_r ≤ W。
    const fn allows(self, publishes_since_replaced: u64) -> bool {
        match self {
            ReadableWindow::Forever => true,
            ReadableWindow::RingWindow => publishes_since_replaced <= READABLE_WINDOW_RING_PUBLISHES,
        }
    }
}

/// 墓碑写流：X2 的 Z（含 G0，都不写变短记录）/ T 与 X3 的三条臂配出来的五条（墓碑只随这两个臂变，别的臂共用一条写路径）。
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
enum TombstoneStream {
    /// Z × E-I、Z × E-L-r：意图那次写整对象墓碑（今天的 `crates/`）。
    WholeAtIntent,
    /// Z × E-L-w：最后一批那次写整对象墓碑。
    WholeAtFinal,
    /// T × E-I：意图那次写整对象墓碑；用户改动变短那次写区间墓碑。
    WholeAtIntentRangesOnUserShrinks,
    /// T × E-L-r：意图那次写整对象墓碑；用户改动变短与不是最后一批的每一批都写区间墓碑。
    WholeAtIntentRangesOnEveryShrink,
    /// T × E-L-w：最后一批那次写整对象墓碑；用户改动变短与不是最后一批的每一批都写区间墓碑。
    WholeAtFinalRangesOnEveryShrink,
}

const TOMBSTONE_STREAM_COUNT: usize = 5;

const TOMBSTONE_STREAMS: [TombstoneStream; TOMBSTONE_STREAM_COUNT] = [
    TombstoneStream::WholeAtIntent,
    TombstoneStream::WholeAtFinal,
    TombstoneStream::WholeAtIntentRangesOnUserShrinks,
    TombstoneStream::WholeAtIntentRangesOnEveryShrink,
    TombstoneStream::WholeAtFinalRangesOnEveryShrink,
];

impl TombstoneStream {
    const fn position(self) -> usize {
        match self {
            TombstoneStream::WholeAtIntent => 0,
            TombstoneStream::WholeAtFinal => 1,
            TombstoneStream::WholeAtIntentRangesOnUserShrinks => 2,
            TombstoneStream::WholeAtIntentRangesOnEveryShrink => 3,
            TombstoneStream::WholeAtFinalRangesOnEveryShrink => 4,
        }
    }

    const fn writes_the_whole_object_tombstone_at_the_intent(self) -> bool {
        match self {
            TombstoneStream::WholeAtIntent
            | TombstoneStream::WholeAtIntentRangesOnUserShrinks
            | TombstoneStream::WholeAtIntentRangesOnEveryShrink => true,
            TombstoneStream::WholeAtFinal | TombstoneStream::WholeAtFinalRangesOnEveryShrink => false,
        }
    }

    const fn writes_ranges_when_a_user_change_shrinks(self) -> bool {
        match self {
            TombstoneStream::WholeAtIntent | TombstoneStream::WholeAtFinal => false,
            TombstoneStream::WholeAtIntentRangesOnUserShrinks
            | TombstoneStream::WholeAtIntentRangesOnEveryShrink
            | TombstoneStream::WholeAtFinalRangesOnEveryShrink => true,
        }
    }

    /// E-L 两臂下不是最后一批的每一批也算变短（登记 5.3 的 T 那一行）。
    const fn writes_ranges_at_every_batch_that_is_not_the_last(self) -> bool {
        match self {
            TombstoneStream::WholeAtIntent
            | TombstoneStream::WholeAtFinal
            | TombstoneStream::WholeAtIntentRangesOnUserShrinks => false,
            TombstoneStream::WholeAtIntentRangesOnEveryShrink | TombstoneStream::WholeAtFinalRangesOnEveryShrink => true,
        }
    }
}

const fn tombstone_stream_of(shrink: ShrinkArm, effect: UnlinkEffectArm) -> TombstoneStream {
    match (shrink, effect) {
        (ShrinkArm::RecordSize | ShrinkArm::ObjectGateOnly, UnlinkEffectArm::AtIntent | UnlinkEffectArm::AtLastBatchRead) => {
            TombstoneStream::WholeAtIntent
        }
        (ShrinkArm::RecordSize | ShrinkArm::ObjectGateOnly, UnlinkEffectArm::AtLastBatchWritten) => TombstoneStream::WholeAtFinal,
        (ShrinkArm::RangeTombstone, UnlinkEffectArm::AtIntent) => TombstoneStream::WholeAtIntentRangesOnUserShrinks,
        (ShrinkArm::RangeTombstone, UnlinkEffectArm::AtLastBatchRead) => TombstoneStream::WholeAtIntentRangesOnEveryShrink,
        (ShrinkArm::RangeTombstone, UnlinkEffectArm::AtLastBatchWritten) => TombstoneStream::WholeAtFinalRangesOnEveryShrink,
    }
}

/// 一个读者 = 底座 × X2 × X3 × W（登记 5.4 末段）。
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct Reader {
    base: BaseArm,
    shrink: ShrinkArm,
    effect: UnlinkEffectArm,
    window: ReadableWindow,
}

impl Reader {
    fn label(self) -> String {
        format!(
            "base={} shrink={} effect={} window={}",
            self.base.label(),
            self.shrink.label(),
            self.effect.label(),
            self.window.label()
        )
    }

    const fn tombstone_stream(self) -> TombstoneStream {
        tombstone_stream_of(self.shrink, self.effect)
    }

    /// 主格：B-on × {Z, T} × X3 三臂 × W 两档。
    fn is_a_main_cell(self) -> bool {
        self.base == BaseArm::RetirementOnLineage && self.shrink != ShrinkArm::ObjectGateOnly
    }
}

/// 每次扫描都读的 20 个读者：主格 12 个，对照格 8 个（登记 5.4 末段），次序固定。
fn every_reader() -> Vec<Reader> {
    let mut readers = Vec::new();
    for effect in UNLINK_EFFECT_ARMS {
        for shrink in [ShrinkArm::RecordSize, ShrinkArm::RangeTombstone] {
            for window in READABLE_WINDOWS {
                readers.push(Reader { base: BaseArm::RetirementOnLineage, shrink, effect, window });
            }
        }
    }
    for base in [BaseArm::WithoutRetirement, BaseArm::RetirementIgnoringLineage] {
        for window in READABLE_WINDOWS {
            readers.push(Reader { base, shrink: ShrinkArm::RecordSize, effect: UnlinkEffectArm::AtIntent, window });
        }
    }
    for effect in [UnlinkEffectArm::AtIntent, UnlinkEffectArm::AtLastBatchWritten] {
        for window in READABLE_WINDOWS {
            readers.push(Reader { base: BaseArm::RetirementOnLineage, shrink: ShrinkArm::ObjectGateOnly, effect, window });
        }
    }
    readers
}

/// 第四段原写路径上的 18 个读者（修订 2 之五）：B-on 12 个（与第二段同、次序同）、B-tsz × {Z, T} × E-I × W 4 个、B-off-ret × Z × E-I × W 2 个。
fn segment_four_original_path_readers() -> Vec<Reader> {
    let mut readers: Vec<Reader> = every_reader().into_iter().filter(|reader| reader.is_a_main_cell()).collect();
    for shrink in [ShrinkArm::RecordSize, ShrinkArm::RangeTombstone] {
        for window in READABLE_WINDOWS {
            readers.push(Reader { base: BaseArm::TombstonesOnly, shrink, effect: UnlinkEffectArm::AtIntent, window });
        }
    }
    for window in READABLE_WINDOWS {
        readers.push(Reader { base: BaseArm::WithoutRetirement, shrink: ShrinkArm::RecordSize, effect: UnlinkEffectArm::AtIntent, window });
    }
    readers
}

/// 第四段 B-stub 写路径上的 12 个读者：B-stub × {Z, T} × {E-I, E-L-w, E-L-r} × W，次序与 B-on 那 12 个逐个对应（配对算差用）。
fn segment_four_stub_path_readers() -> Vec<Reader> {
    every_reader()
        .into_iter()
        .filter(|reader| reader.is_a_main_cell())
        .map(|reader| Reader { base: BaseArm::ZeroRecordStubs, shrink: reader.shrink, effect: reader.effect, window: reader.window })
        .collect()
}

// ---------------------------------------------------------------------------------------------
// 树的形态与三个谓词（登记 5.6；第十一节 V7 在每个发布出去的版本上判同一组谓词）
// ---------------------------------------------------------------------------------------------

#[derive(Clone, Debug, PartialEq, Eq)]
struct ShapeLeaf {
    container_number: u64,
    record_keys: Vec<u64>,
}

impl ShapeLeaf {
    fn largest_key(&self) -> Option<u64> {
        self.record_keys.iter().copied().max()
    }

    fn smallest_key(&self) -> Option<u64> {
        self.record_keys.iter().copied().min()
    }
}

/// 一串叶容器加根里每条条目的分隔 key（分隔 key 与叶一一对应）。
#[derive(Clone, Debug, PartialEq, Eq)]
struct TreeShape {
    leaves: Vec<ShapeLeaf>,
    separators: Vec<u64>,
}

/// I-9.13：每片类型 2 容器记录数 ≥ 1。
fn every_leaf_holds_a_record(shape: &TreeShape) -> bool {
    shape.leaves.iter().all(|leaf| !leaf.record_keys.is_empty())
}

/// I-9.4 的第一、二、四句（今天的写法与 N1 的写法共用）：记录号 ≥ 容器号；沿叶序容器号严格递增；同一片里记录号沿记录序严格递增。
fn container_number_sentences_shared_by_both_readings_hold(shape: &TreeShape) -> bool {
    let records_not_below_the_container_number = shape
        .leaves
        .iter()
        .all(|leaf| leaf.record_keys.iter().all(|key| *key >= leaf.container_number));
    let numbers_increase_along_the_leaf_order =
        shape.leaves.windows(2).all(|adjacent| adjacent[0].container_number < adjacent[1].container_number);
    let records_increase_inside_each_leaf = shape
        .leaves
        .iter()
        .all(|leaf| leaf.record_keys.windows(2).all(|adjacent_keys| adjacent_keys[0] < adjacent_keys[1]));
    records_not_below_the_container_number && numbers_increase_along_the_leaf_order && records_increase_inside_each_leaf
}

/// 今天的 I-9.4（四句）：第三句是「左容器的最大 key < 右容器号」（`crates/singlefs-checker/src/walk.rs:2768-2789`、`:3067`、`:3079`）。
fn container_numbers_hold_today(shape: &TreeShape) -> bool {
    if !container_number_sentences_shared_by_both_readings_hold(shape) {
        return false;
    }
    shape.leaves.windows(2).all(|adjacent| {
        let right_leaf = &adjacent[1];
        match adjacent[0].largest_key() {
            Some(left_largest_key) => left_largest_key < right_leaf.container_number,
            None => true,
        }
    })
}

/// N1 的 I-9.4：第三句换成「左容器的最大 key < 右容器的分隔 key」，其余三句不动（登记 5.3 的 N1 那一行）。
fn container_numbers_hold_with_raised_separators(shape: &TreeShape) -> bool {
    if !container_number_sentences_shared_by_both_readings_hold(shape) {
        return false;
    }
    shape
        .leaves
        .windows(2)
        .zip(shape.separators.iter().skip(1))
        .all(|(adjacent, right_separator)| match adjacent[0].largest_key() {
            Some(left_largest_key) => left_largest_key < *right_separator,
            None => true,
        })
}

/// I-9.12：分隔 key 严格递增；第 i 条 ≤ 第 i 个孩子的最小 key，且 > 第 i−1 个孩子的最大 key（`walk.rs:3245-3266`）。
fn separators_fall_outside_the_child_ranges(shape: &TreeShape) -> bool {
    shape.separators.iter().enumerate().all(|(position, separator)| {
        let not_above_its_child = shape.leaves[position].smallest_key().map_or(true, |smallest| *separator <= smallest);
        if position == 0 {
            return not_above_its_child;
        }
        let above_the_previous_separator = *separator > shape.separators[position - 1];
        let above_the_previous_child = shape.leaves[position - 1].largest_key().map_or(true, |largest| *separator > largest);
        not_above_its_child && above_the_previous_separator && above_the_previous_child
    })
}

/// 路由对不对：对形态里每个 key 取「最后一个分隔 key ≤ 目标的条目」，落到的是不是装它的那一片。
fn routing_finds_every_key(shape: &TreeShape) -> bool {
    shape.leaves.iter().enumerate().all(|(leaf_position, leaf)| {
        leaf.record_keys.iter().all(|key| {
            let routed = shape.separators.iter().rposition(|separator| *separator <= *key);
            routed == Some(leaf_position)
        })
    })
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct PredicateVerdicts {
    today_container_numbers_hold: bool,
    raised_separator_container_numbers_hold: bool,
    separators_hold: bool,
    routing_holds: bool,
}

fn judge_the_shape(shape: &TreeShape) -> PredicateVerdicts {
    PredicateVerdicts {
        today_container_numbers_hold: container_numbers_hold_today(shape),
        raised_separator_container_numbers_hold: container_numbers_hold_with_raised_separators(shape),
        separators_hold: separators_fall_outside_the_child_ranges(shape),
        routing_holds: routing_finds_every_key(shape),
    }
}

const fn red_or_green(holds: bool) -> &'static str {
    if holds {
        "green"
    } else {
        "red"
    }
}

/// 5.6 的一个形态。
#[derive(Clone, Debug)]
struct NamedShape {
    family: &'static str,
    name: String,
    shape: TreeShape,
}

fn leaf_of(container_number: u64, record_keys: &[u64]) -> ShapeLeaf {
    ShapeLeaf { container_number, record_keys: record_keys.to_vec() }
}

fn two_leaves(left: ShapeLeaf, right: ShapeLeaf, separators: [u64; 2]) -> TreeShape {
    TreeShape { leaves: vec![left, right], separators: separators.to_vec() }
}

/// 登记 5.6 的全表：K（现有，逐形抄 `crates/` 的测试）、S（分裂时号给错）、BN1、BN2（借之后）、DV（方向错）。
fn catalogue_of_shapes() -> Vec<NamedShape> {
    let mut shapes = Vec::new();
    let named = |family: &'static str, name: String, shape: TreeShape| NamedShape { family, name, shape };
    // K：`checker_narrow_invariants_and_abandoned_roots.rs:858`（两叶、分隔 key 15）、`checker_known_bad_images.rs:1064-1070`（单叶号 5）、
    // `:10599`（同号两条）、`pool_checker_judges_the_deletion_state_of_every_walked_version.rs:1363`（倒序）。单叶那几形的根条目分隔 key 照旧是 1。
    shapes.push(named("K", "K-0".to_string(), two_leaves(leaf_of(1, &[1, 10]), leaf_of(11, &[20, 30]), [1, 15])));
    shapes.push(named("K", "K-1".to_string(), TreeShape { leaves: vec![leaf_of(5, &[1])], separators: vec![1] }));
    shapes.push(named("K", "K-2".to_string(), two_leaves(leaf_of(1, &[1, 10]), leaf_of(8, &[20, 30]), [1, 15])));
    shapes.push(named("K", "K-3".to_string(), two_leaves(leaf_of(1, &[1, 10]), leaf_of(0, &[20, 30]), [1, 15])));
    shapes.push(named("K", "K-4".to_string(), TreeShape { leaves: vec![leaf_of(1, &[1, 1])], separators: vec![1] }));
    shapes.push(named("K", "K-5".to_string(), TreeShape { leaves: vec![leaf_of(1, &[1, 3, 2])], separators: vec![1] }));
    // S：左 {1, 10} 号 1、分隔 key 1；右 {20, 30} 正确号 20；错号 m 两种分隔 key（= m、= 20）；m = 20 是合法对照。
    for wrong_number in [0, 1, 2, 9, 10, 11, 19, 20, 21, 30, 31] {
        for (separator_name, separator) in [("separator_equals_number", wrong_number), ("separator_twenty", 20)] {
            shapes.push(named(
                "S",
                format!("S-m{wrong_number}-{separator_name}"),
                two_leaves(leaf_of(1, &[1, 10]), leaf_of(wrong_number, &[20, 30]), [1, separator]),
            ));
        }
    }
    // BN1：左 {1, 10, 20} 号 1；右 {30} 号 20；分隔 key s；再在 s = 30 下换右号 m。
    for separator in [20, 25, 30, 31] {
        shapes.push(named(
            "BN1",
            format!("BN1-s{separator}"),
            two_leaves(leaf_of(1, &[1, 10, 20]), leaf_of(20, &[30]), [1, separator]),
        ));
    }
    for right_number in [0, 1, 10, 21, 31] {
        shapes.push(named(
            "BN1",
            format!("BN1-s30-m{right_number}"),
            two_leaves(leaf_of(1, &[1, 10, 20]), leaf_of(right_number, &[30]), [1, 30]),
        ));
    }
    // BN2：左 {1, 10, 20} 号 1；右 {30} 号 = 分隔 key = m。
    for right_number in [20, 25, 30, 31] {
        shapes.push(named(
            "BN2",
            format!("BN2-m{right_number}"),
            two_leaves(leaf_of(1, &[1, 10, 20]), leaf_of(right_number, &[30]), [1, right_number]),
        ));
    }
    // DV：右从左借（分隔 key 20 与 10）、右吸收左（单片，保留右身份与它的分隔 key）。
    for separator in [20, 10] {
        shapes.push(named(
            "DV",
            format!("DV-right-borrows-from-left-s{separator}"),
            two_leaves(leaf_of(1, &[1]), leaf_of(20, &[10, 20, 30]), [1, separator]),
        ));
    }
    shapes.push(named(
        "DV",
        "DV-right-absorbs-left".to_string(),
        TreeShape { leaves: vec![leaf_of(20, &[1, 10, 20, 30])], separators: vec![20] },
    ));
    shapes
}

/// 键布局的第二个取样点（第八节 G-K）：key 乘 100；夹在两个 key 之间的号与分隔 key 取两个 key 乘 100 之后的中点；0 不变。
fn scaled_by_one_hundred(value: u64, sorted_keys: &[u64]) -> u64 {
    if value == 0 || sorted_keys.contains(&value) {
        return value * 100;
    }
    let below = sorted_keys.iter().copied().filter(|key| *key < value).max();
    let above = sorted_keys.iter().copied().filter(|key| *key > value).min();
    match (below, above) {
        (Some(lower_key), Some(upper_key)) => (lower_key * 100 + upper_key * 100) / 2,
        (None, Some(_)) | (Some(_), None) | (None, None) => value * 100,
    }
}

fn sparse_layout_of(shape: &TreeShape) -> TreeShape {
    let mut keys: Vec<u64> = shape.leaves.iter().flat_map(|leaf| leaf.record_keys.iter().copied()).collect();
    keys.sort_unstable();
    keys.dedup();
    TreeShape {
        leaves: shape
            .leaves
            .iter()
            .map(|leaf| ShapeLeaf {
                container_number: scaled_by_one_hundred(leaf.container_number, &keys),
                record_keys: leaf.record_keys.iter().map(|key| key * 100).collect(),
            })
            .collect(),
        separators: shape.separators.iter().map(|separator| scaled_by_one_hundred(*separator, &keys)).collect(),
    }
}

// ---------------------------------------------------------------------------------------------
// 单元、记录与盘（登记 5.1）
// ---------------------------------------------------------------------------------------------

type UnitIndex = usize;

/// 码 3 容器的身份：(出生树, 打包记录类型, 容器号, 容器出生代)（登记第一节读法 1 的容器 key）。
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct ContainerIdentity {
    birth_tree: u64,
    record_type: u64,
    number: u64,
    birth: u64,
}

impl ContainerIdentity {
    const fn inode_leaf(number: u64, birth: u64) -> Self {
        Self { birth_tree: TREE_IDENTIFIER_INODE, record_type: PACKED_TYPE_INODE, number, birth }
    }
}

/// 写序 = (诞生 txg, 实例代号, 事务号)；同组择新取最大（登记 5.1）。
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct WriteOrder {
    birth_txg: u64,
    instance: u64,
    transaction: u64,
}

#[derive(Clone, Copy, PartialEq, Eq)]
struct InodeRecord {
    inode: u64,
    object_birth: u64,
    nlink: u64,
    size_bytes: u64,
    /// 记录偏移 88 的改动计数：最后一次改动所在发布的 txg（读法 11；只有 B-tsz 的 P-C 读它）。
    change_counter: u64,
}

/// 改动计数不进 Debug：第一、二段产物里的停机对拍行按加这个字段之前的四个字段打，复跑要逐字节相同。
impl std::fmt::Debug for InodeRecord {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("InodeRecord")
            .field("inode", &self.inode)
            .field("object_birth", &self.object_birth)
            .field("nlink", &self.nlink)
            .field("size_bytes", &self.size_bytes)
            .finish()
    }
}

impl InodeRecord {
    /// ⌈size ÷ 32634⌉：读法 Z 拿它当这个对象的单元数。
    const fn data_units(self) -> u64 {
        self.size_bytes.div_ceil(DATA_UNIT_PAYLOAD_BYTES)
    }
}

/// 墓碑记录 (对象, 对象出生代, 区间起, 区间止 = 2⁶⁴ − 1, 死亡 txg)；整对象的区间起是 0。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct TombstoneRecord {
    object: u64,
    object_birth: u64,
    range_start_bytes: u64,
    death_txg: u64,
}

/// 退役记录 (四元组, 死亡 txg)。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct RetirementRecord {
    identity: ContainerIdentity,
    death_txg: u64,
}

#[derive(Clone, Debug)]
enum UnitBody {
    /// 码 1 数据单元：类身份 (出生树, 对象 inode 号, 对象出生代, 单元序号 k)。
    Data { object: u64, object_birth: u64, unit_index_in_file: u64 },
    /// 码 3、打包记录类型 2：inode 叶容器。
    InodeLeaf { identity: ContainerIdentity, records: Arc<Vec<InodeRecord>> },
    /// 码 3、打包记录类型 1：墓碑容器，各墓碑写流各有一棵树。
    Tombstones { stream: TombstoneStream, identity: ContainerIdentity, records: Arc<Vec<TombstoneRecord>> },
    /// 码 3、打包记录类型 3：退役容器。
    Retirements { identity: ContainerIdentity, records: Arc<Vec<RetirementRecord>> },
}

/// 择新的组：码 1 按 (对象, 对象出生代, 单元序号)，码 3 按身份（墓碑另带写流）。
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
enum GroupKey {
    Data { object: u64, object_birth: u64, unit_index_in_file: u64 },
    InodeLeaf(ContainerIdentity),
    Tombstones(TombstoneStream, ContainerIdentity),
    Retirements(ContainerIdentity),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
enum GroupCategory {
    Data,
    InodeLeaf,
    Tombstones(TombstoneStream),
    Retirements,
}

const fn category_of(key: GroupKey) -> GroupCategory {
    match key {
        GroupKey::Data { .. } => GroupCategory::Data,
        GroupKey::InodeLeaf(_) => GroupCategory::InodeLeaf,
        GroupKey::Tombstones(stream, _) => GroupCategory::Tombstones(stream),
        GroupKey::Retirements(_) => GroupCategory::Retirements,
    }
}

impl UnitBody {
    const fn group_key(&self) -> GroupKey {
        match self {
            UnitBody::Data { object, object_birth, unit_index_in_file } => GroupKey::Data {
                object: *object,
                object_birth: *object_birth,
                unit_index_in_file: *unit_index_in_file,
            },
            UnitBody::InodeLeaf { identity, .. } => GroupKey::InodeLeaf(*identity),
            UnitBody::Tombstones { stream, identity, .. } => GroupKey::Tombstones(*stream, *identity),
            UnitBody::Retirements { identity, .. } => GroupKey::Retirements(*identity),
        }
    }
}

#[derive(Clone, Debug)]
struct Unit {
    write_order: WriteOrder,
    /// 写它的那次发布（含崩溃时没发布成的那一次）的发布序号。
    written_at_publish: u64,
    /// 被换下的那次发布的序号；现行那一版还引用它时是 None（回退把 R_old 的单元重新算作被引用）。
    replaced_at_publish: Option<u64>,
    body: UnitBody,
}

/// 数据单元的 key (对象, 对象出生代, 单元序号)；别的单元是 None。
fn data_key_of(unit: &Unit) -> Option<(u64, u64, u64)> {
    match &unit.body {
        UnitBody::Data { object, object_birth, unit_index_in_file } => Some((*object, *object_birth, *unit_index_in_file)),
        UnitBody::InodeLeaf { .. } | UnitBody::Tombstones { .. } | UnitBody::Retirements { .. } => None,
    }
}

/// 盘：写过的每个单元（只增），与按组的版本清单（按写入次序，也就是写序次序）。
#[derive(Clone, Debug, Default)]
struct Disk {
    units: Vec<Unit>,
    group_of_key: HashMap<GroupKey, usize>,
    versions_of_group: Vec<Vec<UnitIndex>>,
    groups_by_category: BTreeMap<GroupCategory, Vec<usize>>,
}

impl Disk {
    fn write(&mut self, unit: Unit) -> UnitIndex {
        let unit_index = self.units.len();
        let key = unit.body.group_key();
        let group = if let Some(existing) = self.group_of_key.get(&key) {
            *existing
        } else {
            let created = self.versions_of_group.len();
            self.group_of_key.insert(key, created);
            self.versions_of_group.push(Vec::new());
            self.groups_by_category.entry(category_of(key)).or_default().push(created);
            created
        };
        self.versions_of_group[group].push(unit_index);
        self.units.push(unit);
        unit_index
    }
}

// ---------------------------------------------------------------------------------------------
// 现行那一版（模型的真值；读者不读它，只拿它当计数时的 T）
// ---------------------------------------------------------------------------------------------

#[derive(Clone, Debug)]
struct Leaf {
    identity: ContainerIdentity,
    separator: u64,
    records: Arc<Vec<InodeRecord>>,
    unit: UnitIndex,
}

#[derive(Clone, Debug)]
struct FileUnits {
    object_birth: u64,
    /// 按单元序号。
    units: Vec<UnitIndex>,
}

#[derive(Clone, Debug)]
struct PackedContainer<Record> {
    identity: ContainerIdentity,
    records: Arc<Vec<Record>>,
    unit: UnitIndex,
}

/// 墓碑树、退役树：开放容器有新记录那次才重写、满了才关、容器号从 1 起（D18 已定项 24 第 1、3 条）。
#[derive(Clone, Debug)]
struct PackedTree<Record> {
    containers: Vec<PackedContainer<Record>>,
    next_container_number: u64,
}

impl<Record> PackedTree<Record> {
    const fn empty() -> Self {
        Self { containers: Vec::new(), next_container_number: 1 }
    }

    fn record_count(&self) -> u64 {
        self.containers.iter().map(|container| length_as_u64(container.records.len())).sum()
    }
}

#[derive(Clone, Debug)]
struct Version {
    txg: u64,
    instance: u64,
    transaction: u64,
    next_inode: u64,
    leaves: Vec<Leaf>,
    files: BTreeMap<u64, FileUnits>,
    deleted_queue: Option<u64>,
    tombstone_trees: Vec<PackedTree<TombstoneRecord>>,
    retirement_tree: PackedTree<RetirementRecord>,
    /// 谱系记录 (T_rb, T_old)。
    lineage: Vec<(u64, u64)>,
    /// 意图已发布的对象（O2 的「p 里它的意图已发布」）。
    objects_with_published_intent: BTreeSet<u64>,
}

impl Version {
    fn genesis() -> Self {
        Self {
            txg: NEW_POOL_FILE_CREATION_TXG - 1,
            instance: MAKE_FILESYSTEM_PROCESS_INSTANCE,
            transaction: 0,
            next_inode: FIRST_FILE_INODE,
            leaves: Vec::new(),
            files: BTreeMap::new(),
            deleted_queue: None,
            tombstone_trees: TOMBSTONE_STREAMS.iter().map(|_| PackedTree::empty()).collect(),
            retirement_tree: PackedTree::empty(),
            lineage: Vec::new(),
            objects_with_published_intent: BTreeSet::new(),
        }
    }

    fn record_of(&self, inode: u64) -> Option<InodeRecord> {
        self.leaves.iter().find_map(|leaf| {
            leaf.records
                .binary_search_by_key(&inode, |record| record.inode)
                .ok()
                .map(|position| leaf.records[position])
        })
    }

    fn is_live(&self, inode: u64) -> bool {
        self.record_of(inode).is_some_and(|record| record.nlink > 0)
    }

    /// 活 inode（nlink > 0），按叶序（= 号升序）。
    fn live_inodes(&self) -> Vec<u64> {
        self.leaves
            .iter()
            .flat_map(|leaf| leaf.records.iter())
            .filter(|record| record.nlink > 0)
            .map(|record| record.inode)
            .collect()
    }

    fn data_units_of(&self, inode: u64) -> u64 {
        self.files.get(&inode).map_or(0, |file| length_as_u64(file.units.len()))
    }

    /// 「有树、文件集合为空」（D8 已定项 16 第 5 条；E177 装置 `is_with_trees_and_no_file`）。B-stub 写路径的桩不算记录。
    fn is_with_trees_and_no_file(&self) -> bool {
        self.leaves.iter().all(|leaf| leaf.records.is_empty()) && self.files.is_empty() && self.deleted_queue.is_none()
    }

    /// B-stub 写路径留在叶序里的零条目版本个数（原写路径恒 0）。
    fn stub_count(&self) -> u64 {
        length_as_u64(self.leaves.iter().filter(|leaf| leaf.records.is_empty()).count())
    }

    /// 数据单元的摘要：每个对象的出生代与单元数（第四段 V-Bs1 比两条写路径的真值）。
    fn extent_digest(&self) -> u64 {
        let mut digest = DIGEST_START;
        for (object, file) in &self.files {
            for value in [*object, file.object_birth, length_as_u64(file.units.len())] {
                digest = fold_into_digest(digest, value);
            }
        }
        digest
    }

    /// 这一版里有 nlink = 0 的记录：在某个对象的 unlink 途中（登记第一节读法 4）。
    fn is_inside_an_unlink(&self) -> bool {
        self.leaves.iter().any(|leaf| leaf.records.iter().any(|record| record.nlink == 0))
    }

    /// 这一版引用的全部单元（升序、不重复）。
    fn referenced_units(&self) -> Vec<UnitIndex> {
        let mut units: Vec<UnitIndex> = self.leaves.iter().map(|leaf| leaf.unit).collect();
        units.extend(self.files.values().flat_map(|file| file.units.iter().copied()));
        for tree in &self.tombstone_trees {
            units.extend(tree.containers.iter().map(|container| container.unit));
        }
        units.extend(self.retirement_tree.containers.iter().map(|container| container.unit));
        units.sort_unstable();
        units.dedup();
        units
    }

    fn shape(&self) -> TreeShape {
        TreeShape {
            leaves: self
                .leaves
                .iter()
                .map(|leaf| ShapeLeaf {
                    container_number: leaf.identity.number,
                    record_keys: leaf.records.iter().map(|record| record.inode).collect(),
                })
                .collect(),
            separators: self.leaves.iter().map(|leaf| leaf.separator).collect(),
        }
    }

    /// 活叶容器的摘要：身份、分隔 key、每条记录（A5 比两条写流逐发布相同；第四段 V-Bs1 比 B-stub 与原写路径的活叶）。
    /// 桩（记录数 0）不进摘要，原写路径上没有桩，这一句对它不改任何值。
    fn leaf_digest(&self) -> u64 {
        let mut digest = DIGEST_START;
        for leaf in self.leaves.iter().filter(|leaf| !leaf.records.is_empty()) {
            for value in [leaf.identity.birth_tree, leaf.identity.number, leaf.identity.birth, leaf.separator] {
                digest = fold_into_digest(digest, value);
            }
            for record in leaf.records.iter() {
                for value in [record.inode, record.object_birth, record.nlink, record.size_bytes] {
                    digest = fold_into_digest(digest, value);
                }
            }
        }
        digest
    }
}

// ---------------------------------------------------------------------------------------------
// 一次发布写哪些单元（登记 5.1「一次发布写哪些单元」、5.3 的臂、合并策略 M(θ)）
// ---------------------------------------------------------------------------------------------

/// 一次发布写的单元数，每份计（两盘镜像同内容）。码 2 只数 inode 树根、墓碑树根、退役树根。
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
struct UnitsWritten {
    data_units: u64,
    inode_leaf_containers: u64,
    inode_roots: u64,
    retirement_containers: u64,
    retirement_roots: u64,
    retirement_records: u64,
    tombstone_containers: [u64; TOMBSTONE_STREAM_COUNT],
    tombstone_roots: [u64; TOMBSTONE_STREAM_COUNT],
}

impl UnitsWritten {
    /// 码 3：叶容器 + 退役容器 + 今天那条墓碑写流的墓碑容器（读法 8）。
    const fn code_three_counted(&self) -> u64 {
        self.inode_leaf_containers
            + self.retirement_containers
            + self.tombstone_containers[TombstoneStream::WholeAtIntent.position()]
    }

    /// 码 2：inode 树根 + 退役树根 + 今天那条墓碑写流的墓碑树根。
    const fn code_two_counted(&self) -> u64 {
        self.inode_roots + self.retirement_roots + self.tombstone_roots[TombstoneStream::WholeAtIntent.position()]
    }

    const fn every_counted_unit(&self) -> u64 {
        self.data_units + self.code_three_counted() + self.code_two_counted()
    }
}

/// 一次删记录之后合并策略做了什么。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum MergeAction {
    NoMerge,
    EmptiedAndDropped,
    AbsorbedTheRightNeighbour,
    BorrowedFromTheRightNeighbour { records_moved: u64 },
    AbsorbedIntoTheLeftNeighbour,
}

#[derive(Clone, Debug)]
enum Operation {
    FirstFile,
    RowPublish,
    WarmUp,
    /// 覆盖写、顺序写：对象的内容整份换成 `data_units` 个新单元（登记第一节读法 10）。
    WriteFile { object: u64, data_units: u64 },
    NewInodes { count: u64 },
    UnlinkIntent { object: u64 },
    UnlinkBatch { object: u64, data_units_kept: u64 },
    UnlinkFinal { object: u64 },
    /// 管理员回退：一次向前发布，inode 树、文件、删除队列取 R_old 的，墓碑树、退役树、谱系树随现行那一版。
    RollBack { target: Box<Version> },
}

/// 变短：用户改动把对象的单元数改小（覆盖写、顺序写）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct ShrinkEvent {
    object: u64,
    size_before_bytes: u64,
}

#[derive(Clone, Debug)]
struct WorkingLeaf {
    identity: ContainerIdentity,
    separator: u64,
    records: Arc<Vec<InodeRecord>>,
    /// 照抄上一版的单元；None 是这次重写。
    carried_unit: Option<UnitIndex>,
}

/// 根装不下那么多片叶容器（`crates/` 的 `MoreLeafContainersThanOneRootNodeHolds`），那一格记「装不下」。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct LeafContainersOverflowTheRoot;

/// 容器身份消失（删空、被吸收、N2 换下）那次写什么：原写路径追加退役记录，B-stub 写路径原位落一个零条目版本（修订 2 之五）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ContainerDeathWritePath {
    RetirementRecords,
    ZeroRecordStubs,
}

/// 合并策略找邻片时只看活叶：桩不被借、不被吸收、不算邻片（修订 2 之五；原写路径上没有桩，这一句不改它的任何一步）。
fn is_a_merge_neighbour(leaf: &WorkingLeaf) -> bool {
    !leaf.records.is_empty()
}

struct PublishWork {
    container_death: ContainerDeathWritePath,
    /// B-stub 写路径这次原位落了桩的身份（原写路径恒空）。
    stubbed_identities: Vec<ContainerIdentity>,
    first_new_unit: UnitIndex,
    write_order: WriteOrder,
    publish_sequence: u64,
    new_units: Vec<Unit>,
    leaves: Vec<WorkingLeaf>,
    leaf_set_changed: bool,
    inode_root_rewritten_anyway: bool,
    retired_identities: Vec<ContainerIdentity>,
    moved_objects: Vec<u64>,
    tombstone_appends: Vec<(TombstoneStream, TombstoneRecord)>,
    merge: MergeAction,
    split_checks: Vec<bool>,
    written: UnitsWritten,
    shrink: Option<ShrinkEvent>,
}

fn working_leaves_of(leaves: &[Leaf]) -> Vec<WorkingLeaf> {
    leaves
        .iter()
        .map(|leaf| WorkingLeaf {
            identity: leaf.identity,
            separator: leaf.separator,
            records: Arc::clone(&leaf.records),
            carried_unit: Some(leaf.unit),
        })
        .collect()
}

impl PublishWork {
    fn reserve(&mut self, body: UnitBody) -> UnitIndex {
        self.new_units.push(Unit {
            write_order: self.write_order,
            written_at_publish: self.publish_sequence,
            replaced_at_publish: None,
            body,
        });
        self.first_new_unit + self.new_units.len() - 1
    }

    /// 最右那片活叶（读法 10；原写路径上没有桩，就是最后一片）。
    fn position_of_the_rightmost_live_leaf(&self) -> Option<usize> {
        self.leaves.iter().rposition(|leaf| !leaf.records.is_empty())
    }

    fn largest_key_in_the_tree(&self) -> Option<u64> {
        self.position_of_the_rightmost_live_leaf()
            .and_then(|position| self.leaves[position].records.last())
            .map(|record| record.inode)
    }

    fn merge_neighbour_to_the_right_of(&self, position: usize) -> Option<usize> {
        (position + 1..self.leaves.len()).find(|candidate| is_a_merge_neighbour(&self.leaves[*candidate]))
    }

    fn merge_neighbour_to_the_left_of(&self, position: usize) -> Option<usize> {
        (0..position).rev().find(|candidate| is_a_merge_neighbour(&self.leaves[*candidate]))
    }

    /// B-stub：消失的身份原位换成一个零条目版本（这次写、之后不再重写），右边各片不动（修订 2 之五）。
    fn leave_a_stub_in_place(&mut self, position: usize) {
        let leaf = &mut self.leaves[position];
        Arc::make_mut(&mut leaf.records).clear();
        leaf.carried_unit = None;
        self.leaf_set_changed = true;
        self.stubbed_identities.push(leaf.identity);
    }

    fn position_of_the_leaf_holding(&self, inode: u64) -> Option<usize> {
        self.leaves
            .iter()
            .position(|leaf| leaf.records.binary_search_by_key(&inode, |record| record.inode).is_ok())
    }

    fn update_record(&mut self, inode: u64, update: impl FnOnce(&mut InodeRecord)) {
        let position = self.position_of_the_leaf_holding(inode).expect("改的记录在树里");
        let leaf = &mut self.leaves[position];
        let records = Arc::make_mut(&mut leaf.records);
        let index_in_leaf = records
            .binary_search_by_key(&inode, |record| record.inode)
            .expect("position_of_the_leaf_holding 找到的那一片装着它");
        update(&mut records[index_in_leaf]);
        leaf.carried_unit = None;
    }

    /// 新号只落最右叶；最右那片满了在末尾分裂：右半容器号 = 触发分裂那条记录的 inode 号、出生代 = 本次 txg，左半记录与身份不变
    /// （D8 已定项 6；`crates/singlefs-core/src/inode_tree.rs:181-283`）。第七节 K6 在每次分裂上判。
    fn insert_new_records(&mut self, records: &[InodeRecord], leaf_capacity: u64) {
        let txg = self.write_order.birth_txg;
        for record in records {
            let lands_beyond_every_key = self.largest_key_in_the_tree().map_or(true, |largest| record.inode > largest);
            assert!(lands_beyond_every_key, "inode 号单调：新号大于树里每一个 key");
            let rightmost_live = self.position_of_the_rightmost_live_leaf();
            let rightmost_has_room =
                rightmost_live.is_some_and(|position| length_as_u64(self.leaves[position].records.len()) < leaf_capacity);
            if rightmost_has_room {
                let rightmost = &mut self.leaves[rightmost_live.expect("上一行看过有最右那片活叶")];
                Arc::make_mut(&mut rightmost.records).push(*record);
                rightmost.carried_unit = None;
                continue;
            }
            let left_before_the_split =
                rightmost_live.map(|position| (position, self.leaves[position].identity, Arc::clone(&self.leaves[position].records)));
            self.leaves.push(WorkingLeaf {
                identity: ContainerIdentity::inode_leaf(record.inode, txg),
                separator: record.inode,
                records: Arc::new(vec![*record]),
                carried_unit: None,
            });
            self.leaf_set_changed = true;
            let right_half = self.leaves.last().expect("刚推进去");
            let right_half_holds = right_half.identity.number == record.inode && right_half.identity.birth == txg;
            let left_half_holds = match left_before_the_split {
                Some((left_position, identity, left_records)) => {
                    let left_half = &self.leaves[left_position];
                    left_half.identity == identity && left_half.records == left_records
                }
                None => true,
            };
            self.split_checks.push(right_half_holds && left_half_holds);
        }
    }

    /// 左移的每一片这次全部重写（`inode_tree.rs:286-318` 的位置规则）。
    fn mark_shifted_leaves_rewritten(&mut self, first_shifted_position: usize) {
        self.leaf_set_changed = true;
        for leaf in self.leaves.iter_mut().skip(first_shifted_position) {
            leaf.carried_unit = None;
        }
    }

    /// 左吸收右：左片保留身份与分隔 key，右片的记录全部挪进左片。原写路径右片不再出现、它右边各片左移重写，交回它的身份去写
    /// 退役记录；B-stub 写路径右片原位换成桩、不左移，不交回身份。
    fn absorb_right_into_left(&mut self, left_position: usize, right_position: usize) -> Option<ContainerIdentity> {
        let moving: Vec<InodeRecord> = std::mem::take(Arc::make_mut(&mut self.leaves[right_position].records));
        self.moved_objects.extend(moving.iter().map(|record| record.inode));
        {
            let left = &mut self.leaves[left_position];
            Arc::make_mut(&mut left.records).extend(moving);
            left.carried_unit = None;
        }
        self.leaf_set_changed = true;
        match self.container_death {
            ContainerDeathWritePath::RetirementRecords => {
                let right = self.leaves.remove(right_position);
                let retired_identity = right.identity;
                self.mark_shifted_leaves_rewritten(right_position);
                Some(retired_identity)
            }
            ContainerDeathWritePath::ZeroRecordStubs => {
                self.leave_a_stub_in_place(right_position);
                None
            }
        }
    }

    /// 左从右借：右片最小的 `records_moved` 条挪进左片，两片都留着、都重写；右片身份按 N1 / N2 处理。
    fn borrow_from_the_right(&mut self, left_position: usize, right_position: usize, records_moved: u64, borrow_arm: BorrowArm) {
        let txg = self.write_order.birth_txg;
        let moving: Vec<InodeRecord> = {
            let right_records = Arc::make_mut(&mut self.leaves[right_position].records);
            right_records.drain(..u64_as_index(records_moved)).collect()
        };
        self.moved_objects.extend(moving.iter().map(|record| record.inode));
        {
            let left = &mut self.leaves[left_position];
            Arc::make_mut(&mut left.records).extend(moving);
            left.carried_unit = None;
        }
        let smallest_key_after_borrow =
            self.leaves[right_position].records.first().expect("借之后右片还有记录").inode;
        let right_leaf = &mut self.leaves[right_position];
        right_leaf.carried_unit = None;
        match borrow_arm {
            BorrowArm::RaiseSeparator => {
                right_leaf.separator = smallest_key_after_borrow;
            }
            BorrowArm::RenewIdentity => {
                let replaced_identity = right_leaf.identity;
                let renewed_identity = ContainerIdentity {
                    birth_tree: replaced_identity.birth_tree,
                    record_type: replaced_identity.record_type,
                    number: smallest_key_after_borrow,
                    birth: txg,
                };
                right_leaf.identity = renewed_identity;
                right_leaf.separator = renewed_identity.number;
                self.retired_identities.push(replaced_identity);
            }
        }
    }

    /// 删一条记录，再按 M(θ) 合并（登记 5.1）：删空照今天不落盘；|C| < θ 时有右邻就吸收或借，C 是最右一片就让左邻吸收。
    fn remove_record_and_merge(&mut self, inode: u64, cell: WritePathCell) -> MergeAction {
        let position = self.position_of_the_leaf_holding(inode).expect("删的记录在树里");
        {
            let leaf = &mut self.leaves[position];
            let records = Arc::make_mut(&mut leaf.records);
            let index_in_leaf = records
                .binary_search_by_key(&inode, |record| record.inode)
                .expect("position_of_the_leaf_holding 找到的那一片装着它");
            records.remove(index_in_leaf);
            leaf.carried_unit = None;
        }
        if self.leaves[position].records.is_empty() {
            match self.container_death {
                ContainerDeathWritePath::RetirementRecords => {
                    let emptied_leaf = self.leaves.remove(position);
                    self.retired_identities.push(emptied_leaf.identity);
                    self.mark_shifted_leaves_rewritten(position);
                }
                ContainerDeathWritePath::ZeroRecordStubs => self.leave_a_stub_in_place(position),
            }
            return MergeAction::EmptiedAndDropped;
        }
        let size_of_this_leaf = length_as_u64(self.leaves[position].records.len());
        if cell.merge_threshold == 0 || size_of_this_leaf >= cell.merge_threshold {
            return MergeAction::NoMerge;
        }
        if let Some(right_position) = self.merge_neighbour_to_the_right_of(position) {
            let size_of_the_right_leaf = length_as_u64(self.leaves[right_position].records.len());
            if size_of_this_leaf + size_of_the_right_leaf <= cell.leaf_capacity {
                if let Some(retired_identity) = self.absorb_right_into_left(position, right_position) {
                    self.retired_identities.push(retired_identity);
                }
                return MergeAction::AbsorbedTheRightNeighbour;
            }
            let records_moved = (size_of_the_right_leaf - size_of_this_leaf).div_ceil(2);
            self.borrow_from_the_right(position, right_position, records_moved, cell.borrow_arm);
            return MergeAction::BorrowedFromTheRightNeighbour { records_moved };
        }
        if let Some(left_position) = self.merge_neighbour_to_the_left_of(position) {
            let size_of_the_left_leaf = length_as_u64(self.leaves[left_position].records.len());
            if size_of_the_left_leaf + size_of_this_leaf <= cell.leaf_capacity {
                if let Some(retired_identity) = self.absorb_right_into_left(left_position, position) {
                    self.retired_identities.push(retired_identity);
                }
                return MergeAction::AbsorbedIntoTheLeftNeighbour;
            }
        }
        MergeAction::NoMerge
    }

    /// 叶容器落单元：重写的那几片各写一个新单元；有片重写、片数变了或回退时，inode 树根重写一次。
    fn finish_leaves(&mut self) -> Vec<Leaf> {
        let leaves = std::mem::take(&mut self.leaves);
        let mut finished = Vec::with_capacity(leaves.len());
        let mut any_leaf_rewritten = false;
        for leaf in leaves {
            let unit = if let Some(carried) = leaf.carried_unit {
                carried
            } else {
                any_leaf_rewritten = true;
                self.written.inode_leaf_containers += 1;
                self.reserve(UnitBody::InodeLeaf { identity: leaf.identity, records: Arc::clone(&leaf.records) })
            };
            finished.push(Leaf { identity: leaf.identity, separator: leaf.separator, records: leaf.records, unit });
        }
        if any_leaf_rewritten || self.leaf_set_changed || self.inode_root_rewritten_anyway {
            self.written.inode_roots += 1;
        }
        finished
    }

    /// 往墓碑树或退役树追加：开放容器有新记录那次重写，满了开新容器（号 = 下一个号、出生代 = 本次 txg）。交回这次写了几个容器。
    fn append_into_packed_tree<Record: Clone>(
        &mut self,
        tree: &mut PackedTree<Record>,
        appended: &[Record],
        container_capacity: u64,
        identity_template: (u64, u64),
        body_of: &dyn Fn(ContainerIdentity, Arc<Vec<Record>>) -> UnitBody,
    ) -> u64 {
        let (birth_tree, record_type) = identity_template;
        let mut touched_positions: BTreeSet<usize> = BTreeSet::new();
        for record in appended {
            let needs_a_new_container = tree
                .containers
                .last()
                .map_or(true, |last| length_as_u64(last.records.len()) >= container_capacity);
            if needs_a_new_container {
                let identity = ContainerIdentity {
                    birth_tree,
                    record_type,
                    number: tree.next_container_number,
                    birth: self.write_order.birth_txg,
                };
                tree.next_container_number += 1;
                tree.containers.push(PackedContainer { identity, records: Arc::new(Vec::new()), unit: UnitIndex::MAX });
            }
            let position = tree.containers.len() - 1;
            Arc::make_mut(&mut tree.containers[position].records).push(record.clone());
            touched_positions.insert(position);
        }
        for position in &touched_positions {
            let identity = tree.containers[*position].identity;
            let records = Arc::clone(&tree.containers[*position].records);
            tree.containers[*position].unit = self.reserve(body_of(identity, records));
        }
        length_as_u64(touched_positions.len())
    }
}

/// 一次发布算好、还没落盘的样子。
#[derive(Clone, Debug)]
struct PreparedPublish {
    next: Version,
    new_units: Vec<Unit>,
    written: UnitsWritten,
    merge: MergeAction,
    retired_identities: Vec<ContainerIdentity>,
    stubbed_identities: Vec<ContainerIdentity>,
    moved_objects: Vec<u64>,
    split_checks: Vec<bool>,
    shrink: Option<ShrinkEvent>,
}

/// 逐发布留的账：N1 / N2 配对（Q-X4-1、Q-X4-2、A5）、缺口事件（F11）、X2 的多写（Q-X2-6、Q-X2-7）、第四段的配对（V-Bs1）与桩的账用。
#[derive(Clone, Debug, PartialEq, Eq)]
struct PublishRecord {
    publish_sequence: u64,
    txg: u64,
    merge: MergeAction,
    retired_identities: Vec<ContainerIdentity>,
    written: UnitsWritten,
    shrink: Option<ShrinkEvent>,
    moved_object_shrunk: bool,
    leaf_digest: u64,
    is_a_rollback: bool,
    extent_digest: u64,
    stubs_written: u64,
    /// 这一版的叶片总数（活叶 + 桩）与桩数（Q-Bs-5；原写路径桩数恒 0）。
    leaves_in_the_leaf_order: u64,
    stubs_in_the_leaf_order: u64,
    /// 这一版里至少有一个记录数为 0 的类型 2 容器（Q-Bs-6）。
    holds_an_empty_inode_leaf: bool,
    /// 到这一版做完（最后一批已发布）的 unlink 数（Q-Bs-5）。
    unlinks_completed_so_far: u64,
}

/// 每条断言判了几次、违反几次（第七节、第十一节）。
#[derive(Clone, Debug, Default, PartialEq, Eq)]
struct AssertionTally {
    judged: BTreeMap<&'static str, u64>,
    violated: BTreeMap<&'static str, u64>,
}

impl AssertionTally {
    fn judge(&mut self, name: &'static str, holds: bool) {
        *self.judged.entry(name).or_insert(0) += 1;
        if !holds {
            *self.violated.entry(name).or_insert(0) += 1;
        }
    }

    fn merge(&mut self, other: &AssertionTally) {
        for (name, count) in &other.judged {
            *self.judged.entry(name).or_insert(0) += count;
        }
        for (name, count) in &other.violated {
            *self.violated.entry(name).or_insert(0) += count;
        }
    }

    fn violations_of(&self, name: &str) -> u64 {
        self.violated.get(name).copied().unwrap_or(0)
    }

    fn judged_of(&self, name: &str) -> u64 {
        self.judged.get(name).copied().unwrap_or(0)
    }
}

/// 第十一节 V7：每个发布出去的版本上判 I-9.13、今天的 I-9.4、N1 的 I-9.4、I-9.12，记红的版本数。
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
struct SelfCheckTally {
    versions: u64,
    leaves_hold_a_record_red: u64,
    today_container_numbers_red: u64,
    raised_separator_container_numbers_red: u64,
    separators_red: u64,
}

impl SelfCheckTally {
    fn note(&mut self, shape: &TreeShape) {
        let verdicts = judge_the_shape(shape);
        self.versions += 1;
        self.leaves_hold_a_record_red += u64::from(!every_leaf_holds_a_record(shape));
        self.today_container_numbers_red += u64::from(!verdicts.today_container_numbers_hold);
        self.raised_separator_container_numbers_red += u64::from(!verdicts.raised_separator_container_numbers_hold);
        self.separators_red += u64::from(!verdicts.separators_hold);
    }

    fn add(&mut self, other: SelfCheckTally) {
        self.versions += other.versions;
        self.leaves_hold_a_record_red += other.leaves_hold_a_record_red;
        self.today_container_numbers_red += other.today_container_numbers_red;
        self.raised_separator_container_numbers_red += other.raised_separator_container_numbers_red;
        self.separators_red += other.separators_red;
    }
}

// ---------------------------------------------------------------------------------------------
// 模拟：写路径（一个写路径格一条），可选每次发布之后扫一次
// ---------------------------------------------------------------------------------------------

/// 一次用户改动（E177 装置 `UserChange` 的推广：写文件可以写任一对象）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum UserChange {
    WriteFile { object: u64, data_units: u64 },
    NewInodes { count: u64 },
    Unlink { object: u64 },
}

/// F11 的缺口事件（登记 5.2）。
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
enum GapEvent {
    Absorb,
    Borrow,
    Drop,
    MovedObjectShortened,
}

const GAP_EVENTS: [GapEvent; 4] = [GapEvent::Absorb, GapEvent::Borrow, GapEvent::Drop, GapEvent::MovedObjectShortened];

impl GapEvent {
    const fn label(self) -> &'static str {
        match self {
            GapEvent::Absorb => "absorb",
            GapEvent::Borrow => "borrow",
            GapEvent::Drop => "drop",
            GapEvent::MovedObjectShortened => "moved_object_shortened",
        }
    }

    const fn happened_in(self, merge: MergeAction, moved_object_shrunk: bool) -> bool {
        match self {
            GapEvent::Absorb => {
                matches!(merge, MergeAction::AbsorbedTheRightNeighbour | MergeAction::AbsorbedIntoTheLeftNeighbour)
            }
            GapEvent::Borrow => matches!(merge, MergeAction::BorrowedFromTheRightNeighbour { .. }),
            GapEvent::Drop => matches!(merge, MergeAction::EmptiedAndDropped),
            GapEvent::MovedObjectShortened => moved_object_shrunk,
        }
    }
}

/// 第一次出现缺口事件的那次发布之前那一版（R_old）。
#[derive(Clone, Debug)]
struct GapStart {
    version_before: Version,
    event_publish_sequence: u64,
}

#[derive(Clone, Debug)]
struct Simulation {
    cell: WritePathCell,
    /// 容器身份消失那次写退役记录（原写路径）还是落桩（B-stub 写路径）。
    container_death: ContainerDeathWritePath,
    /// 每次扫描读哪几个读者（第二、三段是 `every_reader` 的 20 个，第四段两条写路径各一张表）。
    readers: Vec<Reader>,
    batch_data_units: u64,
    disk: Disk,
    version: Version,
    /// 跨崩溃单调的发布序号（崩溃那一次没发布成的写也占一个），只给旋钮 W 用。
    publish_sequence: u64,
    /// 实例表：旧实例 → 已施加的最大事务号。
    instance_rows: BTreeMap<u64, u64>,
    publishes: u64,
    refused_steps: u64,
    /// 记录被借走或被吸收过的对象（历史上的事实，回退不清）。
    moved_objects: BTreeSet<u64>,
    /// 被挪过、之后变短过的对象（PC-Bt2-mid 的取样，登记修订 3）。
    moved_then_shortened_objects: BTreeSet<u64>,
    /// 变短过的对象 → 变短之前最大的 size（PC-Z）。
    largest_size_before_a_shrink: BTreeMap<u64, u64>,
    moved_then_shortened: u64,
    moved_then_unlinked: u64,
    publish_records: Vec<PublishRecord>,
    user_changes: Vec<UserChange>,
    self_check: SelfCheckTally,
    assertions: AssertionTally,
    does_not_fit_at: Option<u64>,
    scanning: bool,
    scans: Vec<ScanRecord>,
    gap_watch: Option<GapEvent>,
    gap_start: Option<GapStart>,
    /// 意图那次的 txg，按对象（PC-ELr 植入的版本要早于它）。
    intent_txgs: BTreeMap<u64, u64>,
    /// 第二遍跑时在哪几次扫描上做阳性对照（第一遍的扫描记录定）。
    positive_control_plan: PositiveControlPlan,
    positive_control_outcomes: Vec<PositiveControlOutcome>,
    /// 做完（最后一批已发布）的 unlink 数。
    unlinks_completed: u64,
    /// 最后一批让一个身份消失、而那个身份最后那一版里装着这个对象 nlink = 0 的记录（PC-Bt1 的取样）。
    dead_versions_holding_a_finished_record: Vec<DeadVersionHoldingAFinishedRecord>,
    /// 第四段的探针与对照（第二、三段是 None）。
    segment_four: Option<SegmentFourProbes>,
}

impl Simulation {
    /// 新池新建文件（txg 3，mkfs 那个进程的实例 1）、可写挂载的写行（txg 4）与暖机（txg 5）（登记 5.2，E177 装置第 3331–3336 行）。
    fn start(cell: WritePathCell, batch_data_units: u64, scanning: bool) -> Self {
        Self::start_with(cell, batch_data_units, scanning, ContainerDeathWritePath::RetirementRecords, every_reader())
    }

    fn start_with(
        cell: WritePathCell,
        batch_data_units: u64,
        scanning: bool,
        container_death: ContainerDeathWritePath,
        readers: Vec<Reader>,
    ) -> Self {
        assert!(
            container_death == ContainerDeathWritePath::RetirementRecords || cell.borrow_arm == BorrowArm::RaiseSeparator,
            "B-stub 只配 N1（修订 2 问法一 N2-none）"
        );
        let mut simulation = Self {
            cell,
            container_death,
            readers,
            batch_data_units,
            disk: Disk::default(),
            version: Version::genesis(),
            publish_sequence: 0,
            instance_rows: BTreeMap::new(),
            publishes: 0,
            refused_steps: 0,
            moved_objects: BTreeSet::new(),
            moved_then_shortened_objects: BTreeSet::new(),
            largest_size_before_a_shrink: BTreeMap::new(),
            moved_then_shortened: 0,
            moved_then_unlinked: 0,
            publish_records: Vec::new(),
            user_changes: Vec::new(),
            self_check: SelfCheckTally::default(),
            assertions: AssertionTally::default(),
            does_not_fit_at: None,
            scanning,
            scans: Vec::new(),
            gap_watch: None,
            gap_start: None,
            intent_txgs: BTreeMap::new(),
            positive_control_plan: PositiveControlPlan::default(),
            positive_control_outcomes: Vec::new(),
            unlinks_completed: 0,
            dead_versions_holding_a_finished_record: Vec::new(),
            segment_four: None,
        };
        simulation.publish(&Operation::FirstFile);
        simulation.switch_to_a_new_instance();
        assert_eq!(simulation.version.instance, MOUNT_INSTANCE, "可写挂载取实例 2");
        simulation.publish(&Operation::RowPublish);
        simulation.publish(&Operation::WarmUp);
        simulation
    }

    const fn has_stopped(&self) -> bool {
        self.does_not_fit_at.is_some()
    }

    /// 换实例代号：实例表给旧实例写一行 (实例, 已施加的最大事务号)，新实例的事务号从 1 起。
    fn switch_to_a_new_instance(&mut self) {
        self.instance_rows.insert(self.version.instance, self.version.transaction);
        self.version.instance += 1;
        self.version.transaction = 0;
    }

    fn publish(&mut self, operation: &Operation) -> bool {
        if self.has_stopped() {
            return false;
        }
        match self.prepare(operation) {
            Ok(prepared) => {
                self.adopt(prepared, matches!(operation, Operation::RollBack { .. }));
                true
            }
            Err(LeafContainersOverflowTheRoot) => {
                self.does_not_fit_at = Some(self.version.txg + 1);
                false
            }
        }
    }

    /// 算下一版与这次要写的单元（不落盘）。一次发布一个事务号（D8 已定项 16 第 2 条）。
    fn prepare(&self, operation: &Operation) -> Result<PreparedPublish, LeafContainersOverflowTheRoot> {
        let mut next = self.version.clone();
        next.txg = self.version.txg + 1;
        next.transaction = self.version.transaction + 1;
        let txg = next.txg;
        let mut work = PublishWork {
            container_death: self.container_death,
            stubbed_identities: Vec::new(),
            first_new_unit: self.disk.units.len(),
            write_order: WriteOrder { birth_txg: txg, instance: next.instance, transaction: next.transaction },
            publish_sequence: self.publish_sequence + 1,
            new_units: Vec::new(),
            leaves: working_leaves_of(&next.leaves),
            leaf_set_changed: false,
            inode_root_rewritten_anyway: false,
            retired_identities: Vec::new(),
            moved_objects: Vec::new(),
            tombstone_appends: Vec::new(),
            merge: MergeAction::NoMerge,
            split_checks: Vec::new(),
            written: UnitsWritten::default(),
            shrink: None,
        };
        let leaf_capacity = self.cell.leaf_capacity;
        match operation {
            Operation::FirstFile => {
                let data_unit = work.reserve(UnitBody::Data { object: FIRST_FILE_INODE, object_birth: txg, unit_index_in_file: 0 });
                work.written.data_units += 1;
                next.files.insert(FIRST_FILE_INODE, FileUnits { object_birth: txg, units: vec![data_unit] });
                let record = InodeRecord {
                    inode: FIRST_FILE_INODE,
                    object_birth: txg,
                    nlink: 1,
                    size_bytes: NEW_POOL_FILE_SIZE_BYTES,
                    change_counter: txg,
                };
                work.insert_new_records(&[record], leaf_capacity);
                next.next_inode = FIRST_FILE_INODE + 1;
            }
            Operation::RowPublish | Operation::WarmUp => {}
            Operation::WriteFile { object, data_units } => {
                let record = next.record_of(*object).expect("写文件内容的对象有一条活记录");
                let data_units_before = next.data_units_of(*object);
                let units: Vec<UnitIndex> = (0..*data_units)
                    .map(|unit_index_in_file| {
                        work.reserve(UnitBody::Data { object: *object, object_birth: record.object_birth, unit_index_in_file })
                    })
                    .collect();
                work.written.data_units += *data_units;
                next.files.insert(*object, FileUnits { object_birth: record.object_birth, units });
                let size_after = *data_units * DATA_UNIT_PAYLOAD_BYTES;
                work.update_record(*object, |updated| {
                    updated.size_bytes = size_after;
                    updated.change_counter = txg;
                });
                if *data_units < data_units_before {
                    work.shrink = Some(ShrinkEvent { object: *object, size_before_bytes: record.size_bytes });
                    for stream in TOMBSTONE_STREAMS {
                        if stream.writes_ranges_when_a_user_change_shrinks() {
                            let range = TombstoneRecord {
                                object: *object,
                                object_birth: record.object_birth,
                                range_start_bytes: size_after,
                                death_txg: txg,
                            };
                            work.tombstone_appends.push((stream, range));
                        }
                    }
                }
            }
            Operation::NewInodes { count } => {
                let records: Vec<InodeRecord> = (0..*count)
                    .map(|offset| InodeRecord { inode: next.next_inode + offset, object_birth: txg, nlink: 1, size_bytes: 0, change_counter: txg })
                    .collect();
                work.insert_new_records(&records, leaf_capacity);
                next.next_inode += *count;
            }
            Operation::UnlinkIntent { object } => {
                let record = next.record_of(*object).expect("意图删的 inode 有一条记录");
                assert!(record.nlink > 0, "意图删的 inode 还活着");
                work.update_record(*object, |updated| {
                    updated.nlink = 0;
                    updated.change_counter = txg;
                });
                assert!(next.deleted_queue.is_none(), "任何一版里 deleted_inodes 至多一条 key");
                next.deleted_queue = Some(*object);
                next.objects_with_published_intent.insert(*object);
                for stream in TOMBSTONE_STREAMS {
                    if stream.writes_the_whole_object_tombstone_at_the_intent() {
                        let whole = TombstoneRecord {
                            object: *object,
                            object_birth: record.object_birth,
                            range_start_bytes: WHOLE_OBJECT_RANGE_START_BYTES,
                            death_txg: txg,
                        };
                        work.tombstone_appends.push((stream, whole));
                    }
                }
            }
            Operation::UnlinkBatch { object, data_units_kept } => {
                let record = next.record_of(*object).expect("分批删的 inode 有一条记录");
                let file = next.files.get_mut(object).expect("分批删的对象有数据单元");
                assert!(
                    *data_units_kept >= 1 && u64_as_index(*data_units_kept) < file.units.len(),
                    "一批留下至少一个、至少截掉一个"
                );
                file.units.truncate(u64_as_index(*data_units_kept));
                let size_after = *data_units_kept * DATA_UNIT_PAYLOAD_BYTES;
                work.update_record(*object, |updated| {
                    updated.size_bytes = size_after;
                    updated.change_counter = txg;
                });
                for stream in TOMBSTONE_STREAMS {
                    if stream.writes_ranges_at_every_batch_that_is_not_the_last() {
                        let range = TombstoneRecord {
                            object: *object,
                            object_birth: record.object_birth,
                            range_start_bytes: size_after,
                            death_txg: txg,
                        };
                        work.tombstone_appends.push((stream, range));
                    }
                }
            }
            Operation::UnlinkFinal { object } => {
                let record = next.record_of(*object).expect("最后一批删的 inode 有一条记录");
                next.files.remove(object);
                work.merge = work.remove_record_and_merge(*object, self.cell);
                assert_eq!(next.deleted_queue, Some(*object), "最后一批删的是队列里那一个");
                next.deleted_queue = None;
                for stream in TOMBSTONE_STREAMS {
                    if !stream.writes_the_whole_object_tombstone_at_the_intent() {
                        let whole = TombstoneRecord {
                            object: *object,
                            object_birth: record.object_birth,
                            range_start_bytes: WHOLE_OBJECT_RANGE_START_BYTES,
                            death_txg: txg,
                        };
                        work.tombstone_appends.push((stream, whole));
                    }
                }
            }
            Operation::RollBack { target } => {
                next.leaves = target.leaves.clone();
                next.files = target.files.clone();
                next.deleted_queue = target.deleted_queue;
                next.objects_with_published_intent = target.objects_with_published_intent.clone();
                next.next_inode = self.version.next_inode.max(target.next_inode);
                next.lineage.push((txg, target.txg));
                work.leaves = working_leaves_of(&next.leaves);
                work.inode_root_rewritten_anyway = true;
            }
        }
        let leaves = work.finish_leaves();
        // 根装不下按活叶数判；B-stub 写路径多出来的桩不按 135 停（修订 2 之五「历史与写路径格」）。
        let live_leaves = leaves.iter().filter(|leaf| !leaf.records.is_empty()).count();
        if length_as_u64(live_leaves) > ROOT_NODE_LEAF_CONTAINERS {
            return Err(LeafContainersOverflowTheRoot);
        }
        next.leaves = leaves;
        self.append_the_death_records(&mut next, &mut work, txg);
        Ok(PreparedPublish {
            next,
            new_units: work.new_units,
            written: work.written,
            merge: work.merge,
            retired_identities: work.retired_identities,
            stubbed_identities: work.stubbed_identities,
            moved_objects: work.moved_objects,
            split_checks: work.split_checks,
            shrink: work.shrink,
        })
    }

    /// 墓碑（五条写流各一棵树）与退役记录（身份消失那次追加，登记第一节读法 9）。
    fn append_the_death_records(&self, next: &mut Version, work: &mut PublishWork, txg: u64) {
        for stream in TOMBSTONE_STREAMS {
            let appended: Vec<TombstoneRecord> = work
                .tombstone_appends
                .iter()
                .filter(|(appended_stream, _)| *appended_stream == stream)
                .map(|(_, record)| *record)
                .collect();
            let containers = work.append_into_packed_tree(
                &mut next.tombstone_trees[stream.position()],
                &appended,
                TOMBSTONE_CONTAINER_RECORDS,
                (TREE_IDENTIFIER_TOMBSTONE, PACKED_TYPE_TOMBSTONE),
                &|identity, records| UnitBody::Tombstones { stream, identity, records },
            );
            work.written.tombstone_containers[stream.position()] += containers;
            if containers > 0 {
                work.written.tombstone_roots[stream.position()] += 1;
            }
        }
        let retirements: Vec<RetirementRecord> = work
            .retired_identities
            .iter()
            .map(|identity| RetirementRecord { identity: *identity, death_txg: txg })
            .collect();
        let containers = work.append_into_packed_tree(
            &mut next.retirement_tree,
            &retirements,
            RETIREMENT_CONTAINER_RECORDS,
            (TREE_IDENTIFIER_RETIREMENT_IN_THIS_MODEL, PACKED_TYPE_RETIREMENT),
            &|identity, records| UnitBody::Retirements { identity, records },
        );
        work.written.retirement_containers += containers;
        work.written.retirement_records += length_as_u64(retirements.len());
        if containers > 0 {
            work.written.retirement_roots += 1;
        }
        assert!(self.cell.leaf_capacity >= 1, "叶容器容量至少 1");
    }

    /// 落盘、换版：上一版引用而这一版不再引用的单元记在这次换下；这一版引用的单元都算被引用（回退重新引用 R_old 的）。
    fn adopt(&mut self, prepared: PreparedPublish, is_a_rollback: bool) {
        let publish_sequence = self.publish_sequence + 1;
        for unit in prepared.new_units {
            self.assertions.judge("units_carry_the_publish_that_wrote_them", unit.written_at_publish == publish_sequence);
            self.disk.write(unit);
        }
        self.publish_sequence = publish_sequence;
        let before = self.version.referenced_units();
        let after = prepared.next.referenced_units();
        let still_referenced: HashSet<UnitIndex> = after.iter().copied().collect();
        for unit_index in before {
            if !still_referenced.contains(&unit_index) {
                self.disk.units[unit_index].replaced_at_publish = Some(publish_sequence);
            }
        }
        for unit_index in after {
            self.disk.units[unit_index].replaced_at_publish = None;
        }
        let moved_object_shrunk = prepared.shrink.is_some_and(|shrink| self.moved_objects.contains(&shrink.object));
        if let Some(event) = self.gap_watch {
            if self.gap_start.is_none() && event.happened_in(prepared.merge, moved_object_shrunk) {
                self.gap_start = Some(GapStart { version_before: self.version.clone(), event_publish_sequence: publish_sequence });
            }
        }
        if let Some(shrink) = prepared.shrink {
            let largest = self.largest_size_before_a_shrink.entry(shrink.object).or_insert(0);
            *largest = (*largest).max(shrink.size_before_bytes);
            if moved_object_shrunk {
                self.moved_then_shortened += 1;
                self.moved_then_shortened_objects.insert(shrink.object);
            }
        }
        self.moved_objects.extend(prepared.moved_objects.iter().copied());
        for holds in &prepared.split_checks {
            self.assertions.judge("K6_split_right_half_numbered_after_the_record_left_half_unchanged", *holds);
        }
        let queue_before = self.version.deleted_queue;
        let finished_unlink = match (queue_before, prepared.next.deleted_queue, is_a_rollback) {
            (Some(object), None, false) => Some(object),
            (Some(_) | None, _, _) => None,
        };
        if let Some(object) = finished_unlink {
            self.unlinks_completed += 1;
            self.note_dead_versions_holding_the_finished_record(object, &prepared.retired_identities, &prepared.stubbed_identities);
        }
        self.version = prepared.next;
        self.publishes += 1;
        self.self_check.note(&self.version.shape());
        self.publish_records.push(PublishRecord {
            publish_sequence,
            txg: self.version.txg,
            merge: prepared.merge,
            retired_identities: prepared.retired_identities,
            written: prepared.written,
            shrink: prepared.shrink,
            moved_object_shrunk,
            leaf_digest: self.version.leaf_digest(),
            is_a_rollback,
            extent_digest: self.version.extent_digest(),
            stubs_written: length_as_u64(prepared.stubbed_identities.len()),
            leaves_in_the_leaf_order: length_as_u64(self.version.leaves.len()),
            stubs_in_the_leaf_order: self.version.stub_count(),
            holds_an_empty_inode_leaf: self.version.leaves.iter().map(|leaf| leaf.records.len()).any(|record_count| record_count == 0),
            unlinks_completed_so_far: self.unlinks_completed,
        });
        if let (None, Some(object), false) = (queue_before, self.version.deleted_queue, is_a_rollback) {
            self.intent_txgs.insert(object, self.version.txg);
        }
        if self.scanning {
            let record = self.scan_the_current_version(publish_sequence);
            self.scans.push(record);
            self.run_the_planned_positive_controls(publish_sequence);
            if self.segment_four.is_some() {
                self.run_the_segment_four_probes(publish_sequence);
            }
        }
    }

    /// 剩下 > b ⇒ 留 剩下 − b；剩下 ≤ b ⇒ 最后一批（D8 已定项 16 第 2 条，`crates/singlefs-core/src/unlink.rs:118-130`）。
    const fn unlink_step_for(&self, object: u64, data_units_remaining: u64) -> Operation {
        if data_units_remaining <= self.batch_data_units {
            Operation::UnlinkFinal { object }
        } else {
            Operation::UnlinkBatch { object, data_units_kept: data_units_remaining - self.batch_data_units }
        }
    }

    fn next_unlink_step(&self, object: u64) -> Operation {
        self.unlink_step_for(object, self.version.data_units_of(object))
    }

    /// 把删除队列走空（会话里每次用户改动先做这一步，D8 已定项 16 第 6 条）。
    fn walk_the_deletion_queue_empty(&mut self) {
        while let Some(object) = self.version.deleted_queue {
            let step = self.next_unlink_step(object);
            if !self.publish(&step) {
                return;
            }
        }
    }

    /// 一次用户改动经会话：`crates/` 会拒的照拒、记一次、不发布（E177 装置 `apply`）。
    fn apply(&mut self, change: UserChange) {
        if self.has_stopped() {
            return;
        }
        self.walk_the_deletion_queue_empty();
        self.user_changes.push(change);
        match change {
            UserChange::WriteFile { object, data_units } => {
                if self.version.is_live(object) {
                    self.publish(&Operation::WriteFile { object, data_units });
                } else {
                    self.refused_steps += 1;
                }
            }
            UserChange::NewInodes { count } => {
                self.publish(&Operation::NewInodes { count });
            }
            UserChange::Unlink { object } => {
                if self.version.is_with_trees_and_no_file() || !self.version.is_live(object) {
                    self.refused_steps += 1;
                    return;
                }
                if self.moved_objects.contains(&object) {
                    self.moved_then_unlinked += 1;
                }
                if self.publish(&Operation::UnlinkIntent { object }) {
                    self.walk_the_deletion_queue_empty();
                }
            }
        }
    }

    fn roll_back_to(&mut self, target: Version) -> bool {
        self.publish(&Operation::RollBack { target: Box::new(target) })
    }

    /// 意图到最后一批之间崩溃（登记第一节读法 5）：下一步（一批或最后一批）的单元全部写上了盘、它的根没写。
    /// 这次写在崩溃那一刻算换下；现行那一版不动。交回崩溃那一刻的发布序号。
    fn crash_with_the_next_unlink_step_written(&mut self) -> Option<u64> {
        let object = self.version.deleted_queue?;
        let step = self.next_unlink_step(object);
        let prepared = self.prepare(&step).ok()?;
        let crash_publish = self.publish_sequence + 1;
        for unit in prepared.new_units {
            let mut abandoned = unit;
            abandoned.replaced_at_publish = Some(crash_publish);
            self.disk.write(abandoned);
        }
        self.publish_sequence = crash_publish;
        Some(crash_publish)
    }

    /// 恢复：回到崩溃时现行那一版，换一个实例代号（旧实例的已施加事务号 = 那一版的事务号），照挂载先发写行、暖机，txg 重发。
    fn recover_after_the_crash(&mut self) {
        self.switch_to_a_new_instance();
        self.publish(&Operation::RowPublish);
        self.publish(&Operation::WarmUp);
    }

    /// F10 的覆盖写、顺序写取哪个对象：「记录被借走或被吸收过、单元多于 1」里号最小的活对象，没有就取单元多于 1 的号最小的活对象。
    fn rewrite_target(&self) -> Option<u64> {
        let candidates: Vec<u64> = self
            .version
            .live_inodes()
            .into_iter()
            .filter(|inode| self.version.data_units_of(*inode) > 1)
            .collect();
        candidates
            .iter()
            .copied()
            .find(|inode| self.moved_objects.contains(inode))
            .or_else(|| candidates.first().copied())
    }

    fn count_merges(&self) -> (u64, u64, u64) {
        let mut borrows = 0;
        let mut absorbs = 0;
        let mut drops = 0;
        for record in &self.publish_records {
            match record.merge {
                MergeAction::BorrowedFromTheRightNeighbour { .. } => borrows += 1,
                MergeAction::AbsorbedTheRightNeighbour | MergeAction::AbsorbedIntoTheLeftNeighbour => absorbs += 1,
                MergeAction::EmptiedAndDropped => drops += 1,
                MergeAction::NoMerge => {}
            }
        }
        (borrows, absorbs, drops)
    }

    fn total_written(&self) -> UnitsWritten {
        let mut total = UnitsWritten::default();
        for record in &self.publish_records {
            let written = record.written;
            total.data_units += written.data_units;
            total.inode_leaf_containers += written.inode_leaf_containers;
            total.inode_roots += written.inode_roots;
            total.retirement_containers += written.retirement_containers;
            total.retirement_roots += written.retirement_roots;
            total.retirement_records += written.retirement_records;
            for stream in TOMBSTONE_STREAMS {
                total.tombstone_containers[stream.position()] += written.tombstone_containers[stream.position()];
                total.tombstone_roots[stream.position()] += written.tombstone_roots[stream.position()];
            }
        }
        total
    }
}

// ---------------------------------------------------------------------------------------------
// 读者（登记 5.4）：候选 → 择新 → 底座 → 重建 inode 树与整对象墓碑 → 数据单元 → 计数
// ---------------------------------------------------------------------------------------------

/// 一次扫描一个读者的计数（登记第一节读法 1、6）：r = |R − T|、m = |T − R|，三个名字空间各报；O1、O2 崩溃点用。
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
struct ReaderCounts {
    inode_resurrected: u64,
    inode_missed: u64,
    extent_resurrected: u64,
    extent_missed: u64,
    container_resurrected: u64,
    container_missed: u64,
    /// O1：重建出的记录 nlink > 0、而真值里 nlink > 0 的对象集合没有它。
    live_records_beyond_the_truth: u64,
    /// O2：重建视图里有记录（nlink 不论）、而这一版里它的意图已发布。
    records_whose_intent_is_published: u64,
    /// 读出的 inode 记录条数（第十一节 V8 用）。
    inode_records_read: u64,
    /// 择中的零条目类型 2 版本个数（修订 2 之五「读到的桩数」，V-Bs3 用；原写路径恒 0）。
    stubs_read: u64,
    /// B-tsz：P-C 与 P-W 选出的记录不同的 inode 号个数（修订 2 之七 Q-Bt-2；别的读者恒 0）。
    provenance_differences: u64,
}

const COUNTED_QUANTITIES: [&str; 6] =
    ["inode_r", "inode_m", "extent_r", "extent_m", "container_r", "container_m"];

impl ReaderCounts {
    const fn quantities(self) -> [u64; 6] {
        [
            self.inode_resurrected,
            self.inode_missed,
            self.extent_resurrected,
            self.extent_missed,
            self.container_resurrected,
            self.container_missed,
        ]
    }
}

/// 真值：被扫那一版里的 key 集合。
#[derive(Clone, Debug, Default)]
struct Truth {
    inode_keys: HashSet<(u64, u64)>,
    live_inode_keys: HashSet<(u64, u64)>,
    extent_keys: HashSet<(u64, u64, u64)>,
    container_identities: HashSet<ContainerIdentity>,
}

fn truth_of(version: &Version) -> Truth {
    let mut truth = Truth::default();
    // 真值只收记录数 ≥ 1 的容器身份，桩不算活容器（修订 2 之五容器名字空间的口径）。
    for leaf in version.leaves.iter().filter(|leaf| !leaf.records.is_empty()) {
        truth.container_identities.insert(leaf.identity);
        for record in leaf.records.iter() {
            truth.inode_keys.insert((record.inode, record.object_birth));
            if record.nlink > 0 {
                truth.live_inode_keys.insert((record.inode, record.object_birth));
            }
        }
    }
    for (object, file) in &version.files {
        for unit_index_in_file in 0..length_as_u64(file.units.len()) {
            truth.extent_keys.insert((*object, file.object_birth, unit_index_in_file));
        }
    }
    truth
}

/// 父链（I-9.18）：从被扫那一版的 txg 往回走，有谱系记录 (T_rb, T_old) 的 T_rb 的父是 T_old，其余 txg 的父是 txg − 1。
#[derive(Clone, Debug, PartialEq, Eq)]
struct ParentChain {
    inclusive_ranges: Vec<(u64, u64)>,
}

impl ParentChain {
    fn of(scanned_txg: u64, lineage: &[(u64, u64)]) -> Self {
        let mut inclusive_ranges = Vec::new();
        let mut upper = scanned_txg;
        loop {
            let jump = lineage
                .iter()
                .copied()
                .filter(|(rollback_txg, _)| *rollback_txg <= upper)
                .max_by_key(|(rollback_txg, _)| *rollback_txg);
            match jump {
                Some((rollback_txg, old_txg)) => {
                    assert!(old_txg < rollback_txg, "回退回到更早的一版");
                    inclusive_ranges.push((rollback_txg, upper));
                    upper = old_txg;
                }
                None => {
                    inclusive_ranges.push((1, upper));
                    return Self { inclusive_ranges };
                }
            }
        }
    }

    fn contains(&self, txg: u64) -> bool {
        self.inclusive_ranges.iter().any(|(lowest, highest)| *lowest <= txg && txg <= *highest)
    }
}

/// 阳性对照对读者输入的改动：植入的单元（全新身份）、抹掉的单元、拿掉的一条墓碑记录（各条墓碑写流里同字段的那条一起拿掉）。
#[derive(Clone, Debug, Default)]
struct Overlay {
    planted: Vec<Unit>,
    erased: Option<UnitIndex>,
    removed_tombstone: Option<TombstoneRecord>,
    /// PC-Bt2：供出规则倒过来（取写序最小的那一条）。
    reverse_provenance: bool,
    /// PC-Bt3：整对象墓碑不按谱系滤。
    whole_object_tombstones_ignore_lineage: bool,
}

impl Overlay {
    fn planting(planted: Vec<Unit>) -> Self {
        Self { planted, erased: None, removed_tombstone: None, reverse_provenance: false, whole_object_tombstones_ignore_lineage: false }
    }

    fn erasing(erased: UnitIndex) -> Self {
        Self { planted: Vec::new(), erased: Some(erased), removed_tombstone: None, reverse_provenance: false, whole_object_tombstones_ignore_lineage: false }
    }

    fn removing_a_tombstone(removed: TombstoneRecord) -> Self {
        Self { planted: Vec::new(), erased: None, removed_tombstone: Some(removed), reverse_provenance: false, whole_object_tombstones_ignore_lineage: false }
    }

    fn reversing_the_provenance() -> Self {
        Self { planted: Vec::new(), erased: None, removed_tombstone: None, reverse_provenance: true, whole_object_tombstones_ignore_lineage: false }
    }

    fn ignoring_the_lineage_of_whole_object_tombstones() -> Self {
        Self { planted: Vec::new(), erased: None, removed_tombstone: None, reverse_provenance: false, whole_object_tombstones_ignore_lineage: true }
    }
}

struct ScanInputs<'scan> {
    disk: &'scan Disk,
    overlay: &'scan Overlay,
    scan_publish: u64,
    scanned_txg: u64,
    current_instance: u64,
    current_transaction: u64,
    instance_rows: &'scan BTreeMap<u64, u64>,
    chain: ParentChain,
}

impl ScanInputs<'_> {
    fn unit(&self, unit_index: UnitIndex) -> &Unit {
        if unit_index < self.disk.units.len() {
            &self.disk.units[unit_index]
        } else {
            &self.overlay.planted[unit_index - self.disk.units.len()]
        }
    }

    fn unit_count_with_the_planted(&self) -> usize {
        self.disk.units.len() + self.overlay.planted.len()
    }
}

fn is_readable(unit: &Unit, scan_publish: u64, window: ReadableWindow) -> bool {
    match unit.replaced_at_publish {
        None => true,
        Some(replaced_at) => window.allows(scan_publish.saturating_sub(replaced_at)),
    }
}

/// 已发布谓词（D18 已定项 11 的实例表、C113 定案 P2）：现行实例按现行那一版的事务号判，旧实例按实例表那一行判，没有行的旧实例算已发布。
fn is_published_by_the_instance_table(unit: &Unit, inputs: &ScanInputs<'_>) -> bool {
    let instance = unit.write_order.instance;
    if instance == inputs.current_instance {
        return unit.write_order.transaction <= inputs.current_transaction;
    }
    match inputs.instance_rows.get(&instance) {
        Some(applied_transaction) => unit.write_order.transaction <= *applied_transaction,
        None => instance < inputs.current_instance,
    }
}

/// 候选：读得出、已发布、诞生 txg 在被扫那一版的父链上且不大于它的 txg（登记 5.4 第 1 条）。
fn is_candidate(unit: &Unit, inputs: &ScanInputs<'_>, window: ReadableWindow) -> bool {
    let birth_txg = unit.write_order.birth_txg;
    is_readable(unit, inputs.scan_publish, window)
        && is_published_by_the_instance_table(unit, inputs)
        && birth_txg <= inputs.scanned_txg
        && inputs.chain.contains(birth_txg)
}

/// 类型 2 版本的候选要不要看「诞生 txg 在父链上」：只有 B-stub-lin-off（PC-Bs2）不看。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum InodeLeafLineage {
    Required,
    Ignored,
}

/// 进择新的单元：候选（登记 5.4 第 1 条）；零条目版本照样进，B-stub 的桩靠写序最大被择中（修订 2 之五）。
fn enters_the_selection(unit: &Unit, inputs: &ScanInputs<'_>, window: ReadableWindow, lineage: InodeLeafLineage) -> bool {
    match (lineage, &unit.body) {
        (InodeLeafLineage::Ignored, UnitBody::InodeLeaf { .. }) => {
            is_published_by_the_instance_table(unit, inputs)
                && is_readable(unit, inputs.scan_publish, window)
                && unit.write_order.birth_txg <= inputs.scanned_txg
        }
        (InodeLeafLineage::Required, _)
        | (InodeLeafLineage::Ignored, UnitBody::Data { .. } | UnitBody::Tombstones { .. } | UnitBody::Retirements { .. }) => {
            is_candidate(unit, inputs, window)
        }
    }
}

/// 择新之后每组留下的那一个单元，按类分开、按单元序号升序。
#[derive(Clone, Debug, PartialEq, Eq)]
struct SelectedUnits {
    inode_leaves: Vec<UnitIndex>,
    tombstones: Vec<Vec<UnitIndex>>,
    retirements: Vec<UnitIndex>,
    data: Vec<UnitIndex>,
}

impl SelectedUnits {
    fn empty() -> Self {
        Self {
            inode_leaves: Vec::new(),
            tombstones: vec![Vec::new(); TOMBSTONE_STREAM_COUNT],
            retirements: Vec::new(),
            data: Vec::new(),
        }
    }

    fn push(&mut self, category: GroupCategory, unit_index: UnitIndex) {
        match category {
            GroupCategory::Data => self.data.push(unit_index),
            GroupCategory::InodeLeaf => self.inode_leaves.push(unit_index),
            GroupCategory::Tombstones(stream) => self.tombstones[stream.position()].push(unit_index),
            GroupCategory::Retirements => self.retirements.push(unit_index),
        }
    }

    fn sort(&mut self) {
        self.inode_leaves.sort_unstable();
        for stream_units in &mut self.tombstones {
            stream_units.sort_unstable();
        }
        self.retirements.sort_unstable();
        self.data.sort_unstable();
    }
}

/// 主路径：按组的版本清单从新往旧找第一个候选（版本清单按写入次序，写序随写入次序单调）。
fn select_by_the_group_index(inputs: &ScanInputs<'_>, window: ReadableWindow, lineage: InodeLeafLineage) -> SelectedUnits {
    let mut selected = SelectedUnits::empty();
    for (category, groups) in &inputs.disk.groups_by_category {
        for group in groups {
            let newest_candidate = inputs.disk.versions_of_group[*group].iter().rev().copied().find(|unit_index| {
                inputs.overlay.erased != Some(*unit_index) && enters_the_selection(inputs.unit(*unit_index), inputs, window, lineage)
            });
            if let Some(unit_index) = newest_candidate {
                selected.push(*category, unit_index);
            }
        }
    }
    for (offset, planted) in inputs.overlay.planted.iter().enumerate() {
        if enters_the_selection(planted, inputs, window, lineage) {
            selected.push(category_of(planted.body.group_key()), inputs.disk.units.len() + offset);
        }
    }
    selected.sort();
    selected
}

/// 参照路径：整扫一遍盘上每个单元（加植入的、去掉抹掉的），每组按 (写序, 单元序号) 取最大，不读版本清单。
fn select_by_a_full_scan(inputs: &ScanInputs<'_>, window: ReadableWindow, lineage: InodeLeafLineage) -> SelectedUnits {
    let mut newest: BTreeMap<GroupKey, (WriteOrder, UnitIndex)> = BTreeMap::new();
    for unit_index in 0..inputs.unit_count_with_the_planted() {
        if inputs.overlay.erased == Some(unit_index) {
            continue;
        }
        let unit = inputs.unit(unit_index);
        if !enters_the_selection(unit, inputs, window, lineage) {
            continue;
        }
        let order = (unit.write_order, unit_index);
        newest
            .entry(unit.body.group_key())
            .and_modify(|best| {
                if order > *best {
                    *best = order;
                }
            })
            .or_insert(order);
    }
    let mut selected = SelectedUnits::empty();
    for (key, (_, unit_index)) in newest {
        selected.push(category_of(key), unit_index);
    }
    selected.sort();
    selected
}

/// 整对象墓碑对 (o, b) 施不施行：E-I、E-L-w 直接施行；E-L-r 只在施行之前的树里没有 (o, b) 时施行（登记 5.3、5.4 第 4 条）。
const fn whole_object_tombstone_applies(effect: UnlinkEffectArm, object_is_in_the_tree_before_tombstones: bool) -> bool {
    match effect {
        UnlinkEffectArm::AtIntent | UnlinkEffectArm::AtLastBatchWritten => true,
        UnlinkEffectArm::AtLastBatchRead => !object_is_in_the_tree_before_tombstones,
    }
}

/// 一次扫描读得到的、与读者无关的东西（真值、两档 W 的择新结果）。
struct ScanContext<'scan> {
    inputs: ScanInputs<'scan>,
    truth: Truth,
    objects_with_published_intent: &'scan BTreeSet<u64>,
}

/// 一个读者在择新结果上读出 R、与真值比（登记 5.4 第 3–6 条；两条择新路径共用这一段）。
fn judge_reader(selected: &SelectedUnits, reader: Reader, context: &ScanContext<'_>) -> ReaderCounts {
    let inputs = &context.inputs;
    let truth = &context.truth;
    // 第 3 条 底座：一个身份有一条可施行的退役记录、且这一版的诞生 txg ≤ 它的死亡 txg ⇒ 丢掉这一版。
    let mut retirement_deaths: HashMap<ContainerIdentity, u64> = HashMap::new();
    for unit_index in &selected.retirements {
        let retirement_unit = inputs.unit(*unit_index);
        let UnitBody::Retirements { records, .. } = &retirement_unit.body else {
            continue;
        };
        for retirement in records.iter() {
            let applies = match reader.base {
                BaseArm::RetirementOnLineage => inputs.chain.contains(retirement.death_txg),
                BaseArm::RetirementIgnoringLineage => true,
                BaseArm::WithoutRetirement
                | BaseArm::ZeroRecordStubs
                | BaseArm::ZeroRecordStubsIgnoringLineage
                | BaseArm::TombstonesOnly => false,
            };
            if applies {
                let death = retirement_deaths.entry(retirement.identity).or_insert(retirement.death_txg);
                *death = (*death).max(retirement.death_txg);
            }
        }
    }
    // 第 4 条 重建 inode 树：同号多条时按供出规则取一条（原登记 P-W：写序最大的那个容器里的；B-tsz 取 P-C）。
    let provenance = if inputs.overlay.reverse_provenance { Provenance::OldestWriteOrder } else { provenance_of(reader.base) };
    let mut tree_before_tombstones: HashMap<u64, (InodeRecord, (WriteOrder, UnitIndex))> = HashMap::new();
    let mut newest_write_order_choice: HashMap<u64, (WriteOrder, UnitIndex)> = HashMap::new();
    let mut containers_read: Vec<ContainerIdentity> = Vec::new();
    let mut counts = ReaderCounts::default();
    for unit_index in &selected.inode_leaves {
        let leaf_unit = inputs.unit(*unit_index);
        let UnitBody::InodeLeaf { identity, records } = &leaf_unit.body else {
            continue;
        };
        if retirement_deaths.get(identity).is_some_and(|death_txg| leaf_unit.write_order.birth_txg <= *death_txg) {
            continue;
        }
        // 择中的零条目版本（B-stub 的桩）不出记录、不算读出的容器，另数一个「读到的桩数」（修订 2 之五）。
        if records.is_empty() {
            counts.stubs_read += 1;
            continue;
        }
        containers_read.push(*identity);
        let candidate_order = (leaf_unit.write_order, *unit_index);
        for record in records.iter() {
            let keeps_the_existing = tree_before_tombstones.get(&record.inode).is_some_and(|(existing_record, existing_order)| match provenance {
                Provenance::NewestWriteOrder => *existing_order >= candidate_order,
                Provenance::LargestChangeCounter => {
                    (existing_record.change_counter, *existing_order) >= (record.change_counter, candidate_order)
                }
                Provenance::OldestWriteOrder => *existing_order <= candidate_order,
            });
            if !keeps_the_existing {
                tree_before_tombstones.insert(record.inode, (*record, candidate_order));
            }
            if provenance == Provenance::LargestChangeCounter {
                let newest = newest_write_order_choice.entry(record.inode).or_insert(candidate_order);
                *newest = (*newest).max(candidate_order);
            }
        }
    }
    if provenance == Provenance::LargestChangeCounter {
        counts.provenance_differences = length_as_u64(
            tree_before_tombstones
                .iter()
                .filter(|(inode, (_, chosen_order))| newest_write_order_choice.get(*inode).is_some_and(|newest| newest != chosen_order))
                .count(),
        );
    }
    let objects_in_the_tree: HashSet<(u64, u64)> =
        tree_before_tombstones.values().map(|(record, _)| (record.inode, record.object_birth)).collect();
    // 这个读者那条墓碑写流里在谱系上的整对象墓碑与区间墓碑。
    let stream = reader.tombstone_stream();
    let mut whole_object_deaths: HashMap<(u64, u64), u64> = HashMap::new();
    let mut range_tombstones: HashMap<(u64, u64), Vec<(u64, u64)>> = HashMap::new();
    for unit_index in &selected.tombstones[stream.position()] {
        let UnitBody::Tombstones { records, .. } = &inputs.unit(*unit_index).body else {
            continue;
        };
        for tombstone in records.iter() {
            if inputs.overlay.removed_tombstone == Some(*tombstone) {
                continue;
            }
            let object_key = (tombstone.object, tombstone.object_birth);
            if tombstone.range_start_bytes == WHOLE_OBJECT_RANGE_START_BYTES {
                let whole_object_tombstone_is_on_the_chain = inputs.chain.contains(tombstone.death_txg);
                if whole_object_tombstone_is_on_the_chain || inputs.overlay.whole_object_tombstones_ignore_lineage {
                    let death = whole_object_deaths.entry(object_key).or_insert(tombstone.death_txg);
                    *death = (*death).max(tombstone.death_txg);
                }
            } else {
                let range_tombstone_is_on_the_chain = inputs.chain.contains(tombstone.death_txg);
                if range_tombstone_is_on_the_chain {
                    range_tombstones.entry(object_key).or_default().push((tombstone.range_start_bytes, tombstone.death_txg));
                }
            }
        }
    }
    // inode 读出 = 施行整对象墓碑之前的树，去掉被施行的 (o, b)（读法 2）。
    let mut inode_hits: u64 = 0;
    for (inode, (record, _)) in &tree_before_tombstones {
        let object_key = (*inode, record.object_birth);
        if whole_object_deaths.contains_key(&object_key) && whole_object_tombstone_applies(reader.effect, true) {
            continue;
        }
        counts.inode_records_read += 1;
        if truth.inode_keys.contains(&object_key) {
            inode_hits += 1;
        } else {
            counts.inode_resurrected += 1;
        }
        if record.nlink > 0 && !truth.live_inode_keys.contains(&object_key) {
            counts.live_records_beyond_the_truth += 1;
        }
        if context.objects_with_published_intent.contains(inode) {
            counts.records_whose_intent_is_published += 1;
        }
    }
    counts.inode_missed = difference_or_marker(length_as_u64(truth.inode_keys.len()), inode_hits);
    // 第 5 条 数据单元：按 X2 臂判活（用施行墓碑之前的树），再按 X3 臂施行整对象墓碑。
    let mut extent_hits: u64 = 0;
    for unit_index in &selected.data {
        let data_unit = inputs.unit(*unit_index);
        let Some((object, object_birth, unit_index_in_file)) = data_key_of(data_unit) else {
            continue;
        };
        let object_key = (object, object_birth);
        let object_is_in_the_tree = objects_in_the_tree.contains(&object_key);
        let unit_birth_txg = data_unit.write_order.birth_txg;
        let alive_by_the_shrink_arm = match reader.shrink {
            ShrinkArm::RecordSize => tree_before_tombstones.get(&object).is_some_and(|(record, _)| {
                record.object_birth == object_birth && unit_index_in_file < record.data_units()
            }),
            ShrinkArm::RangeTombstone => {
                object_is_in_the_tree
                    && !range_tombstones.get(&object_key).is_some_and(|ranges| {
                        ranges.iter().any(|(range_start, range_death_txg)| {
                            unit_index_in_file * DATA_UNIT_PAYLOAD_BYTES >= *range_start && unit_birth_txg <= *range_death_txg
                        })
                    })
            }
            ShrinkArm::ObjectGateOnly => object_is_in_the_tree,
        };
        if !alive_by_the_shrink_arm {
            continue;
        }
        let killed_by_a_whole_object_tombstone = whole_object_deaths.get(&object_key).is_some_and(|whole_death_txg| {
            whole_object_tombstone_applies(reader.effect, object_is_in_the_tree) && unit_birth_txg <= *whole_death_txg
        });
        if killed_by_a_whole_object_tombstone {
            continue;
        }
        if truth.extent_keys.contains(&(object, object_birth, unit_index_in_file)) {
            extent_hits += 1;
        } else {
            counts.extent_resurrected += 1;
        }
    }
    counts.extent_missed = difference_or_marker(length_as_u64(truth.extent_keys.len()), extent_hits);
    let mut container_hits: u64 = 0;
    for identity in &containers_read {
        if truth.container_identities.contains(identity) {
            container_hits += 1;
        } else {
            counts.container_resurrected += 1;
        }
    }
    counts.container_missed = difference_or_marker(length_as_u64(truth.container_identities.len()), container_hits);
    counts
}

/// 一次扫描留下的东西：每个读者的计数（次序同 `every_reader`）。
#[derive(Clone, Debug, PartialEq, Eq)]
struct ScanRecord {
    publish_sequence: u64,
    txg: u64,
    inside_an_unlink: bool,
    after_a_rollback: bool,
    truth_inode_keys: u64,
    /// 这一版有没有一个变短过的活对象（PC-Z 取最后一次有的那次扫描），有就是号最小的那个。
    live_shrunk_object: Option<u64>,
    counts: Vec<ReaderCounts>,
}

/// 择新走哪条路（第十一节 V3 的两路）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum SelectionPath {
    GroupIndex,
    FullScan,
}

impl Simulation {
    fn scan_inputs<'scan>(&'scan self, overlay: &'scan Overlay, scan_publish: u64) -> ScanInputs<'scan> {
        ScanInputs {
            disk: &self.disk,
            overlay,
            scan_publish,
            scanned_txg: self.version.txg,
            current_instance: self.version.instance,
            current_transaction: self.version.transaction,
            instance_rows: &self.instance_rows,
            chain: ParentChain::of(self.version.txg, &self.version.lineage),
        }
    }

    /// 这条写路径的读者（第二、三段 20 个）各读一遍现行那一版（植入、抹掉的照 `overlay` 改读者输入）。
    fn read_every_reader(&self, overlay: &Overlay, scan_publish: u64, path: SelectionPath) -> Vec<ReaderCounts> {
        self.read_readers(&self.readers, overlay, scan_publish, path)
    }

    /// 给定的读者各读一遍现行那一版；B-stub-lin-off 用类型 2 不看谱系的那一份择新（PC-Bs2）。
    fn read_readers(&self, readers: &[Reader], overlay: &Overlay, scan_publish: u64, path: SelectionPath) -> Vec<ReaderCounts> {
        let context = ScanContext {
            inputs: self.scan_inputs(overlay, scan_publish),
            truth: truth_of(&self.version),
            objects_with_published_intent: &self.version.objects_with_published_intent,
        };
        let select = |window: ReadableWindow, lineage: InodeLeafLineage| match path {
            SelectionPath::GroupIndex => select_by_the_group_index(&context.inputs, window, lineage),
            SelectionPath::FullScan => select_by_a_full_scan(&context.inputs, window, lineage),
        };
        let forever = select(ReadableWindow::Forever, InodeLeafLineage::Required);
        let ring = select(ReadableWindow::RingWindow, InodeLeafLineage::Required);
        let lineage_ignored = readers.iter().any(|reader| reader.base == BaseArm::ZeroRecordStubsIgnoringLineage).then(|| {
            (select(ReadableWindow::Forever, InodeLeafLineage::Ignored), select(ReadableWindow::RingWindow, InodeLeafLineage::Ignored))
        });
        readers
            .iter()
            .map(|reader| {
                let ignores_lineage = reader.base == BaseArm::ZeroRecordStubsIgnoringLineage;
                let selected = match (reader.window, ignores_lineage, &lineage_ignored) {
                    (ReadableWindow::Forever, true, Some((ignored_forever, _))) => ignored_forever,
                    (ReadableWindow::RingWindow, true, Some((_, ignored_ring))) => ignored_ring,
                    (ReadableWindow::Forever, _, _) => &forever,
                    (ReadableWindow::RingWindow, _, _) => &ring,
                };
                judge_reader(selected, *reader, &context)
            })
            .collect()
    }

    fn live_shrunk_object(&self) -> Option<u64> {
        self.largest_size_before_a_shrink.keys().copied().find(|object| self.version.is_live(*object))
    }

    fn scan_the_current_version(&self, scan_publish: u64) -> ScanRecord {
        ScanRecord {
            publish_sequence: scan_publish,
            txg: self.version.txg,
            inside_an_unlink: self.version.is_inside_an_unlink(),
            after_a_rollback: !self.version.lineage.is_empty(),
            truth_inode_keys: length_as_u64(self.version.leaves.iter().map(|leaf| leaf.records.len()).sum()),
            live_shrunk_object: self.live_shrunk_object(),
            counts: self.read_every_reader(&Overlay::default(), scan_publish, SelectionPath::GroupIndex),
        }
    }
}

/// 一个量的轨迹（第八节）：峰值、峰值所在的发布序号、为正的扫描次数、期末值（这一类扫描的最后一次）。
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
struct QuantityTrajectory {
    scans: u64,
    peak: u64,
    peak_at_publish: u64,
    positive_scans: u64,
    last: u64,
}

impl QuantityTrajectory {
    fn observe(&mut self, value: u64, publish_sequence: u64) {
        self.scans += 1;
        if value > self.peak {
            self.peak = value;
            self.peak_at_publish = publish_sequence;
        }
        self.positive_scans += u64::from(value > 0);
        self.last = value;
    }

    fn absorb(&mut self, other: QuantityTrajectory) {
        self.scans += other.scans;
        if other.peak > self.peak {
            self.peak = other.peak;
            self.peak_at_publish = other.peak_at_publish;
        }
        self.positive_scans += other.positive_scans;
        self.last = self.last.max(other.last);
    }

    fn fields(self, prefix: &str) -> String {
        format!(
            "{prefix}_peak={} {prefix}_peak_at={} {prefix}_positive_scans={} {prefix}_last={}",
            self.peak, self.peak_at_publish, self.positive_scans, self.last
        )
    }
}

/// 扫描按什么分类（第一节读法 2、4，Q-B2「回退之后」）。
const SCAN_SCOPES: [&str; 5] = ["all", "inside_unlink", "outside_unlink", "after_rollback", "after_rollback_outside_unlink"];

const fn scan_scopes_of(record: &ScanRecord) -> [bool; 5] {
    [
        true,
        record.inside_an_unlink,
        !record.inside_an_unlink,
        record.after_a_rollback,
        record.after_a_rollback && !record.inside_an_unlink,
    ]
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
struct ReaderTrajectory {
    by_scope: [[QuantityTrajectory; 6]; 5],
    /// 读出的 inode 记录集合为空而真值不空的扫描数（第十一节 V8 的触发计数）。
    read_out_empty_scans: u64,
}

fn trajectories_of(scans: &[ScanRecord], reader_count: usize) -> Vec<ReaderTrajectory> {
    let mut trajectories = vec![ReaderTrajectory::default(); reader_count];
    for record in scans {
        let scopes = scan_scopes_of(record);
        for (reader_position, counts) in record.counts.iter().enumerate() {
            let trajectory = &mut trajectories[reader_position];
            for (scope_position, in_scope) in scopes.iter().enumerate() {
                if *in_scope {
                    for (quantity_position, value) in counts.quantities().iter().enumerate() {
                        trajectory.by_scope[scope_position][quantity_position].observe(*value, record.publish_sequence);
                    }
                }
            }
            trajectory.read_out_empty_scans += u64::from(counts.inode_records_read == 0 && record.truth_inode_keys > 0);
        }
    }
    trajectories
}

// ---------------------------------------------------------------------------------------------
// 历史族（登记 5.2）：H51 照 E177 装置逐段重放，F10、F11、F12 是本实验加的
// ---------------------------------------------------------------------------------------------

/// 带种子的确定性随机源（splitmix64，E177 装置第 4566–4586 行同一写法）。
#[derive(Clone, Debug)]
struct SeededRandom {
    state: u64,
}

impl SeededRandom {
    const fn new(seed: u64) -> Self {
        Self { state: seed }
    }

    fn next_value(&mut self) -> u64 {
        self.state = self.state.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut mixed = self.state;
        mixed = (mixed ^ (mixed >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        mixed = (mixed ^ (mixed >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        mixed ^ (mixed >> 31)
    }

    fn below(&mut self, bound: u64) -> u64 {
        self.next_value() % bound
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum DeletionOrder {
    Ascending,
    EvenThenOdd,
    Random(u64),
}

impl DeletionOrder {
    fn label(self) -> String {
        match self {
            DeletionOrder::Ascending => "ascending".to_string(),
            DeletionOrder::EvenThenOdd => "even_then_odd".to_string(),
            DeletionOrder::Random(seed) => format!("random_seed_{seed}"),
        }
    }
}

fn deletion_orders() -> Vec<DeletionOrder> {
    let mut orders = vec![DeletionOrder::Ascending, DeletionOrder::EvenThenOdd];
    orders.extend((1..=5).map(DeletionOrder::Random));
    orders
}

/// 删序：inode 2..n 的一个排列（E177 装置 `deletion_order_of`，第 4709–4730 行）。
fn deletion_order_of(inode_count: u64, order: DeletionOrder) -> Vec<u64> {
    let inodes: Vec<u64> = (FIRST_FILE_INODE + 1..=inode_count).collect();
    match order {
        DeletionOrder::Ascending => inodes,
        DeletionOrder::EvenThenOdd => inodes
            .iter()
            .copied()
            .filter(|inode| inode % 2 == 0)
            .chain(inodes.iter().copied().filter(|inode| inode % 2 == 1))
            .collect(),
        DeletionOrder::Random(seed) => {
            let mut shuffled = inodes;
            let mut random = SeededRandom::new(seed);
            for position in (1..shuffled.len()).rev() {
                let other = u64_as_index(random.below(length_as_u64(position) + 1));
                shuffled.swap(position, other);
            }
            shuffled
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum History {
    /// H51 的 F1：建删空 inode。
    BuildAndDeleteEmptyInodes { inode_count: u64, order: DeletionOrder },
    /// H51 的 F8：大文件改写删。
    LargeFileRewriteAndDelete { data_units: u64, rewrites: u64, overwrite_before_unlink: bool, batch: u64 },
    /// H51 的 F3：混合随机。
    MixedRandom { seed: u64 },
    /// F10：多对象带数据、借与吸收。
    ManyObjectsWithData { inode_count: u64, order: DeletionOrder, batch: u64 },
    /// F11：回退越过一个缺口事件。
    RollBackOverAGap { event: GapEvent },
}

impl History {
    const fn family(self) -> &'static str {
        match self {
            History::BuildAndDeleteEmptyInodes { .. } => "F1",
            History::LargeFileRewriteAndDelete { .. } => "F8",
            History::MixedRandom { .. } => "F3",
            History::ManyObjectsWithData { .. } => "F10",
            History::RollBackOverAGap { .. } => "F11",
        }
    }

    fn label(self) -> String {
        match self {
            History::BuildAndDeleteEmptyInodes { inode_count, order } => {
                format!("family=F1 n={inode_count} order={}", order.label())
            }
            History::LargeFileRewriteAndDelete { data_units, rewrites, overwrite_before_unlink, batch } => format!(
                "family=F8 n_u={data_units} k={rewrites} e={} batch={batch}",
                if overwrite_before_unlink { "OU" } else { "U" }
            ),
            History::MixedRandom { seed } => format!("family=F3 seed={seed}"),
            History::ManyObjectsWithData { inode_count, order, batch } => {
                format!("family=F10 n={inode_count} order={} batch={batch}", order.label())
            }
            History::RollBackOverAGap { event } => format!("family=F11 n={F11_INODE_COUNT} gap_event={} batch={PRODUCT_UNLINK_BATCH_DATA_UNITS}", event.label()),
        }
    }

    const fn batch(self) -> u64 {
        match self {
            History::LargeFileRewriteAndDelete { batch, .. } | History::ManyObjectsWithData { batch, .. } => batch,
            History::BuildAndDeleteEmptyInodes { .. } | History::MixedRandom { .. } | History::RollBackOverAGap { .. } => {
                PRODUCT_UNLINK_BATCH_DATA_UNITS
            }
        }
    }
}

/// H51：F1 14 段、F8 32 段、F3 5 段（E177 装置 `segment_one_histories`，第 4670–4697 行）。
fn histories_h51() -> Vec<History> {
    let mut histories = Vec::new();
    for inode_count in [300, 900] {
        for order in deletion_orders() {
            histories.push(History::BuildAndDeleteEmptyInodes { inode_count, order });
        }
    }
    for data_units in [1, 3, 145, 300] {
        for rewrites in [0, 5] {
            for overwrite_before_unlink in [false, true] {
                for batch in [1, PRODUCT_UNLINK_BATCH_DATA_UNITS] {
                    histories.push(History::LargeFileRewriteAndDelete { data_units, rewrites, overwrite_before_unlink, batch });
                }
            }
        }
    }
    for seed in 1..=5 {
        histories.push(History::MixedRandom { seed });
    }
    histories
}

/// F10：n ∈ {300, 900} × 删序 7 种 × 批 ∈ {1, 144}，28 段。
fn histories_f10() -> Vec<History> {
    let mut histories = Vec::new();
    for inode_count in [300, 900] {
        for order in deletion_orders() {
            for batch in [1, PRODUCT_UNLINK_BATCH_DATA_UNITS] {
                histories.push(History::ManyObjectsWithData { inode_count, order, batch });
            }
        }
    }
    histories
}

/// E177 产物 `research/results/e177-per-tree-resurrection-without-tombstones-segment-one.out` 里 F3 五段、`geometry=wide` 那两条
/// `name=simulation` 行的 `publishes=` 与 `refused_steps=`（登记第七节 A1；两条写路径的两行相同，只读这两段）。
const E177_MIXED_RANDOM_PUBLISHES_AND_REFUSALS: [(u64, u64, u64); 5] =
    [(1, 414, 0), (2, 414, 0), (3, 328, 95), (4, 334, 89), (5, 347, 85)];

/// F10、F11 的建与写：每次建 50 个空 inode 到共 n 个；对 i = 2..n 各顺序写 1 + (i mod 5) 个单元（各一次发布）。
fn build_and_write_many_objects(simulation: &mut Simulation, inode_count: u64) {
    let mut created = 1;
    while created < inode_count {
        let count = NEW_INODES_PER_STEP.min(inode_count - created);
        simulation.apply(UserChange::NewInodes { count });
        created += count;
    }
    for object in FIRST_FILE_INODE + 1..=inode_count {
        simulation.apply(UserChange::WriteFile { object, data_units: 1 + object % 5 });
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum PendingChange {
    RewriteTheSelectedObject { data_units: u64 },
    UnlinkTheSmallestLiveObject,
}

/// 「按升序删」带 F10 的覆盖写规则：第 j 次 unlink 之前，j 是 10 的倍数时覆盖写、是 25 的倍数时顺序写成 5 个单元（读法 6：删号最小的活对象）。
#[derive(Clone, Debug, Default)]
struct AscendingDeletionStream {
    unlinks_started: u64,
    pending: std::collections::VecDeque<PendingChange>,
}

impl AscendingDeletionStream {
    fn next_change(&mut self, simulation: &Simulation) -> Option<UserChange> {
        loop {
            if self.pending.is_empty() {
                self.unlinks_started += 1;
                let unlink_number = self.unlinks_started;
                if unlink_number % F10_OVERWRITE_EVERY_UNLINKS == 0 {
                    self.pending.push_back(PendingChange::RewriteTheSelectedObject { data_units: 1 });
                }
                if unlink_number % F10_SEQUENTIAL_WRITE_EVERY_UNLINKS == 0 {
                    self.pending.push_back(PendingChange::RewriteTheSelectedObject { data_units: F10_SEQUENTIAL_WRITE_DATA_UNITS });
                }
                self.pending.push_back(PendingChange::UnlinkTheSmallestLiveObject);
            }
            match self.pending.pop_front()? {
                PendingChange::RewriteTheSelectedObject { data_units } => {
                    if let Some(object) = simulation.rewrite_target() {
                        return Some(UserChange::WriteFile { object, data_units });
                    }
                }
                PendingChange::UnlinkTheSmallestLiveObject => {
                    return simulation
                        .version
                        .live_inodes()
                        .into_iter()
                        .find(|inode| *inode > FIRST_FILE_INODE)
                        .map(|object| UserChange::Unlink { object });
                }
            }
        }
    }
}

/// F11 一段的结局（登记 5.2、5.5 PC-B2）。
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
struct RollbackOutcome {
    event_found: bool,
    distance_in_publishes: u64,
    too_far: bool,
    rollback_publish_sequence: u64,
    retired_in_the_gap_and_present_in_the_old_version: u64,
    retirement_records_after_the_rollback: u64,
}

fn run_rollback_over_a_gap(simulation: &mut Simulation, event: GapEvent) -> RollbackOutcome {
    let mut outcome = RollbackOutcome::default();
    build_and_write_many_objects(simulation, F11_INODE_COUNT);
    simulation.gap_watch = Some(event);
    let mut stream = AscendingDeletionStream::default();
    while simulation.gap_start.is_none() && !simulation.has_stopped() {
        let Some(change) = stream.next_change(simulation) else {
            return outcome;
        };
        simulation.apply(change);
    }
    let Some(gap_start) = simulation.gap_start.clone() else {
        return outcome;
    };
    outcome.event_found = true;
    for _ in 0..F11_CHANGES_BEFORE_ROLLBACK {
        if let Some(change) = stream.next_change(simulation) {
            simulation.apply(change);
        }
    }
    outcome.distance_in_publishes = simulation.version.txg - gap_start.version_before.txg;
    outcome.too_far = outcome.distance_in_publishes > ROLLBACK_DISTANCE_LIMIT_PUBLISHES;
    let identities_in_the_old_version: HashSet<ContainerIdentity> =
        gap_start.version_before.leaves.iter().map(|leaf| leaf.identity).collect();
    let retired_in_the_gap: BTreeSet<ContainerIdentity> = simulation
        .publish_records
        .iter()
        .filter(|record| record.publish_sequence >= gap_start.event_publish_sequence)
        .flat_map(|record| record.retired_identities.iter().copied())
        .filter(|identity| identities_in_the_old_version.contains(identity))
        .collect();
    outcome.retired_in_the_gap_and_present_in_the_old_version = length_as_u64(retired_in_the_gap.len());
    simulation.roll_back_to(gap_start.version_before);
    outcome.rollback_publish_sequence = simulation.publish_sequence;
    for _ in 0..F11_CHANGES_AFTER_ROLLBACK {
        if let Some(change) = stream.next_change(simulation) {
            simulation.apply(change);
        }
    }
    outcome.retirement_records_after_the_rollback = simulation
        .publish_records
        .iter()
        .filter(|record| record.publish_sequence > outcome.rollback_publish_sequence)
        .map(|record| record.written.retirement_records)
        .sum();
    outcome
}

/// 照历史驱动一段模拟；顺带判只在某个族上能判的断言（A1、A8）。F11 交回回退的结局。
fn run_history(history: History, simulation: &mut Simulation) -> Option<RollbackOutcome> {
    match history {
        History::BuildAndDeleteEmptyInodes { inode_count, order } => {
            let mut created = 1;
            while created < inode_count {
                let count = NEW_INODES_PER_STEP.min(inode_count - created);
                simulation.apply(UserChange::NewInodes { count });
                created += count;
            }
            for object in deletion_order_of(inode_count, order) {
                simulation.apply(UserChange::Unlink { object });
            }
            let expected_publishes = 3 + (inode_count - 1).div_ceil(NEW_INODES_PER_STEP) + 2 * (inode_count - 1);
            simulation.assertions.judge("A1_history_publishes", simulation.publishes == expected_publishes);
            simulation.assertions.judge("A1_history_refused_steps", simulation.refused_steps == 0);
            if simulation.cell.merge_threshold == 0 && simulation.container_death == ContainerDeathWritePath::RetirementRecords {
                let expected_retirements = inode_count.div_ceil(simulation.cell.leaf_capacity) - 1;
                let retirements = simulation.total_written().retirement_records;
                simulation.assertions.judge("A8_retirements_equal_identity_disappearances", retirements == expected_retirements);
            }
        }
        History::LargeFileRewriteAndDelete { data_units, rewrites, overwrite_before_unlink, batch } => {
            for _ in 0..=rewrites {
                simulation.apply(UserChange::WriteFile { object: FIRST_FILE_INODE, data_units });
            }
            if overwrite_before_unlink {
                simulation.apply(UserChange::WriteFile { object: FIRST_FILE_INODE, data_units: 1 });
            }
            simulation.apply(UserChange::Unlink { object: FIRST_FILE_INODE });
            let data_units_at_the_intent = if overwrite_before_unlink { 1 } else { data_units };
            let expected_publishes =
                3 + (rewrites + 1) + u64::from(overwrite_before_unlink) + 1 + data_units_at_the_intent.div_ceil(batch).max(1);
            simulation.assertions.judge("A1_history_publishes", simulation.publishes == expected_publishes);
            simulation.assertions.judge("A1_history_refused_steps", simulation.refused_steps == 0);
            if simulation.cell.merge_threshold == 0 && simulation.container_death == ContainerDeathWritePath::RetirementRecords {
                let retirements = simulation.total_written().retirement_records;
                simulation.assertions.judge("A8_retirements_equal_identity_disappearances", retirements == 1);
            }
        }
        History::MixedRandom { seed } => {
            let mut random = SeededRandom::new(seed);
            for _ in 0..MIXED_RANDOM_STEPS {
                let draw = random.below(10);
                let change = if draw < 3 {
                    UserChange::NewInodes { count: 1 + random.below(20) }
                } else if draw < 5 {
                    UserChange::WriteFile { object: FIRST_FILE_INODE, data_units: 1 }
                } else if draw < 6 {
                    UserChange::WriteFile { object: FIRST_FILE_INODE, data_units: 1 + random.below(200) }
                } else {
                    let live = simulation.version.live_inodes();
                    if live.is_empty() {
                        simulation.refused_steps += 1;
                        continue;
                    }
                    UserChange::Unlink { object: live[u64_as_index(random.below(length_as_u64(live.len())))] }
                };
                simulation.apply(change);
            }
            let expected = E177_MIXED_RANDOM_PUBLISHES_AND_REFUSALS.iter().find(|(expected_seed, _, _)| *expected_seed == seed);
            let holds = expected.is_some_and(|(_, publishes, refused)| {
                simulation.publishes == *publishes && simulation.refused_steps == *refused
            });
            simulation.assertions.judge("A1_history_publishes", holds);
            simulation.assertions.judge("A1_history_refused_steps", holds);
        }
        History::ManyObjectsWithData { inode_count, order, .. } => {
            build_and_write_many_objects(simulation, inode_count);
            for (position, victim) in deletion_order_of(inode_count, order).into_iter().enumerate() {
                let unlink_number = length_as_u64(position) + 1;
                if unlink_number % F10_OVERWRITE_EVERY_UNLINKS == 0 {
                    if let Some(object) = simulation.rewrite_target() {
                        simulation.apply(UserChange::WriteFile { object, data_units: 1 });
                    }
                }
                if unlink_number % F10_SEQUENTIAL_WRITE_EVERY_UNLINKS == 0 {
                    if let Some(object) = simulation.rewrite_target() {
                        simulation.apply(UserChange::WriteFile { object, data_units: F10_SEQUENTIAL_WRITE_DATA_UNITS });
                    }
                }
                simulation.apply(UserChange::Unlink { object: victim });
            }
        }
        History::RollBackOverAGap { event } => return Some(run_rollback_over_a_gap(simulation, event)),
    }
    None
}

fn simulate(history: History, cell: WritePathCell, scanning: bool) -> (Simulation, Option<RollbackOutcome>) {
    let mut simulation = Simulation::start(cell, history.batch(), scanning);
    let outcome = run_history(history, &mut simulation);
    (simulation, outcome)
}

// ---------------------------------------------------------------------------------------------
// 第一段（登记 5.8）：X4。H51 与 F10 在 N1 / N2 × 容量 × θ 下只跑写路径；5.6 全表；PC-N、PC-K；停机对拍 S1–S3、S6
// ---------------------------------------------------------------------------------------------

/// 一组观测的最小、最大、总数与个数。
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
struct Extremes {
    observations: u64,
    smallest: Option<u64>,
    largest: Option<u64>,
    total: u64,
}

impl Extremes {
    fn observe(&mut self, value: u64) {
        self.observations += 1;
        self.smallest = Some(self.smallest.map_or(value, |smallest| smallest.min(value)));
        self.largest = Some(self.largest.map_or(value, |largest| largest.max(value)));
        self.total += value;
    }

    fn absorb(&mut self, other: Extremes) {
        self.observations += other.observations;
        self.total += other.total;
        if let Some(value) = other.smallest {
            self.smallest = Some(self.smallest.map_or(value, |smallest| smallest.min(value)));
        }
        if let Some(value) = other.largest {
            self.largest = Some(self.largest.map_or(value, |largest| largest.max(value)));
        }
    }

    fn fields(self, prefix: &str) -> String {
        let shown = |value: Option<u64>| value.map_or_else(|| "none".to_string(), |known| known.to_string());
        format!(
            "{prefix}_count={} {prefix}_smallest={} {prefix}_largest={} {prefix}_total={}",
            self.observations,
            shown(self.smallest),
            shown(self.largest),
            self.total
        )
    }
}

/// 一条写路径跑完留下的账（模拟本身丢掉，免得盘上的单元占着内存）。
#[derive(Clone, Debug, PartialEq, Eq)]
struct WritePathRun {
    publishes: u64,
    refused_steps: u64,
    borrows: u64,
    absorbs: u64,
    drops: u64,
    moved_then_shortened: u64,
    moved_then_unlinked: u64,
    written: UnitsWritten,
    self_check: SelfCheckTally,
    does_not_fit_at: Option<u64>,
    assertions: AssertionTally,
    publish_records: Vec<PublishRecord>,
    user_changes: Vec<UserChange>,
}

fn run_write_path(history: History, cell: WritePathCell) -> WritePathRun {
    let (simulation, _) = simulate(history, cell, false);
    let (borrows, absorbs, drops) = simulation.count_merges();
    WritePathRun {
        publishes: simulation.publishes,
        refused_steps: simulation.refused_steps,
        borrows,
        absorbs,
        drops,
        moved_then_shortened: simulation.moved_then_shortened,
        moved_then_unlinked: simulation.moved_then_unlinked,
        written: simulation.total_written(),
        self_check: simulation.self_check,
        does_not_fit_at: simulation.does_not_fit_at,
        assertions: simulation.assertions.clone(),
        publish_records: simulation.publish_records,
        user_changes: simulation.user_changes,
    }
}

/// 同一段历史、同一个容量与 θ 下 N1 与 N2 两条写流的配对（登记第一节读法 7、Q-X4-1、Q-X4-2、A5）。
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
struct PairStatistics {
    borrow_publishes: u64,
    extra_code_three_per_borrow: Extremes,
    extra_code_two_per_borrow: Extremes,
    retirement_records_per_borrow_renew_identity: Extremes,
    retirement_records_per_borrow_raise_separator: Extremes,
    publishes_outside_borrows_with_a_difference: u64,
    borrow_kinds_disagree: u64,
    renew_identity_counted_units: u64,
    raise_separator_counted_units: u64,
    user_changes_identical: bool,
    publish_counts_identical: bool,
    streams_identical: bool,
}

impl PairStatistics {
    /// N2 多写占这段历史全部写单元的比例，按百万分之一计（读法 8）。
    fn extra_share_in_parts_per_million(self) -> u64 {
        let extra = self.renew_identity_counted_units.saturating_sub(self.raise_separator_counted_units);
        if self.renew_identity_counted_units == 0 {
            return 0;
        }
        extra * 1_000_000 / self.renew_identity_counted_units
    }
}

fn pair_statistics(raise_separator: &WritePathRun, renew_identity: &WritePathRun) -> PairStatistics {
    let mut pair = PairStatistics {
        borrow_publishes: 0,
        extra_code_three_per_borrow: Extremes::default(),
        extra_code_two_per_borrow: Extremes::default(),
        retirement_records_per_borrow_renew_identity: Extremes::default(),
        retirement_records_per_borrow_raise_separator: Extremes::default(),
        publishes_outside_borrows_with_a_difference: 0,
        borrow_kinds_disagree: 0,
        renew_identity_counted_units: 0,
        raise_separator_counted_units: 0,
        user_changes_identical: raise_separator.user_changes == renew_identity.user_changes,
        publish_counts_identical: raise_separator.publish_records.len() == renew_identity.publish_records.len(),
        streams_identical: true,
    };
    for (raised, renewed) in raise_separator.publish_records.iter().zip(&renew_identity.publish_records) {
        pair.renew_identity_counted_units += renewed.written.every_counted_unit();
        pair.raise_separator_counted_units += raised.written.every_counted_unit();
        if raised.written != renewed.written || raised.leaf_digest != renewed.leaf_digest {
            pair.streams_identical = false;
        }
        let renewed_borrowed = matches!(renewed.merge, MergeAction::BorrowedFromTheRightNeighbour { .. });
        let raised_borrowed = matches!(raised.merge, MergeAction::BorrowedFromTheRightNeighbour { .. });
        if renewed_borrowed != raised_borrowed || std::mem::discriminant(&renewed.merge) != std::mem::discriminant(&raised.merge) {
            pair.borrow_kinds_disagree += 1;
        }
        if renewed_borrowed {
            pair.borrow_publishes += 1;
            pair.extra_code_three_per_borrow
                .observe(difference_or_marker(renewed.written.code_three_counted(), raised.written.code_three_counted()));
            pair.extra_code_two_per_borrow
                .observe(difference_or_marker(renewed.written.code_two_counted(), raised.written.code_two_counted()));
            pair.retirement_records_per_borrow_renew_identity.observe(renewed.written.retirement_records);
            pair.retirement_records_per_borrow_raise_separator.observe(raised.written.retirement_records);
        } else if renewed.written.every_counted_unit() != raised.written.every_counted_unit() {
            pair.publishes_outside_borrows_with_a_difference += 1;
        }
    }
    pair
}

/// 第一段的一个工作项：一段历史 × 一个容量 × 一个 θ，N1、N2 各跑两遍（第十一节 V4 的逐遍比）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct SegmentOneItem {
    history: History,
    leaf_capacity: u64,
    merge_threshold: u64,
}

#[derive(Clone, Debug)]
struct SegmentOneResult {
    item: SegmentOneItem,
    raise_separator: WritePathRun,
    renew_identity: WritePathRun,
    pair: PairStatistics,
    two_runs_identical: bool,
}

fn segment_one_items() -> Vec<SegmentOneItem> {
    let mut items = Vec::new();
    for history in histories_h51().into_iter().chain(histories_f10()) {
        for leaf_capacity in LEAF_CAPACITIES {
            for merge_threshold in merge_thresholds_of(leaf_capacity) {
                items.push(SegmentOneItem { history, leaf_capacity, merge_threshold });
            }
        }
    }
    items
}

fn run_segment_one_item(item: SegmentOneItem) -> SegmentOneResult {
    let cell_of = |borrow_arm| WritePathCell { borrow_arm, leaf_capacity: item.leaf_capacity, merge_threshold: item.merge_threshold };
    let mut runs = Vec::new();
    let mut two_runs_identical = true;
    for borrow_arm in BORROW_ARMS {
        let first = run_write_path(item.history, cell_of(borrow_arm));
        let second = run_write_path(item.history, cell_of(borrow_arm));
        two_runs_identical &= first == second;
        runs.push(first);
    }
    let renew_identity = runs.pop().expect("N2 那一遍");
    let raise_separator = runs.pop().expect("N1 那一遍");
    let pair = pair_statistics(&raise_separator, &renew_identity);
    SegmentOneResult { item, raise_separator, renew_identity, pair, two_runs_identical }
}

fn write_path_line(item: SegmentOneItem, borrow_arm: BorrowArm, run: &WritePathRun) -> String {
    let cell = WritePathCell { borrow_arm, leaf_capacity: item.leaf_capacity, merge_threshold: item.merge_threshold };
    let today = TombstoneStream::WholeAtIntent.position();
    format!(
        "name=write_path {} {} publishes={} refused_steps={} borrows={} absorbs={} drops={} moved_then_shortened={} moved_then_unlinked={} retirement_records={} units_data={} units_inode_leaf={} units_inode_root={} units_retirement_container={} units_retirement_root={} units_tombstone_container_today={} units_tombstone_root_today={} v7_versions={} v7_i913_red={} v7_today_i94_red={} v7_n1_i94_red={} v7_i912_red={} k6_judged={} k6_violations={} a1_violations={} a8_judged={} a8_violations={} does_not_fit_at={}",
        item.history.label(),
        cell.label(),
        run.publishes,
        run.refused_steps,
        run.borrows,
        run.absorbs,
        run.drops,
        run.moved_then_shortened,
        run.moved_then_unlinked,
        run.written.retirement_records,
        run.written.data_units,
        run.written.inode_leaf_containers,
        run.written.inode_roots,
        run.written.retirement_containers,
        run.written.retirement_roots,
        run.written.tombstone_containers[today],
        run.written.tombstone_roots[today],
        run.self_check.versions,
        run.self_check.leaves_hold_a_record_red,
        run.self_check.today_container_numbers_red,
        run.self_check.raised_separator_container_numbers_red,
        run.self_check.separators_red,
        run.assertions.judged_of("K6_split_right_half_numbered_after_the_record_left_half_unchanged"),
        run.assertions.violations_of("K6_split_right_half_numbered_after_the_record_left_half_unchanged"),
        run.assertions.violations_of("A1_history_publishes") + run.assertions.violations_of("A1_history_refused_steps"),
        run.assertions.judged_of("A8_retirements_equal_identity_disappearances"),
        run.assertions.violations_of("A8_retirements_equal_identity_disappearances"),
        run.does_not_fit_at.map_or_else(|| "none".to_string(), |txg| txg.to_string()),
    )
}

fn borrow_cost_line(result: &SegmentOneResult) -> String {
    let pair = result.pair;
    format!(
        "name=borrow_cost {} capacity={} theta={} borrow_publishes={} {} {} {} {} differences_outside_borrows={} borrow_kinds_disagree={} n2_counted_units={} n1_counted_units={} n2_extra_share_ppm={} user_changes_identical={} publish_counts_identical={} streams_identical={} two_runs_identical={}",
        result.item.history.label(),
        result.item.leaf_capacity,
        result.item.merge_threshold,
        pair.borrow_publishes,
        pair.extra_code_three_per_borrow.fields("extra_code3_per_borrow"),
        pair.extra_code_two_per_borrow.fields("extra_code2_per_borrow"),
        pair.retirement_records_per_borrow_renew_identity.fields("n2_retirements_per_borrow"),
        pair.retirement_records_per_borrow_raise_separator.fields("n1_retirements_per_borrow"),
        pair.publishes_outside_borrows_with_a_difference,
        pair.borrow_kinds_disagree,
        pair.renew_identity_counted_units,
        pair.raise_separator_counted_units,
        pair.extra_share_in_parts_per_million(),
        pair.user_changes_identical,
        pair.publish_counts_identical,
        pair.streams_identical,
        result.two_runs_identical,
    )
}

/// 5.6 全表（两种键布局）：每一形三个谓词的红绿与路由对不对。
fn shape_table_lines() -> (Vec<String>, Vec<(&'static str, &'static str, String, PredicateVerdicts)>) {
    let mut lines = Vec::new();
    let mut verdicts = Vec::new();
    for named in catalogue_of_shapes() {
        for (layout, shape) in [("dense", named.shape.clone()), ("sparse", sparse_layout_of(&named.shape))] {
            let judged = judge_the_shape(&shape);
            let leaves: Vec<String> = shape
                .leaves
                .iter()
                .map(|leaf| {
                    let keys: Vec<String> = leaf.record_keys.iter().map(u64::to_string).collect();
                    format!("{}:{}", leaf.container_number, keys.join("/"))
                })
                .collect();
            let separators: Vec<String> = shape.separators.iter().map(u64::to_string).collect();
            lines.push(format!(
                "name=shape family={} shape={} layout={layout} leaves={} separators={} today_i94={} n1_i94={} i912={} routing={}",
                named.family,
                named.name,
                leaves.join(","),
                separators.join(","),
                red_or_green(judged.today_container_numbers_hold),
                red_or_green(judged.raised_separator_container_numbers_hold),
                red_or_green(judged.separators_hold),
                if judged.routing_holds { "right" } else { "wrong" },
            ));
            verdicts.push((named.family, layout, named.name.clone(), judged));
        }
    }
    (lines, verdicts)
}

/// A6 的一借小历史 MB：容量 16、θ = 8，建 inode 1..32（两片各 16），按升序 unlink 2..10。
fn run_the_single_borrow_micro_history(borrow_arm: BorrowArm) -> Simulation {
    let cell = WritePathCell { borrow_arm, leaf_capacity: TEST_LEAF_CAPACITY, merge_threshold: TEST_LEAF_CAPACITY / 2 };
    let mut simulation = Simulation::start(cell, PRODUCT_UNLINK_BATCH_DATA_UNITS, false);
    simulation.apply(UserChange::NewInodes { count: 31 });
    for object in 2..=10 {
        simulation.apply(UserChange::Unlink { object });
    }
    simulation
}

/// A6 逐条：恰好一次借、挪 5 条、左 12 右 11、右片最小 key 22；N2 − N1 = 码 3 + 1、码 2 + 1、退役记录 + 1；N2 下右片新号 22、出生代 = 那次 txg。
fn single_borrow_anchor_holds() -> (bool, String) {
    let raise_separator = run_the_single_borrow_micro_history(BorrowArm::RaiseSeparator);
    let renew_identity = run_the_single_borrow_micro_history(BorrowArm::RenewIdentity);
    let borrows: Vec<(usize, &PublishRecord)> = renew_identity
        .publish_records
        .iter()
        .enumerate()
        .filter(|(_, record)| matches!(record.merge, MergeAction::BorrowedFromTheRightNeighbour { .. }))
        .collect();
    let other_merges = renew_identity
        .publish_records
        .iter()
        .filter(|record| !matches!(record.merge, MergeAction::NoMerge | MergeAction::BorrowedFromTheRightNeighbour { .. }))
        .count();
    let Some((position, borrow)) = borrows.first().copied() else {
        return (false, "no_borrow".to_string());
    };
    let raised = &raise_separator.publish_records[position];
    let leaves = &renew_identity.version.leaves;
    let sizes: Vec<usize> = leaves.iter().map(|leaf| leaf.records.len()).collect();
    let right_smallest = leaves.get(1).and_then(|leaf| leaf.records.first()).map(|record| record.inode);
    let right_identity = leaves.get(1).map(|leaf| leaf.identity);
    let raised_right_separator = raise_separator.version.leaves.get(1).map(|leaf| leaf.separator);
    let raised_right_number = raise_separator.version.leaves.get(1).map(|leaf| leaf.identity.number);
    let holds = borrows.len() == 1
        && other_merges == 0
        && borrow.merge == MergeAction::BorrowedFromTheRightNeighbour { records_moved: 5 }
        && sizes == vec![12, 11]
        && right_smallest == Some(22)
        && right_identity.is_some_and(|identity| identity.number == 22 && identity.birth == borrow.txg)
        && raised_right_separator == Some(22)
        && raised_right_number == Some(17)
        && borrow.written.code_three_counted() == raised.written.code_three_counted() + 1
        && borrow.written.code_two_counted() == raised.written.code_two_counted() + 1
        && borrow.written.retirement_records == raised.written.retirement_records + 1
        && renew_identity.publish_records.iter().zip(&raise_separator.publish_records).enumerate().all(|(other_position, (renewed, raised_other))| {
            other_position == position || renewed.written == raised_other.written
        });
    let detail = format!(
        "borrows={} other_merges={other_merges} merge={:?} sizes={sizes:?} right_smallest={right_smallest:?} n2_right_identity_number={:?} n1_right_separator={raised_right_separator:?} n1_right_number={raised_right_number:?} extra_code3={} extra_code2={} extra_retirements={}",
        borrows.len(),
        borrow.merge,
        right_identity.map(|identity| identity.number),
        difference_or_marker(borrow.written.code_three_counted(), raised.written.code_three_counted()),
        difference_or_marker(borrow.written.code_two_counted(), raised.written.code_two_counted()),
        difference_or_marker(borrow.written.retirement_records, raised.written.retirement_records),
    );
    (holds, detail)
}

/// 停机对拍 S1：末尾分裂（`crates/singlefs-core/src/inode_tree.rs:181-283`）——满了才分裂、右半号 = 触发那条记录的号、出生代 = 本次 txg、
/// 左半不动、新号只落最右叶。在容量 16 与 233 上各分裂一次。
fn stop_check_split_matches_the_implementation() -> (bool, String) {
    let mut every_capacity_holds = true;
    let mut details = Vec::new();
    for leaf_capacity in [TEST_LEAF_CAPACITY, INODE_LEAF_RECORDS] {
        let cell = WritePathCell { borrow_arm: BorrowArm::RaiseSeparator, leaf_capacity, merge_threshold: 0 };
        let mut simulation = Simulation::start(cell, PRODUCT_UNLINK_BATCH_DATA_UNITS, false);
        simulation.apply(UserChange::NewInodes { count: leaf_capacity - 1 });
        let one_leaf_before = simulation.version.leaves.len() == 1 && length_as_u64(simulation.version.leaves[0].records.len()) == leaf_capacity;
        let left_before = simulation.version.leaves[0].clone();
        simulation.apply(UserChange::NewInodes { count: 1 });
        let split_txg = simulation.version.txg;
        let leaves = &simulation.version.leaves;
        let holds = one_leaf_before
            && leaves.len() == 2
            && leaves[0].identity == left_before.identity
            && leaves[0].records == left_before.records
            && leaves[0].unit == left_before.unit
            && leaves[1].identity == ContainerIdentity::inode_leaf(leaf_capacity + 1, split_txg)
            && leaves[1].separator == leaf_capacity + 1
            && leaves[1].records.len() == 1
            && simulation.publish_records.last().is_some_and(|record| record.written.inode_leaf_containers == 1 && record.written.inode_roots == 1);
        every_capacity_holds &= holds;
        details.push(format!("capacity_{leaf_capacity}={holds}"));
    }
    (every_capacity_holds, details.join(","))
}

/// 停机对拍 S2：删一条（`inode_tree.rs:286-318`）——删完还有记录只重写那片；删空不落盘、右边各片左移且全部重写、根照样重写；删光交回零片。
fn stop_check_removal_matches_the_implementation() -> (bool, String) {
    let cell = WritePathCell { borrow_arm: BorrowArm::RaiseSeparator, leaf_capacity: TEST_LEAF_CAPACITY, merge_threshold: 0 };
    let mut simulation = Simulation::start(cell, PRODUCT_UNLINK_BATCH_DATA_UNITS, false);
    simulation.apply(UserChange::NewInodes { count: 47 });
    simulation.apply(UserChange::Unlink { object: 20 });
    let partial = written_and_merge_of_the_last_publish(&simulation);
    for object in (17..=32).filter(|object| *object != 20) {
        simulation.apply(UserChange::Unlink { object });
    }
    let emptied = written_and_merge_of_the_last_publish(&simulation);
    let leaves_after_emptying: Vec<u64> = simulation.version.leaves.iter().map(|leaf| leaf.identity.number).collect();
    let mut last_record = Simulation::start(cell, PRODUCT_UNLINK_BATCH_DATA_UNITS, false);
    last_record.apply(UserChange::Unlink { object: FIRST_FILE_INODE });
    let gone = written_and_merge_of_the_last_publish(&last_record);
    let holds = partial.is_some_and(|(written, merge)| written.inode_leaf_containers == 1 && written.inode_roots == 1 && merge == MergeAction::NoMerge)
        && emptied.is_some_and(|(written, merge)| written.inode_leaf_containers == 1 && written.inode_roots == 1 && merge == MergeAction::EmptiedAndDropped)
        && leaves_after_emptying == vec![1, 33]
        && gone.is_some_and(|(written, merge)| written.inode_leaf_containers == 0 && written.inode_roots == 1 && merge == MergeAction::EmptiedAndDropped)
        && last_record.version.leaves.is_empty();
    (holds, format!("partial={partial:?} emptied={emptied:?} leaves_after_emptying={leaves_after_emptying:?} last_record={gone:?}"))
}

fn written_and_merge_of_the_last_publish(simulation: &Simulation) -> Option<(UnitsWritten, MergeAction)> {
    simulation.publish_records.last().map(|record| (record.written, record.merge))
}

/// 停机对拍 S3：unlink 的步（`crates/singlefs-core/src/unlink.rs:118-130`）——剩下 > b ⇒ 留 剩下 − b、size = 留下 × 32634；剩下 ≤ b ⇒ 最后一批。
fn stop_check_unlink_steps_match_the_implementation() -> (bool, String) {
    let cell = WritePathCell { borrow_arm: BorrowArm::RaiseSeparator, leaf_capacity: INODE_LEAF_RECORDS, merge_threshold: 0 };
    let mut simulation = Simulation::start(cell, PRODUCT_UNLINK_BATCH_DATA_UNITS, false);
    let steps_hold = [(300, Some(156)), (145, Some(1)), (144, None), (1, None), (0, None)].iter().all(|(remaining, kept)| {
        match (simulation.unlink_step_for(FIRST_FILE_INODE, *remaining), kept) {
            (Operation::UnlinkBatch { data_units_kept, .. }, Some(expected_kept)) => data_units_kept == *expected_kept,
            (Operation::UnlinkFinal { .. }, None) => true,
            (
                Operation::UnlinkBatch { .. }
                | Operation::UnlinkFinal { .. }
                | Operation::FirstFile
                | Operation::RowPublish
                | Operation::WarmUp
                | Operation::WriteFile { .. }
                | Operation::NewInodes { .. }
                | Operation::UnlinkIntent { .. }
                | Operation::RollBack { .. },
                Some(_) | None,
            ) => false,
        }
    });
    simulation.apply(UserChange::WriteFile { object: FIRST_FILE_INODE, data_units: 300 });
    simulation.publish(&Operation::UnlinkIntent { object: FIRST_FILE_INODE });
    let size_at_the_intent = simulation.version.record_of(FIRST_FILE_INODE).map(|record| record.size_bytes);
    let first_batch = simulation.next_unlink_step(FIRST_FILE_INODE);
    simulation.publish(&first_batch);
    let after_the_first_batch = simulation.version.record_of(FIRST_FILE_INODE);
    let holds = steps_hold
        && size_at_the_intent == Some(300 * DATA_UNIT_PAYLOAD_BYTES)
        && after_the_first_batch.is_some_and(|record| record.size_bytes == 156 * DATA_UNIT_PAYLOAD_BYTES && record.nlink == 0)
        && simulation.version.data_units_of(FIRST_FILE_INODE) == 156;
    (holds, format!("steps_hold={steps_hold} size_at_the_intent={size_at_the_intent:?} after_the_first_batch={after_the_first_batch:?}"))
}

/// 停机对拍 S6：K-0 至 K-5 的判定与 `crates/` 那几条测试断言的相同——K-0 全绿；K-1 至 K-5 只红 I-9.4（I-9.12 绿）。
fn stop_check_known_bad_images_match_the_implementation() -> (bool, String) {
    let mut details = Vec::new();
    let mut every_shape_holds = true;
    for named in catalogue_of_shapes().into_iter().filter(|named| named.family == "K") {
        let verdicts = judge_the_shape(&named.shape);
        let holds = if named.name == "K-0" {
            verdicts.today_container_numbers_hold && verdicts.separators_hold && every_leaf_holds_a_record(&named.shape)
        } else {
            !verdicts.today_container_numbers_hold && verdicts.separators_hold && every_leaf_holds_a_record(&named.shape)
        };
        every_shape_holds &= holds;
        details.push(format!("{}={holds}", named.name));
    }
    (every_shape_holds, details.join(","))
}

/// 第七节 K1–K3、A2 的常量锚点。
fn constant_anchor_lines() -> Vec<String> {
    vec![
        format!("name=anchor id=K1 inode_leaf_records={INODE_LEAF_RECORDS} expected=233 holds={}", INODE_LEAF_RECORDS == 233),
        format!(
            "name=anchor id=K2 tombstone_container_records={TOMBSTONE_CONTAINER_RECORDS} expected=815 holds={}",
            TOMBSTONE_CONTAINER_RECORDS == 815
        ),
        format!(
            "name=anchor id=K3 root_node_leaf_containers={ROOT_NODE_LEAF_CONTAINERS} expected=135 holds={}",
            ROOT_NODE_LEAF_CONTAINERS == 135
        ),
        format!(
            "name=anchor id=A2 retirement_record_bytes={RETIREMENT_RECORD_BYTES} retirement_container_records={RETIREMENT_CONTAINER_RECORDS} expected=34,959 holds={}",
            RETIREMENT_RECORD_BYTES == 34 && RETIREMENT_CONTAINER_RECORDS == 959
        ),
    ]
}

/// 工作项按片并行，产物按工作项的次序收（与线程数无关）。
fn run_in_parallel<Item: Copy + Send + Sync, Output: Send>(
    items: &[Item],
    threads: usize,
    work: impl Fn(Item) -> Output + Sync,
) -> Vec<Output> {
    let next_item = AtomicUsize::new(0);
    let collected: Mutex<Vec<Option<Output>>> = Mutex::new((0..items.len()).map(|_| None).collect());
    std::thread::scope(|scope| {
        for _ in 0..threads.max(1) {
            scope.spawn(|| loop {
                let position = next_item.fetch_add(1, Ordering::SeqCst);
                if position >= items.len() {
                    break;
                }
                let output = work(items[position]);
                collected.lock().expect("收结果的锁没有中毒：工作线程不在持锁时 panic")[position] = Some(output);
            });
        }
    });
    collected
        .into_inner()
        .expect("收结果的锁没有中毒")
        .into_iter()
        .map(|output| output.expect("每个工作项都收回了"))
        .collect()
}

/// V7 要全绿的那几句（第十一节 V7）：θ = 0 与 N2 按今天的 I-9.4 与 I-9.12，N1 按 N1 的 I-9.4 与 I-9.12；I-9.13 都要绿。
fn self_check_holds(borrow_arm: BorrowArm, merge_threshold: u64, tally: SelfCheckTally) -> bool {
    let today_required = merge_threshold == 0 || borrow_arm == BorrowArm::RenewIdentity;
    let raised_required = borrow_arm == BorrowArm::RaiseSeparator;
    tally.leaves_hold_a_record_red == 0
        && tally.separators_red == 0
        && (!today_required || tally.today_container_numbers_red == 0)
        && (!raised_required || tally.raised_separator_container_numbers_red == 0)
}

/// F10 θ > 0 那一格算不算结果（第十一节 V6）：借、「被挪过的对象之后变短」「被挪过的对象之后被 unlink」三个次数都不为 0。
const fn path_was_covered(run: &WritePathRun) -> bool {
    run.borrows > 0 && run.moved_then_shortened > 0 && run.moved_then_unlinked > 0
}

fn segment_one_cell_verdict_lines(results: &[SegmentOneResult]) -> Vec<String> {
    let mut lines = Vec::new();
    for leaf_capacity in LEAF_CAPACITIES {
        for merge_threshold in merge_thresholds_of(leaf_capacity) {
            let in_cell: Vec<&SegmentOneResult> = results
                .iter()
                .filter(|result| result.item.leaf_capacity == leaf_capacity && result.item.merge_threshold == merge_threshold)
                .collect();
            let mut extra_code_three = Extremes::default();
            let mut extra_code_two = Extremes::default();
            let mut renew_retirements = Extremes::default();
            let mut raise_retirements = Extremes::default();
            let mut extra_share = Extremes::default();
            let mut borrow_publishes = 0;
            let mut outside_differences = 0;
            let mut kinds_disagree = 0;
            let mut streams_identical = 0;
            for result in &in_cell {
                let pair = result.pair;
                borrow_publishes += pair.borrow_publishes;
                extra_code_three.absorb(pair.extra_code_three_per_borrow);
                extra_code_two.absorb(pair.extra_code_two_per_borrow);
                renew_retirements.absorb(pair.retirement_records_per_borrow_renew_identity);
                raise_retirements.absorb(pair.retirement_records_per_borrow_raise_separator);
                outside_differences += pair.publishes_outside_borrows_with_a_difference;
                kinds_disagree += pair.borrow_kinds_disagree;
                streams_identical += u64::from(pair.streams_identical);
                if pair.borrow_publishes > 0 {
                    extra_share.observe(pair.extra_share_in_parts_per_million());
                }
            }
            let every_extra_is_one = |extremes: Extremes| extremes.observations == 0 || (extremes.smallest == Some(1) && extremes.largest == Some(1));
            let definition_holds = every_extra_is_one(extra_code_three)
                && every_extra_is_one(extra_code_two)
                && outside_differences == 0
                && kinds_disagree == 0;
            let zero_threshold_streams_identical = merge_threshold != 0 || (borrow_publishes == 0 && streams_identical == length_as_u64(in_cell.len()));
            lines.push(format!(
                "name=verdict question=Q-X4-1 capacity={leaf_capacity} theta={merge_threshold} histories={} borrow_publishes={borrow_publishes} {} {} {} differences_outside_borrows={outside_differences} borrow_kinds_disagree={kinds_disagree} a4_a6_definition_holds={definition_holds} a5_zero_threshold_streams_identical={zero_threshold_streams_identical} judgement=reported",
                in_cell.len(),
                extra_code_three.fields("extra_code3_per_borrow"),
                extra_code_two.fields("extra_code2_per_borrow"),
                extra_share.fields("n2_extra_share_ppm_over_histories_with_borrows"),
            ));
            let retirements_hold = (renew_retirements.observations == 0 || (renew_retirements.smallest == Some(1) && renew_retirements.largest == Some(1)))
                && (raise_retirements.observations == 0 || raise_retirements.largest == Some(0));
            lines.push(format!(
                "name=verdict question=Q-X4-2 capacity={leaf_capacity} theta={merge_threshold} {} {} definition_holds={retirements_hold} judgement=reported",
                renew_retirements.fields("n2_retirements_per_borrow"),
                raise_retirements.fields("n1_retirements_per_borrow"),
            ));
            for family in ["F1", "F8", "F3", "F10"] {
                let in_family: Vec<&&SegmentOneResult> = in_cell.iter().filter(|result| result.item.history.family() == family).collect();
                let mut totals = [0_u64; 7];
                let mut not_covered = 0;
                let mut did_not_fit = 0;
                let mut arms_disagree = 0;
                for result in &in_family {
                    let renew = &result.renew_identity;
                    let raise = &result.raise_separator;
                    for (position, value) in [renew.borrows, renew.absorbs, renew.drops, renew.moved_then_shortened, renew.moved_then_unlinked, renew.publishes, renew.written.retirement_records]
                        .iter()
                        .enumerate()
                    {
                        totals[position] += value;
                    }
                    arms_disagree += u64::from((renew.borrows, renew.absorbs, renew.drops) != (raise.borrows, raise.absorbs, raise.drops));
                    did_not_fit += u64::from(renew.does_not_fit_at.is_some() || raise.does_not_fit_at.is_some());
                    if family == "F10" && merge_threshold > 0 && !path_was_covered(renew) {
                        not_covered += 1;
                    }
                }
                lines.push(format!(
                    "name=verdict question=Q-X4-3 capacity={leaf_capacity} theta={merge_threshold} family={family} segments={} borrows={} absorbs={} drops={} moved_then_shortened={} moved_then_unlinked={} publishes={} n2_retirement_records={} merge_counts_differ_between_n1_n2={arms_disagree} does_not_fit_segments={did_not_fit} v6_segments_without_coverage={not_covered} judgement=reported",
                    in_family.len(),
                    totals[0],
                    totals[1],
                    totals[2],
                    totals[3],
                    totals[4],
                    totals[5],
                    totals[6],
                ));
            }
            for borrow_arm in BORROW_ARMS {
                let mut tally = SelfCheckTally::default();
                for result in &in_cell {
                    let run = match borrow_arm {
                        BorrowArm::RaiseSeparator => &result.raise_separator,
                        BorrowArm::RenewIdentity => &result.renew_identity,
                    };
                    tally.add(run.self_check);
                }
                lines.push(format!(
                    "name=verdict question=V7 capacity={leaf_capacity} theta={merge_threshold} borrow_arm={} versions={} i913_red={} today_i94_red={} n1_i94_red={} i912_red={} required_green_holds={}",
                    borrow_arm.label(),
                    tally.versions,
                    tally.leaves_hold_a_record_red,
                    tally.today_container_numbers_red,
                    tally.raised_separator_container_numbers_red,
                    tally.separators_red,
                    self_check_holds(borrow_arm, merge_threshold, tally),
                ));
            }
        }
    }
    lines
}

/// Q-X4-4 至 Q-X4-7：按键布局、按族列「今天 I-9.4 红而 N1 的 I-9.4 绿」的形态（并上 I-9.12 前后）与路由错的。
fn shape_verdict_lines(verdicts: &[(&'static str, &'static str, String, PredicateVerdicts)]) -> Vec<String> {
    let mut lines = Vec::new();
    for layout in ["dense", "sparse"] {
        for family in ["K", "S", "BN1", "BN2", "DV", "all"] {
            let in_scope: Vec<&(&'static str, &'static str, String, PredicateVerdicts)> = verdicts
                .iter()
                .filter(|(shape_family, shape_layout, _, _)| *shape_layout == layout && (family == "all" || *shape_family == family))
                .collect();
            let missed_by_raised_separators: Vec<&str> = in_scope
                .iter()
                .filter(|(_, _, _, judged)| !judged.today_container_numbers_hold && judged.raised_separator_container_numbers_hold)
                .map(|(_, _, name, _)| name.as_str())
                .collect();
            let missed_with_separators: Vec<&str> = in_scope
                .iter()
                .filter(|(_, _, _, judged)| {
                    let today_with_separators = judged.today_container_numbers_hold && judged.separators_hold;
                    let raised_with_separators = judged.raised_separator_container_numbers_hold && judged.separators_hold;
                    !today_with_separators && raised_with_separators
                })
                .map(|(_, _, name, _)| name.as_str())
                .collect();
            let missed_and_misrouted: Vec<&str> = in_scope
                .iter()
                .filter(|(_, _, _, judged)| {
                    !judged.today_container_numbers_hold && judged.raised_separator_container_numbers_hold && !judged.routing_holds
                })
                .map(|(_, _, name, _)| name.as_str())
                .collect();
            let shown = |names: &[&str]| if names.is_empty() { "none".to_string() } else { names.join(",") };
            lines.push(format!(
                "name=verdict question=Q-X4-4 layout={layout} family={family} shapes={} today_red_n1_green={} names={} n1_misses_shapes={}",
                in_scope.len(),
                missed_by_raised_separators.len(),
                shown(&missed_by_raised_separators),
                !missed_by_raised_separators.is_empty(),
            ));
            lines.push(format!(
                "name=verdict question=Q-X4-5 layout={layout} family={family} shapes={} today_or_i912_red_n1_and_i912_green={} names={} i912_does_not_cover_the_misses={}",
                in_scope.len(),
                missed_with_separators.len(),
                shown(&missed_with_separators),
                !missed_with_separators.is_empty(),
            ));
            lines.push(format!(
                "name=verdict question=Q-X4-6 layout={layout} family={family} misses_with_wrong_routing={} names={} n1_misses_hurt_lookup={}",
                missed_and_misrouted.len(),
                shown(&missed_and_misrouted),
                !missed_and_misrouted.is_empty(),
            ));
            let count_red = |predicate: fn(&PredicateVerdicts) -> bool| in_scope.iter().filter(|(_, _, _, judged)| !predicate(judged)).count();
            lines.push(format!(
                "name=verdict question=Q-X4-7 layout={layout} family={family} shapes={} today_i94_red={} n1_i94_red={} i912_red={} routing_wrong={} judgement=table_listed_in_shape_lines",
                in_scope.len(),
                count_red(|judged| judged.today_container_numbers_hold),
                count_red(|judged| judged.raised_separator_container_numbers_hold),
                count_red(|judged| judged.separators_hold),
                count_red(|judged| judged.routing_holds),
            ));
        }
    }
    let dense: Vec<&(&'static str, &'static str, String, PredicateVerdicts)> =
        verdicts.iter().filter(|(_, layout, _, _)| *layout == "dense").collect();
    let differing: Vec<&str> = dense
        .iter()
        .filter(|(family, _, name, judged)| {
            verdicts
                .iter()
                .find(|(other_family, layout, other_name, _)| *layout == "sparse" && other_family == family && other_name == name)
                .is_some_and(|(_, _, _, sparse_judged)| sparse_judged != judged)
        })
        .map(|(_, _, name, _)| name.as_str())
        .collect();
    lines.push(format!(
        "name=verdict question=G-K shapes={} verdicts_differ_between_layouts={} names={} stable={}",
        dense.len(),
        differing.len(),
        if differing.is_empty() { "none".to_string() } else { differing.join(",") },
        differing.is_empty(),
    ));
    lines
}

/// PC-K：今天的 I-9.4 在 K-1 至 K-5 全红、K-0 全绿；N1 的谓词在 BN1「分隔 key 没抬」那一形（s = 20）红。
fn positive_control_shape_lines() -> Vec<String> {
    let catalogue = catalogue_of_shapes();
    let verdict_of = |name: &str| {
        catalogue.iter().find(|named| named.name == name).map(|named| judge_the_shape(&named.shape)).expect("形态表里有它")
    };
    let today_reddens_k1_to_k5 = ["K-1", "K-2", "K-3", "K-4", "K-5"].iter().all(|name| !verdict_of(name).today_container_numbers_hold);
    let k0 = verdict_of("K-0");
    let k0_green = k0.today_container_numbers_hold && k0.raised_separator_container_numbers_hold && k0.separators_hold;
    let raised_reddens_unraised = !verdict_of("BN1-s20").raised_separator_container_numbers_hold;
    vec![format!(
        "name=positive_control id=PC-K today_i94_reddens_k1_to_k5={today_reddens_k1_to_k5} k0_all_green={k0_green} n1_i94_reddens_bn1_s20={raised_reddens_unraised} holds={}",
        today_reddens_k1_to_k5 && k0_green && raised_reddens_unraised
    )]
}

fn run_segment_one(threads: usize) -> Vec<String> {
    let items = segment_one_items();
    let mut lines = vec![format!(
        "name=config experiment=E178 segment=one work_items={} histories_h51={} histories_f10={} capacities=233,16 thetas=0,quarter,half borrow_arms=N1,N2 inode_leaf_records={INODE_LEAF_RECORDS} tombstone_container_records={TOMBSTONE_CONTAINER_RECORDS} retirement_container_records={RETIREMENT_CONTAINER_RECORDS} root_node_leaf_containers={ROOT_NODE_LEAF_CONTAINERS} data_unit_payload_bytes={DATA_UNIT_PAYLOAD_BYTES} scanning=none",
        items.len(),
        histories_h51().len(),
        histories_f10().len(),
    )];
    lines.extend(constant_anchor_lines());
    let stop_checks = [
        ("S1", stop_check_split_matches_the_implementation()),
        ("S2", stop_check_removal_matches_the_implementation()),
        ("S3", stop_check_unlink_steps_match_the_implementation()),
        ("S6", stop_check_known_bad_images_match_the_implementation()),
    ];
    for (name, (holds, detail)) in &stop_checks {
        lines.push(format!("name=stop_check id={name} holds={holds} detail={detail}"));
    }
    let (shape_lines, shape_verdicts) = shape_table_lines();
    lines.extend(shape_lines);
    let results = run_in_parallel(&items, threads, run_segment_one_item);
    let mut tally = AssertionTally::default();
    let mut every_run_identical = true;
    let mut user_changes_identical = true;
    let mut zero_threshold_streams_identical = true;
    for result in &results {
        lines.push(write_path_line(result.item, BorrowArm::RaiseSeparator, &result.raise_separator));
        lines.push(write_path_line(result.item, BorrowArm::RenewIdentity, &result.renew_identity));
        lines.push(borrow_cost_line(result));
        tally.merge(&result.raise_separator.assertions);
        tally.merge(&result.renew_identity.assertions);
        every_run_identical &= result.two_runs_identical;
        user_changes_identical &= result.pair.user_changes_identical && result.pair.publish_counts_identical;
        if result.item.merge_threshold == 0 {
            zero_threshold_streams_identical &= result.pair.streams_identical && result.pair.borrow_publishes == 0;
        }
    }
    let (single_borrow_holds, single_borrow_detail) = single_borrow_anchor_holds();
    for name in [
        "A1_history_publishes",
        "A1_history_refused_steps",
        "A8_retirements_equal_identity_disappearances",
        "K6_split_right_half_numbered_after_the_record_left_half_unchanged",
    ] {
        lines.push(format!(
            "name=assertion assertion={name} judged={} violated={}",
            tally.judged_of(name),
            tally.violations_of(name)
        ));
    }
    lines.push(format!("name=anchor id=A5 zero_threshold_streams_identical={zero_threshold_streams_identical} holds={zero_threshold_streams_identical}"));
    lines.push(format!("name=anchor id=A6 {single_borrow_detail} holds={single_borrow_holds}"));
    lines.push(format!(
        "name=positive_control id=PC-N zero_borrow_histories_identical={zero_threshold_streams_identical} single_borrow_history_holds={single_borrow_holds} holds={}",
        zero_threshold_streams_identical && single_borrow_holds
    ));
    lines.extend(positive_control_shape_lines());
    lines.extend(segment_one_cell_verdict_lines(&results));
    lines.extend(shape_verdict_lines(&shape_verdicts));
    let a1_violations = tally.violations_of("A1_history_publishes") + tally.violations_of("A1_history_refused_steps");
    let k_violations = tally.violations_of("K6_split_right_half_numbered_after_the_record_left_half_unchanged");
    let constants_hold = INODE_LEAF_RECORDS == 233 && TOMBSTONE_CONTAINER_RECORDS == 815 && ROOT_NODE_LEAF_CONTAINERS == 135;
    let anchors_hold = RETIREMENT_CONTAINER_RECORDS == 959
        && zero_threshold_streams_identical
        && single_borrow_holds
        && tally.violations_of("A8_retirements_equal_identity_disappearances") == 0;
    let stop_checks_hold = stop_checks.iter().all(|(_, (holds, _))| *holds);
    lines.push(format!(
        "name=verdict question=summary v1_history_replay_holds={} v4_two_runs_identical={every_run_identical} v4_user_changes_identical_between_n1_n2={user_changes_identical} v5_anchors_hold={anchors_hold} fl0_clause_constants_and_k6_hold={} stop_checks_hold={stop_checks_hold}",
        a1_violations == 0,
        constants_hold && k_violations == 0,
    ));
    lines
}

// ---------------------------------------------------------------------------------------------
// 阳性对照（登记 5.5）：改读者输入、主路径与参照路径各算一遍增量，两路相同才算（第十一节 V2）
// ---------------------------------------------------------------------------------------------

/// 第二遍在哪几次扫描上做植入类的对照（第一遍的扫描记录定）：最后一次有变短过的活对象（PC-Z、PC-T）、
/// 最后一次在 unlink 途中（PC-EL）、最后一次最后一批之后紧接的那次（PC-ELr）。
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
struct PositiveControlPlan {
    shrunk_object_scan: Option<u64>,
    inside_unlink_scan: Option<u64>,
    after_final_batch_scan: Option<u64>,
}

fn positive_control_plan_of(scans: &[ScanRecord]) -> PositiveControlPlan {
    let shrunk_object_scan = scans.iter().rev().find(|record| record.live_shrunk_object.is_some()).map(|record| record.publish_sequence);
    let inside_unlink_scan = scans.iter().rev().find(|record| record.inside_an_unlink).map(|record| record.publish_sequence);
    let after_final_batch_scan = scans
        .windows(2)
        .rev()
        .find(|adjacent| adjacent[0].inside_an_unlink && !adjacent[1].inside_an_unlink)
        .map(|adjacent| adjacent[1].publish_sequence);
    PositiveControlPlan { shrunk_object_scan, inside_unlink_scan, after_final_batch_scan }
}

const COUNT_FIELDS: usize = 8;

const fn counts_as_array(counts: ReaderCounts) -> [u64; COUNT_FIELDS] {
    [
        counts.inode_resurrected,
        counts.inode_missed,
        counts.extent_resurrected,
        counts.extent_missed,
        counts.container_resurrected,
        counts.container_missed,
        counts.live_records_beyond_the_truth,
        counts.records_whose_intent_is_published,
    ]
}

/// 增量的下标：inode r、inode m、extent r、extent m、container r、container m、O1、O2。
const DELTA_INODE_R: usize = 0;
const DELTA_INODE_M: usize = 1;
const DELTA_EXTENT_R: usize = 2;
const DELTA_EXTENT_M: usize = 3;
const DELTA_CONTAINER_R: usize = 4;
const DELTA_CONTAINER_M: usize = 5;
const DELTA_O1: usize = 6;
const DELTA_O2: usize = 7;

fn signed(value: u64) -> i64 {
    i64::try_from(value).expect("计数装得进 i64")
}

fn deltas_of(before: &[ReaderCounts], after: &[ReaderCounts]) -> Vec<[i64; COUNT_FIELDS]> {
    before
        .iter()
        .zip(after)
        .map(|(baseline, modified)| {
            let baseline_values = counts_as_array(*baseline);
            let modified_values = counts_as_array(*modified);
            let mut delta = [0_i64; COUNT_FIELDS];
            for position in 0..COUNT_FIELDS {
                delta[position] = signed(modified_values[position]) - signed(baseline_values[position]);
            }
            delta
        })
        .collect()
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct PositiveControlOutcome {
    id: &'static str,
    at_publish: u64,
    detail: String,
    /// 增量逐个对应的读者（第二、三段是 `every_reader` 的 20 个）。
    readers: Vec<Reader>,
    main_deltas: Vec<[i64; COUNT_FIELDS]>,
    paths_agree: bool,
    closed_form_holds: bool,
}

impl PositiveControlOutcome {
    fn line(&self, context: &str) -> String {
        let per_reader: Vec<String> = self
            .readers
            .iter()
            .zip(&self.main_deltas)
            .map(|(reader, delta)| {
                format!(
                    "{}/{}/{}/{}:{}",
                    reader.base.label(),
                    reader.shrink.label(),
                    reader.effect.label(),
                    reader.window.label(),
                    delta.iter().map(i64::to_string).collect::<Vec<String>>().join("/")
                )
            })
            .collect();
        format!(
            "name=positive_control id={} {context} at_publish={} {} paths_agree={} closed_form_holds={} deltas_inode_r_m_extent_r_m_container_r_m_o1_o2={}",
            self.id,
            self.at_publish,
            self.detail,
            self.paths_agree,
            self.closed_form_holds,
            per_reader.join(",")
        )
    }
}

impl Simulation {
    /// 写序最大的植入（读法 9）。
    const fn newest_write_order(&self) -> WriteOrder {
        WriteOrder { birth_txg: self.version.txg, instance: self.version.instance, transaction: self.version.transaction }
    }

    /// 植入的记录的改动计数取植入写序的诞生 txg（读法 11），P-C 下与 P-W 下都由它供出（写序最大、改动计数不小于盘上任何一条）。
    fn planted_leaf(planted_record: InodeRecord, write_order: WriteOrder, scan_publish: u64) -> Unit {
        let record = InodeRecord {
            inode: planted_record.inode,
            object_birth: planted_record.object_birth,
            nlink: planted_record.nlink,
            size_bytes: planted_record.size_bytes,
            change_counter: write_order.birth_txg,
        };
        Unit {
            write_order,
            written_at_publish: scan_publish,
            replaced_at_publish: None,
            body: UnitBody::InodeLeaf {
                identity: ContainerIdentity {
                    birth_tree: PLANTED_BIRTH_TREE,
                    record_type: PACKED_TYPE_INODE,
                    number: record.inode,
                    birth: write_order.birth_txg,
                },
                records: Arc::new(vec![record]),
            },
        }
    }

    /// 一个对照：主路径（按组索引）与参照路径（整扫）各算改动前后，增量逐读者逐量相同才算两路对得上；`closed_form` 判登记写的期望。
    fn positive_control(
        &self,
        id: &'static str,
        overlay: &Overlay,
        scan_publish: u64,
        detail: String,
        closed_form: &dyn Fn(&[Reader], &[[i64; COUNT_FIELDS]]) -> bool,
    ) -> PositiveControlOutcome {
        self.positive_control_on(&self.readers.clone(), id, overlay, scan_publish, detail, closed_form)
    }

    /// 同 `positive_control`，读者表另给（PC-Bs2 在 B-stub 写路径上另读 B-stub-lin-off）。
    fn positive_control_on(
        &self,
        readers: &[Reader],
        id: &'static str,
        overlay: &Overlay,
        scan_publish: u64,
        detail: String,
        closed_form: &dyn Fn(&[Reader], &[[i64; COUNT_FIELDS]]) -> bool,
    ) -> PositiveControlOutcome {
        let baseline_main = self.read_readers(readers, &Overlay::default(), scan_publish, SelectionPath::GroupIndex);
        let baseline_reference = self.read_readers(readers, &Overlay::default(), scan_publish, SelectionPath::FullScan);
        let modified_main = self.read_readers(readers, overlay, scan_publish, SelectionPath::GroupIndex);
        let modified_reference = self.read_readers(readers, overlay, scan_publish, SelectionPath::FullScan);
        let main_deltas = deltas_of(&baseline_main, &modified_main);
        let reference_deltas = deltas_of(&baseline_reference, &modified_reference);
        let paths_agree = baseline_main == baseline_reference && main_deltas == reference_deltas;
        let closed_form_holds = closed_form(readers, &main_deltas);
        PositiveControlOutcome { id, at_publish: scan_publish, detail, readers: readers.to_vec(), main_deltas, paths_agree, closed_form_holds }
    }

    /// PC-Z 的手算：k ∈ [o 现在的单元数, 变短前单元数) 里，那一组有一个候选版本（读得出、已发布、在谱系上）的个数，逐单元数、不走择新。
    fn shrunk_units_with_a_candidate(&self, object: u64, object_birth: u64, data_units_range: (u64, u64), window: ReadableWindow) -> u64 {
        let overlay = Overlay::default();
        let inputs = self.scan_inputs(&overlay, self.publish_sequence);
        let mut found: BTreeSet<u64> = BTreeSet::new();
        for unit in &self.disk.units {
            if let Some((unit_object, unit_object_birth, unit_index_in_file)) = data_key_of(unit) {
                let in_range = unit_index_in_file >= data_units_range.0 && unit_index_in_file < data_units_range.1;
                if unit_object == object && unit_object_birth == object_birth && in_range && is_candidate(unit, &inputs, window) {
                    found.insert(unit_index_in_file);
                }
            }
        }
        length_as_u64(found.len())
    }

    fn run_the_planned_positive_controls(&mut self, publish_sequence: u64) {
        let plan = self.positive_control_plan;
        let mut outcomes = Vec::new();
        if plan.shrunk_object_scan == Some(publish_sequence) {
            outcomes.extend(self.shrink_positive_controls(publish_sequence));
        }
        if plan.inside_unlink_scan == Some(publish_sequence) {
            outcomes.extend(self.unlink_effect_positive_control(publish_sequence));
        }
        if plan.after_final_batch_scan == Some(publish_sequence) {
            outcomes.extend(self.deferred_effect_positive_control(publish_sequence));
        }
        self.positive_control_outcomes.extend(outcomes);
    }

    /// PC-Z 与 PC-T（登记 5.5）。
    fn shrink_positive_controls(&self, scan_publish: u64) -> Vec<PositiveControlOutcome> {
        let Some(object) = self.live_shrunk_object() else {
            return Vec::new();
        };
        let Some(record) = self.version.record_of(object) else {
            return Vec::new();
        };
        let size_before = self.largest_size_before_a_shrink.get(&object).copied().unwrap_or(0);
        let range = (self.version.data_units_of(object), size_before.div_ceil(DATA_UNIT_PAYLOAD_BYTES));
        let hand_counts = [
            self.shrunk_units_with_a_candidate(object, record.object_birth, range, ReadableWindow::Forever),
            self.shrunk_units_with_a_candidate(object, record.object_birth, range, ReadableWindow::RingWindow),
        ];
        let planted = InodeRecord { inode: object, object_birth: record.object_birth, nlink: 1, size_bytes: size_before, change_counter: 0 };
        let overlay = Overlay::planting(vec![Self::planted_leaf(planted, self.newest_write_order(), scan_publish)]);
        let shrink_closed_form = |readers: &[Reader], deltas: &[[i64; COUNT_FIELDS]]| {
            readers.iter().zip(deltas).all(|(reader, delta)| {
                let extent_expected = match reader.shrink {
                    ShrinkArm::RecordSize => match reader.window {
                        ReadableWindow::Forever => signed(hand_counts[0]),
                        ReadableWindow::RingWindow => signed(hand_counts[1]),
                    },
                    ShrinkArm::RangeTombstone | ShrinkArm::ObjectGateOnly => 0,
                };
                delta[DELTA_CONTAINER_R] == 1 && delta[DELTA_EXTENT_R] == extent_expected
            })
        };
        let mut outcomes = vec![self.positive_control(
            "PC-Z",
            &overlay,
            scan_publish,
            format!("object={object} data_units_now={} data_units_before_shrink={} hand_count_inf={} hand_count_23={}", range.0, range.1, hand_counts[0], hand_counts[1]),
            &shrink_closed_form,
        )];
        let range_tombstone = self.version.tombstone_trees[TombstoneStream::WholeAtIntentRangesOnUserShrinks.position()]
            .containers
            .iter()
            .flat_map(|container| container.records.iter().copied())
            .filter(|tombstone| tombstone.object == object && tombstone.object_birth == record.object_birth && tombstone.range_start_bytes > 0)
            .last();
        if let Some(removed) = range_tombstone {
            let overlay = Overlay::removing_a_tombstone(removed);
            let range_closed_form = |readers: &[Reader], deltas: &[[i64; COUNT_FIELDS]]| {
                readers.iter().zip(deltas).all(|(reader, delta)| reader.shrink == ShrinkArm::RangeTombstone || delta[DELTA_EXTENT_R] == 0)
            };
            outcomes.push(self.positive_control(
                "PC-T",
                &overlay,
                scan_publish,
                format!("object={object} removed_range_start={} removed_death_txg={}", removed.range_start_bytes, removed.death_txg),
                &range_closed_form,
            ));
        }
        outcomes
    }

    /// PC-EL：unlink 途中植入一个全新身份的墓碑容器（各条写流各一个），装 (N, b, [0, 2⁶⁴ − 1], d = 这一版 txg)。
    fn unlink_effect_positive_control(&self, scan_publish: u64) -> Vec<PositiveControlOutcome> {
        let Some(object) = self.version.deleted_queue else {
            return Vec::new();
        };
        let Some(record) = self.version.record_of(object) else {
            return Vec::new();
        };
        let remaining = signed(self.version.data_units_of(object));
        let whole = TombstoneRecord {
            object,
            object_birth: record.object_birth,
            range_start_bytes: WHOLE_OBJECT_RANGE_START_BYTES,
            death_txg: self.version.txg,
        };
        let planted = TOMBSTONE_STREAMS
            .iter()
            .map(|stream| Unit {
                write_order: self.newest_write_order(),
                written_at_publish: scan_publish,
                replaced_at_publish: None,
                body: UnitBody::Tombstones {
                    stream: *stream,
                    identity: ContainerIdentity { birth_tree: PLANTED_BIRTH_TREE, record_type: PACKED_TYPE_TOMBSTONE, number: 1, birth: self.version.txg },
                    records: Arc::new(vec![whole]),
                },
            })
            .collect();
        let overlay = Overlay::planting(planted);
        let closed_form = |readers: &[Reader], deltas: &[[i64; COUNT_FIELDS]]| {
            readers.iter().zip(deltas).all(|(reader, delta)| match reader.effect {
                UnlinkEffectArm::AtLastBatchWritten => delta[DELTA_INODE_M] == 1 && delta[DELTA_EXTENT_M] == remaining,
                UnlinkEffectArm::AtIntent | UnlinkEffectArm::AtLastBatchRead => delta[DELTA_INODE_M] == 0 && delta[DELTA_EXTENT_M] == 0,
            })
        };
        vec![self.positive_control("PC-EL", &overlay, scan_publish, format!("object={object} data_units_remaining={remaining}"), &closed_form)]
    }

    /// PC-ELr：最后一批之后紧接的那次扫描，植入全新身份、已发布、在谱系上、诞生 txg < N 的意图 txg 的类型 2 容器版本，装 (N, b, nlink 1)。
    fn deferred_effect_positive_control(&self, scan_publish: u64) -> Vec<PositiveControlOutcome> {
        let Some(record) = self.publish_records.last() else {
            return Vec::new();
        };
        let unlinked = self
            .intent_txgs
            .iter()
            .filter(|(object, _)| !self.version.is_live(**object) && self.version.record_of(**object).is_none())
            .max_by_key(|(_, intent_txg)| **intent_txg)
            .map(|(object, intent_txg)| (*object, *intent_txg));
        let Some((object, intent_txg)) = unlinked else {
            return Vec::new();
        };
        let Some(object_birth) = self.object_birth_seen_on_disk(object) else {
            return Vec::new();
        };
        let write_order = WriteOrder { birth_txg: intent_txg - 1, instance: self.version.instance, transaction: 0 };
        let planted = InodeRecord { inode: object, object_birth, nlink: 1, size_bytes: 0, change_counter: 0 };
        let overlay = Overlay::planting(vec![Self::planted_leaf(planted, write_order, scan_publish)]);
        let closed_form = |readers: &[Reader], deltas: &[[i64; COUNT_FIELDS]]| {
            readers.iter().zip(deltas).all(|(reader, delta)| {
                reader.window == ReadableWindow::RingWindow
                    || match reader.effect {
                        UnlinkEffectArm::AtLastBatchRead => delta[DELTA_INODE_R] == 1,
                        UnlinkEffectArm::AtIntent | UnlinkEffectArm::AtLastBatchWritten => delta[DELTA_INODE_R] == 0,
                    }
            })
        };
        vec![self.positive_control(
            "PC-ELr",
            &overlay,
            scan_publish,
            format!("object={object} intent_txg={intent_txg} planted_birth_txg={} last_publish_txg={}", intent_txg - 1, record.txg),
            &closed_form,
        )]
    }

    fn object_birth_seen_on_disk(&self, object: u64) -> Option<u64> {
        self.disk.units.iter().rev().find_map(|unit| match &unit.body {
            UnitBody::InodeLeaf { records, .. } => records.iter().find(|record| record.inode == object).map(|record| record.object_birth),
            UnitBody::Data { .. } | UnitBody::Tombstones { .. } | UnitBody::Retirements { .. } => None,
        })
    }

    /// PC-M：抹掉一个现行的活数据单元（取组里只有它一版的那个，没有就取号最小的），再另做一次抹掉现行的第一片叶容器。
    fn erasure_positive_controls(&self, scan_publish: u64) -> Vec<PositiveControlOutcome> {
        let mut outcomes = Vec::new();
        let live_data_units: Vec<UnitIndex> = self
            .version
            .files
            .iter()
            .filter(|(object, _)| self.version.is_live(**object))
            .flat_map(|(_, file)| file.units.iter().copied())
            .collect();
        let single_version = live_data_units.iter().copied().find(|unit_index| {
            let key = self.disk.units[*unit_index].body.group_key();
            self.disk.group_of_key.get(&key).is_some_and(|group| self.disk.versions_of_group[*group].len() == 1)
        });
        if let Some(erased) = single_version.or_else(|| live_data_units.first().copied()) {
            let overlay = Overlay::erasing(erased);
            let closed_form = |_: &[Reader], deltas: &[[i64; COUNT_FIELDS]]| deltas.iter().all(|delta| delta[DELTA_EXTENT_M] <= 1);
            outcomes.push(self.positive_control("PC-M-data", &overlay, scan_publish, format!("erased_unit={erased} single_version={}", single_version.is_some()), &closed_form));
        }
        if let Some(leaf) = self.version.leaves.first() {
            let overlay = Overlay::erasing(leaf.unit);
            let records = signed(length_as_u64(leaf.records.len()));
            let closed_form = |_: &[Reader], deltas: &[[i64; COUNT_FIELDS]]| deltas.iter().all(|delta| delta[DELTA_INODE_M] <= records);
            outcomes.push(self.positive_control("PC-M-leaf", &overlay, scan_publish, format!("erased_unit={} records={records}", leaf.unit), &closed_form));
        }
        outcomes
    }

    /// 第十一节 V3：这次扫描上主路径与参照路径逐读者逐量相同。
    fn group_index_equals_the_full_scan(&self, scan_publish: u64) -> bool {
        self.read_every_reader(&Overlay::default(), scan_publish, SelectionPath::GroupIndex)
            == self.read_every_reader(&Overlay::default(), scan_publish, SelectionPath::FullScan)
    }
}

// ---------------------------------------------------------------------------------------------
// 第二段（登记 5.8）：H51、F10、F11 全部读者每次扫描都读；Q-B1、Q-B2、Q-X2-1 至 Q-X2-7、Q-X3-1 至 Q-X3-3
// ---------------------------------------------------------------------------------------------

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct SegmentTwoItem {
    history: History,
    cell: WritePathCell,
}

fn histories_f11() -> Vec<History> {
    GAP_EVENTS.iter().map(|event| History::RollBackOverAGap { event: *event }).collect()
}

fn segment_two_items() -> Vec<SegmentTwoItem> {
    let mut items = Vec::new();
    for history in histories_h51().into_iter().chain(histories_f10()).chain(histories_f11()) {
        for leaf_capacity in LEAF_CAPACITIES {
            for merge_threshold in merge_thresholds_of(leaf_capacity) {
                for borrow_arm in BORROW_ARMS {
                    items.push(SegmentTwoItem { history, cell: WritePathCell { borrow_arm, leaf_capacity, merge_threshold } });
                }
            }
        }
    }
    items
}

fn reader_position(wanted: Reader) -> usize {
    every_reader().iter().position(|reader| *reader == wanted).expect("读者表里有它")
}

/// Q-X2-6、Q-X2-7：同一段历史、同一次发布、同一条 X3 臂下，T 那条墓碑写流比 Z 那条多写的码 3、码 2（第一节读法 3）。
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
struct ExtraWrites {
    extra_code_three: Extremes,
    extra_code_two: Extremes,
    multi_write_publishes: u64,
    publishes: u64,
    extra_without_one_and_one: u64,
}

fn extra_writes_of(publish_records: &[PublishRecord], effect: UnlinkEffectArm) -> ExtraWrites {
    let with_ranges = tombstone_stream_of(ShrinkArm::RangeTombstone, effect).position();
    let without_ranges = tombstone_stream_of(ShrinkArm::RecordSize, effect).position();
    let mut extra = ExtraWrites::default();
    for record in publish_records {
        extra.publishes += 1;
        let code_three = difference_or_marker(record.written.tombstone_containers[with_ranges], record.written.tombstone_containers[without_ranges]);
        let code_two = difference_or_marker(record.written.tombstone_roots[with_ranges], record.written.tombstone_roots[without_ranges]);
        if code_three > 0 || code_two > 0 {
            extra.multi_write_publishes += 1;
            extra.extra_code_three.observe(code_three);
            extra.extra_code_two.observe(code_two);
            extra.extra_without_one_and_one += u64::from(code_three != 1 || code_two != 1);
        }
    }
    extra
}

/// A4 的 F8 逐段值（第七节；`anchors.out` 的 `T_extra_*` 列）：E-I 下多写发布数 = [删前覆盖写且 n_u > 1]，E-L 两臂下再加不是最后一批的批数。
fn expected_extra_write_publishes(history: History, effect: UnlinkEffectArm) -> Option<u64> {
    let History::LargeFileRewriteAndDelete { data_units, overwrite_before_unlink, batch, .. } = history else {
        return None;
    };
    let shrink_publishes = u64::from(overwrite_before_unlink && data_units > 1);
    let data_units_at_the_intent = if overwrite_before_unlink { 1 } else { data_units };
    let batches_that_are_not_the_last = data_units_at_the_intent.div_ceil(batch).max(1) - 1;
    Some(match effect {
        UnlinkEffectArm::AtIntent => shrink_publishes,
        UnlinkEffectArm::AtLastBatchWritten | UnlinkEffectArm::AtLastBatchRead => shrink_publishes + batches_that_are_not_the_last,
    })
}

#[derive(Clone, Debug)]
struct SegmentTwoResult {
    item: SegmentTwoItem,
    trajectories: Vec<ReaderTrajectory>,
    extra_writes: Vec<(UnlinkEffectArm, ExtraWrites)>,
    positive_controls: Vec<PositiveControlOutcome>,
    rollback: Option<RollbackOutcome>,
    assertions: AssertionTally,
    lines: Vec<String>,
    two_runs_identical: bool,
    last_scan_paths_agree: bool,
    path_covered: bool,
}

/// 每段第二遍跑完之后判的断言与对照（A4、A7、A9、K4、PC-B1、PC-G0、PC-B2），记进断言账、写一行产物。
fn segment_two_checks(item: SegmentTwoItem, simulation: &Simulation, rollback: Option<RollbackOutcome>, assertions: &mut AssertionTally) -> Vec<String> {
    let mut lines = Vec::new();
    let label = format!("{} {}", item.history.label(), item.cell.label());
    let last_counts = simulation.scans.last().map(|record| record.counts.clone()).unwrap_or_default();
    // PC-B1：B-off-ret 在 W = ∞ 下期末的容器 r = 身份消失次数（期末不在真值里的退役过的身份数）；W = 23 下每次身份消失那次扫描容器 r ≥ 1。
    let final_identities: HashSet<ContainerIdentity> = simulation.version.leaves.iter().map(|leaf| leaf.identity).collect();
    let disappeared: BTreeSet<ContainerIdentity> = simulation
        .publish_records
        .iter()
        .flat_map(|record| record.retired_identities.iter().copied())
        .filter(|identity| !final_identities.contains(identity))
        .collect();
    let without_retirement_forever = reader_position(Reader { base: BaseArm::WithoutRetirement, shrink: ShrinkArm::RecordSize, effect: UnlinkEffectArm::AtIntent, window: ReadableWindow::Forever });
    let without_retirement_ring = reader_position(Reader { base: BaseArm::WithoutRetirement, shrink: ShrinkArm::RecordSize, effect: UnlinkEffectArm::AtIntent, window: ReadableWindow::RingWindow });
    let final_container_r = last_counts.get(without_retirement_forever).map_or(0, |counts| counts.container_resurrected);
    let disappearance_scans_without_r = simulation
        .publish_records
        .iter()
        .zip(&simulation.scans)
        .filter(|(record, _)| !record.retired_identities.is_empty())
        .filter(|(_, scan)| scan.counts[without_retirement_ring].container_resurrected == 0)
        .count();
    if !disappeared.is_empty() {
        let holds = final_container_r == length_as_u64(disappeared.len()) && disappearance_scans_without_r == 0;
        assertions.judge("PC-B1_without_retirement_resurrects_every_disappeared_identity", holds);
        lines.push(format!(
            "name=positive_control id=PC-B1 {label} disappeared_identities={} final_container_r_inf={final_container_r} disappearance_scans_without_r_23={disappearance_scans_without_r} holds={holds}",
            disappeared.len()
        ));
        if let (History::BuildAndDeleteEmptyInodes { inode_count, .. }, 0) = (item.history, item.cell.merge_threshold) {
            let expected = inode_count.div_ceil(item.cell.leaf_capacity) - 1;
            assertions.judge("A9_without_retirement_final_container_r", final_container_r == expected);
        }
    }
    // PC-G0（A3）：F8 删前覆盖写、n_u > 1 的段，覆盖写之后那次扫描 G0 的 extent r = n_u − 1。
    if let History::LargeFileRewriteAndDelete { data_units, overwrite_before_unlink: true, .. } = item.history {
        if data_units > 1 {
            let overwrite_scan = simulation.publish_records.iter().zip(&simulation.scans).find(|(record, _)| record.shrink.is_some()).map(|(_, scan)| scan);
            let observed: Vec<u64> = overwrite_scan.map_or_else(Vec::new, |scan| {
                every_reader()
                    .iter()
                    .zip(&scan.counts)
                    .filter(|(reader, _)| reader.shrink == ShrinkArm::ObjectGateOnly)
                    .map(|(_, counts)| counts.extent_resurrected)
                    .collect()
            });
            let holds = !observed.is_empty() && observed.iter().all(|value| *value == data_units - 1);
            assertions.judge("A3_object_gate_only_resurrects_the_overwritten_units", holds);
            lines.push(format!("name=positive_control id=PC-G0 {label} expected={} observed={observed:?} holds={holds}", data_units - 1));
        }
    }
    // A4：T 每次多写的发布多写码 3 一个、码 2 一个；F8 的多写发布数逐段等于登记的式子。
    for effect in UNLINK_EFFECT_ARMS {
        let extra = extra_writes_of(&simulation.publish_records, effect);
        assertions.judge("A4_extra_writes_are_one_and_one", extra.extra_without_one_and_one == 0);
        if let Some(expected) = expected_extra_write_publishes(item.history, effect) {
            assertions.judge("A4_extra_write_publishes", extra.multi_write_publishes == expected);
        }
    }
    // A7：F8（不删前覆盖写）E-I 下 unlink 途中每次扫描 inode m = 1、extent m = 这一版剩下的单元数（B-on 的四个 E-I 主格读者）。
    if let History::LargeFileRewriteAndDelete { overwrite_before_unlink: false, .. } = item.history {
        let intent_readers: Vec<usize> = every_reader()
            .iter()
            .enumerate()
            .filter(|(_, reader)| reader.is_a_main_cell() && reader.effect == UnlinkEffectArm::AtIntent)
            .map(|(position, _)| position)
            .collect();
        let mut sequence = Vec::new();
        for scan in simulation.scans.iter().filter(|scan| scan.inside_an_unlink) {
            let counts = scan.counts[intent_readers[0]];
            sequence.push(counts.extent_missed);
            let every_reader_agrees = intent_readers.iter().all(|position| scan.counts[*position].inode_missed == 1 && scan.counts[*position].extent_missed == counts.extent_missed);
            assertions.judge("A7_intent_effect_misses_the_record_and_every_remaining_unit", every_reader_agrees);
        }
        lines.push(format!("name=anchor id=A7 {label} in_window_scans={} extent_m_sequence_head={:?}", sequence.len(), sequence.iter().take(3).collect::<Vec<&u64>>()));
    }
    // K4：E-I、E-L-w 下每次 unlink 恰好一条整对象墓碑。
    let unlinks = length_as_u64(simulation.intent_txgs.len());
    let whole_at_intent = simulation.version.tombstone_trees[TombstoneStream::WholeAtIntent.position()].record_count();
    let whole_at_final = simulation.version.tombstone_trees[TombstoneStream::WholeAtFinal.position()].record_count();
    if rollback.is_none() {
        assertions.judge("K4_one_whole_object_tombstone_per_unlink", whole_at_intent == unlinks && whole_at_final == unlinks);
    }
    // PC-B2：F11 回退之后 B-off-lin 的容器 m 与缺口里退役、而在 R_old 里有的身份个数并排报（不当门槛，两路对拍在 PC 行与末次扫描上）。
    if let Some(outcome) = rollback {
        let lineage_ignored = reader_position(Reader { base: BaseArm::RetirementIgnoringLineage, shrink: ShrinkArm::RecordSize, effect: UnlinkEffectArm::AtIntent, window: ReadableWindow::Forever });
        let on_lineage = reader_position(Reader { base: BaseArm::RetirementOnLineage, shrink: ShrinkArm::RecordSize, effect: UnlinkEffectArm::AtIntent, window: ReadableWindow::Forever });
        let after = |position: usize| simulation.scans.iter().filter(|scan| scan.after_a_rollback).map(|scan| scan.counts[position].container_missed).max().unwrap_or(0);
        lines.push(format!(
            "name=positive_control id=PC-B2 {label} event_found={} rollback_distance={} too_far={} expected_from_the_write_path={} b_off_lin_container_m_peak_after_rollback={} b_on_container_m_peak_after_rollback={} retirement_records_after_rollback={}",
            outcome.event_found,
            outcome.distance_in_publishes,
            outcome.too_far,
            outcome.retired_in_the_gap_and_present_in_the_old_version,
            after(lineage_ignored),
            after(on_lineage),
            outcome.retirement_records_after_the_rollback,
        ));
    }
    lines
}

fn run_segment_two_item(item: SegmentTwoItem) -> SegmentTwoResult {
    let (first, _) = simulate(item.history, item.cell, true);
    let plan = positive_control_plan_of(&first.scans);
    let first_scans = first.scans;
    let mut second = Simulation::start(item.cell, item.history.batch(), true);
    second.positive_control_plan = plan;
    let rollback = run_history(item.history, &mut second);
    let two_runs_identical = second.scans == first_scans;
    let last_publish = second.publish_sequence;
    let last_scan_paths_agree = second.group_index_equals_the_full_scan(last_publish);
    let mut positive_controls = second.positive_control_outcomes.clone();
    positive_controls.extend(second.erasure_positive_controls(last_publish));
    let mut assertions = second.assertions.clone();
    let readers = every_reader();
    let trajectories = trajectories_of(&second.scans, readers.len());
    let extra_writes: Vec<(UnlinkEffectArm, ExtraWrites)> =
        UNLINK_EFFECT_ARMS.iter().map(|effect| (*effect, extra_writes_of(&second.publish_records, *effect))).collect();
    let label = format!("{} {}", item.history.label(), item.cell.label());
    let (borrows, absorbs, drops) = second.count_merges();
    let path_covered = second.moved_then_shortened > 0 && second.moved_then_unlinked > 0 && borrows > 0;
    let mut lines = vec![format!(
        "name=simulation {label} publishes={} scans={} refused_steps={} borrows={borrows} absorbs={absorbs} drops={drops} moved_then_shortened={} moved_then_unlinked={} retirement_records={} does_not_fit_at={} two_runs_identical={two_runs_identical} last_scan_paths_agree={last_scan_paths_agree}",
        second.publishes,
        second.scans.len(),
        second.refused_steps,
        second.moved_then_shortened,
        second.moved_then_unlinked,
        second.total_written().retirement_records,
        second.does_not_fit_at.map_or_else(|| "none".to_string(), |txg| txg.to_string()),
    )];
    let scopes_reported = if rollback.is_some() { SCAN_SCOPES.len() } else { 3 };
    for (reader, trajectory) in readers.iter().zip(&trajectories) {
        for (scope_position, scope) in SCAN_SCOPES.iter().enumerate().take(scopes_reported) {
            let fields: Vec<String> = COUNTED_QUANTITIES
                .iter()
                .zip(&trajectory.by_scope[scope_position])
                .map(|(quantity, quantity_trajectory)| quantity_trajectory.fields(quantity))
                .collect();
            lines.push(format!(
                "name=trajectory {label} {} scope={scope} scans={} {} read_out_empty_scans={}",
                reader.label(),
                trajectory.by_scope[scope_position][0].scans,
                fields.join(" "),
                trajectory.read_out_empty_scans
            ));
        }
    }
    for (effect, extra) in &extra_writes {
        lines.push(format!(
            "name=extra_writes {label} effect={} multi_write_publishes={} publishes={} {} {} not_one_and_one={}",
            effect.label(),
            extra.multi_write_publishes,
            extra.publishes,
            extra.extra_code_three.fields("extra_code3"),
            extra.extra_code_two.fields("extra_code2"),
            extra.extra_without_one_and_one,
        ));
    }
    for outcome in &positive_controls {
        lines.push(outcome.line(&label));
    }
    lines.extend(segment_two_checks(item, &second, rollback, &mut assertions));
    SegmentTwoResult {
        item,
        trajectories,
        extra_writes,
        positive_controls,
        rollback,
        assertions,
        lines,
        two_runs_identical,
        last_scan_paths_agree,
        path_covered,
    }
}

/// 判定格：写路径格 × 批大小（第八节的 W、θ、容量、b 都在这一格的键里或读者里）。
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
struct VerdictKey {
    cell: WritePathCell,
    batch: u64,
}

fn aggregate_trajectories(results: &[SegmentTwoResult], family_filter: &dyn Fn(&str) -> bool) -> BTreeMap<VerdictKey, Vec<ReaderTrajectory>> {
    let reader_count = every_reader().len();
    let mut aggregated: BTreeMap<VerdictKey, Vec<ReaderTrajectory>> = BTreeMap::new();
    for result in results.iter().filter(|result| family_filter(result.item.history.family())) {
        let key = VerdictKey { cell: result.item.cell, batch: result.item.history.batch() };
        let entry = aggregated.entry(key).or_insert_with(|| vec![ReaderTrajectory::default(); reader_count]);
        for (aggregate, trajectory) in entry.iter_mut().zip(&result.trajectories) {
            for scope_position in 0..SCAN_SCOPES.len() {
                for quantity_position in 0..COUNTED_QUANTITIES.len() {
                    aggregate.by_scope[scope_position][quantity_position].absorb(trajectory.by_scope[scope_position][quantity_position]);
                }
            }
            aggregate.read_out_empty_scans += trajectory.read_out_empty_scans;
        }
    }
    aggregated
}

const SCOPE_ALL: usize = 0;
const SCOPE_INSIDE_UNLINK: usize = 1;
const SCOPE_OUTSIDE_UNLINK: usize = 2;
const SCOPE_AFTER_ROLLBACK_OUTSIDE_UNLINK: usize = 4;
const QUANTITY_INODE_R: usize = 0;
const QUANTITY_INODE_M: usize = 1;
const QUANTITY_EXTENT_R: usize = 2;
const QUANTITY_EXTENT_M: usize = 3;
const QUANTITY_CONTAINER_R: usize = 4;
const QUANTITY_CONTAINER_M: usize = 5;

fn theta_class_of(cell: WritePathCell) -> usize {
    merge_thresholds_of(cell.leaf_capacity).iter().position(|threshold| *threshold == cell.merge_threshold).expect("θ 取自三档之一")
}

/// 一个带门槛的判定，记下它所在的取样点，供第八节的几何敏感性逐对比。
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
struct JudgedStatus {
    question_and_arms: String,
    borrow_arm: BorrowArm,
    leaf_capacity: u64,
    theta_class: usize,
    batch: u64,
    window: ReadableWindow,
    status: &'static str,
}

/// 第六节的门槛：峰值大于它就判那一格「复活 / 误判死亡 / 漏活」（第八节「判定函数把门槛当参数」）。
const JUDGEMENT_THRESHOLD: u64 = 0;

const fn status_above(peak: u64, threshold: u64, positive: &'static str) -> &'static str {
    if peak > threshold {
        positive
    } else {
        "zero"
    }
}

const fn status_by_threshold(peak: u64, positive: &'static str) -> &'static str {
    status_above(peak, JUDGEMENT_THRESHOLD, positive)
}

fn segment_two_verdict_lines(results: &[SegmentTwoResult]) -> (Vec<String>, Vec<JudgedStatus>) {
    let mut lines = Vec::new();
    let mut statuses = Vec::new();
    let all_families = aggregate_trajectories(results, &|_| true);
    let rollback_only = aggregate_trajectories(results, &|family| family == "F11");
    for (key, trajectories) in &all_families {
        let key_label = format!("{} batch={}", key.cell.label(), key.batch);
        for window in READABLE_WINDOWS {
            let mut judged = |question_and_arms: String, status: &'static str| {
                statuses.push(JudgedStatus {
                    question_and_arms,
                    borrow_arm: key.cell.borrow_arm,
                    leaf_capacity: key.cell.leaf_capacity,
                    theta_class: theta_class_of(key.cell),
                    batch: key.batch,
                    window,
                    status,
                });
            };
            let base_reader = reader_position(Reader { base: BaseArm::RetirementOnLineage, shrink: ShrinkArm::RecordSize, effect: UnlinkEffectArm::AtIntent, window });
            let base_container_r = trajectories[base_reader].by_scope[SCOPE_ALL][QUANTITY_CONTAINER_R];
            let q_b1 = status_by_threshold(base_container_r.peak, "resurrects");
            judged("Q-B1".to_string(), q_b1);
            let rollback_trajectories = rollback_only.get(key);
            let after_rollback = |quantity: usize| {
                rollback_trajectories.map_or_else(QuantityTrajectory::default, |trajectories| trajectories[base_reader].by_scope[SCOPE_AFTER_ROLLBACK_OUTSIDE_UNLINK][quantity])
            };
            let q_b2 = status_by_threshold(after_rollback(QUANTITY_CONTAINER_M).peak, "misjudges_death");
            if rollback_trajectories.is_some() {
                judged("Q-B2".to_string(), q_b2);
            }
            let excluded_by_fl1 = q_b1 != "zero" || (rollback_trajectories.is_some() && q_b2 != "zero");
            lines.push(format!(
                "name=verdict question=Q-B1 {key_label} window={} {} status={q_b1}",
                window.label(),
                base_container_r.fields("container_r")
            ));
            lines.push(format!(
                "name=verdict question=Q-B2 {key_label} window={} rollback_histories={} {} {} status={}",
                window.label(),
                rollback_trajectories.is_some(),
                after_rollback(QUANTITY_CONTAINER_M).fields("container_m"),
                after_rollback(QUANTITY_INODE_M).fields("inode_m"),
                if rollback_trajectories.is_some() { q_b2 } else { "not_run" }
            ));
            for effect in UNLINK_EFFECT_ARMS {
                for shrink in [ShrinkArm::RecordSize, ShrinkArm::RangeTombstone] {
                    let reader = Reader { base: BaseArm::RetirementOnLineage, shrink, effect, window };
                    let trajectory = &trajectories[reader_position(reader)];
                    let arms = format!("shrink={} effect={}", shrink.label(), effect.label());
                    let resurrection = trajectory.by_scope[SCOPE_ALL][QUANTITY_EXTENT_R];
                    let resurrection_question = if shrink == ShrinkArm::RecordSize { "Q-X2-1" } else { "Q-X2-2" };
                    let resurrection_status = status_by_threshold(resurrection.peak, "resurrects");
                    judged(format!("{resurrection_question} {arms}"), resurrection_status);
                    lines.push(format!(
                        "name=verdict question={resurrection_question} {key_label} {} {} {} status={resurrection_status} fl1_cell_excluded={excluded_by_fl1}",
                        reader.label(),
                        resurrection.fields("extent_r"),
                        trajectory.by_scope[SCOPE_ALL][QUANTITY_INODE_R].fields("inode_r"),
                    ));
                    let misjudged = trajectory.by_scope[SCOPE_OUTSIDE_UNLINK][QUANTITY_EXTENT_M];
                    let misjudged_question = if shrink == ShrinkArm::RecordSize { "Q-X2-3" } else { "Q-X2-4" };
                    let misjudged_status = status_by_threshold(misjudged.peak, "misjudges_death");
                    judged(format!("{misjudged_question} {arms}"), misjudged_status);
                    lines.push(format!(
                        "name=verdict question={misjudged_question} {key_label} {} {} {} status={misjudged_status} fl1_cell_excluded={excluded_by_fl1}",
                        reader.label(),
                        misjudged.fields("extent_m_outside_unlink"),
                        trajectory.by_scope[SCOPE_OUTSIDE_UNLINK][QUANTITY_INODE_M].fields("inode_m_outside_unlink"),
                    ));
                    let inside_extent_m = trajectory.by_scope[SCOPE_INSIDE_UNLINK][QUANTITY_EXTENT_M];
                    let inside_inode_m = trajectory.by_scope[SCOPE_INSIDE_UNLINK][QUANTITY_INODE_M];
                    if effect != UnlinkEffectArm::AtIntent {
                        let batch_status = status_by_threshold(inside_extent_m.peak, "misjudges_in_batches");
                        judged(format!("Q-X2-5 {arms}"), batch_status);
                        lines.push(format!(
                            "name=verdict question=Q-X2-5 {key_label} {} {} status={batch_status} fl1_cell_excluded={excluded_by_fl1}",
                            reader.label(),
                            inside_extent_m.fields("extent_m_inside_unlink"),
                        ));
                    }
                    for (question, quantity) in [("Q-X3-1", inside_inode_m), ("Q-X3-2", inside_extent_m)] {
                        let status = if effect == UnlinkEffectArm::AtIntent { "reported" } else { status_by_threshold(quantity.peak, "leaks") };
                        if effect != UnlinkEffectArm::AtIntent {
                            judged(format!("{question} {arms}"), status);
                        }
                        lines.push(format!(
                            "name=verdict question={question} {key_label} {} {} status={status} fl1_cell_excluded={excluded_by_fl1}",
                            reader.label(),
                            quantity.fields("inside_unlink"),
                        ));
                    }
                    lines.push(format!(
                        "name=verdict question=Q-X3-3 {key_label} {} {} {} status=incidental",
                        reader.label(),
                        trajectory.by_scope[SCOPE_INSIDE_UNLINK][QUANTITY_INODE_R].fields("inode_r_inside_unlink"),
                        trajectory.by_scope[SCOPE_INSIDE_UNLINK][QUANTITY_EXTENT_R].fields("extent_r_inside_unlink"),
                    ));
                }
            }
        }
    }
    (lines, statuses)
}

/// Q-X2-6、Q-X2-7：T 每次多写的单元数（每份），与多写发布占比（按族、X3 臂分列；比例按百万分之一计）。
fn extra_write_verdict_lines(results: &[SegmentTwoResult]) -> Vec<String> {
    let mut lines = Vec::new();
    let mut by_cell_family: BTreeMap<(WritePathCell, &'static str, UnlinkEffectArm), ExtraWrites> = BTreeMap::new();
    for result in results {
        for (effect, extra) in &result.extra_writes {
            let entry = by_cell_family.entry((result.item.cell, result.item.history.family(), *effect)).or_default();
            entry.extra_code_three.absorb(extra.extra_code_three);
            entry.extra_code_two.absorb(extra.extra_code_two);
            entry.multi_write_publishes += extra.multi_write_publishes;
            entry.publishes += extra.publishes;
            entry.extra_without_one_and_one += extra.extra_without_one_and_one;
        }
    }
    let mut by_cell: BTreeMap<(WritePathCell, UnlinkEffectArm), ExtraWrites> = BTreeMap::new();
    for ((cell, family, effect), extra) in &by_cell_family {
        let share = if extra.publishes == 0 { 0 } else { extra.multi_write_publishes * 1_000_000 / extra.publishes };
        lines.push(format!(
            "name=verdict question=Q-X2-7 {} family={family} effect={} multi_write_publishes={} publishes={} share_ppm={share} judgement=reported",
            cell.label(),
            effect.label(),
            extra.multi_write_publishes,
            extra.publishes,
        ));
        let entry = by_cell.entry((*cell, *effect)).or_default();
        entry.extra_code_three.absorb(extra.extra_code_three);
        entry.extra_code_two.absorb(extra.extra_code_two);
        entry.multi_write_publishes += extra.multi_write_publishes;
        entry.publishes += extra.publishes;
        entry.extra_without_one_and_one += extra.extra_without_one_and_one;
    }
    for ((cell, effect), extra) in &by_cell {
        lines.push(format!(
            "name=verdict question=Q-X2-6 {} effect={} {} {} not_one_and_one={} judgement=reported",
            cell.label(),
            effect.label(),
            extra.extra_code_three.fields("extra_code3_per_multi_write_publish"),
            extra.extra_code_two.fields("extra_code2_per_multi_write_publish"),
            extra.extra_without_one_and_one,
        ));
    }
    lines
}

/// 第八节：同一个判定在一对方向相反的取样点上翻不翻面（W、θ 四分与二分、容量 233 与 16、b 1 与 144）。
fn stability_lines(statuses: &[JudgedStatus]) -> Vec<String> {
    let mut by_question: BTreeMap<&str, Vec<&JudgedStatus>> = BTreeMap::new();
    for status in statuses {
        by_question.entry(status.question_and_arms.as_str()).or_default().push(status);
    }
    let mut lines = Vec::new();
    for (question_and_arms, judged) in by_question {
        let flips_along = |differs_only_in_the_knob: &dyn Fn(&JudgedStatus, &JudgedStatus) -> bool| {
            let mut pairs = 0_u64;
            let mut flips = 0_u64;
            for first in &judged {
                for second in &judged {
                    if first < second && differs_only_in_the_knob(first, second) {
                        pairs += 1;
                        flips += u64::from(first.status != second.status);
                    }
                }
            }
            (pairs, flips)
        };
        let same_except = |first: &JudgedStatus, second: &JudgedStatus, window: bool, theta: bool, capacity: bool, batch: bool| {
            first.borrow_arm == second.borrow_arm
                && (window || first.window == second.window)
                && (theta || first.theta_class == second.theta_class)
                && (capacity || first.leaf_capacity == second.leaf_capacity)
                && (batch || first.batch == second.batch)
        };
        let window_flips = flips_along(&|first, second| same_except(first, second, true, false, false, false) && first.window != second.window);
        let theta_flips = flips_along(&|first, second| {
            same_except(first, second, false, true, false, false) && first.theta_class == 1 && second.theta_class == 2
        });
        let capacity_flips = flips_along(&|first, second| same_except(first, second, false, false, true, false) && first.leaf_capacity != second.leaf_capacity);
        let batch_flips = flips_along(&|first, second| same_except(first, second, false, false, false, true) && first.batch != second.batch);
        let every_status: BTreeSet<&str> = judged.iter().map(|status| status.status).collect();
        let stable = window_flips.1 == 0 && theta_flips.1 == 0 && capacity_flips.1 == 0 && batch_flips.1 == 0;
        lines.push(format!(
            "name=stability question={question_and_arms} statuses={} window_pairs={} window_flips={} theta_pairs={} theta_flips={} capacity_pairs={} capacity_flips={} batch_pairs={} batch_flips={} stable={stable}",
            every_status.iter().copied().collect::<Vec<&str>>().join(","),
            window_flips.0,
            window_flips.1,
            theta_flips.0,
            theta_flips.1,
            capacity_flips.0,
            capacity_flips.1,
            batch_flips.0,
            batch_flips.1,
        ));
    }
    lines
}

fn run_segment_two(threads: usize) -> Vec<String> {
    let items = segment_two_items();
    let mut lines = vec![format!(
        "name=config experiment=E178 segment=two work_items={} histories_h51={} histories_f10={} histories_f11={} readers={} capacities=233,16 thetas=0,quarter,half borrow_arms=N1,N2 windows=inf,23 ring_window={READABLE_WINDOW_RING_PUBLISHES}",
        items.len(),
        histories_h51().len(),
        histories_f10().len(),
        histories_f11().len(),
        every_reader().len(),
    )];
    lines.extend(constant_anchor_lines());
    for (name, (holds, detail)) in [
        ("S4", stop_check_intent_matches_the_implementation()),
        ("S5", stop_check_tombstone_tree_matches_the_implementation()),
    ] {
        lines.push(format!("name=stop_check id={name} holds={holds} detail={detail}"));
    }
    let (direct_holds, direct_detail) = direct_rollback_positive_control_holds();
    lines.push(format!("name=positive_control id=PC-B2-direct {direct_detail} holds={direct_holds}"));
    let results = run_in_parallel(&items, threads, run_segment_two_item);
    let mut tally = AssertionTally::default();
    let mut every_run_identical = true;
    let mut every_path_agrees = true;
    let mut positive_controls_agree = true;
    let mut positive_controls_closed_form = true;
    let mut uncovered_f10_cells = 0;
    for result in &results {
        lines.extend(result.lines.iter().cloned());
        tally.merge(&result.assertions);
        every_run_identical &= result.two_runs_identical;
        every_path_agrees &= result.last_scan_paths_agree;
        positive_controls_agree &= result.positive_controls.iter().all(|outcome| outcome.paths_agree);
        positive_controls_closed_form &= result.positive_controls.iter().all(|outcome| outcome.closed_form_holds);
        if result.item.history.family() == "F10" && result.item.cell.merge_threshold > 0 && !result.path_covered {
            uncovered_f10_cells += 1;
        }
    }
    positive_controls_closed_form &= direct_holds;
    let (verdict_lines, statuses) = segment_two_verdict_lines(&results);
    lines.extend(verdict_lines);
    lines.extend(extra_write_verdict_lines(&results));
    lines.extend(stability_lines(&statuses));
    let mut every_assertion_holds = true;
    for (name, judged) in &tally.judged {
        let violated = tally.violations_of(name);
        every_assertion_holds &= violated == 0;
        lines.push(format!("name=assertion assertion={name} judged={judged} violated={violated}"));
    }
    let rollbacks_too_far = results.iter().filter(|result| result.rollback.is_some_and(|outcome| outcome.too_far)).count();
    let rollback_events_missing = results.iter().filter(|result| result.rollback.is_some_and(|outcome| !outcome.event_found)).count();
    lines.push(format!(
        "name=verdict question=summary v2_positive_controls_paths_agree={positive_controls_agree} positive_controls_closed_form_holds={positive_controls_closed_form} v3_last_scan_paths_agree={every_path_agrees} v4_two_runs_identical={every_run_identical} v5_anchors_hold={every_assertion_holds} v6_f10_cells_without_coverage={uncovered_f10_cells} f11_rollbacks_too_far={rollbacks_too_far} f11_events_missing={rollback_events_missing}"
    ));
    lines
}

/// 停机对拍 S4：意图那次（`crates/singlefs-core/src/transaction.rs:4065-4071`、`records.rs:119-147`）——记录 nlink = 0、size 不变、
/// 追加一条整对象墓碑 (N, b, [0, 2⁶⁴ − 1])，死亡 txg = 意图 txg（今天那条写流）。
fn stop_check_intent_matches_the_implementation() -> (bool, String) {
    let cell = WritePathCell { borrow_arm: BorrowArm::RaiseSeparator, leaf_capacity: INODE_LEAF_RECORDS, merge_threshold: 0 };
    let mut simulation = Simulation::start(cell, PRODUCT_UNLINK_BATCH_DATA_UNITS, false);
    simulation.apply(UserChange::WriteFile { object: FIRST_FILE_INODE, data_units: 3 });
    let size_before = simulation.version.record_of(FIRST_FILE_INODE).map(|record| record.size_bytes);
    simulation.publish(&Operation::UnlinkIntent { object: FIRST_FILE_INODE });
    let intent_txg = simulation.version.txg;
    let record = simulation.version.record_of(FIRST_FILE_INODE);
    let today = &simulation.version.tombstone_trees[TombstoneStream::WholeAtIntent.position()];
    let tombstones: Vec<TombstoneRecord> = today.containers.iter().flat_map(|container| container.records.iter().copied()).collect();
    let holds = record.is_some_and(|record| record.nlink == 0 && Some(record.size_bytes) == size_before)
        && tombstones.len() == 1
        && tombstones[0].object == FIRST_FILE_INODE
        && tombstones[0].range_start_bytes == WHOLE_OBJECT_RANGE_START_BYTES
        && tombstones[0].death_txg == intent_txg
        && simulation.version.deleted_queue == Some(FIRST_FILE_INODE)
        && simulation.version.tombstone_trees[TombstoneStream::WholeAtFinal.position()].record_count() == 0;
    (holds, format!("record={record:?} tombstones={tombstones:?} intent_txg={intent_txg}"))
}

/// 停机对拍 S5：墓碑树（`crates/singlefs-core/src/tombstone_tree.rs:117` 起的追加）——开放容器追加、满 815 开新容器、号 = 最大号 + 1、
/// 有新记录那次才重写。建 1 + 816 个空 inode 再逐个 unlink 816 个：第 816 次意图开第 2 个容器。
fn stop_check_tombstone_tree_matches_the_implementation() -> (bool, String) {
    let cell = WritePathCell { borrow_arm: BorrowArm::RaiseSeparator, leaf_capacity: INODE_LEAF_RECORDS, merge_threshold: 0 };
    let mut simulation = Simulation::start(cell, PRODUCT_UNLINK_BATCH_DATA_UNITS, false);
    let unlinks = TOMBSTONE_CONTAINER_RECORDS + 1;
    simulation.apply(UserChange::NewInodes { count: unlinks });
    for object in 2..=unlinks + 1 {
        simulation.apply(UserChange::Unlink { object });
    }
    let today = &simulation.version.tombstone_trees[TombstoneStream::WholeAtIntent.position()];
    let sizes: Vec<usize> = today.containers.iter().map(|container| container.records.len()).collect();
    let numbers: Vec<u64> = today.containers.iter().map(|container| container.identity.number).collect();
    let position = TombstoneStream::WholeAtIntent.position();
    let rewrites_only_with_new_records = simulation.publish_records.iter().all(|record| {
        let wrote_a_whole_tombstone = record.written.tombstone_containers[position] > 0;
        wrote_a_whole_tombstone == (record.written.tombstone_roots[position] == 1)
    });
    let intent_publishes = simulation.publish_records.iter().filter(|record| record.written.tombstone_containers[position] == 1).count();
    let holds = sizes == vec![815, 1] && numbers == vec![1, 2] && rewrites_only_with_new_records && length_as_u64(intent_publishes) == unlinks;
    (holds, format!("container_sizes={sizes:?} container_numbers={numbers:?} intent_publishes={intent_publishes}"))
}

/// PC-B2 的直接构造（登记修订 1）：容量 16、θ = 0，建 inode 1..64（四片 L1–L4），删 34..48 之后那一版当 R_old（队列空）；
/// 缺口里删 33（L3 删空、退役记录死亡 txg 在缺口里，L4 左移重写）；回退到 R_old（L3、L4 回到缺口之前那一版）；
/// 再删 49..64（最右的 L4 删空、不左移），退役容器这一次在新父链上重写、装着缺口里那条。期末扫描：B-on 的容器 m = 0，B-off-lin 的 = 1。
fn direct_rollback_positive_control_holds() -> (bool, String) {
    let write_path = WritePathCell { borrow_arm: BorrowArm::RaiseSeparator, leaf_capacity: TEST_LEAF_CAPACITY, merge_threshold: 0 };
    let mut simulation = Simulation::start(write_path, PRODUCT_UNLINK_BATCH_DATA_UNITS, true);
    simulation.apply(UserChange::NewInodes { count: 63 });
    for object in 34..=48 {
        simulation.apply(UserChange::Unlink { object });
    }
    let old_version = simulation.version.clone();
    simulation.apply(UserChange::Unlink { object: 33 });
    let rolled_back = simulation.roll_back_to(old_version);
    for object in 49..=64 {
        simulation.apply(UserChange::Unlink { object });
    }
    let readers = every_reader();
    let container_missed = |base: BaseArm| {
        let position = readers
            .iter()
            .position(|reader| *reader == Reader { base, shrink: ShrinkArm::RecordSize, effect: UnlinkEffectArm::AtIntent, window: ReadableWindow::Forever })
            .expect("读者表里有它");
        simulation.scans.last().map_or(u64::MAX, |scan| scan.counts[position].container_missed)
    };
    let on_lineage = container_missed(BaseArm::RetirementOnLineage);
    let ignoring_lineage = container_missed(BaseArm::RetirementIgnoringLineage);
    let without_retirement = container_missed(BaseArm::WithoutRetirement);
    let paths_agree = simulation.group_index_equals_the_full_scan(simulation.publish_sequence);
    let holds = rolled_back && on_lineage == 0 && ignoring_lineage == 1 && without_retirement == 0 && paths_agree;
    (
        holds,
        format!(
            "rolled_back={rolled_back} b_on_container_m={on_lineage} b_off_lin_container_m={ignoring_lineage} b_off_ret_container_m={without_retirement} expected=0,1,0 paths_agree={paths_agree}"
        ),
    )
}

// ---------------------------------------------------------------------------------------------
// 第三段（登记 5.8）：F12 意图到最后一批之间崩溃的全部崩溃支；Q-X3-O2、Q-X3-4、Q-X3-5；PC-O、PC-M（崩溃点那一份）
// ---------------------------------------------------------------------------------------------

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
enum CrashPrefix {
    /// P-1：新池新建文件、写行、暖机，顺序写 inode 1 共 u 个单元。
    FirstFileOnly,
    /// P-m：F10 的建与写（n = 300），按升序删到第一个记录被借走或被吸收过的活对象 X 出现，把 X 顺序写成 u 个单元。
    MovedObject,
}

impl CrashPrefix {
    const fn label(self) -> &'static str {
        match self {
            CrashPrefix::FirstFileOnly => "P-1",
            CrashPrefix::MovedObject => "P-m",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct SegmentThreeItem {
    prefix: CrashPrefix,
    cell: WritePathCell,
}

/// P-1 与 θ 无关，只跑 θ = 0；P-m 要有借或吸收，只跑 θ > 0。
fn segment_three_items() -> Vec<SegmentThreeItem> {
    let mut items = Vec::new();
    for leaf_capacity in LEAF_CAPACITIES {
        for borrow_arm in BORROW_ARMS {
            items.push(SegmentThreeItem { prefix: CrashPrefix::FirstFileOnly, cell: WritePathCell { borrow_arm, leaf_capacity, merge_threshold: 0 } });
        }
        for merge_threshold in merge_thresholds_of(leaf_capacity).into_iter().skip(1) {
            for borrow_arm in BORROW_ARMS {
                items.push(SegmentThreeItem { prefix: CrashPrefix::MovedObject, cell: WritePathCell { borrow_arm, leaf_capacity, merge_threshold } });
            }
        }
    }
    items
}

/// 前缀只模拟一次（按前缀剪），交回前缀之后的模拟与要删的对象；P-m 删光也没出现 X 时交回 None。
fn build_crash_prefix(prefix: CrashPrefix, cell: WritePathCell) -> Option<(Simulation, u64)> {
    let mut simulation = Simulation::start(cell, PRODUCT_UNLINK_BATCH_DATA_UNITS, false);
    match prefix {
        CrashPrefix::FirstFileOnly => Some((simulation, FIRST_FILE_INODE)),
        CrashPrefix::MovedObject => {
            build_and_write_many_objects(&mut simulation, F12_INODE_COUNT);
            loop {
                let moved_and_live = simulation.moved_objects.iter().copied().find(|object| simulation.version.is_live(*object));
                if let Some(object) = moved_and_live {
                    return Some((simulation, object));
                }
                let next_victim = simulation.version.live_inodes().into_iter().find(|inode| *inode > FIRST_FILE_INODE)?;
                simulation.apply(UserChange::Unlink { object: next_victim });
                if simulation.has_stopped() {
                    return None;
                }
            }
        }
    }
}

/// 一个崩溃支留下的东西。
#[derive(Clone, Debug)]
struct CrashBranchOutcome {
    crash_counts: Vec<ReaderCounts>,
    crash_paths_agree: bool,
    positive_controls: Vec<PositiveControlOutcome>,
    recovery_trajectories: Vec<ReaderTrajectory>,
    recovery_paths_agree: bool,
}

fn run_one_crash_branch(before_the_crash: &Simulation, object: u64) -> Option<CrashBranchOutcome> {
    let mut branch = before_the_crash.clone();
    let record = branch.version.record_of(object)?;
    let intent_txg = branch.intent_txgs.get(&object).copied()?;
    let crash_publish = branch.crash_with_the_next_unlink_step_written()?;
    let crash_counts = branch.read_every_reader(&Overlay::default(), crash_publish, SelectionPath::GroupIndex);
    let crash_paths_agree = crash_counts == branch.read_every_reader(&Overlay::default(), crash_publish, SelectionPath::FullScan);
    let planted = InodeRecord { inode: object, object_birth: record.object_birth, nlink: 1, size_bytes: record.size_bytes, change_counter: 0 };
    let plant = Overlay::planting(vec![Simulation::planted_leaf(planted, branch.newest_write_order(), crash_publish)]);
    let plant_closed_form = |readers: &[Reader], deltas: &[[i64; COUNT_FIELDS]]| {
        readers.iter().zip(deltas).all(|(reader, delta)| match reader.effect {
            UnlinkEffectArm::AtIntent => delta[DELTA_O1] == 0,
            UnlinkEffectArm::AtLastBatchWritten | UnlinkEffectArm::AtLastBatchRead => delta[DELTA_O1] == 1,
        })
    };
    let removal = Overlay::removing_a_tombstone(TombstoneRecord {
        object,
        object_birth: record.object_birth,
        range_start_bytes: WHOLE_OBJECT_RANGE_START_BYTES,
        death_txg: intent_txg,
    });
    let removal_closed_form = |readers: &[Reader], deltas: &[[i64; COUNT_FIELDS]]| {
        readers.iter().zip(deltas).all(|(reader, delta)| reader.effect != UnlinkEffectArm::AtIntent || delta[DELTA_O2] == 1)
    };
    let mut positive_controls = vec![
        branch.positive_control("PC-O-plant", &plant, crash_publish, format!("object={object}"), &plant_closed_form),
        branch.positive_control("PC-O-remove", &removal, crash_publish, format!("object={object} intent_txg={intent_txg}"), &removal_closed_form),
    ];
    positive_controls.extend(branch.erasure_positive_controls(crash_publish));
    branch.scanning = true;
    branch.recover_after_the_crash();
    branch.walk_the_deletion_queue_empty();
    branch.publish(&Operation::NewInodes { count: F12_NEW_INODES_AFTER_RECOVERY });
    let recovery_trajectories = trajectories_of(&branch.scans, every_reader().len());
    let recovery_paths_agree = branch.group_index_equals_the_full_scan(branch.publish_sequence);
    Some(CrashBranchOutcome { crash_counts, crash_paths_agree, positive_controls, recovery_trajectories, recovery_paths_agree })
}

/// 一个前缀上的一个 (u, b)：在意图那一版与每个不是最后一批的批上各分一支。
fn run_crash_branches(prefix: &Simulation, object: u64, data_units: u64, batch: u64) -> Vec<CrashBranchOutcome> {
    let mut simulation = prefix.clone();
    simulation.batch_data_units = batch;
    simulation.apply(UserChange::WriteFile { object, data_units });
    if !simulation.publish(&Operation::UnlinkIntent { object }) {
        return Vec::new();
    }
    let mut outcomes = Vec::new();
    while simulation.version.deleted_queue == Some(object) {
        outcomes.extend(run_one_crash_branch(&simulation, object));
        let step = simulation.next_unlink_step(object);
        if !simulation.publish(&step) {
            break;
        }
    }
    outcomes
}

const F12_DATA_UNITS: [u64; 3] = [3, 145, 300];
const F12_BATCHES: [u64; 2] = [1, PRODUCT_UNLINK_BATCH_DATA_UNITS];

/// 一个 (前缀, 写路径格, u, b) 上，每个读者跨全部崩溃支的汇总。
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
struct CrashSummary {
    crash_points: u64,
    inode_missed: Extremes,
    extent_missed: Extremes,
    inode_missed_positive: u64,
    extent_missed_positive: u64,
    live_records_beyond_the_truth: Extremes,
    records_whose_intent_is_published: Extremes,
    records_whose_intent_is_published_positive: u64,
    recovery_inode_resurrected_peak: u64,
    recovery_extent_resurrected_peak: u64,
    recovery_container_resurrected_peak: u64,
}

#[derive(Clone, Debug)]
struct SegmentThreeResult {
    item: SegmentThreeItem,
    lines: Vec<String>,
    summaries: Vec<((u64, u64), Vec<CrashSummary>)>,
    every_path_agrees: bool,
    positive_controls_agree: bool,
    positive_controls_closed_form: bool,
    prefix_found: bool,
}

fn run_segment_three_item(item: SegmentThreeItem) -> SegmentThreeResult {
    let readers = every_reader();
    let label = format!("prefix={} {}", item.prefix.label(), item.cell.label());
    let Some((prefix, object)) = build_crash_prefix(item.prefix, item.cell) else {
        return SegmentThreeResult {
            item,
            lines: vec![format!("name=crash_prefix {label} found=false")],
            summaries: Vec::new(),
            every_path_agrees: true,
            positive_controls_agree: true,
            positive_controls_closed_form: true,
            prefix_found: false,
        };
    };
    let mut lines = vec![format!("name=crash_prefix {label} found=true object={object} prefix_publishes={}", prefix.publishes)];
    let mut summaries = Vec::new();
    let mut every_path_agrees = true;
    let mut positive_controls_agree = true;
    let mut positive_controls_closed_form = true;
    for data_units in F12_DATA_UNITS {
        for batch in F12_BATCHES {
            let outcomes = run_crash_branches(&prefix, object, data_units, batch);
            let mut per_reader = vec![CrashSummary::default(); readers.len()];
            for outcome in &outcomes {
                every_path_agrees &= outcome.crash_paths_agree && outcome.recovery_paths_agree;
                positive_controls_agree &= outcome.positive_controls.iter().all(|control| control.paths_agree);
                positive_controls_closed_form &= outcome.positive_controls.iter().all(|control| control.closed_form_holds);
                for (position, summary) in per_reader.iter_mut().enumerate() {
                    let counts = outcome.crash_counts[position];
                    summary.crash_points += 1;
                    summary.inode_missed.observe(counts.inode_missed);
                    summary.extent_missed.observe(counts.extent_missed);
                    summary.inode_missed_positive += u64::from(counts.inode_missed > 0);
                    summary.extent_missed_positive += u64::from(counts.extent_missed > 0);
                    summary.live_records_beyond_the_truth.observe(counts.live_records_beyond_the_truth);
                    summary.records_whose_intent_is_published.observe(counts.records_whose_intent_is_published);
                    summary.records_whose_intent_is_published_positive += u64::from(counts.records_whose_intent_is_published > 0);
                    let recovery = &outcome.recovery_trajectories[position].by_scope[SCOPE_ALL];
                    summary.recovery_inode_resurrected_peak = summary.recovery_inode_resurrected_peak.max(recovery[QUANTITY_INODE_R].peak);
                    summary.recovery_extent_resurrected_peak = summary.recovery_extent_resurrected_peak.max(recovery[QUANTITY_EXTENT_R].peak);
                    summary.recovery_container_resurrected_peak = summary.recovery_container_resurrected_peak.max(recovery[QUANTITY_CONTAINER_R].peak);
                }
            }
            for (reader, summary) in readers.iter().zip(&per_reader) {
                lines.push(format!(
                    "name=crash_summary {label} u={data_units} batch={batch} {} crash_points={} {} inode_m_positive={} {} extent_m_positive={} {} {} o2_positive={} recovery_inode_r_peak={} recovery_extent_r_peak={} recovery_container_r_peak={}",
                    reader.label(),
                    summary.crash_points,
                    summary.inode_missed.fields("inode_m"),
                    summary.inode_missed_positive,
                    summary.extent_missed.fields("extent_m"),
                    summary.extent_missed_positive,
                    summary.live_records_beyond_the_truth.fields("o1"),
                    summary.records_whose_intent_is_published.fields("o2"),
                    summary.records_whose_intent_is_published_positive,
                    summary.recovery_inode_resurrected_peak,
                    summary.recovery_extent_resurrected_peak,
                    summary.recovery_container_resurrected_peak,
                ));
            }
            let controls: Vec<&PositiveControlOutcome> = outcomes.iter().flat_map(|outcome| outcome.positive_controls.iter()).collect();
            for id in ["PC-O-plant", "PC-O-remove", "PC-M-data", "PC-M-leaf"] {
                let of_id: Vec<&&PositiveControlOutcome> = controls.iter().filter(|control| control.id == id).collect();
                lines.push(format!(
                    "name=positive_control id={id} {label} u={data_units} batch={batch} crash_points={} paths_agree={} closed_form_holds={}",
                    of_id.len(),
                    of_id.iter().filter(|control| control.paths_agree).count(),
                    of_id.iter().filter(|control| control.closed_form_holds).count(),
                ));
            }
            summaries.push(((data_units, batch), per_reader));
        }
    }
    SegmentThreeResult { item, lines, summaries, every_path_agrees, positive_controls_agree, positive_controls_closed_form, prefix_found: true }
}

fn segment_three_verdict_lines(results: &[SegmentThreeResult]) -> Vec<String> {
    let readers = every_reader();
    let mut lines = Vec::new();
    for result in results.iter().filter(|result| result.prefix_found) {
        for ((data_units, batch), per_reader) in &result.summaries {
            for (reader, summary) in readers.iter().zip(per_reader).filter(|(reader, _)| reader.is_a_main_cell()) {
                let key = format!("prefix={} {} u={data_units} batch={batch} {}", result.item.prefix.label(), result.item.cell.label(), reader.label());
                let extra_objects = summary.records_whose_intent_is_published.largest.unwrap_or(0);
                let extra_status = match (reader.effect, extra_objects > 0) {
                    (UnlinkEffectArm::AtIntent, true) => "fl7_intent_effect_shows_extra_objects",
                    (UnlinkEffectArm::AtLastBatchWritten | UnlinkEffectArm::AtLastBatchRead, true) => "fl6_last_batch_effect_shows_extra_objects",
                    (UnlinkEffectArm::AtIntent | UnlinkEffectArm::AtLastBatchWritten | UnlinkEffectArm::AtLastBatchRead, false) => "zero",
                };
                lines.push(format!(
                    "name=verdict question=Q-X3-O2 {key} {} positive_crash_points={} status={extra_status}",
                    summary.records_whose_intent_is_published.fields("o2"),
                    summary.records_whose_intent_is_published_positive,
                ));
                let missed = summary.inode_missed.largest.unwrap_or(0).max(summary.extent_missed.largest.unwrap_or(0));
                let missed_status = if reader.effect == UnlinkEffectArm::AtIntent { "reported" } else { status_by_threshold(missed, "fl5_leaks_after_the_crash") };
                lines.push(format!(
                    "name=verdict question=Q-X3-4 {key} {} {} status={missed_status}",
                    summary.inode_missed.fields("inode_m"),
                    summary.extent_missed.fields("extent_m"),
                ));
                lines.push(format!(
                    "name=verdict question=Q-X3-5 {key} recovery_inode_r_peak={} recovery_extent_r_peak={} recovery_container_r_peak={} status=incidental",
                    summary.recovery_inode_resurrected_peak,
                    summary.recovery_extent_resurrected_peak,
                    summary.recovery_container_resurrected_peak,
                ));
            }
        }
    }
    lines
}

fn run_segment_three(threads: usize) -> Vec<String> {
    let items = segment_three_items();
    let mut lines = vec![format!(
        "name=config experiment=E178 segment=three work_items={} data_units=3,145,300 batches=1,144 readers={} prefix_inode_count={F12_INODE_COUNT} new_inodes_after_recovery={F12_NEW_INODES_AFTER_RECOVERY}",
        items.len(),
        every_reader().len(),
    )];
    let results = run_in_parallel(&items, threads, run_segment_three_item);
    for result in &results {
        lines.extend(result.lines.iter().cloned());
    }
    lines.extend(segment_three_verdict_lines(&results));
    let every_path_agrees = results.iter().all(|result| result.every_path_agrees);
    let controls_agree = results.iter().all(|result| result.positive_controls_agree);
    let controls_closed_form = results.iter().all(|result| result.positive_controls_closed_form);
    let prefixes_missing = results.iter().filter(|result| !result.prefix_found).count();
    lines.push(format!(
        "name=verdict question=summary v2_positive_controls_paths_agree={controls_agree} positive_controls_closed_form_holds={controls_closed_form} v3_paths_agree={every_path_agrees} prefixes_missing={prefixes_missing}"
    ));
    lines
}

// ---------------------------------------------------------------------------------------------
// 第四段（登记修订 2）：B-stub、B-tsz 两条底座臂；Q-Bs-1 至 Q-Bs-6、Q-Bt-1 至 Q-Bt-4；分片、续跑与多机
// ---------------------------------------------------------------------------------------------

/// PC-Bt1 的取样：一个身份在对象 N 的最后一批那次消失，它最后那一版装着 N（nlink = 0）的记录。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct DeadVersionHoldingAFinishedRecord {
    object: u64,
    object_birth: u64,
    intent_txg: u64,
    last_version: UnitIndex,
}

/// 第四段在第二遍上做对照的扫描（第一遍的探针记下每种情形最后出现的那一次）；下标是 W 那一档（0 是 ∞，1 是 23）。
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
struct SegmentFourPlan {
    /// PC-Bt1：有一个已删对象的记录还在一个读得出的死身份最后那一版里。
    dead_record_scan: [Option<u64>; 2],
    /// PC-Bs1：现行那一版里有桩，且那个身份最后一个非空版本是候选。
    stub_scan: [Option<u64>; 2],
    /// PC-Bt2-mid（登记修订 3）：有一个被挪过、之后变短的活对象（W = ∞）。
    moved_then_shortened_scan: Option<u64>,
}

#[derive(Clone, Debug, Default)]
struct SegmentFourProbes {
    plan: SegmentFourPlan,
    observed: SegmentFourPlan,
    outcomes: Vec<PositiveControlOutcome>,
    /// 这一段做不做 PC-Bt2 与 PC-Bt2-mid（原写路径上的 F10、θ > 0）。
    reversed_provenance_applies: bool,
}

const fn window_slot(window: ReadableWindow) -> usize {
    match window {
        ReadableWindow::Forever => 0,
        ReadableWindow::RingWindow => 1,
    }
}

/// 一个组里写序最大的候选版本（主路径择新的同一条规则，给第四段的取样与手算用）。
fn newest_candidate_of_the_group(disk: &Disk, key: GroupKey, inputs: &ScanInputs<'_>, window: ReadableWindow) -> Option<UnitIndex> {
    let group = disk.group_of_key.get(&key)?;
    disk.versions_of_group[*group].iter().rev().copied().find(|unit_index| is_candidate(&disk.units[*unit_index], inputs, window))
}

impl Simulation {
    /// 最后一批那次消失的身份里，最后那一版装着这个对象记录的，记下来（PC-Bt1 用）。在换版之前调，`self.version` 还是上一版。
    fn note_dead_versions_holding_the_finished_record(&mut self, object: u64, retired: &[ContainerIdentity], stubbed: &[ContainerIdentity]) {
        let Some(intent_txg) = self.intent_txgs.get(&object).copied() else {
            return;
        };
        for identity in retired.iter().chain(stubbed) {
            let Some(leaf) = self.version.leaves.iter().find(|leaf| leaf.identity == *identity) else {
                continue;
            };
            if let Ok(position) = leaf.records.binary_search_by_key(&object, |record| record.inode) {
                let record = leaf.records[position];
                self.dead_versions_holding_a_finished_record.push(DeadVersionHoldingAFinishedRecord {
                    object,
                    object_birth: record.object_birth,
                    intent_txg,
                    last_version: leaf.unit,
                });
            }
        }
    }

    /// PC-Bt1 的取样：对象已不在树里、身份不在现行叶序里、那一版是它那一组写序最大的候选；有几个取意图 txg 最大的。
    fn finished_record_in_a_readable_dead_version(&self, inputs: &ScanInputs<'_>, window: ReadableWindow) -> Option<DeadVersionHoldingAFinishedRecord> {
        let live_identities: HashSet<ContainerIdentity> = self.version.leaves.iter().map(|leaf| leaf.identity).collect();
        self.dead_versions_holding_a_finished_record
            .iter()
            .copied()
            .filter(|dead| self.version.record_of(dead.object).is_none())
            .filter(|dead| {
                let UnitBody::InodeLeaf { identity, .. } = &self.disk.units[dead.last_version].body else {
                    return false;
                };
                !live_identities.contains(identity)
                    && newest_candidate_of_the_group(&self.disk, GroupKey::InodeLeaf(*identity), inputs, window) == Some(dead.last_version)
            })
            .max_by_key(|dead| (dead.intent_txg, dead.object))
    }

    /// PC-Bs1 的取样：现行叶序里的桩，它那一组里桩之前最后一个非空版本是候选；有几个取容器号最小的。交回 (桩的单元, 那个非空版本)。
    fn stub_with_a_candidate_last_record_version(&self, inputs: &ScanInputs<'_>, window: ReadableWindow) -> Option<(UnitIndex, UnitIndex)> {
        let mut stubs: Vec<&Leaf> = self.version.leaves.iter().filter(|leaf| leaf.records.is_empty()).collect();
        stubs.sort_by_key(|leaf| (leaf.identity.number, leaf.identity.birth));
        stubs.into_iter().find_map(|stub| {
            let group = self.disk.group_of_key.get(&GroupKey::InodeLeaf(stub.identity))?;
            let last_record_version = self.disk.versions_of_group[*group].iter().rev().copied().find(|unit_index| {
                matches!(&self.disk.units[*unit_index].body, UnitBody::InodeLeaf { records, .. } if !records.is_empty())
            })?;
            is_candidate(&self.disk.units[last_record_version], inputs, window).then_some((stub.unit, last_record_version))
        })
    }

    /// 第一遍记下取样的扫描，第二遍在计划的那几次扫描上做 PC-Bt1（原写路径）与 PC-Bs1（B-stub 写路径）。
    fn run_the_segment_four_probes(&mut self, publish_sequence: u64) {
        let overlay = Overlay::default();
        let mut dead_record_seen = [false; 2];
        let mut stub_seen = [false; 2];
        {
            let inputs = self.scan_inputs(&overlay, publish_sequence);
            for window in READABLE_WINDOWS {
                let slot = window_slot(window);
                dead_record_seen[slot] = self.finished_record_in_a_readable_dead_version(&inputs, window).is_some();
                stub_seen[slot] = self.stub_with_a_candidate_last_record_version(&inputs, window).is_some();
            }
        }
        let moved_then_shortened_object_is_live = self.moved_then_shortened_objects.iter().any(|object| self.version.is_live(*object));
        let (plan, reversed_provenance_applies) = {
            let probes = self.segment_four.as_mut().expect("第四段的模拟带着探针");
            for slot in 0..2 {
                if dead_record_seen[slot] {
                    probes.observed.dead_record_scan[slot] = Some(publish_sequence);
                }
                if stub_seen[slot] {
                    probes.observed.stub_scan[slot] = Some(publish_sequence);
                }
            }
            if moved_then_shortened_object_is_live {
                probes.observed.moved_then_shortened_scan = Some(publish_sequence);
            }
            (probes.plan, probes.reversed_provenance_applies)
        };
        let mut outcomes = Vec::new();
        if reversed_provenance_applies && plan.moved_then_shortened_scan == Some(publish_sequence) {
            let mut control = self.reversed_provenance_positive_control(publish_sequence);
            control.id = "PC-Bt2-mid";
            outcomes.push(control);
        }
        for window in READABLE_WINDOWS {
            let slot = window_slot(window);
            if plan.dead_record_scan[slot] == Some(publish_sequence) && self.container_death == ContainerDeathWritePath::RetirementRecords {
                outcomes.extend(self.dead_record_positive_control(publish_sequence, window));
            }
            if plan.stub_scan[slot] == Some(publish_sequence) && self.container_death == ContainerDeathWritePath::ZeroRecordStubs {
                outcomes.extend(self.stub_erasure_positive_control(publish_sequence, window));
            }
        }
        self.segment_four.as_mut().expect("第四段的模拟带着探针").outcomes.extend(outcomes);
    }

    /// 这个读者那条墓碑写流里有没有一条在谱系上的 (o, b) 整对象墓碑；有就交回死亡 txg（手算用，与读者那一段分开写）。
    fn whole_object_death_on_the_chain(&self, inputs: &ScanInputs<'_>, stream: TombstoneStream, object: u64, object_birth: u64) -> Option<u64> {
        self.version.tombstone_trees[stream.position()]
            .containers
            .iter()
            .flat_map(|container| container.records.iter())
            .filter(|tombstone| {
                tombstone.object == object
                    && tombstone.object_birth == object_birth
                    && tombstone.range_start_bytes == WHOLE_OBJECT_RANGE_START_BYTES
                    && inputs.chain.contains(tombstone.death_txg)
            })
            .map(|tombstone| tombstone.death_txg)
            .max()
    }

    /// PC-Bs1 的手算（inode r）：抹掉桩之后读回的那个非空版本里，(o, b) 不在真值里、别处没有写序更大的副本、没被这个读者的整对象
    /// 墓碑压掉的记录条数（E-I、E-L-w 施行；E-L-r 的记录又回到树里、不施行）。
    fn hand_count_of_records_a_stub_erasure_brings_back(&self, inputs: &ScanInputs<'_>, last_record_version: UnitIndex, reader: Reader) -> u64 {
        let UnitBody::InodeLeaf { identity: fallen_back_identity, records } = &self.disk.units[last_record_version].body else {
            return 0;
        };
        let fallen_back_order = (self.disk.units[last_record_version].write_order, last_record_version);
        let truth = truth_of(&self.version);
        let selected = select_by_a_full_scan(inputs, reader.window, InodeLeafLineage::Required);
        let newer_copy_exists = |inode: u64| {
            selected.inode_leaves.iter().any(|unit_index| match &self.disk.units[*unit_index].body {
                UnitBody::InodeLeaf { identity, records: other_records } => {
                    identity != fallen_back_identity
                        && (self.disk.units[*unit_index].write_order, *unit_index) > fallen_back_order
                        && other_records.binary_search_by_key(&inode, |record| record.inode).is_ok()
                }
                UnitBody::Data { .. } | UnitBody::Tombstones { .. } | UnitBody::Retirements { .. } => false,
            })
        };
        let stream = reader.tombstone_stream();
        let suppressed = |record: &InodeRecord| match reader.effect {
            UnlinkEffectArm::AtIntent | UnlinkEffectArm::AtLastBatchWritten => {
                self.whole_object_death_on_the_chain(inputs, stream, record.inode, record.object_birth).is_some()
            }
            UnlinkEffectArm::AtLastBatchRead => false,
        };
        length_as_u64(
            records
                .iter()
                .filter(|record| !truth.inode_keys.contains(&(record.inode, record.object_birth)))
                .filter(|record| !newer_copy_exists(record.inode))
                .filter(|record| !suppressed(record))
                .count(),
        )
    }

    /// PC-Bs1：抹掉一个桩（读者回落到那个身份最后一个非空版本）。同一档 W 的 B-stub 读者：容器 r + 1，inode r = 手算值。
    fn stub_erasure_positive_control(&self, scan_publish: u64, window: ReadableWindow) -> Vec<PositiveControlOutcome> {
        let overlay = Overlay::default();
        let inputs = self.scan_inputs(&overlay, scan_publish);
        let Some((stub_unit, last_record_version)) = self.stub_with_a_candidate_last_record_version(&inputs, window) else {
            return Vec::new();
        };
        let hand_counts: Vec<u64> = self
            .readers
            .iter()
            .map(|reader| self.hand_count_of_records_a_stub_erasure_brings_back(&inputs, last_record_version, *reader))
            .collect();
        let closed_form = |readers: &[Reader], deltas: &[[i64; COUNT_FIELDS]]| {
            readers.iter().zip(deltas).zip(&hand_counts).all(|((reader, delta), hand)| {
                reader.window != window || (delta[DELTA_CONTAINER_R] == 1 && delta[DELTA_INODE_R] == signed(*hand))
            })
        };
        vec![self.positive_control(
            "PC-Bs1",
            &Overlay::erasing(stub_unit),
            scan_publish,
            format!(
                "window_taken={} erased_stub_unit={stub_unit} last_record_version={last_record_version} hand_inode_r={}",
                window.label(),
                hand_counts.iter().map(u64::to_string).collect::<Vec<String>>().join("/")
            ),
            &closed_form,
        )]
    }

    /// PC-Bt1：拿掉对象 N 的整对象墓碑；N 的 nlink = 0 记录还在一个读得出的死身份最后那一版里。同一档 W 的 B-tsz 读者 inode r + 1，
    /// 同一档 W 的 B-on × E-I 读者 inode r、extent r 都 + 0（那一版被退役记录丢掉）。
    fn dead_record_positive_control(&self, scan_publish: u64, window: ReadableWindow) -> Vec<PositiveControlOutcome> {
        let overlay = Overlay::default();
        let inputs = self.scan_inputs(&overlay, scan_publish);
        let Some(dead) = self.finished_record_in_a_readable_dead_version(&inputs, window) else {
            return Vec::new();
        };
        let removed = TombstoneRecord {
            object: dead.object,
            object_birth: dead.object_birth,
            range_start_bytes: WHOLE_OBJECT_RANGE_START_BYTES,
            death_txg: dead.intent_txg,
        };
        let closed_form = |readers: &[Reader], deltas: &[[i64; COUNT_FIELDS]]| {
            readers.iter().zip(deltas).all(|(reader, delta)| {
                if reader.window != window {
                    return true;
                }
                if reader.base == BaseArm::TombstonesOnly {
                    return delta[DELTA_INODE_R] == 1;
                }
                if reader.base == BaseArm::RetirementOnLineage && reader.effect == UnlinkEffectArm::AtIntent {
                    return delta[DELTA_INODE_R] == 0 && delta[DELTA_EXTENT_R] == 0;
                }
                true
            })
        };
        vec![self.positive_control(
            "PC-Bt1",
            &Overlay::removing_a_tombstone(removed),
            scan_publish,
            format!("window_taken={} object={} intent_txg={} dead_version={}", window.label(), dead.object, dead.intent_txg, dead.last_version),
            &closed_form,
        )]
    }

    /// PC-Bt2 的手算（B-tsz × Z × E-I × W = ∞）：供出规则由 P-C 换成「写序最小」之后 extent r 的增量——按单元逐个比两条规则选出的
    /// 记录（对象出生代与 size 折成的单元数），不在真值里、又没被整对象墓碑压掉的才算（两条规则各判一次，相减）。
    fn hand_count_of_units_the_oldest_copies_bring_back(&self, inputs: &ScanInputs<'_>) -> i64 {
        let selected = select_by_a_full_scan(inputs, ReadableWindow::Forever, InodeLeafLineage::Required);
        let mut largest_change_counter: HashMap<u64, ((u64, (WriteOrder, UnitIndex)), InodeRecord)> = HashMap::new();
        let mut oldest_write_order: HashMap<u64, ((WriteOrder, UnitIndex), InodeRecord)> = HashMap::new();
        for unit_index in &selected.inode_leaves {
            let UnitBody::InodeLeaf { records, .. } = &self.disk.units[*unit_index].body else {
                continue;
            };
            let order = (self.disk.units[*unit_index].write_order, *unit_index);
            for record in records.iter() {
                let change_key = (record.change_counter, order);
                let replaces_the_chosen = largest_change_counter.get(&record.inode).map_or(true, |(existing, _)| change_key > *existing);
                if replaces_the_chosen {
                    largest_change_counter.insert(record.inode, (change_key, *record));
                }
                let replaces_the_oldest = oldest_write_order.get(&record.inode).map_or(true, |(existing, _)| order < *existing);
                if replaces_the_oldest {
                    oldest_write_order.insert(record.inode, (order, *record));
                }
            }
        }
        let truth = truth_of(&self.version);
        let stream = tombstone_stream_of(ShrinkArm::RecordSize, UnlinkEffectArm::AtIntent);
        let mut delta = 0_i64;
        for unit_index in &selected.data {
            let data_unit = &self.disk.units[*unit_index];
            let Some((object, object_birth, unit_index_in_file)) = data_key_of(data_unit) else {
                continue;
            };
            if truth.extent_keys.contains(&(object, object_birth, unit_index_in_file)) {
                continue;
            }
            let killed = self
                .whole_object_death_on_the_chain(inputs, stream, object, object_birth)
                .is_some_and(|death_txg| data_unit.write_order.birth_txg <= death_txg);
            if killed {
                continue;
            }
            let alive_under = |record: Option<&InodeRecord>| {
                record.is_some_and(|record| record.object_birth == object_birth && record.data_units() > unit_index_in_file)
            };
            let alive_now = alive_under(largest_change_counter.get(&object).map(|(_, record)| record));
            let alive_reversed = alive_under(oldest_write_order.get(&object).map(|(_, record)| record));
            delta += i64::from(alive_reversed) - i64::from(alive_now);
        }
        delta
    }

    /// PC-Bt2：F10、θ > 0 每段最后一次扫描（W = ∞），供出规则倒过来另读一遍。B-tsz × Z 的 extent r 增量 = 手算值，B-tsz × T 的 + 0，
    /// 两者的 inode r + 0（修订 2 之六）。
    fn reversed_provenance_positive_control(&self, scan_publish: u64) -> PositiveControlOutcome {
        let overlay = Overlay::default();
        let inputs = self.scan_inputs(&overlay, scan_publish);
        let hand = self.hand_count_of_units_the_oldest_copies_bring_back(&inputs);
        let closed_form = |readers: &[Reader], deltas: &[[i64; COUNT_FIELDS]]| {
            readers.iter().zip(deltas).all(|(reader, delta)| {
                if reader.base != BaseArm::TombstonesOnly || reader.window != ReadableWindow::Forever {
                    return true;
                }
                let extent_expected = match reader.shrink {
                    ShrinkArm::RecordSize => hand,
                    ShrinkArm::RangeTombstone | ShrinkArm::ObjectGateOnly => 0,
                };
                delta[DELTA_EXTENT_R] == extent_expected && delta[DELTA_INODE_R] == 0
            })
        };
        self.positive_control("PC-Bt2", &Overlay::reversing_the_provenance(), scan_publish, format!("hand_extent_r_z={hand}"), &closed_form)
    }
}

/// 修订 1 的 PC-B2-direct 构造（容量 16、θ = 0、N1：建 1..64，删 34..48 当 R_old，缺口里删 33，回退，再删 49..64）在给定写路径上跑一遍。
fn direct_rollback_construction(container_death: ContainerDeathWritePath, readers: Vec<Reader>) -> (Simulation, bool) {
    let write_path = WritePathCell { borrow_arm: BorrowArm::RaiseSeparator, leaf_capacity: TEST_LEAF_CAPACITY, merge_threshold: 0 };
    let mut simulation = Simulation::start_with(write_path, PRODUCT_UNLINK_BATCH_DATA_UNITS, false, container_death, readers);
    simulation.apply(UserChange::NewInodes { count: 63 });
    for object in 34..=48 {
        simulation.apply(UserChange::Unlink { object });
    }
    let old_version = simulation.version.clone();
    simulation.apply(UserChange::Unlink { object: 33 });
    let rolled_back = simulation.roll_back_to(old_version);
    for object in 49..=64 {
        simulation.apply(UserChange::Unlink { object });
    }
    (simulation, rolled_back)
}

/// PC-Bs2（修订 2 之六）：构造期末，Z × E-I × W = ∞。B-stub 容器 m 0、inode m 0；B-stub-lin-off 容器 m 1、inode m 1、extent m 0。
/// 交回 (成立, 产物行的字段)；B-stub 那两个数也是 Q-Bs-2 要另报的「PC-Bs2 构造上 B-stub 的值」。
fn stub_lineage_positive_control() -> (bool, String) {
    let (simulation, rolled_back) = direct_rollback_construction(ContainerDeathWritePath::ZeroRecordStubs, segment_four_stub_path_readers());
    let readers = [
        Reader { base: BaseArm::ZeroRecordStubs, shrink: ShrinkArm::RecordSize, effect: UnlinkEffectArm::AtIntent, window: ReadableWindow::Forever },
        Reader { base: BaseArm::ZeroRecordStubsIgnoringLineage, shrink: ShrinkArm::RecordSize, effect: UnlinkEffectArm::AtIntent, window: ReadableWindow::Forever },
    ];
    let main = simulation.read_readers(&readers, &Overlay::default(), simulation.publish_sequence, SelectionPath::GroupIndex);
    let reference = simulation.read_readers(&readers, &Overlay::default(), simulation.publish_sequence, SelectionPath::FullScan);
    let paths_agree = main == reference;
    let stub = main[0];
    let lineage_off = main[1];
    let holds = rolled_back
        && paths_agree
        && (stub.container_missed, stub.inode_missed) == (0, 0)
        && (lineage_off.container_missed, lineage_off.inode_missed, lineage_off.extent_missed) == (1, 1, 0);
    (
        holds,
        format!(
            "rolled_back={rolled_back} b_stub_container_m={} b_stub_inode_m={} b_stub_extent_m={} b_stub_lin_off_container_m={} b_stub_lin_off_inode_m={} b_stub_lin_off_extent_m={} expected=0,0,1,1,0 paths_agree={paths_agree}",
            stub.container_missed, stub.inode_missed, stub.extent_missed, lineage_off.container_missed, lineage_off.inode_missed, lineage_off.extent_missed
        ),
    )
}

/// PC-Bt3（修订 2 之六）：构造期末，B-tsz × Z × E-I × W = ∞，整对象墓碑不按谱系滤：inode m + 1（缺口里 inode 33 的墓碑被施行），不改时 0。
fn tombstone_lineage_positive_control() -> (bool, String) {
    let (simulation, rolled_back) = direct_rollback_construction(ContainerDeathWritePath::RetirementRecords, segment_four_original_path_readers());
    let readers = [Reader { base: BaseArm::TombstonesOnly, shrink: ShrinkArm::RecordSize, effect: UnlinkEffectArm::AtIntent, window: ReadableWindow::Forever }];
    let overlay = Overlay::ignoring_the_lineage_of_whole_object_tombstones();
    let read = |overlay: &Overlay, path: SelectionPath| simulation.read_readers(&readers, overlay, simulation.publish_sequence, path)[0];
    let baseline = read(&Overlay::default(), SelectionPath::GroupIndex);
    let modified = read(&overlay, SelectionPath::GroupIndex);
    let paths_agree = baseline == read(&Overlay::default(), SelectionPath::FullScan) && modified == read(&overlay, SelectionPath::FullScan);
    let holds = rolled_back && paths_agree && baseline.inode_missed == 0 && modified.inode_missed == baseline.inode_missed + 1;
    (
        holds,
        format!(
            "rolled_back={rolled_back} b_tsz_inode_m={} b_tsz_container_m={} b_tsz_extent_m={} b_tsz_inode_m_ignoring_lineage={} expected=0,1 paths_agree={paths_agree}",
            baseline.inode_missed, baseline.container_missed, baseline.extent_missed, modified.inode_missed
        ),
    )
}

/// 第四段一条写路径跑两遍（第十一节 V4：逐次扫描比；第一遍定对照的扫描），留第二遍。
struct SegmentFourPathRun {
    simulation: Simulation,
    rollback: Option<RollbackOutcome>,
    two_runs_identical: bool,
    last_scan_paths_agree: bool,
    positive_controls: Vec<PositiveControlOutcome>,
}

fn run_segment_four_path(history: History, cell: WritePathCell, container_death: ContainerDeathWritePath, readers: &[Reader]) -> SegmentFourPathRun {
    let reversed_provenance_applies = container_death == ContainerDeathWritePath::RetirementRecords
        && history.family() == "F10"
        && cell.merge_threshold > 0;
    let start = || {
        let mut simulation = Simulation::start_with(cell, history.batch(), true, container_death, readers.to_vec());
        simulation.segment_four = Some(SegmentFourProbes {
            plan: SegmentFourPlan::default(),
            observed: SegmentFourPlan::default(),
            outcomes: Vec::new(),
            reversed_provenance_applies,
        });
        simulation
    };
    let mut first = start();
    run_history(history, &mut first);
    let plan = positive_control_plan_of(&first.scans);
    let four_plan = first.segment_four.as_ref().map_or_else(SegmentFourPlan::default, |probes| probes.observed);
    let first_scans = std::mem::take(&mut first.scans);
    drop(first);
    let mut second = start();
    second.positive_control_plan = plan;
    if let Some(probes) = second.segment_four.as_mut() {
        probes.plan = four_plan;
    }
    let rollback = run_history(history, &mut second);
    let two_runs_identical = second.scans == first_scans;
    let last_publish = second.publish_sequence;
    let last_scan_paths_agree = second.group_index_equals_the_full_scan(last_publish);
    let mut positive_controls = second.positive_control_outcomes.clone();
    positive_controls.extend(second.segment_four.as_ref().map(|probes| probes.outcomes.clone()).unwrap_or_default());
    positive_controls.extend(second.erasure_positive_controls(last_publish));
    if reversed_provenance_applies {
        positive_controls.push(second.reversed_provenance_positive_control(last_publish));
    }
    SegmentFourPathRun { simulation: second, rollback, two_runs_identical, last_scan_paths_agree, positive_controls }
}

/// 轨迹行，与第二段 `run_segment_two_item` 写的同一个形态（第四段 V-Bt2 按行比 B-on、B-off-ret 那几行）。
fn trajectory_lines(label: &str, readers: &[Reader], trajectories: &[ReaderTrajectory], scopes_reported: usize) -> Vec<String> {
    let mut lines = Vec::new();
    for (reader, trajectory) in readers.iter().zip(trajectories) {
        for (scope_position, scope) in SCAN_SCOPES.iter().enumerate().take(scopes_reported) {
            let fields: Vec<String> = COUNTED_QUANTITIES
                .iter()
                .zip(&trajectory.by_scope[scope_position])
                .map(|(quantity, quantity_trajectory)| quantity_trajectory.fields(quantity))
                .collect();
            lines.push(format!(
                "name=trajectory {label} {} scope={scope} scans={} {} read_out_empty_scans={}",
                reader.label(),
                trajectory.by_scope[scope_position][0].scans,
                fields.join(" "),
                trajectory.read_out_empty_scans
            ));
        }
    }
    lines
}

/// 「差」的四条轨迹：同一次扫描、同一 (X2, X3, W) 下新臂减 B-on，两个方向各一条（修订 2 之七）。
const DIFFERENCE_QUANTITIES: [&str; 4] = ["extent_r_more", "extent_r_less", "extent_m_more", "extent_m_less"];
const DIFFERENCE_EXTENT_R_MORE: usize = 0;
const DIFFERENCE_EXTENT_M_MORE: usize = 2;

fn difference_trajectories(new_scans: &[ScanRecord], new_position: usize, base_scans: &[ScanRecord], base_position: usize) -> [[QuantityTrajectory; 4]; 5] {
    let mut trajectories = [[QuantityTrajectory::default(); 4]; 5];
    for (new_scan, base_scan) in new_scans.iter().zip(base_scans) {
        let new_counts = new_scan.counts[new_position];
        let base_counts = base_scan.counts[base_position];
        let values = [
            new_counts.extent_resurrected.saturating_sub(base_counts.extent_resurrected),
            base_counts.extent_resurrected.saturating_sub(new_counts.extent_resurrected),
            new_counts.extent_missed.saturating_sub(base_counts.extent_missed),
            base_counts.extent_missed.saturating_sub(new_counts.extent_missed),
        ];
        for (scope_position, in_scope) in scan_scopes_of(base_scan).iter().enumerate() {
            if *in_scope {
                for (quantity_position, value) in values.iter().enumerate() {
                    trajectories[scope_position][quantity_position].observe(*value, base_scan.publish_sequence);
                }
            }
        }
    }
    trajectories
}

fn difference_lines(label: &str, reader: Reader, trajectories: &[[QuantityTrajectory; 4]; 5], scopes_reported: usize) -> Vec<String> {
    SCAN_SCOPES
        .iter()
        .enumerate()
        .take(scopes_reported)
        .map(|(scope_position, scope)| {
            let fields: Vec<String> =
                DIFFERENCE_QUANTITIES.iter().zip(&trajectories[scope_position]).map(|(quantity, trajectory)| trajectory.fields(quantity)).collect();
            format!(
                "name=difference {label} {} paired_base=B-on scope={scope} scans={} {}",
                reader.label(),
                trajectories[scope_position][0].scans,
                fields.join(" ")
            )
        })
        .collect()
}

/// 第四段的阳性对照行：同第二段的形态，PC-M 另报登记字面（每个读者 extent m + 1；抹容器那一次 inode m + 记录数、容器 m + 1）成不成立。
fn segment_four_positive_control_line(outcome: &PositiveControlOutcome, context: &str) -> String {
    let literal = match outcome.id {
        "PC-M-data" => Some(outcome.main_deltas.iter().all(|delta| delta[DELTA_EXTENT_M] == 1)),
        "PC-M-leaf" => {
            let records: Option<i64> = outcome.detail.split(' ').find_map(|field| field.strip_prefix("records=")).and_then(|value| value.parse().ok());
            Some(records.is_some_and(|records| outcome.main_deltas.iter().all(|delta| delta[DELTA_INODE_M] == records && delta[DELTA_CONTAINER_M] == 1)))
        }
        _ => None,
    };
    match literal {
        Some(holds) => format!("{} registered_literal_holds={holds}", outcome.line(context)),
        None => outcome.line(context),
    }
}

/// A11 的闭式（修订 2 之八、`anchors3.py`）：H51 F1 升序、θ = 0，第一个桩出现在第几次发布（新池新建文件是第 1 次）与有桩的版本数。
/// 第一个删空的是第二片（号 容量 + 1 .. 2 × 容量），删到它最大那个号的最后一批那次。
fn first_stub_publish_and_versions_with_a_stub(inode_count: u64, leaf_capacity: u64) -> (u64, u64) {
    let creation_publishes = (inode_count - 1).div_ceil(NEW_INODES_PER_STEP);
    let total_publishes = 3 + creation_publishes + 2 * (inode_count - 1);
    let first_emptied_inode = inode_count.min(2 * leaf_capacity);
    let first_stub_publish = 3 + creation_publishes + 2 * (first_emptied_inode - 1);
    (first_stub_publish, total_publishes - first_stub_publish + 1)
}

/// 两条写路径的配对前提（修订 2 之五、V-Bs1）：用户改动序列、发布数、每一版的 txg、活叶（身份、分隔 key、记录）、数据单元、
/// 每次扫描的发布序号、txg 与真值的记录数都逐个相同。
fn pairing_holds(original: &Simulation, stub: &Simulation) -> bool {
    original.user_changes == stub.user_changes
        && original.publishes == stub.publishes
        && original.publish_records.len() == stub.publish_records.len()
        && original.publish_records.iter().zip(&stub.publish_records).all(|(original_record, stub_record)| {
            original_record.txg == stub_record.txg
                && original_record.publish_sequence == stub_record.publish_sequence
                && original_record.leaf_digest == stub_record.leaf_digest
                && original_record.extent_digest == stub_record.extent_digest
                && std::mem::discriminant(&original_record.merge) == std::mem::discriminant(&stub_record.merge)
        })
        && original.scans.len() == stub.scans.len()
        && original.scans.iter().zip(&stub.scans).all(|(original_scan, stub_scan)| {
            (original_scan.publish_sequence, original_scan.txg, original_scan.truth_inode_keys, original_scan.inside_an_unlink)
                == (stub_scan.publish_sequence, stub_scan.txg, stub_scan.truth_inode_keys, stub_scan.inside_an_unlink)
        })
}

/// Q-Bs-5、Q-Bs-6：B-stub 写路径每一版的叶片总数、桩数、桩占比（百万分之一）、做完的 unlink 数；撞 I-9.13 的版本数（修订 2 之七）。
fn stub_leaves_line(label: &str, stub: &Simulation, original: &Simulation) -> String {
    let mut total_leaves = QuantityTrajectory::default();
    let mut stubs = QuantityTrajectory::default();
    let mut stub_share = QuantityTrajectory::default();
    let mut live_leaves = QuantityTrajectory::default();
    let mut versions_over_the_root = 0_u64;
    let mut first_over_the_root: Option<u64> = None;
    let mut first_stub_publish: Option<u64> = None;
    for (position, record) in stub.publish_records.iter().enumerate() {
        let publish_number = length_as_u64(position) + 1;
        total_leaves.observe(record.leaves_in_the_leaf_order, publish_number);
        stubs.observe(record.stubs_in_the_leaf_order, publish_number);
        let share = if record.leaves_in_the_leaf_order == 0 { 0 } else { record.stubs_in_the_leaf_order * 1_000_000 / record.leaves_in_the_leaf_order };
        stub_share.observe(share, publish_number);
        // 活叶数 = 叶片总数 − 桩数；配对成立时它逐版等于同格原写路径的叶容器数（V-Bs1）。
        live_leaves.observe(difference_or_marker(record.leaves_in_the_leaf_order, record.stubs_in_the_leaf_order), publish_number);
        if record.leaves_in_the_leaf_order > ROOT_NODE_LEAF_CONTAINERS {
            versions_over_the_root += 1;
            first_over_the_root.get_or_insert(publish_number);
        }
        if record.stubs_in_the_leaf_order > 0 {
            first_stub_publish.get_or_insert(publish_number);
        }
    }
    let shown = |value: Option<u64>| value.map_or_else(|| "none".to_string(), |known| known.to_string());
    let versions_with_an_empty_leaf = stub.publish_records.iter().filter(|record| record.holds_an_empty_inode_leaf).count();
    format!(
        "name=stub_leaves {label} versions={} versions_with_an_empty_inode_leaf={versions_with_an_empty_leaf} first_stub_publish={} stubs_written={} {} {} {} {} versions_over_the_root={versions_over_the_root} first_over_the_root={} unlinks_completed={} original_live_leaves_last={}",
        stub.publish_records.len(),
        shown(first_stub_publish),
        stub.publish_records.iter().map(|record| record.stubs_written).sum::<u64>(),
        total_leaves.fields("leaves"),
        stubs.fields("stubs"),
        stub_share.fields("stub_share_ppm"),
        live_leaves.fields("live_leaves"),
        shown(first_over_the_root),
        stub.unlinks_completed,
        original.version.leaves.len(),
    )
}

/// 第四段一个 (历史段, 写路径格) 工作项的全部产物行：原写路径 18 个读者，N1 的格另跑 B-stub 写路径 12 个读者（问法一 N2-none）。
fn segment_four_history_item_lines(history: History, cell: WritePathCell) -> Vec<String> {
    let label = format!("{} {}", history.label(), cell.label());
    let original_readers = segment_four_original_path_readers();
    let original = run_segment_four_path(history, cell, ContainerDeathWritePath::RetirementRecords, &original_readers);
    let mut checks = AssertionTally::default();
    let mut lines = Vec::new();
    let scopes_reported = if original.rollback.is_some() { SCAN_SCOPES.len() } else { 3 };
    let simulation = &original.simulation;
    let (borrows, absorbs, drops) = simulation.count_merges();
    lines.push(format!(
        "name=simulation segment=four {label} path=original publishes={} scans={} refused_steps={} borrows={borrows} absorbs={absorbs} drops={drops} moved_then_shortened={} moved_then_unlinked={} retirement_records={} does_not_fit_at={} rollback_event_found={} rollback_too_far={} two_runs_identical={} last_scan_paths_agree={}",
        simulation.publishes,
        simulation.scans.len(),
        simulation.refused_steps,
        simulation.moved_then_shortened,
        simulation.moved_then_unlinked,
        simulation.total_written().retirement_records,
        simulation.does_not_fit_at.map_or_else(|| "none".to_string(), |txg| txg.to_string()),
        original.rollback.map_or_else(|| "none".to_string(), |outcome| outcome.event_found.to_string()),
        original.rollback.map_or_else(|| "none".to_string(), |outcome| outcome.too_far.to_string()),
        original.two_runs_identical,
        original.last_scan_paths_agree,
    ));
    let original_trajectories = trajectories_of(&simulation.scans, original_readers.len());
    lines.extend(trajectory_lines(&label, &original_readers, &original_trajectories, scopes_reported));
    let without_retirement_position = |window: ReadableWindow| {
        original_readers
            .iter()
            .position(|reader| *reader == Reader { base: BaseArm::WithoutRetirement, shrink: ShrinkArm::RecordSize, effect: UnlinkEffectArm::AtIntent, window })
            .expect("原写路径读者表里有 B-off-ret")
    };
    for (position, reader) in original_readers.iter().enumerate().filter(|(_, reader)| reader.base == BaseArm::TombstonesOnly) {
        let paired = Reader { base: BaseArm::RetirementOnLineage, shrink: reader.shrink, effect: reader.effect, window: reader.window };
        let paired_position = original_readers.iter().position(|candidate| *candidate == paired).expect("B-on 同一 (X2, X3, W) 在表里");
        let differences = difference_trajectories(&simulation.scans, position, &simulation.scans, paired_position);
        lines.extend(difference_lines(&label, *reader, &differences, scopes_reported));
        let provenance_difference_scans = simulation.scans.iter().filter(|scan| scan.counts[position].provenance_differences > 0).count();
        let provenance_differences_total: u64 = simulation.scans.iter().map(|scan| scan.counts[position].provenance_differences).sum();
        lines.push(format!(
            "name=reader_extras {label} {} provenance_difference_scans={provenance_difference_scans} provenance_differences_total={provenance_differences_total}",
            reader.label()
        ));
        // A12：B-tsz 的容器 r 与同 W 的 B-off-ret × Z × E-I 逐次扫描相等。
        let comparison = without_retirement_position(reader.window);
        for scan in &simulation.scans {
            checks.judge("A12_tombstones_only_container_r_equals_without_retirement", scan.counts[position].container_resurrected == scan.counts[comparison].container_resurrected);
        }
        if let (History::BuildAndDeleteEmptyInodes { inode_count, .. }, 0, ReadableWindow::Forever) = (history, cell.merge_threshold, reader.window) {
            let expected = inode_count.div_ceil(cell.leaf_capacity) - 1;
            let last = simulation.scans.last().map_or(u64::MAX, |scan| scan.counts[position].container_resurrected);
            checks.judge("A12_tombstones_only_final_container_r_equals_a8", last == expected);
        }
    }
    for outcome in &original.positive_controls {
        lines.push(segment_four_positive_control_line(outcome, &format!("{label} path=original")));
        checks.judge("V2_positive_control_paths_agree", outcome.paths_agree);
        checks.judge("V2_positive_control_closed_form_holds", outcome.closed_form_holds);
    }
    for (name, judged) in &simulation.assertions.judged {
        lines.push(format!("name=item_assertion {label} path=original assertion={name} judged={judged} violated={}", simulation.assertions.violations_of(name)));
    }
    if cell.borrow_arm == BorrowArm::RaiseSeparator {
        lines.extend(stub_path_lines(history, cell, &label, &original, &mut checks));
    }
    for (name, judged) in &checks.judged {
        lines.push(format!("name=item_assertion {label} path=segment_four assertion={name} judged={judged} violated={}", checks.violations_of(name)));
    }
    lines
}

/// B-stub 写路径那一半（N1 的格）：12 个读者的轨迹、与原写路径同 (X2, X3, W) 的 B-on 配对算差、V-Bs1、V-Bs3、A10、A11、Q-Bs-5、Q-Bs-6。
fn stub_path_lines(history: History, cell: WritePathCell, label: &str, original: &SegmentFourPathRun, checks: &mut AssertionTally) -> Vec<String> {
    let stub_readers = segment_four_stub_path_readers();
    let original_readers = segment_four_original_path_readers();
    let stub = run_segment_four_path(history, cell, ContainerDeathWritePath::ZeroRecordStubs, &stub_readers);
    let simulation = &stub.simulation;
    let scopes_reported = if stub.rollback.is_some() { SCAN_SCOPES.len() } else { 3 };
    let paired = pairing_holds(&original.simulation, simulation);
    checks.judge("V-Bs1_pairing_holds", paired);
    let mut stub_read_mismatches = 0_u64;
    let mut lines = Vec::new();
    for (position, reader) in stub_readers.iter().enumerate() {
        let mismatched_scans = simulation
            .scans
            .iter()
            .zip(&simulation.publish_records)
            .filter(|(scan, record)| scan.counts[position].stubs_read != record.stubs_in_the_leaf_order)
            .count();
        stub_read_mismatches += length_as_u64(mismatched_scans);
        lines.push(format!(
            "name=reader_extras {label} {} stub_read_mismatch_scans={mismatched_scans} stubs_read_last={}",
            reader.label(),
            simulation.scans.last().map_or(0, |scan| scan.counts[position].stubs_read)
        ));
    }
    checks.judge("V-Bs3_stubs_read_equal_the_current_stubs", stub_read_mismatches == 0 && simulation.scans.len() == simulation.publish_records.len());
    lines.insert(
        0,
        format!(
            "name=simulation segment=four {label} path=stub publishes={} scans={} refused_steps={} stubs_written={} retirement_records={} does_not_fit_at={} pairing_holds={paired} stub_read_mismatch_scans={stub_read_mismatches} two_runs_identical={} last_scan_paths_agree={}",
            simulation.publishes,
            simulation.scans.len(),
            simulation.refused_steps,
            simulation.publish_records.iter().map(|record| record.stubs_written).sum::<u64>(),
            simulation.total_written().retirement_records,
            simulation.does_not_fit_at.map_or_else(|| "none".to_string(), |txg| txg.to_string()),
            stub.two_runs_identical,
            stub.last_scan_paths_agree,
        ),
    );
    let stub_trajectories = trajectories_of(&simulation.scans, stub_readers.len());
    lines.extend(trajectory_lines(label, &stub_readers, &stub_trajectories, scopes_reported));
    if paired {
        for (position, reader) in stub_readers.iter().enumerate() {
            let base = Reader { base: BaseArm::RetirementOnLineage, shrink: reader.shrink, effect: reader.effect, window: reader.window };
            let base_position = original_readers.iter().position(|candidate| *candidate == base).expect("B-on 同一 (X2, X3, W) 在表里");
            let differences = difference_trajectories(&simulation.scans, position, &original.simulation.scans, base_position);
            lines.extend(difference_lines(label, *reader, &differences, scopes_reported));
        }
    }
    lines.push(stub_leaves_line(label, simulation, &original.simulation));
    if let (History::BuildAndDeleteEmptyInodes { inode_count, order }, 0) = (history, cell.merge_threshold) {
        let disappearances = inode_count.div_ceil(cell.leaf_capacity) - 1;
        let stubs_written: u64 = simulation.publish_records.iter().map(|record| record.stubs_written).sum();
        let final_leaves = length_as_u64(simulation.version.leaves.len());
        checks.judge(
            "A10_stubs_written_equal_final_stubs_equal_a8",
            stubs_written == disappearances && simulation.version.stub_count() == disappearances && final_leaves == inode_count.div_ceil(cell.leaf_capacity),
        );
        if order == DeletionOrder::Ascending {
            let (expected_first, expected_versions) = first_stub_publish_and_versions_with_a_stub(inode_count, cell.leaf_capacity);
            let first = simulation.publish_records.iter().position(|record| record.stubs_in_the_leaf_order > 0).map(|position| length_as_u64(position) + 1);
            let versions = length_as_u64(simulation.publish_records.iter().filter(|record| record.holds_an_empty_inode_leaf).count());
            checks.judge("A11_first_stub_publish_and_versions_with_a_stub", first == Some(expected_first) && versions == expected_versions);
        }
    }
    for outcome in &stub.positive_controls {
        lines.push(segment_four_positive_control_line(outcome, &format!("{label} path=stub")));
        checks.judge("V2_positive_control_paths_agree", outcome.paths_agree);
        checks.judge("V2_positive_control_closed_form_holds", outcome.closed_form_holds);
    }
    for (name, judged) in &simulation.assertions.judged {
        lines.push(format!("name=item_assertion {label} path=stub assertion={name} judged={judged} violated={}", simulation.assertions.violations_of(name)));
    }
    lines
}

/// 第四段的工作项（修订 2 之十三：H51 51 段 + F10 28 段 + F11 4 段，各 × 12 格，加 PC-B2-direct 构造 1 项，共 997 项）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum SegmentFourItem {
    HistoryCell { history: History, cell: WritePathCell },
    DirectRollbackConstruction,
}

impl SegmentFourItem {
    fn label(self) -> String {
        match self {
            SegmentFourItem::HistoryCell { history, cell } => format!("{} {}", history.label(), cell.label()),
            SegmentFourItem::DirectRollbackConstruction => "construction=PC-B2-direct".to_string(),
        }
    }
}

fn segment_four_items() -> Vec<SegmentFourItem> {
    let mut items: Vec<SegmentFourItem> =
        segment_two_items().into_iter().map(|item| SegmentFourItem::HistoryCell { history: item.history, cell: item.cell }).collect();
    items.push(SegmentFourItem::DirectRollbackConstruction);
    items
}

fn segment_four_item_lines(item: SegmentFourItem) -> Vec<String> {
    match item {
        SegmentFourItem::HistoryCell { history, cell } => segment_four_history_item_lines(history, cell),
        SegmentFourItem::DirectRollbackConstruction => {
            let (stub_holds, stub_detail) = stub_lineage_positive_control();
            let (tombstone_holds, tombstone_detail) = tombstone_lineage_positive_control();
            vec![
                format!("name=positive_control id=PC-Bs2 {stub_detail} holds={stub_holds}"),
                format!("name=positive_control id=PC-Bt3 {tombstone_detail} holds={tombstone_holds}"),
            ]
        }
    }
}

/// SHA-256（FIPS 180-4）：续跑的键里装置源文件的指纹（修订 2 之十三）。单测钉空串与 "abc" 的标准值。
fn sha256_hex(message: &[u8]) -> String {
    const ROUND_CONSTANTS: [u32; 64] = [
        0x428a_2f98, 0x7137_4491, 0xb5c0_fbcf, 0xe9b5_dba5, 0x3956_c25b, 0x59f1_11f1, 0x923f_82a4, 0xab1c_5ed5, 0xd807_aa98, 0x1283_5b01,
        0x2431_85be, 0x550c_7dc3, 0x72be_5d74, 0x80de_b1fe, 0x9bdc_06a7, 0xc19b_f174, 0xe49b_69c1, 0xefbe_4786, 0x0fc1_9dc6, 0x240c_a1cc,
        0x2de9_2c6f, 0x4a74_84aa, 0x5cb0_a9dc, 0x76f9_88da, 0x983e_5152, 0xa831_c66d, 0xb003_27c8, 0xbf59_7fc7, 0xc6e0_0bf3, 0xd5a7_9147,
        0x06ca_6351, 0x1429_2967, 0x27b7_0a85, 0x2e1b_2138, 0x4d2c_6dfc, 0x5338_0d13, 0x650a_7354, 0x766a_0abb, 0x81c2_c92e, 0x9272_2c85,
        0xa2bf_e8a1, 0xa81a_664b, 0xc24b_8b70, 0xc76c_51a3, 0xd192_e819, 0xd699_0624, 0xf40e_3585, 0x106a_a070, 0x19a4_c116, 0x1e37_6c08,
        0x2748_774c, 0x34b0_bcb5, 0x391c_0cb3, 0x4ed8_aa4a, 0x5b9c_ca4f, 0x682e_6ff3, 0x748f_82ee, 0x78a5_636f, 0x84c8_7814, 0x8cc7_0208,
        0x90be_fffa, 0xa450_6ceb, 0xbef9_a3f7, 0xc671_78f2,
    ];
    let mut state: [u32; 8] = [0x6a09_e667, 0xbb67_ae85, 0x3c6e_f372, 0xa54f_f53a, 0x510e_527f, 0x9b05_688c, 0x1f83_d9ab, 0x5be0_cd19];
    let message_bits = length_as_u64(message.len()) * 8;
    let mut padded = message.to_vec();
    padded.push(0x80);
    while padded.len() % 64 != 56 {
        padded.push(0);
    }
    padded.extend_from_slice(&message_bits.to_be_bytes());
    for block in padded.chunks_exact(64) {
        let mut schedule = [0_u32; 64];
        for (word_position, word) in block.chunks_exact(4).enumerate() {
            schedule[word_position] = u32::from_be_bytes([word[0], word[1], word[2], word[3]]);
        }
        for word_position in 16..64 {
            let earlier = schedule[word_position - 15];
            let recent = schedule[word_position - 2];
            let small_sigma_zero = earlier.rotate_right(7) ^ earlier.rotate_right(18) ^ (earlier >> 3);
            let small_sigma_one = recent.rotate_right(17) ^ recent.rotate_right(19) ^ (recent >> 10);
            schedule[word_position] = schedule[word_position - 16]
                .wrapping_add(small_sigma_zero)
                .wrapping_add(schedule[word_position - 7])
                .wrapping_add(small_sigma_one);
        }
        let mut working = state;
        for (round_constant, scheduled_word) in ROUND_CONSTANTS.iter().zip(schedule) {
            let big_sigma_one = working[4].rotate_right(6) ^ working[4].rotate_right(11) ^ working[4].rotate_right(25);
            let choice = (working[4] & working[5]) ^ (!working[4] & working[6]);
            let first_sum =
                working[7].wrapping_add(big_sigma_one).wrapping_add(choice).wrapping_add(*round_constant).wrapping_add(scheduled_word);
            let big_sigma_zero = working[0].rotate_right(2) ^ working[0].rotate_right(13) ^ working[0].rotate_right(22);
            let majority = (working[0] & working[1]) ^ (working[0] & working[2]) ^ (working[1] & working[2]);
            let second_sum = big_sigma_zero.wrapping_add(majority);
            working = [
                first_sum.wrapping_add(second_sum),
                working[0],
                working[1],
                working[2],
                working[3].wrapping_add(first_sum),
                working[4],
                working[5],
                working[6],
            ];
        }
        for (word, added) in state.iter_mut().zip(working) {
            *word = word.wrapping_add(added);
        }
    }
    state.iter().map(|word| format!("{word:08x}")).collect()
}

/// 装置自己的源码（编进二进制），续跑的键用它的 sha256：源码一改，旧项一律重算。
const DEVICE_SOURCE: &str = include_str!("e178_leaf_and_record_death_counts.rs");

/// 第四段跑法的选项：分片 (i, k)、读不读项目录、项目录在哪。
#[derive(Clone, Debug, PartialEq, Eq)]
struct SegmentFourOptions {
    shard: Option<(usize, usize)>,
    reuse: bool,
    items_directory: std::path::PathBuf,
}

/// 项文件名：序号加几何，只留字母、数字与连字符。
fn segment_four_item_name(position: usize, item: SegmentFourItem) -> String {
    let geometry: String = item.label().chars().map(|character| if character.is_ascii_alphanumeric() || character == '-' { character } else { '_' }).collect();
    format!("{position:04}-{geometry}")
}

/// 项文件的第一行：项名、装置源 sha256、几何参数（修订 2 之十三）。
fn segment_four_item_key(name: &str, item: SegmentFourItem, device_source_sha256: &str) -> String {
    format!("key item={name} device_source_sha256={device_source_sha256} geometry={}", item.label())
}

/// 读回一个项文件：第一行键相同、末行 `done` 报的行数对得上才算数，不然交回 None（重算）。
fn read_back_the_item(path: &std::path::Path, key: &str) -> Option<Vec<String>> {
    let contents = std::fs::read_to_string(path).ok()?;
    let mut lines: Vec<String> = contents.lines().map(str::to_string).collect();
    if lines.first().map(String::as_str) != Some(key) {
        return None;
    }
    let done = lines.pop()?;
    let item_lines: Vec<String> = lines.into_iter().skip(1).collect();
    (done == format!("done lines={}", item_lines.len())).then_some(item_lines)
}

/// 一项算完原子落盘：同目录临时文件写完、fsync、改名，再 fsync 目录。
fn write_the_item_atomically(directory: &std::path::Path, name: &str, key: &str, item_lines: &[String]) -> std::io::Result<()> {
    use std::io::Write;
    let final_path = directory.join(format!("{name}.out"));
    let temporary_path = directory.join(format!(".{name}.tmp-{}", std::process::id()));
    {
        let mut file = std::fs::File::create(&temporary_path)?;
        writeln!(file, "{key}")?;
        for line in item_lines {
            writeln!(file, "{line}")?;
        }
        writeln!(file, "done lines={}", item_lines.len())?;
        file.sync_all()?;
    }
    std::fs::rename(&temporary_path, &final_path)?;
    std::fs::File::open(directory)?.sync_all()
}

/// 算（或读回）给定的项：分片之外的项不碰，交回 (每项的行，没轮到的是 None；现判数；复用数)。
fn compute_or_reuse_the_items(items: &[SegmentFourItem], threads: usize, options: &SegmentFourOptions) -> (Vec<Option<Vec<String>>>, u64, u64) {
    let device_source_sha256 = sha256_hex(DEVICE_SOURCE.as_bytes());
    std::fs::create_dir_all(&options.items_directory).expect("项目录建得出来（E178_SEGMENT_FOUR_ITEMS）");
    let positions: Vec<usize> = (0..items.len()).collect();
    let outcomes = run_in_parallel(&positions, threads, |position| {
        if let Some((shard_index, shard_count)) = options.shard {
            if position % shard_count != shard_index {
                return (None, false);
            }
        }
        let item = items[position];
        let name = segment_four_item_name(position, item);
        let key = segment_four_item_key(&name, item, &device_source_sha256);
        let path = options.items_directory.join(format!("{name}.out"));
        if options.reuse {
            if let Some(item_lines) = read_back_the_item(&path, &key) {
                return (Some(item_lines), true);
            }
        }
        let item_lines = segment_four_item_lines(item);
        write_the_item_atomically(&options.items_directory, &name, &key, &item_lines).expect("项文件写得进项目录");
        (Some(item_lines), false)
    });
    let computed = length_as_u64(outcomes.iter().filter(|(item_lines, reused)| item_lines.is_some() && !reused).count());
    let reused = length_as_u64(outcomes.iter().filter(|(_, reused)| *reused).count());
    (outcomes.into_iter().map(|(item_lines, _)| item_lines).collect(), computed, reused)
}

/// 产物行的字段：按空格切开的 `键=值`（值里没有空格；`name=` 后面那一段也在里面）。
fn fields_of(line: &str) -> BTreeMap<&str, &str> {
    line.split(' ').filter_map(|field| field.split_once('=')).collect()
}

fn number_field(fields: &BTreeMap<&str, &str>, key: &str) -> u64 {
    fields.get(key).and_then(|value| value.parse::<u64>().ok()).unwrap_or_else(|| panic!("产物行带数字字段 {key}：{fields:?}"))
}

/// 从轨迹行（`trajectory_lines`、`difference_lines` 写的）读回一个量的轨迹。
fn quantity_trajectory_from(fields: &BTreeMap<&str, &str>, prefix: &str) -> QuantityTrajectory {
    QuantityTrajectory {
        scans: number_field(fields, "scans"),
        peak: number_field(fields, &format!("{prefix}_peak")),
        peak_at_publish: number_field(fields, &format!("{prefix}_peak_at")),
        positive_scans: number_field(fields, &format!("{prefix}_positive_scans")),
        last: number_field(fields, &format!("{prefix}_last")),
    }
}

/// 聚合的键：(族，None 是全部族；判定格)。F11 回退之后那一档取族 = F11。
type AggregateKey = (Option<&'static str>, VerdictKey);

/// 第四段拼产物时从各项的行里读回、再聚合的轨迹（判定行由它算，第八节「判定函数把门槛当参数」照用 `status_by_threshold`）。
#[derive(Debug, Default)]
struct SegmentFourAggregate {
    trajectories: BTreeMap<(AggregateKey, Reader, usize), [QuantityTrajectory; 6]>,
    differences: BTreeMap<(AggregateKey, Reader, usize), [QuantityTrajectory; 4]>,
    provenance_differences: BTreeMap<(AggregateKey, Reader), u64>,
    read_out_empty_trajectories: BTreeMap<Reader, u64>,
}

impl SegmentFourAggregate {
    fn trajectory(&self, key: AggregateKey, reader: Reader, scope: usize) -> [QuantityTrajectory; 6] {
        self.trajectories.get(&(key, reader, scope)).copied().unwrap_or_default()
    }

    fn difference(&self, key: AggregateKey, reader: Reader, scope: usize) -> [QuantityTrajectory; 4] {
        self.differences.get(&(key, reader, scope)).copied().unwrap_or_default()
    }

    fn has_data_for(&self, key: AggregateKey) -> bool {
        self.trajectories.keys().any(|(stored_key, _, _)| *stored_key == key)
    }
}

/// 第四段的读者按产物行里的 `base= shrink= effect= window=` 认回来（两张读者表的并集）。
fn segment_four_reader_of(fields: &BTreeMap<&str, &str>) -> Option<Reader> {
    segment_four_original_path_readers().into_iter().chain(segment_four_stub_path_readers()).find(|reader| {
        fields.get("base") == Some(&reader.base.label())
            && fields.get("shrink") == Some(&reader.shrink.label())
            && fields.get("effect") == Some(&reader.effect.label())
            && fields.get("window") == Some(&reader.window.label())
    })
}

fn scope_position_of(fields: &BTreeMap<&str, &str>) -> usize {
    let scope = fields.get("scope").copied().unwrap_or_default();
    SCAN_SCOPES.iter().position(|known| *known == scope).unwrap_or_else(|| panic!("轨迹行的 scope 认得出：{scope}"))
}

fn aggregate_segment_four(items: &[SegmentFourItem], item_lines: &[Vec<String>]) -> SegmentFourAggregate {
    let mut aggregate = SegmentFourAggregate::default();
    for (item, lines) in items.iter().zip(item_lines) {
        let SegmentFourItem::HistoryCell { history, cell } = item else {
            continue;
        };
        let verdict_key = VerdictKey { cell: *cell, batch: history.batch() };
        let keys = [(None, verdict_key), (Some(history.family()), verdict_key)];
        for line in lines {
            let fields = fields_of(line);
            match fields.get("name").copied() {
                Some("trajectory") => {
                    let Some(reader) = segment_four_reader_of(&fields) else {
                        continue;
                    };
                    let scope = scope_position_of(&fields);
                    let parsed: Vec<QuantityTrajectory> = COUNTED_QUANTITIES.iter().map(|quantity| quantity_trajectory_from(&fields, quantity)).collect();
                    for key in keys {
                        let entry = aggregate.trajectories.entry((key, reader, scope)).or_default();
                        for (stored, observed) in entry.iter_mut().zip(&parsed) {
                            stored.absorb(*observed);
                        }
                    }
                    if scope == SCOPE_ALL && number_field(&fields, "read_out_empty_scans") > 0 {
                        *aggregate.read_out_empty_trajectories.entry(reader).or_default() += 1;
                    }
                }
                Some("difference") => {
                    let Some(reader) = segment_four_reader_of(&fields) else {
                        continue;
                    };
                    let scope = scope_position_of(&fields);
                    let parsed: Vec<QuantityTrajectory> = DIFFERENCE_QUANTITIES.iter().map(|quantity| quantity_trajectory_from(&fields, quantity)).collect();
                    for key in keys {
                        let entry = aggregate.differences.entry((key, reader, scope)).or_default();
                        for (stored, observed) in entry.iter_mut().zip(&parsed) {
                            stored.absorb(*observed);
                        }
                    }
                }
                Some("reader_extras") => {
                    if let (Some(reader), Some(total)) = (segment_four_reader_of(&fields), fields.get("provenance_differences_total")) {
                        for key in keys {
                            *aggregate.provenance_differences.entry((key, reader)).or_default() += total.parse::<u64>().expect("条数是数字");
                        }
                    }
                }
                Some(_) | None => {}
            }
        }
    }
    aggregate
}

/// 第四段的读者：B-on、B-stub、B-tsz 在同一 (X2, X3, W) 上。
const fn reader_with(base: BaseArm, shrink: ShrinkArm, effect: UnlinkEffectArm, window: ReadableWindow) -> Reader {
    Reader { base, shrink, effect, window }
}

const fn x2_resurrection_question(shrink: ShrinkArm) -> &'static str {
    match shrink {
        ShrinkArm::RecordSize => "Q-X2-1",
        ShrinkArm::RangeTombstone | ShrinkArm::ObjectGateOnly => "Q-X2-2",
    }
}

const fn x2_misjudged_death_question(shrink: ShrinkArm) -> &'static str {
    match shrink {
        ShrinkArm::RecordSize => "Q-X2-3",
        ShrinkArm::RangeTombstone | ShrinkArm::ObjectGateOnly => "Q-X2-4",
    }
}

/// 一个读者在 Q-X2-1 至 Q-X2-5 口径下的判定（与第二段 `segment_two_verdict_lines` 同门槛同量）：(子问题, 状态)。
fn x2_statuses(trajectories_all: &[QuantityTrajectory; 6], trajectories_outside: &[QuantityTrajectory; 6], trajectories_inside: &[QuantityTrajectory; 6], reader: Reader) -> Vec<(&'static str, &'static str)> {
    let mut statuses = vec![
        (x2_resurrection_question(reader.shrink), status_by_threshold(trajectories_all[QUANTITY_EXTENT_R].peak, "resurrects")),
        (x2_misjudged_death_question(reader.shrink), status_by_threshold(trajectories_outside[QUANTITY_EXTENT_M].peak, "misjudges_death")),
    ];
    if reader.effect != UnlinkEffectArm::AtIntent {
        statuses.push(("Q-X2-5", status_by_threshold(trajectories_inside[QUANTITY_EXTENT_M].peak, "misjudges_in_batches")));
    }
    statuses
}

/// 一条新底座读者在一个判定格、一档 W 上的判定行（Q-Bs-1 至 Q-Bs-4 或 Q-Bt-1 至 Q-Bt-4；修订 2 之七）。
fn new_base_verdict_lines(
    aggregate: &SegmentFourAggregate,
    verdict_key: VerdictKey,
    reader: Reader,
    statuses: &mut Vec<JudgedStatus>,
    differing_x2_cells: &mut Vec<String>,
) -> Vec<String> {
    let (prefix, base_label) = match reader.base {
        BaseArm::TombstonesOnly => ("Q-Bt", "B-tsz"),
        BaseArm::ZeroRecordStubs
        | BaseArm::ZeroRecordStubsIgnoringLineage
        | BaseArm::RetirementOnLineage
        | BaseArm::WithoutRetirement
        | BaseArm::RetirementIgnoringLineage => ("Q-Bs", "B-stub"),
    };
    let all_key: AggregateKey = (None, verdict_key);
    let rollback_key: AggregateKey = (Some("F11"), verdict_key);
    let paired = reader_with(BaseArm::RetirementOnLineage, reader.shrink, reader.effect, reader.window);
    let key_label = format!("borrow_arm={} capacity={} theta={} batch={}", verdict_key.cell.borrow_arm.label(), verdict_key.cell.leaf_capacity, verdict_key.cell.merge_threshold, verdict_key.batch);
    let arms = format!("shrink={} effect={}", reader.shrink.label(), reader.effect.label());
    let mut lines = Vec::new();
    let mut judged = |question_and_arms: String, status: &'static str| {
        statuses.push(JudgedStatus {
            question_and_arms,
            borrow_arm: verdict_key.cell.borrow_arm,
            leaf_capacity: verdict_key.cell.leaf_capacity,
            theta_class: theta_class_of(verdict_key.cell),
            batch: verdict_key.batch,
            window: reader.window,
            status,
        });
    };
    let new_all = aggregate.trajectory(all_key, reader, SCOPE_ALL);
    let base_all = aggregate.trajectory(all_key, paired, SCOPE_ALL);
    let difference_all = aggregate.difference(all_key, reader, SCOPE_ALL);
    // 复活：容器 r、inode r、extent r 差（新臂多出的方向）。B-tsz 的容器 r 是定义的推论，只报数（A12 钉）。
    let resurrection = [
        ("container_r", new_all[QUANTITY_CONTAINER_R], base_all[QUANTITY_CONTAINER_R].peak, "resurrects_containers"),
        ("inode_r", new_all[QUANTITY_INODE_R], base_all[QUANTITY_INODE_R].peak, "resurrects_records"),
        ("extent_r_more_than_b_on", difference_all[DIFFERENCE_EXTENT_R_MORE], 0, "resurrects_more_units"),
    ];
    for (quantity, trajectory, base_peak, positive) in resurrection {
        let reported_only = reader.base == BaseArm::TombstonesOnly && quantity == "container_r";
        let status = if reported_only { "reported" } else { status_by_threshold(trajectory.peak, positive) };
        if !reported_only {
            judged(format!("{prefix}-1 {arms} quantity={quantity}"), status);
        }
        lines.push(format!(
            "name=verdict question={prefix}-1 base={base_label} {key_label} window={} {arms} quantity={quantity} {} b_on_peak={base_peak} status={status}",
            reader.window.label(),
            trajectory.fields(quantity),
        ));
    }
    // 误判死亡：「全部」（不在 unlink 途中的扫描）与「F11 回退之后」两档。
    for (tier, key, scope) in [("all", all_key, SCOPE_OUTSIDE_UNLINK), ("after_rollback", rollback_key, SCOPE_AFTER_ROLLBACK_OUTSIDE_UNLINK)] {
        let has_data = aggregate.has_data_for(key);
        let new_tier = aggregate.trajectory(key, reader, scope);
        let base_tier = aggregate.trajectory(key, paired, scope);
        let difference_tier = aggregate.difference(key, reader, scope);
        let misjudged = [
            ("container_m", new_tier[QUANTITY_CONTAINER_M], base_tier[QUANTITY_CONTAINER_M].peak, "misjudges_container_death"),
            ("inode_m", new_tier[QUANTITY_INODE_M], base_tier[QUANTITY_INODE_M].peak, "misjudges_record_death"),
            ("extent_m_more_than_b_on", difference_tier[DIFFERENCE_EXTENT_M_MORE], 0, "misjudges_more_units"),
        ];
        for (quantity, trajectory, base_peak, positive) in misjudged {
            let status = if has_data { status_by_threshold(trajectory.peak, positive) } else { "not_run" };
            if has_data {
                judged(format!("{prefix}-2 {arms} tier={tier} quantity={quantity}"), status);
            }
            lines.push(format!(
                "name=verdict question={prefix}-2 base={base_label} {key_label} window={} {arms} tier={tier} quantity={quantity} {} b_on_peak={base_peak} status={status}",
                reader.window.label(),
                trajectory.fields(quantity),
            ));
        }
    }
    // unlink 途中漏活：E-I 只报数（A7），E-L 两臂峰值 > 0 判漏活；并排 B-on 同一格的判定。
    let new_inside = aggregate.trajectory(all_key, reader, SCOPE_INSIDE_UNLINK);
    let base_inside = aggregate.trajectory(all_key, paired, SCOPE_INSIDE_UNLINK);
    for (quantity, position) in [("inode_m", QUANTITY_INODE_M), ("extent_m", QUANTITY_EXTENT_M)] {
        let (status, base_status) = if reader.effect == UnlinkEffectArm::AtIntent {
            ("reported", "reported")
        } else {
            (status_by_threshold(new_inside[position].peak, "leaks"), status_by_threshold(base_inside[position].peak, "leaks"))
        };
        if reader.effect != UnlinkEffectArm::AtIntent {
            judged(format!("{prefix}-3 {arms} quantity={quantity}"), status);
        }
        lines.push(format!(
            "name=verdict question={prefix}-3 base={base_label} {key_label} window={} {arms} quantity={quantity} {} b_on_status={base_status} status={status}",
            reader.window.label(),
            new_inside[position].fields(&format!("{quantity}_inside_unlink")),
        ));
    }
    // X2 在新底座下的判定与 B-on 下并排（Q-Bs-4 / Q-Bt-4）。
    let new_x2 = x2_statuses(&new_all, &aggregate.trajectory(all_key, reader, SCOPE_OUTSIDE_UNLINK), &new_inside, reader);
    let base_x2 = x2_statuses(&base_all, &aggregate.trajectory(all_key, paired, SCOPE_OUTSIDE_UNLINK), &base_inside, paired);
    for ((sub_question, new_status), (_, base_status)) in new_x2.iter().zip(&base_x2) {
        let differs = new_status != base_status;
        judged(format!("{prefix}-4 {arms} sub_question={sub_question}"), new_status);
        if differs {
            differing_x2_cells.push(format!("{prefix}-4:{sub_question}:{}:{}:{}:{}:{}:{}", base_label, verdict_key.cell.label().replace(' ', ","), verdict_key.batch, reader.window.label(), reader.shrink.label(), reader.effect.label()));
        }
        lines.push(format!(
            "name=verdict question={prefix}-4 base={base_label} {key_label} window={} {arms} sub_question={sub_question} new_base_status={new_status} b_on_status={base_status} differs={differs}",
            reader.window.label()
        ));
    }
    if reader.base == BaseArm::TombstonesOnly {
        lines.push(format!(
            "name=verdict question=Q-Bt-2 base=B-tsz {key_label} window={} {arms} quantity=provenance_differences total={} judgement=reported",
            reader.window.label(),
            aggregate.provenance_differences.get(&(all_key, reader)).copied().unwrap_or(0)
        ));
    }
    lines
}

/// 交回时的并排表（修订 2 之七末段）：每个 (族 × 写路径格 × 批 × X2 × X3 × W) 一行，三个底座的复活（容器 r、inode r、extent r 峰值）
/// 与误判死亡（不在途中的容器 m、inode m、extent m 峰值）并排；B-stub 只在 N1 的格、B-tsz 只在 E-I 有。
fn side_by_side_lines(aggregate: &SegmentFourAggregate, families: &[&'static str], verdict_keys: &[VerdictKey]) -> Vec<String> {
    let peaks = |key: AggregateKey, reader: Reader| {
        let all = aggregate.trajectory(key, reader, SCOPE_ALL);
        let outside = aggregate.trajectory(key, reader, SCOPE_OUTSIDE_UNLINK);
        format!(
            "{}/{}/{}:{}/{}/{}",
            all[QUANTITY_CONTAINER_R].peak,
            all[QUANTITY_INODE_R].peak,
            all[QUANTITY_EXTENT_R].peak,
            outside[QUANTITY_CONTAINER_M].peak,
            outside[QUANTITY_INODE_M].peak,
            outside[QUANTITY_EXTENT_M].peak
        )
    };
    let mut lines = Vec::new();
    for family in families {
        for verdict_key in verdict_keys {
            let key: AggregateKey = (Some(family), *verdict_key);
            if !aggregate.has_data_for(key) {
                continue;
            }
            for window in READABLE_WINDOWS {
                for effect in UNLINK_EFFECT_ARMS {
                    for shrink in [ShrinkArm::RecordSize, ShrinkArm::RangeTombstone] {
                        let stub = if verdict_key.cell.borrow_arm == BorrowArm::RaiseSeparator {
                            peaks(key, reader_with(BaseArm::ZeroRecordStubs, shrink, effect, window))
                        } else {
                            "n/a".to_string()
                        };
                        let tombstones_only = if effect == UnlinkEffectArm::AtIntent {
                            peaks(key, reader_with(BaseArm::TombstonesOnly, shrink, effect, window))
                        } else {
                            "n/a".to_string()
                        };
                        lines.push(format!(
                            "name=side_by_side family={family} {} batch={} window={} shrink={} effect={} resurrected_c_i_e_and_misjudged_c_i_e b_on={} b_stub={stub} b_tsz={tombstones_only}",
                            verdict_key.cell.label(),
                            verdict_key.batch,
                            window.label(),
                            shrink.label(),
                            effect.label(),
                            peaks(key, reader_with(BaseArm::RetirementOnLineage, shrink, effect, window)),
                        ));
                    }
                }
            }
        }
    }
    lines
}

/// Q-Bs-5、Q-Bs-6 按 (族, 写路径格, 批) 聚合的数（修订 2 之七，报数不设门槛）。
fn stub_leaf_verdict_lines(items: &[SegmentFourItem], item_lines: &[Vec<String>]) -> Vec<String> {
    #[derive(Default)]
    struct StubLeafTotals {
        segments: u64,
        versions: u64,
        versions_with_an_empty_inode_leaf: u64,
        stubs_written: u64,
        leaves_peak_largest: u64,
        stubs_last_largest: u64,
        stub_share_last_ppm_largest: u64,
        versions_over_the_root: u64,
        first_stub_publish_smallest: Option<u64>,
    }
    let mut totals: BTreeMap<(&'static str, VerdictKey), StubLeafTotals> = BTreeMap::new();
    for (item, lines) in items.iter().zip(item_lines) {
        let SegmentFourItem::HistoryCell { history, cell } = item else {
            continue;
        };
        for line in lines.iter().filter(|line| line.starts_with("name=stub_leaves ")) {
            let fields = fields_of(line);
            let entry = totals.entry((history.family(), VerdictKey { cell: *cell, batch: history.batch() })).or_default();
            entry.segments += 1;
            entry.versions += number_field(&fields, "versions");
            entry.versions_with_an_empty_inode_leaf += number_field(&fields, "versions_with_an_empty_inode_leaf");
            entry.stubs_written += number_field(&fields, "stubs_written");
            entry.leaves_peak_largest = entry.leaves_peak_largest.max(number_field(&fields, "leaves_peak"));
            entry.stubs_last_largest = entry.stubs_last_largest.max(number_field(&fields, "stubs_last"));
            entry.stub_share_last_ppm_largest = entry.stub_share_last_ppm_largest.max(number_field(&fields, "stub_share_ppm_last"));
            entry.versions_over_the_root += number_field(&fields, "versions_over_the_root");
            if let Some(first) = fields.get("first_stub_publish").and_then(|value| value.parse::<u64>().ok()) {
                entry.first_stub_publish_smallest = Some(entry.first_stub_publish_smallest.map_or(first, |smallest| smallest.min(first)));
            }
        }
    }
    let mut lines = Vec::new();
    for ((family, verdict_key), entry) in &totals {
        let share = if entry.versions == 0 { 0 } else { entry.versions_with_an_empty_inode_leaf * 1_000_000 / entry.versions };
        lines.push(format!(
            "name=verdict question=Q-Bs-5 family={family} {} batch={} segments={} stubs_written={} leaves_peak_largest={} stubs_last_largest={} stub_share_last_ppm_largest={} versions_over_the_root={} judgement=reported",
            verdict_key.cell.label(),
            verdict_key.batch,
            entry.segments,
            entry.stubs_written,
            entry.leaves_peak_largest,
            entry.stubs_last_largest,
            entry.stub_share_last_ppm_largest,
            entry.versions_over_the_root,
        ));
        lines.push(format!(
            "name=verdict question=Q-Bs-6 family={family} {} batch={} segments={} versions={} versions_with_an_empty_inode_leaf={} share_ppm={share} first_stub_publish_smallest={} judgement=reported",
            verdict_key.cell.label(),
            verdict_key.batch,
            entry.segments,
            entry.versions,
            entry.versions_with_an_empty_inode_leaf,
            entry.first_stub_publish_smallest.map_or_else(|| "none".to_string(), |first| first.to_string()),
        ));
    }
    lines
}

/// 第四段从各项的行拼出整份产物：配置、常量锚点、各项的行（按项的次序）、判定行、并排表、Q-Bs-5/6、几何敏感性、断言总账、汇总。
/// 只读各项交回的行，不读项是现判的还是复用的（续跑与一次跑完的产物逐字节相同）。
fn assemble_segment_four(items: &[SegmentFourItem], item_lines: &[Vec<String>]) -> Vec<String> {
    let mut lines = vec![format!(
        "name=config experiment=E178 segment=four work_items={} original_path_readers={} stub_path_readers={} stub_path_borrow_arms=N1 provenance=P-C capacities=233,16 thetas=0,quarter,half windows=inf,23 ring_window={READABLE_WINDOW_RING_PUBLISHES}",
        items.len(),
        segment_four_original_path_readers().len(),
        segment_four_stub_path_readers().len(),
    )];
    lines.extend(constant_anchor_lines());
    for item_lines_of_one in item_lines {
        lines.extend(item_lines_of_one.iter().cloned());
    }
    let aggregate = aggregate_segment_four(items, item_lines);
    let mut verdict_keys: Vec<VerdictKey> = Vec::new();
    let mut families: Vec<&'static str> = Vec::new();
    for item in items {
        if let SegmentFourItem::HistoryCell { history, cell } = item {
            let key = VerdictKey { cell: *cell, batch: history.batch() };
            if !verdict_keys.contains(&key) {
                verdict_keys.push(key);
            }
            if !families.contains(&history.family()) {
                families.push(history.family());
            }
        }
    }
    verdict_keys.sort();
    let mut statuses = Vec::new();
    let mut differing_x2_cells = Vec::new();
    for verdict_key in &verdict_keys {
        for window in READABLE_WINDOWS {
            if verdict_key.cell.borrow_arm == BorrowArm::RaiseSeparator {
                for effect in UNLINK_EFFECT_ARMS {
                    for shrink in [ShrinkArm::RecordSize, ShrinkArm::RangeTombstone] {
                        let reader = reader_with(BaseArm::ZeroRecordStubs, shrink, effect, window);
                        lines.extend(new_base_verdict_lines(&aggregate, *verdict_key, reader, &mut statuses, &mut differing_x2_cells));
                    }
                }
            }
            for shrink in [ShrinkArm::RecordSize, ShrinkArm::RangeTombstone] {
                let reader = reader_with(BaseArm::TombstonesOnly, shrink, UnlinkEffectArm::AtIntent, window);
                lines.extend(new_base_verdict_lines(&aggregate, *verdict_key, reader, &mut statuses, &mut differing_x2_cells));
            }
        }
    }
    lines.push(format!(
        "name=verdict question=FL-Bx x2_cells_whose_judgement_changes_with_the_base={} cells={}",
        differing_x2_cells.len(),
        if differing_x2_cells.is_empty() { "none".to_string() } else { differing_x2_cells.join(";") }
    ));
    lines.extend(side_by_side_lines(&aggregate, &families, &verdict_keys));
    lines.extend(stub_leaf_verdict_lines(items, item_lines));
    lines.extend(stability_lines(&statuses));
    lines.extend(segment_four_summary_lines(items, item_lines, &aggregate));
    lines
}

/// 第四段的断言总账与汇总行（V2、V3、V4、V5、V6、V8、V-Bs1、V-Bs3、V-Bt1 与阳性对照的覆盖）。
fn segment_four_summary_lines(items: &[SegmentFourItem], item_lines: &[Vec<String>], aggregate: &SegmentFourAggregate) -> Vec<String> {
    let mut assertion_totals: BTreeMap<(String, String), (u64, u64)> = BTreeMap::new();
    let mut runs_not_identical = 0_u64;
    let mut last_scan_paths_disagree = 0_u64;
    let mut pairing_fails = 0_u64;
    let mut stub_read_mismatch_segments = 0_u64;
    let mut does_not_fit = 0_u64;
    let mut uncovered_f10_cells = 0_u64;
    let mut rollbacks_too_far = 0_u64;
    let mut rollback_events_missing = 0_u64;
    let mut controls: BTreeMap<String, (u64, u64, u64)> = BTreeMap::new();
    let mut literal_failures: BTreeMap<String, u64> = BTreeMap::new();
    let mut reversed_provenance_positive = 0_u64;
    let mut reversed_provenance_positive_mid = 0_u64;
    let mut stub_erasure_families: BTreeSet<String> = BTreeSet::new();
    let mut dead_record_families: BTreeSet<String> = BTreeSet::new();
    let mut construction = Vec::new();
    for (item, lines) in items.iter().zip(item_lines) {
        let family = match item {
            SegmentFourItem::HistoryCell { history, .. } => history.family(),
            SegmentFourItem::DirectRollbackConstruction => "construction",
        };
        for line in lines {
            let fields = fields_of(line);
            match fields.get("name").copied() {
                Some("simulation") => {
                    runs_not_identical += u64::from(fields.get("two_runs_identical") == Some(&"false"));
                    last_scan_paths_disagree += u64::from(fields.get("last_scan_paths_agree") == Some(&"false"));
                    pairing_fails += u64::from(fields.get("pairing_holds") == Some(&"false"));
                    stub_read_mismatch_segments += u64::from(fields.get("stub_read_mismatch_scans").is_some_and(|value| *value != "0"));
                    does_not_fit += u64::from(fields.get("does_not_fit_at").is_some_and(|value| *value != "none"));
                    rollbacks_too_far += u64::from(fields.get("rollback_too_far") == Some(&"true"));
                    rollback_events_missing += u64::from(fields.get("rollback_event_found") == Some(&"false"));
                    let original_f10_with_merges = family == "F10" && fields.get("path") == Some(&"original") && fields.get("theta") != Some(&"0");
                    if original_f10_with_merges {
                        let covered = ["borrows", "moved_then_shortened", "moved_then_unlinked"].iter().all(|key| number_field(&fields, key) > 0);
                        uncovered_f10_cells += u64::from(!covered);
                    }
                }
                Some("positive_control") => {
                    let identifier = fields.get("id").copied().unwrap_or_default().to_string();
                    if matches!(identifier.as_str(), "PC-Bs2" | "PC-Bt3") {
                        construction.push(format!("{}={}", identifier, fields.get("holds").copied().unwrap_or("missing")));
                        continue;
                    }
                    let entry = controls.entry(identifier.clone()).or_default();
                    entry.0 += 1;
                    entry.1 += u64::from(fields.get("paths_agree") != Some(&"true"));
                    entry.2 += u64::from(fields.get("closed_form_holds") != Some(&"true"));
                    if fields.get("registered_literal_holds") == Some(&"false") {
                        *literal_failures.entry(identifier.clone()).or_default() += 1;
                    }
                    if identifier == "PC-Bt2" && fields.get("hand_extent_r_z").and_then(|value| value.parse::<i64>().ok()).is_some_and(|hand| hand > 0) {
                        reversed_provenance_positive += 1;
                    }
                    if identifier == "PC-Bt2-mid" && fields.get("hand_extent_r_z").and_then(|value| value.parse::<i64>().ok()).is_some_and(|hand| hand > 0) {
                        reversed_provenance_positive_mid += 1;
                    }
                    let window_taken = fields.get("window_taken").copied().unwrap_or_default();
                    if identifier == "PC-Bs1" {
                        stub_erasure_families.insert(format!("{family}/W{window_taken}"));
                    }
                    if identifier == "PC-Bt1" {
                        dead_record_families.insert(format!("{family}/W{window_taken}"));
                    }
                }
                Some("item_assertion") => {
                    let key = (fields.get("path").copied().unwrap_or_default().to_string(), fields.get("assertion").copied().unwrap_or_default().to_string());
                    let entry = assertion_totals.entry(key).or_default();
                    entry.0 += number_field(&fields, "judged");
                    entry.1 += number_field(&fields, "violated");
                }
                Some(_) | None => {}
            }
        }
    }
    let mut lines = Vec::new();
    let mut every_assertion_holds = true;
    for ((path, assertion), (judged, violated)) in &assertion_totals {
        every_assertion_holds &= *violated == 0;
        lines.push(format!("name=assertion path={path} assertion={assertion} judged={judged} violated={violated}"));
    }
    for (identifier, (count, paths_disagree, closed_form_fails)) in &controls {
        lines.push(format!(
            "name=positive_control_tally id={identifier} count={count} paths_disagree={paths_disagree} closed_form_fails={closed_form_fails} registered_literal_fails={}",
            literal_failures.get(identifier).copied().unwrap_or(0)
        ));
    }
    let new_base_read_out_empty: u64 = aggregate
        .read_out_empty_trajectories
        .iter()
        .filter(|(reader, _)| matches!(reader.base, BaseArm::ZeroRecordStubs | BaseArm::TombstonesOnly))
        .map(|(_, count)| *count)
        .sum();
    lines.push(format!(
        "name=verdict question=summary v2_paths_disagree={} v2_closed_form_fails={} v3_last_scan_paths_disagree={last_scan_paths_disagree} v4_runs_not_identical={runs_not_identical} v5_anchors_hold={every_assertion_holds} v6_f10_cells_without_coverage={uncovered_f10_cells} v8_new_base_trajectories_with_empty_reads={new_base_read_out_empty} v_bs1_pairing_fails={pairing_fails} v_bs3_segments_with_stub_read_mismatches={stub_read_mismatch_segments} does_not_fit_segments={does_not_fit} f11_rollbacks_too_far={rollbacks_too_far} f11_events_missing={rollback_events_missing} pc_bt2_segments_with_a_positive_hand_count={reversed_provenance_positive} pc_bt2_mid_segments_with_a_positive_hand_count={reversed_provenance_positive_mid} pc_bs1_taken={} pc_bt1_taken={} construction={}",
        controls.values().map(|(_, paths_disagree, _)| paths_disagree).sum::<u64>(),
        controls.values().map(|(_, _, closed_form_fails)| closed_form_fails).sum::<u64>(),
        if stub_erasure_families.is_empty() { "none".to_string() } else { stub_erasure_families.into_iter().collect::<Vec<String>>().join(",") },
        if dead_record_families.is_empty() { "none".to_string() } else { dead_record_families.into_iter().collect::<Vec<String>>().join(",") },
        construction.join(","),
    ));
    lines
}

/// 第四段入口：补算没落盘的项；带 `--shard` 只落项文件、打一行进度；不带时 997 项齐了拼产物。现判、复用数打到 stderr（产物与续跑无关）。
fn run_segment_four(threads: usize, options: &SegmentFourOptions) -> Vec<String> {
    let items = segment_four_items();
    let (item_lines, computed, reused) = compute_or_reuse_the_items(&items, threads, options);
    eprintln!("segment-four: 现判 {computed}、复用 {reused}（共 {} 项，分片 {:?}，项目录 {}）", items.len(), options.shard, options.items_directory.display());
    if options.shard.is_some() {
        return vec![format!("name=shard_progress computed={computed} reused={reused} items={}", items.len())];
    }
    let complete: Vec<Vec<String>> = item_lines.into_iter().map(|lines| lines.expect("不分片时每一项都算过或读回")).collect();
    assemble_segment_four(&items, &complete)
}

/// `segment-four` 的参数：`--shard i/k`、`--no-reuse`；项目录取 E178_SEGMENT_FOUR_ITEMS，默认在 work 目录下。
fn segment_four_options_from(arguments: &[String]) -> Result<SegmentFourOptions, String> {
    let mut shard = None;
    let mut reuse = true;
    let mut remaining = arguments.iter();
    while let Some(argument) = remaining.next() {
        match argument.as_str() {
            "--no-reuse" => reuse = false,
            "--shard" => {
                let value = remaining.next().ok_or("--shard 后面缺 i/k")?;
                let (index, count) = value.split_once('/').ok_or("--shard 的值写成 i/k")?;
                let index: usize = index.parse().map_err(|_| "--shard 的 i 不是数")?;
                let count: usize = count.parse().map_err(|_| "--shard 的 k 不是数")?;
                if count == 0 || index >= count {
                    return Err("--shard 要 0 ≤ i < k".to_string());
                }
                shard = Some((index, count));
            }
            other => return Err(format!("认不得的参数：{other}")),
        }
    }
    let items_directory = std::env::var_os("E178_SEGMENT_FOUR_ITEMS").map_or_else(
        || std::path::PathBuf::from(std::env::var_os("HOME").unwrap_or_default()).join(".local/share/singlefs/work/e178-segment-four/items"),
        std::path::PathBuf::from,
    );
    Ok(SegmentFourOptions { shard, reuse, items_directory })
}

fn threads_from_the_environment() -> usize {
    std::env::var("E178_THREADS")
        .ok()
        .and_then(|value| value.parse::<usize>().ok())
        .filter(|threads| *threads >= 1)
        .unwrap_or(DEFAULT_THREADS)
}

fn main() {
    let arguments: Vec<String> = std::env::args().collect();
    let mode = arguments.get(1).cloned().unwrap_or_default();
    let threads = threads_from_the_environment();
    let usage = "用法：e178-leaf-and-record-death-counts segment-one | segment-two | segment-three | segment-four [--shard i/k] [--no-reuse]";
    let emitted_lines = match mode.as_str() {
        "segment-one" => run_segment_one(threads),
        "segment-two" => run_segment_two(threads),
        "segment-three" => run_segment_three(threads),
        "segment-four" => match segment_four_options_from(&arguments[2..]) {
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

#[cfg(test)]
mod tests {
    use super::*;

    fn cell(borrow_arm: BorrowArm, leaf_capacity: u64, merge_threshold: u64) -> WritePathCell {
        WritePathCell { borrow_arm, leaf_capacity, merge_threshold }
    }

    fn position_of(base: BaseArm, shrink: ShrinkArm, effect: UnlinkEffectArm, window: ReadableWindow) -> usize {
        reader_position(Reader { base, shrink, effect, window })
    }

    fn shape_named(name: &str) -> TreeShape {
        catalogue_of_shapes().into_iter().find(|named| named.name == name).expect("形态表里有它").shape
    }

    /// K1、K2、K3、A2 与净荷容量：条款给的值（加法写，不写减法）。
    #[test]
    fn format_constants_match_the_clause_values() {
        assert_eq!(INODE_LEAF_RECORDS, 232 + 1);
        assert_eq!(TOMBSTONE_CONTAINER_RECORDS, 814 + 1);
        assert_eq!(ROOT_NODE_LEAF_CONTAINERS, 134 + 1);
        assert_eq!(RETIREMENT_RECORD_BYTES, 33 + 1);
        assert_eq!(RETIREMENT_CONTAINER_RECORDS, 958 + 1);
        assert_eq!(DATA_UNIT_PAYLOAD_BYTES, 32_633 + 1);
        assert_eq!(every_reader().len(), 12 + 8);
    }

    #[test]
    fn the_shape_catalogue_holds_every_registered_family() {
        let catalogue = catalogue_of_shapes();
        let count = |family: &str| catalogue.iter().filter(|named| named.family == family).count();
        assert_eq!((count("K"), count("S"), count("BN1"), count("BN2"), count("DV")), (6, 22, 9, 4, 3));
    }

    /// 停机对拍 S6：K-0 全绿，K-1 至 K-5 只红今天的 I-9.4；K-2（右号 8）N1 的 I-9.4 绿、I-9.12 绿（登记第四节第 4 条推的那一格）。
    #[test]
    fn known_bad_shapes_redden_only_the_container_number_invariant() {
        let (holds, detail) = stop_check_known_bad_images_match_the_implementation();
        assert!(holds, "{detail}");
        let second = judge_the_shape(&shape_named("K-2"));
        assert!(!second.today_container_numbers_hold && second.raised_separator_container_numbers_hold && second.separators_hold);
    }

    /// N1 的谓词：BN1 分隔 key 没抬（s = 20，左最大 20）红，抬对（s = 30）绿（第九节 M12 盯这一句）。
    #[test]
    fn an_unraised_separator_reddens_the_raised_separator_reading() {
        assert!(!judge_the_shape(&shape_named("BN1-s20")).raised_separator_container_numbers_hold);
        assert!(judge_the_shape(&shape_named("BN1-s30")).raised_separator_container_numbers_hold);
        assert!(!judge_the_shape(&shape_named("BN1-s30")).today_container_numbers_hold);
    }

    #[test]
    fn routing_takes_the_last_separator_not_above_the_key() {
        assert!(judge_the_shape(&shape_named("K-0")).routing_holds);
        assert!(!judge_the_shape(&shape_named("DV-right-absorbs-left")).routing_holds);
        assert!(!judge_the_shape(&shape_named("BN1-s31")).routing_holds);
    }

    #[test]
    fn the_sparse_layout_scales_keys_and_takes_midpoints_between_keys() {
        let sparse = sparse_layout_of(&shape_named("K-0"));
        assert_eq!(sparse.separators, vec![100, 1500]);
        assert_eq!(sparse.leaves[1].container_number, 1500);
        assert_eq!(sparse.leaves[0].record_keys, vec![100, 1000]);
        assert_eq!(scaled_by_one_hundred(0, &[1, 10]), 0);
        assert_eq!(scaled_by_one_hundred(31, &[1, 10, 20, 30]), 3100);
    }

    #[test]
    fn stop_check_split_matches_the_implementation_rules() {
        let (holds, detail) = stop_check_split_matches_the_implementation();
        assert!(holds, "{detail}");
    }

    #[test]
    fn stop_check_removal_matches_the_implementation_rules() {
        let (holds, detail) = stop_check_removal_matches_the_implementation();
        assert!(holds, "{detail}");
    }

    #[test]
    fn stop_check_unlink_steps_match_the_implementation_rules() {
        let (holds, detail) = stop_check_unlink_steps_match_the_implementation();
        assert!(holds, "{detail}");
    }

    #[test]
    fn stop_check_intent_matches_the_implementation_rules() {
        let (holds, detail) = stop_check_intent_matches_the_implementation();
        assert!(holds, "{detail}");
    }

    #[test]
    fn stop_check_tombstone_tree_matches_the_implementation_rules() {
        let (holds, detail) = stop_check_tombstone_tree_matches_the_implementation();
        assert!(holds, "{detail}");
    }

    /// A6：一借小历史（第九节 M5、M6 在这里红：N2 不换身份则右号仍是 17，N1 不抬分隔 key 则右分隔 key 仍是 17）。
    #[test]
    fn a_single_borrow_history_moves_five_records_and_costs_one_retirement() {
        let (holds, detail) = single_borrow_anchor_holds();
        assert!(holds, "{detail}");
    }

    /// A5：θ = 0 时 N1、N2 两条写流逐发布相同（叶容器摘要与写的单元数），一次借都没有。
    #[test]
    fn without_merges_both_borrow_arms_write_the_same_stream() {
        let history = History::BuildAndDeleteEmptyInodes { inode_count: 300, order: DeletionOrder::Random(1) };
        let raise_separator = run_write_path(history, cell(BorrowArm::RaiseSeparator, TEST_LEAF_CAPACITY, 0));
        let renew_identity = run_write_path(history, cell(BorrowArm::RenewIdentity, TEST_LEAF_CAPACITY, 0));
        let pair = pair_statistics(&raise_separator, &renew_identity);
        assert!(pair.streams_identical && pair.user_changes_identical && pair.publish_counts_identical);
        assert_eq!(pair.borrow_publishes, 0);
    }

    /// A8：θ = 0 时退役记录数 = 身份消失次数，F1 n = 300 容量 16 是 ⌈300 ÷ 16⌉ − 1 = 18（第九节 M4 在这里红）。
    #[test]
    fn at_zero_threshold_every_emptied_leaf_writes_one_retirement_record() {
        let history = History::BuildAndDeleteEmptyInodes { inode_count: 300, order: DeletionOrder::Ascending };
        let run = run_write_path(history, cell(BorrowArm::RenewIdentity, TEST_LEAF_CAPACITY, 0));
        assert_eq!(run.written.retirement_records, 17 + 1);
        assert_eq!(run.drops, 17 + 1);
        assert_eq!(run.assertions.violations_of("A8_retirements_equal_identity_disappearances"), 0);
    }

    /// A1：H51 逐段重放的发布数与被拒步数（F1 式子 607；F8 式子；F3 五段与 E177 产物那两段相同）（第九节 M14 在这里红）。
    #[test]
    fn the_replayed_histories_match_the_registered_publish_counts() {
        let first = run_write_path(History::BuildAndDeleteEmptyInodes { inode_count: 300, order: DeletionOrder::EvenThenOdd }, cell(BorrowArm::RaiseSeparator, INODE_LEAF_RECORDS, 0));
        assert_eq!(first.publishes, 606 + 1);
        let large = run_write_path(
            History::LargeFileRewriteAndDelete { data_units: 300, rewrites: 5, overwrite_before_unlink: false, batch: 144 },
            cell(BorrowArm::RaiseSeparator, INODE_LEAF_RECORDS, 0),
        );
        assert_eq!(large.publishes, 12 + 1);
        for (seed, publishes, refused) in E177_MIXED_RANDOM_PUBLISHES_AND_REFUSALS {
            let mixed = run_write_path(History::MixedRandom { seed }, cell(BorrowArm::RenewIdentity, TEST_LEAF_CAPACITY, 8));
            assert_eq!((mixed.publishes, mixed.refused_steps), (publishes, refused), "seed {seed}");
            assert_eq!(mixed.assertions.violations_of("A1_history_publishes"), 0);
        }
    }

    /// V7：F10 在 θ > 0 下，N2 的每一版按今天的 I-9.4 与 I-9.12 全绿，N1 的每一版按 N1 的 I-9.4 与 I-9.12 全绿；借、吸收、
    /// 被挪过的对象变短与被 unlink 都跑到（第九节 M5、M6、M7 在这里红）。
    #[test]
    fn merging_many_objects_keeps_every_published_tree_legal() {
        let history = History::ManyObjectsWithData { inode_count: 300, order: DeletionOrder::Ascending, batch: 144 };
        for threshold in [4, 8] {
            for borrow_arm in BORROW_ARMS {
                let run = run_write_path(history, cell(borrow_arm, TEST_LEAF_CAPACITY, threshold));
                assert!(self_check_holds(borrow_arm, threshold, run.self_check), "{borrow_arm:?} θ {threshold}: {:?}", run.self_check);
                assert!(path_was_covered(&run), "{borrow_arm:?} θ {threshold}: borrows {} shortened {} unlinked {}", run.borrows, run.moved_then_shortened, run.moved_then_unlinked);
                assert!(run.absorbs > 0);
                assert_eq!(run.assertions.violations_of("K6_split_right_half_numbered_after_the_record_left_half_unchanged"), 0);
                if borrow_arm == BorrowArm::RaiseSeparator {
                    assert!(run.self_check.today_container_numbers_red > 0, "N1 借之后今天的 I-9.4 红是定义的推论");
                }
            }
        }
    }

    /// Q-X4-1、Q-X4-2 的定义推论：每次借 N2 − N1 = 码 3 + 1、码 2 + 1，N2 退役记录 1 条、N1 0 条，别的发布逐单元相同。
    #[test]
    fn every_borrow_costs_renewed_identity_one_container_and_one_root() {
        let history = History::ManyObjectsWithData { inode_count: 300, order: DeletionOrder::Random(2), batch: 1 };
        let raise_separator = run_write_path(history, cell(BorrowArm::RaiseSeparator, TEST_LEAF_CAPACITY, 8));
        let renew_identity = run_write_path(history, cell(BorrowArm::RenewIdentity, TEST_LEAF_CAPACITY, 8));
        let pair = pair_statistics(&raise_separator, &renew_identity);
        assert!(pair.borrow_publishes > 0);
        assert_eq!((pair.extra_code_three_per_borrow.smallest, pair.extra_code_three_per_borrow.largest), (Some(1), Some(1)));
        assert_eq!((pair.extra_code_two_per_borrow.smallest, pair.extra_code_two_per_borrow.largest), (Some(1), Some(1)));
        assert_eq!(pair.retirement_records_per_borrow_renew_identity.largest, Some(1));
        assert_eq!(pair.retirement_records_per_borrow_raise_separator.largest, Some(0));
        assert_eq!(pair.publishes_outside_borrows_with_a_difference + pair.borrow_kinds_disagree, 0);
        assert!(pair.user_changes_identical);
    }

    #[test]
    fn the_ring_window_reads_a_unit_replaced_twenty_three_publishes_ago() {
        assert!(ReadableWindow::RingWindow.allows(22 + 1));
        assert!(!ReadableWindow::RingWindow.allows(23 + 1));
        assert!(ReadableWindow::Forever.allows(u64::MAX));
    }

    #[test]
    fn the_parent_chain_skips_the_segment_a_rollback_abandoned() {
        let chain = ParentChain::of(30, &[(20, 10)]);
        assert!(chain.contains(30) && chain.contains(20) && chain.contains(10) && chain.contains(1));
        assert!(!chain.contains(15) && !chain.contains(19) && !chain.contains(11) && !chain.contains(31));
        assert!(ParentChain::of(9, &[(20, 10)]).contains(9));
    }

    #[test]
    fn the_judgement_threshold_moves_a_cell_from_green_to_red() {
        assert_eq!(status_above(3, 4, "resurrects"), "zero");
        assert_eq!(status_above(3, 2, "resurrects"), "resurrects");
        assert_eq!(status_by_threshold(1, "resurrects"), "resurrects");
        assert_eq!(status_by_threshold(0, "resurrects"), "zero");
    }

    #[test]
    fn a_trajectory_reports_peak_positive_scans_and_last() {
        let mut trajectory = QuantityTrajectory::default();
        for (value, publish) in [(0, 1), (3, 2), (1, 3), (0, 4)] {
            trajectory.observe(value, publish);
        }
        assert_eq!((trajectory.peak, trajectory.peak_at_publish, trajectory.positive_scans, trajectory.last, trajectory.scans), (3, 2, 2, 0, 4));
    }

    /// A3 与第九节 M1、M2 的取样点：F8 n_u = 3、删前覆盖写，覆盖写之后那次扫描 Z、T 的 extent r 都是 0，G0 是 n_u − 1 = 2。
    #[test]
    fn after_an_overwrite_shrinks_a_file_only_the_object_gate_resurrects_its_tail() {
        let history = History::LargeFileRewriteAndDelete { data_units: 3, rewrites: 0, overwrite_before_unlink: true, batch: 144 };
        let (simulation, _) = simulate(history, cell(BorrowArm::RaiseSeparator, INODE_LEAF_RECORDS, 0), true);
        let overwrite_scan = simulation
            .publish_records
            .iter()
            .zip(&simulation.scans)
            .find(|(record, _)| record.shrink.is_some())
            .map(|(_, scan)| scan.clone())
            .expect("覆盖写那次变短");
        for (reader, counts) in every_reader().iter().zip(&overwrite_scan.counts) {
            let expected = if reader.shrink == ShrinkArm::ObjectGateOnly { 2 } else { 0 };
            assert_eq!(counts.extent_resurrected, expected, "{}", reader.label());
            assert_eq!(counts.extent_missed, 0, "{}", reader.label());
        }
    }

    /// A7 与第九节 M8 的取样点：F8 n_u = 300、b = 144，E-I 下 unlink 途中三次扫描 inode m = 1、extent m = 300、156、12。
    #[test]
    fn the_intent_effect_misses_the_record_and_every_remaining_unit_in_the_window() {
        let history = History::LargeFileRewriteAndDelete { data_units: 300, rewrites: 0, overwrite_before_unlink: false, batch: 144 };
        let (simulation, _) = simulate(history, cell(BorrowArm::RaiseSeparator, INODE_LEAF_RECORDS, 0), true);
        let intent_effect = position_of(BaseArm::RetirementOnLineage, ShrinkArm::RecordSize, UnlinkEffectArm::AtIntent, ReadableWindow::Forever);
        let deferred_read = position_of(BaseArm::RetirementOnLineage, ShrinkArm::RecordSize, UnlinkEffectArm::AtLastBatchRead, ReadableWindow::Forever);
        let in_window: Vec<&ScanRecord> = simulation.scans.iter().filter(|scan| scan.inside_an_unlink).collect();
        let extent_missed: Vec<u64> = in_window.iter().map(|scan| scan.counts[intent_effect].extent_missed).collect();
        assert_eq!(extent_missed, vec![300, 156, 12]);
        assert!(in_window.iter().all(|scan| scan.counts[intent_effect].inode_missed == 1));
        assert!(in_window.iter().all(|scan| scan.counts[deferred_read].inode_missed == 0 && scan.counts[deferred_read].extent_missed == 0));
    }

    /// 第九节 M9 的取样点：H51 F1 n = 300，E-L-r 的途中 inode m 恒 0，E-I 的是 1。
    #[test]
    fn the_deferred_read_effect_keeps_the_record_while_it_is_still_in_the_tree() {
        let history = History::BuildAndDeleteEmptyInodes { inode_count: 300, order: DeletionOrder::Ascending };
        let (simulation, _) = simulate(history, cell(BorrowArm::RaiseSeparator, INODE_LEAF_RECORDS, 0), true);
        let deferred_read = position_of(BaseArm::RetirementOnLineage, ShrinkArm::RecordSize, UnlinkEffectArm::AtLastBatchRead, ReadableWindow::Forever);
        let intent_effect = position_of(BaseArm::RetirementOnLineage, ShrinkArm::RecordSize, UnlinkEffectArm::AtIntent, ReadableWindow::Forever);
        let trajectories = trajectories_of(&simulation.scans, every_reader().len());
        assert_eq!(trajectories[deferred_read].by_scope[SCOPE_INSIDE_UNLINK][QUANTITY_INODE_M].peak, 0);
        assert_eq!(trajectories[intent_effect].by_scope[SCOPE_INSIDE_UNLINK][QUANTITY_INODE_M].peak, 1);
        assert_eq!(trajectories[intent_effect].by_scope[SCOPE_INSIDE_UNLINK][QUANTITY_INODE_M].positive_scans, 299);
    }

    /// 第九节 M10 的取样点：F12 P-1、u = 145、b = 1，意图那一版之后崩溃（被抛弃的是第一批），恢复之后写行那一版的扫描：
    /// txg 已重发、被抛弃那一批的叶容器落在父链上，只有已发布谓词挡它；E-L-w × Z 的 extent m 是 0。
    #[test]
    fn the_published_predicate_hides_the_batch_the_crash_abandoned() {
        let (prefix, object) = build_crash_prefix(CrashPrefix::FirstFileOnly, cell(BorrowArm::RaiseSeparator, INODE_LEAF_RECORDS, 0)).expect("P-1 前缀");
        let mut simulation = prefix;
        simulation.batch_data_units = 1;
        simulation.apply(UserChange::WriteFile { object, data_units: 145 });
        assert!(simulation.publish(&Operation::UnlinkIntent { object }));
        let abandoned_txg = simulation.version.txg + 1;
        assert!(simulation.crash_with_the_next_unlink_step_written().is_some());
        simulation.scanning = true;
        simulation.recover_after_the_crash();
        let row_scan = &simulation.scans[0];
        assert_eq!(row_scan.txg, abandoned_txg, "写行那次重发了被抛弃的 txg");
        for window in READABLE_WINDOWS {
            let last_batch_written = position_of(BaseArm::RetirementOnLineage, ShrinkArm::RecordSize, UnlinkEffectArm::AtLastBatchWritten, window);
            assert_eq!(row_scan.counts[last_batch_written].extent_missed, 0);
            assert_eq!(row_scan.counts[last_batch_written].inode_missed, 0);
        }
    }

    /// 第九节 M13：同号多条时取写序最大的那个容器里的那条。植入一片写序最大、装 (1, b, size 0) 的叶容器，Z 判 inode 1 的单元死（extent m = 1）。
    #[test]
    fn duplicate_records_are_taken_from_the_container_with_the_newest_write_order() {
        let simulation = Simulation::start(cell(BorrowArm::RaiseSeparator, INODE_LEAF_RECORDS, 0), PRODUCT_UNLINK_BATCH_DATA_UNITS, false);
        let record = simulation.version.record_of(FIRST_FILE_INODE).expect("新池新建文件");
        let planted = InodeRecord { inode: FIRST_FILE_INODE, object_birth: record.object_birth, nlink: 1, size_bytes: 0, change_counter: 0 };
        let overlay = Overlay::planting(vec![Simulation::planted_leaf(planted, simulation.newest_write_order(), simulation.publish_sequence)]);
        for path in [SelectionPath::GroupIndex, SelectionPath::FullScan] {
            let counts = simulation.read_every_reader(&overlay, simulation.publish_sequence, path);
            let record_size = position_of(BaseArm::RetirementOnLineage, ShrinkArm::RecordSize, UnlinkEffectArm::AtIntent, ReadableWindow::Forever);
            let object_gate = position_of(BaseArm::RetirementOnLineage, ShrinkArm::ObjectGateOnly, UnlinkEffectArm::AtIntent, ReadableWindow::Forever);
            assert_eq!(counts[record_size].extent_missed, 1, "{path:?}");
            assert_eq!(counts[object_gate].extent_missed, 0, "{path:?}");
            assert_eq!(counts[record_size].container_resurrected, 1, "{path:?}");
        }
    }

    /// 第九节 M15 的取样点：F11 吸收缺口（容量 16、θ = 8、N2），回退之后又追加过墓碑与退役记录。B-on × E-I 下回退之后不在途中的扫描
    /// 容器 m、inode m 都是 0（缺口里的退役记录与墓碑死亡 txg 不在新父链上，不施行）。B-off-lin 在这一段量到的容器 m 不在这里钉：
    /// 回退之后第一次追加退役记录就是重跑缺口里第一次身份消失，左移重写又给后面几片换了诞生 txg（修订 1）。
    #[test]
    fn after_a_rollback_retirements_and_tombstones_from_the_gap_do_not_apply() {
        let (simulation, outcome) = simulate(History::RollBackOverAGap { event: GapEvent::Absorb }, cell(BorrowArm::RenewIdentity, TEST_LEAF_CAPACITY, 8), true);
        let outcome = outcome.expect("F11 交回回退的结局");
        assert!(outcome.event_found);
        assert!(outcome.retirement_records_after_the_rollback > 0);
        assert!(outcome.retired_in_the_gap_and_present_in_the_old_version > 0);
        let trajectories = trajectories_of(&simulation.scans, every_reader().len());
        let on_lineage = position_of(BaseArm::RetirementOnLineage, ShrinkArm::RecordSize, UnlinkEffectArm::AtIntent, ReadableWindow::Forever);
        let after_rollback = &trajectories[on_lineage].by_scope[SCOPE_AFTER_ROLLBACK_OUTSIDE_UNLINK];
        assert!(after_rollback[QUANTITY_CONTAINER_M].scans > 0);
        assert_eq!(after_rollback[QUANTITY_CONTAINER_M].peak, 0);
        assert_eq!(after_rollback[QUANTITY_INODE_M].peak, 0);
    }

    /// 第九节 M3 的敏感取样点与 PC-B2 的方向（修订 1 的直接构造）：缺口里删空 L3（退役记录死亡 txg 在缺口里），回退之后 L3 回来、
    /// 版本还是缺口之前那一版；再删空最右的 L4（不左移）让退役容器在新父链上重写。B-on 的容器 m 是 0，B-off-lin 是 1。
    #[test]
    fn a_retirement_from_the_abandoned_segment_applies_only_when_lineage_is_ignored() {
        let (holds, detail) = direct_rollback_positive_control_holds();
        assert!(holds, "{detail}");
    }

    /// 第十一节 V3：主路径（按组索引）与参照路径（整扫）在多对象带数据、合并与回退的历史末尾逐读者逐量相同。
    #[test]
    fn the_group_index_reads_the_same_as_a_full_scan() {
        for (history, write_path) in [
            (History::ManyObjectsWithData { inode_count: 300, order: DeletionOrder::Random(3), batch: 1 }, cell(BorrowArm::RenewIdentity, TEST_LEAF_CAPACITY, 4)),
            (History::RollBackOverAGap { event: GapEvent::Drop }, cell(BorrowArm::RaiseSeparator, TEST_LEAF_CAPACITY, 0)),
            (History::MixedRandom { seed: 3 }, cell(BorrowArm::RaiseSeparator, TEST_LEAF_CAPACITY, 8)),
        ] {
            let (simulation, _) = simulate(history, write_path, false);
            assert!(simulation.group_index_equals_the_full_scan(simulation.publish_sequence), "{}", history.label());
        }
    }

    /// 第二段一个工作项：两遍逐次扫描相同；PC-Z、PC-T、PC-EL、PC-ELr、PC-M 两路对得上、登记写的期望成立；A4、K4 不红。
    #[test]
    fn positive_controls_on_a_many_objects_history_agree_on_both_paths() {
        let item = SegmentTwoItem {
            history: History::ManyObjectsWithData { inode_count: 300, order: DeletionOrder::Ascending, batch: 144 },
            cell: cell(BorrowArm::RenewIdentity, TEST_LEAF_CAPACITY, 4),
        };
        let result = run_segment_two_item(item);
        assert!(result.two_runs_identical && result.last_scan_paths_agree);
        let identifiers: BTreeSet<&str> = result.positive_controls.iter().map(|outcome| outcome.id).collect();
        for expected in ["PC-Z", "PC-T", "PC-EL", "PC-ELr", "PC-M-data", "PC-M-leaf"] {
            assert!(identifiers.contains(expected), "{expected} 没做：{identifiers:?}");
        }
        for outcome in &result.positive_controls {
            assert!(outcome.paths_agree && outcome.closed_form_holds, "{}", outcome.line(""));
        }
        for name in ["A4_extra_writes_are_one_and_one", "K4_one_whole_object_tombstone_per_unlink"] {
            assert!(result.assertions.judged_of(name) > 0 && result.assertions.violations_of(name) == 0, "{name}");
        }
    }

    /// 第三段：P-1、u = 3、b = 1 上三个崩溃点；两路对得上，PC-O 两项与 PC-M 的期望成立；E-I 下崩溃点 extent m = 这一版剩下的单元数。
    #[test]
    fn crash_branches_count_what_the_rebuilt_view_misses() {
        let (prefix, object) = build_crash_prefix(CrashPrefix::FirstFileOnly, cell(BorrowArm::RaiseSeparator, INODE_LEAF_RECORDS, 0)).expect("P-1 前缀");
        let outcomes = run_crash_branches(&prefix, object, 3, 1);
        assert_eq!(outcomes.len(), 3);
        let intent_effect = position_of(BaseArm::RetirementOnLineage, ShrinkArm::RecordSize, UnlinkEffectArm::AtIntent, ReadableWindow::Forever);
        let deferred_read = position_of(BaseArm::RetirementOnLineage, ShrinkArm::RecordSize, UnlinkEffectArm::AtLastBatchRead, ReadableWindow::Forever);
        let missed: Vec<u64> = outcomes.iter().map(|outcome| outcome.crash_counts[intent_effect].extent_missed).collect();
        assert_eq!(missed, vec![3, 2, 1]);
        for outcome in &outcomes {
            assert!(outcome.crash_paths_agree && outcome.recovery_paths_agree);
            assert_eq!(outcome.crash_counts[deferred_read].records_whose_intent_is_published, 1);
            assert_eq!(outcome.crash_counts[intent_effect].records_whose_intent_is_published, 0);
            for control in &outcome.positive_controls {
                assert!(control.paths_agree && control.closed_form_holds, "{}", control.line(""));
            }
        }
    }

    /// 第一段的 5.6 表：「今天 I-9.4 红而 N1 的 I-9.4 绿」的名单（判定由谓词算，不是抄的）里有 K-2。
    #[test]
    fn the_shape_table_lists_the_shapes_the_raised_separator_reading_misses() {
        let (_, verdicts) = shape_table_lines();
        let lines = shape_verdict_lines(&verdicts);
        let dense_k = lines.iter().find(|line| line.contains("question=Q-X4-4 layout=dense family=K ")).expect("有 K 族那一行");
        assert!(dense_k.contains("K-2"), "{dense_k}");
    }

    // ---------------------------------------------------------------------------------------------
    // 第四段（登记修订 2）
    // ---------------------------------------------------------------------------------------------

    fn stub_path_simulation(history: History, write_path: WritePathCell, scanning: bool) -> (Simulation, Option<RollbackOutcome>) {
        let mut simulation = Simulation::start_with(write_path, history.batch(), scanning, ContainerDeathWritePath::ZeroRecordStubs, segment_four_stub_path_readers());
        let outcome = run_history(history, &mut simulation);
        (simulation, outcome)
    }

    fn original_path_simulation(history: History, write_path: WritePathCell, scanning: bool) -> Simulation {
        let mut simulation =
            Simulation::start_with(write_path, history.batch(), scanning, ContainerDeathWritePath::RetirementRecords, segment_four_original_path_readers());
        run_history(history, &mut simulation);
        simulation
    }

    fn position_in(readers: &[Reader], wanted: Reader) -> usize {
        readers.iter().position(|reader| *reader == wanted).expect("读者表里有它")
    }

    #[test]
    fn sha256_matches_the_standard_test_vectors() {
        assert_eq!(sha256_hex(b""), "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855");
        assert_eq!(sha256_hex(b"abc"), "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad");
        assert_eq!(sha256_hex(&[b'a'; 1_000]).len(), 63 + 1);
    }

    /// A10、A11 与第九节修订 2 的 M20、M21：H51 F1 n = 300 升序、容量 16、θ = 0，B-stub 写 18 个桩、期末 18 个桩、叶片总数 19；
    /// 第一个桩在第 71 次发布，有桩的版本 537 个（`anchors3.out`、`anchors4.out`）。
    #[test]
    fn stub_write_path_on_emptied_inodes_matches_the_closed_forms() {
        let history = History::BuildAndDeleteEmptyInodes { inode_count: 300, order: DeletionOrder::Ascending };
        let (simulation, _) = stub_path_simulation(history, cell(BorrowArm::RaiseSeparator, TEST_LEAF_CAPACITY, 0), false);
        let stubs_written: u64 = simulation.publish_records.iter().map(|record| record.stubs_written).sum();
        assert_eq!((stubs_written, simulation.version.stub_count()), (17 + 1, 17 + 1));
        let last = simulation.publish_records.last().expect("有发布");
        assert_eq!((last.leaves_in_the_leaf_order, last.stubs_in_the_leaf_order), (18 + 1, 17 + 1));
        let versions_with_an_empty_leaf = simulation.publish_records.iter().filter(|record| record.holds_an_empty_inode_leaf).count();
        assert_eq!(versions_with_an_empty_leaf, 536 + 1);
        let first_stub = simulation.publish_records.iter().position(|record| record.stubs_in_the_leaf_order > 0).map(|position| position + 1);
        assert_eq!(first_stub, Some(70 + 1));
        assert_eq!(first_stub_publish_and_versions_with_a_stub(300, TEST_LEAF_CAPACITY), (70 + 1, 536 + 1));
        assert_eq!(first_stub_publish_and_versions_with_a_stub(900, INODE_LEAF_RECORDS), (950 + 1, 868 + 1));
        assert_eq!(simulation.total_written().retirement_records, 0);
        let wide = History::BuildAndDeleteEmptyInodes { inode_count: 900, order: DeletionOrder::EvenThenOdd };
        let (wide_simulation, _) = stub_path_simulation(wide, cell(BorrowArm::RaiseSeparator, INODE_LEAF_RECORDS, 0), false);
        assert_eq!((wide_simulation.version.stub_count(), length_as_u64(wide_simulation.version.leaves.len())), (2 + 1, 3 + 1));
    }

    /// V-Bs1 与第九节修订 2 的 M22：B-stub 写路径与原写路径逐版的活叶、数据单元、txg 与扫描相同（桩不当邻片）。
    #[test]
    fn stub_write_path_keeps_every_live_leaf_of_the_original_write_path() {
        for (history, write_path) in [
            (History::ManyObjectsWithData { inode_count: 300, order: DeletionOrder::Ascending, batch: 144 }, cell(BorrowArm::RaiseSeparator, TEST_LEAF_CAPACITY, 8)),
            (History::ManyObjectsWithData { inode_count: 300, order: DeletionOrder::Random(2), batch: 1 }, cell(BorrowArm::RaiseSeparator, TEST_LEAF_CAPACITY, 4)),
            (History::MixedRandom { seed: 3 }, cell(BorrowArm::RaiseSeparator, TEST_LEAF_CAPACITY, 8)),
            (History::RollBackOverAGap { event: GapEvent::Absorb }, cell(BorrowArm::RaiseSeparator, TEST_LEAF_CAPACITY, 8)),
        ] {
            let original = original_path_simulation(history, write_path, false);
            let (stub, _) = stub_path_simulation(history, write_path, false);
            assert!(pairing_holds(&original, &stub), "{}", history.label());
            assert!(stub.version.stub_count() > 0, "{} 一个桩都没有", history.label());
            assert!(stub.publish_records.iter().any(|record| matches!(record.merge, MergeAction::AbsorbedTheRightNeighbour | MergeAction::AbsorbedIntoTheLeftNeighbour)));
        }
    }

    /// PC-Bs2、PC-Bt3（修订 2 之六）与第九节 M15 补的那一格：构造期末 B-stub 0 / 0、B-stub-lin-off 1 / 1 / 0；B-tsz 不按谱系滤时 inode m + 1。
    #[test]
    fn the_rollback_construction_separates_the_lineage_of_stubs_and_tombstones() {
        let (stub_holds, stub_detail) = stub_lineage_positive_control();
        assert!(stub_holds, "{stub_detail}");
        let (tombstone_holds, tombstone_detail) = tombstone_lineage_positive_control();
        assert!(tombstone_holds, "{tombstone_detail}");
    }

    /// 第九节修订 2 的 M17：H51 F1 n = 300 升序、容量 16、θ = 0、W = ∞，B-stub × Z × E-I 期末容器 r = 0、读到的桩 18 个（跳过桩就多 18）。
    #[test]
    fn stub_readers_select_the_stub_and_read_no_dead_container() {
        let history = History::BuildAndDeleteEmptyInodes { inode_count: 300, order: DeletionOrder::Ascending };
        let (simulation, _) = stub_path_simulation(history, cell(BorrowArm::RaiseSeparator, TEST_LEAF_CAPACITY, 0), true);
        let readers = segment_four_stub_path_readers();
        let reader = position_in(&readers, reader_with(BaseArm::ZeroRecordStubs, ShrinkArm::RecordSize, UnlinkEffectArm::AtIntent, ReadableWindow::Forever));
        let last = simulation.scans.last().expect("有扫描").counts[reader];
        assert_eq!((last.container_resurrected, last.container_missed, last.stubs_read), (0, 0, 17 + 1));
        assert!(simulation.scans.iter().zip(&simulation.publish_records).all(|(scan, record)| scan.counts[reader].stubs_read == record.stubs_in_the_leaf_order));
    }

    /// A12 与第九节修订 2 的 M19：同一取样点上 B-tsz × Z × E-I 期末容器 r = 18（= B-off-ret，A8）、inode r = 0（不施行整对象墓碑就多 18）。
    #[test]
    fn tombstones_only_reads_every_dead_container_and_drops_its_deleted_records() {
        let history = History::BuildAndDeleteEmptyInodes { inode_count: 300, order: DeletionOrder::Ascending };
        let simulation = original_path_simulation(history, cell(BorrowArm::RaiseSeparator, TEST_LEAF_CAPACITY, 0), true);
        let readers = segment_four_original_path_readers();
        let tombstones_only = position_in(&readers, reader_with(BaseArm::TombstonesOnly, ShrinkArm::RecordSize, UnlinkEffectArm::AtIntent, ReadableWindow::Forever));
        let without_retirement = position_in(&readers, reader_with(BaseArm::WithoutRetirement, ShrinkArm::RecordSize, UnlinkEffectArm::AtIntent, ReadableWindow::Forever));
        let last = simulation.scans.last().expect("有扫描");
        assert_eq!(last.counts[tombstones_only].container_resurrected, 17 + 1);
        assert_eq!(last.counts[tombstones_only].inode_resurrected, 0);
        assert!(simulation.scans.iter().all(|scan| scan.counts[tombstones_only].container_resurrected == scan.counts[without_retirement].container_resurrected));
        // PC-Bt1：删空那 18 片的最后一版装着最后删掉的那条（nlink = 0）；拿掉它的整对象墓碑，B-tsz inode r + 1、B-on + 0。
        for window in READABLE_WINDOWS {
            let controls = simulation.dead_record_positive_control(simulation.publish_sequence, window);
            assert_eq!(controls.len(), 1, "{window:?}");
            assert!(controls[0].paths_agree && controls[0].closed_form_holds, "{}", controls[0].line(""));
        }
    }

    /// 第九节修订 2 的 M16：F11 吸收缺口（容量 16、θ = 8、N1），W = ∞，B-stub × Z × E-I 从那次吸收到回退之前的扫描里容器 r 都是 0
    /// （吸收时不落桩，被吸收那片的最后一版就被读回）。
    #[test]
    fn an_absorbed_leaf_leaves_a_stub_that_hides_its_last_version() {
        let (simulation, outcome) = stub_path_simulation(History::RollBackOverAGap { event: GapEvent::Absorb }, cell(BorrowArm::RaiseSeparator, TEST_LEAF_CAPACITY, 8), true);
        let outcome = outcome.expect("F11 交回回退的结局");
        let gap_start = simulation.gap_start.clone().expect("吸收缺口出现了");
        let readers = segment_four_stub_path_readers();
        let reader = position_in(&readers, reader_with(BaseArm::ZeroRecordStubs, ShrinkArm::RecordSize, UnlinkEffectArm::AtIntent, ReadableWindow::Forever));
        let in_the_gap: Vec<&ScanRecord> = simulation
            .scans
            .iter()
            .filter(|scan| scan.publish_sequence >= gap_start.event_publish_sequence && scan.publish_sequence < outcome.rollback_publish_sequence)
            .collect();
        assert!(!in_the_gap.is_empty());
        assert!(in_the_gap.iter().all(|scan| scan.counts[reader].container_resurrected == 0));
        assert!(in_the_gap.iter().any(|scan| scan.counts[reader].stubs_read > 0));
    }

    /// PC-Bt2、PC-Bt2-mid（登记修订 3）与第九节修订 2 的 M18：F10、N2、容量 16、θ = 4、W = ∞。期末那次扫描（原 PC-Bt2）手算值是 0
    /// （F10 删光 2..n）；最后一次「有被挪过、之后变短的活对象」那次扫描上，倒置供出规则之后 B-tsz × Z 的 extent r 增量（手算）> 0，
    /// 而 P-C 下那次扫描 B-tsz × Z × E-I 的 extent r 是 0（供出规则取写序最小的副本就变成那个增量）。
    #[test]
    fn reversed_provenance_resurrects_the_tail_of_a_moved_then_shortened_object() {
        let write_path = cell(BorrowArm::RenewIdentity, TEST_LEAF_CAPACITY, 4);
        let readers = segment_four_original_path_readers();
        let tombstones_only = position_in(&readers, reader_with(BaseArm::TombstonesOnly, ShrinkArm::RecordSize, UnlinkEffectArm::AtIntent, ReadableWindow::Forever));
        let hand_of = |control: &PositiveControlOutcome| -> i64 { control.detail.trim_start_matches("hand_extent_r_z=").parse().expect("手算值是数") };
        let mut found = false;
        for order in deletion_orders() {
            let history = History::ManyObjectsWithData { inode_count: 300, order, batch: PRODUCT_UNLINK_BATCH_DATA_UNITS };
            let run = run_segment_four_path(history, write_path, ContainerDeathWritePath::RetirementRecords, &readers);
            let at_the_end = run.positive_controls.iter().find(|control| control.id == "PC-Bt2").expect("期末那次做了 PC-Bt2");
            assert!(at_the_end.paths_agree && at_the_end.closed_form_holds && hand_of(at_the_end) == 0, "{}", at_the_end.line(""));
            let Some(mid) = run.positive_controls.iter().find(|control| control.id == "PC-Bt2-mid") else {
                continue;
            };
            assert!(mid.paths_agree && mid.closed_form_holds, "{}", mid.line(""));
            if hand_of(mid) > 0 {
                let scan = run.simulation.scans.iter().find(|scan| scan.publish_sequence == mid.at_publish).expect("那次扫描在");
                assert_eq!(scan.counts[tombstones_only].extent_resurrected, 0, "{order:?}");
                found = true;
                break;
            }
        }
        assert!(found, "F10 n = 300、N2、容量 16、θ = 4、批 144 的七段里没有一段 PC-Bt2-mid 手算值 > 0");
    }

    /// 第四段一个工作项（F10、N1、容量 16、θ = 4）：两条写路径配对、读到的桩数对得上、阳性对照两路一致且期望成立、A12 不红。
    #[test]
    fn a_segment_four_item_holds_its_pairing_controls_and_anchors() {
        let lines = segment_four_history_item_lines(
            History::ManyObjectsWithData { inode_count: 300, order: DeletionOrder::Ascending, batch: 144 },
            cell(BorrowArm::RaiseSeparator, TEST_LEAF_CAPACITY, 4),
        );
        let stub_simulation = lines.iter().find(|line| line.starts_with("name=simulation segment=four") && line.contains("path=stub")).expect("有 B-stub 写路径那一行");
        assert!(stub_simulation.contains("pairing_holds=true") && stub_simulation.contains("stub_read_mismatch_scans=0"), "{stub_simulation}");
        let controls: Vec<&String> = lines.iter().filter(|line| line.starts_with("name=positive_control")).collect();
        for expected in ["id=PC-Bs1", "id=PC-Bt2", "id=PC-Z", "id=PC-EL ", "id=PC-M-leaf"] {
            assert!(controls.iter().any(|line| line.contains(expected)), "{expected} 没做");
        }
        for line in &controls {
            assert!(line.contains("paths_agree=true") && line.contains("closed_form_holds=true"), "{line}");
        }
        for line in lines.iter().filter(|line| line.starts_with("name=item_assertion")) {
            assert!(line.ends_with("violated=0"), "{line}");
        }
        assert!(lines.iter().any(|line| line.starts_with("name=difference") && line.contains("base=B-stub")));
    }

    /// 续跑（修订 2 之十三）：小输入上关掉复用跑一遍、再开着复用跑一遍，拼出的产物逐行相同，第二遍全是复用。
    #[test]
    fn resuming_from_the_item_files_assembles_the_same_product() {
        let items = vec![
            SegmentFourItem::HistoryCell {
                history: History::LargeFileRewriteAndDelete { data_units: 3, rewrites: 0, overwrite_before_unlink: true, batch: 1 },
                cell: cell(BorrowArm::RaiseSeparator, TEST_LEAF_CAPACITY, 0),
            },
            SegmentFourItem::DirectRollbackConstruction,
        ];
        let directory = std::env::temp_dir().join(format!("e178-segment-four-resume-test-{}", std::process::id()));
        let fresh_options = SegmentFourOptions { shard: None, reuse: false, items_directory: directory.clone() };
        let (fresh_lines, fresh_computed, fresh_reused) = compute_or_reuse_the_items(&items, 2, &fresh_options);
        let reuse_options = SegmentFourOptions { shard: None, reuse: true, items_directory: directory.clone() };
        let (reused_lines, reused_computed, reused_reused) = compute_or_reuse_the_items(&items, 1, &reuse_options);
        let shard_options = SegmentFourOptions { shard: Some((1, 2)), reuse: true, items_directory: directory.clone() };
        let (shard_lines, _, _) = compute_or_reuse_the_items(&items, 1, &shard_options);
        std::fs::remove_dir_all(&directory).expect("删得掉测试自己建的项目录");
        assert_eq!((fresh_computed, fresh_reused, reused_computed, reused_reused), (2, 0, 0, 2));
        assert!(shard_lines[0].is_none() && shard_lines[1].is_some());
        let fresh: Vec<Vec<String>> = fresh_lines.into_iter().map(|lines| lines.expect("每项都算了")).collect();
        let reused: Vec<Vec<String>> = reused_lines.into_iter().map(|lines| lines.expect("每项都读回了")).collect();
        assert_eq!(assemble_segment_four(&items, &fresh), assemble_segment_four(&items, &reused));
    }

    /// 第二段 PC-B1 在三段 F11 上不成立（产物里那三行 `holds=false`）的成因：消失过的身份里期末没被 B-off-ret × Z × E-I × ∞ 读回的，
    /// 盘上每一版的诞生 txg 都不在期末那一版的父链上（在缺口里生、又在缺口里消失）；登记 PC-B1 的期望数的是消失过的身份，没扣这一类。
    /// (消失过的身份数, 期末容器 r) 照第二段产物那三行钉：(4, 3)、(6, 4)、(5, 4)。
    #[test]
    fn identities_born_and_retired_inside_the_abandoned_gap_are_what_pc_b1_overcounts() {
        let without_retirement_forever = position_of(BaseArm::WithoutRetirement, ShrinkArm::RecordSize, UnlinkEffectArm::AtIntent, ReadableWindow::Forever);
        for (event, threshold, expected) in [(GapEvent::Borrow, 4, (4, 3)), (GapEvent::Borrow, 8, (6, 4)), (GapEvent::MovedObjectShortened, 8, (5, 4))] {
            let (simulation, _) = simulate(History::RollBackOverAGap { event }, cell(BorrowArm::RenewIdentity, TEST_LEAF_CAPACITY, threshold), false);
            let final_identities: HashSet<ContainerIdentity> = simulation.version.leaves.iter().map(|leaf| leaf.identity).collect();
            let disappeared: BTreeSet<ContainerIdentity> = simulation
                .publish_records
                .iter()
                .flat_map(|record| record.retired_identities.iter().copied())
                .filter(|identity| !final_identities.contains(identity))
                .collect();
            let chain = ParentChain::of(simulation.version.txg, &simulation.version.lineage);
            let born_off_the_chain_only = disappeared
                .iter()
                .filter(|identity| {
                    simulation.disk.units.iter().all(|unit| match &unit.body {
                        UnitBody::InodeLeaf { identity: unit_identity, .. } => unit_identity != *identity || !chain.contains(unit.write_order.birth_txg),
                        UnitBody::Data { .. } | UnitBody::Tombstones { .. } | UnitBody::Retirements { .. } => true,
                    })
                })
                .count();
            let counts = simulation.read_every_reader(&Overlay::default(), simulation.publish_sequence, SelectionPath::GroupIndex);
            let final_container_r = counts[without_retirement_forever].container_resurrected;
            assert_eq!((length_as_u64(disappeared.len()), final_container_r), expected, "{event:?} θ {threshold}");
            assert!(born_off_the_chain_only > 0, "{event:?} θ {threshold}");
            assert_eq!(final_container_r + length_as_u64(born_off_the_chain_only), length_as_u64(disappeared.len()), "{event:?} θ {threshold}");
        }
    }

    #[test]
    fn segment_four_arguments_take_a_shard_and_refuse_a_wrong_one() {
        let parse = |arguments: &[&str]| segment_four_options_from(&arguments.iter().map(|argument| (*argument).to_string()).collect::<Vec<String>>());
        assert_eq!(parse(&["--shard", "1/3", "--no-reuse"]).map(|options| (options.shard, options.reuse)), Ok((Some((1, 3)), false)));
        assert!(parse(&["--shard", "3/3"]).is_err());
        assert!(parse(&["--shard"]).is_err());
        assert!(parse(&["--bogus"]).is_err());
    }
}
