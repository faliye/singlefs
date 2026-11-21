//! 存结论块与读结论块的测试：按种类与版本复用、读回的计数向量与恢复字、不同的结论拒存、两个写者撞同一把键、读时违例行在变。

use super::*;
use crate::crash_judge_conclusions::{LandedRootWords, RecoveryWordsOfAPass};

/// 同一块几份结论各存各的：镜像判定存了一把版本的，另一把版本（另一版判定定义）与观察者照样缺着（只排那两种）；
/// 种类不同、版本字节相同也互不顶替；两个观察者（种类相同、两把版本）各是各的；都存了就一种都不缺，结论块表四行。
/// 镜像判定不分 CPU 还是 GPU 判的：同一把版本下只有一份（在哪判的只在块属性里，另一条用例判）。
#[test]
fn conclusions_of_one_block_are_reused_and_missing_separately_by_kind_and_version() {
    let library = TestLibrary::new("three-kinds");
    let mut store = library.open();
    let clean = store
        .register_verdict_vector(&verdict_vector(b""))
        .expect("登记");
    let red = store
        .register_verdict_vector(&verdict_vector(b"red"))
        .expect("登记");
    let block_key = block_key_named("node", 0);
    let observer = ConclusionKind::Observer;
    let required = [
        (ConclusionKind::ImageJudgement, JUDGEMENT_CONCLUSION_VERSION),
        (
            ConclusionKind::ImageJudgement,
            ANOTHER_JUDGEMENT_CONCLUSION_VERSION,
        ),
        (observer.clone(), OBSERVER_CONCLUSION_VERSION),
    ];
    assert_eq!(
        store
            .conclusions_missing_for_block(&block_key, &required)
            .expect("对账"),
        required.to_vec(),
        "空库三种都缺"
    );
    store
        .store_conclusion_block(
            &block_key,
            &ConclusionKind::ImageJudgement,
            &JUDGEMENT_CONCLUSION_VERSION,
            &conclusion_record(block_of(&[clean, red]), &[1]),
        )
        .expect("存一把版本下的镜像判定");
    assert_eq!(
        store
            .conclusions_missing_for_block(&block_key, &required)
            .expect("对账"),
        required[1..].to_vec(),
        "存了这一把版本的，只缺另一把版本与观察者"
    );
    assert_eq!(
        store
            .read_conclusion_block(
                &block_key,
                &ConclusionKind::ImageJudgement,
                &ANOTHER_JUDGEMENT_CONCLUSION_VERSION
            )
            .expect("读"),
        None,
        "另一把版本读不到这一把下的结论"
    );
    assert_eq!(
        store
            .read_conclusion_block(
                &block_key,
                &ConclusionKind::Observer,
                &JUDGEMENT_CONCLUSION_VERSION
            )
            .expect("读"),
        None,
        "观察者种类拿同样的版本字节读不到镜像判定"
    );
    store
        .store_conclusion_block(
            &block_key,
            &ConclusionKind::ImageJudgement,
            &ANOTHER_JUDGEMENT_CONCLUSION_VERSION,
            &conclusion_record(block_of(&[clean, clean]), &[]),
        )
        .expect("存另一把版本下的镜像判定：与前一把下的不同也不算冲突");
    assert_eq!(
        store
            .conclusions_missing_for_block(&block_key, &required)
            .expect("对账"),
        required[2..].to_vec(),
        "只缺观察者"
    );
    let another_observer_version = ConclusionVersion([0x54; SHA256_DIGEST_BYTES]);
    store
        .store_conclusion_block(
            &block_key,
            &ConclusionKind::Observer,
            &another_observer_version,
            &conclusion_record(block_of(&[clean, clean]), &[]),
        )
        .expect("存另一个观察者的结果");
    assert_eq!(
        store
            .conclusions_missing_for_block(&block_key, &required)
            .expect("对账"),
        required[2..].to_vec(),
        "另一个观察者（另一把版本）顶替不了这一个"
    );
    store
        .store_conclusion_block(
            &block_key,
            &observer,
            &OBSERVER_CONCLUSION_VERSION,
            &conclusion_record(block_of(&[red, clean]), &[0]),
        )
        .expect("存观察者结果");
    assert_eq!(
        store
            .conclusions_missing_for_block(&block_key, &required)
            .expect("对账"),
        Vec::new(),
        "三种都有了"
    );
    assert_eq!(
        store
            .read_conclusion_block(
                &block_key,
                &ConclusionKind::ImageJudgement,
                &JUDGEMENT_CONCLUSION_VERSION
            )
            .expect("读")
            .map(|stored| stored.block),
        Some(block_of(&[clean, red])),
        "第一把版本下的结论没被后存的几份顶掉"
    );
    assert_eq!(
        store.stored_conclusion_keys().expect("列").len(),
        4,
        "结论块表四行：两把版本的镜像判定、两个观察者"
    );
}

/// 一块一种结论连同计数向量、块属性与恢复字写下去再读回逐项相同；关库重开照样相同。恢复字另一张表存，不进判定向量编号：
/// 存完判定向量还是登记的那两种。
#[test]
fn a_conclusion_block_reads_back_with_its_count_vector_attributes_and_recovery_words() {
    let library = TestLibrary::new("conclusion-read-back");
    let block_key = block_key_named("node", 65536);
    let (record, red) = {
        let mut store = library.open();
        let clean = store
            .register_verdict_vector(&verdict_vector(b""))
            .expect("登记");
        let red = store
            .register_verdict_vector(&verdict_vector(b"red"))
            .expect("登记");
        let record = conclusion_record(block_of(&[clean, red, clean]), &[1]);
        assert_eq!(
            store
                .store_conclusion_block(
                    &block_key,
                    &ConclusionKind::ImageJudgement,
                    &ANOTHER_JUDGEMENT_CONCLUSION_VERSION,
                    &record
                )
                .expect("存"),
            VerdictBlockStoring::WrittenAndSynced
        );
        assert_eq!(
            store.verdict_vector_by_number.len(),
            2,
            "恢复字不进判定向量编号"
        );
        let recovery_words_row = library
            .raw_value(
                &VerdictStoreKey::RecoveryWords(
                    &block_key,
                    &ConclusionKind::ImageJudgement,
                    &ANOTHER_JUDGEMENT_CONCLUSION_VERSION,
                )
                .to_key_bytes()
                .0,
            )
            .expect("恢复字在旁表里");
        assert_eq!(
            recovery_words_row.len(),
            3 * RECOVERY_WORDS_PER_STATE * 4,
            "旁表里每个状态 RECOVERY_WORDS_PER_STATE 个字、每字四个字节"
        );
        (record, red)
    };
    let store = library.open();
    let stored = store
        .read_conclusion_block(
            &block_key,
            &ConclusionKind::ImageJudgement,
            &ANOTHER_JUDGEMENT_CONCLUSION_VERSION,
        )
        .expect("读")
        .expect("存过");
    assert_eq!(stored.block, record.block);
    assert_eq!(
        stored.violations,
        vec![StoredViolation {
            offset_in_block: StateOffsetInBlock(1),
            verdict_vector_number: red,
        }]
    );
    assert_eq!(stored.count_vector, record.count_vector, "计数向量随块读回");
    assert_eq!(stored.judged_on, record.judged_on, "块属性随块读回");
    assert_eq!(
        stored.recovery_words, record.recovery_words,
        "恢复字随块读回"
    );
    assert_eq!(
        stored.circumstances,
        JudgementCircumstances {
            judged_date: tokyo_date_text_of(std::time::SystemTime::now()),
            run_kind: RunKindOfTheJudgement::of_this_process(),
        },
        "块属性带判定日期（东京日期）与跑法"
    );
    assert!(
        stored.circumstances.judged_date.len() == "YYYY-MM-DD".len()
            && stored.circumstances.judged_date.as_bytes()[4] == b'-'
            && stored.circumstances.judged_date.as_bytes()[7] == b'-',
        "判定日期写成 YYYY-MM-DD：{}",
        stored.circumstances.judged_date
    );
}

/// 审计复核 T01（`records/2026-10-24-singlefs-checker-tier-审计复核.md` checkpoint_txg「存完整 u64」）：恢复字里落到的根
/// checkpoint_txg 高 32 位非 0（1、`0xFFFF_FFFF`）、低 32 位相同的两个状态，连同一遍没落到根与落到实例代号 `u32::MAX`、checkpoint_txg
/// `u64::MAX` 那条根的一个状态，存进判定库、关库重开再读回逐字相同；读回的每一遍落到的根照完整的 checkpoint_txg 认得出。
#[test]
fn recovery_words_landing_on_checkpoint_txgs_past_thirty_two_bits_read_back_word_for_word() {
    let library = TestLibrary::new("recovery-words-past-thirty-two-bits");
    let block_key = block_key_named("node", 0);
    let pass_landed_on = |instance: u32, checkpoint_txg: u64| RecoveryWordsOfAPass {
        landed_root: LandedRootWords::of_the_landed_root(
            InstanceGeneration(instance),
            CheckpointTxg(checkpoint_txg),
        ),
        outcome: 1,
        failure_point: 0,
    };
    let pass_landed_on_no_root = RecoveryWordsOfAPass {
        landed_root: LandedRootWords::NOT_LANDED,
        outcome: 2,
        failure_point: 3,
    };
    let recovery_words = vec![
        RecoveryWordsOfState::of_the_two_passes(
            pass_landed_on(1, (1 << 32) | 7),
            pass_landed_on(1, 7),
        ),
        RecoveryWordsOfState::of_the_two_passes(
            pass_landed_on(1, (0xFFFF_FFFF << 32) | 7),
            pass_landed_on(1, (1 << 32) | 7),
        ),
        RecoveryWordsOfState::of_the_two_passes(
            pass_landed_on(u32::MAX, u64::MAX),
            pass_landed_on_no_root,
        ),
    ];
    {
        let mut store = library.open();
        let clean = store
            .register_verdict_vector(&verdict_vector(b""))
            .expect("登记");
        let mut record = conclusion_record(block_of(&[clean, clean, clean]), &[]);
        record.recovery_words = Some(recovery_words.clone());
        assert_eq!(
            store
                .store_conclusion_block(
                    &block_key,
                    &ConclusionKind::ImageJudgement,
                    &ANOTHER_JUDGEMENT_CONCLUSION_VERSION,
                    &record
                )
                .expect("存"),
            VerdictBlockStoring::WrittenAndSynced
        );
    }
    let store = library.open();
    let read_back = store
        .read_conclusion_block(
            &block_key,
            &ConclusionKind::ImageJudgement,
            &ANOTHER_JUDGEMENT_CONCLUSION_VERSION,
        )
        .expect("读")
        .expect("存过")
        .recovery_words
        .expect("恢复字随块存过");
    assert_eq!(
        read_back, recovery_words,
        "恢复字逐字读回，连同高 32 位与标记"
    );
    assert_eq!(
        read_back
            .iter()
            .map(|words| (
                words.consulted_pass().landed_root.landed_root(),
                words.ignored_pass().landed_root.landed_root()
            ))
            .collect::<Vec<_>>(),
        vec![
            (
                Some((InstanceGeneration(1), CheckpointTxg((1 << 32) | 7))),
                Some((InstanceGeneration(1), CheckpointTxg(7)))
            ),
            (
                Some((
                    InstanceGeneration(1),
                    CheckpointTxg((0xFFFF_FFFF << 32) | 7)
                )),
                Some((InstanceGeneration(1), CheckpointTxg((1 << 32) | 7)))
            ),
            (
                Some((InstanceGeneration(u32::MAX), CheckpointTxg(u64::MAX))),
                None
            ),
        ],
        "读回的每一遍落到的根照完整的 checkpoint_txg 认得出，没落到根读回 None"
    );
}

/// 同一（块键，种类，版本）下两份都不红而换一份结论（块、计数向量、恢复字任一不同）拒绝，拒了一个字都不写（两份里有一份红的照新的换上：
/// 记红、撤绿，`crash_amplification` 的 `storing_a_red_conclusion_over_a_green_one_replaces_it_and_back_but_two_different_green_ones_are_refused` 钉）；
/// 结论相同只换块属性（换一张卡判的）算已存、不写、块属性留先存的那一份；逐项相同的再存放行、不写。
#[test]
fn a_different_conclusion_under_a_stored_key_is_refused_while_another_judging_device_is_accepted_and_the_library_is_unchanged(
) {
    let library = TestLibrary::new("different-conclusion");
    let mut store = library.open();
    let clean = store
        .register_verdict_vector(&verdict_vector(b""))
        .expect("登记");
    let red = store
        .register_verdict_vector(&verdict_vector(b"red"))
        .expect("登记");
    let block_key = block_key_named("node", 0);
    let record = conclusion_record(block_of(&[clean, red]), &[]);
    store
        .store_conclusion_block(
            &block_key,
            &ConclusionKind::ImageJudgement,
            &JUDGEMENT_CONCLUSION_VERSION,
            &record,
        )
        .expect("存");
    let entries_before = store.every_stored_entry();
    let mut other_counts = record.clone();
    other_counts
        .count_vector
        .per_observer
        .insert("mounted_states".to_string(), 99);
    let mut other_recovery_words = record.clone();
    other_recovery_words.recovery_words = Some(recovery_words_of_the_test_block(3)[1..].to_vec());
    let mut other_block = record.clone();
    other_block.block = block_of(&[clean, clean]);
    for (different, what) in [
        (other_counts, "计数向量不同"),
        (other_recovery_words, "恢复字不同"),
        (other_block, "块不同"),
    ] {
        let refused = store.store_conclusion_block(
            &block_key,
            &ConclusionKind::ImageJudgement,
            &JUDGEMENT_CONCLUSION_VERSION,
            &different,
        );
        assert!(
                matches!(
                    refused,
                    Err(VerdictStoreError::DifferentConclusionUnderAStoredKeyNotDecidedInTheFirstVersion { .. })
                ),
                "{what}要拒：{refused:?}"
            );
    }
    let mut other_device = record.clone();
    other_device.judged_on = JudgingDevice::Gpu {
        card_type: "probe card".to_string(),
        driver_version: "probe driver 2".to_string(),
    };
    assert_eq!(
        store
            .store_conclusion_block(
                &block_key,
                &ConclusionKind::ImageJudgement,
                &JUDGEMENT_CONCLUSION_VERSION,
                &other_device,
            )
            .expect("结论相同、只有在哪判的不同：算已存"),
        VerdictBlockStoring::AlreadyStoredIdentically,
        "同一张表换一张卡判出同一份结论，算已存"
    );
    assert_eq!(
        store.every_stored_entry(),
        entries_before,
        "算已存就一个字都不写，块属性留先存的那一份"
    );
    assert_eq!(
        store
            .read_conclusion_block(
                &block_key,
                &ConclusionKind::ImageJudgement,
                &JUDGEMENT_CONCLUSION_VERSION,
            )
            .expect("读")
            .expect("存着")
            .judged_on,
        record.judged_on,
        "留的是先存的那一份块属性"
    );
    assert_eq!(
        store
            .store_conclusion_block(
                &block_key,
                &ConclusionKind::ImageJudgement,
                &JUDGEMENT_CONCLUSION_VERSION,
                &record
            )
            .expect("再存同一份"),
        VerdictBlockStoring::AlreadyStoredIdentically
    );
    assert_eq!(store.every_stored_entry(), entries_before);
}

/// 同一个（块键，种类，版本）下两份都不红而不同的结论：存第二份拒在 `DifferentConclusionUnderAStoredKeyNotDecidedInTheFirstVersion` 上，
/// 带着两份差在哪（块内哪个状态的判定向量不同，一处一行，不交空的）与两份各在哪判的：两台各判同一块撞上时只读报错就定得出差在哪一样。
/// 判别力（`crates/mutations.tsv`）：报错不带两份差在哪（差异清单交空的、两份在哪判的都写成 CPU），这条红。
#[test]
fn a_refused_different_green_conclusion_names_the_state_that_differs_and_where_each_was_judged() {
    let library = TestLibrary::new("different-green-names-the-difference");
    let mut store = library.open();
    let clean = store
        .register_verdict_vector(&verdict_vector(b""))
        .expect("登记");
    let judged_differently = store
        .register_verdict_vector(&verdict_vector(b"judged differently"))
        .expect("登记");
    let block_key = block_key_named("node-with-two-green-conclusions", 0);
    let stored = conclusion_record(block_of(&[clean, clean]), &[]);
    store
        .store_conclusion_block(
            &block_key,
            &ConclusionKind::ImageJudgement,
            &JUDGEMENT_CONCLUSION_VERSION,
            &stored,
        )
        .expect("存头一份");
    let mut requested = conclusion_record(block_of(&[clean, judged_differently]), &[]);
    requested.judged_on = JudgingDevice::Cpu;
    match store.store_conclusion_block(
        &block_key,
        &ConclusionKind::ImageJudgement,
        &JUDGEMENT_CONCLUSION_VERSION,
        &requested,
    ) {
        Err(VerdictStoreError::DifferentConclusionUnderAStoredKeyNotDecidedInTheFirstVersion {
            differences,
            ..
        }) => {
            assert!(
                !differences.differences.is_empty(),
                "两份差在哪一处一行，不交空的"
            );
            assert_eq!(
                differences.stored_judged_on, stored.judged_on,
                "库里那一份在哪判的"
            );
            assert_eq!(
                differences.requested_judged_on,
                JudgingDevice::Cpu,
                "要存的那一份在哪判的"
            );
        }
        outcome => panic!("两份都不红而不同要拒、带着差在哪：{outcome:?}"),
    }
}

/// 清掉一个读窗口上登记的动作（用例红了 panic 时也清，同一个线程上跑的下一条用例不沾）。
struct ActionInAReadWindowClearedWhenDropped(&'static std::thread::LocalKey<ActionInAReadWindow>);

impl Drop for ActionInAReadWindowClearedWhenDropped {
    fn drop(&mut self) {
        self.0.set(None);
    }
}

/// 跑 `run_the_reads`，其间每一次走到 `read_window` 跑一次 `action`。交回 `run_the_reads` 的结局与 `action` 跑了几次。
fn run_with_an_action_in_a_read_window<Outcome>(
    read_window: &'static std::thread::LocalKey<ActionInAReadWindow>,
    mut action: impl FnMut() + 'static,
    run_the_reads: impl FnOnce() -> Outcome,
) -> (Outcome, usize) {
    let action_runs = std::rc::Rc::new(std::cell::Cell::new(0usize));
    let action_runs_counted_by_the_action = std::rc::Rc::clone(&action_runs);
    read_window.set(Some(Box::new(move || {
        action_runs_counted_by_the_action.set(action_runs_counted_by_the_action.get() + 1);
        action();
    })));
    let _cleared_when_dropped = ActionInAReadWindowClearedWhenDropped(read_window);
    let outcome = run_the_reads();
    (outcome, action_runs.get())
}

/// 跑 `store_the_block`，其间每一次存块之前那一帧读到结论块不在、旁表有行，重读结论块之前跑一次 `action`。交回 `store_the_block` 的结局与 `action` 跑了几次。
fn store_with_an_action_between_the_two_reads<Stored>(
    action: impl FnMut() + 'static,
    store_the_block: impl FnOnce() -> Stored,
) -> (Stored, usize) {
    run_with_an_action_in_a_read_window(
        &ACTION_BETWEEN_THE_CONCLUSION_READ_AND_THE_SIDE_ROW_READ,
        action,
        store_the_block,
    )
}

/// 两个写者都开着、都登记过绿与红两种判定向量；库里还什么结论都没存。
struct TwoWritersOfOneKey {
    _library: TestLibrary,
    this_writer: VerdictStore,
    other_writer: VerdictStore,
    block_key: VerdictBlockKey,
    green_record: ConclusionBlockRecord,
    red_record: ConclusionBlockRecord,
}

fn two_writers_of_one_key(library_name: &str) -> TwoWritersOfOneKey {
    let library = TestLibrary::new(library_name);
    let mut this_writer = library.open();
    let mut other_writer = library.open();
    let clean = this_writer
        .register_verdict_vector(&verdict_vector(b""))
        .expect("登记");
    let red = this_writer
        .register_verdict_vector(&verdict_vector(b"red"))
        .expect("登记");
    other_writer
        .register_verdict_vector(&verdict_vector(b""))
        .expect("另一个写者登记");
    other_writer
        .register_verdict_vector(&verdict_vector(b"red"))
        .expect("另一个写者登记");
    TwoWritersOfOneKey {
        _library: library,
        this_writer,
        other_writer,
        block_key: block_key_named("node", 0),
        green_record: conclusion_record(block_of(&[clean, clean]), &[]),
        red_record: conclusion_record(block_of(&[clean, red]), &[1]),
    }
}

/// `store` 把 `record` 存在 `block_key` 这把键下（用例里那一种结论、那一把版本），写不下就停。
fn store_the_record(
    store: &mut VerdictStore,
    block_key: &VerdictBlockKey,
    record: &ConclusionBlockRecord,
) -> VerdictBlockStoring {
    store
        .store_conclusion_block(
            block_key,
            &ConclusionKind::ImageJudgement,
            &JUDGEMENT_CONCLUSION_VERSION,
            record,
        )
        .expect("存得下")
}

/// 读结论块那一帧之后、后扫违例行之前，另一个写者把库里的红结论换成绿的（同批删掉违例行）；这一趟要存的也是绿的：
/// 两次扫不同，整遍重读，读到绿的、比出已存相同，这一趟不判红。动作跑两次：换的那一遍与重读的那一遍。
/// 判别力（`crates/mutations.tsv`）：不比两次扫、读完结论块才扫违例，这条红在报两份都不红而不同。
#[test]
fn writer_replacing_a_red_conclusion_with_a_green_one_after_the_rows_read_compares_as_already_stored(
) {
    let TwoWritersOfOneKey {
        _library,
        mut this_writer,
        mut other_writer,
        block_key,
        green_record,
        red_record,
    } = two_writers_of_one_key("writer-red-to-green-after-the-rows-read");
    store_the_record(&mut this_writer, &block_key, &red_record);
    let block_key_of_the_other_writer = block_key.clone();
    let green_record_of_the_other_writer = green_record.clone();
    let (stored, action_runs) = run_with_an_action_in_a_read_window(
        &ACTION_BETWEEN_THE_CONCLUSION_ROWS_READ_AND_THE_LATER_VIOLATION_SCAN,
        move || {
            store_the_record(
                &mut other_writer,
                &block_key_of_the_other_writer,
                &green_record_of_the_other_writer,
            );
        },
        || {
            this_writer.store_conclusion_block(
                &block_key,
                &ConclusionKind::ImageJudgement,
                &JUDGEMENT_CONCLUSION_VERSION,
                &green_record,
            )
        },
    );
    assert_eq!(
        stored.expect("读到的不许是红的结论块配上没有违例：两份都不红而不同这一趟就判红了"),
        VerdictBlockStoring::AlreadyStoredIdentically,
        "重读读到另一个写者换上的绿结论，比出相同"
    );
    assert_eq!(action_runs, 2, "换的那一遍对不上，重读一遍对上就停");
}

/// 先扫违例行之后、读结论块那一帧之前，另一个写者把库里的绿结论换成红的；这一趟要存的是绿的：
/// 两次扫不同，整遍重读，读到红的、连着违例，照记红的规矩换上，不判红。
/// 判别力（`crates/mutations.tsv`）：不比两次扫、交先扫的违例行，这条红在读到红的结论块配上没有违例、报两份都不红而不同。
#[test]
fn writer_replacing_a_green_conclusion_with_a_red_one_before_the_rows_read_is_replaced_back() {
    let TwoWritersOfOneKey {
        _library,
        mut this_writer,
        mut other_writer,
        block_key,
        green_record,
        red_record,
    } = two_writers_of_one_key("writer-green-to-red-before-the-rows-read");
    store_the_record(&mut this_writer, &block_key, &green_record);
    let block_key_of_the_other_writer = block_key.clone();
    let (stored, action_runs) = run_with_an_action_in_a_read_window(
        &ACTION_BETWEEN_THE_EARLIER_VIOLATION_SCAN_AND_THE_CONCLUSION_ROWS_READ,
        move || {
            store_the_record(
                &mut other_writer,
                &block_key_of_the_other_writer,
                &red_record,
            );
        },
        || {
            this_writer.store_conclusion_block(
                &block_key,
                &ConclusionKind::ImageJudgement,
                &JUDGEMENT_CONCLUSION_VERSION,
                &green_record,
            )
        },
    );
    assert_eq!(
        stored.expect("读到的不许是红的结论块配上没有违例：两份都不红而不同这一趟就判红了"),
        VerdictBlockStoring::ReplacedBecauseOneOfTheTwoIsRed,
        "重读读到红结论连着违例，两份里有一份红的照这一份换上"
    );
    assert_eq!(action_runs, 2, "换的那一遍对不上，重读一遍对上就停");
}

/// 只读不存的读者走同一份读法：读结论块那一帧之后另一个写者把红结论换成绿的，交回的是绿的结论块、没有违例，不是红的结论块配上没有违例。
/// 判别力（`crates/mutations.tsv`）：不比两次扫、读完结论块才扫违例，这条红在结论块是红的。
#[test]
fn reader_during_a_replacement_gets_the_block_and_the_violations_of_one_batch() {
    let TwoWritersOfOneKey {
        _library,
        mut this_writer,
        mut other_writer,
        block_key,
        green_record,
        red_record,
    } = two_writers_of_one_key("reader-during-a-replacement");
    store_the_record(&mut this_writer, &block_key, &red_record);
    let block_key_of_the_other_writer = block_key.clone();
    let green_block = green_record.block.clone();
    let (read_back, _action_runs) = run_with_an_action_in_a_read_window(
        &ACTION_BETWEEN_THE_CONCLUSION_ROWS_READ_AND_THE_LATER_VIOLATION_SCAN,
        move || {
            store_the_record(
                &mut other_writer,
                &block_key_of_the_other_writer,
                &green_record,
            );
        },
        || {
            this_writer.read_conclusion_block(
                &block_key,
                &ConclusionKind::ImageJudgement,
                &JUDGEMENT_CONCLUSION_VERSION,
            )
        },
    );
    let read_back = read_back.expect("读块").expect("这把键一直有结论");
    assert_eq!(read_back.block, green_block, "结论块是换上的绿结论");
    assert!(read_back.violations.is_empty(), "绿结论没有违例");
}

/// 别的写者每一遍都换这把键的结论：读够上限那么多遍（写在用例里：`CONCLUSION_READ_ATTEMPTS_WHILE_THE_VIOLATION_ROWS_KEEP_CHANGING`）就报结论违例表，不交半批，读者一行不写。
/// 判别力（`crates/mutations.tsv`）：读够遍数之后当没存过，这条红在结局不是错；上限改小，这条红在动作跑的遍数。
#[test]
fn violation_rows_changing_on_every_attempt_report_the_violations_table_after_the_bound() {
    let TwoWritersOfOneKey {
        _library,
        mut this_writer,
        mut other_writer,
        block_key,
        green_record,
        red_record,
    } = two_writers_of_one_key("violation-rows-changing-on-every-attempt");
    store_the_record(&mut this_writer, &block_key, &red_record);
    let block_key_of_the_other_writer = block_key.clone();
    let mut next_record_is_green = true;
    let (read_back, action_runs) = run_with_an_action_in_a_read_window(
        &ACTION_BETWEEN_THE_CONCLUSION_ROWS_READ_AND_THE_LATER_VIOLATION_SCAN,
        move || {
            let next_record = if next_record_is_green {
                &green_record
            } else {
                &red_record
            };
            store_the_record(
                &mut other_writer,
                &block_key_of_the_other_writer,
                next_record,
            );
            next_record_is_green = !next_record_is_green;
        },
        || {
            this_writer.read_conclusion_block(
                &block_key,
                &ConclusionKind::ImageJudgement,
                &JUDGEMENT_CONCLUSION_VERSION,
            )
        },
    );
    assert!(
            matches!(
                read_back,
                Err(VerdictStoreError::StoredBytesDoNotMatchTheLayout {
                    table: VerdictStoreTable::ConclusionViolations,
                    reason: "读结论块那一帧前后两次扫到的结论违例行每一遍都不同：别的进程一直在换这把键的结论",
                    ..
                })
            ),
            "每一遍都对不上要报结论违例表：{read_back:?}"
        );
    assert_eq!(action_runs, 8, "上限是 8 遍，每一遍动作跑一次");
}

/// 没有别的写者时，读一遍就对上：动作只跑一次，交回存下的结论块与违例。
/// 判别力（`crates/mutations.tsv`）：两次扫相同反当成对不上，这条红在读够遍数报错。
#[test]
fn reading_a_conclusion_nobody_is_replacing_takes_one_attempt_and_returns_its_violations() {
    let TwoWritersOfOneKey {
        _library,
        mut this_writer,
        other_writer: _other_writer,
        block_key,
        green_record: _green_record,
        red_record,
    } = two_writers_of_one_key("reading-a-conclusion-nobody-is-replacing");
    store_the_record(&mut this_writer, &block_key, &red_record);
    let (read_back, action_runs) = run_with_an_action_in_a_read_window(
        &ACTION_BETWEEN_THE_CONCLUSION_ROWS_READ_AND_THE_LATER_VIOLATION_SCAN,
        || {},
        || {
            this_writer.read_conclusion_block(
                &block_key,
                &ConclusionKind::ImageJudgement,
                &JUDGEMENT_CONCLUSION_VERSION,
            )
        },
    );
    let read_back = read_back.expect("读块").expect("存过");
    assert_eq!(read_back.block, red_record.block, "结论块是存下的");
    assert_eq!(
        read_back
            .violations
            .iter()
            .map(|violation| violation.offset_in_block)
            .collect::<Vec<StateOffsetInBlock>>(),
        vec![StateOffsetInBlock(1)],
        "违例是存下的"
    );
    assert_eq!(action_runs, 1, "两次扫相同，读一遍就停");
}

/// 每个状态点名同一条被抛弃根（点名的被抛弃根旁表的值）。
fn abandoned_roots_of_each_of_the_test_states(
    states: usize,
) -> Vec<Vec<AbandonedRootNamedByThePoolChecker>> {
    vec![
        vec![AbandonedRootNamedByThePoolChecker {
            instance: InstanceGeneration(3),
            checkpoint_txg: CheckpointTxg(9),
        }];
        states
    ]
}

/// 一份库里几行的原样（键，值）。
type RawRowsOfALibrary = Vec<(Vec<u8>, Vec<u8>)>;

/// 另一个写者在它自己那一份库（`library_name`，登记 `verdict_vectors` 那几种、与被测的那一份同序）里存下 `record`（`roots` 给了就连同点名的根）之后，
/// 这一块那几行的原样（键，值）：旁表（恢复字、点名的被抛弃根）里有的那几行在前一份，结论块、计数向量、块属性三行在后一份。
/// 用例把前一份先直接写进被测的那一份库、后一份在存块之前那一帧之后才写，造「成批点读读过结论块那一键之后，别的进程把整批写进去」的那一刻
/// （一帧成批点读各键不是同一刻读的：结论块那一键读到不在、旁表那一键读到在）。
fn rows_of_a_conclusion_another_writer_stored(
    library_name: &str,
    verdict_vectors: &[&[u8]],
    block_key: &VerdictBlockKey,
    record: &ConclusionBlockRecord,
    roots: Option<&[Vec<AbandonedRootNamedByThePoolChecker>]>,
) -> (RawRowsOfALibrary, RawRowsOfALibrary) {
    let library_of_the_other_writer = TestLibrary::new(library_name);
    let mut other_writer = library_of_the_other_writer.open();
    for bytes in verdict_vectors {
        other_writer
            .register_verdict_vector(&verdict_vector(bytes))
            .expect("另一个写者登记");
    }
    match roots {
        Some(roots) => other_writer.store_conclusion_block_naming_the_abandoned_roots(
            block_key,
            &ConclusionKind::ImageJudgement,
            &JUDGEMENT_CONCLUSION_VERSION,
            record,
            roots,
        ),
        None => other_writer.store_conclusion_block(
            block_key,
            &ConclusionKind::ImageJudgement,
            &JUDGEMENT_CONCLUSION_VERSION,
            record,
        ),
    }
    .expect("另一个写者在它自己那一份库里存得下");
    let kind = ConclusionKind::ImageJudgement;
    let rows_of = |keys: Vec<VerdictStoreKeyBytes>| -> RawRowsOfALibrary {
        keys.into_iter()
            .filter_map(|key| {
                library_of_the_other_writer
                    .raw_value(&key.0)
                    .map(|value| (key.0, value))
            })
            .collect()
    };
    (
        rows_of(vec![
            VerdictStoreKey::RecoveryWords(block_key, &kind, &JUDGEMENT_CONCLUSION_VERSION)
                .to_key_bytes(),
            VerdictStoreKey::AbandonedRootsNamedByThePoolChecker(
                block_key,
                &kind,
                &JUDGEMENT_CONCLUSION_VERSION,
            )
            .to_key_bytes(),
        ]),
        rows_of(vec![
            VerdictStoreKey::ConclusionBlock(block_key, &kind, &JUDGEMENT_CONCLUSION_VERSION)
                .to_key_bytes(),
            VerdictStoreKey::ConclusionCountVector(block_key, &kind, &JUDGEMENT_CONCLUSION_VERSION)
                .to_key_bytes(),
            VerdictStoreKey::ConclusionBlockAttributes(
                block_key,
                &kind,
                &JUDGEMENT_CONCLUSION_VERSION,
            )
            .to_key_bytes(),
        ]),
    )
}

/// 存块之前那一帧读到结论块不在、恢复字旁表有行，读完之后另一个写者那一批的结论块那几行才落（另一个写者存的是同一份结论）：这不是孤儿，
/// 重读读到那一份、照常比出已存相同，这一趟不判红。动作只跑一次：重读之后不再读旁表。
/// 判别力（`crates/mutations.tsv`）：不重读结论块（当重读仍没有），这条红在报恢复字旁表孤儿。
#[test]
fn writer_landing_between_the_conclusion_read_and_the_recovery_words_read_compares_as_already_stored(
) {
    let library = TestLibrary::new("writer-between-reads-recovery-words");
    let mut this_writer = library.open();
    let clean = this_writer
        .register_verdict_vector(&verdict_vector(b""))
        .expect("登记");
    let block_key = block_key_named("node", 0);
    let record = conclusion_record(block_of(&[clean, clean]), &[]);
    let (side_rows, conclusion_rows) = rows_of_a_conclusion_another_writer_stored(
        "writer-between-reads-recovery-words-other-writer",
        &[b""],
        &block_key,
        &record,
        None,
    );
    assert_eq!(
        (side_rows.len(), conclusion_rows.len()),
        (1, 3),
        "前提：另一个写者那一批旁表只有恢复字一行，结论块那一侧三行"
    );
    library.write_raw_rows(side_rows);
    let service_of_the_other_writer = library.service();
    let (stored, action_runs) = store_with_an_action_between_the_two_reads(
        move || {
            service_of_the_other_writer
                .write_rows_for_a_test(conclusion_rows.clone())
                .expect("另一个写者那一批的结论块那几行写得进去");
        },
        || {
            this_writer.store_conclusion_block(
                &block_key,
                &ConclusionKind::ImageJudgement,
                &JUDGEMENT_CONCLUSION_VERSION,
                &record,
            )
        },
    );
    assert_eq!(
        stored.expect("另一个写者写进去的是同一把键的整批，不是孤儿"),
        VerdictBlockStoring::AlreadyStoredIdentically,
        "重读读到另一个写者那一份，比出相同"
    );
    assert_eq!(action_runs, 1, "结论块第一读没有之后只读一次旁表");
}

/// 同上，另一个写者那一批只带点名的被抛弃根、不带恢复字：那一帧里恢复字旁表没有行、被抛弃根旁表有，照样重读、比出已存相同。
/// 判别力（`crates/mutations.tsv`）：只看恢复字旁表、不看被抛弃根旁表，这条红在当没存过写下去。
#[test]
fn writer_landing_between_the_conclusion_read_and_the_abandoned_roots_read_compares_as_already_stored(
) {
    let library = TestLibrary::new("writer-between-reads-abandoned-roots");
    let mut this_writer = library.open();
    let clean = this_writer
        .register_verdict_vector(&verdict_vector(b""))
        .expect("登记");
    let block_key = block_key_named("node", 0);
    let mut record = conclusion_record(block_of(&[clean, clean]), &[]);
    record.recovery_words = None;
    let roots = abandoned_roots_of_each_of_the_test_states(2);
    let (side_rows, conclusion_rows) = rows_of_a_conclusion_another_writer_stored(
        "writer-between-reads-abandoned-roots-other-writer",
        &[b""],
        &block_key,
        &record,
        Some(roots.as_slice()),
    );
    assert_eq!(
        (side_rows.len(), conclusion_rows.len()),
        (1, 3),
        "前提：另一个写者那一批旁表只有被抛弃根一行，结论块那一侧三行"
    );
    library.write_raw_rows(side_rows);
    let service_of_the_other_writer = library.service();
    let (stored, _action_runs) = store_with_an_action_between_the_two_reads(
        move || {
            service_of_the_other_writer
                .write_rows_for_a_test(conclusion_rows.clone())
                .expect("另一个写者那一批的结论块那几行写得进去");
        },
        || {
            this_writer.store_conclusion_block_naming_the_abandoned_roots(
                &block_key,
                &ConclusionKind::ImageJudgement,
                &JUDGEMENT_CONCLUSION_VERSION,
                &record,
                &roots,
            )
        },
    );
    assert_eq!(
        stored.expect("另一个写者写进去的是同一把键的整批，不是孤儿"),
        VerdictBlockStoring::AlreadyStoredIdentically,
        "重读读到另一个写者那一份，比出相同"
    );
}

/// 重读读到的那一份走的是存块那一整条比法，不只是比相同：另一个写者在那一帧之后落下绿的，这一趟判红的照记红的规矩换上，读回是这一趟的红结论。
/// 判别力（`crates/mutations.tsv`）：重读到了也当没存过，这条红在结局是写下而不是换上。
#[test]
fn writer_landing_between_the_two_reads_with_a_green_conclusion_is_replaced_by_a_red_one() {
    let library = TestLibrary::new("writer-between-reads-green-replaced-by-red");
    let mut this_writer = library.open();
    let clean = this_writer
        .register_verdict_vector(&verdict_vector(b""))
        .expect("登记");
    let red = this_writer
        .register_verdict_vector(&verdict_vector(b"red"))
        .expect("登记");
    let block_key = block_key_named("node", 0);
    let green_record = conclusion_record(block_of(&[clean, clean]), &[]);
    let red_record = conclusion_record(block_of(&[clean, red]), &[1]);
    let (side_rows, conclusion_rows) = rows_of_a_conclusion_another_writer_stored(
        "writer-between-reads-green-replaced-by-red-other-writer",
        &[b"", b"red"],
        &block_key,
        &green_record,
        None,
    );
    library.write_raw_rows(side_rows);
    let service_of_the_other_writer = library.service();
    let (stored, _action_runs) = store_with_an_action_between_the_two_reads(
        move || {
            service_of_the_other_writer
                .write_rows_for_a_test(conclusion_rows.clone())
                .expect("另一个写者那一批的结论块那几行写得进去");
        },
        || {
            this_writer.store_conclusion_block(
                &block_key,
                &ConclusionKind::ImageJudgement,
                &JUDGEMENT_CONCLUSION_VERSION,
                &red_record,
            )
        },
    );
    assert_eq!(
        stored.expect("两份里有一份红的照这一份换上，不报错"),
        VerdictBlockStoring::ReplacedBecauseOneOfTheTwoIsRed,
        "重读读到绿的那一份，这一趟红的换上"
    );
    let read_back = this_writer
        .read_conclusion_block(
            &block_key,
            &ConclusionKind::ImageJudgement,
            &JUDGEMENT_CONCLUSION_VERSION,
        )
        .expect("读块")
        .expect("换上之后在");
    assert_eq!(
        read_back.block,
        block_of(&[clean, red]),
        "读回是这一趟的红结论"
    );
}

/// 没存过的块（存块那一处的常态）存块之前只读一帧：结论块那五行一帧成批点读，结论块与两张旁表都不在就当没存过写下去，
/// 不扫违例行、不另读旁表、不重读（两处读窗口上的动作一次都不跑）；写下的读回逐项是这一份。
/// 判别力（`crates/mutations.tsv`）：「判定库 存块之前那一帧不在时旁表一律当有行」那一行让这一条红在报恢复字旁表孤儿。
#[test]
fn a_block_never_stored_is_stored_after_one_read_of_its_rows() {
    let library = TestLibrary::new("block-never-stored-one-read");
    let mut store = library.open();
    let clean = store
        .register_verdict_vector(&verdict_vector(b""))
        .expect("登记");
    let red = store
        .register_verdict_vector(&verdict_vector(b"red"))
        .expect("登记");
    let block_key = block_key_named("node", 0);
    let record = conclusion_record(block_of(&[clean, red]), &[1]);
    let ((stored, side_row_reads), violation_scans) = run_with_an_action_in_a_read_window(
        &ACTION_BETWEEN_THE_EARLIER_VIOLATION_SCAN_AND_THE_CONCLUSION_ROWS_READ,
        || {},
        || {
            store_with_an_action_between_the_two_reads(
                || {},
                || {
                    store.store_conclusion_block(
                        &block_key,
                        &ConclusionKind::ImageJudgement,
                        &JUDGEMENT_CONCLUSION_VERSION,
                        &record,
                    )
                },
            )
        },
    );
    assert_eq!(
        stored.expect("没存过的块存得下"),
        VerdictBlockStoring::WrittenAndSynced
    );
    assert_eq!(
        (violation_scans, side_row_reads),
        (0, 0),
        "存块之前只读那一帧：不整遍读结论块（不扫违例行）、不重读"
    );
    let read_back = store
        .read_conclusion_block(
            &block_key,
            &ConclusionKind::ImageJudgement,
            &JUDGEMENT_CONCLUSION_VERSION,
        )
        .expect("读块")
        .expect("写下之后在");
    assert_eq!(
        (
            read_back.block,
            read_back.violations.len(),
            read_back.recovery_words
        ),
        (block_of(&[clean, red]), 1, record.recovery_words.clone()),
        "读回逐项是写下的这一份"
    );
}

/// 场景（判器交回的逐盘槽数与结论同一批写）：一块结论先不带槽数存下；同一份结论再连同判器交回的逐盘槽数存一次（结论在、槽数不在的块照判一遍补上的那一路）。
/// 预期：第二次交回已存相同，槽数那一行照样写下、读回逐字节是交的那一份；另一块头一回就连同槽数存的，结论与槽数都读得回。
/// 判别力：`crates/mutations.tsv` 里「判定库 结论已存着相同时槽数那一行也不写」那一行让这一条红在第二次存之后读不回槽数。
#[test]
fn the_slots_handed_back_are_written_along_even_when_the_conclusion_is_already_stored() {
    let library = TestLibrary::new("slots-written-along-the-conclusion");
    let mut store = library.open();
    let clean = store
        .register_verdict_vector(&verdict_vector(b""))
        .expect("登记");
    let kind = ConclusionKind::ImageJudgement;
    let record = conclusion_record(block_of(&[clean, clean]), &[]);
    let slots_bytes = b"slots handed back by the judge in the unit test".to_vec();
    let stored_first_without_the_slots = block_key_named("node", 0);
    store
        .store_conclusion_block(
            &stored_first_without_the_slots,
            &kind,
            &JUDGEMENT_CONCLUSION_VERSION,
            &record,
        )
        .expect("存得下");
    assert_eq!(
        store
            .store_conclusion_block_with_the_slots_the_judge_handed_back(
                &stored_first_without_the_slots,
                &kind,
                &JUDGEMENT_CONCLUSION_VERSION,
                record.clone(),
                Some(slots_bytes.clone()),
            )
            .expect("存得下"),
        VerdictBlockStoring::AlreadyStoredIdentically,
        "结论逐项相同：算已存"
    );
    assert_eq!(
        store
            .read_observer_block(
                &stored_first_without_the_slots,
                &kind,
                &JUDGEMENT_CONCLUSION_VERSION
            )
            .expect("读得了"),
        Some(slots_bytes.clone()),
        "结论已存着，槽数那一行照样写下"
    );
    let stored_with_the_slots = block_key_named("node", 2);
    assert_eq!(
        store
            .store_conclusion_block_with_the_slots_the_judge_handed_back(
                &stored_with_the_slots,
                &kind,
                &JUDGEMENT_CONCLUSION_VERSION,
                record.clone(),
                Some(slots_bytes.clone()),
            )
            .expect("存得下"),
        VerdictBlockStoring::WrittenAndSynced
    );
    assert_eq!(
        (
            store
                .read_observer_block(&stored_with_the_slots, &kind, &JUDGEMENT_CONCLUSION_VERSION)
                .expect("读得了"),
            store
                .read_conclusion_block(&stored_with_the_slots, &kind, &JUDGEMENT_CONCLUSION_VERSION)
                .expect("读得了")
                .map(|stored| stored.block),
        ),
        (Some(slots_bytes), Some(record.block)),
        "头一回连同槽数存的：结论与槽数都读得回"
    );
}

/// 真孤儿：恢复字旁表有这一行、结论块表怎么读都没有：重读一次仍没有就报恢复字旁表不合布局，只读一次旁表（重读有上限），库里一行不动。
/// 判别力（`crates/mutations.tsv`）：重读仍没有也当没存过，这条红在把整批写了下去。
#[test]
fn recovery_words_row_without_its_conclusion_block_after_the_reread_reports_the_orphan_and_writes_nothing(
) {
    let library = TestLibrary::new("recovery-words-orphan");
    let mut store = library.open();
    let clean = store
        .register_verdict_vector(&verdict_vector(b""))
        .expect("登记");
    let block_key = block_key_named("node", 0);
    let record = conclusion_record(block_of(&[clean, clean]), &[]);
    library.write_raw_rows(vec![(
        VerdictStoreKey::RecoveryWords(
            &block_key,
            &ConclusionKind::ImageJudgement,
            &JUDGEMENT_CONCLUSION_VERSION,
        )
        .to_key_bytes()
        .0,
        recovery_words_value_bytes(&recovery_words_of_the_test_block(2)),
    )]);
    let rows_before = store.every_stored_entry();
    let (stored, action_runs) = store_with_an_action_between_the_two_reads(
        || {},
        || {
            store.store_conclusion_block(
                &block_key,
                &ConclusionKind::ImageJudgement,
                &JUDGEMENT_CONCLUSION_VERSION,
                &record,
            )
        },
    );
    assert!(
        matches!(
            stored,
            Err(VerdictStoreError::StoredBytesDoNotMatchTheLayout {
                table: VerdictStoreTable::RecoveryWords,
                reason: "恢复字旁表有这一块这一种结论这一版本，结论块表没有",
                ..
            })
        ),
        "重读仍没有要报恢复字旁表孤儿：{stored:?}"
    );
    assert_eq!(action_runs, 1, "只读一次旁表、不循环");
    assert_eq!(
        store.every_stored_entry(),
        rows_before,
        "报错之前一行都没写"
    );
}

/// 真孤儿：只有被抛弃根旁表有这一行：报的是被抛弃根那一张，不是恢复字那一张。判别力（`crates/mutations.tsv`）：报错点名的表写成恢复字旁表，这条红。
#[test]
fn abandoned_roots_row_without_its_conclusion_block_after_the_reread_reports_that_table() {
    let library = TestLibrary::new("abandoned-roots-orphan");
    let mut store = library.open();
    let clean = store
        .register_verdict_vector(&verdict_vector(b""))
        .expect("登记");
    let block_key = block_key_named("node", 0);
    let record = conclusion_record(block_of(&[clean, clean]), &[]);
    library.write_raw_rows(vec![(
        VerdictStoreKey::AbandonedRootsNamedByThePoolChecker(
            &block_key,
            &ConclusionKind::ImageJudgement,
            &JUDGEMENT_CONCLUSION_VERSION,
        )
        .to_key_bytes()
        .0,
        abandoned_roots_named_value_bytes(&abandoned_roots_of_each_of_the_test_states(2)),
    )]);
    let stored = store.store_conclusion_block(
        &block_key,
        &ConclusionKind::ImageJudgement,
        &JUDGEMENT_CONCLUSION_VERSION,
        &record,
    );
    assert!(
        matches!(
            stored,
            Err(VerdictStoreError::StoredBytesDoNotMatchTheLayout {
                table: VerdictStoreTable::AbandonedRootsNamedByThePoolChecker,
                reason: "点名的被抛弃根旁表有这一块这一种结论这一版本，结论块表没有",
                ..
            })
        ),
        "重读仍没有要报被抛弃根旁表孤儿：{stored:?}"
    );
}
