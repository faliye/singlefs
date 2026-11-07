//! 验证包模块：crash
//! 记录核对器第二条判据（恢复自称的 txg ≥ 某次发布、那次发布的某个单元两份都缺席）按恢复落到的那一版的实例表豁免被抛弃的发布
//! （`singlefs_verification::crash::check_records_against` 里的 `abandoned_by_the_landed_version`，判法是
//! `singlefs_checker::walk::InstanceTableOfRootRecord::abandons`：有行 (i, Ti, Wi) 且 T > Ti，D23（journal 的角色与格式） 已定项 14
//! 回退段的候选集规则）。两头各一条用例，都把写表、两份镜像与恢复自称的那一版直接交给 CPU 那一份记录核对器：
//! - 豁免要起作用的那一头：崩溃恢复抛弃实例 2 的那次覆盖写，关着影子账的那次挂载（新实例）看不见它，接着覆盖写时把它独占的槽分出去写了别的。
//!   被抛弃那次覆盖写的根还在根环里，那次复用按回收谓词证得出过不了（`reuse_is_not_proven_illegal_by_the_reclaim_predicate`），
//!   不豁免就判缺席；恢复落到的那一版（新实例最新那条根）的实例表判它被抛弃，它的单元不算「该在」，记录核对器不报缺席。
//! - 豁免不许多罩的那一头：行 (i, Ti) 那一版自己（T = Ti）不算被抛弃。同一段历史抛弃之后不再写，新实例的实例表里实例 1 那一行的 Ti
//!   就是实例 1 那次覆盖写的 txg；把那次覆盖写的数据单元两份都扣下，记录核对器照报缺席。
//!
//! 写表的取法：抛弃那一步（`common::abandon_the_newest_root_by_a_recovery_that_lands_on_the_root_before_with_the_shadow_ledger`）为了造出
//! 「崩在那次覆盖写的根槽 FUA 之后、它那次系统配置轮换之前，挂载时它的根槽与数据单元暂时读不出」，往同一条录制流里写了几步不是实现发出的写：
//! 开头把那几处清零、把每块盘见证它的系统配置槽改写成轮换之前的那一代，挂载之后把清零的那几处原样写回。这几步从写表里剔掉；
//! 剩下的是崩溃之前的历史录制流接上崩溃之后那次挂载起的录制流，在接缝处断开（`RecordStreamSeam::AfterACrash`），
//! 与崩溃注入第二截交给记录核对器的写表同形（`check_records_against` 文档「装置里各处交什么」）。

mod common;

use common::{
    abandon_the_newest_root_by_a_recovery_that_lands_on_the_root_before_with_the_shadow_ledger,
    build_pool, geometry, parameters, publish_overwrite_in_process, BuiltPool,
    FIXED_WRITE_TIME_SECONDS,
};
use singlefs_checker::walk::InstanceTableOfRootRecord;
use singlefs_core::address::{
    CheckpointTxg, DeviceIdentity, DeviceOffsetInBytes, InstanceGeneration,
};
use singlefs_core::mount::{mount_writable, Mounted, ShadowLedger};
use singlefs_core::recovery::{recover, JournalPolicy, PoolReader};
use singlefs_core::transaction::TransactionOutput;
use singlefs_verification::crash::check_records_against;
use singlefs_verification::memory_pool::{
    publishes_of_one_recording, root_identity_of_write, writes_and_segments, CrashImage,
    MemoryPool, RecordCheck, RecordStreamContinuity, RecordStreamSeam, RetainedWrite,
};
use singlefs_verification::segments::StepKind;
use singlefs_verification::RecordedOperationKind;

/// 关着影子账的那次挂载里最多接着覆盖写几次：它的分配器把被抛弃那次覆盖写独占的槽当空闲槽，接着写总会分到其中一个
/// （`crates/singlefs-verification/tests/checker_narrow_invariants_and_abandoned_roots.rs` 在同一段历史上用的同一个上限）。
const OVERWRITES_IN_THE_BLIND_MOUNT_AT_MOST: u64 = 6;

/// 抛弃之前的那段历史：池里第一个文件那一版之后实例 1 覆盖写一次（`overwrite_at_the_instance_row`），重开可写挂载取号 2、写行、暖机，
/// 实例 2 再覆盖写一次（`overwrite_to_abandon`）。搭法与 `checker_narrow_invariants_and_abandoned_roots.rs` 的
/// `pool_through_the_third_version` 同一段（那份文件是另一个测试目标，测试目标之间互相引不到，这里照抄）。
struct PoolBeforeTheAbandoningRecovery {
    pool: BuiltPool,
    /// 实例 1 的那次覆盖写：取号 2 那次挂载落在它上，写下实例 1 那一行，行里的 Ti 就是它的 txg。
    overwrite_at_the_instance_row: TransactionOutput,
    /// 实例 2 的那次覆盖写：之后的崩溃恢复抛弃它。
    overwrite_to_abandon: TransactionOutput,
}

fn pool_before_the_abandoning_recovery(tag: &str) -> PoolBeforeTheAbandoningRecovery {
    let mut pool = build_pool(tag);
    let file_version_before_the_overwrites = pool.output.clone();
    let content_at_the_instance_row: Vec<u8> = (0..4100usize)
        .map(|index| u8::try_from((index * 7 + 3) % 253).expect("小于 256"))
        .collect();
    let overwrite_at_the_instance_row = publish_overwrite_in_process(
        &mut pool,
        &file_version_before_the_overwrites,
        &content_at_the_instance_row,
        FIXED_WRITE_TIME_SECONDS + 60,
        InstanceGeneration(1),
    )
    .expect("实例 1 覆盖写");
    pool.output = overwrite_at_the_instance_row.clone();
    let mut devices_reopened_for_the_new_instance = pool.reopen_recorded();
    let mounted = mount_writable(&parameters(), &mut devices_reopened_for_the_new_instance)
        .expect("实例 1 覆盖写之后重开，可写挂载取号");
    let instance_of_the_reopened_mount = mounted.output.instance;
    pool.devices = Some(devices_reopened_for_the_new_instance);
    pool.allocator = mounted.allocator;
    let current = mounted
        .current
        .into_file_version()
        .expect("实例 1 覆盖写之后重开，现行那一版带文件");
    let content_to_abandon: Vec<u8> = (0..2500usize)
        .map(|index| u8::try_from((index * 7 + 11) % 253).expect("小于 256"))
        .collect();
    let overwrite_to_abandon = publish_overwrite_in_process(
        &mut pool,
        &current,
        &content_to_abandon,
        FIXED_WRITE_TIME_SECONDS + 120,
        instance_of_the_reopened_mount,
    )
    .expect("重开那次挂载的实例覆盖写");
    pool.output = overwrite_to_abandon.clone();
    PoolBeforeTheAbandoningRecovery {
        pool,
        overwrite_at_the_instance_row,
        overwrite_to_abandon,
    }
}

/// 抛弃那一步在录制流里的位置：它之前、之后录制流各多长，以及挂载时暂时读不出的几处（每处在开头清零一步、挂载之后写回一步）。
struct AbandoningRecoveryInTheStream {
    operations_before_the_abandoning: usize,
    operations_after_the_abandoning: usize,
    places_hidden_during_the_mount: usize,
}

/// 崩溃恢复抛弃 `abandoned`（池里最新那一版），那次挂载的影子账由调用方给；交回那次挂载与它在录制流里的位置。
/// 暂时读不出的是根槽一处加 `abandoned` 的数据单元每份一处（抛弃那一步的构造）。
fn abandon_by_a_recovery_that_lands_on_the_root_before(
    pool: &mut BuiltPool,
    abandoned: &TransactionOutput,
    shadow_ledger: ShadowLedger,
) -> (Mounted, AbandoningRecoveryInTheStream) {
    let operations_before_the_abandoning = pool.retained_operations().len();
    let mounted =
        abandon_the_newest_root_by_a_recovery_that_lands_on_the_root_before_with_the_shadow_ledger(
            pool,
            abandoned,
            shadow_ledger,
        );
    let operations_after_the_abandoning = pool.retained_operations().len();
    let data_unit_copies: usize = abandoned
        .data_pointers
        .iter()
        .map(|pointer| pointer.locations.len())
        .sum();
    (
        mounted,
        AbandoningRecoveryInTheStream {
            operations_before_the_abandoning,
            operations_after_the_abandoning,
            places_hidden_during_the_mount: 1 + data_unit_copies,
        },
    )
}

/// 崩溃之前的历史录制流（mkfs 之后到抛弃那一步之前）接上崩溃之后那次挂载起的录制流（抛弃那一步之后接着写的也在里面），
/// 剔掉抛弃那一步自己写的几步；交回写表与接缝之后第一个写在写表里的下标。
/// 剔掉的几步逐步核：开头每处一步全零的普通写、每块盘一步系统配置槽写，挂载之后每处一步普通写、落点与长度与开头清零的那几步逐步相同。
///
/// # Panics
/// 抛弃那一步在录制流里的形状与上面写的对不上（它的构造改了，这里的剔法要跟着改）。
fn writes_of_the_implementation_around_the_abandoning_recovery(
    pool: &BuiltPool,
    recording: &AbandoningRecoveryInTheStream,
) -> (Vec<RetainedWrite>, usize) {
    let operations = pool.retained_operations();
    let hidden = recording.places_hidden_during_the_mount;
    let device_count = pool.devices.as_ref().expect("镜像还开着").len();
    let zeroing_begins = recording.operations_before_the_abandoning;
    let rewriting_begins = zeroing_begins + hidden;
    let mount_begins = rewriting_begins + device_count;
    let writing_back_begins = recording.operations_after_the_abandoning - hidden;
    let zeroing = &operations[zeroing_begins..rewriting_begins];
    let writing_back = &operations[writing_back_begins..recording.operations_after_the_abandoning];
    for zero_write in zeroing {
        assert!(
            zero_write.operation.kind == RecordedOperationKind::Write
                && zero_write
                    .contents
                    .as_deref()
                    .is_some_and(|bytes| bytes.iter().all(|byte| *byte == 0)),
            "抛弃那一步开头每处一步全零的普通写：{:?}",
            zero_write.operation
        );
    }
    for rewrite in &operations[rewriting_begins..mount_begins] {
        assert_eq!(
            geometry().classify(&rewrite.operation),
            StepKind::SystemConfigurationSlot,
            "清零之后每块盘一步系统配置槽写（见证被抛弃那一版的那一槽改写成轮换之前的那一代）：{:?}",
            rewrite.operation
        );
    }
    for (zero_write, write_back) in zeroing.iter().zip(writing_back) {
        assert_eq!(
            (
                write_back.operation.kind,
                write_back.operation.device,
                write_back.operation.offset,
                write_back.operation.length
            ),
            (
                RecordedOperationKind::Write,
                zero_write.operation.device,
                zero_write.operation.offset,
                zero_write.operation.length
            ),
            "挂载之后把清零的那几处原样写回，次序与清零相同"
        );
    }
    let (history, _history_segments) = writes_and_segments(
        &operations[pool.mkfs_operation_count..zeroing_begins],
        &geometry(),
    );
    let (mount, _mount_segments) =
        writes_and_segments(&operations[mount_begins..writing_back_begins], &geometry());
    let (after_the_abandoning, _segments_after_the_abandoning) = writes_and_segments(
        &operations[recording.operations_after_the_abandoning..],
        &geometry(),
    );
    let first_write_after_the_crash = history.len();
    (
        history
            .into_iter()
            .chain(mount)
            .chain(after_the_abandoning)
            .collect(),
        first_write_after_the_crash,
    )
}

/// `pool` 此刻录制流里最新那次发布（必须是 `newest` 那一版）自己写出的单元写。
fn unit_writes_of_the_newest_publish(
    pool: &BuiltPool,
    newest: &TransactionOutput,
) -> Vec<RetainedWrite> {
    let operations = pool.retained_operations();
    let (history, _segments) =
        writes_and_segments(&operations[pool.mkfs_operation_count..], &geometry());
    let newest_publish = publishes_of_one_recording(&history, 0)
        .pop()
        .expect("mkfs 之后至少一次发布");
    assert_eq!(
        (
            InstanceGeneration(newest_publish.instance),
            CheckpointTxg(newest_publish.checkpoint_txg)
        ),
        (newest.root.instance, newest.root.checkpoint_txg),
        "录制流里最新那次发布就是 `newest` 那一版"
    );
    assert!(!newest_publish.units.is_empty(), "覆盖写那次发布写了单元");
    newest_publish
        .units
        .iter()
        .map(|unit| history[*unit].clone())
        .collect()
}

/// 这次写写下的字节还在不在 `image` 上。
fn write_is_still_on(image: &MemoryPool, write: &RetainedWrite) -> bool {
    let length = usize::try_from(write.length_in_bytes()).expect("写长装得进 usize");
    PoolReader::read(image, write.device, write.offset, length)
        .is_some_and(|bytes| write.contents.still_on_disk(&bytes))
}

/// 写表里最新那次根槽写。
fn newest_root_write(writes: &[RetainedWrite]) -> &RetainedWrite {
    writes
        .iter()
        .rev()
        .find(|write| write.kind == StepKind::RootRecordFua)
        .expect("写表里有根槽写")
}

/// 把写表、持久集合与恢复自称的那一版交给 CPU 那一份记录核对器：崩溃态镜像是接缝之前那一截按持久集合落的盘（接缝之后的写一个都不落），
/// 恢复后的镜像是整张写表按持久集合落的盘；写表在接缝处断开，接缝那一头记着那次崩溃之后恢复落到的那一版。
fn record_check_across_the_abandoning_recovery(
    base: &MemoryPool,
    writes: &[RetainedWrite],
    persisted: &[bool],
    first_write_after_the_crash: usize,
    version_landed_on_after_the_crash: (InstanceGeneration, CheckpointTxg),
    effective_root: (InstanceGeneration, CheckpointTxg),
) -> RecordCheck {
    let crash_state_image = CrashImage {
        base,
        writes,
        persisted: persisted
            .iter()
            .enumerate()
            .map(|(index, is_persisted)| *is_persisted && index < first_write_after_the_crash)
            .collect(),
    };
    let recovered_image = CrashImage {
        base,
        writes,
        persisted: persisted.to_vec(),
    };
    let seams = [RecordStreamSeam::AfterACrash {
        first_write_after_the_crash,
        version_landed_on_after_the_crash: Some(version_landed_on_after_the_crash),
    }];
    check_records_against(
        &crash_state_image,
        &recovered_image,
        writes,
        persisted,
        RecordStreamContinuity::BrokenAtSeams(&seams),
        Some(effective_root),
    )
}

/// 豁免要起作用的那一头（文件头）。判别力（`crates/mutations.tsv`「实八 (c) 记录核对器：不按恢复落到那一版的实例表豁免被抛弃的发布」）：
/// 拿掉 `abandoned_by_the_landed_version` 那一判，被复用的那份单元按回收谓词解释不了，第二条判据报缺席，这条红。
#[test]
fn unit_of_the_abandoned_publish_reused_by_a_mount_blind_to_it_is_not_judged_missing_under_the_landed_instance_table(
) {
    let PoolBeforeTheAbandoningRecovery {
        mut pool,
        overwrite_to_abandon,
        ..
    } = pool_before_the_abandoning_recovery("record-checker-abandoned-publish-reused");
    let units_of_the_abandoned_publish =
        unit_writes_of_the_newest_publish(&pool, &overwrite_to_abandon);
    // 那次挂载关着影子账（连 C694 的 C 一起关）：产品路径上看得见被抛弃那次覆盖写的记录、隔离它写出的单元，造不出这一形。
    let (blind_mount, recording) = abandon_by_a_recovery_that_lands_on_the_root_before(
        &mut pool,
        &overwrite_to_abandon,
        ShadowLedger::Off,
    );
    // 循环：至多 OVERWRITES_IN_THE_BLIND_MOUNT_AT_MOST 轮，每轮在池上接着覆盖写一次；被抛弃那次覆盖写的单元写有一份不在盘上了就停。
    let mut reused_by_the_blind_mount = false;
    for round in 0..OVERWRITES_IN_THE_BLIND_MOUNT_AT_MOST {
        let round_index = usize::try_from(round).expect("轮次装得进 usize");
        let content: Vec<u8> = (0..(1800 + round_index * 53))
            .map(|index| u8::try_from((index * 13 + round_index) % 251).expect("小于 256"))
            .collect();
        let previous = pool.output.clone();
        pool.output = publish_overwrite_in_process(
            &mut pool,
            &previous,
            &content,
            FIXED_WRITE_TIME_SECONDS + 300 + round,
            blind_mount.output.instance,
        )
        .expect("看不见被抛弃那一版的那次挂载里接着覆盖写");
        let image = pool.memory_pool();
        if units_of_the_abandoned_publish
            .iter()
            .any(|unit| !write_is_still_on(&image, unit))
        {
            reused_by_the_blind_mount = true;
            break;
        }
    }
    assert!(
        reused_by_the_blind_mount,
        "看不见被抛弃那一版的那次挂载写了 {OVERWRITES_IN_THE_BLIND_MOUNT_AT_MOST} 版，一个都没落在它独占的槽上"
    );

    let (writes, first_write_after_the_crash) =
        writes_of_the_implementation_around_the_abandoning_recovery(&pool, &recording);
    let newest_root = newest_root_write(&writes);
    let effective_root = root_identity_of_write(newest_root);
    assert_eq!(
        effective_root,
        (pool.output.root.instance, pool.output.root.checkpoint_txg),
        "写表里最新那条根是那次挂载最后一次覆盖写的"
    );
    let image = pool.memory_pool();
    assert_eq!(
        recover(&image, JournalPolicy::Consult).effective_root,
        Some(effective_root),
        "恢复落到那次挂载最后一次覆盖写"
    );
    let abandoned_root = (
        overwrite_to_abandon.root.instance,
        overwrite_to_abandon.root.checkpoint_txg,
    );
    let abandoned_root_write = writes[..first_write_after_the_crash]
        .iter()
        .rev()
        .find(|write| {
            write.kind == StepKind::RootRecordFua && root_identity_of_write(write) == abandoned_root
        })
        .expect("崩溃之前的历史里有被抛弃那一版的根槽写");
    assert!(
        write_is_still_on(&image, abandoned_root_write),
        "被抛弃那一版的根还在根环里：那次复用时环里最旧的根不新于它，回收谓词证得出那次复用过不了，不豁免就判缺席"
    );
    let landed_instance_table = InstanceTableOfRootRecord::read(
        &image,
        newest_root.bytes().expect("根槽 FUA 写是普通写，带着字节"),
    )
    .expect("恢复落到的那一版的实例表读得出");
    assert!(
        landed_instance_table.abandons(abandoned_root.0 .0, abandoned_root.1 .0),
        "恢复落到的那一版的实例表判被抛弃那一版出局：{landed_instance_table:?}"
    );

    let check = record_check_across_the_abandoning_recovery(
        &pool.memory_pool_after_mkfs(),
        &writes,
        &vec![true; writes.len()],
        first_write_after_the_crash,
        (
            blind_mount.output.effective_root.instance,
            blind_mount.output.effective_root.checkpoint_txg,
        ),
        effective_root,
    );
    assert!(
        !check.claimed_state_missing_unit,
        "被抛弃那一版的单元不算「该在」：被那次挂载复用之后记录核对器不报缺席"
    );
}

/// 豁免不许多罩的那一头（文件头）。影子账开着（产品路径）：那次挂载只写行与暖机，之后不再写。
/// 判别力（`crates/mutations.tsv`「实八 (c) checker：按实例表判抛弃时 T = Ti 也算被抛弃」）：`abandons` 的 `>` 改成 `>=`，
/// 实例 1 那次覆盖写（T = Ti）也算被抛弃，扣下它的数据单元不再判红，这条红。
#[test]
fn withheld_unit_of_the_publish_at_the_txg_of_its_instance_row_is_still_judged_missing_under_the_landed_instance_table(
) {
    let PoolBeforeTheAbandoningRecovery {
        mut pool,
        overwrite_at_the_instance_row,
        overwrite_to_abandon,
    } = pool_before_the_abandoning_recovery("record-checker-publish-at-the-instance-row");
    let (mount, recording) = abandon_by_a_recovery_that_lands_on_the_root_before(
        &mut pool,
        &overwrite_to_abandon,
        ShadowLedger::On,
    );
    let (writes, first_write_after_the_crash) =
        writes_of_the_implementation_around_the_abandoning_recovery(&pool, &recording);
    let newest_root = newest_root_write(&writes);
    let effective_root = root_identity_of_write(newest_root);
    assert_eq!(
        effective_root.0, mount.output.instance,
        "写表里最新那条根是抛弃那次挂载的新实例写的"
    );
    let image = pool.memory_pool();
    assert_eq!(
        recover(&image, JournalPolicy::Consult).effective_root,
        Some(effective_root),
        "恢复落到那次挂载最新那条根"
    );
    let row_instance = overwrite_at_the_instance_row.root.instance.0;
    let row_checkpoint_txg = overwrite_at_the_instance_row.root.checkpoint_txg.0;
    let landed_instance_table = InstanceTableOfRootRecord::read(
        &image,
        newest_root.bytes().expect("根槽 FUA 写是普通写，带着字节"),
    )
    .expect("恢复落到的那一版的实例表读得出");
    // 两个探点各在 Ti 的一侧、都不等于 Ti：`>` 与 `>=` 在这两点上同值，这一句只钉「实例 1 那一行的 Ti 就是那次覆盖写的 txg」，
    // 不替记录核对器那一判下结论。
    assert!(
        landed_instance_table.abandons(row_instance, row_checkpoint_txg + 1)
            && !landed_instance_table.abandons(
                row_instance,
                row_checkpoint_txg
                    .checked_sub(1)
                    .expect("覆盖写的 txg 在暖机与第一个文件之后，不为 0")
            ),
        "恢复落到的那一版的实例表里实例 {row_instance} 那一行的 Ti 是 {row_checkpoint_txg}：{landed_instance_table:?}"
    );
    let data_unit_places: Vec<(DeviceIdentity, DeviceOffsetInBytes)> =
        overwrite_at_the_instance_row
            .data_pointers
            .iter()
            .flat_map(|pointer| pointer.locations)
            .map(|location| (location.device, location.slot.to_device_offset()))
            .collect();
    assert!(
        !data_unit_places.is_empty(),
        "实例 1 那次覆盖写写了数据单元"
    );
    let withheld_writes: Vec<usize> = writes
        .iter()
        .enumerate()
        .filter(|(_, write)| {
            write.kind == StepKind::UnitWrite
                && data_unit_places.contains(&(write.device, write.offset))
        })
        .map(|(index, _)| index)
        .collect();
    assert_eq!(
        withheld_writes.len(),
        data_unit_places.len(),
        "实例 1 那次覆盖写的数据单元每份一次写，之后没有别的单元写落在那几处"
    );

    let base = pool.memory_pool_after_mkfs();
    let version_landed_on_after_the_crash = (
        mount.output.effective_root.instance,
        mount.output.effective_root.checkpoint_txg,
    );
    let every_write_landed = vec![true; writes.len()];
    assert!(
        !record_check_across_the_abandoning_recovery(
            &base,
            &writes,
            &every_write_landed,
            first_write_after_the_crash,
            version_landed_on_after_the_crash,
            effective_root,
        )
        .claimed_state_missing_unit,
        "整张写表都落了盘：一份单元都不缺席"
    );
    let mut persisted_without_the_data_unit = every_write_landed;
    for index in &withheld_writes {
        persisted_without_the_data_unit[*index] = false;
    }
    assert!(
        record_check_across_the_abandoning_recovery(
            &base,
            &writes,
            &persisted_without_the_data_unit,
            first_write_after_the_crash,
            version_landed_on_after_the_crash,
            effective_root,
        )
        .claimed_state_missing_unit,
        "实例 1 那次覆盖写的 txg 等于它那一行的 Ti，不算被抛弃：它的数据单元两份都没落，照报缺席"
    );
}
