use super::*;

/// 两段的录入段写进库的一条小流程：一块盘写过两个扇区，三次写（普通写、FUA 写、整段清零），两段（一段展开、一段不展开），
/// 一版带文件、一版没有文件，两个点（一个 ignore）。
fn a_small_recorded_flow() -> RecordedFlow {
    let mut base = MemoryPool::with_devices(&[singlefs_core::address::DeviceIdentity(3)], 1 << 20);
    let device = base
        .devices
        .get_mut(&singlefs_core::address::DeviceIdentity(3))
        .expect("刚建的盘");
    device.write(singlefs_core::address::DeviceOffsetInBytes(0), &[7u8; 512]);
    device.write(
        singlefs_core::address::DeviceOffsetInBytes(4096),
        &[9u8; 512],
    );
    RecordedFlow {
        base,
        writes: vec![
            RetainedWrite {
                device: singlefs_core::address::DeviceIdentity(3),
                kind: StepKind::UnitWrite,
                is_force_unit_access: false,
                offset: singlefs_core::address::DeviceOffsetInBytes(8192),
                contents: WrittenContents::Bytes(vec![1, 2, 3, 4]),
            },
            RetainedWrite {
                device: singlefs_core::address::DeviceIdentity(3),
                kind: StepKind::RootRecordFua,
                is_force_unit_access: true,
                offset: singlefs_core::address::DeviceOffsetInBytes(512),
                contents: WrittenContents::Bytes(vec![5; 512]),
            },
            RetainedWrite {
                device: singlefs_core::address::DeviceIdentity(3),
                kind: StepKind::ZeroFill,
                is_force_unit_access: false,
                offset: singlefs_core::address::DeviceOffsetInBytes(16384),
                contents: WrittenContents::Zeros { length: 4096 },
            },
        ],
        segments: vec![vec![0, 1], vec![2]],
        judged_root_index: 1,
        versions: vec![PublishedVersion {
            instance: InstanceGeneration(2),
            checkpoint_txg: CheckpointTxg(5),
            content: b"content".to_vec(),
        }],
        versions_without_a_file: vec![PublishedVersionWithoutAFile {
            instance: InstanceGeneration(2),
            checkpoint_txg: CheckpointTxg(6),
        }],
        segment_expansions: vec![
            CrashPointReplaySegmentExpansion::EveryProperSubset,
            CrashPointReplaySegmentExpansion::NotExpanded,
        ],
        crash_points: vec![
            CrashPointSpan {
                name: "写入".to_string(),
                code_file_in_repository: "crates/a.rs".to_string(),
                writes: 0..2,
                ignored: false,
            },
            CrashPointSpan {
                name: "清零".to_string(),
                code_file_in_repository: "crates/b.rs".to_string(),
                writes: 2..3,
                ignored: true,
            },
        ],
        // 第 0 段（两次写全展开，3 个态）只判段内编号 [1, 3)：片起点与片长随流一起往返（C790）
        segment_slices: vec![
            Some(CrashPointReplaySegmentSlice {
                start_within_the_domain: StateCount1280Bits::from_u64(1),
                state_count: 2,
            }),
            None,
        ],
    }
}

/// 录入段写进库的流程读回来与原样逐样相同：盘上写过的扇区、每次写的盘、种类、FUA、偏移与内容（含整段清零）、分段、被判根、
/// 两种版本、每段的展开方式、崩溃点（含 ignore）、每段的片（C790）；读回的再写一遍逐字节相同。
#[test]
fn a_recorded_flow_reads_back_from_its_bytes_exactly_as_it_was_recorded() {
    let recorded = a_small_recorded_flow();
    let bytes = recorded.to_bytes();
    let read_back = RecordedFlow::from_bytes(&bytes).expect("刚写的字节读得回");
    assert_eq!(read_back, recorded, "读回的与录下的逐样相同");
    assert_eq!(read_back.to_bytes(), bytes, "读回的再写一遍逐字节相同");
    assert_eq!(read_back.digest(), recorded.digest());
}

/// 库里那份字节坏了读不回：换了头、截短一个字节、多出一个字节、写的种类码认不得，都交回 `None`（判段照「读不回」停下，不猜）。
#[test]
fn a_damaged_recorded_flow_does_not_read_back() {
    let bytes = a_small_recorded_flow().to_bytes();
    let mut other_magic = bytes.clone();
    other_magic[0] ^= 1;
    assert_eq!(RecordedFlow::from_bytes(&other_magic), None, "换了头");
    assert_eq!(
        RecordedFlow::from_bytes(&bytes[..bytes.len() - 1]),
        None,
        "截短一个字节"
    );
    let mut longer = bytes.clone();
    longer.push(0);
    assert_eq!(RecordedFlow::from_bytes(&longer), None, "多出一个字节");
    let mut recorded = a_small_recorded_flow();
    recorded.writes.truncate(1);
    let one_write = recorded.to_bytes();
    // 第一次写的种类码在：头 8、盘大小 8、盘数 8、盘号 4、扇区数 8、两个扇区（各 8 + 8 + 512）、写数 8、盘号 4 之后
    let kind_at = 8 + 8 + 8 + 4 + 8 + 2 * (8 + 8 + 512) + 8 + 4;
    assert_eq!(one_write[kind_at], 1, "那一格是单元写的码");
    let mut unknown_kind = one_write.clone();
    unknown_kind[kind_at] = 0xEE;
    assert_eq!(
        RecordedFlow::from_bytes(&unknown_kind),
        None,
        "种类码认不得"
    );
}

/// 录入段登的清单读回来与原样相同：两个摘要、配了 GPU 时判表的摘要（与没配时的 `None`）、全域判定、登记了的段。
#[test]
fn a_recorded_run_manifest_reads_back_from_its_bytes() {
    for packed_judge_tables_digest in [None, Some([3u8; DIGEST_BYTES])] {
        let manifest = RecordedRunManifest {
            recorded_flow_digest: [1u8; DIGEST_BYTES],
            facts_digest: [2u8; DIGEST_BYTES],
            packed_judge_tables_digest,
            registered_segments: vec![RegisteredSegment {
                segment_index: 7,
                write_count: 26,
                write_table_digest_text: "0f".repeat(DIGEST_BYTES),
            }],
            exhaustive: packed_judge_tables_digest.is_some(),
        };
        let bytes = manifest.to_bytes();
        assert_eq!(
            RecordedRunManifest::from_bytes(&bytes),
            Some(manifest.clone())
        );
        assert_eq!(
            RecordedRunManifest::from_bytes(&bytes[..bytes.len() - 1]),
            None,
            "截短一个字节"
        );
    }
}

/// 判段只认库：录入段写进库之后把库里那份流程换成另一份（清单的摘要不改），判段读回时摘要对不上，整趟停下报「库里那份坏了」；
/// 清单不在（录入段没跑）也停下，不去用调用方内存里的流程。
#[test]
fn the_judge_stage_reads_only_the_library_and_stops_when_the_recorded_flow_there_was_changed() {
    // 一次性库留着：改坏库里的流程看判段停下，坏行不许进全仓那一份
    let directory = crate::crash_library::DisposableLibrary::new("two-stage-read-back")
        .unwrap_or_else(|refusal| panic!("{refusal}"));
    let mut store = directory
        .open_store(crate::verdict_store::PruneServerFailureHandling::ReturnTheError)
        .expect("开得了一次性库");
    let recording_key = RecordingKey([0x2a; DIGEST_BYTES]);
    let missing = read_back_the_recorded_run(
        &LibraryTakenAtEachReadOrWrite::held_by_this_caller_alone(&mut store),
        &recording_key,
        "没录过的流",
        false,
    )
    .err()
    .expect("没录过的流读不回");
    assert!(
        missing.contains("没录过的流") && missing.contains("先跑录入段"),
        "{missing}"
    );
    let recorded = a_small_recorded_flow();
    let bytes = recorded.to_bytes();
    let digest = sha256_digest(&bytes);
    store.store_recorded_flow(&digest, &bytes).expect("写得进");
    let facts = recorded.with_the_flow(|flow| {
        crate::crash_facts::extract_flow_facts(
            flow.base,
            flow.writes,
            flow.segments,
            flow.expansion,
        )
    });
    store
        .store_flow_facts(&facts.digest(), &facts.to_bytes())
        .expect("写得进");
    let manifest = RecordedRunManifest {
        recorded_flow_digest: digest,
        facts_digest: facts.digest(),
        packed_judge_tables_digest: None,
        registered_segments: Vec::new(),
        exhaustive: true,
    };
    store
        .store_recorded_run_manifest(&recording_key, &manifest.to_bytes())
        .expect("写得进");
    let read_back = read_back_the_recorded_run(
        &LibraryTakenAtEachReadOrWrite::held_by_this_caller_alone(&mut store),
        &recording_key,
        "一条流",
        false,
    )
    .expect("读得回");
    assert_eq!(
        read_back.flow.recorded(),
        &recorded,
        "读回的就是录下的那一份"
    );
    assert_eq!(*read_back.facts, facts);
    let mut changed = recorded.clone();
    changed.versions[0].content = b"another".to_vec();
    store
        .store_recorded_flow(&digest, &changed.to_bytes())
        .expect("写得进");
    let refused = read_back_the_recorded_run(
        &LibraryTakenAtEachReadOrWrite::held_by_this_caller_alone(&mut store),
        &recording_key,
        "一条流",
        false,
    )
    .err()
    .expect("库里那份换了，摘要对不上");
    assert!(refused.contains("摘要与清单对不上"), "{refused}");
    let refused_for_the_gpu = {
        store.store_recorded_flow(&digest, &bytes).expect("写得进");
        read_back_the_recorded_run(
            &LibraryTakenAtEachReadOrWrite::held_by_this_caller_alone(&mut store),
            &recording_key,
            "一条流",
            true,
        )
        .err()
        .expect("要 GPU 判而清单里没有判表")
    };
    assert!(
        refused_for_the_gpu.contains("没有 GPU 判表"),
        "{refused_for_the_gpu}"
    );
    drop(store);
    drop(directory);
}

/// 给定状态那一路写进库的一张表读回来与原样逐样相同：名字、基础镜像、写表、切段、两种接缝（掉电之后、崩溃之后带与不带落到的那一版）、
/// 每个状态的持久集合；读回的再写一遍逐字节相同。截短一个字节、持久集合里写了 0、1 之外的值都读不回。
#[test]
fn a_recorded_given_states_table_reads_back_from_its_bytes_and_a_damaged_one_does_not() {
    use crate::memory_pool::RecordStreamSeam;
    let flow = a_small_recorded_flow();
    let table = RecordedGivenStatesTable {
        table_name: "第一截/种子 3".to_string(),
        base: flow.base,
        writes: flow.writes,
        segments: flow.segments,
        seams: vec![
            RecordStreamSeam::AfterAPowerLoss {
                first_write_after_the_power_loss: 1,
            },
            RecordStreamSeam::AfterACrash {
                first_write_after_the_crash: 2,
                version_landed_on_after_the_crash: Some((InstanceGeneration(4), CheckpointTxg(9))),
            },
            RecordStreamSeam::AfterACrash {
                first_write_after_the_crash: 2,
                version_landed_on_after_the_crash: None,
            },
        ],
        persisted_sets: vec![vec![true, false, true], vec![false, false, false]],
    };
    let bytes = table.to_bytes();
    let read_back = RecordedGivenStatesTable::from_bytes(&bytes).expect("刚写的字节读得回");
    assert_eq!(read_back, table, "读回的与写进去的逐样相同");
    assert_eq!(read_back.to_bytes(), bytes, "读回的再写一遍逐字节相同");
    assert_eq!(
        RecordedGivenStatesTable::from_bytes(&bytes[..bytes.len() - 1]),
        None,
        "截短一个字节"
    );
    let mut not_a_bool = bytes.clone();
    *not_a_bool.last_mut().expect("至少一个字节") = 2;
    assert_eq!(
        RecordedGivenStatesTable::from_bytes(&not_a_bool),
        None,
        "持久集合里写了 2"
    );
}

/// 带两栏的那一份读回来与原样逐样相同（表连同三棵容器树各自的旋钮值，没装旋钮的那一棵读回也没装；恢复自称的那一版三种写法逐个相同）；
/// 旋钮值越出那一棵树的范围、恢复自称的那一版码认不得、个数与持久集合不等、头不对都读不回。
#[test]
fn a_recorded_given_states_table_with_its_columns_reads_back_with_its_knobs_and_an_out_of_range_knob_does_not(
) {
    use singlefs_checker::walk::{
        ContainerCapacityKnobs, LineageContainerCapacityKnob, TombstoneContainerCapacityKnob,
    };
    let flow = a_small_recorded_flow();
    let recorded = RecordedGivenStatesTableWithItsColumns {
        table: RecordedGivenStatesTable {
            table_name: "墓碑容器压到一片 2 条".to_string(),
            base: flow.base,
            writes: flow.writes,
            segments: flow.segments,
            seams: Vec::new(),
            persisted_sets: vec![
                vec![true, true, false],
                vec![true, false, false],
                vec![false, false, false],
            ],
        },
        container_capacity_knobs: ContainerCapacityKnobs {
            tombstone: Some(TombstoneContainerCapacityKnob::new(2).expect("1..=815")),
            lineage: Some(LineageContainerCapacityKnob::new(1).expect("1..=1359")),
            retirement: None,
        },
        claimed_versions: vec![
            ClaimedVersionOfAGivenState::NewestRootWrittenByTheMountAndItsPublish(Some((
                InstanceGeneration(3),
                CheckpointTxg((1 << 40) | 7),
            ))),
            ClaimedVersionOfAGivenState::NewestRootWrittenByTheMountAndItsPublish(None),
            ClaimedVersionOfAGivenState::LandedOnByTheConsultedRecovery,
        ],
    };
    let bytes = recorded.to_bytes();
    assert_eq!(
        RecordedGivenStatesTableWithItsColumns::from_bytes(&bytes),
        Some(recorded.clone()),
        "读回的与写进去的逐样相同（退役树没装旋钮读回也没装）"
    );
    let tombstone_knob_offset = RECORDED_GIVEN_STATES_TABLE_WITH_ITS_COLUMNS_MAGIC.len();
    let mut out_of_range = bytes.clone();
    out_of_range[tombstone_knob_offset..tombstone_knob_offset + 8]
        .copy_from_slice(&816u64.to_le_bytes());
    assert_eq!(
        RecordedGivenStatesTableWithItsColumns::from_bytes(&out_of_range),
        None,
        "墓碑容器旋钮值 816 越出 1..=815"
    );
    assert_eq!(
        RecordedGivenStatesTableWithItsColumns::from_bytes(&recorded.table.to_bytes()),
        None,
        "不带两栏的那一种字节头不对"
    );
    let first_claimed_version_offset = tombstone_knob_offset + 3 * 8 + 8;
    let mut unknown_claimed_version_code = bytes.clone();
    unknown_claimed_version_code[first_claimed_version_offset] = 3;
    assert_eq!(
        RecordedGivenStatesTableWithItsColumns::from_bytes(&unknown_claimed_version_code),
        None,
        "恢复自称的那一版码 3 认不得"
    );
    let mut one_claimed_version_short = recorded.clone();
    one_claimed_version_short.claimed_versions.pop();
    assert_eq!(
        RecordedGivenStatesTableWithItsColumns::from_bytes(&one_claimed_version_short.to_bytes()),
        None,
        "恢复自称的那一版比持久集合少一个"
    );
}

/// 场景名：录入键那两条用例的场景标识（崩溃枚举用例的项名、崩溃放量的流名都是这一类字串）。
const RECORDING_KEY_SCENARIO: &str = "recording-key-of-two-writes";

/// 那条两个扇区的小流在场景 `scenario` 下的录入键（[`recording_of_the_run`]：崩溃放量录入段取录入键的那一处）。
fn recording_key_of_the_flow(scenario: &str, recorded: &RecordedFlow) -> RecordingKey {
    recorded.with_the_flow(|flow| {
        recording_of_the_run(
            scenario,
            flow,
            &vec![None; flow.segments.len()],
            &[],
            false,
            false,
        )
        .recording_key
        .key
    })
}

/// 场景：同一份写流（那条两个扇区的小流）的录入键取三次：在这条测试的线程上取；在另起的一条线程上取（线程名不是任何一条测试的名字）；
/// 写流经库那一份字节读回、换一份由另一段代码造出来的同样的流再取。
/// 预期：三把相同，都等于钉住的组成——场景标识与写流摘要（[`recording_inputs_digest_of_the_flow_whatever_its_slices`]）拼起来取 SHA-256，
/// 别的一样都不进：产品代码改了而写流没变，录入键不变（录入键不经测试线程的名字认是哪条测试，不读编进二进制的判法单元键表）。
/// 判别力（`crates/mutations.tsv`「录入键另拼一样写流与场景之外的东西」）：录入键另拼上这条线程的名字，这条红在「另起的线程上」那一句。
#[test]
fn the_recording_key_stays_while_the_written_stream_and_the_scenario_stay() {
    let recorded = a_flow_of_two_writes_to_two_sectors();
    let on_this_test = recording_key_of_the_flow(RECORDING_KEY_SCENARIO, &recorded);
    let on_another_thread = std::thread::Builder::new()
        .name("a-thread-that-is-no-test".to_string())
        .spawn(|| {
            recording_key_of_the_flow(
                RECORDING_KEY_SCENARIO,
                &a_flow_of_two_writes_to_two_sectors(),
            )
        })
        .expect("起得了线程")
        .join()
        .expect("线程不 panic");
    let read_back = RecordedFlow::from_bytes(&recorded.to_bytes()).expect("写流的字节读得回");
    let built_elsewhere = recording_key_of_the_flow(RECORDING_KEY_SCENARIO, &read_back);
    let inputs = recorded.with_the_flow(|flow| {
        recording_inputs_digest_of_the_flow_whatever_its_slices(flow, &[], false, false)
    });
    let pinned = RecordingKey(sha256_digest(
        format!(
            "singlefs recording key\nscenario={RECORDING_KEY_SCENARIO}\ninputs={}\n",
            hexadecimal_text(&inputs.0)
        )
        .as_bytes(),
    ));
    assert_eq!(
        on_this_test, pinned,
        "录入键就是场景标识与写流摘要拼起来的那一把，别的一样都不进"
    );
    assert_eq!(
        on_another_thread, on_this_test,
        "另起的线程上（不是任何一条测试）取到同一把：录入键不认是哪条测试的代码"
    );
    assert_eq!(
        built_elsewhere, on_this_test,
        "同样的写流换一段代码造出来，录入键不变"
    );
}

/// 场景：那条两个扇区的小流改一样再取录入键：写到不读的那个扇区的那一次写改一个字节；两次写之间多一道屏障（一段切成两段）；同一份写流换一条场景。
/// 预期：三处改动各自都让录入键变——写流变了照录，场景不同各录各的。
/// 判别力（`crates/mutations.tsv`「录入键只拼场景标识」）：录入键不拼写流摘要，前两处改了键照旧，这条红在「写的字节改了」那一句。
#[test]
fn the_recording_key_changes_with_one_written_byte_a_moved_barrier_or_another_scenario() {
    let recorded = a_flow_of_two_writes_to_two_sectors();
    let base_key = recording_key_of_the_flow(RECORDING_KEY_SCENARIO, &recorded);
    let mut one_byte_changed = recorded.clone();
    let RetainedWrite {
        contents: WrittenContents::Bytes(bytes),
        ..
    } = &mut one_byte_changed.writes[1]
    else {
        panic!("小流里写到不读的那个扇区的那一次写是整扇区的字节");
    };
    bytes[0] ^= 1;
    let mut barrier_between_the_writes = recorded.clone();
    barrier_between_the_writes.segments = vec![vec![0], vec![1]];
    barrier_between_the_writes.segment_expansions =
        vec![CrashPointReplaySegmentExpansion::EveryProperSubset; 2];
    barrier_between_the_writes.segment_slices = vec![None; 2];
    for (changed_key, what) in [
        (
            recording_key_of_the_flow(RECORDING_KEY_SCENARIO, &one_byte_changed),
            "写的字节改了",
        ),
        (
            recording_key_of_the_flow(RECORDING_KEY_SCENARIO, &barrier_between_the_writes),
            "两次写之间多一道屏障",
        ),
        (
            recording_key_of_the_flow("another-scenario-of-two-writes", &recorded),
            "换一条场景",
        ),
    ] {
        assert_ne!(changed_key, base_key, "{what}：录入键变");
    }
}
