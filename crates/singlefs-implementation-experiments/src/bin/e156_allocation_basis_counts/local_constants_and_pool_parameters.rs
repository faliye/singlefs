//! 本地常量（盘、根环、洞族与新池新建文件的落槽）与 mkfs 参数、新池的盘。

use super::*;

/// E156 装置的固定 fsid：与别的 `eNNN` 装置各自独立取一份，登记在这里方便复跑时核对。
const E156_FILESYSTEM_IDENTIFIER: [u8; 16] = [
    0x5f, 0x53, 0x46, 0x53, 0x2d, 0x45, 0x31, 0x35, 0x36, 0x2d, 0x30, 0x30, 0x30, 0x31, 0x2d, 0x00,
];
pub(super) const FIXED_WRITE_TIME_SECONDS: u64 = 1_788_000_000;
pub(super) const IMAGE_BYTES: u64 = 4 << 30;
/// 一个单元的槽数（跨度 2 的数据单元 / inode 叶容器 / 实例表单元，登记「一」读法写死）。
pub(super) const INSTANCE_TABLE_SPAN_SLOTS: u64 = 2;
/// β0、U11、U13 那几段历史的每区槽数 S：**本地常量**，值抄自 `.claude/kb/layout/01-first-txn.md`「每区槽数 S」那一行
/// （`| 几何 | 每区槽数 S | 1 | 8 |`）。[`main`] 开头那条断言把它与实现侧 mkfs 默认回比。
pub(super) const E156_SLOTS_PER_REGION: u64 = 8;
/// 岔路 1 的 S 三个取样点（第 4 次重跑登记表头第 1 行，全部都跑）：值抄自 `.claude/kb/decisions/22-单元原子性怎么合成.md`
/// 「每区槽数 S」那一行（`| **每区槽数 S** |`：下界 4、上界 16，「4..16 之间取值」）与 `.claude/kb/layout/01-first-txn.md`
/// 「每区槽数 S」那一行（`| 几何 | 每区槽数 S | 1 | 8 |`，mkfs 写 8）。
/// [`main`] 开头回比实现侧的上下界与 mkfs 默认；每格另核「系统配置里读回来的 S = 本地值」（K8）。
pub(super) const E156_SLOTS_PER_REGION_SAMPLING_POINTS: [u64; 3] = [4, 8, 16];
pub(super) const E156_SLOTS_PER_REGION_MINIMUM: u64 = 4;
pub(super) const E156_SLOTS_PER_REGION_MAXIMUM: u64 = 16;
/// 区域设备（登记表头第 2 行，D22（单元原子性怎么合成） 已定项 16 第 4 句的区域归属 [0, 1, 0]）；[`main`] 开头与 mkfs 参数回比。
pub(super) const E156_REGION_DEVICE_NUMBERS: [u32; 3] = [0, 1, 0];
/// 区域数 R（登记表头第 2 行；`.claude/kb/decisions/22-单元原子性怎么合成.md`「`| **区域数 R** | **3**`」那一行）：根环 3S 槽、U13 (a) 的 3N 次覆盖写都从它算；
/// [`main`] 开头与 `singlefs_format::ROOT_RING_REGIONS` 回比。
pub(super) const E156_ROOT_RING_REGIONS: u64 = 3;
/// 判据门槛（登记表头第 3 行）：一个跨度 2 的单元 2 槽、跨度 1 的单元 1 槽（原登记「一」读法写死「一个单元的槽数」行；
/// D3（空间分配） 已定项 7 落点粒度一个 16 KiB 槽、跨度 1 或 2）。只住装置，没有实现侧的对应物，不回比。
pub(super) const E156_THRESHOLD_ONE_SPAN_TWO_UNIT_SLOTS: i64 = 2;
pub(super) const E156_THRESHOLD_ONE_SPAN_ONE_UNIT_SLOTS: i64 = 1;
/// 第 7 次重跑登记（`research/prompts/e156-r7-prereg.md`）第七节 A20 的本地常量表，值抄自该登记第十三节命令一 `anchors_e156_r7.py`
/// 的输出（`S=4 k_full_front=3 k_full_back=2` 等行，与各族 `holes=` 列），不读 `crates/`：前位置的洞全部落在第一圈（txg < 3S）里的最大洞数
/// `k_满(S)`、前位置排得下的最大洞数、后位置排得下的最大洞数。[`hole_family_anchor_cells_of`] 用锚点模型现算，对不上就停机 S20。
pub(super) const E156_HOLE_FAMILY_FRONT_HOLE_COUNT_INSIDE_THE_FIRST_RING: [(u64, u64); 3] =
    [(4, 2), (8, 4), (16, 8)];
pub(super) const E156_HOLE_FAMILY_FRONT_HOLE_COUNT_THAT_FITS: [(u64, u64); 3] =
    [(4, 3), (8, 5), (16, 9)];
pub(super) const E156_HOLE_FAMILY_BACK_HOLE_COUNT_THAT_FITS: [(u64, u64); 3] =
    [(4, 2), (8, 4), (16, 8)];
/// 族标签（第 7 次重跑登记 W29）：Hh = 间距不变造到洞数为止，Hc = 同一间距一直造到跨过两圈、三圈，Hs = 洞钉在同一个环槽、每圈一个。
pub(super) const E156_FAMILY_HOLE_COUNT_SWEEP: &str = "Hh";
pub(super) const E156_FAMILY_CONTINUED_SWEEP: &str = "Hc";
pub(super) const E156_FAMILY_SAME_RING_SLOT: &str = "Hs";
/// PC-同槽（登记 5.3、E25）的族标签。
pub(super) const E156_FAMILY_SAME_SLOT_CONTROL: &str = "PC-same-slot";
/// A1：单元区起点槽号。抄自 D3（空间分配） 已定项 7 依据表「落点」行（`.claude/kb/decisions/03-空间分配.md`；
/// D18（块里携带什么信息） 已定项 9 逼出的粒度）与 `crates/singlefs-format` 的同名常量——[`main`] 第一条断言把它与
/// `UNIT_AREA_START_SLOT` 回比。
pub(super) const E156_UNIT_AREA_START_SLOT: u64 = 50176;
/// A7：分配记录节点容量。抄自 `research/prompts/e156-r2-prereg.md` 表头「分配记录节点容量」行的算式
/// `⌊(16384 − 135) ÷ 20⌋`；节点头 135 字节 = 86（固定头）+ 2 × 10（子节点指针，容量算式的一部分）+ 29（其余头字段）。
pub(super) const E156_ALLOCATION_RECORD_NODE_HEADER_BYTES: u64 = 86 + 2 * 10 + 29;
pub(super) const E156_ALLOCATION_RECORD_BYTES: u64 = 20;
/// S1(f)：新池新建文件之后的分配记录条数（E156 第 3 次重跑登记「七」7.2 K1-2：28 = 2 × 14）。
pub(super) const E156_NEW_POOL_FILE_CREATION_EXPECTED_RECORD_COUNT: usize = 28;
/// K1-1（第 3 次重跑登记「七」7.2，第 4 次重跑登记第十三节命令三重跑逐行相同）：新池新建文件之后 D0 记账第 1 项 17 槽、第 5 项 1 槽。
pub(super) const E156_NEW_POOL_FILE_CREATION_ITEM1_SLOTS: u64 = 17;
pub(super) const E156_NEW_POOL_FILE_CREATION_ITEM5_SLOTS: u64 = 1;
/// K1-1 的落点表（同一条命令三的输出 `new_pool_file_creation = [...]`，按 (槽, 跨度) 升序）：数据 2、extent 1、inode 叶 2、inode 根 1、
/// 分配记录树 5 个节点各 1、记账 1、映射 1、树表 1。
pub(super) const E156_NEW_POOL_FILE_CREATION_PLACEMENTS: [(u64, u64); 12] = [
    (50180, 2),
    (50240, 1),
    (50242, 2),
    (50244, 1),
    (50245, 1),
    (50246, 1),
    (50247, 1),
    (50248, 1),
    (50249, 1),
    (50250, 1),
    (50251, 1),
    (50252, 1),
];
/// R-4（第 3 次重跑登记「七」7.2，命令三重跑逐行相同）：Hh(0, —, S = 8, ρ = 1) 前 4 次覆盖写（环还没转过）每次 D0 释放 14 槽、
/// D0 + D1 记录 + 24。
pub(super) const E156_OVERWRITES_CHECKED_BEFORE_THE_RING_WRAPS: u64 = 4;
pub(super) const E156_OVERWRITE_RELEASED_SLOTS_BEFORE_THE_RING_WRAPS: u64 = 14;
pub(super) const E156_OVERWRITE_RECORD_DELTA_BEFORE_THE_RING_WRAPS: u64 = 24;
/// R-3 本地常量①：分配记录树叶宽 W。**本地常量，值抄自 kb，不从 `crates/` 引**：抄自 `.claude/kb/decisions/08-核心索引结构.md` 里的
/// `<!-- format-const: ALLOCATION_RECORD_TREE_LEAF_SLOTS = 812 -->`。[`main`] 里 `anchor_a_d8` 那一行把它
/// 与 `singlefs_format::ALLOCATION_RECORD_TREE_LEAF_SLOTS` 回比（只观测，F21：对不上不作废、不停机）。
pub(super) const E156_ALLOCATION_RECORD_TREE_LEAF_SLOTS: u64 = 812;
/// R-3 本地常量②：分配记录树内部扇出 F。抄自 `.claude/kb/decisions/08-核心索引结构.md` 里的
/// `<!-- format-const: ALLOCATION_RECORD_TREE_INTERNAL_FANOUT = 169 -->`。
pub(super) const E156_ALLOCATION_RECORD_TREE_INTERNAL_FANOUT: u64 = 169;
/// R-3：一次覆盖写里，非分配记录树自身的「其余单元」在 D0 上的释放槽数（E156 第 3 次重跑登记「七」7.2
/// R-3：数据 2 + extent 1 + inode 叶 2 + inode 根 1 + 记账 1 + 映射 1 + 树表 1 = 9）。
/// R-3 这一族闭式与它们的函数只剩单测 U10、U12 与 PC-闭式在用（第 4 次重跑的 `main` 不跑 H0），编成只在测试里有。
#[cfg(test)]
pub(super) const E156_OVERWRITE_OTHER_UNITS_RELEASED_SLOTS: u64 = 9;
/// R-3：同一批「其余单元」里会产生新分配记录条目的槽数（不含数据：数据槽被 `data_slot()` 复用同一个
/// 已有的记录条目，不产生新条目，登记「七」7.2 R-4 的算术）。
#[cfg(test)]
pub(super) const E156_OVERWRITE_OTHER_UNITS_NEW_RECORD_SLOTS: u64 = 7;
/// R-3：一次空发布里，非分配记录树自身的「其余单元」（记账 1 + 映射 1 + 树表 1 = 3，三者都走 bump、
/// 每次都是新槽，释放与新增同值，登记「七」7.2 R-5）。
#[cfg(test)]
pub(super) const E156_EMPTY_PUBLISH_OTHER_UNITS_SLOTS: u64 = 3;

pub(super) fn parameters() -> MakeFilesystemParameters {
    MakeFilesystemParameters {
        filesystem_identifier: E156_FILESYSTEM_IDENTIFIER,
        region_devices: [DeviceIdentity(0), DeviceIdentity(1), DeviceIdentity(0)],
        geometry: SystemImmutableSizes {
            physical_block_size: 512,
            minimum_input_output_bytes: 512,
            fixed_structure_slot_spacing: 4096,
            journal_ring_bytes: JOURNAL_RING_DEFAULT_BYTES,
            root_ring_slots_per_region: RootRingSlotsPerRegion::AT_MAKE_FILESYSTEM,
        },
    }
}

/// 岔路 1 的 S 这一维：除 `root_ring_slots_per_region` 外与 [`parameters`] 逐字段相同——S 走 mkfs 参数，不必在仓副本里改常量重编。
pub(super) fn parameters_with_slots_per_region(count: u64) -> MakeFilesystemParameters {
    let slots_per_region = RootRingSlotsPerRegion::from_system_configuration_field(count)
        .expect("S 落在 4..16 闭区间");
    MakeFilesystemParameters {
        filesystem_identifier: E156_FILESYSTEM_IDENTIFIER,
        region_devices: [DeviceIdentity(0), DeviceIdentity(1), DeviceIdentity(0)],
        geometry: SystemImmutableSizes {
            physical_block_size: 512,
            minimum_input_output_bytes: 512,
            fixed_structure_slot_spacing: 4096,
            journal_ring_bytes: JOURNAL_RING_DEFAULT_BYTES,
            root_ring_slots_per_region: slots_per_region,
        },
    }
}

pub(super) fn new_pool_devices() -> Vec<(DeviceIdentity, SparseBlockDevice)> {
    (0..2u32)
        .map(|number| {
            (
                DeviceIdentity(number),
                SparseBlockDevice::new(IMAGE_BYTES, PhysicalBlockSizeInBytes(512)),
            )
        })
        .collect()
}
