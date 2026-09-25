//! 里程碑「第二个事务」增补 2 收口表第 36 行（并行线一：末条装不下）与第 9 行（P6 链首锚）的读者一半：
//! 记录标志位 0「本次发布末条」与本次发布内序号，恢复怎么读它们。
//!
//! 压着的条款：D23（journal 的角色与格式） 已定项 4（记录标志 1：位 0 = 本次发布末条，其余位写 0、读到非 0 当损坏；
//! 本次发布内序号 1..N、与 jsn 同步，序号 0 或一次发布之内跳号当那条记录损坏、断链即止）、已定项 17（只有真正的最后一条带标志）、
//! 已定项 14 第六条（发布边界按记录标志位 0 认）与注 1（锚点按末条标志认，读法乙：所选根那次发布读得出的几条都不带标志 ⇒
//! 当锚点读不出，走「序号为 1 的第一条可读记录当链首」那一支）、已定项 7（提交标记只在一个事务的最后一条记录上）。
//!
//! 历史都一样：A（第一个文件，jsn 3）→ 顺序写两个数据单元的 B（jsn 4、5，txg 4，B 的根落了）→ 顺序写三个数据单元的 C
//! （jsn 6、7、8，txg 5，事务号 4、5、6），崩在 C 的根槽 FUA 之前：所选根是 B 的，C 的三条记录全落了。
//! 什么都不改时恢复施加 C（阳性对照）；每条用例只改 B 或 C 的一条记录（两盘各一份，经 `JournalRecord::to_bytes` 重封校验和），
//! 看恢复施加到哪一版。

mod common;

use common::{
    build_pool, crash_state_devices, parameters, BuiltPool, FIXED_WRITE_TIME_SECONDS, IMAGE_BYTES,
};
use singlefs_checker::image::InvariantVerdict;
use singlefs_checker::walk::check_pool_image;
use singlefs_core::address::{CheckpointTxg, DeviceIdentity, InstanceGeneration};
use singlefs_core::checksum::wide_checksum_with_field_zeroed;
use singlefs_core::journal::{
    record_offset, JournalRecord, JournalRecordOrdinalWithinPublish, JournalRecordPlaceInPublish,
    JOURNAL_HEADER_CHECKSUM_OFFSET,
};
use singlefs_core::mount::{mount_writable, MountError};
use singlefs_core::recovery::{
    recover, JournalPolicy, PoolReader, RecoveryFailure, RecoveryOutcome, RecoveryReport,
};
use singlefs_core::transaction::{
    publish_sequential_write, FirstFile, PoolWriter, TransactionOutput,
};
use singlefs_core::unit::{data_unit_payload_capacity, unit_filesystem_identifier};
use singlefs_format::{JOURNAL_RECORD_BYTES, JOURNAL_RING_DEFAULT_BYTES};
use singlefs_harness::crash::MemoryPool;
use singlefs_harness::{RecordedOperationKind, SharedStream};

const DEVICES: [DeviceIdentity; 2] = [DeviceIdentity(0), DeviceIdentity(1)];

/// 恰好要 `data_units` 个数据单元的内容：最后一个单元装一半。
fn content_needing(data_units: usize, seed: usize) -> Vec<u8> {
    let payload_capacity = data_unit_payload_capacity();
    (0..(data_units - 1) * payload_capacity + payload_capacity / 2)
        .map(|index| u8::try_from((index * 13 + seed * 5 + 3) % 251).expect("小于 256"))
        .collect()
}

fn sequential_write(pool: &mut BuiltPool, content: &[u8]) -> TransactionOutput {
    let publish_parameters = parameters();
    let devices = pool.devices.as_mut().expect("镜像还开着");
    let mut writer = PoolWriter::new(&publish_parameters, devices.as_mut_slice());
    let previous = pool.output.clone();
    let output = publish_sequential_write(
        &mut writer,
        &mut pool.allocator,
        &previous,
        FirstFile {
            content,
            write_time_seconds: FIXED_WRITE_TIME_SECONDS + 60,
        },
        InstanceGeneration(1),
    )
    .expect("顺序写");
    pool.output = output.clone();
    output
}

/// 这份历史里要用到的东西：崩在 C 的根之前的镜像、B 与 C 各自的根与内容、各自的记录计数器。
struct CrashBeforeTheThirdRoot {
    image: MemoryPool,
    second_version: (InstanceGeneration, CheckpointTxg),
    second_content: Vec<u8>,
    second_counters: Vec<u64>,
    third_version: (InstanceGeneration, CheckpointTxg),
    third_content: Vec<u8>,
    third_counters: Vec<u64>,
}

fn counters_of(output: &TransactionOutput) -> Vec<u64> {
    output
        .earlier_records_of_this_publish
        .iter()
        .map(|written| written.record.counter)
        .chain([output.record.counter])
        .collect()
}

fn crash_before_the_third_root(tag: &str) -> CrashBeforeTheThirdRoot {
    let mut pool = build_pool(tag);
    let second_content = content_needing(2, 1);
    let second = sequential_write(&mut pool, &second_content);
    let third_content = content_needing(3, 2);
    let third = sequential_write(&mut pool, &third_content);
    let second_counters = counters_of(&second);
    let third_counters = counters_of(&third);
    assert_eq!(
        (second_counters.clone(), third_counters.clone()),
        (vec![4, 5], vec![6, 7, 8]),
        "B 两条记录、C 三条记录，接着 A 那条（jsn 3）"
    );
    assert_eq!(
        (second.root.checkpoint_txg, third.root.checkpoint_txg),
        (CheckpointTxg(4), CheckpointTxg(5))
    );
    let operations = pool.retained_operations();
    let last_root_slot_write = operations
        .iter()
        .rposition(|retained| {
            retained.operation.kind == RecordedOperationKind::WriteForceUnitAccess
        })
        .expect("写过根槽");
    let mut image = MemoryPool::with_devices(&DEVICES, IMAGE_BYTES);
    image.apply(&operations[..last_root_slot_write]);
    CrashBeforeTheThirdRoot {
        image,
        second_version: (second.root.instance, second.root.checkpoint_txg),
        second_content,
        second_counters,
        third_version: (third.root.instance, third.root.checkpoint_txg),
        third_content,
        third_counters,
    }
}

fn filesystem_identifier() -> u64 {
    unit_filesystem_identifier(&parameters().filesystem_identifier)
}

fn record_bytes_on(image: &MemoryPool, device: DeviceIdentity, counter: u64) -> Vec<u8> {
    PoolReader::read(
        image,
        device,
        record_offset(counter, JOURNAL_RING_DEFAULT_BYTES),
        usize::try_from(JOURNAL_RECORD_BYTES).expect("4096"),
    )
    .expect("环里那一格读得到")
}

fn write_record_bytes(image: &mut MemoryPool, device: DeviceIdentity, counter: u64, bytes: &[u8]) {
    image
        .devices
        .get_mut(&device)
        .expect("有这块盘")
        .write(record_offset(counter, JOURNAL_RING_DEFAULT_BYTES), bytes);
}

/// 把 `counter` 那条记录的字段改一处，两盘各一份，`JournalRecord::to_bytes` 重封载荷与头部校验和：
/// 这条记录照样自证得过，拦住它的只剩被改的那个字段。
fn rewrite_record(image: &mut MemoryPool, counter: u64, change: impl Fn(&mut JournalRecord)) {
    for device in DEVICES {
        let mut record = JournalRecord::parse(
            &record_bytes_on(image, device, counter),
            filesystem_identifier(),
        )
        .expect("改之前这条记录自证过");
        change(&mut record);
        write_record_bytes(image, device, counter, &record.to_bytes());
    }
}

/// 改记录头里的原始字节（写者的类型写不出来的值），再重封头部校验和（罩整条 4096、自身按 0 参与）。
fn rewrite_record_header_byte(image: &mut MemoryPool, counter: u64, offset: usize, value: &[u8]) {
    for device in DEVICES {
        let mut bytes = record_bytes_on(image, device, counter);
        bytes[offset..offset + value.len()].copy_from_slice(value);
        let record_bytes = usize::try_from(JOURNAL_RECORD_BYTES).expect("4096");
        let digest =
            wide_checksum_with_field_zeroed(&bytes, record_bytes, JOURNAL_HEADER_CHECKSUM_OFFSET);
        bytes[JOURNAL_HEADER_CHECKSUM_OFFSET..JOURNAL_HEADER_CHECKSUM_OFFSET + 32]
            .copy_from_slice(&digest);
        write_record_bytes(image, device, counter, &bytes);
    }
}

/// 把 `counter` 那条记录两份都撕掉（记录头之后的一个字节翻过来，头部校验和罩整条 4096 ⇒ 自证不过）。
fn tear_record(image: &mut MemoryPool, counter: u64) {
    for device in DEVICES {
        image.flip_byte(
            device,
            record_offset(counter, JOURNAL_RING_DEFAULT_BYTES),
            300,
        );
    }
}

fn assert_recovered_version(
    report: &RecoveryReport,
    expected_version: (InstanceGeneration, CheckpointTxg),
    expected_content: &[u8],
    expected_applied: usize,
    what: &str,
) {
    assert_eq!(
        (report.effective_root, report.journal.prefix_applied),
        (Some(expected_version), expected_applied),
        "{what}：施加之后走的根与施加的记录条数（{:?}）",
        report.journal
    );
    let RecoveryOutcome::FileRead { content, .. } = &report.outcome else {
        panic!("{what}：要读回一个文件，实际 {:?}", report.outcome);
    };
    assert!(
        content == expected_content,
        "{what}：读回的内容（{} 字节）",
        content.len()
    );
}

/// D23（journal 的角色与格式） 已定项 14 注 1，读法乙（用户 2026-09-24 定）：锚点「所选根覆盖的最后一条」按末条标志认。
/// 所选根是 B 的，把 B 的末条（jsn 5，带标志）两份都撕掉、B 的第一条（jsn 4，不带标志）读得出 ⇒ B 那次发布读得出的记录里
/// 没有一条带末条标志，就算锚点读不出，走「序号为 1 的第一条可读记录当链首」那一支：C 的第一条（jsn 6）txg = B + 1、序号 1，
/// 接上，C 三条整次施加、读回 C。读法甲（取读得出的同 txg 记录里 jsn 最大那条，jsn 4）期待的下一条是读不出的 jsn 5，
/// 断号即止，C 一条都不施加、读回 B。阳性对照：什么都不撕时锚在 jsn 5，同样施加 C。
#[test]
fn when_the_last_record_of_the_chosen_roots_publish_is_unreadable_its_readable_earlier_record_is_no_anchor_and_the_next_publish_starts_the_chain(
) {
    let history = crash_before_the_third_root("last-flag-anchor-reading-b");
    assert_recovered_version(
        &recover(&history.image, JournalPolicy::Consult),
        history.third_version,
        &history.third_content,
        3,
        "什么都不撕：锚在 B 带末条标志的 jsn 5，C 整次施加",
    );
    let mut image = history.image.clone();
    tear_record(&mut image, history.second_counters[1]);
    assert_recovered_version(
        &recover(&image, JournalPolicy::Consult),
        history.third_version,
        &history.third_content,
        3,
        "B 的末条两份都撕了、第一条读得出：它不带末条标志，不当锚点；链首是 C 序号 1 的第一条，C 整次施加",
    );
}

/// 所选根那次发布读得出的记录里带末条标志的多于一条：锚点按末条标志认（已定项 14 注 1），两条都带时认哪一条**条款没有写**
/// ⇒ 第一版不支持：恢复在施加任何记录之前停下，报 `RootPublishCarriesMoreThanOneLastRecordFlagWhoseAnchorIsUndecided`；
/// 可写挂载同样在任何落盘动作之前返回这个成员，两块盘逐字节不变、录制流一步都没多。
/// 造法：B 的第一条（jsn 4）也写上末条标志、重封校验和（写者造不出，要一条记录坏了而校验和恰好仍对得上）。
/// 只钉「返回这个成员、盘上逐字节不变」，不钉之后怎么办。
#[test]
fn two_last_record_flags_in_the_chosen_roots_publish_stop_recovery_and_a_writable_mount_before_any_write(
) {
    let history = crash_before_the_third_root("last-flag-two-flags-in-the-root-publish");
    let mut image = history.image.clone();
    rewrite_record(&mut image, history.second_counters[0], |record| {
        record.place_in_publish = JournalRecordPlaceInPublish::LastRecordOfThePublish;
    });
    let expected_failure =
        RecoveryFailure::RootPublishCarriesMoreThanOneLastRecordFlagWhoseAnchorIsUndecided {
            instance: history.second_version.0,
            checkpoint_txg: history.second_version.1,
            counters: history.second_counters.clone(),
        };
    let report = recover(&image, JournalPolicy::Consult);
    assert_eq!(
        (
            report.outcome,
            report.effective_root,
            report.journal.prefix_applied
        ),
        (
            RecoveryOutcome::Failed {
                root: Some(history.second_version),
                failure: expected_failure.clone(),
            },
            None,
            0
        ),
        "恢复在施加任何记录之前停下"
    );

    let stream = SharedStream::retaining_contents();
    let mut devices = crash_state_devices(&image, &[], &[], &stream);
    let before = common::memory_pool_of_sparse_devices(&devices);
    assert!(before == image, "挂载之前两块盘就是这份镜像");
    let mounted = mount_writable(&parameters(), &mut devices);
    let Err(MountError::Recovery(failure)) = mounted else {
        panic!("可写挂载该报恢复失败");
    };
    assert_eq!(failure, expected_failure, "可写挂载交回同一个成员");
    assert_eq!(stream.operations().len(), 0, "一个写、一道屏障都没发");
    assert!(
        common::memory_pool_of_sparse_devices(&devices) == before,
        "两块盘逐字节不变"
    );
}

/// 已定项 4：一次发布的 N 条记录序号依次 1..N、与 jsn 同步；一次发布之内跳号，当那条记录损坏、断链即止。
/// C 的第二条（jsn 7）序号从 2 改成 3（重封校验和，jsn 仍连着）⇒ 断在 jsn 7，C 走不到末条、整体不施加，读回 B。
/// 不判跳号的读者照样接上 jsn 7、jsn 8，走到带末条标志的 jsn 8，把 C 整次施加。
#[test]
fn an_ordinal_that_skips_within_a_publish_breaks_the_chain_and_the_publish_is_not_applied() {
    let history = crash_before_the_third_root("last-flag-ordinal-skips");
    let mut image = history.image.clone();
    rewrite_record(&mut image, history.third_counters[1], |record| {
        record.ordinal_within_publish = JournalRecordOrdinalWithinPublish(3);
    });
    assert_recovered_version(
        &recover(&image, JournalPolicy::Consult),
        history.second_version,
        &history.second_content,
        0,
        "C 的序号 1、3、3：一次发布之内跳号，C 整体不施加",
    );
}

/// 已定项 4：序号 0 当那条记录损坏；记录标志位 0 之外有位为 1 当那条记录损坏。两样都与校验和不过同一个结局：
/// C 的第二条（jsn 7）不算在，jsn 断号即止，C 走不到末条、整体不施加，读回 B。头部校验和按改过的字节重封过。
#[test]
fn a_record_whose_ordinal_is_zero_or_whose_flags_set_another_bit_counts_as_torn_and_its_publish_is_not_applied(
) {
    let history = crash_before_the_third_root("last-flag-ordinal-zero-or-other-flag-bits");
    for (offset, value, what) in [
        (87usize, 0u32.to_le_bytes().to_vec(), "C 的第二条序号 0"),
        (7usize, vec![0b0000_0010u8], "C 的第二条记录标志位 1 为 1"),
    ] {
        let mut image = history.image.clone();
        rewrite_record_header_byte(&mut image, history.third_counters[1], offset, &value);
        assert_recovered_version(
            &recover(&image, JournalPolicy::Consult),
            history.second_version,
            &history.second_content,
            0,
            what,
        );
    }
}

/// 已定项 7：一个事务可以跨多条记录，只有它的最后一条带提交标记；「丢掉提交标记还没出现的那个事务的全部记录」，
/// 施加的单位是一次发布（第六条）。C 的第一条（jsn 6，事务号 4）的提交标记改成 0，下一条（jsn 7）换成了事务号 5 ⇒
/// 事务 4 的提交标记没出现，它所在的 C 整体不施加，读回 B。只看末条标志的读者会把缺了提交标记的事务 4 连着 C 一起施加。
#[test]
fn a_transaction_whose_commit_marker_never_appears_before_the_next_transaction_keeps_its_publish_unapplied(
) {
    let history = crash_before_the_third_root("last-flag-commit-marker-missing-mid-publish");
    let mut image = history.image.clone();
    rewrite_record(&mut image, history.third_counters[0], |record| {
        record.is_commit = false;
    });
    assert_recovered_version(
        &recover(&image, JournalPolicy::Consult),
        history.second_version,
        &history.second_content,
        0,
        "C 的事务 4 没有提交标记、下一条换了事务号：C 整体不施加",
    );
}

/// 已定项 7 与第六条：带末条标志的那一条不带提交标记 ⇒ 这次发布最后一个事务没提交，整体不施加。
/// C 的末条（jsn 8）提交标记改成 0 ⇒ 读回 B。末条之外不带提交标记的记录要接着往下走（一个事务跨多条记录），
/// 只在「是末条」这一格停；不判这一格的读者把没提交的最后一个事务连着 C 一起施加。
#[test]
fn a_last_record_of_a_publish_without_a_commit_marker_keeps_its_publish_unapplied() {
    let history = crash_before_the_third_root("last-flag-last-record-without-commit-marker");
    let mut image = history.image.clone();
    rewrite_record(&mut image, history.third_counters[2], |record| {
        record.is_commit = false;
    });
    assert_recovered_version(
        &recover(&image, JournalPolicy::Consult),
        history.second_version,
        &history.second_content,
        0,
        "C 的末条不带提交标记：C 整体不施加",
    );
}

/// 池级 checker 在这份镜像上对 I-8.9（一次发布的记录序号连续且只有末条带标志）的判定。
fn publish_ordinal_verdict(image: &MemoryPool) -> InvariantVerdict {
    check_pool_image(image)
        .into_iter()
        .find(|(invariant, _)| *invariant == "I-8.9")
        .map(|(_, verdict)| verdict)
        .expect("checker 每次都报 I-8.9")
}

/// D23（journal 的角色与格式） 已定项 4 读者规则（用户 2026-09-24 定，C539（锚点读得出时下一次发布的首条序号不是 1））：
/// 锚点读得出时，下一次发布的首条序号不是 1，当那条记录损坏、断在这一条，那次发布整体不施加。
/// C 的三条（jsn 6、7、8）序号改成 2、3、4（重封校验和；一次发布之内照样连号，「跳号」那一判拦不住它）：锚点是 B 带末条标志的 jsn 5，
/// 接上的 jsn 6 是 C 的首条、序号 2 ⇒ 断在 jsn 6，C 一条都不施加、读回 B。照接的读者（改之前）把 C 整次施加、读回 C。
/// 池级 checker 的 I-8.9 在同一份镜像上判「不从 1 起」。
#[test]
fn with_a_readable_anchor_a_next_publish_whose_first_ordinal_is_not_one_is_not_applied_and_the_checker_reddens(
) {
    let history = crash_before_the_third_root("last-flag-first-ordinal-not-one");
    let mut image = history.image.clone();
    for (position, counter) in history.third_counters.iter().enumerate() {
        let shifted_ordinal = u32::try_from(position + 2).expect("三条记录");
        rewrite_record(&mut image, *counter, |record| {
            record.ordinal_within_publish = JournalRecordOrdinalWithinPublish(shifted_ordinal);
        });
    }
    assert_recovered_version(
        &recover(&image, JournalPolicy::Consult),
        history.second_version,
        &history.second_content,
        0,
        "C 的首条序号是 2：断在 C 的首条，C 整体不施加",
    );
    let verdict = publish_ordinal_verdict(&image);
    assert!(
        matches!(&verdict, InvariantVerdict::Violated(detail) if detail.contains("不从 1 起")),
        "checker 的 I-8.9 判「不从 1 起」：{verdict:?}"
    );
}

/// D23（journal 的角色与格式） 已定项 4 读者规则（用户 2026-09-24 定，C540（末条标志坏在一次发布中间，读者切出两次发布））：
/// 同一 (实例代号, checkpoint_txg) 里带末条标志的那一条之后还有记录，当那条损坏、断在带标志的那一条，那次发布整体不施加。
/// C 的末条标志挪到首条（jsn 6 写上、jsn 8 抹掉，两条都重封校验和；写者造不出，要记录坏了而校验和恰好仍对得上）：
/// jsn 6 之后同一个 txg 5 还有 jsn 7、8 ⇒ 断在 jsn 6，C 一条都不施加、读回 B。照字面按标志认边界的读者（改之前）把 {jsn 6}
/// 当一次发布施加（它带着 C 的整个新根段），读回 C。只挪不添，checker 这一格判的就是「末条之后还有记录」、不是「多于一条末条」。
#[test]
fn a_last_record_flag_followed_by_a_record_of_the_same_publish_breaks_the_chain_at_the_flag_and_the_checker_reddens(
) {
    let history = crash_before_the_third_root("last-flag-followed-within-its-publish");
    let mut image = history.image.clone();
    rewrite_record(&mut image, history.third_counters[0], |record| {
        record.place_in_publish = JournalRecordPlaceInPublish::LastRecordOfThePublish;
    });
    rewrite_record(&mut image, history.third_counters[2], |record| {
        record.place_in_publish = JournalRecordPlaceInPublish::MoreRecordsOfThePublishFollow;
    });
    assert_recovered_version(
        &recover(&image, JournalPolicy::Consult),
        history.second_version,
        &history.second_content,
        0,
        "C 的首条带末条标志、后面还有同一次发布的两条：断在 C 的首条，C 整体不施加",
    );
    let verdict = publish_ordinal_verdict(&image);
    assert!(
        matches!(&verdict, InvariantVerdict::Violated(detail) if detail.contains("末条之后还有记录")),
        "checker 的 I-8.9 判「末条之后还有记录」：{verdict:?}"
    );
}
