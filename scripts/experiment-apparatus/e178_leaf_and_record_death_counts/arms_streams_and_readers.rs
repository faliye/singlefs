//! 借的臂、写路径的格、底座臂与墓碑写流、读者的窗口与按臂取的参数。

use super::*;

// ---------------------------------------------------------------------------------------------
// 臂（登记 5.3）与写路径格、读者
// ---------------------------------------------------------------------------------------------

/// X4 的两条臂：借时右片怎么处理身份。
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(super) enum BorrowArm {
    /// N1：右片保留身份，分隔 key 抬到借之后右片的最小 key。
    RaiseSeparator,
    /// N2：右片换身份（容器号 = 借之后的最小 key、出生代 = 本次 txg），旧身份写退役记录。
    RenewIdentity,
}

pub(super) const BORROW_ARMS: [BorrowArm; 2] = [BorrowArm::RaiseSeparator, BorrowArm::RenewIdentity];

impl BorrowArm {
    pub(super) const fn label(self) -> &'static str {
        match self {
            BorrowArm::RaiseSeparator => "N1",
            BorrowArm::RenewIdentity => "N2",
        }
    }
}

/// 一个写路径格：N × 叶容器容量 × 合并门槛 θ（登记 5.1 几何旋钮、5.2 末段）。
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(super) struct WritePathCell {
    pub(super) borrow_arm: BorrowArm,
    pub(super) leaf_capacity: u64,
    pub(super) merge_threshold: u64,
}

impl WritePathCell {
    pub(super) fn label(self) -> String {
        format!(
            "borrow_arm={} capacity={} theta={}",
            self.borrow_arm.label(),
            self.leaf_capacity,
            self.merge_threshold
        )
    }
}

pub(super) const LEAF_CAPACITIES: [u64; 2] = [INODE_LEAF_RECORDS, TEST_LEAF_CAPACITY];

/// θ ∈ {0, ⌊容量 ÷ 4⌋, ⌊容量 ÷ 2⌋}（登记 5.1）。
pub(super) const fn merge_thresholds_of(leaf_capacity: u64) -> [u64; 3] {
    [0, leaf_capacity / 4, leaf_capacity / 2]
}

/// 底座三臂（登记 5.3）。
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(super) enum BaseArm {
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
    pub(super) const fn label(self) -> &'static str {
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
pub(super) enum Provenance {
    /// P-W：所在版本写序最大的那一条。
    NewestWriteOrder,
    /// P-C：记录偏移 88 的改动计数最大的那一条，平局按 P-W。
    LargestChangeCounter,
    /// PC-Bt2 的倒置：所在版本写序最小的那一条。
    OldestWriteOrder,
}

/// 读者用哪条供出规则：B-tsz 取 P-C（修订 2 问法二），别的底座照原登记取 P-W。
pub(super) const fn provenance_of(base: BaseArm) -> Provenance {
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
pub(super) enum ShrinkArm {
    /// Z：按重建出的 inode 记录的对象出生代与 size 折成的单元数判死活。
    RecordSize,
    /// T：变短那次往墓碑树写区间墓碑。
    RangeTombstone,
    /// G0：只按对象门控、不看 size（阳性对照）。
    ObjectGateOnly,
}

impl ShrinkArm {
    pub(super) const fn label(self) -> &'static str {
        match self {
            ShrinkArm::RecordSize => "Z",
            ShrinkArm::RangeTombstone => "T",
            ShrinkArm::ObjectGateOnly => "G0",
        }
    }
}

/// X3 的三条臂：unlink 的整对象墓碑从哪一次发布起生效。
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[allow(clippy::enum_variant_names, reason = "变体名里的 At 说明墓碑从哪一次发布起生效，去掉前缀就看不出各臂的差别")]
pub(super) enum UnlinkEffectArm {
    /// E-I：意图那次写、那次起生效（今天的 `crates/`）。
    AtIntent,
    /// E-L-w：最后一批那次写。
    AtLastBatchWritten,
    /// E-L-r：意图那次写，读者只在树里已经没有那条记录时才施行。
    AtLastBatchRead,
}

impl UnlinkEffectArm {
    pub(super) const fn label(self) -> &'static str {
        match self {
            UnlinkEffectArm::AtIntent => "E-I",
            UnlinkEffectArm::AtLastBatchWritten => "E-L-w",
            UnlinkEffectArm::AtLastBatchRead => "E-L-r",
        }
    }
}

pub(super) const UNLINK_EFFECT_ARMS: [UnlinkEffectArm; 3] =
    [UnlinkEffectArm::AtIntent, UnlinkEffectArm::AtLastBatchWritten, UnlinkEffectArm::AtLastBatchRead];

/// 旋钮 W：换下的单元在之后几次发布的扫描里还读得出（登记 5.1）。
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(super) enum ReadableWindow {
    Forever,
    RingWindow,
}

pub(super) const READABLE_WINDOWS: [ReadableWindow; 2] = [ReadableWindow::Forever, ReadableWindow::RingWindow];

impl ReadableWindow {
    pub(super) const fn label(self) -> &'static str {
        match self {
            ReadableWindow::Forever => "inf",
            ReadableWindow::RingWindow => "23",
        }
    }

    /// 在发布序号 p_r 那次被换下的单元，在发布序号 p 的扫描里读得出当且仅当 p − p_r ≤ W。
    pub(super) const fn allows(self, publishes_since_replaced: u64) -> bool {
        match self {
            ReadableWindow::Forever => true,
            ReadableWindow::RingWindow => publishes_since_replaced <= READABLE_WINDOW_RING_PUBLISHES,
        }
    }
}

/// 墓碑写流：X2 的 Z（含 G0，都不写变短记录）/ T 与 X3 的三条臂配出来的五条（墓碑只随这两个臂变，别的臂共用一条写路径）。
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[allow(clippy::enum_variant_names, reason = "变体名里的 WholeAt 与 RangesOn 分别说明整对象墓碑与区间墓碑各在哪一次写，去掉前缀就混了")]
pub(super) enum TombstoneStream {
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

pub(super) const TOMBSTONE_STREAM_COUNT: usize = 5;

pub(super) const TOMBSTONE_STREAMS: [TombstoneStream; TOMBSTONE_STREAM_COUNT] = [
    TombstoneStream::WholeAtIntent,
    TombstoneStream::WholeAtFinal,
    TombstoneStream::WholeAtIntentRangesOnUserShrinks,
    TombstoneStream::WholeAtIntentRangesOnEveryShrink,
    TombstoneStream::WholeAtFinalRangesOnEveryShrink,
];

impl TombstoneStream {
    pub(super) const fn position(self) -> usize {
        match self {
            TombstoneStream::WholeAtIntent => 0,
            TombstoneStream::WholeAtFinal => 1,
            TombstoneStream::WholeAtIntentRangesOnUserShrinks => 2,
            TombstoneStream::WholeAtIntentRangesOnEveryShrink => 3,
            TombstoneStream::WholeAtFinalRangesOnEveryShrink => 4,
        }
    }

    pub(super) const fn writes_the_whole_object_tombstone_at_the_intent(self) -> bool {
        match self {
            TombstoneStream::WholeAtIntent
            | TombstoneStream::WholeAtIntentRangesOnUserShrinks
            | TombstoneStream::WholeAtIntentRangesOnEveryShrink => true,
            TombstoneStream::WholeAtFinal | TombstoneStream::WholeAtFinalRangesOnEveryShrink => false,
        }
    }

    pub(super) const fn writes_ranges_when_a_user_change_shrinks(self) -> bool {
        match self {
            TombstoneStream::WholeAtIntent | TombstoneStream::WholeAtFinal => false,
            TombstoneStream::WholeAtIntentRangesOnUserShrinks
            | TombstoneStream::WholeAtIntentRangesOnEveryShrink
            | TombstoneStream::WholeAtFinalRangesOnEveryShrink => true,
        }
    }

    /// E-L 两臂下不是最后一批的每一批也算变短（登记 5.3 的 T 那一行）。
    pub(super) const fn writes_ranges_at_every_batch_that_is_not_the_last(self) -> bool {
        match self {
            TombstoneStream::WholeAtIntent
            | TombstoneStream::WholeAtFinal
            | TombstoneStream::WholeAtIntentRangesOnUserShrinks => false,
            TombstoneStream::WholeAtIntentRangesOnEveryShrink | TombstoneStream::WholeAtFinalRangesOnEveryShrink => true,
        }
    }
}

pub(super) const fn tombstone_stream_of(shrink: ShrinkArm, effect: UnlinkEffectArm) -> TombstoneStream {
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
pub(super) struct Reader {
    pub(super) base: BaseArm,
    pub(super) shrink: ShrinkArm,
    pub(super) effect: UnlinkEffectArm,
    pub(super) window: ReadableWindow,
}

impl Reader {
    pub(super) fn label(self) -> String {
        format!(
            "base={} shrink={} effect={} window={}",
            self.base.label(),
            self.shrink.label(),
            self.effect.label(),
            self.window.label()
        )
    }

    pub(super) const fn tombstone_stream(self) -> TombstoneStream {
        tombstone_stream_of(self.shrink, self.effect)
    }

    /// 主格：B-on × {Z, T} × X3 三臂 × W 两档。
    pub(super) fn is_a_main_cell(self) -> bool {
        self.base == BaseArm::RetirementOnLineage && self.shrink != ShrinkArm::ObjectGateOnly
    }
}

/// 每次扫描都读的 20 个读者：主格 12 个，对照格 8 个（登记 5.4 末段），次序固定。
pub(super) fn every_reader() -> Vec<Reader> {
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
pub(super) fn segment_four_original_path_readers() -> Vec<Reader> {
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
pub(super) fn segment_four_stub_path_readers() -> Vec<Reader> {
    every_reader()
        .into_iter()
        .filter(|reader| reader.is_a_main_cell())
        .map(|reader| Reader { base: BaseArm::ZeroRecordStubs, shrink: reader.shrink, effect: reader.effect, window: reader.window })
        .collect()
}
