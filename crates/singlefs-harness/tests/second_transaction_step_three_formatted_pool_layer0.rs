//! 只做过 mkfs 的池上的可写挂载（里程碑「第二个事务」步 3 的另一格，2026-09-17 用户定）在层 0 的证据：mkfs → 进程退出、重开走可写挂载
//! （取号 1、零单元的写行发布 txg 1 与暖机 txg 2）→ 同一个进程里发布第一个文件版本（txg 3），把 mkfs 之后的整条录制流按 D13（验证路线） 已定项 4
//! 取崩溃状态，快的那条（18 写的那一段不展开）跑恢复 + 多版本 oracle、池级 checker、记录核对器。
//! 这条流的基线、写表（带内容）、段序列与 mkfs 同一个进程里跑第一个事务那条流逐项相同（用例钉住），全量枚举就是第一个事务那条流的全量，不另跑。

mod common;

use common::{
    build_pool, file_content, format_pool, geometry, parameters, FIXED_WRITE_TIME_SECONDS,
};
use singlefs_core::address::{CheckpointTxg, InstanceGeneration};
use singlefs_core::mount::mount_writable;
use singlefs_core::transaction::{publish_first_file, FirstFile, PoolWriter};
use singlefs_harness::crash::{
    closed_form_state_count, enumerate_layer0_selecting_versions, writes_and_segments, Layer0Tally,
    MemoryPool, PublishedVersion, RetainedWrite,
};
use singlefs_harness::segments::StepKind;

struct Prepared {
    base: MemoryPool,
    writes: Vec<RetainedWrite>,
    segments: Vec<Vec<usize>>,
    /// 被判的那次根槽 FUA 写：第一个文件版本的根。
    judged_root_index: usize,
    versions: Vec<PublishedVersion>,
}

fn prepare(tag: &str) -> Prepared {
    let mut formatted = format_pool(tag);
    let mut devices = formatted.reopen_recorded();
    let mounted = mount_writable(&parameters(), &mut devices).expect("只做过 mkfs 的池可写挂载");
    let mut allocator = mounted.allocator;
    let content = file_content();
    {
        let publish_parameters = parameters();
        let mut writer = PoolWriter::new(&publish_parameters, devices.as_mut_slice());
        publish_first_file(
            &mut writer,
            &mut allocator,
            mounted.current.root(),
            FirstFile {
                content: &content,
                write_time_seconds: FIXED_WRITE_TIME_SECONDS,
            },
            InstanceGeneration(1),
            mounted.current.record_bytes(),
        )
        .expect("第一个文件版本");
    }
    formatted.devices = Some(devices);
    let base = formatted.memory_pool_after_mkfs();
    let operations = formatted.retained_operations();
    let (writes, segments) =
        writes_and_segments(&operations[formatted.mkfs_operation_count..], &geometry());
    assert_eq!(
        segments.iter().map(Vec::len).collect::<Vec<_>>(),
        vec![2, 2, 1, 2, 2, 1, 18, 2, 1, 2],
        "取号两写 | 写行发布的记录两写 | 根 | 超级块两写 | 暖机的记录两写 | 根 | 超级块两写与第一个文件版本的十六个单元写 | 记录两写 | 根 | 超级块两写"
    );
    assert_eq!(
        writes.len(),
        33,
        "取号 2 + 两次零单元发布各 5 + 文件版本 21"
    );
    let root_indexes: Vec<usize> = writes
        .iter()
        .enumerate()
        .filter(|(_, write)| write.kind == StepKind::RootRecordFua)
        .map(|(index, _)| index)
        .collect();
    assert_eq!(root_indexes.len(), 3, "txg 1、2、3 各一条根槽写");
    Prepared {
        base,
        writes,
        segments,
        judged_root_index: *root_indexes.last().expect("三条根槽写"),
        versions: vec![PublishedVersion {
            instance: InstanceGeneration(1),
            checkpoint_txg: CheckpointTxg(3),
            content,
        }],
    }
}

fn assert_checker_and_record_checker_clean(tally: &Layer0Tally) {
    assert_eq!(
        (
            tally.record_root_without_record,
            tally.record_claimed_state_missing_unit
        ),
        (0, 0),
        "记录核对器两条判据在已定的持久顺序下恒 0"
    );
    for invariant in singlefs_checker::image::IMPLEMENTED_INVARIANTS {
        assert_eq!(
            tally
                .checker_violated_states
                .get(invariant)
                .copied()
                .unwrap_or(0),
            0,
            "{invariant} 判违例的状态数必须是 0：{:?}",
            tally.checker_first_violation.get(invariant)
        );
    }
    for must_evaluate in ["I-3.1", "I-5.2", "I-5.1", "I-5.4", "I-7.2", "I-7.7"] {
        assert!(
            tally
                .checker_evaluated_states
                .get(must_evaluate)
                .copied()
                .unwrap_or(0)
                > 0,
            "{must_evaluate} 至少在一个状态上真被评估过"
        );
    }
}

/// 段序列：十段、闭式 262165（1 + 六个 2 写段各 3 + 三个 1 写段各 1 + 一个 18 写段 262143），与 mkfs 同一个进程里跑第一个事务那条流相同。
#[test]
fn the_formatted_pool_mount_and_first_file_stream_keeps_the_first_transaction_segment_sequence() {
    let prepared = prepare("formatted-layer0-registered");
    assert_eq!(prepared.segments.len(), 10);
    assert_eq!(closed_form_state_count(&prepared.segments), 262_165);
}

/// 平时跑的那一份：18 写的那一段不展开（只以整段持久进入后面的状态），其余每段任意子集。
#[test]
fn every_crash_state_outside_the_unit_segment_of_the_formatted_pool_mount_recovers_to_the_version_its_root_claims(
) {
    let prepared = prepare("formatted-layer0-fast");
    let expand = |_segment_index: usize, segment: &[usize]| segment.len() < 10;
    let tally = enumerate_layer0_selecting_versions(
        &prepared.base,
        &prepared.writes,
        &prepared.segments,
        prepared.judged_root_index,
        &prepared.versions,
        &expand,
    );
    let expanded: Vec<Vec<usize>> = prepared
        .segments
        .iter()
        .filter(|segment| segment.len() < 10)
        .cloned()
        .collect();
    assert_eq!(
        tally.states,
        closed_form_state_count(&expanded),
        "展开的段按闭式数"
    );
    assert_eq!(tally.states, 22, "1 + 六个 2 写段各 3 + 三个 1 写段各 1");
    assert_eq!(
        tally.violations, 0,
        "第一处违例：{:?}",
        tally.first_violation
    );
    assert_eq!(
        tally.ignored_violations, 0,
        "不看 journal 那一遍恢复同样过 oracle：{:?}",
        tally.first_ignored_violation
    );
    assert_eq!(tally.failed_states, 0);
    assert!(tally.file_read_states > 0 && tally.no_file_states > 0);
    assert_checker_and_record_checker_clean(&tally);
}

/// 这条流的崩溃状态与第一个事务那条流逐个相同，所以不另跑全量：只做过 mkfs 的池可写挂载再发第一个文件版本，与 mkfs 同一个进程里
/// 第一个事务，两条流的基线镜像（mkfs 之后）、写表（带内容）、段序列逐项相等——层 0 按「基线 + 写表 + 段」枚举，三样相等就是同一批镜像，
/// 第一个事务那条流的全量枚举（`first_transaction_step_seven_layer0.rs`，门禁 54 号）已经罩住（m2-emptypool-nonempty-r1 云端攻方腿 Z4）。
/// 层 0 每个状态只跑恢复与 checker、不重开挂载，挂载路径在崩溃之后怎样不在这条证据里。
#[test]
fn formatted_pool_mount_stream_has_the_same_base_writes_and_segments_as_the_first_transaction_stream(
) {
    let prepared = prepare("formatted-layer0-same-as-first-transaction");
    let reference = build_pool("formatted-layer0-first-transaction-reference");
    let (reference_writes, reference_segments) = writes_and_segments(
        &reference.retained_operations()[reference.mkfs_operation_count..],
        &geometry(),
    );
    assert!(
        prepared.base == reference.memory_pool_after_mkfs(),
        "mkfs 之后的基线镜像相同"
    );
    assert_eq!(prepared.writes.len(), reference_writes.len());
    for (index, (write, reference_write)) in
        prepared.writes.iter().zip(&reference_writes).enumerate()
    {
        assert!(
            write == reference_write,
            "写表第 {index} 条（{:?}）连同内容相同",
            write.kind
        );
    }
    assert_eq!(prepared.segments, reference_segments, "段序列相同");
}
