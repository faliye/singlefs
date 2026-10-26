//! E181（落得下那一判与挂载下界）第一段（固定点与落得下段，跑前登记 `research/prompts/e181-preregistration.md` 5.5 节第一段）：
//! 岔路单 `research/prompts/m3-close-admission-and-shadow-ledger-forks.md` 的三行——
//! 固定点上界第二半（F9：上界 P1 与今天的 P2 在中央映射树两层、三层的写上逐次比真写的固定点）、
//! 各盘同槽（F11：先拒那一道算各盘最低落点的扫描代价随单元区怎么长）、
//! 两槽的提交内生块要不要在落得下那一判按槽对数（F10：造得出让没挡的槽够、对齐的空槽对不够的可达盘面吗）。
//! 第二段（截短盘，F14）不在这个二进制里。
//!
//! 入库装置：只读、只驱动、只观测 `singlefs-core` 的公开入口，不改核心层。每个会读 `crates/` 常量的量在这里是本地常量，
//! 值抄自 kb 的登记位，注明出处，[`check_local_constants_against_the_crates`] 逐个回比。
//! 产物按 `E7RESULT` 行打印，复跑登记在 `scripts/experiment-apparatus/replay.sh`。
//!
//! 准入写 `always` 而不写 `inputs-changed`：复跑登记（`replay.sh` 的 `E181` 一行）与长跑脚本（`--full` 重做全部片、出错后原样重起同一片）
//! 都要在输入不变时再跑同一段、拿同一份输入核产物逐字节相同；写 `inputs-changed` 的话，第二次跑会被判「输入没变」拒绝（退出码 78），
//! 复跑与重做都跑不了。强制跑（`--force`）时产物开头多一行 `name=preflight forced=…`，满足时不打，产物与没有这一层时逐字节相同。
//! 退出码：0 正常；3 停机（含「边没找到」，产物里有 `name=stop`）；准入判法退 78 时照 78 退。

// admission: always 复跑登记与长跑脚本都要在输入不变时再跑同一段、比逐字节相同的产物，上一次跑完不替这一次作保
// run-condition: command python3

use std::collections::BTreeSet;
use std::sync::Arc;
use std::time::Instant;

use singlefs_core::address::{
    CheckpointTxg, DataUnitIndexInFile, DeviceIdentity, DeviceOffsetInBytes, InodeNumber,
    InstanceGeneration, SlotNumber,
};
use singlefs_core::admission::{
    accounting_tree_nodes_after_the_publish_built_on, admit_the_units_landing_on_every_device,
    central_mapping_entries_of_one_lineage_container_cow,
    central_mapping_entries_of_one_retirement_container_cow,
    checkpoint_cost_of_one_lineage_container_cow, checkpoint_cost_of_one_retirement_container_cow,
    checkpoint_cost_of_the_version_to_build_on_with_node_capacities, space_budget_of_role,
    PlacementRoomOnOneDevice, SpaceBudgetOfARole, UnitsOfAPublishToLand,
};
use singlefs_core::allocator::{PoolAllocator, UnitAreaStart};
use singlefs_core::block_device::{
    BlockDevice, BlockDeviceError, DiscardOutcome, PhysicalBlockSizeInBytes, WriteDurability,
};
use singlefs_core::code_two_tree::CodeTwoTreeNodeCapacity;
use singlefs_core::make_filesystem::{
    allocator_after_make_filesystem, make_filesystem, MakeFilesystemParameters,
    INSTANCE_TABLE_SLOT, TREE_TABLE_GENESIS_SLOT,
};
use singlefs_core::mount::{mount_writable, unmount};
use singlefs_core::mounted_session::{MountedSession, UserChange};
use singlefs_core::root_ring::RootRingSlotsPerRegion;
use singlefs_core::system_configuration::SystemImmutableSizes;
use singlefs_core::transaction::{
    acquire_instance, publish_first_file, publish_new_inodes, publish_sequential_write, warm_up,
    CodeTwoTreeNodeCapacities, FirstFile, MultiLevelCodeTwoTree, PoolWriter, PublishError,
    TransactionOutput, TransactionUnit,
};
use singlefs_format::JOURNAL_RECORD_BYTES;
use singlefs_harness::memory_pool::{SparseDevice, SECTOR_BYTES};

/// 本地常量：槽 16 KiB（D3（空间分配） 已定项 10 ①：「段长 64 个 16 KiB 槽」）。
const SLOT_BYTES_LOCAL: u64 = 16384;
/// 本地常量：聚簇段 64 槽（D3（空间分配） 已定项 10 ①）。
const CLUSTER_SEGMENT_SLOTS_LOCAL: u64 = 64;
/// 本地常量：默认 768 MiB 环的单元区起始槽号 U（D3（空间分配） 已定项 10 ④：「默认 768 MiB 环是 50176」）。
const UNIT_AREA_START_SLOT_LOCAL: u64 = 50176;
/// 本地常量：默认 journal 环 768 MiB（D3（空间分配） 已定项 10 ④）。
const DEFAULT_JOURNAL_RING_BYTES_LOCAL: u64 = 805_306_368;
/// 本地常量：mkfs 的每区槽数 S = 8（`.claude/kb/layout/01-first-txn.md`「每区槽数 S」那一行）。
const ROOT_RING_SLOTS_PER_REGION_LOCAL: u64 = 8;
/// 本地常量：分配记录树层级 0 每位置罩 812 槽，内部扇出 169（D28（挂载期承诺量） 已定项 4：S_L = 812 × 169^L）。
const ALLOCATION_RECORD_TREE_LEAF_SLOTS_LOCAL: u64 = 812;
const ALLOCATION_RECORD_TREE_INTERNAL_FANOUT_LOCAL: u64 = 169;
/// 本地常量：mkfs 的实例表落 U、U + 1，树表第 0 版落 U + 2（D3（空间分配） 已定项 8 射程）。
const INSTANCE_TABLE_OFFSET_FROM_UNIT_AREA_START_LOCAL: u64 = 0;
const TREE_TABLE_OFFSET_FROM_UNIT_AREA_START_LOCAL: u64 = 2;
/// 本地常量：一个数据单元两槽（D3（空间分配） 已定项 8 第 1 条：起点 32768 对齐的两槽）。
const DATA_UNIT_SLOTS_LOCAL: u64 = 2;
/// 一个数据单元的字节数：两槽。
const DATA_UNIT_BYTES_LOCAL: usize = 32768;
/// 固定写入时间（只进 inode 记录，不影响落点与槽数）。
const FIXED_WRITE_TIME_SECONDS: u64 = 1_788_000_000;
/// 单元数不少于这个数的写，把发布用时打到标准错误（只给人看进度，不进产物）。
const LOGGED_WRITE_UNITS: usize = 5000;
/// 本装置的固定文件系统身份（与别的 eNNN 装置各自独立取一份）。
const E181_FILESYSTEM_IDENTIFIER: [u8; 16] = [
    0x5f, 0x53, 0x46, 0x53, 0x2d, 0x45, 0x31, 0x38, 0x31, 0x2d, 0x30, 0x30, 0x30, 0x31, 0x2d, 0x00,
];
/// 第一个文件的字节数（E142 的 3000 字节，一个数据单元装得下；A3、A4 钉的 t1@50180 就是这个文件）。
const FIRST_FILE_BYTES: usize = 3000;
const GIBIBYTE: u64 = 1 << 30;

/// 这个进程已经打出的 `E7RESULT` 行数（收尾行的 `emitted` 要等于整份产物的行数，复跑脚本的完整性闸核它）。
static EMITTED_LINES: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);

/// 打一行产物：`E7RESULT <字段>`。
fn emit(fields: &str) {
    EMITTED_LINES.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
    println!("E7RESULT {fields}");
}

/// 收尾行：`name=done emitted=<行数，含这一行> loadavg1=<1 分钟负载> cpus=<在线 CPU 数>`（复跑脚本计时行的负载关读这两个字段）。
fn emit_the_done_line() {
    let emitted = EMITTED_LINES.load(std::sync::atomic::Ordering::SeqCst) + 1;
    let loadavg_text = std::fs::read_to_string("/proc/loadavg").expect("读得到 /proc/loadavg");
    let loadavg_one_minute = loadavg_text
        .split_whitespace()
        .next()
        .expect("loadavg 第一列");
    let cpuinfo_text = std::fs::read_to_string("/proc/cpuinfo").expect("读得到 /proc/cpuinfo");
    let cpus = cpuinfo_text
        .lines()
        .filter(|line| line.starts_with("processor"))
        .count();
    println!("E7RESULT name=done emitted={emitted} loadavg1={loadavg_one_minute} cpus={cpus}");
}

/// 稀疏设备的一层：写过的扇区加被整段清零或 discard 过的扇区区间。清零先于同一层里的写生效（读时先抹区间、再盖写过的扇区）。
#[derive(Clone, Debug, Default)]
struct DeviceLayer {
    written: SparseDevice,
    zeroed_sector_ranges: Vec<(u64, u64)>,
}

/// 写时复制的内存盘：封存的层只读、可被几份拷贝共享，顶层只属于这一份。
/// 拷贝一个已填满的池只拷层的引用，不拷几 GiB 的扇区（跑前登记 5.2 节「64 个种子各在写时复制的包装上走 300 步」）。
struct ForkedDevice {
    sealed_layers: Vec<Arc<DeviceLayer>>,
    top_layer: DeviceLayer,
    size_in_bytes: u64,
    physical_block_size: PhysicalBlockSizeInBytes,
}

impl ForkedDevice {
    fn empty(size_in_bytes: u64) -> Self {
        Self {
            sealed_layers: Vec::new(),
            top_layer: DeviceLayer::default(),
            size_in_bytes,
            physical_block_size: PhysicalBlockSizeInBytes(512),
        }
    }

    /// 把顶层封存，交回一份与它读到同样字节的拷贝（拷贝之后各写各的顶层）。
    fn fork(&mut self) -> ForkedDevice {
        let top_layer_is_empty = self.top_layer.written.written_sectors().is_empty()
            && self.top_layer.zeroed_sector_ranges.is_empty();
        if !top_layer_is_empty {
            self.sealed_layers
                .push(Arc::new(std::mem::take(&mut self.top_layer)));
        }
        ForkedDevice {
            sealed_layers: self.sealed_layers.clone(),
            top_layer: DeviceLayer::default(),
            size_in_bytes: self.size_in_bytes,
            physical_block_size: self.physical_block_size,
        }
    }

    /// 同 [`ForkedDevice::fork`]，但不封存：顶层必须已经是空的（交给几个线程共用同一份底时用）。
    fn fork_of_a_sealed_base(&self) -> ForkedDevice {
        assert!(
            self.top_layer.written.written_sectors().is_empty()
                && self.top_layer.zeroed_sector_ranges.is_empty(),
            "共用的底顶层是空的：先 fork 过一次再交给线程"
        );
        ForkedDevice {
            sealed_layers: self.sealed_layers.clone(),
            top_layer: DeviceLayer::default(),
            size_in_bytes: self.size_in_bytes,
            physical_block_size: self.physical_block_size,
        }
    }

    fn check_request(
        &self,
        offset: DeviceOffsetInBytes,
        length_in_bytes: u64,
    ) -> Result<(), BlockDeviceError> {
        let physical_block_bytes = u64::from(self.physical_block_size.0);
        if !offset.0.is_multiple_of(physical_block_bytes)
            || !length_in_bytes.is_multiple_of(physical_block_bytes)
        {
            return Err(BlockDeviceError::Unaligned {
                offset,
                length: length_in_bytes,
                physical_block_size: self.physical_block_size.0,
            });
        }
        match offset.0.checked_add(length_in_bytes) {
            Some(end) if end <= self.size_in_bytes => Ok(()),
            Some(_) | None => Err(BlockDeviceError::OutOfRange {
                offset,
                length: length_in_bytes,
                device_size: self.size_in_bytes,
            }),
        }
    }

    fn zero_range_in_the_top_layer(&mut self, offset: DeviceOffsetInBytes, length: u64) {
        self.top_layer.written.zero_fill(offset, length);
        let first_sector = offset.0 / SECTOR_BYTES;
        self.top_layer
            .zeroed_sector_ranges
            .push((first_sector, first_sector + length / SECTOR_BYTES));
    }
}

impl BlockDevice for ForkedDevice {
    fn read_at(
        &self,
        offset: DeviceOffsetInBytes,
        buffer: &mut [u8],
    ) -> Result<(), BlockDeviceError> {
        let length_in_bytes = u64::try_from(buffer.len()).expect("读长装得进 u64");
        self.check_request(offset, length_in_bytes)?;
        buffer.fill(0);
        let sector_bytes = usize::try_from(SECTOR_BYTES).expect("扇区宽装得进 usize");
        let first_sector = offset.0 / SECTOR_BYTES;
        let end_sector = first_sector + length_in_bytes / SECTOR_BYTES;
        let layers_bottom_up = self
            .sealed_layers
            .iter()
            .map(|layer| &**layer)
            .chain(std::iter::once(&self.top_layer));
        for layer in layers_bottom_up {
            for (zero_start, zero_end) in &layer.zeroed_sector_ranges {
                let from = (*zero_start).max(first_sector);
                let to = (*zero_end).min(end_sector);
                if from < to {
                    let from_byte =
                        usize::try_from(from - first_sector).expect("下标") * sector_bytes;
                    let to_byte = usize::try_from(to - first_sector).expect("下标") * sector_bytes;
                    buffer[from_byte..to_byte].fill(0);
                }
            }
            for (sector, bytes) in layer
                .written
                .written_sectors()
                .range(first_sector..end_sector)
            {
                let start = usize::try_from(sector - first_sector).expect("下标") * sector_bytes;
                buffer[start..start + sector_bytes].copy_from_slice(bytes);
            }
        }
        Ok(())
    }

    fn write_at(
        &mut self,
        offset: DeviceOffsetInBytes,
        bytes: &[u8],
        _durability: WriteDurability,
    ) -> Result<(), BlockDeviceError> {
        self.check_request(offset, u64::try_from(bytes.len()).expect("写长装得进 u64"))?;
        self.top_layer.written.write(offset, bytes);
        Ok(())
    }

    fn write_zeroes_at(
        &mut self,
        offset: DeviceOffsetInBytes,
        length: u64,
    ) -> Result<(), BlockDeviceError> {
        self.check_request(offset, length)?;
        self.zero_range_in_the_top_layer(offset, length);
        Ok(())
    }

    fn discard_at(
        &mut self,
        offset: DeviceOffsetInBytes,
        length: u64,
    ) -> Result<DiscardOutcome, BlockDeviceError> {
        self.check_request(offset, length)?;
        self.zero_range_in_the_top_layer(offset, length);
        Ok(DiscardOutcome::Applied)
    }

    fn barrier(&mut self) -> Result<(), BlockDeviceError> {
        Ok(())
    }

    fn probe_physical_block_size(&self) -> PhysicalBlockSizeInBytes {
        self.physical_block_size
    }

    fn size_in_bytes(&self) -> u64 {
        self.size_in_bytes
    }

    /// 各层写过的扇区里落在记录槽边界上的偏移（多给不要紧，漏一个才错；清零过的扇区多给一个无害）。
    fn journal_record_offsets_hint(
        &self,
        ring_start: DeviceOffsetInBytes,
        ring_bytes: u64,
    ) -> Option<Vec<DeviceOffsetInBytes>> {
        let mut offsets: BTreeSet<u64> = BTreeSet::new();
        for layer in self
            .sealed_layers
            .iter()
            .map(|layer| &**layer)
            .chain(std::iter::once(&self.top_layer))
        {
            for sector in layer.written.written_sectors_in(ring_start, ring_bytes) {
                let offset = sector * SECTOR_BYTES;
                if (offset - ring_start.0).is_multiple_of(JOURNAL_RECORD_BYTES) {
                    offsets.insert(offset);
                }
            }
        }
        Some(offsets.into_iter().map(DeviceOffsetInBytes).collect())
    }
}

/// 池的几何：两块盘的字节数。都用默认 768 MiB 环、S = 8（跑前登记第五节「几何」）。
#[derive(Clone, Copy, Debug)]
struct Geometry {
    name: &'static str,
    device_bytes: [u64; 2],
    journal_ring_bytes: u64,
}

/// G默认：2 × 4 GiB。
const GEOMETRY_DEFAULT: Geometry = Geometry {
    name: "default",
    device_bytes: [4 * GIBIBYTE, 4 * GIBIBYTE],
    journal_ring_bytes: DEFAULT_JOURNAL_RING_BYTES_LOCAL,
};
/// G窄：2 × (3 GiB + 33 槽)（单元区 146465 槽 = 2288 个整段加 33 槽）。
const GEOMETRY_NARROW: Geometry = Geometry {
    name: "narrow",
    device_bytes: [
        3 * GIBIBYTE + 33 * SLOT_BYTES_LOCAL,
        3 * GIBIBYTE + 33 * SLOT_BYTES_LOCAL,
    ],
    journal_ring_bytes: DEFAULT_JOURNAL_RING_BYTES_LOCAL,
};
/// G不等大：4 GiB + (3 GiB + 33 槽)。
const GEOMETRY_UNEQUAL: Geometry = Geometry {
    name: "unequal",
    device_bytes: [4 * GIBIBYTE, 3 * GIBIBYTE + 33 * SLOT_BYTES_LOCAL],
    journal_ring_bytes: DEFAULT_JOURNAL_RING_BYTES_LOCAL,
};
/// G宽：2 × 16 GiB（稀疏设备，只给 F11 量扫描随单元区的增长）。
const GEOMETRY_WIDE: Geometry = Geometry {
    name: "wide",
    device_bytes: [16 * GIBIBYTE, 16 * GIBIBYTE],
    journal_ring_bytes: DEFAULT_JOURNAL_RING_BYTES_LOCAL,
};

/// 补一个小单元区的几何（登记修订 1）：2 × 432 MiB，96 MiB 环（在飞记录上限 8192 条，装得下一次写 7000 个单元的 7000 多条记录），
/// 单元区起点 U = 1024 + 6144 = 7168（段边界上），单元区 20480 槽 = 320 个整段。K1 在这里只有几十个节点，
/// 中央映射树的删插才是 ckpt_cost 里的大头——大几何里 K1 的几百个节点把这一项盖住了。
const GEOMETRY_SMALL: Geometry = Geometry {
    name: "small_unit_area",
    device_bytes: [SMALL_GEOMETRY_DEVICE_BYTES, SMALL_GEOMETRY_DEVICE_BYTES],
    journal_ring_bytes: SMALL_GEOMETRY_JOURNAL_RING_BYTES,
};
const SMALL_GEOMETRY_JOURNAL_RING_BYTES: u64 = 96 << 20;
const SMALL_GEOMETRY_UNIT_AREA_START_SLOT: u64 = 1024 + (96 << 20) / SLOT_BYTES_LOCAL;
const SMALL_GEOMETRY_UNIT_AREA_SLOTS: u64 = 20480;
const SMALL_GEOMETRY_DEVICE_BYTES: u64 =
    (SMALL_GEOMETRY_UNIT_AREA_START_SLOT + SMALL_GEOMETRY_UNIT_AREA_SLOTS) * SLOT_BYTES_LOCAL;

fn make_filesystem_parameters(geometry: Geometry) -> MakeFilesystemParameters {
    MakeFilesystemParameters {
        filesystem_identifier: E181_FILESYSTEM_IDENTIFIER,
        region_devices: [DeviceIdentity(0), DeviceIdentity(1), DeviceIdentity(0)],
        geometry: SystemImmutableSizes {
            physical_block_size: 512,
            minimum_input_output_bytes: 512,
            fixed_structure_slot_spacing: 4096,
            journal_ring_bytes: geometry.journal_ring_bytes,
            root_ring_slots_per_region: RootRingSlotsPerRegion::AT_MAKE_FILESYSTEM,
        },
    }
}

fn first_file_content() -> Vec<u8> {
    (0..FIRST_FILE_BYTES)
        .map(|index| u8::try_from(index % 251).expect("小于 256"))
        .collect()
}

/// 一个池：盘、分配器、现行那一版、实例代号。拷贝它（[`Pool::fork`]）不拷盘上的扇区。
struct Pool {
    parameters: MakeFilesystemParameters,
    devices: Vec<(DeviceIdentity, ForkedDevice)>,
    allocator: PoolAllocator,
    current: TransactionOutput,
    instance: InstanceGeneration,
}

impl Pool {
    /// 封存各盘的顶层（之后用 [`Pool::fork_of_a_sealed_base`] 在几个线程里各拷一份）。
    fn seal(&mut self) {
        for (_identity, device) in &mut self.devices {
            drop(device.fork());
        }
    }

    /// 底已经封存过（[`Pool::seal`]）时不需要 `&mut` 的拷贝。
    fn fork_of_a_sealed_base(&self) -> Pool {
        Pool {
            parameters: self.parameters.clone(),
            devices: self
                .devices
                .iter()
                .map(|(identity, device)| (*identity, device.fork_of_a_sealed_base()))
                .collect(),
            allocator: self.allocator.clone(),
            current: self.current.clone(),
            instance: self.instance,
        }
    }

    fn fork(&mut self) -> Pool {
        Pool {
            parameters: self.parameters.clone(),
            devices: self
                .devices
                .iter_mut()
                .map(|(identity, device)| (*identity, device.fork()))
                .collect(),
            allocator: self.allocator.clone(),
            current: self.current.clone(),
            instance: self.instance,
        }
    }
}

/// mkfs、取号、暖机、新池新建文件（3000 字节）都在同一个进程里做完的池（没经过可写挂载）：A3、A4 钉的 H1 就是它。
fn build_pool_after_the_first_file(
    geometry: Geometry,
    capacities: CodeTwoTreeNodeCapacities,
) -> Pool {
    let parameters = make_filesystem_parameters(geometry);
    let mut devices: Vec<(DeviceIdentity, ForkedDevice)> = geometry
        .device_bytes
        .iter()
        .enumerate()
        .map(|(device_number, device_bytes)| {
            (
                DeviceIdentity(u32::try_from(device_number).expect("盘号")),
                ForkedDevice::empty(*device_bytes),
            )
        })
        .collect();
    let genesis = make_filesystem(&parameters, &mut devices).expect("mkfs 收下这个几何");
    let mut allocator = allocator_after_make_filesystem(&parameters, &devices, &genesis);
    let content = first_file_content();
    let (instance, first_file) = {
        let mut writer = PoolWriter::new(&parameters, &mut devices);
        if let CodeTwoTreeNodeCapacities::CappedForTests { .. } = capacities {
            writer.set_code_two_tree_node_capacities(capacities);
        }
        let instance = acquire_instance(&mut writer).expect("取号");
        let warmed = warm_up(&mut writer, &genesis.root, instance).expect("暖机");
        let first_file = publish_first_file(
            &mut writer,
            &mut allocator,
            warmed.roots.last().expect("暖机两代根"),
            FirstFile {
                content: &content,
                write_time_seconds: FIXED_WRITE_TIME_SECONDS,
            },
            instance,
            &warmed.last_record_bytes,
        )
        .expect("新池新建文件");
        (instance, first_file)
    };
    Pool {
        parameters,
        devices,
        allocator,
        current: first_file,
        instance,
    }
}

/// 一次发布在中央映射树里多删多插的普通分配条目数（F9 的 P1 在 ckpt_cost 的删插数上加的两项）：
/// 删 = 上一版有、这一版没有的普通分配角色的映射 key（这次换下的数据单元、被重写的 extent 与 inode 单元），
/// 插 = 这一版有、上一版没有的（这次新写的同几类单元）。按两版 `mapped_units` 里普通分配角色的 key 集合作差，
/// 不读发布内部，所以与「走固定点之前就定下了的普通分配那一半」是同一个数（普通分配的角色在固定点之前就定，
/// `transaction::prepare_the_version_publish`）。
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
struct OrdinaryMappingChanges {
    deleted: u64,
    inserted: u64,
}

fn ordinary_mapping_keys(version: &TransactionOutput) -> BTreeSet<&[u8]> {
    version
        .mapped_units
        .iter()
        .filter(|(role, _key)| {
            space_budget_of_role(*role) == SpaceBudgetOfARole::OrdinaryAllocation
        })
        .map(|(_role, key)| key.as_slice())
        .collect()
}

fn ordinary_mapping_changes(
    previous: &TransactionOutput,
    output: &TransactionOutput,
) -> OrdinaryMappingChanges {
    let keys_before = ordinary_mapping_keys(previous);
    let keys_after = ordinary_mapping_keys(output);
    OrdinaryMappingChanges {
        deleted: u64::try_from(keys_before.difference(&keys_after).count()).expect("条数"),
        inserted: u64::try_from(keys_after.difference(&keys_before).count()).expect("条数"),
    }
}

/// 本地重算的 ckpt_cost（D28（挂载期承诺量） 已定项 4，登记 5.1 节）：分配记录树按 K1、中央映射树按逐层路径式子、
/// 记账树与谱系、退役两项取核心层同名公开函数（读了的共用项，B3 逐版对拍它们），加树表 1。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct LocalCheckpointCost {
    allocation_record_tree_nodes: u64,
    central_mapping_tree_nodes: u64,
    accounting_tree_nodes: u64,
    lineage_container_cow: u64,
    retirement_container_cow: u64,
    total: u64,
}

/// 分配记录树根之下每个层级一个位置的槽数：812 × 169^层级。
fn allocation_record_tree_span_in_slots_at_level(level: u8) -> u64 {
    let mut span = ALLOCATION_RECORD_TREE_LEAF_SLOTS_LOCAL;
    for _climbed in 0..level {
        span *= ALLOCATION_RECORD_TREE_INTERNAL_FANOUT_LOCAL;
    }
    span
}

/// K1（D28（挂载期承诺量） 已定项 4）：1 + Σ_盘 Σ_{层级 0 … 根层级 − 1} 这块盘单元区 [起点, 起点 + 槽数) 在这一层罩到的位置数。
fn local_allocation_record_tree_nodes(root_level: u8, unit_areas: &[(u64, u64)]) -> u64 {
    let mut nodes = 1;
    for (first_slot, slot_count) in unit_areas {
        let last_slot = first_slot + slot_count - 1;
        for level in 0..root_level {
            let span = allocation_record_tree_span_in_slots_at_level(level);
            nodes += last_slot / span - first_slot / span + 1;
        }
    }
    nodes
}

/// 中央映射树每层有几个节点（自叶到根）。
fn local_nodes_at_each_level(version: &TransactionOutput) -> Vec<u64> {
    let height =
        version.height_read_from_the_root_node_header(MultiLevelCodeTwoTree::CentralMapping);
    let mut counts = vec![0_u64; usize::try_from(height).expect("树高装得进 usize")];
    for node in version.central_mapping_tree.shape.nodes() {
        counts[usize::from(node.position.level)] += 1;
    }
    assert!(
        counts.iter().all(|count| *count > 0),
        "中央映射树每一层至少一个节点：{counts:?}"
    );
    counts
}

/// 一层上至多切几次：原有的节点第一次切至少要加 1 条，切出来的新节点再切一次至少要加 `additions_per_split_of_a_fresh_node` 条。
fn local_splits_at_one_level_at_most(
    pre_existing_nodes: u64,
    additions: u64,
    additions_per_split_of_a_fresh_node: u64,
) -> u64 {
    let first_splits = pre_existing_nodes.min(additions);
    first_splits + (additions - first_splits) / additions_per_split_of_a_fresh_node
}

/// 中央映射树一次发布至多重写的节点数：每层 min(这一层的节点数, 删 + 插) 加这一层切出来的，根切开就长高。
fn local_central_mapping_tree_nodes_rewritten_at_most(
    nodes_at_each_level: &[u64],
    entries_before: u64,
    deleted: u64,
    inserted: u64,
    capacity: CodeTwoTreeNodeCapacity,
) -> u64 {
    let leaf_entries = u64::try_from(capacity.leaf_entries).expect("容量");
    let internal_entries = u64::try_from(capacity.internal_entries).expect("容量");
    let leaf_additions = leaf_entries.div_ceil(2);
    let internal_additions = internal_entries.div_ceil(2);
    let changed_paths = deleted + inserted;
    let mut rewritten = 0;
    let mut splits_below = 0;
    for (level, nodes_here) in nodes_at_each_level.iter().copied().enumerate() {
        let splits_here = if level == 0 {
            if entries_before + inserted <= leaf_entries {
                0
            } else {
                local_splits_at_one_level_at_most(nodes_here, inserted, leaf_additions)
            }
        } else if nodes_at_each_level[level - 1] + splits_below <= internal_entries {
            0
        } else {
            local_splits_at_one_level_at_most(nodes_here, splits_below, internal_additions)
        };
        rewritten += nodes_here.min(changed_paths) + splits_here;
        splits_below = splits_here;
    }
    while splits_below > 0 {
        let children_of_the_new_root = splits_below + 1;
        let splits_of_the_new_level = if children_of_the_new_root <= internal_entries {
            0
        } else {
            (splits_below - 1) / internal_additions
        };
        rewritten += 1 + splits_of_the_new_level;
        splits_below = splits_of_the_new_level;
    }
    rewritten
}

fn local_checkpoint_cost(
    previous: &TransactionOutput,
    allocator: &PoolAllocator,
    capacities: CodeTwoTreeNodeCapacities,
    extra: OrdinaryMappingChanges,
) -> LocalCheckpointCost {
    let root_level = u8::try_from(
        previous
            .position_addressed_tree_heights_read_from_the_root_node_headers()
            .allocation_record_tree
            - 1,
    )
    .expect("层级装得进 u8");
    let unit_areas: Vec<(u64, u64)> = allocator
        .devices
        .iter()
        .map(|device_map| {
            (
                device_map.unit_area_start().slot().0,
                device_map.unit_area_slots(),
            )
        })
        .collect();
    let allocation_record_tree_nodes = local_allocation_record_tree_nodes(root_level, &unit_areas);
    let accounting_nodes_before =
        u64::try_from(previous.accounting_tree.node_count()).expect("节点数");
    let accounting_nodes_after = accounting_tree_nodes_after_the_publish_built_on(
        previous,
        capacities.of_tree(MultiLevelCodeTwoTree::Accounting),
    );
    let lineage_levels = previous.lineage_tree_internal_levels();
    let retirement_levels = previous.retirement_tree_internal_levels();
    let lineage_entries = central_mapping_entries_of_one_lineage_container_cow(lineage_levels);
    let retirement_entries =
        central_mapping_entries_of_one_retirement_container_cow(retirement_levels);
    let deleted = allocation_record_tree_nodes
        + accounting_nodes_before
        + lineage_entries
        + retirement_entries
        + extra.deleted;
    let inserted = allocation_record_tree_nodes
        + accounting_nodes_after
        + lineage_entries
        + retirement_entries
        + extra.inserted;
    let entries_before = u64::try_from(
        previous.mapped_units.len() + previous.mapping_entries_kept_for_the_snapshots.len(),
    )
    .expect("条数");
    let central_mapping_tree_nodes = local_central_mapping_tree_nodes_rewritten_at_most(
        &local_nodes_at_each_level(previous),
        entries_before,
        deleted,
        inserted,
        capacities.of_tree(MultiLevelCodeTwoTree::CentralMapping),
    );
    let lineage_container_cow = checkpoint_cost_of_one_lineage_container_cow(lineage_levels).0;
    let retirement_container_cow =
        checkpoint_cost_of_one_retirement_container_cow(retirement_levels).0;
    const TREE_TABLE_UNITS_PER_PUBLISH: u64 = 1;
    LocalCheckpointCost {
        allocation_record_tree_nodes,
        central_mapping_tree_nodes,
        accounting_tree_nodes: accounting_nodes_after,
        lineage_container_cow,
        retirement_container_cow,
        total: allocation_record_tree_nodes
            + central_mapping_tree_nodes
            + accounting_nodes_after
            + lineage_container_cow
            + retirement_container_cow
            + TREE_TABLE_UNITS_PER_PUBLISH,
    }
}

/// 核心层的 ckpt_cost（落得下那一判今天拿它顶固定点）。
fn core_checkpoint_cost(
    previous: &TransactionOutput,
    allocator: &PoolAllocator,
    capacities: CodeTwoTreeNodeCapacities,
) -> u64 {
    checkpoint_cost_of_the_version_to_build_on_with_node_capacities(
        Some(previous),
        allocator,
        capacities,
    )
    .0
}

/// 这次发布在每块盘上新占的固定点槽数，两条独立的路各算一遍：
/// 路径一「角色」：这次真写出的角色里不是普通分配的那几个，各自跨度相加（每个单元落每一块盘，所以逐盘相同）；
/// 路径二「记录」（登记第一节「读法写死」）：这一版分配记录里分配代等于这次 txg、没释放的记录，逐盘加跨度，再减普通分配角色的槽数。
#[derive(Clone, Debug, PartialEq, Eq)]
struct FixedPointAccount {
    by_roles: u64,
    by_records_per_device: Vec<(DeviceIdentity, u64)>,
    ordinary_slots: u64,
    instance_switch_slots: u64,
}

fn fixed_point_account(
    output: &TransactionOutput,
    devices: &[DeviceIdentity],
) -> FixedPointAccount {
    let mut by_roles = 0;
    let mut ordinary_slots = 0;
    let mut instance_switch_slots = 0;
    for role in &output.rewritten {
        match space_budget_of_role(*role) {
            SpaceBudgetOfARole::CheckpointReservePool => by_roles += role.span_slots(),
            SpaceBudgetOfARole::OrdinaryAllocation => ordinary_slots += role.span_slots(),
            SpaceBudgetOfARole::InstanceSwitchReserve => instance_switch_slots += role.span_slots(),
        }
    }
    let txg = output.root.checkpoint_txg;
    let by_records_per_device = devices
        .iter()
        .map(|device| {
            let allocated_in_this_publish: u64 = output
                .allocation_records
                .iter()
                .filter(|record| {
                    record.device == *device && record.generation == txg && !record.is_released
                })
                .map(|record| u64::from(record.span_slots))
                .sum();
            (
                *device,
                allocated_in_this_publish - ordinary_slots - instance_switch_slots,
            )
        })
        .collect();
    FixedPointAccount {
        by_roles,
        by_records_per_device,
        ordinary_slots,
        instance_switch_slots,
    }
}

/// 一次被拒的写是哪一道拒的（登记 5.1 节「记每次是哪一道拒」）。先拒那一道（落得下那一判）与式子那一判报同一个成员，
/// 装置不拆开，带着不够的盘的读数原样。
#[derive(Clone, Debug, PartialEq, Eq)]
enum RefusalClass {
    /// `shortfall_bytes`：不够的盘里需求减可用的最大值（字节，每块盘一份），填满函数按它折成要少写几个单元。
    SpaceAdmission {
        short_devices: String,
        shortfall_bytes: i128,
    },
    FixedPointPlacement {
        unit: String,
        refusal: String,
    },
    /// `journal_records_over_the_limit`：被拒的是这次发布的 journal 记录数超过上限时，超出的条数（条数减上限）；别的拒因是 `None`。
    Other {
        description: String,
        journal_records_over_the_limit: Option<u64>,
    },
}

impl RefusalClass {
    fn of(error: &PublishError) -> Self {
        match error {
            PublishError::SpaceAdmissionRefused(refused) => RefusalClass::SpaceAdmission {
                short_devices: format!("{:?}", refused.short_devices),
                shortfall_bytes: refused
                    .short_devices
                    .iter()
                    .map(|short| i128::from(short.demand.0) - short.available.0)
                    .max()
                    .unwrap_or(0),
            },
            PublishError::PlacementRefused { unit, refusal } => RefusalClass::FixedPointPlacement {
                unit: format!("{unit:?}"),
                refusal: format!("{refusal:?}"),
            },
            PublishError::JournalRecordsOfThePublishExceedTheLimit {
                records_of_the_publish,
                record_limit,
            } => RefusalClass::Other {
                description: format!("{error:?}"),
                journal_records_over_the_limit: Some(
                    records_of_the_publish.saturating_sub(*record_limit),
                ),
            },
            PublishError::EmptyClusterSegmentsBelowTheFloor(_)
            | PublishError::MultiLevelCodeTwoTreeRefused { .. }
            | PublishError::ReleaseNotInMapping { .. }
            | PublishError::ReleaseTargetNotAllocated { .. }
            | PublishError::ReleaseTargetAlreadyReleased { .. }
            | PublishError::ReleaseTargetLocationsOnDifferentSlots { .. }
            | PublishError::ReleaseSpanMismatch { .. }
            | PublishError::MappingEntryNarrowerThanItsFieldTable { .. }
            | PublishError::MappingEntryLocationOnADeviceOutsideThePool { .. }
            | PublishError::MappingEntryLocationsOnTheSameDevice { .. }
            | PublishError::ContentExceedsDataUnit { .. }
            | PublishError::AllocationRecordTreeRewriteSetDidNotSettle { .. }
            | PublishError::InodeTreeWriteRefused(_)
            | PublishError::FirstFileVersionDoesNotFollowTheVersionItBuildsOn { .. }
            | PublishError::FirstFileVersionOnAVersionThatAlreadyHasAFile { .. }
            | PublishError::TreeTableOfTheVersionToBuildOnUnreadable { .. }
            | PublishError::TreeIdentifierWatermarkLeavesNoRoomForTheFileVersionTrees(_)
            | PublishError::PublishFrozenAfterAWriteFailureIsNotResentYet { .. }
            | PublishError::NextCheckpointTxgPastTheTopOfItsRange { .. }
            | PublishError::NextContainerNumberPastTheTopOfItsRange(_)
            | PublishError::WriteToAnInodeBeingDeleted { .. }
            | PublishError::WriteToAnInodeThatIsNotInTheTree { .. }
            | PublishError::DataWriteToAnInodeBornAfterTheFirstFileWasDeletedIsOutsideTheFirstVersion { .. }
            | PublishError::UnlinkOfAnInodeThatIsNotALiveRecord { .. }
            | PublishError::DeletedInodesQueueRefused(_)
            | PublishError::SnapshotCreationRefused(_)
            | PublishError::SnapshotDestructionRefused(_)
            | PublishError::DeadlistEntryNotReleasable { .. }
            | PublishError::TruncationRefused(_)
            | PublishError::BlockDevice(_)
            | PublishError::ReadFailedWhileRehearsingTheContinuationBeforeAcquisition(_)
            | PublishError::ReadBackFailingSelfVerificationWhileRehearsingTheContinuationBeforeAcquisition(
                _,
            ) => RefusalClass::Other {
                description: format!("{error:?}"),
                journal_records_over_the_limit: None,
            },
        }
    }

    /// 产物行里写的拒因细节：不够的盘的读数、落点那一道的单元与拒因、别的拒因的原文。
    fn detail(&self) -> String {
        match self {
            RefusalClass::SpaceAdmission { short_devices, .. } => short_devices.clone(),
            RefusalClass::FixedPointPlacement { unit, refusal } => format!("{unit} {refusal}"),
            RefusalClass::Other { description, .. } => description.clone(),
        }
    }

    fn name(&self) -> &'static str {
        match self {
            RefusalClass::SpaceAdmission { .. } => "space_admission_refused",
            RefusalClass::FixedPointPlacement { .. } => "placement_refused_in_the_fixed_point",
            RefusalClass::Other { .. } => "other_refusal",
        }
    }

    /// 空间准入那一道拒时，不够的盘里需求减可用的最大值（字节）；别的拒因没有缺口读数。
    fn space_shortfall_bytes(&self) -> Option<i128> {
        match self {
            RefusalClass::SpaceAdmission {
                shortfall_bytes, ..
            } => Some(*shortfall_bytes),
            RefusalClass::FixedPointPlacement { .. } | RefusalClass::Other { .. } => None,
        }
    }

    /// 哪一道拒、缺口多少（没有空格，可以放进产物行的字段）：标准错误上被拒那一行与边查找的产物行都用它。
    fn gate_and_gap_text(&self) -> String {
        match self {
            RefusalClass::SpaceAdmission {
                shortfall_bytes, ..
            } => {
                format!("{}(shortfall_bytes={shortfall_bytes})", self.name())
            }
            RefusalClass::FixedPointPlacement { .. } => self.name().to_string(),
            RefusalClass::Other {
                journal_records_over_the_limit: Some(surplus),
                ..
            } => format!("{}(journal_records_over_the_limit={surplus})", self.name()),
            RefusalClass::Other {
                journal_records_over_the_limit: None,
                ..
            } => self.name().to_string(),
        }
    }
}

/// 一块盘上缺 `shortfall_bytes` 字节，要少写几个数据单元才补上：一个单元在一块盘上占两槽（32768 字节），向上取整；
/// 缺口读数不是正数时按一字节算。被拒之后往下找（[`units_to_try_after_a_refusal`]）与二分按缺口预测边（[`RefusalsSeen::predicted_edge`]）共用。
fn units_a_shortfall_is_worth(shortfall_bytes: i128) -> i128 {
    let bytes_per_unit = i128::try_from(DATA_UNIT_BYTES_LOCAL).expect("单元字节装得进 i128");
    (shortfall_bytes.max(1) + bytes_per_unit - 1) / bytes_per_unit
}

/// 一次发成的写的账。
#[derive(Clone, Debug)]
struct PublishedWrite {
    units: u64,
    height_before: u64,
    height_after: u64,
    true_fixed_point: u64,
    true_fixed_point_by_records: Vec<(DeviceIdentity, u64)>,
    checkpoint_cost: u64,
    local_checkpoint_cost: u64,
    checkpoint_cost_with_ordinary_mapping_changes: u64,
    ordinary: OrdinaryMappingChanges,
}

impl PublishedWrite {
    /// Q1：P1 − 真写的固定点。
    fn checkpoint_cost_with_ordinary_mapping_changes_minus_true(&self) -> i64 {
        signed_difference(
            self.checkpoint_cost_with_ordinary_mapping_changes,
            self.true_fixed_point,
        )
    }
    /// Q2：ckpt_cost − 真写的固定点（今天那一判的余量）。
    fn checkpoint_cost_minus_true(&self) -> i64 {
        signed_difference(self.checkpoint_cost, self.true_fixed_point)
    }
    /// Q3：P1 − ckpt_cost（P1 比今天多扣的槽）。
    fn checkpoint_cost_with_ordinary_mapping_changes_minus_checkpoint_cost(&self) -> i64 {
        signed_difference(
            self.checkpoint_cost_with_ordinary_mapping_changes,
            self.checkpoint_cost,
        )
    }
}

fn signed_difference(left: u64, right: u64) -> i64 {
    i64::try_from(left).expect("槽数装得进 i64") - i64::try_from(right).expect("槽数装得进 i64")
}

#[derive(Clone, Debug)]
enum WriteOutcome {
    Published(Box<PublishedWrite>),
    Refused(RefusalClass),
}

/// 整份内容的缓冲：一次写 n 个数据单元就是前 n × 32768 字节，缓冲只长不缩。
struct ContentBuffer {
    bytes: Vec<u8>,
}

impl ContentBuffer {
    fn new() -> Self {
        Self { bytes: Vec::new() }
    }

    fn of_units(&mut self, units: usize) -> &[u8] {
        let wanted = units * DATA_UNIT_BYTES_LOCAL;
        if self.bytes.len() < wanted {
            let already = self.bytes.len();
            self.bytes.extend(
                (already..wanted)
                    .map(|index| u8::try_from((index / 7 + index % 251) % 256).expect("小于 256")),
            );
        }
        &self.bytes[..wanted]
    }
}

fn device_identities(pool: &Pool) -> Vec<DeviceIdentity> {
    pool.devices.iter().map(|(identity, _)| *identity).collect()
}

/// 接着现行那一版整份换写成 `units` 个数据单元（`publish_sequential_write`），发成时账里带三个量：
/// 真写的固定点、核心层 ckpt_cost、本地 P1；每次先拿本地 ckpt_cost（普通分配删插为 0）与核心层的同名量对拍
/// （B3，对不上返回 `Err`，由调用方停机）。
fn write_units_and_account(
    pool: &mut Pool,
    units: usize,
    capacities: CodeTwoTreeNodeCapacities,
    content: &mut ContentBuffer,
) -> Result<WriteOutcome, String> {
    write_units_from_bytes_and_account(pool, units, capacities, content.of_units(units))
}

/// 标准错误上一次写的进度行（只给人看，不进产物）：单元数、发布用时，发成，或被拒加拒因（哪一道拒、缺口多少）。
fn progress_line_of_a_write(
    units: usize,
    publish_seconds: f64,
    published: &Result<TransactionOutput, PublishError>,
) -> String {
    let result = match published {
        Ok(_) => "发成".to_string(),
        Err(error) => format!("被拒，拒因 {}", RefusalClass::of(error).gate_and_gap_text()),
    };
    format!("[e181] 一次写 {units} 个单元的发布用了 {publish_seconds:.1} 秒，{result}")
}

/// 同 [`write_units_and_account`]，内容由调用方给（几个线程共用同一份只读的内容）：`content_bytes` 恰是 `units` 个数据单元。
fn write_units_from_bytes_and_account(
    pool: &mut Pool,
    units: usize,
    capacities: CodeTwoTreeNodeCapacities,
    content_bytes: &[u8],
) -> Result<WriteOutcome, String> {
    assert_eq!(content_bytes.len(), units * DATA_UNIT_BYTES_LOCAL);
    let checkpoint_cost = core_checkpoint_cost(&pool.current, &pool.allocator, capacities);
    let local_without_ordinary = local_checkpoint_cost(
        &pool.current,
        &pool.allocator,
        capacities,
        OrdinaryMappingChanges::default(),
    );
    if local_without_ordinary.total != checkpoint_cost {
        return Err(format!(
            "本地 checkpoint 代价 {local_without_ordinary:?} 与核心层的 {checkpoint_cost} 对不上（本地与核心层的 checkpoint 代价对拍停机）"
        ));
    }
    let height_before = pool
        .current
        .height_read_from_the_root_node_header(MultiLevelCodeTwoTree::CentralMapping);
    let started = Instant::now();
    let published = {
        let mut writer = PoolWriter::new(&pool.parameters, &mut pool.devices);
        if let CodeTwoTreeNodeCapacities::CappedForTests { .. } = capacities {
            writer.set_code_two_tree_node_capacities(capacities);
        }
        publish_sequential_write(
            &mut writer,
            &mut pool.allocator,
            &pool.current,
            FirstFile {
                content: content_bytes,
                write_time_seconds: FIXED_WRITE_TIME_SECONDS,
            },
            pool.instance,
        )
    };
    if units >= LOGGED_WRITE_UNITS {
        eprintln!(
            "{}",
            progress_line_of_a_write(units, started.elapsed().as_secs_f64(), &published)
        );
    }
    match published {
        Err(error) => Ok(WriteOutcome::Refused(RefusalClass::of(&error))),
        Ok(output) => {
            let devices = device_identities(pool);
            let account = fixed_point_account(&output, &devices);
            let ordinary = ordinary_mapping_changes(&pool.current, &output);
            let local_with_ordinary =
                local_checkpoint_cost(&pool.current, &pool.allocator, capacities, ordinary);
            let report = PublishedWrite {
                units: u64::try_from(units).expect("单元数"),
                height_before,
                height_after: output
                    .height_read_from_the_root_node_header(MultiLevelCodeTwoTree::CentralMapping),
                true_fixed_point: account.by_roles,
                true_fixed_point_by_records: account.by_records_per_device,
                checkpoint_cost,
                local_checkpoint_cost: local_without_ordinary.total,
                checkpoint_cost_with_ordinary_mapping_changes: local_with_ordinary.total,
                ordinary,
            };
            pool.current = output;
            Ok(WriteOutcome::Published(Box::new(report)))
        }
    }
}

/// 一类写（历史、几何、中央映射树的高）的 Q1、Q2、Q3 汇总：次数、最小值、最大值、为负的次数。
#[derive(Clone, Debug, Default)]
struct WriteTally {
    writes: u64,
    checkpoint_cost_with_ordinary_mapping_changes_minus_true_minimum: Option<i64>,
    checkpoint_cost_with_ordinary_mapping_changes_below_true_writes: u64,
    checkpoint_cost_minus_true_minimum: Option<i64>,
    checkpoint_cost_below_true_writes: u64,
    checkpoint_cost_with_ordinary_mapping_changes_minus_checkpoint_cost_maximum: Option<i64>,
    checkpoint_cost_with_ordinary_mapping_changes_minus_checkpoint_cost_minimum: Option<i64>,
    true_fixed_point_maximum: u64,
}

impl WriteTally {
    fn add(&mut self, write: &PublishedWrite) {
        self.writes += 1;
        let checkpoint_cost_with_ordinary_mapping_changes_minus_true =
            write.checkpoint_cost_with_ordinary_mapping_changes_minus_true();
        let checkpoint_cost_minus_true = write.checkpoint_cost_minus_true();
        let checkpoint_cost_with_ordinary_mapping_changes_minus_checkpoint_cost =
            write.checkpoint_cost_with_ordinary_mapping_changes_minus_checkpoint_cost();
        self.checkpoint_cost_with_ordinary_mapping_changes_minus_true_minimum = Some(
            self.checkpoint_cost_with_ordinary_mapping_changes_minus_true_minimum
                .map_or(
                    checkpoint_cost_with_ordinary_mapping_changes_minus_true,
                    |old| old.min(checkpoint_cost_with_ordinary_mapping_changes_minus_true),
                ),
        );
        self.checkpoint_cost_minus_true_minimum = Some(
            self.checkpoint_cost_minus_true_minimum
                .map_or(checkpoint_cost_minus_true, |old| {
                    old.min(checkpoint_cost_minus_true)
                }),
        );
        self.checkpoint_cost_with_ordinary_mapping_changes_minus_checkpoint_cost_maximum = Some(
            self.checkpoint_cost_with_ordinary_mapping_changes_minus_checkpoint_cost_maximum
                .map_or(
                    checkpoint_cost_with_ordinary_mapping_changes_minus_checkpoint_cost,
                    |old| {
                        old.max(checkpoint_cost_with_ordinary_mapping_changes_minus_checkpoint_cost)
                    },
                ),
        );
        self.checkpoint_cost_with_ordinary_mapping_changes_minus_checkpoint_cost_minimum = Some(
            self.checkpoint_cost_with_ordinary_mapping_changes_minus_checkpoint_cost_minimum
                .map_or(
                    checkpoint_cost_with_ordinary_mapping_changes_minus_checkpoint_cost,
                    |old| {
                        old.min(checkpoint_cost_with_ordinary_mapping_changes_minus_checkpoint_cost)
                    },
                ),
        );
        self.checkpoint_cost_with_ordinary_mapping_changes_below_true_writes +=
            u64::from(checkpoint_cost_with_ordinary_mapping_changes_minus_true < 0);
        self.checkpoint_cost_below_true_writes += u64::from(checkpoint_cost_minus_true < 0);
        self.true_fixed_point_maximum = self.true_fixed_point_maximum.max(write.true_fixed_point);
    }
}

/// 判据（登记第六节 Q1）的门槛：最小值 ≥ 0 判「P1 是这批写的上界」。
const MINIMUM_SLOTS_FOR_AN_UPPER_BOUND: i64 = 0;
/// 判据（登记第六节 Q5）的门槛：G10 为真的状态数 ≥ 1 判「可达」。
const MINIMUM_SLOTS_ENOUGH_AND_ALIGNED_PAIRS_SHORT_STATES_FOR_REACHABLE: u64 = 1;

fn checkpoint_cost_with_ordinary_mapping_changes_is_an_upper_bound_given_threshold(
    tally: &WriteTally,
    threshold: i64,
) -> Option<bool> {
    tally
        .checkpoint_cost_with_ordinary_mapping_changes_minus_true_minimum
        .map(|minimum| minimum >= threshold)
}

fn checkpoint_cost_with_ordinary_mapping_changes_is_an_upper_bound_on_every_write(
    tally: &WriteTally,
) -> Option<bool> {
    checkpoint_cost_with_ordinary_mapping_changes_is_an_upper_bound_given_threshold(
        tally,
        MINIMUM_SLOTS_FOR_AN_UPPER_BOUND,
    )
}

fn slots_enough_and_aligned_pairs_short_is_reachable_given_threshold(
    slots_enough_and_aligned_pairs_short_states: u64,
    threshold: u64,
) -> bool {
    slots_enough_and_aligned_pairs_short_states >= threshold
}

fn text_of(value: Option<i64>) -> String {
    value.map_or_else(|| "none".to_string(), |number| number.to_string())
}

/// F9 的全部观测：逐类汇总、被拒的写、停机原因。
#[derive(Default)]
struct FixedPointUpperBoundCollector {
    tallies: std::collections::BTreeMap<String, WriteTally>,
    refusal_counts: std::collections::BTreeMap<&'static str, u64>,
    stop_reasons: Vec<String>,
}

impl FixedPointUpperBoundCollector {
    fn record_write(&mut self, history: &str, geometry: &str, step: usize, write: &PublishedWrite) {
        let class = format!(
            "{history}/{geometry}/height_before_{}_after_{}",
            write.height_before, write.height_after
        );
        self.tallies.entry(class).or_default().add(write);
        self.tallies
            .entry(format!("{history}/{geometry}/all"))
            .or_default()
            .add(write);
        self.tallies
            .entry("all".to_string())
            .or_default()
            .add(write);
        let records_text: Vec<String> = write
            .true_fixed_point_by_records
            .iter()
            .map(|(device, slots)| format!("{}:{slots}", device.0))
            .collect();
        emit(&format!(
            "name=fixed_point_upper_bound_write history={history} geometry={geometry} step={step} units={} height_before={} height_after={} true_fixed_point={} true_fixed_point_by_records={} checkpoint_cost={} local_checkpoint_cost={} checkpoint_cost_with_ordinary_mapping_changes={} ordinary_deleted={} ordinary_inserted={} checkpoint_cost_with_ordinary_mapping_changes_minus_true={} checkpoint_cost_minus_true={} checkpoint_cost_with_ordinary_mapping_changes_minus_checkpoint_cost={}",
            write.units,
            write.height_before,
            write.height_after,
            write.true_fixed_point,
            records_text.join(","),
            write.checkpoint_cost,
            write.local_checkpoint_cost,
            write.checkpoint_cost_with_ordinary_mapping_changes,
            write.ordinary.deleted,
            write.ordinary.inserted,
            write.checkpoint_cost_with_ordinary_mapping_changes_minus_true(),
            write.checkpoint_cost_minus_true(),
            write.checkpoint_cost_with_ordinary_mapping_changes_minus_checkpoint_cost(),
        ));
    }

    fn record_refusal(
        &mut self,
        history: &str,
        geometry: &str,
        step: usize,
        units: usize,
        refusal: &RefusalClass,
    ) {
        *self.refusal_counts.entry(refusal.name()).or_default() += 1;
        let detail = refusal.detail();
        emit(&format!(
            "name=fixed_point_upper_bound_refused history={history} geometry={geometry} step={step} units={units} refusal={} detail={:?}",
            refusal.name(),
            detail
        ));
    }

    fn emit_tallies(&self) {
        for (class, tally) in &self.tallies {
            emit(&format!(
                "name=fixed_point_upper_bound_tally class={class} writes={} checkpoint_cost_with_ordinary_mapping_changes_minus_true_minimum={} checkpoint_cost_with_ordinary_mapping_changes_minus_true_negative_writes={} checkpoint_cost_minus_true_minimum={} checkpoint_cost_minus_true_negative_writes={} checkpoint_cost_with_ordinary_mapping_changes_minus_checkpoint_cost_minimum={} checkpoint_cost_with_ordinary_mapping_changes_minus_checkpoint_cost_maximum={} true_fixed_point_maximum={}",
                tally.writes,
                text_of(tally.checkpoint_cost_with_ordinary_mapping_changes_minus_true_minimum),
                tally.checkpoint_cost_with_ordinary_mapping_changes_below_true_writes,
                text_of(tally.checkpoint_cost_minus_true_minimum),
                tally.checkpoint_cost_below_true_writes,
                text_of(tally.checkpoint_cost_with_ordinary_mapping_changes_minus_checkpoint_cost_minimum),
                text_of(tally.checkpoint_cost_with_ordinary_mapping_changes_minus_checkpoint_cost_maximum),
                tally.true_fixed_point_maximum,
            ));
        }
    }
}

/// 一次写：发成的进汇总、被拒的记一笔；B3 对拍不等返回 `Err`（停机）。
#[allow(
    clippy::too_many_arguments,
    reason = "八个参数各是一样独立的东西（汇总、历史名、几何名、步号、池、单元数、节点容量、内容缓冲），收成结构体只是换个地方再摊开；编码纪律：参数不按个数限"
)]
fn run_one_write(
    collector: &mut FixedPointUpperBoundCollector,
    history: &str,
    geometry: &str,
    step: usize,
    pool: &mut Pool,
    units: usize,
    capacities: CodeTwoTreeNodeCapacities,
    content: &mut ContentBuffer,
) -> Result<WriteOutcome, String> {
    let outcome = write_units_and_account(pool, units, capacities, content)?;
    match &outcome {
        WriteOutcome::Published(write) => collector.record_write(history, geometry, step, write),
        WriteOutcome::Refused(refusal) => {
            collector.record_refusal(history, geometry, step, units, refusal);
        }
    }
    Ok(outcome)
}

/// 登记 5.1 节 HF9-产品容量：第一个文件 1 单元，接五档 n 各一次换写（每档都从同一个 1 单元的版本起，在拷贝上做）。
const PRODUCT_CAPACITY_UNIT_COUNTS: [usize; 5] = [1000, 2500, 4000, 5500, 7000];

/// 登记 5.1 节 HF9-三层压小的容量：叶 8 条、内部 4 个孩子（记账树与中央映射树同一档）。
const SMALL_TREE_NODE_CAPACITY: CodeTwoTreeNodeCapacity = CodeTwoTreeNodeCapacity {
    leaf_entries: 8,
    internal_entries: 4,
};

const SMALL_TREE_NODE_CAPACITIES: CodeTwoTreeNodeCapacities =
    CodeTwoTreeNodeCapacities::CappedForTests {
        accounting: SMALL_TREE_NODE_CAPACITY,
        central_mapping: SMALL_TREE_NODE_CAPACITY,
    };

fn run_product_capacity_history(
    collector: &mut FixedPointUpperBoundCollector,
    geometry: Geometry,
    content: &mut ContentBuffer,
) -> Result<(), String> {
    let mut base =
        build_pool_after_the_first_file(geometry, CodeTwoTreeNodeCapacities::FromTheNodeFormat);
    for (step, units) in PRODUCT_CAPACITY_UNIT_COUNTS.iter().enumerate() {
        let mut fork = base.fork();
        run_one_write(
            collector,
            "product_capacity_history",
            geometry.name,
            step,
            &mut fork,
            *units,
            CodeTwoTreeNodeCapacities::FromTheNodeFormat,
            content,
        )?;
    }
    Ok(())
}

/// 登记 5.1 节 HF9-三层：压小容量，从 1 单元起每次把文件换写得长 `step_units` 个单元，
/// 连写到中央映射树高 ≥ 4（层级 + 1 = 4）为止，取过 2 层与 3 层的每一次写。
/// `step_units` = 64 是登记的那一臂；另补一条臂 `step_units` = 4（登记修订里写明：64 一步只经过 2、3 层各一两次）。
fn run_three_level_history(
    collector: &mut FixedPointUpperBoundCollector,
    history: &str,
    geometry: Geometry,
    step_units: usize,
    content: &mut ContentBuffer,
) -> Result<usize, String> {
    const MOST_STEPS: usize = 400;
    const STOP_AT_HEIGHT: u64 = 4;
    let mut pool = build_pool_after_the_first_file(geometry, SMALL_TREE_NODE_CAPACITIES);
    let mut units = 1;
    let mut writes = 0;
    for step in 0..MOST_STEPS {
        units += step_units;
        match run_one_write(
            collector,
            history,
            geometry.name,
            step,
            &mut pool,
            units,
            SMALL_TREE_NODE_CAPACITIES,
            content,
        )? {
            WriteOutcome::Published(write) => {
                writes += 1;
                if write.height_after >= STOP_AT_HEIGHT {
                    return Ok(writes);
                }
            }
            WriteOutcome::Refused(_) => return Ok(writes),
        }
    }
    Ok(writes)
}

/// 落得下那一判放行了吗：发成，或被拒在固定点（落得下那一判与式子那一判都过了）。
fn passed_the_landing_check_and_the_formula(outcome: &WriteOutcome) -> bool {
    match outcome {
        WriteOutcome::Published(_) => true,
        WriteOutcome::Refused(RefusalClass::FixedPointPlacement { .. }) => true,
        WriteOutcome::Refused(RefusalClass::SpaceAdmission { .. } | RefusalClass::Other { .. }) => {
            false
        }
    }
}

/// 一块盘上数据单元落得下的槽对数（段外，两槽都没挡）。
fn data_unit_pairs_outside_the_clusters_of(allocator: &PoolAllocator, device_index: usize) -> u64 {
    allocator.devices[device_index]
        .unblocked_slot_pairs_outside_the_cluster_segments(allocator.cluster_segments())
}

fn data_unit_pairs_outside_the_clusters(pool: &Pool, device_index: usize) -> u64 {
    data_unit_pairs_outside_the_clusters_of(&pool.allocator, device_index)
}

/// HF9-边 的结果：边界与窗口里的次数。
#[derive(Clone, Debug, Default)]
struct EdgeReport {
    pairs_outside_the_clusters: u64,
    filled_units: u64,
    boundary_units: u64,
    /// 找下界的结果：在填过的底上观测到放行的单元数（下界一个也没找到时是 `None`）。
    lower_bound_units: Option<u64>,
    /// 找下界时没放行的试写，每条是「单元数:拒因」（拒因见 [`RefusalClass::gate_and_gap_text`]）。
    lower_bound_refused_tries: Vec<String>,
    bisection_rounds: u64,
    bisection_points_tried: u64,
    bisection_points_passed: u64,
    /// 边没被观测夹住（下界放行、下界 + 1 被拒）时的原因；找到了是 `None`。没找到时窗口不试，窗口里的各项都是 0。
    edge_not_found_reason: Option<String>,
    window_first: u64,
    window_last: u64,
    published: u64,
    refused_by_space_admission: u64,
    refused_in_the_fixed_point: u64,
    refused_other: u64,
    /// 窗口里前 64 个（n* − 63 … n*）之中，落得下放行而固定点拒的次数。
    fixed_point_refusals_in_the_first_sixty_four: u64,
    fixed_point_refused_units: Vec<u64>,
    space_refused_units_first_and_last: (Option<u64>, Option<u64>),
}

/// 在 `base` 的几份拷贝上并行试这些单元数的整份换写（每份各自从 `base` 起，互不影响）：交回每个单元数的结局，按单元数升序。
/// 线程数见 [`probe_thread_count`]。
fn probe_writes_in_parallel(
    base: &Pool,
    unit_counts: &[usize],
    content_bytes: &[u8],
) -> Result<Vec<(usize, WriteOutcome)>, String> {
    let threads = probe_thread_count();
    let next_index = std::sync::atomic::AtomicUsize::new(0);
    let finished: std::sync::Mutex<Vec<(usize, Result<WriteOutcome, String>)>> =
        std::sync::Mutex::new(Vec::new());
    std::thread::scope(|scope| {
        for _thread in 0..threads.min(unit_counts.len()) {
            scope.spawn(|| loop {
                let index = next_index.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                let Some(units) = unit_counts.get(index).copied() else {
                    break;
                };
                let mut fork = base.fork_of_a_sealed_base();
                let outcome = write_units_from_bytes_and_account(
                    &mut fork,
                    units,
                    CodeTwoTreeNodeCapacities::FromTheNodeFormat,
                    &content_bytes[..units * DATA_UNIT_BYTES_LOCAL],
                );
                finished.lock().expect("锁").push((units, outcome));
            });
        }
    });
    let mut results = finished.into_inner().expect("锁");
    results.sort_by_key(|(units, _)| *units);
    results
        .into_iter()
        .map(|(units, outcome)| outcome.map(|outcome| (units, outcome)))
        .collect()
}

/// 同时试写的线程数：设了 `E181_PROBE_THREADS` 照它，没设取每台默认重活线程数（`singlefs_harness::heavy_threads`）。
/// 每个线程各拷一份底，一份 3 GiB 级盘上 3 万多个单元的写，拷贝加发布的账要占 5 GiB 上下，所以长跑脚本按内存上限显式给这个变量。
fn probe_thread_count() -> usize {
    let thread_cap_value =
        std::env::var(singlefs_harness::heavy_threads::THREAD_CAP_ENVIRONMENT_VARIABLE).ok();
    singlefs_harness::heavy_threads::own_variable_or_default_heavy_threads(
        "E181_PROBE_THREADS",
        std::env::var("E181_PROBE_THREADS").ok().as_deref(),
        thread_cap_value.as_deref(),
    )
    .get()
}

/// 找下界最多试几次（每次都是一整份换写，几十到几百秒）。
const LOWER_BOUND_PROBES_AT_MOST: usize = 40;

/// [`find_a_lower_bound_that_passes`] 的结局。
#[derive(Debug)]
enum LowerBoundSearch {
    /// `units` 是放行的那个数（发成，或被拒在固定点）；`refused_before` 是它之前没放行的试写（单元数与拒因，按试的次序，单元数严格递减）。
    Found {
        units: usize,
        refused_before: Vec<(usize, RefusalClass)>,
    },
    /// 从填过的单元数一路试到 1 个单元，或试满 [`LOWER_BOUND_PROBES_AT_MOST`] 次，都没有放行的。
    NotFound {
        reason: String,
        refused: Vec<(usize, RefusalClass)>,
    },
}

/// 在填过的底上找一个放行的单元数当多路二分的下界：先试 `filled_units`（它是在更小的底上发成的，在填过的底上要换掉一个同样大的文件，另算），
/// 被拒就按 [`units_to_try_after_a_refusal`] 往下找（空间准入那一道按缺口一步跳过去），直到放行；放行的定义与二分一致
/// （[`passed_the_landing_check_and_the_formula`]：发成，或被拒在固定点）。
fn find_a_lower_bound_that_passes(
    filled_units: usize,
    mut try_one_unit_count: impl FnMut(usize) -> Result<WriteOutcome, String>,
) -> Result<LowerBoundSearch, String> {
    assert!(filled_units >= 1, "填过的底至少有 1 个单元");
    let mut units = filled_units;
    let mut refused: Vec<(usize, RefusalClass)> = Vec::new();
    loop {
        let outcome = try_one_unit_count(units)?;
        if passed_the_landing_check_and_the_formula(&outcome) {
            return Ok(LowerBoundSearch::Found {
                units,
                refused_before: refused,
            });
        }
        let WriteOutcome::Refused(refusal) = outcome else {
            unreachable!("发成的写算放行，上面已经返回");
        };
        refused.push((units, refusal.clone()));
        if units == 1 {
            return Ok(LowerBoundSearch::NotFound {
                reason: format!(
                    "填过的底上，从 {filled_units} 个单元一路试到 1 个单元的整份换写都没放行"
                ),
                refused,
            });
        }
        if refused.len() >= LOWER_BOUND_PROBES_AT_MOST {
            return Ok(LowerBoundSearch::NotFound {
                reason: format!(
                    "填过的底上试了 {LOWER_BOUND_PROBES_AT_MOST} 次整份换写还没放行，最后试的是 {units} 个单元"
                ),
                refused,
            });
        }
        let next =
            units_to_try_after_a_refusal(&refusal, u64::try_from(units).expect("单元数装得进 u64"));
        units = usize::try_from(next).expect("单元数装得进 usize");
    }
}

/// 二分已经见过的被拒试写：单元数最小的（已知的上界），和空间准入拒的里单元数最小的（带缺口）。
#[derive(Debug, Default)]
struct RefusalsSeen {
    lowest_refused_units: Option<usize>,
    nearest_space_refusal: Option<(usize, i128)>,
}

impl RefusalsSeen {
    fn note(&mut self, units: usize, refusal: &RefusalClass) {
        self.lowest_refused_units = Some(
            self.lowest_refused_units
                .map_or(units, |lowest| lowest.min(units)),
        );
        if let Some(shortfall_bytes) = refusal.space_shortfall_bytes() {
            if self
                .nearest_space_refusal
                .is_none_or(|(nearest, _)| units < nearest)
            {
                self.nearest_space_refusal = Some((units, shortfall_bytes));
            }
        }
    }

    /// 按最近一次空间准入拒的缺口预测的边：被拒的单元数减去缺口折成的单元数（需求随单元数线性降、每个单元在一块盘上两槽时，
    /// 这就是空间那一道放行的最大单元数）。只是预测：二分只信观测到的放行与被拒，预测错了照样收敛。
    fn predicted_edge(&self) -> Option<usize> {
        let (units, shortfall_bytes) = self.nearest_space_refusal?;
        let worth = usize::try_from(units_a_shortfall_is_worth(shortfall_bytes)).ok()?;
        units.checked_sub(worth)
    }
}

/// 多路二分一轮试哪些点（都严格落在 `(low, high)` 里，所以每轮至少让一端移动）：有预测的边时先取预测的边与它上面一格
/// （预测对了，一轮就夹住），余下的线程把 `(low, high)` 等分；没有预测就只等分。
fn points_of_a_bisection_round(
    low: usize,
    high: usize,
    threads: usize,
    predicted_edge: Option<usize>,
) -> Vec<usize> {
    assert!(high - low > 1, "区间里至少还有一个点可试");
    let gap = high - low;
    let mut points: BTreeSet<usize> = BTreeSet::new();
    if let Some(edge) = predicted_edge {
        for candidate in [edge, edge + 1] {
            if candidate > low && candidate < high && points.len() < threads {
                points.insert(candidate);
            }
        }
    }
    for index in 1..=threads {
        if points.len() >= threads {
            break;
        }
        let point = low + gap * index / (threads + 1);
        if point > low && point < high {
            points.insert(point);
        }
    }
    if points.is_empty() {
        points.insert(low + gap / 2);
    }
    points.into_iter().collect()
}

/// 边查找的结局。
#[derive(Debug, PartialEq, Eq)]
enum EdgeSearchEnd {
    /// 边界 n*：`boundary_units` 放行（观测到的），`boundary_units + 1` 被拒（观测到的）。
    Edge { boundary_units: usize },
    /// 没有被观测到的放行与被拒夹住的边。
    NotFound { reason: String },
}

#[derive(Debug)]
struct EdgeSearch {
    end: EdgeSearchEnd,
    /// 找到的下界（没找到时是 `None`）与它之前没放行的试写。
    lower_bound_units: Option<usize>,
    lower_bound_refused_before: Vec<(usize, RefusalClass)>,
    bisection_rounds: u64,
    bisection_points_tried: u64,
    bisection_points_passed: u64,
}

/// HF9-边 找 n*（落得下与式子两道都放行的最大单元数）：先在填过的底上找一个观测到放行的下界（[`find_a_lower_bound_that_passes`]），
/// 再在 `(下界, 上界)` 里多路二分，上界取找下界时见过的最小的被拒单元数（没有就取 `upper_sentinel`），每轮的点见 [`points_of_a_bisection_round`]。
/// 收尾要求边被观测夹住：下界放行、下界 + 1 被拒；一直放行到 `upper_sentinel` 都没有被拒的，或被拒的最小单元数不是下界 + 1（放行与被拒次序不一致），
/// 都不交出边界，交 `NotFound`。
fn search_for_the_edge(
    filled_units: usize,
    upper_sentinel: usize,
    threads: usize,
    mut try_one_unit_count: impl FnMut(usize) -> Result<WriteOutcome, String>,
    mut try_unit_counts: impl FnMut(&[usize]) -> Result<Vec<(usize, WriteOutcome)>, String>,
) -> Result<EdgeSearch, String> {
    assert!(threads >= 1, "至少一个线程");
    let (lower_bound_units, refused_before) =
        match find_a_lower_bound_that_passes(filled_units, &mut try_one_unit_count)? {
            LowerBoundSearch::Found {
                units,
                refused_before,
            } => (units, refused_before),
            LowerBoundSearch::NotFound { reason, refused } => {
                return Ok(EdgeSearch {
                    end: EdgeSearchEnd::NotFound { reason },
                    lower_bound_units: None,
                    lower_bound_refused_before: refused,
                    bisection_rounds: 0,
                    bisection_points_tried: 0,
                    bisection_points_passed: 0,
                });
            }
        };
    let mut seen = RefusalsSeen::default();
    for (units, refusal) in &refused_before {
        seen.note(*units, refusal);
    }
    let mut low = lower_bound_units;
    let mut high = seen
        .lowest_refused_units
        .map_or(upper_sentinel, |units| units.min(upper_sentinel));
    assert!(high > low, "上界 {high} 要比下界 {low} 大");
    let mut rounds = 0_u64;
    let mut points_tried = 0_u64;
    let mut points_passed = 0_u64;
    while high - low > 1 {
        rounds += 1;
        let points = points_of_a_bisection_round(low, high, threads, seen.predicted_edge());
        for (units, outcome) in try_unit_counts(&points)? {
            points_tried += 1;
            if passed_the_landing_check_and_the_formula(&outcome) {
                points_passed += 1;
                low = low.max(units);
            } else {
                let WriteOutcome::Refused(refusal) = &outcome else {
                    unreachable!("发成的写算放行，上面已经归了放行");
                };
                high = high.min(units);
                seen.note(units, refusal);
            }
        }
        high = high.max(low + 1);
    }
    let end = if seen.lowest_refused_units == Some(low + 1) {
        EdgeSearchEnd::Edge {
            boundary_units: low,
        }
    } else if let Some(lowest) = seen.lowest_refused_units {
        EdgeSearchEnd::NotFound {
            reason: format!(
                "被拒的试写里最小的是 {lowest} 个单元，不是放行的下界 {low} 之上一格：放行与被拒在下界附近次序不一致"
            ),
        }
    } else {
        EdgeSearchEnd::NotFound {
            reason: format!("一直放行到上限 {upper_sentinel} 个单元都没有被拒的试写"),
        }
    };
    Ok(EdgeSearch {
        end,
        lower_bound_units: Some(lower_bound_units),
        lower_bound_refused_before: refused_before,
        bisection_rounds: rounds,
        bisection_points_tried: points_tried,
        bisection_points_passed: points_passed,
    })
}

/// 在封存的底的一份拷贝上试一个单元数的整份换写。
fn probe_one_write_on_the_sealed_base(
    base: &Pool,
    units: usize,
    content_bytes: &[u8],
) -> Result<WriteOutcome, String> {
    let mut outcomes = probe_writes_in_parallel(base, &[units], content_bytes)?;
    let (probed_units, outcome) = outcomes
        .pop()
        .ok_or_else(|| format!("试 {units} 个单元的整份换写没有结局"))?;
    assert_eq!(probed_units, units, "交回的结局是被试的那个单元数的");
    Ok(outcome)
}

/// 登记 5.1 节 HF9-边（G窄）：先换写到「段外槽对数的一半」的九成，再在落得下那一判刚放行的 n 附近逐单元试，
/// 记每次是哪一道拒。边界 n* = 落得下与式子两道都放行的最大 n（多路二分，在拷贝上试，被拒的不改池）；
/// 试的范围取 [n* − 63, n* + 64]（登记写「往上逐单元试 64 次，一次都没有时把范围加倍再试」，装置直接取加倍后的超集，
/// 前 64 个单独记数）。
fn run_edge_history(
    collector: &mut FixedPointUpperBoundCollector,
    geometry: Geometry,
    content: &mut ContentBuffer,
) -> Result<EdgeReport, String> {
    let mut base =
        build_pool_after_the_first_file(geometry, CodeTwoTreeNodeCapacities::FromTheNodeFormat);
    let pairs = data_unit_pairs_outside_the_clusters(&base, 0);
    let filled = pairs / 2 * 9 / 10;
    let filled_units = usize::try_from(filled).expect("单元数");
    match run_one_write(
        collector,
        "edge_history_fill",
        geometry.name,
        0,
        &mut base,
        filled_units,
        CodeTwoTreeNodeCapacities::FromTheNodeFormat,
        content,
    )? {
        WriteOutcome::Published(_) => {}
        WriteOutcome::Refused(refusal) => {
            return Err(format!("填到九成那一次就被拒：{refusal:?}"));
        }
    }
    base.seal();
    let upper_sentinel = usize::try_from(pairs).expect("槽对数") + 1024;
    let content_bytes: &[u8] = content.of_units(upper_sentinel);
    let search = search_for_the_edge(
        filled_units,
        upper_sentinel,
        probe_thread_count(),
        |units| probe_one_write_on_the_sealed_base(&base, units, content_bytes),
        |points| probe_writes_in_parallel(&base, points, content_bytes),
    )?;
    let mut report = EdgeReport {
        pairs_outside_the_clusters: pairs,
        filled_units: filled,
        lower_bound_units: search
            .lower_bound_units
            .map(|units| u64::try_from(units).expect("单元数")),
        lower_bound_refused_tries: search
            .lower_bound_refused_before
            .iter()
            .map(|(units, refusal)| format!("{units}:{}", refusal.gate_and_gap_text()))
            .collect(),
        bisection_rounds: search.bisection_rounds,
        bisection_points_tried: search.bisection_points_tried,
        bisection_points_passed: search.bisection_points_passed,
        ..EdgeReport::default()
    };
    let boundary = match search.end {
        EdgeSearchEnd::Edge { boundary_units } => boundary_units,
        EdgeSearchEnd::NotFound { reason } => {
            report.edge_not_found_reason = Some(reason);
            return Ok(report);
        }
    };
    let window_first = boundary.saturating_sub(63).max(1);
    let window_last = boundary + 64;
    report.boundary_units = u64::try_from(boundary).expect("单元数");
    report.window_first = u64::try_from(window_first).expect("单元数");
    report.window_last = u64::try_from(window_last).expect("单元数");
    let window: Vec<usize> = (window_first..=window_last).collect();
    for (units, outcome) in probe_writes_in_parallel(&base, &window, content_bytes)? {
        let units_u64 = u64::try_from(units).expect("单元数");
        match &outcome {
            WriteOutcome::Published(write) => {
                collector.record_write("edge_history", geometry.name, units, write);
                report.published += 1;
            }
            WriteOutcome::Refused(refusal) => {
                collector.record_refusal("edge_history", geometry.name, units, units, refusal);
            }
        }
        match outcome {
            WriteOutcome::Published(_) => {}
            WriteOutcome::Refused(RefusalClass::SpaceAdmission { .. }) => {
                report.refused_by_space_admission += 1;
                let (first, _last) = report.space_refused_units_first_and_last;
                report.space_refused_units_first_and_last =
                    (first.or(Some(units_u64)), Some(units_u64));
            }
            WriteOutcome::Refused(RefusalClass::FixedPointPlacement { .. }) => {
                report.refused_in_the_fixed_point += 1;
                report.fixed_point_refused_units.push(units_u64);
                if units <= boundary {
                    report.fixed_point_refusals_in_the_first_sixty_four += 1;
                }
            }
            WriteOutcome::Refused(RefusalClass::Other { .. }) => report.refused_other += 1,
        }
    }
    Ok(report)
}

/// 每块盘上把最低的 `percent`% 个段外槽对标成已分配（只改空闲图，不写盘、不写分配记录）：
/// 扫描挂钟只读空闲图，所以拿它造「最低落点靠后」的状态，不必真写几 GiB 的数据（登记修订里写明代替了什么）。
fn mark_the_lowest_pairs_outside_the_clusters(allocator: &mut PoolAllocator, percent: u64) {
    let cluster_segments: BTreeSet<SlotNumber> = allocator.cluster_segments().clone();
    for device_map in &mut allocator.devices {
        let total_pairs =
            device_map.unblocked_slot_pairs_outside_the_cluster_segments(&cluster_segments);
        let wanted = total_pairs * percent / 100;
        let mut marked = 0;
        let mut candidate = device_map.unit_area_start().slot().0;
        let end = candidate + device_map.unit_area_slots();
        while marked < wanted && candidate + 1 < end {
            let inside_cluster = cluster_segments.iter().any(|segment| {
                candidate >= segment.0 && candidate < segment.0 + CLUSTER_SEGMENT_SLOTS_LOCAL
            });
            if !inside_cluster
                && device_map.is_free(SlotNumber(candidate))
                && device_map.is_free(SlotNumber(candidate + 1))
            {
                device_map.mark_allocated(SlotNumber(candidate), DATA_UNIT_SLOTS_LOCAL);
                marked += 1;
            }
            candidate += DATA_UNIT_SLOTS_LOCAL;
        }
        assert_eq!(marked, wanted, "单元区里装得下要标的槽对数");
    }
}

/// 一块盘上 `lowest_user_data_slot` 的挂钟：重复 `repetitions` 次，取中位数与最大值；扫过的候选数 = 从单元区起点按 2 槽步进到落点的步数加 1。
#[derive(Clone, Copy, Debug)]
struct ScanTiming {
    lowest_slot: Option<u64>,
    scanned_candidates: u64,
    median_nanoseconds: u128,
    maximum_nanoseconds: u128,
}

fn median_and_maximum(mut samples: Vec<u128>) -> (u128, u128) {
    samples.sort_unstable();
    (
        samples[samples.len() / 2],
        *samples.last().expect("至少一次"),
    )
}

fn time_the_lowest_user_data_slot(
    allocator: &PoolAllocator,
    device_index: usize,
    repetitions: usize,
) -> ScanTiming {
    let device_map = &allocator.devices[device_index];
    let cluster_segments = allocator.cluster_segments();
    let mut samples = Vec::with_capacity(repetitions);
    let mut answer = None;
    for _repetition in 0..repetitions {
        let started = Instant::now();
        answer = std::hint::black_box(
            device_map.lowest_user_data_slot(std::hint::black_box(cluster_segments)),
        );
        samples.push(started.elapsed().as_nanos());
    }
    let (median_nanoseconds, maximum_nanoseconds) = median_and_maximum(samples);
    let start = device_map.unit_area_start().slot().0;
    ScanTiming {
        lowest_slot: answer.map(|slot| slot.0),
        scanned_candidates: answer.map_or_else(
            || device_map.unit_area_slots() / DATA_UNIT_SLOTS_LOCAL,
            |slot| (slot.0 - start) / DATA_UNIT_SLOTS_LOCAL + 1,
        ),
        median_nanoseconds,
        maximum_nanoseconds,
    }
}

/// 落点那一道今天为一个数据单元已经花的挂钟：对分配器的拷贝调 `try_allocate_user_data`（每块盘一次 `lowest_user_data_slot` 加各盘同槽的核），
/// 拷贝不进计时。
fn time_the_placement_of_one_data_unit(
    allocator: &PoolAllocator,
    repetitions: usize,
) -> (u128, u128) {
    let generation = CheckpointTxg(1 << 40);
    let mut samples = Vec::with_capacity(repetitions);
    for _repetition in 0..repetitions {
        let mut copy = allocator.clone();
        let started = Instant::now();
        let placement = std::hint::black_box(copy.try_allocate_user_data(generation));
        samples.push(started.elapsed().as_nanos());
        assert!(placement.is_ok(), "这个状态下数据单元有落点：{placement:?}");
    }
    median_and_maximum(samples)
}

const SCAN_REPETITIONS: usize = 1000;
const PLACEMENT_REPETITIONS: usize = 200;

/// F11 的一个状态：几何、名字、每块盘的扫描挂钟、落点那一道的挂钟。
#[derive(Clone, Debug)]
struct ScanStateReport {
    geometry: &'static str,
    state: &'static str,
    unit_area_slots: u64,
    scans: Vec<ScanTiming>,
    placement_median_nanoseconds: u128,
}

fn measure_scan_state(
    geometry: &'static str,
    state: &'static str,
    allocator: &PoolAllocator,
) -> ScanStateReport {
    let scans: Vec<ScanTiming> = (0..allocator.devices.len())
        .map(|device_index| {
            time_the_lowest_user_data_slot(allocator, device_index, SCAN_REPETITIONS)
        })
        .collect();
    let (placement_median, placement_maximum) =
        time_the_placement_of_one_data_unit(allocator, PLACEMENT_REPETITIONS);
    for (device_index, scan) in scans.iter().enumerate() {
        emit(&format!(
            "name=per_device_lowest_slot_scan geometry={geometry} state={state} device={device_index} unit_area_slots={} lowest_slot={} scanned_candidates={} median_nanoseconds={} maximum_nanoseconds={} repetitions={SCAN_REPETITIONS}",
            allocator.devices[device_index].unit_area_slots(),
            scan.lowest_slot.map_or_else(|| "none".to_string(), |slot| slot.to_string()),
            scan.scanned_candidates,
            scan.median_nanoseconds,
            scan.maximum_nanoseconds,
        ));
    }
    emit(&format!(
        "name=per_device_lowest_slot_placement geometry={geometry} state={state} median_nanoseconds={placement_median} maximum_nanoseconds={placement_maximum} repetitions={PLACEMENT_REPETITIONS}"
    ));
    ScanStateReport {
        geometry,
        state,
        unit_area_slots: allocator.devices[0].unit_area_slots(),
        scans,
        placement_median_nanoseconds: placement_median,
    }
}

/// 登记 5.3 节：G窄、G默认、G宽 各三种状态——刚写完第一个文件、段外槽对填了一半、填了九成（这两种用空闲图标出来）。
/// 先拒那一道多出的挂钟（登记第六节「先拒那一道扫描」那一项）= 每块盘一次扫描，落点那一道已花的挂钟（登记第六节「落点那一道今天已花」那一项）= 一个数据单元的落点，两者之比逐状态报。
fn run_scan_growth(content: &mut ContentBuffer) -> Result<Vec<ScanStateReport>, String> {
    let mut reports = Vec::new();
    for geometry in [GEOMETRY_NARROW, GEOMETRY_DEFAULT, GEOMETRY_WIDE] {
        let mut pool =
            build_pool_after_the_first_file(geometry, CodeTwoTreeNodeCapacities::FromTheNodeFormat);
        reports.push(measure_scan_state(
            geometry.name,
            "after_the_first_file",
            &pool.allocator,
        ));
        for (state, percent) in [("half_marked", 50_u64), ("ninety_percent_marked", 90)] {
            let mut marked = pool.allocator.clone();
            mark_the_lowest_pairs_outside_the_clusters(&mut marked, percent);
            reports.push(measure_scan_state(geometry.name, state, &marked));
        }
        if geometry.name == GEOMETRY_NARROW.name {
            let pairs = data_unit_pairs_outside_the_clusters(&pool, 0);
            for (state, percent) in [("half_real", 50_u64), ("ninety_percent_real", 90)] {
                let units = usize::try_from(pairs * percent / 100).expect("单元数");
                let mut collector = FixedPointUpperBoundCollector::default();
                let mut fork = pool.fork();
                match run_one_write(
                    &mut collector,
                    "per_device_lowest_slot_real_fill",
                    geometry.name,
                    0,
                    &mut fork,
                    units,
                    CodeTwoTreeNodeCapacities::FromTheNodeFormat,
                    content,
                )? {
                    WriteOutcome::Published(_) => {
                        reports.push(measure_scan_state(geometry.name, state, &fork.allocator));
                    }
                    WriteOutcome::Refused(refusal) => {
                        emit(&format!(
                            "name=per_device_lowest_slot_real_fill_refused geometry={} state={state} units={units} refusal={:?}",
                            geometry.name, refusal
                        ));
                    }
                }
            }
        }
    }
    Ok(reports)
}

/// 会话层的池：盘加一次可写挂载之后的会话。拷贝它不拷盘上的扇区。
struct SessionPool {
    parameters: MakeFilesystemParameters,
    devices: Vec<(DeviceIdentity, ForkedDevice)>,
    session: MountedSession,
    /// 这个池里用 `NewInodes` 建出来、还没删的 inode 号（`DeleteACreatedInode` 从尾上取）。
    created_inodes: Vec<u64>,
    /// 下一次写进记录的时间秒：每次改动加 1，只为让记录不同。
    write_time_seconds: u64,
}

impl SessionPool {
    fn of_a_mount(
        parameters: MakeFilesystemParameters,
        mut devices: Vec<(DeviceIdentity, ForkedDevice)>,
        created_inodes: Vec<u64>,
        write_time_seconds: u64,
    ) -> Result<Self, String> {
        let mounted = mount_writable(&parameters, &mut devices)
            .map_err(|error| format!("可写挂载被拒：{error:?}"))?;
        let (_output, session) = MountedSession::of_the_mount(mounted);
        Ok(Self {
            parameters,
            devices,
            session,
            created_inodes,
            write_time_seconds,
        })
    }

    fn fork(&mut self) -> SessionPool {
        for (_identity, device) in &mut self.devices {
            drop(device.fork());
        }
        self.fork_of_a_sealed_base()
    }

    /// 底已经封存过（顶层空）时，不需要 `&mut` 的拷贝：几个线程各拷一份。
    fn fork_of_a_sealed_base(&self) -> SessionPool {
        SessionPool {
            parameters: self.parameters.clone(),
            devices: self
                .devices
                .iter()
                .map(|(identity, device)| (*identity, device.fork_of_a_sealed_base()))
                .collect(),
            session: MountedSession {
                allocator: self.session.allocator.clone(),
                current: self.session.current.clone(),
                instance: self.session.instance,
                shadow_ledger: self.session.shadow_ledger,
                parameters_and_device_table: self.session.parameters_and_device_table.clone(),
                compaction: self.session.compaction.clone(),
            },
            created_inodes: self.created_inodes.clone(),
            write_time_seconds: self.write_time_seconds,
        }
    }
}

/// 在 [`build_pool_after_the_first_file`] 的盘上可写挂载一次，交回会话层的池。
fn session_pool_of(pool: &mut Pool) -> Result<SessionPool, String> {
    let devices = pool
        .devices
        .iter_mut()
        .map(|(identity, device)| (*identity, device.fork()))
        .collect();
    SessionPool::of_a_mount(
        pool.parameters.clone(),
        devices,
        Vec::new(),
        FIXED_WRITE_TIME_SECONDS,
    )
}

/// 种子驱动的随机数（splitmix64）：每个种子一条自己的流，凭种子能单独复现。
struct SeededStream {
    state: u64,
}

impl SeededStream {
    fn of_seed(seed: u64) -> Self {
        Self {
            state: seed
                .wrapping_mul(0x9E37_79B9_7F4A_7C15)
                .wrapping_add(0x1234_5678_9ABC_DEF1),
        }
    }

    fn next_number(&mut self) -> u64 {
        self.state = self.state.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut mixed = self.state;
        mixed = (mixed ^ (mixed >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        mixed = (mixed ^ (mixed >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        mixed ^ (mixed >> 31)
    }

    fn below(&mut self, bound: u64) -> u64 {
        self.next_number() % bound
    }
}

/// 一步走法（登记 5.2 节：{NewInodes 1 / 8 / 64 个、删一个 NewInodes 建的空 inode、正常卸载再可写挂载}）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum WalkAction {
    CreateOneInode,
    CreateEightInodes,
    CreateSixtyFourInodes,
    DeleteACreatedInode,
    UnmountAndRemount,
}

const WALK_ACTIONS: [WalkAction; 5] = [
    WalkAction::CreateOneInode,
    WalkAction::CreateEightInodes,
    WalkAction::CreateSixtyFourInodes,
    WalkAction::DeleteACreatedInode,
    WalkAction::UnmountAndRemount,
];

impl WalkAction {
    fn name(self) -> &'static str {
        match self {
            WalkAction::CreateOneInode => "create_one_inode",
            WalkAction::CreateEightInodes => "create_eight_inodes",
            WalkAction::CreateSixtyFourInodes => "create_sixty_four_inodes",
            WalkAction::DeleteACreatedInode => "delete_a_created_inode",
            WalkAction::UnmountAndRemount => "unmount_and_remount",
        }
    }
}

/// 走法的结局：成员名（成了、或被拒的成员名）。
fn variant_name_of(debug_text: &str) -> String {
    debug_text
        .split(['(', ' ', '{'])
        .next()
        .unwrap_or("")
        .to_string()
}

fn apply_walk_action(state: &mut SessionPool, action: WalkAction) -> Result<String, String> {
    state.write_time_seconds += 1;
    let change = match action {
        WalkAction::CreateOneInode => Some(UserChange::NewInodes {
            count: 1,
            write_time_seconds: state.write_time_seconds,
        }),
        WalkAction::CreateEightInodes => Some(UserChange::NewInodes {
            count: 8,
            write_time_seconds: state.write_time_seconds,
        }),
        WalkAction::CreateSixtyFourInodes => Some(UserChange::NewInodes {
            count: 64,
            write_time_seconds: state.write_time_seconds,
        }),
        WalkAction::DeleteACreatedInode => {
            state.created_inodes.last().map(|inode| UserChange::Unlink {
                inode: InodeNumber(*inode),
                write_time_seconds: state.write_time_seconds,
            })
        }
        WalkAction::UnmountAndRemount => None,
    };
    match (action, change) {
        (WalkAction::UnmountAndRemount, _) => {
            unmount(
                &state.parameters,
                &mut state.devices,
                &mut state.session.allocator,
                &mut state.session.current,
                state.session.shadow_ledger,
            )
            .map_err(|error| format!("卸载被拒：{error:?}"))?;
            let devices = state
                .devices
                .iter_mut()
                .map(|(identity, device)| (*identity, device.fork()))
                .collect();
            let remounted = SessionPool::of_a_mount(
                state.parameters.clone(),
                devices,
                std::mem::take(&mut state.created_inodes),
                state.write_time_seconds,
            )?;
            *state = remounted;
            Ok("remounted".to_string())
        }
        (_, None) => Ok("nothing_to_delete".to_string()),
        (_, Some(change)) => {
            let first_new_inode = state
                .session
                .current
                .file_version()
                .map(TransactionOutput::inode_number_watermark);
            let published = state
                .session
                .publish_user_change(&mut state.devices, change);
            match published {
                Ok(_published) => {
                    match (action, first_new_inode) {
                        (WalkAction::CreateOneInode, Some(first)) => {
                            state.created_inodes.push(first)
                        }
                        (WalkAction::CreateEightInodes, Some(first)) => {
                            state.created_inodes.extend(first..first + 8);
                        }
                        (WalkAction::CreateSixtyFourInodes, Some(first)) => {
                            state.created_inodes.extend(first..first + 64);
                        }
                        (WalkAction::DeleteACreatedInode, _) => {
                            state.created_inodes.pop();
                        }
                        (
                            WalkAction::CreateOneInode
                            | WalkAction::CreateEightInodes
                            | WalkAction::CreateSixtyFourInodes
                            | WalkAction::UnmountAndRemount,
                            None,
                        )
                        | (WalkAction::UnmountAndRemount, Some(_)) => {}
                    }
                    Ok("published".to_string())
                }
                Err(refusal) => Ok(format!(
                    "refused_{}",
                    variant_name_of(&format!("{refusal:?}"))
                )),
            }
        }
    }
}

/// F10 的盘面谓词 G10 的读数（登记 5.2 节）：下一次 `NewInodes` 1 个那次发布要落的提交内生块——inode 叶容器（两槽）与 inode 树根（一槽）
/// 是普通分配那一半，加固定点按 ckpt_cost 的最坏数顶（`admission::units_of_a_publish_to_land`）——对照每块盘此刻没挡的槽与对齐的空槽对。
const NEW_INODE_ORDINARY_COMMIT_GENERATED_SLOTS: u64 = 3;
/// 同一次发布要落的两槽提交内生块个数：一片 inode 叶容器。
const NEW_INODE_TWO_SLOT_COMMIT_GENERATED_UNITS: u64 = 1;

#[derive(Clone, Debug, PartialEq, Eq)]
struct RoomReading {
    /// 每块盘对齐的空槽对数：起点偶数、两槽都没挡，聚簇段内外都算（空集合作段表，登记「读法写死」）。
    aligned_free_pairs: Vec<u64>,
    /// 每块盘没挡的槽数。
    unblocked_slots: Vec<u64>,
    needed_slots: u64,
    needed_pairs: u64,
}

impl RoomReading {
    fn of(state: &SessionPool) -> Self {
        let allocator = &state.session.allocator;
        let no_cluster_segments: BTreeSet<SlotNumber> = BTreeSet::new();
        let version = state
            .session
            .current
            .file_version()
            .expect("会话的现行那一版带文件");
        Self {
            aligned_free_pairs: allocator
                .devices
                .iter()
                .map(|device_map| {
                    device_map
                        .unblocked_slot_pairs_outside_the_cluster_segments(&no_cluster_segments)
                })
                .collect(),
            unblocked_slots: allocator
                .devices
                .iter()
                .map(|device_map| device_map.unblocked_slots())
                .collect(),
            needed_slots: NEW_INODE_ORDINARY_COMMIT_GENERATED_SLOTS
                + core_checkpoint_cost(
                    version,
                    allocator,
                    CodeTwoTreeNodeCapacities::FromTheNodeFormat,
                ),
            needed_pairs: NEW_INODE_TWO_SLOT_COMMIT_GENERATED_UNITS,
        }
    }

    /// 落得下那一判 ②（只数槽）放行：每块盘没挡的槽都够。
    fn slots_are_enough_on_every_device(&self) -> bool {
        self.unblocked_slots
            .iter()
            .all(|slots| *slots >= self.needed_slots)
    }

    /// 至少一块盘的对齐空槽对不够。
    fn aligned_pairs_are_short_on_some_device(&self) -> bool {
        self.aligned_free_pairs
            .iter()
            .any(|pairs| *pairs < self.needed_pairs)
    }

    /// G10 = 没挡的槽够 ∧ 对齐的空槽对不够。两个量各报各的，这里是登记写死的那一个合取。
    fn holds(&self) -> bool {
        self.slots_are_enough_on_every_device() && self.aligned_pairs_are_short_on_some_device()
    }
}

/// 在状态的拷贝上直接走发布路径发一次 `NewInodes` 1 个（今天那一臂 Q2）：落得下那一判与固定点各拒不拒。
fn probe_one_new_inode_today(state: &mut SessionPool) -> RefusalOrPublished {
    let mut copy = state.fork();
    let previous = copy
        .session
        .current
        .file_version()
        .expect("现行那一版带文件")
        .clone();
    let mut writer = PoolWriter::new(&copy.parameters, &mut copy.devices);
    match publish_new_inodes(
        &mut writer,
        &mut copy.session.allocator,
        &previous,
        1,
        FIXED_WRITE_TIME_SECONDS,
        copy.session.instance,
    ) {
        Ok(_output) => RefusalOrPublished::Published,
        Err(error) => RefusalOrPublished::Refused(RefusalClass::of(&error)),
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum RefusalOrPublished {
    Published,
    Refused(RefusalClass),
}

impl RefusalOrPublished {
    fn name(&self) -> &'static str {
        match self {
            RefusalOrPublished::Published => "published",
            RefusalOrPublished::Refused(refusal) => refusal.name(),
        }
    }
}

/// 在状态的拷贝上经会话发一次 `NewInodes` 1 个：会话推抬 F 之后用户看到的结局。
fn session_outcome_of_one_new_inode(state: &mut SessionPool) -> String {
    let mut copy = state.fork();
    let write_time_seconds = copy.write_time_seconds + 1;
    let published = copy.session.publish_user_change(
        &mut copy.devices,
        UserChange::NewInodes {
            count: 1,
            write_time_seconds,
        },
    );
    match published {
        Ok(done) => format!("published_after_{}_floor_raises", done.floor_raises.len()),
        Err(refusal) => format!("refused_{}", variant_name_of(&format!("{refusal:?}"))),
    }
}

/// 一个种子走完 300 步的账。
#[derive(Clone, Debug, Default)]
struct WalkTally {
    walks: u64,
    states: u64,
    slots_enough_and_aligned_pairs_short_states: u64,
    first_slots_enough_and_aligned_pairs_short_step: Option<u64>,
    aligned_pairs_peak: u64,
    aligned_pairs_positive_states: u64,
    aligned_pairs_final_minimum: Option<u64>,
    unblocked_slots_peak: u64,
    unblocked_slots_final_minimum: Option<u64>,
    slots_enough_states: u64,
    pairs_short_states: u64,
    action_refusals: std::collections::BTreeMap<String, u64>,
    slots_enough_and_aligned_pairs_short_today_outcomes:
        std::collections::BTreeMap<&'static str, u64>,
    slots_enough_and_aligned_pairs_short_session_outcomes: std::collections::BTreeMap<String, u64>,
    model_checks: u64,
    placement_refusals_with_slots_enough_and_aligned_pairs_short_false: u64,
    slots_enough_and_aligned_pairs_short_probes_that_published: u64,
}

impl WalkTally {
    fn merge(&mut self, other: &WalkTally) {
        self.walks += other.walks;
        self.states += other.states;
        self.slots_enough_and_aligned_pairs_short_states +=
            other.slots_enough_and_aligned_pairs_short_states;
        self.first_slots_enough_and_aligned_pairs_short_step = match (
            self.first_slots_enough_and_aligned_pairs_short_step,
            other.first_slots_enough_and_aligned_pairs_short_step,
        ) {
            (Some(left), Some(right)) => Some(left.min(right)),
            (Some(only), None) | (None, Some(only)) => Some(only),
            (None, None) => None,
        };
        self.aligned_pairs_peak = self.aligned_pairs_peak.max(other.aligned_pairs_peak);
        self.aligned_pairs_positive_states += other.aligned_pairs_positive_states;
        self.aligned_pairs_final_minimum = min_of_unsigned_options(
            self.aligned_pairs_final_minimum,
            other.aligned_pairs_final_minimum,
        );
        self.unblocked_slots_peak = self.unblocked_slots_peak.max(other.unblocked_slots_peak);
        self.unblocked_slots_final_minimum = min_of_unsigned_options(
            self.unblocked_slots_final_minimum,
            other.unblocked_slots_final_minimum,
        );
        self.slots_enough_states += other.slots_enough_states;
        self.pairs_short_states += other.pairs_short_states;
        for (action, count) in &other.action_refusals {
            *self.action_refusals.entry(action.clone()).or_default() += count;
        }
        for (outcome, count) in &other.slots_enough_and_aligned_pairs_short_today_outcomes {
            *self
                .slots_enough_and_aligned_pairs_short_today_outcomes
                .entry(outcome)
                .or_default() += count;
        }
        for (outcome, count) in &other.slots_enough_and_aligned_pairs_short_session_outcomes {
            *self
                .slots_enough_and_aligned_pairs_short_session_outcomes
                .entry(outcome.clone())
                .or_default() += count;
        }
        self.model_checks += other.model_checks;
        self.placement_refusals_with_slots_enough_and_aligned_pairs_short_false +=
            other.placement_refusals_with_slots_enough_and_aligned_pairs_short_false;
        self.slots_enough_and_aligned_pairs_short_probes_that_published +=
            other.slots_enough_and_aligned_pairs_short_probes_that_published;
    }
}

fn min_of_unsigned_options(left: Option<u64>, right: Option<u64>) -> Option<u64> {
    match (left, right) {
        (Some(left), Some(right)) => Some(left.min(right)),
        (Some(only), None) | (None, Some(only)) => Some(only),
        (None, None) => None,
    }
}

/// 每隔几步，在 G10 为假的状态上也发一次今天那一臂，核谓词没漏掉被拒在固定点的状态。
const MODEL_CHECK_EVERY_STEPS: u64 = 25;

fn observe_state(tally: &mut WalkTally, state: &mut SessionPool, step: u64) {
    let reading = RoomReading::of(state);
    tally.states += 1;
    let aligned_minimum = reading
        .aligned_free_pairs
        .iter()
        .copied()
        .min()
        .unwrap_or(0);
    let unblocked_minimum = reading.unblocked_slots.iter().copied().min().unwrap_or(0);
    tally.aligned_pairs_peak = tally.aligned_pairs_peak.max(aligned_minimum);
    tally.unblocked_slots_peak = tally.unblocked_slots_peak.max(unblocked_minimum);
    tally.aligned_pairs_positive_states += u64::from(aligned_minimum > 0);
    tally.slots_enough_states += u64::from(reading.slots_are_enough_on_every_device());
    tally.pairs_short_states += u64::from(reading.aligned_pairs_are_short_on_some_device());
    if reading.holds() {
        tally.slots_enough_and_aligned_pairs_short_states += 1;
        tally.first_slots_enough_and_aligned_pairs_short_step = tally
            .first_slots_enough_and_aligned_pairs_short_step
            .or(Some(step));
        let today = probe_one_new_inode_today(state);
        *tally
            .slots_enough_and_aligned_pairs_short_today_outcomes
            .entry(today.name())
            .or_default() += 1;
        tally.slots_enough_and_aligned_pairs_short_probes_that_published +=
            u64::from(today == RefusalOrPublished::Published);
        let session_outcome = session_outcome_of_one_new_inode(state);
        *tally
            .slots_enough_and_aligned_pairs_short_session_outcomes
            .entry(session_outcome)
            .or_default() += 1;
    } else if step.is_multiple_of(MODEL_CHECK_EVERY_STEPS) {
        tally.model_checks += 1;
        let today = probe_one_new_inode_today(state);
        tally.placement_refusals_with_slots_enough_and_aligned_pairs_short_false +=
            u64::from(matches!(
                today,
                RefusalOrPublished::Refused(RefusalClass::FixedPointPlacement { .. })
            ));
    }
}

/// 一个种子的 300 步：每步从五种走法里按种子挑一个，走完观测一次盘面。`state` 是这个种子自己的一份拷贝。
fn walk_one_seed_from(state: &mut SessionPool, seed: u64, steps: u64) -> Result<WalkTally, String> {
    let mut stream = SeededStream::of_seed(seed);
    let mut tally = WalkTally {
        walks: 1,
        ..WalkTally::default()
    };
    for step in 0..steps {
        let action = WALK_ACTIONS[usize::try_from(stream.below(5)).expect("下标")];
        let result = apply_walk_action(state, action)?;
        if result.starts_with("refused_") {
            *tally
                .action_refusals
                .entry(format!("{}:{result}", action.name()))
                .or_default() += 1;
        }
        observe_state(&mut tally, state, step);
    }
    let reading = RoomReading::of(state);
    tally.aligned_pairs_final_minimum = reading.aligned_free_pairs.iter().copied().min();
    tally.unblocked_slots_final_minimum = reading.unblocked_slots.iter().copied().min();
    Ok(tally)
}

/// 填满那一步最多发几次（保险丝）：每次发布取走当时能发成的最大一批，正常两三次就到头；超过就停下，产物里记 `publish_limit_reached`。
const FILL_PUBLISHES_AT_MOST: usize = 16;
/// 为一次发布找「能发成的单元数」最多试几次（每次都在拷贝上试）；超过就当这一次没找到，产物里记试了几次。
const FILL_PROBES_AT_MOST_PER_PUBLISH: u64 = 40;
/// 不限每次发布的单元数（产品跑法；单测传一个小数，才能在小几何上造出好几次发布）。
const FILL_NO_CAP_ON_THE_UNITS_OF_A_PUBLISH: u64 = u64::MAX;

/// 一次发成的填满发布。
#[derive(Clone, Debug, PartialEq, Eq)]
struct FillPublish {
    /// 这一次一开始要的单元数：段外槽对数与每次发布的单元数上限中小的那个。
    units_asked: u64,
    /// 发成的单元数。
    units: u64,
    /// 为找到它试了几次（含发成的那一次）。
    probes: u64,
}

/// 填满为什么停。
#[derive(Clone, Debug, PartialEq, Eq)]
enum FillStop {
    /// 段外再没有对齐的空槽对（登记 5.2 节「填满」的判法）。
    NoAlignedPairOutsideTheClusters,
    /// 核心层拒了：`units_asked` 是最后试的单元数（1 = 连一个单元的写也落不下），`refusal` 是那一次拒在哪一道，`probes` 是为这一次发布试了几次。
    Refused {
        units_asked: u64,
        refusal: RefusalClass,
        probes: u64,
    },
    /// 发了 [`FILL_PUBLISHES_AT_MOST`] 次还没到头。
    PublishLimitReached,
}

/// 填满的账：每次发布的单元数、停在哪、停时各盘段外还剩的对齐空槽对数、各盘 defer 队列里的槽数（换下的上一批在里面，仍占着槽）。
#[derive(Clone, Debug, PartialEq, Eq)]
struct FillReport {
    publishes: Vec<FillPublish>,
    stop: FillStop,
    remaining_pairs_outside_the_clusters: Vec<u64>,
    deferred_slots: Vec<u64>,
}

impl FillReport {
    /// 各次发布写下去的单元数之和（换下的批次也算：它们都占过槽对）。
    fn filled_units(&self) -> u64 {
        self.publishes.iter().map(|publish| publish.units).sum()
    }

    /// 最后一次发布写成的文件的单元数（现行那一版里活着的数据单元数）。
    fn units_of_the_file_now(&self) -> u64 {
        self.publishes.last().map_or(0, |publish| publish.units)
    }

    /// 产物行里的一段字段（`aligned_pair_shortage_setup`、`aligned_pair_shortage_measure_setup`、`verdict_aligned_pair_shortage_controls` 共用）。
    fn fields(&self) -> String {
        let units_per_publish: Vec<u64> =
            self.publishes.iter().map(|publish| publish.units).collect();
        let probes_per_publish: Vec<u64> = self
            .publishes
            .iter()
            .map(|publish| publish.probes)
            .collect();
        let stop = match &self.stop {
            FillStop::NoAlignedPairOutsideTheClusters => "no_aligned_pair_left_outside_the_clusters".to_string(),
            FillStop::Refused { units_asked, refusal, probes } => format!(
                "refused fill_refused_at_units={units_asked} fill_refusal_gate={} fill_refusal_probes={probes} fill_refusal_detail={:?}",
                refusal.name(),
                refusal.detail()
            ),
            FillStop::PublishLimitReached => "publish_limit_reached".to_string(),
        };
        format!(
            "fill_publishes={} fill_units_per_publish={units_per_publish:?} fill_probes_per_publish={probes_per_publish:?} filled_units={} units_of_the_file_now={} fill_stop={stop} remaining_aligned_pairs_outside_the_clusters={:?} deferred_slots={:?}",
            self.publishes.len(),
            self.filled_units(),
            self.units_of_the_file_now(),
            self.remaining_pairs_outside_the_clusters,
            self.deferred_slots,
        )
    }
}

/// 各盘段外对齐的空槽对数（两槽都没挡、起点偶数、不在开过的聚簇段里，登记 5.2 节「填满」数的就是它）。
fn remaining_pairs_outside_the_clusters(pool: &Pool) -> Vec<u64> {
    (0..pool.devices.len())
        .map(|device_index| data_unit_pairs_outside_the_clusters(pool, device_index))
        .collect()
}

/// 被拒之后，下一次试几个单元（`units` 是刚被拒的那个数，至少 2）：
/// journal 记录数超限 ⇒ 减去超出的条数；式子或落得下那一判不够 ⇒ 减去缺口折成的单元数（缺口字节 ÷ 一个单元在一块盘上的两槽，向上取整，
/// 缺口读数不是正数时按一字节算）；落点那一道拒与别的拒因 ⇒ 折半。结果夹进 [1, `units` − 1]，所以单元数每次严格减小、不会降到 0。
fn units_to_try_after_a_refusal(refusal: &RefusalClass, units: u64) -> u64 {
    assert!(units >= 2, "刚被拒的单元数至少 2，才有更小的可试：{units}");
    let units_wide = i128::from(units);
    let wanted = match refusal {
        RefusalClass::Other {
            journal_records_over_the_limit: Some(surplus),
            ..
        } => units_wide - i128::from(*surplus),
        RefusalClass::SpaceAdmission {
            shortfall_bytes, ..
        } => units_wide - units_a_shortfall_is_worth(*shortfall_bytes),
        RefusalClass::FixedPointPlacement { .. } | RefusalClass::Other { .. } => units_wide / 2,
    };
    u64::try_from(wanted.clamp(1, units_wide - 1)).expect("夹进 [1, 上一次 − 1] 之后是正数")
}

/// [`land_the_most_units_up_to`] 的结局。
enum Landing {
    Landed(FillPublish),
    NothingLands {
        units_asked: u64,
        refusal: RefusalClass,
        probes: u64,
    },
}

/// 在 `pool` 上接着换写，先试 `asked` 个单元：每次试都在 `pool` 的拷贝上做，发成了就把那份拷贝换成 `pool`（不重写一遍）；
/// 被拒了按 [`units_to_try_after_a_refusal`] 算下一次试几个，夹进 [1, 上一次 − 1]，单元数严格减小，直到发成，或一个单元也被拒，
/// 或试满 [`FILL_PROBES_AT_MOST_PER_PUBLISH`] 次（这两样交回 `NothingLands`，带最后一次的拒因）。
fn land_the_most_units_up_to(
    pool: &mut Pool,
    asked: u64,
    content: &mut ContentBuffer,
) -> Result<Landing, String> {
    let mut units = asked;
    let mut probes = 0_u64;
    loop {
        probes += 1;
        let mut probe = pool.fork();
        let outcome = write_units_and_account(
            &mut probe,
            usize::try_from(units).expect("单元数"),
            CodeTwoTreeNodeCapacities::FromTheNodeFormat,
            content,
        )?;
        let refusal = match outcome {
            WriteOutcome::Published(_) => {
                *pool = probe;
                return Ok(Landing::Landed(FillPublish {
                    units_asked: asked,
                    units,
                    probes,
                }));
            }
            WriteOutcome::Refused(refusal) => refusal,
        };
        if units == 1 || probes >= FILL_PROBES_AT_MOST_PER_PUBLISH {
            return Ok(Landing::NothingLands {
                units_asked: units,
                refusal,
                probes,
            });
        }
        units = units_to_try_after_a_refusal(&refusal, units);
    }
}

/// 把数据那一侧填满（登记 12.11）：从第一个文件起多次发布，每次接着换写（`publish_sequential_write`，从偏移 0 整份换写；
/// 核心层公开入口里给数据单元的只有这一条路，没有写别的文件、续写的入口）。一次发布受 journal 记录数上限（`JournalRecordsOfThePublishExceedTheLimit`）
/// 所限，所以每次发布先要「段外槽对数」与 `units_per_publish_at_most` 中小的那个，被拒了按 [`land_the_most_units_up_to`] 往下找能发成的最大一批；
/// 换下的上一批进 defer 队列、仍占着槽，下一次发布只能落在剩下的空槽对上。停在两处之一：段外再没有对齐的空槽对，或核心层拒了一个单元的写
/// （这时拒因、剩下的槽对数都在账里，不再往下试别的）。准入式（D28 九项）扣着环窗口、checkpoint 保留池、实例切换预留，
/// 所以填满的上限可能不是槽对数。
fn fill_the_data_side(
    pool: &mut Pool,
    content: &mut ContentBuffer,
    units_per_publish_at_most: u64,
) -> Result<FillReport, String> {
    assert!(units_per_publish_at_most >= 1, "每次发布至少 1 个单元");
    let mut publishes: Vec<FillPublish> = Vec::new();
    let stop = loop {
        let fewest_pairs = remaining_pairs_outside_the_clusters(pool)
            .into_iter()
            .min()
            .expect("至少一块盘");
        if fewest_pairs == 0 {
            break FillStop::NoAlignedPairOutsideTheClusters;
        }
        if publishes.len() >= FILL_PUBLISHES_AT_MOST {
            break FillStop::PublishLimitReached;
        }
        match land_the_most_units_up_to(pool, fewest_pairs.min(units_per_publish_at_most), content)?
        {
            Landing::Landed(publish) => publishes.push(publish),
            Landing::NothingLands {
                units_asked,
                refusal,
                probes,
            } => {
                break FillStop::Refused {
                    units_asked,
                    refusal,
                    probes,
                }
            }
        }
    };
    Ok(FillReport {
        publishes,
        stop,
        remaining_pairs_outside_the_clusters: remaining_pairs_outside_the_clusters(pool),
        deferred_slots: pool
            .allocator
            .devices
            .iter()
            .map(|device_map| device_map.deferred_slots())
            .collect(),
    })
}

/// 阳性对照（登记 5.2 节，不可达，只当对照）：用分配器公开的隔离入口把每个空的偶数起点槽对的后一槽隔离掉，
/// G10 必须为真、今天那一臂必须拒在固定点、Q1 的判法必须先拒。
fn run_aligned_pair_shortage_positive_control(
    label: &str,
    base: &mut SessionPool,
) -> Result<bool, String> {
    let mut copy = base.fork();
    for device_map in &mut copy.session.allocator.devices {
        let first = device_map.unit_area_start().slot().0;
        let end = first + device_map.unit_area_slots();
        let mut slot = first;
        while slot + 1 < end {
            if device_map.is_free(SlotNumber(slot)) && device_map.is_free(SlotNumber(slot + 1)) {
                device_map.isolate(SlotNumber(slot + 1), 1);
            }
            slot += DATA_UNIT_SLOTS_LOCAL;
        }
    }
    let reading = RoomReading::of(&copy);
    let today = probe_one_new_inode_today(&mut copy);
    let landing_check_counting_aligned_pairs_would_refuse_first =
        reading.aligned_pairs_are_short_on_some_device();
    let control_hits = reading.holds()
        && matches!(
            today,
            RefusalOrPublished::Refused(RefusalClass::FixedPointPlacement { .. })
        )
        && landing_check_counting_aligned_pairs_would_refuse_first;
    emit(&format!(
        "name=aligned_pair_shortage_positive_control state={label} slots_enough_and_aligned_pairs_short={} slots_enough={} aligned_pairs_short={} aligned_free_pairs={:?} unblocked_slots={:?} needed_slots={} today_outcome={} landing_check_counting_aligned_pairs_would_refuse_before_the_fixed_point={landing_check_counting_aligned_pairs_would_refuse_first} control_hits={control_hits}",
        reading.holds(),
        reading.slots_are_enough_on_every_device(),
        reading.aligned_pairs_are_short_on_some_device(),
        reading.aligned_free_pairs,
        reading.unblocked_slots,
        reading.needed_slots,
        today.name(),
    ));
    Ok(control_hits)
}

/// 开跑前量 F10 的 10 步（登记 5.6 节）：G默认上填满数据那一侧、可写挂载、走 10 步，外推两个几何 × 64 种子 × 300 步要多久。
fn run_aligned_pair_shortage_measure(content: &mut ContentBuffer) -> Result<(), String> {
    let build_started = Instant::now();
    let mut pool = build_pool_after_the_first_file(
        GEOMETRY_DEFAULT,
        CodeTwoTreeNodeCapacities::FromTheNodeFormat,
    );
    let built_seconds = build_started.elapsed().as_secs_f64();
    let fill_started = Instant::now();
    let fill = fill_the_data_side(&mut pool, content, FILL_NO_CAP_ON_THE_UNITS_OF_A_PUBLISH)?;
    let filled_seconds = fill_started.elapsed().as_secs_f64();
    let mount_started = Instant::now();
    let mut state = session_pool_of(&mut pool)?;
    drop(pool);
    let mounted_seconds = mount_started.elapsed().as_secs_f64();
    emit(&format!(
        "name=aligned_pair_shortage_measure_setup geometry={} build_seconds={built_seconds:.3} fill_seconds={filled_seconds:.3} {} mount_seconds={mounted_seconds:.3}",
        GEOMETRY_DEFAULT.name,
        fill.fields()
    ));
    let mut stream = SeededStream::of_seed(0);
    let mut tally = WalkTally::default();
    let mut step_seconds = Vec::new();
    for step in 0..10_u64 {
        let action = WALK_ACTIONS[usize::try_from(stream.below(5)).expect("下标")];
        let action_started = Instant::now();
        let result = apply_walk_action(&mut state, action)?;
        let action_seconds = action_started.elapsed().as_secs_f64();
        let observe_started = Instant::now();
        observe_state(&mut tally, &mut state, step);
        let observe_seconds = observe_started.elapsed().as_secs_f64();
        step_seconds.push(action_seconds + observe_seconds);
        emit(&format!(
            "name=aligned_pair_shortage_measure_step step={step} action={} result={result} action_seconds={action_seconds:.3} observe_seconds={observe_seconds:.3}",
            action.name()
        ));
    }
    step_seconds.sort_by(f64::total_cmp);
    let median = step_seconds[step_seconds.len() / 2];
    let total_steps = ALIGNED_PAIR_SHORTAGE_GEOMETRY_COUNT
        * ALIGNED_PAIR_SHORTAGE_SEEDS
        * ALIGNED_PAIR_SHORTAGE_STEPS;
    #[allow(
        clippy::cast_precision_loss,
        reason = "小数的步数，外推出来的小时数只报一位小数"
    )]
    let extrapolated_single_thread_hours = median * (total_steps as f64) / 3600.0;
    emit(&format!(
        "name=f10_measure_extrapolation median_step_seconds={median:.3} total_steps={total_steps} extrapolated_hours_single_thread={extrapolated_single_thread_hours:.1} budget_hours=2"
    ));
    Ok(())
}

const ALIGNED_PAIR_SHORTAGE_GEOMETRY_COUNT: u64 = 2;
const ALIGNED_PAIR_SHORTAGE_SEEDS: u64 = 64;
const ALIGNED_PAIR_SHORTAGE_STEPS: u64 = 300;

/// 一个状态上各盘最低落点是否不等，以及这一个单元落得下那一判放不放行（登记 5.3 节 Q9）。
#[derive(Clone, Debug, PartialEq, Eq)]
struct UnequalLandingReading {
    lowest_slot_per_device: Vec<Option<u64>>,
    lowest_slots_differ: bool,
    one_data_unit_lands: bool,
}

fn unequal_landing_reading(pool: &Pool) -> UnequalLandingReading {
    let cluster_segments = pool.allocator.cluster_segments();
    let lowest_slot_per_device: Vec<Option<u64>> = pool
        .allocator
        .devices
        .iter()
        .map(|device_map| {
            device_map
                .lowest_user_data_slot(cluster_segments)
                .map(|slot| slot.0)
        })
        .collect();
    let lowest_slots_differ = lowest_slot_per_device
        .iter()
        .any(|slot| *slot != lowest_slot_per_device[0]);
    let checkpoint_cost = core_checkpoint_cost(
        &pool.current,
        &pool.allocator,
        CodeTwoTreeNodeCapacities::FromTheNodeFormat,
    );
    let one_data_unit_lands = admit_the_units_landing_on_every_device(
        &PlacementRoomOnOneDevice::of_every_device_of_the_allocator(&pool.allocator),
        UnitsOfAPublishToLand {
            data_units: 1,
            commit_generated_slots: NEW_INODE_ORDINARY_COMMIT_GENERATED_SLOTS + checkpoint_cost,
        },
    )
    .is_ok();
    UnequalLandingReading {
        lowest_slot_per_device,
        lowest_slots_differ,
        one_data_unit_lands,
    }
}

/// 登记 5.3 节 Q9 与它的阳性对照：G不等大上从第一个文件起换写到小盘段外槽对数的 25%、50%、75%，每个状态判一次；
/// 对照把小盘单元区填到末尾（照 `unequal_devices.rs` 那一形，只改空闲图），各盘最低落点必须不等。
fn run_unequal_landing_states(content: &mut ContentBuffer) -> Result<(u64, u64, bool), String> {
    let mut base = build_pool_after_the_first_file(
        GEOMETRY_UNEQUAL,
        CodeTwoTreeNodeCapacities::FromTheNodeFormat,
    );
    let smaller_pairs = data_unit_pairs_outside_the_clusters(&base, 1);
    let mut states = 0;
    let mut unequal_and_landing = 0;
    let mut readings = vec![(
        "after_the_first_file".to_string(),
        unequal_landing_reading(&base),
    )];
    for percent in [25_u64, 50, 75] {
        let mut fork = base.fork();
        let units = usize::try_from(smaller_pairs * percent / 100).expect("单元数");
        match write_units_and_account(
            &mut fork,
            units,
            CodeTwoTreeNodeCapacities::FromTheNodeFormat,
            content,
        )? {
            WriteOutcome::Published(_) => {
                readings.push((
                    format!("filled_{percent}_percent_of_the_smaller_device"),
                    unequal_landing_reading(&fork),
                ));
            }
            WriteOutcome::Refused(refusal) => {
                emit(&format!(
                    "name=per_device_lowest_slot_unequal_fill_refused percent={percent} units={units} refusal={refusal:?}"
                ));
            }
        }
    }
    for (state, reading) in &readings {
        states += 1;
        unequal_and_landing +=
            u64::from(reading.lowest_slots_differ && reading.one_data_unit_lands);
        emit(&format!(
            "name=per_device_lowest_slot_unequal_state geometry={} state={state} lowest_slot_per_device={:?} lowest_slots_differ={} one_data_unit_lands={}",
            GEOMETRY_UNEQUAL.name,
            reading.lowest_slot_per_device,
            reading.lowest_slots_differ,
            reading.one_data_unit_lands
        ));
    }
    let mut control = base.fork();
    let smaller_end = control.allocator.devices[1].unit_area_start().slot().0
        + control.allocator.devices[1].unit_area_slots();
    for device_map in &mut control.allocator.devices {
        let first = device_map.unit_area_start().slot().0;
        for slot in first..smaller_end {
            if device_map.is_free(SlotNumber(slot)) {
                device_map.mark_allocated(SlotNumber(slot), 1);
            }
        }
    }
    let control_reading = unequal_landing_reading(&control);
    emit(&format!(
        "name=per_device_lowest_slot_unequal_positive_control lowest_slot_per_device={:?} lowest_slots_differ={} one_data_unit_lands={} control_hits={}",
        control_reading.lowest_slot_per_device,
        control_reading.lowest_slots_differ,
        control_reading.one_data_unit_lands,
        control_reading.lowest_slots_differ
    ));
    Ok((
        states,
        unequal_and_landing,
        control_reading.lowest_slots_differ,
    ))
}

/// 开写前的本地常量回比（登记第五节「要读 `crates/` 常量的量改成本地常量加回比断言」）：对不上返回一句话，作废。
fn check_local_constants_against_the_crates() -> Result<(), String> {
    let comparisons: [(&str, u64, u64); 13] = [
        ("槽字节数", SLOT_BYTES_LOCAL, singlefs_format::SLOT_BYTES),
        (
            "聚簇段槽数",
            CLUSTER_SEGMENT_SLOTS_LOCAL,
            singlefs_format::CLUSTER_SEGMENT_SLOTS,
        ),
        (
            "单元区起始槽号",
            UNIT_AREA_START_SLOT_LOCAL,
            singlefs_format::UNIT_AREA_START_SLOT,
        ),
        (
            "默认环字节数",
            DEFAULT_JOURNAL_RING_BYTES_LOCAL,
            singlefs_format::JOURNAL_RING_DEFAULT_BYTES,
        ),
        (
            "每区槽数 S（mkfs 默认）",
            ROOT_RING_SLOTS_PER_REGION_LOCAL,
            RootRingSlotsPerRegion::AT_MAKE_FILESYSTEM.count(),
        ),
        (
            "分配记录树叶罩槽数",
            ALLOCATION_RECORD_TREE_LEAF_SLOTS_LOCAL,
            singlefs_format::ALLOCATION_RECORD_TREE_LEAF_SLOTS,
        ),
        (
            "分配记录树内部扇出",
            ALLOCATION_RECORD_TREE_INTERNAL_FANOUT_LOCAL,
            singlefs_format::ALLOCATION_RECORD_TREE_INTERNAL_FANOUT,
        ),
        (
            "实例表落点",
            UNIT_AREA_START_SLOT_LOCAL + INSTANCE_TABLE_OFFSET_FROM_UNIT_AREA_START_LOCAL,
            INSTANCE_TABLE_SLOT.0,
        ),
        (
            "树表第 0 版落点",
            UNIT_AREA_START_SLOT_LOCAL + TREE_TABLE_OFFSET_FROM_UNIT_AREA_START_LOCAL,
            TREE_TABLE_GENESIS_SLOT.0,
        ),
        (
            "数据单元槽数",
            DATA_UNIT_SLOTS_LOCAL,
            singlefs_format::DATA_UNIT_BYTES / singlefs_format::SLOT_BYTES,
        ),
        (
            "数据单元字节数",
            u64::try_from(DATA_UNIT_BYTES_LOCAL).expect("字节数"),
            singlefs_format::DATA_UNIT_BYTES,
        ),
        (
            "小几何的单元区起点",
            SMALL_GEOMETRY_UNIT_AREA_START_SLOT,
            UnitAreaStart::following_the_journal_ring(SMALL_GEOMETRY_JOURNAL_RING_BYTES)
                .expect("小环末尾在段边界上")
                .slot()
                .0,
        ),
        (
            "默认环的单元区起点",
            UNIT_AREA_START_SLOT_LOCAL,
            UnitAreaStart::following_the_journal_ring(DEFAULT_JOURNAL_RING_BYTES_LOCAL)
                .expect("默认环末尾在段边界上")
                .slot()
                .0,
        ),
    ];
    let mut mismatches = Vec::new();
    for (name, local, crate_value) in comparisons {
        emit(&format!(
            "name=constant_check item={name} local={local} crate={crate_value} equal={}",
            local == crate_value
        ));
        if local != crate_value {
            mismatches.push(format!("{name}：本地 {local}，crates {crate_value}"));
        }
    }
    if mismatches.is_empty() {
        Ok(())
    } else {
        Err(format!("本地常量回比不等：{}", mismatches.join("；")))
    }
}

/// 登记第七节的钉绝对值的断言：A1（G默认 本地 K1 = 529）、A3（H1 之后第一个文件的数据单元在 50180、每块盘最低落点 50182）、
/// A4（H1 所选那一版账里记录在盘 0 上的最大末槽 = 50253）、B1（单元区槽数与整段数）、B2（K1 逐层罩到的位置数）。
/// 对不上走 Y7「条款可能错」：返回 `Err`，停机。
fn run_anchors() -> Result<(), String> {
    let pool = build_pool_after_the_first_file(
        GEOMETRY_DEFAULT,
        CodeTwoTreeNodeCapacities::FromTheNodeFormat,
    );
    let unit_areas: Vec<(u64, u64)> = pool
        .allocator
        .devices
        .iter()
        .map(|device_map| {
            (
                device_map.unit_area_start().slot().0,
                device_map.unit_area_slots(),
            )
        })
        .collect();
    let root_level_read = pool
        .current
        .position_addressed_tree_heights_read_from_the_root_node_headers()
        .allocation_record_tree
        - 1;
    let local_allocation_record_tree_nodes = local_allocation_record_tree_nodes(
        u8::try_from(root_level_read).expect("层级"),
        &unit_areas,
    );
    let level_zero_positions = (unit_areas[0].0 + unit_areas[0].1 - 1)
        / ALLOCATION_RECORD_TREE_LEAF_SLOTS_LOCAL
        - unit_areas[0].0 / ALLOCATION_RECORD_TREE_LEAF_SLOTS_LOCAL
        + 1;
    emit(&format!(
        "name=anchor_allocation_record_tree_nodes local_allocation_record_tree_nodes={local_allocation_record_tree_nodes} expected_allocation_record_tree_nodes=529 root_level_read_from_the_header={root_level_read} level_zero_positions_per_device={level_zero_positions} expected_level_zero_positions=262 unit_areas={unit_areas:?}"
    ));
    let first_file_slot = pool
        .current
        .units
        .iter()
        .find(|unit| unit.identity == TransactionUnit::Data(DataUnitIndexInFile::FIRST))
        .map(|unit| unit.slot.0);
    let lowest_slots: Vec<Option<u64>> = pool
        .allocator
        .devices
        .iter()
        .map(|device_map| {
            device_map
                .lowest_user_data_slot(pool.allocator.cluster_segments())
                .map(|slot| slot.0)
        })
        .collect();
    emit(&format!(
        "name=anchor_first_file_slot_and_lowest_slots first_file_data_unit_slot={first_file_slot:?} expected=Some(50180) lowest_user_data_slot_per_device={lowest_slots:?} expected_lowest=Some(50182)"
    ));
    let largest_end_on_device_zero = pool
        .current
        .allocation_records
        .iter()
        .filter(|record| record.device == DeviceIdentity(0))
        .map(|record| record.slot.0 + u64::from(record.span_slots))
        .max();
    emit(&format!(
        "name=anchor_largest_record_end_on_device_zero largest_record_end_on_device_zero={largest_end_on_device_zero:?} expected=Some(50253)"
    ));
    let unit_area_slots_of = |geometry: Geometry| -> Vec<u64> {
        geometry
            .device_bytes
            .iter()
            .map(|device_bytes| device_bytes / SLOT_BYTES_LOCAL - UNIT_AREA_START_SLOT_LOCAL)
            .collect()
    };
    let default_areas = unit_area_slots_of(GEOMETRY_DEFAULT);
    let narrow_areas = unit_area_slots_of(GEOMETRY_NARROW);
    emit(&format!(
        "name=anchor_unit_area_slots_and_whole_segments default_unit_area_slots={default_areas:?} default_whole_segments={} narrow_unit_area_slots={narrow_areas:?} narrow_whole_segments={} narrow_tail_slots={}",
        default_areas[0] / CLUSTER_SEGMENT_SLOTS_LOCAL,
        narrow_areas[0] / CLUSTER_SEGMENT_SLOTS_LOCAL,
        narrow_areas[0] % CLUSTER_SEGMENT_SLOTS_LOCAL,
    ));
    let mut failures = Vec::new();
    if local_allocation_record_tree_nodes != 529 || level_zero_positions != 262 {
        failures.push(format!(
            "分配记录树节点数：{local_allocation_record_tree_nodes}（应 529），层级 0 每盘 {level_zero_positions}（应 262）"
        ));
    }
    if first_file_slot != Some(50180) || lowest_slots != vec![Some(50182), Some(50182)] {
        failures.push(format!(
            "第一个文件的落点与各盘最低落点：{first_file_slot:?} {lowest_slots:?}"
        ));
    }
    if largest_end_on_device_zero != Some(50253) {
        failures.push(format!(
            "盘 0 上记录的最大末槽：{largest_end_on_device_zero:?}"
        ));
    }
    if default_areas != vec![211_968, 211_968]
        || narrow_areas != vec![146_465, 146_465]
        || default_areas[0] / CLUSTER_SEGMENT_SLOTS_LOCAL != 3312
        || narrow_areas[0] / CLUSTER_SEGMENT_SLOTS_LOCAL != 2288
        || narrow_areas[0] % CLUSTER_SEGMENT_SLOTS_LOCAL != 33
    {
        failures.push(format!(
            "单元区槽数与整段数：{default_areas:?} {narrow_areas:?}"
        ));
    }
    let real_areas: Vec<u64> = pool
        .allocator
        .devices
        .iter()
        .map(|device_map| device_map.unit_area_slots())
        .collect();
    if real_areas != default_areas {
        failures.push(format!(
            "单元区槽数：分配器的单元区槽数 {real_areas:?} 与按字节数算的 {default_areas:?} 不等"
        ));
    }
    if failures.is_empty() {
        Ok(())
    } else {
        Err(failures.join("；"))
    }
}

fn emit_fixed_point_upper_bound_verdicts(
    collector: &FixedPointUpperBoundCollector,
    edges: &[(&str, EdgeReport)],
) {
    let all = collector.tallies.get("all").cloned().unwrap_or_default();
    let three_levels_mechanism_present = collector
        .tallies
        .iter()
        .filter(|(class, _)| class.starts_with("three_level_history"))
        .any(|(class, tally)| {
            class.ends_with("/all") && tally.checkpoint_cost_below_true_writes > 0
        });
    let edge_cell_built = edges
        .iter()
        .any(|(_, report)| report.refused_in_the_fixed_point > 0);
    let edge_found_in_every_edge_geometry = edges
        .iter()
        .all(|(_, report)| report.edge_not_found_reason.is_none());
    emit(&format!(
        "name=verdict_fixed_point_upper_bound writes={} checkpoint_cost_with_ordinary_mapping_changes_is_an_upper_bound_on_every_write={:?} checkpoint_cost_with_ordinary_mapping_changes_below_true_writes={} checkpoint_cost_below_true_writes={} checkpoint_cost_with_ordinary_mapping_changes_minus_checkpoint_cost_maximum={} checkpoint_cost_with_ordinary_mapping_changes_equals_checkpoint_cost_on_every_write={} three_level_history_shows_true_above_checkpoint_cost={three_levels_mechanism_present} edge_cell_fixed_point_refusals_built={edge_cell_built} edge_found_in_every_edge_geometry={edge_found_in_every_edge_geometry} stops={}",
        all.writes,
        checkpoint_cost_with_ordinary_mapping_changes_is_an_upper_bound_on_every_write(&all),
        all.checkpoint_cost_with_ordinary_mapping_changes_below_true_writes,
        all.checkpoint_cost_below_true_writes,
        text_of(all.checkpoint_cost_with_ordinary_mapping_changes_minus_checkpoint_cost_maximum),
        all.checkpoint_cost_with_ordinary_mapping_changes_minus_checkpoint_cost_maximum == Some(0),
        collector.stop_reasons.len(),
    ));
    for (kind, count) in &collector.refusal_counts {
        emit(&format!(
            "name=fixed_point_upper_bound_refusal_count kind={kind} count={count}"
        ));
    }
    for (geometry, report) in edges {
        emit(&format!(
            "name=fixed_point_upper_bound_edge_summary geometry={geometry} pairs_outside_the_clusters={} filled_units={} boundary_units={} window_first={} window_last={} published={} refused_by_space_admission={} refused_in_the_fixed_point={} refused_other={} fixed_point_refusals_in_the_first_sixty_four={} fixed_point_refused_units={:?} first_and_last_space_refused={:?}",
            report.pairs_outside_the_clusters,
            report.filled_units,
            report.boundary_units,
            report.window_first,
            report.window_last,
            report.published,
            report.refused_by_space_admission,
            report.refused_in_the_fixed_point,
            report.refused_other,
            report.fixed_point_refusals_in_the_first_sixty_four,
            report.fixed_point_refused_units,
            report.space_refused_units_first_and_last,
        ));
        emit(&format!(
            "name=fixed_point_upper_bound_edge_search geometry={geometry} edge_found={} filled_units={} lower_bound_units={} lower_bound_refused_tries={:?} bisection_rounds={} bisection_points_tried={} bisection_points_passed={} edge_not_found_reason={:?}",
            report.edge_not_found_reason.is_none(),
            report.filled_units,
            text_of_unsigned(report.lower_bound_units),
            report.lower_bound_refused_tries,
            report.bisection_rounds,
            report.bisection_points_tried,
            report.bisection_points_passed,
            report.edge_not_found_reason.as_deref().unwrap_or("none"),
        ));
    }
}

/// 有一个边历史没找到边（下界没找到、一直放行到上限、或放行与被拒次序不一致）就停机：这样的边界不是观测夹住的，窗口没试，
/// 不能当 Q4 的读数。产物照打（含判决行），停机行由 `main` 打，退出码 3。
fn stop_when_an_edge_was_not_found(edges: &[(&str, EdgeReport)]) -> Result<(), String> {
    let not_found: Vec<String> = edges
        .iter()
        .filter_map(|(geometry, report)| {
            report
                .edge_not_found_reason
                .as_ref()
                .map(|reason| format!("{geometry}：{reason}"))
        })
        .collect();
    if not_found.is_empty() {
        Ok(())
    } else {
        Err(format!("没找到边：{}", not_found.join("；")))
    }
}

fn run_fixed_point_upper_bound(content: &mut ContentBuffer) -> Result<(), String> {
    let mut collector = FixedPointUpperBoundCollector::default();
    for geometry in [GEOMETRY_DEFAULT, GEOMETRY_NARROW, GEOMETRY_SMALL] {
        run_product_capacity_history(&mut collector, geometry, content)?;
    }
    for geometry in [GEOMETRY_DEFAULT, GEOMETRY_SMALL] {
        let sixty_four = run_three_level_history(
            &mut collector,
            "three_level_history_step_64",
            geometry,
            64,
            content,
        )?;
        let four = run_three_level_history(
            &mut collector,
            "three_level_history_step_4",
            geometry,
            4,
            content,
        )?;
        emit(&format!(
            "name=fixed_point_upper_bound_three_level_histories geometry={} writes_step_64={sixty_four} writes_step_4={four}",
            geometry.name
        ));
    }
    let requested_edges = std::env::var("E181_FIXED_POINT_UPPER_BOUND_EDGE_GEOMETRIES")
        .unwrap_or_else(|_unset| format!("{},{}", GEOMETRY_NARROW.name, GEOMETRY_SMALL.name));
    emit(&format!(
        "name=fixed_point_upper_bound_setup edge_geometries={requested_edges} edge_geometries_are_the_registered_and_added_ones={}",
        requested_edges == format!("{},{}", GEOMETRY_NARROW.name, GEOMETRY_SMALL.name)
    ));
    let mut edges = Vec::new();
    for geometry in [GEOMETRY_NARROW, GEOMETRY_SMALL] {
        if requested_edges.split(',').any(|name| name == geometry.name) {
            edges.push((
                geometry.name,
                run_edge_history(&mut collector, geometry, content)?,
            ));
        }
    }
    collector.emit_tallies();
    emit_fixed_point_upper_bound_verdicts(&collector, &edges);
    stop_when_an_edge_was_not_found(&edges)
}

fn run_per_device_lowest_slot(content: &mut ContentBuffer) -> Result<(), String> {
    let reports = run_scan_growth(content)?;
    for state in [
        "after_the_first_file",
        "half_marked",
        "ninety_percent_marked",
    ] {
        let by_geometry: Vec<&ScanStateReport> = reports
            .iter()
            .filter(|report| report.state == state)
            .collect();
        let medians: Vec<String> = by_geometry
            .iter()
            .map(|report| {
                let scan = report
                    .scans
                    .iter()
                    .map(|scan| scan.median_nanoseconds)
                    .max()
                    .unwrap_or(0);
                format!("{}:{}:{scan}", report.geometry, report.unit_area_slots)
            })
            .collect();
        emit(&format!(
            "name=per_device_lowest_slot_growth state={state} geometry_unit_area_slots_slowest_device_median_nanoseconds={}",
            medians.join(",")
        ));
        for report in by_geometry {
            let scan_both_devices: u128 = report
                .scans
                .iter()
                .map(|scan| scan.median_nanoseconds)
                .sum();
            emit(&format!(
                "name=per_device_lowest_slot_ratio geometry={} state={state} gate_extra_nanoseconds_both_devices={scan_both_devices} placement_one_data_unit_nanoseconds={} gate_extra_over_placement_percent={}",
                report.geometry,
                report.placement_median_nanoseconds,
                scan_both_devices * 100 / report.placement_median_nanoseconds.max(1),
            ));
        }
    }
    let (states, unequal_and_landing, control_hits) = run_unequal_landing_states(content)?;
    emit(&format!(
        "name=verdict_per_device_lowest_slot unequal_states_checked={states} unequal_lowest_slots_with_the_landing_check_passing={unequal_and_landing} positive_control_unequal_lowest_slots_built={control_hits}"
    ));
    Ok(())
}

fn run_aligned_pair_shortage_controls(
    geometry: Geometry,
    content: &mut ContentBuffer,
) -> Result<(), String> {
    let mut unfilled =
        build_pool_after_the_first_file(geometry, CodeTwoTreeNodeCapacities::FromTheNodeFormat);
    let mut unfilled_session = session_pool_of(&mut unfilled)?;
    let unfilled_hits = run_aligned_pair_shortage_positive_control(
        &format!("{}_after_the_first_file", geometry.name),
        &mut unfilled_session,
    )?;
    drop(unfilled_session);
    let fill = fill_the_data_side(
        &mut unfilled,
        content,
        FILL_NO_CAP_ON_THE_UNITS_OF_A_PUBLISH,
    )?;
    let mut filled_session = session_pool_of(&mut unfilled)?;
    let filled_hits = run_aligned_pair_shortage_positive_control(
        &format!("{}_data_side_filled", geometry.name),
        &mut filled_session,
    )?;
    emit(&format!(
        "name=verdict_aligned_pair_shortage_controls geometry={} {} control_hits_after_the_first_file={unfilled_hits} control_hits_data_side_filled={filled_hits}",
        geometry.name,
        fill.fields()
    ));
    Ok(())
}

/// 走种子的线程数：设了 `E181_THREADS` 照它，没设取每台默认重活线程数（`singlefs_harness::heavy_threads`）。
fn walk_thread_count() -> usize {
    let thread_cap_value =
        std::env::var(singlefs_harness::heavy_threads::THREAD_CAP_ENVIRONMENT_VARIABLE).ok();
    singlefs_harness::heavy_threads::own_variable_or_default_heavy_threads(
        "E181_THREADS",
        std::env::var("E181_THREADS").ok().as_deref(),
        thread_cap_value.as_deref(),
    )
    .get()
}

/// 登记 5.2 节的整段 F10：填满数据那一侧、可写挂载，64 个种子各走 300 步（线程数见 [`walk_thread_count`]；
/// 每个线程各拷一份底，种子按序号取，合并按种子序，所以线程数不改产物）。种子数与步数可由 `E181_ALIGNED_PAIR_SHORTAGE_SEEDS`、`E181_ALIGNED_PAIR_SHORTAGE_STEPS` 缩小，
/// 缩过的那一趟在产物里写明（登记第八节：缩过规模的不当整段）。
fn run_aligned_pair_shortage_walks(
    geometry: Geometry,
    content: &mut ContentBuffer,
) -> Result<WalkTally, String> {
    let seeds = number_from_the_environment(
        "E181_ALIGNED_PAIR_SHORTAGE_SEEDS",
        ALIGNED_PAIR_SHORTAGE_SEEDS,
    );
    let steps = number_from_the_environment(
        "E181_ALIGNED_PAIR_SHORTAGE_STEPS",
        ALIGNED_PAIR_SHORTAGE_STEPS,
    );
    let threads = walk_thread_count();
    let mut pool =
        build_pool_after_the_first_file(geometry, CodeTwoTreeNodeCapacities::FromTheNodeFormat);
    let fill = fill_the_data_side(&mut pool, content, FILL_NO_CAP_ON_THE_UNITS_OF_A_PUBLISH)?;
    let mut base = session_pool_of(&mut pool)?;
    drop(pool);
    let initial = RoomReading::of(&base);
    emit(&format!(
        "name=aligned_pair_shortage_setup geometry={} {} seeds={seeds} steps={steps} seeds_and_steps_are_the_registered_ones={} threads={threads} initial_aligned_free_pairs={:?} initial_unblocked_slots={:?} needed_slots={}",
        geometry.name,
        fill.fields(),
        seeds == ALIGNED_PAIR_SHORTAGE_SEEDS && steps == ALIGNED_PAIR_SHORTAGE_STEPS,
        initial.aligned_free_pairs,
        initial.unblocked_slots,
        initial.needed_slots,
    ));
    drop(base.fork());
    let sealed_base = &base;
    let next_seed = std::sync::atomic::AtomicU64::new(0);
    let finished: std::sync::Mutex<Vec<(u64, Result<WalkTally, String>)>> =
        std::sync::Mutex::new(Vec::new());
    std::thread::scope(|scope| {
        for _thread in 0..threads {
            scope.spawn(|| loop {
                let seed = next_seed.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                if seed >= seeds {
                    break;
                }
                let mut state = sealed_base.fork_of_a_sealed_base();
                let walked = walk_one_seed_from(&mut state, seed, steps);
                finished.lock().expect("锁").push((seed, walked));
            });
        }
    });
    let mut results = finished.into_inner().expect("锁");
    results.sort_by_key(|(seed, _)| *seed);
    let mut total = WalkTally::default();
    for (seed, walked) in results {
        let tally = walked.map_err(|reason| format!("种子 {seed}：{reason}"))?;
        emit(&format!(
            "name=aligned_pair_shortage_seed geometry={} seed={seed} slots_enough_and_aligned_pairs_short_states={} first_slots_enough_and_aligned_pairs_short_step={} aligned_pairs_peak={} aligned_pairs_positive_states={} aligned_pairs_final={} unblocked_slots_peak={} unblocked_slots_final={}",
            geometry.name,
            tally.slots_enough_and_aligned_pairs_short_states,
            tally.first_slots_enough_and_aligned_pairs_short_step.map_or_else(|| "none".to_string(), |step| step.to_string()),
            tally.aligned_pairs_peak,
            tally.aligned_pairs_positive_states,
            text_of_unsigned(tally.aligned_pairs_final_minimum),
            tally.unblocked_slots_peak,
            text_of_unsigned(tally.unblocked_slots_final_minimum),
        ));
        total.merge(&tally);
    }
    emit_aligned_pair_shortage_summary(geometry.name, seeds, steps, &total);
    Ok(total)
}

fn text_of_unsigned(value: Option<u64>) -> String {
    value.map_or_else(|| "none".to_string(), |number| number.to_string())
}

fn number_from_the_environment(variable: &str, default: u64) -> u64 {
    match std::env::var(variable) {
        Ok(text) => text.parse::<u64>().unwrap_or_else(|error| {
            panic!("环境变量 {variable} 要是十进制整数：{text:?}：{error}")
        }),
        Err(_unset) => default,
    }
}

fn emit_aligned_pair_shortage_summary(geometry: &str, seeds: u64, steps: u64, total: &WalkTally) {
    let refusals: Vec<String> = total
        .action_refusals
        .iter()
        .map(|(action, count)| format!("{action}={count}"))
        .collect();
    let today: Vec<String> = total
        .slots_enough_and_aligned_pairs_short_today_outcomes
        .iter()
        .map(|(outcome, count)| format!("{outcome}={count}"))
        .collect();
    let session: Vec<String> = total
        .slots_enough_and_aligned_pairs_short_session_outcomes
        .iter()
        .map(|(outcome, count)| format!("{outcome}={count}"))
        .collect();
    emit(&format!(
        "name=aligned_pair_shortage_summary geometry={geometry} seeds={seeds} steps={steps} walks={} states={} slots_enough_on_every_device_states={} aligned_pairs_short_states={} slots_enough_and_aligned_pairs_short_states={} first_slots_enough_and_aligned_pairs_short_step={} aligned_pairs_peak={} aligned_pairs_positive_states={} unblocked_slots_peak={} action_refusals=[{}] slots_enough_and_aligned_pairs_short_today_outcomes=[{}] slots_enough_and_aligned_pairs_short_session_outcomes=[{}] model_checks={} placement_refusals_with_slots_enough_and_aligned_pairs_short_false={} slots_enough_and_aligned_pairs_short_probes_that_published={}",
        total.walks,
        total.states,
        total.slots_enough_states,
        total.pairs_short_states,
        total.slots_enough_and_aligned_pairs_short_states,
        total.first_slots_enough_and_aligned_pairs_short_step.map_or_else(|| "none".to_string(), |step| step.to_string()),
        total.aligned_pairs_peak,
        total.aligned_pairs_positive_states,
        total.unblocked_slots_peak,
        refusals.join(","),
        today.join(","),
        session.join(","),
        total.model_checks,
        total.placement_refusals_with_slots_enough_and_aligned_pairs_short_false,
        total.slots_enough_and_aligned_pairs_short_probes_that_published,
    ));
    emit(&format!(
        "name=verdict_aligned_pair_shortage geometry={geometry} slots_enough_and_aligned_pairs_short_reachable={} slots_enough_and_aligned_pairs_short_states={} slots_enough_and_pairs_short_never_together={}",
        slots_enough_and_aligned_pairs_short_is_reachable_given_threshold(total.slots_enough_and_aligned_pairs_short_states, MINIMUM_SLOTS_ENOUGH_AND_ALIGNED_PAIRS_SHORT_STATES_FOR_REACHABLE),
        total.slots_enough_and_aligned_pairs_short_states,
        total.slots_enough_and_aligned_pairs_short_states == 0
    ));
}

/// 准入判法退 78（拒绝）时装置也退 78；别的非 0 退 1（`.claude/singlefs-ai-sop/rules/preflight-discipline.md`「开头先判」Rust 那一行）。
const PREFLIGHT_REFUSED_EXIT_CODE: i32 = 78;

/// `main` 的第一句：直接起 `python3 preflight.py check <本文件> [--force] -- <参数…>`（不经 `sh -c`）判准入与运行条件，
/// 不满足就照判法的退出码退出、一步都不做；强制跑（判法的第一行以 `forced` 起头）时交回没满足的条件摘要，要写进产物。
fn preflight() -> Option<String> {
    let manifest_directory = env!("CARGO_MANIFEST_DIR");
    let source = format!(
        "{manifest_directory}/src/bin/e181_landing_check_fixed_point_and_mount_lower_bound.rs"
    );
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
        std::process::exit(PREFLIGHT_REFUSED_EXIT_CODE)
    });
    eprint!("{}", String::from_utf8_lossy(&output.stderr));
    match output.status.code() {
        Some(0) => {}
        Some(PREFLIGHT_REFUSED_EXIT_CODE) => std::process::exit(PREFLIGHT_REFUSED_EXIT_CODE),
        Some(_) | None => std::process::exit(1),
    }
    let verdict_line = String::from_utf8_lossy(&output.stdout)
        .lines()
        .next()
        .unwrap_or("")
        .to_string();
    forced_summary_of_the_preflight_verdict(&verdict_line)
}

/// 准入判法 stdout 第一行（制表符分段）以 `forced` 起头时，交回没满足的条件摘要（最后一段，空白换成下划线），要写进产物；
/// 满足（`met`）或没声明时是 `None`。
fn forced_summary_of_the_preflight_verdict(verdict_line: &str) -> Option<String> {
    if verdict_line.starts_with("forced") {
        let last_segment = verdict_line.rsplit('\t').next().unwrap_or("");
        Some(
            last_segment
                .split_whitespace()
                .collect::<Vec<_>>()
                .join("_"),
        )
    } else {
        None
    }
}

fn main() {
    let forced_summary = preflight();
    if let Some(summary) = forced_summary {
        emit(&format!("name=preflight forced={summary}"));
    }
    let requested: Vec<String> = std::env::args()
        .skip(1)
        .filter(|argument| argument != "--force")
        .collect();
    let sections: Vec<&str> = if requested.is_empty() {
        vec![
            "constants",
            "anchors",
            "selfcheck",
            "f10-controls-small",
            "f11",
            "f9",
            "f10-measure",
            "f10-controls",
            "f10-narrow",
            "f10-unequal",
            "f10-small",
        ]
    } else {
        requested.iter().map(String::as_str).collect()
    };
    let mut content = ContentBuffer::new();
    for section in sections {
        let started = Instant::now();
        let result = match section {
            "constants" => check_local_constants_against_the_crates(),
            "anchors" => run_anchors(),
            "selfcheck" => run_selfcheck(),
            "f9" => run_fixed_point_upper_bound(&mut content),
            "f11" => run_per_device_lowest_slot(&mut content),
            "f10-measure" => run_aligned_pair_shortage_measure(&mut content),
            "f10-controls" => run_aligned_pair_shortage_controls(GEOMETRY_NARROW, &mut content),
            "f10-controls-small" => {
                run_aligned_pair_shortage_controls(GEOMETRY_SMALL, &mut content)
            }
            "f10-narrow" => {
                run_aligned_pair_shortage_walks(GEOMETRY_NARROW, &mut content).map(drop)
            }
            "f10-unequal" => {
                run_aligned_pair_shortage_walks(GEOMETRY_UNEQUAL, &mut content).map(drop)
            }
            "f10-small" => run_aligned_pair_shortage_walks(GEOMETRY_SMALL, &mut content).map(drop),
            unknown => Err(format!("不认识的段 {unknown}")),
        };
        match result {
            Ok(()) => {
                // 段用时只打到标准错误：它随机器忙闲变，进产物会让登记的逐字节复跑的那几段每次不同。
                eprintln!(
                    "[e181] 段 {section} 用了 {:.1} 秒",
                    started.elapsed().as_secs_f64()
                );
                emit(&format!("name=section_done section={section}"));
            }
            Err(reason) => {
                emit(&format!("name=stop section={section} reason={reason:?}"));
                std::process::exit(3);
            }
        }
    }
    emit_the_done_line();
}

/// 钉绝对值的断言与对拍（登记第七节 A1–A4、B1–B3，加装置自己的判法）。写成普通函数，单测（`tests` 模块）与产物里的 `selfcheck` 段各调一遍：
/// 单测归提交时的 checker 档，`selfcheck` 在装置里直接跑，红在哪一条看产物里的 `name=selfcheck` 行。
mod claims {
    use super::*;
    use singlefs_core::allocator::PlacementRefusal;

    fn first_file_pool(geometry: Geometry) -> Pool {
        build_pool_after_the_first_file(geometry, CodeTwoTreeNodeCapacities::FromTheNodeFormat)
    }

    /// A1 与 B2：两块 4 GiB 盘单元区 [50176, 262144)，层级 0 每盘罩 262 个位置（⌊262143 ÷ 812⌋ − ⌊50176 ÷ 812⌋ + 1 = 322 − 61 + 1）、
    /// 层级 1 每盘罩 2 个（⌊262143 ÷ 137228⌋ − ⌊50176 ÷ 137228⌋ + 1 = 1 − 0 + 1）、层级 2 每盘 1 个；根层级 1、2、3 的 K1 各是 525、529、531。
    pub fn allocation_record_tree_nodes_of_two_four_gibibyte_devices_are_525_529_and_531_at_root_levels_one_two_and_three(
    ) {
        let unit_areas = [(50176, 211_968), (50176, 211_968)];
        assert_eq!(
            local_allocation_record_tree_nodes(1, &unit_areas),
            1 + 2 * 262
        );
        assert_eq!(local_allocation_record_tree_nodes(2, &unit_areas), 529);
        assert_eq!(
            local_allocation_record_tree_nodes(3, &unit_areas),
            1 + 2 * (262 + 2 + 1)
        );
        assert_eq!(
            local_allocation_record_tree_nodes(1, &unit_areas),
            525,
            "根层级 1 的分配记录树节点数是 525（手算 1 + 2 × 262）"
        );
        assert_eq!(
            local_allocation_record_tree_nodes(3, &unit_areas),
            531,
            "根层级 3 的分配记录树节点数是 531（手算 1 + 2 × 265）"
        );
    }

    /// 小几何单元区 [7168, 27648)：层级 0 罩 ⌊27647 ÷ 812⌋ − ⌊7168 ÷ 812⌋ + 1 = 34 − 8 + 1 = 27 个，层级 1 罩 1 个。
    pub fn allocation_record_tree_nodes_of_the_small_geometry_are_55_at_root_level_one_and_57_at_root_level_two(
    ) {
        let unit_areas = [(7168, 20_480), (7168, 20_480)];
        assert_eq!(local_allocation_record_tree_nodes(1, &unit_areas), 55);
        assert_eq!(local_allocation_record_tree_nodes(2, &unit_areas), 57);
    }

    /// 手算：每层节点数 [3, 1]、容量 8 / 4、已有 100 条、删 4 插 6：叶层 3 个节点各至少切一次（⌈8 ÷ 2⌉ = 4 条才再切）切 3 次，
    /// 重写 min(3, 10) + 3 = 6；内部层 3 + 3 = 6 个孩子 > 4，原有 1 个节点至少切 1 次、其余 3 − 1 = 2 条每 2 条切一次，共 2 次，
    /// 重写 min(1, 10) + 2 = 3；根切开长高，新根起步 3 个孩子 ≤ 4，不再切，加 1。合计 6 + 3 + 1 = 10。
    pub fn central_mapping_nodes_rewritten_at_most_pins_the_hand_computed_examples() {
        let capacity = SMALL_TREE_NODE_CAPACITY;
        assert_eq!(
            local_central_mapping_tree_nodes_rewritten_at_most(&[3, 1], 100, 4, 6, capacity),
            10
        );
        // 一片叶、已有 5 条、删 1 插 1：装得下，不切，重写 min(1, 2) = 1。
        assert_eq!(
            local_central_mapping_tree_nodes_rewritten_at_most(&[1], 5, 1, 1, capacity),
            1
        );
        // 一片叶、已有 8 条（满）、插 4：切 1 次，重写 min(1, 4) + 1 = 2，根切开长高加 1 个新根，合计 3。
        assert_eq!(
            local_central_mapping_tree_nodes_rewritten_at_most(&[1], 8, 0, 4, capacity),
            3
        );
        assert_eq!(
            local_splits_at_one_level_at_most(2, 5, 3),
            3,
            "2 + ⌊(5 − 2) ÷ 3⌋ = 3"
        );
    }

    pub fn sparse_layers_share_reads_and_keep_writes_apart() {
        let mut base = ForkedDevice::empty(1 << 20);
        let first = vec![7_u8; 1024];
        base.write_at(DeviceOffsetInBytes(512), &first, WriteDurability::Plain)
            .expect("写");
        let mut fork = base.fork();
        let mut read = vec![0_u8; 1024];
        fork.read_at(DeviceOffsetInBytes(512), &mut read)
            .expect("读");
        assert_eq!(read, first, "拷贝读到封存的层");
        fork.write_at(
            DeviceOffsetInBytes(512),
            &[9_u8; 512],
            WriteDurability::Plain,
        )
        .expect("写");
        base.read_at(DeviceOffsetInBytes(512), &mut read)
            .expect("读");
        assert_eq!(read, first, "拷贝上的写不进原来的盘");
        fork.read_at(DeviceOffsetInBytes(512), &mut read)
            .expect("读");
        assert_eq!(&read[..512], &[9_u8; 512][..], "拷贝读到自己的写");
        assert_eq!(&read[512..], &[7_u8; 512][..], "没写的扇区读到封存的层");
    }

    pub fn zeroing_masks_the_sealed_layers_and_a_later_write_shows_through_the_mask() {
        let mut base = ForkedDevice::empty(1 << 20);
        base.write_at(
            DeviceOffsetInBytes(0),
            &[5_u8; 2048],
            WriteDurability::Plain,
        )
        .expect("写");
        let mut fork = base.fork();
        fork.write_zeroes_at(DeviceOffsetInBytes(512), 1024)
            .expect("清零");
        let mut read = vec![1_u8; 2048];
        fork.read_at(DeviceOffsetInBytes(0), &mut read).expect("读");
        assert_eq!(&read[..512], &[5_u8; 512][..]);
        assert_eq!(&read[512..1536], &[0_u8; 1024][..], "清零盖住封存的层");
        assert_eq!(&read[1536..], &[5_u8; 512][..]);
        fork.write_at(
            DeviceOffsetInBytes(1024),
            &[3_u8; 512],
            WriteDurability::Plain,
        )
        .expect("写");
        fork.read_at(DeviceOffsetInBytes(0), &mut read).expect("读");
        assert_eq!(&read[1024..1536], &[3_u8; 512][..], "清零之后的写读得到");
        assert_eq!(&read[512..1024], &[0_u8; 512][..]);
        let outcome = fork
            .discard_at(DeviceOffsetInBytes(0), 512)
            .expect("discard");
        assert_eq!(outcome, DiscardOutcome::Applied);
        fork.read_at(DeviceOffsetInBytes(0), &mut read).expect("读");
        assert_eq!(&read[..512], &[0_u8; 512][..], "discard 当整段清零");
        assert!(matches!(
            fork.read_at(DeviceOffsetInBytes(100), &mut read),
            Err(BlockDeviceError::Unaligned { .. })
        ));
        assert!(matches!(
            fork.read_at(DeviceOffsetInBytes(1 << 20), &mut read[..512]),
            Err(BlockDeviceError::OutOfRange { .. })
        ));
    }

    /// A3 与 A4：mkfs 同一个进程里写完第一个文件（3000 字节）之后，第一个数据单元在 50180，每块盘最低落点 50182，
    /// 盘 0 上账里记录的最大末槽（不含）是 50253（D3 已定项 10 ⑤ 落点表：树表单元落 50252）。
    pub fn the_first_file_lands_at_50180_and_its_records_end_at_50253() {
        let pool = first_file_pool(GEOMETRY_DEFAULT);
        let first_unit = pool
            .current
            .units
            .iter()
            .find(|unit| unit.identity == TransactionUnit::Data(DataUnitIndexInFile::FIRST))
            .expect("第一个数据单元");
        assert_eq!(first_unit.slot, SlotNumber(50180));
        for device_map in &pool.allocator.devices {
            assert_eq!(
                device_map.lowest_user_data_slot(pool.allocator.cluster_segments()),
                Some(SlotNumber(50182))
            );
        }
        let largest_end = pool
            .current
            .allocation_records
            .iter()
            .filter(|record| record.device == DeviceIdentity(0))
            .map(|record| record.slot.0 + u64::from(record.span_slots))
            .max();
        assert_eq!(largest_end, Some(50253));
    }

    /// 第一个文件那次发布：普通分配 6 槽（数据 2 + extent 树根 1 + inode 叶容器 2 + inode 树根 1），固定点 8 槽
    /// （分配记录树 5 个节点 + 记账树 1 + 中央映射树 1 + 树表 1，D3 已定项 10 ⑤ 落点表 t5–t12）；两条路径（角色、记录）逐盘相等。
    pub fn the_first_file_publish_has_six_ordinary_and_eight_fixed_point_slots_by_both_paths() {
        let pool = first_file_pool(GEOMETRY_DEFAULT);
        let account = fixed_point_account(&pool.current, &device_identities(&pool));
        assert_eq!(account.ordinary_slots, 6);
        assert_eq!(account.by_roles, 8);
        assert_eq!(account.instance_switch_slots, 0);
        assert_eq!(
            account.by_records_per_device,
            vec![(DeviceIdentity(0), 8), (DeviceIdentity(1), 8)]
        );
    }

    /// B3：本地 ckpt_cost（普通分配删插为 0）与核心层的同名量在第一个文件那一版、一次换写之后的一版上逐个相等，三个几何、两档容量。
    pub fn the_local_checkpoint_cost_equals_the_core_one_on_every_version_measured() {
        let mut content = ContentBuffer::new();
        for (geometry, capacities) in [
            (
                GEOMETRY_DEFAULT,
                CodeTwoTreeNodeCapacities::FromTheNodeFormat,
            ),
            (
                GEOMETRY_NARROW,
                CodeTwoTreeNodeCapacities::FromTheNodeFormat,
            ),
            (GEOMETRY_SMALL, CodeTwoTreeNodeCapacities::FromTheNodeFormat),
            (GEOMETRY_DEFAULT, SMALL_TREE_NODE_CAPACITIES),
        ] {
            let mut pool = build_pool_after_the_first_file(geometry, capacities);
            for units in [3_usize, 70, 200] {
                let core = core_checkpoint_cost(&pool.current, &pool.allocator, capacities);
                let local = local_checkpoint_cost(
                    &pool.current,
                    &pool.allocator,
                    capacities,
                    OrdinaryMappingChanges::default(),
                );
                assert_eq!(local.total, core, "{} 容量 {capacities:?}", geometry.name);
                let outcome = write_units_and_account(&mut pool, units, capacities, &mut content)
                    .expect("本地与核心层的 checkpoint 代价对拍过");
                assert!(matches!(outcome, WriteOutcome::Published(_)));
            }
        }
    }

    /// 这批普通分配的删插：第一个文件那一版有 4 个进映射的普通分配单元（数据、extent 树根、inode 叶容器、inode 树根），
    /// 换写成 70 个单元后全部换下（删 4），插的 = 这次重写的角色里进映射的普通分配角色数。
    pub fn a_growth_write_deletes_the_four_first_file_units_and_inserts_every_rewritten_mapped_ordinary_role(
    ) {
        let mut pool = first_file_pool(GEOMETRY_DEFAULT);
        let mut content = ContentBuffer::new();
        let previous = pool.current.clone();
        let outcome = write_units_and_account(
            &mut pool,
            70,
            CodeTwoTreeNodeCapacities::FromTheNodeFormat,
            &mut content,
        )
        .expect("对拍过");
        let WriteOutcome::Published(write) = outcome else {
            panic!("70 个单元的写发得成");
        };
        assert_eq!(write.ordinary.deleted, 4);
        let mapped_ordinary_roles: BTreeSet<TransactionUnit> = pool
            .current
            .mapped_units
            .iter()
            .map(|(role, _key)| *role)
            .filter(|role| space_budget_of_role(*role) == SpaceBudgetOfARole::OrdinaryAllocation)
            .collect();
        let rewritten_and_mapped = pool
            .current
            .rewritten
            .iter()
            .filter(|role| mapped_ordinary_roles.contains(role))
            .count();
        assert_eq!(
            write.ordinary.inserted,
            u64::try_from(rewritten_and_mapped).expect("条数")
        );
        assert!(
            write.ordinary.inserted >= 70,
            "至少 70 个数据单元的映射条目"
        );
        assert_eq!(
            ordinary_mapping_changes(&previous, &pool.current),
            write.ordinary
        );
    }

    /// 压小容量的三层树上，一次加 64 个单元的写：P1 比 ckpt_cost 多（Q3 > 0），因为中央映射树每层多了普通分配的删插；
    /// 去掉 P1 里普通分配的删插，P1 就退回 ckpt_cost（Q3 = 0）。
    pub fn a_sixty_four_unit_write_on_the_small_capacity_trees_makes_the_checkpoint_cost_with_ordinary_mapping_changes_exceed_the_checkpoint_cost(
    ) {
        let mut pool =
            build_pool_after_the_first_file(GEOMETRY_DEFAULT, SMALL_TREE_NODE_CAPACITIES);
        let mut content = ContentBuffer::new();
        let outcome =
            write_units_and_account(&mut pool, 65, SMALL_TREE_NODE_CAPACITIES, &mut content)
                .expect("对拍过");
        let WriteOutcome::Published(write) = outcome else {
            panic!("65 个单元的写发得成");
        };
        assert!(
            write.checkpoint_cost_with_ordinary_mapping_changes_minus_checkpoint_cost() > 0,
            "{write:?}"
        );
        assert_eq!(write.local_checkpoint_cost, write.checkpoint_cost);
        assert!(
            write.checkpoint_cost_with_ordinary_mapping_changes_minus_true() >= 0,
            "带普通分配删插的 checkpoint 代价在这一次写上是上界：{write:?}"
        );
    }

    /// 小几何、压小容量的树上写了 1500 个单元之后，中央映射树的叶比 ckpt_cost 自己的删插数（K1 加记账树节点，几十把）多得多：
    /// 这时多删的普通分配条目、多插的普通分配条目各自都让本地 ckpt_cost 变大（每层可能改的路径数是 min(这一层的节点数, 删 + 插)，插的还多切节点）。
    pub fn extra_deletes_and_inserts_each_raise_the_local_checkpoint_cost_on_a_wide_tree() {
        let mut pool = build_pool_after_the_first_file(GEOMETRY_SMALL, SMALL_TREE_NODE_CAPACITIES);
        let mut content = ContentBuffer::new();
        let outcome =
            write_units_and_account(&mut pool, 1500, SMALL_TREE_NODE_CAPACITIES, &mut content)
                .expect("对拍过");
        assert!(matches!(outcome, WriteOutcome::Published(_)));
        let cost = |extra: OrdinaryMappingChanges| {
            local_checkpoint_cost(
                &pool.current,
                &pool.allocator,
                SMALL_TREE_NODE_CAPACITIES,
                extra,
            )
            .total
        };
        let without_extra = cost(OrdinaryMappingChanges::default());
        let with_deletes = cost(OrdinaryMappingChanges {
            deleted: 1000,
            inserted: 0,
        });
        let with_inserts = cost(OrdinaryMappingChanges {
            deleted: 0,
            inserted: 1000,
        });
        assert!(
            with_deletes > without_extra,
            "多删 1000 条：{with_deletes} 对 {without_extra}"
        );
        assert!(
            with_inserts > without_extra,
            "多插 1000 条：{with_inserts} 对 {without_extra}"
        );
    }

    pub fn the_judges_flip_when_the_threshold_moves_between_two_readings() {
        let mut upper = WriteTally::default();
        let mut lower = WriteTally::default();
        let sample =
            |checkpoint_cost_with_ordinary_mapping_changes_minus_true: i64| PublishedWrite {
                units: 1,
                height_before: 1,
                height_after: 1,
                true_fixed_point: 10,
                true_fixed_point_by_records: Vec::new(),
                checkpoint_cost: 10,
                local_checkpoint_cost: 10,
                checkpoint_cost_with_ordinary_mapping_changes: u64::try_from(
                    10 + checkpoint_cost_with_ordinary_mapping_changes_minus_true,
                )
                .expect("非负"),
                ordinary: OrdinaryMappingChanges::default(),
            };
        upper.add(&sample(3));
        lower.add(&sample(0));
        assert_eq!(
            checkpoint_cost_with_ordinary_mapping_changes_is_an_upper_bound_on_every_write(&upper),
            Some(true)
        );
        assert_eq!(
            checkpoint_cost_with_ordinary_mapping_changes_is_an_upper_bound_on_every_write(&lower),
            Some(true)
        );
        assert_eq!(
            checkpoint_cost_with_ordinary_mapping_changes_is_an_upper_bound_given_threshold(
                &upper, 1
            ),
            Some(true)
        );
        assert_eq!(
            checkpoint_cost_with_ordinary_mapping_changes_is_an_upper_bound_given_threshold(
                &lower, 1
            ),
            Some(false),
            "门槛挪到两个读数（3 与 0）之间，0 那一边必须翻"
        );
        assert_eq!(
            checkpoint_cost_with_ordinary_mapping_changes_is_an_upper_bound_on_every_write(
                &WriteTally::default()
            ),
            None
        );
        let mut below = WriteTally::default();
        below.add(&sample(-1));
        assert_eq!(
            checkpoint_cost_with_ordinary_mapping_changes_is_an_upper_bound_on_every_write(&below),
            Some(false)
        );
        assert_eq!(
            below.checkpoint_cost_with_ordinary_mapping_changes_below_true_writes,
            1
        );
        assert!(slots_enough_and_aligned_pairs_short_is_reachable_given_threshold(1, 1));
        assert!(!slots_enough_and_aligned_pairs_short_is_reachable_given_threshold(0, 1));
        assert!(
            !slots_enough_and_aligned_pairs_short_is_reachable_given_threshold(1, 2),
            "门槛挪到 1 与 2 之间，1 个状态那一边必须翻"
        );
    }

    pub fn slots_enough_and_aligned_pairs_short_holds_only_when_the_slots_are_enough_everywhere_and_the_aligned_pairs_are_short_somewhere(
    ) {
        let reading = |aligned: [u64; 2], unblocked: [u64; 2]| RoomReading {
            aligned_free_pairs: aligned.to_vec(),
            unblocked_slots: unblocked.to_vec(),
            needed_slots: 400,
            needed_pairs: 1,
        };
        assert!(reading([0, 5], [500, 500]).holds());
        assert!(
            reading([0, 0], [400, 400]).holds(),
            "没挡的槽恰等于要的槽，够"
        );
        assert!(
            !reading([0, 0], [399, 500]).holds(),
            "一块盘的槽差 1 个，不够"
        );
        assert!(
            !reading([1, 1], [500, 500]).holds(),
            "对齐的空槽对恰等于要的个数，够"
        );
        assert!(reading([0, 1], [500, 500]).aligned_pairs_are_short_on_some_device());
        assert!(!reading([1, 2], [500, 500]).aligned_pairs_are_short_on_some_device());
    }

    pub fn marking_every_pair_outside_the_clusters_leaves_no_lowest_slot_and_marking_none_leaves_50182(
    ) {
        let mut pool = first_file_pool(GEOMETRY_DEFAULT);
        let untouched = unequal_landing_reading(&pool);
        assert_eq!(
            untouched.lowest_slot_per_device,
            vec![Some(50182), Some(50182)]
        );
        assert!(!untouched.lowest_slots_differ);
        assert!(untouched.one_data_unit_lands);
        let scan_before = time_the_lowest_user_data_slot(&pool.allocator, 0, 3);
        assert_eq!(scan_before.lowest_slot, Some(50182));
        assert_eq!(
            scan_before.scanned_candidates,
            (50182 - 50176) / 2 + 1,
            "从单元区起点按 2 槽步进到 50182：50176、50178、50180、50182 四个候选"
        );
        mark_the_lowest_pairs_outside_the_clusters(&mut pool.allocator, 100);
        let full = unequal_landing_reading(&pool);
        assert_eq!(
            full.lowest_slot_per_device,
            vec![None, None],
            "开放段里还有空槽对，但落点不进开过的段"
        );
        assert!(
            !full.one_data_unit_lands,
            "段外一对都不剩，一个数据单元也落不下"
        );
        let scan = time_the_lowest_user_data_slot(&pool.allocator, 0, 3);
        assert_eq!(scan.lowest_slot, None);
        assert_eq!(scan.scanned_candidates, 211_968 / 2, "一直扫到单元区末尾");
    }

    /// 对齐的空槽对数把聚簇段内外都算：开放段 [50240, 50304) 里还空着的槽对多出来。
    pub fn aligned_free_pairs_count_the_open_segment_too() {
        let mut pool = first_file_pool(GEOMETRY_DEFAULT);
        let mut state = session_pool_of(&mut pool).expect("可写挂载");
        let reading = RoomReading::of(&state);
        let device_map = &state.session.allocator.devices[0];
        let mut free_pairs_by_scan = 0;
        let first = device_map.unit_area_start().slot().0;
        let mut slot = first;
        while slot + 1 < first + device_map.unit_area_slots() {
            if device_map.is_free(SlotNumber(slot)) && device_map.is_free(SlotNumber(slot + 1)) {
                free_pairs_by_scan += 1;
            }
            slot += 2;
        }
        assert_eq!(
            reading.aligned_free_pairs[0], free_pairs_by_scan,
            "逐槽扫一遍数出来的"
        );
        let outside = data_unit_pairs_outside_the_clusters_of(&state.session.allocator, 0);
        assert!(
            reading.aligned_free_pairs[0] > outside,
            "开放段里有空槽对：{} 对 {outside}",
            reading.aligned_free_pairs[0]
        );
        assert!(state.created_inodes.is_empty());
        let result = apply_walk_action(&mut state, WalkAction::CreateOneInode).expect("走法");
        assert_eq!(result, "published");
        assert_eq!(state.created_inodes.len(), 1);
    }

    pub fn the_seeded_stream_repeats_for_a_seed_and_differs_between_seeds() {
        let mut first = SeededStream::of_seed(1);
        let mut again = SeededStream::of_seed(1);
        let mut other = SeededStream::of_seed(2);
        let drawn: Vec<u64> = (0..8).map(|_| first.below(5)).collect();
        let repeated: Vec<u64> = (0..8).map(|_| again.below(5)).collect();
        let different: Vec<u64> = (0..8).map(|_| other.below(5)).collect();
        assert_eq!(drawn, repeated);
        assert_ne!(drawn, different);
        assert!(drawn.iter().all(|number| *number < 5));
    }

    pub fn refusals_are_named_by_the_gate_that_refused() {
        let placement = PublishError::PlacementRefused {
            unit: TransactionUnit::TreeTable,
            refusal: PlacementRefusal::NoFreeSlotOnAnyDevice,
        };
        assert_eq!(
            RefusalClass::of(&placement).name(),
            "placement_refused_in_the_fixed_point"
        );
        assert!(passed_the_landing_check_and_the_formula(
            &WriteOutcome::Refused(RefusalClass::of(&placement))
        ));
        assert!(!passed_the_landing_check_and_the_formula(
            &WriteOutcome::Refused(RefusalClass::SpaceAdmission {
                short_devices: String::new(),
                shortfall_bytes: 0
            })
        ));
        assert_eq!(
            variant_name_of("refused_NoSpace(Box { .. })"),
            "refused_NoSpace"
        );
        assert_eq!(signed_difference(3, 5), -2);
    }

    /// 被拒之后下一次试几个单元：journal 记录数超限减去超出的条数（登记 12.4 的读数：65877 个单元发了 66167 条、上限 65536，超出 631），
    /// 缺口按一个单元在一块盘上的 32768 字节折算、向上取整，落点那一道折半；结果夹进 [1, 上一次 − 1]，单元数严格减小。
    pub fn the_units_to_try_after_a_refusal_follow_the_surplus_the_shortfall_or_half() {
        let over_the_limit = RefusalClass::Other {
            description: String::new(),
            journal_records_over_the_limit: Some(631),
        };
        assert_eq!(
            units_to_try_after_a_refusal(&over_the_limit, 65_877),
            65_246
        );
        let without_a_reading = RefusalClass::Other {
            description: String::new(),
            journal_records_over_the_limit: None,
        };
        assert_eq!(units_to_try_after_a_refusal(&without_a_reading, 100), 50);
        let in_placement = RefusalClass::FixedPointPlacement {
            unit: String::new(),
            refusal: String::new(),
        };
        assert_eq!(units_to_try_after_a_refusal(&in_placement, 100), 50);
        let short = |shortfall_bytes: i128| RefusalClass::SpaceAdmission {
            short_devices: String::new(),
            shortfall_bytes,
        };
        assert_eq!(units_to_try_after_a_refusal(&short(32_768), 1000), 999);
        assert_eq!(units_to_try_after_a_refusal(&short(32_769), 1000), 998);
        assert_eq!(units_to_try_after_a_refusal(&short(3 * 32_768), 1000), 997);
        assert_eq!(
            units_to_try_after_a_refusal(&short(0), 1000),
            999,
            "缺口读数不是正数时按一字节算"
        );
        assert_eq!(
            units_to_try_after_a_refusal(&short(40_000_000), 10),
            1,
            "算出负数时夹到 1"
        );
        assert_eq!(
            units_to_try_after_a_refusal(&over_the_limit, 400),
            1,
            "超出的条数比单元数还多时夹到 1"
        );
        assert_eq!(units_to_try_after_a_refusal(&over_the_limit, 2), 1);
        assert_eq!(units_to_try_after_a_refusal(&without_a_reading, 3), 1);
    }

    /// 填满（登记 12.11）在小几何上：每次发布至多 3000 个单元，所以要好几次发布才填得下；换下的上一批仍占着槽
    /// （各盘 defer 队列里至少有最早那次发布的 2 槽 × 单元数）；账里剩下的各盘段外对齐空槽对数等于分配器此刻的读数；
    /// 停在「段外一个对齐的空槽对都没剩」或「核心层拒了一个单元的写」：后一种在账里的状态上再发一个单元的写，照样被拒。
    pub fn a_capped_fill_of_the_small_geometry_publishes_several_times_and_ends_with_nothing_left_or_a_refused_single_unit(
    ) {
        let mut content = ContentBuffer::new();
        let mut pool = first_file_pool(GEOMETRY_SMALL);
        let report = fill_the_data_side(&mut pool, &mut content, 3000).expect("填满不触发停机");
        assert!(report.publishes.len() >= 3, "{report:?}");
        assert!(
            report.publishes.iter().all(|publish| publish.units >= 1
                && publish.units <= publish.units_asked
                && publish.units_asked <= 3000),
            "{report:?}"
        );
        assert_eq!(
            report.remaining_pairs_outside_the_clusters,
            remaining_pairs_outside_the_clusters(&pool),
            "账里剩的槽对数是分配器此刻的读数"
        );
        let slots_of_the_earliest_batch = DATA_UNIT_SLOTS_LOCAL * report.publishes[0].units;
        assert!(
            report
                .deferred_slots
                .iter()
                .all(|deferred| *deferred >= slots_of_the_earliest_batch),
            "换下的最早那一批仍占着槽：{report:?}"
        );
        let fewest = report
            .remaining_pairs_outside_the_clusters
            .iter()
            .copied()
            .min()
            .expect("至少一块盘");
        match &report.stop {
            FillStop::NoAlignedPairOutsideTheClusters => assert_eq!(fewest, 0, "{report:?}"),
            FillStop::Refused { units_asked, .. } => {
                assert!(fewest > 0, "{report:?}");
                assert_eq!(*units_asked, 1, "连一个单元也落不下才停：{report:?}");
                let mut probe = pool.fork();
                let outcome = write_units_and_account(
                    &mut probe,
                    1,
                    CodeTwoTreeNodeCapacities::FromTheNodeFormat,
                    &mut content,
                )
                .expect("对拍过");
                assert!(matches!(outcome, WriteOutcome::Refused(_)), "{report:?}");
            }
            FillStop::PublishLimitReached => panic!("发布次数到了保险丝：{report:?}"),
        }
    }

    fn published_outcome() -> WriteOutcome {
        WriteOutcome::Published(Box::new(PublishedWrite {
            units: 1,
            height_before: 1,
            height_after: 1,
            true_fixed_point: 0,
            true_fixed_point_by_records: Vec::new(),
            checkpoint_cost: 0,
            local_checkpoint_cost: 0,
            checkpoint_cost_with_ordinary_mapping_changes: 0,
            ordinary: OrdinaryMappingChanges::default(),
        }))
    }

    fn refused_by_space(shortfall_bytes: i128) -> WriteOutcome {
        WriteOutcome::Refused(RefusalClass::SpaceAdmission {
            short_devices: String::new(),
            shortfall_bytes,
        })
    }

    fn refused_in_the_fixed_point() -> WriteOutcome {
        WriteOutcome::Refused(RefusalClass::FixedPointPlacement {
            unit: String::new(),
            refusal: String::new(),
        })
    }

    fn refused_without_a_gap() -> WriteOutcome {
        WriteOutcome::Refused(RefusalClass::Other {
            description: String::new(),
            journal_records_over_the_limit: None,
        })
    }

    /// 合成的盘面：单元数不超过 `edge` 减 3 的写发成，`edge` 减 2 到 `edge` 的三个被拒在固定点（也算放行），更大的被空间准入拒，
    /// 缺口 = 超出的单元数 × 32768 × `bytes_per_excess_unit_factor` − 5 字节（减 5 是为了折单元数时向上取整才对得上）。
    fn outcome_on_a_synthetic_base_with_a_gap(
        units: usize,
        edge: usize,
        bytes_per_excess_unit_factor: i128,
    ) -> WriteOutcome {
        if units + 3 <= edge {
            published_outcome()
        } else if units <= edge {
            refused_in_the_fixed_point()
        } else {
            let excess = i128::try_from(units - edge).expect("单元数装得进 i128");
            refused_by_space(excess * 32_768 * bytes_per_excess_unit_factor - 5)
        }
    }

    /// 同上，被拒的都不带缺口读数（别的拒因）。
    fn outcome_on_a_synthetic_base_without_a_gap(units: usize, edge: usize) -> WriteOutcome {
        if units + 3 <= edge {
            published_outcome()
        } else if units <= edge {
            refused_in_the_fixed_point()
        } else {
            refused_without_a_gap()
        }
    }

    /// 找下界（登记 12.12）：先在填过的底上试填过的单元数，不把它直接当放行；被拒按拒因往下找，到第一个放行的数；
    /// 固定点那一道拒也算放行（与二分同一个判法）；一路试到 1 个单元、或试满次数都没放行，交 `NotFound`。
    pub fn the_lower_bound_is_tried_on_the_filled_base_first_and_follows_the_refusals_down_to_a_passing_count(
    ) {
        {
            let mut tried = Vec::new();
            let found = find_a_lower_bound_that_passes(1000, |units| {
                tried.push(units);
                Ok(published_outcome())
            })
            .expect("试写不出错");
            let LowerBoundSearch::Found {
                units,
                refused_before,
            } = found
            else {
                panic!("填过的单元数放行，该找到");
            };
            assert_eq!((units, refused_before.len()), (1000, 0));
            assert_eq!(tried, vec![1000], "填过的单元数自己先试一次");
        }
        {
            let mut tried = Vec::new();
            let found = find_a_lower_bound_that_passes(1000, |units| {
                tried.push(units);
                Ok(if units > 997 {
                    refused_by_space(3 * 32_768)
                } else {
                    published_outcome()
                })
            })
            .expect("试写不出错");
            let LowerBoundSearch::Found {
                units,
                refused_before,
            } = found
            else {
                panic!("按缺口一步跳到 997，该找到");
            };
            assert_eq!(units, 997, "缺口 3 个单元：1000 减 3");
            assert_eq!(refused_before.len(), 1);
            assert_eq!(refused_before[0].0, 1000);
            assert_eq!(refused_before[0].1.name(), "space_admission_refused");
            assert_eq!(tried, vec![1000, 997]);
        }
        {
            let found =
                find_a_lower_bound_that_passes(1000, |_units| Ok(refused_in_the_fixed_point()))
                    .expect("试写不出错");
            let LowerBoundSearch::Found {
                units,
                refused_before,
            } = found
            else {
                panic!("被拒在固定点算放行，该找到");
            };
            assert_eq!((units, refused_before.len()), (1000, 0));
        }
        {
            let found = find_a_lower_bound_that_passes(1000, |_units| Ok(refused_without_a_gap()))
                .expect("试写不出错");
            let LowerBoundSearch::NotFound { reason, refused } = found else {
                panic!("一路折半到 1 个单元都被拒，不该找到");
            };
            let refused_units: Vec<usize> = refused.iter().map(|(units, _)| *units).collect();
            assert_eq!(
                refused_units,
                vec![1000, 500, 250, 125, 62, 31, 15, 7, 3, 1],
                "没有缺口读数时每次折半，试到 1 个单元为止"
            );
            assert!(reason.contains("1 个单元"), "{reason}");
        }
        {
            let found = find_a_lower_bound_that_passes(1000, |_units| Ok(refused_by_space(1)))
                .expect("试写不出错");
            let LowerBoundSearch::NotFound { reason, refused } = found else {
                panic!("缺口只有一字节、每次只少 1 个单元，试满次数都没放行，不该找到");
            };
            assert_eq!(refused.len(), 40, "试满 40 次停");
            assert_eq!(
                refused[39].0, 961,
                "1000 起每次少 1 个单元，第 40 次试的是 961"
            );
            assert!(reason.contains("40 次"), "{reason}");
        }
    }

    /// 二分（登记 12.12）：缺口读数线性时，按最近一次空间准入拒的缺口预测边，一轮就夹住；预测错了（缺口随单元数不是线性）仍收敛到同一个边；
    /// 没有缺口读数时退回等分，也收敛。夹住的意思是下界放行、下界 + 1 被拒，两样都是观测到的。
    pub fn the_edge_search_predicts_the_edge_from_the_shortfall_and_closes_it_with_observed_points()
    {
        let edge = 4000;
        {
            let mut tried_one = Vec::new();
            let mut points_of_the_rounds: Vec<Vec<usize>> = Vec::new();
            let search = search_for_the_edge(
                4600,
                5600,
                2,
                |units| {
                    tried_one.push(units);
                    Ok(outcome_on_a_synthetic_base_with_a_gap(units, edge, 1))
                },
                |points| {
                    points_of_the_rounds.push(points.to_vec());
                    Ok(points
                        .iter()
                        .map(|units| {
                            (
                                *units,
                                outcome_on_a_synthetic_base_with_a_gap(*units, edge, 1),
                            )
                        })
                        .collect())
                },
            )
            .expect("试写不出错");
            assert_eq!(
                search.end,
                EdgeSearchEnd::Edge {
                    boundary_units: 4000
                }
            );
            assert_eq!(search.lower_bound_units, Some(4000));
            assert_eq!(
                tried_one,
                vec![4600, 4000],
                "4600 被拒，缺口 600 个单元，一步跳到 4000"
            );
            assert_eq!(
                points_of_the_rounds,
                vec![vec![4001, 4200]],
                "预测的边 4000 已是下界，试它上面一格 4001，余下的线程等分 (4000, 4600)"
            );
            assert_eq!(
                (
                    search.bisection_rounds,
                    search.bisection_points_tried,
                    search.bisection_points_passed
                ),
                (1, 2, 0)
            );
        }
        {
            let search = search_for_the_edge(
                4600,
                5600,
                2,
                |units| Ok(outcome_on_a_synthetic_base_with_a_gap(units, edge, 2)),
                |points| {
                    Ok(points
                        .iter()
                        .map(|units| {
                            (
                                *units,
                                outcome_on_a_synthetic_base_with_a_gap(*units, edge, 2),
                            )
                        })
                        .collect())
                },
            )
            .expect("试写不出错");
            assert_eq!(
                search.end,
                EdgeSearchEnd::Edge {
                    boundary_units: 4000
                },
                "缺口随单元数涨得比线性快一倍，预测落在边下面，二分仍夹住同一个边"
            );
            assert_eq!(
                search.lower_bound_units,
                Some(3400),
                "预测错的一步：4600 减 1200"
            );
        }
        {
            let mut points_of_the_rounds: Vec<Vec<usize>> = Vec::new();
            let search = search_for_the_edge(
                4600,
                5600,
                2,
                |units| Ok(outcome_on_a_synthetic_base_without_a_gap(units, edge)),
                |points| {
                    points_of_the_rounds.push(points.to_vec());
                    Ok(points
                        .iter()
                        .map(|units| {
                            (
                                *units,
                                outcome_on_a_synthetic_base_without_a_gap(*units, edge),
                            )
                        })
                        .collect())
                },
            )
            .expect("试写不出错");
            assert_eq!(
                search.end,
                EdgeSearchEnd::Edge {
                    boundary_units: 4000
                },
                "没有缺口读数就等分"
            );
            assert_eq!(
                search.lower_bound_units,
                Some(2300),
                "4600 被拒、折半到 2300 放行"
            );
            assert_eq!(
                points_of_the_rounds[0],
                vec![3066, 3833],
                "(2300, 4600) 三等分"
            );
            assert!(
                points_of_the_rounds.len() > 1
                    && points_of_the_rounds.iter().all(|points| points.len() <= 2),
                "等分要好几轮，每轮不超过线程数：{points_of_the_rounds:?}"
            );
        }
    }

    /// 二分一轮试哪些点：都落在 (下界, 上界) 里，有预测的边时先取预测的边与它上面一格，余下的线程等分。
    pub fn a_bisection_round_tries_the_predicted_edge_and_the_next_count_before_splitting_evenly() {
        assert_eq!(
            points_of_a_bisection_round(4000, 4600, 2, Some(4000)),
            vec![4001, 4200]
        );
        assert_eq!(
            points_of_a_bisection_round(3900, 4600, 2, Some(4000)),
            vec![4000, 4001]
        );
        assert_eq!(
            points_of_a_bisection_round(3900, 4600, 1, Some(4000)),
            vec![4000],
            "一个线程只试预测的边"
        );
        assert_eq!(
            points_of_a_bisection_round(4000, 4600, 2, None),
            vec![4200, 4400]
        );
        assert_eq!(
            points_of_a_bisection_round(4000, 4600, 2, Some(3990)),
            vec![4200, 4400],
            "预测的边在下界之下，用不上，等分"
        );
        assert_eq!(
            points_of_a_bisection_round(4000, 4002, 2, None),
            vec![4001],
            "只剩一个点可试"
        );
        assert_eq!(points_of_a_bisection_round(4000, 4002, 1, None), vec![4001]);
    }

    /// 边没被观测夹住时不交边界（登记 12.12）：一直放行到上限没有被拒的；被拒与放行的次序不一致；下界一个也没找到。
    pub fn an_edge_that_no_observation_brackets_is_reported_as_not_found() {
        {
            let search = search_for_the_edge(
                100,
                110,
                2,
                |_units| Ok(published_outcome()),
                |points| {
                    Ok(points
                        .iter()
                        .map(|units| (*units, published_outcome()))
                        .collect())
                },
            )
            .expect("试写不出错");
            let EdgeSearchEnd::NotFound { reason } = &search.end else {
                panic!("一直放行到上限，不该交边界：{search:?}");
            };
            assert!(reason.contains("上限 110"), "{reason}");
        }
        {
            let search = search_for_the_edge(
                3000,
                3010,
                2,
                |_units| Ok(published_outcome()),
                |points| {
                    Ok(points
                        .iter()
                        .map(|units| {
                            (
                                *units,
                                if *units == 3003 {
                                    refused_without_a_gap()
                                } else {
                                    published_outcome()
                                },
                            )
                        })
                        .collect())
                },
            )
            .expect("试写不出错");
            let EdgeSearchEnd::NotFound { reason } = &search.end else {
                panic!("3003 被拒而 3006 放行，次序不一致，不该交边界：{search:?}");
            };
            assert!(reason.contains("3003"), "{reason}");
        }
        {
            let search = search_for_the_edge(
                3,
                100,
                2,
                |_units| Ok(refused_without_a_gap()),
                |_points| panic!("下界没找到，不该再二分"),
            )
            .expect("试写不出错");
            let EdgeSearchEnd::NotFound { reason } = &search.end else {
                panic!("下界一个也没找到，不该交边界：{search:?}");
            };
            assert!(reason.contains("1 个单元"), "{reason}");
            assert_eq!(search.lower_bound_units, None);
            let refused_units: Vec<usize> = search
                .lower_bound_refused_before
                .iter()
                .map(|(units, _)| *units)
                .collect();
            assert_eq!(refused_units, vec![3, 1]);
        }
    }

    /// 验收（登记 12.12）：小几何填到段外槽对数的六成之后，同样大小的整份换写在这个底上被空间准入那一道拒；
    /// 下界不能是填过的单元数，要落在观测到放行的数上：先试的就是填过的单元数、被拒的是空间那一道、边界比它小，
    /// 边界在独立的一次试写里放行、边界 + 1 在独立的一次试写里不放行。
    pub fn a_small_base_filled_to_six_tenths_gets_an_edge_below_the_filled_units_that_independent_probes_confirm(
    ) {
        let mut content = ContentBuffer::new();
        let mut base = first_file_pool(GEOMETRY_SMALL);
        let pairs = data_unit_pairs_outside_the_clusters(&base, 0);
        let filled_units = usize::try_from(pairs * 6 / 10).expect("单元数");
        let filled = write_units_and_account(
            &mut base,
            filled_units,
            CodeTwoTreeNodeCapacities::FromTheNodeFormat,
            &mut content,
        )
        .expect("对拍过");
        assert!(
            matches!(filled, WriteOutcome::Published(_)),
            "填到六成该发成：{filled:?}"
        );
        base.seal();
        let upper_sentinel = usize::try_from(pairs).expect("槽对数") + 1024;
        let content_bytes: &[u8] = content.of_units(upper_sentinel);
        let search = search_for_the_edge(
            filled_units,
            upper_sentinel,
            2,
            |units| probe_one_write_on_the_sealed_base(&base, units, content_bytes),
            |points| probe_writes_in_parallel(&base, points, content_bytes),
        )
        .expect("试写对拍过");
        let EdgeSearchEnd::Edge { boundary_units } = &search.end else {
            panic!("该找到边：{search:?}");
        };
        let boundary_units = *boundary_units;
        let refusal_at_the_filled_units = search
            .lower_bound_refused_before
            .first()
            .unwrap_or_else(|| panic!("填过的单元数在填过的底上该被拒：{search:?}"));
        assert_eq!(
            refusal_at_the_filled_units.0, filled_units,
            "先试的是填过的单元数：{search:?}"
        );
        assert_eq!(
            refusal_at_the_filled_units.1.name(),
            "space_admission_refused",
            "被拒的是空间准入那一道：{search:?}"
        );
        assert!(boundary_units < filled_units, "{search:?}");
        let lower_bound = search.lower_bound_units.expect("找到了边就有下界");
        assert!(
            lower_bound <= boundary_units && lower_bound < filled_units,
            "{search:?}"
        );
        let at_the_edge = probe_one_write_on_the_sealed_base(&base, boundary_units, content_bytes)
            .expect("对拍过");
        assert!(
            passed_the_landing_check_and_the_formula(&at_the_edge),
            "边界 {boundary_units} 在独立的一次试写里该放行：{at_the_edge:?}"
        );
        let above_the_edge =
            probe_one_write_on_the_sealed_base(&base, boundary_units + 1, content_bytes)
                .expect("对拍过");
        assert!(
            !passed_the_landing_check_and_the_formula(&above_the_edge),
            "边界 + 1 在独立的一次试写里该不放行：{above_the_edge:?}"
        );
    }

    /// 标准错误上被拒那一行的拒因文字与缺口折算（登记 12.12 只改标准错误，不改产物行）。
    pub fn the_refusal_note_names_the_gate_and_the_gap_and_the_shortfall_rounds_up_to_whole_units()
    {
        let space = RefusalClass::SpaceAdmission {
            short_devices: String::new(),
            shortfall_bytes: 98_304,
        };
        assert_eq!(
            space.gate_and_gap_text(),
            "space_admission_refused(shortfall_bytes=98304)"
        );
        assert_eq!(space.space_shortfall_bytes(), Some(98_304));
        let in_placement = RefusalClass::FixedPointPlacement {
            unit: String::new(),
            refusal: String::new(),
        };
        assert_eq!(
            in_placement.gate_and_gap_text(),
            "placement_refused_in_the_fixed_point"
        );
        assert_eq!(in_placement.space_shortfall_bytes(), None);
        let over_the_limit = RefusalClass::Other {
            description: String::new(),
            journal_records_over_the_limit: Some(631),
        };
        assert_eq!(
            over_the_limit.gate_and_gap_text(),
            "other_refusal(journal_records_over_the_limit=631)"
        );
        assert_eq!(over_the_limit.space_shortfall_bytes(), None);
        let without_a_reading = RefusalClass::Other {
            description: String::new(),
            journal_records_over_the_limit: None,
        };
        assert_eq!(without_a_reading.gate_and_gap_text(), "other_refusal");
        assert_eq!(
            units_a_shortfall_is_worth(0),
            1,
            "缺口读数不是正数时按一字节算"
        );
        assert_eq!(units_a_shortfall_is_worth(1), 1);
        assert_eq!(units_a_shortfall_is_worth(32_768), 1);
        assert_eq!(units_a_shortfall_is_worth(32_769), 2, "向上取整");
        assert_eq!(units_a_shortfall_is_worth(98_304), 3);
        let refused_in_placement: Result<TransactionOutput, PublishError> =
            Err(PublishError::PlacementRefused {
                unit: TransactionUnit::TreeTable,
                refusal: PlacementRefusal::NoFreeSlotOnAnyDevice,
            });
        assert_eq!(
            progress_line_of_a_write(6000, 1.54, &refused_in_placement),
            "[e181] 一次写 6000 个单元的发布用了 1.5 秒，被拒，拒因 placement_refused_in_the_fixed_point"
        );
        let over_the_record_limit: Result<TransactionOutput, PublishError> =
            Err(PublishError::JournalRecordsOfThePublishExceedTheLimit {
                records_of_the_publish: 66_167,
                record_limit: 65_536,
            });
        assert_eq!(
            progress_line_of_a_write(65_877, 120.0, &over_the_record_limit),
            "[e181] 一次写 65877 个单元的发布用了 120.0 秒，被拒，拒因 other_refusal(journal_records_over_the_limit=631)",
            "登记 12.4 的读数：66167 条、上限 65536，超出 631"
        );
        assert_eq!(
            RefusalClass::of(&PublishError::ContentExceedsDataUnit {
                bytes: 1,
                capacity: 0
            }),
            RefusalClass::Other {
                description: "ContentExceedsDataUnit { bytes: 1, capacity: 0 }".to_string(),
                journal_records_over_the_limit: None
            },
            "别的拒因原样带 Debug 文字，不带超出条数"
        );
    }

    /// 有一个边历史没找到边，固定点上界那一段停机（带几何名与原因）；都找到了不停。
    pub fn a_geometry_whose_edge_was_not_found_stops_the_fixed_point_upper_bound_section() {
        let found = EdgeReport::default();
        let not_found = EdgeReport {
            edge_not_found_reason: Some("一直放行到上限 110 个单元都没有被拒的试写".to_string()),
            ..EdgeReport::default()
        };
        assert_eq!(
            stop_when_an_edge_was_not_found(&[
                ("narrow", found.clone()),
                ("small_unit_area", found.clone())
            ]),
            Ok(())
        );
        let stopped =
            stop_when_an_edge_was_not_found(&[("narrow", found), ("small_unit_area", not_found)])
                .expect_err("有一个没找到边，该停机");
        assert!(
            stopped.starts_with("没找到边：")
                && stopped.contains("small_unit_area")
                && !stopped.contains("narrow"),
            "{stopped}"
        );
    }

    /// 准入判法的第一行以 `forced` 起头时才交回摘要（最后一段，空白换成下划线）；满足或没声明时不交。
    pub fn the_preflight_verdict_line_gives_a_summary_only_when_forced() {
        assert_eq!(
            forced_summary_of_the_preflight_verdict("forced\tinputs-unchanged\ta b\tc d"),
            Some("c_d".to_string())
        );
        assert_eq!(forced_summary_of_the_preflight_verdict("met\tabc123"), None);
        assert_eq!(forced_summary_of_the_preflight_verdict("undeclared"), None);
        assert_eq!(forced_summary_of_the_preflight_verdict(""), None);
    }

    pub fn local_constants_agree_with_the_crates() {
        check_local_constants_against_the_crates().expect("本地常量回比");
    }
}

/// 每一条断言的名字与函数（`selfcheck` 段按这张表逐条跑）。
const CLAIMS: [(&str, fn()); 28] = [
    ("allocation_record_tree_nodes_of_two_four_gibibyte_devices_are_525_529_and_531_at_root_levels_one_two_and_three", claims::allocation_record_tree_nodes_of_two_four_gibibyte_devices_are_525_529_and_531_at_root_levels_one_two_and_three),
    ("allocation_record_tree_nodes_of_the_small_geometry_are_55_at_root_level_one_and_57_at_root_level_two", claims::allocation_record_tree_nodes_of_the_small_geometry_are_55_at_root_level_one_and_57_at_root_level_two),
    ("central_mapping_nodes_rewritten_at_most_pins_the_hand_computed_examples", claims::central_mapping_nodes_rewritten_at_most_pins_the_hand_computed_examples),
    ("sparse_layers_share_reads_and_keep_writes_apart", claims::sparse_layers_share_reads_and_keep_writes_apart),
    ("zeroing_masks_the_sealed_layers_and_a_later_write_shows_through_the_mask", claims::zeroing_masks_the_sealed_layers_and_a_later_write_shows_through_the_mask),
    ("the_first_file_lands_at_50180_and_its_records_end_at_50253", claims::the_first_file_lands_at_50180_and_its_records_end_at_50253),
    ("the_first_file_publish_has_six_ordinary_and_eight_fixed_point_slots_by_both_paths", claims::the_first_file_publish_has_six_ordinary_and_eight_fixed_point_slots_by_both_paths),
    ("the_local_checkpoint_cost_equals_the_core_one_on_every_version_measured", claims::the_local_checkpoint_cost_equals_the_core_one_on_every_version_measured),
    ("a_growth_write_deletes_the_four_first_file_units_and_inserts_every_rewritten_mapped_ordinary_role", claims::a_growth_write_deletes_the_four_first_file_units_and_inserts_every_rewritten_mapped_ordinary_role),
    ("a_sixty_four_unit_write_on_the_small_capacity_trees_makes_the_checkpoint_cost_with_ordinary_mapping_changes_exceed_the_checkpoint_cost", claims::a_sixty_four_unit_write_on_the_small_capacity_trees_makes_the_checkpoint_cost_with_ordinary_mapping_changes_exceed_the_checkpoint_cost),
    ("extra_deletes_and_inserts_each_raise_the_local_checkpoint_cost_on_a_wide_tree", claims::extra_deletes_and_inserts_each_raise_the_local_checkpoint_cost_on_a_wide_tree),
    ("the_judges_flip_when_the_threshold_moves_between_two_readings", claims::the_judges_flip_when_the_threshold_moves_between_two_readings),
    ("slots_enough_and_aligned_pairs_short_holds_only_when_the_slots_are_enough_everywhere_and_the_aligned_pairs_are_short_somewhere", claims::slots_enough_and_aligned_pairs_short_holds_only_when_the_slots_are_enough_everywhere_and_the_aligned_pairs_are_short_somewhere),
    ("marking_every_pair_outside_the_clusters_leaves_no_lowest_slot_and_marking_none_leaves_50182", claims::marking_every_pair_outside_the_clusters_leaves_no_lowest_slot_and_marking_none_leaves_50182),
    ("aligned_free_pairs_count_the_open_segment_too", claims::aligned_free_pairs_count_the_open_segment_too),
    ("the_seeded_stream_repeats_for_a_seed_and_differs_between_seeds", claims::the_seeded_stream_repeats_for_a_seed_and_differs_between_seeds),
    ("refusals_are_named_by_the_gate_that_refused", claims::refusals_are_named_by_the_gate_that_refused),
    ("the_units_to_try_after_a_refusal_follow_the_surplus_the_shortfall_or_half", claims::the_units_to_try_after_a_refusal_follow_the_surplus_the_shortfall_or_half),
    ("a_capped_fill_of_the_small_geometry_publishes_several_times_and_ends_with_nothing_left_or_a_refused_single_unit", claims::a_capped_fill_of_the_small_geometry_publishes_several_times_and_ends_with_nothing_left_or_a_refused_single_unit),
    ("the_lower_bound_is_tried_on_the_filled_base_first_and_follows_the_refusals_down_to_a_passing_count", claims::the_lower_bound_is_tried_on_the_filled_base_first_and_follows_the_refusals_down_to_a_passing_count),
    ("the_edge_search_predicts_the_edge_from_the_shortfall_and_closes_it_with_observed_points", claims::the_edge_search_predicts_the_edge_from_the_shortfall_and_closes_it_with_observed_points),
    ("a_bisection_round_tries_the_predicted_edge_and_the_next_count_before_splitting_evenly", claims::a_bisection_round_tries_the_predicted_edge_and_the_next_count_before_splitting_evenly),
    ("an_edge_that_no_observation_brackets_is_reported_as_not_found", claims::an_edge_that_no_observation_brackets_is_reported_as_not_found),
    ("a_small_base_filled_to_six_tenths_gets_an_edge_below_the_filled_units_that_independent_probes_confirm", claims::a_small_base_filled_to_six_tenths_gets_an_edge_below_the_filled_units_that_independent_probes_confirm),
    ("the_refusal_note_names_the_gate_and_the_gap_and_the_shortfall_rounds_up_to_whole_units", claims::the_refusal_note_names_the_gate_and_the_gap_and_the_shortfall_rounds_up_to_whole_units),
    ("a_geometry_whose_edge_was_not_found_stops_the_fixed_point_upper_bound_section", claims::a_geometry_whose_edge_was_not_found_stops_the_fixed_point_upper_bound_section),
    ("the_preflight_verdict_line_gives_a_summary_only_when_forced", claims::the_preflight_verdict_line_gives_a_summary_only_when_forced),
    ("local_constants_agree_with_the_crates", claims::local_constants_agree_with_the_crates),
];

/// 逐条跑 [`CLAIMS`]，每条一行产物；有一条红就返回 `Err`（红在哪一条看产物行）。
fn run_selfcheck() -> Result<(), String> {
    let mut failed = Vec::new();
    for (name, claim) in CLAIMS {
        let outcome = std::panic::catch_unwind(claim);
        emit(&format!(
            "name=selfcheck item={name} passed={}",
            outcome.is_ok()
        ));
        if outcome.is_err() {
            failed.push(name);
        }
    }
    if failed.is_empty() {
        Ok(())
    } else {
        Err(format!("自证红了：{failed:?}"))
    }
}

#[cfg(test)]
mod tests {
    use super::claims;

    #[test]
    fn allocation_record_tree_nodes_of_two_four_gibibyte_devices_are_525_529_and_531_at_root_levels_one_two_and_three(
    ) {
        claims::allocation_record_tree_nodes_of_two_four_gibibyte_devices_are_525_529_and_531_at_root_levels_one_two_and_three(
        );
    }

    #[test]
    fn allocation_record_tree_nodes_of_the_small_geometry_are_55_at_root_level_one_and_57_at_root_level_two(
    ) {
        claims::allocation_record_tree_nodes_of_the_small_geometry_are_55_at_root_level_one_and_57_at_root_level_two();
    }

    #[test]
    fn central_mapping_nodes_rewritten_at_most_pins_the_hand_computed_examples() {
        claims::central_mapping_nodes_rewritten_at_most_pins_the_hand_computed_examples();
    }

    #[test]
    fn sparse_layers_share_reads_and_keep_writes_apart() {
        claims::sparse_layers_share_reads_and_keep_writes_apart();
    }

    #[test]
    fn zeroing_masks_the_sealed_layers_and_a_later_write_shows_through_the_mask() {
        claims::zeroing_masks_the_sealed_layers_and_a_later_write_shows_through_the_mask();
    }

    #[test]
    fn the_first_file_lands_at_50180_and_its_records_end_at_50253() {
        claims::the_first_file_lands_at_50180_and_its_records_end_at_50253();
    }

    #[test]
    fn the_first_file_publish_has_six_ordinary_and_eight_fixed_point_slots_by_both_paths() {
        claims::the_first_file_publish_has_six_ordinary_and_eight_fixed_point_slots_by_both_paths();
    }

    #[test]
    fn the_local_checkpoint_cost_equals_the_core_one_on_every_version_measured() {
        claims::the_local_checkpoint_cost_equals_the_core_one_on_every_version_measured();
    }

    #[test]
    fn a_growth_write_deletes_the_four_first_file_units_and_inserts_every_rewritten_mapped_ordinary_role(
    ) {
        claims::a_growth_write_deletes_the_four_first_file_units_and_inserts_every_rewritten_mapped_ordinary_role();
    }

    #[test]
    fn a_sixty_four_unit_write_on_the_small_capacity_trees_makes_the_checkpoint_cost_with_ordinary_mapping_changes_exceed_the_checkpoint_cost(
    ) {
        claims::a_sixty_four_unit_write_on_the_small_capacity_trees_makes_the_checkpoint_cost_with_ordinary_mapping_changes_exceed_the_checkpoint_cost();
    }

    #[test]
    fn extra_deletes_and_inserts_each_raise_the_local_checkpoint_cost_on_a_wide_tree() {
        claims::extra_deletes_and_inserts_each_raise_the_local_checkpoint_cost_on_a_wide_tree();
    }

    #[test]
    fn the_judges_flip_when_the_threshold_moves_between_two_readings() {
        claims::the_judges_flip_when_the_threshold_moves_between_two_readings();
    }

    #[test]
    fn slots_enough_and_aligned_pairs_short_holds_only_when_the_slots_are_enough_everywhere_and_the_aligned_pairs_are_short_somewhere(
    ) {
        claims::slots_enough_and_aligned_pairs_short_holds_only_when_the_slots_are_enough_everywhere_and_the_aligned_pairs_are_short_somewhere();
    }

    #[test]
    fn marking_every_pair_outside_the_clusters_leaves_no_lowest_slot_and_marking_none_leaves_50182()
    {
        claims::marking_every_pair_outside_the_clusters_leaves_no_lowest_slot_and_marking_none_leaves_50182();
    }

    #[test]
    fn aligned_free_pairs_count_the_open_segment_too() {
        claims::aligned_free_pairs_count_the_open_segment_too();
    }

    #[test]
    fn the_seeded_stream_repeats_for_a_seed_and_differs_between_seeds() {
        claims::the_seeded_stream_repeats_for_a_seed_and_differs_between_seeds();
    }

    #[test]
    fn refusals_are_named_by_the_gate_that_refused() {
        claims::refusals_are_named_by_the_gate_that_refused();
    }

    #[test]
    fn the_units_to_try_after_a_refusal_follow_the_surplus_the_shortfall_or_half() {
        claims::the_units_to_try_after_a_refusal_follow_the_surplus_the_shortfall_or_half();
    }

    #[test]
    fn a_capped_fill_of_the_small_geometry_publishes_several_times_and_ends_with_nothing_left_or_a_refused_single_unit(
    ) {
        claims::a_capped_fill_of_the_small_geometry_publishes_several_times_and_ends_with_nothing_left_or_a_refused_single_unit();
    }

    #[test]
    fn the_lower_bound_is_tried_on_the_filled_base_first_and_follows_the_refusals_down_to_a_passing_count(
    ) {
        claims::the_lower_bound_is_tried_on_the_filled_base_first_and_follows_the_refusals_down_to_a_passing_count();
    }

    #[test]
    fn the_edge_search_predicts_the_edge_from_the_shortfall_and_closes_it_with_observed_points() {
        claims::the_edge_search_predicts_the_edge_from_the_shortfall_and_closes_it_with_observed_points();
    }

    #[test]
    fn a_bisection_round_tries_the_predicted_edge_and_the_next_count_before_splitting_evenly() {
        claims::a_bisection_round_tries_the_predicted_edge_and_the_next_count_before_splitting_evenly();
    }

    #[test]
    fn an_edge_that_no_observation_brackets_is_reported_as_not_found() {
        claims::an_edge_that_no_observation_brackets_is_reported_as_not_found();
    }

    #[test]
    fn a_small_base_filled_to_six_tenths_gets_an_edge_below_the_filled_units_that_independent_probes_confirm(
    ) {
        claims::a_small_base_filled_to_six_tenths_gets_an_edge_below_the_filled_units_that_independent_probes_confirm();
    }

    #[test]
    fn the_refusal_note_names_the_gate_and_the_gap_and_the_shortfall_rounds_up_to_whole_units() {
        claims::the_refusal_note_names_the_gate_and_the_gap_and_the_shortfall_rounds_up_to_whole_units();
    }

    #[test]
    fn a_geometry_whose_edge_was_not_found_stops_the_fixed_point_upper_bound_section() {
        claims::a_geometry_whose_edge_was_not_found_stops_the_fixed_point_upper_bound_section();
    }

    #[test]
    fn the_preflight_verdict_line_gives_a_summary_only_when_forced() {
        claims::the_preflight_verdict_line_gives_a_summary_only_when_forced();
    }

    #[test]
    fn local_constants_agree_with_the_crates() {
        claims::local_constants_agree_with_the_crates();
    }
}
