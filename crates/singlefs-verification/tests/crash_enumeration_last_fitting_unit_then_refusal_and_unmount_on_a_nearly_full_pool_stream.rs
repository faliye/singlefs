//! 验证包模块：crash、crash_state_enumeration、writable_mount_phases、crash_library、crash_amplification
//! 贴边池上写最后一个装得下的单元、拒写下一个单元、卸载的流（384 槽小盘）：
//! 第一个文件之后崩了再可写挂载、建一个空的第二个文件，把第一个文件顺序写到「再写第二个文件一个单元恰好还装得下」那么长（这几步在基镜像里，不枚举）；
//! 录的是：第二个文件写一个单元（最后一个装得下的单元，进崩溃域），再写一个单元被空间准入拒（拒之前准入先推了几次抬 F 的空发布，没写数据单元），
//! 卸载（抬 F 发布）。
//! 状态数（录制流闭式计数）写在 `the_recorded_stream_counts_its_crash_states`，全量没判过。
//! 每个崩溃状态跑恢复（看 / 不看 journal）、多版本 oracle、池级 checker 每条已实现的不变量、记录核对器；另挂两个观察者：
//! 只读挂载读回恢复读出的第一个文件（`observe_the_read_only_mount_reading_back_the_recovered_content`），
//! 可写接管、给第二个文件再写一个单元（`observe_the_writable_mount_writing_one_more_unit`）：恢复落到基镜像那一版（写最后一个单元之前）、
//! 写最后一个单元那一版或卸载完成那一版时断言被空间拒（这三版的根都在录流时现取）；落到拒写那一次准入推的抬 F 空发布那几版、
//! 卸载最后一版之前那几版时，写进与被拒按这两类各自照实计数、不断言哪一种：卸载先把新 F 写进系统配置、再发带新 F 的根，
//! 恢复取根上与系统配置里 F 的较大者，同一个根在系统配置那一步落没落盘的两种状态里结局不同，按根分不开
//! （批次 7 调查员在 harness 探针上逐段量过）。
//! 每段都展开、与闭式比，登记成崩溃枚举用例，带断点续跑。

mod common;
mod common_admission_pool;
mod common_crash_points;

use common_admission_pool::{
    content_of, device_bytes_of_the_384_slot_small_devices,
    fixed_geometry_of_the_384_slot_small_devices, parameters_of_the_384_slot_small_devices,
    plain_device, plain_devices_on, DeviceOverASparseImage, PoolUnderTest,
};
use singlefs_core::address::{
    CheckpointTxg, DeviceIdentity, FileOffsetInBytes, InodeNumber, InstanceGeneration,
};
use singlefs_core::block_device::PhysicalBlockSizeInBytes;
use singlefs_core::mount::{mount_writable, unmount, Unmounted};
use singlefs_core::mounted_read::mount_read_only;
use singlefs_core::mounted_session::{MountedSession, UserChange, UserChangeRefused};
use singlefs_core::recovery::{RecoveryOutcome, RecoveryReport};
use singlefs_core::transaction::FIRST_INODE_NUMBER;
use singlefs_core::unit::{data_unit_payload_capacity, parse_data_unit};
use singlefs_verification::crash::{
    crash_state_count_with_torn_in_place_overwrites, full_expansion,
    CrashPointReplayObserverCounts, CrashPointReplaySegmentExpansion, CrashPointReplayTally,
    CrashPointReplayVersionTable,
};
use singlefs_verification::crash_amplification::{
    ImageAfterTheObserver, InvariantsJudgedOnTheImageAfterTheObserver, ObserverDeduplication,
    ReadRangeOnTheCrashImage,
};
use singlefs_verification::crash_library::StageOutcome;
use singlefs_verification::crash_state_enumeration::{
    enumerate_crash_states_with_every_kind_of_observer, CrashPointReplayItem,
    CrashPointReplayObserver, CrashPointReplayObserverHandingBackAnImage,
};
use singlefs_verification::memory_pool::{
    root_identity_of_write, writes_and_segments,
    writes_and_segments_with_stream_indexes_and_entries, CrashImage, MemoryPool, PublishedVersion,
    RetainedWrite, SparseBlockDevice,
};
use singlefs_verification::segments::StepKind;
use singlefs_verification::sha256::{sha256_digest, DIGEST_BYTES};
use singlefs_verification::writable_mount_phases::materialized_crash_image;
use singlefs_verification::{
    RecordedEntrySpan, RecordedPublishEntry, RecordingBlockDevice, SharedStream,
};

/// 这份测试文件的仓内路径：崩溃枚举用例的设点文件。
const PATH_OF_THIS_TEST_FILE: &str =
    "crates/singlefs-verification/tests/crash_enumeration_last_fitting_unit_then_refusal_and_unmount_on_a_nearly_full_pool_stream.rs";

/// 崩溃点重放项名第一个 `::` 之前那一截（来历表里这条流的名字）。
const FLOW_NAME_AT_THE_ENUMERATION: &str =
    "last_fitting_unit_then_refusal_and_unmount_on_a_nearly_full_pool";

/// 崩溃放量里这条流的名字（来历表里与之并列的一行）。
const FLOW_NAME_AT_THE_AMPLIFICATION: &str =
    "last-fitting-unit-then-refusal-and-unmount-on-a-nearly-full-pool";

/// 录制流闭式计数（`crash_state_count_with_torn_in_place_overwrites` 按全量展开数）：126 次写、26 段，第二台上只录不枚举的探针现算。
const FULL_STATE_COUNT: u64 = 16_920_659;

const READ_ONLY_MOUNT_READ_BACK_THE_RECOVERED_CONTENT: &str =
    "read_only_mount_read_back_the_recovered_content";
/// 恢复落到断言被拒的那几版（基镜像那一版、写最后一个单元那一版、卸载完成那一版），再写一个单元被空间拒。
const ONE_MORE_UNIT_REFUSED_WHERE_THE_REFUSAL_IS_ASSERTED: &str =
    "one_more_unit_refused_where_the_refusal_is_asserted";
/// 恢复落到拒写那一次准入推的抬 F 空发布那几版，再写一个单元写进了（照实计数，不断言）。
const ONE_MORE_UNIT_WRITTEN_ON_A_FLOOR_RAISE_OF_THE_REFUSED_WRITE: &str =
    "one_more_unit_written_on_a_floor_raise_of_the_refused_write";
/// 恢复落到拒写那一次准入推的抬 F 空发布那几版，再写一个单元被空间拒（照实计数，不断言）。
const ONE_MORE_UNIT_REFUSED_ON_A_FLOOR_RAISE_OF_THE_REFUSED_WRITE: &str =
    "one_more_unit_refused_on_a_floor_raise_of_the_refused_write";
/// 恢复落到卸载抬 F 中途那几版，再写一个单元写进了（照实计数，不断言）。
const ONE_MORE_UNIT_WRITTEN_IN_THE_MIDDLE_OF_THE_UNMOUNT: &str =
    "one_more_unit_written_in_the_middle_of_the_unmount";
/// 恢复落到卸载抬 F 中途那几版，再写一个单元被空间拒（照实计数，不断言）。
const ONE_MORE_UNIT_REFUSED_IN_THE_MIDDLE_OF_THE_UNMOUNT: &str =
    "one_more_unit_refused_in_the_middle_of_the_unmount";
const WRITABLE_MOUNT_WRITING_ONE_MORE_UNIT_CHECKER_CLEAN: &str =
    "writable_mount_writing_one_more_unit_checker_clean";

/// 贴边搜索的上界：比这些小盘装得下的数据单元总数多，搜索时核它真被拒。
const DATA_UNITS_MORE_THAN_THE_SMALL_DEVICES_HOLD: u64 = 700;

/// 第二个文件每次写的内容长度：一个数据单元以内（`UserChange::OverwriteOfAnInode` 的上限）。
fn content_length_of_one_unit_of_the_second_file() -> usize {
    data_unit_payload_capacity() / 2
}

fn payload_capacity_in_bytes() -> u64 {
    u64::try_from(data_unit_payload_capacity()).expect("净荷容量装得进 u64")
}

fn content_length_of_data_units(data_units: u64) -> usize {
    usize::try_from(data_units * payload_capacity_in_bytes()).expect("长度装得进 usize")
}

fn copy_of_the_pool(pool: &PoolUnderTest<SparseBlockDevice>) -> PoolUnderTest<SparseBlockDevice> {
    let session = pool.session();
    PoolUnderTest {
        parameters: pool.parameters.clone(),
        device_bytes: pool.device_bytes,
        devices: plain_devices_on(&pool.image()),
        session: Some(MountedSession {
            allocator: session.allocator.clone(),
            current: session.current.clone(),
            instance: session.instance,
            shadow_ledger: session.shadow_ledger,
            parameters_and_device_table: session.parameters_and_device_table.clone(),
            compaction: session.compaction.clone(),
        }),
        write_time_seconds: pool.write_time_seconds,
        content_of_the_current_version: pool.content_of_the_current_version.clone(),
        content_of_each_root: pool.content_of_each_root.clone(),
    }
}

fn pool_after_the_remount<Device: DeviceOverASparseImage>(
    wrap: impl Fn(DeviceIdentity, SparseBlockDevice) -> Device,
) -> PoolUnderTest<Device> {
    let mut pool = PoolUnderTest::start_after_the_first_file_on_devices_of(
        parameters_of_the_384_slot_small_devices(),
        device_bytes_of_the_384_slot_small_devices(),
        wrap,
    );
    pool.crash_and_mount_writable()
        .unwrap_or_else(|error| panic!("崩溃后再可写挂载：{error:?}"));
    pool
}

/// 给第二个文件写一个单元（整份换成一个单元以内的内容）。
fn write_one_unit_of_the_second_file<Device: DeviceOverASparseImage>(
    pool: &mut PoolUnderTest<Device>,
    second_file: InodeNumber,
) -> Result<singlefs_core::mounted_session::UserChangePublished, UserChangeRefused> {
    pool.write_time_seconds += 1;
    let write_time_seconds = pool.write_time_seconds;
    let content = content_of(
        content_length_of_one_unit_of_the_second_file(),
        write_time_seconds,
    );
    pool.publish(
        UserChange::OverwriteOfAnInode {
            inode: second_file,
            content: &content,
            write_time_seconds,
        },
        None,
    )
}

/// 第一个文件顺序写成 `data_units` 个单元之后，第二个文件再写一个单元还装得下（在副本上试，不动 `pool`）。
/// 空间不够之外的拒绝都是录流的前提坏了。
fn one_unit_of_the_second_file_still_fits_after_filling(
    pool: &PoolUnderTest<SparseBlockDevice>,
    second_file: InodeNumber,
    data_units: u64,
) -> bool {
    let mut copy = copy_of_the_pool(pool);
    let outcome = copy
        .sequential_write(content_length_of_data_units(data_units))
        .and_then(|_filled| write_one_unit_of_the_second_file(&mut copy, second_file));
    match outcome {
        Ok(_written) => true,
        Err(UserChangeRefused::NoSpaceAfterRaisingTheFloor(_)) => false,
        Err(refusal) => panic!("填池探测只能因空间不够被拒：{refusal:?}"),
    }
}

/// 第一个文件顺序写成多少个单元之后，第二个文件恰好还写得下一个单元（二分；上界核过被拒、下界核过写得进）。
fn data_units_filling_the_pool_up_to_one_last_unit(
    pool: &PoolUnderTest<SparseBlockDevice>,
    second_file: InodeNumber,
) -> u64 {
    let mut low = 1;
    let mut high = DATA_UNITS_MORE_THAN_THE_SMALL_DEVICES_HOLD;
    assert!(one_unit_of_the_second_file_still_fits_after_filling(
        pool,
        second_file,
        low
    ));
    assert!(
        !one_unit_of_the_second_file_still_fits_after_filling(pool, second_file, high),
        "搜索上界必须实际拒写"
    );
    while high - low > 1 {
        let middle = low + (high - low) / 2;
        if one_unit_of_the_second_file_still_fits_after_filling(pool, second_file, middle) {
            low = middle;
        } else {
            high = middle;
        }
    }
    low
}

/// 观察者另读的输入：第二个文件的 inode 号；断言再写一个单元被拒的那几版的根（基镜像那一版、写最后一个单元那一版、卸载完成那一版，
/// 恰好这三个，录流时现取）；拒写那一次准入推的抬 F 空发布那几版的根（拒绝里带回的抬 F 那几串发的）；卸载抬 F 中途那几版的根
/// （卸载发的、最后一版之前的那几版）。后两类只计数。
#[derive(Debug, Clone)]
pub struct InputsOfTheObservers {
    second_file: InodeNumber,
    roots_where_one_more_unit_is_refused: Vec<(InstanceGeneration, CheckpointTxg)>,
    roots_of_the_floor_raises_of_the_refused_write: Vec<(InstanceGeneration, CheckpointTxg)>,
    roots_written_by_the_unmount_before_its_last_publish: Vec<(InstanceGeneration, CheckpointTxg)>,
}

/// 恢复落到的那一版在这条流里是哪一种：定可写接管之后「再写一个单元」断不断言被拒。封闭集合。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum LandedVersionForOneMoreUnit {
    /// 基镜像那一版、写最后一个单元那一版、卸载完成那一版：断言被空间拒。
    RefusalAsserted,
    /// 拒写那一次准入推的抬 F 空发布那几版：写进与被拒照实分开计数，不断言哪一种（卸载已把新 F 写进系统配置、根还停在这里的状态写得进）。
    OnAFloorRaiseOfTheRefusedWrite,
    /// 卸载抬 F 中途那几版：写进与被拒照实分开计数，不断言哪一种。
    InTheMiddleOfTheUnmount,
}

/// 恢复落到的根是哪一种版本。
///
/// # Panics
/// 没择到根，或择到的根不是基镜像那一版、写最后一个单元那一版、拒写那一次推的抬 F 空发布、卸载发的哪一版。
fn landed_version_for_one_more_unit(
    inputs: &InputsOfTheObservers,
    effective_root: Option<(InstanceGeneration, CheckpointTxg)>,
) -> LandedVersionForOneMoreUnit {
    let root = effective_root.expect("每个状态都择到根（oracle 另判）");
    if inputs.roots_where_one_more_unit_is_refused.contains(&root) {
        LandedVersionForOneMoreUnit::RefusalAsserted
    } else if inputs
        .roots_of_the_floor_raises_of_the_refused_write
        .contains(&root)
    {
        LandedVersionForOneMoreUnit::OnAFloorRaiseOfTheRefusedWrite
    } else if inputs
        .roots_written_by_the_unmount_before_its_last_publish
        .contains(&root)
    {
        LandedVersionForOneMoreUnit::InTheMiddleOfTheUnmount
    } else {
        panic!(
            "恢复落到 {root:?}：不是基镜像那一版、写最后一个单元那一版、拒写那一次推的抬 F 空发布，也不是卸载发的哪一版"
        )
    }
}

/// 根身份按先后去重（同一版的根可能逐盘各写一次）。
fn distinct_roots_in_order(
    roots: impl IntoIterator<Item = (InstanceGeneration, CheckpointTxg)>,
) -> Vec<(InstanceGeneration, CheckpointTxg)> {
    let mut distinct = Vec::new();
    for root in roots {
        if !distinct.contains(&root) {
            distinct.push(root);
        }
    }
    distinct
}

/// 几条写里根槽写写下的根身份，按先后去重。
fn roots_written_among(writes: &[RetainedWrite]) -> Vec<(InstanceGeneration, CheckpointTxg)> {
    distinct_roots_in_order(
        writes
            .iter()
            .filter(|write| write.kind == StepKind::RootRecordFua)
            .map(root_identity_of_write),
    )
}

impl InputsOfTheObservers {
    fn digest(&self) -> [u8; DIGEST_BYTES] {
        sha256_digest(format!("{self:?}").as_bytes())
    }
}

/// 录流时核的形状（`the_recorded_stream_writes_the_last_fitting_unit_inside_the_crash_domain` 与
/// `the_refusal_is_asserted_on_exactly_three_roots_and_the_floor_raises_of_the_refused_write_are_only_counted` 判）。
#[derive(Debug, Clone)]
struct ShapeOfTheLastFittingUnitStream {
    data_units_of_the_first_file: u64,
    /// 第一个文件再多顺序写一个单元（`data_units_of_the_first_file + 1` 个）之后，第二个文件还写不写得下一个单元（在探针池的副本上试）：
    /// 池填到了贴边才是 false。少填一个单元时拒写那一步照样被拒（写一个单元连同元数据要的槽不止一个），录流的形状分不出，只有这一判分得出。
    one_unit_of_the_second_file_fits_after_one_more_unit_of_the_first_file: bool,
    /// 写最后一个单元那一次录进流的写里有几条写的是数据单元（[`data_unit_writes_among`]）：
    /// 判数据单元的那一判在写得进的这一次上数得出，拒写那一次数出 0 才说明得了什么。
    data_unit_writes_of_the_last_fitting_unit: usize,
    /// 拒写那一次之前与之后录制流的长度：准入拒之前先推了几次抬 F 的空发布，之后比之前长。
    operations_before_the_refusal: usize,
    operations_after_the_refusal: usize,
    /// 拒写那一次录进流的写里有几条根槽写（抬 F 的空发布发出的根）。
    root_record_writes_of_the_refusal: usize,
    /// 拒写那一次录进流的写里有几条写的是数据单元（被拒的那个单元不该写下）。
    data_unit_writes_of_the_refusal: usize,
    refusal: String,
    unmount_publishes: usize,
    /// 下面几样给 `the_refusal_is_asserted_on_exactly_three_roots_and_the_floor_raises_of_the_refused_write_are_only_counted` 判，
    /// 都在录流时现取：基镜像那一版的根、写最后一个单元那一次发成的根（发成之后现行那一版）、卸载发的每一版的根（按先后）。
    root_of_the_base_image: (InstanceGeneration, CheckpointTxg),
    root_of_the_last_fitting_unit: (InstanceGeneration, CheckpointTxg),
    roots_of_the_unmount: Vec<(InstanceGeneration, CheckpointTxg)>,
    /// 拒写那一次录进流的根槽写写下的根（按先后去重），与拒绝里带回的抬 F 那几串发的根对照。
    roots_written_by_the_refusal: Vec<(InstanceGeneration, CheckpointTxg)>,
    /// 整条录制流的根槽写写下的根（按先后去重）：每一个都要落进观察者认得的那几类。
    roots_written_in_the_recorded_stream: Vec<(InstanceGeneration, CheckpointTxg)>,
}

/// 几条写写的是数据单元：整条写按数据单元的格式解析得过。
/// 段种类 [`StepKind::UnitWrite`] 把数据单元、索引节点、打包记录单元归成一种，抬 F 的空发布也发单元写，按段种类分不开；
/// 每条单元写是整一个单元逐盘写一次（`CommitStep::WriteUnitToEveryDevice`），所以按写下的字节判。
fn data_unit_writes_among(writes: &[RetainedWrite]) -> usize {
    writes
        .iter()
        .filter(|write| write.kind == StepKind::UnitWrite)
        .filter(|write| {
            write
                .bytes()
                .is_some_and(|bytes| parse_data_unit(bytes).is_ok())
        })
        .count()
}

struct PreparedLastFittingUnitStream {
    base: MemoryPool,
    writes: Vec<RetainedWrite>,
    segments: Vec<Vec<usize>>,
    judged_root_index: usize,
    versions: Vec<PublishedVersion>,
    inputs: InputsOfTheObservers,
    shape: ShapeOfTheLastFittingUnitStream,
}

fn prepare_stream() -> PreparedLastFittingUnitStream {
    let stream = SharedStream::retaining_contents_and_reads();
    let wrap = |identity, device| {
        RecordingBlockDevice::with_shared_stream(identity, device, stream.clone())
    };
    let mut pool = pool_after_the_remount(wrap);
    let mut probe_pool = pool_after_the_remount(plain_device);
    let second_file = InodeNumber(pool.current_file_version().inode_number_watermark());
    pool.create_one_inode()
        .unwrap_or_else(|refusal| panic!("建第二个文件：{refusal:?}"));
    probe_pool
        .create_one_inode()
        .unwrap_or_else(|refusal| panic!("探针池上建第二个文件：{refusal:?}"));
    let data_units_of_the_first_file =
        data_units_filling_the_pool_up_to_one_last_unit(&probe_pool, second_file);
    let one_unit_of_the_second_file_fits_after_one_more_unit_of_the_first_file =
        one_unit_of_the_second_file_still_fits_after_filling(
            &probe_pool,
            second_file,
            data_units_of_the_first_file + 1,
        );
    pool.sequential_write(content_length_of_data_units(data_units_of_the_first_file))
        .unwrap_or_else(|refusal| panic!("第一个文件顺序写到只剩最后一个单元的空间：{refusal:?}"));

    let root_of_the_base_image = {
        let version = pool.current_file_version();
        (version.root.instance, version.root.checkpoint_txg)
    };
    let operations_before_the_last_fitting_unit = stream.retained_operations().len();
    let base = pool.image();

    write_one_unit_of_the_second_file(&mut pool, second_file)
        .unwrap_or_else(|refusal| panic!("最后一个装得下的单元写得进：{refusal:?}"));
    let root_of_the_last_fitting_unit = {
        let version = pool.current_file_version();
        (version.root.instance, version.root.checkpoint_txg)
    };
    let operations_before_the_refusal = stream.retained_operations().len();
    let (writes_of_the_last_fitting_unit, _segments_of_the_last_fitting_unit) = writes_and_segments(
        &stream.retained_operations()[operations_before_the_last_fitting_unit..],
        &fixed_geometry_of_the_384_slot_small_devices(),
    );
    let refusal = write_one_unit_of_the_second_file(&mut pool, second_file)
        .expect_err("最后一个单元写完之后再写一个单元被拒");
    let UserChangeRefused::NoSpaceAfterRaisingTheFloor(no_space) = &refusal else {
        panic!("最后一个单元写完之后再写一个单元，拒的原因是空间不够：{refusal:?}");
    };
    let roots_of_the_floor_raises_of_the_refused_write = distinct_roots_in_order(
        no_space
            .floor_raises
            .iter()
            .flat_map(|raised| raised.publishes.iter())
            .map(|published| (published.root.instance, published.root.checkpoint_txg)),
    );
    let operations_after_the_refusal = stream.retained_operations().len();
    let (writes_of_the_refusal, _segments_of_the_refusal) = writes_and_segments(
        &stream.retained_operations()[operations_before_the_refusal..operations_after_the_refusal],
        &fixed_geometry_of_the_384_slot_small_devices(),
    );

    let mut session = pool.session.take().expect("拒写之后会话还在");
    let Unmounted::FloorRaisedToTheCurrentVersion(unmounted) = stream
        .record_entry(RecordedPublishEntry::Unmount, || {
            unmount(
                &pool.parameters,
                &mut pool.devices,
                &mut session.allocator,
                &mut session.current,
                session.shadow_ledger,
            )
        })
        .expect("拒写之后正常卸载")
    else {
        panic!("带文件的卸载要抬 F 发布");
    };
    for published in &unmounted.publishes {
        pool.note_root_of_the_current_content(&published.root);
    }
    assert_read_only_content(&pool.devices, &pool.content_of_the_current_version);

    let recorded = stream.retained_operations()[operations_before_the_last_fitting_unit..].to_vec();
    let entries: Vec<_> = stream
        .entry_spans()
        .into_iter()
        .map(|span| RecordedEntrySpan {
            entry: span.entry,
            operations: span.operations.start - operations_before_the_last_fitting_unit
                ..span.operations.end - operations_before_the_last_fitting_unit,
        })
        .collect();
    let (writes, segments, _) = writes_and_segments_with_stream_indexes_and_entries(
        &recorded,
        &entries,
        &fixed_geometry_of_the_384_slot_small_devices(),
    );
    let judged_root_index = writes
        .iter()
        .rposition(|write| write.kind == StepKind::RootRecordFua)
        .expect("录下的写里至少有一次根槽写");
    let versions: Vec<PublishedVersion> = pool
        .content_of_each_root
        .iter()
        .map(|(root, content)| PublishedVersion {
            instance: root.0,
            checkpoint_txg: root.1,
            content: content.clone(),
        })
        .collect();
    let roots_of_the_unmount: Vec<(InstanceGeneration, CheckpointTxg)> = unmounted
        .publishes
        .iter()
        .map(|published| (published.root.instance, published.root.checkpoint_txg))
        .collect();
    let (root_after_the_unmount, roots_written_by_the_unmount_before_its_last_publish) =
        roots_of_the_unmount
            .split_last()
            .expect("带文件的卸载抬 F 至少发一版");
    // 断言被拒的恰好这三版；拒写那一次推的抬 F 空发布那几版不在里面：卸载把新 F 写进系统配置之后、根还停在那里的状态写得进
    let roots_where_one_more_unit_is_refused: Vec<(InstanceGeneration, CheckpointTxg)> = vec![
        root_of_the_base_image,
        root_of_the_last_fitting_unit,
        *root_after_the_unmount,
    ];
    let shape = ShapeOfTheLastFittingUnitStream {
        data_units_of_the_first_file,
        one_unit_of_the_second_file_fits_after_one_more_unit_of_the_first_file,
        data_unit_writes_of_the_last_fitting_unit: data_unit_writes_among(
            &writes_of_the_last_fitting_unit,
        ),
        operations_before_the_refusal,
        operations_after_the_refusal,
        root_record_writes_of_the_refusal: writes_of_the_refusal
            .iter()
            .filter(|write| write.kind == StepKind::RootRecordFua)
            .count(),
        data_unit_writes_of_the_refusal: data_unit_writes_among(&writes_of_the_refusal),
        refusal: format!("{refusal:?}"),
        unmount_publishes: unmounted.publishes.len(),
        root_of_the_base_image,
        root_of_the_last_fitting_unit,
        roots_of_the_unmount: roots_of_the_unmount.clone(),
        roots_written_by_the_refusal: roots_written_among(&writes_of_the_refusal),
        roots_written_in_the_recorded_stream: roots_written_among(&writes),
    };

    PreparedLastFittingUnitStream {
        base,
        writes,
        segments,
        judged_root_index,
        versions,
        inputs: InputsOfTheObservers {
            second_file,
            roots_where_one_more_unit_is_refused,
            roots_of_the_floor_raises_of_the_refused_write,
            roots_written_by_the_unmount_before_its_last_publish:
                roots_written_by_the_unmount_before_its_last_publish.to_vec(),
        },
        shape,
    }
}

fn assert_read_only_content(image: &dyn singlefs_core::recovery::PoolReader, expected: &[u8]) {
    let mounted = mount_read_only(image).expect("只读挂载成功");
    let file = mounted
        .mounted
        .open_file(image, InodeNumber(FIRST_INODE_NUMBER))
        .expect("只读打开第一个文件");
    let expected_length = u64::try_from(expected.len()).expect("长度装得进 u64");
    let read = file
        .read_at(image, FileOffsetInBytes(0), expected_length)
        .expect("只读读回完整内容");
    assert_eq!(read.bytes, expected, "只读挂载读回的第一个文件与期望相同");
    assert!(
        file.read_at(image, FileOffsetInBytes(expected_length), 1)
            .is_err(),
        "文件长度与期望相同，不能只比较共同前缀"
    );
}

/// 每个崩溃状态上只读挂载读回恢复读出的第一个文件（只读挂载与恢复共用择根：根选错由 oracle 判，这里判只读挂载那条读路径）。
pub fn observe_the_read_only_mount_reading_back_the_recovered_content(
    _inputs: &InputsOfTheObservers,
    image: &CrashImage<'_>,
    report: &RecoveryReport,
    counts: &mut CrashPointReplayObserverCounts,
) {
    let RecoveryOutcome::FileRead { content, .. } = &report.outcome else {
        panic!("每个状态都恢复出第一个文件：{:?}", report.outcome);
    };
    assert_read_only_content(image, content);
    counts.add(READ_ONLY_MOUNT_READ_BACK_THE_RECOVERED_CONTENT, 1);
}

/// 每个崩溃状态上可写接管、给第二个文件再写一个单元：恢复落到基镜像那一版（写最后一个单元之前）、写最后一个单元那一版或卸载完成那一版时
/// 断言被空间拒；落到拒写那一次推的抬 F 空发布那几版、卸载抬 F 中途那几版时，写进与被拒按这两类各自照实计数、不断言哪一种
/// （[`landed_version_for_one_more_unit`]）。
/// 之后会话丢掉，经只读挂载读回第一个文件不变。交回的盘由判段跑池级 checker。
// gpu-exempt: writable-mount
pub fn observe_the_writable_mount_writing_one_more_unit(
    inputs: &InputsOfTheObservers,
    image: &CrashImage<'_>,
    report: &RecoveryReport,
    counts: &mut CrashPointReplayObserverCounts,
) -> ImageAfterTheObserver {
    let mount_parameters = parameters_of_the_384_slot_small_devices();
    let materialized = materialized_crash_image(image);
    let stream = SharedStream::retaining_contents_and_reads();
    let devices: Vec<_> = materialized
        .devices
        .iter()
        .map(|(identity, sectors)| {
            let mut device = SparseBlockDevice::new(
                materialized.device_size_in_bytes,
                PhysicalBlockSizeInBytes(mount_parameters.geometry.physical_block_size),
            );
            device.image = sectors.clone();
            (
                *identity,
                RecordingBlockDevice::with_shared_stream(*identity, device, stream.clone()),
            )
        })
        .collect();
    let RecoveryOutcome::FileRead { content, .. } = &report.outcome else {
        panic!("恢复读出第一个文件：{:?}", report.outcome);
    };
    let landed = landed_version_for_one_more_unit(inputs, report.effective_root);
    let mut pool = PoolUnderTest {
        parameters: mount_parameters.clone(),
        device_bytes: materialized.device_size_in_bytes,
        devices,
        session: None,
        write_time_seconds: common::FIXED_WRITE_TIME_SECONDS + 600,
        content_of_the_current_version: content.clone(),
        content_of_each_root: Default::default(),
    };
    {
        let mounted = mount_writable(&mount_parameters, &mut pool.devices)
            .unwrap_or_else(|error| panic!("崩溃状态上可写挂载接管成功：{error:?}"));
        let (_output, session) = MountedSession::of_the_mount(mounted);
        pool.session = Some(session);
    }
    match (
        write_one_unit_of_the_second_file(&mut pool, inputs.second_file),
        landed,
    ) {
        (
            Err(UserChangeRefused::NoSpaceAfterRaisingTheFloor(_)),
            LandedVersionForOneMoreUnit::RefusalAsserted,
        ) => counts.add(ONE_MORE_UNIT_REFUSED_WHERE_THE_REFUSAL_IS_ASSERTED, 1),
        (
            Err(UserChangeRefused::NoSpaceAfterRaisingTheFloor(_)),
            LandedVersionForOneMoreUnit::OnAFloorRaiseOfTheRefusedWrite,
        ) => counts.add(
            ONE_MORE_UNIT_REFUSED_ON_A_FLOOR_RAISE_OF_THE_REFUSED_WRITE,
            1,
        ),
        (Ok(_), LandedVersionForOneMoreUnit::OnAFloorRaiseOfTheRefusedWrite) => {
            counts.add(
                ONE_MORE_UNIT_WRITTEN_ON_A_FLOOR_RAISE_OF_THE_REFUSED_WRITE,
                1,
            );
        }
        (
            Err(UserChangeRefused::NoSpaceAfterRaisingTheFloor(_)),
            LandedVersionForOneMoreUnit::InTheMiddleOfTheUnmount,
        ) => counts.add(ONE_MORE_UNIT_REFUSED_IN_THE_MIDDLE_OF_THE_UNMOUNT, 1),
        (Ok(_), LandedVersionForOneMoreUnit::InTheMiddleOfTheUnmount) => {
            counts.add(ONE_MORE_UNIT_WRITTEN_IN_THE_MIDDLE_OF_THE_UNMOUNT, 1);
        }
        (outcome, landed_version) => panic!(
            "恢复落到 {:?}（{landed_version:?}），给第二个文件再写一个单元：{outcome:?}",
            report.effective_root
        ),
    }
    pool.session = None;
    assert_read_only_content(&pool.devices, content);
    let disks_after_the_observer = MemoryPool {
        devices: pool
            .devices
            .iter()
            .map(|(identity, device)| (*identity, device.wrapped_device().image.clone()))
            .collect(),
        device_size_in_bytes: materialized.device_size_in_bytes,
    };
    let (writes_on_the_crash_image, _segments_of_the_mount) = writes_and_segments(
        &stream.retained_operations(),
        &fixed_geometry_of_the_384_slot_small_devices(),
    );

    ImageAfterTheObserver {
        writes_on_the_crash_image,
        disks_after_the_observer,
        ranges_read_on_the_crash_image: ReadRangeOnTheCrashImage::of_the_recorded_read_ranges(
            &stream.read_ranges(),
        ),
        invariants_judged: InvariantsJudgedOnTheImageAfterTheObserver::EveryImplementedInvariant,
        counter_when_the_pool_checker_is_clean: WRITABLE_MOUNT_WRITING_ONE_MORE_UNIT_CHECKER_CLEAN
            .to_string(),
    }
}

fn enumerate(
    item_label: &str,
    prepared: &PreparedLastFittingUnitStream,
    expansion: &dyn Fn(usize, &[usize]) -> CrashPointReplaySegmentExpansion,
) -> StageOutcome<CrashPointReplayTally> {
    let inputs = &prepared.inputs;
    let observe_read_only =
        |_ordinal: u64,
         image: &CrashImage<'_>,
         report: &RecoveryReport,
         counts: &mut CrashPointReplayObserverCounts| {
            observe_the_read_only_mount_reading_back_the_recovered_content(
                inputs, image, report, counts,
            );
        };
    let observe_writable =
        |_ordinal: u64,
         image: &CrashImage<'_>,
         report: &RecoveryReport,
         counts: &mut CrashPointReplayObserverCounts| {
            observe_the_writable_mount_writing_one_more_unit(inputs, image, report, counts)
        };

    let item_name = format!("{FLOW_NAME_AT_THE_ENUMERATION}::{item_label}");
    enumerate_crash_states_with_every_kind_of_observer(
        CrashPointReplayItem {
            name: &item_name,
            setting_file: PATH_OF_THIS_TEST_FILE,
        },
        &prepared.base,
        &prepared.writes,
        &prepared.segments,
        prepared.judged_root_index,
        CrashPointReplayVersionTable::with_a_file_only(&prepared.versions),
        expansion,
        &[CrashPointReplayObserver {
            name: "observe_the_read_only_mount_reading_back_the_recovered_content",
            extra_inputs_digest: inputs.digest(),
            observe: &observe_read_only,
        }],
        &[CrashPointReplayObserverHandingBackAnImage {
            name: "observe_the_writable_mount_writing_one_more_unit",
            extra_inputs_digest: inputs.digest(),
            observe: &observe_writable,
            deduplication: ObserverDeduplication::ByTheRecoveryReportAndTheReadSet,
        }],
    )
}

/// 判段整份的结果。只录那一趟（`SINGLEFS_CRASH_AMPLIFICATION_STAGE=record`、配了 GPU 而这份二进制没编 gpu 特性）交只录，
/// 照两段协议收工、不判；两台分片的一份交「只判了自己那一份」，只在设了分片变量时认，别的时候判红。
fn judged_tally_or_left_to_another_stage(
    outcome: StageOutcome<CrashPointReplayTally>,
) -> Option<CrashPointReplayTally> {
    match outcome {
        StageOutcome::Judged(tally) => Some(tally),
        StageOutcome::RecordedOnly => None,
        StageOutcome::JudgedMyShareOnly {
            blocks_judged_here,
            blocks_left_to_other_hosts,
        } => {
            assert!(
                std::env::var_os("SINGLEFS_CRASH_AMPLIFICATION_SHARE").is_some(),
                "没设分片变量却只判了自己那一份（本机 {blocks_judged_here} 块、留给别的机器 {blocks_left_to_other_hosts} 块）"
            );
            None
        }
    }
}

fn assert_clean(tally: &CrashPointReplayTally) {
    assert_eq!(tally.violations, 0, "oracle：{:?}", tally.first_violation);
    assert_eq!(
        tally.ignored_violations, 0,
        "不看 journal 那一遍恢复同样过 oracle：{:?}",
        tally.first_ignored_violation
    );
    assert_eq!(tally.failed_states, 0, "走读一次都不失败");
    assert_eq!(tally.record_root_without_record, 0);
    assert_eq!(tally.record_claimed_state_missing_unit, 0);
    assert_eq!(
        tally
            .observer_counts
            .count_named(READ_ONLY_MOUNT_READ_BACK_THE_RECOVERED_CONTENT),
        tally.states
    );
    let refused_where_asserted = tally
        .observer_counts
        .count_named(ONE_MORE_UNIT_REFUSED_WHERE_THE_REFUSAL_IS_ASSERTED);
    let written_on_a_floor_raise_of_the_refused_write = tally
        .observer_counts
        .count_named(ONE_MORE_UNIT_WRITTEN_ON_A_FLOOR_RAISE_OF_THE_REFUSED_WRITE);
    let refused_on_a_floor_raise_of_the_refused_write = tally
        .observer_counts
        .count_named(ONE_MORE_UNIT_REFUSED_ON_A_FLOOR_RAISE_OF_THE_REFUSED_WRITE);
    let written_in_the_middle_of_the_unmount = tally
        .observer_counts
        .count_named(ONE_MORE_UNIT_WRITTEN_IN_THE_MIDDLE_OF_THE_UNMOUNT);
    let refused_in_the_middle_of_the_unmount = tally
        .observer_counts
        .count_named(ONE_MORE_UNIT_REFUSED_IN_THE_MIDDLE_OF_THE_UNMOUNT);
    assert_eq!(
        refused_where_asserted
            + written_on_a_floor_raise_of_the_refused_write
            + refused_on_a_floor_raise_of_the_refused_write
            + written_in_the_middle_of_the_unmount
            + refused_in_the_middle_of_the_unmount,
        tally.states,
        "每个状态上都可写接管、再写了一个单元：断言被拒的那几版 {refused_where_asserted}，拒写那一次推的抬 F 空发布上写进 {written_on_a_floor_raise_of_the_refused_write}、被拒 {refused_on_a_floor_raise_of_the_refused_write}，卸载中途写进 {written_in_the_middle_of_the_unmount}、被拒 {refused_in_the_middle_of_the_unmount}"
    );
    singlefs_verification::crash_state_enumeration::assert_the_count_is_above_zero_unless_narrowed(
        refused_where_asserted,
        "至少有一个状态落到断言被拒的那几版（基镜像那一版、写最后一个单元那一版、卸载完成那一版），在那里再写一个单元被拒",
    );
    assert_eq!(
        tally
            .observer_counts
            .count_named(WRITABLE_MOUNT_WRITING_ONE_MORE_UNIT_CHECKER_CLEAN),
        tally.states
    );
    for invariant in singlefs_checker::image::IMPLEMENTED_INVARIANTS {
        assert_eq!(
            tally
                .checker_violated_states
                .get(invariant)
                .copied()
                .unwrap_or(0),
            0,
            "{invariant} 判违例：{:?}",
            tally.checker_first_violation.get(invariant)
        );
    }
}

#[test]
fn full_enumeration_of_the_last_fitting_unit_then_refusal_and_unmount_on_a_nearly_full_pool_stream_is_exhaustive_and_clean(
) {
    let prepared = prepare_stream();
    let Some(tally) =
        judged_tally_or_left_to_another_stage(enumerate("full", &prepared, &full_expansion))
    else {
        return;
    };
    let closed_form = crash_state_count_with_torn_in_place_overwrites(
        &prepared.base,
        &prepared.writes,
        &prepared.segments,
        &full_expansion,
    );
    println!(
        "CRASH_POINT_REPLAY_LAST_FITTING_UNIT_THEN_REFUSAL_AND_UNMOUNT_ON_A_NEARLY_FULL_POOL states={} closed_form={} exhaustive={} violations={} observer {:?} checker {} first_violation={}",
        tally.states,
        closed_form,
        tally.states == closed_form,
        tally.violations,
        tally.observer_counts,
        tally.checker_counts_by_invariant(),
        tally.first_violation.as_deref().unwrap_or("none")
    );
    assert_clean(&tally);
    singlefs_verification::crash_state_enumeration::assert_the_counted_states_equal_the_closed_form_unless_narrowed(
        tally.states,
        closed_form,
        "last_fitting_unit_then_refusal_and_unmount_on_a_nearly_full_pool_stream：用例自己的展开下的闭式",
    );
}

#[test]
fn crash_amplification_of_the_last_fitting_unit_then_refusal_and_unmount_on_a_nearly_full_pool_stream_is_clean(
) {
    let prepared = prepare_stream();
    let crash_points = common_crash_points::one_crash_point_over_every_write(
        file!(),
        FLOW_NAME_AT_THE_AMPLIFICATION,
        prepared.writes.len(),
    );
    common_crash_points::amplify_prepared_flow(
        FLOW_NAME_AT_THE_AMPLIFICATION,
        &prepared.base,
        &prepared.writes,
        &prepared.segments,
        prepared.judged_root_index,
        &prepared.versions,
        crash_points,
        singlefs_verification::crash_amplification::EXPAND_UP_TO_WRITES_PER_SEGMENT_OF_THE_AMPLIFICATION,
    );
}

/// 写最后一个装得下的单元那一次在崩溃域里（录制流里有它的数据单元写）；之后再写一个单元被空间拒：拒之前准入先推了几次抬 F 的空发布
/// （录进流里，带根槽写），没写数据单元；再卸载抬 F。基镜像里第一个文件填到了贴边：再多一个单元，第二个文件就写不下一个单元。只录流，不枚举。
#[test]
fn the_recorded_stream_writes_the_last_fitting_unit_inside_the_crash_domain() {
    let prepared = prepare_stream();
    let shape = &prepared.shape;
    assert!(
        shape.data_unit_writes_of_the_last_fitting_unit > 0,
        "写最后一个单元那一次的数据单元写在录制流里：{shape:?}"
    );
    assert!(
        shape.operations_after_the_refusal > shape.operations_before_the_refusal,
        "拒写之前准入先推了抬 F 的空发布，录制流变长：{shape:?}"
    );
    assert!(
        shape.root_record_writes_of_the_refusal > 0,
        "拒写之前推的抬 F 空发布写了根槽：{shape:?}"
    );
    assert_eq!(
        shape.data_unit_writes_of_the_refusal, 0,
        "拒写那一次没写数据单元：{shape:?}"
    );
    assert!(
        shape.refusal.starts_with("NoSpaceAfterRaisingTheFloor"),
        "拒写的原因是空间不够：{shape:?}"
    );
    assert!(shape.unmount_publishes > 0, "卸载抬 F 发布了：{shape:?}");
    assert!(
        shape.data_units_of_the_first_file > 1,
        "第一个文件顺序写成多个单元把池填到贴边：{shape:?}"
    );
    assert!(
        !shape.one_unit_of_the_second_file_fits_after_one_more_unit_of_the_first_file,
        "池填到了贴边：第一个文件再多一个单元，第二个文件就写不下一个单元：{shape:?}"
    );
}

/// 断言被拒的根恰好三个、两两不同（基镜像那一版、写最后一个单元那一版、卸载完成那一版，录流时现取）；拒写那一次准入推的抬 F 空发布那几版
/// 与卸载最后一版之前那几版只计数：观察者按根分类时各落进自己那一类，不落进断言被拒那一类；录制流里每一次根槽写写下的根都在这几类里
/// （观察者不会因为认不得根而 panic）。只录流，不枚举。
#[test]
fn the_refusal_is_asserted_on_exactly_three_roots_and_the_floor_raises_of_the_refused_write_are_only_counted(
) {
    let prepared = prepare_stream();
    let shape = &prepared.shape;
    let inputs = &prepared.inputs;
    let (root_after_the_unmount, roots_of_the_unmount_before_its_last_publish) = shape
        .roots_of_the_unmount
        .split_last()
        .expect("带文件的卸载抬 F 至少发一版");
    let expected_roots_where_the_refusal_is_asserted = vec![
        shape.root_of_the_base_image,
        shape.root_of_the_last_fitting_unit,
        *root_after_the_unmount,
    ];
    assert_eq!(
        distinct_roots_in_order(expected_roots_where_the_refusal_is_asserted.iter().copied()),
        expected_roots_where_the_refusal_is_asserted,
        "基镜像那一版、写最后一个单元那一版、卸载完成那一版是三个不同的根：{shape:?}"
    );
    assert_eq!(
        inputs.roots_where_one_more_unit_is_refused, expected_roots_where_the_refusal_is_asserted,
        "断言被拒的恰好这三个根：{inputs:?}"
    );
    assert!(
        !inputs
            .roots_of_the_floor_raises_of_the_refused_write
            .is_empty(),
        "拒写那一次准入先推了抬 F 的空发布：{inputs:?}"
    );
    assert_eq!(
        inputs.roots_of_the_floor_raises_of_the_refused_write, shape.roots_written_by_the_refusal,
        "拒绝里带回的抬 F 那几串发的根，就是拒写那一次录进流的根槽写写下的根"
    );
    for root in &inputs.roots_of_the_floor_raises_of_the_refused_write {
        assert!(
            !inputs.roots_where_one_more_unit_is_refused.contains(root),
            "拒写那一次推的抬 F 空发布 {root:?} 不断言被拒：{inputs:?}"
        );
    }
    let mut expected_roots_written_in_the_recorded_stream =
        vec![shape.root_of_the_last_fitting_unit];
    expected_roots_written_in_the_recorded_stream.extend(
        inputs
            .roots_of_the_floor_raises_of_the_refused_write
            .iter()
            .copied(),
    );
    expected_roots_written_in_the_recorded_stream
        .extend(shape.roots_of_the_unmount.iter().copied());
    assert_eq!(
        shape.roots_written_in_the_recorded_stream, expected_roots_written_in_the_recorded_stream,
        "录制流的根槽写依次写下写最后一个单元那一版、拒写那一次推的抬 F 空发布、卸载发的每一版"
    );
    let mut expected_landing = vec![
        (
            shape.root_of_the_base_image,
            LandedVersionForOneMoreUnit::RefusalAsserted,
        ),
        (
            shape.root_of_the_last_fitting_unit,
            LandedVersionForOneMoreUnit::RefusalAsserted,
        ),
        (
            *root_after_the_unmount,
            LandedVersionForOneMoreUnit::RefusalAsserted,
        ),
    ];
    expected_landing.extend(
        inputs
            .roots_of_the_floor_raises_of_the_refused_write
            .iter()
            .map(|root| {
                (
                    *root,
                    LandedVersionForOneMoreUnit::OnAFloorRaiseOfTheRefusedWrite,
                )
            }),
    );
    expected_landing.extend(
        roots_of_the_unmount_before_its_last_publish
            .iter()
            .map(|root| (*root, LandedVersionForOneMoreUnit::InTheMiddleOfTheUnmount)),
    );
    for (root, expected) in expected_landing {
        assert_eq!(
            landed_version_for_one_more_unit(inputs, Some(root)),
            expected,
            "恢复落到 {root:?} 时观察者归的类：{inputs:?}"
        );
    }
}

/// 只录流，按写流算全量状态数，不枚举、不开判定库：录法变了而 `FULL_STATE_COUNT` 没跟着改就红。
#[test]
fn the_recorded_stream_counts_its_crash_states() {
    let prepared = prepare_stream();
    let states = crash_state_count_with_torn_in_place_overwrites(
        &prepared.base,
        &prepared.writes,
        &prepared.segments,
        &full_expansion,
    );
    println!(
        "CRASH_ENUMERATION_STATE_COUNT name={FLOW_NAME_AT_THE_ENUMERATION} states={states} writes={} segments={:?}",
        prepared.writes.len(),
        prepared.segments.iter().map(Vec::len).collect::<Vec<_>>()
    );
    assert_eq!(states, FULL_STATE_COUNT, "录制流的全量状态数");
}
