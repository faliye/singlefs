//! 判器所在的卡出问题时取活循环的样子：一张卡坏了的汇总行、在腾空的卡上也显存不够的块点名留给别的服务且不再认领。

use super::*;

/// 起取活循环（替身卡照 `faults` 出毛病），`client_side` 在测试这一侧做完（panic 了也一样）就叫它停；交回取活循环的账。
fn serve_with_faulty_stand_in_cards(
    pool: &mut LibraryPendingPool<StandInVerificationPackageSide<StandInMaterials>>,
    faults: &StandInCardFaults,
    client_side: impl FnOnce(),
) -> ServingSummary {
    let configuration = ServiceConfiguration {
        oldest_pending_wait_limit: Duration::ZERO,
        library_scan_interval: SCAN_INTERVAL,
        claim_lease: Duration::from_millis(u64::from(LONG_LEASE_MILLISECONDS)),
    };
    let break_switches = BreakSwitches::of_the_environment();
    let stop = Arc::new(AtomicBool::new(false));
    std::thread::scope(|scope| {
        let stop_for_the_loop = Arc::clone(&stop);
        let faults_of_the_cards = Arc::clone(faults);
        let configuration_of_the_loop = &configuration;
        let break_switches_of_the_loop = &break_switches;
        let serving = scope.spawn(move || {
            let mut open_the_cards = || -> Result<StandInCards, String> {
                Ok(StandInCards {
                    let_go: Arc::new(AtomicU64::new(0)),
                    dispatch_lines: Arc::new(Mutex::new(Vec::new())),
                    faults: Arc::clone(&faults_of_the_cards),
                })
            };
            serve_from_the_library(
                configuration_of_the_loop,
                break_switches_of_the_loop,
                pool,
                &mut open_the_cards,
                &|| stop_for_the_loop.load(Ordering::SeqCst),
            )
        });
        let stop_on_drop = StopOnDrop(Arc::clone(&stop));
        client_side();
        drop(stop_on_drop);
        serving.join().expect("取活循环没 panic")
    })
    .unwrap_or_else(|failure| panic!("取活循环没跑下去：{failure:?}"))
}

/// 照设定出的毛病还剩几回没出（这条流没设定的是 0）。
fn faults_left(faults: &StandInCardFaults, tag: u32) -> usize {
    faults
        .lock()
        .expect("没有线程拿着锁 panic")
        .get(&tag)
        .map_or(0, |(_, times)| *times)
}

/// 判的时候设备丢了（`src/serving_from_the_library.rs` 文件头「日志」）：取活循环打恰好一行卡坏（卡号、`nvidia-smi` 的序号、卡名、PCI 总线号、
/// 设备丢了、原因），停的时候循环的账（STOPPED 那一行打它）里有这张卡；没有接住 Out of Memory 那一行。
/// 弄坏开关：cards-failed-left-out-of-the-summary（会话收尾时丢掉坏了的卡）。
#[test]
fn a_card_failure_leaves_one_card_failed_line_and_the_summary_names_the_card() {
    let library = TemporaryLibraryService::start("card-failed-line");
    let conclusion_version = conclusion_version_of_the_crash_image_judgement();
    let mut pool = pool_on(
        &library,
        stand_in_side_with(&library, &[5]),
        LONG_LEASE_MILLISECONDS,
    );
    pool.keep_the_service_lines();
    let block = block_of(
        5,
        0,
        0..4,
        conclusion_version,
        ForcedRejudging::NotForced,
        1,
    );
    put_pending_rows(&library.open_store(), std::slice::from_ref(&block));
    let faults: StandInCardFaults = Arc::new(Mutex::new(BTreeMap::from([(
        5,
        (StandInCardFault::DeviceLost, 1),
    )])));
    let summary = serve_with_faulty_stand_in_cards(&mut pool, &faults, || {
        wait_until("判第 5 条流的时候设备照设定丢了", &mut || {
            faults_left(&faults, 5) == 0
        });
    });
    assert_eq!(
        summary.cards_failed,
        vec![CardIndex(0)],
        "循环的账：{summary:?}"
    );
    let lines = pool.kept_service_lines();
    let failed: Vec<&String> = lines
        .iter()
        .filter(|line| line.starts_with(&format!("{CARD_FAILED_LINE_PREFIX} ")))
        .collect();
    assert_eq!(failed.len(), 1, "恰好一行卡坏：{lines:?}");
    let card = stand_in_card();
    for field in [
        "card=0".to_string(),
        format!("nvidia_smi_index={}", card.card_index),
        format!("pci={STAND_IN_PCI_BUS_IDENTIFIER}"),
        "device_lost=yes".to_string(),
    ] {
        assert!(
            failed[0].split_whitespace().any(|word| word == field),
            "卡坏那一行带 {field}：{}",
            failed[0]
        );
    }
    assert!(
        failed[0].contains(&format!("name=\"{}\"", card.card_name))
            && failed[0].contains("替身照设定丢了设备"),
        "卡坏那一行带卡名与原因：{}",
        failed[0]
    );
    assert!(
        !is_a_dispatch_line(failed[0].as_bytes()),
        "那一行不被服务日志滤掉"
    );
    assert!(
        !lines
            .iter()
            .any(|line| line.starts_with(OUT_OF_MEMORY_CAUGHT_LINE_PREFIX)),
        "设备丢了不打接住 Out of Memory 那一行：{lines:?}"
    );
}

/// 一块在判的时候显存不够、单独在腾空的卡上又显存不够（`src/service_loop.rs` 文件头「接住 Out of Memory」，`src/serving_from_the_library.rs`
/// 文件头「判不了」）：取活循环打两行接住 Out of Memory、不打卡坏那一行，另打恰好一行判不了（族、流键头、块号、卡、为什么、认领放掉了），
/// 那一行不被服务日志滤掉；那一块的待判行留在库里，认领当场放掉（别的持有者随即认领得到），这个进程只认领它一回（不再取）；
/// 卡照常用：之后的另一块判完写回。
/// 一个场景：第 6 条流的一块显存不够两回，之后放进第 7 条流的一块；取活循环停了之后另一个持有者认领那一块。
/// 弄坏开关：block-refused-for-memory-claimed-again（这个进程下一轮照样认领）、block-not-judgeable-claim-kept（认领不放、留到租期过了）、
/// block-not-judgeable-line-left-out（不打判不了那一行）。
#[test]
fn a_block_running_out_of_memory_on_an_emptied_card_is_named_left_to_another_service_and_not_claimed_again_here(
) {
    let library = TemporaryLibraryService::start("out-of-memory-on-an-emptied-card");
    let conclusion_version = conclusion_version_of_the_crash_image_judgement();
    let mut pool = pool_on(
        &library,
        stand_in_side_with(&library, &[6, 7]),
        LONG_LEASE_MILLISECONDS,
    );
    pool.keep_the_service_lines();
    let refused = block_of(
        6,
        0,
        0..4,
        conclusion_version,
        ForcedRejudging::NotForced,
        1,
    );
    let next = block_of(
        7,
        0,
        0..4,
        conclusion_version,
        ForcedRejudging::NotForced,
        2,
    );
    let store = library.open_store();
    put_pending_rows(&store, std::slice::from_ref(&refused));
    let faults: StandInCardFaults = Arc::new(Mutex::new(BTreeMap::from([(
        6,
        (StandInCardFault::OutOfMemory, 2),
    )])));
    let summary = serve_with_faulty_stand_in_cards(&mut pool, &faults, || {
        wait_until(
            "第 6 条流的那一块照设定显存不够两回",
            &mut || faults_left(&faults, 6) == 0,
        );
        put_pending_rows(&store, std::slice::from_ref(&next));
        wait_until(
            &format!("{} 判完写回、待判行删掉", next.0),
            &mut || !pending_names(&store).contains(&next.0),
        );
        std::thread::sleep(IDLE_SPAN);
    });
    assert!(
        summary.cards_failed.is_empty(),
        "显存不够不是卡坏：{summary:?}"
    );
    assert_eq!(summary.out_of_memory_caught, 2);
    assert_eq!(summary.blocks_refused_for_running_out_of_memory, 1);
    assert!(
        pending_names(&store).contains(&refused.0),
        "{} 的待判行留在库里",
        refused.0
    );
    let lines = pool.kept_service_lines();
    let count_of = |prefix: &str, named: &str| {
        lines
            .iter()
            .filter(|line| line.starts_with(&format!("{prefix} ")) && line.contains(named))
            .count()
    };
    let refused_name = refused.0.to_string();
    assert_eq!(
        count_of(OUT_OF_MEMORY_CAUGHT_LINE_PREFIX, "card=0"),
        2,
        "两行接住 Out of Memory：{lines:?}"
    );
    assert_eq!(
        count_of(BLOCK_CLAIMED_LINE_PREFIX, &refused_name),
        1,
        "{refused_name} 这个进程只认领一回：{lines:?}"
    );
    assert_eq!(
        count_of(CARD_FAILED_LINE_PREFIX, ""),
        0,
        "没有卡坏那一行：{lines:?}"
    );
    let not_judgeable: Vec<&String> = lines
        .iter()
        .filter(|line| line.starts_with(&format!("{BLOCK_NOT_JUDGEABLE_LINE_PREFIX} ")))
        .collect();
    assert_eq!(not_judgeable.len(), 1, "恰好一行判不了：{lines:?}");
    for field in [
        "family=image-judgement".to_string(),
        format!(
            "stream={}",
            hexadecimal_text(&refused.0.stream.0[..STREAM_KEY_HEAD_BYTES_IN_LINES])
        ),
        format!("block={}", refused.0.block_number),
        "card=0".to_string(),
        "why=out-of-memory-on-an-emptied-card".to_string(),
        "claim=released".to_string(),
    ] {
        assert!(
            not_judgeable[0]
                .split_whitespace()
                .any(|word| word == field),
            "判不了那一行带 {field}：{}",
            not_judgeable[0]
        );
    }
    assert!(
        not_judgeable[0].contains("reason=\""),
        "判不了那一行带原因：{}",
        not_judgeable[0]
    );
    assert!(
        !is_a_dispatch_line(not_judgeable[0].as_bytes()),
        "那一行不被服务日志滤掉"
    );
    let mut other = library.open_client();
    assert_eq!(
        claim_a_pending_block(&mut other, &refused.0, LONG_LEASE_MILLISECONDS).expect("认领得了"),
        ClaimAnswer::Claimed,
        "{} 的认领当场放掉了，别的持有者认领得到",
        refused.0
    );
}
