//! I-7.8（根记录树 ID 水位不低于全池最大树 ID） 扫描方向的改法 D（`research/prompts/m2-wave3-code-r1-main-verification.md`
//! 第三节 Y1、第四节第 1 条，被攻过零轮）：第一个文件版本那次发布崩在录制流的每一个前缀上 → 可写挂载 → 挂载之后六种动作，
//! 池级 checker 一条违例都没有。两条流：
//! ① mkfs → 取号 → 暖机 → 第一个文件版本（从水位 11 发八棵树）；
//! ② ① 跑完之后回退到暖机根 (1, 2)（回退行 (1, 2, 0)），在回退那个会话里再发第一个文件版本（从带过来的水位 19 发）。
//!
//! 崩在记录落盘之前的那几个前缀上，盘上留着那次发布写出的码 2 节点（孤儿），头里的树 ID 不低于根环里的水位；
//! 下一个实例第一次发布取的 txg 等于孤儿的诞生代号（D23（journal 的角色与格式） 已定项 14 注 3 的「≥ max(…) + 1」取等号），
//! 只按「诞生代号 ≤ 根环最大 txg」数的读法就把孤儿数进「出现过的」、判红——而实例表行 (崩掉的实例, Ti, ·) 说它只被施加到 Ti，
//! 诞生代号 > Ti 的节点没发布过（`.claude/kb/invariants.md` I-7.8 那一行的 checker 读法注：崩在发布之前的孤儿不算「出现过」）。
//! 用例同时数出改法 D 之前那个读法会在几格上红（[`the_birth_txg_only_reading_would_redden`]），每条流都要大于 0：
//! 那个数是 0 就说明扫描根本没走到孤儿留在盘上的状态，「全绿」不算数。

mod common;

use common::{
    crash_state_devices, memory_pool_of_sparse_devices, parameters, FIXED_WRITE_TIME_SECONDS,
    IMAGE_BYTES,
};
use singlefs_checker::image::InvariantVerdict;
use singlefs_checker::index_node_view;
use singlefs_checker::walk::check_pool_image;
use singlefs_core::address::{
    CheckpointTxg, DeviceIdentity, DeviceOffsetInBytes, InstanceGeneration,
};
use singlefs_core::allocator::{DeviceFreeMap, Placement, PoolAllocator};
use singlefs_core::journal::back_chain_of;
use singlefs_core::make_filesystem::{
    make_filesystem, INSTANCE_TABLE_SLOT, TREE_TABLE_GENESIS_SLOT,
};
use singlefs_core::mount::{mount_rollback, mount_writable, Mounted, RollbackTarget, ShadowLedger};
use singlefs_core::recovery::{readable_roots, PoolReader};
use singlefs_core::transaction::{
    acquire_instance, publish_first_file, publish_without_units, warm_up, FirstFile, PoolVersion,
    PoolWriter, ZeroUnitPublishPlan,
};
use singlefs_format::{NODE_BYTES, UNIT_AREA_START_SLOT};
use singlefs_harness::crash::{MemoryPool, SparseBlockDevice};
use singlefs_harness::{RecordingBlockDevice, SharedStream};

type RecordedMemoryDevice = RecordingBlockDevice<SparseBlockDevice>;

fn content_of(length: usize, seed: usize) -> Vec<u8> {
    (0..length)
        .map(|index| u8::try_from((index * 7 + seed) % 253).expect("小于 256"))
        .collect()
}

fn empty_memory_pool() -> MemoryPool {
    MemoryPool::with_devices(&[DeviceIdentity(0), DeviceIdentity(1)], IMAGE_BYTES)
}

/// 一条录制流里第一个文件版本那次发布占的一段：流里第 `start` 步起、到第 `end` 步之前。
/// 前缀 k ∈ [start, end] 是「崩在这次发布之前、之中、刚做完」的每一个崩溃状态。
struct FirstFilePublishInTheStream {
    stream: SharedStream,
    start: usize,
    end: usize,
}

/// 流 ①：mkfs → 取号（实例 1）→ 暖机 (1, 1)、(1, 2) → 第一个文件版本 (1, 3)，同一个进程、内存盘。交回那段发布与跑完之后的盘。
fn first_file_publish_after_mkfs() -> (
    FirstFilePublishInTheStream,
    Vec<(DeviceIdentity, RecordedMemoryDevice)>,
) {
    let stream = SharedStream::retaining_contents();
    let mut devices = crash_state_devices(&empty_memory_pool(), &[], &[], &stream);
    let publish_parameters = parameters();
    let genesis = make_filesystem(&publish_parameters, &mut devices).expect("mkfs");
    let mut allocator = PoolAllocator::new(vec![
        DeviceFreeMap::new(DeviceIdentity(0), IMAGE_BYTES),
        DeviceFreeMap::new(DeviceIdentity(1), IMAGE_BYTES),
    ]);
    allocator.mark_format_time_units(
        Placement {
            slot: INSTANCE_TABLE_SLOT,
            span: 2,
        },
        Placement {
            slot: TREE_TABLE_GENESIS_SLOT,
            span: 1,
        },
    );
    let (start, end) = {
        let mut writer = PoolWriter::new(&publish_parameters, &mut devices);
        let instance = acquire_instance(&mut writer).expect("取号");
        let warm = warm_up(&mut writer, &genesis.root, instance).expect("暖机");
        let start = stream.operation_count();
        let first_file = publish_first_file(
            &mut writer,
            &mut allocator,
            warm.roots.last().expect("暖机两代根"),
            FirstFile {
                content: &content_of(3000, 1),
                write_time_seconds: FIXED_WRITE_TIME_SECONDS,
            },
            instance,
            &warm.last_record_bytes,
        )
        .expect("第一个文件版本");
        assert_eq!(
            (
                first_file.root.checkpoint_txg,
                first_file.root.tree_identifier_watermark
            ),
            (CheckpointTxg(3), 19),
            "流 ①：第一个文件版本 (1, 3) 从 mkfs 的水位 11 发 11..18、新水位 19"
        );
        (start, stream.operation_count())
    };
    (FirstFilePublishInTheStream { stream, start, end }, devices)
}

/// 流 ②：流 ① 跑完之后回退到暖机根 (1, 2)（取号 2、回退行 (1, 2, 0)，带过来根环里的水位 19），在回退那个会话里
/// 接着现行那一版（树表 0 条）再发第一个文件版本。交回的那段是回退之后那次第一个文件版本的发布。
fn first_file_publish_after_rolling_back_to_the_warm_up_root() -> FirstFilePublishInTheStream {
    let (after_mkfs, mut devices) = first_file_publish_after_mkfs();
    let rolled_back = mount_rollback(
        &parameters(),
        &mut devices,
        RollbackTarget {
            instance: InstanceGeneration(1),
            checkpoint_txg: CheckpointTxg(2),
        },
        ShadowLedger::On,
    )
    .expect("环里还留着带文件版本的根 (1, 3) 时回退到暖机根照常做");
    let PoolVersion::WithoutFile(version) = &rolled_back.current else {
        panic!("回退到树表 0 条的暖机根：现行那一版仍是「没有文件版本」的一版")
    };
    assert_eq!(
        version.root.tree_identifier_watermark, 19,
        "回退那一版带根环里的水位 max 19（D8（核心索引结构） 已定项 8 ②）"
    );
    let mut allocator = rolled_back.allocator.clone();
    let publish_parameters = parameters();
    let mut writer = PoolWriter::new(&publish_parameters, &mut devices);
    let start = after_mkfs.stream.operation_count();
    let first_file = publish_first_file(
        &mut writer,
        &mut allocator,
        &version.root,
        FirstFile {
            content: &content_of(3300, 17),
            write_time_seconds: FIXED_WRITE_TIME_SECONDS + 120,
        },
        rolled_back.output.instance,
        &version.record_bytes,
    )
    .expect("回退之后第一个文件版本");
    assert_eq!(
        first_file.root.tree_identifier_watermark, 27,
        "流 ②：回退之后第一个文件版本从 19 发 19..26、新水位 27"
    );
    let end = after_mkfs.stream.operation_count();
    FirstFilePublishInTheStream {
        stream: after_mkfs.stream,
        start,
        end,
    }
}

/// 可写挂载之后用户接着做的事（Y1-a 攻方驱动 `opus_attack_y1.rs` 的 `after_mount` 那六种）。
#[derive(Clone, Copy, Debug)]
enum ActionAfterTheMount {
    Nothing,
    /// 接着现行那一版连推几次零单元发布（只对树表 0 条的一版做得了）。
    ZeroUnitPublishes {
        count: u64,
    },
    /// 退出、重开再可写挂载一次。
    RemountWritable,
    /// 接着现行那一版发第一个文件版本（只对树表 0 条的一版做得了）。
    FirstFileVersion,
}

const ACTIONS_AFTER_THE_MOUNT: [ActionAfterTheMount; 6] = [
    ActionAfterTheMount::Nothing,
    ActionAfterTheMount::ZeroUnitPublishes { count: 1 },
    ActionAfterTheMount::ZeroUnitPublishes { count: 2 },
    ActionAfterTheMount::ZeroUnitPublishes { count: 3 },
    ActionAfterTheMount::RemountWritable,
    ActionAfterTheMount::FirstFileVersion,
];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ActionOutcome {
    Performed,
    /// 挂载择到的现行那一版已经带文件（崩在根槽落盘之后）：零单元发布与第一个文件版本都接不上它。
    NotApplicableToAVersionWithAFile,
}

fn perform_after_the_mount(
    action: ActionAfterTheMount,
    devices: &mut Vec<(DeviceIdentity, RecordedMemoryDevice)>,
    mounted: &Mounted,
) -> ActionOutcome {
    let publish_parameters = parameters();
    match (action, &mounted.current) {
        (ActionAfterTheMount::Nothing, _) => ActionOutcome::Performed,
        (ActionAfterTheMount::RemountWritable, _) => {
            mount_writable(&publish_parameters, devices).expect("再可写挂载一次");
            ActionOutcome::Performed
        }
        (ActionAfterTheMount::ZeroUnitPublishes { count }, PoolVersion::WithoutFile(version)) => {
            let mut current = version.clone();
            for _ in 0..count {
                let mut writer = PoolWriter::new(&publish_parameters, devices.as_mut_slice());
                current = publish_without_units(
                    &mut writer,
                    &current.root,
                    ZeroUnitPublishPlan {
                        txg: CheckpointTxg(current.root.checkpoint_txg.0 + 1),
                        counter: current.record.counter + 1,
                        instance: mounted.output.instance,
                        back_chain: back_chain_of(&current.record_bytes),
                        rollback_floor: current.root.rollback_floor,
                        tree_identifier_watermark: current.root.tree_identifier_watermark,
                    },
                )
                .expect("零单元发布");
            }
            ActionOutcome::Performed
        }
        (ActionAfterTheMount::FirstFileVersion, PoolVersion::WithoutFile(version)) => {
            let mut allocator = mounted.allocator.clone();
            let mut writer = PoolWriter::new(&publish_parameters, devices.as_mut_slice());
            publish_first_file(
                &mut writer,
                &mut allocator,
                &version.root,
                FirstFile {
                    content: &content_of(2000, 5),
                    write_time_seconds: FIXED_WRITE_TIME_SECONDS + 999,
                },
                mounted.output.instance,
                &version.record_bytes,
            )
            .expect("挂载之后第一个文件版本");
            ActionOutcome::Performed
        }
        (
            ActionAfterTheMount::ZeroUnitPublishes { .. } | ActionAfterTheMount::FirstFileVersion,
            PoolVersion::WithFile(_),
        ) => ActionOutcome::NotApplicableToAVersionWithAFile,
    }
}

/// 扫孤儿时从单元区起点往后看多少个槽：两条流里每次发布的落点都在前几百个槽里。
const SLOTS_SCANNED_FOR_INDEX_NODES: u64 = 1024;
/// 单元头读多少就认得出是不是码 2：magic 4 + 类标签在偏移 6。
const HEADER_BYTES_READ_FIRST: usize = 512;

/// 改法 D 之前那个读法（扫描方向只按「诞生代号 ≤ 根环最大 txg」数码 2 节点）在这张镜像上会不会判红：
/// 盘 0 单元区里有没有一个两道校验和都过的码 2 节点，诞生代号不超过根环里最大的 txg、树 ID 不低于根环里最大的水位。
/// 用例拿它数「孤儿真留在盘上、而水位没盖住它」的格子，与 checker 是两份独立的算法（这里用核心层的择根读根、checker 的码 2 解析）。
fn the_birth_txg_only_reading_would_redden(image: &MemoryPool) -> bool {
    let publish_parameters = parameters();
    let roots = readable_roots(
        image,
        &publish_parameters.region_devices,
        &publish_parameters.geometry,
        &publish_parameters.filesystem_identifier,
    );
    let highest_root_txg = roots
        .iter()
        .map(|root| root.checkpoint_txg.0)
        .max()
        .expect("可写挂载之后根环里至少有一条根");
    let highest_watermark = roots
        .iter()
        .map(|root| root.tree_identifier_watermark)
        .max()
        .expect("可写挂载之后根环里至少有一条根");
    let node_bytes = usize::try_from(NODE_BYTES).expect("16384");
    (UNIT_AREA_START_SLOT..UNIT_AREA_START_SLOT + SLOTS_SCANNED_FOR_INDEX_NODES).any(|slot| {
        let offset = DeviceOffsetInBytes(slot * NODE_BYTES);
        let is_index_node = image
            .read(DeviceIdentity(0), offset, HEADER_BYTES_READ_FIRST)
            .is_some_and(|header| &header[..4] == b"SFSU" && header[6] == 2);
        is_index_node
            && image
                .read(DeviceIdentity(0), offset, node_bytes)
                .and_then(|bytes| index_node_view(&bytes).ok())
                .is_some_and(|node| {
                    node.birth_txg <= highest_root_txg && node.tree_identifier >= highest_watermark
                })
    })
}

/// 一次扫完数出来的格子：每格一个 (前缀, 动作)。
#[derive(Debug, Default)]
struct SweepTally {
    cells: usize,
    cells_performed: usize,
    cells_where_the_birth_txg_only_reading_reddens: usize,
    red_cells: Vec<String>,
}

/// 那段发布的每一个前缀 × 挂载之后的每一种动作：崩溃镜像 → 可写挂载 → 那个动作 → 池级 checker。
/// 每一格的违例都收下来，扫完再判（改法 D 被拿掉时报的是一共红了几格、红在哪条上）。
fn sweep_every_crash_inside_the_publish(
    publish: &FirstFilePublishInTheStream,
    stream_name: &str,
) -> SweepTally {
    let operations = publish.stream.retained_operations();
    let mut tally = SweepTally::default();
    for prefix in publish.start..=publish.end {
        let mut crash_image = empty_memory_pool();
        crash_image.apply(&operations[..prefix]);
        for action in ACTIONS_AFTER_THE_MOUNT {
            tally.cells += 1;
            let side_stream = SharedStream::new();
            let mut devices = crash_state_devices(&crash_image, &[], &[], &side_stream);
            let mounted = mount_writable(&parameters(), &mut devices).unwrap_or_else(|error| {
                panic!("{stream_name} 前缀 {prefix}：可写挂载要挂得上：{error:?}")
            });
            if perform_after_the_mount(action, &mut devices, &mounted) == ActionOutcome::Performed {
                tally.cells_performed += 1;
            }
            let image = memory_pool_of_sparse_devices(&devices);
            if the_birth_txg_only_reading_would_redden(&image) {
                tally.cells_where_the_birth_txg_only_reading_reddens += 1;
            }
            let violated: Vec<String> = check_pool_image(&image)
                .into_iter()
                .filter_map(|(invariant, verdict)| match verdict {
                    InvariantVerdict::Violated(detail) => Some(format!("{invariant}：{detail}")),
                    InvariantVerdict::Holds | InvariantVerdict::NotApplicable(_) => None,
                })
                .collect();
            if !violated.is_empty() {
                tally
                    .red_cells
                    .push(format!("前缀 {prefix}、挂载之后 {action:?}：{violated:?}"));
            }
        }
    }
    tally
}

fn assert_the_sweep_stays_green_and_reached_the_orphans(tally: &SweepTally, stream_name: &str) {
    assert!(
        tally.red_cells.is_empty(),
        "{stream_name}：{} 格里红了 {} 格（第一格：{:?}）",
        tally.cells,
        tally.red_cells.len(),
        tally.red_cells.first()
    );
    assert!(
        tally.cells_where_the_birth_txg_only_reading_reddens > 0,
        "{stream_name}：扫了 {} 格（做了动作的 {} 格），一格都没走到「孤儿留在盘上、树 ID 不低于根环水位」的状态——全绿不算数",
        tally.cells,
        tally.cells_performed
    );
}

/// 流 ①：崩在第一个文件版本那次发布的每一个前缀上、可写挂载、再做那六种动作，池级 checker 一条违例都没有；
/// 改法 D 之前的读法在其中有格子判红（孤儿 11..15、水位 11）。
#[test]
fn every_crash_inside_the_first_file_version_after_mkfs_then_a_writable_mount_leaves_the_orphans_out_of_the_tree_identifier_watermark(
) {
    let (publish, _) = first_file_publish_after_mkfs();
    let tally = sweep_every_crash_inside_the_publish(&publish, "流 ①（mkfs 之后）");
    assert_the_sweep_stays_green_and_reached_the_orphans(&tally, "流 ①（mkfs 之后）");
}

/// 流 ②：回退到暖机根之后那次第一个文件版本崩在每一个前缀上、可写挂载、再做那六种动作，池级 checker 一条违例都没有；
/// 改法 D 之前的读法在其中有格子判红（孤儿 19..23、水位 19）。回退行 (1, 2, 0) 不排除 (1, 3) 那一版写出的码 2 节点
/// （树 ID 11..15）：它们低于水位 19，照样被数进「出现过的」。
#[test]
fn every_crash_inside_the_first_file_version_after_rolling_back_to_the_warm_up_root_then_a_writable_mount_leaves_the_orphans_out_of_the_tree_identifier_watermark(
) {
    let publish = first_file_publish_after_rolling_back_to_the_warm_up_root();
    let tally = sweep_every_crash_inside_the_publish(&publish, "流 ②（回退到暖机根之后）");
    assert_the_sweep_stays_green_and_reached_the_orphans(&tally, "流 ②（回退到暖机根之后）");
}
