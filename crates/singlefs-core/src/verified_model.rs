//! core 的值对到抽象状态的规格模型：分配器记录、中央映射、根环与两层 F 的 spec 类型与 spec fn，另有被验区域用到的
//! `singlefs-format` 常量的规格。
//!
//! 出处：「层与射程」「抽象状态」
//! 这份文件是规格底座：只放 spec 与钉住它们取值的见证（proof fn），不接协议核，也不被产品代码调用；
//! 把这里的模型接到 `singlefs-replay-prefix` 的 `gc_ring_interval_proofs`（搬迁协议的根环版）与 `gc_crash_proofs`（回收门槛）归后面的簇。
//! 与协议核的抽象状态对得上的地方，每一处在注释里写一行指过去；两边的差别（协议层是数学整数、没有设备，core 是 u64、两块盘各一份）
//! 也写在那一行。
//!
//! 每个 core 值的 `View`（把运行时的分配器、映射、根环投影成这里的模型）照门禁「被验类型的 trait impl 一律写在 `verus!` 里」写在定义它的
//! 那份文件里，由那份文件所在的簇写。
use vstd::prelude::*;

use crate::address::{CheckpointTxg, DeviceIdentity, SlotNumber};

verus! {

// ===== singlefs-format 常量的规格 =====
// Verus 不认没进 verus! 的 crate 里的常量（直接引报「`singlefs_format::SLOT_BYTES` is not supported」），所以被验区域用到的格式常量
// 各写一条 assume_specification，交回它在 singlefs-format 里的值。
// 约定：各簇把自己用到的常量规格写在自己的文件里（这一段只放 SLOT_BYTES 与 MAPPING_KEY_BYTES），每条另在信任清单
// .claude/gate.d/verus-trust-list.tsv 登一行（由主 agent 登）；跨文件的一致性由本文件 tests 模块的
// every_format_constant_specification_returns_the_value_singlefs_format_defines 判：它扫 core 全部源文件里的这种规格，同一个常量
// 有两处规格、交回值或类型后缀与 singlefs-format 源码里那一行整数字面量不符、点名的常量在那里找不到或不是整数字面量，都红。
// Verus 编的时候把 assume_specification 展开成一个 unsafe 的 fn 声明，撞 lib.rs 的 deny(unsafe_code)，所以每一条各带一个 allow；
// 平时的 cargo build 把整条擦掉，产品代码里没有它。
#[allow(unsafe_code, reason = "Verus 把 assume_specification 展开成 unsafe fn 声明；只在 Verus 编的时候有，产品构建里擦掉")]
pub assume_specification[ singlefs_format::SLOT_BYTES ] -> (slot_bytes: u64) returns 16384u64;
#[allow(unsafe_code, reason = "Verus 把 assume_specification 展开成 unsafe fn 声明；只在 Verus 编的时候有，产品构建里擦掉")]
pub assume_specification[ singlefs_format::MAPPING_KEY_BYTES ] -> (mapping_key_bytes: u64) returns 27u64;

// ===== 槽区间 =====
// 一个单元在一块盘上占的槽：起点加跨度（槽数）。对到 gc_ring_interval_proofs 的 SlotRun：那里起点与跨度是 int、不带设备，
// 这里起点是 core 的 SlotNumber（u64），接协议核时要另证起点加跨度不越过 u64（slot_run_fits_in_u64）。
pub struct SlotRunModel {
    pub start: SlotNumber,
    pub span_slots: nat,
}

pub open spec fn slot_run_end(run: SlotRunModel) -> int {
    run.start.0 + run.span_slots
}

pub open spec fn slot_run_fits_in_u64(run: SlotRunModel) -> bool {
    &&& run.span_slots >= 1
    &&& slot_run_end(run) <= u64::MAX
}

// 对到 gc_ring_interval_proofs 的 run_covers。
pub open spec fn slot_run_covers(run: SlotRunModel, slot: SlotNumber) -> bool {
    run.start.0 <= slot.0 && slot.0 < slot_run_end(run)
}

// 对到 gc_ring_interval_proofs 的 runs_overlap。
pub open spec fn slot_runs_overlap(this_run: SlotRunModel, other_run: SlotRunModel) -> bool {
    this_run.start.0 < slot_run_end(other_run) && other_run.start.0 < slot_run_end(this_run)
}

// ===== 分配器记录 =====
// 一条分配记录在它的生命里处在哪一种：仍分配（带分配代）、仍分配而被隔离（隔离记号置上，分配代不变）、已释放（带释放代）。
// 已释放的记录不带隔离记号（D19（块指针的结构与宽度预算）已定项 5「带记号的记录必须未释放」），所以没有「已释放而被隔离」这一种。
pub enum AllocationRecordLifeModel {
    Allocated { allocation_generation: CheckpointTxg },
    AllocatedAndQuarantined { allocation_generation: CheckpointTxg },
    Released { release_generation: CheckpointTxg },
}

// 分配记录 20（D3（空间分配）已定项 7 / 已定项 11）的模型：哪块盘、罩住哪一段槽、处在哪一种生命。
pub struct AllocationRecordModel {
    pub device: DeviceIdentity,
    pub run: SlotRunModel,
    pub life: AllocationRecordLifeModel,
}

// 分配记录树的 key：(设备身份, 起点槽号)，与 allocator.rs 里 AllocationRecord::sort_key 同序。
pub struct AllocationRecordKeyModel {
    pub device: DeviceIdentity,
    pub start: SlotNumber,
}

// 分配器的记录账：key 到记录。
pub struct AllocatorRecordsModel {
    pub records: Map<AllocationRecordKeyModel, AllocationRecordModel>,
}

pub open spec fn record_is_keyed_by_its_start(key: AllocationRecordKeyModel, record: AllocationRecordModel) -> bool {
    key.device == record.device && key.start == record.run.start
}

pub open spec fn records_are_keyed_by_their_start(records: AllocatorRecordsModel) -> bool {
    forall|key: AllocationRecordKeyModel| records.records.contains_key(key)
        ==> record_is_keyed_by_its_start(key, #[trigger] records.records[key])
}

// I-5.4（分配记录罩住的槽互不相交）：同一块盘上两条不同的记录，罩住的槽不重叠。
pub open spec fn records_on_one_device_are_disjoint(records: AllocatorRecordsModel) -> bool {
    forall|this_key: AllocationRecordKeyModel, other_key: AllocationRecordKeyModel|
        #![trigger records.records[this_key], records.records[other_key]]
        records.records.contains_key(this_key) && records.records.contains_key(other_key) && this_key != other_key
            && this_key.device == other_key.device
            ==> !slot_runs_overlap(records.records[this_key].run, records.records[other_key].run)
}

pub open spec fn record_is_released(record: AllocationRecordModel) -> bool {
    record.life matches AllocationRecordLifeModel::Released { .. }
}

// 回收门槛：已释放、释放代不大于 floor。对到 gc_crash_proofs 的 released_record_can_be_reclaimed（产品经它判，allocator.rs 回收时调它）
// 那条 ensures 的 released && generation <= floor；那里的 generation 与 floor 是裸 u64，这里是 CheckpointTxg。
pub open spec fn record_can_be_reclaimed_under_the_floor(record: AllocationRecordModel, floor: CheckpointTxg) -> bool {
    match record.life {
        AllocationRecordLifeModel::Allocated { .. } => false,
        AllocationRecordLifeModel::AllocatedAndQuarantined { .. } => false,
        AllocationRecordLifeModel::Released { release_generation } => release_generation.0 <= floor.0,
    }
}

// 这块盘上这个槽被一条没释放的记录罩着（仍分配，或仍分配而被隔离）。
pub open spec fn slot_is_allocated_on_the_device(records: AllocatorRecordsModel, device: DeviceIdentity, slot: SlotNumber) -> bool {
    exists|key: AllocationRecordKeyModel|
        records.records.contains_key(key) && key.device == device && slot_run_covers((#[trigger] records.records[key]).run, slot)
            && !record_is_released(records.records[key])
}

// ===== 中央映射 =====
// 中央映射里一条的一个位置：哪块盘、哪一段槽（D19（块指针的结构与宽度预算）已定项 10 的位置条目）。
pub struct MappingLocationModel {
    pub device: DeviceIdentity,
    pub run: SlotRunModel,
}

// 中央映射里一条的值：两条位置条目，两盘同槽（D2（RAID 条带策略）已定项 10：一个单元整个落在一列上、两盘各一份）。
pub struct MappingValueModel {
    pub copies: Seq<MappingLocationModel>,
}

pub open spec fn mapping_value_has_one_copy_on_each_of_two_devices(value: MappingValueModel) -> bool {
    &&& value.copies.len() == 2
    &&& value.copies[0].device != value.copies[1].device
    &&& value.copies[0].run == value.copies[1].run
    &&& slot_run_fits_in_u64(value.copies[0].run)
}

// 一版的中央映射：映射 key（MAPPING_KEY_BYTES 字节）到值。对到 gc_ring_interval_proofs 的 RunView 的 mapping：那里 key 是 u64、值是一段 SlotRun，
// 这里 key 是盘上的字节串、值是两盘同槽的两份；接协议核时由后面的簇给出 key 的单射与「两份同槽」到一段 SlotRun 的投影。
pub struct CentralMappingModel {
    pub entries: Map<Seq<u8>, MappingValueModel>,
}

pub open spec fn central_mapping_is_shaped(mapping: CentralMappingModel) -> bool {
    forall|key: Seq<u8>| mapping.entries.contains_key(key)
        ==> key.len() == singlefs_format::MAPPING_KEY_BYTES && mapping_value_has_one_copy_on_each_of_two_devices(#[trigger] mapping.entries[key])
}

// 对到 gc_ring_interval_proofs 的 mapping_avoids_slot：映射里没有一条在这块盘上罩住这个槽。
pub open spec fn central_mapping_avoids_the_slot(mapping: CentralMappingModel, device: DeviceIdentity, slot: SlotNumber) -> bool {
    forall|key: Seq<u8>, copy: int| mapping.entries.contains_key(key) && 0 <= copy < mapping.entries[key].copies.len()
        && (#[trigger] mapping.entries[key].copies[copy]).device == device
        ==> !slot_run_covers(mapping.entries[key].copies[copy].run, slot)
}

// ===== 根环与两层 F =====
// 根是谁发的：用户那一侧的发布（用户点得到它的版本），或整理的发布（占根环的一个根，用户点不到）。
// 对到 gc_ring_interval_proofs 的 RootName：那里用户的根带的是协议的版本号，这里带的是根记录的 checkpoint 号。
pub enum RootPublisherModel {
    UserPublish,
    CompactionPublish,
}

// 根环里一条持久的根：根记录的 checkpoint 号、它带着的回退下界 F（root_record.rs 的 RootRecord::rollback_floor）、它那一版的中央映射、谁发的。
// 对到 gc_ring_interval_proofs 的 RingRoot。
pub struct RingRootModel {
    pub checkpoint_txg: CheckpointTxg,
    pub rollback_floor: CheckpointTxg,
    pub mapping: CentralMappingModel,
    pub publisher: RootPublisherModel,
}

// 根环与两层 F。对到 gc_ring_interval_proofs 的 RunRingState：
// - roots 旧到新，最后一条是崩溃后重现的那一条；
// - in_memory_floor 是内存 F（回退候选的下界），persisted_floor 是持久 F（最后一条持久的根带着的 F，回收与覆写要避开它之上的每个根）；
//   协议里两层 F 是根序列的下标，这里是 checkpoint 号，对上要另证根的 checkpoint 号严格递增（roots_ascend_by_checkpoint_txg）；
// - head 是内存里的现行映射，可以领先最后一条持久的根；
// - window 是根环容得下的根数。
// 抬 F 先只改内存 F，带新 F 的根落满每块盘之后持久 F 才跟上（allocator.rs 的 hold_until_floor_takes_effect 与 release_reclaim_holds 扣住、
// 放开回收出来的槽）；崩溃让内存 F 回到持久 F（root_ring_after_a_crash）。
pub struct RootRingModel {
    pub roots: Seq<RingRootModel>,
    pub in_memory_floor: CheckpointTxg,
    pub persisted_floor: CheckpointTxg,
    pub head: CentralMappingModel,
    pub window: nat,
}

pub open spec fn roots_ascend_by_checkpoint_txg(roots: Seq<RingRootModel>) -> bool {
    forall|earlier: int, later: int|
        #![trigger roots[earlier], roots[later]]
        0 <= earlier < later < roots.len() ==> roots[earlier].checkpoint_txg.0 < roots[later].checkpoint_txg.0
}

// 对到 gc_ring_interval_proofs 的 run_ring_is_shaped，逐条：
// - 至少一条根、根环至少容一条：那边的 roots.len() >= 1 与 window >= 1；
// - 持久 F 不高于内存 F：那边的 persisted_floor <= in_memory_floor；
// - 内存 F 不高于最后一条根的号（内存 F 之上至少留一条根）：那边的 in_memory_floor < roots.len()。那边的 F 是根序列的下标，这里是 checkpoint 号，
//   根按号严格递增时「内存 F 之上至少一条根」就是最后一条根的号不低于内存 F；
// - 持久 F 之上的根装得进根环（受保护的根都在最后 window 条里）：那边的 roots.len() - persisted_floor <= window。按号递增时受保护的根是一段后缀，
//   这一段不长于 window；
// - 按号严格递增、持久 F 是最后一条根带的 F：那边没有（F 是下标、根不带 F），这里换算要用。
// 「内存 F 不高于最后一条根的号」与「持久 F 之上的根装得进根环」这两条是成形的条件；见证在
// witness_the_root_ring_shape_keeps_a_root_above_the_floor_and_fits_the_window。
pub open spec fn root_ring_is_shaped(ring: RootRingModel) -> bool {
    &&& ring.roots.len() >= 1
    &&& ring.window >= 1
    &&& roots_ascend_by_checkpoint_txg(ring.roots)
    &&& ring.persisted_floor == ring.roots.last().rollback_floor
    &&& ring.persisted_floor.0 <= ring.in_memory_floor.0
    &&& ring.in_memory_floor.0 <= ring.roots.last().checkpoint_txg.0
    &&& forall|index: int| #[trigger] root_is_protected(ring, index) ==> ring.roots.len() - ring.window <= index
}

// root_ring_is_shaped 补的两条各拒一个不成形的根环：两条根（checkpoint 号 1、2，都带 F = 1）、持久 F 与内存 F 都是 1、根环容 2 条，成形；
// 只把内存 F 抬到 3（高过最后一条根，那边的内存 F 之上一条根都不剩），或只把根环容量改成 1（持久 F 之上两条根装不进），其余五条都还满足，
// 补的两条各拒其中一个。
pub proof fn witness_the_root_ring_shape_keeps_a_root_above_the_floor_and_fits_the_window() {
    let mapping = CentralMappingModel { entries: Map::<Seq<u8>, MappingValueModel>::empty() };
    let older = RingRootModel {
        checkpoint_txg: CheckpointTxg(1),
        rollback_floor: CheckpointTxg(1),
        mapping,
        publisher: RootPublisherModel::UserPublish,
    };
    let newer = RingRootModel {
        checkpoint_txg: CheckpointTxg(2),
        rollback_floor: CheckpointTxg(1),
        mapping,
        publisher: RootPublisherModel::UserPublish,
    };
    let roots = seq![older, newer];
    assert(roots_ascend_by_checkpoint_txg(roots)) by {
        assert forall|earlier: int, later: int|
            #![trigger roots[earlier], roots[later]]
            0 <= earlier < later < roots.len() implies roots[earlier].checkpoint_txg.0 < roots[later].checkpoint_txg.0 by {
            assert(earlier == 0 && later == 1);
        }
    }
    assert(roots.last() == newer);
    let shaped = RootRingModel {
        roots,
        in_memory_floor: CheckpointTxg(1),
        persisted_floor: CheckpointTxg(1),
        head: mapping,
        window: 2,
    };
    assert(root_ring_is_shaped(shaped));
    let in_memory_floor_above_every_root = RootRingModel {
        roots,
        in_memory_floor: CheckpointTxg(3),
        persisted_floor: CheckpointTxg(1),
        head: mapping,
        window: 2,
    };
    assert(!root_ring_is_shaped(in_memory_floor_above_every_root));
    let window_shorter_than_the_protected_roots = RootRingModel {
        roots,
        in_memory_floor: CheckpointTxg(1),
        persisted_floor: CheckpointTxg(1),
        head: mapping,
        window: 1,
    };
    assert(roots[0] == older);
    assert(root_is_protected(window_shorter_than_the_protected_roots, 0));
    assert(!root_ring_is_shaped(window_shorter_than_the_protected_roots));
}

// 持久 F 之上的根：回收与覆写要避开它们罩着的每个槽。
pub open spec fn root_is_protected(ring: RootRingModel, index: int) -> bool {
    0 <= index < ring.roots.len() && ring.persisted_floor.0 <= ring.roots[index].checkpoint_txg.0
}

// 回退候选：内存 F 之上、用户那一侧发的根。
pub open spec fn root_is_a_rollback_candidate(ring: RootRingModel, index: int) -> bool {
    &&& 0 <= index < ring.roots.len()
    &&& ring.in_memory_floor.0 <= ring.roots[index].checkpoint_txg.0
    &&& ring.roots[index].publisher matches RootPublisherModel::UserPublish
}

// 对到 gc_ring_interval_proofs 的 slot_is_unreferenced（读者那一项在会话那一侧，不在这个模型里）：现行映射与持久 F 之上的每个根都没罩住这个槽。
pub open spec fn slot_is_unreferenced_by_the_ring(ring: RootRingModel, device: DeviceIdentity, slot: SlotNumber) -> bool {
    &&& central_mapping_avoids_the_slot(ring.head, device, slot)
    &&& forall|index: int| root_is_protected(ring, index)
        ==> central_mapping_avoids_the_slot((#[trigger] ring.roots[index]).mapping, device, slot)
}

// 崩溃之后：内存 F 回到持久 F，现行映射回到最后一条持久的根那一版。对到 gc_ring_interval_proofs 里 RunRingStep::Crash 那一步。
pub open spec fn root_ring_after_a_crash(ring: RootRingModel) -> RootRingModel {
    RootRingModel {
        roots: ring.roots,
        in_memory_floor: ring.persisted_floor,
        persisted_floor: ring.persisted_floor,
        head: ring.roots.last().mapping,
        window: ring.window,
    }
}

} // verus!

#[cfg(test)]
mod tests {
    //! 格式常量规格的跨文件一致性：core 每份源文件里每一条 `assume_specification[ singlefs_format::<名> ] … returns <值>;`，
    //! 与 singlefs-format 源码里那一行 `pub const <名>: <类型> = <整数字面量>;` 逐条比。真值取自 format 的源码而不是编进来的常量，
    //! 所以 format 那一行改成算式时这里拒（不猜它的值），而不是照算出来的值放过。
    use std::collections::BTreeMap;
    use std::fmt;
    use std::path::Path;

    /// 测试跑在哪棵树里就读哪棵树的源码：core 的清单目录，format 是它的兄弟目录。
    const CORE_MANIFEST_DIRECTORY: &str = env!("CARGO_MANIFEST_DIR");
    const CORE_SOURCE_PATH_FROM_REPOSITORY_ROOT: &str = "crates/singlefs-core/src";
    const FORMAT_SOURCE_PATH_FROM_REPOSITORY_ROOT: &str = "crates/singlefs-format/src";
    /// 规格声明的关键字；按整词找，注释与字符串、字符字面量里的不算。
    const SPECIFICATION_KEYWORD: &str = "assume_specification";
    /// 格式常量规格方括号里那一项写全路径，以它起头。
    const FORMAT_CRATE_PATH_PREFIX: &str = "singlefs_format::";
    /// 认作整数的类型名：规格交回值的类型后缀、`->` 写的交回类型与 format 里常量的类型都只认这几个。
    const INTEGER_TYPE_NAMES: [&str; 12] = [
        "u8", "u16", "u32", "u64", "u128", "usize", "i8", "i16", "i32", "i64", "i128", "isize",
    ];

    #[derive(Clone)]
    struct SourceLocation {
        path_from_repository_root: String,
        line_number: usize,
    }

    impl fmt::Display for SourceLocation {
        fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(
                formatter,
                "{}:{}",
                self.path_from_repository_root, self.line_number
            )
        }
    }

    struct RustSource {
        path_from_repository_root: String,
        text: String,
    }

    /// 一条声明：关键字所在的位置，与从关键字起到同一层的 `;` 为止的那一截（注释与字面量已换成空格，连续空白并成一个空格）。
    struct DeclarationText {
        location: SourceLocation,
        text: String,
    }

    /// singlefs-format 源码里的一行 `pub const <名>: <类型> = <右边>;`。
    struct FormatConstantDefinition {
        location: SourceLocation,
        type_name: String,
        right_hand_side: String,
    }

    /// core 里一条点名 `singlefs_format::<名>` 的规格。`declared_type` 是 `->` 写的交回类型。
    struct FormatConstantSpecification {
        location: SourceLocation,
        name: String,
        declared_type: String,
        returned_literal: String,
    }

    /// 一条 `assume_specification` 读出来是什么。
    enum SpecificationReading {
        FormatConstant(FormatConstantSpecification),
        /// 方括号里不是 `singlefs_format::` 起头的路径（别的外部函数的规格），不归这条测试比，只看它是不是绕开了全路径的格式常量。
        OtherPath {
            location: SourceLocation,
            path: String,
        },
        Unreadable {
            location: SourceLocation,
            reason: &'static str,
            text: String,
        },
    }

    /// 判红的每一种。
    enum SpecificationInconsistency {
        SpecifiedMoreThanOnce {
            name: String,
            locations: Vec<SourceLocation>,
        },
        ValueDiffersFromFormat {
            specification: SourceLocation,
            name: String,
            specified_value: u128,
            definition: SourceLocation,
            defined_value: u128,
        },
        ConstantMissingInFormat {
            specification: SourceLocation,
            name: String,
        },
        ConstantDefinedMoreThanOnceInFormat {
            specification: SourceLocation,
            name: String,
            definitions: Vec<SourceLocation>,
        },
        FormatDefinitionIsNotAnIntegerLiteral {
            specification: SourceLocation,
            name: String,
            definition: SourceLocation,
            type_name: String,
            right_hand_side: String,
        },
        TypeDiffersFromFormat {
            specification: SourceLocation,
            name: String,
            which_type: &'static str,
            specified_type: String,
            format_type: String,
        },
        ReturnedValueIsNotAnIntegerLiteral {
            specification: SourceLocation,
            name: String,
            returned_literal: String,
        },
        SpecificationUnreadable {
            location: SourceLocation,
            reason: &'static str,
            text: String,
        },
        FormatConstantNamedWithoutFullPath {
            location: SourceLocation,
            path: String,
            constant_name: String,
        },
    }

    impl fmt::Display for SpecificationInconsistency {
        fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
            match self {
                Self::SpecifiedMoreThanOnce { name, locations } => {
                    let joined: Vec<String> = locations.iter().map(ToString::to_string).collect();
                    write!(
                        formatter,
                        "同一个常量两处有规格：singlefs_format::{name} 的 assume_specification 写了 {} 处（一个常量只许一处，值同也不行）：{}",
                        locations.len(),
                        joined.join("、")
                    )
                }
                Self::ValueDiffersFromFormat { specification, name, specified_value, definition, defined_value } => write!(
                    formatter,
                    "规格的值与 format 不同：{specification} 的 singlefs_format::{name} 交回 {specified_value}，{definition} 定义的是 {defined_value}"
                ),
                Self::ConstantMissingInFormat { specification, name } => write!(
                    formatter,
                    "规格点名的常量在 format 里找不到：{specification} 点名 singlefs_format::{name}，{FORMAT_SOURCE_PATH_FROM_REPOSITORY_ROOT} 里没有 pub const {name}"
                ),
                Self::ConstantDefinedMoreThanOnceInFormat { specification, name, definitions } => {
                    let joined: Vec<String> = definitions.iter().map(ToString::to_string).collect();
                    write!(
                        formatter,
                        "规格点名的常量在 format 里不止一处：{specification} 点名 singlefs_format::{name}，format 源码里 pub const {name} 有 {} 处（{}），不猜是哪一处",
                        definitions.len(),
                        joined.join("、")
                    )
                }
                Self::FormatDefinitionIsNotAnIntegerLiteral { specification, name, definition, type_name, right_hand_side } => write!(
                    formatter,
                    "规格点名的常量在 format 里不是整数字面量：{specification} 点名 singlefs_format::{name}，{definition} 写的是 `{type_name} = {right_hand_side}`（format 改成算式或非整数类型时拒，不猜它的值）"
                ),
                Self::TypeDiffersFromFormat { specification, name, which_type, specified_type, format_type } => write!(
                    formatter,
                    "规格的类型与 format 不同：{specification} 的 singlefs_format::{name}，{which_type}是 {specified_type}，format 里的类型是 {format_type}"
                ),
                Self::ReturnedValueIsNotAnIntegerLiteral { specification, name, returned_literal } => write!(
                    formatter,
                    "规格交回的不是整数字面量：{specification} 的 singlefs_format::{name} returns `{returned_literal}`"
                ),
                Self::SpecificationUnreadable { location, reason, text } => write!(
                    formatter,
                    "assume_specification 的写法认不出：{location}（{reason}）：`{text}`"
                ),
                Self::FormatConstantNamedWithoutFullPath { location, path, constant_name } => write!(
                    formatter,
                    "格式常量的规格没写全路径：{location} 点名 `{path}`，末段是 format 的常量名；写成 singlefs_format::{constant_name}，这条测试才对得上它"
                ),
            }
        }
    }

    /// 一个目录下的全部 `.rs`（逐层往下），按路径排好；路径从仓根写起，报红时给 文件:行。
    fn read_rust_sources_under(
        directory: &Path,
        path_from_repository_root: &str,
    ) -> Vec<RustSource> {
        let mut entry_paths: Vec<std::path::PathBuf> = std::fs::read_dir(directory)
            .unwrap_or_else(|error| panic!("读不了源码目录 {}：{error}", directory.display()))
            .map(|entry| {
                entry
                    .unwrap_or_else(|error| {
                        panic!("读不了源码目录 {} 的一项：{error}", directory.display())
                    })
                    .path()
            })
            .collect();
        entry_paths.sort();
        let mut sources = Vec::new();
        for entry_path in entry_paths {
            let file_name = entry_path
                .file_name()
                .and_then(|name| name.to_str())
                .expect("源码目录下的文件名是 UTF-8");
            let entry_path_from_repository_root =
                format!("{path_from_repository_root}/{file_name}");
            if entry_path.is_dir() {
                sources.extend(read_rust_sources_under(
                    &entry_path,
                    &entry_path_from_repository_root,
                ));
            } else if entry_path
                .extension()
                .is_some_and(|extension| extension == "rs")
            {
                let text = std::fs::read_to_string(&entry_path).unwrap_or_else(|error| {
                    panic!("读不了 {entry_path_from_repository_root}：{error}")
                });
                sources.push(RustSource {
                    path_from_repository_root: entry_path_from_repository_root,
                    text,
                });
            }
        }
        sources
    }

    fn is_identifier_character(character: char) -> bool {
        character.is_alphanumeric() || character == '_'
    }

    /// `start` 处起头的若是注释、字符串或字符字面量，交回它结束之后的下标；不是（代码、生命周期、标签）交回 None。
    fn end_of_comment_or_literal_starting_at(characters: &[char], start: usize) -> Option<usize> {
        let character_at = |offset: usize| characters.get(start + offset).copied();
        let character_before = |offset: usize| {
            start
                .checked_sub(offset)
                .and_then(|index| characters.get(index))
                .copied()
        };
        let current = character_at(0)?;
        if current == '/' && character_at(1) == Some('/') {
            let line_end = characters[start..]
                .iter()
                .position(|character| *character == '\n')
                .map_or(characters.len(), |offset| start + offset);
            return Some(line_end);
        }
        if current == '/' && character_at(1) == Some('*') {
            let mut depth = 0_usize;
            let mut position = start;
            while position < characters.len() {
                if characters[position] == '/' && characters.get(position + 1) == Some(&'*') {
                    depth += 1;
                    position += 2;
                } else if characters[position] == '*' && characters.get(position + 1) == Some(&'/')
                {
                    depth -= 1;
                    position += 2;
                    if depth == 0 {
                        return Some(position);
                    }
                } else {
                    position += 1;
                }
            }
            return Some(characters.len());
        }
        if current == '"' {
            let mut position = start + 1;
            while position < characters.len() {
                match characters[position] {
                    '\\' => position += 2,
                    '"' => return Some(position + 1),
                    _ => position += 1,
                }
            }
            return Some(characters.len());
        }
        if current == 'r' {
            // 原始字符串 r"…"、r#"…"#、br#"…"#：r 前面不接标识符（b 前缀除外）、后面是若干个 # 再接 "。
            let starts_a_token = match character_before(1) {
                None => true,
                Some('b') => !character_before(2).is_some_and(is_identifier_character),
                Some(previous) => !is_identifier_character(previous),
            };
            if !starts_a_token {
                return None;
            }
            let hash_count = characters[start + 1..]
                .iter()
                .take_while(|character| **character == '#')
                .count();
            if characters.get(start + 1 + hash_count) != Some(&'"') {
                return None;
            }
            let mut position = start + 2 + hash_count;
            while position < characters.len() {
                let closes = characters[position] == '"'
                    && (1..=hash_count)
                        .all(|offset| characters.get(position + offset) == Some(&'#'));
                if closes {
                    return Some(position + 1 + hash_count);
                }
                position += 1;
            }
            return Some(characters.len());
        }
        if current == '\'' {
            if character_at(1) == Some('\\') {
                // 转义的字符字面量：跳过反斜杠后面那一个字，再找收尾的单引号（'\''、'\u{…}' 都对）。
                let closing = characters
                    .get(start + 3..)
                    .unwrap_or(&[])
                    .iter()
                    .position(|character| *character == '\'')
                    .map_or(characters.len(), |offset| start + 3 + offset + 1);
                return Some(closing);
            }
            if character_at(2) == Some('\'') {
                return Some(start + 3);
            }
            return None;
        }
        None
    }

    /// 注释、字符串与字符字面量换成空格，换行照留：行号不变，注释与字面量里的字也不被当成代码认。
    fn blank_comments_and_literals(source: &str) -> String {
        let characters: Vec<char> = source.chars().collect();
        let mut blanked = String::with_capacity(source.len());
        let mut position = 0;
        while position < characters.len() {
            if let Some(end) = end_of_comment_or_literal_starting_at(&characters, position) {
                for character in &characters[position..end] {
                    blanked.push(if *character == '\n' { '\n' } else { ' ' });
                }
                position = end;
            } else {
                blanked.push(characters[position]);
                position += 1;
            }
        }
        blanked
    }

    /// `text` 里 `open_position` 处的开括号配对的闭括号的下标（只数同一种括号）。
    fn matching_close_position(
        text: &str,
        open_position: usize,
        open: char,
        close: char,
    ) -> Option<usize> {
        let mut depth = 0_usize;
        for (offset, character) in text[open_position..].char_indices() {
            if character == open {
                depth += 1;
            } else if character == close {
                depth -= 1;
                if depth == 0 {
                    return Some(open_position + offset);
                }
            }
        }
        None
    }

    /// 已去掉注释与字面量的源码里每一处整词 `word`：从它起到同一层括号里第一个 `;` 为止（包括 `;`）。
    fn declarations_starting_with_word(
        blanked_source: &str,
        word: &str,
        path_from_repository_root: &str,
    ) -> Vec<DeclarationText> {
        let mut declarations = Vec::new();
        for (word_position, _matched) in blanked_source.match_indices(word) {
            let is_whole_word_start = !blanked_source[..word_position]
                .chars()
                .next_back()
                .is_some_and(is_identifier_character);
            let is_whole_word_end = !blanked_source[word_position + word.len()..]
                .chars()
                .next()
                .is_some_and(is_identifier_character);
            if !(is_whole_word_start && is_whole_word_end) {
                continue;
            }
            let mut bracket_depth = 0_i64;
            let mut declaration_end = blanked_source.len();
            for (offset, character) in blanked_source[word_position..].char_indices() {
                match character {
                    '(' | '[' | '{' => bracket_depth += 1,
                    ')' | ']' | '}' => bracket_depth -= 1,
                    ';' if bracket_depth == 0 => {
                        declaration_end = word_position + offset + 1;
                        break;
                    }
                    _ => {}
                }
            }
            let text = blanked_source[word_position..declaration_end]
                .split_whitespace()
                .collect::<Vec<_>>()
                .join(" ");
            let line_number = 1 + blanked_source[..word_position].matches('\n').count();
            declarations.push(DeclarationText {
                location: SourceLocation {
                    path_from_repository_root: path_from_repository_root.to_owned(),
                    line_number,
                },
                text,
            });
        }
        declarations
    }

    fn without_whitespace(text: &str) -> String {
        text.chars()
            .filter(|character| !character.is_whitespace())
            .collect()
    }

    /// 整数字面量的值与类型后缀（没写后缀是 None）；不是整数字面量（算式、浮点、别的后缀）交回 None。
    fn parse_integer_literal(text: &str) -> Option<(u128, Option<&str>)> {
        let (radix, body) = if let Some(hexadecimal) = text.strip_prefix("0x") {
            (16, hexadecimal)
        } else if let Some(octal) = text.strip_prefix("0o") {
            (8, octal)
        } else if let Some(binary) = text.strip_prefix("0b") {
            (2, binary)
        } else {
            (10, text)
        };
        let digits_end = body
            .find(|character: char| !(character == '_' || character.is_digit(radix)))
            .unwrap_or(body.len());
        let (digits, suffix) = body.split_at(digits_end);
        let digits_without_separators: String = digits
            .chars()
            .filter(|character| *character != '_')
            .collect();
        if digits_without_separators.is_empty() {
            return None;
        }
        let value = u128::from_str_radix(&digits_without_separators, radix).ok()?;
        if suffix.is_empty() {
            return Some((value, None));
        }
        INTEGER_TYPE_NAMES
            .contains(&suffix)
            .then_some((value, Some(suffix)))
    }

    /// singlefs-format 源码里每一个 `pub const`，按名字收（同名几处都留着，比的时候判）。
    fn format_constant_definitions(
        format_sources: &[RustSource],
    ) -> BTreeMap<String, Vec<FormatConstantDefinition>> {
        let mut definitions: BTreeMap<String, Vec<FormatConstantDefinition>> = BTreeMap::new();
        for source in format_sources {
            let blanked = blank_comments_and_literals(&source.text);
            for declaration in declarations_starting_with_word(
                &blanked,
                "const",
                &source.path_from_repository_root,
            ) {
                let line_start = blanked
                    .lines()
                    .nth(declaration.location.line_number - 1)
                    .expect("行号由同一份文本数出来");
                let is_public = line_start
                    .split_whitespace()
                    .take_while(|token| *token != "const")
                    .any(|token| token == "pub");
                let Some(after_keyword) = declaration.text.strip_prefix("const ") else {
                    continue;
                };
                let Some((name, after_name)) = after_keyword.split_once(':') else {
                    continue;
                };
                let Some((type_name, right_hand_side)) = after_name.split_once('=') else {
                    continue;
                };
                if !is_public || name.trim().starts_with("fn ") {
                    continue;
                }
                definitions.entry(name.trim().to_owned()).or_default().push(
                    FormatConstantDefinition {
                        location: declaration.location,
                        type_name: without_whitespace(type_name),
                        right_hand_side: right_hand_side
                            .trim()
                            .trim_end_matches(';')
                            .trim()
                            .to_owned(),
                    },
                );
            }
        }
        definitions
    }

    /// 一条 `assume_specification … ;` 读成什么：`[` 里的路径、`->` 后的交回类型、`returns` 后的字面量。
    fn read_specification(declaration: DeclarationText) -> SpecificationReading {
        let unreadable = |reason: &'static str, unread_declaration: DeclarationText| {
            SpecificationReading::Unreadable {
                location: unread_declaration.location,
                reason,
                text: unread_declaration.text,
            }
        };
        let after_keyword = declaration.text[SPECIFICATION_KEYWORD.len()..].trim_start();
        if !after_keyword.starts_with('[') {
            return unreadable("关键字后面不是方括号", declaration);
        }
        let Some(close_position) = matching_close_position(after_keyword, 0, '[', ']') else {
            return unreadable("方括号没有配对", declaration);
        };
        let path = without_whitespace(&after_keyword[1..close_position]);
        let after_path = after_keyword[close_position + 1..].trim_start();
        let Some(name) = path
            .strip_prefix(FORMAT_CRATE_PATH_PREFIX)
            .map(str::to_owned)
        else {
            return SpecificationReading::OtherPath {
                location: declaration.location,
                path,
            };
        };
        let Some(after_arrow) = after_path.strip_prefix("->").map(str::trim_start) else {
            return unreadable("方括号后面没写 -> 交回类型", declaration);
        };
        let (declared_type, after_type) = if after_arrow.starts_with('(') {
            let Some(binding_close) = matching_close_position(after_arrow, 0, '(', ')') else {
                return unreadable("交回值的括号没有配对", declaration);
            };
            let Some((_binding_name, binding_type)) = after_arrow[1..binding_close].split_once(':')
            else {
                return unreadable("交回值的括号里没写「名字: 类型」", declaration);
            };
            (
                without_whitespace(binding_type),
                &after_arrow[binding_close + 1..],
            )
        } else {
            let type_end = after_arrow
                .find(char::is_whitespace)
                .unwrap_or(after_arrow.len());
            (
                without_whitespace(&after_arrow[..type_end]),
                &after_arrow[type_end..],
            )
        };
        let Some(after_returns) = after_type.trim_start().strip_prefix("returns ") else {
            return unreadable("没写 returns，规格不交回值", declaration);
        };
        let returned_literal = after_returns.trim().trim_end_matches(';').trim().to_owned();
        SpecificationReading::FormatConstant(FormatConstantSpecification {
            location: declaration.location,
            name,
            declared_type,
            returned_literal,
        })
    }

    /// 一条格式常量规格与 format 那一行比：找得到、是整数字面量、类型与值相同。
    fn inconsistencies_against_format(
        specification: &FormatConstantSpecification,
        definitions: &BTreeMap<String, Vec<FormatConstantDefinition>>,
    ) -> Vec<SpecificationInconsistency> {
        let name = specification.name.clone();
        let matching_definitions = definitions
            .get(&specification.name)
            .map_or(&[][..], Vec::as_slice);
        let definition = match matching_definitions {
            [] => {
                return vec![SpecificationInconsistency::ConstantMissingInFormat {
                    specification: specification.location.clone(),
                    name,
                }];
            }
            [only_definition] => only_definition,
            [_, _, ..] => {
                return vec![
                    SpecificationInconsistency::ConstantDefinedMoreThanOnceInFormat {
                        specification: specification.location.clone(),
                        name,
                        definitions: matching_definitions
                            .iter()
                            .map(|definition| definition.location.clone())
                            .collect(),
                    },
                ];
            }
        };
        let defined_literal = parse_integer_literal(&definition.right_hand_side)
            .filter(|_literal| INTEGER_TYPE_NAMES.contains(&definition.type_name.as_str()));
        let Some((defined_value, _defined_suffix)) = defined_literal else {
            return vec![
                SpecificationInconsistency::FormatDefinitionIsNotAnIntegerLiteral {
                    specification: specification.location.clone(),
                    name,
                    definition: definition.location.clone(),
                    type_name: definition.type_name.clone(),
                    right_hand_side: definition.right_hand_side.clone(),
                },
            ];
        };
        let mut inconsistencies = Vec::new();
        if specification.declared_type != definition.type_name {
            inconsistencies.push(SpecificationInconsistency::TypeDiffersFromFormat {
                specification: specification.location.clone(),
                name: name.clone(),
                which_type: "-> 写的交回类型",
                specified_type: specification.declared_type.clone(),
                format_type: definition.type_name.clone(),
            });
        }
        let Some((specified_value, specified_suffix)) =
            parse_integer_literal(&specification.returned_literal)
        else {
            inconsistencies.push(
                SpecificationInconsistency::ReturnedValueIsNotAnIntegerLiteral {
                    specification: specification.location.clone(),
                    name,
                    returned_literal: specification.returned_literal.clone(),
                },
            );
            return inconsistencies;
        };
        if specified_suffix != Some(definition.type_name.as_str()) {
            inconsistencies.push(SpecificationInconsistency::TypeDiffersFromFormat {
                specification: specification.location.clone(),
                name: name.clone(),
                which_type: "returns 字面量的类型后缀",
                specified_type: specified_suffix.unwrap_or("（没写后缀）").to_owned(),
                format_type: definition.type_name.clone(),
            });
        }
        if specified_value != defined_value {
            inconsistencies.push(SpecificationInconsistency::ValueDiffersFromFormat {
                specification: specification.location.clone(),
                name,
                specified_value,
                definition: definition.location.clone(),
                defined_value,
            });
        }
        inconsistencies
    }

    #[test]
    fn every_format_constant_specification_returns_the_value_singlefs_format_defines() {
        let core_manifest_directory = Path::new(CORE_MANIFEST_DIRECTORY);
        let core_sources = read_rust_sources_under(
            &core_manifest_directory.join("src"),
            CORE_SOURCE_PATH_FROM_REPOSITORY_ROOT,
        );
        let format_sources = read_rust_sources_under(
            &core_manifest_directory.join("../singlefs-format/src"),
            FORMAT_SOURCE_PATH_FROM_REPOSITORY_ROOT,
        );
        let definitions = format_constant_definitions(&format_sources);
        assert!(
            !definitions.is_empty(),
            "{FORMAT_SOURCE_PATH_FROM_REPOSITORY_ROOT} 里一个 pub const 都没收到：读 format 源码的那一段写坏了"
        );

        let mut specifications: Vec<FormatConstantSpecification> = Vec::new();
        let mut inconsistencies: Vec<SpecificationInconsistency> = Vec::new();
        for source in &core_sources {
            let blanked = blank_comments_and_literals(&source.text);
            for declaration in declarations_starting_with_word(
                &blanked,
                SPECIFICATION_KEYWORD,
                &source.path_from_repository_root,
            ) {
                match read_specification(declaration) {
                    SpecificationReading::FormatConstant(specification) => {
                        specifications.push(specification)
                    }
                    SpecificationReading::OtherPath { location, path } => {
                        let last_segment =
                            path.rsplit("::").next().unwrap_or(path.as_str()).to_owned();
                        if definitions.contains_key(&last_segment) {
                            inconsistencies.push(
                                SpecificationInconsistency::FormatConstantNamedWithoutFullPath {
                                    location,
                                    path,
                                    constant_name: last_segment,
                                },
                            );
                        }
                    }
                    SpecificationReading::Unreadable {
                        location,
                        reason,
                        text,
                    } => {
                        inconsistencies.push(SpecificationInconsistency::SpecificationUnreadable {
                            location,
                            reason,
                            text,
                        });
                    }
                }
            }
        }
        assert!(
            !specifications.is_empty(),
            "扫了 {CORE_SOURCE_PATH_FROM_REPOSITORY_ROOT} 下 {} 份源文件，一条 assume_specification[ singlefs_format::<名> ] 都没收到：\
             至少 verified_model.rs 里有 SLOT_BYTES 与 MAPPING_KEY_BYTES 两条，扫描写坏了",
            core_sources.len()
        );

        let mut locations_by_name: BTreeMap<&str, Vec<SourceLocation>> = BTreeMap::new();
        for specification in &specifications {
            locations_by_name
                .entry(specification.name.as_str())
                .or_default()
                .push(specification.location.clone());
        }
        for (name, locations) in locations_by_name {
            if locations.len() > 1 {
                inconsistencies.push(SpecificationInconsistency::SpecifiedMoreThanOnce {
                    name: name.to_owned(),
                    locations,
                });
            }
        }
        for specification in &specifications {
            inconsistencies.extend(inconsistencies_against_format(specification, &definitions));
        }

        let reported: Vec<String> = inconsistencies.iter().map(ToString::to_string).collect();
        assert!(
            reported.is_empty(),
            "core 里 {} 条格式常量规格与 singlefs-format 的源码对不上（{} 处）：\n{}",
            specifications.len(),
            reported.len(),
            reported.join("\n")
        );
    }
}
