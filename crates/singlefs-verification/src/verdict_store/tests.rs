//! 判定存储的测试：几份测试共用的测试库与造键、造块、造结论的辅助，空块与块键，照判、不入库的行与两路有序流的归并，表标签。

use super::*;
use crate::crash_judge_conclusions::{LandedRootWords, RecoveryWordsOfAPass};

/// 测试用的一次性库：它自己在这个进程里起的临时 prune-server 里的一段名字空间（`crate::crash_library::DisposableLibrary`），
/// 各条用例各一份、各从空库起；临时服务起不来就当场失败，不跳过。
pub(super) struct TestLibrary {
    disposable: crate::crash_library::DisposableLibrary,
}

impl TestLibrary {
    pub(super) fn new(test_name: &str) -> Self {
        // 一次性库留着：存放布局的自证，各条从空库起
        let disposable = crate::crash_library::DisposableLibrary::new(&format!(
            "verdict-library-unit-test-{test_name}"
        ))
        .unwrap_or_else(|refusal| panic!("{refusal}"));
        Self { disposable }
    }

    /// 开这一份（一个新的客户端连同它自己的追加日志）：读写不成交回错误、不退出进程。同一份开几次读到的是同一批行（重开）。
    pub(super) fn open(&self) -> VerdictStore {
        self.disposable
            .open_store(PruneServerFailureHandling::ReturnTheError)
            .unwrap_or_else(|refusal| panic!("开不了一次性库：{refusal}"))
    }

    /// 这一份在服务里那一端：用例直接改行（造坏行、删行）用。
    pub(super) fn service(&self) -> Arc<PruneServerStorage> {
        self.disposable
            .connect(PruneServerFailureHandling::ReturnTheError)
            .unwrap_or_else(|refusal| panic!("连不上临时 prune-server：{refusal}"))
    }

    /// 原样读一行（不经照判不入库），删除记号读成不在。
    pub(super) fn raw_value(&self, key_bytes: &[u8]) -> Option<Vec<u8>> {
        self.service()
            .read_values(&[key_bytes])
            .expect("读得了临时 prune-server")
            .pop()
            .flatten()
    }

    /// 原样写几行（空值是删除记号）。
    pub(super) fn write_raw_rows(&self, rows: Vec<(Vec<u8>, Vec<u8>)>) {
        self.service()
            .write_rows_for_a_test(rows)
            .expect("写得了临时 prune-server");
    }
}

pub(super) const FINGERPRINT_BYTE: u8 = 0x11;
pub(super) const TEST_JUDGING_UNITS_FINGERPRINT: JudgingUnitsFingerprint =
    JudgingUnitsFingerprint([0x33; SHA256_DIGEST_BYTES]);
pub(super) const OTHER_JUDGING_UNITS_FINGERPRINT: JudgingUnitsFingerprint =
    JudgingUnitsFingerprint([0x44; SHA256_DIGEST_BYTES]);
pub(super) const PLAN_HASH_BYTE: u8 = 0x22;

/// 测试里按名字造的段写表与起步镜像摘要：名字只用来造出互不相同的摘要，块键里没有名字。
fn node_input_digest_named(fingerprint_byte: u8, name: &str) -> NodeInputDigest {
    let mut message = vec![fingerprint_byte];
    message.extend_from_slice(name.as_bytes());
    NodeInputDigest(crate::sha256::sha256_digest(&message))
}

pub(super) fn block_key_of(
    fingerprint_byte: u8,
    name: &str,
    plan_hash_byte: u8,
    block_start: u64,
) -> VerdictBlockKey {
    VerdictBlockKey {
        node_input_digest: node_input_digest_named(fingerprint_byte, name),
        enumeration_plan_hash: EnumerationPlanHash([plan_hash_byte; SHA256_DIGEST_BYTES]),
        block_start: BlockStartStateIndex(block_start),
    }
}

pub(super) fn block_key_named(name: &str, block_start: u64) -> VerdictBlockKey {
    block_key_of(FINGERPRINT_BYTE, name, PLAN_HASH_BYTE, block_start)
}

pub(super) fn verdict_vector(bytes: &[u8]) -> EncodedVerdictVector {
    EncodedVerdictVector(bytes.to_vec())
}

pub(super) fn block_of(numbers: &[VerdictVectorNumber]) -> VerdictBlock {
    VerdictBlock::new(numbers.to_vec()).expect("测试里的块至少一个状态")
}

#[test]
fn block_without_any_state_is_refused() {
    assert_eq!(
        VerdictBlock::new(Vec::new()),
        Err(EmptyVerdictBlockRejected)
    );
}

/// 块键定长、只由内容定：字节读回逐字段相同，后缀原样交回；不够长的字节认不回。
#[test]
fn a_block_key_is_fixed_length_content_and_reads_back_from_its_bytes() {
    let key = block_key_of(0x11, "probe", 0x22, 70_000);
    let mut bytes = key.to_block_key_bytes();
    assert_eq!(bytes.len(), BLOCK_KEY_BYTES, "块键定长 72 字节");
    bytes.extend_from_slice(b"suffix");
    assert_eq!(
        VerdictBlockKey::from_block_key_bytes_with_suffix(&bytes),
        Some((key.clone(), &b"suffix"[..]))
    );
    assert_eq!(
        VerdictBlockKey::from_block_key_bytes_with_suffix(&bytes[..BLOCK_KEY_BYTES - 1]),
        None
    );
}

pub(super) const JUDGEMENT_CONCLUSION_VERSION: ConclusionVersion =
    ConclusionVersion([0x51; SHA256_DIGEST_BYTES]);
pub(super) const ANOTHER_JUDGEMENT_CONCLUSION_VERSION: ConclusionVersion =
    ConclusionVersion([0x52; SHA256_DIGEST_BYTES]);
pub(super) const OBSERVER_CONCLUSION_VERSION: ConclusionVersion =
    ConclusionVersion([0x53; SHA256_DIGEST_BYTES]);

pub(super) fn count_vector_of(states: u64) -> BlockCountVector {
    BlockCountVector {
        per_invariant: BTreeMap::from([
            (
                "allocation_record_matches_the_tree".to_string(),
                InvariantStateCounts {
                    judged: states - 1,
                    violated: 1,
                    not_applicable: 1,
                },
            ),
            (
                "root_record_names_a_reachable_tree".to_string(),
                InvariantStateCounts {
                    judged: states,
                    violated: 0,
                    not_applicable: 0,
                },
            ),
        ]),
        per_observer: BTreeMap::from([("mounted_states".to_string(), states)]),
        states_by_publish: BTreeMap::from([
            ("instance2_txg5".to_string(), states - 1),
            ("every_write_persisted".to_string(), 1),
        ]),
    }
}

/// 测试块每个状态的恢复字：看 journal 那一遍落到 checkpoint_txg 高 32 位非 0 的根（随状态变），不看那一遍落到 txg 4 那条根。
pub(super) fn recovery_words_of_the_test_block(states: u32) -> Vec<RecoveryWordsOfState> {
    (0..states)
        .map(|state| {
            RecoveryWordsOfState::of_the_two_passes(
                RecoveryWordsOfAPass {
                    landed_root: LandedRootWords::of_the_landed_root(
                        InstanceGeneration(2),
                        CheckpointTxg((u64::from(state + 1) << 32) | u64::from(5 + state)),
                    ),
                    outcome: 1,
                    failure_point: 0,
                },
                RecoveryWordsOfAPass {
                    landed_root: LandedRootWords::of_the_landed_root(
                        InstanceGeneration(2),
                        CheckpointTxg(4),
                    ),
                    outcome: 1,
                    failure_point: 100 + state,
                },
            )
        })
        .collect()
}

pub(super) fn conclusion_record(
    block: VerdictBlock,
    violating_states: &[u64],
) -> ConclusionBlockRecord {
    let states =
        u32::try_from(block.verdict_vector_number_of_each_state().len()).expect("测试里的块不长");
    ConclusionBlockRecord {
        block,
        violating_states: violating_states
            .iter()
            .copied()
            .map(StateOffsetInBlock)
            .collect(),
        count_vector: count_vector_of(u64::from(states)),
        judged_on: JudgingDevice::Gpu {
            card_type: "probe card".to_string(),
            driver_version: "probe driver 1".to_string(),
        },
        recovery_words: Some(recovery_words_of_the_test_block(states)),
    }
}

/// 4-2a 报告第四节第 7 条后半句：登记成照判不入库的版本，键里带着它的行（结论块连同违例、计数向量、块属性）只留在这个进程里——
/// 这一份库照读得回、列得出，关库重开就没了；同一块别的版本的结论照常写进库、重开读得回。登记之前写进库的行不动。
/// 判别力（`crates/mutations.tsv`）：照判不入库的行照旧写进 RocksDB（重开还读得回），这条红。
#[test]
fn rows_of_a_version_kept_out_of_the_library_are_read_in_this_process_and_never_written() {
    let library = TestLibrary::new("kept-out-versions");
    let block_key = block_key_named("kept-out", 0);
    let kept_out_version = ConclusionVersion([0x61; SHA256_DIGEST_BYTES]);
    {
        let mut store = library.open();
        let clean = store
            .register_verdict_vector(&verdict_vector(b""))
            .expect("登记");
        let red = store
            .register_verdict_vector(&verdict_vector(b"red"))
            .expect("登记");
        store.keep_the_rows_of_these_versions_out_of_the_library([kept_out_version.0]);
        assert_eq!(
            store.versions_kept_out_of_the_library(),
            vec![kept_out_version.0]
        );
        let kept_out_record = conclusion_record(block_of(&[clean, red]), &[1]);
        store
            .store_conclusion_block(
                &block_key,
                &ConclusionKind::ImageJudgement,
                &kept_out_version,
                &kept_out_record,
            )
            .expect("照判不入库的那一把照样存得进这个进程");
        store
            .store_conclusion_block(
                &block_key,
                &ConclusionKind::ImageJudgement,
                &ANOTHER_JUDGEMENT_CONCLUSION_VERSION,
                &conclusion_record(block_of(&[clean, clean]), &[]),
            )
            .expect("别的版本照常存进库");
        assert_eq!(
            store
                .read_conclusion_block(
                    &block_key,
                    &ConclusionKind::ImageJudgement,
                    &kept_out_version
                )
                .expect("读")
                .map(|stored| (stored.block, stored.violations.len())),
            Some((block_of(&[clean, red]), 1)),
            "这个进程里读得回照判不入库的那一块，连同违例"
        );
        assert_eq!(
            store.stored_conclusion_keys().expect("列").len(),
            2,
            "这个进程里列得出两把版本的结论"
        );
    }
    let reopened = library.open();
    assert_eq!(
        reopened
            .read_conclusion_block(
                &block_key,
                &ConclusionKind::ImageJudgement,
                &kept_out_version
            )
            .expect("读"),
        None,
        "照判不入库的那一块没进 RocksDB：重开就没了"
    );
    assert!(
        reopened
            .read_conclusion_block(
                &block_key,
                &ConclusionKind::ImageJudgement,
                &ANOTHER_JUDGEMENT_CONCLUSION_VERSION
            )
            .expect("读")
            .is_some(),
        "别的版本的结论在库里"
    );
    assert_eq!(
        reopened.stored_conclusion_keys().expect("列"),
        vec![(
            block_key,
            ConclusionKind::ImageJudgement,
            ANOTHER_JUDGEMENT_CONCLUSION_VERSION
        )],
        "库里只有入库的那一把"
    );
}

/// crates 第 1 条：照判不入库只比键里版本那一截（按表标签定的偏移）。版本那一截是那一把的行，十四种键各自不入库；那一把只出现在
/// 键里别处（块键里的两个摘要、判定向量的字节、录下的流程的摘要、录入键、对拍的输入摘要、镜像去重键、分片进度的段摘要）的行照入库——
/// 在整条键上按 32 字节滑着找的话，后一半全判成不入库。
/// 判别力（`crates/mutations.tsv`）：结论那几张表的版本起点错一个字节，「结论块」那一行判成入库，这条红。
#[test]
fn a_kept_out_version_is_matched_only_at_the_version_field_of_each_table() {
    let kept_out_version: [u8; SHA256_DIGEST_BYTES] =
        std::array::from_fn(|position| 0xA0 + u8::try_from(position).expect("32 个位置装得进 u8"));
    let kept_out = RowsKeptOutOfTheLibrary {
        versions: BTreeSet::from([kept_out_version]),
        rows: BTreeMap::new(),
    };
    let block_key = block_key_named("kept-out-offsets", 7);
    let fingerprint = JudgingUnitsFingerprint(kept_out_version);
    let version = ConclusionVersion(kept_out_version);
    let other_digest = [0x5C; SHA256_DIGEST_BYTES];
    let progress_under_the_fingerprint = SlicedSegmentProgressKey {
        segment_content_digest: other_digest,
        judge: JudgeOfTheProgress::TheJudgingUnitsFingerprint(fingerprint),
    };
    let progress_under_the_conclusion_kind = SlicedSegmentProgressKey {
        segment_content_digest: other_digest,
        judge: JudgeOfTheProgress::TheConclusionKind {
            kind: ConclusionKind::ImageJudgement,
            version,
        },
    };
    let carrying_it_in_the_version_field = [
        (
            "判定块",
            VerdictStoreKey::VerdictBlock(&block_key, &fingerprint),
        ),
        (
            "违例",
            VerdictStoreKey::Violation(&block_key, &fingerprint, StateOffsetInBlock(3)),
        ),
        (
            "核对表",
            VerdictStoreKey::VerifierBlock(&block_key, &kept_out_version),
        ),
        (
            "组成清单",
            VerdictStoreKey::JudgingUnitsComposition(&kept_out_version),
        ),
        (
            "结论块",
            VerdictStoreKey::ConclusionBlock(&block_key, &ConclusionKind::ImageJudgement, &version),
        ),
        (
            "结论违例",
            VerdictStoreKey::ConclusionViolation(
                &block_key,
                &ConclusionKind::ImageJudgement,
                &version,
                StateOffsetInBlock(3),
            ),
        ),
        (
            "恢复字",
            VerdictStoreKey::RecoveryWords(&block_key, &ConclusionKind::ImageJudgement, &version),
        ),
        (
            "点名的被抛弃根",
            VerdictStoreKey::AbandonedRootsNamedByThePoolChecker(
                &block_key,
                &ConclusionKind::ImageJudgement,
                &version,
            ),
        ),
        (
            "计数向量",
            VerdictStoreKey::ConclusionCountVector(
                &block_key,
                &ConclusionKind::ImageJudgement,
                &version,
            ),
        ),
        (
            "块属性",
            VerdictStoreKey::ConclusionBlockAttributes(
                &block_key,
                &ConclusionKind::ImageJudgement,
                &version,
            ),
        ),
        (
            "观察者录入块",
            VerdictStoreKey::ObserverBlock(&block_key, &ConclusionKind::Observer, &version),
        ),
        (
            "镜像上的池级 checker",
            VerdictStoreKey::ObservedImageJudgement(
                &other_digest,
                &ConclusionKind::ImageJudgement,
                &version,
            ),
        ),
        (
            "分片进度（判法单元指纹那一套）",
            VerdictStoreKey::SlicedSegmentProgress(&progress_under_the_fingerprint),
        ),
        (
            "分片进度（结论种类那一套）",
            VerdictStoreKey::SlicedSegmentProgress(&progress_under_the_conclusion_kind),
        ),
    ];
    for (table, key) in &carrying_it_in_the_version_field {
        assert!(
            kept_out.keeps_out(&key.to_key_bytes().0),
            "{table}：版本那一截是照判不入库的那一把，这一行不入库"
        );
    }
    let block_key_made_of_it = VerdictBlockKey {
        node_input_digest: NodeInputDigest(kept_out_version),
        enumeration_plan_hash: EnumerationPlanHash(kept_out_version),
        block_start: BlockStartStateIndex(7),
    };
    let verdict_vector_holding_it = EncodedVerdictVector(kept_out_version.to_vec());
    let recording_key_equal_to_it = RecordingKey(kept_out_version);
    let pairing_of_inputs_equal_to_it = PairingConclusionKey {
        inputs_digest: kept_out_version,
    };
    let progress_of_a_segment_digest_equal_to_it = SlicedSegmentProgressKey {
        segment_content_digest: kept_out_version,
        judge: JudgeOfTheProgress::TheJudgingUnitsFingerprint(TEST_JUDGING_UNITS_FINGERPRINT),
    };
    let carrying_it_elsewhere = [
        (
            "判定块的块键",
            VerdictStoreKey::VerdictBlock(&block_key_made_of_it, &TEST_JUDGING_UNITS_FINGERPRINT),
        ),
        (
            "录入表的块键",
            VerdictStoreKey::RecordedBlock(&block_key_made_of_it),
        ),
        (
            "结论块的块键",
            VerdictStoreKey::ConclusionBlock(
                &block_key_made_of_it,
                &ConclusionKind::ImageJudgement,
                &ANOTHER_JUDGEMENT_CONCLUSION_VERSION,
            ),
        ),
        (
            "判定向量的字节",
            VerdictStoreKey::VerdictVectorNumberByContent(&verdict_vector_holding_it),
        ),
        (
            "录下的流程的摘要",
            VerdictStoreKey::RecordedFlow(&kept_out_version),
        ),
        (
            "录入键",
            VerdictStoreKey::RecordedRunManifest(&recording_key_equal_to_it),
        ),
        (
            "对拍的输入摘要",
            VerdictStoreKey::PairingConclusion(&pairing_of_inputs_equal_to_it),
        ),
        (
            "镜像去重键",
            VerdictStoreKey::ObservedImageJudgement(
                &kept_out_version,
                &ConclusionKind::ImageJudgement,
                &ANOTHER_JUDGEMENT_CONCLUSION_VERSION,
            ),
        ),
        (
            "分片进度的段摘要",
            VerdictStoreKey::SlicedSegmentProgress(&progress_of_a_segment_digest_equal_to_it),
        ),
    ];
    for (table, key) in &carrying_it_elsewhere {
        assert!(
            !kept_out.keeps_out(&key.to_key_bytes().0),
            "{table}：那一把只出现在键里别处、不在版本那一截，这一行照入库"
        );
    }
}

/// 表标签读回它那张表（[`VerdictStoreTable::of_key_tag`] 是 [`VerdictStoreTable::key_tag`] 的逆，认得的标签正好 33 个），
/// 结论种类每一种在键里都是 [`CONCLUSION_KIND_KEY_BYTES`] 个字节：照判不入库按这两样定版本那一截的起点。
#[test]
fn every_table_tag_reads_back_to_its_table_and_every_conclusion_kind_takes_its_key_bytes() {
    let mut tables_read_back = 0;
    for key_tag in 0..=u8::MAX {
        if let Some(table) = VerdictStoreTable::of_key_tag(key_tag) {
            assert_eq!(table.key_tag(), key_tag, "{table:?} 的标签读回它自己");
            tables_read_back += 1;
        }
    }
    assert_eq!(tables_read_back, 33, "33 张表各一个标签");
    for kind in [ConclusionKind::ImageJudgement, ConclusionKind::Observer] {
        assert_eq!(
            conclusion_kind_key_bytes(&kind).len(),
            CONCLUSION_KIND_KEY_BYTES,
            "{kind:?} 在键里的字节数"
        );
    }
}

/// 认得的表标签从 `01` 起连着排到最后一张表，中间不留空号；`00` 不是任何一张表的标签。
/// 判别力（`crates/mutations.tsv`）：读集见证行那张表的标签读回时认成 `1D`（`1A` 空出来），这条红。
#[test]
fn the_table_tags_run_from_one_to_the_last_table_without_a_gap() {
    let recognized_tags: Vec<u8> = (0..=u8::MAX)
        .filter(|key_tag| VerdictStoreTable::of_key_tag(*key_tag).is_some())
        .collect();
    let expected_tags: Vec<u8> = (0x01..=0x21).collect();
    assert_eq!(
        recognized_tags, expected_tags,
        "认得的表标签是 01 到 21 连着的 33 个"
    );
}

/// 观察者那一种结论在键里的字节是 `02`（镜像判定是 `01`，两种连着排）。
/// 判别力（`crates/mutations.tsv`）：观察者的键字节写成 `03`，这条红。
#[test]
fn the_observer_conclusion_kind_takes_key_byte_two() {
    assert_eq!(
        conclusion_kind_key_bytes(&ConclusionKind::Observer),
        vec![0x02],
        "观察者那一种结论的键字节"
    );
}

/// 一路假的库行：键是 `2 × 序号` 的四字节大端，值是 `library`；每交出一行记一次数（不整段缓冲的判据就是被取走的行数）。
struct LibraryRowsCountingTheRowsTaken<'count> {
    ordinals: std::ops::Range<u32>,
    rows_taken: &'count std::cell::Cell<u64>,
}

impl Iterator for LibraryRowsCountingTheRowsTaken<'_> {
    type Item = Result<StoredEntry, VerdictStoreError>;

    fn next(&mut self) -> Option<Self::Item> {
        let ordinal = self.ordinals.next()?;
        self.rows_taken.set(self.rows_taken.get() + 1);
        Some(Ok((
            (2 * ordinal).to_be_bytes().to_vec().into_boxed_slice(),
            b"library".to_vec().into_boxed_slice(),
        )))
    }
}

/// 测试里的「服务那一路读错」：服务不可用的退出码连同一句说明。
fn a_library_read_error() -> PruneServerFailure {
    PruneServerFailure {
        exit_code: PRUNE_SERVER_UNAVAILABLE_EXIT_CODE,
        description: "前缀扫不成：测试里造的读错".to_string(),
    }
}

fn kept_out_row(key_number: u32) -> (Vec<u8>, Vec<u8>) {
    (key_number.to_be_bytes().to_vec(), b"kept out".to_vec())
}

/// crates 第 2 条：前缀下的行两路有序流逐行归并，库那一路十万行里只取到交出去的那几行再多看一行，不整段读进来；交出的次序与值：
/// 两路交替按键升序，同一个键（4）交照判不入库那一行。
/// 判别力（`crates/mutations.tsv`）：库那一路先整段收进内存再归并，被取走的行数是十万，这条红。
#[test]
fn merging_the_library_rows_with_the_kept_out_rows_takes_one_row_at_a_time() {
    let rows_taken = std::cell::Cell::new(0);
    let library_rows = LibraryRowsCountingTheRowsTaken {
        ordinals: 0..100_000,
        rows_taken: &rows_taken,
    };
    let kept_out_rows = [1, 3, 4, 5].into_iter().map(kept_out_row);
    let first_five: Vec<(Vec<u8>, Vec<u8>)> =
        RowsOfTwoSourcesMergedByKey::new(library_rows, kept_out_rows)
            .take(5)
            .map(|row| {
                let (key_bytes, value_bytes) = row.expect("假的库行不报错");
                (key_bytes.into_vec(), value_bytes.into_vec())
            })
            .collect();
    assert_eq!(
        first_five,
        vec![
            (0u32.to_be_bytes().to_vec(), b"library".to_vec()),
            kept_out_row(1),
            (2u32.to_be_bytes().to_vec(), b"library".to_vec()),
            kept_out_row(3),
            kept_out_row(4),
        ],
        "两路按键升序交替，同一个键交照判不入库那一行"
    );
    assert!(
        rows_taken.get() <= 4,
        "交出五行，库那一路只取到交出去的几行与往前看的一行（十万行不整段读进来）：取了 {}",
        rows_taken.get()
    );
}

/// 对照用的合并（库里那一段整段读进一张有序表，再用照判不入库那一段逐行盖）：只留着给对照，交出的次序与值是归并要逐条对得上的。
fn rows_merged_as_before(
    library_rows: &[(Vec<u8>, Vec<u8>)],
    kept_out_rows: &[(Vec<u8>, Vec<u8>)],
) -> Vec<(Vec<u8>, Vec<u8>)> {
    let mut merged: BTreeMap<Vec<u8>, Vec<u8>> = library_rows.iter().cloned().collect();
    merged.extend(kept_out_rows.iter().cloned());
    merged.into_iter().collect()
}

/// crates 第 2 条：归并交出的行与对照用的合并逐条相同——两路交错、同一个键两路都有（交照判不入库那一行）、一路空着、两路都空。
/// 判别力（`crates/mutations.tsv`）：同一个键两路都有时交库里那一行，「c」「f」两行的值对不上，这条红。
#[test]
fn merging_gives_the_rows_and_the_order_of_the_former_whole_prefix_merge() {
    let row = |key: &str, value: &str| (key.as_bytes().to_vec(), value.as_bytes().to_vec());
    let library_rows = vec![
        row("a", "library a"),
        row("c", "library c"),
        row("d", "library d"),
        row("f", "library f"),
    ];
    let kept_out_rows = vec![
        row("b", "kept out b"),
        row("c", "kept out c"),
        row("e", "kept out e"),
        row("f", "kept out f"),
        row("g", "kept out g"),
    ];
    for (case, library_side, kept_out_side) in [
        (
            "两路交错、同键两处",
            library_rows.clone(),
            kept_out_rows.clone(),
        ),
        ("照判不入库那一路空着", library_rows.clone(), Vec::new()),
        ("库那一路空着", Vec::new(), kept_out_rows.clone()),
        ("两路都空", Vec::new(), Vec::new()),
    ] {
        let merged: Vec<(Vec<u8>, Vec<u8>)> = RowsOfTwoSourcesMergedByKey::new(
            library_side
                .clone()
                .into_iter()
                .map(|(key_bytes, value_bytes)| {
                    Ok::<StoredEntry, VerdictStoreError>((
                        key_bytes.into_boxed_slice(),
                        value_bytes.into_boxed_slice(),
                    ))
                }),
            kept_out_side.clone().into_iter(),
        )
        .map(|merged_row| {
            let (key_bytes, value_bytes) = merged_row.expect("没有读错");
            (key_bytes.into_vec(), value_bytes.into_vec())
        })
        .collect();
        assert_eq!(
            merged,
            rows_merged_as_before(&library_side, &kept_out_side),
            "{case}：与改前的合并逐条相同"
        );
    }
}

/// 库那一路读错：错误照它出现的位置交出，交出之后归并到此为止（照判不入库那一路还剩的行不再交，调用方见错即停）。
#[test]
fn a_library_read_error_is_handed_over_where_it_occurs_and_ends_the_merge() {
    let read_error = a_library_read_error();
    let library_rows = vec![
        Ok((
            b"a".to_vec().into_boxed_slice(),
            b"library a".to_vec().into_boxed_slice(),
        )),
        Err(VerdictStoreError::PruneServer(read_error.clone())),
        Ok((
            b"z".to_vec().into_boxed_slice(),
            b"library z".to_vec().into_boxed_slice(),
        )),
    ];
    let kept_out_rows = vec![(b"b".to_vec(), b"kept out b".to_vec())];
    let merged: Vec<Result<Vec<u8>, String>> =
        RowsOfTwoSourcesMergedByKey::new(library_rows.into_iter(), kept_out_rows.into_iter())
            .map(|merged_row| {
                merged_row
                    .map(|(key_bytes, _value_bytes)| key_bytes.into_vec())
                    .map_err(|error| format!("{error:?}"))
            })
            .collect();
    assert_eq!(
        merged,
        vec![
            Ok(b"a".to_vec()),
            Err(format!("{:?}", VerdictStoreError::PruneServer(read_error)))
        ]
    );
}

/// 照判不入库那一路一次取一行、两次取之间不拿着锁：取了一行之后再往这个进程里写，键落在已取过的键之前的不再取，之后的照样取到，
/// 前缀之外的不取。
#[test]
fn the_kept_out_rows_are_taken_one_at_a_time_without_holding_the_lock_between_takes() {
    let rows = std::sync::Mutex::new(RowsKeptOutOfTheLibrary {
        versions: BTreeSet::new(),
        rows: [
            (b"p1".to_vec(), b"one".to_vec()),
            (b"p2".to_vec(), b"two".to_vec()),
            (b"q1".to_vec(), b"outside".to_vec()),
        ]
        .into_iter()
        .collect(),
    });
    let mut kept_out_rows = KeptOutRowsWithThePrefix {
        rows_kept_out_of_the_library: &rows,
        prefix: b"p".to_vec(),
        last_key_taken: None,
    };
    assert_eq!(
        kept_out_rows.next(),
        Some((b"p1".to_vec(), b"one".to_vec()))
    );
    {
        let mut written_between_the_takes = rows
            .try_lock()
            .expect("两次取之间没人拿着锁（拿着的话这里取不到锁）");
        written_between_the_takes
            .rows
            .insert(b"p0".to_vec(), b"before".to_vec());
        written_between_the_takes
            .rows
            .insert(b"p3".to_vec(), b"after".to_vec());
    }
    assert_eq!(
        kept_out_rows.collect::<Vec<_>>(),
        vec![
            (b"p2".to_vec(), b"two".to_vec()),
            (b"p3".to_vec(), b"after".to_vec()),
        ]
    );
}
