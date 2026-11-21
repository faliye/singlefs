//! 截断：截断中间态与洞（I-4.6、I-4.10、I-9.38、I-4.4），inode 记录的截断字段、I-9.7 拆段、I-9.39 与 incompat 位 2。

use super::*;

// ───────────── 截断那一批：I-4.6 改判、I-4.10、I-9.38、I-4.4 收截断条目、上段叶条目标签 3、4（判决 m4-trunc-r1 第三节 K5、K7，第六节 S1、S2）─────────────
// GPU 恢复内核照 core 读截断中间态与洞（判决 K2、K5 的读者那一半与第六节 S5：size 之外挂着的单元不读、洞读 0），这几条流都整份比：
// 红位、池级 checker、两遍恢复的结局与落到的根（`Comparison::mismatches` 与 `recovery_mismatches`）。

/// 截断与洞那条流走到哪一步停（[`record_new_pool_through_truncations_and_holes`]）。
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(super) enum TruncationsAndHolesFlowStop {
    /// 三个单元的文件（txg 4）：下段根、标签 1，三条下段叶记录偏移 0、P、2P（P 是净荷容量）。
    AfterTheThreeUnitFile,
    /// 缩到一个半净荷那次意图（txg 5）：单元 1 重写成半个净荷，单元 2 在 size 之外挂着、截断条目点名第一个文件。
    AfterTheIntentOfTheFirstTruncation,
    /// 缩完（最后一批放掉单元 2、删截断条目）再变长到三个半净荷（txg 7）：单元 1 补零重写成一整个净荷，之后到 size 是洞，标签 3。
    AfterTheHoleBehindALowerSegment,
    /// 走到底：再缩到一个净荷（意图 + 各批）、变长到两个半净荷（一个单元之后是洞，标签 3 或 4）、缩到 0（意图 + 放 0 个单元的最后一批）。
    AtTheEnd,
}

/// 第一个文件改成 `new_size` 字节那一版读出的内容：`current` 截到 `new_size`，或在它后面补 0 到 `new_size`（变长的洞读 0）。
fn content_after_the_size_change(current: &[u8], new_size: usize) -> Vec<u8> {
    let mut content = current[..new_size.min(current.len())].to_vec();
    content.resize(new_size, 0);
    content
}

/// 截断的各批（每批一个单元）一直发到截断条目删掉为止（最后一批放完剩下的、同一次发布删条目，可以 0 个单元，判决 K2、K11），
/// 每一版的内容都是 `content`（意图那次已把 size 写成目标）。
fn publish_the_truncation_batches_until_the_entry_is_gone(
    pool: &mut BuiltPool,
    versions: &mut Vec<PublishedVersion>,
    content: &[u8],
) {
    const MOST_BATCHES_OF_ONE_ON_THIS_FLOW: usize = 4;
    for _batch_index in 0..MOST_BATCHES_OF_ONE_ON_THIS_FLOW {
        if common_truncation::truncation_entry_of(&pool.output).is_none() {
            return;
        }
        let published = common_truncation::publish_the_next_truncation_step_and_advance(
            pool,
            common_truncation::batches_of_one(),
        );
        versions.push(PublishedVersion {
            instance: published.root.instance,
            checkpoint_txg: published.root.checkpoint_txg,
            content: content.to_vec(),
        });
    }
    assert!(
        common_truncation::truncation_entry_of(&pool.output).is_none(),
        "这条流上每次截断至多放 {MOST_BATCHES_OF_ONE_ON_THIS_FLOW} 批就删掉截断条目"
    );
}

/// 第一个文件改成 `new_size` 字节（截断时只发意图那一次），版本表记下这一版，交回这一版读出的内容。
fn change_the_size_and_note_the_version(
    pool: &mut BuiltPool,
    versions: &mut Vec<PublishedVersion>,
    current: &[u8],
    new_size: usize,
) -> Vec<u8> {
    let published = common_truncation::change_the_size_of_the_first_file_and_advance(
        pool,
        u64::try_from(new_size).expect("装得进 u64"),
    );
    let content = content_after_the_size_change(current, new_size);
    versions.push(PublishedVersion {
        instance: published.root.instance,
        checkpoint_txg: published.root.checkpoint_txg,
        content: content.clone(),
    });
    content
}

/// 截断与洞的流（`common_truncation` 的搭建，批大小 1）：新池新建文件（txg 3）、第一个文件顺序写成三个数据单元（txg 4，两个半净荷）、
/// 缩到一个半净荷（意图 txg 5，最后一批 txg 6）、变长到三个半净荷（txg 7，标签 3）、缩到一个净荷（意图 + 最后一批：洞整个截掉、换标签写在意图那次，
/// S1）、变长到两个半净荷（一个单元之后是洞）、缩到 0（意图把单元 0 重写成声明长度 0，最后一批放 0 个单元）。版本表照每一版读出的内容记。
pub(super) fn record_new_pool_through_truncations_and_holes(
    tag: &str,
    name: &'static str,
    stop: TruncationsAndHolesFlowStop,
) -> RecordedFlow {
    let payload = singlefs_core::unit::data_unit_payload_capacity();
    let mut pool = pool_with_a_three_unit_file(tag);
    let mut versions = versions_with_the_first_file_before_its_deletion();
    let mut content = content_of_three_data_units();
    if stop >= TruncationsAndHolesFlowStop::AfterTheIntentOfTheFirstTruncation {
        content = change_the_size_and_note_the_version(
            &mut pool,
            &mut versions,
            &content,
            payload + payload / 2,
        );
    }
    if stop >= TruncationsAndHolesFlowStop::AfterTheHoleBehindALowerSegment {
        publish_the_truncation_batches_until_the_entry_is_gone(&mut pool, &mut versions, &content);
        content = change_the_size_and_note_the_version(
            &mut pool,
            &mut versions,
            &content,
            3 * payload + payload / 2,
        );
        assert_eq!(
            common_truncation::upper_leaf_tag_of_the_first_file(&pool.output),
            3,
            "两个单元之后到三个半净荷是洞：下段根、标签 3"
        );
    }
    if stop >= TruncationsAndHolesFlowStop::AtTheEnd {
        content = change_the_size_and_note_the_version(&mut pool, &mut versions, &content, payload);
        publish_the_truncation_batches_until_the_entry_is_gone(&mut pool, &mut versions, &content);
        content = change_the_size_and_note_the_version(
            &mut pool,
            &mut versions,
            &content,
            2 * payload + payload / 2,
        );
        assert_eq!(
            common_truncation::upper_leaf_tag_of_the_first_file(&pool.output),
            4,
            "缩到一个净荷之后只剩单元 0、内联（只有一个单元的文件不建下段），变长到两个半净荷之后到 size 是洞：标签 4"
        );
        content = change_the_size_and_note_the_version(&mut pool, &mut versions, &content, 0);
        publish_the_truncation_batches_until_the_entry_is_gone(&mut pool, &mut versions, &content);
    }
    recorded_flow_of_the_pool(&pool, name, versions)
}

/// 截断那几条流上 GPU 判器与 CPU 逐状态整份相同（[`assert_the_gpu_judge_agrees_with_the_cpu_on_every_state_of_a_truncation_flow`]），
/// CPU 池级 checker 在这条流上红 `invariant`。本机没有 NVIDIA 独显适配器时报「本次未跑」返回。
fn assert_the_gpu_judge_agrees_with_the_cpu_on_a_truncation_flow_reddening(
    test_name: &str,
    flow: &RecordedFlow,
    invariant: &'static str,
) {
    let Some(cards) = gpu_card_pool_or_report_not_run(test_name) else {
        return;
    };
    let Some(comparison) = compare_on_every_state(cards, flow, expand_up_to()) else {
        return;
    };
    assert!(
        comparison.cpu_checker_red_invariants.contains(invariant),
        "流 {} 上 CPU 池级 checker 要红 {invariant}（红过的：{:?}）",
        flow.name,
        comparison.cpu_checker_red_invariants
    );
    assert_the_gpu_judge_agrees_with_the_cpu_on_every_state_of_a_truncation_flow(flow, &comparison);
}

/// 截断那几条流上每个状态整份相同：对拍字、红位、池级 checker 违例位逐位相同、没有判不了的（[`Comparison::mismatches`] 是空的），
/// 两遍恢复的结局与看 journal 那一遍落到的根相同（[`Comparison::recovery_mismatches`] 是 0；它也算进 `mismatches`，单列一句是为了红的时候
/// 一眼看出是恢复内核那一半），池级 checker 那一半另核一遍（[`Comparison::checker_mismatches`] 是空的）。
fn assert_the_gpu_judge_agrees_with_the_cpu_on_every_state_of_a_truncation_flow(
    flow: &RecordedFlow,
    comparison: &Comparison,
) {
    let first_lines = |lines: &[String]| {
        lines
            .iter()
            .filter(|line| !line.is_empty())
            .cloned()
            .collect::<Vec<_>>()
            .join("\n")
    };
    assert_eq!(
        comparison.recovery_mismatches,
        0,
        "流 {}：{} 个状态 GPU 与 CPU 两遍恢复的结局或看 journal 那一遍落到的根不同（不一致的前几个）：\n{}",
        flow.name,
        comparison.recovery_mismatches,
        first_lines(&comparison.mismatches)
    );
    assert!(
        comparison.checker_mismatches.is_empty(),
        "流 {}：{} 个状态 GPU 与 CPU 的池级 checker 不一致（前几个）：\n{}",
        flow.name,
        comparison.checker_mismatches.state_count(),
        first_lines(&comparison.checker_mismatches)
    );
    assert!(
        comparison.mismatches.is_empty(),
        "流 {}：{} 个状态 GPU 与 CPU 不一致（前几个）：\n{}",
        flow.name,
        comparison.mismatches.state_count(),
        first_lines(&comparison.mismatches)
    );
}

/// 截断与洞那条合法流走到底：每个状态 GPU 与 CPU 整份相同（两遍恢复读截断中间态与洞、池级 checker）；CPU 池级 checker 在这条流上判过 I-4.6、
/// I-4.10、I-9.38、I-4.4 收的截断条目与 I-9.39（截断字段之外的活单元不早于它，每条记录判一次，截断字段都是 0 的记录照判成立）而成立、一个状态都不红。GPU 两个池级 checker 内核都认标签 3、4（不认的话走读内核红 I-1.1、扫描内核读不全
/// extent 树，判到位对不上）；恢复内核认标签 3、4、单元数按标签对 size、size 之外挂着的单元不读、洞按全零比（不照的话截断中间态与有洞的那几版
/// 两遍恢复报走读失败或内容不同，CPU 读出文件）。
#[test]
fn the_gpu_judge_agrees_with_the_cpu_on_every_state_of_truncations_and_holes() {
    const TEST_NAME: &str =
        "the_gpu_judge_agrees_with_the_cpu_on_every_state_of_truncations_and_holes";
    let flow = record_new_pool_through_truncations_and_holes(
        "crash-judge-gpu-truncations-and-holes",
        "truncations-and-holes",
        TruncationsAndHolesFlowStop::AtTheEnd,
    );
    let Some(cards) = gpu_card_pool_or_report_not_run(TEST_NAME) else {
        return;
    };
    let Some(comparison) = compare_on_every_state(cards, &flow, expand_up_to()) else {
        return;
    };
    assert_the_gpu_judge_agrees_with_the_cpu_on_every_state_of_a_truncation_flow(
        &flow,
        &comparison,
    );
    for invariant in ["I-4.6", "I-4.10", "I-9.38", "I-4.4", "I-9.39"] {
        assert!(
            comparison.cpu_checker_held_invariants.contains(invariant),
            "截断与洞的流上 CPU 池级 checker 判过 {invariant} 而成立（成立过的：{:?}）",
            comparison.cpu_checker_held_invariants
        );
        assert!(
            !comparison.cpu_checker_red_invariants.contains(invariant),
            "截断与洞的流上 CPU 池级 checker 不红 {invariant}（红过的：{:?}）",
            comparison.cpu_checker_red_invariants
        );
    }
}

/// 截断条目 value 里 inode 号、对象出生代、目标 size 之后的补零（判决 K1：value 34 = 8 + 8 + 8 + 补零 10）的偏移与对象出生代的偏移。
const TRUNCATION_INTENT_OBJECT_BIRTH_OFFSET_IN_THE_VALUE: usize = 8;

const TRUNCATION_INTENT_PADDING_OFFSET_IN_THE_VALUE: usize = 24;

/// logged_ops 条目的种类码：2 是截断（判决 K1）。
const LOGGED_OPERATION_KIND_TRUNCATION: u16 = 2;

/// 缩到一个半净荷那次意图之后的流上，改坏 logged_ops 根（码 2、key 宽 10、条目宽 44）最后写的那一份（两份副本）里第一条条目——那条截断条目，重封到根。
fn record_the_first_truncation_intent_with_its_logged_operation_changed(
    tag: &str,
    name: &'static str,
    mutate_the_truncation_entry: &dyn Fn(&mut [u8]),
) -> RecordedFlow {
    let mut flow = record_new_pool_through_truncations_and_holes(
        tag,
        name,
        TruncationsAndHolesFlowStop::AfterTheIntentOfTheFirstTruncation,
    );
    let logged_operations_root = |bytes: &[u8]| {
        unit_class(bytes) == 2
            && index_node_key_width(bytes) == 10
            && index_node_entry_width(bytes) == 44
    };
    let targets = last_unit_copies(
        &flow,
        "截断意图那次写的 logged_ops 根",
        &logged_operations_root,
        false,
    );
    corrupt_units_and_reseal_up_to_the_root(&mut flow, &targets, &|bytes| {
        let entry = first_index_node_entry_offset(bytes);
        assert_eq!(
            read_u16_at(bytes, entry),
            LOGGED_OPERATION_KIND_TRUNCATION,
            "logged_ops 根里第一条是截断条目"
        );
        mutate_the_truncation_entry(&mut bytes[entry..entry + 44]);
    });
    flow
}

/// I-4.4（意图只描述目标态）：截断条目补零的第一个字节写 1（「再截 N 个」这样的增量只能落在补零里），
/// 每个状态 GPU 与 CPU 整份相同（两遍恢复与池级 checker）、CPU 红 I-4.4。
#[test]
fn the_gpu_judge_agrees_with_the_cpu_when_a_truncation_intent_carries_an_increment_in_its_padding()
{
    const TEST_NAME: &str =
        "the_gpu_judge_agrees_with_the_cpu_when_a_truncation_intent_carries_an_increment_in_its_padding";
    let flow = record_the_first_truncation_intent_with_its_logged_operation_changed(
        "crash-judge-gpu-truncation-intent-padding",
        "truncation-intent-with-an-increment-in-its-padding",
        &|entry| {
            entry[LOGGED_OPERATION_VALUE_OFFSET_IN_THE_ENTRY
                + TRUNCATION_INTENT_PADDING_OFFSET_IN_THE_VALUE] = 1;
        },
    );
    assert_the_gpu_judge_agrees_with_the_cpu_on_a_truncation_flow_reddening(
        TEST_NAME, &flow, "I-4.4",
    );
}

/// I-4.10（截断中间态有账，判决 K7 T-2）：截断条目的对象出生代加一，它不再对得上第一个文件的记录，size 之外挂着的单元 2 没有账
/// （I-4.4 与 I-4.6 跟着红），每个状态 GPU 与 CPU 整份相同（两遍恢复与池级 checker）、CPU 红 I-4.10。
#[test]
fn the_gpu_judge_agrees_with_the_cpu_when_the_truncation_intent_names_another_object_birth() {
    const TEST_NAME: &str =
        "the_gpu_judge_agrees_with_the_cpu_when_the_truncation_intent_names_another_object_birth";
    let flow = record_the_first_truncation_intent_with_its_logged_operation_changed(
        "crash-judge-gpu-truncation-intent-object-birth",
        "truncation-intent-naming-another-object-birth",
        &|entry| {
            let at = LOGGED_OPERATION_VALUE_OFFSET_IN_THE_ENTRY
                + TRUNCATION_INTENT_OBJECT_BIRTH_OFFSET_IN_THE_VALUE;
            let object_birth = read_u64_at(entry, at);
            entry[at..at + 8].copy_from_slice(&(object_birth + 1).to_le_bytes());
        },
    );
    assert_the_gpu_judge_agrees_with_the_cpu_on_a_truncation_flow_reddening(
        TEST_NAME, &flow, "I-4.10",
    );
}

/// I-9.38（数据单元声明长度对 size，判决 K7 T-4）：缩到一个半净荷那次意图重写的尾单元（单元 1，声明长度半个净荷）声明长度多一（多出来的那个字节是补零，
/// 单元照样解得开），每个状态 GPU 与 CPU 整份相同（落到意图那一版时两遍恢复都在这个单元上判声明长度不符、走读失败；池级 checker）、CPU 红 I-9.38。
#[test]
fn the_gpu_judge_agrees_with_the_cpu_when_a_data_unit_within_the_size_declares_one_byte_more() {
    const TEST_NAME: &str =
        "the_gpu_judge_agrees_with_the_cpu_when_a_data_unit_within_the_size_declares_one_byte_more";
    let mut flow = record_new_pool_through_truncations_and_holes(
        "crash-judge-gpu-truncation-tail-declared-length",
        "truncation-tail-declaring-one-byte-more",
        TruncationsAndHolesFlowStop::AfterTheIntentOfTheFirstTruncation,
    );
    let data_unit = |bytes: &[u8]| unit_class(bytes) == 1;
    let targets = last_unit_copies(&flow, "截断意图重写的尾单元", &data_unit, false);
    corrupt_units_and_reseal_up_to_the_root(&mut flow, &targets, &|bytes| {
        let payload =
            u16::try_from(singlefs_core::unit::data_unit_payload_capacity()).expect("32634");
        let declared_length = read_u16_at(bytes, UNIT_DECLARED_LENGTH_OFFSET);
        assert_eq!(declared_length, payload / 2, "尾单元的声明长度是半个净荷");
        bytes[UNIT_DECLARED_LENGTH_OFFSET..UNIT_DECLARED_LENGTH_OFFSET + 2]
            .copy_from_slice(&(declared_length + 1).to_le_bytes());
    });
    assert_the_gpu_judge_agrees_with_the_cpu_on_a_truncation_flow_reddening(
        TEST_NAME, &flow, "I-9.38",
    );
}

/// I-4.6 ①（存在的单元从 0 起连号，判决 K7 T-3）：三个单元的文件那一版下段叶里偏移 2P 那条记录的 key 偏移改成 3P（单元 2 跳过一个净荷），
/// 每个状态 GPU 与 CPU 整份相同（落到三个单元那一版时两遍恢复都判单元不连号、走读失败；池级 checker）、CPU 红 I-4.6。
#[test]
fn the_gpu_judge_agrees_with_the_cpu_when_the_data_units_of_a_file_skip_an_offset() {
    const TEST_NAME: &str =
        "the_gpu_judge_agrees_with_the_cpu_when_the_data_units_of_a_file_skip_an_offset";
    let mut flow = record_new_pool_through_truncations_and_holes(
        "crash-judge-gpu-data-units-skipping-an-offset",
        "data-units-skipping-an-offset",
        TruncationsAndHolesFlowStop::AfterTheThreeUnitFile,
    );
    let extent_lower_leaf = |bytes: &[u8]| {
        unit_class(bytes) == 2
            && usize::from(index_node_key_width(bytes)) == EXTENT_KEY_BYTES
            && index_node_entry_width(bytes) == EXTENT_LOWER_LEAF_RECORD_BYTES
    };
    let targets = last_unit_copies(&flow, "三个单元那一版的下段叶", &extent_lower_leaf, false);
    corrupt_units_and_reseal_up_to_the_root(&mut flow, &targets, &|bytes| {
        let payload =
            u64::try_from(singlefs_core::unit::data_unit_payload_capacity()).expect("32634");
        let records_start = index_node_entries_start(EXTENT_KEY_BYTES);
        let record_count = usize::from(read_u16_at(
            bytes,
            index_node_entry_count_offset(EXTENT_KEY_BYTES),
        ));
        let record_bytes = usize::from(EXTENT_LOWER_LEAF_RECORD_BYTES);
        let offset_field = (0..record_count)
            .map(|record_index| {
                records_start + record_index * record_bytes + EXTENT_KEY_OFFSET_OF_THE_FILE_OFFSET
            })
            .find(|offset_field| read_u64_at(bytes, *offset_field) == 2 * payload)
            .expect("下段叶里有偏移 2P 那条记录");
        bytes[offset_field..offset_field + 8].copy_from_slice(&(3 * payload).to_le_bytes());
    });
    assert_the_gpu_judge_agrees_with_the_cpu_on_a_truncation_flow_reddening(
        TEST_NAME, &flow, "I-4.6",
    );
}

/// extent key 里文件偏移那一段（locality 8、inode 8 之后）。
const EXTENT_KEY_OFFSET_OF_THE_FILE_OFFSET: usize = 16;

/// 上段叶条目标签 3、1（判决 K5：3 是下段根、之后到 size 是洞；1 是下段根、之后没有洞）。
const EXTENT_UPPER_LEAF_TAG_LOWER_SEGMENT_ROOT_FOLLOWED_BY_A_HOLE: u8 = 3;

/// I-4.6 ②（单元数对标签，判决 K7 T-3 与第六节 S1）：变长出洞那一版上段叶里第一个文件那一条的标签 3 改写成 1——两个单元、size 要四个、没有截断条目，
/// 标签 1 要不少于四个，每个状态 GPU 与 CPU 整份相同（落到变长出洞那一版时两遍恢复都判单元数对不上 size、走读失败；池级 checker）、CPU 红 I-4.6。
#[test]
fn the_gpu_judge_agrees_with_the_cpu_when_a_hole_tag_is_rewritten_as_no_hole() {
    const TEST_NAME: &str =
        "the_gpu_judge_agrees_with_the_cpu_when_a_hole_tag_is_rewritten_as_no_hole";
    let mut flow = record_new_pool_through_truncations_and_holes(
        "crash-judge-gpu-hole-tag-rewritten-as-no-hole",
        "hole-tag-rewritten-as-no-hole",
        TruncationsAndHolesFlowStop::AfterTheHoleBehindALowerSegment,
    );
    let targets = last_unit_copies(
        &flow,
        "变长出洞那一版的上段叶",
        &is_an_extent_upper_leaf,
        false,
    );
    corrupt_units_and_reseal_up_to_the_root(&mut flow, &targets, &|bytes| {
        let tag = pointer_of_the_first_extent_upper_leaf_entry(
            bytes,
            EXTENT_UPPER_LEAF_TAG_LOWER_SEGMENT_ROOT_FOLLOWED_BY_A_HOLE,
        ) - 1;
        bytes[tag] = EXTENT_UPPER_LEAF_TAG_LOWER_SEGMENT_ROOT;
    });
    assert_the_gpu_judge_agrees_with_the_cpu_on_a_truncation_flow_reddening(
        TEST_NAME, &flow, "I-4.6",
    );
}

/// 洞的内容按期望那一版逐单元比（判决 m4-trunc-r1 第六节 S5：内核读到洞时现算全零的载荷区 CRC，与判表从版本表给的期望比）：变长出洞那一版
/// （两个单元、size 三个半净荷、标签 3）的期望内容在洞里（单元 2 那一段的中间）塞一个非零字节，盘上一个字都不改。读出的洞是 0，落到这一版的状态上
/// CPU 看 journal 那一遍红「内容不同」；每个状态 GPU 与 CPU 整份相同——GPU 不比洞（或比洞时不照期望那一版）的话，这些状态上 GPU 判内容相同、对拍红。
#[test]
fn the_gpu_judge_agrees_with_the_cpu_when_the_expected_content_has_a_nonzero_byte_in_the_hole() {
    const TEST_NAME: &str =
        "the_gpu_judge_agrees_with_the_cpu_when_the_expected_content_has_a_nonzero_byte_in_the_hole";
    let payload = singlefs_core::unit::data_unit_payload_capacity();
    let mut flow = record_new_pool_through_truncations_and_holes(
        "crash-judge-gpu-nonzero-byte-in-the-hole",
        "nonzero-byte-in-the-hole-of-the-expected-content",
        TruncationsAndHolesFlowStop::AfterTheHoleBehindALowerSegment,
    );
    let version_with_the_hole = flow
        .versions
        .last_mut()
        .expect("变长出洞那一版是版本表里最后一版");
    assert_eq!(
        version_with_the_hole.content.len(),
        3 * payload + payload / 2,
        "变长出洞那一版的期望内容是三个半净荷"
    );
    let byte_in_the_hole = 2 * payload + payload / 2;
    assert_eq!(
        version_with_the_hole.content[byte_in_the_hole], 0,
        "洞里的期望内容是 0"
    );
    version_with_the_hole.content[byte_in_the_hole] = 1;
    let Some(cards) = gpu_card_pool_or_report_not_run(TEST_NAME) else {
        return;
    };
    let wrong_content = consulted_oracle_item(CrashPointReplayOracleViolationKind::WrongContent);
    let Some(comparison) = compare_on_every_state_examining(
        cards,
        &flow,
        expand_up_to(),
        &[StateExamination::CountStatesWhere(
            StateCondition::CpuRedItemsContain(wrong_content),
        )],
    ) else {
        return;
    };
    assert!(
        comparison.examined[0].states_counted > 0,
        "流 {} 上落到变长出洞那一版的状态 CPU 看 journal 那一遍红「内容不同」",
        flow.name
    );
    assert_the_gpu_judge_agrees_with_the_cpu_on_every_state_of_a_truncation_flow(
        &flow,
        &comparison,
    );
}

// ───────────── 截断字段、I-9.7 拆段、I-9.39、incompat 位 2─────────────
// 记录与系统配置都按字节改。incompat 位 2 是这一版布局身份的一部分，录下的流每一槽都带它。

/// inode 记录里截断字段两段与两段恒零区（判决 R1）：T 在偏移 120、N_w 在偏移 128；截断字段之前那一段预留的最后一个字节在 119，之后那一段在 136–139。
const INODE_RECORD_TRUNCATION_FIELD_TXG_OFFSET: usize = 120;

const INODE_RECORD_TRUNCATION_FIELD_SIZE_OFFSET: usize = 128;

pub(super) const INODE_RECORD_LAST_RESERVED_BYTE_BEFORE_THE_TRUNCATION_FIELD: usize = 119;

pub(super) const INODE_RECORD_LAST_RESERVED_BYTE_AFTER_THE_TRUNCATION_FIELD: usize = 139;

/// 系统配置槽里 incompat 位图第一个字节的偏移（magic 4 + 格式版本 2）与这一字节里这一版布局的两位：位 1 布局身份、位 2 inode 记录带截断字段
/// （D15（格式冻结政策） 已定项 4）；位 3 没分出去。
pub(super) const FIRST_INCOMPAT_BYTE_OFFSET: usize = 6;

pub(super) const INCOMPAT_FIRST_SSD_LINE_BIT: u8 = 0x02;

pub(super) const INCOMPAT_INODE_RECORD_TRUNCATION_FIELD_BIT: u8 = 0x04;

pub(super) const INCOMPAT_UNASSIGNED_BIT_THREE: u8 = 0x08;

/// 三个单元的文件那次顺序写的 checkpoint_txg（新池新建文件 txg 3 之后的那一次）：三个单元的单元头诞生代号都是它。
pub(super) const CHECKPOINT_TXG_OF_THE_THREE_UNIT_FILE: u64 = 4;

/// 新池新建文件那一版的 checkpoint_txg（取号、两次暖机之后）：它唯一那个单元的诞生代号。
pub(super) const CHECKPOINT_TXG_OF_THE_NEW_POOL_FILE: u64 = 3;

/// 缩到一个半净荷那次截断意图的 checkpoint_txg。
pub(super) const CHECKPOINT_TXG_OF_THE_FIRST_TRUNCATION_INTENT: u64 = 5;

/// 截断字段的两段写成 (T, N_w)。
pub(super) fn write_the_truncation_field(
    record: &mut [u8],
    truncation_field_txg: u64,
    truncation_field_size_in_bytes: u64,
) {
    record[INODE_RECORD_TRUNCATION_FIELD_TXG_OFFSET..INODE_RECORD_TRUNCATION_FIELD_TXG_OFFSET + 8]
        .copy_from_slice(&truncation_field_txg.to_le_bytes());
    record
        [INODE_RECORD_TRUNCATION_FIELD_SIZE_OFFSET..INODE_RECORD_TRUNCATION_FIELD_SIZE_OFFSET + 8]
        .copy_from_slice(&truncation_field_size_in_bytes.to_le_bytes());
}

/// 流里最后写的那片 inode 叶容器（两份副本）上第一个文件的记录照 `change` 改、重封到根。
pub(super) fn change_the_record_of_the_first_file_in_the_last_inode_leaf(
    flow: &mut RecordedFlow,
    change: &dyn Fn(&mut [u8]),
) {
    let targets = last_unit_copies(
        flow,
        "最后写的那片 inode 叶容器",
        &is_an_inode_leaf_container,
        false,
    );
    corrupt_units_and_reseal_up_to_the_root(flow, &targets, &|bytes| {
        let record = offset_of_the_inode_record(bytes, FIRST_FILE.0);
        change(&mut bytes[record..record + INODE_RECORD_BYTES]);
    });
}

/// 一个系统配置槽（magic 对得上的那几份；全零的槽不碰）incompat 第一个字节照 `change` 改、重封整槽校验和。
fn change_the_first_incompat_byte_of_a_system_configuration_slot(
    slot: &mut [u8],
    change: &dyn Fn(u8) -> u8,
) {
    if &slot[..4] != b"SFSB" {
        return;
    }
    slot[FIRST_INCOMPAT_BYTE_OFFSET] = change(slot[FIRST_INCOMPAT_BYTE_OFFSET]);
    let digest = singlefs_core::checksum::wide_checksum_with_field_zeroed(
        slot,
        4096,
        SYSTEM_CONFIGURATION_CHECKSUM_FIELD,
    );
    slot[SYSTEM_CONFIGURATION_CHECKSUM_FIELD..SYSTEM_CONFIGURATION_CHECKSUM_FIELD + 32]
        .copy_from_slice(&digest);
}

/// `devices` 里每块盘上流里每一次系统配置槽写的 incompat 第一个字节照 `change` 改，交回改了几次。基镜像里 mkfs 写下的两槽不动：
/// 判表照它们取落点几何（`crash_judge_tables` 的 `JudgeTablesRefusal::BaseSystemConfigurationUnreadable`：基线两槽都解不开就建不出表）。
pub(super) fn change_the_first_incompat_byte_of_every_system_configuration_slot_write_on(
    flow: &mut RecordedFlow,
    devices: &[DeviceIdentity],
    change: &dyn Fn(u8) -> u8,
) -> usize {
    let mut writes_changed = 0;
    for write in &mut flow.writes {
        if write.kind != StepKind::SystemConfigurationSlot || !devices.contains(&write.device) {
            continue;
        }
        match &mut write.contents {
            WrittenContents::Bytes(bytes) => {
                change_the_first_incompat_byte_of_a_system_configuration_slot(bytes, change);
                writes_changed += 1;
            }
            WrittenContents::Zeros { .. } => panic!("系统配置槽写是普通写"),
        }
    }
    writes_changed
}

/// 两块盘。
pub(super) const BOTH_DEVICES: [DeviceIdentity; 2] = [DeviceIdentity(0), DeviceIdentity(1)];

/// 三个单元的文件那一版（txg 4）停下的截断与洞流，第一个文件的记录照 `change` 改。
pub(super) fn record_the_three_unit_file_with_its_record_changed(
    tag: &str,
    name: &'static str,
    change: &dyn Fn(&mut [u8]),
) -> RecordedFlow {
    let mut flow = record_new_pool_through_truncations_and_holes(
        tag,
        name,
        TruncationsAndHolesFlowStop::AfterTheThreeUnitFile,
    );
    change_the_record_of_the_first_file_in_the_last_inode_leaf(&mut flow, change);
    flow
}

/// 截断字段这几条流上 GPU 判器与 CPU 逐状态比：CPU 池级 checker 红的恰是 `expected_red`、判过而成立的里有 `expected_held`；
/// 池级 checker 那一半（违例位、判到位）每个状态逐位相同，再比两遍恢复与整份对拍字。
pub(super) fn assert_the_gpu_judge_agrees_with_the_cpu_on_a_truncation_field_flow(
    test_name: &str,
    flow: &RecordedFlow,
    expected_red: &[&'static str],
    expected_held: &[&'static str],
) {
    let Some(cards) = gpu_card_pool_or_report_not_run(test_name) else {
        return;
    };
    let Some(comparison) = compare_on_every_state(cards, flow, expand_up_to()) else {
        return;
    };
    let expected_red: std::collections::BTreeSet<&'static str> =
        expected_red.iter().copied().collect();
    assert_eq!(
        comparison.cpu_checker_red_invariants, expected_red,
        "流 {} 上 CPU 池级 checker 红的那几条",
        flow.name
    );
    for invariant in expected_held {
        assert!(
            comparison.cpu_checker_held_invariants.contains(invariant),
            "流 {} 上 CPU 池级 checker 判过 {invariant} 而成立（成立过的：{:?}）",
            flow.name,
            comparison.cpu_checker_held_invariants
        );
    }
    assert_the_pool_checker_half_then_the_recovery_half_agree(flow, &comparison);
}

/// 池级 checker 那一半先比，再比恢复那一半与整份。
pub(super) fn assert_the_pool_checker_half_then_the_recovery_half_agree(
    flow: &RecordedFlow,
    comparison: &Comparison,
) {
    let first_lines = |lines: &[String]| {
        lines
            .iter()
            .filter(|line| !line.is_empty())
            .cloned()
            .collect::<Vec<_>>()
            .join("\n")
    };
    assert!(
        comparison.checker_mismatches.is_empty(),
        "流 {}：{} 个状态 GPU 与 CPU 的池级 checker 不一致（前几个）：\n{}",
        flow.name,
        comparison.checker_mismatches.state_count(),
        first_lines(&comparison.checker_mismatches)
    );
    assert_eq!(
        comparison.recovery_mismatches,
        0,
        "流 {}：{} 个状态 GPU 与 CPU 两遍恢复的结局或落到的根不同：\n{}",
        flow.name,
        comparison.recovery_mismatches,
        first_lines(&comparison.mismatches)
    );
    assert!(
        comparison.mismatches.is_empty(),
        "流 {}：{} 个状态 GPU 与 CPU 不一致（前几个）：\n{}",
        flow.name,
        comparison.mismatches.state_count(),
        first_lines(&comparison.mismatches)
    );
}

/// I-9.7 截断字段之前那一段预留（偏移 108–119）的最后一个字节写 1：CPU 只红 I-9.7，两遍恢复都把这条记录判不可用；每个状态整份相同。
#[test]
fn the_gpu_judge_agrees_with_the_cpu_when_the_reserved_bytes_before_the_truncation_field_are_not_zero(
) {
    const TEST_NAME: &str =
        "the_gpu_judge_agrees_with_the_cpu_when_the_reserved_bytes_before_the_truncation_field_are_not_zero";
    let flow = record_the_three_unit_file_with_its_record_changed(
        "crash-judge-gpu-reserved-byte-before-the-truncation-field",
        "reserved-byte-before-the-truncation-field",
        &|record| record[INODE_RECORD_LAST_RESERVED_BYTE_BEFORE_THE_TRUNCATION_FIELD] = 1,
    );
    assert_the_gpu_judge_agrees_with_the_cpu_on_a_truncation_field_flow(
        TEST_NAME,
        &flow,
        &["I-9.7"],
        &[],
    );
}

/// I-9.7 截断字段之后那一段预留（偏移 136–139）的最后一个字节写 1：CPU 只红 I-9.7；每个状态整份相同。
#[test]
fn the_gpu_judge_agrees_with_the_cpu_when_the_reserved_bytes_after_the_truncation_field_are_not_zero(
) {
    const TEST_NAME: &str =
        "the_gpu_judge_agrees_with_the_cpu_when_the_reserved_bytes_after_the_truncation_field_are_not_zero";
    let flow = record_the_three_unit_file_with_its_record_changed(
        "crash-judge-gpu-reserved-byte-after-the-truncation-field",
        "reserved-byte-after-the-truncation-field",
        &|record| record[INODE_RECORD_LAST_RESERVED_BYTE_AFTER_THE_TRUNCATION_FIELD] = 1,
    );
    assert_the_gpu_judge_agrees_with_the_cpu_on_a_truncation_field_flow(
        TEST_NAME,
        &flow,
        &["I-9.7"],
        &[],
    );
}
