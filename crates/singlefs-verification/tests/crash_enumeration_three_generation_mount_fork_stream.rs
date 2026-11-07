//! 验证包模块：crash、crash_state_enumeration、writable_mount_phases、crash_library、crash_amplification
//! 三代实例挂载与分叉流：
//! 实例 1 新建文件、覆盖写 B，进程退出；实例 2 可写挂载、覆盖写 C，再覆盖写 D，D 的单元写落了盘、journal 记录一条都没写时断电。
//! 实例 3 挂在断电那一刻的盘上（落到 C，比实例 2 那一侧最后落盘的根旧），覆盖写 E，再一次一次建空 inode，直到根环三个区都绕回写过根的槽；
//! 实例 2 那一侧接着把 D 发完、再覆盖写一次，那两版在实例 3 这一侧看不到（分叉）。
//!
//! 基镜像是断电那一刻的盘（实例 1、实例 2 的写与 D 落了盘的单元写都在里面，不枚举）；枚举的是实例 3 那一条录制流。
//! 状态数（录制流闭式计数）写在 `the_recorded_stream_counts_its_crash_states`，全量没判过。
//! 每个崩溃状态跑恢复（看 / 不看 journal）、多版本 oracle、池级 checker 每条已实现的不变量、记录核对器；
//! 另挂两个观察者：择到的根是不是实例 3 这一侧那一代写的（`observe_root_fork_isolation`），实例 4 在崩溃状态上可写挂载、覆盖写、读回（`observe_mount_continuation`）。
//!
//! 每一段都展开任意真子集，与闭式比；登记成崩溃枚举用例，带断点续跑、可双机分片。

mod common;
mod common_crash_point_observers;
mod common_crash_points;

use common::{
    build_pool, file_content, geometry, overwrite_in_process_as_instance_one_and_advance_the_pool,
    parameters, FIXED_WRITE_TIME_SECONDS,
};
use singlefs_core::address::{CheckpointTxg, DeviceIdentity, InstanceGeneration};
use singlefs_core::block_device::PhysicalBlockSizeInBytes;
use singlefs_core::mount::{mount_writable, Mounted};
use singlefs_core::recovery::{recover, JournalPolicy, RecoveryOutcome, RecoveryReport};
use singlefs_core::root_record::RootRecord;
use singlefs_core::transaction::{
    publish_new_inodes, publish_overwrite, FirstFile, PoolWriter, TransactionOutput,
};
use singlefs_format::ROOT_RING_REGIONS;
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
    newest_persisted_root, root_identity_of_write, writes_and_segments,
    writes_and_segments_with_stream_indexes, CrashImage, MemoryPool, PublishedVersion,
    RetainedWrite, SparseBlockDevice,
};
use singlefs_verification::segments::StepKind;
use singlefs_verification::sha256::{sha256_digest, DIGEST_BYTES};
use singlefs_verification::writable_mount_phases::materialized_crash_image;
use singlefs_verification::{RecordingBlockDevice, SharedStream};

/// 这份测试文件的仓内路径：崩溃枚举用例的设点文件。
const PATH_OF_THIS_TEST_FILE: &str =
    "crates/singlefs-verification/tests/crash_enumeration_three_generation_mount_fork_stream.rs";

/// 崩溃点重放项名第一个 `::` 之前那一截（来历表里这条流的名字）。
const FLOW_NAME_AT_THE_ENUMERATION: &str = "three_generation_mount_fork";

/// 崩溃放量里这条流的名字（来历表里与之并列的一行）。
const FLOW_NAME_AT_THE_AMPLIFICATION: &str = "three-generation-mount-fork";

/// 录制流闭式计数（`crash_state_count_with_torn_in_place_overwrites` 按全量展开数）：485 次写、78 段，第二台上只录不枚举的探针现算。
const FULL_STATE_COUNT: u64 = 289_472_738;

const CONTENT_OF_INITIAL_OVERWRITE: &[u8] = b"file B written by instance one";
const CONTENT_OF_SECOND_INSTANCE_FIRST_OVERWRITE: &[u8] =
    b"file C written by instance two (first write)";
const CONTENT_OF_SECOND_INSTANCE_SECOND_OVERWRITE: &[u8] =
    b"file D written by instance two (cut by the power loss on the side of instance three)";
const CONTENT_OF_SECOND_INSTANCE_AFTER_THE_FORK: &[u8] =
    b"file written by instance two after the fork (never seen by instance three)";
const CONTENT_OF_THIRD_INSTANCE_FORKED_OVERWRITE: &[u8] =
    b"file E written by instance three (forked write)";
const CONTENT_OF_FOURTH_INSTANCE_TAKEOVER: &[u8] =
    b"file F written by instance four after recovery";

/// 实例 3 建空 inode 至多几次：三个区各绕回一次写过根的槽要 3 × 8 槽 + 3 次发布上下，超了就是根环没绕回来。
const NEW_INODE_PUBLISHES_OF_THE_THIRD_INSTANCE_AT_MOST: u64 = 48;

const INSTANCE_TWO_ROOT_CHOSEN: &str = "instance_two_root_chosen";
const INSTANCE_THREE_ROOT_CHOSEN: &str = "instance_three_root_chosen";
const CONTINUATION_MOUNT_SUCCEEDED: &str = "continuation_mount_succeeded";

/// 观察者另读的输入：实例 3 这一侧每条根（基镜像里的与实例 3 写的）是哪一代写的，按 txg 认（分叉之后两侧同一个 txg 是不同实例写的）。
#[derive(Debug, Clone)]
pub struct InputsOfTheObservers {
    instance_two: InstanceGeneration,
    instance_three: InstanceGeneration,
    generation_of_each_root_on_this_side: Vec<(CheckpointTxg, InstanceGeneration)>,
}

impl InputsOfTheObservers {
    fn digest(&self) -> [u8; DIGEST_BYTES] {
        sha256_digest(format!("{self:?}").as_bytes())
    }
}

/// 录流时核的形状：分叉与根环绕回那几样（`the_recorded_stream_forks_off_a_publish_cut_by_the_power_loss_and_wraps_the_root_ring` 判）。
#[derive(Debug, Clone)]
struct ShapeOfTheThreeGenerationStream {
    /// D 落了盘的单元写有几个（断电那一刻 D 在发布中途）。
    unit_writes_of_the_publish_cut_by_the_power_loss: usize,
    /// 实例 3 挂载落到的那一版（施加前缀之后的根）。
    root_landed_on_by_the_third_instance: (InstanceGeneration, CheckpointTxg),
    /// 实例 2 那一侧发完 D 的那一版与之后那一版。
    root_of_the_cut_publish_on_the_side_of_instance_two: (InstanceGeneration, CheckpointTxg),
    last_root_on_the_side_of_instance_two: (InstanceGeneration, CheckpointTxg),
    content_read_on_the_side_of_instance_two: Vec<u8>,
    content_read_on_the_side_of_instance_three: Vec<u8>,
    /// 实例 3 写的录制流里实例 2 写的根（应当一条都没有）。
    roots_of_instance_two_in_the_recorded_stream: Vec<(InstanceGeneration, CheckpointTxg)>,
    /// 录制流里每个根环区里有几次根槽写落在写过根（不算 mkfs 种的第 0 代根）的槽上。
    root_slot_reuses_by_region: Vec<u64>,
}

struct RecordedThreeGenerationStream {
    base: MemoryPool,
    writes: Vec<RetainedWrite>,
    segments: Vec<Vec<usize>>,
    judged_root_index: usize,
    versions: Vec<PublishedVersion>,
    inputs: InputsOfTheObservers,
    shape: ShapeOfTheThreeGenerationStream,
}

fn published_version_of(output: &TransactionOutput, content: &[u8]) -> PublishedVersion {
    PublishedVersion {
        instance: output.root.instance,
        checkpoint_txg: output.root.checkpoint_txg,
        content: content.to_vec(),
    }
}

/// 一次可写挂载写的每一版（写行、暖机、准入推的抬 F），都带挂载落到的那一版的内容。
fn versions_written_by_the_mount(mounted: &Mounted, content: &[u8]) -> Vec<PublishedVersion> {
    let mut roots: Vec<RootRecord> = vec![*mounted.output.row_publish.root()];
    roots.extend(
        mounted
            .output
            .warm_up_publishes
            .iter()
            .map(|published| *published.root()),
    );
    roots.extend(
        mounted
            .output
            .space_admission
            .floor_raises()
            .iter()
            .flat_map(|raised| raised.publishes.iter().map(|publish| publish.root)),
    );
    roots
        .into_iter()
        .map(|root| PublishedVersion {
            instance: root.instance,
            checkpoint_txg: root.checkpoint_txg,
            content: content.to_vec(),
        })
        .collect()
}

/// 录制流里每个根环区里有几次根槽写落在写过根的槽上：同一块盘、同一个偏移，基镜像里那一槽读得出一条不是第 0 代的根，
/// 或录制流里更早有一次根槽写落在那里。区按 txg 落点公式取（`root_ring::target_for_publish`：txg 模区数）。
fn root_slot_reuses_by_region(
    base: &MemoryPool,
    writes: &[RetainedWrite],
    filesystem_identifier: &[u8; 16],
) -> Vec<u64> {
    let region_count = usize::try_from(ROOT_RING_REGIONS).expect("三个区");
    let mut reuses = vec![0_u64; region_count];
    for (index, write) in writes.iter().enumerate() {
        if write.kind != StepKind::RootRecordFua {
            continue;
        }
        let length = usize::try_from(write.length_in_bytes()).expect("一条根记录装得进 usize");
        let base_slot_held_a_root = base.devices.get(&write.device).is_some_and(|device| {
            RootRecord::parse_slot(&device.read(write.offset, length), filesystem_identifier)
                .is_some_and(|root| root.checkpoint_txg != CheckpointTxg(0))
        });
        let earlier_write_held_a_root = writes[..index].iter().any(|earlier| {
            earlier.kind == StepKind::RootRecordFua
                && earlier.device == write.device
                && earlier.offset == write.offset
        });
        if base_slot_held_a_root || earlier_write_held_a_root {
            let (_instance, checkpoint_txg) = root_identity_of_write(write);
            let region =
                usize::try_from(checkpoint_txg.0 % ROOT_RING_REGIONS).expect("区号小于区数");
            reuses[region] += 1;
        }
    }
    reuses
}

fn content_of_the_file(reader: &dyn singlefs_core::recovery::PoolReader) -> Vec<u8> {
    let outcome = recover(reader, JournalPolicy::Consult).outcome;
    match outcome {
        RecoveryOutcome::FileRead { content, .. } => content,
        RecoveryOutcome::NoFile { .. } | RecoveryOutcome::Failed { .. } => {
            panic!("这条流的每一版都带文件：{outcome:?}")
        }
    }
}

fn record_the_stream() -> RecordedThreeGenerationStream {
    let publish_parameters = parameters();
    let filesystem_identifier = publish_parameters.filesystem_identifier;
    let mut pool = build_pool("crash-enumeration-three-generation-mount-fork");
    let first_file = pool.output.clone();
    let initial_overwrite = overwrite_in_process_as_instance_one_and_advance_the_pool(
        &mut pool,
        CONTENT_OF_INITIAL_OVERWRITE,
    );
    let mut versions = vec![
        published_version_of(&first_file, &file_content()),
        published_version_of(&initial_overwrite, CONTENT_OF_INITIAL_OVERWRITE),
    ];

    // 实例 2：可写挂载（落到 B）、覆盖写 C、覆盖写 D。
    let mut devices = pool.reopen_recorded();
    let mut mounted_second_instance = mount_writable(&publish_parameters, &mut devices)
        .unwrap_or_else(|error| panic!("实例 2 可写挂载：{error:?}"));
    let instance_two = mounted_second_instance.output.instance;
    versions.extend(versions_written_by_the_mount(
        &mounted_second_instance,
        CONTENT_OF_INITIAL_OVERWRITE,
    ));
    let landed_initial_version = mounted_second_instance
        .current
        .file_version()
        .expect("实例 2 挂载落到初始覆盖写那一版")
        .clone();
    let second_instance_first_overwrite = {
        let mut writer = PoolWriter::new(&publish_parameters, devices.as_mut_slice());
        publish_overwrite(
            &mut writer,
            &mut mounted_second_instance.allocator,
            &landed_initial_version,
            FirstFile {
                content: CONTENT_OF_SECOND_INSTANCE_FIRST_OVERWRITE,
                write_time_seconds: FIXED_WRITE_TIME_SECONDS + 60,
            },
            instance_two,
        )
        .unwrap_or_else(|error| panic!("实例 2 覆盖写 C：{error:?}"))
    };
    versions.push(published_version_of(
        &second_instance_first_overwrite,
        CONTENT_OF_SECOND_INSTANCE_FIRST_OVERWRITE,
    ));
    let image_before_the_cut_publish = pool.memory_pool();
    let operations_before_the_cut_publish = pool.stream.retained_operations().len();
    let second_instance_second_overwrite = {
        let mut writer = PoolWriter::new(&publish_parameters, devices.as_mut_slice());
        publish_overwrite(
            &mut writer,
            &mut mounted_second_instance.allocator,
            &second_instance_first_overwrite,
            FirstFile {
                content: CONTENT_OF_SECOND_INSTANCE_SECOND_OVERWRITE,
                write_time_seconds: FIXED_WRITE_TIME_SECONDS + 120,
            },
            instance_two,
        )
        .unwrap_or_else(|error| panic!("实例 2 覆盖写 D：{error:?}"))
    };

    // 断电落在 D 的第一条 journal 记录写之前：D 的单元写（与关它那一段的屏障）都落了盘，记录与根一条都没有。
    let operations_of_the_cut_publish =
        pool.stream.retained_operations()[operations_before_the_cut_publish..].to_vec();
    let (writes_of_the_cut_publish, _segments_of_the_cut_publish, stream_indexes) =
        writes_and_segments_with_stream_indexes(&operations_of_the_cut_publish, &geometry());
    let power_loss_operation = writes_of_the_cut_publish
        .iter()
        .zip(&stream_indexes)
        .find(|(write, _stream_index)| write.kind == StepKind::JournalRecord)
        .map(|(_write, stream_index)| *stream_index)
        .expect("覆盖写 D 写了 journal 记录");
    let unit_writes_of_the_publish_cut_by_the_power_loss = writes_of_the_cut_publish
        .iter()
        .zip(&stream_indexes)
        .filter(|(write, stream_index)| {
            write.kind == StepKind::UnitWrite && **stream_index < power_loss_operation
        })
        .count();
    let mut base = image_before_the_cut_publish;
    base.apply(&operations_of_the_cut_publish[..power_loss_operation]);

    // 实例 2 那一侧（断电之前的活盘）接着把 D 发完了，再覆盖写一次：这两版只在这一侧。
    let second_instance_after_the_fork = {
        let mut writer = PoolWriter::new(&publish_parameters, devices.as_mut_slice());
        publish_overwrite(
            &mut writer,
            &mut mounted_second_instance.allocator,
            &second_instance_second_overwrite,
            FirstFile {
                content: CONTENT_OF_SECOND_INSTANCE_AFTER_THE_FORK,
                write_time_seconds: FIXED_WRITE_TIME_SECONDS + 150,
            },
            instance_two,
        )
        .unwrap_or_else(|error| panic!("实例 2 分叉之后再覆盖写：{error:?}"))
    };
    let content_read_on_the_side_of_instance_two = content_of_the_file(&devices);
    pool.devices = Some(devices);

    // 实例 3：断电那一刻的盘上可写挂载，覆盖写 E，再一次一次建空 inode，直到三个区都绕回写过根的槽。
    let stream = SharedStream::retaining_contents();
    let mut devices_of_the_third_instance: Vec<(
        DeviceIdentity,
        RecordingBlockDevice<SparseBlockDevice>,
    )> = base
        .devices
        .iter()
        .map(|(identity, sectors)| {
            let mut device = SparseBlockDevice::new(
                base.device_size_in_bytes,
                PhysicalBlockSizeInBytes(publish_parameters.geometry.physical_block_size),
            );
            device.image = sectors.clone();
            (
                *identity,
                RecordingBlockDevice::with_shared_stream(*identity, device, stream.clone()),
            )
        })
        .collect();
    let mut mounted_third_instance =
        mount_writable(&publish_parameters, &mut devices_of_the_third_instance)
            .unwrap_or_else(|error| panic!("实例 3 在断电那一刻的盘上可写挂载：{error:?}"));
    let instance_three = mounted_third_instance.output.instance;
    let landed_root = mounted_third_instance.output.effective_root;
    let landed_for_third_instance = mounted_third_instance
        .current
        .file_version()
        .expect("实例 3 挂载落到带文件的一版")
        .clone();
    let landed_content = content_of_the_file(&devices_of_the_third_instance);
    versions.extend(versions_written_by_the_mount(
        &mounted_third_instance,
        &landed_content,
    ));
    let third_instance_forked_overwrite = {
        let mut writer = PoolWriter::new(
            &publish_parameters,
            devices_of_the_third_instance.as_mut_slice(),
        );
        publish_overwrite(
            &mut writer,
            &mut mounted_third_instance.allocator,
            &landed_for_third_instance,
            FirstFile {
                content: CONTENT_OF_THIRD_INSTANCE_FORKED_OVERWRITE,
                write_time_seconds: FIXED_WRITE_TIME_SECONDS + 180,
            },
            instance_three,
        )
        .unwrap_or_else(|error| panic!("实例 3 覆盖写 E：{error:?}"))
    };
    versions.push(published_version_of(
        &third_instance_forked_overwrite,
        CONTENT_OF_THIRD_INSTANCE_FORKED_OVERWRITE,
    ));
    let mut previous = third_instance_forked_overwrite;
    let mut new_inode_publishes = 0_u64;
    loop {
        let (writes_so_far, _segments_so_far) =
            writes_and_segments(&stream.retained_operations(), &geometry());
        if root_slot_reuses_by_region(&base, &writes_so_far, &filesystem_identifier)
            .iter()
            .all(|reuses| *reuses > 0)
        {
            break;
        }
        assert!(
            new_inode_publishes < NEW_INODE_PUBLISHES_OF_THE_THIRD_INSTANCE_AT_MOST,
            "实例 3 建了 {new_inode_publishes} 次空 inode，根环还有区没绕回写过根的槽"
        );
        new_inode_publishes += 1;
        let mut writer = PoolWriter::new(
            &publish_parameters,
            devices_of_the_third_instance.as_mut_slice(),
        );
        previous = publish_new_inodes(
            &mut writer,
            &mut mounted_third_instance.allocator,
            &previous,
            1,
            FIXED_WRITE_TIME_SECONDS + 180 + new_inode_publishes,
            instance_three,
        )
        .unwrap_or_else(|error| panic!("实例 3 建空 inode：{error:?}"));
        versions.push(published_version_of(
            &previous,
            CONTENT_OF_THIRD_INSTANCE_FORKED_OVERWRITE,
        ));
    }
    let content_read_on_the_side_of_instance_three =
        content_of_the_file(&devices_of_the_third_instance);

    let (writes, segments) = writes_and_segments(&stream.retained_operations(), &geometry());
    let judged_root_index = writes
        .iter()
        .rposition(|write| write.kind == StepKind::RootRecordFua)
        .expect("实例 3 写了根");
    let roots_in_the_recorded_stream: Vec<(InstanceGeneration, CheckpointTxg)> = writes
        .iter()
        .filter(|write| write.kind == StepKind::RootRecordFua)
        .map(root_identity_of_write)
        .collect();
    for root in &roots_in_the_recorded_stream {
        assert!(
            versions
                .iter()
                .any(|version| (version.instance, version.checkpoint_txg) == *root),
            "录制流里的根 {root:?} 是实例 3 挂载或发布交回过的一版"
        );
    }
    let shape = ShapeOfTheThreeGenerationStream {
        unit_writes_of_the_publish_cut_by_the_power_loss,
        root_landed_on_by_the_third_instance: (landed_root.instance, landed_root.checkpoint_txg),
        root_of_the_cut_publish_on_the_side_of_instance_two: (
            second_instance_second_overwrite.root.instance,
            second_instance_second_overwrite.root.checkpoint_txg,
        ),
        last_root_on_the_side_of_instance_two: (
            second_instance_after_the_fork.root.instance,
            second_instance_after_the_fork.root.checkpoint_txg,
        ),
        content_read_on_the_side_of_instance_two,
        content_read_on_the_side_of_instance_three,
        roots_of_instance_two_in_the_recorded_stream: roots_in_the_recorded_stream
            .iter()
            .copied()
            .filter(|(instance, _txg)| *instance == instance_two)
            .collect(),
        root_slot_reuses_by_region: root_slot_reuses_by_region(
            &base,
            &writes,
            &filesystem_identifier,
        ),
    };
    let generation_of_each_root_on_this_side = versions
        .iter()
        .map(|version| (version.checkpoint_txg, version.instance))
        .collect();

    RecordedThreeGenerationStream {
        base,
        writes,
        segments,
        judged_root_index,
        versions,
        inputs: InputsOfTheObservers {
            instance_two,
            instance_three,
            generation_of_each_root_on_this_side,
        },
        shape,
    }
}

/// 择到的根是实例 3 这一侧那一代写的：择根为空、择到的 txg 不是这一侧写过的根、或是另一代写的（分叉另一侧那一版），记红（panic）；
/// 盘上持久了实例 3 的根槽写的状态，择到的必须是实例 3 的根。按择到的是哪一代计数。
pub fn observe_root_fork_isolation(
    inputs: &InputsOfTheObservers,
    image: &CrashImage<'_>,
    report: &RecoveryReport,
    counts: &mut CrashPointReplayObserverCounts,
) {
    let Some((effective_instance, effective_txg)) = report.effective_root else {
        panic!("择根为空：基镜像里持久着实例 2 写的 C 那一版，每个崩溃状态都有根可择");
    };
    let generation_on_this_side = inputs
        .generation_of_each_root_on_this_side
        .iter()
        .find(|(checkpoint_txg, _instance)| *checkpoint_txg == effective_txg)
        .map(|(_checkpoint_txg, instance)| *instance)
        .unwrap_or_else(|| {
            panic!(
                "择到的根 txg {} 不是实例 3 这一侧写过的根（实例 {} 写的）",
                effective_txg.0, effective_instance.0
            )
        });
    assert_eq!(
        effective_instance, generation_on_this_side,
        "txg {} 在实例 3 这一侧是实例 {} 写的，择到的却是实例 {} 的根（分叉另一侧那一版）",
        effective_txg.0, generation_on_this_side.0, effective_instance.0
    );
    if let Some((_newest_txg, newest_instance)) =
        newest_persisted_root(image.writes, &image.persisted)
    {
        if newest_instance == inputs.instance_three {
            assert_eq!(
                effective_instance, inputs.instance_three,
                "盘上持久了实例 3 的根槽写，择到的却是实例 {} 的根",
                effective_instance.0
            );
        }
    }
    let counter = if effective_instance == inputs.instance_three {
        INSTANCE_THREE_ROOT_CHOSEN
    } else if effective_instance == inputs.instance_two {
        INSTANCE_TWO_ROOT_CHOSEN
    } else {
        panic!(
            "择到实例 {} 的根：基镜像里最新的根是实例 2 写的 C，更旧的根不该被择",
            effective_instance.0
        );
    };
    counts.add(counter, 1);
}

// gpu-exempt: writable-mount
pub fn observe_mount_continuation(
    _inputs: &InputsOfTheObservers,
    image: &CrashImage<'_>,
    _report: &RecoveryReport,
    _counts: &mut CrashPointReplayObserverCounts,
) -> ImageAfterTheObserver {
    let mount_parameters = parameters();
    let materialized = materialized_crash_image(image);
    let stream = SharedStream::retaining_contents_and_reads();
    let mut devices: Vec<_> = materialized
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

    let mut mounted = mount_writable(&mount_parameters, &mut devices)
        .unwrap_or_else(|error| panic!("每个崩溃状态上实例 4 可写挂载接管成功：{error:?}"));
    let landed = mounted
        .current
        .file_version()
        .expect("挂载落到有效版本")
        .clone();

    let fourth_instance_takeover_overwrite = {
        let mut writer = PoolWriter::new(&mount_parameters, devices.as_mut_slice());
        publish_overwrite(
            &mut writer,
            &mut mounted.allocator,
            &landed,
            FirstFile {
                content: CONTENT_OF_FOURTH_INSTANCE_TAKEOVER,
                write_time_seconds: FIXED_WRITE_TIME_SECONDS + 240,
            },
            mounted.output.instance,
        )
        .unwrap_or_else(|error| panic!("实例 4 覆盖写 F：{error:?}"))
    };
    assert_eq!(
        fourth_instance_takeover_overwrite.root.instance,
        mounted.output.instance
    );
    assert_eq!(
        content_of_the_file(&devices),
        CONTENT_OF_FOURTH_INSTANCE_TAKEOVER,
        "实例 4 发布之后读回它写的内容"
    );

    let disks_after_the_observer = MemoryPool {
        devices: devices
            .iter()
            .map(|(identity, device)| (*identity, device.wrapped_device().image.clone()))
            .collect(),
        device_size_in_bytes: materialized.device_size_in_bytes,
    };
    let (writes_on_the_crash_image, _segments_of_the_mount) =
        writes_and_segments(&stream.retained_operations(), &geometry());

    ImageAfterTheObserver {
        writes_on_the_crash_image,
        disks_after_the_observer,
        ranges_read_on_the_crash_image: ReadRangeOnTheCrashImage::of_the_recorded_read_ranges(
            &stream.read_ranges(),
        ),
        invariants_judged: InvariantsJudgedOnTheImageAfterTheObserver::EveryImplementedInvariant,
        counter_when_the_pool_checker_is_clean: CONTINUATION_MOUNT_SUCCEEDED.to_string(),
    }
}

fn enumerate(
    item_label: &str,
    recorded: &RecordedThreeGenerationStream,
    expansion: &dyn Fn(usize, &[usize]) -> CrashPointReplaySegmentExpansion,
) -> StageOutcome<CrashPointReplayTally> {
    let inputs = &recorded.inputs;
    let observe_root = |_ordinal: u64,
                        image: &CrashImage<'_>,
                        report: &RecoveryReport,
                        counts: &mut CrashPointReplayObserverCounts| {
        observe_root_fork_isolation(inputs, image, report, counts);
    };
    let observe_continuation =
        |_ordinal: u64,
         image: &CrashImage<'_>,
         report: &RecoveryReport,
         counts: &mut CrashPointReplayObserverCounts| {
            observe_mount_continuation(inputs, image, report, counts)
        };

    let item_name = format!("{FLOW_NAME_AT_THE_ENUMERATION}::{item_label}");
    enumerate_crash_states_with_every_kind_of_observer(
        CrashPointReplayItem {
            name: &item_name,
            setting_file: PATH_OF_THIS_TEST_FILE,
        },
        &recorded.base,
        &recorded.writes,
        &recorded.segments,
        recorded.judged_root_index,
        CrashPointReplayVersionTable::with_a_file_only(&recorded.versions),
        expansion,
        &[CrashPointReplayObserver {
            name: "observe_root_fork_isolation",
            extra_inputs_digest: inputs.digest(),
            observe: &observe_root,
        }],
        &[CrashPointReplayObserverHandingBackAnImage {
            name: "observe_mount_continuation",
            extra_inputs_digest: inputs.digest(),
            observe: &observe_continuation,
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
            .count_named(CONTINUATION_MOUNT_SUCCEEDED),
        tally.states,
        "每个状态上实例 4 接管、发布之后的盘都过池级 checker"
    );
    let instance_two_chosen = tally.observer_counts.count_named(INSTANCE_TWO_ROOT_CHOSEN);
    let instance_three_chosen = tally
        .observer_counts
        .count_named(INSTANCE_THREE_ROOT_CHOSEN);
    assert_eq!(
        instance_two_chosen + instance_three_chosen,
        tally.states,
        "每个状态择到的根都过了分叉核（择根为空、择到另一侧的根都在观察者里判红）"
    );
    singlefs_verification::crash_state_enumeration::assert_the_count_is_above_zero_unless_narrowed(
        instance_two_chosen,
        &format!(
            "实例 2 的 C 有状态择到：实例 2 {instance_two_chosen}、实例 3 {instance_three_chosen}"
        ),
    );
    singlefs_verification::crash_state_enumeration::assert_the_count_is_above_zero_unless_narrowed(
        instance_three_chosen,
        &format!(
            "实例 3 的根有状态择到：实例 2 {instance_two_chosen}、实例 3 {instance_three_chosen}"
        ),
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
fn full_enumeration_of_the_three_generation_mount_fork_stream_is_exhaustive_and_clean() {
    let recorded = record_the_stream();
    let Some(tally) =
        judged_tally_or_left_to_another_stage(enumerate("full", &recorded, &full_expansion))
    else {
        return;
    };
    let closed_form = crash_state_count_with_torn_in_place_overwrites(
        &recorded.base,
        &recorded.writes,
        &recorded.segments,
        &full_expansion,
    );
    println!(
        "CRASH_POINT_REPLAY_THREE_GENERATION_MOUNT_FORK states={} closed_form={} exhaustive={} violations={} observer {:?} checker {} first_violation={}",
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
        "three_generation_mount_fork_stream：用例自己的展开下的闭式",
    );
}

#[test]
fn crash_amplification_of_the_three_generation_mount_fork_stream_is_clean() {
    let recorded = record_the_stream();
    let crash_points = common_crash_points::one_crash_point_over_every_write(
        file!(),
        FLOW_NAME_AT_THE_AMPLIFICATION,
        recorded.writes.len(),
    );
    common_crash_points::amplify_prepared_flow(
        FLOW_NAME_AT_THE_AMPLIFICATION,
        &recorded.base,
        &recorded.writes,
        &recorded.segments,
        recorded.judged_root_index,
        &recorded.versions,
        crash_points,
        singlefs_verification::crash_amplification::EXPAND_UP_TO_WRITES_PER_SEGMENT_OF_THE_AMPLIFICATION,
    );
}

/// 录下的流有名字说的三样：实例 3 挂在 D 发布中途断电的盘上、落到比实例 2 那一侧最后落盘的根更旧的 C；
/// 两侧各写各的，实例 2 分叉之后的内容在实例 3 这一侧看不到；根环三个区都有根槽写落在写过根的槽上。只录流，不枚举。
// crash-case-check:not-a-crash-case 只录流、核录下的流有分叉与根环绕回的形状，不枚举（用例文档写明），枚举在同文件登记的 crash-case:three-generation-mount-fork-stream
#[test]
fn the_recorded_stream_forks_off_a_publish_cut_by_the_power_loss_and_wraps_the_root_ring() {
    let recorded = record_the_stream();
    let shape = &recorded.shape;
    assert!(
        shape.unit_writes_of_the_publish_cut_by_the_power_loss > 0,
        "断电那一刻 D 的单元写落了盘（发布中途）：{shape:?}"
    );
    assert_eq!(
        shape.root_landed_on_by_the_third_instance.0, recorded.inputs.instance_two,
        "实例 3 挂载落到实例 2 写的一版：{shape:?}"
    );
    assert!(
        shape.root_landed_on_by_the_third_instance.1
            < shape.root_of_the_cut_publish_on_the_side_of_instance_two.1
            && shape.root_of_the_cut_publish_on_the_side_of_instance_two.1
                < shape.last_root_on_the_side_of_instance_two.1,
        "实例 3 落到的根比实例 2 那一侧发完 D 的根、最后落盘的根都旧：{shape:?}"
    );
    assert_eq!(
        shape.content_read_on_the_side_of_instance_two, CONTENT_OF_SECOND_INSTANCE_AFTER_THE_FORK,
        "实例 2 那一侧读回它分叉之后写的内容"
    );
    assert_eq!(
        shape.content_read_on_the_side_of_instance_three,
        CONTENT_OF_THIRD_INSTANCE_FORKED_OVERWRITE,
        "实例 3 这一侧读回实例 3 写的内容，看不到 D 与实例 2 分叉之后写的"
    );
    assert!(
        recorded.versions.iter().all(|version| {
            version.content != CONTENT_OF_SECOND_INSTANCE_SECOND_OVERWRITE
                && version.content != CONTENT_OF_SECOND_INSTANCE_AFTER_THE_FORK
        }),
        "实例 3 这一侧的版本表里没有实例 2 分叉之后的版本"
    );
    assert!(
        shape
            .roots_of_instance_two_in_the_recorded_stream
            .is_empty(),
        "实例 3 写的录制流里没有实例 2 的根：{shape:?}"
    );
    assert!(
        shape.root_slot_reuses_by_region.len()
            == usize::try_from(ROOT_RING_REGIONS).expect("三个区")
            && shape
                .root_slot_reuses_by_region
                .iter()
                .all(|reuses| *reuses > 0),
        "根环三个区都有根槽写落在写过根的槽上（绕回）：{shape:?}"
    );
}

/// 只录流，按写流算全量状态数，不枚举、不开判定库：录法变了而 `FULL_STATE_COUNT` 没跟着改就红。
// crash-case-check:not-a-crash-case 只录流、按写流算全量状态数，不枚举、不开判定库（用例文档写明）；枚举在同文件登记的 crash-case:three-generation-mount-fork-stream
#[test]
fn the_recorded_stream_counts_its_crash_states() {
    let recorded = record_the_stream();
    let states = crash_state_count_with_torn_in_place_overwrites(
        &recorded.base,
        &recorded.writes,
        &recorded.segments,
        &full_expansion,
    );
    println!(
        "CRASH_ENUMERATION_STATE_COUNT name={FLOW_NAME_AT_THE_ENUMERATION} states={states} writes={} segments={:?}",
        recorded.writes.len(),
        recorded.segments.iter().map(Vec::len).collect::<Vec<_>>()
    );
    assert_eq!(states, FULL_STATE_COUNT, "录制流的全量状态数");
}

/// 观察者的版本表与录制流对得上（择根核按它认每个 txg 是哪一代写的）：分叉另一侧那两版的 txg 在这一侧是实例 3 写的。
// crash-case-check:not-a-crash-case 只读录下的流核观察者输入里点名的根是这一侧的版本，不枚举；枚举在同文件登记的 crash-case:three-generation-mount-fork-stream
#[test]
fn the_roots_named_in_the_inputs_of_the_observers_are_the_versions_of_this_side() {
    let recorded = record_the_stream();
    let shape = &recorded.shape;
    let generation_of = |checkpoint_txg: CheckpointTxg| {
        recorded
            .inputs
            .generation_of_each_root_on_this_side
            .iter()
            .find(|(txg, _instance)| *txg == checkpoint_txg)
            .map(|(_txg, instance)| *instance)
    };
    assert_eq!(
        generation_of(shape.root_of_the_cut_publish_on_the_side_of_instance_two.1),
        Some(recorded.inputs.instance_three),
        "D 在实例 2 那一侧的 txg，在这一侧是实例 3 写的：{shape:?}"
    );
    assert_eq!(
        generation_of(shape.root_landed_on_by_the_third_instance.1),
        Some(recorded.inputs.instance_two),
        "实例 3 落到的那一版是实例 2 写的：{shape:?}"
    );
}
