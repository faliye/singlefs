//! 里程碑「第二个事务」步 0 在发布 B 上的那一半：把「取号 → 暖机 → A → B」整条录制流按 D13（验证路线） 已定项 4 枚举全部崩溃状态，
//! 与第一个事务同一种切法（上一次发布的超级块槽写与下一次发布的单元写落在同一段，登记表八末尾那条 ⚠️），
//! 每个状态跑恢复 + 多版本 oracle（实际走的根是哪一代就得读出那一代的内容）、池级 checker、记录核对器。
//! 平时 `cargo test` 跳过两个 18 写的段；全量那条标 ignored，54 号门禁在 release 下跑它。再加四组靶向的阳性对照。
//! 另有两条只展开小段的流：基镜像里预置一条残留记录（步 0 预想的细节第三条的正例），与到 E 之后再复用一次、改坏 tail（步 6 必红「陈旧 tail + 已复用的块」）。

mod common;

use common::{build_pool, file_content, geometry, parameters, BuiltPool, FIXED_WRITE_TIME_SECONDS};
use singlefs_core::address::{CheckpointTxg, InstanceGeneration, SlotNumber};
use singlefs_core::block_device::{BlockDevice, WriteDurability};
use singlefs_core::checksum::crc32_castagnoli;
use singlefs_core::mount::{
    mount_rollback, mount_writable, raise_rollback_floor, InstanceRow, RollbackTarget, ShadowLedger,
};
use singlefs_core::recovery::{
    choose_superblock, recover, scan_journal, JournalPolicy, PoolReader, RecoveryOutcome,
};
use singlefs_core::superblock::Superblock;
use singlefs_core::transaction::{publish_overwrite, FirstFile, PoolWriter, TransactionOutput};
use singlefs_core::unit::{UNIT_CLASS_DATA, UNIT_CLASS_INDEX_NODE, UNIT_CLASS_PACKED};
use singlefs_format::{DATA_UNIT_BYTES, NODE_BYTES};
use singlefs_harness::crash::{
    closed_form_state_count, enumerate_layer0_selecting_versions,
    enumerate_layer0_selecting_versions_observing_each_state, enumerate_layer0_versions,
    evaluate_state_for_versions, writes_and_segments, CrashImage, Layer0Tally, MemoryPool,
    PublishedVersion, RetainedWrite,
};
use singlefs_harness::segments::StepKind;
use singlefs_harness::RetainedOperation;

const SECOND_FILE_BYTES: usize = 4100;

fn second_content() -> Vec<u8> {
    (0..SECOND_FILE_BYTES)
        .map(|index| u8::try_from((index * 7 + 3) % 253).expect("小于 256"))
        .collect()
}

struct Prepared {
    base: MemoryPool,
    writes: Vec<RetainedWrite>,
    segments: Vec<Vec<usize>>,
    /// 被判的那次根槽 FUA 写在写表里的下标：脚本最后一次发布的根（到 B 为止就是 B 的，到 C 为止是 C 的）。
    judged_root_index: usize,
    /// A 的根槽 FUA 写在写表里的下标：它之后的写都是 B 的。
    first_root_index: usize,
    versions: Vec<PublishedVersion>,
}

/// 固定脚本跑到哪一步（里程碑「第二个事务」步 0）：到 B 为止是靶向对照用的两次发布流；到 C 为止多了进程重开、可写挂载
/// （取号、写行、暖机两次）与发布 C，是层 0 全量跑的那条流。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Script {
    SecondVersionOnly,
    ThirdVersion,
    /// 到 D 为止：C 之后进程再退出、重开走管理员回退到 A 的根 (1, 3)——取号 3、写回退行与中间实例行的发布 D（txg 9）、暖机一次（txg 10）。
    RollbackToFirstVersion,
    /// 到 E 为止：回退之后再覆盖写四次（txg 11–14，第一次释放 A 的数据单元、释放代 11），抬 F 到 11（两次空发布 txg 15、16），
    /// 再发布 E（txg 17）——数据单元落回 50178。
    ReuseAfterRaisingFloor,
    /// 到 E 之后再覆盖写一次（txg 18）：数据单元落回 A 的数据单元那一对槽 50180，A 那条记录（jsn 3）点名的单元从此校验和对不上——
    /// 环里第一条点名块被合法复用的记录（到 E 为止环里 17 条记录点名的单元都还对得上）。
    ReuseOfTheFirstDataUnitSlotAfterFloorRaisingPublish,
}

const THIRD_FILE_BYTES: usize = 2500;

fn later_content(seed: usize) -> Vec<u8> {
    (0..3000 + seed)
        .map(|index| u8::try_from((index * 5 + seed) % 251).expect("小于 256"))
        .collect()
}

fn third_content() -> Vec<u8> {
    (0..THIRD_FILE_BYTES)
        .map(|index| u8::try_from((index * 7 + 11) % 253).expect("小于 256"))
        .collect()
}

fn overwrite(
    pool: &mut BuiltPool,
    content: &[u8],
    instance: InstanceGeneration,
) -> TransactionOutput {
    let parameters = parameters();
    let devices = pool.devices.as_mut().expect("镜像还开着");
    let mut writer = PoolWriter::new(&parameters, devices.as_mut_slice());
    publish_overwrite(
        &mut writer,
        &mut pool.allocator,
        &pool.output,
        FirstFile {
            content,
            write_time_seconds: FIXED_WRITE_TIME_SECONDS + 60,
        },
        instance,
    )
    .expect("覆盖写")
}

fn prepare(tag: &str, script: Script) -> Prepared {
    let mut pool = build_pool(tag);
    let second = overwrite(&mut pool, &second_content(), InstanceGeneration(1));
    let mut versions = vec![
        PublishedVersion {
            instance: InstanceGeneration(1),
            checkpoint_txg: CheckpointTxg(3),
            content: file_content(),
        },
        PublishedVersion {
            instance: InstanceGeneration(1),
            checkpoint_txg: CheckpointTxg(4),
            content: second_content(),
        },
    ];
    if script != Script::SecondVersionOnly {
        pool.output = second;
        // 进程重开：取号 2、写行、暖机两次（txg 5 / 6 落盘 0、txg 7 落盘 1），文件还是第二次的内容。
        let mut devices = pool.reopen_recorded();
        let mounted = mount_writable(&parameters(), &mut devices).expect("可写挂载");
        pool.devices = Some(devices);
        pool.allocator = mounted.allocator;
        pool.output = mounted
            .current
            .into_file_version()
            .expect("B 之后重开，现行那一版带文件");
        for txg in 5..=7 {
            versions.push(PublishedVersion {
                instance: InstanceGeneration(2),
                checkpoint_txg: CheckpointTxg(txg),
                content: second_content(),
            });
        }
        let third = overwrite(&mut pool, &third_content(), InstanceGeneration(2));
        versions.push(PublishedVersion {
            instance: InstanceGeneration(2),
            checkpoint_txg: CheckpointTxg(8),
            content: third_content(),
        });
        if script != Script::ThirdVersion {
            pool.output = third;
            // 进程再退出、重开走回退到 A 的根：取号 3、发布 D（txg 9 落盘 0）写回退行 (1, 3, 0) 与中间实例行 (2, 0, 0)、
            // 暖机一次（txg 10 落盘 1）；文件回到第一次的内容。
            let mut reopened_for_rollback = pool.reopen_recorded();
            let rolled_back = mount_rollback(
                &parameters(),
                &mut reopened_for_rollback,
                RollbackTarget {
                    instance: InstanceGeneration(1),
                    checkpoint_txg: CheckpointTxg(3),
                },
                ShadowLedger::On,
            )
            .expect("回退");
            pool.devices = Some(reopened_for_rollback);
            pool.allocator = rolled_back.allocator;
            pool.output = rolled_back
                .current
                .into_file_version()
                .expect("回退到 A，现行那一版带文件");
            for txg in 9..=10 {
                versions.push(PublishedVersion {
                    instance: InstanceGeneration(3),
                    checkpoint_txg: CheckpointTxg(txg),
                    content: file_content(),
                });
            }
            if matches!(
                script,
                Script::ReuseAfterRaisingFloor
                    | Script::ReuseOfTheFirstDataUnitSlotAfterFloorRaisingPublish
            ) {
                let mut latest = Vec::new();
                for (txg, seed) in [(11u64, 17usize), (12, 19), (13, 23), (14, 29)] {
                    latest = later_content(seed);
                    pool.output = overwrite(&mut pool, &latest, InstanceGeneration(3));
                    versions.push(PublishedVersion {
                        instance: InstanceGeneration(3),
                        checkpoint_txg: CheckpointTxg(txg),
                        content: latest.clone(),
                    });
                }
                let mut current = pool.output.clone();
                let raise_devices = pool.devices.as_mut().expect("镜像还开着");
                let raised = raise_rollback_floor(
                    &parameters(),
                    raise_devices,
                    &mut pool.allocator,
                    &mut current,
                    CheckpointTxg(11),
                    ShadowLedger::On,
                )
                .expect("抬 F");
                assert_eq!(raised.publishes.len(), 2, "txg 15 落盘 0、txg 16 落盘 1");
                pool.output = current;
                for txg in 15..=16 {
                    versions.push(PublishedVersion {
                        instance: InstanceGeneration(3),
                        checkpoint_txg: CheckpointTxg(txg),
                        content: latest.clone(),
                    });
                }
                let reuse = overwrite(&mut pool, &later_content(31), InstanceGeneration(3));
                assert_eq!(
                    reuse.data_pointer.locations[0].slot.0,
                    50178,
                    "E 的数据单元落回最低的可再分配偶数槽对（mkfs 树表那 1 槽回收了、50179 从没分配过）"
                );
                versions.push(PublishedVersion {
                    instance: InstanceGeneration(3),
                    checkpoint_txg: CheckpointTxg(17),
                    content: later_content(31),
                });
                if script == Script::ReuseOfTheFirstDataUnitSlotAfterFloorRaisingPublish {
                    pool.output = reuse;
                    let first_data_unit_slot_reused =
                        overwrite(&mut pool, &later_content(37), InstanceGeneration(3));
                    assert_eq!(
                        first_data_unit_slot_reused.data_pointer.locations[0].slot.0,
                        50180,
                        "txg 18 的数据单元落回 A 的数据单元那一对槽（抬 F 回收了、E 用掉的是更低的 50178）"
                    );
                    versions.push(PublishedVersion {
                        instance: InstanceGeneration(3),
                        checkpoint_txg: CheckpointTxg(18),
                        content: later_content(37),
                    });
                }
            }
        }
    }
    let base = pool.memory_pool_after_mkfs();
    let operations = pool.retained_operations();
    let (writes, segments) =
        writes_and_segments(&operations[pool.mkfs_operation_count..], &geometry());
    let sizes: Vec<usize> = segments.iter().map(Vec::len).collect();
    match script {
        Script::SecondVersionOnly => {
            assert_eq!(
                sizes,
                vec![2, 2, 1, 2, 2, 1, 18, 2, 1, 18, 2, 1, 2],
                "A 的两个超级块槽写与 B 的 16 个单元写合成一段：整条流按屏障切，不按发布切"
            );
            assert_eq!(writes.len(), 54, "取号 2 + 暖机 10 + A 21 + B 21 次写");
        }
        Script::ThirdVersion => {
            // B 的两个超级块槽写与取号的两个合成一段（4）；写行发布 10 个单元写（实例表 + 四个固定点单元，各两盘）；
            // 每次暖机 8 个单元写与上一次发布的两个超级块槽写合成一段（10）；C 的 16 个单元写与 txg 7 的超级块槽写合成 18。
            assert_eq!(
                sizes,
                vec![
                    2, 2, 1, 2, 2, 1, 18, 2, 1, 18, 2, 1, 4, 10, 2, 1, 10, 2, 1, 10, 2, 1, 18, 2,
                    1, 2
                ],
                "固定脚本到 C 为止的段序列"
            );
            assert_eq!(
                writes.len(),
                54 + 2 + 15 + 13 + 13 + 21,
                "取号 2 + 写行 15 + 暖机 13 × 2 + C 21"
            );
        }
        Script::RollbackToFirstVersion => {
            // C 的两个超级块槽写与回退取号的两个合成一段（4）；D 是写回退行的发布：10 个单元写（实例表 + 四个固定点单元，各两盘）；
            // 暖机一次 8 个单元写与 D 的两个超级块槽写合成一段（10）。
            assert_eq!(
                sizes,
                vec![
                    2, 2, 1, 2, 2, 1, 18, 2, 1, 18, 2, 1, 4, 10, 2, 1, 10, 2, 1, 10, 2, 1, 18, 2,
                    1, 4, 10, 2, 1, 10, 2, 1, 2
                ],
                "固定脚本到 D 为止的段序列"
            );
            assert_eq!(
                writes.len(),
                118 + 2 + 15 + 13,
                "到 C 118 + 取号 2 + D 15 + 暖机 13"
            );
        }
        Script::ReuseAfterRaisingFloor => {
            // 四次覆盖写各 16 个单元写并上一次的两个超级块槽写（18）；抬 F 的两次空发布各 8 个单元写并上两个超级块槽写（10）；E 同覆盖写。
            assert_eq!(
                sizes,
                vec![
                    2, 2, 1, 2, 2, 1, 18, 2, 1, 18, 2, 1, 4, 10, 2, 1, 10, 2, 1, 10, 2, 1, 18, 2,
                    1, 4, 10, 2, 1, 10, 2, 1, 18, 2, 1, 18, 2, 1, 18, 2, 1, 18, 2, 1, 10, 2, 1, 10,
                    2, 1, 18, 2, 1, 2
                ],
                "固定脚本到 E 为止的段序列"
            );
            assert_eq!(
                writes.len(),
                148 + 4 * 21 + 2 * 13 + 21,
                "到 D 148 + 四次覆盖写 84 + 抬 F 两次 26 + E 21"
            );
        }
        Script::ReuseOfTheFirstDataUnitSlotAfterFloorRaisingPublish => {
            // 到 E 的段序列末尾那段（E 的两个超级块槽写）并进 txg 18 的 16 个单元写（18），再接记录、根槽、超级块槽。
            assert_eq!(
                sizes,
                vec![
                    2, 2, 1, 2, 2, 1, 18, 2, 1, 18, 2, 1, 4, 10, 2, 1, 10, 2, 1, 10, 2, 1, 18, 2,
                    1, 4, 10, 2, 1, 10, 2, 1, 18, 2, 1, 18, 2, 1, 18, 2, 1, 18, 2, 1, 10, 2, 1, 10,
                    2, 1, 18, 2, 1, 18, 2, 1, 2
                ],
                "固定脚本到 E 之后再覆盖写一次的段序列"
            );
            assert_eq!(writes.len(), 279 + 21, "到 E 279 + txg 18 的覆盖写 21");
        }
    }
    let root_indexes: Vec<usize> = writes
        .iter()
        .enumerate()
        .filter(|(_, write)| write.kind == StepKind::RootRecordFua)
        .map(|(index, _)| index)
        .collect();
    let expected_roots = match script {
        Script::SecondVersionOnly => 4,
        Script::ThirdVersion => 8,
        Script::RollbackToFirstVersion => 10,
        Script::ReuseAfterRaisingFloor => 17,
        Script::ReuseOfTheFirstDataUnitSlotAfterFloorRaisingPublish => 18,
    };
    assert_eq!(root_indexes.len(), expected_roots, "每次发布一条根槽写");
    Prepared {
        base,
        writes,
        segments,
        judged_root_index: *root_indexes.last().expect("至少一条根槽写"),
        first_root_index: root_indexes[2],
        versions,
    }
}

fn assert_checker_and_record_checker_clean(tally: &Layer0Tally) {
    assert_checker_and_record_checker_counts(tally, &[], 0);
}

/// `known_checker_violations` 与 `known_record_claimed_state_missing_unit` 只给两条只展开小段的流用：它们钉的是两处口径未定的分歧的现状
/// （见各自用例的注释），不是认下来的行为；其余不变量判违例的状态数必须是 0。
fn assert_checker_and_record_checker_counts(
    tally: &Layer0Tally,
    known_checker_violations: &[(&str, u64)],
    known_record_claimed_state_missing_unit: u64,
) {
    assert_eq!(
        (
            tally.record_root_without_record,
            tally.record_claimed_state_missing_unit
        ),
        (0, known_record_claimed_state_missing_unit),
        "记录核对器两条判据在已定的持久顺序下恒 0（第二条在合法复用上的已知分歧除外）"
    );
    for invariant in singlefs_checker::image::IMPLEMENTED_INVARIANTS {
        let expected_violated_states = known_checker_violations
            .iter()
            .find(|(known_invariant, _)| *known_invariant == invariant)
            .map_or(0, |(_, violated_states)| *violated_states);
        assert_eq!(
            tally
                .checker_violated_states
                .get(invariant)
                .copied()
                .unwrap_or(0),
            expected_violated_states,
            "{invariant} 判违例的状态数：{:?}",
            tally.checker_first_violation.get(invariant)
        );
    }
    // 里程碑「第二个事务」步 6 新接的三条（I-3.8、I-7.4、I-4.8）、改按回退候选集判的两条（I-3.1、I-2.1），
    // 与 C374（释放代与树表诞生 txg 只有验收断言盯着） 定案接的两条（I-3.9、I-9.14——这条流上发布 B 起每次发布都重写树表，
    // 跨根比得出来）、代码三方第二轮 Z1-d 之后接的 I-5.4（分配记录罩住的槽互不相交）：阴性结果要与「代码没跑到」分开，
    // 每条至少在一个状态上真被评估过；评估过的与报「不适用」的加起来恰是状态数，一个状态都不漏记。
    for must_evaluate in [
        "I-3.1", "I-5.2", "I-5.1", "I-7.2", "I-2.1", "I-3.8", "I-7.4", "I-4.8", "I-3.9", "I-9.14",
        "I-5.4",
    ] {
        assert!(
            tally
                .checker_evaluated_states
                .get(must_evaluate)
                .copied()
                .unwrap_or(0)
                > 0,
            "{must_evaluate} 至少在一个状态上真被评估过：{}",
            tally.checker_counts_by_invariant()
        );
    }
    for invariant in singlefs_checker::image::IMPLEMENTED_INVARIANTS {
        assert_eq!(
            tally
                .checker_evaluated_states
                .get(invariant)
                .copied()
                .unwrap_or(0)
                + tally
                    .checker_not_applicable_states
                    .get(invariant)
                    .copied()
                    .unwrap_or(0),
            tally.states,
            "{invariant} 评估过的状态数 + 不适用的状态数 = 状态数"
        );
    }
}

/// 到 C 为止的固定脚本：段序列登记表（layout/01-first-txn.md 八）「装置钉住」的那条数组由 `prepare` 里的断言钉住；层 0 全量与快的那条都在到 D 的脚本上跑。
#[test]
fn the_fixed_script_through_the_third_publish_keeps_its_registered_segment_sequence() {
    let prepared = prepare("layer0-c-registered", Script::ThirdVersion);
    assert_eq!(prepared.segments.len(), 26);
    assert_eq!(closed_form_state_count(&prepared.segments), 789_555);
}

/// 到 D 为止的固定脚本：同上，33 段、闭式 791624。
#[test]
fn the_fixed_script_through_the_rollback_publish_keeps_its_registered_segment_sequence() {
    let prepared = prepare("layer0-d-registered", Script::RollbackToFirstVersion);
    assert_eq!(prepared.segments.len(), 33);
    assert_eq!(closed_form_state_count(&prepared.segments), 791_624);
}

/// 平时跑的那一份：三个 18 写的段与三个 10 写的段不展开（只以整段持久进入后面的状态），其余每段任意子集。
#[test]
fn every_crash_state_outside_the_two_unit_segments_recovers_to_the_version_its_root_claims() {
    let prepared = prepare("layer0-e-fast", Script::ReuseAfterRaisingFloor);
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
    assert_eq!(
        tally.states, 108,
        "1 + 二十个 2 写段各 3 + 两个 4 写段各 15 + 十七个 1 写段各 1"
    );
    println!(
        "LAYER0B_FAST states={} checker_by_invariant(evaluated/violated/not_applicable) {}",
        tally.states,
        tally.checker_counts_by_invariant()
    );
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
    assert!(tally.file_read_states > 0 && tally.no_file_states > 0);
    assert_checker_and_record_checker_clean(&tally);
}

/// 全量：固定脚本到 E 为止 54 段、闭式 2 104 413 个状态（八个 18 写段各 262143、七个 10 写段各 1023、两个 4 写段各 15，其余 2 写段各 3、1 写段各 1）。
/// 54 号门禁在 release 下跑它，认下面打印的 `LAYER0B` 行里 `exhaustive=true`。
#[test]
#[ignore = "全量 2104413 个状态、每个两遍恢复 + checker，debug 下半小时以上；门禁 54 号在 release 下跑"]
fn full_enumeration_of_the_fixed_script_stream_is_exhaustive_and_clean() {
    let prepared = prepare("layer0-e-full", Script::ReuseAfterRaisingFloor);
    let closed_form = closed_form_state_count(&prepared.segments);
    assert_eq!(closed_form, 2_104_413, "闭式：1 + Σ(2^|段| − 1)，五十四段");
    let tally = enumerate_layer0_versions(
        &prepared.base,
        &prepared.writes,
        &prepared.segments,
        prepared.judged_root_index,
        &prepared.versions,
    );
    let checker_violations: u64 = tally.checker_violated_states.values().sum();
    // 每条不变量报成 `I-x.y=评估过/判违例/不适用` 夹在 checker_violations 与 first_violation 之间（步 6 验收第 3 条：阴性结果与「代码没跑到」分开）；
    // 54 号门禁只认行首 `LAYER0B ` 与 `exhaustive=true`，整行原样报出来。
    println!(
        "LAYER0B states={} closed_form={closed_form} exhaustive={} violations={} root_persisted_states={} no_file={} file_read={} failed={} journal_differing={} verification_ran={} verification_failed={} record_root_without_record={} record_claimed_state_missing_unit={} checker_violations={checker_violations} {} first_violation={}",
        tally.states,
        tally.states == closed_form,
        tally.violations,
        tally.root_persisted_states,
        tally.no_file_states,
        tally.file_read_states,
        tally.failed_states,
        tally.journal_differing_states,
        tally.verification_ran_states,
        tally.verification_failed_states,
        tally.record_root_without_record,
        tally.record_claimed_state_missing_unit,
        tally.checker_counts_by_invariant(),
        tally.first_violation.as_deref().unwrap_or("none")
    );
    assert_eq!(tally.states, closed_form, "枚举到的状态数要等于闭式");
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
    assert_checker_and_record_checker_clean(&tally);
}

/// 靶向的阳性对照：把持久集合手工摆成四个形状，oracle、journal 承重、记录核对器各要在它该红的那一格红。
#[test]
fn targeted_controls_on_the_second_publish_go_red_where_they_should() {
    let prepared = prepare("layer0-b-controls", Script::SecondVersionOnly);
    let all = vec![true; prepared.writes.len()];
    let is_second_publish = |index: usize| index > prepared.first_root_index;
    let kinds_of_second_publish = |kind: StepKind| -> Vec<usize> {
        prepared
            .writes
            .iter()
            .enumerate()
            .filter(|(index, write)| is_second_publish(*index) && write.kind == kind)
            .map(|(index, _)| index)
            .collect()
    };
    let second_publish_units = kinds_of_second_publish(StepKind::UnitWrite);
    let second_publish_records = kinds_of_second_publish(StepKind::JournalRecord);
    assert_eq!(
        (second_publish_units.len(), second_publish_records.len()),
        (16, 2)
    );
    let evaluate = |persisted: Vec<bool>| {
        let mut tally = Layer0Tally::default();
        let report = evaluate_state_for_versions(
            &prepared.base,
            &prepared.writes,
            persisted,
            prepared.judged_root_index,
            &prepared.versions,
            &mut tally,
        );
        (report, tally)
    };

    // ① 全部持久：走 B 的根、读回第二次的内容，零违例。
    let (all_persisted_report, all_persisted_tally) = evaluate(all.clone());
    assert_eq!(
        all_persisted_report.effective_root,
        Some((InstanceGeneration(1), CheckpointTxg(4)))
    );
    assert!(matches!(
        all_persisted_report.outcome,
        RecoveryOutcome::FileRead { .. }
    ));
    assert_eq!(all_persisted_tally.violations, 0);

    // ② B 的根槽已持久、B 的十六个单元写一份都没持久：oracle 必须红（走读失败），记录核对器判「自称新态而单元缺席」。
    let mut root_without_units = all.clone();
    for index in &second_publish_units {
        root_without_units[*index] = false;
    }
    let (_, root_without_units_tally) = evaluate(root_without_units);
    assert_eq!(
        root_without_units_tally.violations, 1,
        "根槽已持久而单元不在：{:?}",
        root_without_units_tally.first_violation
    );
    assert_eq!(
        root_without_units_tally.record_claimed_state_missing_unit,
        1
    );
    assert_eq!(
        root_without_units_tally.ignored_violations, 1,
        "不看 journal 那一遍同样走读失败"
    );

    // ③ B 的根槽没持久、记录与单元都持久：看 journal 由记录重建第 4 代根读出第二次的内容，不看就退回 A——journal 在这一格承重。
    let mut root_missing = all.clone();
    root_missing[prepared.judged_root_index] = false;
    let (root_missing_report, root_missing_tally) = evaluate(root_missing);
    assert_eq!(
        root_missing_report.effective_root,
        Some((InstanceGeneration(1), CheckpointTxg(4)))
    );
    assert_eq!(root_missing_report.journal.prefix_applied, 1);
    assert_eq!(root_missing_tally.journal_differing_states, 1);
    assert_eq!(root_missing_tally.violations, 0);

    // ④ B 的根槽已持久、两份记录都没持久：走读没问题（根记录自带全部字段）、oracle 不红，但记录核对器判「根在案而记录缺席」。
    let mut root_without_records = all.clone();
    for index in &second_publish_records {
        root_without_records[*index] = false;
    }
    let (_, root_without_records_tally) = evaluate(root_without_records);
    assert_eq!(root_without_records_tally.violations, 0);
    assert_eq!(root_without_records_tally.record_root_without_record, 1);

    // ⑤ B 一个字节都没持久：走 A 的根、读回第一次的内容，零违例——旧态在它自己的根下面是合法的。
    let mut nothing_of_second_publish = all;
    for (write_index, persisted) in nothing_of_second_publish.iter_mut().enumerate() {
        if is_second_publish(write_index) {
            *persisted = false;
        }
    }
    let (nothing_of_second_publish_report, nothing_of_second_publish_tally) =
        evaluate(nothing_of_second_publish);
    assert_eq!(
        nothing_of_second_publish_report.effective_root,
        Some((InstanceGeneration(1), CheckpointTxg(3)))
    );
    assert_eq!(
        nothing_of_second_publish_report.outcome,
        RecoveryOutcome::FileRead {
            root: (InstanceGeneration(1), CheckpointTxg(3)),
            content: file_content()
        }
    );
    assert_eq!(nothing_of_second_publish_tally.violations, 0);
}

fn residual_content() -> Vec<u8> {
    later_content(5)
}

/// 预置的残留记录在 (实例, txg) 上的身份：B 之后同一个实例里又发布了一次、根槽没落盘，留在环里的是它的记录（jsn 5、txg 5、事务号 3）。
const RESIDUAL_RECORD_ROOT: (InstanceGeneration, CheckpointTxg) =
    (InstanceGeneration(1), CheckpointTxg(5));

/// 造那条残留记录与它点名的单元：另开一个池走同一段历史（mkfs → 取号 → 暖机 → A → B，字节确定），在实例 1 里再覆盖写一次，
/// 只留下这次发布的单元写与 journal 记录写（根槽与超级块槽的写丢掉 = 根槽没落盘）。返回种子与种子之前那段历史的录制流。
fn residual_record_and_its_named_units(
    tag: &str,
) -> (Vec<RetainedOperation>, Vec<RetainedOperation>) {
    let mut scratch = build_pool(tag);
    scratch.output = overwrite(&mut scratch, &second_content(), InstanceGeneration(1));
    let history = scratch.retained_operations();
    let residual = overwrite(&mut scratch, &residual_content(), InstanceGeneration(1));
    assert_eq!(
        (
            residual.record.instance,
            residual.record.checkpoint_txg,
            residual.record.counter,
            residual.record.transaction,
            residual.record.is_commit,
            residual.record.named.len()
        ),
        (RESIDUAL_RECORD_ROOT.0, RESIDUAL_RECORD_ROOT.1, 5, 3, true, 8),
        "残留记录：实例 1、txg 5、jsn 5（接在 B 的 jsn 4 之后）、事务号 3、带提交标记、点名八个单元"
    );
    let seed: Vec<RetainedOperation> = scratch.retained_operations()[history.len()..]
        .iter()
        .filter(|retained| match geometry().classify(&retained.operation) {
            StepKind::UnitWrite | StepKind::JournalRecord => true,
            StepKind::RootRecordFua | StepKind::SuperblockSlot | StepKind::Barrier => false,
        })
        .cloned()
        .collect();
    assert_eq!(seed.len(), 16 + 2, "八个单元各两盘、记录两份");
    (seed, history)
}

/// 预置基镜像的流：基镜像 = mkfs 之后 + 残留记录与它点名的单元；录制流 = 取号 → 暖机 → A → B → 进程退出、重开可写挂载
/// （恢复施加残留记录、给实例 1 写行 (1, 5, 3)、写行发布 txg 6 = max(根环 4, 记录 5) + 1、暖机一次 txg 7 落盘 1）→ C（txg 8）。
/// 文件镜像上种子在 B 之后才写进去（不经录制器），崩溃镜像里种子在基镜像里：两者落点不相交（种子的槽是 B 之后才分出去的、记录槽 jsn 5 流里没人写），
/// 全部持久那个状态的恢复与冷启动读文件镜像的恢复逐项相等由下面的断言钉住。
fn prepare_with_residual_record_seeded(tag: &str) -> Prepared {
    let (seed, seed_history) = residual_record_and_its_named_units(&format!("{tag}-seed"));
    let mut pool = build_pool(tag);
    pool.output = overwrite(&mut pool, &second_content(), InstanceGeneration(1));
    assert!(
        pool.retained_operations() == seed_history,
        "种子是在同一段历史之后造的：两个池到 B 为止的录制流逐项相同（带内容）"
    );
    let mut cold_devices = pool.reopen_cold();
    for seeded in &seed {
        let (_, device) = cold_devices
            .iter_mut()
            .find(|(identity, _)| *identity == seeded.operation.device)
            .expect("种子落在池里的盘上");
        device
            .write_at(
                seeded.operation.offset,
                seeded.contents.as_ref().expect("种子来自开了内容保留的流"),
                WriteDurability::Plain,
            )
            .expect("把种子写进文件镜像");
    }
    drop(cold_devices);
    let mut devices = pool.reopen_recorded();
    let mounted = mount_writable(&parameters(), &mut devices).expect("可写挂载");
    pool.devices = Some(devices);
    assert_eq!(
        (
            mounted.output.chosen_root.instance,
            mounted.output.chosen_root.checkpoint_txg,
            mounted.output.journal.prefix_applied
        ),
        (InstanceGeneration(1), CheckpointTxg(4), 1),
        "重开那一刻所选根是 B 的根，残留记录被施加"
    );
    assert_eq!(
        mounted.output.rows_written,
        vec![InstanceRow {
            instance: InstanceGeneration(1),
            selected_root_txg: CheckpointTxg(5),
            applied_transaction_high_water: 3,
            is_rollback: false,
        }]
    );
    assert_eq!(
        mounted.output.row_publish.root().checkpoint_txg,
        CheckpointTxg(6)
    );
    assert_eq!(
        mounted.output.warm_up_publishes.len(),
        1,
        "写行发布 txg 6 落盘 0，暖机 txg 7 落盘 1，一次就覆盖两块盘"
    );
    pool.allocator = mounted.allocator;
    pool.output = mounted
        .current
        .into_file_version()
        .expect("施加了残留记录，现行那一版带文件");
    let third = overwrite(&mut pool, &third_content(), InstanceGeneration(2));
    assert_eq!(third.root.checkpoint_txg, CheckpointTxg(8));
    let mut versions = vec![
        PublishedVersion {
            instance: InstanceGeneration(1),
            checkpoint_txg: CheckpointTxg(3),
            content: file_content(),
        },
        PublishedVersion {
            instance: InstanceGeneration(1),
            checkpoint_txg: CheckpointTxg(4),
            content: second_content(),
        },
        PublishedVersion {
            instance: RESIDUAL_RECORD_ROOT.0,
            checkpoint_txg: RESIDUAL_RECORD_ROOT.1,
            content: residual_content(),
        },
    ];
    for txg in 6..=7 {
        versions.push(PublishedVersion {
            instance: InstanceGeneration(2),
            checkpoint_txg: CheckpointTxg(txg),
            content: residual_content(),
        });
    }
    versions.push(PublishedVersion {
        instance: InstanceGeneration(2),
        checkpoint_txg: CheckpointTxg(8),
        content: third_content(),
    });
    let mut base = pool.memory_pool_after_mkfs();
    base.apply(&seed);
    let operations = pool.retained_operations();
    let (writes, segments) =
        writes_and_segments(&operations[pool.mkfs_operation_count..], &geometry());
    // 与到 C 的固定脚本同型，只少一次暖机（写行发布落在 txg 6 的盘 0 上，txg 7 就到盘 1）。
    assert_eq!(
        segments.iter().map(Vec::len).collect::<Vec<usize>>(),
        vec![2, 2, 1, 2, 2, 1, 18, 2, 1, 18, 2, 1, 4, 10, 2, 1, 10, 2, 1, 18, 2, 1, 2],
        "预置残留记录那条流的段序列"
    );
    assert_eq!(writes.len(), 54 + 2 + 15 + 13 + 21);
    let root_indexes: Vec<usize> = writes
        .iter()
        .enumerate()
        .filter(|(_, write)| write.kind == StepKind::RootRecordFua)
        .map(|(index, _)| index)
        .collect();
    assert_eq!(
        root_indexes.len(),
        7,
        "txg 1–4 与 6–8，txg 5 的根槽从没写过"
    );
    let all_persisted = CrashImage {
        base: &base,
        writes: &writes,
        persisted: vec![true; writes.len()],
    };
    let cold = pool.reopen_cold();
    assert_eq!(
        recover(&all_persisted, JournalPolicy::Consult),
        recover(&cold, JournalPolicy::Consult),
        "崩溃镜像全部持久那个状态与文件镜像冷启动恢复逐项相等"
    );
    Prepared {
        base,
        writes,
        segments,
        judged_root_index: *root_indexes.last().expect("至少一条根槽写"),
        first_root_index: root_indexes[2],
        versions,
    }
}

/// 步 0「预想的细节」第三条的正例（C42（残留记录冒充合法前缀）、E32（上一条时间线的残留）；验收第 2 条的正例那一半）：基镜像里预置一条
/// jsn 连续、校验和过、点名单元也在、属于所选根实例、所选根的实例表里没有回退行的记录。只展开 B 的根槽之后的小段，每个状态按独立的谓词判它该不该被施加：
/// 实例 2 的根一条都没持久（所选根还是 B 的 (1, 4)，链从 jsn 4 接得到 jsn 5）⇔ 恢复落在 (1, 5)、读出它那一版的内容。报出跑到的状态数。
#[test]
fn residual_record_seeded_into_the_base_image_is_applied_in_every_crash_state_whose_chain_reaches_it(
) {
    let prepared = prepare_with_residual_record_seeded("layer0-residual-record");
    let root_indexes: Vec<usize> = prepared
        .writes
        .iter()
        .enumerate()
        .filter(|(_, write)| write.kind == StepKind::RootRecordFua)
        .map(|(index, _)| index)
        .collect();
    let second_publish_root_index = root_indexes[3];
    let second_publish_record_indexes: Vec<usize> = prepared
        .writes
        .iter()
        .enumerate()
        .filter(|(index, write)| {
            write.kind == StepKind::JournalRecord
                && *index > root_indexes[2]
                && *index < second_publish_root_index
        })
        .map(|(index, _)| index)
        .collect();
    assert_eq!(second_publish_record_indexes.len(), 2, "B 的记录两份");
    let second_instance_root_indexes = &root_indexes[4..];
    // 只展开 B 的根槽段之后的段：种子是 B 之后才写下的，B 的根槽没持久而种子已在环里的状态走不到——
    // 取号的超级块槽都没持久时环里就有实例 1 的记录，checker 的 I-7.7（超级块实例代号不低于根环）① 正确地判红（先前全部展开时 3 个状态）。
    let second_publish_root_segment_index = prepared
        .segments
        .iter()
        .position(|segment| segment.contains(&second_publish_root_index))
        .expect("B 的根槽写在某一段里");
    let expand = |segment_index: usize, segment: &[usize]| {
        segment_index > second_publish_root_segment_index && segment.len() < 10
    };
    let mut states_whose_chain_reaches_the_residual_record = 0u64;
    let mut states_contradicting_the_predicate: Vec<String> = Vec::new();
    let tally = enumerate_layer0_selecting_versions_observing_each_state(
        &prepared.base,
        &prepared.writes,
        &prepared.segments,
        prepared.judged_root_index,
        &prepared.versions,
        &expand,
        &mut |crash_image, consulted_report| {
            let persisted = &crash_image.persisted;
            assert!(
                persisted[second_publish_root_index]
                    && second_publish_record_indexes
                        .iter()
                        .all(|record_index| persisted[*record_index]),
                "展开的状态里 B 的记录与根槽都已持久"
            );
            // B 的根槽已持久、所选根的实例表（mkfs 那一版）里没有回退行：水位还在 (1, 4)，链从 jsn 4 接到 jsn 5，直到实例 2 的某条根持久。
            let chain_reaches_the_residual_record = !second_instance_root_indexes
                .iter()
                .any(|root_index| persisted[*root_index]);
            if chain_reaches_the_residual_record {
                states_whose_chain_reaches_the_residual_record += 1;
            }
            let landed_on_the_residual_record = consulted_report.effective_root
                == Some(RESIDUAL_RECORD_ROOT)
                && consulted_report.outcome
                    == (RecoveryOutcome::FileRead {
                        root: (InstanceGeneration(1), CheckpointTxg(4)),
                        content: residual_content(),
                    });
            if chain_reaches_the_residual_record != landed_on_the_residual_record {
                states_contradicting_the_predicate.push(format!(
                    "谓词 {chain_reaches_the_residual_record}、实际走的根 {:?}、施加 {}",
                    consulted_report.effective_root, consulted_report.journal.prefix_applied
                ));
            }
        },
    );
    let expanded: Vec<Vec<usize>> = prepared
        .segments
        .iter()
        .enumerate()
        .filter(|(segment_index, segment)| expand(*segment_index, segment))
        .map(|(_, segment)| segment.clone())
        .collect();
    println!(
        "RESIDUAL_RECORD states={} states_whose_chain_reaches_the_residual_record={} violations={} checker_by_invariant(evaluated/violated/not_applicable) {}",
        tally.states,
        states_whose_chain_reaches_the_residual_record,
        tally.violations,
        tally.checker_counts_by_invariant()
    );
    assert!(
        states_contradicting_the_predicate.is_empty(),
        "该施加残留记录与恢复实际落在 (1, 5) 对不上的状态：{states_contradicting_the_predicate:?}"
    );
    assert_eq!(tally.states, closed_form_state_count(&expanded));
    assert_eq!(
        (tally.states, states_whose_chain_reaches_the_residual_record),
        (31, 19),
        "跑到的：B 与取号的超级块槽段 15 + 写行发布的记录段 3 + 写行发布的根槽段 1；跑不到的 12 个是实例 2 的某条根已持久"
    );
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
    // I-3.1（已分配统计对得上）在实例 2 的根为最新的 12 个状态上判红（口径未定，2026-09-17 写这条用例时发现）：记账的已分配逐盘比遍历候选根多 65536 字节，
    // 正是残留记录那一版（(1, 5)，根槽从没落盘、只由记录施加出来）自己的四个固定点单元 50261–50264——写行发布 txg 6 把它们释放进 defer，
    // 环里没有一条根引用它们。checker 的「已分配 = 候选根引用的并集」与分配器「defer 里的仍算已分配」在「由记录施加出来的那一版」上分歧，
    // 改哪一边是 I-3.1 口径的设计问题；这里钉的是现状，不是认下来的行为。
    assert_checker_and_record_checker_counts(&tally, &[("I-3.1", 12)], 0);
}

/// 改坏 tail 的 tail 值：窗口 [3, 18] 里有 A 那条记录（jsn 3），它点名的 50180 在 txg 18 被合法复用。
const STALE_JOURNAL_TAIL: u64 = 2;

/// 改坏 tail：录制流里每一次超级块槽写都换成 tail = `stale_tail` 的那一份（按字段重写、校验和重算，其余字段原样）。
fn writes_with_stale_journal_tail(writes: &[RetainedWrite], stale_tail: u64) -> Vec<RetainedWrite> {
    writes
        .iter()
        .map(|write| match write.kind {
            StepKind::SuperblockSlot => {
                let mut superblock =
                    Superblock::parse_slot(&write.bytes).expect("录到的超级块槽写都自证得过");
                assert_eq!(
                    superblock.to_slot(),
                    write.bytes,
                    "按字段重写一遍与录到的逐字节相同：改坏的只有 tail"
                );
                superblock.journal_tail = stale_tail;
                RetainedWrite {
                    device: write.device,
                    kind: write.kind,
                    is_force_unit_access: write.is_force_unit_access,
                    offset: write.offset,
                    bytes: superblock.to_slot(),
                }
            }
            StepKind::UnitWrite
            | StepKind::JournalRecord
            | StepKind::RootRecordFua
            | StepKind::Barrier => write.clone(),
        })
        .collect()
}

/// 环里 jsn 大于 `tail` 的记录中，点名的单元有一份校验和对不上的那些记录的 jsn。记录用恢复同一个扫描器解出，点名单元的比对在这里按位置条目逐份读、另算 CRC-32C。
fn records_after_tail_naming_a_mismatched_unit(
    crash_image: &CrashImage<'_>,
    tail: u64,
) -> Vec<u64> {
    let superblock = choose_superblock(crash_image).expect("超级块");
    scan_journal(crash_image, &superblock)
        .values()
        .filter(|record| record.counter > tail)
        .filter(|record| {
            record.named.iter().any(|named| {
                let unit_bytes = match named.unit_class {
                    UNIT_CLASS_INDEX_NODE => NODE_BYTES,
                    UNIT_CLASS_DATA | UNIT_CLASS_PACKED => DATA_UNIT_BYTES,
                    unknown_unit_class => panic!("记录点名了没登记的单元类 {unknown_unit_class}"),
                };
                named.locations.iter().any(|location| {
                    PoolReader::read(
                        crash_image,
                        location.device,
                        location.slot.to_device_offset(),
                        usize::try_from(unit_bytes).expect("单元字节数"),
                    )
                    .is_none_or(|bytes| crc32_castagnoli(&bytes) != location.unit_checksum)
                })
            })
        })
        .map(|record| record.counter)
        .collect()
}

/// 步 6 必红「陈旧 tail + 已复用的块」（verification-build.md 崩溃点重放第一版必红用例表；C77（重放起点未定义）；D23（journal 的角色与格式） 已定项 3
/// 那条 ⚠️ 与已定项 14）：固定脚本到 E 之后再覆盖写一次（txg 18 的数据单元落回 50180，A 那条记录点名的单元被合法复用），录制流里每次超级块槽写的
/// tail 都改成 2。txg 18 的 16 个单元写全持久之后的每个崩溃状态：恢复必须完成、终态与 tail 没改坏的同一个状态逐项相等、施加前验证一次都不失败；
/// 从陈旧 tail 起逐条验证、失配即中止的恢复在这些状态上中止，红在逐项相等那条。再注入一次真撕裂：施加前验证恰判失败一次（陈旧失配不进这个计数器）。
#[test]
fn stale_tail_with_a_reused_named_unit_in_its_window_recovers_every_crash_state_to_the_same_end_state(
) {
    let prepared = prepare(
        "layer0-stale-tail",
        Script::ReuseOfTheFirstDataUnitSlotAfterFloorRaisingPublish,
    );
    let stale_writes = writes_with_stale_journal_tail(&prepared.writes, STALE_JOURNAL_TAIL);
    let root_indexes: Vec<usize> = prepared
        .writes
        .iter()
        .enumerate()
        .filter(|(_, write)| write.kind == StepKind::RootRecordFua)
        .map(|(index, _)| index)
        .collect();
    let publish_after_raising_floor_root_index = root_indexes[16];
    let reuse_units_segment_index = prepared
        .segments
        .iter()
        .position(|segment| segment.contains(&(publish_after_raising_floor_root_index + 1)))
        .expect("E 的超级块槽写与 txg 18 的单元写同段");
    assert_eq!(prepared.segments[reuse_units_segment_index].len(), 18);
    let expand = |segment_index: usize, segment: &[usize]| {
        segment_index > reuse_units_segment_index && segment.len() < 10
    };
    let mut states_with_a_reused_named_unit_after_the_stale_tail = 0u64;
    let mut records_with_a_reused_named_unit: Vec<u64> = Vec::new();
    let mut states_differing_from_the_true_tail: Vec<String> = Vec::new();
    let tally = enumerate_layer0_selecting_versions_observing_each_state(
        &prepared.base,
        &stale_writes,
        &prepared.segments,
        prepared.judged_root_index,
        &prepared.versions,
        &expand,
        &mut |stale_tail_image, stale_tail_report| {
            assert_eq!(
                choose_superblock(stale_tail_image)
                    .expect("超级块")
                    .journal_tail,
                STALE_JOURNAL_TAIL,
                "展开的状态里择到的超级块都带陈旧的 tail"
            );
            let mismatched =
                records_after_tail_naming_a_mismatched_unit(stale_tail_image, STALE_JOURNAL_TAIL);
            if !mismatched.is_empty() {
                states_with_a_reused_named_unit_after_the_stale_tail += 1;
                records_with_a_reused_named_unit = mismatched;
            }
            let true_tail_image = CrashImage {
                base: &prepared.base,
                writes: &prepared.writes,
                persisted: stale_tail_image.persisted.clone(),
            };
            let true_tail_report = recover(&true_tail_image, JournalPolicy::Consult);
            if *stale_tail_report != true_tail_report {
                states_differing_from_the_true_tail.push(format!(
                    "tail 陈旧：{:?} 走 {:?}；tail 没改坏：{:?} 走 {:?}",
                    stale_tail_report.outcome,
                    stale_tail_report.effective_root,
                    true_tail_report.outcome,
                    true_tail_report.effective_root
                ));
            }
        },
    );
    println!(
        "STALE_TAIL states={} states_with_a_reused_named_unit_after_the_stale_tail={} records_with_a_reused_named_unit={:?} violations={} failed={} verification_failed={}",
        tally.states,
        states_with_a_reused_named_unit_after_the_stale_tail,
        records_with_a_reused_named_unit,
        tally.violations,
        tally.failed_states,
        tally.verification_failed_states
    );
    assert!(
        states_differing_from_the_true_tail.is_empty(),
        "改坏 tail 之后恢复的终态与 tail 没改坏的同一个状态不同：{states_differing_from_the_true_tail:?}"
    );
    assert_eq!(
        tally.states, 8,
        "txg 18 的记录段 3 + 根槽段 1 + 超级块槽段 3 + 全部持久 1"
    );
    assert_eq!(
        (
            states_with_a_reused_named_unit_after_the_stale_tail,
            records_with_a_reused_named_unit
        ),
        (8, vec![3]),
        "8 个状态里 txg 18 的单元写都已持久：陈旧 tail 之后 A 那条记录（jsn 3）点名的 50180 两份都已被复用"
    );
    assert_eq!(
        tally.violations, 0,
        "第一处违例：{:?}",
        tally.first_violation
    );
    assert_eq!(tally.failed_states, 0, "恢复在每个状态上都完成");
    assert_eq!(
        tally.verification_failed_states, 0,
        "陈旧失配不进施加前验证的计数器：水位之下的记录不验"
    );
    assert_eq!(tally.ignored_violations, 0);
    // 记录核对器第二条判据（恢复自称的 txg ≥ 某次发布、那次发布的某个单元两份都不在）在 8 个状态上都判：A（txg 3）的数据单元两份被 txg 18 合法复用了。
    // 这条判据写成时流里没有复用，它不认「被流里更晚、已持久的写盖掉」——口径未定（2026-09-17 写这条用例时发现），这里钉的是现状，不是认下来的行为。
    assert_checker_and_record_checker_counts(&tally, &[], 8);

    // 撕裂注入：txg 18 的记录已持久、根槽与超级块槽没持久，再把它点名的数据单元两份都改坏——施加前验证判失败、恢复停在 E (3, 17)、
    // 验证失败恰为 1 次：陈旧 tail 之后那条点名块已被复用的记录（jsn 3）不进这个计数器，真撕裂与陈旧失配分得开。
    let mut torn_writes = stale_writes.clone();
    let reused_data_unit_offset = SlotNumber(50180).to_device_offset();
    let mut torn_copies = 0;
    for (write_index, write) in torn_writes.iter_mut().enumerate() {
        if write_index > publish_after_raising_floor_root_index
            && write.kind == StepKind::UnitWrite
            && write.offset == reused_data_unit_offset
        {
            write.bytes[4000] ^= 0xff;
            torn_copies += 1;
        }
    }
    assert_eq!(torn_copies, 2, "txg 18 的数据单元两盘各一份");
    let persisted_before_the_reuse_root: Vec<bool> = (0..torn_writes.len())
        .map(|write_index| write_index < prepared.judged_root_index)
        .collect();
    let mut torn_tally = Layer0Tally::default();
    let torn_report = evaluate_state_for_versions(
        &prepared.base,
        &torn_writes,
        persisted_before_the_reuse_root,
        prepared.judged_root_index,
        &prepared.versions,
        &mut torn_tally,
    );
    assert_eq!(
        (
            torn_report.effective_root,
            torn_report.journal.verification_failed,
            torn_report.journal.prefix_applied
        ),
        (Some((InstanceGeneration(3), CheckpointTxg(17))), 1, 0),
        "撕裂的那条被旗标、不施加：{:?}",
        torn_report.journal
    );
    assert_eq!(
        torn_tally.violations, 0,
        "停在 E、读出 E 的内容：{:?}",
        torn_tally.first_violation
    );
}
