//! 实审 A4 / A4b / A4d（代码审阅第 20 条；用户 2026-09-27 定「改条款按盘分路计并改实现」，再定「一律按最坏情况计」，
//! 再定分配记录树那一项取 K1「几何上整棵树」）：准入的 ckpt_cost 与一次空发布实际写出的固定点单元数并排量。
//! 分配记录树那一项取 K1 之后每格多扣的块数逐格钉住（实审 A4c 报告 `research/prompts/m2-rev-a4c-implementer-report.md` 第二节量表 K1 那一列）；
//! 实审 A4c 量到每块盘两条叶路径（K0）少扣的那两段随机历史（开放段装不下开新段、回落散到几片叶；换下很久以前落在别处的节点）也在这里重放。
//!
//! 空发布（暖机、抬 F）一个普通分配都没有、不判准入，它写的固定点从式子第八项 checkpoint 保留池里出（`singlefs_core::admission` 的模块文档）；
//! 保留池每块盘扣 ckpt_cost 个 16 KiB 槽，暖机那一半的 c_max 也取同一个数，所以一次空发布在每块盘上新写的槽数不许多于它之前那一版现算的 ckpt_cost。
//! 每格两块同宽的稀疏内存盘：mkfs（同一个进程接着写的分配器装着根环表，根环转过之后照常回收）→ 取号 → 暖机 → 第一个文件版本 → 覆盖写一次
//! （中央映射树两层那一格再顺序写到几百个数据单元），再接着推空发布；每次发布之前按那一版现算 ckpt_cost，发完按 `TransactionOutput::rewritten`
//! 逐棵、分配记录树逐盘数这次写出的单元。每一次发布一行 `name=a4-checkpoint-cost …`、每格一行 `name=a4b-checkpoint-cost-cell …`
//! （`--nocapture` 看得到），量表的原样行就是它们。
//!
//! 盘数只有 2：mkfs 断言两块盘（`make_filesystem::make_filesystem`，D2（RAID 条带策略） 已定项 9），映射条目的位置项也只装两块盘，
//! 三块盘走不到发布路径。三块盘那一格只在分配记录树这一层量（[`three_devices_rewrite_at_most_two_leaf_paths_on_each_device_and_the_shared_root`]）。
mod common;
mod common_admission;

use std::collections::{BTreeMap, BTreeSet};

use common::{file_content, E142_FILESYSTEM_IDENTIFIER, FIXED_WRITE_TIME_SECONDS};
use common_admission::{content_of, PoolUnderTest, FIRST_WRITE_TIME_SECONDS, OVERWRITE_BYTES};
use singlefs_core::address::{CheckpointTxg, DeviceIdentity};
use singlefs_core::admission::{
    checkpoint_cost_of_the_version_to_build_on,
    checkpoint_cost_of_the_version_to_build_on_with_node_capacities, space_budget_of_role,
    MetadataBlocks, SpaceBudgetOfARole,
};
use singlefs_core::allocation_record_tree::{
    nodes_whose_contents_changed, AllocationRecordTreeGeometry, AllocationRecordTreeNode,
    AllocationRecordTreeVersion,
};
use singlefs_core::allocator::{
    AllocationRecordTreeOfTheVersionWithoutFile, DeviceFreeMap, PoolAllocator, UnitAreaStart,
    UnitFootprint,
};
use singlefs_core::block_device::PhysicalBlockSizeInBytes;
use singlefs_core::code_two_tree::CodeTwoTreeNodeCapacity;
use singlefs_core::journal::back_chain_of;
use singlefs_core::make_filesystem::{
    allocator_after_make_filesystem, make_filesystem, MakeFilesystemParameters,
};
use singlefs_core::mount::{ParametersAndDeviceTableOfTheMount, ShadowLedger};
use singlefs_core::mounted_session::{MountedSession, UserChangeRefused};
use singlefs_core::root_ring::RootRingSlotsPerRegion;
use singlefs_core::system_configuration::SystemImmutableSizes;
use singlefs_core::transaction::{
    acquire_instance, publish_first_file, publish_overwrite, publish_sequential_write,
    publish_version, warm_up, CodeTwoTreeNodeCapacities, FirstFile, InstanceTablePlan,
    MultiLevelCodeTwoTree, PoolVersion, PoolWriter, PublishPlan, TransactionOutput,
    TransactionUnit,
};
use singlefs_core::unit::data_unit_payload_capacity;
use singlefs_format::{JOURNAL_RING_DEFAULT_BYTES, SLOT_BYTES};
use singlefs_harness::history::HistoryDeviceWidth;
use singlefs_harness::memory_pool::SparseBlockDevice;

const GIBIBYTE: u64 = 1 << 30;

/// 1 GiB 那一格的 journal 环：mkfs 要求环不超过设备容量的四分之一，默认 768 MiB 的环放不进 1 GiB 的盘。
/// 单元区起点随环长走（环末尾的下一个槽，实审 A3b）：159 MiB 的环末尾的下一个槽是 1024 + 159 × 64 = 11200 = 50176 − 3 × 12992，
/// 12992 是分配记录树一片叶的 812 槽与聚簇段 64 槽的最小公倍数，所以起点在叶里、在段里的位置与默认环的起点 50176 相同：
/// 落在叶 13（[10556, 11368)）里，离叶末槽 11367 还有 168 槽，同默认环那几格离叶 61 末槽 50343 的 168 槽。
/// 改之前取 128 MiB（起点是编译期的 50176）；128 MiB 的环起点落到 9216，离叶 11 末槽 528 槽，60 次空发布走不到跨叶。
const JOURNAL_RING_BYTES_OF_THE_ONE_GIBIBYTE_DEVICES: u64 = 159 << 20;

/// 每格覆盖写之后推几次空发布：根环 24 槽转过两圈多，回收照常走；bump 游标从单元区起点（默认环那几格是槽 50176，落在叶 61 里；
/// 1 GiB 那一格是槽 11200，落在叶 13 里）往后走，168 槽之后走出那片叶（叶 61 在槽 50343 结束、叶 13 在槽 11367 结束），
/// 走过它的那两次空发布改的记录跨两片叶。
const EMPTY_PUBLISHES_PER_CELL: usize = 60;

/// 1 TiB 那一格推几次空发布：每块盘三张 6700 万位的位图，每次发布整个分配器拷几份，推满 60 次太慢；30 次也走过叶 61 的末槽。
const EMPTY_PUBLISHES_OF_THE_ONE_TEBIBYTE_CELL: usize = 30;

/// 中央映射树两层那一格顺序写到几个数据单元：产品容量下映射树一片叶装 294 条，进映射的单元（每个数据单元、extent 树与 inode 树的节点、
/// 分配记录树与记账树的节点）多过它才长到两层；300 个数据单元加上别的十几条，一片叶装不下、两片装得下。
const DATA_UNITS_OF_THE_TWO_LEVEL_CENTRAL_MAPPING_CELL: usize = 300;

/// 覆盖写之后、推空发布之前文件怎么长。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum FileGrowthBeforeTheEmptyPublishes {
    StaysAtOneDataUnit,
    SequentialWriteToDataUnits(usize),
}

/// 一格几何：两块同宽的稀疏盘、journal 环、记账树与中央映射树的节点容量、覆盖写之后文件怎么长、推几次空发布，
/// 与这一格全部空发布每块盘多扣的块数（ckpt_cost − 实写的固定点槽数）的最小与最大（实审 A4c 量表 K1 那一列）。
struct PoolGeometryCell {
    name: &'static str,
    device_bytes: u64,
    journal_ring_bytes: u64,
    capacities: CodeTwoTreeNodeCapacities,
    file_growth: FileGrowthBeforeTheEmptyPublishes,
    empty_publishes: usize,
    over_reserved_slots_of_the_empty_publishes: OverReservedSlots,
}

/// 一格里全部空发布每块盘多扣的块数（ckpt_cost − 那一次每块盘实写的固定点槽数）的最小与最大。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct OverReservedSlots {
    fewest: i128,
    most: i128,
}

const fn over_reserved(fewest: i128, most: i128) -> OverReservedSlots {
    OverReservedSlots { fewest, most }
}

fn capped(
    accounting: (usize, usize),
    central_mapping: (usize, usize),
) -> CodeTwoTreeNodeCapacities {
    CodeTwoTreeNodeCapacities::CappedForTests {
        accounting: CodeTwoTreeNodeCapacity {
            leaf_entries: accounting.0,
            internal_entries: accounting.1,
        },
        central_mapping: CodeTwoTreeNodeCapacity {
            leaf_entries: central_mapping.0,
            internal_entries: central_mapping.1,
        },
    }
}

/// 产品路径的节点容量（按节点格式算），盘宽四档：分配记录树高 2（1 GiB）、3（4 GiB、64 GiB）、4（1 TiB），文件一个数据单元、中央映射树 1 层；
/// 再加一格 4 GiB 两盘、文件顺序写到 300 个数据单元，中央映射树 2 层。
fn cells_with_the_node_format_capacities() -> Vec<PoolGeometryCell> {
    let one_data_unit = |name,
                         device_bytes,
                         journal_ring_bytes,
                         empty_publishes,
                         over_reserved_slots_of_the_empty_publishes| {
        PoolGeometryCell {
            name,
            device_bytes,
            journal_ring_bytes,
            capacities: CodeTwoTreeNodeCapacities::FromTheNodeFormat,
            file_growth: FileGrowthBeforeTheEmptyPublishes::StaysAtOneDataUnit,
            empty_publishes,
            over_reserved_slots_of_the_empty_publishes,
        }
    };
    vec![
        one_data_unit(
            "two-1GiB",
            GIBIBYTE,
            JOURNAL_RING_BYTES_OF_THE_ONE_GIBIBYTE_DEVICES,
            EMPTY_PUBLISHES_PER_CELL,
            over_reserved(132, 134),
        ),
        one_data_unit(
            "two-4GiB",
            4 * GIBIBYTE,
            JOURNAL_RING_DEFAULT_BYTES,
            EMPTY_PUBLISHES_PER_CELL,
            over_reserved(527, 529),
        ),
        one_data_unit(
            "two-64GiB",
            64 * GIBIBYTE,
            JOURNAL_RING_DEFAULT_BYTES,
            EMPTY_PUBLISHES_PER_CELL,
            over_reserved(10337, 10339),
        ),
        one_data_unit(
            "two-1TiB",
            1024 * GIBIBYTE,
            JOURNAL_RING_DEFAULT_BYTES,
            EMPTY_PUBLISHES_OF_THE_ONE_TEBIBYTE_CELL,
            over_reserved(167_298, 167_300),
        ),
        PoolGeometryCell {
            name: "two-4GiB-file-300-units-mapping-two-levels",
            device_bytes: 4 * GIBIBYTE,
            journal_ring_bytes: JOURNAL_RING_DEFAULT_BYTES,
            capacities: CodeTwoTreeNodeCapacities::FromTheNodeFormat,
            file_growth: FileGrowthBeforeTheEmptyPublishes::SequentialWriteToDataUnits(
                DATA_UNITS_OF_THE_TWO_LEVEL_CENTRAL_MAPPING_CELL,
            ),
            empty_publishes: EMPTY_PUBLISHES_PER_CELL,
            over_reserved_slots_of_the_empty_publishes: over_reserved(528, 530),
        },
    ]
}

/// 只供测试的开关压小中央映射树（与记账树）的节点容量（`CodeTwoTreeNodeCapacities::CappedForTests`）：4 GiB 两盘上中央映射树长到 2 层、3 层与更高，
/// 记账树长到 4 个节点，空发布里映射树照样切分、长高。
fn cells_with_capped_code_two_trees() -> Vec<PoolGeometryCell> {
    let capped_cell =
        |name, capacities, over_reserved_slots_of_the_empty_publishes| PoolGeometryCell {
            name,
            device_bytes: 4 * GIBIBYTE,
            journal_ring_bytes: JOURNAL_RING_DEFAULT_BYTES,
            capacities,
            file_growth: FileGrowthBeforeTheEmptyPublishes::StaysAtOneDataUnit,
            empty_publishes: EMPTY_PUBLISHES_PER_CELL,
            over_reserved_slots_of_the_empty_publishes,
        };
    vec![
        capped_cell(
            "two-4GiB-mapping-4-8",
            capped((477, 150), (4, 8)),
            over_reserved(877, 882),
        ),
        capped_cell(
            "two-4GiB-mapping-3-3",
            capped((477, 150), (3, 3)),
            over_reserved(1057, 1067),
        ),
        capped_cell(
            "two-4GiB-mapping-2-2-accounting-8-3",
            capped((8, 3), (2, 2)),
            over_reserved(144_961, 145_503),
        ),
    ]
}

fn parameters_of(cell: &PoolGeometryCell) -> MakeFilesystemParameters {
    MakeFilesystemParameters {
        filesystem_identifier: E142_FILESYSTEM_IDENTIFIER,
        region_devices: [DeviceIdentity(0), DeviceIdentity(1), DeviceIdentity(0)],
        geometry: SystemImmutableSizes {
            physical_block_size: 512,
            minimum_input_output_bytes: 512,
            fixed_structure_slot_spacing: 4096,
            journal_ring_bytes: cell.journal_ring_bytes,
            root_ring_slots_per_region: RootRingSlotsPerRegion::AT_MAKE_FILESYSTEM,
        },
    }
}

/// 一次发布是哪一种：覆盖写有普通分配、它的固定点在需求里（准入判）；空发布只写固定点、从保留池出（准入不判）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum MeasuredPublishKind {
    Overwrite,
    Empty,
}

/// 一次发布写出的固定点单元，逐棵数、分配记录树逐盘数；另记这次发布之前那一版现算的 ckpt_cost、那一版各棵树的高、这次之后中央映射树的高。
struct MeasuredPublish {
    kind: MeasuredPublishKind,
    label: String,
    checkpoint_cost_before: MetadataBlocks,
    checkpoint_cost_by_tree_height_before: u64,
    checkpoint_cost_one_leaf_path_per_device_before: u64,
    allocation_record_tree_height_before: u64,
    central_mapping_tree_height_before: u64,
    central_mapping_tree_height_after: u64,
    accounting_tree_nodes_before: u64,
    allocation_record_tree_nodes_below_the_root_on_each_device: BTreeMap<DeviceIdentity, u64>,
    allocation_record_tree_roots: u64,
    central_mapping_tree_nodes: u64,
    accounting_tree_nodes: u64,
    tree_table_units: u64,
    ordinary_allocation_slots: u64,
    fixed_point_slots_on_each_device: u64,
}

impl MeasuredPublish {
    /// 这次写出的固定点槽数比之前现算的 ckpt_cost 多几块（正 = 保留池少扣；负 = 多扣）。
    fn difference_in_slots(&self) -> i128 {
        i128::from(self.fixed_point_slots_on_each_device)
            - i128::from(self.checkpoint_cost_before.0)
    }

    /// 有一块盘上分配记录树改的不止一条从叶到根之下那一层的路径（高 − 1 个节点）：这次改的记录在那块盘上跨了两片叶。
    fn rewrites_more_than_one_leaf_path_on_some_device(&self) -> bool {
        let one_leaf_path = self.allocation_record_tree_height_before - 1;
        self.allocation_record_tree_nodes_below_the_root_on_each_device
            .values()
            .any(|nodes| *nodes > one_leaf_path)
    }

    fn line(&self, cell: &PoolGeometryCell) -> String {
        let per_device: Vec<String> = self
            .allocation_record_tree_nodes_below_the_root_on_each_device
            .iter()
            .map(|(device, nodes)| format!("d{}:{}", device.0, nodes))
            .collect();
        format!(
            "name=a4-checkpoint-cost cell={} device_bytes={} publish={} \
             height_allocation_record_tree={} height_central_mapping_tree={} height_central_mapping_tree_after={} \
             accounting_nodes_before={} \
             checkpoint_cost_by_tree_height={} checkpoint_cost_one_leaf_path_per_device={} checkpoint_cost={} \
             allocation_record_tree_rewritten={}+root{} central_mapping_rewritten={} accounting_rewritten={} \
             tree_table_rewritten={} ordinary_slots={} fixed_point_slots_per_device={} \
             difference={} difference_one_leaf_path_per_device={} difference_by_tree_height={}",
            cell.name,
            cell.device_bytes,
            self.label,
            self.allocation_record_tree_height_before,
            self.central_mapping_tree_height_before,
            self.central_mapping_tree_height_after,
            self.accounting_tree_nodes_before,
            self.checkpoint_cost_by_tree_height_before,
            self.checkpoint_cost_one_leaf_path_per_device_before,
            self.checkpoint_cost_before.0,
            per_device.join(","),
            self.allocation_record_tree_roots,
            self.central_mapping_tree_nodes,
            self.accounting_tree_nodes,
            self.tree_table_units,
            self.ordinary_allocation_slots,
            self.fixed_point_slots_on_each_device,
            self.difference_in_slots(),
            i128::from(self.fixed_point_slots_on_each_device)
                - i128::from(self.checkpoint_cost_one_leaf_path_per_device_before),
            i128::from(self.fixed_point_slots_on_each_device)
                - i128::from(self.checkpoint_cost_by_tree_height_before),
        )
    }
}

/// D28（挂载期承诺量） 已定项 4 用户 2026-09-27 两次定案之前的字面，量表并排报它：Σ（分配记录树、中央映射树当前的高）+ 记账树每发布的节点数 + 1（树表）。
fn checkpoint_cost_by_tree_height(version: &TransactionOutput) -> u64 {
    version
        .position_addressed_tree_heights_read_from_the_root_node_headers()
        .allocation_record_tree
        + version.height_read_from_the_root_node_header(MultiLevelCodeTwoTree::CentralMapping)
        + u64::try_from(version.accounting_tree.node_count()).expect("节点数装得进 u64")
        + 1
}

/// 实审 A4 那一版（按盘分路、每块盘一条叶路径、中央映射树按树高），量表并排报它：盘数 × (分配记录树高 − 1) + 1 + 中央映射树高 + 记账树节点数 + 1。
fn checkpoint_cost_one_leaf_path_per_device(
    version: &TransactionOutput,
    allocator: &PoolAllocator,
) -> u64 {
    let devices = u64::try_from(allocator.devices.len()).expect("盘数装得进 u64");
    devices
        * (version
            .position_addressed_tree_heights_read_from_the_root_node_headers()
            .allocation_record_tree
            - 1)
        + 1
        + version.height_read_from_the_root_node_header(MultiLevelCodeTwoTree::CentralMapping)
        + u64::try_from(version.accounting_tree.node_count()).expect("节点数装得进 u64")
        + 1
}

/// 这一格写入口装的节点容量下现算的 ckpt_cost；产品容量那几格同时核产品路径那一个函数给同一个数。
fn checkpoint_cost_under_the_capacities_of_the_cell(
    previous: &TransactionOutput,
    allocator_before: &PoolAllocator,
    capacities: CodeTwoTreeNodeCapacities,
) -> MetadataBlocks {
    let under_the_capacities = checkpoint_cost_of_the_version_to_build_on_with_node_capacities(
        Some(previous),
        allocator_before,
        capacities,
    );
    match capacities {
        CodeTwoTreeNodeCapacities::FromTheNodeFormat => assert_eq!(
            checkpoint_cost_of_the_version_to_build_on(Some(previous), allocator_before),
            under_the_capacities,
            "产品路径（发布与可写挂载的准入）调的那一个与按节点格式容量算的是同一个数"
        ),
        CodeTwoTreeNodeCapacities::CappedForTests { .. } => {}
    }
    under_the_capacities
}

fn measure(
    kind: MeasuredPublishKind,
    label: String,
    previous: &TransactionOutput,
    allocator_before: &PoolAllocator,
    published: &TransactionOutput,
    capacities: CodeTwoTreeNodeCapacities,
) -> MeasuredPublish {
    let mut measured =
        MeasuredPublish {
            kind,
            label,
            checkpoint_cost_before: checkpoint_cost_under_the_capacities_of_the_cell(
                previous,
                allocator_before,
                capacities,
            ),
            checkpoint_cost_by_tree_height_before: checkpoint_cost_by_tree_height(previous),
            checkpoint_cost_one_leaf_path_per_device_before:
                checkpoint_cost_one_leaf_path_per_device(previous, allocator_before),
            allocation_record_tree_height_before: previous
                .position_addressed_tree_heights_read_from_the_root_node_headers()
                .allocation_record_tree,
            central_mapping_tree_height_before: previous
                .height_read_from_the_root_node_header(MultiLevelCodeTwoTree::CentralMapping),
            central_mapping_tree_height_after: published
                .height_read_from_the_root_node_header(MultiLevelCodeTwoTree::CentralMapping),
            accounting_tree_nodes_before: u64::try_from(previous.accounting_tree.node_count())
                .expect("节点数装得进 u64"),
            allocation_record_tree_nodes_below_the_root_on_each_device: allocator_before
                .devices
                .iter()
                .map(|device_map| (device_map.device, 0))
                .collect(),
            allocation_record_tree_roots: 0,
            central_mapping_tree_nodes: 0,
            accounting_tree_nodes: 0,
            tree_table_units: 0,
            ordinary_allocation_slots: 0,
            fixed_point_slots_on_each_device: 0,
        };
    for role in &published.rewritten {
        match role {
            TransactionUnit::AllocationTreeNodeBelowTheRoot(position) => {
                *measured
                    .allocation_record_tree_nodes_below_the_root_on_each_device
                    .get_mut(&position.device)
                    .expect("分配记录树根之下的节点属于池里的一块盘") += 1;
                measured.fixed_point_slots_on_each_device += role.span_slots();
            }
            TransactionUnit::AllocationTree => {
                measured.allocation_record_tree_roots += 1;
                measured.fixed_point_slots_on_each_device += role.span_slots();
            }
            TransactionUnit::MappingTreeNodeBelowTheRoot(_) | TransactionUnit::MappingTree => {
                measured.central_mapping_tree_nodes += 1;
                measured.fixed_point_slots_on_each_device += role.span_slots();
            }
            TransactionUnit::AccountingTreeNodeBelowTheRoot(_)
            | TransactionUnit::AccountingTree => {
                measured.accounting_tree_nodes += 1;
                measured.fixed_point_slots_on_each_device += role.span_slots();
            }
            TransactionUnit::TreeTable => {
                measured.tree_table_units += 1;
                measured.fixed_point_slots_on_each_device += role.span_slots();
            }
            TransactionUnit::Data(_)
            | TransactionUnit::ExtentLowerNode(_)
            | TransactionUnit::ExtentUpperNodeBelowTheRoot(_)
            | TransactionUnit::ExtentRoot
            | TransactionUnit::InodeLeafContainer(_)
            | TransactionUnit::InodeRoot
            | TransactionUnit::InstanceTable
            | TransactionUnit::InstanceTablePageAfterTheFirst(_) => {
                measured.ordinary_allocation_slots += role.span_slots();
            }
        }
    }
    measured
}

/// 一格：mkfs → 取号 → 暖机 → 第一个文件版本 → 覆盖写一次 → `cell.empty_publishes` 次空发布；交回覆盖写与每次空发布的量，
/// 每一次都打一行原样行。
fn measure_the_cell(cell: &PoolGeometryCell) -> Vec<MeasuredPublish> {
    let parameters = parameters_of(cell);
    let mut devices: Vec<(DeviceIdentity, SparseBlockDevice)> = [0u32, 1]
        .into_iter()
        .map(|device_number| {
            (
                DeviceIdentity(device_number),
                SparseBlockDevice::new(cell.device_bytes, PhysicalBlockSizeInBytes(512)),
            )
        })
        .collect();
    let genesis = make_filesystem(&parameters, &mut devices).expect("mkfs");
    let mut allocator = allocator_after_make_filesystem(&parameters, &devices, &genesis);
    let mut writer = PoolWriter::new(&parameters, devices.as_mut_slice());
    writer.set_code_two_tree_node_capacities(cell.capacities);
    let instance = acquire_instance(&mut writer).expect("取号");
    let warmed_up = warm_up(&mut writer, &genesis.root, instance).expect("暖机");
    let first_file = publish_first_file(
        &mut writer,
        &mut allocator,
        warmed_up.roots.last().expect("暖机两代根"),
        FirstFile {
            content: &file_content(),
            write_time_seconds: FIXED_WRITE_TIME_SECONDS,
        },
        instance,
        &warmed_up.last_record_bytes,
    )
    .expect("第一个文件版本");
    let mut measured = Vec::new();
    let allocator_before_the_overwrite = allocator.clone();
    let overwritten = publish_overwrite(
        &mut writer,
        &mut allocator,
        &first_file,
        FirstFile {
            content: &file_content()[..2500],
            write_time_seconds: FIXED_WRITE_TIME_SECONDS + 60,
        },
        instance,
    )
    .expect("覆盖写");
    measured.push(measure(
        MeasuredPublishKind::Overwrite,
        format!("overwrite@txg{}", overwritten.root.checkpoint_txg.0),
        &first_file,
        &allocator_before_the_overwrite,
        &overwritten,
        cell.capacities,
    ));
    let mut current = match cell.file_growth {
        FileGrowthBeforeTheEmptyPublishes::StaysAtOneDataUnit => overwritten,
        FileGrowthBeforeTheEmptyPublishes::SequentialWriteToDataUnits(data_units) => {
            let content: Vec<u8> = (0..data_units * data_unit_payload_capacity())
                .map(|index| u8::try_from(index % 241).expect("小于 256"))
                .collect();
            let grown = publish_sequential_write(
                &mut writer,
                &mut allocator,
                &overwritten,
                FirstFile {
                    content: &content,
                    write_time_seconds: FIXED_WRITE_TIME_SECONDS + 120,
                },
                instance,
            )
            .expect("顺序写到几百个数据单元");
            assert_eq!(
                grown.data_pointers.len(),
                data_units,
                "文件长到 {data_units} 个数据单元"
            );
            grown
        }
    };
    for empty_publish_index in 0..cell.empty_publishes {
        let allocator_before = allocator.clone();
        let published = publish_version(
            &mut writer,
            &mut allocator,
            PublishPlan {
                txg: CheckpointTxg(current.root.checkpoint_txg.0 + 1),
                counter: current.record.counter + 1,
                transaction: 0,
                highest_transaction_number_before_this_publish: current
                    .highest_transaction_number_in_this_instance,
                instance: current.root.instance,
                back_chain: back_chain_of(&current.record_bytes),
                file: None,
                new_inode_records: &[],
                instance_table: InstanceTablePlan::Carry(current.root.instance_table),
                tree_birth_txg: current.tree_birth_txg(),
                tree_identifier_watermark: current.root.tree_identifier_watermark,
                rollback_floor: current.root.rollback_floor,
            },
            Some(&current),
        )
        .expect("空发布");
        measured.push(measure(
            MeasuredPublishKind::Empty,
            format!(
                "empty#{empty_publish_index}@txg{}",
                published.root.checkpoint_txg.0
            ),
            &current,
            &allocator_before,
            &published,
            cell.capacities,
        ));
        current = published;
    }
    for measured_publish in &measured {
        println!("{}", measured_publish.line(cell));
    }
    let empty_publishes: Vec<&MeasuredPublish> = measured
        .iter()
        .filter(|measured_publish| measured_publish.kind == MeasuredPublishKind::Empty)
        .collect();
    let differences: Vec<i128> = empty_publishes
        .iter()
        .map(|measured_publish| measured_publish.difference_in_slots())
        .collect();
    println!(
        "name=a4b-checkpoint-cost-cell cell={} empty_publishes={} crossings={} taller_central_mapping={} \
         central_mapping_grew={} difference_min={} difference_max={}",
        cell.name,
        empty_publishes.len(),
        empty_publishes
            .iter()
            .filter(|measured_publish| measured_publish.rewrites_more_than_one_leaf_path_on_some_device())
            .count(),
        empty_publishes
            .iter()
            .filter(|measured_publish| measured_publish.central_mapping_tree_height_before > 1)
            .count(),
        empty_publishes
            .iter()
            .filter(|measured_publish| measured_publish.central_mapping_tree_height_after
                > measured_publish.central_mapping_tree_height_before)
            .count(),
        differences.iter().min().expect("每格至少一次空发布"),
        differences.iter().max().expect("每格至少一次空发布"),
    );
    measured
}

/// 一格里全部空发布每块盘多扣的块数的最小与最大。
fn over_reserved_slots_of_the_empty_publishes(measured: &[MeasuredPublish]) -> OverReservedSlots {
    let over_reserved_slots: Vec<i128> = measured
        .iter()
        .filter(|measured_publish| measured_publish.kind == MeasuredPublishKind::Empty)
        .map(|measured_publish| -measured_publish.difference_in_slots())
        .collect();
    OverReservedSlots {
        fewest: *over_reserved_slots
            .iter()
            .min()
            .expect("每格至少一次空发布"),
        most: *over_reserved_slots
            .iter()
            .max()
            .expect("每格至少一次空发布"),
    }
}

/// 这一格量到的多扣与钉住的（实审 A4c 量表 K1 那一列）对不上时的一行；对得上是 `None`。
fn over_reservation_that_differs_from_the_pinned_one(
    cell: &PoolGeometryCell,
    measured: &[MeasuredPublish],
) -> Option<String> {
    let measured_over_reservation = over_reserved_slots_of_the_empty_publishes(measured);
    (measured_over_reservation != cell.over_reserved_slots_of_the_empty_publishes).then(|| {
        format!(
            "{}：量到 {measured_over_reservation:?}，钉住的是 {:?}",
            cell.name, cell.over_reserved_slots_of_the_empty_publishes
        )
    })
}

/// 验收第 1 步（实审 A4b，用户 2026-09-27 定一律按最坏情况计；实审 A4d，分配记录树那一项取 K1 整棵树）：产品容量下每一次空发布——
/// 每块盘一片叶的、bump 游标跨叶的、中央映射树两层的——每块盘写出的固定点槽数都不多于之前那一版现算的 ckpt_cost（保留池不少扣）。
/// 五格各自量到的：四档盘宽各至少一次跨叶（A4 量表那几次：1 GiB txg 18–19、4 GiB 与 64 GiB txg 14–15、1 TiB txg 12–13），
/// 中央映射树两层那一格每次空发布之前映射树都是两层。记账树与树表照实写口径核一遍：每次空发布重写的记账树节点数等于之前那一版的节点数、树表一个。
/// 每格多扣的块数（K1 的代价）钉成实审 A4c 量表 K1 那一列：4 GiB 527–529、64 GiB 10337–10339、1 TiB 167298–167300、
/// 文件 300 单元那一格 528–530；1 GiB 那一格 132–134。1 GiB 这一格按式子现算：产品容量下分配记录树高 2，K1 = 1 + 2 块盘 × 每块盘
/// 单元区罩到的叶数，加中央映射树 1、记账树 1、树表 1；每次空发布每块盘实写 6 块（跨叶那两次 8 块）。单元区起点随环长走之后
/// （实审 A3b）这一格的环取 159 MiB，单元区 [11200, 65536) 罩叶 13–80 共 68 片，ckpt_cost = 1 + 136 + 3 = 140，多扣 134 / 132；
/// A4c 量的是单元区 [50176, 65536)、叶 61–80 共 20 片，ckpt_cost 44，多扣 38 / 36。跨叶那两次仍在 txg 18–19。
/// 判别力：分配记录树那一项退回每块盘两条叶路径（K0）或少数一层、少数一块盘、少数根时，多扣的块数与钉住的对不上，红在最后一条断言
/// （K0 在这五格上不少扣，它少扣的那几段历史在 [`empty_publishes_of_the_histories_where_one_device_rewrites_more_than_two_leaves_fit_in_the_whole_tree_checkpoint_cost`]）。
#[test]
fn every_empty_publish_under_the_node_format_capacities_fits_in_the_worst_case_checkpoint_cost_before_it(
) {
    let mut under_reserved = Vec::new();
    let mut over_reservations_that_differ = Vec::new();
    for cell in cells_with_the_node_format_capacities() {
        let mut crossings = 0;
        let mut empty_publishes_on_a_taller_central_mapping_tree = 0;
        let mut empty_publishes = 0;
        let measured_publishes = measure_the_cell(&cell);
        over_reservations_that_differ.extend(over_reservation_that_differs_from_the_pinned_one(
            &cell,
            &measured_publishes,
        ));
        for measured in measured_publishes {
            if measured.kind != MeasuredPublishKind::Empty {
                continue;
            }
            empty_publishes += 1;
            assert_eq!(
                (measured.accounting_tree_nodes, measured.tree_table_units),
                (measured.accounting_tree_nodes_before, 1),
                "记账树每发布整批重写之前那一版的节点数、树表一个：{}",
                measured.line(&cell)
            );
            if measured.rewrites_more_than_one_leaf_path_on_some_device() {
                crossings += 1;
            }
            if measured.central_mapping_tree_height_before > 1 {
                empty_publishes_on_a_taller_central_mapping_tree += 1;
            }
            if measured.difference_in_slots() > 0 {
                under_reserved.push(measured.line(&cell));
            }
        }
        assert!(
            crossings > 0,
            "{}：至少量到一次 bump 游标跨叶的空发布",
            cell.name
        );
        match cell.file_growth {
            FileGrowthBeforeTheEmptyPublishes::StaysAtOneDataUnit => assert_eq!(
                empty_publishes_on_a_taller_central_mapping_tree, 0,
                "{}：文件一个数据单元时产品容量下的中央映射树一层",
                cell.name
            ),
            FileGrowthBeforeTheEmptyPublishes::SequentialWriteToDataUnits(_) => assert_eq!(
                empty_publishes_on_a_taller_central_mapping_tree, empty_publishes,
                "{}：顺序写到几百个数据单元之后每次空发布之前中央映射树都高于一层",
                cell.name
            ),
        }
    }
    assert!(
        under_reserved.is_empty(),
        "空发布每块盘写出的固定点槽数多于之前那一版现算的 ckpt_cost（保留池少扣）：\n{}",
        under_reserved.join("\n")
    );
    assert!(
        over_reservations_that_differ.is_empty(),
        "每格多扣的块数与钉住的（实审 A4c 量表 K1 那一列）对不上：\n{}",
        over_reservations_that_differ.join("\n")
    );
}

/// 只供测试的压小容量下（中央映射树 2 层、3 层与空发布里照样切分、长高的那几格，记账树 4 个节点）每一次空发布每块盘写出的固定点槽数
/// 都不多于按同一个容量现算的 ckpt_cost（`checkpoint_cost_of_the_version_to_build_on_with_node_capacities`）。
/// 核这一路真的走到了：映射树高于一层的空发布、空发布里映射树长高的，三格合起来各至少一次。
/// 每格多扣的块数钉成实审 A4c 量表 K1 那一列：4-8 那一格 877–882、3-3 那一格 1057–1067、2-2 记账 8-3 那一格 144961–145503
/// （分配记录树那一项取 K1 之后删插的映射条目跟着变多，压小容量下映射树那一项跟着涨）。
/// 判别力：中央映射树那一项退回树高（不按每层可能改的路径数计）、或按节点格式的容量算时，多扣的块数与钉住的对不上，红在最后一条断言
/// （K1 的分配记录树那一项多扣几百块，不再少扣）。
#[test]
fn every_empty_publish_under_capped_code_two_tree_capacities_fits_in_the_worst_case_checkpoint_cost_counted_with_them(
) {
    let mut under_reserved = Vec::new();
    let mut over_reservations_that_differ = Vec::new();
    let mut empty_publishes_on_a_taller_central_mapping_tree = 0;
    let mut empty_publishes_that_grew_the_central_mapping_tree = 0;
    for cell in cells_with_capped_code_two_trees() {
        let measured_publishes = measure_the_cell(&cell);
        over_reservations_that_differ.extend(over_reservation_that_differs_from_the_pinned_one(
            &cell,
            &measured_publishes,
        ));
        for measured in measured_publishes {
            if measured.kind != MeasuredPublishKind::Empty {
                continue;
            }
            if measured.central_mapping_tree_height_before > 1 {
                empty_publishes_on_a_taller_central_mapping_tree += 1;
            }
            if measured.central_mapping_tree_height_after
                > measured.central_mapping_tree_height_before
            {
                empty_publishes_that_grew_the_central_mapping_tree += 1;
            }
            if measured.difference_in_slots() > 0 {
                under_reserved.push(measured.line(&cell));
            }
        }
    }
    assert!(
        empty_publishes_on_a_taller_central_mapping_tree > 0,
        "压小容量那几格至少量到一次中央映射树高于一层的空发布"
    );
    assert!(
        empty_publishes_that_grew_the_central_mapping_tree > 0,
        "压小容量那几格至少量到一次空发布里中央映射树长高"
    );
    assert!(
        under_reserved.is_empty(),
        "空发布每块盘写出的固定点槽数多于按同一个容量现算的 ckpt_cost（保留池少扣）：\n{}",
        under_reserved.join("\n")
    );
    assert!(
        over_reservations_that_differ.is_empty(),
        "每格多扣的块数与钉住的（实审 A4c 量表 K1 那一列）对不上：\n{}",
        over_reservations_that_differ.join("\n")
    );
}

/// 一段随机历史的两块盘多宽。
#[derive(Clone, Copy, Debug)]
enum UnitAreaOfTheHistory {
    /// 两块 4 GiB 的盘（`HistoryDeviceWidth::FourGibibytes` 的参数）。
    FourGibibyteDevices,
    /// 参数照 `HistoryDeviceWidth::UnitAreaOf384Slots` 那一档（journal 环 6 MiB），单元区从那条环末尾的下一个槽（1408）起这么多槽
    /// （单元区起点随环长走，实审 A3b；改之前从编译期的 50176 起）。
    SlotsFromTheUnitAreaStartOfTheSmallDeviceRing(u64),
}

/// 抽中覆盖写那一档（55–69）时做什么：「逼满」的历史把它也换成顺序写。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum OverwritesOfTheHistory {
    StayOverwrites,
    BecomeSequentialWrites,
}

/// 实审 A4c 探针（`probe_a4c_leaves.rs`，报告第二节「怎么量的」）抽的一段历史：两块稀疏盘 mkfs → 取号 → 暖机 → 第一个文件，
/// 之后经挂着的会话每步按抽签：55% 直接发一次空发布、15% 覆盖写 2999 字节、20% 顺序写到 1..=`data_units_at_most` 个数据单元、10% 崩了再挂。
/// 抽签与探针逐位相同（同一个 xorshift64、同一个次序），每块盘两条叶路径（K0）的实现上重放出来的就是探针量过的那一段。
struct HistoryOfTheA4cProbe {
    name: &'static str,
    unit_area: UnitAreaOfTheHistory,
    seed: u64,
    steps: usize,
    data_units_at_most: u64,
    overwrites: OverwritesOfTheHistory,
}

/// 探针的抽签：xorshift64，从 `seed | 1` 起。
struct DrawsOfTheA4cProbe(u64);

impl DrawsOfTheA4cProbe {
    fn next_draw(&mut self) -> u64 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        self.0
    }
}

/// 一段历史里量到的每一次空发布（重写的角色都是固定点：`admission::space_budget_of_role` 是 checkpoint 保留池那一格）。
struct EmptyPublishOfTheHistory {
    label: String,
    checkpoint_cost_before: MetadataBlocks,
    fixed_point_slots_on_each_device: u64,
    most_leaves_rewritten_on_one_device: usize,
}

impl EmptyPublishOfTheHistory {
    fn line(&self) -> String {
        format!(
            "name=a4d-history-empty-publish publish={} checkpoint_cost={} fixed_point_slots_per_device={} most_leaves_on_one_device={}",
            self.label,
            self.checkpoint_cost_before.0,
            self.fixed_point_slots_on_each_device,
            self.most_leaves_rewritten_on_one_device
        )
    }
}

/// 一次发布是不是空发布（只写固定点单元）；是的话量它，接在 `previous` 后面。
fn measure_if_empty_publish(
    label: String,
    previous: &TransactionOutput,
    published: &TransactionOutput,
    allocator: &PoolAllocator,
) -> Option<EmptyPublishOfTheHistory> {
    let only_fixed_point_units = published
        .rewritten
        .iter()
        .all(|role| space_budget_of_role(*role) == SpaceBudgetOfARole::CheckpointReservePool);
    if !only_fixed_point_units {
        return None;
    }
    let mut leaves_on_each_device: BTreeMap<DeviceIdentity, usize> = BTreeMap::new();
    for role in &published.rewritten {
        if let TransactionUnit::AllocationTreeNodeBelowTheRoot(position) = role {
            if position.level == 0 {
                *leaves_on_each_device.entry(position.device).or_default() += 1;
            }
        }
    }
    Some(EmptyPublishOfTheHistory {
        label,
        checkpoint_cost_before: checkpoint_cost_of_the_version_to_build_on(
            Some(previous),
            allocator,
        ),
        fixed_point_slots_on_each_device: published
            .rewritten
            .iter()
            .map(|role| role.span_slots())
            .sum(),
        most_leaves_rewritten_on_one_device: leaves_on_each_device
            .values()
            .copied()
            .max()
            .unwrap_or(0),
    })
}

/// 两块盘 mkfs、取号、暖机、第一个文件（3000 字节），会话接着第一个文件那一版（`common_admission` 的起步，盘宽由 `unit_area` 定）。
fn pool_after_the_first_file_on(
    unit_area: UnitAreaOfTheHistory,
) -> PoolUnderTest<SparseBlockDevice> {
    let (parameters, device_bytes) = match unit_area {
        UnitAreaOfTheHistory::FourGibibyteDevices => (
            HistoryDeviceWidth::FourGibibytes.parameters(),
            HistoryDeviceWidth::FourGibibytes.device_bytes(),
        ),
        UnitAreaOfTheHistory::SlotsFromTheUnitAreaStartOfTheSmallDeviceRing(unit_area_slots) => {
            let parameters = HistoryDeviceWidth::UnitAreaOf384Slots.parameters();
            let unit_area_start =
                UnitAreaStart::following_the_journal_ring(parameters.geometry.journal_ring_bytes)
                    .expect("小盘那一档的 6 MiB 环末尾的下一个槽 1408 = 22 × 64，落在聚簇段边界上");
            (
                parameters,
                (unit_area_start.slot().0 + unit_area_slots) * SLOT_BYTES,
            )
        }
    };
    let mut devices: Vec<(DeviceIdentity, SparseBlockDevice)> =
        [DeviceIdentity(0), DeviceIdentity(1)]
            .into_iter()
            .map(|identity| {
                (
                    identity,
                    SparseBlockDevice::new(device_bytes, PhysicalBlockSizeInBytes(512)),
                )
            })
            .collect();
    let genesis = make_filesystem(&parameters, &mut devices).expect("mkfs");
    let mut allocator = allocator_after_make_filesystem(&parameters, &devices, &genesis);
    let first_content = content_of(3000, 0);
    let (instance, first_version) = {
        let mut writer = PoolWriter::new(&parameters, devices.as_mut_slice());
        let instance = acquire_instance(&mut writer).expect("取号");
        let warmed_up = warm_up(&mut writer, &genesis.root, instance).expect("暖机");
        let first_version = publish_first_file(
            &mut writer,
            &mut allocator,
            warmed_up.roots.last().expect("暖机两代根"),
            FirstFile {
                content: &first_content,
                write_time_seconds: FIRST_WRITE_TIME_SECONDS,
            },
            instance,
            &warmed_up.last_record_bytes,
        )
        .expect("第一个文件");
        (instance, first_version)
    };
    let parameters_and_device_table =
        ParametersAndDeviceTableOfTheMount::of_a_pool_on_disk(&devices)
            .expect("mkfs 同一个进程刚写成的两块盘：系统配置读得出、与盘表相符");
    PoolUnderTest {
        parameters,
        device_bytes,
        devices,
        session: Some(MountedSession {
            allocator,
            current: PoolVersion::WithFile(first_version),
            instance,
            shadow_ledger: ShadowLedger::On,
            parameters_and_device_table,
        }),
        write_time_seconds: FIRST_WRITE_TIME_SECONDS,
        content_of_the_current_version: first_content,
        content_of_each_root: BTreeMap::new(),
    }
}

/// 重放一段历史，交回其中每一次空发布（直接发的、会话推的抬 F、崩了再挂之后的暖机与挂载推的抬 F）。某一步被拒照探针的做法记一行、接着走。
fn replay_the_history(
    history: &HistoryOfTheA4cProbe,
) -> (Vec<EmptyPublishOfTheHistory>, Vec<String>) {
    let mut pool = pool_after_the_first_file_on(history.unit_area);
    let mut draws = DrawsOfTheA4cProbe(history.seed | 1);
    let mut empty_publishes = Vec::new();
    let mut refusals = Vec::new();
    // 迭代上界是 `history.steps`；跨轮携带的是池（盘、会话）与抽签的状态。
    for step in 0..history.steps {
        let label = format!("{}#{step}", history.name);
        let draw = draws.next_draw() % 100;
        let data_units_of_the_change = match (draw, history.overwrites) {
            (0..=54, _) => None,
            (55..=69, OverwritesOfTheHistory::StayOverwrites) => Some(0),
            (55..=69, OverwritesOfTheHistory::BecomeSequentialWrites) | (70..=89, _) => {
                Some(1 + draws.next_draw() % history.data_units_at_most)
            }
            (90.., _) => {
                match pool.crash_and_mount_writable() {
                    Ok(output) => {
                        let allocator = pool.session().allocator.clone();
                        let mut previous = output.row_publish.file_version().cloned();
                        let floor_raise_publishes = output
                            .space_admission
                            .floor_raises()
                            .iter()
                            .flat_map(|raised| {
                                raised.publishes.iter().cloned().map(PoolVersion::WithFile)
                            });
                        for published in output
                            .warm_up_publishes
                            .iter()
                            .cloned()
                            .chain(floor_raise_publishes)
                        {
                            if let (Some(previous_version), Some(published_version)) =
                                (previous.as_ref(), published.file_version())
                            {
                                empty_publishes.extend(measure_if_empty_publish(
                                    format!("{label}/mount"),
                                    previous_version,
                                    published_version,
                                    &allocator,
                                ));
                            }
                            previous = published.file_version().cloned();
                        }
                    }
                    Err(error) => refusals.push(format!("{label} 崩了再挂：{error:?}")),
                }
                continue;
            }
        };
        let previous = pool.current_file_version().clone();
        let Some(data_units) = data_units_of_the_change else {
            let session = pool.session.as_mut().expect("会话");
            let mut writer = PoolWriter::new(&pool.parameters, pool.devices.as_mut_slice());
            match publish_version(
                &mut writer,
                &mut session.allocator,
                empty_publish_plan_after(&previous),
                Some(&previous),
            ) {
                Ok(published) => {
                    empty_publishes.extend(measure_if_empty_publish(
                        label,
                        &previous,
                        &published,
                        &session.allocator,
                    ));
                    session.current = PoolVersion::WithFile(published);
                }
                Err(error) => refusals.push(format!("{label} 空发布：{error:?}")),
            }
            continue;
        };
        let outcome = if data_units == 0 {
            pool.overwrite(OVERWRITE_BYTES)
        } else {
            pool.sequential_write(
                usize::try_from(data_units).expect("单元数") * data_unit_payload_capacity(),
            )
        };
        let floor_raises = match &outcome {
            Ok(published) => published.floor_raises.clone(),
            Err(UserChangeRefused::NoSpaceAfterRaisingTheFloor(no_space)) => {
                no_space.floor_raises.clone()
            }
            Err(other) => {
                refusals.push(format!("{label} 用户改动：{other:?}"));
                Vec::new()
            }
        };
        let allocator = pool.session().allocator.clone();
        let mut previous_of_the_raise = previous;
        for published in floor_raises
            .iter()
            .flat_map(|raised| raised.publishes.iter())
        {
            empty_publishes.extend(measure_if_empty_publish(
                format!("{label}/raise"),
                &previous_of_the_raise,
                published,
                &allocator,
            ));
            previous_of_the_raise = published.clone();
        }
    }
    (empty_publishes, refusals)
}

/// 接在 `current` 后面的一次空发布的计划（暖机、抬 F 与卸载推的那一种：不写文件内容、实例表照抄）。
fn empty_publish_plan_after(current: &TransactionOutput) -> PublishPlan<'static> {
    PublishPlan {
        txg: CheckpointTxg(current.root.checkpoint_txg.0 + 1),
        counter: current.record.counter + 1,
        transaction: 0,
        highest_transaction_number_before_this_publish: current
            .highest_transaction_number_in_this_instance,
        instance: current.root.instance,
        back_chain: back_chain_of(&current.record_bytes),
        file: None,
        new_inode_records: &[],
        instance_table: InstanceTablePlan::Carry(current.root.instance_table),
        tree_birth_txg: current.tree_birth_txg(),
        tree_identifier_watermark: current.root.tree_identifier_watermark,
        rollback_floor: current.root.rollback_floor,
    }
}

/// 实审 A4c 量到每块盘两条叶路径（K0）少扣的两段历史（报告第二节「三种情形各一段历史」），在分配记录树那一项取 K1 整棵树之后重放：
/// - 单元区 2600 槽、逼满、种子 13、200 步：情形 ②（没有全空段，回落到最低空槽、一个单元一处）。A4c 量的那一段单元区从 50176 起，
///   K0 下第 22 步崩了再挂之后的暖机一块盘改叶 61–64、实写 12 / ckpt_cost 8（少扣 4），200 步里少扣 4 次。单元区起点随环长走之后
///   （实审 A3b）小盘那一档的 6 MiB 环把单元区挪到 [1408, 4008)、罩叶 1–4；K0 下重放照样在第 22 步崩了再挂之后的暖机一块盘改 4 片叶、
///   实写 12 / ckpt_cost 8，200 步里少扣 4 次（第 22、58、137、154 步，实测）。K1 下 144 次空发布里一块盘改的叶多于两片的有 4 次（实测）；
/// - 两块 4 GiB、种子 2、135 步：情形 ③（换下的上一版节点很久以前落在别的叶），这段历史里准入一次都没拒、K0 与 K1 走的是同一段；
///   第 81 步起一块盘改的叶多于两片，K0 下第 132 步（txg 150）一块盘改 4 片叶、实写 14 / ckpt_cost 12（少扣 2）。
///
/// 每段都要有一次空发布一块盘改的叶多于两片（那种情形真走到了），且每一次空发布每块盘实写的固定点槽数都不多于之前那一版现算的 ckpt_cost。
/// 抽签与探针逐位相同：两块 4 GiB 那一段（默认环，单元区起点仍是 50176）K0 下重放出来的就是探针量过的那一段（第 132 步的数与报告第二节那一行相同）。
/// 判别力：分配记录树那一项退回每块盘两条叶路径（K0）时，两段历史都少扣，红在最后一条断言。
#[test]
#[ignore = "harness 重档：debug 下单条跑过 60 秒；随时跑：cargo test -p singlefs-harness -- --ignored，经内存包装"]
fn empty_publishes_of_the_histories_where_one_device_rewrites_more_than_two_leaves_fit_in_the_whole_tree_checkpoint_cost(
) {
    let histories = [
        HistoryOfTheA4cProbe {
            name: "unit-area-2600-filled-seed-13",
            unit_area: UnitAreaOfTheHistory::SlotsFromTheUnitAreaStartOfTheSmallDeviceRing(2600),
            seed: 13,
            steps: 200,
            data_units_at_most: 200,
            overwrites: OverwritesOfTheHistory::BecomeSequentialWrites,
        },
        HistoryOfTheA4cProbe {
            name: "two-4GiB-seed-2",
            unit_area: UnitAreaOfTheHistory::FourGibibyteDevices,
            seed: 2,
            steps: 135,
            data_units_at_most: 900,
            overwrites: OverwritesOfTheHistory::StayOverwrites,
        },
    ];
    let mut under_reserved = Vec::new();
    for history in &histories {
        let (empty_publishes, refusals) = replay_the_history(history);
        for empty_publish in &empty_publishes {
            println!("{}", empty_publish.line());
        }
        let more_than_two_leaves_on_one_device = empty_publishes
            .iter()
            .filter(|empty_publish| empty_publish.most_leaves_rewritten_on_one_device > 2)
            .count();
        println!(
            "name=a4d-history history={} empty_publishes={} more_than_two_leaves_on_one_device={} refusals={}",
            history.name,
            empty_publishes.len(),
            more_than_two_leaves_on_one_device,
            refusals.len()
        );
        for refusal in &refusals {
            println!("name=a4d-history-refusal {refusal}");
        }
        assert!(
            more_than_two_leaves_on_one_device > 0,
            "{}：至少一次空发布一块盘改的叶多于两片",
            history.name
        );
        under_reserved.extend(
            empty_publishes
                .iter()
                .filter(|empty_publish| {
                    empty_publish.fixed_point_slots_on_each_device
                        > empty_publish.checkpoint_cost_before.0
                })
                .map(EmptyPublishOfTheHistory::line),
        );
    }
    assert!(
        under_reserved.is_empty(),
        "空发布每块盘写出的固定点槽数多于之前那一版现算的 ckpt_cost（保留池少扣）：\n{}",
        under_reserved.join("\n")
    );
}

/// 一次空发布的固定点单元数（三块盘那一格按它取、放）：两块 4 GiB 盘上每块盘一片叶那几次实写 8 个。
const FIXED_POINT_UNITS_OF_AN_EMPTY_PUBLISH: usize = 8;

/// 三块 4 GiB 盘的分配器（第一版每个单元落池里每一块盘、各盘同槽），装着一棵空的分配记录树（树表 0 条、写过行的那一版）；
/// 按 `case` 先取单槽的提交内生块把 bump 游标往前推（跨叶那一种推到游标所在那片叶只剩一代那么多槽），再取一代 `FIXED_POINT_UNITS_OF_AN_EMPTY_PUBLISH` 个、
/// 下一代把它们全释放、再取同样多个——空发布在分配记录树上改的记录就是这两样。交回改了记录的节点（每块盘根之下几个、根改没改）与两代各自落在哪片叶。
struct ThreeDevicePublishOnTheAllocationRecordTree {
    geometry: AllocationRecordTreeGeometry,
    allocator: PoolAllocator,
    nodes_below_the_root_on_each_device: BTreeMap<DeviceIdentity, u64>,
    root_changed: bool,
    leaves_of_the_released_generation: BTreeSet<AllocationRecordTreeNode>,
    leaves_of_the_new_generation: BTreeSet<AllocationRecordTreeNode>,
}

fn three_device_publish_on_the_allocation_record_tree(
    case: ThreeDeviceCase,
) -> ThreeDevicePublishOnTheAllocationRecordTree {
    let devices = [DeviceIdentity(0), DeviceIdentity(1), DeviceIdentity(2)];
    let mut allocator = PoolAllocator::new(
        devices
            .iter()
            .map(|device| DeviceFreeMap::new(*device, 4 * GIBIBYTE))
            .collect(),
    );
    allocator.note_allocation_record_tree_of_the_version_without_file(
        AllocationRecordTreeOfTheVersionWithoutFile {
            version: AllocationRecordTreeVersion::default(),
            records: Vec::new(),
        },
    );
    let geometry = AllocationRecordTreeGeometry::of_allocator(&allocator);
    let mut allocate_one_generation = |units: usize, generation: CheckpointTxg| -> Vec<_> {
        (0..units)
            .map(|_| {
                allocator
                    .allocate_commit_generated(UnitFootprint::OneSlot, generation)
                    .expect("空盘上分得到单槽节点")
            })
            .collect()
    };
    match case {
        ThreeDeviceCase::BothGenerationsInOneLeaf => {}
        ThreeDeviceCase::ReleasedInTheOldLeafAllocatedInTheNext => {
            let leaf_slots = singlefs_format::ALLOCATION_RECORD_TREE_LEAF_SLOTS;
            let slots_per_generation =
                u64::try_from(FIXED_POINT_UNITS_OF_AN_EMPTY_PUBLISH).expect("一代的单元数");
            // 迭代上界是一片叶的槽数：每一轮游标往前一槽，走不出一片叶就停在「只剩一代那么多槽」那一格。
            for filler_index in 0..=leaf_slots {
                assert!(
                    filler_index < leaf_slots,
                    "推游标一片叶之内停得下来：每一轮挪一槽"
                );
                let filler = allocate_one_generation(1, CheckpointTxg(1));
                let next_slot = filler[0].slot.0 + filler[0].span;
                if leaf_slots - next_slot % leaf_slots == slots_per_generation {
                    break;
                }
            }
        }
    }
    let released_generation =
        allocate_one_generation(FIXED_POINT_UNITS_OF_AN_EMPTY_PUBLISH, CheckpointTxg(1));
    let records_before = allocator.records().to_vec();
    for placement in &released_generation {
        allocator.release(*placement, CheckpointTxg(2));
    }
    let new_generation: Vec<_> = (0..FIXED_POINT_UNITS_OF_AN_EMPTY_PUBLISH)
        .map(|_| {
            allocator
                .allocate_commit_generated(UnitFootprint::OneSlot, CheckpointTxg(2))
                .expect("空盘上分得到单槽节点")
        })
        .collect();
    let changed = nodes_whose_contents_changed(&geometry, &records_before, allocator.records());
    let mut nodes_below_the_root_on_each_device: BTreeMap<DeviceIdentity, u64> =
        devices.iter().map(|device| (*device, 0)).collect();
    for node in &changed {
        match node {
            AllocationRecordTreeNode::BelowTheRoot(position) => {
                *nodes_below_the_root_on_each_device
                    .get_mut(&position.device)
                    .expect("池里的一块盘") += 1;
            }
            AllocationRecordTreeNode::Root => {}
        }
    }
    let leaves_on_device_zero = |placements: &[singlefs_core::allocator::Placement]| {
        placements
            .iter()
            .map(|placement| geometry.leaf_of_slot(DeviceIdentity(0), placement.slot))
            .collect::<BTreeSet<_>>()
    };
    let leaves_of_the_released_generation = leaves_on_device_zero(&released_generation);
    let leaves_of_the_new_generation = leaves_on_device_zero(&new_generation);
    ThreeDevicePublishOnTheAllocationRecordTree {
        root_changed: changed.contains(&AllocationRecordTreeNode::Root),
        geometry,
        allocator,
        nodes_below_the_root_on_each_device,
        leaves_of_the_released_generation,
        leaves_of_the_new_generation,
    }
}

/// 三块盘那一格的两种走法：两代落在同一片叶，或换下的一代在旧叶、新取的一代在新叶（bump 游标跨叶）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ThreeDeviceCase {
    BothGenerationsInOneLeaf,
    ReleasedInTheOldLeafAllocatedInTheNext,
}

impl ThreeDeviceCase {
    fn label(self) -> &'static str {
        match self {
            ThreeDeviceCase::BothGenerationsInOneLeaf => "one-leaf",
            ThreeDeviceCase::ReleasedInTheOldLeafAllocatedInTheNext => "crossing",
        }
    }
}

/// 三块盘（发布路径走不到，见模块文档）只在分配记录树这一层量：
/// - 两代都落在单元区起点那片叶里（游标从单元区起点起）：每块盘一条从叶到根之下那一层的路径（高 3 ⇒ 2 个节点）加共用的根，3 × 2 + 1 = 7；
/// - 游标先推到那片叶的末槽前 8 格，换下的一代在旧叶、新取的一代在新叶（跨叶那一次）：每块盘两片叶加它们共用的层级 1 节点 = 3 个，3 × 3 + 1 = 10。
///
/// 两格都不多于每块盘两条路径（2 × 2 = 4 个）加根；树表 0 条、写过行的那一版（`checkpoint_cost_of_the_version_to_build_on` 的 `None` 臂）上
/// 分配记录树那一项取 K1 整棵树（实审 A4d，用户 2026-09-27 定）：每块盘单元区 [50176, 262144) 罩叶 262 个、层级 1 两个，
/// 1 + 3 × 264 = 793，ckpt_cost = 793 + 树表 1 = 794，不少于两格实改的节点数加树表 1。
/// 每块盘两条路径（A4b 那一版，K0）是 3 × 2 × 2 + 1 + 1 = 14；每块盘只算一条路径（A4 那一版）是 3 × 2 + 1 + 1 = 8，跨叶那一格少扣；
/// 按树高算是 3 + 1 = 4。
#[test]
fn three_devices_rewrite_at_most_two_leaf_paths_on_each_device_and_the_shared_root() {
    for (case, expected_nodes_below_the_root_on_each_device) in [
        (ThreeDeviceCase::BothGenerationsInOneLeaf, 2),
        (ThreeDeviceCase::ReleasedInTheOldLeafAllocatedInTheNext, 3),
    ] {
        let label = case.label();
        let measured = three_device_publish_on_the_allocation_record_tree(case);
        assert_eq!(
            measured.geometry.height(),
            3,
            "三块 4 GiB 盘：分配记录树 3 层"
        );
        let checkpoint_cost = checkpoint_cost_of_the_version_to_build_on(None, &measured.allocator);
        println!(
            "name=a4-checkpoint-cost cell=three-4GiB-allocation-record-tree-only-{label} height_allocation_record_tree={} \
             allocation_record_tree_changed={}+root{} checkpoint_cost_of_the_version_without_file={}",
            measured.geometry.height(),
            measured
                .nodes_below_the_root_on_each_device
                .iter()
                .map(|(device, nodes)| format!("d{}:{}", device.0, nodes))
                .collect::<Vec<String>>()
                .join(","),
            u64::from(measured.root_changed),
            checkpoint_cost.0,
        );
        match case {
            ThreeDeviceCase::ReleasedInTheOldLeafAllocatedInTheNext => assert!(
                measured.leaves_of_the_released_generation.len() == 1
                    && measured.leaves_of_the_new_generation.len() == 1
                    && measured.leaves_of_the_released_generation
                        != measured.leaves_of_the_new_generation,
                "{label}：换下的一代在旧叶、新取的一代在新叶"
            ),
            ThreeDeviceCase::BothGenerationsInOneLeaf => assert!(
                measured.leaves_of_the_released_generation.len() == 1
                    && measured.leaves_of_the_released_generation
                        == measured.leaves_of_the_new_generation,
                "{label}：两代落在同一片叶"
            ),
        }
        assert_eq!(
            measured.nodes_below_the_root_on_each_device,
            [DeviceIdentity(0), DeviceIdentity(1), DeviceIdentity(2)]
                .into_iter()
                .map(|device| (device, expected_nodes_below_the_root_on_each_device))
                .collect(),
            "{label}：每块盘根之下改的节点"
        );
        assert!(
            measured.root_changed,
            "{label}：根罩整个 key 空间，每次都改"
        );
        let rewritten_nodes = 3 * expected_nodes_below_the_root_on_each_device + 1;
        assert_eq!(
            checkpoint_cost,
            MetadataBlocks(1 + 3 * (262 + 2) + 1),
            "{label}：分配记录树那一项取整棵树（1 + 3 × (叶 262 + 层级 1 两个) = 793）加树表 1"
        );
        assert!(
            checkpoint_cost.0 > rewritten_nodes,
            "{label}：ckpt_cost {checkpoint_cost:?} 不少于实改的 {rewritten_nodes} 个节点加树表 1"
        );
    }
}
