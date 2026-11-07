//! 几份场景文件（`tests/common_images_for_the_given_states_engine/<测试文件名>.rs`）共用的常量、类型与搭建函数，只留这一份：
//! 给定状态用例（`crash_given_states_of_the_images_harness_scenarios_hand_out.rs`）把全部场景文件编进同一个测试目标，各抄一份就是同一个目标里的撞名。
//! 引这一份的场景文件用 `crate::common_shared_by_harness_scenarios::…`；引那份场景文件的测试根文件各 `mod` 这一份一次。
//! 这一份只依赖 `crate::common` 与库，不引池级 checker。
#![allow(dead_code, reason = "引它的测试目标各只用其中几样")]

use crate::common::{
    parameters, BuiltPool, Recorded, CONTENT_BYTES_OF_PUBLISH_B, CONTENT_BYTES_OF_PUBLISH_C,
    FIXED_WRITE_TIME_SECONDS, IMAGE_BYTES,
};
use crate::common_images_for_the_given_states_engine::{PoolCheckerExpectation, PoolCheckerImages};
use singlefs_core::address::{
    CheckpointTxg, DeviceIdentity, FileOffsetInBytes, InodeNumber, InstanceGeneration,
};
use singlefs_core::allocation_record_tree::AllocationRecordTreeNodePosition;
use singlefs_core::block_device::PhysicalBlockSizeInBytes;
use singlefs_core::container_tree::ContainerTreeNodeEntries;
use singlefs_core::lineage_tree::LineageContainerCapacity;
use singlefs_core::mount::{
    mount_writable, roll_back_by_a_forward_publish, RollbackError, RollbackTarget, RolledBack,
};
use singlefs_core::mounted_read::mount_read_only;
use singlefs_core::mounted_session::MountedSession;
use singlefs_core::recovery::{rebuild_version, RebuiltVersion};
use singlefs_core::retirement_tree::RetirementContainerCapacity;
use singlefs_core::root_ring::target_for_publish;
use singlefs_core::tombstone_tree::TombstoneContainerCapacity;
use singlefs_core::transaction::{
    acquire_instance, publish_overwrite, publish_sequential_write_of_an_inode, FirstFile,
    PoolWriter, TransactionOutput, TransactionUnit,
};
use singlefs_core::unit::data_unit_payload_capacity;
use singlefs_core::unlink::{
    DeletionSwitches, UnlinkBatchDataUnitsCappedForTests, UnlinkBatchSize,
};
use singlefs_verification::fault_injection::{
    FaultCounting, FaultDeviceSelector, FaultOccurrence, FaultPlacement, FaultSchedule,
    InjectedFault, SharedFaultPlan,
};
use singlefs_verification::history::HistoryDeviceWidth;
use singlefs_verification::memory_pool::{MemoryPool, SparseBlockDevice};
use singlefs_verification::{RecordingBlockDevice, SharedStream};

/// 两块盘的身份。
pub const DISKS: [DeviceIdentity; 2] = [DeviceIdentity(0), DeviceIdentity(1)];

/// 两块录制着的内存稀疏盘。
pub type Devices = Vec<(DeviceIdentity, RecordingBlockDevice<SparseBlockDevice>)>;

/// 两块不录制的内存稀疏盘。
pub type SparseDevices = Vec<(DeviceIdentity, SparseBlockDevice)>;

/// 这个进程（新池新建文件与它之后在同一个进程里的发布）的实例代号。
pub const INSTANCE_OF_THE_PROCESS: InstanceGeneration = InstanceGeneration(1);

/// 判的几条：记账与遍历（I-3.1、I-3.11）、被引用单元读回来对不对（I-2.1、I-7.4、I-4.8）。
pub const INVARIANTS_THE_REREAD_KEEPS_GREEN: [&str; 5] =
    ["I-3.1", "I-3.11", "I-2.1", "I-7.4", "I-4.8"];

/// 一次不注错的再挂里，根环每个槽第几次被读是重建分配器读根环那一遍（从 1 数；见引它的两份场景文件的文件头）。
pub const READ_OF_EACH_ROOT_RING_SLOT_BY_THE_ALLOCATOR_REBUILD: u64 = 6;

/// 一片实例表里数据行的上限：370 条记录减去链指针那一条。
pub const ROWS_PER_PAGE: usize = 369;

/// 内存稀疏盘按 512 字节扇区存：物理块宽 512。
pub const SECTOR_PHYSICAL_BLOCK_SIZE: PhysicalBlockSizeInBytes = PhysicalBlockSizeInBytes(512);

/// 洞那一槽里留着的上一圈的根。
pub const TXG_OF_THE_STALE_ROOT_IN_THE_HOLE: u64 = 7;

/// 根槽没落的那次发布的 txg：它的槽里留着上一圈 txg 7 的根（24 槽的环，31 − 24 = 7）。
pub const TXG_WHOSE_ROOT_SLOT_DID_NOT_PERSIST: u64 = 31;

/// 一次覆盖写的内容长度：一个数据单元装得下。
pub const OVERWRITE_BYTES_WITHIN_ONE_DATA_UNIT: usize = 3000;

/// X 那一版之后、正常退出之前的空发布次数（攻方 `AFTER_X` 的默认值）。
pub const EMPTY_PUBLISHES_AFTER_X: u64 = 2;

/// 建 inode、按 inode 号写、unlink 那几版的写入时间：比池里第一个文件晚。
pub const WRITE_TIME_SECONDS_AFTER_THE_FIRST_FILE: u64 = FIXED_WRITE_TIME_SECONDS + 600;

/// 长 `length` 字节、按 `seed` 起的一段内容（第 `index` 个字节是 `(index × 7 + seed) mod 253`）。
pub fn content_seeded_by(length: usize, seed: usize) -> Vec<u8> {
    (0..length)
        .map(|index| u8::try_from((index * 7 + seed) % 253).expect("小于 256"))
        .collect()
}

/// txg 为 `checkpoint_txg` 的那次覆盖写的内容：一个数据单元装得下，每个字节都是 txg 对 251 取余。
pub fn content_of_txg(checkpoint_txg: u64) -> Vec<u8> {
    vec![
        u8::try_from(checkpoint_txg % 251).expect("小于 256");
        OVERWRITE_BYTES_WITHIN_ONE_DATA_UNIT
    ]
}

/// 分配记录树根之下 (层级, 盘, 同盘同层序号) 那个节点的角色（D8（核心索引结构） 已定项 14：按绝对槽号按位置寻址）。
pub fn allocation_record_tree_node(
    level: u8,
    device: u32,
    index_in_device: u64,
) -> TransactionUnit {
    TransactionUnit::AllocationTreeNodeBelowTheRoot(AllocationRecordTreeNodePosition {
        level,
        device: DeviceIdentity(device),
        index_in_device,
    })
}

/// 只读挂载这张镜像、打开 `inode`、从偏移 0 读整个文件。
pub fn content_read_back(image: &MemoryPool, inode: u64) -> Vec<u8> {
    let mounted = mount_read_only(image).expect("只读挂载");
    let file = mounted
        .mounted
        .open_file(image, InodeNumber(inode))
        .unwrap_or_else(|failure| panic!("打开 inode {inode}：{failure:?}"));
    let size = file.inode_record().size;
    file.read_at(image, FileOffsetInBytes(0), size)
        .unwrap_or_else(|failure| panic!("读 inode {inode} 的 {size} 字节：{failure:?}"))
        .bytes
}

/// 连着 `count` 次「取号之后崩溃」：每次取一个新号写进两块盘的系统配置，写行那次发布没发出去。
pub fn crash_right_after_acquisition(devices: &mut Devices, count: u32) {
    let publish_parameters = parameters();
    let mut writer = PoolWriter::new(&publish_parameters, devices.as_mut_slice());
    for _ in 0..count {
        acquire_instance(&mut writer).expect("取号");
    }
}

/// 删除批大小 1 个数据单元（只供测试的开关，挂载的开关链装上去，续做照它）。
pub fn deletion_switches_with_batches_of_one() -> DeletionSwitches {
    DeletionSwitches {
        batch_size: UnlinkBatchSize::CappedForTests(
            UnlinkBatchDataUnitsCappedForTests::new(1).expect("1 在 1..=144 里"),
        ),
        tombstone_container_capacity: TombstoneContainerCapacity::FromTheContainerFormat,
        lineage_container_capacity: LineageContainerCapacity::FromTheContainerFormat,
        retirement_container_capacity: RetirementContainerCapacity::FromTheContainerFormat,
        container_tree_node_entries: ContainerTreeNodeEntries::FromTheNodeFormat,
    }
}

/// 交一张池级 checker 一条都不红的镜像，标签是「这是第几张-`what`」。
pub fn hand_out_a_clean_image(images: &mut PoolCheckerImages, image: &MemoryPool, what: &str) {
    let label = format!("{}-{what}", images.handed_out());
    images.hand_out(
        &label,
        || image.clone(),
        PoolCheckerExpectation::no_invariant_violated(),
    );
}

/// 整池从第 `first_failing_write` 次写起每次写都报错的注错计划（掉电那一形：盘停在之前那几个写之后）。
pub fn plan_failing_every_write_from(
    device_width: HistoryDeviceWidth,
    first_failing_write: u64,
) -> SharedFaultPlan {
    SharedFaultPlan::armed(
        device_width.fixed_geometry(),
        FaultSchedule {
            fault: InjectedFault::WriteFails,
            device: FaultDeviceSelector::EveryDevice,
            placement: FaultPlacement::AnyOffset,
            counting: FaultCounting::AcrossThePool,
            occurrence: FaultOccurrence::EveryMatchingCallFromTheNthOnward(first_failing_write),
        },
    )
}

/// 关掉镜像（进程退出）、重开可写挂载，交回接着这次挂载的会话与它的盘。
pub fn session_after_a_writable_remount(
    pool: &mut BuiltPool,
) -> (MountedSession, Vec<(DeviceIdentity, Recorded)>) {
    let mut devices = pool.reopen_recorded();
    let mounted = mount_writable(&parameters(), &mut devices).expect("重开可写挂载");
    let (_, session) = MountedSession::of_the_mount(mounted);
    (session, devices)
}

/// 同一个进程里按 inode 号顺序写 `inode` 整份成 `content`（`inode` 是 1 时就是写第一个文件）。
pub fn write_in_process(pool: &mut BuiltPool, inode: u64, content: &[u8]) {
    let publish_parameters = parameters();
    let previous = pool.output.clone();
    let devices = pool.devices.as_mut().expect("镜像还开着");
    let mut writer = PoolWriter::new(&publish_parameters, devices.as_mut_slice());
    pool.output = publish_sequential_write_of_an_inode(
        &mut writer,
        &mut pool.allocator,
        &previous,
        InodeNumber(inode),
        content,
        WRITE_TIME_SECONDS_AFTER_THE_FIRST_FILE,
        INSTANCE_OF_THE_PROCESS,
    )
    .unwrap_or_else(|refusal| panic!("按 inode 号写 inode {inode}：{refusal:?}"));
}

/// 同一个进程里覆盖写第一个文件成 `content`（写入时间比第一个文件晚 60 秒），交回这一版并换进 `pool.output`。
pub fn overwrite_in_process(
    pool: &mut BuiltPool,
    content: &[u8],
    instance: InstanceGeneration,
) -> TransactionOutput {
    let publish_parameters = parameters();
    let devices = pool.devices.as_mut().expect("镜像还开着");
    let mut writer = PoolWriter::new(&publish_parameters, devices.as_mut_slice());
    let previous = pool.output.clone();
    let output = publish_overwrite(
        &mut writer,
        &mut pool.allocator,
        &previous,
        FirstFile {
            content,
            write_time_seconds: FIXED_WRITE_TIME_SECONDS + 60,
        },
        instance,
    )
    .expect("覆盖写");
    pool.output = output.clone();
    output
}

/// txg 为 `txg` 的那次发布的根落到哪个区域，那个区域所在的盘。
pub fn region_device(txg: u64) -> DeviceIdentity {
    let target = target_for_publish(
        CheckpointTxg(txg),
        parameters().geometry.root_ring_slots_per_region,
    );
    parameters().region_devices[usize::try_from(target.region).expect("区域号")]
}

/// 挂着的时候回退：交回回退的结局，做成时 `pool.output` 已换成回退那次发布之后的一版。
pub fn roll_back(
    pool: &mut BuiltPool,
    target: RollbackTarget,
) -> Result<RolledBack, RollbackError> {
    let rollback_parameters = parameters();
    let devices = pool.devices.as_mut().expect("镜像还开着");
    roll_back_by_a_forward_publish(
        &rollback_parameters,
        devices,
        &mut pool.allocator,
        &mut pool.output,
        target,
    )
}

/// `pool.output` 那一版从盘上重建（有树的一版）。
pub fn rebuilt_from_disk(pool: &BuiltPool) -> TransactionOutput {
    match rebuild_version(
        &pool.memory_pool(),
        &pool.output.root,
        Some(pool.output.record.clone()),
    )
    .expect("从盘上重建")
    {
        RebuiltVersion::WithFile(version) | RebuiltVersion::WithTreesAndNoFile(version) => version,
        RebuiltVersion::WithoutFile => panic!("有树的一版"),
    }
}

/// 两块 4 GiB 内存稀疏盘，写都录进 `stream`。
pub fn sparse_devices(stream: &SharedStream) -> Devices {
    DISKS
        .iter()
        .map(|identity| {
            (
                *identity,
                RecordingBlockDevice::with_shared_stream(
                    *identity,
                    SparseBlockDevice::new(IMAGE_BYTES, PhysicalBlockSizeInBytes(512)),
                    stream.clone(),
                ),
            )
        })
        .collect()
}

/// 定长的伪随机字节流（xorshift64，固定种子）：错位一个字节就对不上。
pub fn pseudo_random_content(length: usize, seed: u64) -> Vec<u8> {
    let mut state = seed;
    (0..length)
        .map(|_| {
            state ^= state << 13;
            state ^= state >> 7;
            state ^= state << 17;
            u8::try_from(state & 0xff).expect("低 8 位")
        })
        .collect()
}

/// `data_units` 个数据单元的伪随机内容（[`pseudo_random_content`]），最后一个单元差 7 个字节不满。
pub fn pseudo_random_content_seven_bytes_short_of_data_units(
    data_units: usize,
    seed: u64,
) -> Vec<u8> {
    pseudo_random_content(data_units * data_unit_payload_capacity() - 7, seed)
}

/// 定长的伪随机字节流（xorshift64，种子取奇数）：`data_units` 个数据单元差 5 个字节写满。
pub fn pseudo_random_content_five_bytes_short_of_data_units(
    data_units: usize,
    seed: u64,
) -> Vec<u8> {
    let mut state = seed | 1;
    (0..data_units * data_unit_payload_capacity() - 5)
        .map(|_| {
            state ^= state << 13;
            state ^= state >> 7;
            state ^= state << 17;
            u8::try_from(state & 0xff).expect("低 8 位")
        })
        .collect()
}

/// 固定脚本里 B 那一次写的内容（[`CONTENT_BYTES_OF_PUBLISH_B`] 字节）。
pub fn second_content() -> Vec<u8> {
    content_seeded_by(CONTENT_BYTES_OF_PUBLISH_B, 3)
}

/// 固定脚本里 C 那一次写的内容（[`CONTENT_BYTES_OF_PUBLISH_C`] 字节）。
pub fn third_content() -> Vec<u8> {
    content_seeded_by(CONTENT_BYTES_OF_PUBLISH_C, 11)
}
