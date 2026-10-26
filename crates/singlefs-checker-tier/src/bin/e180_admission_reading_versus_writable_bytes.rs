//! E180（准入读数与真可写量）入库装置：跑前登记 `research/prompts/e180-preregistration.md`（岔路单
//! `research/prompts/m3-close-admission-and-shadow-ledger-forks.md` 的 F2、F5、F6、F8 四行）。只读、只驱动与观测 `crates/` 今天的
//! 会话发布路径、准入读数与分配器，不改 `crates/singlefs-core`、`crates/singlefs-checker`。本文件是第一段（读数段）：
//! F2 的全部臂、F5 的残差、F8 的轨迹；F6 的造格、重试与崩溃点枚举（第二段）不在这里。
//!
//! 用法：`e180_admission_reading_versus_writable_bytes reading [--geometry <几何名>] [--family <族名>]`（产物打到标准输出，末行 `E name=done emitted=<行数>`）；
//! `… reading --geometry <几何名> --family <族名> --shard-output <文件>`（只跑一片，写片文件，见 [`shard_text`]）；
//! `… list-shards`（全部片与各占的线程数）；`… merge <片目录>`（把齐了的片合成与一次跑全部逐字节相同的产物）；
//! `e180_admission_reading_versus_writable_bytes calibrate`（在默认几何上量探测与恢复的挂钟，外推整段，不进产物）。
//! 分片、续跑与多机的跑法在 `scripts/experiment-apparatus/e180-reading-shards.sh`。
//! 每一行产物以 `E name=` 开头；停机条款不过打 `E name=stop reason=…` 并退 4。
//!
//! 「这一刻写得进的最大字节数」T 的探测（登记 5.1）：在状态的拷贝上经会话做一次 `SequentialWrite`，设备是写时复制的叠层，
//! 第一个数据单元的写到达时报块设备错，会话交回这个错、设备记「数据写到达」即记放行；空间不够由会话交回
//! `NoSpaceAfterRaisingTheFloor` 记被拒。一次 journal 记录数超过上限（`JournalRecordsOfThePublishExceedTheLimit`）的拒绝不是空间不够，
//! 按登记的字面那个状态 T 记「取不出」；登记没料到默认环的在飞记录上限（65536 条）把「空间很空」的状态的 T 截在一个下界上，
//! 装置对这类状态另记「截尾」并给出下界，登记第十二节修订写了补的判据。

use std::collections::{BTreeMap, BTreeSet};
use std::ops::Range;
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Mutex;
use std::time::Instant;

use singlefs_core::address::{DeviceIdentity, DeviceOffsetInBytes, SlotNumber};
use singlefs_core::admission::{
    admission_reading_before_a_publish, checkpoint_cost_of_the_version_to_build_on,
    AdmissionReading,
};
use singlefs_core::allocator::PoolAllocator;
use singlefs_core::block_device::{
    BlockDevice, BlockDeviceError, DiscardOutcome, PhysicalBlockSizeInBytes, WriteDurability,
};
use singlefs_core::make_filesystem::{make_filesystem, MakeFilesystemParameters};
use singlefs_core::mount::{mount_writable, unmount, MountOutput, Mounted, Unmounted};
use singlefs_core::mounted_session::{
    refusal_is_short_of_space, MountedSession, UserChange, UserChangePublished, UserChangeRefused,
};
use singlefs_core::recovery::allocation_records_under_root;
use singlefs_core::root_ring::RootRingSlotsPerRegion;
use singlefs_core::system_configuration::SystemImmutableSizes;
use singlefs_core::transaction::{
    publish_first_file, FirstFile, PoolVersion, PoolWriter, PublishError, TransactionOutput,
    TransactionUnit,
};

// 下面这些常量抄自 kb 的登记位（值与出处在每条的注释里），不引 `crates/` 的同名常量：装置与实现不共用一个数，
// `local_constants_against_the_implementation` 逐个回比，两边一起错时红（登记第五节「本地常量」表）。这一条挡不住 kb 与实现一起取错。
/// 槽字节：D3 已定项 10 ①「64 个 16 KiB 槽」；实现侧 `singlefs_format::SLOT_BYTES`。
const SLOT_BYTES: u64 = 16384;
/// 数据单元载荷：D18 已定项 16（32768 − 头 134）；实现侧 `singlefs_core::unit::data_unit_payload_capacity()`。
const DATA_UNIT_PAYLOAD_BYTES: u64 = 32634;
/// 数据单元字节：D18 已定项 3；实现侧 `singlefs_format::DATA_UNIT_BYTES`。
const DATA_UNIT_BYTES: u64 = 32768;
/// 聚簇段槽数：D3 已定项 10 ①；实现侧 `singlefs_format::CLUSTER_SEGMENT_SLOTS`。
const CLUSTER_SEGMENT_SLOTS: u64 = 64;
/// 默认环下单元区起始槽号：D3 已定项 10 ④；实现侧 `singlefs_format::UNIT_AREA_START_SLOT`。
const DEFAULT_UNIT_AREA_START_SLOT: u64 = 50176;
/// 默认 journal 环长：D23 已定项 19 ③；实现侧 `singlefs_format::JOURNAL_RING_DEFAULT_BYTES`。
const DEFAULT_JOURNAL_RING_BYTES: u64 = 805_306_368;
/// journal 环起点槽号：D23 已定项 19；实现侧 `singlefs_format::JOURNAL_RING_START_SLOT`。
const JOURNAL_RING_START_SLOT: u64 = 1024;
/// 一条 journal 记录字节：D23 已定项 19；实现侧 `singlefs_format::JOURNAL_RECORD_BYTES`。
const JOURNAL_RECORD_BYTES: u64 = 4096;
/// mkfs 的每区槽数 S：D22 已定项 2 参数表与已定项 16 第 3 句（默认 min(8, …)，这个装置的槽距下取 8）；实现侧 `singlefs_format::ROOT_RING_SLOTS_PER_REGION_AT_MAKE_FILESYSTEM`。
const ROOT_RING_SLOTS_PER_REGION_AT_MAKE_FILESYSTEM: u64 = 8;
/// 根环区数：D22 已定项 2；实现侧 `singlefs_format::ROOT_RING_REGIONS`。
const ROOT_RING_REGIONS: u64 = 3;
/// 一次准入的发布数 B = 4 + 2 k_tol（k_tol = 2）：D16 已定项 1「准入」一行；实现侧 `singlefs_core::mount::PUBLISHES_PER_ADMISSION_AT_MOST`。
const PUBLISHES_PER_ADMISSION_AT_MOST: u64 = 8;
/// 下段叶罩的数据单元数：D8 已定项 14；实现侧 `singlefs_format::EXTENT_TREE_LOWER_LEAF_DATA_UNITS`。
const EXTENT_TREE_LOWER_LEAF_DATA_UNITS: u64 = 144;
/// extent 树内部扇出：D8 已定项 14；实现侧 `singlefs_format::EXTENT_TREE_INTERNAL_FANOUT`。
const EXTENT_TREE_INTERNAL_FANOUT: u64 = 147;
/// 复制数：第一版两块盘互为镜像（登记 1.2 副本数固定 2）。
const DEVICE_COUNT: u64 = 2;
/// 数据单元头里的魔数与类标签：`singlefs_core::unit::{UNIT_MAGIC, UNIT_CLASS_DATA}`（只用来认「这一次写是数据单元的写」）。
const UNIT_MAGIC_BYTES: [u8; 4] = *b"SFSU";
const UNIT_CLASS_DATA_CODE: u8 = 1;
/// 单元头里类标签的字节下标（魔数 4 + 版本 2）。
const UNIT_CLASS_BYTE_INDEX: usize = 6;
const SECTOR_BYTES: usize = 512;
const PHYSICAL_BLOCK_BYTES: u32 = 512;
const WRITE_TIME_SECONDS: u64 = 1_788_000_000;
const STOP_EXIT_CODE: i32 = 4;
/// 登记 5.1 的 T 探测上界用的记录数：会话没法按「第 N 条」指定，装置从拒绝里读（`records_of_the_publish`、`record_limit`）。
const PROBE_FAR_ABOVE_ANY_JOURNAL_CAP_UNITS: u64 = 120_000;

/// 本地常量逐个与 `crates/` 回比；对不上就停（登记第十一节作废条款：本地常量与实现侧回比有一条不等）。交回比过的个数。
fn local_constants_against_the_implementation() -> usize {
    let pairs: [(&str, u64, u64); 14] = [
        ("SLOT_BYTES", SLOT_BYTES, singlefs_format::SLOT_BYTES),
        (
            "DATA_UNIT_PAYLOAD_BYTES",
            DATA_UNIT_PAYLOAD_BYTES,
            u64::try_from(singlefs_core::unit::data_unit_payload_capacity()).expect("载荷字节数"),
        ),
        (
            "DATA_UNIT_BYTES",
            DATA_UNIT_BYTES,
            singlefs_format::DATA_UNIT_BYTES,
        ),
        (
            "CLUSTER_SEGMENT_SLOTS",
            CLUSTER_SEGMENT_SLOTS,
            singlefs_format::CLUSTER_SEGMENT_SLOTS,
        ),
        (
            "DEFAULT_UNIT_AREA_START_SLOT",
            DEFAULT_UNIT_AREA_START_SLOT,
            singlefs_format::UNIT_AREA_START_SLOT,
        ),
        (
            "DEFAULT_JOURNAL_RING_BYTES",
            DEFAULT_JOURNAL_RING_BYTES,
            singlefs_format::JOURNAL_RING_DEFAULT_BYTES,
        ),
        (
            "JOURNAL_RING_START_SLOT",
            JOURNAL_RING_START_SLOT,
            singlefs_format::JOURNAL_RING_START_SLOT,
        ),
        (
            "JOURNAL_RECORD_BYTES",
            JOURNAL_RECORD_BYTES,
            singlefs_format::JOURNAL_RECORD_BYTES,
        ),
        (
            "ROOT_RING_SLOTS_PER_REGION_AT_MAKE_FILESYSTEM",
            ROOT_RING_SLOTS_PER_REGION_AT_MAKE_FILESYSTEM,
            singlefs_format::ROOT_RING_SLOTS_PER_REGION_AT_MAKE_FILESYSTEM,
        ),
        (
            "ROOT_RING_REGIONS",
            ROOT_RING_REGIONS,
            singlefs_format::ROOT_RING_REGIONS,
        ),
        (
            "PUBLISHES_PER_ADMISSION_AT_MOST",
            PUBLISHES_PER_ADMISSION_AT_MOST,
            u64::try_from(singlefs_core::mount::PUBLISHES_PER_ADMISSION_AT_MOST)
                .expect("一次准入的发布数"),
        ),
        (
            "EXTENT_TREE_LOWER_LEAF_DATA_UNITS",
            EXTENT_TREE_LOWER_LEAF_DATA_UNITS,
            singlefs_format::EXTENT_TREE_LOWER_LEAF_DATA_UNITS,
        ),
        (
            "EXTENT_TREE_INTERNAL_FANOUT",
            EXTENT_TREE_INTERNAL_FANOUT,
            singlefs_format::EXTENT_TREE_INTERNAL_FANOUT,
        ),
        (
            "UNIT_CLASS_DATA_CODE",
            u64::from(UNIT_CLASS_DATA_CODE),
            u64::from(singlefs_core::unit::UNIT_CLASS_DATA),
        ),
    ];
    for (name, local, implementation) in pairs {
        if local != implementation {
            stop_with(
                "local_constant_differs_from_the_implementation",
                &format!("constant={name} local={local} implementation={implementation}"),
            );
        }
    }
    assert_eq!(
        UNIT_MAGIC_BYTES,
        singlefs_core::unit::UNIT_MAGIC,
        "数据单元魔数本地值与实现不同"
    );
    pairs.len() + 1
}

/// 产物行的出口：整行经一把锁打到标准输出，不同线程的行不交错；线程里攒行的另走 [`LineBuffer`]。
static OUTPUT_LOCK: Mutex<()> = Mutex::new(());

fn print_product_line(line: &str) {
    let _guard = OUTPUT_LOCK.lock().expect("输出锁没被毒化");
    println!("{line}");
}

/// 一段历史攒下的产物行：线程里先攒着，整段跑完由主线程按固定次序打出，行序与线程调度无关。
#[derive(Default)]
struct LineBuffer {
    lines: Vec<String>,
}

impl LineBuffer {
    fn push(&mut self, line: String) {
        self.lines.push(line);
    }
}

/// 停机：打一行 `E name=stop` 并退 [`STOP_EXIT_CODE`]（登记第十一节停机条款）。
fn stop_with(reason: &str, detail: &str) -> ! {
    print_product_line(&format!("E name=stop reason={reason} {detail}"));
    std::process::exit(STOP_EXIT_CODE)
}

// ---------------------------------------------------------------------------------------------------------------------
// 设备：只存非零扇区的稀疏盘，加一层写时复制的叠层
// ---------------------------------------------------------------------------------------------------------------------

type Sector = [u8; SECTOR_BYTES];

/// 按扇区存的盘面。基底（没有叠层）的盘不存全零扇区；叠层存全零扇区，用来盖住底下基底里的非零扇区。
#[derive(Clone, Default)]
struct SectorStore {
    sectors: BTreeMap<u64, Box<Sector>>,
}

impl SectorStore {
    fn write_sectors(&mut self, byte_offset: u64, bytes: &[u8], keeps_zero_sectors: bool) {
        let first_sector = byte_offset / u64::try_from(SECTOR_BYTES).expect("512");
        for (index_in_write, chunk) in bytes.as_chunks::<SECTOR_BYTES>().0.iter().enumerate() {
            let sector_number =
                first_sector + u64::try_from(index_in_write).expect("写里的扇区下标");
            let chunk_is_all_zero = chunk.iter().all(|byte| *byte == 0);
            if chunk_is_all_zero && !keeps_zero_sectors {
                self.sectors.remove(&sector_number);
            } else {
                let mut sector: Box<Sector> = Box::new([0u8; SECTOR_BYTES]);
                sector.copy_from_slice(chunk);
                self.sectors.insert(sector_number, sector);
            }
        }
    }

    fn copy_into(&self, byte_offset: u64, buffer: &mut [u8]) {
        let sector_bytes = u64::try_from(SECTOR_BYTES).expect("512");
        let first_sector = byte_offset / sector_bytes;
        let sector_count = u64::try_from(buffer.len()).expect("缓冲长度") / sector_bytes;
        for (sector_number, sector) in self
            .sectors
            .range(first_sector..first_sector + sector_count)
        {
            let start =
                usize::try_from((sector_number - first_sector) * sector_bytes).expect("缓冲内偏移");
            buffer[start..start + SECTOR_BYTES].copy_from_slice(&sector[..]);
        }
    }

    fn sector_numbers_in(&self, sector_range: Range<u64>) -> Vec<u64> {
        self.sectors
            .range(sector_range)
            .map(|(number, _)| *number)
            .collect()
    }
}

/// 数据单元的写到达时怎么办（登记 5.1 探测捷径）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum DataWritePolicy {
    /// 照写。
    AllowDataWrites,
    /// 第一个数据单元的写到达时记「到达」并报块设备错（会话把它当作发布途中的写错交回）。
    FailTheFirstDataWriteAndNoteIt,
}

static NEXT_RECORDED_WRITE_SEQUENCE: AtomicU64 = AtomicU64::new(0);

/// 设备上一次交给它的写，按到达次序（崩溃态按「写到第 k 次」截取用）。
#[derive(Clone, Debug)]
struct RecordedWrite {
    /// 全局写序号：两块盘上的写按到达次序排得回来（崩溃态按「写到根槽的 FUA 写」截取用）。
    sequence: u64,
    byte_offset: u64,
    bytes: Vec<u8>,
    is_force_unit_access: bool,
}

/// 一块盘：自己的扇区，加可选的只读基底（叠层用）。
struct Disk<'base> {
    size_in_bytes: u64,
    base: Option<&'base SectorStore>,
    own: SectorStore,
    data_write_policy: DataWritePolicy,
    data_write_reached: bool,
    /// 读这些字节区间一律报读错（登记 5「历史族」表 HC 一行，被抛弃根与净释放族的挂载乙：根槽与它的 journal 记录每次读都失败）。
    unreadable_byte_ranges: Vec<Range<u64>>,
    recording: Option<Vec<RecordedWrite>>,
}

impl Disk<'static> {
    fn blank(size_in_bytes: u64) -> Self {
        Disk {
            size_in_bytes,
            base: None,
            own: SectorStore::default(),
            data_write_policy: DataWritePolicy::AllowDataWrites,
            data_write_reached: false,
            unreadable_byte_ranges: Vec::new(),
            recording: None,
        }
    }

    /// 从另一块盘的盘面拷一份独立的盘（崩溃态、被抛弃根与净释放族的挂载乙用）。
    fn independent_copy_of(store: &SectorStore, size_in_bytes: u64) -> Self {
        let mut disk = Disk::blank(size_in_bytes);
        disk.own = store.clone();
        disk
    }
}

impl Disk<'_> {
    /// 写时复制的叠层：底下共享这块盘的盘面，只存这一次探测写的字节。
    fn overlay(&self, data_write_policy: DataWritePolicy) -> Disk<'_> {
        assert!(self.base.is_none(), "叠层不套叠层");
        Disk {
            size_in_bytes: self.size_in_bytes,
            base: Some(&self.own),
            own: SectorStore::default(),
            data_write_policy,
            data_write_reached: false,
            unreadable_byte_ranges: Vec::new(),
            recording: None,
        }
    }

    fn keeps_zero_sectors(&self) -> bool {
        self.base.is_some()
    }

    fn check_request(
        &self,
        offset: DeviceOffsetInBytes,
        length: u64,
    ) -> Result<(), BlockDeviceError> {
        let block = u64::from(PHYSICAL_BLOCK_BYTES);
        if !offset.0.is_multiple_of(block) || !length.is_multiple_of(block) {
            return Err(BlockDeviceError::Unaligned {
                offset,
                length,
                physical_block_size: PHYSICAL_BLOCK_BYTES,
            });
        }
        match offset.0.checked_add(length) {
            Some(end) if end <= self.size_in_bytes => Ok(()),
            Some(_) | None => Err(BlockDeviceError::OutOfRange {
                offset,
                length,
                device_size: self.size_in_bytes,
            }),
        }
    }

    fn zero_fill(&mut self, offset: u64, length: u64) {
        let sector_bytes = u64::try_from(SECTOR_BYTES).expect("512");
        let sector_range = offset / sector_bytes..(offset + length) / sector_bytes;
        let own_numbers = self.own.sector_numbers_in(sector_range.clone());
        let base_numbers = self
            .base
            .map(|base| base.sector_numbers_in(sector_range))
            .unwrap_or_default();
        for number in own_numbers {
            if self.base.is_some() {
                self.own
                    .sectors
                    .insert(number, Box::new([0u8; SECTOR_BYTES]));
            } else {
                self.own.sectors.remove(&number);
            }
        }
        for number in base_numbers {
            self.own
                .sectors
                .insert(number, Box::new([0u8; SECTOR_BYTES]));
        }
    }

    fn is_data_unit_write(bytes: &[u8]) -> bool {
        bytes.len() == usize::try_from(DATA_UNIT_BYTES).expect("32768")
            && bytes[..4] == UNIT_MAGIC_BYTES
            && bytes[UNIT_CLASS_BYTE_INDEX] == UNIT_CLASS_DATA_CODE
    }
}

impl BlockDevice for Disk<'_> {
    fn read_at(
        &self,
        offset: DeviceOffsetInBytes,
        buffer: &mut [u8],
    ) -> Result<(), BlockDeviceError> {
        let length = u64::try_from(buffer.len()).expect("读长");
        self.check_request(offset, length)?;
        let request = offset.0..offset.0 + length;
        if self
            .unreadable_byte_ranges
            .iter()
            .any(|range| range.start < request.end && request.start < range.end)
        {
            return Err(BlockDeviceError::InputOutput(std::io::Error::other(
                "装置注入的读错：这一段字节区间每次读都失败",
            )));
        }
        buffer.fill(0);
        if let Some(base) = self.base {
            base.copy_into(offset.0, buffer);
        }
        self.own.copy_into(offset.0, buffer);
        Ok(())
    }

    fn write_at(
        &mut self,
        offset: DeviceOffsetInBytes,
        bytes: &[u8],
        durability: WriteDurability,
    ) -> Result<(), BlockDeviceError> {
        self.check_request(offset, u64::try_from(bytes.len()).expect("写长"))?;
        if self.data_write_policy == DataWritePolicy::FailTheFirstDataWriteAndNoteIt
            && Self::is_data_unit_write(bytes)
        {
            self.data_write_reached = true;
            return Err(BlockDeviceError::InputOutput(std::io::Error::other(
                "装置注入的写错：第一个数据单元的写到达",
            )));
        }
        if let Some(recording) = self.recording.as_mut() {
            recording.push(RecordedWrite {
                sequence: NEXT_RECORDED_WRITE_SEQUENCE.fetch_add(1, Ordering::Relaxed),
                byte_offset: offset.0,
                bytes: bytes.to_vec(),
                is_force_unit_access: durability == WriteDurability::ForceUnitAccess,
            });
        }
        let keeps_zero_sectors = self.keeps_zero_sectors();
        self.own.write_sectors(offset.0, bytes, keeps_zero_sectors);
        Ok(())
    }

    fn write_zeroes_at(
        &mut self,
        offset: DeviceOffsetInBytes,
        length: u64,
    ) -> Result<(), BlockDeviceError> {
        self.check_request(offset, length)?;
        self.zero_fill(offset.0, length);
        Ok(())
    }

    fn discard_at(
        &mut self,
        offset: DeviceOffsetInBytes,
        length: u64,
    ) -> Result<DiscardOutcome, BlockDeviceError> {
        self.check_request(offset, length)?;
        self.zero_fill(offset.0, length);
        Ok(DiscardOutcome::Applied)
    }

    fn barrier(&mut self) -> Result<(), BlockDeviceError> {
        Ok(())
    }

    fn probe_physical_block_size(&self) -> PhysicalBlockSizeInBytes {
        PhysicalBlockSizeInBytes(PHYSICAL_BLOCK_BYTES)
    }

    fn size_in_bytes(&self) -> u64 {
        self.size_in_bytes
    }

    /// 写过的扇区所在的记录槽（只列记录起点上的扇区；叠层里全零的扇区是盖住基底用的，不算写过）。
    fn journal_record_offsets_hint(
        &self,
        ring_start: DeviceOffsetInBytes,
        ring_bytes: u64,
    ) -> Option<Vec<DeviceOffsetInBytes>> {
        let sector_bytes = u64::try_from(SECTOR_BYTES).expect("512");
        let sector_range = ring_start.0 / sector_bytes..(ring_start.0 + ring_bytes) / sector_bytes;
        let mut numbers: BTreeSet<u64> = BTreeSet::new();
        if let Some(base) = self.base {
            numbers.extend(base.sector_numbers_in(sector_range.clone()));
        }
        for (number, sector) in self.own.sectors.range(sector_range) {
            if sector.iter().all(|byte| *byte == 0) {
                numbers.remove(number);
            } else {
                numbers.insert(*number);
            }
        }
        Some(
            numbers
                .into_iter()
                .map(|number| number * sector_bytes)
                .filter(|offset| (offset - ring_start.0).is_multiple_of(JOURNAL_RECORD_BYTES))
                .map(DeviceOffsetInBytes)
                .collect(),
        )
    }
}

// ---------------------------------------------------------------------------------------------------------------------
// 几何、池、会话的拷贝
// ---------------------------------------------------------------------------------------------------------------------

/// 登记第五节的几何：盘字节数（两块）与每区槽数 S；默认环 768 MiB、io_min 4096 不变。
#[derive(Clone, Copy, Debug)]
struct Geometry {
    name: &'static str,
    device_bytes: [u64; 2],
    slots_per_region: u64,
}

const FOUR_GIBIBYTES: u64 = 4 << 30;
/// 3 GiB 加 33 槽：`crates/singlefs-harness/tests/unequal_devices.rs` 里已在用、mkfs 收的尺寸。
const THREE_GIBIBYTES_AND_33_SLOTS: u64 = (3 << 30) + 33 * SLOT_BYTES;

const ALL_GEOMETRIES: [Geometry; 5] = [
    Geometry {
        name: "default_geometry",
        device_bytes: [FOUR_GIBIBYTES, FOUR_GIBIBYTES],
        slots_per_region: 8,
    },
    Geometry {
        name: "unequal_devices_geometry",
        device_bytes: [FOUR_GIBIBYTES, THREE_GIBIBYTES_AND_33_SLOTS],
        slots_per_region: 8,
    },
    Geometry {
        name: "narrow_devices_geometry",
        device_bytes: [THREE_GIBIBYTES_AND_33_SLOTS, THREE_GIBIBYTES_AND_33_SLOTS],
        slots_per_region: 8,
    },
    Geometry {
        name: "default_geometry_with_four_slots_per_region",
        device_bytes: [FOUR_GIBIBYTES, FOUR_GIBIBYTES],
        slots_per_region: 4,
    },
    Geometry {
        name: "default_geometry_with_sixteen_slots_per_region",
        device_bytes: [FOUR_GIBIBYTES, FOUR_GIBIBYTES],
        slots_per_region: 16,
    },
];

impl Geometry {
    fn unit_area_slots_of_device(&self, device_index: usize) -> u64 {
        self.device_bytes[device_index] / SLOT_BYTES - DEFAULT_UNIT_AREA_START_SLOT
    }

    fn parameters(&self) -> MakeFilesystemParameters {
        MakeFilesystemParameters {
            filesystem_identifier: *b"_SFS-E180-ADMRD\0",
            region_devices: [DeviceIdentity(0), DeviceIdentity(1), DeviceIdentity(0)],
            geometry: SystemImmutableSizes {
                physical_block_size: PHYSICAL_BLOCK_BYTES,
                minimum_input_output_bytes: PHYSICAL_BLOCK_BYTES,
                fixed_structure_slot_spacing: 4096,
                journal_ring_bytes: DEFAULT_JOURNAL_RING_BYTES,
                root_ring_slots_per_region:
                    RootRingSlotsPerRegion::from_system_configuration_field(self.slots_per_region)
                        .expect("每区槽数只取 4、8、16，都在挂载判的区间里"),
            },
        }
    }

    fn blank_devices(&self) -> Vec<(DeviceIdentity, Disk<'static>)> {
        self.device_bytes
            .iter()
            .enumerate()
            .map(|(index, bytes)| {
                (
                    DeviceIdentity(u32::try_from(index).expect("盘号")),
                    Disk::blank(*bytes),
                )
            })
            .collect()
    }
}

fn geometry_named(name: &str) -> Geometry {
    *ALL_GEOMETRIES
        .iter()
        .find(|geometry| geometry.name == name)
        .unwrap_or_else(|| stop_with("unknown_geometry", &format!("geometry={name}")))
}

/// 池上每次发布的 txg 记在两个集合里：用户改动发的（非空发布）与别的（写行、暖机、抬 F、卸载的空发布）。
/// 登记 5.1 滞后量里「空发布的 txg 由装置自己记的历史日志认」。
#[derive(Clone, Debug, Default)]
struct PublishLog {
    user_change_txgs: BTreeSet<u64>,
    empty_publish_txgs: BTreeSet<u64>,
}

fn current_version(version: &PoolVersion) -> &TransactionOutput {
    match version {
        PoolVersion::WithFile(output) | PoolVersion::WithTreesAndNoFile(output) => output,
        PoolVersion::WithoutFile(_) => stop_with("current_version_has_no_file", ""),
    }
}

fn txg_of_pool_version(version: &PoolVersion) -> u64 {
    match version {
        PoolVersion::WithFile(output) | PoolVersion::WithTreesAndNoFile(output) => {
            output.root.checkpoint_txg.0
        }
        PoolVersion::WithoutFile(output) => output.root.checkpoint_txg.0,
    }
}

impl PublishLog {
    fn note_mount(&mut self, output: &MountOutput) {
        self.empty_publish_txgs
            .insert(txg_of_pool_version(&output.row_publish));
        for warm_up in &output.warm_up_publishes {
            self.empty_publish_txgs.insert(txg_of_pool_version(warm_up));
        }
    }

    fn note_floor_raises(&mut self, raises: &[singlefs_core::mount::RaisedFloor]) {
        for raised in raises {
            for publish in &raised.publishes {
                self.empty_publish_txgs
                    .insert(publish.root.checkpoint_txg.0);
            }
        }
    }

    fn note_published(&mut self, published: &UserChangePublished) {
        self.note_floor_raises(&published.floor_raises);
        for version in &published.versions_published {
            self.user_change_txgs.insert(version.root.checkpoint_txg.0);
        }
    }

    fn note_refusal(&mut self, refusal: &UserChangeRefused) {
        match refusal {
            UserChangeRefused::NoSpaceAfterRaisingTheFloor(refused) => {
                self.note_floor_raises(&refused.floor_raises);
                for version in &refused.versions_published {
                    self.user_change_txgs.insert(version.root.checkpoint_txg.0);
                }
            }
            UserChangeRefused::Publish {
                floor_raises,
                versions_published,
                ..
            } => {
                self.note_floor_raises(floor_raises);
                for version in versions_published {
                    self.user_change_txgs.insert(version.root.checkpoint_txg.0);
                }
            }
            UserChangeRefused::NoFileVersionToChange {
                floor_raises,
                versions_published,
                ..
            } => {
                self.note_floor_raises(floor_raises);
                for version in versions_published {
                    self.user_change_txgs.insert(version.root.checkpoint_txg.0);
                }
            }
            UserChangeRefused::DeviceTableOtherThanTheOneOfTheMount { .. }
            | UserChangeRefused::DevicesWithoutASelfVerifiedSystemConfiguration { .. }
            | UserChangeRefused::OwnDeviceNumbersDifferFromTheDeviceTable { .. }
            | UserChangeRefused::DevicesBehindTheCurrentVersionAndMissingItsUnitsOrItsRoot {
                ..
            }
            | UserChangeRefused::FloorRaiseFailedWhilePushingForSpace(_) => {}
        }
    }
}

/// 一个挂着的池：参数、两块盘、会话和发布日志。
struct Pool {
    geometry: Geometry,
    parameters: MakeFilesystemParameters,
    devices: Vec<(DeviceIdentity, Disk<'static>)>,
    session: MountedSession,
    log: PublishLog,
}

fn clone_session(session: &MountedSession) -> MountedSession {
    MountedSession {
        allocator: session.allocator.clone(),
        current: session.current.clone(),
        instance: session.instance,
        shadow_ledger: session.shadow_ledger,
        parameters_and_device_table: session.parameters_and_device_table.clone(),
        compaction: session.compaction.clone(),
    }
}

/// 现行那一版里数据单元的字节和前面各次发布的 journal 记录字节：探测的拷贝用不到，几万个单元时一份就上 GB，拷贝前先摘下来
/// （探测与整写同判那一步用带字节的整份拷贝核「摘掉不改判定」）。
struct StrippedBytes {
    data_unit_bytes: Vec<(usize, Vec<u8>)>,
    earlier_records: Vec<singlefs_core::transaction::WrittenJournalRecord>,
}

fn current_version_mut(version: &mut PoolVersion) -> &mut TransactionOutput {
    match version {
        PoolVersion::WithFile(output) | PoolVersion::WithTreesAndNoFile(output) => output,
        PoolVersion::WithoutFile(_) => stop_with("current_version_has_no_file", ""),
    }
}

fn strip_bytes_the_probe_does_not_need(session: &mut MountedSession) -> StrippedBytes {
    let output = current_version_mut(&mut session.current);
    let mut data_unit_bytes = Vec::new();
    for (index, unit) in output.units.iter_mut().enumerate() {
        if matches!(unit.identity, TransactionUnit::Data(_)) {
            data_unit_bytes.push((index, std::mem::take(&mut unit.bytes)));
        }
    }
    let earlier_records = std::mem::take(&mut output.earlier_records_of_this_publish);
    StrippedBytes {
        data_unit_bytes,
        earlier_records,
    }
}

fn restore_stripped_bytes(session: &mut MountedSession, stripped: StrippedBytes) {
    let output = current_version_mut(&mut session.current);
    for (index, bytes) in stripped.data_unit_bytes {
        output.units[index].bytes = bytes;
    }
    output.earlier_records_of_this_publish = stripped.earlier_records;
}

// ---------------------------------------------------------------------------------------------------------------------
// 造池：mkfs → 可写挂载 → 第一个文件（一个单元）→ 重开可写挂载
// ---------------------------------------------------------------------------------------------------------------------

/// 第一个文件发布之后、重开之前（mkfs 同一个进程）量到的锚点（登记第七节 A1、A2）。
#[derive(Clone, Debug, PartialEq, Eq)]
struct FirstFileAnchors {
    open_segment_start_slot: Option<u64>,
    closed_free_slots_in_the_open_segment_on_device_zero: Option<u64>,
    first_data_unit_slot: Option<u64>,
}

fn zero_content(data_units: u64) -> Vec<u8> {
    vec![0u8; usize::try_from(data_units * DATA_UNIT_PAYLOAD_BYTES).expect("内容字节数")]
}

fn build_pool(geometry: Geometry) -> (Pool, FirstFileAnchors) {
    let parameters = geometry.parameters();
    let mut devices = geometry.blank_devices();
    if let Err(error) = make_filesystem(&parameters, &mut devices) {
        stop_with(
            "make_filesystem_refused",
            &format!("geometry={} error={error:?}", geometry.name),
        );
    }
    let mut log = PublishLog::default();
    let mut mounted: Mounted = match mount_writable(&parameters, &mut devices) {
        Ok(mounted) => mounted,
        Err(error) => stop_with(
            "first_mount_refused",
            &format!("geometry={} error={error:?}", geometry.name),
        ),
    };
    log.note_mount(&mounted.output);
    let instance = mounted.output.instance;
    let (root, record_bytes) = match &mounted.current {
        PoolVersion::WithoutFile(version) => (version.root, version.record_bytes.clone()),
        PoolVersion::WithFile(_) | PoolVersion::WithTreesAndNoFile(_) => stop_with(
            "new_pool_already_has_a_file",
            &format!("geometry={}", geometry.name),
        ),
    };
    let content = zero_content(1);
    let first = {
        let mut writer = PoolWriter::new(&parameters, devices.as_mut_slice());
        publish_first_file(
            &mut writer,
            &mut mounted.allocator,
            &root,
            FirstFile {
                content: &content,
                write_time_seconds: WRITE_TIME_SECONDS,
            },
            instance,
            &record_bytes,
        )
    };
    let first = match first {
        Ok(first) => first,
        Err(error) => stop_with(
            "first_file_refused",
            &format!("geometry={} error={error:?}", geometry.name),
        ),
    };
    log.user_change_txgs.insert(first.root.checkpoint_txg.0);
    let open_segment = mounted.allocator.open_segment();
    let anchors = FirstFileAnchors {
        open_segment_start_slot: open_segment.map(|slot| slot.0),
        closed_free_slots_in_the_open_segment_on_device_zero: open_segment.map(|segment_start| {
            u64::try_from(
                (0..CLUSTER_SEGMENT_SLOTS)
                    .filter(|offset| {
                        mounted.allocator.devices[0].is_free(SlotNumber(segment_start.0 + offset))
                    })
                    .count(),
            )
            .expect("槽数")
        }),
        first_data_unit_slot: first
            .units
            .iter()
            .find(|unit| matches!(unit.identity, TransactionUnit::Data(_)))
            .map(|unit| unit.slot.0),
    };
    drop(mounted);
    let remounted = match mount_writable(&parameters, &mut devices) {
        Ok(remounted) => remounted,
        Err(error) => stop_with(
            "remount_refused",
            &format!("geometry={} error={error:?}", geometry.name),
        ),
    };
    let (output, session) = MountedSession::of_the_mount(remounted);
    log.note_mount(&output);
    (
        Pool {
            geometry,
            parameters,
            devices,
            session,
            log,
        },
        anchors,
    )
}

// ---------------------------------------------------------------------------------------------------------------------
// 历史里的改动
// ---------------------------------------------------------------------------------------------------------------------

/// 全零内容的缓冲：探测与历史都从它切前缀（虚拟内存，没写过的页读出来全 0，不占常驻内存）。
fn zero_bytes_of_units(units: u64) -> &'static [u8] {
    static ZERO_BYTES: std::sync::OnceLock<Vec<u8>> = std::sync::OnceLock::new();
    let all = ZERO_BYTES.get_or_init(|| {
        vec![
            0u8;
            usize::try_from(PROBE_FAR_ABOVE_ANY_JOURNAL_CAP_UNITS * DATA_UNIT_PAYLOAD_BYTES)
                .expect("探测缓冲字节数")
        ]
    });
    let length = usize::try_from(units * DATA_UNIT_PAYLOAD_BYTES).expect("内容字节数");
    assert!(length <= all.len(), "内容比探测缓冲还长：{units} 个单元");
    &all[..length]
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ChangeKind {
    /// 覆盖写：内容一个数据单元（`publish_overwrite` 的契约），第一个文件变成一个单元。
    OverwriteWithOneUnit,
    /// 顺序写：第一个文件整份换成这么多个数据单元。
    SequentialWriteOfUnits(u64),
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum ChangeOutcome {
    Published { floor_raises: usize },
    RefusedForSpace,
    RefusedForAnotherReason(String),
}

impl ChangeOutcome {
    fn label(&self) -> String {
        match self {
            ChangeOutcome::Published { floor_raises } => {
                format!("published_floor_raises_{floor_raises}")
            }
            ChangeOutcome::RefusedForSpace => "refused_for_space".to_string(),
            ChangeOutcome::RefusedForAnotherReason(reason) => format!("refused_other_{reason}"),
        }
    }
}

fn short_debug(value: &dyn std::fmt::Debug) -> String {
    let text = format!("{value:?}");
    let shortened: String = text.chars().take(80).collect();
    shortened.replace([' ', '\n', '\t', '"'], "_")
}

fn apply_change(pool: &mut Pool, kind: ChangeKind) -> ChangeOutcome {
    let write_time_seconds =
        WRITE_TIME_SECONDS + u64::try_from(pool.log.user_change_txgs.len()).expect("计数");
    let content = match kind {
        ChangeKind::OverwriteWithOneUnit => zero_bytes_of_units(1),
        ChangeKind::SequentialWriteOfUnits(units) => zero_bytes_of_units(units),
    };
    let file = FirstFile {
        content,
        write_time_seconds,
    };
    let change = match kind {
        ChangeKind::OverwriteWithOneUnit => UserChange::Overwrite(file),
        ChangeKind::SequentialWriteOfUnits(_) => UserChange::SequentialWrite(file),
    };
    match pool.session.publish_user_change(&mut pool.devices, change) {
        Ok(published) => {
            pool.log.note_published(&published);
            ChangeOutcome::Published {
                floor_raises: published.floor_raises.len(),
            }
        }
        Err(refusal) => {
            pool.log.note_refusal(&refusal);
            match &refusal {
                UserChangeRefused::NoSpaceAfterRaisingTheFloor(_) => ChangeOutcome::RefusedForSpace,
                UserChangeRefused::Publish { cause, .. } if refusal_is_short_of_space(cause) => {
                    ChangeOutcome::RefusedForSpace
                }
                other @ (UserChangeRefused::DeviceTableOtherThanTheOneOfTheMount { .. }
                | UserChangeRefused::NoFileVersionToChange { .. }
                | UserChangeRefused::DevicesWithoutASelfVerifiedSystemConfiguration { .. }
                | UserChangeRefused::OwnDeviceNumbersDifferFromTheDeviceTable { .. }
                | UserChangeRefused::DevicesBehindTheCurrentVersionAndMissingItsUnitsOrItsRoot {
                    ..
                }
                | UserChangeRefused::Publish { .. }
                | UserChangeRefused::FloorRaiseFailedWhilePushingForSpace(_)) => {
                    ChangeOutcome::RefusedForAnotherReason(short_debug(other))
                }
            }
        }
    }
}

/// 一次正常卸载推了几次空发布、卸载那一刻现行 txg 是多少（登记 A3 核：现行 txg mod 3 = 1 时 3 次，= 0 或 2 时 2 次）。
struct UnmountSequenceObserved {
    current_txg_before_the_unmount: u64,
    empty_publishes_pushed: usize,
}

/// 正常卸载再可写挂载（多次挂载族、被抛弃根与净释放族的「一轮」）：卸载推空发布的 txg 记进日志，挂载写行与暖机的 txg 同样记。
fn unmount_and_mount_again(pool: &mut Pool) -> UnmountSequenceObserved {
    let current_txg_before_the_unmount = txg_of_pool_version(&pool.session.current);
    let mut empty_publishes_pushed = 0usize;
    match unmount(
        &pool.parameters,
        &mut pool.devices,
        &mut pool.session.allocator,
        &mut pool.session.current,
        pool.session.shadow_ledger,
    ) {
        Ok(Unmounted::FloorRaisedToTheCurrentVersion(raised)) => {
            empty_publishes_pushed = raised.publishes.len();
            for publish in &raised.publishes {
                pool.log
                    .empty_publish_txgs
                    .insert(publish.root.checkpoint_txg.0);
            }
        }
        Ok(Unmounted::NothingWrittenOnAVersionWithoutFile { .. }) => {}
        Err(error) => stop_with(
            "unmount_refused",
            &format!(
                "geometry={} error={}",
                pool.geometry.name,
                short_debug(&error)
            ),
        ),
    }
    mount_again(pool);
    UnmountSequenceObserved {
        current_txg_before_the_unmount,
        empty_publishes_pushed,
    }
}

fn mount_again(pool: &mut Pool) -> MountOutput {
    let mounted = match mount_writable(&pool.parameters, &mut pool.devices) {
        Ok(mounted) => mounted,
        Err(error) => stop_with(
            "remount_refused",
            &format!(
                "geometry={} error={}",
                pool.geometry.name,
                short_debug(&error)
            ),
        ),
    };
    let (output, session) = MountedSession::of_the_mount(mounted);
    pool.log.note_mount(&output);
    pool.session = session;
    output
}

// ---------------------------------------------------------------------------------------------------------------------
// 一个状态上的读数（不含探测）
// ---------------------------------------------------------------------------------------------------------------------

#[derive(Clone, Debug)]
struct DeviceReading {
    capacity_slots: u64,
    /// 现行账里仍分配的槽（从盘上读的现行那一版的分配记录）。
    live_slots: u64,
    /// 同一个量从内存里的现行那一版取；两份对不上记 `records_agree_with_the_disk` 为假。
    live_slots_in_memory_account: u64,
    available_bytes: i128,
    /// 「挂载期承诺量 + 环窗口 + 不可回收 + 全池三项摊到这块盘的份」（登记 5.2 的 Cm），字节。
    common_commitments_bytes: i128,
    ring_window_slots: u64,
    unblocked_slots: u64,
    overlap_slots_allocated_and_isolated: u64,
    isolated_slots: u64,
    allocated_slots: u64,
    deferred_slots: u64,
    /// 这次挂载开过的聚簇段里、三种挡位都没置的槽（登记「段内关着的空槽」）。
    closed_free_slots_in_cluster_segments: u64,
}

impl DeviceReading {
    fn available_slots_rounded_down(&self) -> i128 {
        self.available_bytes.div_euclid(i128::from(SLOT_BYTES))
    }

    /// (可用 + Cm) 换成槽：等于 容量 − 已分配 − 隔离（分解成立时，见 [`StateReading::decomposition_holds`]）。
    fn available_plus_common_slots(&self) -> i128 {
        (self.available_bytes + self.common_commitments_bytes).div_euclid(i128::from(SLOT_BYTES))
    }
}

#[derive(Clone, Debug)]
struct StateReading {
    devices: Vec<DeviceReading>,
    data_units: u64,
    checkpoint_cost_blocks: u64,
    /// 滞后量（登记 5.1）：按全池口径的槽数。
    lag_slots: u64,
    /// 释放代高于门槛、却既不在用户改动日志里也不在空发布日志里的已释放槽（该是 0）。
    lag_unattributed_slots: u64,
    oldest_valid_root_txg: u64,
    fourth_newest_nonempty_valid_root_txg: Option<u64>,
    cluster_segment_count: u64,
    inode_tree_internal_levels: u64,
    records_agree_with_the_disk: bool,
    /// 容量 − 已分配 − 隔离 = 可用 + Cm 在每块盘上都成立（Cm 的拆法没丢项）。
    decomposition_holds: bool,
    available_is_whole_slots: bool,
}

fn floor_half(value: i128) -> i128 {
    value.div_euclid(2)
}

/// 现行账里的记录按盘汇总仍分配的槽数。
fn live_slots_per_device(
    records: &[singlefs_core::allocator::AllocationRecord],
    devices: usize,
) -> Vec<u64> {
    let mut live = vec![0u64; devices];
    for record in records.iter().filter(|record| !record.is_released) {
        let index = usize::try_from(record.device.0).expect("盘号");
        live[index] += u64::from(record.span_slots);
    }
    live
}

fn valid_root_txgs_in_the_ring(allocator: &PoolAllocator) -> Vec<u64> {
    let occupancy = allocator
        .root_ring_occupancy()
        .expect("分配器上装着根环：挂载与 mkfs 建的分配器都装");
    occupancy
        .ring_slots_known_to_hold_a_root()
        .into_iter()
        .filter_map(|ring_slot| occupancy.checkpoint_txg_of_the_valid_root_recorded_on(ring_slot))
        .map(|txg| txg.0)
        .collect()
}

/// 登记 5.2 的残差（单位槽，一块盘）：改写检查残差（登记记作 R2）= (可用 + Cm) − (没挡的 + 扣住账 − 重叠)；
/// 原式残差（登记记作 R3）= 可用 + Cm − 没挡的（欠账行 C356 原式）；阳性对照用的「改写检查残差去掉重叠项」。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Residuals {
    rewritten_check_residual_with_the_held_and_overlap_terms: i128,
    rewritten_check_residual_without_the_overlap_term: i128,
    original_formula_residual: i128,
}

fn residuals_of(
    available_plus_common_slots: i128,
    unblocked_slots: i128,
    overlap_allocated_and_isolated_slots: i128,
    held_ledger_slots: i128,
) -> Residuals {
    Residuals {
        rewritten_check_residual_with_the_held_and_overlap_terms: available_plus_common_slots
            - (unblocked_slots + held_ledger_slots - overlap_allocated_and_isolated_slots),
        rewritten_check_residual_without_the_overlap_term: available_plus_common_slots
            - (unblocked_slots + held_ledger_slots),
        original_formula_residual: available_plus_common_slots - unblocked_slots,
    }
}

/// 段内关着的空槽（登记第一节「段内关着的空槽」）：这次挂载开过的聚簇段里、三种挡位都没置的槽数；数据单元落不进去（D3 已定项 8 第 2 条）。
fn closed_free_slots_in_segments(
    map: &singlefs_core::allocator::DeviceFreeMap,
    segments_opened_by_this_mount: &BTreeSet<SlotNumber>,
) -> u64 {
    let counted: usize = segments_opened_by_this_mount
        .iter()
        .map(|segment_start| {
            (0..CLUSTER_SEGMENT_SLOTS)
                .filter(|offset| map.is_free(SlotNumber(segment_start.0 + offset)))
                .count()
        })
        .sum();
    u64::try_from(counted).expect("槽数")
}

/// 滞后量那一项的读数（登记 5.1）：门槛 = max(第 4 新的非空有效根的 txg，环里最旧有效根的 txg)（非空有效根不足 4 个时第 4 新取最旧有效根）；
/// 滞后 = 现行账里已释放、释放代高于门槛、释放它的那次发布是用户改动（不是空发布）的槽，全池口径（各盘的跨度之和除以盘数）。
#[derive(Clone, Debug, PartialEq, Eq)]
struct LagReading {
    lag_slots: u64,
    unattributed_slots: u64,
    oldest_valid_root_txg: u64,
    fourth_newest_nonempty_valid_root_txg: Option<u64>,
}

fn lag_reading_of(
    records: &[singlefs_core::allocator::AllocationRecord],
    valid_root_txgs: &[u64],
    log: &PublishLog,
    device_count: u64,
) -> LagReading {
    let oldest_valid_root_txg = valid_root_txgs.iter().copied().min().unwrap_or(0);
    let mut nonempty_valid: Vec<u64> = valid_root_txgs
        .iter()
        .copied()
        .filter(|txg| log.user_change_txgs.contains(txg))
        .collect();
    nonempty_valid.sort_unstable_by(|left, right| right.cmp(left));
    let fourth_newest_nonempty_valid_root_txg = nonempty_valid.get(3).copied();
    let threshold = fourth_newest_nonempty_valid_root_txg
        .unwrap_or(oldest_valid_root_txg)
        .max(oldest_valid_root_txg);
    let mut lagging_span_slots = 0u64;
    let mut unattributed_span_slots = 0u64;
    for record in records
        .iter()
        .filter(|record| record.is_released && record.generation.0 > threshold)
    {
        let txg = record.generation.0;
        if log.user_change_txgs.contains(&txg) {
            lagging_span_slots += u64::from(record.span_slots);
        } else if !log.empty_publish_txgs.contains(&txg) {
            unattributed_span_slots += u64::from(record.span_slots);
        }
    }
    LagReading {
        lag_slots: lagging_span_slots / device_count,
        unattributed_slots: unattributed_span_slots,
        oldest_valid_root_txg,
        fourth_newest_nonempty_valid_root_txg,
    }
}

fn read_state(pool: &Pool) -> StateReading {
    let session = &pool.session;
    let version = current_version(&session.current);
    let allocator = &session.allocator;
    let reading: AdmissionReading = admission_reading_before_a_publish(allocator, Some(version));
    let replicas = reading.replicas();
    let pool_wide = reading.pool_wide();
    let shares: i128 = [
        pool_wide.pending_delete,
        pool_wide.committed_reservation,
        pool_wide.checkpoint_reserve_pool,
    ]
    .iter()
    .map(|commitment| i128::from(commitment.share_of_one_device(replicas).0))
    .sum();
    let available = reading.available_on_each_device();
    let disk_records = allocation_records_under_root(&pool.devices, &version.root);
    let device_count = allocator.devices.len();
    let live_in_memory = live_slots_per_device(&version.allocation_records, device_count);
    let (live_on_disk, records_agree_with_the_disk) = match &disk_records {
        Ok(records) => {
            let mut from_disk = records.clone();
            from_disk.sort();
            let mut from_memory = version.allocation_records.clone();
            from_memory.sort();
            (
                live_slots_per_device(records, device_count),
                from_disk == from_memory,
            )
        }
        Err(_) => (live_in_memory.clone(), false),
    };
    let lag = lag_reading_of(
        &version.allocation_records,
        &valid_root_txgs_in_the_ring(allocator),
        &pool.log,
        u64::try_from(device_count).expect("盘数"),
    );
    let cluster_segments = allocator.cluster_segments();
    let mut decomposition_holds = true;
    let mut available_is_whole_slots = true;
    let devices: Vec<DeviceReading> = allocator
        .devices
        .iter()
        .zip(reading.per_device())
        .zip(available.iter())
        .enumerate()
        .map(|(index, ((map, terms), (_, available_bytes)))| {
            let common_commitments_bytes = i128::from(terms.mount_time_commitment.0)
                + i128::from(terms.ring_window.0)
                + i128::from(terms.unreclaimable.0)
                + shares;
            let closed_free_slots_in_cluster_segments =
                closed_free_slots_in_segments(map, cluster_segments);
            let capacity_bytes = i128::from(terms.capacity.0);
            let by_decomposition = capacity_bytes
                - i128::from(terms.allocated.0)
                - i128::from(
                    terms
                        .isolated_for_abandoned_roots_and_unapplied_journal_records
                        .0,
                );
            if available_bytes.0 + common_commitments_bytes != by_decomposition {
                decomposition_holds = false;
            }
            if available_bytes.0.rem_euclid(i128::from(SLOT_BYTES)) != 0 {
                available_is_whole_slots = false;
            }
            DeviceReading {
                capacity_slots: map.unit_area_slots(),
                live_slots: live_on_disk[index],
                live_slots_in_memory_account: live_in_memory[index],
                available_bytes: available_bytes.0,
                common_commitments_bytes,
                ring_window_slots: terms.ring_window.0 / SLOT_BYTES,
                unblocked_slots: map.unblocked_slots(),
                overlap_slots_allocated_and_isolated: map
                    .slots_counted_both_as_allocated_and_as_isolated(),
                isolated_slots: map.isolated_slots(),
                allocated_slots: map.allocated_slots(),
                deferred_slots: map.deferred_slots(),
                closed_free_slots_in_cluster_segments,
            }
        })
        .collect();
    let inode_tree_internal_levels = match version.inode_leaf_containers.len() {
        0 | 1 => 0,
        containers => stop_with(
            "inode_tree_has_more_than_one_leaf_container",
            &format!("containers={containers}"),
        ),
    };
    StateReading {
        devices,
        data_units: u64::try_from(version.data_pointers.len()).expect("单元数"),
        checkpoint_cost_blocks: checkpoint_cost_of_the_version_to_build_on(
            Some(version),
            allocator,
        )
        .0,
        lag_slots: lag.lag_slots,
        lag_unattributed_slots: lag.unattributed_slots,
        oldest_valid_root_txg: lag.oldest_valid_root_txg,
        fourth_newest_nonempty_valid_root_txg: lag.fourth_newest_nonempty_valid_root_txg,
        cluster_segment_count: u64::try_from(cluster_segments.len()).expect("段数"),
        inode_tree_internal_levels,
        records_agree_with_the_disk,
        decomposition_holds,
        available_is_whole_slots,
    }
}

// ---------------------------------------------------------------------------------------------------------------------
// F2 的臂（登记 5.1）：每条臂报的字节
// ---------------------------------------------------------------------------------------------------------------------

/// 登记 5.1 的候选臂加两条控制（恒多报的阳性对照、恒报 0 的阴性对照）。次序固定，产物行按这个次序打。
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
enum Arm {
    Candidate(CandidateArm),
    PositiveControlAlwaysOverReports,
    NegativeControlReportsZero,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
enum CandidateArm {
    /// D16 已定项 1 `df` 字面，活元数据取 0。
    DfLiteralWithoutLiveMetadata,
    /// D16 已定项 1 `df` 字面，活元数据取现行账里活着的非数据槽。
    DfLiteralWithAllLiveMetadata,
    /// D28 式子可用、各盘取小，按槽字节报。
    AvailableMinimumOverDevicesInSlotBytes,
    /// 同一臂，按单元换载荷字节。
    AvailableMinimumOverDevicesInPayloadBytes,
    /// D28 可用两盘相加 ÷ 副本数。
    AvailableSummedOverDevicesDividedByReplicas,
    /// W2 再扣写 n 单元那一次的非数据需求上界。
    AvailableMinimumLessTheNonDataDemandOfTheWrite,
}

const ALL_CANDIDATE_ARMS: [CandidateArm; 6] = [
    CandidateArm::DfLiteralWithoutLiveMetadata,
    CandidateArm::DfLiteralWithAllLiveMetadata,
    CandidateArm::AvailableMinimumOverDevicesInSlotBytes,
    CandidateArm::AvailableMinimumOverDevicesInPayloadBytes,
    CandidateArm::AvailableSummedOverDevicesDividedByReplicas,
    CandidateArm::AvailableMinimumLessTheNonDataDemandOfTheWrite,
];

impl CandidateArm {
    /// 产物里的臂名（登记 5.1 的 W1-零、W1-全、W2-字节、W2-载荷、W3、W4）。
    fn label(self) -> &'static str {
        match self {
            CandidateArm::DfLiteralWithoutLiveMetadata => "df_literal_without_live_metadata",
            CandidateArm::DfLiteralWithAllLiveMetadata => "df_literal_with_all_live_metadata",
            CandidateArm::AvailableMinimumOverDevicesInSlotBytes => {
                "available_minimum_over_devices_in_slot_bytes"
            }
            CandidateArm::AvailableMinimumOverDevicesInPayloadBytes => {
                "available_minimum_over_devices_in_payload_bytes"
            }
            CandidateArm::AvailableSummedOverDevicesDividedByReplicas => {
                "available_summed_over_devices_divided_by_replicas"
            }
            CandidateArm::AvailableMinimumLessTheNonDataDemandOfTheWrite => {
                "available_minimum_less_the_non_data_demand_of_the_write"
            }
        }
    }
}

impl Arm {
    fn label(self) -> &'static str {
        match self {
            Arm::Candidate(candidate) => candidate.label(),
            Arm::PositiveControlAlwaysOverReports => "always_over_reports_control",
            Arm::NegativeControlReportsZero => "zero_control",
        }
    }
}

fn all_arms() -> Vec<Arm> {
    let mut arms: Vec<Arm> = ALL_CANDIDATE_ARMS
        .iter()
        .copied()
        .map(Arm::Candidate)
        .collect();
    arms.push(Arm::PositiveControlAlwaysOverReports);
    arms.push(Arm::NegativeControlReportsZero);
    arms
}

fn ceiling_division(numerator: u64, denominator: u64) -> u64 {
    numerator.div_ceil(denominator)
}

/// 写 n 个数据单元那一次的非数据需求上界，单位槽（登记 5.1 W4）：下段叶 ⌈n ÷ 144⌉ 逐层 ⌈÷ 147⌉ 到 1 的节点数之和（n ≤ 1 取 0）
/// + 1（上段叶）+ 2（inode 叶容器）+ inode 树这一版的内部层数 + c_max。
fn non_data_demand_upper_bound_slots(
    units: u64,
    inode_tree_internal_levels: u64,
    checkpoint_cost_blocks: u64,
) -> u64 {
    let lower_tree_nodes = if units <= 1 {
        0
    } else {
        let mut level_nodes = ceiling_division(units, EXTENT_TREE_LOWER_LEAF_DATA_UNITS);
        let mut total = level_nodes;
        while level_nodes > 1 {
            level_nodes = ceiling_division(level_nodes, EXTENT_TREE_INTERNAL_FANOUT);
            total += level_nodes;
        }
        total
    };
    lower_tree_nodes + 1 + 2 + inode_tree_internal_levels + checkpoint_cost_blocks
}

/// W4：满足 2n + U(n) ≤ 各盘可用槽数的最小值 的最大 n（单调，二分）。
fn largest_units_satisfying_the_w4_demand(
    available_slots: u64,
    inode_tree_internal_levels: u64,
    checkpoint_cost_blocks: u64,
) -> u64 {
    let fits = |units: u64| {
        2 * units
            + non_data_demand_upper_bound_slots(
                units,
                inode_tree_internal_levels,
                checkpoint_cost_blocks,
            )
            <= available_slots
    };
    if !fits(0) {
        return 0;
    }
    let mut known_to_fit = 0u64;
    let mut known_not_to_fit = available_slots / 2 + 1;
    while known_not_to_fit - known_to_fit > 1 {
        let middle = known_to_fit + (known_not_to_fit - known_to_fit) / 2;
        if fits(middle) {
            known_to_fit = middle;
        } else {
            known_not_to_fit = middle;
        }
    }
    known_to_fit
}

impl StateReading {
    fn minimum_available_slots(&self) -> u64 {
        let minimum = self
            .devices
            .iter()
            .map(DeviceReading::available_slots_rounded_down)
            .min()
            .expect("至少一块盘");
        u64::try_from(minimum.max(0)).expect("可用槽数")
    }

    fn summed_available_slots(&self) -> u64 {
        let total: i128 = self
            .devices
            .iter()
            .map(|device| device.available_slots_rounded_down().max(0))
            .sum();
        u64::try_from(total).expect("可用槽数之和")
    }

    fn pool_free_slots_counting_released(&self) -> i128 {
        self.devices
            .iter()
            .map(|device| i128::from(device.capacity_slots) - i128::from(device.live_slots))
            .sum()
    }

    fn live_non_data_slots_summed(&self) -> i128 {
        self.devices
            .iter()
            .map(|device| i128::from(device.live_slots) - i128::from(2 * self.data_units))
            .sum()
    }

    /// 这条臂此刻报的字节（可为负；报给用户时按 0 封底，见 [`reported_bytes`]）。
    fn raw_bytes_of(&self, arm: Arm) -> i128 {
        let slot_bytes = i128::from(SLOT_BYTES);
        let payload_bytes = i128::from(DATA_UNIT_PAYLOAD_BYTES);
        let cost = i128::from(self.checkpoint_cost_blocks);
        let reserve_slots = 10 + 7 * cost;
        let residue_slots = 5 + 6 * cost;
        let lag = i128::from(self.lag_slots);
        match arm {
            Arm::Candidate(CandidateArm::DfLiteralWithoutLiveMetadata) => {
                slot_bytes
                    * (floor_half(self.pool_free_slots_counting_released())
                        - reserve_slots
                        - residue_slots
                        - lag)
            }
            Arm::Candidate(CandidateArm::DfLiteralWithAllLiveMetadata) => {
                slot_bytes
                    * (floor_half(self.pool_free_slots_counting_released())
                        + floor_half(self.live_non_data_slots_summed())
                        - reserve_slots
                        - residue_slots
                        - lag)
            }
            Arm::Candidate(CandidateArm::AvailableMinimumOverDevicesInSlotBytes) => {
                slot_bytes * i128::from(self.minimum_available_slots())
            }
            Arm::Candidate(CandidateArm::AvailableMinimumOverDevicesInPayloadBytes) => {
                payload_bytes * i128::from(self.minimum_available_slots() / 2)
            }
            Arm::Candidate(CandidateArm::AvailableSummedOverDevicesDividedByReplicas) => {
                slot_bytes * i128::from(self.summed_available_slots() / DEVICE_COUNT)
            }
            Arm::Candidate(CandidateArm::AvailableMinimumLessTheNonDataDemandOfTheWrite) => {
                payload_bytes
                    * i128::from(largest_units_satisfying_the_w4_demand(
                        self.minimum_available_slots(),
                        self.inode_tree_internal_levels,
                        self.checkpoint_cost_blocks,
                    ))
            }
            Arm::PositiveControlAlwaysOverReports => {
                slot_bytes * self.pool_free_slots_counting_released()
            }
            Arm::NegativeControlReportsZero => 0,
        }
    }
}

/// 报给用户的字节：负数按 0 报（`df` 不报负）。
fn reported_bytes(raw: i128) -> u64 {
    u64::try_from(raw.max(0)).expect("报的字节装得进 u64")
}

// ---------------------------------------------------------------------------------------------------------------------
// T 的探测（登记 5.1）
// ---------------------------------------------------------------------------------------------------------------------

#[derive(Clone, Debug, PartialEq, Eq)]
enum ProbeVerdict {
    /// 数据单元的写到达了设备：三道空间闸都放行。
    Admitted,
    /// 整写做成了（只在整写同判那一步出现）。
    Completed,
    RefusedForSpace,
    /// 在飞记录数超过上限：不是空间不够，也盖住了后面的式子那一判。
    JournalLimit {
        records_of_the_publish: u64,
        record_limit: u64,
    },
    Other(String),
}

fn run_probe(
    devices: &[(DeviceIdentity, Disk<'static>)],
    mut session: MountedSession,
    units: u64,
    policy: DataWritePolicy,
) -> ProbeVerdict {
    let mut overlay: Vec<(DeviceIdentity, Disk<'_>)> = devices
        .iter()
        .map(|(identity, disk)| (*identity, disk.overlay(policy)))
        .collect();
    let change = UserChange::SequentialWrite(FirstFile {
        content: zero_bytes_of_units(units),
        write_time_seconds: WRITE_TIME_SECONDS + 1,
    });
    let result = session.publish_user_change(&mut overlay, change);
    let data_write_reached = overlay.iter().any(|(_, disk)| disk.data_write_reached);
    classify_probe_result(result, data_write_reached)
}

/// 一次探测的会话结果换成探测结局：数据写到达了设备 ⇒ 放行；空间不够的两种 ⇒ 被空间拒；记录数超上限 ⇒ 不是空间不够；别的 ⇒ 取不出。
fn classify_probe_result(
    result: Result<UserChangePublished, UserChangeRefused>,
    data_write_reached: bool,
) -> ProbeVerdict {
    match result {
        Ok(_) if data_write_reached => {
            ProbeVerdict::Other("data_write_reached_yet_published".to_string())
        }
        Ok(_) => ProbeVerdict::Completed,
        Err(_) if data_write_reached => ProbeVerdict::Admitted,
        Err(UserChangeRefused::NoSpaceAfterRaisingTheFloor(_)) => ProbeVerdict::RefusedForSpace,
        Err(UserChangeRefused::Publish { cause, .. }) => {
            if refusal_is_short_of_space(&cause) {
                ProbeVerdict::RefusedForSpace
            } else if let PublishError::JournalRecordsOfThePublishExceedTheLimit {
                records_of_the_publish,
                record_limit,
            } = cause
            {
                ProbeVerdict::JournalLimit {
                    records_of_the_publish,
                    record_limit,
                }
            } else {
                ProbeVerdict::Other(short_debug(&cause))
            }
        }
        Err(other) => ProbeVerdict::Other(short_debug(&other)),
    }
}

/// 一个状态上的探测器：拿着摘掉了大字节的会话（[`strip_bytes_the_probe_does_not_need`]），每次探测拷一份会话、盘上套叠层。
struct StateProber<'pool> {
    pool: &'pool Pool,
    probe_threads: usize,
    probes_run: AtomicU64,
}

/// 探测搜索要的东西：一批单元数各探一次（数据单元的写到达就放行），和一轮最多并行探几个。真探测器是 [`StateProber`]，单测里换假的。
trait UnitsProbe {
    fn probe_in_parallel(&self, unit_counts: &[u64]) -> Vec<ProbeVerdict>;
    fn parallel_probe_count(&self) -> usize;
    fn probe_one(&self, units: u64) -> ProbeVerdict {
        self.probe_in_parallel(&[units]).remove(0)
    }
}

impl UnitsProbe for StateProber<'_> {
    fn probe_in_parallel(&self, unit_counts: &[u64]) -> Vec<ProbeVerdict> {
        self.probe_many(unit_counts, DataWritePolicy::FailTheFirstDataWriteAndNoteIt)
    }

    fn parallel_probe_count(&self) -> usize {
        self.probe_threads
    }
}

impl StateProber<'_> {
    fn probe_many(&self, unit_counts: &[u64], policy: DataWritePolicy) -> Vec<ProbeVerdict> {
        self.probes_run.fetch_add(
            u64::try_from(unit_counts.len()).expect("探测数"),
            Ordering::Relaxed,
        );
        let sessions: Vec<MountedSession> = unit_counts
            .iter()
            .map(|_| clone_session(&self.pool.session))
            .collect();
        let devices = &self.pool.devices;
        if sessions.len() == 1 {
            let session = sessions.into_iter().next().expect("一份会话");
            return vec![run_probe(devices, session, unit_counts[0], policy)];
        }
        std::thread::scope(|scope| {
            let handles: Vec<_> = sessions
                .into_iter()
                .zip(unit_counts.iter().copied())
                .map(|(session, units)| {
                    std::thread::Builder::new()
                        .stack_size(THREAD_STACK_BYTES)
                        .spawn_scoped(scope, move || run_probe(devices, session, units, policy))
                        .expect("起探测线程")
                })
                .collect();
            handles
                .into_iter()
                .map(|handle| handle.join().expect("探测线程没 panic"))
                .collect()
        })
    }
}

/// 默认环的在飞记录上限把「空间很空」的状态的探测截住：记录数 = 单元数 + 固定的几条，固定那几条从拒绝里读（`records_of_the_publish − 试的单元数`）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct JournalModel {
    /// 不带 journal 记录上限就不会被拒的最大单元数；还没从拒绝里学到时 `None`。
    largest_units_below_the_record_limit: Option<u64>,
    records_beyond_one_per_unit: Option<u64>,
    /// 拒绝里报的在飞记录上限（条）：一次顺序写至少每单元一条记录，放行的单元数不会超过它，按需收紧拿它当下界的上界。
    record_limit: Option<u64>,
}

impl JournalModel {
    const NOT_LEARNED_YET: JournalModel = JournalModel {
        largest_units_below_the_record_limit: None,
        records_beyond_one_per_unit: None,
        record_limit: None,
    };

    fn learn_from(&mut self, tried_units: u64, records_of_the_publish: u64, record_limit: u64) {
        // 记录数 = 单元数 + 最后一个事务点名的非数据角色摊成的几条；后者随单元数与状态缓慢地变，所以这里学到的只是起点，
        // 探测到上限处被拒时由 [`probe_at_the_journal_cap`] 按超出的条数往下收。
        let extra_records = records_of_the_publish
            .checked_sub(tried_units)
            .unwrap_or_else(|| {
                stop_with(
                    "journal_records_fewer_than_units",
                    &format!("units={tried_units} records={records_of_the_publish}"),
                )
            });
        self.records_beyond_one_per_unit = Some(extra_records);
        self.largest_units_below_the_record_limit =
            Some(record_limit.saturating_sub(extra_records));
        self.record_limit = Some(record_limit);
    }
}

/// 一个状态上 T 的结果。
#[derive(Clone, Debug, PartialEq, Eq)]
enum WritableUnits {
    /// 二分到 T 放行、T + 1 因空间被拒。
    Exact {
        units: u64,
        non_monotone_probes_seen: bool,
    },
    /// 记录上限处放行（空间比记录上限还宽）：T 至少这么多个单元。登记字面按「被别的原因拒」记取不出。
    /// `lower_bound_was_tightened`：下界是不是往上收过（[`LowerBoundTightening`]）；没收的停在记录上限处放行的那个点，下界照样成立。
    AtLeastBecauseOfTheJournalCap {
        lower_bound_units: u64,
        lower_bound_was_tightened: bool,
    },
    Unreadable(String),
}

/// 一个状态上空间的两个必要条件给出的 T 的上界（单位单元）：装不下的必拒，所以 T 不超过它们；只用来定探测的起点，不进判定。
fn upper_bound_units_by_the_space_conditions(pool: &Pool, reading: &StateReading) -> u64 {
    let formula_bound = reading.minimum_available_slots() / 2;
    let landing_bound = pool
        .session
        .allocator
        .devices
        .iter()
        .map(|map| {
            map.unblocked_slot_pairs_outside_the_cluster_segments(
                pool.session.allocator.cluster_segments(),
            )
        })
        .min()
        .expect("至少一块盘");
    formula_bound.min(landing_bound)
}

fn interior_points(low_exclusive: u64, high_exclusive: u64, count: usize) -> Vec<u64> {
    let gap = high_exclusive - low_exclusive;
    let mut points: Vec<u64> = (1..=count)
        .map(|index| {
            low_exclusive
                + gap * u64::try_from(index).expect("下标")
                    / (u64::try_from(count).expect("个数") + 1)
        })
        .filter(|point| *point > low_exclusive && *point < high_exclusive)
        .collect();
    points.dedup();
    points
}

/// 单元数的空间上界高过这个数时，先在这个状态上学一次 journal 记录上限（默认环的上限 65536 条在它之上）。
const UNITS_ABOVE_WHICH_THE_JOURNAL_CAP_IS_LEARNED: u64 = 60_000;

fn unreadable_because(verdict: &ProbeVerdict, units: u64) -> WritableUnits {
    WritableUnits::Unreadable(format!("units_{units}_{}", short_debug(verdict)))
}

/// 在 journal 记录上限附近探测：被记录上限拒时，按超出的条数把单元数往下收再试（最多 16 次），收到的单元数记回模型。交回最后试的单元数与它的结局。
fn probe_at_the_journal_cap<Prober: UnitsProbe>(
    prober: &Prober,
    model: &mut JournalModel,
    starting_units: u64,
) -> (u64, ProbeVerdict) {
    let mut units = starting_units;
    for _ in 0..16 {
        match prober.probe_one(units) {
            ProbeVerdict::JournalLimit {
                records_of_the_publish,
                record_limit,
            } => {
                let overshoot = records_of_the_publish.saturating_sub(record_limit).max(1);
                units = units.saturating_sub(overshoot);
                model.largest_units_below_the_record_limit = Some(units);
                model.record_limit = Some(record_limit);
            }
            other @ (ProbeVerdict::Admitted
            | ProbeVerdict::Completed
            | ProbeVerdict::RefusedForSpace
            | ProbeVerdict::Other(_)) => return (units, other),
        }
    }
    (
        units,
        ProbeVerdict::Other("journal_cap_did_not_settle".to_string()),
    )
}

/// 往上收下界时，一轮里步长翻倍的点最多取几个（最远 2^12 个单元之外；再远的点会撞上落点那一道，把收紧提前停掉）。
const TIGHTENING_DOUBLINGS_AT_MOST: usize = 12;

/// 往上收下界时最多跑几轮。
const TIGHTENING_ROUNDS_AT_MOST: usize = 64;

/// 记录上限处放行之后，把下界往上收到「再多一个单元就被记录数拒」。
/// 模型里的上限是在更远的单元数处学来的（[`JournalModel::learn_from`]），固定的那几条记录随单元数缓慢地变，所以它偏低，放行的点不一定贴着上限。
/// 先用 1、2、4、… 倍的步长往上找一个被记录数拒的点，再在放行点与被拒点之间 K 路收缩；
/// 同一轮里排在最小的被记录数拒的点之后的结局不看（探测次序是落点、记录数、式子，被拒点之上的结局被它盖住）；
/// 碰到放行与记录数拒之外的结局就停在当时的下界：放行的点已经证明 T 至少这么多，只是没收紧。
/// 收到的下界不写回模型：下一个状态的起探点只取学到的记录上限，与这一个状态收没收紧无关，按需收紧与一律收紧才在每个状态上给同样的起探点。
fn tighten_the_lower_bound_to_the_record_limit<Prober: UnitsProbe>(
    prober: &Prober,
    record_limit: Option<u64>,
    admitted_units: u64,
) -> u64 {
    let parallel = prober.parallel_probe_count().max(1);
    let doublings = parallel.min(TIGHTENING_DOUBLINGS_AT_MOST);
    let mut lowest_admitted = admitted_units;
    let mut lowest_over_the_record_limit: Option<u64> = None;
    let mut step: u64 = 1;
    for _ in 0..TIGHTENING_ROUNDS_AT_MOST {
        let points: Vec<u64> = match lowest_over_the_record_limit {
            Some(over) => {
                if over - lowest_admitted <= 1 {
                    break;
                }
                interior_points(lowest_admitted, over, parallel)
            }
            None => (0..doublings)
                .map(|index| lowest_admitted.saturating_add(step.saturating_mul(1u64 << index)))
                .collect(),
        };
        let verdicts = prober.probe_in_parallel(&points);
        let mut admitted_points = Vec::new();
        let mut smallest_over_the_record_limit = None;
        let mut stopped_by_another_verdict = false;
        for (point, verdict) in points.iter().zip(&verdicts) {
            match verdict {
                ProbeVerdict::Admitted => admitted_points.push(*point),
                ProbeVerdict::JournalLimit { .. } => {
                    smallest_over_the_record_limit = Some(*point);
                    break;
                }
                ProbeVerdict::Completed
                | ProbeVerdict::RefusedForSpace
                | ProbeVerdict::Other(_) => {
                    stopped_by_another_verdict = true;
                    break;
                }
            }
        }
        if let Some(point) = admitted_points.iter().copied().max() {
            lowest_admitted = lowest_admitted.max(point);
        }
        if let Some(point) = smallest_over_the_record_limit.filter(|point| *point > lowest_admitted)
        {
            lowest_over_the_record_limit =
                Some(lowest_over_the_record_limit.map_or(point, |over| over.min(point)));
        }
        if stopped_by_another_verdict {
            break;
        }
        if lowest_over_the_record_limit.is_none() {
            step = step.saturating_mul(1u64 << doublings);
        }
    }
    if let Some(limit) = record_limit {
        if lowest_admitted > limit {
            stop_with(
                "admitted_above_the_record_limit",
                &format!("admitted_units={lowest_admitted} record_limit={limit}"),
            );
        }
    }
    lowest_admitted
}

/// 截尾状态上收不收紧下界（登记第十二节修订 4）。判定只看「臂要的单元数 ≤ 下界」：收紧只把下界从当前值抬到不超过记录上限的某个值，
/// 所以只有要的单元数落在（当前下界, 记录上限] 里的臂，判定才可能从「未定」变成「一定不多报」；别的臂收不收紧判定都一样。
#[derive(Clone, Debug, PartialEq, Eq)]
enum LowerBoundTightening {
    /// 每个截尾状态都收紧（单测拿它当对照）。
    #[cfg_attr(
        not(test),
        expect(dead_code, reason = "一律收紧只在单测里当对照，产物那一趟只走按需收紧")
    )]
    AtEveryCensoredState,
    /// 只在某条臂要的单元数落在（当前下界, 记录上限] 里时收紧；`units_asked_for_by_the_arms` 是这个状态上每条臂的 ⌈X ÷ 32634⌉。
    OnlyWhereAnArmJudgementCanChange {
        units_asked_for_by_the_arms: Vec<u64>,
    },
}

/// 收紧能不能改某条臂的判定：要的单元数落在（当前下界, 记录上限] 里。记录上限还没学到时按「能」算（照收）。
fn tightening_can_change_an_arm_judgement(
    current_lower_bound_units: u64,
    record_limit: Option<u64>,
    units_asked_for_by_the_arms: &[u64],
) -> bool {
    units_asked_for_by_the_arms.iter().any(|asked| {
        *asked > current_lower_bound_units && record_limit.is_none_or(|limit| *asked <= limit)
    })
}

/// 记录上限处放行之后的截尾结果：按 `tightening` 收不收紧下界。
fn censored_at_the_journal_cap<Prober: UnitsProbe>(
    prober: &Prober,
    model: &JournalModel,
    admitted_units: u64,
    tightening: &LowerBoundTightening,
) -> WritableUnits {
    let tightens = match tightening {
        LowerBoundTightening::AtEveryCensoredState => true,
        LowerBoundTightening::OnlyWhereAnArmJudgementCanChange {
            units_asked_for_by_the_arms,
        } => tightening_can_change_an_arm_judgement(
            admitted_units,
            model.record_limit,
            units_asked_for_by_the_arms,
        ),
    };
    let lower_bound_units = if tightens {
        tighten_the_lower_bound_to_the_record_limit(prober, model.record_limit, admitted_units)
    } else {
        admitted_units
    };
    WritableUnits::AtLeastBecauseOfTheJournalCap {
        lower_bound_units,
        lower_bound_was_tightened: tightens,
    }
}

/// 找这个状态上写得进的最大单元数 T（探测的搜索：K 路并行的区间收缩，K = 探测线程数）。截尾的状态上下界收不收紧照 `tightening`。
fn find_writable_units<Prober: UnitsProbe>(
    prober: &Prober,
    space_bound: u64,
    model: &mut JournalModel,
    tightening: &LowerBoundTightening,
) -> WritableUnits {
    if model.largest_units_below_the_record_limit.is_none()
        && space_bound > UNITS_ABOVE_WHICH_THE_JOURNAL_CAP_IS_LEARNED
    {
        let learning_units = space_bound.min(PROBE_FAR_ABOVE_ANY_JOURNAL_CAP_UNITS);
        if let ProbeVerdict::JournalLimit {
            records_of_the_publish,
            record_limit,
        } = prober.probe_one(learning_units)
        {
            model.learn_from(learning_units, records_of_the_publish, record_limit);
        }
    }
    let journal_cap_before_the_first_probe = model.largest_units_below_the_record_limit;
    let seed = space_bound + 1;
    let first_units = match journal_cap_before_the_first_probe {
        Some(cap) if seed > cap => cap,
        Some(_) | None => seed,
    };
    let (first_units, first_verdict) = probe_at_the_journal_cap(prober, model, first_units);
    let journal_cap_after_the_first_probe = model.largest_units_below_the_record_limit;
    let (mut lowest_admitted, mut lowest_refused) = match first_verdict {
        ProbeVerdict::RefusedForSpace => (0u64, first_units),
        ProbeVerdict::Admitted => match journal_cap_after_the_first_probe {
            Some(cap) if first_units >= cap => {
                return censored_at_the_journal_cap(prober, model, first_units, tightening);
            }
            Some(cap) => {
                let (cap_units, cap_verdict) = probe_at_the_journal_cap(prober, model, cap);
                match cap_verdict {
                    ProbeVerdict::Admitted => {
                        return censored_at_the_journal_cap(prober, model, cap_units, tightening);
                    }
                    ProbeVerdict::RefusedForSpace => (first_units, cap_units),
                    other @ (ProbeVerdict::Completed
                    | ProbeVerdict::JournalLimit { .. }
                    | ProbeVerdict::Other(_)) => return unreadable_because(&other, cap_units),
                }
            }
            None => return unreadable_because(&ProbeVerdict::Admitted, first_units),
        },
        other @ (ProbeVerdict::Completed
        | ProbeVerdict::JournalLimit { .. }
        | ProbeVerdict::Other(_)) => return unreadable_because(&other, first_units),
    };
    let mut non_monotone_probes_seen = false;
    while lowest_refused - lowest_admitted > 1 {
        let points = interior_points(
            lowest_admitted,
            lowest_refused,
            prober.parallel_probe_count(),
        );
        assert!(
            !points.is_empty(),
            "区间 ({lowest_admitted}, {lowest_refused}) 里取不出内部点"
        );
        let verdicts = prober.probe_in_parallel(&points);
        let mut admitted_points = Vec::new();
        let mut refused_points = Vec::new();
        for (point, verdict) in points.iter().zip(&verdicts) {
            match verdict {
                ProbeVerdict::Admitted => admitted_points.push(*point),
                ProbeVerdict::RefusedForSpace => refused_points.push(*point),
                other @ (ProbeVerdict::Completed
                | ProbeVerdict::JournalLimit { .. }
                | ProbeVerdict::Other(_)) => return unreadable_because(other, *point),
            }
        }
        if let Some(smallest_refused) = refused_points.iter().copied().min() {
            lowest_refused = lowest_refused.min(smallest_refused);
        }
        let admitted_below_the_refused = admitted_points
            .iter()
            .copied()
            .filter(|point| *point < lowest_refused)
            .max();
        if admitted_points.iter().any(|point| *point > lowest_refused) {
            non_monotone_probes_seen = true;
        }
        if let Some(point) = admitted_below_the_refused {
            lowest_admitted = lowest_admitted.max(point);
        }
    }
    WritableUnits::Exact {
        units: lowest_admitted,
        non_monotone_probes_seen,
    }
}

// ---------------------------------------------------------------------------------------------------------------------
// 判定：多报、少报、截尾（登记第一节「多报」「少报」，第六节 Q1–Q4）
// ---------------------------------------------------------------------------------------------------------------------

#[derive(Clone, Debug, PartialEq, Eq)]
enum Judgement {
    /// 照这个字节数写会被拒：多报 `over_bytes` = X − T。
    OverReports { over_bytes: u64 },
    /// 不多报；`under_bytes` = T − X。
    DoesNotOverReport { under_bytes: u64 },
    /// T 只知道下界，X 不超过下界所以一定不多报。
    CertainlyDoesNotOverReportBelowTheLowerBound,
    /// T 只知道下界或取不出，判不了。
    Undetermined,
}

impl Judgement {
    fn label(&self) -> &'static str {
        match self {
            Judgement::OverReports { .. } => "over",
            Judgement::DoesNotOverReport { .. } => "not_over",
            Judgement::CertainlyDoesNotOverReportBelowTheLowerBound => "not_over_below_lower_bound",
            Judgement::Undetermined => "undetermined",
        }
    }
}

/// 报 `reported` 字节的臂要的单元数 ⌈X ÷ 32634⌉（登记第一节）：判定与按需收紧共用这一处。
fn units_asked_for_by(reported: u64) -> u64 {
    reported.div_ceil(DATA_UNIT_PAYLOAD_BYTES)
}

impl StateReading {
    /// 这个状态上每条臂（[`all_arms`] 的次序）要的单元数：按需收紧看它们落没落在（当前下界, 记录上限] 里。
    fn units_asked_for_by_every_arm(&self) -> Vec<u64> {
        all_arms()
            .into_iter()
            .map(|arm| units_asked_for_by(reported_bytes(self.raw_bytes_of(arm))))
            .collect()
    }
}

/// 候选在 `reported` 字节上对着 T 的判定：⌈X ÷ 32634⌉ > n 即多报（登记第一节）。
fn judge(reported: u64, writable: &WritableUnits) -> Judgement {
    let units_asked_for = units_asked_for_by(reported);
    match writable {
        WritableUnits::Exact { units, .. } => {
            let writable_bytes = units * DATA_UNIT_PAYLOAD_BYTES;
            if units_asked_for > *units {
                Judgement::OverReports {
                    over_bytes: reported - writable_bytes,
                }
            } else {
                Judgement::DoesNotOverReport {
                    under_bytes: writable_bytes - reported.min(writable_bytes),
                }
            }
        }
        WritableUnits::AtLeastBecauseOfTheJournalCap {
            lower_bound_units, ..
        } => {
            if units_asked_for <= *lower_bound_units {
                Judgement::CertainlyDoesNotOverReportBelowTheLowerBound
            } else {
                Judgement::Undetermined
            }
        }
        WritableUnits::Unreadable(_) => Judgement::Undetermined,
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
struct ArmTally {
    over_states: u64,
    not_over_states: u64,
    not_over_below_lower_bound_states: u64,
    undetermined_states: u64,
    max_over_bytes: u64,
    max_over_at: String,
    max_under_bytes: u64,
    max_under_at: String,
    sum_under_bytes: u128,
    under_states: u64,
}

impl ArmTally {
    fn count(&mut self, judgement: &Judgement, state_name: &str) {
        match judgement {
            Judgement::OverReports { over_bytes } => {
                self.over_states += 1;
                if *over_bytes > self.max_over_bytes {
                    self.max_over_bytes = *over_bytes;
                    self.max_over_at = state_name.to_string();
                }
            }
            Judgement::DoesNotOverReport { under_bytes } => {
                self.not_over_states += 1;
                if *under_bytes > 0 {
                    self.under_states += 1;
                    self.sum_under_bytes += u128::from(*under_bytes);
                }
                if *under_bytes > self.max_under_bytes {
                    self.max_under_bytes = *under_bytes;
                    self.max_under_at = state_name.to_string();
                }
            }
            Judgement::CertainlyDoesNotOverReportBelowTheLowerBound => {
                self.not_over_below_lower_bound_states += 1;
            }
            Judgement::Undetermined => self.undetermined_states += 1,
        }
    }

    fn merge(&mut self, other: &ArmTally) {
        self.over_states += other.over_states;
        self.not_over_states += other.not_over_states;
        self.not_over_below_lower_bound_states += other.not_over_below_lower_bound_states;
        self.undetermined_states += other.undetermined_states;
        if other.max_over_bytes > self.max_over_bytes {
            self.max_over_bytes = other.max_over_bytes;
            self.max_over_at.clone_from(&other.max_over_at);
        }
        if other.max_under_bytes > self.max_under_bytes {
            self.max_under_bytes = other.max_under_bytes;
            self.max_under_at.clone_from(&other.max_under_at);
        }
        self.sum_under_bytes += other.sum_under_bytes;
        self.under_states += other.under_states;
    }
}

/// 一对「残差」的计数（登记 5.2、第六节 Q5–Q7），一块盘一份。
#[derive(Clone, Debug, Default, PartialEq, Eq)]
struct ResidualTally {
    states: u64,
    rewritten_check_residual_nonzero_states: u64,
    original_formula_residual_nonzero_states: u64,
    maximum_absolute_rewritten_check_residual: i128,
    maximum_absolute_original_formula_residual: i128,
    states_with_overlap: u64,
    states_with_held_ledger: u64,
    /// 阳性对照：改写检查残差去掉重叠项之后在重叠 > 0 的状态上必须 ≠ 0；这里数它没 ≠ 0 的状态。
    control_without_overlap_term_stayed_zero: u64,
    first_state_with_a_nonzero_rewritten_check_residual: String,
    first_state_with_a_nonzero_original_formula_residual: String,
}

impl ResidualTally {
    fn merge(&mut self, other: &ResidualTally) {
        self.states += other.states;
        self.rewritten_check_residual_nonzero_states +=
            other.rewritten_check_residual_nonzero_states;
        self.original_formula_residual_nonzero_states +=
            other.original_formula_residual_nonzero_states;
        self.maximum_absolute_rewritten_check_residual = self
            .maximum_absolute_rewritten_check_residual
            .max(other.maximum_absolute_rewritten_check_residual);
        self.maximum_absolute_original_formula_residual = self
            .maximum_absolute_original_formula_residual
            .max(other.maximum_absolute_original_formula_residual);
        self.states_with_overlap += other.states_with_overlap;
        self.states_with_held_ledger += other.states_with_held_ledger;
        self.control_without_overlap_term_stayed_zero +=
            other.control_without_overlap_term_stayed_zero;
        if self
            .first_state_with_a_nonzero_rewritten_check_residual
            .is_empty()
        {
            self.first_state_with_a_nonzero_rewritten_check_residual
                .clone_from(&other.first_state_with_a_nonzero_rewritten_check_residual);
        }
        if self
            .first_state_with_a_nonzero_original_formula_residual
            .is_empty()
        {
            self.first_state_with_a_nonzero_original_formula_residual
                .clone_from(&other.first_state_with_a_nonzero_original_formula_residual);
        }
    }
}

/// 被条款当谓词输入的量的轨迹（登记第八节）：初始供给（第一个状态）、之后的峰值、为正的状态数、期末值。
#[derive(Clone, Debug, Default, PartialEq, Eq)]
struct TrajectoryTally {
    states: u64,
    first_value: i128,
    peak_after_the_first: Option<i128>,
    positive_states_after_the_first: u64,
    last_value: i128,
}

impl TrajectoryTally {
    fn record(&mut self, value: i128) {
        if self.states == 0 {
            self.first_value = value;
        } else {
            self.peak_after_the_first = Some(
                self.peak_after_the_first
                    .map_or(value, |peak| peak.max(value)),
            );
            if value > 0 {
                self.positive_states_after_the_first += 1;
            }
        }
        self.last_value = value;
        self.states += 1;
    }
}

// ---------------------------------------------------------------------------------------------------------------------
// 一族历史上的累计与产物行
// ---------------------------------------------------------------------------------------------------------------------

struct ProductLine {
    text: String,
}

impl ProductLine {
    fn new(name: &str) -> Self {
        ProductLine {
            text: format!("E name={name}"),
        }
    }

    fn field(mut self, key: &str, value: impl std::fmt::Display) -> Self {
        self.text.push(' ');
        self.text.push_str(key);
        self.text.push('=');
        self.text.push_str(&value.to_string());
        self
    }

    fn finish(self) -> String {
        self.text
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
struct FamilyTally {
    states: u64,
    exact_states: u64,
    censored_states: u64,
    unreadable_states: u64,
    arms: BTreeMap<Arm, ArmTally>,
    residuals: Vec<ResidualTally>,
    closed_minus_ring_window: Vec<TrajectoryTally>,
    closed_alone: Vec<TrajectoryTally>,
    closed_minus_all_common_commitments: Vec<TrajectoryTally>,
    control_failures: Vec<String>,
    shortcut_pairs_checked: u64,
    shortcut_differences: Vec<String>,
    records_disagree_with_the_disk_states: u64,
    decomposition_failure_states: u64,
    not_whole_slot_available_states: u64,
    lag_unattributed_states: u64,
    non_monotone_states: u64,
    probes_run: u64,
    stop_clause_held_ledger_mismatch_states: u64,
    unmount_pushes_checked: u64,
    unmount_pushes_differing: Vec<String>,
    panics: Vec<String>,
}

impl FamilyTally {
    fn new(device_count: usize) -> Self {
        FamilyTally {
            residuals: vec![ResidualTally::default(); device_count],
            closed_minus_ring_window: vec![TrajectoryTally::default(); device_count],
            closed_alone: vec![TrajectoryTally::default(); device_count],
            closed_minus_all_common_commitments: vec![TrajectoryTally::default(); device_count],
            ..FamilyTally::default()
        }
    }

    /// 同一几何格里几族的累计并起来（轨迹不并：轨迹按族报）。
    fn merge_counts_of(&mut self, other: &FamilyTally) {
        self.states += other.states;
        self.exact_states += other.exact_states;
        self.censored_states += other.censored_states;
        self.unreadable_states += other.unreadable_states;
        for (arm, tally) in &other.arms {
            self.arms.entry(*arm).or_default().merge(tally);
        }
        if self.residuals.len() < other.residuals.len() {
            self.residuals = vec![ResidualTally::default(); other.residuals.len()];
        }
        for (mine, theirs) in self.residuals.iter_mut().zip(&other.residuals) {
            mine.merge(theirs);
        }
        self.control_failures
            .extend(other.control_failures.iter().cloned());
        self.shortcut_pairs_checked += other.shortcut_pairs_checked;
        self.shortcut_differences
            .extend(other.shortcut_differences.iter().cloned());
        self.records_disagree_with_the_disk_states += other.records_disagree_with_the_disk_states;
        self.decomposition_failure_states += other.decomposition_failure_states;
        self.not_whole_slot_available_states += other.not_whole_slot_available_states;
        self.lag_unattributed_states += other.lag_unattributed_states;
        self.non_monotone_states += other.non_monotone_states;
        self.probes_run += other.probes_run;
        self.stop_clause_held_ledger_mismatch_states +=
            other.stop_clause_held_ledger_mismatch_states;
        self.unmount_pushes_checked += other.unmount_pushes_checked;
        self.unmount_pushes_differing
            .extend(other.unmount_pushes_differing.iter().cloned());
        self.panics.extend(other.panics.iter().cloned());
    }
}

/// 一族历史的运行态：池、产物行缓冲、累计。
struct HistoryRun {
    pool: Pool,
    family: &'static str,
    with_probes: bool,
    /// 只对填满族做的「捷径与整写同判」（登记 5.1 探测捷径）。
    checks_the_shortcut_against_full_writes: bool,
    probe_threads: usize,
    model: JournalModel,
    state_index: u64,
    /// 每隔几个状态打一行 `state`（长挂载族两千个状态只抽着打，累计照样逐状态数）。
    emit_state_line_every: u64,
    lines: LineBuffer,
    tally: FamilyTally,
}

impl HistoryRun {
    fn new(pool: Pool, family: &'static str, with_probes: bool, probe_threads: usize) -> Self {
        let device_count = pool.devices.len();
        HistoryRun {
            pool,
            family,
            with_probes,
            checks_the_shortcut_against_full_writes: family == "fill",
            probe_threads,
            model: JournalModel::NOT_LEARNED_YET,
            state_index: 0,
            emit_state_line_every: 1,
            lines: LineBuffer::default(),
            tally: FamilyTally::new(device_count),
        }
    }

    fn state_name(&self, index: u64) -> String {
        format!("{}/{}/{index}", self.pool.geometry.name, self.family)
    }

    fn measure_writable_units(&mut self, reading: &StateReading) -> WritableUnits {
        let stripped = strip_bytes_the_probe_does_not_need(&mut self.pool.session);
        let (writable, probes_run) = {
            let prober = StateProber {
                pool: &self.pool,
                probe_threads: self.probe_threads,
                probes_run: AtomicU64::new(0),
            };
            let writable = find_writable_units(
                &prober,
                upper_bound_units_by_the_space_conditions(&self.pool, reading),
                &mut self.model,
                &LowerBoundTightening::OnlyWhereAnArmJudgementCanChange {
                    units_asked_for_by_the_arms: reading.units_asked_for_by_every_arm(),
                },
            );
            (writable, prober.probes_run.load(Ordering::Relaxed))
        };
        restore_stripped_bytes(&mut self.pool.session, stripped);
        self.tally.probes_run += probes_run;
        writable
    }

    /// 捷径与整写同判：T 与 T + 1 各整写一遍（不中途报错、带字节的整份拷贝），判法逐项相同才算。
    fn check_the_shortcut_against_full_writes(&mut self, units: u64, state_name: &str) {
        for tried in [units, units + 1] {
            if tried == 0 {
                continue;
            }
            let expected_by_the_shortcut = if tried == units {
                ProbeVerdict::Completed
            } else {
                ProbeVerdict::RefusedForSpace
            };
            let full = run_probe(
                &self.pool.devices,
                clone_session(&self.pool.session),
                tried,
                DataWritePolicy::AllowDataWrites,
            );
            self.tally.shortcut_pairs_checked += 1;
            if full != expected_by_the_shortcut {
                self.tally.shortcut_differences.push(format!(
                    "{state_name}:units={tried}:shortcut_said={}:full_write_said={}",
                    if tried == units {
                        "admitted"
                    } else {
                        "refused_for_space"
                    },
                    short_debug(&full)
                ));
            }
        }
    }
}

impl HistoryRun {
    /// 在当前状态上量一遍：读数、残差、段内关着的空槽、（要探测的族）T 与全部臂的判定；打一行 `state`，累计进族的计数。
    fn observe(&mut self, kind: &str, outcome: &str) {
        let index = self.state_index;
        self.state_index += 1;
        let state_name = self.state_name(index);
        let reading = read_state(&self.pool);
        self.tally.states += 1;
        if !reading.records_agree_with_the_disk {
            self.tally.records_disagree_with_the_disk_states += 1;
        }
        if !reading.decomposition_holds {
            self.tally.decomposition_failure_states += 1;
        }
        if !reading.available_is_whole_slots {
            self.tally.not_whole_slot_available_states += 1;
        }
        if reading.lag_unattributed_slots > 0 {
            self.tally.lag_unattributed_states += 1;
        }
        let mut line = ProductLine::new("state")
            .field("geometry", self.pool.geometry.name)
            .field("family", self.family)
            .field("index", index)
            .field("kind", kind)
            .field("outcome", outcome)
            .field("data_units", reading.data_units)
            .field("checkpoint_cost_blocks", reading.checkpoint_cost_blocks)
            .field("lag_slots", reading.lag_slots)
            .field("lag_unattributed_slots", reading.lag_unattributed_slots)
            .field("oldest_valid_root_txg", reading.oldest_valid_root_txg)
            .field(
                "fourth_newest_nonempty_valid_root_txg",
                reading
                    .fourth_newest_nonempty_valid_root_txg
                    .map_or("none".to_string(), |txg| txg.to_string()),
            )
            .field("cluster_segments", reading.cluster_segment_count)
            .field(
                "records_agree_with_the_disk",
                reading.records_agree_with_the_disk,
            )
            .field("decomposition_holds", reading.decomposition_holds)
            .field(
                "abandoned_root_ring_slots",
                self.pool
                    .session
                    .allocator
                    .root_ring_occupancy()
                    .map_or(0, |occupancy| {
                        occupancy
                            .ring_slots_recorded_as_holding_an_abandoned_root()
                            .len()
                    }),
            );
        line = self.record_residuals_and_trajectories(line, &reading, &state_name);
        if self.with_probes {
            let writable = self.measure_writable_units(&reading);
            line = self.record_arms(line, &reading, &writable, &state_name);
        }
        if index.is_multiple_of(self.emit_state_line_every) {
            self.lines.push(line.finish());
        }
    }

    fn record_residuals_and_trajectories(
        &mut self,
        mut line: ProductLine,
        reading: &StateReading,
        state_name: &str,
    ) -> ProductLine {
        for (device_index, device) in reading.devices.iter().enumerate() {
            let available_plus_common = device.available_plus_common_slots();
            let unblocked = i128::from(device.unblocked_slots);
            let overlap = i128::from(device.overlap_slots_allocated_and_isolated);
            // 装置自记的扣住账（登记第二节「扣住位」）：第一段的历史里抬 F 那一串都整串发成、整串放开，没有留着的扣住位，恒 0。
            let held_ledger = 0i128;
            let residuals = residuals_of(available_plus_common, unblocked, overlap, held_ledger);
            let residual_with_both_terms =
                residuals.rewritten_check_residual_with_the_held_and_overlap_terms;
            let residual_without_the_overlap_term =
                residuals.rewritten_check_residual_without_the_overlap_term;
            let original_check_residual = residuals.original_formula_residual;
            let residual_tally = &mut self.tally.residuals[device_index];
            residual_tally.states += 1;
            if residual_with_both_terms != 0 {
                residual_tally.rewritten_check_residual_nonzero_states += 1;
                self.tally.stop_clause_held_ledger_mismatch_states += 1;
                if residual_tally
                    .first_state_with_a_nonzero_rewritten_check_residual
                    .is_empty()
                {
                    residual_tally.first_state_with_a_nonzero_rewritten_check_residual =
                        state_name.to_string();
                }
                residual_tally.maximum_absolute_rewritten_check_residual = residual_tally
                    .maximum_absolute_rewritten_check_residual
                    .max(residual_with_both_terms.abs());
            }
            if original_check_residual != 0 {
                residual_tally.original_formula_residual_nonzero_states += 1;
                if residual_tally
                    .first_state_with_a_nonzero_original_formula_residual
                    .is_empty()
                {
                    residual_tally.first_state_with_a_nonzero_original_formula_residual =
                        state_name.to_string();
                }
                residual_tally.maximum_absolute_original_formula_residual = residual_tally
                    .maximum_absolute_original_formula_residual
                    .max(original_check_residual.abs());
            }
            if overlap > 0 {
                residual_tally.states_with_overlap += 1;
                if residual_without_the_overlap_term == 0 {
                    residual_tally.control_without_overlap_term_stayed_zero += 1;
                }
            }
            let closed = i128::from(device.closed_free_slots_in_cluster_segments);
            let ring_window = i128::from(device.ring_window_slots);
            let all_common_slots = device
                .common_commitments_bytes
                .div_euclid(i128::from(SLOT_BYTES));
            self.tally.closed_minus_ring_window[device_index].record(closed - ring_window);
            self.tally.closed_alone[device_index].record(closed);
            self.tally.closed_minus_all_common_commitments[device_index]
                .record(closed - all_common_slots);
            let key = |name: &str| format!("{name}_{device_index}");
            line = line
                .field(&key("capacity_slots"), device.capacity_slots)
                .field(&key("live_slots"), device.live_slots)
                .field(
                    &key("live_slots_in_memory_account"),
                    device.live_slots_in_memory_account,
                )
                .field(
                    &key("available_slots"),
                    device.available_slots_rounded_down(),
                )
                .field(&key("allocated_slots"), device.allocated_slots)
                .field(&key("deferred_slots"), device.deferred_slots)
                .field(&key("isolated_slots"), device.isolated_slots)
                .field(&key("overlap_allocated_and_isolated_slots"), overlap)
                .field(&key("unblocked_slots"), unblocked)
                .field(&key("closed_free_slots_in_cluster_segments"), closed)
                .field(&key("ring_window_slots"), ring_window)
                .field(&key("common_commitments_slots"), all_common_slots)
                .field(&key("rewritten_check_residual"), residual_with_both_terms)
                .field(&key("original_formula_residual"), original_check_residual)
                .field(
                    &key("rewritten_check_residual_without_the_overlap_term"),
                    residual_without_the_overlap_term,
                );
        }
        line.field("held_ledger_slots", 0)
    }
}

impl HistoryRun {
    fn record_arms(
        &mut self,
        mut line: ProductLine,
        reading: &StateReading,
        writable: &WritableUnits,
        state_name: &str,
    ) -> ProductLine {
        match writable {
            WritableUnits::Exact {
                units,
                non_monotone_probes_seen,
            } => {
                self.tally.exact_states += 1;
                if *non_monotone_probes_seen {
                    self.tally.non_monotone_states += 1;
                }
                line = line
                    .field("writable_status", "exact")
                    .field("writable_units", units)
                    .field("writable_bytes", units * DATA_UNIT_PAYLOAD_BYTES)
                    .field("non_monotone_probes_seen", non_monotone_probes_seen);
            }
            WritableUnits::AtLeastBecauseOfTheJournalCap {
                lower_bound_units,
                lower_bound_was_tightened,
            } => {
                self.tally.censored_states += 1;
                line = line
                    .field("writable_status", "censored_by_the_journal_record_cap")
                    .field("writable_lower_bound_units", lower_bound_units)
                    .field(
                        "writable_lower_bound_bytes",
                        lower_bound_units * DATA_UNIT_PAYLOAD_BYTES,
                    )
                    .field("writable_lower_bound_tightened", lower_bound_was_tightened);
            }
            WritableUnits::Unreadable(reason) => {
                self.tally.unreadable_states += 1;
                line = line
                    .field("writable_status", "unreadable")
                    .field("reason", reason);
            }
        }
        for arm in all_arms() {
            let raw = reading.raw_bytes_of(arm);
            let reported = reported_bytes(raw);
            let judgement = judge(reported, writable);
            let tally = self.tally.arms.entry(arm).or_default();
            tally.count(&judgement, state_name);
            let (over_bytes, under_bytes) = match &judgement {
                Judgement::OverReports { over_bytes } => (*over_bytes, 0),
                Judgement::DoesNotOverReport { under_bytes } => (0, *under_bytes),
                Judgement::CertainlyDoesNotOverReportBelowTheLowerBound
                | Judgement::Undetermined => (0, 0),
            };
            let label = arm.label();
            line = line
                .field(&format!("reported_bytes_{label}"), reported)
                .field(&format!("judgement_{label}"), judgement.label())
                .field(&format!("over_bytes_{label}"), over_bytes)
                .field(&format!("under_bytes_{label}"), under_bytes);
            if let WritableUnits::Exact { units, .. } = writable {
                self.check_the_controls_of_one_arm(arm, reported, *units, state_name);
            }
        }
        if let WritableUnits::Exact { units, .. } = writable {
            if self.checks_the_shortcut_against_full_writes {
                self.check_the_shortcut_against_full_writes(*units, state_name);
            }
        }
        line
    }

    /// 阳性对照（登记 5.1）：每条臂的「这条臂的值再顶到 T 之上一个载荷」必须判多报；恒多报的阳性对照在 T > 0 的状态上判多报且大于 2T（登记第七节 B4）；
    /// 恒报 0 的阴性对照从不判多报。只在 T 取得准的状态上跑。
    fn check_the_controls_of_one_arm(
        &mut self,
        arm: Arm,
        reported: u64,
        writable_units: u64,
        state_name: &str,
    ) {
        let writable = WritableUnits::Exact {
            units: writable_units,
            non_monotone_probes_seen: false,
        };
        let writable_bytes = writable_units * DATA_UNIT_PAYLOAD_BYTES;
        let pushed_above_the_writable_bytes =
            reported.max(writable_bytes) + DATA_UNIT_PAYLOAD_BYTES;
        if !matches!(
            judge(pushed_above_the_writable_bytes, &writable),
            Judgement::OverReports { .. }
        ) {
            self.tally.control_failures.push(format!(
                "{state_name}:arm={}:pushed_above_the_writable_bytes_not_judged_over",
                arm.label()
            ));
        }
        match arm {
            Arm::PositiveControlAlwaysOverReports if writable_units > 0 => {
                if !matches!(judge(reported, &writable), Judgement::OverReports { .. }) {
                    self.tally.control_failures.push(format!(
                        "{state_name}:always_over_reports_control_not_judged_over"
                    ));
                }
                if reported <= 2 * writable_bytes {
                    self.tally.control_failures.push(format!(
                        "{state_name}:always_over_reports_control_not_above_twice_the_writable_bytes:reported={reported}:writable_bytes={writable_bytes}"
                    ));
                }
            }
            Arm::NegativeControlReportsZero => {
                if matches!(judge(0, &writable), Judgement::OverReports { .. }) {
                    self.tally
                        .control_failures
                        .push(format!("{state_name}:zero_control_judged_over"));
                }
            }
            Arm::Candidate(_) | Arm::PositiveControlAlwaysOverReports => {}
        }
    }
}

// ---------------------------------------------------------------------------------------------------------------------
// 历史族（登记 5 的「历史族」表）：每族一条确定的脚本，种子写死
// ---------------------------------------------------------------------------------------------------------------------

/// 登记里覆盖写族、多次挂载族的循环大小。
const OVERWRITE_FAMILY_SEQUENTIAL_SIZES: [u64; 6] = [64, 512, 2048, 128, 1024, 64];
/// 登记里长挂载族的循环大小。
const LONG_MOUNT_SEQUENTIAL_SIZES: [u64; 5] = [1, 8, 64, 512, 4096];
const FILL_FAMILY_STEPS_UP_TO_THE_FULL_RANGE: u64 = 20;
/// 一条爬升梯最多爬几档（让循环有上界）。
const LADDER_CLIMBS_AT_MOST: u64 = 200;

/// M = 这个几何上段外槽对数的一半（登记 5：填满族）：在第一个状态上按各盘「用户数据落得下的槽对数」取小再除以 2。
fn half_of_the_pairs_outside_the_cluster_segments(pool: &Pool) -> u64 {
    pool.session
        .allocator
        .devices
        .iter()
        .map(|map| {
            map.unblocked_slot_pairs_outside_the_cluster_segments(
                pool.session.allocator.cluster_segments(),
            )
        })
        .min()
        .expect("至少一块盘")
        / 2
}

fn ladder_step_units(half_pairs: u64) -> u64 {
    half_pairs.div_ceil(FILL_FAMILY_STEPS_UP_TO_THE_FULL_RANGE)
}

fn change_for_the_overwrite_cycle(change_number: u64) -> ChangeKind {
    if change_number.is_multiple_of(20) {
        let cycle_index = usize::try_from(change_number / 20 - 1).expect("循环下标")
            % OVERWRITE_FAMILY_SEQUENTIAL_SIZES.len();
        ChangeKind::SequentialWriteOfUnits(OVERWRITE_FAMILY_SEQUENTIAL_SIZES[cycle_index])
    } else {
        ChangeKind::OverwriteWithOneUnit
    }
}

fn change_and_observe(run: &mut HistoryRun, kind: ChangeKind) -> ChangeOutcome {
    let outcome = apply_change(&mut run.pool, kind);
    run.observe("after_change", &format!("{kind:?}:{}", outcome.label()));
    outcome
}

/// 填满族：1 单元起，按 ⌈M ÷ 20⌉ 单元一档往上换写到被拒，换写回 1 单元，再往上换写到被拒。
fn drive_fill_family(run: &mut HistoryRun) {
    run.observe("mounted", "after_first_file_and_remount");
    let half_pairs = half_of_the_pairs_outside_the_cluster_segments(&run.pool);
    let step_units = ladder_step_units(half_pairs);
    run.lines.push(
        ProductLine::new("family_parameters")
            .field("geometry", run.pool.geometry.name)
            .field("family", run.family)
            .field("half_of_the_pairs_outside_the_cluster_segments", half_pairs)
            .field("ladder_step_units", step_units)
            .finish(),
    );
    for round in 0..2 {
        if round == 1 {
            change_and_observe(run, ChangeKind::SequentialWriteOfUnits(1));
        }
        for climb in 1..=LADDER_CLIMBS_AT_MOST {
            let units = (climb * step_units).min(PROBE_FAR_ABOVE_ANY_JOURNAL_CAP_UNITS);
            let outcome = change_and_observe(run, ChangeKind::SequentialWriteOfUnits(units));
            if !matches!(outcome, ChangeOutcome::Published { .. }) {
                break;
            }
        }
    }
}

/// 覆盖写族：第一个文件 64 单元；120 次改动，每 20 次里 19 次覆盖写、1 次顺序写。
fn drive_overwrite_family(run: &mut HistoryRun) {
    run.observe("mounted", "after_first_file_and_remount");
    change_and_observe(run, ChangeKind::SequentialWriteOfUnits(64));
    for change_number in 1..=120 {
        change_and_observe(run, change_for_the_overwrite_cycle(change_number));
    }
}

/// 删了再写族：换写到 M 的八成，换写回 1 单元，再 3 次覆盖写，之后每次改动都试换写回八成（被拒也是一个状态）。
fn drive_delete_then_write_family(run: &mut HistoryRun) {
    run.observe("mounted", "after_first_file_and_remount");
    let half_pairs = half_of_the_pairs_outside_the_cluster_segments(&run.pool);
    let eighty_percent_units = half_pairs * 8 / 10;
    run.lines.push(
        ProductLine::new("family_parameters")
            .field("geometry", run.pool.geometry.name)
            .field("family", run.family)
            .field("half_of_the_pairs_outside_the_cluster_segments", half_pairs)
            .field("eighty_percent_units", eighty_percent_units)
            .finish(),
    );
    change_and_observe(
        run,
        ChangeKind::SequentialWriteOfUnits(eighty_percent_units),
    );
    change_and_observe(run, ChangeKind::SequentialWriteOfUnits(1));
    for _ in 0..3 {
        change_and_observe(run, ChangeKind::OverwriteWithOneUnit);
    }
    for _ in 0..25 {
        change_and_observe(
            run,
            ChangeKind::SequentialWriteOfUnits(eighty_percent_units),
        );
    }
}

/// 多次挂载族：三轮「可写挂载 → 10 次改动（覆盖写族的循环接着数）→ 正常卸载」，取每次改动之后与每次挂载做成之后的状态。
fn drive_many_mounts_family(run: &mut HistoryRun) {
    let mut change_number = 0u64;
    for round in 0..3 {
        if round > 0 {
            let observed = unmount_and_mount_again(&mut run.pool);
            let expected_pushes = if observed.current_txg_before_the_unmount % 3 == 1 {
                3
            } else {
                2
            };
            run.tally.unmount_pushes_checked += 1;
            if observed.empty_publishes_pushed != expected_pushes {
                run.tally.unmount_pushes_differing.push(format!(
                    "{}:txg_before={}:pushed={}:expected={expected_pushes}",
                    run.pool.geometry.name,
                    observed.current_txg_before_the_unmount,
                    observed.empty_publishes_pushed
                ));
            }
        }
        run.observe("mounted", &format!("round_{round}"));
        for _ in 0..10 {
            change_number += 1;
            change_and_observe(run, change_for_the_overwrite_cycle(change_number));
        }
    }
}

/// 长挂载族：一次挂载里 2000 次改动，每 50 次里 49 次覆盖写、1 次顺序写，中间不卸载。
fn drive_long_mount_family(run: &mut HistoryRun) {
    run.observe("mounted", "after_first_file_and_remount");
    for change_number in 1..=2000u64 {
        let kind = if change_number.is_multiple_of(50) {
            let cycle_index = usize::try_from(change_number / 50 - 1).expect("循环下标")
                % LONG_MOUNT_SEQUENTIAL_SIZES.len();
            ChangeKind::SequentialWriteOfUnits(LONG_MOUNT_SEQUENTIAL_SIZES[cycle_index])
        } else {
            ChangeKind::OverwriteWithOneUnit
        };
        change_and_observe(run, kind);
    }
}

/// 崩溃态：挂载甲发 P 之前的盘面，加 P 发布交给设备的写里到「P 的根槽 FUA 写」为止（含它）的那些，两块盘按全局写序号合起来。
/// P 的系统配置轮换在根槽写之后，不在内。
struct CrashStateAfterTheRootSlotWrite {
    devices: Vec<(DeviceIdentity, Disk<'static>)>,
    root_slot_device_index: usize,
    root_slot_byte_range: Range<u64>,
    /// 每块盘上 P 的 journal 记录占的字节区间。
    journal_record_byte_ranges: Vec<Vec<Range<u64>>>,
}

fn crash_state_after_the_root_slot_write(
    before_publish: &[SectorStore],
    device_bytes: &[u64; 2],
    recordings: &[Vec<RecordedWrite>],
) -> Option<CrashStateAfterTheRootSlotWrite> {
    let (root_slot_device_index, root_write) = recordings
        .iter()
        .enumerate()
        .flat_map(|(index, writes)| writes.iter().map(move |write| (index, write)))
        .filter(|(_, write)| write.is_force_unit_access)
        .min_by_key(|(_, write)| write.sequence)?;
    let cutoff = root_write.sequence;
    let ring_start = JOURNAL_RING_START_SLOT * SLOT_BYTES;
    let ring_end = ring_start + DEFAULT_JOURNAL_RING_BYTES;
    let mut devices = Vec::new();
    let mut journal_record_byte_ranges = Vec::new();
    for (index, store) in before_publish.iter().enumerate() {
        let mut disk = Disk::independent_copy_of(store, device_bytes[index]);
        let mut ranges = Vec::new();
        for write in recordings[index]
            .iter()
            .filter(|write| write.sequence <= cutoff)
        {
            disk.own
                .write_sectors(write.byte_offset, &write.bytes, false);
            let length = u64::try_from(write.bytes.len()).expect("写长");
            if write.byte_offset >= ring_start && write.byte_offset + length <= ring_end {
                ranges.push(write.byte_offset..write.byte_offset + length);
            }
        }
        devices.push((DeviceIdentity(u32::try_from(index).expect("盘号")), disk));
        journal_record_byte_ranges.push(ranges);
    }
    let root_slot_byte_range = root_write.byte_offset
        ..root_write.byte_offset + u64::try_from(root_write.bytes.len()).expect("写长");
    Some(CrashStateAfterTheRootSlotWrite {
        devices,
        root_slot_device_index,
        root_slot_byte_range,
        journal_record_byte_ranges,
    })
}

/// 被抛弃根与净释放族：挂载甲（大文件、同大小重写、换写成小文件 P，P 的根槽 FUA 写之后截断），挂载乙（P 的根槽与它的 journal 记录每次读都失败，
/// 可写挂载做成后撤故障，10 次改动，正常卸载），挂载丙（不注故障，20 次改动，含两次换写变小）。登记 5 的「历史族」表 HC 一行。
fn drive_abandoned_root_and_net_release_family(run: &mut HistoryRun) {
    run.observe("mounted", "mount_a");
    change_and_observe(run, ChangeKind::SequentialWriteOfUnits(256));
    // 会话的覆盖写入口只收一个单元的内容、会把文件换成一个单元；要让 P 是净释放，这里的「8 次覆盖写」用同样 256 单元的顺序写（登记里这一族的意图）。
    for _ in 0..8 {
        change_and_observe(run, ChangeKind::SequentialWriteOfUnits(256));
    }
    let before_publish: Vec<SectorStore> = run
        .pool
        .devices
        .iter()
        .map(|(_, disk)| disk.own.clone())
        .collect();
    for (_, disk) in &mut run.pool.devices {
        disk.recording = Some(Vec::new());
    }
    let outcome_of_p = apply_change(&mut run.pool, ChangeKind::SequentialWriteOfUnits(32));
    let recordings: Vec<Vec<RecordedWrite>> = run
        .pool
        .devices
        .iter_mut()
        .map(|(_, disk)| disk.recording.take().unwrap_or_default())
        .collect();
    run.observe(
        "after_change",
        &format!("p_net_release:{}", outcome_of_p.label()),
    );
    let txg_of_p = run
        .pool
        .log
        .user_change_txgs
        .iter()
        .max()
        .copied()
        .unwrap_or(0);
    let Some(crash_state) = crash_state_after_the_root_slot_write(
        &before_publish,
        &run.pool.geometry.device_bytes,
        &recordings,
    ) else {
        run.lines.push(
            ProductLine::new("abandoned_root_and_net_release_family_status")
                .field("status", "no_root_slot_write_recorded")
                .finish(),
        );
        return;
    };
    drop(before_publish);
    let mut devices_of_mount_b = crash_state.devices;
    devices_of_mount_b[crash_state.root_slot_device_index]
        .1
        .unreadable_byte_ranges
        .push(crash_state.root_slot_byte_range.clone());
    for (index, ranges) in crash_state.journal_record_byte_ranges.iter().enumerate() {
        devices_of_mount_b[index]
            .1
            .unreadable_byte_ranges
            .extend(ranges.iter().cloned());
    }
    let mut log_of_mount_b = run.pool.log.clone();
    log_of_mount_b.user_change_txgs.remove(&txg_of_p);
    let mounted = match mount_writable(&run.pool.parameters, &mut devices_of_mount_b) {
        Ok(mounted) => mounted,
        Err(error) => {
            run.lines.push(
                ProductLine::new("abandoned_root_and_net_release_family_status")
                    .field("status", "mount_b_refused")
                    .field("error", short_debug(&error))
                    .finish(),
            );
            return;
        }
    };
    let (output_of_mount_b, session_of_mount_b) = MountedSession::of_the_mount(mounted);
    log_of_mount_b.note_mount(&output_of_mount_b);
    run.pool.devices = devices_of_mount_b;
    run.pool.session = session_of_mount_b;
    run.pool.log = log_of_mount_b;
    for (_, disk) in &mut run.pool.devices {
        disk.unreadable_byte_ranges.clear();
    }
    run.observe("mounted", "mount_b_faults_removed");
    for change_number in 1..=10u64 {
        change_and_observe(run, change_for_the_overwrite_cycle(change_number));
    }
    unmount_and_mount_again(&mut run.pool);
    run.observe("mounted", "mount_c");
    for change_number in 1..=20u64 {
        let kind = match change_number {
            1 => ChangeKind::SequentialWriteOfUnits(128),
            10 => ChangeKind::SequentialWriteOfUnits(64),
            _ => ChangeKind::OverwriteWithOneUnit,
        };
        change_and_observe(run, kind);
    }
    let abandoned_after_mount_c =
        run.pool
            .session
            .allocator
            .root_ring_occupancy()
            .map_or(0, |occupancy| {
                occupancy
                    .ring_slots_recorded_as_holding_an_abandoned_root()
                    .len()
            });
    run.lines.push(
        ProductLine::new("abandoned_root_and_net_release_family_status")
            .field("geometry", run.pool.geometry.name)
            .field("status", "mounts_made")
            .field(
                "abandoned_root_ring_slots_at_the_end",
                abandoned_after_mount_c,
            )
            .finish(),
    );
}

// ---------------------------------------------------------------------------------------------------------------------
// 一个几何格：几族历史并行，产物行按固定次序打
// ---------------------------------------------------------------------------------------------------------------------

const PROBED_FAMILIES: [&str; 5] = [
    "fill",
    "overwrite",
    "delete_then_write",
    "many_mounts",
    "abandoned_root_and_net_release",
];
const LONG_MOUNT_FAMILY: &str = "long_mount";
const THREAD_STACK_BYTES: usize = 256 << 20;
const TOTAL_THREADS_ENVIRONMENT_VARIABLE: &str = "E180_THREADS";

fn drive_family(family: &str, run: &mut HistoryRun) {
    match family {
        "fill" => drive_fill_family(run),
        "overwrite" => drive_overwrite_family(run),
        "delete_then_write" => drive_delete_then_write_family(run),
        "many_mounts" => drive_many_mounts_family(run),
        "abandoned_root_and_net_release" => drive_abandoned_root_and_net_release_family(run),
        "long_mount" => drive_long_mount_family(run),
        unknown => stop_with("unknown_family", &format!("family={unknown}")),
    }
}

fn static_family_name(family: &str) -> &'static str {
    PROBED_FAMILIES
        .iter()
        .copied()
        .chain([LONG_MOUNT_FAMILY])
        .find(|known| *known == family)
        .unwrap_or_else(|| stop_with("unknown_family", &format!("family={family}")))
}

#[derive(Debug, PartialEq, Eq)]
struct FamilyOutcome {
    family: &'static str,
    lines: Vec<String>,
    tally: FamilyTally,
    anchors: FirstFileAnchors,
    seconds: u64,
    unit_area_slots_per_device: Vec<u64>,
    trajectory_names: [&'static str; 3],
}

fn panic_message(payload: &(dyn std::any::Any + Send)) -> String {
    payload
        .downcast_ref::<&str>()
        .map(|text| (*text).to_string())
        .or_else(|| payload.downcast_ref::<String>().cloned())
        .map_or_else(
            || "panic_without_text".to_string(),
            |text| short_debug(&text),
        )
}

fn run_family(geometry: Geometry, family: &'static str, probe_threads: usize) -> FamilyOutcome {
    let started = Instant::now();
    let (pool, anchors) = build_pool(geometry);
    let unit_area_slots_per_device: Vec<u64> = pool
        .session
        .allocator
        .devices
        .iter()
        .map(|map| map.unit_area_slots())
        .collect();
    let mut run = HistoryRun::new(pool, family, family != LONG_MOUNT_FAMILY, probe_threads);
    if family == LONG_MOUNT_FAMILY {
        run.emit_state_line_every = 50;
    }
    let result = catch_unwind(AssertUnwindSafe(|| drive_family(family, &mut run)));
    if let Err(payload) = result {
        let message = panic_message(payload.as_ref());
        run.lines.push(
            ProductLine::new("panic")
                .field("geometry", geometry.name)
                .field("family", family)
                .field("states_before", run.state_index)
                .field("message", &message)
                .finish(),
        );
        run.tally.panics.push(message);
    }
    FamilyOutcome {
        family,
        lines: std::mem::take(&mut run.lines.lines),
        tally: std::mem::take(&mut run.tally),
        anchors,
        seconds: started.elapsed().as_secs(),
        unit_area_slots_per_device,
        trajectory_names: TRAJECTORY_NAMES,
    }
}

fn total_threads() -> usize {
    let own = std::env::var(TOTAL_THREADS_ENVIRONMENT_VARIABLE).ok();
    match own {
        Some(text) => text
            .parse::<usize>()
            .ok()
            .filter(|count| *count > 0)
            .unwrap_or_else(|| {
                stop_with(
                    "bad_thread_variable",
                    &format!("{TOTAL_THREADS_ENVIRONMENT_VARIABLE}={text}"),
                )
            }),
        None => singlefs_harness::heavy_threads::default_heavy_threads_of_the_cap_value(
            std::env::var(singlefs_harness::heavy_threads::THREAD_CAP_ENVIRONMENT_VARIABLE)
                .ok()
                .as_deref(),
        )
        .get(),
    }
}

/// 一个几何格上跑这几族（并行，每族自己一池），交回按 `families` 的次序排好的结果。
/// 每族的探测线程数按这个几何格的全部族数分（[`probe_threads_of_the_geometry`]），只跑其中一族（一片）时也一样。
fn run_geometry_cell(
    geometry: Geometry,
    families: &[&'static str],
    total_threads: usize,
) -> Vec<FamilyOutcome> {
    let probe_threads = probe_threads_of_the_geometry(geometry, total_threads);
    std::thread::scope(|scope| {
        let handles: Vec<_> = families
            .iter()
            .copied()
            .map(|family| {
                std::thread::Builder::new()
                    .stack_size(THREAD_STACK_BYTES)
                    .spawn_scoped(scope, move || run_family(geometry, family, probe_threads))
                    .expect("起族线程")
            })
            .collect();
        handles
            .into_iter()
            .map(|handle| {
                handle
                    .join()
                    .unwrap_or_else(|_| stop_with("family_thread_panicked_outside_the_guard", ""))
            })
            .collect()
    })
}

// ---------------------------------------------------------------------------------------------------------------------
// 汇总行
// ---------------------------------------------------------------------------------------------------------------------

/// 登记第七节 B1：每块盘单元区槽数的独立算术（默认 211968、窄 146465），与分配器报的一致。
fn append_geometry_header(
    output: &mut Vec<String>,
    geometry: Geometry,
    unit_area_slots_per_device: &[u64],
) {
    let mut holds = true;
    for (index, reported) in unit_area_slots_per_device.iter().enumerate() {
        if *reported != geometry.unit_area_slots_of_device(index) {
            holds = false;
        }
    }
    let expected_literal = match geometry.name {
        "default_geometry"
        | "default_geometry_with_four_slots_per_region"
        | "default_geometry_with_sixteen_slots_per_region" => Some((211_968u64, 211_968u64)),
        "narrow_devices_geometry" => Some((146_465, 146_465)),
        "unequal_devices_geometry" => Some((211_968, 146_465)),
        _ => None,
    };
    let literal_holds = expected_literal
        .is_none_or(|(first, second)| unit_area_slots_per_device == [first, second]);
    output.push(
        ProductLine::new("geometry")
            .field("geometry", geometry.name)
            .field("device_bytes_0", geometry.device_bytes[0])
            .field("device_bytes_1", geometry.device_bytes[1])
            .field("slots_per_region", geometry.slots_per_region)
            .field("unit_area_slots_0", unit_area_slots_per_device[0])
            .field("unit_area_slots_1", unit_area_slots_per_device[1])
            .field("unit_area_slots_match_the_independent_arithmetic", holds)
            .field(
                "unit_area_slots_match_the_registered_numbers",
                literal_holds,
            )
            .finish(),
    );
}

fn append_anchors(output: &mut Vec<String>, geometry: Geometry, anchors: &FirstFileAnchors) {
    let open_segment_closed_free_slots_anchor_holds = anchors.open_segment_start_slot
        == Some(50_240)
        && anchors.closed_free_slots_in_the_open_segment_on_device_zero == Some(52);
    let first_data_unit_slot_anchor_holds = anchors.first_data_unit_slot == Some(50_180);
    output.push(
        ProductLine::new("anchors")
            .field("geometry", geometry.name)
            .field(
                "open_segment_start_slot",
                format!("{:?}", anchors.open_segment_start_slot),
            )
            .field(
                "closed_free_slots_in_the_open_segment_on_device_zero",
                format!(
                    "{:?}",
                    anchors.closed_free_slots_in_the_open_segment_on_device_zero
                ),
            )
            .field(
                "open_segment_closed_free_slots_anchor_expected",
                "segment_50240_with_52_closed_free_slots",
            )
            .field(
                "open_segment_closed_free_slots_anchor_holds",
                open_segment_closed_free_slots_anchor_holds,
            )
            .field(
                "first_data_unit_slot",
                format!("{:?}", anchors.first_data_unit_slot),
            )
            .field("first_data_unit_slot_anchor_expected", 50_180)
            .field(
                "first_data_unit_slot_anchor_holds",
                first_data_unit_slot_anchor_holds,
            )
            .finish(),
    );
}

fn append_arm_tally(
    output: &mut Vec<String>,
    scope: &str,
    geometry: &str,
    family: &str,
    arm: Arm,
    tally: &ArmTally,
) {
    output.push(
        ProductLine::new("arm_tally")
            .field("scope", scope)
            .field("geometry", geometry)
            .field("family", family)
            .field("arm", arm.label())
            .field("over_states", tally.over_states)
            .field("not_over_states", tally.not_over_states)
            .field(
                "not_over_below_lower_bound_states",
                tally.not_over_below_lower_bound_states,
            )
            .field("undetermined_states", tally.undetermined_states)
            .field("max_over_bytes", tally.max_over_bytes)
            .field(
                "max_over_at",
                if tally.max_over_at.is_empty() {
                    "none"
                } else {
                    tally.max_over_at.as_str()
                },
            )
            .field("under_states", tally.under_states)
            .field("max_under_bytes", tally.max_under_bytes)
            .field(
                "max_under_at",
                if tally.max_under_at.is_empty() {
                    "none"
                } else {
                    tally.max_under_at.as_str()
                },
            )
            .field(
                "mean_under_bytes_over_the_states_that_under_report",
                tally
                    .sum_under_bytes
                    .checked_div(u128::from(tally.under_states))
                    .unwrap_or(0),
            )
            .finish(),
    );
}

fn append_residual_tally(
    output: &mut Vec<String>,
    scope: &str,
    geometry: &str,
    family: &str,
    device_index: usize,
    tally: &ResidualTally,
) {
    output.push(
        ProductLine::new("residual_tally")
            .field("scope", scope)
            .field("geometry", geometry)
            .field("family", family)
            .field("device", device_index)
            .field("states", tally.states)
            .field("rewritten_check_residual_nonzero_states", tally.rewritten_check_residual_nonzero_states)
            .field("maximum_absolute_rewritten_check_residual", tally.maximum_absolute_rewritten_check_residual)
            .field(
                "first_state_with_a_nonzero_rewritten_check_residual",
                if tally.first_state_with_a_nonzero_rewritten_check_residual.is_empty() {
                    "none"
                } else {
                    tally.first_state_with_a_nonzero_rewritten_check_residual.as_str()
                },
            )
            .field("original_formula_residual_nonzero_states", tally.original_formula_residual_nonzero_states)
            .field("maximum_absolute_original_formula_residual", tally.maximum_absolute_original_formula_residual)
            .field(
                "first_state_with_a_nonzero_original_formula_residual",
                if tally.first_state_with_a_nonzero_original_formula_residual.is_empty() {
                    "none"
                } else {
                    tally.first_state_with_a_nonzero_original_formula_residual.as_str()
                },
            )
            .field(
                "states_with_overlap_allocated_and_isolated",
                tally.states_with_overlap,
            )
            .field("states_with_held_ledger", tally.states_with_held_ledger)
            .field(
                "control_rewritten_check_residual_without_the_overlap_term_stayed_zero_in_states_with_overlap",
                tally.control_without_overlap_term_stayed_zero,
            )
            .finish(),
    );
}

fn append_trajectory(
    output: &mut Vec<String>,
    geometry: &str,
    family: &str,
    quantity: &str,
    device_index: usize,
    tally: &TrajectoryTally,
) {
    output.push(
        ProductLine::new("trajectory")
            .field("geometry", geometry)
            .field("family", family)
            .field("quantity", quantity)
            .field("device", device_index)
            .field("states", tally.states)
            .field("first_value_slots", tally.first_value)
            .field(
                "peak_after_the_first_slots",
                tally
                    .peak_after_the_first
                    .map_or("none".to_string(), |peak| peak.to_string()),
            )
            .field(
                "positive_states_after_the_first",
                tally.positive_states_after_the_first,
            )
            .field("last_value_slots", tally.last_value)
            .finish(),
    );
}

fn append_family_summary(output: &mut Vec<String>, geometry: &str, outcome: &FamilyOutcome) {
    let tally = &outcome.tally;
    output.push(
        ProductLine::new("family_summary")
            .field("geometry", geometry)
            .field("family", outcome.family)
            .field("seconds", outcome.seconds)
            .field("states", tally.states)
            .field("exact_states", tally.exact_states)
            .field("censored_states", tally.censored_states)
            .field("unreadable_states", tally.unreadable_states)
            .field("probes_run", tally.probes_run)
            .field("non_monotone_states", tally.non_monotone_states)
            .field("control_failures", tally.control_failures.len())
            .field("shortcut_pairs_checked", tally.shortcut_pairs_checked)
            .field("shortcut_differences", tally.shortcut_differences.len())
            .field(
                "records_disagree_with_the_disk_states",
                tally.records_disagree_with_the_disk_states,
            )
            .field(
                "decomposition_failure_states",
                tally.decomposition_failure_states,
            )
            .field(
                "not_whole_slot_available_states",
                tally.not_whole_slot_available_states,
            )
            .field("lag_unattributed_states", tally.lag_unattributed_states)
            .field(
                "held_ledger_mismatch_device_states",
                tally.stop_clause_held_ledger_mismatch_states,
            )
            .field("unmount_pushes_checked", tally.unmount_pushes_checked)
            .field(
                "unmount_pushes_differing",
                tally.unmount_pushes_differing.len(),
            )
            .field("panics", tally.panics.len())
            .finish(),
    );
    for detail in tally
        .control_failures
        .iter()
        .chain(&tally.shortcut_differences)
        .chain(&tally.unmount_pushes_differing)
    {
        output.push(
            ProductLine::new("problem_detail")
                .field("geometry", geometry)
                .field("family", outcome.family)
                .field("detail", detail)
                .finish(),
        );
    }
    for (arm, arm_tally) in &tally.arms {
        append_arm_tally(output, "family", geometry, outcome.family, *arm, arm_tally);
    }
    for (device_index, residual) in tally.residuals.iter().enumerate() {
        append_residual_tally(
            output,
            "family",
            geometry,
            outcome.family,
            device_index,
            residual,
        );
    }
    for (name, trajectories) in [
        (outcome.trajectory_names[0], &tally.closed_minus_ring_window),
        (outcome.trajectory_names[1], &tally.closed_alone),
        (
            outcome.trajectory_names[2],
            &tally.closed_minus_all_common_commitments,
        ),
    ] {
        for (device_index, trajectory) in trajectories.iter().enumerate() {
            append_trajectory(
                output,
                geometry,
                outcome.family,
                name,
                device_index,
                trajectory,
            );
        }
    }
}

/// 同一个几何格上几族的累计并起来（只并探测过的族；长挂载族不带探测，它的轨迹按族报）。
fn merged_counts_of_the_probed_families(outcomes: &[FamilyOutcome]) -> FamilyTally {
    let mut merged = FamilyTally::default();
    for outcome in outcomes
        .iter()
        .filter(|outcome| outcome.family != LONG_MOUNT_FAMILY)
    {
        merged.merge_counts_of(&outcome.tally);
    }
    merged
}

fn per_mille(part: u64, whole: u64) -> u64 {
    (part * 1000).checked_div(whole).unwrap_or(0)
}

fn append_cell_summary(
    output: &mut Vec<String>,
    geometry: &str,
    outcomes: &[FamilyOutcome],
) -> FamilyTally {
    let merged = merged_counts_of_the_probed_families(outcomes);
    for (arm, arm_tally) in &merged.arms {
        append_arm_tally(
            output,
            "geometry",
            geometry,
            "all_probed_families",
            *arm,
            arm_tally,
        );
    }
    for (device_index, residual) in merged.residuals.iter().enumerate() {
        append_residual_tally(
            output,
            "geometry",
            geometry,
            "all_probed_families",
            device_index,
            residual,
        );
    }
    let not_exact = merged.censored_states + merged.unreadable_states;
    output.push(
        ProductLine::new("writable_units_coverage")
            .field("geometry", geometry)
            .field("probed_states", merged.states)
            .field("exact_states", merged.exact_states)
            .field(
                "censored_by_the_journal_record_cap_states",
                merged.censored_states,
            )
            .field("unreadable_states", merged.unreadable_states)
            .field(
                "literal_per_mille_of_states_without_an_exact_writable_amount",
                per_mille(not_exact, merged.states),
            )
            .field(
                "literal_states_without_an_exact_writable_amount_exceed_ten_percent",
                not_exact * 10 > merged.states,
            )
            .field(
                "unreadable_only_per_mille_of_states",
                per_mille(merged.unreadable_states, merged.states),
            )
            .field(
                "unreadable_only_states_exceed_ten_percent",
                merged.unreadable_states * 10 > merged.states,
            )
            .finish(),
    );
    // 登记第六节 Q5、Q7 按族分开报：被抛弃根与净释放族与其余几族（登记第六节 Q5：扣住位族是第二段，这一段没有）。
    for scope in [
        "abandoned_root_and_net_release_only",
        "all_families_but_abandoned_root_and_net_release",
    ] {
        let mut residual_of_the_scope = vec![ResidualTally::default(); merged.residuals.len()];
        for outcome in outcomes {
            let is_abandoned_root_and_net_release_family =
                outcome.family == "abandoned_root_and_net_release";
            if (scope == "abandoned_root_and_net_release_only")
                == is_abandoned_root_and_net_release_family
            {
                for (mine, theirs) in residual_of_the_scope
                    .iter_mut()
                    .zip(&outcome.tally.residuals)
                {
                    mine.merge(theirs);
                }
            }
        }
        for (device_index, residual) in residual_of_the_scope.iter().enumerate() {
            append_residual_tally(output, scope, geometry, "see_scope", device_index, residual);
        }
    }
    merged
}

fn append_verdict(
    output: &mut Vec<String>,
    geometry: &str,
    outcomes: &[FamilyOutcome],
    merged: &FamilyTally,
    local_constants_checked: usize,
) {
    let anchors = &outcomes[0].anchors;
    let open_segment_closed_free_slots_anchor_holds = anchors.open_segment_start_slot
        == Some(50_240)
        && anchors.closed_free_slots_in_the_open_segment_on_device_zero == Some(52);
    let first_data_unit_slot_anchor_holds = anchors.first_data_unit_slot == Some(50_180);
    output.push(
        ProductLine::new("verdict")
            .field("geometry", geometry)
            .field("local_constants_checked", local_constants_checked)
            .field("control_failures", merged.control_failures.len())
            .field("shortcut_differences", merged.shortcut_differences.len())
            .field(
                "open_segment_closed_free_slots_anchor_holds",
                open_segment_closed_free_slots_anchor_holds,
            )
            .field(
                "first_data_unit_slot_anchor_holds",
                first_data_unit_slot_anchor_holds,
            )
            .field(
                "unmount_pushes_anchor_differences",
                merged.unmount_pushes_differing.len(),
            )
            .field("panics", merged.panics.len())
            .field(
                "records_disagree_with_the_disk_states",
                merged.records_disagree_with_the_disk_states,
            )
            .field(
                "decomposition_failure_states",
                merged.decomposition_failure_states,
            )
            .field("lag_unattributed_states", merged.lag_unattributed_states)
            .field(
                "held_ledger_mismatch_device_states",
                merged.stop_clause_held_ledger_mismatch_states,
            )
            .field("non_monotone_states", merged.non_monotone_states)
            .field("unreadable_states", merged.unreadable_states)
            .field(
                "censored_by_the_journal_record_cap_states",
                merged.censored_states,
            )
            .finish(),
    );
}

// ---------------------------------------------------------------------------------------------------------------------
// 几何敏感性（登记第八节）：每对方向相反的取样点上各判一次，翻面记不稳定；判别力自证：门槛挪到两点读数之间，判定必须由一边转到另一边
// ---------------------------------------------------------------------------------------------------------------------

/// 门槛放在两个读数中间，「读数高于门槛」的判定在两点上必须不同；两点读数相等时没有可自证的，交回 `None`。
fn threshold_between_flips_the_judgement(reading_a: i128, reading_b: i128) -> Option<bool> {
    if reading_a == reading_b {
        return None;
    }
    let twice_the_threshold = reading_a + reading_b;
    Some((2 * reading_a > twice_the_threshold) != (2 * reading_b > twice_the_threshold))
}

fn append_sensitivity_pair(
    output: &mut Vec<String>,
    quantity: &str,
    point_a: &str,
    reading_a: i128,
    point_b: &str,
    reading_b: i128,
    flips_on_the_registered_threshold: bool,
) {
    let self_check = match threshold_between_flips_the_judgement(reading_a, reading_b) {
        Some(true) => "flips",
        Some(false) => "does_not_flip_stop",
        None => "equal_readings_not_applicable",
    };
    output.push(
        ProductLine::new("geometry_sensitivity")
            .field("quantity", quantity)
            .field("point_a", point_a)
            .field("reading_a", reading_a)
            .field("point_b", point_b)
            .field("reading_b", reading_b)
            .field(
                "registered_threshold_judgement_flips",
                flips_on_the_registered_threshold,
            )
            .field(
                "verdict_is_unstable_across_the_pair",
                flips_on_the_registered_threshold,
            )
            .field("discrimination_self_check", self_check)
            .finish(),
    );
}

fn append_geometry_sensitivity(
    output: &mut Vec<String>,
    merged_by_geometry: &BTreeMap<&'static str, FamilyTally>,
    trajectories: &BTreeMap<(&'static str, &'static str), TrajectoryTally>,
) {
    let pairs = [
        ("default_geometry", "narrow_devices_geometry"),
        (
            "default_geometry_with_four_slots_per_region",
            "default_geometry_with_sixteen_slots_per_region",
        ),
        ("default_geometry", "unequal_devices_geometry"),
    ];
    for (first, second) in pairs {
        let (Some(first_tally), Some(second_tally)) = (
            merged_by_geometry.get(first),
            merged_by_geometry.get(second),
        ) else {
            continue;
        };
        for arm in all_arms() {
            let over = |tally: &FamilyTally| {
                tally
                    .arms
                    .get(&arm)
                    .map_or(0, |arm_tally| arm_tally.over_states)
            };
            let (reading_first, reading_second) = (over(first_tally), over(second_tally));
            append_sensitivity_pair(
                output,
                &format!("over_reporting_states_of_{}", arm.label()),
                first,
                i128::from(reading_first),
                second,
                i128::from(reading_second),
                (reading_first >= 1) != (reading_second >= 1),
            );
        }
    }
    for (first, second) in [("default_geometry", "narrow_devices_geometry")] {
        let (Some(first_tally), Some(second_tally)) = (
            merged_by_geometry.get(first),
            merged_by_geometry.get(second),
        ) else {
            continue;
        };
        let nonzero = |tally: &FamilyTally, pick: fn(&ResidualTally) -> u64| -> u64 {
            tally.residuals.iter().map(pick).sum()
        };
        for (quantity, pick) in [
            (
                "rewritten_check_residual_nonzero_states",
                (|residual: &ResidualTally| residual.rewritten_check_residual_nonzero_states)
                    as fn(&ResidualTally) -> u64,
            ),
            (
                "original_formula_residual_nonzero_states",
                (|residual: &ResidualTally| residual.original_formula_residual_nonzero_states)
                    as fn(&ResidualTally) -> u64,
            ),
        ] {
            let (reading_first, reading_second) =
                (nonzero(first_tally, pick), nonzero(second_tally, pick));
            append_sensitivity_pair(
                output,
                quantity,
                first,
                i128::from(reading_first),
                second,
                i128::from(reading_second),
                (reading_first >= 1) != (reading_second >= 1),
            );
        }
    }
    for (first, second) in [
        (
            "default_geometry_with_sixteen_slots_per_region",
            "default_geometry_with_four_slots_per_region",
        ),
        ("default_geometry", "narrow_devices_geometry"),
    ] {
        for family in ["overwrite", "many_mounts", "long_mount"] {
            let (Some(first_trajectory), Some(second_trajectory)) = (
                trajectories.get(&(first, family)),
                trajectories.get(&(second, family)),
            ) else {
                continue;
            };
            let peak = |trajectory: &TrajectoryTally| trajectory.peak_after_the_first.unwrap_or(0);
            let (reading_first, reading_second) = (peak(first_trajectory), peak(second_trajectory));
            append_sensitivity_pair(
                output,
                &format!("peak_closed_minus_ring_window_slots_device_0_{family}"),
                first,
                reading_first,
                second,
                reading_second,
                (reading_first > 0) != (reading_second > 0),
            );
        }
    }
}

// ---------------------------------------------------------------------------------------------------------------------
// 入口
// ---------------------------------------------------------------------------------------------------------------------

/// 登记第七节 B2、B3：独立算术。
fn append_independent_arithmetic(output: &mut Vec<String>) {
    let payload_by_subtraction = DATA_UNIT_BYTES - 134;
    // B3：每盘分配记录树层级 0 罩 262 个位置、层级 1 罩 2 个，两盘 528 加根 1。
    const LEAF_SLOTS: u64 = 812;
    const INTERNAL_FANOUT: u64 = 169;
    let first_slot = DEFAULT_UNIT_AREA_START_SLOT;
    let last_slot = FOUR_GIBIBYTES / SLOT_BYTES - 1;
    let level_zero_positions = last_slot / LEAF_SLOTS - first_slot / LEAF_SLOTS + 1;
    let level_one_span = LEAF_SLOTS * INTERNAL_FANOUT;
    let level_one_positions = last_slot / level_one_span - first_slot / level_one_span + 1;
    let nodes_over_two_devices = DEVICE_COUNT * (level_zero_positions + level_one_positions) + 1;
    output.push(
        ProductLine::new("independent_arithmetic")
            .field("payload_bytes_by_subtraction", payload_by_subtraction)
            .field("payload_bytes_by_subtraction_holds", payload_by_subtraction == DATA_UNIT_PAYLOAD_BYTES)
            .field("allocation_record_tree_level_zero_positions_per_device", level_zero_positions)
            .field("allocation_record_tree_level_one_positions_per_device", level_one_positions)
            .field("allocation_record_tree_nodes_over_two_default_devices", nodes_over_two_devices)
            .field("allocation_record_tree_nodes_by_independent_arithmetic_holds", nodes_over_two_devices == 529)
            .field(
                "allocation_record_tree_nodes_anchor_implementation_side",
                "not_observable_through_a_public_entry_allocation_record_tree_positions_over_the_unit_areas_is_private",
            )
            .finish(),
    );
}

struct ReadingOptions {
    geometry: Option<String>,
    family: Option<String>,
    /// 给了就只跑一片（`--geometry` 与 `--family` 都要给），结果写成片文件（[`shard_text`]），原子换名到这个路径，标准输出不打产物。
    shard_output: Option<String>,
}

fn parse_reading_options(options: &[String]) -> ReadingOptions {
    let mut parsed = ReadingOptions {
        geometry: None,
        family: None,
        shard_output: None,
    };
    let mut index = 0;
    while index < options.len() {
        let value = options.get(index + 1);
        match (options[index].as_str(), value) {
            ("--geometry", Some(name)) => parsed.geometry = Some(name.clone()),
            ("--family", Some(name)) => parsed.family = Some(name.clone()),
            ("--shard-output", Some(path)) => parsed.shard_output = Some(path.clone()),
            (other, _) => stop_with("bad_option", &format!("option={other}")),
        }
        index += 2;
    }
    if parsed.shard_output.is_some() && (parsed.geometry.is_none() || parsed.family.is_none()) {
        stop_with("bad_option", "shard_output_needs_both_geometry_and_family");
    }
    parsed
}

fn families_of_the_geometry(geometry: Geometry, only: Option<&str>) -> Vec<&'static str> {
    let mut families: Vec<&'static str> = PROBED_FAMILIES.to_vec();
    if geometry.name != "unequal_devices_geometry" {
        families.push(LONG_MOUNT_FAMILY);
    }
    match only {
        Some(name) => vec![static_family_name(name)],
        None => families,
    }
}

/// 一族的探测线程数 K（一轮探几个点）：总线程数按这个几何格的全部族数平分，与这一次只跑其中几族无关。
/// 一片只跑一族，K 照样按全部族数分，片跑出来的与一次跑全部的那一族逐字节相同（K 会改探测点，见 [`find_writable_units`]）。
fn probe_threads_of_the_geometry(geometry: Geometry, total_threads: usize) -> usize {
    (total_threads / families_of_the_geometry(geometry, None).len()).max(1)
}

/// 片文件名：`<几何名>-<族名>.shard`；合并按 [`ALL_GEOMETRIES`] 与 [`families_of_the_geometry`] 的次序找。
fn shard_file_name(geometry: Geometry, family: &str) -> String {
    format!("{}-{family}.shard", geometry.name)
}

/// 一个几何格的结果，族按 [`families_of_the_geometry`] 的次序。
struct GeometryCellOutcome {
    geometry: Geometry,
    outcomes: Vec<FamilyOutcome>,
}

/// 产物的全部行（一次跑全部与合并片文件共用这一个函数，两条路打出的产物因此逐字节相同）：本地常量、独立算术，
/// 每个几何格的几何行、锚点、各族的状态行与族汇总、格汇总、判决行，最后几何敏感性与完成标记 `E name=done emitted=<行数>`。
fn reading_product_lines(
    local_constants_checked: usize,
    cells: &[GeometryCellOutcome],
) -> Vec<String> {
    let mut output = Vec::new();
    output.push(
        ProductLine::new("local_constants_checked")
            .field("count", local_constants_checked)
            .finish(),
    );
    append_independent_arithmetic(&mut output);
    let mut merged_by_geometry: BTreeMap<&'static str, FamilyTally> = BTreeMap::new();
    let mut trajectories: BTreeMap<(&'static str, &'static str), TrajectoryTally> = BTreeMap::new();
    for cell in cells {
        let geometry = cell.geometry;
        let outcomes = &cell.outcomes;
        append_geometry_header(
            &mut output,
            geometry,
            &outcomes[0].unit_area_slots_per_device,
        );
        append_anchors(&mut output, geometry, &outcomes[0].anchors);
        for outcome in outcomes {
            output.extend(outcome.lines.iter().cloned());
            append_family_summary(&mut output, geometry.name, outcome);
            if let Some(trajectory) = outcome.tally.closed_minus_ring_window.first() {
                trajectories.insert((geometry.name, outcome.family), trajectory.clone());
            }
        }
        let merged = append_cell_summary(&mut output, geometry.name, outcomes);
        append_verdict(
            &mut output,
            geometry.name,
            outcomes,
            &merged,
            local_constants_checked,
        );
        merged_by_geometry.insert(geometry.name, merged);
    }
    append_geometry_sensitivity(&mut output, &merged_by_geometry, &trajectories);
    let emitted = output.len() + 1;
    output.push(ProductLine::new("done").field("emitted", emitted).finish());
    output
}

// ---------------------------------------------------------------------------------------------------------------------
// 片文件：一片 = 一个几何格里的一族（`reading --geometry <几何名> --family <族名> --shard-output <文件>`），`merge` 把齐了的片合成产物
// ---------------------------------------------------------------------------------------------------------------------

/// 片文件的格式标记：改了片文件的写法就换这个值，合并认不得旧写法的片。
const SHARD_FORMAT: &str = "e180-reading-shard-v1";
/// 片文件末行的键：值是整份的行数（含这一行）；合并与长跑脚本按它认这一片做完了。
const SHARD_DONE_KEY: &str = "shard_done";
/// 三条轨迹的名字（产物 `trajectory` 行的 `quantity`），次序同 [`FamilyTally`] 里三个轨迹字段。
const TRAJECTORY_NAMES: [&str; 3] = [
    "closed_minus_ring_window",
    "closed_alone",
    "closed_minus_all_common_commitments",
];

fn escape_shard_value(text: &str) -> String {
    let mut escaped = String::with_capacity(text.len());
    for character in text.chars() {
        match character {
            '\\' => escaped.push_str("\\\\"),
            '\t' => escaped.push_str("\\t"),
            '\n' => escaped.push_str("\\n"),
            '\r' => escaped.push_str("\\r"),
            other => escaped.push(other),
        }
    }
    escaped
}

fn unescape_shard_value(text: &str) -> Result<String, String> {
    let mut plain = String::with_capacity(text.len());
    let mut characters = text.chars();
    while let Some(character) = characters.next() {
        if character != '\\' {
            plain.push(character);
            continue;
        }
        match characters.next() {
            Some('\\') => plain.push('\\'),
            Some('t') => plain.push('\t'),
            Some('n') => plain.push('\n'),
            Some('r') => plain.push('\r'),
            other => return Err(format!("bad_escape_after_backslash={other:?}")),
        }
    }
    Ok(plain)
}

/// 片文件的写端：一条一行「键 TAB 转义过的值」，次序就是读端 [`ShardReader`] 取的次序。
#[derive(Default)]
struct ShardWriter {
    lines: Vec<String>,
}

impl ShardWriter {
    fn put(&mut self, key: &str, value: impl std::fmt::Display) {
        self.lines
            .push(format!("{key}\t{}", escape_shard_value(&value.to_string())));
    }

    fn put_optional<Value: std::fmt::Display>(&mut self, key: &str, value: Option<Value>) {
        match value {
            Some(present) => self.put(key, present),
            None => self.put(key, "none"),
        }
    }

    /// 先写条数，再一条一行。
    fn put_texts(&mut self, count_key: &str, item_key: &str, texts: &[String]) {
        self.put(count_key, texts.len());
        for text in texts {
            self.put(item_key, text);
        }
    }

    /// 整份文本：末行是 `shard_done TAB <行数>`，每行以换行收尾。
    fn finish(mut self) -> String {
        let line_count = self.lines.len() + 1;
        self.lines.push(format!("{SHARD_DONE_KEY}\t{line_count}"));
        let mut text = self.lines.join("\n");
        text.push('\n');
        text
    }
}

/// 片文件的读端：按写端的次序逐条取，键对不上、条数不够、末行不是做完标记都交回错误（合并据此停机）。
struct ShardReader {
    records: std::collections::VecDeque<(String, String)>,
}

impl ShardReader {
    fn new(text: &str) -> Result<Self, String> {
        if !text.ends_with('\n') {
            return Err("shard_text_does_not_end_with_a_newline".to_string());
        }
        let lines: Vec<&str> = text.lines().collect();
        let expected_done_line = format!("{SHARD_DONE_KEY}\t{}", lines.len());
        match lines.last() {
            Some(last) if *last == expected_done_line => {}
            Some(last) => {
                return Err(format!(
                    "last_line_is_not_the_done_marker last={last:?} expected={expected_done_line:?}"
                ))
            }
            None => return Err("empty_shard".to_string()),
        }
        let mut records = std::collections::VecDeque::new();
        for line in &lines[..lines.len() - 1] {
            let (key, value) = line
                .split_once('\t')
                .ok_or_else(|| format!("line_without_a_tab={line:?}"))?;
            records.push_back((key.to_string(), unescape_shard_value(value)?));
        }
        Ok(ShardReader { records })
    }

    fn take(&mut self, key: &str) -> Result<String, String> {
        match self.records.pop_front() {
            Some((found, value)) if found == key => Ok(value),
            Some((found, _)) => Err(format!("expected_key={key} found_key={found}")),
            None => Err(format!("expected_key={key} found_the_end")),
        }
    }

    fn take_parsed<Value: std::str::FromStr>(&mut self, key: &str) -> Result<Value, String> {
        let text = self.take(key)?;
        text.parse::<Value>()
            .map_err(|_| format!("key={key} unparsable_value={text:?}"))
    }

    fn take_optional<Value: std::str::FromStr>(
        &mut self,
        key: &str,
    ) -> Result<Option<Value>, String> {
        let text = self.take(key)?;
        if text == "none" {
            return Ok(None);
        }
        text.parse::<Value>()
            .map(Some)
            .map_err(|_| format!("key={key} unparsable_value={text:?}"))
    }

    fn take_texts(&mut self, count_key: &str, item_key: &str) -> Result<Vec<String>, String> {
        let count: usize = self.take_parsed(count_key)?;
        (0..count).map(|_| self.take(item_key)).collect()
    }

    fn finish(self) -> Result<(), String> {
        match self.records.front() {
            None => Ok(()),
            Some((key, _)) => Err(format!("records_left_over_from_key={key}")),
        }
    }
}

fn put_arm_tally(writer: &mut ShardWriter, tally: &ArmTally) {
    let ArmTally {
        over_states,
        not_over_states,
        not_over_below_lower_bound_states,
        undetermined_states,
        max_over_bytes,
        max_over_at,
        max_under_bytes,
        max_under_at,
        sum_under_bytes,
        under_states,
    } = tally;
    writer.put("over_states", over_states);
    writer.put("not_over_states", not_over_states);
    writer.put(
        "not_over_below_lower_bound_states",
        not_over_below_lower_bound_states,
    );
    writer.put("undetermined_states", undetermined_states);
    writer.put("max_over_bytes", max_over_bytes);
    writer.put("max_over_at", max_over_at);
    writer.put("max_under_bytes", max_under_bytes);
    writer.put("max_under_at", max_under_at);
    writer.put("sum_under_bytes", sum_under_bytes);
    writer.put("under_states", under_states);
}

fn take_arm_tally(reader: &mut ShardReader) -> Result<ArmTally, String> {
    Ok(ArmTally {
        over_states: reader.take_parsed("over_states")?,
        not_over_states: reader.take_parsed("not_over_states")?,
        not_over_below_lower_bound_states: reader
            .take_parsed("not_over_below_lower_bound_states")?,
        undetermined_states: reader.take_parsed("undetermined_states")?,
        max_over_bytes: reader.take_parsed("max_over_bytes")?,
        max_over_at: reader.take("max_over_at")?,
        max_under_bytes: reader.take_parsed("max_under_bytes")?,
        max_under_at: reader.take("max_under_at")?,
        sum_under_bytes: reader.take_parsed("sum_under_bytes")?,
        under_states: reader.take_parsed("under_states")?,
    })
}

fn put_residual_tally(writer: &mut ShardWriter, tally: &ResidualTally) {
    let ResidualTally {
        states,
        rewritten_check_residual_nonzero_states,
        original_formula_residual_nonzero_states,
        maximum_absolute_rewritten_check_residual,
        maximum_absolute_original_formula_residual,
        states_with_overlap,
        states_with_held_ledger,
        control_without_overlap_term_stayed_zero,
        first_state_with_a_nonzero_rewritten_check_residual,
        first_state_with_a_nonzero_original_formula_residual,
    } = tally;
    writer.put("residual_states", states);
    writer.put(
        "rewritten_check_residual_nonzero_states",
        rewritten_check_residual_nonzero_states,
    );
    writer.put(
        "original_formula_residual_nonzero_states",
        original_formula_residual_nonzero_states,
    );
    writer.put(
        "maximum_absolute_rewritten_check_residual",
        maximum_absolute_rewritten_check_residual,
    );
    writer.put(
        "maximum_absolute_original_formula_residual",
        maximum_absolute_original_formula_residual,
    );
    writer.put("states_with_overlap", states_with_overlap);
    writer.put("states_with_held_ledger", states_with_held_ledger);
    writer.put(
        "control_without_overlap_term_stayed_zero",
        control_without_overlap_term_stayed_zero,
    );
    writer.put(
        "first_state_with_a_nonzero_rewritten_check_residual",
        first_state_with_a_nonzero_rewritten_check_residual,
    );
    writer.put(
        "first_state_with_a_nonzero_original_formula_residual",
        first_state_with_a_nonzero_original_formula_residual,
    );
}

fn take_residual_tally(reader: &mut ShardReader) -> Result<ResidualTally, String> {
    Ok(ResidualTally {
        states: reader.take_parsed("residual_states")?,
        rewritten_check_residual_nonzero_states: reader
            .take_parsed("rewritten_check_residual_nonzero_states")?,
        original_formula_residual_nonzero_states: reader
            .take_parsed("original_formula_residual_nonzero_states")?,
        maximum_absolute_rewritten_check_residual: reader
            .take_parsed("maximum_absolute_rewritten_check_residual")?,
        maximum_absolute_original_formula_residual: reader
            .take_parsed("maximum_absolute_original_formula_residual")?,
        states_with_overlap: reader.take_parsed("states_with_overlap")?,
        states_with_held_ledger: reader.take_parsed("states_with_held_ledger")?,
        control_without_overlap_term_stayed_zero: reader
            .take_parsed("control_without_overlap_term_stayed_zero")?,
        first_state_with_a_nonzero_rewritten_check_residual: reader
            .take("first_state_with_a_nonzero_rewritten_check_residual")?,
        first_state_with_a_nonzero_original_formula_residual: reader
            .take("first_state_with_a_nonzero_original_formula_residual")?,
    })
}

fn put_trajectory_tally(writer: &mut ShardWriter, tally: &TrajectoryTally) {
    let TrajectoryTally {
        states,
        first_value,
        peak_after_the_first,
        positive_states_after_the_first,
        last_value,
    } = tally;
    writer.put("trajectory_states", states);
    writer.put("first_value", first_value);
    writer.put_optional("peak_after_the_first", *peak_after_the_first);
    writer.put(
        "positive_states_after_the_first",
        positive_states_after_the_first,
    );
    writer.put("last_value", last_value);
}

fn take_trajectory_tally(reader: &mut ShardReader) -> Result<TrajectoryTally, String> {
    Ok(TrajectoryTally {
        states: reader.take_parsed("trajectory_states")?,
        first_value: reader.take_parsed("first_value")?,
        peak_after_the_first: reader.take_optional("peak_after_the_first")?,
        positive_states_after_the_first: reader.take_parsed("positive_states_after_the_first")?,
        last_value: reader.take_parsed("last_value")?,
    })
}

fn put_trajectories(writer: &mut ShardWriter, name: &str, trajectories: &[TrajectoryTally]) {
    writer.put("trajectory_devices", name);
    writer.put("devices", trajectories.len());
    for trajectory in trajectories {
        put_trajectory_tally(writer, trajectory);
    }
}

fn take_trajectories(reader: &mut ShardReader, name: &str) -> Result<Vec<TrajectoryTally>, String> {
    let found = reader.take("trajectory_devices")?;
    if found != name {
        return Err(format!("expected_trajectory={name} found={found}"));
    }
    let devices: usize = reader.take_parsed("devices")?;
    (0..devices)
        .map(|_| take_trajectory_tally(reader))
        .collect()
}

fn put_family_tally(writer: &mut ShardWriter, tally: &FamilyTally) {
    let FamilyTally {
        states,
        exact_states,
        censored_states,
        unreadable_states,
        arms,
        residuals,
        closed_minus_ring_window,
        closed_alone,
        closed_minus_all_common_commitments,
        control_failures,
        shortcut_pairs_checked,
        shortcut_differences,
        records_disagree_with_the_disk_states,
        decomposition_failure_states,
        not_whole_slot_available_states,
        lag_unattributed_states,
        non_monotone_states,
        probes_run,
        stop_clause_held_ledger_mismatch_states,
        unmount_pushes_checked,
        unmount_pushes_differing,
        panics,
    } = tally;
    writer.put("states", states);
    writer.put("exact_states", exact_states);
    writer.put("censored_states", censored_states);
    writer.put("unreadable_states", unreadable_states);
    writer.put("arms", arms.len());
    for (arm, arm_tally) in arms {
        writer.put("arm", arm.label());
        put_arm_tally(writer, arm_tally);
    }
    writer.put("residual_devices", residuals.len());
    for residual in residuals {
        put_residual_tally(writer, residual);
    }
    put_trajectories(writer, TRAJECTORY_NAMES[0], closed_minus_ring_window);
    put_trajectories(writer, TRAJECTORY_NAMES[1], closed_alone);
    put_trajectories(
        writer,
        TRAJECTORY_NAMES[2],
        closed_minus_all_common_commitments,
    );
    writer.put_texts("control_failures", "control_failure", control_failures);
    writer.put("shortcut_pairs_checked", shortcut_pairs_checked);
    writer.put_texts(
        "shortcut_differences",
        "shortcut_difference",
        shortcut_differences,
    );
    writer.put(
        "records_disagree_with_the_disk_states",
        records_disagree_with_the_disk_states,
    );
    writer.put("decomposition_failure_states", decomposition_failure_states);
    writer.put(
        "not_whole_slot_available_states",
        not_whole_slot_available_states,
    );
    writer.put("lag_unattributed_states", lag_unattributed_states);
    writer.put("non_monotone_states", non_monotone_states);
    writer.put("probes_run", probes_run);
    writer.put(
        "stop_clause_held_ledger_mismatch_states",
        stop_clause_held_ledger_mismatch_states,
    );
    writer.put("unmount_pushes_checked", unmount_pushes_checked);
    writer.put_texts(
        "unmount_pushes_differing",
        "unmount_push_differing",
        unmount_pushes_differing,
    );
    writer.put_texts("panics", "panic", panics);
}

fn take_family_tally(reader: &mut ShardReader) -> Result<FamilyTally, String> {
    let states = reader.take_parsed("states")?;
    let exact_states = reader.take_parsed("exact_states")?;
    let censored_states = reader.take_parsed("censored_states")?;
    let unreadable_states = reader.take_parsed("unreadable_states")?;
    let arm_count: usize = reader.take_parsed("arms")?;
    let mut arms = BTreeMap::new();
    for _ in 0..arm_count {
        let label = reader.take("arm")?;
        let arm = all_arms()
            .into_iter()
            .find(|arm| arm.label() == label)
            .ok_or_else(|| format!("unknown_arm={label}"))?;
        arms.insert(arm, take_arm_tally(reader)?);
    }
    let residual_devices: usize = reader.take_parsed("residual_devices")?;
    let residuals = (0..residual_devices)
        .map(|_| take_residual_tally(reader))
        .collect::<Result<Vec<_>, String>>()?;
    let closed_minus_ring_window = take_trajectories(reader, TRAJECTORY_NAMES[0])?;
    let closed_alone = take_trajectories(reader, TRAJECTORY_NAMES[1])?;
    let closed_minus_all_common_commitments = take_trajectories(reader, TRAJECTORY_NAMES[2])?;
    Ok(FamilyTally {
        states,
        exact_states,
        censored_states,
        unreadable_states,
        arms,
        residuals,
        closed_minus_ring_window,
        closed_alone,
        closed_minus_all_common_commitments,
        control_failures: reader.take_texts("control_failures", "control_failure")?,
        shortcut_pairs_checked: reader.take_parsed("shortcut_pairs_checked")?,
        shortcut_differences: reader.take_texts("shortcut_differences", "shortcut_difference")?,
        records_disagree_with_the_disk_states: reader
            .take_parsed("records_disagree_with_the_disk_states")?,
        decomposition_failure_states: reader.take_parsed("decomposition_failure_states")?,
        not_whole_slot_available_states: reader.take_parsed("not_whole_slot_available_states")?,
        lag_unattributed_states: reader.take_parsed("lag_unattributed_states")?,
        non_monotone_states: reader.take_parsed("non_monotone_states")?,
        probes_run: reader.take_parsed("probes_run")?,
        stop_clause_held_ledger_mismatch_states: reader
            .take_parsed("stop_clause_held_ledger_mismatch_states")?,
        unmount_pushes_checked: reader.take_parsed("unmount_pushes_checked")?,
        unmount_pushes_differing: reader
            .take_texts("unmount_pushes_differing", "unmount_push_differing")?,
        panics: reader.take_texts("panics", "panic")?,
    })
}

/// 片头：这一片是哪一格、跑它的那一次的总线程数与探测线程数、本地常量回比了几个；合并核每片的格对得上、几片的这几样一致。
#[derive(Clone, Debug, PartialEq, Eq)]
struct ShardHeader {
    geometry_name: String,
    family: String,
    total_threads: usize,
    probe_threads: usize,
    local_constants_checked: usize,
}

/// 一片的片文件：片头、这一族的耗时与单元区槽数、锚点、三条轨迹名、产物行、累计；次序就是 [`parse_shard`] 读的次序。
fn shard_text(header: &ShardHeader, outcome: &FamilyOutcome) -> String {
    let FamilyOutcome {
        family,
        lines,
        tally,
        anchors,
        seconds,
        unit_area_slots_per_device,
        trajectory_names,
    } = outcome;
    assert_eq!(header.family, *family, "片头的族与这一族的结果一致");
    let mut writer = ShardWriter::default();
    writer.put("shard_format", SHARD_FORMAT);
    writer.put("geometry", &header.geometry_name);
    writer.put("family", &header.family);
    writer.put("total_threads", header.total_threads);
    writer.put("probe_threads", header.probe_threads);
    writer.put("local_constants_checked", header.local_constants_checked);
    writer.put("seconds", seconds);
    writer.put("unit_area_devices", unit_area_slots_per_device.len());
    for slots in unit_area_slots_per_device {
        writer.put("unit_area_slots", slots);
    }
    let FirstFileAnchors {
        open_segment_start_slot,
        closed_free_slots_in_the_open_segment_on_device_zero,
        first_data_unit_slot,
    } = anchors;
    writer.put_optional("open_segment_start_slot", *open_segment_start_slot);
    writer.put_optional(
        "closed_free_slots_in_the_open_segment_on_device_zero",
        *closed_free_slots_in_the_open_segment_on_device_zero,
    );
    writer.put_optional("first_data_unit_slot", *first_data_unit_slot);
    for name in trajectory_names {
        writer.put("trajectory_name", name);
    }
    writer.put_texts("product_lines", "product_line", lines);
    put_family_tally(&mut writer, tally);
    writer.finish()
}

fn parse_shard(text: &str) -> Result<(ShardHeader, FamilyOutcome), String> {
    let mut reader = ShardReader::new(text)?;
    let format = reader.take("shard_format")?;
    if format != SHARD_FORMAT {
        return Err(format!("shard_format={format} expected={SHARD_FORMAT}"));
    }
    let geometry_name = reader.take("geometry")?;
    let family_text = reader.take("family")?;
    let family = PROBED_FAMILIES
        .iter()
        .copied()
        .chain([LONG_MOUNT_FAMILY])
        .find(|known| *known == family_text)
        .ok_or_else(|| format!("unknown_family={family_text}"))?;
    let header = ShardHeader {
        geometry_name,
        family: family.to_string(),
        total_threads: reader.take_parsed("total_threads")?,
        probe_threads: reader.take_parsed("probe_threads")?,
        local_constants_checked: reader.take_parsed("local_constants_checked")?,
    };
    let seconds = reader.take_parsed("seconds")?;
    let unit_area_devices: usize = reader.take_parsed("unit_area_devices")?;
    let unit_area_slots_per_device = (0..unit_area_devices)
        .map(|_| reader.take_parsed::<u64>("unit_area_slots"))
        .collect::<Result<Vec<u64>, String>>()?;
    let anchors = FirstFileAnchors {
        open_segment_start_slot: reader.take_optional("open_segment_start_slot")?,
        closed_free_slots_in_the_open_segment_on_device_zero: reader
            .take_optional("closed_free_slots_in_the_open_segment_on_device_zero")?,
        first_data_unit_slot: reader.take_optional("first_data_unit_slot")?,
    };
    for expected in TRAJECTORY_NAMES {
        let found = reader.take("trajectory_name")?;
        if found != expected {
            return Err(format!("trajectory_name={found} expected={expected}"));
        }
    }
    let lines = reader.take_texts("product_lines", "product_line")?;
    let tally = take_family_tally(&mut reader)?;
    reader.finish()?;
    let outcome = FamilyOutcome {
        family,
        lines,
        tally,
        anchors,
        seconds,
        unit_area_slots_per_device,
        trajectory_names: TRAJECTORY_NAMES,
    };
    Ok((header, outcome))
}

/// 片文件（键是文件名）合成产物的全部行：全部片要齐（[`ALL_GEOMETRIES`] × [`families_of_the_geometry`]），每片装的格与文件名对得上，
/// 几片的总线程数与本地常量回比个数一致、探测线程数等于 [`probe_threads_of_the_geometry`]，不许有多出来的片；产物经 [`reading_product_lines`] 出。
fn merge_shard_texts(texts: &BTreeMap<String, String>) -> Result<Vec<String>, String> {
    let mut cells = Vec::new();
    let mut expected_names = BTreeSet::new();
    let mut common_threads_and_constants: Option<(usize, usize)> = None;
    for geometry in ALL_GEOMETRIES {
        let mut outcomes = Vec::new();
        for family in families_of_the_geometry(geometry, None) {
            let name = shard_file_name(geometry, family);
            let text = texts
                .get(&name)
                .ok_or_else(|| format!("missing_shard={name}"))?;
            let (header, outcome) =
                parse_shard(text).map_err(|problem| format!("shard={name} {problem}"))?;
            if header.geometry_name != geometry.name || header.family != family {
                return Err(format!(
                    "shard={name} holds_geometry={} holds_family={}",
                    header.geometry_name, header.family
                ));
            }
            let this_shard = (header.total_threads, header.local_constants_checked);
            let common = *common_threads_and_constants.get_or_insert(this_shard);
            if common != this_shard {
                return Err(format!(
                    "shard={name} total_threads_and_local_constants={this_shard:?} others={common:?}"
                ));
            }
            let expected_probe_threads =
                probe_threads_of_the_geometry(geometry, header.total_threads);
            if header.probe_threads != expected_probe_threads {
                return Err(format!(
                    "shard={name} probe_threads={} expected={expected_probe_threads}",
                    header.probe_threads
                ));
            }
            expected_names.insert(name);
            outcomes.push(outcome);
        }
        cells.push(GeometryCellOutcome { geometry, outcomes });
    }
    if let Some(unexpected) = texts.keys().find(|name| !expected_names.contains(*name)) {
        return Err(format!("unexpected_shard={unexpected}"));
    }
    let (_, local_constants_checked) =
        common_threads_and_constants.expect("ALL_GEOMETRIES 非空，至少读过一片");
    Ok(reading_product_lines(local_constants_checked, &cells))
}

/// 片文件原子落盘：先写同目录的 `<路径>.partial-<进程号>`、刷盘，再换名成 `<路径>`；跑到一半被打断的片不会以 `.shard` 结尾留下。
fn write_shard_atomically(path: &str, text: &str) {
    let temporary = format!("{path}.partial-{}", std::process::id());
    let written = std::fs::File::create(&temporary)
        .and_then(|mut file| {
            use std::io::Write;
            file.write_all(text.as_bytes())?;
            file.sync_all()
        })
        .and_then(|()| std::fs::rename(&temporary, path));
    if let Err(error) = written {
        // 删临时文件失败也照停：停机行里已经写明是哪一片没写成，留下的临时文件不以 `.shard` 结尾，合并不认它。
        let _ = std::fs::remove_file(&temporary);
        stop_with("shard_write_failed", &format!("path={path} error={error}"));
    }
}

fn reading(options: &[String]) {
    let options = parse_reading_options(options);
    let local_constants_checked = local_constants_against_the_implementation();
    let geometries: Vec<Geometry> = match &options.geometry {
        Some(name) => vec![geometry_named(name)],
        None => ALL_GEOMETRIES.to_vec(),
    };
    let total_threads = total_threads();
    let mut cells = Vec::new();
    for geometry in geometries {
        let families = families_of_the_geometry(geometry, options.family.as_deref());
        let outcomes = run_geometry_cell(geometry, &families, total_threads);
        cells.push(GeometryCellOutcome { geometry, outcomes });
    }
    if let Some(path) = &options.shard_output {
        let cell = cells.pop().expect("一片只有一个几何格");
        let outcome = cell.outcomes.into_iter().next().expect("一片只有一族");
        let header = ShardHeader {
            geometry_name: cell.geometry.name.to_string(),
            family: outcome.family.to_string(),
            total_threads,
            probe_threads: probe_threads_of_the_geometry(cell.geometry, total_threads),
            local_constants_checked,
        };
        write_shard_atomically(path, &shard_text(&header, &outcome));
        print_product_line(
            &ProductLine::new("shard_written")
                .field("shard", shard_file_name(cell.geometry, outcome.family))
                .field("states", outcome.tally.states)
                .field("probes_run", outcome.tally.probes_run)
                .field("seconds", outcome.seconds)
                .finish(),
        );
        return;
    }
    for line in reading_product_lines(local_constants_checked, &cells) {
        print_product_line(&line);
    }
}

/// 列出全部片：一行一片「片文件名 TAB 几何名 TAB 族名 TAB 这一片占的线程数」，次序就是合并的次序。
/// 占的线程数：带探测的族是 [`probe_threads_of_the_geometry`]（按 `E180_THREADS` 或登记的每台默认重活线程数算），长挂载族不探测记 1；长跑脚本按它往一台上并排放片。
fn list_shards() {
    for line in shard_list_lines(total_threads()) {
        println!("{line}");
    }
}

/// [`list_shards`] 打的行（单测数片数、钉每片的线程数）。
fn shard_list_lines(total_threads: usize) -> Vec<String> {
    let mut lines = Vec::new();
    for geometry in ALL_GEOMETRIES {
        for family in families_of_the_geometry(geometry, None) {
            let threads_of_the_shard = if family == LONG_MOUNT_FAMILY {
                1
            } else {
                probe_threads_of_the_geometry(geometry, total_threads)
            };
            lines.push(format!(
                "{}\t{}\t{family}\t{threads_of_the_shard}",
                shard_file_name(geometry, family),
                geometry.name
            ));
        }
    }
    lines
}

/// `merge <片目录>`：读目录里全部 `*.shard`（别的文件不看），经 [`merge_shard_texts`] 合成产物打到标准输出；片不齐、对不上就停机。
fn merge(arguments: &[String]) {
    let [directory] = arguments else {
        stop_with("bad_option", "usage=merge_<shard_directory>")
    };
    let entries = std::fs::read_dir(directory).unwrap_or_else(|error| {
        stop_with(
            "shard_directory_unreadable",
            &format!("directory={directory} error={error}"),
        )
    });
    let mut texts = BTreeMap::new();
    for entry in entries {
        let entry = entry.unwrap_or_else(|error| {
            stop_with(
                "shard_directory_unreadable",
                &format!("directory={directory} error={error}"),
            )
        });
        let name = entry
            .file_name()
            .into_string()
            .unwrap_or_else(|raw| stop_with("shard_name_is_not_utf8", &format!("name={raw:?}")));
        if !name.ends_with(".shard") {
            continue;
        }
        let text = std::fs::read_to_string(entry.path()).unwrap_or_else(|error| {
            stop_with("shard_unreadable", &format!("shard={name} error={error}"))
        });
        texts.insert(name, text);
    }
    match merge_shard_texts(&texts) {
        Ok(lines) => {
            for line in lines {
                print_product_line(&line);
            }
        }
        Err(problem) => stop_with("shard_merge_refused", &problem),
    }
}

/// 开跑前量一小段外推挂钟（登记 5.6）：默认几何上量 20 次探测与 20 次恢复，再在一个有分量的状态上量一次完整的 T 搜索。不进产物（打标准错误）。
fn calibrate() {
    let geometry = geometry_named("default_geometry");
    let build_pool_started = Instant::now();
    let (mut pool, _) = build_pool(geometry);
    eprintln!(
        "E180-calibration build_pool_seconds={:.2}",
        build_pool_started.elapsed().as_secs_f64()
    );
    let probe_threads = total_threads();
    let change_started = Instant::now();
    let outcome = apply_change(&mut pool, ChangeKind::SequentialWriteOfUnits(5000));
    eprintln!(
        "E180-calibration change_sequential_write_of_5000_units_seconds={:.2} outcome={}",
        change_started.elapsed().as_secs_f64(),
        outcome.label()
    );
    let read_state_started = Instant::now();
    let state = read_state(&pool);
    eprintln!(
        "E180-calibration read_state_seconds={:.3}",
        read_state_started.elapsed().as_secs_f64()
    );
    let stripped = strip_bytes_the_probe_does_not_need(&mut pool.session);
    let probes_started = Instant::now();
    let verdicts: Vec<ProbeVerdict> = {
        let prober = StateProber {
            pool: &pool,
            probe_threads,
            probes_run: AtomicU64::new(0),
        };
        (0..20).map(|_| prober.probe_one(2000)).collect()
    };
    eprintln!(
        "E180-calibration twenty_serial_probes_of_2000_units_seconds={:.2} first_verdict={:?}",
        probes_started.elapsed().as_secs_f64(),
        verdicts[0]
    );
    let search_started = Instant::now();
    let mut model = JournalModel::NOT_LEARNED_YET;
    let writable = {
        let prober = StateProber {
            pool: &pool,
            probe_threads,
            probes_run: AtomicU64::new(0),
        };
        let writable = find_writable_units(
            &prober,
            upper_bound_units_by_the_space_conditions(&pool, &state),
            &mut model,
            &LowerBoundTightening::OnlyWhereAnArmJudgementCanChange {
                units_asked_for_by_the_arms: state.units_asked_for_by_every_arm(),
            },
        );
        eprintln!(
            "E180-calibration probes_in_one_search={}",
            prober.probes_run.load(Ordering::Relaxed)
        );
        writable
    };
    eprintln!(
        "E180-calibration one_full_search_seconds={:.2} result={writable:?} journal_model={model:?}",
        search_started.elapsed().as_secs_f64()
    );
    restore_stripped_bytes(&mut pool.session, stripped);
    let mounts_started = Instant::now();
    for _ in 0..20 {
        mount_again(&mut pool);
    }
    eprintln!(
        "E180-calibration twenty_writable_mounts_seconds={:.2}",
        mounts_started.elapsed().as_secs_f64()
    );
}

fn main() {
    let arguments: Vec<String> = std::env::args().skip(1).collect();
    match arguments.first().map(String::as_str) {
        Some("reading") => reading(&arguments[1..]),
        Some("list-shards") => list_shards(),
        Some("merge") => merge(&arguments[1..]),
        Some("calibrate") => calibrate(),
        Some(other) => stop_with("unknown_mode", &format!("mode={other}")),
        None => stop_with("missing_mode", "usage=reading|list-shards|merge|calibrate"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn block_of(byte: u8, sectors: usize) -> Vec<u8> {
        vec![byte; sectors * SECTOR_BYTES]
    }

    fn read_back(disk: &Disk<'_>, offset: u64, sectors: usize) -> Vec<u8> {
        let mut buffer = vec![0u8; sectors * SECTOR_BYTES];
        disk.read_at(DeviceOffsetInBytes(offset), &mut buffer)
            .expect("读得出");
        buffer
    }

    #[test]
    fn local_constants_match_the_implementation_values() {
        assert_eq!(local_constants_against_the_implementation(), 15);
    }

    #[test]
    fn a_blank_disk_reads_zero_and_keeps_what_was_written() {
        let mut disk = Disk::blank(1 << 20);
        disk.write_at(
            DeviceOffsetInBytes(1024),
            &block_of(7, 2),
            WriteDurability::Plain,
        )
        .expect("写得进");
        assert_eq!(read_back(&disk, 0, 2), block_of(0, 2));
        assert_eq!(read_back(&disk, 1024, 2), block_of(7, 2));
        assert_eq!(disk.own.sectors.len(), 2, "只存写过的非零扇区");
    }

    #[test]
    fn a_base_disk_does_not_store_all_zero_sectors() {
        let mut disk = Disk::blank(1 << 20);
        disk.write_at(
            DeviceOffsetInBytes(0),
            &block_of(0, 4),
            WriteDurability::Plain,
        )
        .expect("写得进");
        assert!(disk.own.sectors.is_empty());
    }

    #[test]
    fn an_overlay_write_does_not_reach_the_base_and_reads_back_over_it() {
        let mut base = Disk::blank(1 << 20);
        base.write_at(
            DeviceOffsetInBytes(0),
            &block_of(1, 4),
            WriteDurability::Plain,
        )
        .expect("写得进");
        let mut overlay = base.overlay(DataWritePolicy::AllowDataWrites);
        overlay
            .write_at(
                DeviceOffsetInBytes(512),
                &block_of(9, 1),
                WriteDurability::Plain,
            )
            .expect("写得进");
        assert_eq!(
            read_back(&overlay, 0, 1),
            block_of(1, 1),
            "没写的扇区读到底下基底"
        );
        assert_eq!(
            read_back(&overlay, 512, 1),
            block_of(9, 1),
            "写过的扇区读到叠层"
        );
        assert_eq!(read_back(&base, 512, 1), block_of(1, 1), "基底一个字节没动");
    }

    #[test]
    fn an_overlay_zero_fill_hides_the_base_bytes_without_touching_them() {
        let mut base = Disk::blank(1 << 20);
        base.write_at(
            DeviceOffsetInBytes(0),
            &block_of(5, 8),
            WriteDurability::Plain,
        )
        .expect("写得进");
        let mut overlay = base.overlay(DataWritePolicy::AllowDataWrites);
        overlay
            .write_zeroes_at(DeviceOffsetInBytes(1024), 1024)
            .expect("清零");
        assert_eq!(read_back(&overlay, 1024, 2), block_of(0, 2));
        assert_eq!(read_back(&overlay, 0, 2), block_of(5, 2));
        assert_eq!(read_back(&base, 1024, 2), block_of(5, 2));
    }

    #[test]
    fn the_first_data_unit_write_is_refused_and_noted_but_other_writes_pass() {
        let base = Disk::blank(1 << 20);
        let mut overlay = base.overlay(DataWritePolicy::FailTheFirstDataWriteAndNoteIt);
        let mut data_unit = vec![0u8; usize::try_from(DATA_UNIT_BYTES).expect("32768")];
        data_unit[..4].copy_from_slice(&UNIT_MAGIC_BYTES);
        data_unit[UNIT_CLASS_BYTE_INDEX] = UNIT_CLASS_DATA_CODE;
        let mut index_node = data_unit.clone();
        index_node[UNIT_CLASS_BYTE_INDEX] = 2;
        overlay
            .write_at(DeviceOffsetInBytes(0), &index_node, WriteDurability::Plain)
            .expect("索引节点的写放行");
        assert!(!overlay.data_write_reached);
        assert!(overlay
            .write_at(
                DeviceOffsetInBytes(32768),
                &data_unit,
                WriteDurability::Plain
            )
            .is_err());
        assert!(overlay.data_write_reached);
    }

    #[test]
    fn reads_inside_an_unreadable_range_fail_and_reads_outside_succeed() {
        let mut disk = Disk::blank(1 << 20);
        disk.unreadable_byte_ranges.push(1024..2048);
        let mut buffer = vec![0u8; SECTOR_BYTES];
        assert!(disk
            .read_at(DeviceOffsetInBytes(1536), &mut buffer)
            .is_err());
        assert!(disk.read_at(DeviceOffsetInBytes(2048), &mut buffer).is_ok());
    }

    #[test]
    fn the_journal_hint_lists_record_starts_the_overlay_wrote_and_hides_zeroed_ones() {
        let mut base = Disk::blank(1 << 24);
        let ring_start = 4096u64;
        base.write_at(
            DeviceOffsetInBytes(ring_start),
            &block_of(1, 8),
            WriteDurability::Plain,
        )
        .expect("写得进");
        base.write_at(
            DeviceOffsetInBytes(ring_start + 4096),
            &block_of(1, 8),
            WriteDurability::Plain,
        )
        .expect("写得进");
        let mut overlay = base.overlay(DataWritePolicy::AllowDataWrites);
        overlay
            .write_at(
                DeviceOffsetInBytes(ring_start + 8192),
                &block_of(2, 8),
                WriteDurability::Plain,
            )
            .expect("写得进");
        overlay
            .write_zeroes_at(DeviceOffsetInBytes(ring_start), 4096)
            .expect("清零");
        let hint = overlay
            .journal_record_offsets_hint(DeviceOffsetInBytes(ring_start), 3 * 4096)
            .expect("内存盘答得出");
        assert_eq!(
            hint,
            vec![
                DeviceOffsetInBytes(ring_start + 4096),
                DeviceOffsetInBytes(ring_start + 8192)
            ]
        );
    }

    #[test]
    fn the_non_data_demand_bound_adds_the_tree_nodes_the_fixed_slots_and_the_checkpoint_cost() {
        // n ≤ 1：下段那一项取 0，inode 树的内部层数这里也是 0，只剩上段叶 1、inode 叶容器 2 与检查点 10。
        assert_eq!(non_data_demand_upper_bound_slots(1, 0, 10), 1 + 2 + 10);
        // n = 144：下段叶 1 个，没有更高层。
        assert_eq!(
            non_data_demand_upper_bound_slots(144, 0, 10),
            1 + 1 + 2 + 10
        );
        // n = 145：下段叶 2 个，内部常规层 个。
        assert_eq!(
            non_data_demand_upper_bound_slots(145, 0, 10),
            (2 + 1) + 1 + 2 + 10
        );
        // n = 144 × 147 + 1：叶 148 个，内部大负载层 个，再 1 个根。
        assert_eq!(
            non_data_demand_upper_bound_slots(144 * 147 + 1, 1, 0),
            (148 + 2 + 1) + 1 + 2 + 1
        );
    }

    #[test]
    fn the_w4_search_returns_the_largest_n_whose_demand_fits() {
        let available = 1000;
        let n = largest_units_satisfying_the_w4_demand(available, 0, 10);
        assert!(2 * n + non_data_demand_upper_bound_slots(n, 0, 10) <= available);
        assert!(2 * (n + 1) + non_data_demand_upper_bound_slots(n + 1, 0, 10) > available);
        assert_eq!(
            largest_units_satisfying_the_w4_demand(5, 0, 10),
            0,
            "连固定部分都装不下"
        );
    }

    #[test]
    fn the_judgement_marks_an_over_report_by_the_units_asked_for() {
        let exact = WritableUnits::Exact {
            units: 10,
            non_monotone_probes_seen: false,
        };
        assert_eq!(
            judge(10 * DATA_UNIT_PAYLOAD_BYTES, &exact),
            Judgement::DoesNotOverReport { under_bytes: 0 }
        );
        assert_eq!(
            judge(10 * DATA_UNIT_PAYLOAD_BYTES + 1, &exact),
            Judgement::OverReports { over_bytes: 1 }
        );
        assert_eq!(
            judge(0, &exact),
            Judgement::DoesNotOverReport {
                under_bytes: 10 * DATA_UNIT_PAYLOAD_BYTES
            }
        );
        let censored = WritableUnits::AtLeastBecauseOfTheJournalCap {
            lower_bound_units: 10,
            lower_bound_was_tightened: true,
        };
        assert_eq!(
            judge(10 * DATA_UNIT_PAYLOAD_BYTES, &censored),
            Judgement::CertainlyDoesNotOverReportBelowTheLowerBound
        );
        assert_eq!(
            judge(10 * DATA_UNIT_PAYLOAD_BYTES + 1, &censored),
            Judgement::Undetermined
        );
        assert_eq!(
            judge(1, &WritableUnits::Unreadable("x".to_string())),
            Judgement::Undetermined
        );
    }

    #[test]
    fn the_interior_points_split_the_gap_into_distinct_points_strictly_inside() {
        let points = interior_points(0, 100, 4);
        assert_eq!(points, vec![20, 40, 60, 80]);
        let narrow = interior_points(10, 12, 6);
        assert_eq!(narrow, vec![11]);
    }

    #[test]
    fn the_threshold_between_two_readings_flips_the_judgement_only_when_they_differ() {
        assert_eq!(threshold_between_flips_the_judgement(0, 5), Some(true));
        assert_eq!(threshold_between_flips_the_judgement(5, 0), Some(true));
        assert_eq!(threshold_between_flips_the_judgement(3, 3), None);
    }

    #[test]
    fn the_trajectory_counts_peak_and_positive_states_after_the_first_state_only() {
        let mut trajectory = TrajectoryTally::default();
        for value in [100, -5, 7, 3, -1] {
            trajectory.record(value);
        }
        assert_eq!(trajectory.first_value, 100);
        assert_eq!(trajectory.peak_after_the_first, Some(7));
        assert_eq!(trajectory.positive_states_after_the_first, 2);
        assert_eq!(trajectory.last_value, -1);
    }
}

#[cfg(test)]
mod arm_and_residual_tests {
    use super::*;
    use singlefs_core::allocator::{AllocationRecord, DeviceFreeMap, PlacementRefusal};

    fn device_reading(
        capacity_slots: u64,
        live_slots: u64,
        available_slots: i128,
    ) -> DeviceReading {
        DeviceReading {
            capacity_slots,
            live_slots,
            live_slots_in_memory_account: live_slots,
            available_bytes: available_slots * i128::from(SLOT_BYTES),
            common_commitments_bytes: 0,
            ring_window_slots: 0,
            unblocked_slots: 0,
            overlap_slots_allocated_and_isolated: 0,
            isolated_slots: 0,
            allocated_slots: live_slots,
            deferred_slots: 0,
            closed_free_slots_in_cluster_segments: 0,
        }
    }

    fn state_reading(
        devices: Vec<DeviceReading>,
        data_units: u64,
        cost: u64,
        lag_slots: u64,
    ) -> StateReading {
        StateReading {
            devices,
            data_units,
            checkpoint_cost_blocks: cost,
            lag_slots,
            lag_unattributed_slots: 0,
            oldest_valid_root_txg: 0,
            fourth_newest_nonempty_valid_root_txg: None,
            cluster_segment_count: 1,
            inode_tree_internal_levels: 0,
            records_agree_with_the_disk: true,
            decomposition_holds: true,
            available_is_whole_slots: true,
        }
    }

    fn arm_bytes(reading: &StateReading, arm: CandidateArm) -> i128 {
        reading.raw_bytes_of(Arm::Candidate(arm))
    }

    #[test]
    fn the_df_literal_arms_subtract_the_reserve_the_residue_and_the_lag_and_differ_by_the_live_metadata(
    ) {
        // 两盘各容量 1000 槽、活 100 槽，第一个文件 10 个数据单元（每盘 20 槽），c_max = 2，滞后 6。
        let reading = state_reading(
            vec![
                device_reading(1000, 100, 700),
                device_reading(1000, 100, 700),
            ],
            10,
            2,
            6,
        );
        // 没用过的加已释放的 = (900 + 900) ÷ 2 = 900；保留池 10 + 7 × 2 = 24；残留 5 + 6 × 2 = 17。
        let slot_bytes = i128::from(SLOT_BYTES);
        assert_eq!(
            arm_bytes(&reading, CandidateArm::DfLiteralWithoutLiveMetadata),
            slot_bytes * (900 - 24 - 17 - 6)
        );
        // 活元数据 = 每盘 (100 − 20) 槽，两盘之和 ÷ 2 = 80。
        assert_eq!(
            arm_bytes(&reading, CandidateArm::DfLiteralWithAllLiveMetadata),
            slot_bytes * (900 + 80 - 24 - 17 - 6)
        );
    }

    /// 按需收紧看的「各臂要的单元数」与判定用的是同一个门槛：下界恰等于它判一定不多报，少一个单元判未定（登记第十二节修订 4）。
    #[test]
    fn the_units_asked_for_by_every_arm_are_the_threshold_the_judgement_uses() {
        let reading = state_reading(
            vec![
                device_reading(1000, 100, 700),
                device_reading(1000, 100, 700),
            ],
            10,
            2,
            6,
        );
        let asks = reading.units_asked_for_by_every_arm();
        assert_eq!(asks.len(), all_arms().len());
        // 钉绝对值：W2 按槽字节 700 × 16384 = 11468800 字节 ⇒ ⌈÷ 32634⌉ = 352；W2 按载荷 350 个单元；W3 两盘 1400 ÷ 2 槽同 W2 按槽；阴性对照 0。
        assert_eq!(asks[2], 352);
        assert_eq!(asks[3], 350);
        assert_eq!(asks[4], 352);
        assert_eq!(asks[7], 0);
        for (arm, asked) in all_arms().into_iter().zip(&asks) {
            let reported = reported_bytes(reading.raw_bytes_of(arm));
            let censored_at =
                |lower_bound_units: u64| WritableUnits::AtLeastBecauseOfTheJournalCap {
                    lower_bound_units,
                    lower_bound_was_tightened: true,
                };
            assert_eq!(
                judge(reported, &censored_at(*asked)),
                Judgement::CertainlyDoesNotOverReportBelowTheLowerBound,
                "{}",
                arm.label()
            );
            if *asked > 0 {
                assert_eq!(
                    judge(reported, &censored_at(asked - 1)),
                    Judgement::Undetermined,
                    "{}",
                    arm.label()
                );
            }
        }
    }

    #[test]
    fn the_available_arms_take_the_minimum_over_devices_not_the_maximum() {
        let reading = state_reading(
            vec![
                device_reading(1000, 100, 700),
                device_reading(800, 100, 500),
            ],
            10,
            2,
            0,
        );
        let slot_bytes = i128::from(SLOT_BYTES);
        let payload_bytes = i128::from(DATA_UNIT_PAYLOAD_BYTES);
        assert_eq!(
            arm_bytes(
                &reading,
                CandidateArm::AvailableMinimumOverDevicesInSlotBytes
            ),
            slot_bytes * 500
        );
        assert_eq!(
            arm_bytes(
                &reading,
                CandidateArm::AvailableMinimumOverDevicesInPayloadBytes
            ),
            payload_bytes * 250
        );
        // 两盘相加 ÷ 副本数 = (700 + 500) ÷ 2。
        assert_eq!(
            arm_bytes(
                &reading,
                CandidateArm::AvailableSummedOverDevicesDividedByReplicas
            ),
            slot_bytes * 600
        );
    }

    #[test]
    fn the_payload_arm_counts_whole_units_of_the_registered_payload_size() {
        let reading = state_reading(
            vec![device_reading(1000, 0, 7), device_reading(1000, 0, 9)],
            0,
            0,
            0,
        );
        // 最小 7 槽 ⇒ 3 个整单元，每个 32634 字节。
        assert_eq!(
            arm_bytes(
                &reading,
                CandidateArm::AvailableMinimumOverDevicesInPayloadBytes
            ),
            3 * 32634
        );
    }

    #[test]
    fn the_w4_arm_leaves_room_for_the_non_data_demand_of_the_write() {
        let reading = state_reading(
            vec![device_reading(1000, 0, 1000), device_reading(1000, 0, 1000)],
            0,
            10,
            0,
        );
        let units = arm_bytes(
            &reading,
            CandidateArm::AvailableMinimumLessTheNonDataDemandOfTheWrite,
        ) / i128::from(DATA_UNIT_PAYLOAD_BYTES);
        let units = u64::try_from(units).expect("单元数");
        assert!(units < 500, "比不扣非数据需求的 500 个单元少");
        assert_eq!(units, largest_units_satisfying_the_w4_demand(1000, 0, 10));
    }

    #[test]
    fn the_always_over_reporting_control_reports_the_whole_free_pool_without_dividing_by_replicas()
    {
        let reading = state_reading(
            vec![device_reading(1000, 100, 0), device_reading(1000, 100, 0)],
            0,
            0,
            0,
        );
        assert_eq!(
            reading.raw_bytes_of(Arm::PositiveControlAlwaysOverReports),
            i128::from(SLOT_BYTES) * 1800
        );
        assert_eq!(reading.raw_bytes_of(Arm::NegativeControlReportsZero), 0);
    }

    #[test]
    fn a_negative_arm_value_is_reported_as_zero_bytes() {
        assert_eq!(reported_bytes(-5), 0);
        assert_eq!(reported_bytes(7), 7);
    }

    #[test]
    fn the_residuals_use_the_overlap_and_held_terms_as_registered() {
        // 可用 + Cm = 100，没挡的 = 90，扣住账 = 0，重叠 = 0 ⇒ 三个残差都是 10（扣住的槽没人记账）。
        let plain = residuals_of(100, 90, 0, 0);
        assert_eq!(
            plain.rewritten_check_residual_with_the_held_and_overlap_terms,
            10
        );
        assert_eq!(plain.original_formula_residual, 10);
        // 重叠 3 槽：原式残差照旧 10，改写检查残差的两项式 13，去掉重叠项的对照 10。
        let overlapping = residuals_of(100, 90, 3, 0);
        assert_eq!(
            overlapping.rewritten_check_residual_with_the_held_and_overlap_terms,
            13
        );
        assert_eq!(
            overlapping.rewritten_check_residual_without_the_overlap_term,
            10
        );
        assert_eq!(overlapping.original_formula_residual, 10);
        // 扣住账 10 槽、没有重叠：改写检查残差恰为 0，原式残差仍是 10。
        let held = residuals_of(100, 90, 0, 10);
        assert_eq!(
            held.rewritten_check_residual_with_the_held_and_overlap_terms,
            0
        );
        assert_eq!(held.original_formula_residual, 10);
    }

    fn record(
        device: u32,
        slot: u64,
        span_slots: u16,
        generation: u64,
        is_released: bool,
    ) -> AllocationRecord {
        AllocationRecord {
            device: DeviceIdentity(device),
            slot: SlotNumber(slot),
            span_slots,
            generation: singlefs_core::address::CheckpointTxg(generation),
            is_released,
            quarantine_mark:
                singlefs_core::allocator::AllocationRecordQuarantineMark::NotQuarantined,
        }
    }

    #[test]
    fn the_lag_counts_releases_by_user_changes_above_the_fourth_newest_nonempty_root_only() {
        let mut log = PublishLog::default();
        log.user_change_txgs.extend([10, 11, 12, 13, 14, 15]);
        log.empty_publish_txgs.extend([16, 17]);
        // 环里有效根 10..=17；非空的 10..=15，从新到旧第 4 个是 12；最旧有效根 10；门槛 = 12。
        let valid_roots: Vec<u64> = (10..=17).collect();
        let records = vec![
            record(0, 100, 2, 11, true), // 释放代 11 ≤ 门槛：不滞后
            record(1, 100, 2, 11, true),
            record(0, 200, 2, 13, true), // 用户改动放的，高于门槛：滞后
            record(1, 200, 2, 13, true),
            record(0, 300, 1, 16, true), // 空发布放的：不算
            record(1, 300, 1, 16, true),
            record(0, 400, 2, 14, false), // 仍分配：不算
            record(0, 500, 4, 99, true),  // 释放代不在任何日志里：记未归属
        ];
        let reading = lag_reading_of(&records, &valid_roots, &log, 2);
        assert_eq!(reading.fourth_newest_nonempty_valid_root_txg, Some(12));
        assert_eq!(reading.oldest_valid_root_txg, 10);
        assert_eq!(reading.lag_slots, 2, "两盘各 2 槽，除以盘数");
        assert_eq!(reading.unattributed_slots, 4);
    }

    #[test]
    fn the_lag_threshold_falls_back_to_the_oldest_valid_root_when_fewer_than_four_roots_are_nonempty(
    ) {
        let mut log = PublishLog::default();
        log.user_change_txgs.extend([5, 6]);
        let reading = lag_reading_of(&[record(0, 1, 2, 5, true)], &[3, 4, 5, 6], &log, 2);
        assert_eq!(reading.fourth_newest_nonempty_valid_root_txg, None);
        assert_eq!(reading.lag_slots, 1, "门槛取最旧有效根 3，释放代 5 高于它");
    }

    #[test]
    fn the_closed_free_slots_count_only_the_segments_this_mount_opened() {
        let mut map = DeviceFreeMap::new(DeviceIdentity(0), FOUR_GIBIBYTES);
        let opened_by_this_mount: BTreeSet<SlotNumber> = [SlotNumber(50240)].into_iter().collect();
        assert_eq!(
            closed_free_slots_in_segments(&map, &opened_by_this_mount),
            64
        );
        map.mark_allocated(SlotNumber(50241), 2);
        assert_eq!(
            closed_free_slots_in_segments(&map, &opened_by_this_mount),
            62
        );
        // 另一段里有分配、但不是这次挂载开的：不算。
        map.mark_allocated(SlotNumber(50304), 2);
        assert_eq!(
            closed_free_slots_in_segments(&map, &opened_by_this_mount),
            62
        );
        let two_segments: BTreeSet<SlotNumber> =
            [SlotNumber(50240), SlotNumber(50304)].into_iter().collect();
        assert_eq!(closed_free_slots_in_segments(&map, &two_segments), 62 + 62);
    }

    #[test]
    fn the_probe_result_is_classified_by_what_reached_the_device_and_why_it_was_refused() {
        let space_admission = PublishError::SpaceAdmissionRefused(
            singlefs_core::admission::AdmissionRefusedOnSomeDevices {
                short_devices: Vec::new(),
            },
        );
        let refused_for_placement = PublishError::PlacementRefused {
            unit: TransactionUnit::InodeRoot,
            refusal: PlacementRefusal::NoFreeSlotOnAnyDevice,
        };
        let publish_refused = |cause| UserChangeRefused::Publish {
            cause,
            floor_raises: Vec::new(),
            versions_published: Vec::new(),
        };
        assert_eq!(
            classify_probe_result(Err(publish_refused(space_admission)), false),
            ProbeVerdict::RefusedForSpace
        );
        assert_eq!(
            classify_probe_result(Err(publish_refused(refused_for_placement)), false),
            ProbeVerdict::RefusedForSpace
        );
        assert_eq!(
            classify_probe_result(
                Err(publish_refused(
                    PublishError::JournalRecordsOfThePublishExceedTheLimit {
                        records_of_the_publish: 70_000,
                        record_limit: 65_536,
                    }
                )),
                false
            ),
            ProbeVerdict::JournalLimit {
                records_of_the_publish: 70_000,
                record_limit: 65_536
            }
        );
        // 数据单元的写到达了设备：不论会话把它当成什么错，都是放行。
        let device_table_error = UserChangeRefused::DeviceTableOtherThanTheOneOfTheMount {
            device_table_of_the_mount: Vec::new(),
            device_table_handed_in: Vec::new(),
        };
        assert_eq!(
            classify_probe_result(Err(device_table_error), true),
            ProbeVerdict::Admitted
        );
        let device_table_error_without_a_data_write =
            UserChangeRefused::DeviceTableOtherThanTheOneOfTheMount {
                device_table_of_the_mount: Vec::new(),
                device_table_handed_in: Vec::new(),
            };
        assert!(matches!(
            classify_probe_result(Err(device_table_error_without_a_data_write), false),
            ProbeVerdict::Other(_)
        ));
    }
}

#[cfg(test)]
mod search_tests {
    use super::*;

    /// 假探测器：落点那一道、journal 记录上限、式子那一判各有一个界，次序同 `prepare_the_version_publish`（先落点、后记录数、最后式子）。
    struct FakeProber {
        landing_units: u64,
        record_limit: u64,
        /// 记录数 = 单元数 + 固定几条 + 每多少个单元再多一条（让「固定几条」随单元数缓慢地变）。
        fixed_extra_records: u64,
        units_per_additional_record: u64,
        writable_units: u64,
        parallel_probes: usize,
        /// 探了几个点（按需收紧与一律收紧的探测次数对着报）。
        probes_run: std::cell::Cell<u64>,
    }

    impl FakeProber {
        fn records_of(&self, units: u64) -> u64 {
            units + self.fixed_extra_records + units / self.units_per_additional_record
        }
    }

    impl UnitsProbe for FakeProber {
        fn probe_in_parallel(&self, unit_counts: &[u64]) -> Vec<ProbeVerdict> {
            self.probes_run
                .set(self.probes_run.get() + u64::try_from(unit_counts.len()).expect("探测数"));
            unit_counts
                .iter()
                .map(|units| {
                    if *units > self.landing_units {
                        ProbeVerdict::RefusedForSpace
                    } else if self.records_of(*units) > self.record_limit {
                        ProbeVerdict::JournalLimit {
                            records_of_the_publish: self.records_of(*units),
                            record_limit: self.record_limit,
                        }
                    } else if *units > self.writable_units {
                        ProbeVerdict::RefusedForSpace
                    } else {
                        ProbeVerdict::Admitted
                    }
                })
                .collect()
        }

        fn parallel_probe_count(&self) -> usize {
            self.parallel_probes
        }
    }

    fn prober(writable_units: u64, landing_units: u64, parallel_probes: usize) -> FakeProber {
        FakeProber {
            landing_units,
            record_limit: 65_536,
            fixed_extra_records: 3,
            units_per_additional_record: 1000,
            writable_units,
            parallel_probes,
            probes_run: std::cell::Cell::new(0),
        }
    }

    #[test]
    fn the_search_finds_the_exact_boundary_below_the_journal_cap_for_every_parallelism() {
        for parallel_probes in [1, 2, 6, 31] {
            for writable_units in [0, 1, 2, 777, 1234, 40_000] {
                let mut model = JournalModel::NOT_LEARNED_YET;
                let found = find_writable_units(
                    &prober(writable_units, 100_000, parallel_probes),
                    writable_units + 500,
                    &mut model,
                    &LowerBoundTightening::AtEveryCensoredState,
                );
                assert_eq!(
                    found,
                    WritableUnits::Exact {
                        units: writable_units,
                        non_monotone_probes_seen: false
                    },
                    "writable={writable_units} parallel={parallel_probes}"
                );
            }
        }
    }

    #[test]
    fn a_pool_roomier_than_the_journal_cap_is_censored_at_the_largest_units_that_still_pass_the_record_limit(
    ) {
        // 固定那几条记录随单元数变（每 1000 个单元多一条）：在 12 万个单元处学来的上限偏低，下界得往上收到记录上限边上。
        // 每多少个单元多一条取几种，其中一种大到不变；并行度取 1、2、6、31，覆盖步长翻倍那一支的各种点数。
        for units_per_additional_record in [1000, 100, 7000, u64::MAX] {
            for parallel_probes in [1, 2, 6, 31] {
                let mut fake = prober(200_000, 200_000, parallel_probes);
                fake.units_per_additional_record = units_per_additional_record;
                let mut model = JournalModel::NOT_LEARNED_YET;
                let found = find_writable_units(
                    &fake,
                    130_000,
                    &mut model,
                    &LowerBoundTightening::AtEveryCensoredState,
                );
                match found {
                    WritableUnits::AtLeastBecauseOfTheJournalCap {
                        lower_bound_units, ..
                    } => {
                        assert!(fake.records_of(lower_bound_units) <= fake.record_limit);
                        assert!(
                            fake.records_of(lower_bound_units + 1) > fake.record_limit,
                            "下界已经收到记录上限边上：再多一个单元就被记录数拒 \
                             (每多少单元多一条={units_per_additional_record} 并行={parallel_probes} 下界={lower_bound_units})"
                        );
                    }
                    other @ (WritableUnits::Exact { .. } | WritableUnits::Unreadable(_)) => {
                        panic!("该是截尾，得到 {other:?}")
                    }
                }
                assert!(model.largest_units_below_the_record_limit.is_some());
            }
        }
    }

    #[test]
    fn a_space_bound_that_is_not_a_bound_is_widened_to_the_journal_cap_when_the_cap_is_known() {
        let fake = prober(30_000, 100_000, 4);
        let mut model = JournalModel {
            largest_units_below_the_record_limit: Some(64_000),
            records_beyond_one_per_unit: Some(3),
            record_limit: Some(65_536),
        };
        // 给的上界 10_000 偏低（第一个探测点 10_001 其实放行）：装置往记录上限那一边找，仍找得到 30_000。
        let found = find_writable_units(
            &fake,
            10_000,
            &mut model,
            &LowerBoundTightening::AtEveryCensoredState,
        );
        assert_eq!(
            found,
            WritableUnits::Exact {
                units: 30_000,
                non_monotone_probes_seen: false
            }
        );
    }

    #[test]
    fn a_state_where_the_landing_gate_refuses_before_the_record_limit_is_found_exactly_at_zero() {
        let fake = prober(0, 0, 3);
        let mut model = JournalModel::NOT_LEARNED_YET;
        assert_eq!(
            find_writable_units(
                &fake,
                0,
                &mut model,
                &LowerBoundTightening::AtEveryCensoredState
            ),
            WritableUnits::Exact {
                units: 0,
                non_monotone_probes_seen: false
            }
        );
    }

    #[test]
    fn the_search_does_not_report_the_boundary_one_unit_too_high() {
        // 二分多一单元的变异要在这里红：T 必须恰是最大放行的单元数。
        let fake = prober(5_000, 100_000, 6);
        let mut model = JournalModel::NOT_LEARNED_YET;
        let found = find_writable_units(
            &fake,
            5_100,
            &mut model,
            &LowerBoundTightening::AtEveryCensoredState,
        );
        let WritableUnits::Exact { units, .. } = found else {
            panic!("该是精确值")
        };
        assert_eq!(fake.probe_one(units), ProbeVerdict::Admitted);
        assert_eq!(fake.probe_one(units + 1), ProbeVerdict::RefusedForSpace);
    }

    #[test]
    fn the_journal_model_learns_the_cap_from_the_records_beyond_one_per_unit() {
        let mut model = JournalModel::NOT_LEARNED_YET;
        model.learn_from(120_000, 120_003, 65_536);
        assert_eq!(model.records_beyond_one_per_unit, Some(3));
        assert_eq!(model.largest_units_below_the_record_limit, Some(65_533));
        assert_eq!(model.record_limit, Some(65_536));
    }

    /// 一串假状态里的一个（同一个模型从头学起，像一族历史那样一个接一个探）：可写的单元数、空间上界、各臂要的单元数。
    struct FakeState {
        name: &'static str,
        writable_units: u64,
        space_bound: u64,
        units_asked_for_by_the_arms: Vec<u64>,
    }

    /// 记录上限边上的单元数：再多一个就被记录数拒。照假探测器的记录数逐个往上数，不经搜索。
    fn units_at_the_record_limit_edge(fake: &FakeProber) -> u64 {
        let mut units = 0;
        while fake.records_of(units + 1) <= fake.record_limit {
            units += 1;
        }
        units
    }

    /// 在远处（[`PROBE_FAR_ABOVE_ANY_JOURNAL_CAP_UNITS`]）学一次记录上限之后，记录上限处放行的那个点：上限减去远处那一次多出来的记录数。
    fn admitted_point_learned_far_above_the_cap(fake: &FakeProber) -> u64 {
        let far = PROBE_FAR_ABOVE_ANY_JOURNAL_CAP_UNITS;
        fake.record_limit - (fake.records_of(far) - far)
    }

    /// 各臂要的单元数取在：放行点及以下、放行点之上一个、记录上限边上两侧、记录上限之上，外加一个可写量落在放行点与边之间的截尾状态
    /// 和一个不截尾的状态。
    fn fake_states(admitted_point: u64, edge: u64, record_limit: u64) -> Vec<FakeState> {
        let between = admitted_point + (edge - admitted_point) / 2;
        vec![
            FakeState {
                name: "censored_every_ask_at_or_below_the_admitted_point",
                writable_units: 200_000,
                space_bound: 130_000,
                units_asked_for_by_the_arms: vec![0, admitted_point / 2, admitted_point],
            },
            FakeState {
                name: "censored_an_ask_one_above_the_admitted_point",
                writable_units: 200_000,
                space_bound: 130_000,
                units_asked_for_by_the_arms: vec![0, admitted_point + 1],
            },
            FakeState {
                name: "censored_asks_on_both_sides_of_the_record_limit_edge",
                writable_units: 200_000,
                space_bound: 130_000,
                units_asked_for_by_the_arms: vec![edge, edge + 1, record_limit],
            },
            FakeState {
                name: "censored_asks_above_the_record_limit_only",
                writable_units: 200_000,
                space_bound: 130_000,
                units_asked_for_by_the_arms: vec![record_limit + 1, 10 * record_limit],
            },
            FakeState {
                name: "exact_below_the_cap",
                writable_units: 40_000,
                space_bound: 40_500,
                units_asked_for_by_the_arms: vec![39_999, 40_000, 40_001, admitted_point + 1],
            },
            FakeState {
                name: "censored_writable_between_the_admitted_point_and_the_edge",
                writable_units: between,
                space_bound: 130_000,
                units_asked_for_by_the_arms: vec![between, between + 1],
            },
            FakeState {
                name: "censored_after_an_exact_state_every_ask_at_or_below_the_admitted_point",
                writable_units: 200_000,
                space_bound: 130_000,
                units_asked_for_by_the_arms: vec![admitted_point],
            },
        ]
    }

    /// 一个状态上量到的：T 的结果、每条臂的判定、探了几个点。
    struct MeasuredFakeState {
        writable: WritableUnits,
        judgements: Vec<Judgement>,
        probes_run: u64,
    }

    fn tightening_at_every_censored_state(_state: &FakeState) -> LowerBoundTightening {
        LowerBoundTightening::AtEveryCensoredState
    }

    fn tightening_only_where_an_arm_judgement_can_change(
        state: &FakeState,
    ) -> LowerBoundTightening {
        LowerBoundTightening::OnlyWhereAnArmJudgementCanChange {
            units_asked_for_by_the_arms: state.units_asked_for_by_the_arms.clone(),
        }
    }

    /// 照一族历史的样子一个状态接一个探（模型跨状态带着），截尾的状态收不收紧照 `tightening_of`。
    fn measure_fake_states(
        states: &[FakeState],
        units_per_additional_record: u64,
        parallel_probes: usize,
        tightening_of: fn(&FakeState) -> LowerBoundTightening,
    ) -> Vec<MeasuredFakeState> {
        let mut model = JournalModel::NOT_LEARNED_YET;
        states
            .iter()
            .map(|state| {
                let mut fake = prober(state.writable_units, 200_000, parallel_probes);
                fake.units_per_additional_record = units_per_additional_record;
                let tightening = tightening_of(state);
                let writable =
                    find_writable_units(&fake, state.space_bound, &mut model, &tightening);
                let judgements = state
                    .units_asked_for_by_the_arms
                    .iter()
                    .map(|units| judge(units * DATA_UNIT_PAYLOAD_BYTES, &writable))
                    .collect();
                MeasuredFakeState {
                    writable,
                    judgements,
                    probes_run: fake.probes_run.get(),
                }
            })
            .collect()
    }

    /// 登记第十二节修订 4：截尾状态上只在某条臂要的单元数落在（放行点, 记录上限] 里时收紧，每条臂在每个状态上的判定与一律收紧逐项相同；
    /// 探测次数逐状态打出来（`--nocapture` 看），按需收紧的不多于一律收紧，合计严格少。
    #[test]
    fn tightening_only_where_an_arm_judgement_can_change_judges_every_arm_as_tightening_at_every_censored_state(
    ) {
        for units_per_additional_record in [1000, 100, 7000, u64::MAX] {
            for parallel_probes in [1, 2, 6, 31] {
                let mut reference = prober(200_000, 200_000, parallel_probes);
                reference.units_per_additional_record = units_per_additional_record;
                let edge = units_at_the_record_limit_edge(&reference);
                let admitted_point = admitted_point_learned_far_above_the_cap(&reference);
                let states = fake_states(admitted_point, edge, reference.record_limit);
                let at_every = measure_fake_states(
                    &states,
                    units_per_additional_record,
                    parallel_probes,
                    tightening_at_every_censored_state,
                );
                let on_demand = measure_fake_states(
                    &states,
                    units_per_additional_record,
                    parallel_probes,
                    tightening_only_where_an_arm_judgement_can_change,
                );
                let case = format!(
                    "每多少单元多一条={units_per_additional_record} 并行={parallel_probes} 放行点={admitted_point} 边={edge}"
                );
                let mut probes_at_every_in_total = 0;
                let mut probes_on_demand_in_total = 0;
                for ((state, every), demand) in states.iter().zip(&at_every).zip(&on_demand) {
                    assert_eq!(
                        demand.judgements, every.judgements,
                        "{case} 状态={}：按需收紧与一律收紧给的判定不同",
                        state.name
                    );
                    assert!(
                        demand.probes_run <= every.probes_run,
                        "{case} 状态={}：按需收紧探得比一律收紧多（{} > {}）",
                        state.name,
                        demand.probes_run,
                        every.probes_run
                    );
                    println!(
                        "E180-tightening-probes units_per_additional_record={units_per_additional_record} \
                         parallel={parallel_probes} state={} at_every_censored_state={} \
                         only_where_an_arm_judgement_can_change={} result_at_every={:?} result_on_demand={:?}",
                        state.name, every.probes_run, demand.probes_run, every.writable, demand.writable
                    );
                    probes_at_every_in_total += every.probes_run;
                    probes_on_demand_in_total += demand.probes_run;
                }
                assert!(
                    probes_on_demand_in_total < probes_at_every_in_total,
                    "{case}：合计探测次数按需收紧 {probes_on_demand_in_total} 应严格少于一律收紧 {probes_at_every_in_total}"
                );
            }
        }
    }

    /// 钉绝对值：哪一个状态收、哪一个不收，不收的下界停在放行点、探测次数是学上限 1 次加放行点 1 次（第一个状态）或只有放行点 1 次。
    #[test]
    fn tightening_on_demand_leaves_the_lower_bound_at_the_admitted_point_unless_an_ask_lies_above_it_within_the_record_limit(
    ) {
        for units_per_additional_record in [1000, 100, 7000, u64::MAX] {
            for parallel_probes in [1, 2, 6, 31] {
                let mut reference = prober(200_000, 200_000, parallel_probes);
                reference.units_per_additional_record = units_per_additional_record;
                let edge = units_at_the_record_limit_edge(&reference);
                let admitted_point = admitted_point_learned_far_above_the_cap(&reference);
                let states = fake_states(admitted_point, edge, reference.record_limit);
                let measured = measure_fake_states(
                    &states,
                    units_per_additional_record,
                    parallel_probes,
                    tightening_only_where_an_arm_judgement_can_change,
                );
                let case = format!(
                    "每多少单元多一条={units_per_additional_record} 并行={parallel_probes}"
                );
                let untightened = WritableUnits::AtLeastBecauseOfTheJournalCap {
                    lower_bound_units: admitted_point,
                    lower_bound_was_tightened: false,
                };
                assert_eq!(measured[0].writable, untightened, "{case} 第一个状态不收");
                assert_eq!(
                    measured[0].probes_run, 2,
                    "{case} 第一个状态：学上限 1 次 + 放行点 1 次"
                );
                assert_eq!(
                    measured[3].writable, untightened,
                    "{case} 只有记录上限之上的臂：不收"
                );
                assert_eq!(
                    measured[3].probes_run, 1,
                    "{case} 学过上限之后不收的状态只探放行点 1 次"
                );
                assert_eq!(
                    measured[4].writable,
                    WritableUnits::Exact {
                        units: 40_000,
                        non_monotone_probes_seen: false
                    },
                    "{case} 不截尾的状态照旧取精确值"
                );
                assert_eq!(
                    measured[6].writable, untightened,
                    "{case} 精确状态之后再截尾：不收"
                );
                assert_eq!(measured[6].probes_run, 1, "{case}");
                assert_eq!(
                    measured[2].writable,
                    WritableUnits::AtLeastBecauseOfTheJournalCap {
                        lower_bound_units: edge,
                        lower_bound_was_tightened: true,
                    },
                    "{case} 有臂要边上的单元数：收到边上"
                );
                assert_eq!(
                    measured[2].judgements,
                    vec![
                        Judgement::CertainlyDoesNotOverReportBelowTheLowerBound,
                        Judgement::Undetermined,
                        Judgement::Undetermined,
                    ],
                    "{case} 边上的判一定不多报，边之上一个与记录上限判未定"
                );
                assert_eq!(
                    measured[3].judgements,
                    vec![Judgement::Undetermined, Judgement::Undetermined],
                    "{case}"
                );
                // 可写量在放行点与边之间：收紧碰到被空间拒的点就停（下界照样成立），停在放行点与可写量之间，不一定贴着可写量。
                let between = admitted_point + (edge - admitted_point) / 2;
                match &measured[5].writable {
                    WritableUnits::AtLeastBecauseOfTheJournalCap {
                        lower_bound_units,
                        lower_bound_was_tightened: true,
                    } => assert!(
                        (admitted_point..=between).contains(lower_bound_units),
                        "{case} 下界 {lower_bound_units} 该在放行点 {admitted_point} 与可写量 {between} 之间"
                    ),
                    other @ (WritableUnits::Exact { .. }
                    | WritableUnits::AtLeastBecauseOfTheJournalCap { .. }
                    | WritableUnits::Unreadable(_)) => {
                        panic!("{case} 可写量在放行点与边之间：该是收过的截尾，得到 {other:?}")
                    }
                }
            }
        }
    }

    /// 记录上限还没学到时（模型没有上限），按需收紧照收：判不了收紧改不改判定，就按改算。
    #[test]
    fn tightening_on_demand_counts_an_unknown_record_limit_as_one_that_could_change_a_judgement() {
        assert!(tightening_can_change_an_arm_judgement(10, None, &[11]));
        assert!(!tightening_can_change_an_arm_judgement(10, None, &[10]));
        assert!(tightening_can_change_an_arm_judgement(10, Some(11), &[11]));
        assert!(!tightening_can_change_an_arm_judgement(10, Some(11), &[12]));
        assert!(!tightening_can_change_an_arm_judgement(10, Some(11), &[]));
    }
}

#[cfg(test)]
mod shard_tests {
    use super::*;

    /// 一个几何格里一族的假结果：每个字段都取非零、随（几何、族）变的值，字符串里带制表符、换行与反斜杠（片文件要转义的三样）。
    fn synthetic_outcome(geometry: Geometry, family: &'static str) -> FamilyOutcome {
        let geometry_index = ALL_GEOMETRIES
            .iter()
            .position(|known| known.name == geometry.name)
            .expect("几何在全部几何里");
        let family_index = families_of_the_geometry(geometry, None)
            .iter()
            .position(|known| *known == family)
            .expect("族在这个几何格的族里");
        let seed = u64::try_from(geometry_index * 10 + family_index).expect("种子") + 1;
        let state_name = format!("{}/{family}/{seed}", geometry.name);
        let mut tally = FamilyTally::new(2);
        tally.states = 100 + seed;
        tally.exact_states = 60 + seed;
        tally.censored_states = 30;
        tally.unreadable_states = 10;
        if family != LONG_MOUNT_FAMILY {
            for (arm_index, arm) in all_arms().into_iter().enumerate() {
                let arm_number = u64::try_from(arm_index).expect("臂号");
                tally.arms.insert(
                    arm,
                    ArmTally {
                        over_states: seed + arm_number,
                        not_over_states: 2 * seed + arm_number,
                        not_over_below_lower_bound_states: arm_number,
                        undetermined_states: seed % 3,
                        max_over_bytes: 32634 * (seed + arm_number),
                        max_over_at: state_name.clone(),
                        max_under_bytes: 16384 * seed,
                        max_under_at: format!("{state_name}\twith a tab"),
                        sum_under_bytes: u128::from(seed) << 70,
                        under_states: seed,
                    },
                );
            }
        }
        for (device_index, residual) in tally.residuals.iter_mut().enumerate() {
            let device_number = i128::try_from(device_index).expect("盘号");
            *residual = ResidualTally {
                states: 100 + seed,
                rewritten_check_residual_nonzero_states: seed,
                original_formula_residual_nonzero_states: seed + 1,
                maximum_absolute_rewritten_check_residual: -7 - device_number,
                maximum_absolute_original_formula_residual: i128::from(seed) << 80,
                states_with_overlap: 3,
                states_with_held_ledger: 0,
                control_without_overlap_term_stayed_zero: u64::try_from(device_index)
                    .expect("盘号"),
                first_state_with_a_nonzero_rewritten_check_residual: format!(
                    "{state_name}\nwith a line break"
                ),
                first_state_with_a_nonzero_original_formula_residual: String::new(),
            };
        }
        for (trajectory_index, trajectories) in [
            &mut tally.closed_minus_ring_window,
            &mut tally.closed_alone,
            &mut tally.closed_minus_all_common_commitments,
        ]
        .into_iter()
        .enumerate()
        {
            let offset = i128::try_from(trajectory_index).expect("轨迹号");
            for (device_index, trajectory) in trajectories.iter_mut().enumerate() {
                if device_index == 1 && trajectory_index == 2 {
                    continue; // 留一条一个状态都没记的轨迹：峰值是 None。
                }
                for value in [52, -3 - offset, i128::from(seed), 0] {
                    trajectory.record(value);
                }
            }
        }
        tally.control_failures = vec![format!("{state_name}:back\\slash")];
        tally.shortcut_pairs_checked = seed;
        tally.shortcut_differences = Vec::new();
        tally.records_disagree_with_the_disk_states = 1;
        tally.decomposition_failure_states = 2;
        tally.not_whole_slot_available_states = 3;
        tally.lag_unattributed_states = 4;
        tally.non_monotone_states = 5;
        tally.probes_run = 1000 * seed;
        tally.stop_clause_held_ledger_mismatch_states = 6;
        tally.unmount_pushes_checked = 7;
        tally.unmount_pushes_differing = vec!["none".to_string(), String::new()];
        tally.panics = Vec::new();
        FamilyOutcome {
            family,
            lines: vec![
                ProductLine::new("state")
                    .field("geometry", geometry.name)
                    .field("family", family)
                    .field("index", 0)
                    .finish(),
                ProductLine::new("state")
                    .field("geometry", geometry.name)
                    .field("family", family)
                    .field("index", seed)
                    .field("reason", "a\\b\tc")
                    .finish(),
            ],
            tally,
            anchors: FirstFileAnchors {
                open_segment_start_slot: Some(50_240),
                closed_free_slots_in_the_open_segment_on_device_zero: Some(52 + seed % 2),
                first_data_unit_slot: None,
            },
            seconds: 3600 + seed,
            unit_area_slots_per_device: vec![
                geometry.unit_area_slots_of_device(0),
                geometry.unit_area_slots_of_device(1),
            ],
            trajectory_names: TRAJECTORY_NAMES,
        }
    }

    const LOCAL_CONSTANTS_CHECKED: usize = 15;

    /// 一次跑全部：每个几何格、每族都是 [`synthetic_outcome`]。
    fn every_cell_at_once() -> Vec<GeometryCellOutcome> {
        ALL_GEOMETRIES
            .iter()
            .map(|geometry| GeometryCellOutcome {
                geometry: *geometry,
                outcomes: families_of_the_geometry(*geometry, None)
                    .into_iter()
                    .map(|family| synthetic_outcome(*geometry, family))
                    .collect(),
            })
            .collect()
    }

    /// 分片跑：每一片的片文件，片头是这一片跑的那一次的总线程数与探测线程数。
    fn shard_texts_of_every_cell(total_threads: usize) -> BTreeMap<String, String> {
        let mut texts = BTreeMap::new();
        for geometry in ALL_GEOMETRIES {
            for family in families_of_the_geometry(geometry, None) {
                let header = ShardHeader {
                    geometry_name: geometry.name.to_string(),
                    family: family.to_string(),
                    total_threads,
                    probe_threads: probe_threads_of_the_geometry(geometry, total_threads),
                    local_constants_checked: LOCAL_CONSTANTS_CHECKED,
                };
                texts.insert(
                    shard_file_name(geometry, family),
                    shard_text(&header, &synthetic_outcome(geometry, family)),
                );
            }
        }
        texts
    }

    fn lines_named(lines: &[String], name: &str) -> usize {
        let prefix = format!("E name={name} ");
        lines
            .iter()
            .filter(|line| line.starts_with(&prefix))
            .count()
    }

    #[test]
    fn merging_the_shards_of_every_cell_gives_the_product_of_running_every_cell_at_once_byte_for_byte(
    ) {
        let at_once = reading_product_lines(LOCAL_CONSTANTS_CHECKED, &every_cell_at_once());
        let merged =
            merge_shard_texts(&shard_texts_of_every_cell(31)).expect("29 片齐了，合并该成");
        assert_eq!(
            merged.join("\n").into_bytes(),
            at_once.join("\n").into_bytes()
        );
        // 钉绝对值：两条路一起错（比如都少了一格）时这几条红。
        assert_eq!(
            at_once.last().expect("产物非空"),
            &format!("E name=done emitted={}", at_once.len())
        );
        assert_eq!(lines_named(&at_once, "geometry"), 5);
        assert_eq!(lines_named(&at_once, "verdict"), 5);
        assert_eq!(lines_named(&at_once, "family_summary"), 29);
        assert_eq!(lines_named(&at_once, "state"), 2 * 29);
        assert_eq!(lines_named(&at_once, "local_constants_checked"), 1);
        assert!(at_once.contains(
            &"E name=state geometry=unequal_devices_geometry family=abandoned_root_and_net_release index=15 reason=a\\b\tc"
                .to_string()
        ));
    }

    #[test]
    fn a_shard_text_parses_back_to_the_same_header_and_family_outcome() {
        let geometry = geometry_named("default_geometry_with_four_slots_per_region");
        let header = ShardHeader {
            geometry_name: geometry.name.to_string(),
            family: "abandoned_root_and_net_release".to_string(),
            total_threads: 31,
            probe_threads: 5,
            local_constants_checked: LOCAL_CONSTANTS_CHECKED,
        };
        let text = shard_text(
            &header,
            &synthetic_outcome(geometry, "abandoned_root_and_net_release"),
        );
        assert_eq!(
            text.lines().filter(|line| line.contains('\t')).count(),
            text.lines().count()
        );
        assert_eq!(
            parse_shard(&text).expect("刚写的片读得回"),
            (
                header,
                synthetic_outcome(geometry, "abandoned_root_and_net_release")
            )
        );
    }

    #[test]
    fn a_merge_refuses_a_missing_shard() {
        let mut texts = shard_texts_of_every_cell(31);
        texts.remove("narrow_devices_geometry-many_mounts.shard");
        let refusal = merge_shard_texts(&texts).expect_err("少一片不合并");
        assert!(
            refusal.contains("missing_shard=narrow_devices_geometry-many_mounts.shard"),
            "{refusal}"
        );
    }

    #[test]
    fn a_merge_refuses_a_shard_whose_done_line_is_missing() {
        let mut texts = shard_texts_of_every_cell(31);
        let text = texts
            .get_mut("default_geometry-overwrite.shard")
            .expect("片在");
        let without_the_done_line: Vec<&str> = text
            .lines()
            .filter(|line| !line.starts_with("shard_done\t"))
            .collect();
        *text = format!("{}\n", without_the_done_line.join("\n"));
        let refusal = merge_shard_texts(&texts).expect_err("没有完成标记的片不合并");
        assert!(
            refusal.contains(
                "shard=default_geometry-overwrite.shard last_line_is_not_the_done_marker"
            ),
            "{refusal}"
        );
    }

    #[test]
    fn a_merge_refuses_a_shard_cut_short_even_if_it_ends_with_a_done_line() {
        let mut texts = shard_texts_of_every_cell(31);
        let text = texts.get_mut("default_geometry-fill.shard").expect("片在");
        let lines: Vec<&str> = text.lines().collect();
        let mut kept: Vec<String> = lines[..lines.len() / 2]
            .iter()
            .map(|line| (*line).to_string())
            .collect();
        kept.push(lines[lines.len() - 1].to_string());
        *text = format!("{}\n", kept.join("\n"));
        let refusal = merge_shard_texts(&texts).expect_err("截短的片不合并");
        assert!(
            refusal.contains("shard=default_geometry-fill.shard last_line_is_not_the_done_marker"),
            "完成标记里的行数对不上就拒，不等读到一半才发现缺键：{refusal}"
        );
    }

    #[test]
    fn a_merge_refuses_an_unexpected_shard_and_a_shard_holding_another_family() {
        let mut texts = shard_texts_of_every_cell(31);
        let extra = texts["default_geometry-fill.shard"].clone();
        texts.insert("default_geometry-hx.shard".to_string(), extra);
        let extra_shard_refusal = merge_shard_texts(&texts).expect_err("多出来的片不合并");
        assert!(
            extra_shard_refusal.contains("unexpected_shard=default_geometry-hx.shard"),
            "{extra_shard_refusal}"
        );
        let mut swapped = shard_texts_of_every_cell(31);
        let other_family = swapped["default_geometry-delete_then_write.shard"].clone();
        swapped.insert("default_geometry-overwrite.shard".to_string(), other_family);
        let swapped_shard_refusal =
            merge_shard_texts(&swapped).expect_err("片名与片里装的族对不上不合并");
        assert!(
            swapped_shard_refusal.contains(
                "shard=default_geometry-overwrite.shard holds_geometry=default_geometry holds_family=delete_then_write"
            ),
            "{swapped_shard_refusal}"
        );
    }

    #[test]
    fn a_merge_refuses_shards_run_with_different_total_threads() {
        let mut texts = shard_texts_of_every_cell(31);
        let other_threads = shard_texts_of_every_cell(30);
        texts.insert(
            "default_geometry_with_sixteen_slots_per_region-delete_then_write.shard".to_string(),
            other_threads["default_geometry_with_sixteen_slots_per_region-delete_then_write.shard"]
                .clone(),
        );
        let refusal =
            merge_shard_texts(&texts).expect_err("总线程数不同的片不合并（探测点随线程数变）");
        assert!(
            refusal.contains(
                "shard=default_geometry_with_sixteen_slots_per_region-delete_then_write.shard total_threads_and_local_constants=(30, 15)"
            ),
            "{refusal}"
        );
    }

    #[test]
    fn a_merge_refuses_a_shard_whose_probe_threads_are_not_those_of_its_geometry_cell() {
        let mut texts = shard_texts_of_every_cell(31);
        let geometry = geometry_named("narrow_devices_geometry");
        let header = ShardHeader {
            geometry_name: geometry.name.to_string(),
            family: "fill".to_string(),
            total_threads: 31,
            probe_threads: 6,
            local_constants_checked: LOCAL_CONSTANTS_CHECKED,
        };
        texts.insert(
            "narrow_devices_geometry-fill.shard".to_string(),
            shard_text(&header, &synthetic_outcome(geometry, "fill")),
        );
        let refusal = merge_shard_texts(&texts).expect_err("探测线程数不是这一格的不合并");
        assert!(
            refusal.contains("shard=narrow_devices_geometry-fill.shard probe_threads=6 expected=5"),
            "{refusal}"
        );
    }

    /// 片的清单：5 个几何 × 5 个带探测的族，加除 unequal_devices_geometry 之外 4 个几何的 long_mount，共 29 片，片名不重；
    /// 一片的探测线程数与一次跑全部时那一格里的那一族相同（按这个几何格的全部族数分，不按这一次跑几族分）。
    #[test]
    fn the_shard_list_has_twenty_nine_shards_each_with_the_probe_threads_of_its_whole_geometry_cell(
    ) {
        let lines = shard_list_lines(31);
        assert_eq!(lines.len(), 29);
        let names: BTreeSet<&str> = lines
            .iter()
            .map(|line| line.split('\t').next().expect("片名"))
            .collect();
        assert_eq!(names.len(), 29);
        assert!(lines.contains(
            &"default_geometry-overwrite.shard\tdefault_geometry\toverwrite\t5".to_string()
        ));
        assert!(lines.contains(&"unequal_devices_geometry-abandoned_root_and_net_release.shard\tunequal_devices_geometry\tabandoned_root_and_net_release\t6".to_string()));
        assert!(lines.contains(
            &"default_geometry_with_sixteen_slots_per_region-long_mount.shard\tdefault_geometry_with_sixteen_slots_per_region\tlong_mount\t1"
                .to_string()
        ));
        assert!(!names.contains("unequal_devices_geometry-long_mount.shard"));
        for geometry in ALL_GEOMETRIES {
            assert_eq!(
                probe_threads_of_the_geometry(geometry, 31),
                31 / families_of_the_geometry(geometry, None).len(),
                "{}",
                geometry.name
            );
        }
        assert_eq!(
            probe_threads_of_the_geometry(geometry_named("default_geometry"), 3),
            1
        );
    }

    #[test]
    fn a_shard_value_with_backslashes_tabs_and_line_breaks_survives_escaping() {
        for text in ["", "none", "a\\b", "\t", "\n\r", "\\t", "末尾一个反斜杠\\"] {
            assert_eq!(
                unescape_shard_value(&escape_shard_value(text)).expect("转义过的读得回"),
                text
            );
            assert!(!escape_shard_value(text).contains(['\t', '\n', '\r']));
        }
        assert!(unescape_shard_value("\\x").is_err());
        assert!(unescape_shard_value("\\").is_err());
    }
}
