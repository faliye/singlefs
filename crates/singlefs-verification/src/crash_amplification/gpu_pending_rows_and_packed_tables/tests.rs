use super::*;

/// 场景：判段取卡没成，调用方交的卡串没设、是空串、只有空白，或交了一张卡（[`way_after_the_cards_are_not_taken`]）。
/// 预期：没设与空白都当调用方没交卡，判表交 GPU 批处理服务判；交了卡串而取不成的照取卡报的那一句报错，不交服务判。
/// 判别力（`crates/mutations.tsv`「判段取卡没成 空白卡串当交了卡」）：空白卡串当交了，走报错那一支，这条红。
#[cfg(feature = "gpu")]
#[test]
fn a_judging_stage_whose_card_string_is_missing_or_blank_hands_its_tables_to_the_gpu_verification_server(
) {
    for card_string in [None, Some(""), Some("  ")] {
        assert_eq!(
            way_after_the_cards_are_not_taken(card_string, "取卡没成".to_string()),
            WayAfterTheCardsAreNotTaken::ThroughTheGpuVerificationServer,
            "卡串 {card_string:?}：当调用方没交卡，判表交 GPU 批处理服务判"
        );
    }
    assert_eq!(
        way_after_the_cards_are_not_taken(Some("192.0.2.1:0:32G"), "取卡没成".to_string()),
        WayAfterTheCardsAreNotTaken::CardStringHandedOverButNotTaken {
            reason: "取卡没成".to_string()
        },
        "交了卡串而取不成：照取卡报的那一句报错"
    );
}

/// 判表待判行那几条等候单测的键与行：判表摘要、旋钮摘要各一个写死的字节，两个状态、产品旋钮，强制现判由调用方给。
#[cfg(feature = "gpu")]
fn a_packed_tables_key_and_row(
    forced: crate::verdict_store::ForcedRejudging,
) -> (
    crate::verdict_store::PackedTablesKey,
    crate::verdict_store::PackedTablesPendingRow,
) {
    (
        crate::verdict_store::PackedTablesKey {
            packed_judge_tables_digest: [0x31; DIGEST_BYTES],
            container_capacity_knobs_digest: [0x32; DIGEST_BYTES],
        },
        crate::verdict_store::PackedTablesPendingRow {
            container_capacity_knobs: ContainerCapacityKnobs::PRODUCT,
            state_count: 2,
            forced,
            pool_entry_number: 7,
        },
    )
}

/// 两个状态的一份结论字：态序号 0 的那个状态每字是 `word`，态序号 1 的每字是 `word + 1`。
#[cfg(feature = "gpu")]
fn verdict_words_of_two_states(word: u32) -> Vec<[u32; crate::crash_judge_gpu::VERDICT_WORDS]> {
    vec![
        [word; crate::crash_judge_gpu::VERDICT_WORDS],
        [word + 1; crate::crash_judge_gpu::VERDICT_WORDS],
    ]
}

/// 替服务把一份结论字写进库（一帧：写表 `21`、删表 `20`），设备是替身那一张卡。
#[cfg(feature = "gpu")]
fn write_the_verdict_words_as_the_service_would(
    library: &LibraryTakenAtEachReadOrWrite<'_>,
    key: &crate::verdict_store::PackedTablesKey,
    row: &crate::verdict_store::PackedTablesPendingRow,
    verdict_words: Vec<[u32; crate::crash_judge_gpu::VERDICT_WORDS]>,
) {
    let frame = library
        .take()
        .packed_tables_verdict_words_rows_for_one_frame(
            key,
            row,
            &verdict_words,
            &the_card_of_the_packed_tables_stand_in(),
        )
        .expect("结论字与待判行记的态数相同");
    library
        .take()
        .write_the_packed_tables_verdict_words_rows_of_one_frame(&frame)
        .expect("写得进库");
}

/// 等候那一层的等法：两次现算之间做 `between_two_reckonings`，进度行至少隔五分钟。
#[cfg(feature = "gpu")]
fn waiting_every_reckoning<'wait>(
    between_two_reckonings: &'wait dyn Fn(&LibraryTakenAtEachReadOrWrite<'_>),
) -> WaitingForTheGpuVerificationServer<'wait> {
    WaitingForTheGpuVerificationServer {
        progress_interval: std::time::Duration::from_secs(300),
        between_two_reckonings,
    }
}

/// 场景：库里已有这张判表的判表结论字，判段不强制现判。
/// 预期：交回库里那一份，一行判表待判行都不写、一圈都不等。
/// 判别力（`crates/mutations.tsv`）：不强制的那一份不先读表 `21`（强制与不强制两支对调），这条红。
#[cfg(feature = "gpu")]
#[test]
fn packed_tables_verdict_words_already_in_the_library_are_used_without_a_pending_row() {
    let (_disposable, mut store) = an_empty_library_for("packed-tables-words-already-there");
    let library = LibraryTakenAtEachReadOrWrite::held_by_this_caller_alone(&mut store);
    let (key, row) = a_packed_tables_key_and_row(crate::verdict_store::ForcedRejudging::NotForced);
    write_the_verdict_words_as_the_service_would(
        &library,
        &key,
        &row,
        verdict_words_of_two_states(0x10),
    );
    let ended = wait_for_the_verdict_words_of_the_packed_tables(
        &library,
        &key,
        &row,
        &waiting_every_reckoning(&|_library| panic!("库里已有判表结论字：一圈都不等")),
        &|| false,
    )
    .expect("读得了库");
    assert_eq!(
        ended,
        PackedTablesThroughTheGpuVerificationServerEnded::Judged(
            crate::verdict_store::PackedTablesVerdictWordsRow {
                verdict_words: verdict_words_of_two_states(0x10),
                judged_on: the_card_of_the_packed_tables_stand_in(),
            }
        ),
        "交回库里那一份"
    );
    let pending_row = library
        .take()
        .read_packed_tables_pending_row(&key)
        .expect("读得了库");
    assert_eq!(pending_row, None, "不写判表待判行");
}

/// 场景：库里已有这张判表的一份旧结论字，判段这一份强制现判；替身服务第一圈什么都不做（只核待判行在、是强制的），第二圈写一份新的结论字、删待判行。
/// 预期：写下强制的判表待判行（库里的旧结论字不用），等到待判行不在、表 `21` 在才收，交回的是新那一份；等了两圈。
/// 判别力（`crates/mutations.tsv`）：强制的那一份见表 `21` 在就收（不等待判行不在），交回旧的那一份，这条红。
#[cfg(feature = "gpu")]
#[test]
fn a_forced_packed_tables_request_waits_for_the_verdict_words_written_after_its_pending_row() {
    let (_disposable, mut store) = an_empty_library_for("packed-tables-forced-rejudging");
    let library = LibraryTakenAtEachReadOrWrite::held_by_this_caller_alone(&mut store);
    let (key, row) = a_packed_tables_key_and_row(crate::verdict_store::ForcedRejudging::Forced);
    write_the_verdict_words_as_the_service_would(
        &library,
        &key,
        &row,
        verdict_words_of_two_states(0x10),
    );
    let reckonings = std::sync::atomic::AtomicUsize::new(0);
    let between_two_reckonings = |library_of_the_service: &LibraryTakenAtEachReadOrWrite<'_>| {
        let reckoning = reckonings.fetch_add(1, std::sync::atomic::Ordering::SeqCst) + 1;
        let pending_row = library_of_the_service
            .take()
            .read_packed_tables_pending_row(&key)
            .expect("读得了库");
        match reckoning {
            1 => assert_eq!(
                pending_row.map(|present| present.forced),
                Some(crate::verdict_store::ForcedRejudging::Forced),
                "第一圈：强制的判表待判行在"
            ),
            2 => write_the_verdict_words_as_the_service_would(
                library_of_the_service,
                &key,
                &pending_row.expect("第二圈：判表待判行还在"),
                verdict_words_of_two_states(0x20),
            ),
            _ => panic!("第 {reckoning} 圈还在等：服务第二圈就写了新的结论字"),
        }
    };
    let ended = wait_for_the_verdict_words_of_the_packed_tables(
        &library,
        &key,
        &row,
        &WaitingForTheGpuVerificationServer {
            progress_interval: std::time::Duration::from_secs(300),
            between_two_reckonings: &between_two_reckonings,
        },
        &|| false,
    )
    .expect("读写得了库");
    assert_eq!(
        ended,
        PackedTablesThroughTheGpuVerificationServerEnded::Judged(
            crate::verdict_store::PackedTablesVerdictWordsRow {
                verdict_words: verdict_words_of_two_states(0x20),
                judged_on: the_card_of_the_packed_tables_stand_in(),
            }
        ),
        "交回服务在待判行之后写的那一份"
    );
    assert_eq!(
        reckonings.load(std::sync::atomic::Ordering::SeqCst),
        2,
        "等了两圈"
    );
}

/// 场景：判表待判行写下之后，第一圈之间它不在了（被删了）而表 `21` 也不在；第二圈之间替身见到它又在了才写结论字（不在就叫停）。
/// 预期：照不在才写补写一次，等到结论字，交回替身写的那一份。
/// 判别力（`crates/mutations.tsv`）：待判行不在而结论字也不在时不补写，替身第二圈见不到行、叫停，这条红。
#[cfg(feature = "gpu")]
#[test]
fn a_packed_tables_pending_row_gone_without_its_verdict_words_is_put_again() {
    let (_disposable, mut store) = an_empty_library_for("packed-tables-pending-row-put-again");
    let library = LibraryTakenAtEachReadOrWrite::held_by_this_caller_alone(&mut store);
    let (key, row) = a_packed_tables_key_and_row(crate::verdict_store::ForcedRejudging::NotForced);
    let reckonings = std::sync::atomic::AtomicUsize::new(0);
    let stopped = std::sync::atomic::AtomicBool::new(false);
    let between_two_reckonings = |library_of_the_service: &LibraryTakenAtEachReadOrWrite<'_>| {
        let reckoning = reckonings.fetch_add(1, std::sync::atomic::Ordering::SeqCst) + 1;
        match reckoning {
            1 => library_of_the_service
                .take()
                .delete_the_packed_tables_pending_row(&key)
                .expect("删得了"),
            2 => {
                let pending_row = library_of_the_service
                    .take()
                    .read_packed_tables_pending_row(&key)
                    .expect("读得了库");
                match pending_row {
                    Some(present) => write_the_verdict_words_as_the_service_would(
                        library_of_the_service,
                        &key,
                        &present,
                        verdict_words_of_two_states(0x40),
                    ),
                    None => stopped.store(true, std::sync::atomic::Ordering::SeqCst),
                }
            }
            _ => panic!("第 {reckoning} 圈还在等"),
        }
    };
    let ended = wait_for_the_verdict_words_of_the_packed_tables(
        &library,
        &key,
        &row,
        &WaitingForTheGpuVerificationServer {
            progress_interval: std::time::Duration::from_secs(300),
            between_two_reckonings: &between_two_reckonings,
        },
        &|| stopped.load(std::sync::atomic::Ordering::SeqCst),
    )
    .expect("读写得了库");
    assert_eq!(
        ended,
        PackedTablesThroughTheGpuVerificationServerEnded::Judged(
            crate::verdict_store::PackedTablesVerdictWordsRow {
                verdict_words: verdict_words_of_two_states(0x40),
                judged_on: the_card_of_the_packed_tables_stand_in(),
            }
        ),
        "补写之后等到结论字"
    );
}

/// 场景：态数多于一帧装得下的那一份交等候那一层。
/// 预期：一行判表待判行都没写就报错（第一版不支持）、一圈都不等。
/// 判别力（`crates/mutations.tsv`）：不核态数（照样写，等的那一圈叫停），这条红。
#[cfg(feature = "gpu")]
#[test]
fn a_packed_tables_request_with_too_many_states_writes_nothing() {
    let (_disposable, mut store) = an_empty_library_for("packed-tables-refused-requests");
    let library = LibraryTakenAtEachReadOrWrite::held_by_this_caller_alone(&mut store);
    let (key, row) = a_packed_tables_key_and_row(crate::verdict_store::ForcedRejudging::NotForced);
    let too_large_row = crate::verdict_store::PackedTablesPendingRow {
        state_count: crate::verdict_store::MOST_STATES_OF_A_PACKED_TABLE_IN_ONE_VERDICT_WORDS_FRAME
            + 1,
        ..row
    };
    let stopped = std::sync::atomic::AtomicBool::new(false);
    let stop_at_the_first_reckoning = |_library: &LibraryTakenAtEachReadOrWrite<'_>| {
        stopped.store(true, std::sync::atomic::Ordering::SeqCst);
    };
    let ended = wait_for_the_verdict_words_of_the_packed_tables(
        &library,
        &key,
        &too_large_row,
        &waiting_every_reckoning(&stop_at_the_first_reckoning),
        &|| stopped.load(std::sync::atomic::Ordering::SeqCst),
    );
    assert!(
        ended
            .as_ref()
            .is_err_and(|refusal| refusal.contains("第一版不支持")),
        "态数太多：写之前报错：{ended:?}"
    );
    let pending_row = library
        .take()
        .read_packed_tables_pending_row(&key)
        .expect("读得了库");
    assert_eq!(pending_row, None, "不写判表待判行");
    assert!(
        !stopped.load(std::sync::atomic::Ordering::SeqCst),
        "一圈都不等"
    );
}

/// 场景：判表结论字在不在库里（服务认领前后问的那一句），四种：不强制而在、不强制而不在、强制而在、不强制而在的那一行态数与待判行记的不同。
/// 预期：只有不强制而在、态数相同的那一种算判完。
/// 判别力（`crates/mutations.tsv`）：强制的那一份照不强制判（表 `21` 在就算判完），这条红。
#[cfg(feature = "gpu")]
#[test]
fn packed_tables_verdict_words_count_as_judged_only_for_a_request_not_forced() {
    let (_disposable, mut store) = an_empty_library_for("packed-tables-judged-in-the-library");
    let library = LibraryTakenAtEachReadOrWrite::held_by_this_caller_alone(&mut store);
    let (key, row) = a_packed_tables_key_and_row(crate::verdict_store::ForcedRejudging::NotForced);
    let forced_row = crate::verdict_store::PackedTablesPendingRow {
        forced: crate::verdict_store::ForcedRejudging::Forced,
        ..row
    };
    let three_state_row = crate::verdict_store::PackedTablesPendingRow {
        state_count: 3,
        ..row
    };
    let judged = |request_row: &crate::verdict_store::PackedTablesPendingRow| {
        packed_tables_verdict_words_are_in_the_library(&library.take(), &key, request_row)
            .expect("读得了库")
    };
    let before = judged(&row);
    write_the_verdict_words_as_the_service_would(
        &library,
        &key,
        &row,
        verdict_words_of_two_states(0x10),
    );
    assert_eq!(
        (
            before,
            judged(&row),
            judged(&forced_row),
            judged(&three_state_row)
        ),
        (false, true, false, false),
        "不强制而不在、不强制而在、强制而在、态数不同"
    );
}

/// 场景：判器编的内核源码（产品那一份与照产品旋钮取的那一份）、两组旋钮（产品的与墓碑树压了旋钮的）取旋钮摘要，判表待判行照行里的旋钮栏取内核源码。
/// 预期：`of_the_judge()` 与照产品旋钮取的那一份是同一份三份源码（观察者交回的盘那一种写产品旋钮，服务取出的就是它在本进程编的那一份）；
/// 两组旋钮的摘要不同；行里旋钮的摘要与键里那一段相同才交回照它取的内核源码，不同就拒。
/// 判别力（`crates/mutations.tsv`）：取内核源码之前不核旋钮摘要与键里那一段，这条红。
#[cfg(feature = "gpu")]
#[test]
fn the_kernel_sources_of_a_pending_row_are_those_of_its_container_capacity_knobs() {
    use crate::crash_judge_gpu::JudgeKernelSources;
    let of_the_judge = JudgeKernelSources::of_the_judge();
    let of_the_product_knobs = JudgeKernelSources::of_the_judge_given_the_container_capacity_knobs(
        ContainerCapacityKnobs::PRODUCT,
    );
    assert_eq!(
        of_the_judge, of_the_product_knobs,
        "of_the_judge() 就是照产品旋钮取的那三份"
    );
    let with_a_knob = ContainerCapacityKnobs {
        tombstone: Some(
            singlefs_checker::walk::TombstoneContainerCapacityKnob::new(2).expect("1..=815"),
        ),
        lineage: None,
        retirement: None,
    };
    assert_ne!(
        container_capacity_knobs_digest(with_a_knob),
        container_capacity_knobs_digest(ContainerCapacityKnobs::PRODUCT),
        "压了旋钮的那一组摘要不同"
    );
    let (key, row) = a_packed_tables_key_and_row(crate::verdict_store::ForcedRejudging::NotForced);
    let matching_key = crate::verdict_store::PackedTablesKey {
        container_capacity_knobs_digest: container_capacity_knobs_digest(
            ContainerCapacityKnobs::PRODUCT,
        ),
        ..key
    };
    assert_eq!(
        kernel_sources_of_a_packed_tables_pending_row(&matching_key, &row),
        Ok(of_the_product_knobs),
        "键里那一段对得上：交回照行里的旋钮取的那三份"
    );
    assert!(
        kernel_sources_of_a_packed_tables_pending_row(&key, &row)
            .is_err_and(|refusal| refusal.contains("对不上")),
        "键里那一段对不上：拒这一行"
    );
}

/// 场景：库里存一份两个状态的打包判表，服务照判表待判行读回它：行里记的态数是 2 的一份、是 3 的一份。
/// 预期：态数相同的读回、交回那一份判表；态数不同的拒（点名两个数）。
/// 判别力（`crates/mutations.tsv`）：读回之后不核态数，这条红。
#[cfg(feature = "gpu")]
#[test]
fn packed_judge_tables_of_a_pending_row_are_read_back_only_with_its_state_count() {
    let (_disposable, mut store) = an_empty_library_for("packed-tables-read-for-a-pending-row");
    let mut bytes = b"SFJTAB01".to_vec();
    bytes.extend_from_slice(&2u64.to_le_bytes());
    for word in [7u32, 8, 9] {
        bytes.extend_from_slice(&word.to_le_bytes());
    }
    let packed = crate::crash_judge_dispatch::PackedJudgeTables::from_bytes(&bytes)
        .expect("头、态数、三个字解得出");
    assert_eq!(
        packed.to_bytes(),
        bytes,
        "前提：写死的字节就是这份判表的字节"
    );
    let digest = sha256_digest(&bytes);
    store
        .store_packed_judge_tables(&digest, &bytes)
        .expect("写得进库");
    let (key, row) = a_packed_tables_key_and_row(crate::verdict_store::ForcedRejudging::NotForced);
    let key_of_the_tables = crate::verdict_store::PackedTablesKey {
        packed_judge_tables_digest: digest,
        ..key
    };
    let read_back = read_packed_judge_tables_for_a_pending_row(&store, &key_of_the_tables, &row)
        .expect("态数相同：读回");
    assert_eq!(*read_back, packed, "读回的就是存进去的那一份");
    let three_states = crate::verdict_store::PackedTablesPendingRow {
        state_count: 3,
        ..row
    };
    assert!(
        read_packed_judge_tables_for_a_pending_row(&store, &key_of_the_tables, &three_states)
            .is_err_and(|refusal| refusal.contains("读回有 2 个状态")),
        "态数不同：拒这一行"
    );
}

/// 等一张判表的进度行：镜像判定那一族那一行的写法（还剩几块、几个状态、写了几块、等了几秒），头之后带判表这一族的族名。
/// 判别力（`crates/mutations.tsv`）：进度行不带族名，这条红。
#[cfg(feature = "gpu")]
#[test]
fn the_packed_tables_waiting_line_names_its_family() {
    assert_eq!(
            packed_tables_waiting_line(1, 65_536, 1, 3.4),
            "CRASH_AMPLIFICATION_WAITING_FOR_THE_GPU_VERIFICATION_SERVER family=packed-tables blocks_remaining=1 states_remaining=65536 blocks_written=1 seconds_waited=3",
            "等一张判表的进度行逐字是这一句"
        );
}

/// GPU 批处理服务那三样的单测用的打包判表：照这条流的状态数，表是空的（这几条小流的基线里没有系统配置槽、建不出真判表）；
/// 只钉服务照判表摘要读回的是库里那一份，不上卡、不判。
#[cfg(feature = "gpu")]
fn empty_packed_judge_tables_of_this_state_count(
    state_count: u64,
) -> crate::crash_judge_dispatch::PackedJudgeTables {
    crate::crash_judge_dispatch::PackedJudgeTables::of(&crate::crash_judge_tables::JudgeTables {
        recorded_write_count: 0,
        enumerated_write_count: 0,
        is_tearable: Vec::new(),
        torn_images: Vec::new(),
        segments: Vec::new(),
        segment_expansions: Vec::new(),
        state_source: crate::crash_judge_tables::JudgeStateSource::SegmentExpansions,
        state_count,
        devices: Vec::new(),
        device_bytes: 0,
        unit_area_start_in_bytes: 0,
        judged_root_index: 0,
        writes: Vec::new(),
        locations: Vec::new(),
        publishes: Vec::new(),
        expected_versions: Vec::new(),
    })
}

/// 服务那三样的单测共用的那一份：两个扇区的小流录进一份空库（`record_the_flow_and_read_it_back`），空判表照摘要写进去；
/// 交回库、读回的流、计划、录下的流程的摘要、判表摘要与判表的字节。
#[cfg(feature = "gpu")]
#[allow(
    clippy::type_complexity,
    reason = "一份库连同读回的流、计划与两样摘要、判表字节交回，各是各的东西"
)]
fn the_two_sector_flow_recorded_for_the_gpu_verification_server(
    test_name: &str,
) -> (
    crate::crash_library::DisposableLibrary,
    VerdictStore,
    RecordedFlowReadBack,
    CrashPlan,
    [u8; DIGEST_BYTES],
    [u8; DIGEST_BYTES],
    Vec<u8>,
) {
    let (directory, mut store) = an_empty_library_for(test_name);
    let recorded = a_flow_of_two_writes_to_two_sectors();
    // 录进库的那一份按 `RecordedFlow::of` 抄、按它的摘要存（`record_the_flow_and_read_it_back`）
    let recorded_flow_digest = recorded.with_the_flow(|flow| RecordedFlow::of(flow).digest());
    let read_back = recorded.with_the_flow(|flow| {
        record_the_flow_and_read_it_back(&mut store, flow).expect("流写得进库、读得回")
    });
    let plan = plan_crash_points(&read_back.flow());
    let tables_bytes = empty_packed_judge_tables_of_this_state_count(
        plan.points
            .iter()
            .map(|point| point.ordinals.end)
            .max()
            .unwrap_or(0),
    )
    .to_bytes();
    let packed_judge_tables_digest = sha256_digest(&tables_bytes);
    store
        .store_packed_judge_tables(&packed_judge_tables_digest, &tables_bytes)
        .expect("判表写得进库");
    (
        directory,
        store,
        read_back,
        plan,
        recorded_flow_digest,
        packed_judge_tables_digest,
        tables_bytes,
    )
}

/// 计划里态序号不从 0 起的那个点的头一块（点、块在点里的区间），写成待判行的样子（不强制、入池序号写死）。
fn a_pending_row_of_the_first_block_of_a_point_past_ordinal_zero(
    plan: &CrashPlan,
    conclusion_version: ConclusionVersion,
    recorded_flow_digest: [u8; DIGEST_BYTES],
    packed_judge_tables_digest: [u8; DIGEST_BYTES],
) -> (usize, Range<u64>, crate::verdict_store::GpuPendingRow) {
    let point_index = plan
        .points
        .iter()
        .position(|point| point.ordinals.start > 0 && !point.blocks().is_empty())
        .expect("前提：至少一个点的态序号不从 0 起、有块（态序号区间要加点的起点才分得出）");
    let point = &plan.points[point_index];
    let local = point.blocks()[0].clone();
    let ordinals_of_the_block_in_the_flow =
        point.ordinals.start + local.start..point.ordinals.start + local.end;
    let row = crate::verdict_store::GpuPendingRow {
        recorded_block: point.block_key(&local),
        recorded_flow_digest,
        packed_judge_tables_digest,
        state_ordinals: ordinals_of_the_block_in_the_flow,
        conclusion_version,
        forced: crate::verdict_store::ForcedRejudging::NotForced,
        pool_entry_number: 43,
    };
    (point_index, local, row)
}

/// 场景：两个扇区的小流录进一份空库、空判表照摘要写进去；服务照一行待判行读回材料。
/// 预期：判表是库里照这一行判表摘要存的那一份（字节相同）、内核源码是判段上卡那一处编的那一份（产品旋钮）、计划与判段读回流之后排的
/// 逐点相同；行里的流程摘要指到库里没有的流程时读不出、点名「不在库里」。
/// 判别力（`crates/mutations.tsv`）：照流程摘要去读判表（读到的不是这一行的判表），这条红。
#[cfg(feature = "gpu")]
#[test]
fn the_material_of_a_pending_stream_is_what_the_judge_stage_reads_back_for_that_flow() {
    let (
        _directory,
        store,
        read_back,
        plan,
        recorded_flow_digest,
        packed_judge_tables_digest,
        tables_bytes,
    ) = the_two_sector_flow_recorded_for_the_gpu_verification_server("pending-row-material");
    let (_point_index, _local, row) = a_pending_row_of_the_first_block_of_a_point_past_ordinal_zero(
        &plan,
        ConclusionVersion([0x5A; DIGEST_BYTES]),
        recorded_flow_digest,
        packed_judge_tables_digest,
    );
    let material = material_of_a_gpu_pending_stream(&store, &row)
        .unwrap_or_else(|refusal| panic!("读得回材料：{refusal}"));
    assert_eq!(
        material.packed_judge_tables.to_bytes(),
        tables_bytes,
        "判表是照这一行的判表摘要存的那一份"
    );
    assert_eq!(
        material.kernel_sources,
        crate::crash_judge_gpu::JudgeKernelSources::of_the_judge(),
        "内核源码是判段上卡那一处编的那一份"
    );
    let points_of = |plan_to_list: &CrashPlan| {
        plan_to_list
            .points
            .iter()
            .map(|point| {
                (
                    point.name.clone(),
                    point.ordinals.clone(),
                    point
                        .blocks()
                        .iter()
                        .map(|local| point.block_key(local))
                        .collect::<Vec<VerdictBlockKey>>(),
                )
            })
            .collect::<Vec<_>>()
    };
    assert_eq!(
        points_of(&material.plan),
        points_of(&plan),
        "计划与判段读回流之后排的逐点相同"
    );
    assert_eq!(
        material.devices_of_the_pool,
        singlefs_core::recovery::PoolReader::device_identities(read_back.flow().base)
            .into_iter()
            .map(|device| device.0)
            .collect::<Vec<u32>>(),
        "池里盘的次序照读回的流"
    );
    let missing_flow = crate::verdict_store::GpuPendingRow {
        recorded_flow_digest: [0xEE; DIGEST_BYTES],
        ..row
    };
    let refusal = material_of_a_gpu_pending_stream(&store, &missing_flow)
        .err()
        .expect("库里没有这份流程读不出");
    assert!(refusal.contains("不在库里"), "{refusal}");
}

/// 场景：两个扇区的小流，一行待判行指的那一块依次处在：没有结论、存了不红的结论（连同逐盘槽数）、换成红的结论。
/// 预期：每一步现算的答案与判段那两个口径（`ConclusionsStoredUnder::is_judged_with_the_slots`、`is_stored_for_reuse`，计划里那个点的那一把版本）
/// 逐项相同，依次是没判完、判完可复用、判完而红。
/// 判别力（`crates/mutations.tsv`）：不看红（判完的一律答可复用），这条红。
#[test]
fn the_judging_of_a_pending_block_in_the_library_answers_the_two_measures_of_the_judge_stage() {
    let (plan, _block_count) = plan_of_the_two_sector_flow_with_its_block_count();
    let (_directory, mut store) = an_empty_library_for("pending-row-judged-in-the-library");
    let versions =
        vec![
            ConclusionVersion(sha256_digest(b"version of the pending row judging test\n"));
            plan.points.len()
        ];
    let conclusions = ConclusionsStoredUnder::TheConclusionKind {
        kind: ConclusionKind::ImageJudgement,
        version_of_each_point: &versions,
    };
    let (point_index, local, row) = a_pending_row_of_the_first_block_of_a_point_past_ordinal_zero(
        &plan,
        versions[0],
        STAND_IN_RECORDED_FLOW_DIGEST,
        STAND_IN_PACKED_JUDGE_TABLES_DIGEST,
    );
    let key = plan.points[point_index].block_key(&local);
    let mut answers = Vec::new();
    // 每一步问之前存不存、存的红不红：头一步不存，第二步存不红的，第三步换成红的
    for conclusion_stored_before_asking in [None, Some(None), Some(Some("unit_test_red_item"))] {
        if let Some(red_item) = conclusion_stored_before_asking {
            store_a_judge_conclusion_of_the_block(
                &mut store,
                &plan,
                &conclusions,
                point_index,
                &local,
                red_item,
                "unit_test_publish",
            );
        }
        let answer = judging_of_a_gpu_pending_block_in_the_library(&store, &row).expect("读得了库");
        assert_eq!(
            (
                answer.is_judged_with_the_slots(),
                answer.is_stored_for_reuse()
            ),
            (
                conclusions
                    .is_judged_with_the_slots(&store, point_index, &key)
                    .expect("读得了库"),
                conclusions
                    .is_stored_for_reuse(&store, point_index, &key)
                    .expect("读得了库")
            ),
            "第 {} 步：与判段那两个口径逐项相同",
            answers.len()
        );
        answers.push(answer);
    }
    assert_eq!(
        answers,
        vec![
            GpuPendingBlockInTheLibrary::NotJudged,
            GpuPendingBlockInTheLibrary::JudgedAndReusable,
            GpuPendingBlockInTheLibrary::JudgedRed,
        ],
        "依次是没判完、判完可复用、判完而红"
    );
}

/// 场景：同一块同一份假的结论字（头一个状态绿、第二个一项红、其余由种子定），两份库各录两个扇区的小流：一份照判段在卡上判完存块那一处
/// （解结论字、记按发布分的态数、`store_judged_block`）存进去；另一份照服务那三样读回材料、编一帧，把那一帧的行写进库。
/// 预期：两份库这一块的结论（判定向量本身、违例、计数向量、恢复字）、块属性里在哪判的、逐盘槽数那一行逐项相同；帧里守卫那一行排最后、
/// 编之前读到的守卫是不在；结论摘要与判段那一份库里算的相同。写进去之后同一份结论再编一帧：结论已存着、逐项相同，帧里没有结论行，
/// 读到的守卫是库里那一行。
/// 判别力（`crates/mutations.tsv`）：编帧时不记按发布分的态数（计数向量少了那一截），这条红。
#[cfg(feature = "gpu")]
#[test]
fn a_conclusion_frame_of_a_pending_block_written_as_one_batch_is_the_conclusion_the_judge_stage_stores(
) {
    let (
        _service_directory,
        mut service_store,
        _service_read_back,
        plan,
        recorded_flow_digest,
        packed_judge_tables_digest,
        _tables_bytes,
    ) = the_two_sector_flow_recorded_for_the_gpu_verification_server("pending-row-frame-service");
    let versions = vec![
        ConclusionVersion(sha256_digest(b"version of the pending row frame test\n"));
        plan.points.len()
    ];
    let (point_index, local, row) = a_pending_row_of_the_first_block_of_a_point_past_ordinal_zero(
        &plan,
        versions[0],
        recorded_flow_digest,
        packed_judge_tables_digest,
    );
    let card = CardThatJudgedAPendingBlock {
        card_index: 0,
        card_name: "测试造的卡".to_string(),
        card_driver_version: "测试造的驱动".to_string(),
    };
    let words = verdict_words_of_a_made_up_block(
        &mut 0x5EED_F2A3_u64,
        usize::try_from(local.end - local.start).expect("装得进 usize"),
    );
    let (_judge_stage_directory, mut judge_stage_store) =
        an_empty_library_for("pending-row-frame-judge-stage");
    let judge_stage_read_back = a_flow_of_two_writes_to_two_sectors().with_the_flow(|flow| {
        record_the_flow_and_read_it_back(&mut judge_stage_store, flow).expect("流写得进库、读得回")
    });
    let judge_stage_flow = judge_stage_read_back.flow();
    let conclusions = ConclusionsStoredUnder::TheConclusionKind {
        kind: ConclusionKind::ImageJudgement,
        version_of_each_point: &versions,
    };
    let devices_of_the_pool: Vec<u32> =
        singlefs_core::recovery::PoolReader::device_identities(judge_stage_flow.base)
            .into_iter()
            .map(|device| device.0)
            .collect();
    let mut judged_by_the_judge_stage = decoded_gpu_block(
        GpuVerdictWordsOfABlock {
            pending_index: 0,
            words_of_each_state: words.clone(),
            card_index: card.card_index,
            card_name: card.card_name.clone(),
            card_driver_version: card.card_driver_version.clone(),
        },
        &devices_of_the_pool,
    );
    judged_by_the_judge_stage.state_counts.absorb(
        PublishesOfTheStates::of_the_plan(&judge_stage_flow, &plan)
            .state_counts_of(&row.state_ordinals),
    );
    store_judged_block(
        &mut judge_stage_store,
        point_index,
        &plan.points[point_index],
        &local,
        &conclusions,
        judged_by_the_judge_stage,
        &self_verified_system_configuration_slots_of_each_device(judge_stage_flow.base),
        &mut CheckingTally::default(),
    )
    .expect("判段存得进库");
    let material = material_of_a_gpu_pending_stream(&service_store, &row)
        .unwrap_or_else(|refusal| panic!("读得回材料：{refusal}"));
    let frame = conclusion_frame_of_a_gpu_pending_block(
        &mut service_store,
        &material,
        &row,
        words.clone(),
        &card,
    )
    .unwrap_or_else(|refusal| panic!("编得成一帧：{refusal}"));
    assert_eq!(
        frame
            .rows
            .rows_to_put
            .last()
            .map(|(key, _value)| key.clone()),
        Some(frame.rows.guard_key.clone()),
        "守卫那一行排最后"
    );
    assert_eq!(frame.rows.guard_row_as_read, None, "空库里守卫那一行不在");
    service_store
        .write_the_conclusion_rows_of_one_frame(&frame.rows)
        .expect("那一帧写得进库");
    let key = &row.recorded_block;
    let read = |store: &VerdictStore| {
        let stored = store
            .read_conclusion_block(key, &ConclusionKind::ImageJudgement, &versions[0])
            .expect("读得了库")
            .expect("有这一块的结论");
        (
            ConclusionSpelledOutInVerdictVectors::of_the_stored_block(store, &stored),
            stored.judged_on,
            store
                .read_observer_block(key, &ConclusionKind::ImageJudgement, &versions[0])
                .expect("读得了库"),
        )
    };
    let judge_stage_conclusion = read(&judge_stage_store);
    assert_eq!(
        read(&service_store),
        judge_stage_conclusion,
        "服务那一帧写下的与判段存的逐项相同：结论、在哪判的、逐盘槽数"
    );
    assert_eq!(
        frame.conclusion_digest,
        judge_stage_conclusion.0.conclusion_digest(),
        "结论摘要与判段那一份库里算的相同"
    );
    let again =
        conclusion_frame_of_a_gpu_pending_block(&mut service_store, &material, &row, words, &card)
            .unwrap_or_else(|refusal| panic!("编得成一帧：{refusal}"));
    assert_eq!(
        again.rows.storing,
        crate::verdict_store::VerdictBlockStoring::AlreadyStoredIdentically,
        "同一份结论已存着"
    );
    assert!(
        !again
            .rows
            .rows_to_put
            .iter()
            .any(|(row_key, _value)| *row_key == again.rows.guard_key),
        "结论已存着：帧里没有结论块那一行"
    );
    assert!(
        again.rows.guard_row_as_read.is_some(),
        "读到的守卫是库里那一行"
    );
    assert_eq!(
        again.conclusion_digest, frame.conclusion_digest,
        "同一份结论同一个摘要"
    );
}

/// 场景：服务那一份库是一次性库（临时服务里的一段名字空间：判段登记了它，GPU 批处理服务照登记连过来、用它的客户端原样写那一帧）；
/// 照两个扇区的小流编一块待判块的结论帧。
/// 预期：帧里每一把键（要写的行、守卫、要删的行）都以这一份的名字空间起头（库里的键以表标签起头，不会以名字空间起头），
/// 写帧的一方原样写下去就落在那一份临时库里，不落到键原样的那一份。
/// 判别力（`crates/mutations.tsv`）：守卫与要写的行的键不接名字空间，这条红。
#[cfg(feature = "gpu")]
#[test]
fn conclusion_frame_keys_of_a_disposable_library_carry_its_namespace() {
    let (
        _service_directory,
        mut service_store,
        _service_read_back,
        plan,
        recorded_flow_digest,
        packed_judge_tables_digest,
        _tables_bytes,
    ) = the_two_sector_flow_recorded_for_the_gpu_verification_server(
        "pending-row-frame-keys-namespaced",
    );
    let versions = vec![
        ConclusionVersion(sha256_digest(
            b"version of the pending row frame keys namespaced test\n"
        ));
        plan.points.len()
    ];
    let (_point_index, local, row) = a_pending_row_of_the_first_block_of_a_point_past_ordinal_zero(
        &plan,
        versions[0],
        recorded_flow_digest,
        packed_judge_tables_digest,
    );
    let card = CardThatJudgedAPendingBlock {
        card_index: 0,
        card_name: "测试造的卡".to_string(),
        card_driver_version: "测试造的驱动".to_string(),
    };
    let words = verdict_words_of_a_made_up_block(
        &mut 0x5EED_F2A4_u64,
        usize::try_from(local.end - local.start).expect("装得进 usize"),
    );
    let material = material_of_a_gpu_pending_stream(&service_store, &row)
        .unwrap_or_else(|refusal| panic!("读得回材料：{refusal}"));
    let frame =
        conclusion_frame_of_a_gpu_pending_block(&mut service_store, &material, &row, words, &card)
            .unwrap_or_else(|refusal| panic!("编得成一帧：{refusal}"));
    let namespace = service_store.key_space().key_prefix().to_vec();
    assert!(!namespace.is_empty(), "前提：一次性库的名字空间不空");
    let keys_of_the_frame: Vec<&Vec<u8>> = frame
        .rows
        .rows_to_put
        .iter()
        .map(|(row_key, _value)| row_key)
        .chain(frame.rows.keys_to_delete.iter())
        .chain(std::iter::once(&frame.rows.guard_key))
        .collect();
    assert!(
        keys_of_the_frame.len() >= 2,
        "前提：空库里编的帧至少有要写的结论块那一行与守卫：{}",
        keys_of_the_frame.len()
    );
    for key in keys_of_the_frame {
        assert!(
            key.starts_with(&namespace),
            "帧里的键接上这一份的名字空间：{key:02x?}"
        );
    }
}
