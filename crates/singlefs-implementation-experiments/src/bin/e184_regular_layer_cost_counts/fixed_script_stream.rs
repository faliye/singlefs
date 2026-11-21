#![allow(
    dead_code,
    reason = "照抄的测试代码里只有 prepare_the_fixed_script_stream 的这一支被调到，别的脚本变体、常量与函数在这个装置里没人用"
)]

use super::experiment_pool::{
    build_pool, file_content, geometry, parameters, BuiltPool, FIXED_WRITE_TIME_SECONDS,
};
use singlefs_core::address::{CheckpointTxg, InstanceGeneration};
use singlefs_core::mount::{
    mount_writable, raise_rollback_floor, roll_back_by_a_forward_publish, unmount,
    RollbackTarget, ShadowLedger, UnmountRaisedTheFloor, Unmounted,
};
use singlefs_core::transaction::{
    publish_overwrite, FirstFile, PoolVersion, PoolWriter, TransactionOutput,
};
use singlefs_core::unit::{UNIT_CLASS_INDEX_NODE, UNIT_CLASS_PACKED};
use singlefs_format::{DATA_UNIT_BYTES, NODE_BYTES, SLOT_BYTES};
use singlefs_verification::memory_pool::{
    writes_and_segments_with_stream_indexes_and_entries, MemoryPool, PublishedVersion,
    RetainedWrite,
};
use singlefs_verification::segments::StepKind;
use singlefs_verification::{RecordedEntrySpan, RecordedPublishEntry};
use std::sync::Arc;

// COPIED-BEGIN fixed_script_head crates/singlefs-verification/tests/crash_enumeration_fixed_script_stream.rs:55-92
const SECOND_FILE_BYTES: usize = 4100;

fn second_content() -> Vec<u8> {
    (0..SECOND_FILE_BYTES)
        .map(|index| u8::try_from((index * 7 + 3) % 253).expect("小于 256"))
        .collect()
}

struct PreparedFixedScriptStream {
    base: MemoryPool,
    writes: Vec<RetainedWrite>,
    segments: Vec<Vec<usize>>,
    /// 被判的那次根槽 FUA 写在写表里的下标：脚本最后一次发布的根（到 B 为止就是 B 的，到 C 为止是 C 的）。
    judged_root_index: usize,
    /// A 的根槽 FUA 写在写表里的下标：它之后的写都是 B 的。
    first_root_index: usize,
    versions: Vec<PublishedVersion>,
}

/// 固定脚本跑到哪一步（里程碑「覆盖写、释放、回退与复用」步 0）：到 B 为止是靶向对照用的两次发布流；到 E 再正常卸载是崩溃点重放全量跑的那条流。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Script {
    SecondVersionOnly,
    /// 到 C 为止：B 之后进程重开、可写挂载（取号、写行、暖机两次），再发布 C。
    ThirdVersion,
    /// 到 D 为止：C 之后在同一个会话里挂着的时候回退到 A 的根 (1, 3)——一次向前发布 D（txg 9，实例仍是 2，
    /// D23（journal 的角色与格式） 已定项 14），不取号、不写行、不暖机。
    RollbackToFirstVersion,
    /// 到 E 为止再正常卸载：回退之后再覆盖写五次（txg 10–14，第一次释放 D 复活的 A 的数据单元、释放代 10），抬 F 到 11
    /// （两次空发布 txg 15、16），再发布 E（txg 17）——数据单元落回 mkfs 实例表那两槽 50176；最后正常卸载（B1，D16（发布语义）
    /// 已定项 1「正常卸载」）：先把 F = 17 写进每块盘的系统配置、过一道屏障，再推带卸载记号的空发布直到每块盘上都有一条（txg 18、19）。
    /// 这是崩溃点重放全量跑的那条流。卸载放在最后：它把 F 抬到现行 txg，排在回退之前的话回退候选集里就没有 A 了（D16（发布语义）
    /// 已定项 1 的候选集要 txg ≥ F_生效，与「抬 F 排在回退之后」同一个理由）。
    ReuseAfterRaisingFloorThenNormalUnmount,
    /// 到 E 之后（不卸载）再覆盖写两次：txg 18 的数据单元落回 50178–50179（mkfs 那片第 0 版树表那一槽起，没有哪条记录点名它），
    /// txg 19 的数据单元落回 A 的数据单元那一对槽 50180——环里第一条点名块被合法复用的记录是 A 那一条。
    ReuseOfTheFirstDataUnitSlotAfterFloorRaisingPublish,
}
// COPIED-END fixed_script_head

// COPIED-BEGIN fixed_script_body crates/singlefs-verification/tests/crash_enumeration_fixed_script_stream.rs:139-574
/// 回退 D（txg 9）那次发布在零根的谱系树上第一次追加谱系记录（D18（块里携带什么信息） 已定项 24 第 5 条），多写的两个单元的落点：
/// 谱系容器（码 3 打包记录单元，32768 字节）与谱系树根（码 2 索引节点，16384 字节），两块盘同槽各一份
/// （这条流上量到的槽号）。
const LINEAGE_CONTAINER_SLOT_OF_THE_ROLLBACK_PUBLISH: u64 = 50352;
const LINEAGE_TREE_ROOT_SLOT_OF_THE_ROLLBACK_PUBLISH: u64 = 50354;

/// 回退 D 是这条脚本的第几条根（从 0 数：实例 1 的 txg 1–4，实例 2 的 txg 5–8，D 是 txg 9）。
const ROOT_INDEX_OF_THE_ROLLBACK_PUBLISH: usize = 8;

/// 回退 D 那次发布的单元写里落在 `slot` 起、单元类 `unit_class`、整单元 `unit_bytes` 字节的有几个（每块盘一份就是 2）。
fn units_of_the_rollback_publish_at(
    writes: &[RetainedWrite],
    root_indexes: &[usize],
    slot: u64,
    unit_class: u8,
    unit_bytes: u64,
) -> usize {
    let after_the_root_before_it = root_indexes[ROOT_INDEX_OF_THE_ROLLBACK_PUBLISH - 1] + 1;
    let its_root = root_indexes[ROOT_INDEX_OF_THE_ROLLBACK_PUBLISH];
    writes[after_the_root_before_it..its_root]
        .iter()
        .filter(|write| write.kind == StepKind::UnitWrite)
        .filter(|write| write.offset.0 == slot * SLOT_BYTES)
        .filter(|write| {
            write.bytes().is_some_and(|bytes| {
                u64::try_from(bytes.len()).expect("一个单元的字节数装得进 u64") == unit_bytes
                    && bytes[6] == unit_class
            })
        })
        .count()
}

/// 到 E 之后再覆盖写一次（txg 18）那一版的数据单元落点。
const TXG_18_DATA_UNIT_SLOT: u64 = 50178;
/// A（txg 3）的数据单元那一对槽：到 E 之后第二次覆盖写（txg 19）复用它。
const FIRST_DATA_UNIT_SLOT: u64 = 50180;

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

/// 正常卸载（影子账开着），录制流上把这一段记成卸载入口发的（C557（卸载记号只由卸载入口打没有检查））：带文件的一版上卸载必抬 F，
/// 交回那一串；现行版本换成那一串最后落盘的那一次。
fn unmount_recorded(pool: &mut BuiltPool) -> UnmountRaisedTheFloor {
    let mut current = PoolVersion::WithFile(Arc::new(pool.output.clone()));
    let stream = pool.stream.clone();
    let devices = pool.devices.as_mut().expect("镜像还开着");
    let allocator = &mut pool.allocator;
    let unmounted = stream
        .record_entry(RecordedPublishEntry::Unmount, || {
            unmount(
                &parameters(),
                devices,
                allocator,
                &mut current,
                ShadowLedger::On,
            )
        })
        .expect("正常卸载");
    pool.output = current
        .into_file_version()
        .expect("带文件的一版卸载之后仍带文件");
    match unmounted {
        Unmounted::FloorRaisedToTheCurrentVersion(raised) => raised,
        Unmounted::NothingWrittenOnAVersionWithoutFile {
            current: reported_version,
        } => {
            panic!("带文件的一版上卸载报成「树表 0 条、一个字节都不写」：{reported_version:?}")
        }
    }
}

/// mkfs 之后那一段录制流切成写表与段：入口段（卸载入口发的那几段）一起交给切段，带卸载记号的根槽写只许落在卸载入口里（C557）。
fn writes_and_segments_after_mkfs(pool: &BuiltPool) -> (Vec<RetainedWrite>, Vec<Vec<usize>>) {
    let operations = pool.retained_operations();
    let entry_spans: Vec<RecordedEntrySpan> = pool
        .stream
        .entry_spans()
        .into_iter()
        .map(|span| RecordedEntrySpan {
            entry: span.entry,
            operations: span.operations.start - pool.mkfs_operation_count
                ..span.operations.end - pool.mkfs_operation_count,
        })
        .collect();
    let (writes, segments, _stream_indexes) = writes_and_segments_with_stream_indexes_and_entries(
        &operations[pool.mkfs_operation_count..],
        &entry_spans,
        &geometry(),
    );
    (writes, segments)
}

fn prepare_the_fixed_script_stream(tag: &str, script: Script) -> PreparedFixedScriptStream {
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
        // 进程重开：取号两轮各 2、写行、暖机两次（txg 5 / 6 落盘 0、txg 7 落盘 1），文件还是第二次的内容。
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
            // 同一个会话里挂着的时候回退到 A 的根：一次向前发布 D（txg 9 落盘 0），文件回到第一次的内容。
            {
                let rollback_parameters = parameters();
                let open_devices = pool.devices.as_mut().expect("镜像还开着");
                roll_back_by_a_forward_publish(
                    &rollback_parameters,
                    open_devices,
                    &mut pool.allocator,
                    &mut pool.output,
                    RollbackTarget {
                        instance: InstanceGeneration(1),
                        checkpoint_txg: CheckpointTxg(3),
                    },
                )
                .expect("挂着的时候回退到 A");
            }
            versions.push(PublishedVersion {
                instance: InstanceGeneration(2),
                checkpoint_txg: CheckpointTxg(9),
                content: file_content(),
            });
            if matches!(
                script,
                Script::ReuseAfterRaisingFloorThenNormalUnmount
                    | Script::ReuseOfTheFirstDataUnitSlotAfterFloorRaisingPublish
            ) {
                let mut latest = Vec::new();
                for (txg, seed) in [(10u64, 13usize), (11, 17), (12, 19), (13, 23), (14, 29)] {
                    latest = later_content(seed);
                    pool.output = overwrite(&mut pool, &latest, InstanceGeneration(2));
                    versions.push(PublishedVersion {
                        instance: InstanceGeneration(2),
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
                        instance: InstanceGeneration(2),
                        checkpoint_txg: CheckpointTxg(txg),
                        content: latest.clone(),
                    });
                }
                let reuse = overwrite(&mut pool, &later_content(31), InstanceGeneration(2));
                assert_eq!(
                    reuse.data_pointers[0].locations[0].slot.0,
                    50176,
                    "E 的数据单元落回最低的可再分配偶数槽对：mkfs 实例表那两槽（释放代 5，抬 F 到 11 回收了）"
                );
                versions.push(PublishedVersion {
                    instance: InstanceGeneration(2),
                    checkpoint_txg: CheckpointTxg(17),
                    content: later_content(31),
                });
                pool.output = reuse;
                match script {
                    Script::ReuseAfterRaisingFloorThenNormalUnmount => {
                        let unmounted = unmount_recorded(&mut pool);
                        assert_eq!(
                            unmounted
                                .publishes
                                .iter()
                                .map(|publish| publish.root.checkpoint_txg)
                                .collect::<Vec<_>>(),
                            vec![CheckpointTxg(18), CheckpointTxg(19)],
                            "卸载推两次空发布：txg 18 落盘 0、txg 19 落盘 1（17 除以 3 余 2）"
                        );
                        for txg in 18..=19 {
                            versions.push(PublishedVersion {
                                instance: InstanceGeneration(2),
                                checkpoint_txg: CheckpointTxg(txg),
                                content: later_content(31),
                            });
                        }
                    }
                    Script::ReuseOfTheFirstDataUnitSlotAfterFloorRaisingPublish => {
                        for (txg, seed, data_unit_slot, why) in [
                            (
                                18u64,
                                37usize,
                                TXG_18_DATA_UNIT_SLOT,
                                "txg 18 的数据单元落回下一对可再分配的槽（抬 F 回收了、E 用掉的是更低的 50176）",
                            ),
                            (
                                19,
                                41,
                                FIRST_DATA_UNIT_SLOT,
                                "txg 19 的数据单元落回 A 的数据单元那一对槽（释放代 10，抬 F 到 11 回收了）",
                            ),
                        ] {
                            let later = overwrite(&mut pool, &later_content(seed), InstanceGeneration(2));
                            assert_eq!(
                                later.data_pointers[0].locations[0].slot.0,
                                data_unit_slot,
                                "{why}"
                            );
                            versions.push(PublishedVersion {
                                instance: InstanceGeneration(2),
                                checkpoint_txg: CheckpointTxg(txg),
                                content: later_content(seed),
                            });
                            pool.output = later;
                        }
                    }
                    Script::SecondVersionOnly
                    | Script::ThirdVersion
                    | Script::RollbackToFirstVersion => {
                        unreachable!("外面那一层只放行到 E 的两个脚本")
                    }
                }
            }
        }
    }
    let base = pool.memory_pool_after_mkfs();
    let (writes, segments) = writes_and_segments_after_mkfs(&pool);
    let root_indexes: Vec<usize> = writes
        .iter()
        .enumerate()
        .filter(|(_, write)| write.kind == StepKind::RootRecordFua)
        .map(|(index, _)| index)
        .collect();
    // 谱系那两个单元的槽号先于段序列钉：回退不追加谱系记录时 D 那一段从 24 退回 20，段序列那一条也红，这一条先说清缺的是谱系那两个单元。
    match script {
        Script::SecondVersionOnly | Script::ThirdVersion => {}
        Script::RollbackToFirstVersion
        | Script::ReuseAfterRaisingFloorThenNormalUnmount
        | Script::ReuseOfTheFirstDataUnitSlotAfterFloorRaisingPublish => {
            assert_eq!(
                (
                    units_of_the_rollback_publish_at(
                        &writes,
                        &root_indexes,
                        LINEAGE_CONTAINER_SLOT_OF_THE_ROLLBACK_PUBLISH,
                        UNIT_CLASS_PACKED,
                        DATA_UNIT_BYTES
                    ),
                    units_of_the_rollback_publish_at(
                        &writes,
                        &root_indexes,
                        LINEAGE_TREE_ROOT_SLOT_OF_THE_ROLLBACK_PUBLISH,
                        UNIT_CLASS_INDEX_NODE,
                        NODE_BYTES
                    )
                ),
                (2, 2),
                "回退 D 那次发布写出谱系容器（码 3，槽 {LINEAGE_CONTAINER_SLOT_OF_THE_ROLLBACK_PUBLISH}）与谱系树根（码 2，槽 {LINEAGE_TREE_ROOT_SLOT_OF_THE_ROLLBACK_PUBLISH}），两块盘各一份"
            );
        }
    }
    let sizes: Vec<usize> = segments.iter().map(Vec::len).collect();
    // 每次发布：它的单元写、两份 journal 记录、一条根槽写、之后两块盘各一次系统配置槽轮换，共「单元写数 + 5」次写；
    // C577 之后轮换之后、发布返回之前有一道屏障，上一次发布的系统配置槽轮换自成一段，不再与这一次的单元写并成一段
    // （登记表八末尾那条 ⚠️ 说的并段不再有）。单元写数：暖机 0、A 与 B 与 C 各 24、
    // 写行发布 18、写行之后的暖机各 16、回退 D 24（分配记录树 7 个节点 + 记账 + 映射 + 树表 = 10 个固定点单元，加谱系容器与谱系树根
    // 2 个——D18（块里携带什么信息） 已定项 24 第 5 条，回退那次追加一条谱系记录；加谱系记录之前 20）、覆盖写与 E 各 28、
    // 抬 F 与卸载推的空发布各 16。取号、抬 F 与卸载那一串开头各多
    // 两块盘一次系统配置写（取号写实例代号，抬 F 与卸载先写新 F，D16（发布语义） 已定项 1「抬 F 那一串」），它们之前各一道屏障
    // （代码审阅第 19 条：覆写较旧那一槽之前，上一次轮换写的那一槽先持久），与上一次的轮换各自成段（2、2）；
    // 取号写两轮（D22（单元原子性怎么合成） 已定项 8 落地），每轮两盘各一写、轮后池屏障，两轮各成一段（2、2），这条流开头与 B 之后重开各一处。
    match script {
        Script::SecondVersionOnly => {
            assert_eq!(
                sizes,
                vec![2, 2, 2, 1, 2, 2, 1, 2, 24, 2, 1, 2, 24, 2, 1, 2],
                "取号两轮各成一段；A 的两个系统配置槽写自成一段、B 的 24 个单元写自成一段：C577 之后轮换之后有屏障（改之前并成 26）"
            );
            assert_eq!(
                writes.len(),
                72,
                "取号两轮各 2 + 暖机 5 × 2 + A 29 + B 29 次写"
            );
        }
        Script::ThirdVersion => {
            // B 的两个系统配置槽写自成一段、取号两轮各两个、各自成一段（2、2、2：取号写之前与每轮之后有屏障）；写行发布 18 个单元写自成一段（取号后一轮之后有屏障）；
            // 暖机两次各 16 个单元写、C 的 24 个单元写各自成一段，上一次发布的两个系统配置槽写在它们前面自成一段（C577 之前并成 18、18、26）。
            assert_eq!(
                sizes,
                vec![
                    2, 2, 2, 1, 2, 2, 1, 2, 24, 2, 1, 2, 24, 2, 1, 2, 2, 2, 18, 2, 1, 2, 16, 2, 1,
                    2, 16, 2, 1, 2, 24, 2, 1, 2
                ],
                "固定脚本到 C 为止的段序列"
            );
            assert_eq!(
                writes.len(),
                72 + 4 + 23 + 21 + 21 + 29,
                "到 B 72 + 取号两轮 4 + 写行 23 + 暖机 21 × 2 + C 29"
            );
        }
        Script::RollbackToFirstVersion => {
            // 挂着时的向前回退 D：24 个单元写自成一段（10 个固定点单元加谱系容器与谱系树根，各两块盘；加谱系记录之前 20）、
            // C 的两个系统配置槽写在它前面自成一段（C577 之前并成 22），不取号、不写行、不暖机。
            assert_eq!(
                sizes,
                vec![
                    2, 2, 2, 1, 2, 2, 1, 2, 24, 2, 1, 2, 24, 2, 1, 2, 2, 2, 18, 2, 1, 2, 16, 2, 1,
                    2, 16, 2, 1, 2, 24, 2, 1, 2, 24, 2, 1, 2
                ],
                "固定脚本到 D 为止的段序列"
            );
            assert_eq!(writes.len(), 170 + 29, "到 C 170 + D 29");
        }
        Script::ReuseAfterRaisingFloorThenNormalUnmount => {
            // 五次覆盖写各 28 个单元写自成一段、上一次的两个系统配置槽写在它前面自成一段（C577 之前并成 30）；txg 14 的轮换自成一段、
            // 抬 F 先写系统配置自成一段（2、2），两次空发布各 16 个单元写各自成一段（第二次前面是第一次的轮换 2，C577 之前并成 18）；
            // E 同覆盖写（2、28）；卸载同抬 F（2、2、16、2、16），末尾是 txg 19 的轮换。
            assert_eq!(
                sizes,
                vec![
                    2, 2, 2, 1, 2, 2, 1, 2, 24, 2, 1, 2, 24, 2, 1, 2, 2, 2, 18, 2, 1, 2, 16, 2, 1,
                    2, 16, 2, 1, 2, 24, 2, 1, 2, 24, 2, 1, 2, 28, 2, 1, 2, 28, 2, 1, 2, 28, 2, 1,
                    2, 28, 2, 1, 2, 28, 2, 1, 2, 2, 16, 2, 1, 2, 16, 2, 1, 2, 28, 2, 1, 2, 2, 16,
                    2, 1, 2, 16, 2, 1, 2
                ],
                "固定脚本到 E 再正常卸载的段序列"
            );
            assert_eq!(
                writes.len(),
                199 + 5 * 33 + (2 + 2 * 21) + 33 + (2 + 2 * 21),
                "到 D 199 + 五次覆盖写 165 + 抬 F 44 + E 33 + 卸载 44"
            );
        }
        Script::ReuseOfTheFirstDataUnitSlotAfterFloorRaisingPublish => {
            // 到 E 的段序列末尾那段（E 的两个系统配置槽写）自成一段，txg 18 的 28 个单元写自成一段（C577 之前并成 30），txg 19 同形。
            assert_eq!(
                sizes,
                vec![
                    2, 2, 2, 1, 2, 2, 1, 2, 24, 2, 1, 2, 24, 2, 1, 2, 2, 2, 18, 2, 1, 2, 16, 2, 1,
                    2, 16, 2, 1, 2, 24, 2, 1, 2, 24, 2, 1, 2, 28, 2, 1, 2, 28, 2, 1, 2, 28, 2, 1,
                    2, 28, 2, 1, 2, 28, 2, 1, 2, 2, 16, 2, 1, 2, 16, 2, 1, 2, 28, 2, 1, 2, 28, 2,
                    1, 2, 28, 2, 1, 2
                ],
                "固定脚本到 E 之后再覆盖写两次的段序列"
            );
            assert_eq!(
                writes.len(),
                199 + 5 * 33 + 44 + 33 + 2 * 33,
                "到 E 441 + txg 18、19 的覆盖写各 33"
            );
        }
    }
    let expected_roots = match script {
        Script::SecondVersionOnly => 4,
        Script::ThirdVersion => 8,
        Script::RollbackToFirstVersion => 9,
        Script::ReuseAfterRaisingFloorThenNormalUnmount
        | Script::ReuseOfTheFirstDataUnitSlotAfterFloorRaisingPublish => 19,
    };
    assert_eq!(root_indexes.len(), expected_roots, "每次发布一条根槽写");
    PreparedFixedScriptStream {
        base,
        writes,
        segments,
        judged_root_index: *root_indexes.last().expect("至少一条根槽写"),
        first_root_index: root_indexes[2],
        versions,
    }
}
// COPIED-END fixed_script_body

/// 装置要的几样：基镜像、写表、段、被判的根槽写下标、版本表。
pub struct PreparedFixedScript {
    pub base: MemoryPool,
    pub writes: Vec<RetainedWrite>,
    pub segments: Vec<Vec<usize>>,
    pub judged_root_index: usize,
    pub versions: Vec<PublishedVersion>,
}

pub fn prepare_through_the_unmount(tag: &str) -> PreparedFixedScript {
    let prepared = prepare_the_fixed_script_stream(tag, Script::ReuseAfterRaisingFloorThenNormalUnmount);
    PreparedFixedScript {
        base: prepared.base,
        writes: prepared.writes,
        segments: prepared.segments,
        judged_root_index: prepared.judged_root_index,
        versions: prepared.versions,
    }
}
