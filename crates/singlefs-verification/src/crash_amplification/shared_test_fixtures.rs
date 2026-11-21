//! 几个子模块的单测共用的搭建：小流、空库、存一块结论、观察者单测那一套、GPU 批处理服务与控制命令的替身。

use super::*;

/// 一块假的结论字：块里第一个状态全零（绿），第二个只置看 journal 那一遍 oracle 的头一类（恰好一项红），其余逐字取种子推出的数，
/// 判不了那一位与那一字清零（卡线程交来之前判不了的已经拒过）。
#[cfg(feature = "gpu")]
pub(super) fn verdict_words_of_a_made_up_block(
    seed: &mut u64,
    states: usize,
) -> Vec<[u32; crate::crash_judge_gpu::VERDICT_WORDS]> {
    use crate::crash_judge_gpu::{verdict_bits, POOL_CHECKER_UNDECIDABLE_WORD, VERDICT_WORDS};
    (0..states)
        .map(|position| {
            let mut state_words = [0u32; VERDICT_WORDS];
            match position {
                0 => {}
                1 => state_words[0] = 1 << verdict_bits::CONSULTED_ORACLE_FIRST,
                _ => {
                    for word in &mut state_words {
                        *word = u32::try_from(next_number_of_the_reuse_sentinel_seed(seed) >> 32)
                            .expect("高 32 位装得进 u32");
                    }
                    state_words[0] &= !(1 << verdict_bits::UNDECIDABLE);
                    state_words[POOL_CHECKER_UNDECIDABLE_WORD] = 0;
                }
            }
            state_words
        })
        .collect()
}

/// 造一条流：每段按（单元写数，journal 记录写数）给，单元写在前；每次写一个扇区（512 字节，不取第三态）、各写各的块。
/// 最后两段是一次根槽写、两次系统配置槽写（段写数 1、2）。一个崩溃点罩全部写、不 ignore。
pub(super) fn flow_parts_of_unit_and_journal_writes(
    unit_and_journal_writes_of_the_long_segments: &[(usize, usize)],
) -> (Vec<RetainedWrite>, Vec<Vec<usize>>) {
    let one_sector_write = |write_index: usize, kind: StepKind| RetainedWrite {
        device: singlefs_core::address::DeviceIdentity(1),
        kind,
        is_force_unit_access: kind == StepKind::RootRecordFua,
        offset: singlefs_core::address::DeviceOffsetInBytes(
            u64::try_from(write_index).expect("写表下标装得进 u64") * 4096,
        ),
        contents: WrittenContents::Bytes(vec![1; 512]),
    };
    let mut writes = Vec::new();
    let mut segments = Vec::new();
    let mut segment_of_kinds = |kinds: Vec<StepKind>, writes_so_far: &mut Vec<RetainedWrite>| {
        let segment: Vec<usize> = kinds
            .into_iter()
            .map(|kind| {
                writes_so_far.push(one_sector_write(writes_so_far.len(), kind));
                writes_so_far.len() - 1
            })
            .collect();
        segments.push(segment);
    };
    for (unit_writes, journal_writes) in unit_and_journal_writes_of_the_long_segments {
        let kinds = std::iter::repeat_n(StepKind::UnitWrite, *unit_writes)
            .chain(std::iter::repeat_n(
                StepKind::JournalRecord,
                *journal_writes,
            ))
            .collect();
        segment_of_kinds(kinds, &mut writes);
    }
    segment_of_kinds(vec![StepKind::RootRecordFua], &mut writes);
    segment_of_kinds(vec![StepKind::SystemConfigurationSlot; 2], &mut writes);
    (writes, segments)
}

/// 造一条只有录入段那几样的流（C790 分片用）：`flow_parts_of_unit_and_journal_writes` 那一形，空的基础镜像，一个崩溃点罩全部写；
/// 第 0 段登记、按 [`expansion_of_a_registered_segment`] 照展开上限 6 展开（超过的取甲二），别的段照展开上限 6。交给 `use_the_flow`。
pub(super) fn with_a_flow_registering_its_first_segment<Outcome>(
    unit_and_journal_writes_of_the_long_segments: &[(usize, usize)],
    use_the_flow: impl FnOnce(&CrashFlow<'_>, &[RegisteredSegment]) -> Outcome,
) -> Outcome {
    with_a_flow_registering_its_first_segment_and_expanding_the_others_up_to(
        unit_and_journal_writes_of_the_long_segments,
        ExpandUpToWritesPerSegment(6),
        use_the_flow,
    )
}

/// 同 [`with_a_flow_registering_its_first_segment`]，登记的那一段与别的段都照 `expand_up_to` 展开（登记的那一段超过的取甲二）。
pub(super) fn with_a_flow_registering_its_first_segment_and_expanding_the_others_up_to<Outcome>(
    unit_and_journal_writes_of_the_long_segments: &[(usize, usize)],
    expand_up_to: ExpandUpToWritesPerSegment,
    use_the_flow: impl FnOnce(&CrashFlow<'_>, &[RegisteredSegment]) -> Outcome,
) -> Outcome {
    let (writes, segments) =
        flow_parts_of_unit_and_journal_writes(unit_and_journal_writes_of_the_long_segments);
    let base = MemoryPool {
        devices: BTreeMap::new(),
        device_size_in_bytes: 1 << 30,
    };
    let registered = vec![RegisteredSegment {
        segment_index: 0,
        write_count: segments[0].len(),
        write_table_digest_text: "cc".repeat(DIGEST_BYTES),
    }];
    let expansion_with_the_registered_segment = |segment_index: usize, segment: &[usize]| {
        if segment_index == 0 {
            expansion_of_a_registered_segment(segment, expand_up_to)
        } else {
            expand_up_to.expansion_of(segment)
        }
    };
    let judged_root_index = writes
        .iter()
        .rposition(|write| write.kind == StepKind::RootRecordFua)
        .expect("造的流有一次根槽写");
    let flow = CrashFlow {
        base: &base,
        writes: &writes,
        segments: &segments,
        judged_root_index,
        versions: &[],
        versions_without_a_file: &[],
        expansion: &expansion_with_the_registered_segment,
        crash_points: vec![CrashPointSpan {
            name: "probe".to_string(),
            code_file_in_repository: "crates/singlefs-verification/src/crash_amplification.rs"
                .to_string(),
            writes: 0..writes.len(),
            ignored: false,
        }],
    };
    use_the_flow(&flow, &registered)
}

/// 一个空的判定库（临时 prune-server 里的一段一次性名字空间，用例名进名字空间；临时 prune-server 由它自己在这个进程里起，起不来当场失败）。
pub(super) fn an_empty_library_for(
    test_name: &str,
) -> (crate::crash_library::DisposableLibrary, VerdictStore) {
    // 一次性库留着：分片的续点从空库数
    let directory =
        crate::crash_library::DisposableLibrary::new(&format!("c790-slices-{test_name}"))
            .unwrap_or_else(|refusal| panic!("{refusal}"));
    let store = directory
        .open_store(crate::verdict_store::PruneServerFailureHandling::ReturnTheError)
        .expect("开得了一次性库");
    (directory, store)
}

/// 假的判段：计划里还没有判定块的块，每个状态记同一个判定向量（只钉块键、续点与进度，不判真的崩溃状态）。
pub(super) fn store_a_clean_verdict_for_every_unjudged_block(
    store: &mut VerdictStore,
    plan: &CrashPlan,
    version: &JudgingUnitsFingerprint,
) {
    let clean = store
        .register_verdict_vector(&crate::verdict_store::EncodedVerdictVector(
            b"clean".to_vec(),
        ))
        .expect("登记判定向量");
    for point in &plan.points {
        for local in point.blocks() {
            let key = point.block_key(&local);
            if store
                .read_verdict_block(&key, version)
                .expect("读块")
                .is_none()
            {
                let states = usize::try_from(local.end - local.start).expect("一块装得进 usize");
                store
                    .store_verdict_block(
                        &key,
                        version,
                        &VerdictBlock::new(vec![clean; states]).expect("一块至少一个状态"),
                        &[],
                    )
                    .expect("存块");
            }
        }
    }
}

// ------------------------------ 观察者两段与按恢复镜像去重 ------------------------------

/// 观察者两段那几条单测的设点文件与观察者代码文件（只当名字用：单测的观察者版本照下面另造，不经编译时的摘要表）。
pub(super) const OBSERVER_UNIT_TEST_SETTING_FILE: &str =
    "crates/singlefs-verification/src/crash_amplification.rs";
/// 那条小流唯一那个崩溃点的名字。
pub(super) const OBSERVER_UNIT_TEST_POINT_NAME: &str = "two_writes_of_the_same_bytes";

/// 一条小流：一块盘，一段两个单元写写同一个扇区、同一份字节（这一段展开任意真子集）。状态是一个都没落、只落前一个、只落后一个，
/// 加末节点（两个都落）：后三个的恢复镜像逐字节相同，与第一个不同。崩溃点一个、罩住两个写。
pub(super) fn a_flow_of_two_writes_of_the_same_bytes_to_one_sector() -> RecordedFlow {
    let device = singlefs_core::address::DeviceIdentity(1);
    let write = RetainedWrite {
        device,
        kind: StepKind::UnitWrite,
        is_force_unit_access: false,
        offset: singlefs_core::address::DeviceOffsetInBytes(8192),
        contents: WrittenContents::Bytes(vec![5; 512]),
    };
    RecordedFlow {
        base: MemoryPool::with_devices(&[device], 1 << 20),
        writes: vec![write.clone(), write],
        segments: vec![vec![0, 1]],
        judged_root_index: 0,
        versions: Vec::new(),
        versions_without_a_file: Vec::new(),
        segment_expansions: vec![CrashPointReplaySegmentExpansion::EveryProperSubset],
        crash_points: vec![CrashPointSpan {
            name: OBSERVER_UNIT_TEST_POINT_NAME.to_string(),
            code_file_in_repository: OBSERVER_UNIT_TEST_SETTING_FILE.to_string(),
            writes: 0..2,
            ignored: false,
        }],
        segment_slices: vec![None],
    }
}

/// 两次写落在两个不同的扇区（字节也不同）：段内三个状态加末节点，四个状态的镜像字节两两不同；只读第一个扇区的观察者按读集合成两张。
pub(super) fn a_flow_of_two_writes_to_two_sectors() -> RecordedFlow {
    let device = singlefs_core::address::DeviceIdentity(1);
    let write_to = |sector: u64, byte: u8| RetainedWrite {
        device,
        kind: StepKind::UnitWrite,
        is_force_unit_access: false,
        offset: singlefs_core::address::DeviceOffsetInBytes(
            sector * crate::memory_pool::SECTOR_BYTES,
        ),
        contents: WrittenContents::Bytes(vec![byte; 512]),
    };
    RecordedFlow {
        base: MemoryPool::with_devices(&[device], 1 << 20),
        writes: vec![
            write_to(TWO_SECTOR_FLOW_READ_SECTOR, 5),
            write_to(TWO_SECTOR_FLOW_UNREAD_SECTOR, 7),
        ],
        segments: vec![vec![0, 1]],
        judged_root_index: 0,
        versions: Vec::new(),
        versions_without_a_file: Vec::new(),
        segment_expansions: vec![CrashPointReplaySegmentExpansion::EveryProperSubset],
        crash_points: vec![CrashPointSpan {
            name: OBSERVER_UNIT_TEST_POINT_NAME.to_string(),
            code_file_in_repository: OBSERVER_UNIT_TEST_SETTING_FILE.to_string(),
            writes: 0..2,
            ignored: false,
        }],
        segment_slices: vec![None],
    }
}

pub(super) const TWO_SECTOR_FLOW_READ_SECTOR: u64 = 16;
const TWO_SECTOR_FLOW_UNREAD_SECTOR: u64 = 20;

/// 单测的观察者那几把版本：观察者那一把随另读的输入变，代码那一把不随（与 [`observer_versions_keeping_the_refused_out`] 同形，摘要另造）。
pub(super) fn versions_of_the_unit_test_observer(
    extra_inputs_digest: [u8; DIGEST_BYTES],
) -> ObserverVersions {
    let observer = ConclusionVersion(sha256_digest(
        &[
            b"observer version of the unit test\n".as_slice(),
            &extra_inputs_digest,
        ]
        .concat(),
    ));
    let pool_checker_version =
        ConclusionVersion(sha256_digest(b"cpu judge version of the unit test\n"));
    ObserverVersions {
        observer,
        observer_code: ConclusionVersion(sha256_digest(
            b"observer code version of the unit test\n",
        )),
        pool_checker: JudgedBy::CpuJudge,
        pool_checker_version,
        conclusion: ConclusionVersion(sha256_digest(
            &[observer.0, pool_checker_version.0].concat(),
        )),
    }
}

// ------------------------------ 两槽全废盘面观察者在判段（判器交回的逐盘自证槽数「你做你的 GPU版让GPU去做」） ------------------------------

/// 判器那一块的结论（每个状态同一个空的判定向量、不违例、按发布分的态数全记在一次发布下）写进 `store`，`slots_of_each_ordinal` 给了就连同
/// 判器交回的逐盘自证槽数一起写（经判段写结论块的那一处：[`ConclusionsStoredUnder::store_block`]），键与结论块同一把。
pub(super) fn store_a_judge_conclusion_for_every_block(
    store: &mut VerdictStore,
    plan: &CrashPlan,
    conclusions: &ConclusionsStoredUnder<'_>,
    slots_of_each_device_on_the_base_image: &[(u32, u32)],
    slots_of_each_ordinal: Option<&dyn Fn(u64) -> Vec<(u32, u32)>>,
) {
    let clean = store
        .register_verdict_vector(&encoded_verdict_of_the_red_items(&[]))
        .expect("登记得了");
    for (point_index, point) in plan.points.iter().enumerate() {
        for local in point.blocks() {
            let states = local.end - local.start;
            let mut slots_of_the_states =
                SelfVerifiedSystemConfigurationSlotsOfTheStates::default();
            if let Some(slots_of_the_ordinal) = slots_of_each_ordinal {
                for ordinal in point.ordinals.start + local.start..point.ordinals.start + local.end
                {
                    slots_of_the_states.push_a_state(slots_of_the_ordinal(ordinal));
                }
            }
            conclusions
                .store_block(
                    store,
                    point_index,
                    &point.block_key(&local),
                    VerdictBlock::new(vec![
                        clean;
                        usize::try_from(states).expect("一块装得进 usize")
                    ])
                    .expect("一块至少一个状态"),
                    Vec::new(),
                    StoredAlongTheVerdicts {
                        count_vector: BlockCountVector {
                            per_invariant: BTreeMap::new(),
                            per_observer: BTreeMap::new(),
                            states_by_publish: BTreeMap::from([(
                                "unit_test_publish".to_string(),
                                states,
                            )]),
                        },
                        recovery_words: None,
                        judged_on: JudgingDevice::Cpu,
                        self_verified_system_configuration_slots: slots_of_the_states
                            .handed_back_for_the_block(
                                states,
                                slots_of_each_device_on_the_base_image,
                            ),
                    },
                )
                .expect("写得进库");
        }
    }
}

/// 那条小流挂上两槽全废盘面观察者（判段从判器交回的逐盘自证槽数看的那一种），挂在它的点与末节点上。
pub(super) fn the_two_slot_observer_on_the_nodes_of_the_unit_test_flow(
) -> ObserverOnTheNodes<'static> {
    ObserverOnTheNodes {
            observer_name: ObserverName::new("unit_test_two_slot_observer").expect("名字合规矩"),
            observer_code_file: OBSERVER_UNIT_TEST_SETTING_FILE.to_string(),
            extra_inputs_digest: [0; DIGEST_BYTES],
            nodes: vec![
                NodeOfAnObserver {
                    setting_file: OBSERVER_UNIT_TEST_SETTING_FILE.to_string(),
                    entry_name: OBSERVER_UNIT_TEST_POINT_NAME.to_string(),
                },
                NodeOfAnObserver {
                    setting_file: OBSERVER_UNIT_TEST_SETTING_FILE.to_string(),
                    entry_name: END_OF_FLOW_NODE_NAME.to_string(),
                },
            ],
            observe: ObserverCode::FromTheSelfVerifiedSystemConfigurationSlotsTheJudgeHandsBack(
                &crate::crash_point_replay_provenance::observe_the_devices_without_a_self_verified_system_configuration_slot,
            ),
        }
}

/// 一块判器结论写进 `store`（经判段写结论块的那一处 [`ConclusionsStoredUnder::store_block`]）：每个状态同一个判定向量（`red_item` 给了就是只有它一项的红，
/// 块里头一个状态记违例；没给就是空的、不违例），按发布分的态数全记在 `publish` 下，连同判器交回的逐盘自证槽数（每个状态一份空的）。
pub(super) fn store_a_judge_conclusion_of_the_block(
    store: &mut VerdictStore,
    plan: &CrashPlan,
    conclusions: &ConclusionsStoredUnder<'_>,
    point_index: usize,
    local: &Range<u64>,
    red_item: Option<&str>,
    publish: &str,
) {
    let items: Vec<String> = red_item.map(str::to_string).into_iter().collect();
    let number = store
        .register_verdict_vector(&encoded_verdict_of_the_red_items(&items))
        .expect("登记得了");
    let states = local.end - local.start;
    let mut slots_of_the_states = SelfVerifiedSystemConfigurationSlotsOfTheStates::default();
    for _ in local.start..local.end {
        slots_of_the_states.push_a_state(Vec::new());
    }
    let violating = if red_item.is_some() {
        vec![StateOffsetInBlock(0)]
    } else {
        Vec::new()
    };
    let key = plan.points[point_index].block_key(local);
    conclusions
        .store_block(
            store,
            point_index,
            &key,
            VerdictBlock::new(vec![
                number;
                usize::try_from(states).expect("一块装得进 usize")
            ])
            .expect("一块至少一个状态"),
            violating,
            StoredAlongTheVerdicts {
                count_vector: BlockCountVector {
                    per_invariant: BTreeMap::new(),
                    per_observer: BTreeMap::new(),
                    states_by_publish: BTreeMap::from([(publish.to_string(), states)]),
                },
                recovery_words: None,
                judged_on: JudgingDevice::Cpu,
                self_verified_system_configuration_slots: slots_of_the_states
                    .handed_back_for_the_block(states, &[]),
            },
        )
        .expect("写得进库");
}

/// 复用哨兵那几条的库：那条两个扇区的小流，每一块存一份不红的判器结论（连同逐盘自证槽数）；交回计划、版本与块数。
pub(super) fn a_library_where_every_block_of_the_two_sector_flow_is_reusable(
    test_name: &str,
) -> (
    crate::crash_library::DisposableLibrary,
    VerdictStore,
    CrashPlan,
    Vec<ConclusionVersion>,
) {
    let (plan, _block_count) = plan_of_the_two_sector_flow_with_its_block_count();
    let (library, mut store) = an_empty_library_for(test_name);
    let versions =
        vec![
            ConclusionVersion(sha256_digest(b"version of the reuse sentinel unit tests\n"));
            plan.points.len()
        ];
    let conclusions = ConclusionsStoredUnder::TheConclusionKind {
        kind: ConclusionKind::ImageJudgement,
        version_of_each_point: &versions,
    };
    for (point_index, point) in plan.points.iter().enumerate() {
        for local in point.blocks() {
            store_a_judge_conclusion_of_the_block(
                &mut store,
                &plan,
                &conclusions,
                point_index,
                &local,
                None,
                "unit_test_publish",
            );
        }
    }
    (library, store, plan, versions)
}

/// 待判行那几条用的录下的流程摘要与判表摘要（单测不建判表、不起卡：替身服务照块号用 CPU 判器判，不照摘要读回）。
pub(super) const STAND_IN_RECORDED_FLOW_DIGEST: [u8; DIGEST_BYTES] = [0x2B; DIGEST_BYTES];
pub(super) const STAND_IN_PACKED_JUDGE_TABLES_DIGEST: [u8; DIGEST_BYTES] = [0x3C; DIGEST_BYTES];

/// 判表待判行那几条单测里替身服务写的设备：替身那一张卡。
#[cfg(feature = "gpu")]
pub(super) fn the_card_of_the_packed_tables_stand_in() -> JudgingDevice {
    JudgingDevice::Gpu {
        card_type: "替身服务的卡".to_string(),
        driver_version: "替身服务的驱动".to_string(),
    }
}

/// 替身服务最多走几圈现算：等的那一边写的判表待判行与替身读到的对不上时，替身不空等下去、当场 panic。
#[cfg(feature = "gpu")]
const RECKONINGS_OF_THE_PACKED_TABLES_STAND_IN_AT_MOST: u64 = 8;

/// 判表待判行的替身服务（[`GpuVerificationServerStandIn`]）：两次现算之间读库里全部判表待判行，逐行照 `verdict_words_of_the_row` 交的结论字
/// 编一帧（[`VerdictStore::packed_tables_verdict_words_rows_for_one_frame`]，设备是 [`the_card_of_the_packed_tables_stand_in`]），
/// 写进库、删那一行；读到的每一行记进 `rows_seen`。走满 [`RECKONINGS_OF_THE_PACKED_TABLES_STAND_IN_AT_MOST`] 圈就 panic。
#[cfg(feature = "gpu")]
pub(super) fn packed_tables_stand_in(
    verdict_words_of_the_row: impl Fn(
            &crate::verdict_store::PackedTablesKey,
            &crate::verdict_store::PackedTablesPendingRow,
        ) -> Vec<[u32; crate::crash_judge_gpu::VERDICT_WORDS]>
        + Send
        + Sync
        + 'static,
    rows_seen: std::sync::Arc<
        std::sync::Mutex<
            Vec<(
                crate::verdict_store::PackedTablesKey,
                crate::verdict_store::PackedTablesPendingRow,
            )>,
        >,
    >,
) -> std::sync::Arc<GpuVerificationServerStandIn> {
    let reckonings = std::sync::atomic::AtomicU64::new(0);
    std::sync::Arc::new(GpuVerificationServerStandIn {
        between_two_reckonings: Box::new(move |library: &LibraryTakenAtEachReadOrWrite<'_>| {
            let reckoning = reckonings.fetch_add(1, std::sync::atomic::Ordering::SeqCst) + 1;
            assert!(
                    reckoning <= RECKONINGS_OF_THE_PACKED_TABLES_STAND_IN_AT_MOST,
                    "替身服务走了 {reckoning} 圈现算还在等：等的那一边写的判表待判行与替身读到、写回的对不上"
                );
            let rows = library
                .take()
                .every_packed_tables_pending_row()
                .expect("替身扫得了判表待判行");
            for (key, row) in rows {
                let frame = library
                    .take()
                    .packed_tables_verdict_words_rows_for_one_frame(
                        &key,
                        &row,
                        &verdict_words_of_the_row(&key, &row),
                        &the_card_of_the_packed_tables_stand_in(),
                    )
                    .expect("替身交的结论字与待判行记的态数相同");
                library
                    .take()
                    .write_the_packed_tables_verdict_words_rows_of_one_frame(&frame)
                    .expect("替身写得进库");
                rows_seen
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner)
                    .push((key, row));
            }
        }),
    })
}

/// 在库里给计划第 `point_index` 个点的 `local` 那一块存一份观察者结论（`red_item` 给了就是头一个状态违例的红结论）。
pub(super) fn store_an_observer_conclusion_of_the_block(
    store: &mut VerdictStore,
    plan: &CrashPlan,
    observer: &ObserverForTheReuseSentinels,
    point_index: usize,
    local: &Range<u64>,
    red_item: Option<&str>,
) {
    let items: Vec<String> = red_item.map(str::to_string).into_iter().collect();
    let number = store
        .register_verdict_vector(&encoded_verdict_of_the_red_items(&items))
        .expect("登记得了");
    let states = local.end - local.start;
    let violating_states = if red_item.is_some() {
        vec![StateOffsetInBlock(0)]
    } else {
        Vec::new()
    };
    store
        .store_conclusion_block(
            &plan.points[point_index].block_key(local),
            &ConclusionKind::Observer,
            &observer.conclusion,
            &ConclusionBlockRecord {
                block: VerdictBlock::new(vec![
                    number;
                    usize::try_from(states).expect("一块装得进 usize")
                ])
                .expect("一块至少一个状态"),
                violating_states,
                count_vector: BlockCountVector {
                    per_invariant: BTreeMap::new(),
                    per_observer: BTreeMap::from([(
                        "unit_test_observer_counter".to_string(),
                        states,
                    )]),
                    states_by_publish: BTreeMap::from([("unit_test_publish".to_string(), states)]),
                },
                judged_on: JudgingDevice::Cpu,
                recovery_words: None,
            },
        )
        .expect("写得进库");
}

/// 在库里给计划第 `point_index` 个点的 `local` 那一块存观察者判段重拼要读的那一行（[`ObserverForTheReuseSentinels::input_row`]）。
pub(super) fn store_the_observer_input_row_of_the_block(
    store: &mut VerdictStore,
    plan: &CrashPlan,
    observer: &ObserverForTheReuseSentinels,
    point_index: usize,
    local: &Range<u64>,
) {
    let (input_kind, input_version) = &observer.input_row;
    store
        .store_observer_block(
            &plan.points[point_index].block_key(local),
            input_kind,
            input_version,
            b"observer block recorded for the reuse sentinel unit tests",
        )
        .expect("写得进库");
}

/// 一块判器结论不带判器交回的逐盘自证槽数写进 `store`（取样那一路存的那种）：每个状态同一个不红的判定向量。
pub(super) fn store_a_judge_conclusion_without_the_slots(
    store: &mut VerdictStore,
    key: &VerdictBlockKey,
    version: &ConclusionVersion,
    local: &Range<u64>,
) {
    let clean = store
        .register_verdict_vector(&encoded_verdict_of_the_red_items(&[]))
        .expect("登记得了");
    let states = local.end - local.start;
    store
        .store_conclusion_block(
            key,
            &ConclusionKind::ImageJudgement,
            version,
            &ConclusionBlockRecord {
                block: VerdictBlock::new(vec![
                    clean;
                    usize::try_from(states).expect("一块装得进 usize")
                ])
                .expect("一块至少一个状态"),
                violating_states: Vec::new(),
                count_vector: BlockCountVector {
                    per_invariant: BTreeMap::new(),
                    per_observer: BTreeMap::new(),
                    states_by_publish: BTreeMap::from([("unit_test_publish".to_string(), states)]),
                },
                judged_on: JudgingDevice::Cpu,
                recovery_words: None,
            },
        )
        .expect("写得进库");
}

/// 那条两个扇区的小流的计划与它的块数（块键只由内容定，库里录不录这条流都一样）。
pub(super) fn plan_of_the_two_sector_flow_with_its_block_count() -> (CrashPlan, usize) {
    let plan = a_flow_of_two_writes_to_two_sectors().with_the_flow(plan_crash_points);
    let block_count = plan.points.iter().map(|point| point.blocks().len()).sum();
    (plan, block_count)
}

/// 看各段的线程到了几条、每个状态交来的镜像叠在哪一份枚举写表上（写表首元素的地址）。
#[derive(Default)]
pub(super) struct ThreadsAndEnumeratedWritesSeenByTheObserver {
    pub(super) threads: std::collections::HashSet<std::thread::ThreadId>,
    pub(super) enumerated_writes_address_of_each_state: Vec<usize>,
}

/// 判段、录入段里判与看的那一下睡多久（单测造的慢判器、慢观察者），与那时别的线程取同一条连接的锁最多许等多久。
pub(super) const TIME_THE_SLOW_JUDGE_OR_OBSERVER_SLEEPS: std::time::Duration =
    std::time::Duration::from_millis(200);
pub(super) const LONGEST_WAIT_FOR_THE_LIBRARY_LOCK_MEANWHILE: std::time::Duration =
    std::time::Duration::from_millis(50);

/// 另起一条线程跑 `judge_or_record`（判段或录入段），等 `has_started_sleeping` 设上（慢判器、慢观察者睡着了）再取一次 `library` 的锁，
/// 交回等了多久；那条线程先收工（没走到睡那一步）也照样取，交回的等锁时长照实。判段或录入段的结局原样交回。
pub(super) fn wait_for_the_library_lock_while_the_judge_or_the_observer_sleeps<
    StageOutcomeOfTheTest: Send,
>(
    library: &LibraryOpenInThisProcess,
    has_started_sleeping: &std::sync::atomic::AtomicBool,
    judge_or_record: impl FnOnce() -> StageOutcomeOfTheTest + Send,
) -> (std::time::Duration, StageOutcomeOfTheTest) {
    std::thread::scope(|scope| {
        let judging_or_recording_thread = scope.spawn(judge_or_record);
        // 迭代上界：慢判器或慢观察者睡着就设，那条线程先收工（出错、panic）也停
        while !has_started_sleeping.load(std::sync::atomic::Ordering::SeqCst)
            && !judging_or_recording_thread.is_finished()
        {
            std::thread::yield_now();
        }
        let asked_at = std::time::Instant::now();
        drop(library.lock());
        let waited = asked_at.elapsed();
        let outcome_of_the_stage = judging_or_recording_thread
            .join()
            .unwrap_or_else(|payload| std::panic::resume_unwind(payload));
        (waited, outcome_of_the_stage)
    })
}

/// 观察者两段那几条单测一趟的结果：录入段、判段的账，观察者真跑了几次，逐状态的红项、每块的计数向量与在哪判的、每块录入块里的观察结果几份。
pub(super) struct ObserverUnitTestRun {
    pub(super) recording: Option<ObserverRecordingTally>,
    pub(super) observing: Result<ObservingOfTheRun, String>,
    pub(super) observer_calls: u64,
    pub(super) items_of_each_state: Vec<Vec<String>>,
    pub(super) count_vectors: Vec<BlockCountVector>,
    pub(super) judged_on_of_each_block: Vec<JudgingDevice>,
    pub(super) observations_of_each_block: Vec<usize>,
}

/// 在 `store` 上照 `stage` 跑那条小流的一个交回镜像的观察者（每看一张镜像加一次调用数，交回崩溃镜像本身、一个写都不发；判段一条不变量都不判，
/// 判干净时 `judged_clean` 加一）：录入段只录、判段只读库（判段 CPU 那一路）。判段报错照原样交回。
pub(super) fn run_the_unit_test_observer(
    store: &mut VerdictStore,
    extra_inputs_digest: [u8; DIGEST_BYTES],
    deduplication: ObserverDeduplication,
    stage: RunStage,
) -> ObserverUnitTestRun {
    run_the_unit_test_observer_on(
        store,
        a_flow_of_two_writes_of_the_same_bytes_to_one_sector(),
        DeclaredReads::WholePool,
        extra_inputs_digest,
        deduplication,
        stage,
    )
}

/// 单测的观察者交回它读过的段：整个池（只与镜像逐字节相同的状态合并），或只有一个扇区（只看那个扇区的字节）。
#[derive(Clone, Copy)]
pub(super) enum DeclaredReads {
    WholePool,
    OnlySector(u64),
}

pub(super) fn run_the_unit_test_observer_on(
    store: &mut VerdictStore,
    recorded: RecordedFlow,
    declared_reads: DeclaredReads,
    extra_inputs_digest: [u8; DIGEST_BYTES],
    deduplication: ObserverDeduplication,
    stage: RunStage,
) -> ObserverUnitTestRun {
    let calls = std::sync::atomic::AtomicU64::new(0);
    let observe = |_ordinal: u64,
                   image: &CrashImage<'_>,
                   _report: &RecoveryReport,
                   counts: &mut CrashPointReplayObserverCounts|
     -> ImageAfterTheObserver {
        calls.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        counts.add("images_seen", 1);
        ImageAfterTheObserver {
            writes_on_the_crash_image: Vec::new(),
            disks_after_the_observer: pool_of_the_crash_image(
                image.base,
                image.writes,
                &image.persisted,
            ),
            ranges_read_on_the_crash_image: match declared_reads {
                DeclaredReads::WholePool => ReadRangeOnTheCrashImage::whole_pool(image.base),
                DeclaredReads::OnlySector(sector) => image
                    .base
                    .devices
                    .keys()
                    .map(|identity| ReadRangeOnTheCrashImage {
                        device: identity.0,
                        first_sector: sector,
                        end_sector: sector + 1,
                    })
                    .collect(),
            },
            invariants_judged: InvariantsJudgedOnTheImageAfterTheObserver::Only(Vec::new()),
            counter_when_the_pool_checker_is_clean: "judged_clean".to_string(),
        }
    };
    let versions = [versions_of_the_unit_test_observer(extra_inputs_digest)];
    let observer_name = ObserverName::new("unit_test_image_observer").expect("名字合规矩");
    recorded.with_the_flow(|flow| {
        let plan = plan_crash_points(flow);
        let observers = [ObserverOnTheNodes {
            observer_name: observer_name.clone(),
            observer_code_file: OBSERVER_UNIT_TEST_SETTING_FILE.to_string(),
            extra_inputs_digest,
            nodes: vec![
                NodeOfAnObserver {
                    setting_file: OBSERVER_UNIT_TEST_SETTING_FILE.to_string(),
                    entry_name: OBSERVER_UNIT_TEST_POINT_NAME.to_string(),
                },
                NodeOfAnObserver {
                    setting_file: OBSERVER_UNIT_TEST_SETTING_FILE.to_string(),
                    entry_name: END_OF_FLOW_NODE_NAME.to_string(),
                },
            ],
            observe: ObserverCode::HandsBackAnImage {
                observe: &observe,
                deduplication,
            },
        }];
        let recording = stage.records().then(|| {
            record_the_observers_of_the_flow(
                &LibraryTakenAtEachReadOrWrite::held_by_this_caller_alone(store),
                flow,
                &plan,
                &observers,
                &versions,
                &|_| true,
                &|| false,
            )
            .expect("录入段写得进库")
        });
        let observer_calls = calls.load(std::sync::atomic::Ordering::SeqCst);
        let observing = if stage.judges() {
            judge_the_observers_of_the_flow(
                &LibraryTakenAtEachReadOrWrite::held_by_this_caller_alone(store),
                &plan,
                &observers,
                &versions,
                &PoolCheckerOfTheImagesAfterTheObservers::OnTheCpu,
                &|_| true,
            )
        } else {
            Ok(ObservingOfTheRun::default())
        };
        assert_eq!(
            calls.load(std::sync::atomic::Ordering::SeqCst),
            observer_calls,
            "判段不调观察者：只从库里读"
        );
        let kind = ConclusionKind::Observer;
        let mut run = ObserverUnitTestRun {
            recording,
            observing,
            observer_calls,
            items_of_each_state: Vec::new(),
            count_vectors: Vec::new(),
            judged_on_of_each_block: Vec::new(),
            observations_of_each_block: Vec::new(),
        };
        for point in &plan.points {
            for local in point.blocks() {
                let key = point.block_key(&local);
                if let Some(bytes) = store
                    .read_observer_block(&key, &kind, &versions[0].observer)
                    .expect("库读得了")
                {
                    run.observations_of_each_block.push(
                        ObserverBlockRecord::from_bytes(&bytes)
                            .expect("录入块解得开")
                            .observations
                            .len(),
                    );
                }
                let Some(stored) = store
                    .read_conclusion_block(&key, &kind, &versions[0].conclusion)
                    .expect("库读得了")
                else {
                    continue;
                };
                for number in stored.block.verdict_vector_number_of_each_state() {
                    run.items_of_each_state
                        .push(red_items_of_the_encoded_verdict(
                            store
                                .verdict_vector_of_number(*number)
                                .expect("块里的判定向量编号库里登过"),
                        ));
                }
                run.count_vectors.push(stored.count_vector);
                run.judged_on_of_each_block.push(stored.judged_on);
            }
        }
        run
    })
}

/// 那条一个扇区两次写同一份字节的小流挂一个只记计数的观察者（崩溃点与末节点），名字没在 `OBSERVER_ENTRIES` 登记：它那一组判法单元拒算。
pub(super) fn the_counting_observer_without_an_observer_entry<'observer>(
    observe: &'observer ObserveACrashState<'observer>,
) -> ObserverOnTheNodes<'observer> {
    ObserverOnTheNodes {
        observer_name: ObserverName::new("unit_test_observer_without_an_observer_entry")
            .expect("名字合规矩"),
        observer_code_file: OBSERVER_UNIT_TEST_SETTING_FILE.to_string(),
        extra_inputs_digest: [6; DIGEST_BYTES],
        nodes: vec![
            NodeOfAnObserver {
                setting_file: OBSERVER_UNIT_TEST_SETTING_FILE.to_string(),
                entry_name: OBSERVER_UNIT_TEST_POINT_NAME.to_string(),
            },
            NodeOfAnObserver {
                setting_file: OBSERVER_UNIT_TEST_SETTING_FILE.to_string(),
                entry_name: END_OF_FLOW_NODE_NAME.to_string(),
            },
        ],
        observe: ObserverCode::CountsOnly(observe),
    }
}

/// 那个没登记的观察者在给定的进程标记下的几把版本与照判不入库的那几把（同 [`observer_versions_keeping_the_refused_out`]，
/// 标记由单测给：两个标记就是录入与判段各在一个进程里的样子）。`pool_checker` 是这一趟判观察者交回的盘的那一路，
/// 与判段拿到的池级 checker 的 [`PoolCheckerOfTheImagesAfterTheObservers::judged_by`] 一致。
pub(super) fn versions_of_the_observer_without_an_observer_entry_under_the_process_marker(
    observer: &ObserverOnTheNodes<'_>,
    pool_checker: JudgedBy,
    process_marker_byte: u8,
) -> (ObserverVersions, Vec<[u8; DIGEST_BYTES]>) {
    let (versions, refusals, versions_kept_out) =
        observer_versions_keeping_the_refused_out_under_the_process_marker(
            observer,
            pool_checker,
            ObserverJudgingUnitsKeyTable::OfTheVerificationPackage,
            &[process_marker_byte; DIGEST_BYTES],
        );
    assert!(
            refusals.iter().any(|refusal| matches!(
                refusal,
                JudgingUnitsKeyRefused::NotComputedAtTheBuild { units, .. }
                    if *units == observer_code_units_of(observer)
            )),
            "前提：没登记的观察者那一组判法单元拒算（not_computed_at_the_build；崩溃枚举用例照它把块计入 refused=）：{:?}",
            refusals
                .iter()
                .map(JudgingUnitsKeyRefused::sentence)
                .collect::<Vec<String>>()
        );
    assert!(
        versions_kept_out.contains(&versions.observer.0)
            && versions_kept_out.contains(&versions.conclusion.0),
        "前提：观察者那一把与结论那一把照判不入库"
    );
    (versions, versions_kept_out)
}

/// 观察者交回的盘建判表那几条单测的基底：两块 4 GiB 的稀疏内存盘上做完 mkfs（判表从基底第一块盘的系统配置槽取落点几何）。
pub(super) fn a_formatted_pool_for_the_judge_tables_of_the_observer_images() -> MemoryPool {
    const DEVICE_BYTES: u64 = 4 << 30;
    let parameters = crate::scenario::e142_parameters(512, 512);
    let mut devices: Vec<(
        singlefs_core::address::DeviceIdentity,
        crate::memory_pool::SparseBlockDevice,
    )> = [0u32, 1]
        .into_iter()
        .map(|device_number| {
            (
                singlefs_core::address::DeviceIdentity(device_number),
                crate::memory_pool::SparseBlockDevice::new(
                    DEVICE_BYTES,
                    singlefs_core::block_device::PhysicalBlockSizeInBytes(512),
                ),
            )
        })
        .collect();
    singlefs_core::make_filesystem::make_filesystem(&parameters, &mut devices)
        .expect("两块空的内存盘上做得了 mkfs");
    MemoryPool {
        devices: devices
            .into_iter()
            .map(|(identity, device)| (identity, device.image))
            .collect(),
        device_size_in_bytes: DEVICE_BYTES,
    }
}

/// 写在一块盘系统配置槽 0 号（偏移 0）上的一次 4096 字节的写，内容是 `byte` 重复。
pub(super) fn a_system_configuration_slot_write(device_number: u32, byte: u8) -> RetainedWrite {
    RetainedWrite {
        device: singlefs_core::address::DeviceIdentity(device_number),
        kind: StepKind::SystemConfigurationSlot,
        is_force_unit_access: true,
        offset: singlefs_core::address::DeviceOffsetInBytes(0),
        contents: WrittenContents::Bytes(vec![byte; 4096]),
    }
}
