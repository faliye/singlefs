//! harness 测试文件 `crates/singlefs-harness/tests/journal_last_record_flag_of_a_multi_record_publish.rs` 的场景函数：搭场景、核实现的行为，池级 checker 要判的镜像经 [`PoolCheckerImages`] 交出去
//! （做法见 `common_images_for_the_checker_tier/mod.rs`）。harness 档那份测试文件与 checker 档的
//! `crates/singlefs-checker-tier/tests/crash_given_states_of_the_images_harness_scenarios_hand_out.rs` 各经 `#[path]` 引这一份。
#![allow(dead_code, reason = "两档各只调其中一部分")]

use crate::common::{build_pool, parameters, BuiltPool, FIXED_WRITE_TIME_SECONDS, IMAGE_BYTES};
use crate::common_images_for_the_checker_tier::{PoolCheckerExpectation, PoolCheckerImages};
use singlefs_core::address::{CheckpointTxg, DeviceIdentity, InstanceGeneration};
use singlefs_core::journal::{
    back_chain_of, record_offset, JournalRecord, JournalRecordOrdinalWithinPublish,
    JournalRecordPlaceInPublish,
};
use singlefs_core::recovery::{
    recover, JournalPolicy, PoolReader, RecoveryOutcome, RecoveryReport,
};
use singlefs_core::transaction::{
    publish_sequential_write, FirstFile, PoolWriter, TransactionOutput,
};
use singlefs_core::unit::{data_unit_payload_capacity, unit_filesystem_identifier};
use singlefs_format::{JOURNAL_RECORD_BYTES, JOURNAL_RING_DEFAULT_BYTES};
use singlefs_harness::memory_pool::MemoryPool;
use singlefs_harness::RecordedOperationKind;

pub const DEVICES: [DeviceIdentity; 2] = [DeviceIdentity(0), DeviceIdentity(1)];

/// 恰好要 `data_units` 个数据单元的内容：最后一个单元装一半。
pub fn content_needing(data_units: usize, seed: usize) -> Vec<u8> {
    let payload_capacity = data_unit_payload_capacity();
    (0..(data_units - 1) * payload_capacity + payload_capacity / 2)
        .map(|index| u8::try_from((index * 13 + seed * 5 + 3) % 251).expect("小于 256"))
        .collect()
}

pub fn sequential_write(pool: &mut BuiltPool, content: &[u8]) -> TransactionOutput {
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
pub struct CrashBeforeTheThirdRoot {
    pub image: MemoryPool,
    pub second_version: (InstanceGeneration, CheckpointTxg),
    pub second_content: Vec<u8>,
    pub second_counters: Vec<u64>,
    pub third_version: (InstanceGeneration, CheckpointTxg),
    pub third_content: Vec<u8>,
    pub third_counters: Vec<u64>,
}

pub fn counters_of(output: &TransactionOutput) -> Vec<u64> {
    output
        .earlier_records_of_this_publish
        .iter()
        .map(|written| written.record.counter)
        .chain([output.record.counter])
        .collect()
}

pub fn crash_before_the_third_root(tag: &str) -> CrashBeforeTheThirdRoot {
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

pub fn filesystem_identifier() -> u64 {
    unit_filesystem_identifier(&parameters().filesystem_identifier)
}

pub fn record_bytes_on(image: &MemoryPool, device: DeviceIdentity, counter: u64) -> Vec<u8> {
    PoolReader::read(
        image,
        device,
        record_offset(counter, JOURNAL_RING_DEFAULT_BYTES),
        usize::try_from(JOURNAL_RECORD_BYTES).expect("4096"),
    )
    .expect("环里那一格读得到")
}

pub fn write_record_bytes(
    image: &mut MemoryPool,
    device: DeviceIdentity,
    counter: u64,
    bytes: &[u8],
) {
    image
        .devices
        .get_mut(&device)
        .expect("有这块盘")
        .write(record_offset(counter, JOURNAL_RING_DEFAULT_BYTES), bytes);
}

/// 把 `counter` 那条记录的字段改一处，两盘各一份，`JournalRecord::to_bytes` 重封载荷与头部校验和；
/// 它后面一直到环里第一条解不出的记录，每一条的反向链按前一条改后的记录头重算、重封（D23（journal 的角色与格式） 已定项 8）：
/// 这条记录照样自证得过、下一条的反向链也接得上，拦住它的只剩被改的那个字段。
/// 不重算的话，读者删掉被测的那一判之后照样断在下一条的反向链上、结局不变，用例拦不住那一判
/// （`research/prompts/m3-verus-land-code-r1-opus-output.md` C5-1：`crates/mutations.tsv` 里改这三判的三行套上都不红）。
pub fn rewrite_record(image: &mut MemoryPool, counter: u64, change: impl Fn(&mut JournalRecord)) {
    for device in DEVICES {
        let mut record = JournalRecord::parse(
            &record_bytes_on(image, device, counter),
            filesystem_identifier(),
        )
        .expect("改之前这条记录自证过");
        change(&mut record);
        write_record_bytes(image, device, counter, &record.to_bytes());
        // 迭代次数的上界是这份历史在 `counter` 之后写过的记录条数（至多 C 的三条）：环里下一格没写过、解不出就是出口。
        let mut later_counter = counter + 1;
        while let Some(mut later) = JournalRecord::parse(
            &record_bytes_on(image, device, later_counter),
            filesystem_identifier(),
        ) {
            later.back_chain = back_chain_of(&record_bytes_on(image, device, later_counter - 1));
            write_record_bytes(image, device, later_counter, &later.to_bytes());
            later_counter += 1;
        }
    }
}

pub fn assert_recovered_version(
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

/// D23（journal 的角色与格式） 已定项 4 读者规则（用户 2026-09-24 定，C539（锚点读得出时下一次发布的首条序号不是 1））：
/// 锚点读得出时，下一次发布的首条序号不是 1，当那条记录损坏、断在这一条，那次发布整体不施加。
/// C 的三条（jsn 6、7、8）序号改成 2、3、4（重封校验和；一次发布之内照样连号，「跳号」那一判拦不住它）：锚点是 B 带末条标志的 jsn 5，
/// 接上的 jsn 6 是 C 的首条、序号 2 ⇒ 断在 jsn 6，C 一条都不施加、读回 B。照接的读者（改之前）把 C 整次施加、读回 C。
/// 池级 checker 的 I-8.9 在同一份镜像上判「不从 1 起」。
pub fn with_a_readable_anchor_a_next_publish_whose_first_ordinal_is_not_one_is_not_applied_and_the_checker_reddens(
    images: &mut PoolCheckerImages,
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
    // checker 的 I-8.9 判「不从 1 起」
    images.hand_out(
        "不从 1 起",
        || image.clone(),
        PoolCheckerExpectation::only_the_named_invariants()
            .and_violated_with_a_detail_containing("I-8.9", "不从 1 起"),
    );
}

/// D23（journal 的角色与格式） 已定项 4 读者规则（用户 2026-09-24 定，C540（末条标志坏在一次发布中间，读者切出两次发布））：
/// 同一 (实例代号, checkpoint_txg) 里带末条标志的那一条之后还有记录，当那条损坏、断在带标志的那一条，那次发布整体不施加。
/// C 的末条标志挪到首条（jsn 6 写上、jsn 8 抹掉，两条都重封校验和；写者造不出，要记录坏了而校验和恰好仍对得上）：
/// jsn 6 之后同一个 txg 5 还有 jsn 7、8 ⇒ 断在 jsn 6，C 一条都不施加、读回 B。照字面按标志认边界的读者（改之前）把 {jsn 6}
/// 当一次发布施加（它带着 C 的整个新根段），读回 C。只挪不添，checker 这一格判的就是「末条之后还有记录」、不是「多于一条末条」。
pub fn last_record_flag_followed_by_the_record_of_the_same_publish_breaks_the_chain_at_the_flag_and_the_checker_reddens(
    images: &mut PoolCheckerImages,
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
    // checker 的 I-8.9 判「末条之后还有记录」
    images.hand_out(
        "末条之后还有记录",
        || image.clone(),
        PoolCheckerExpectation::only_the_named_invariants()
            .and_violated_with_a_detail_containing("I-8.9", "末条之后还有记录"),
    );
}

/// 同一条读者规则（C540），换一个取样点：C 的末条标志挪到第二条（jsn 7 写上、jsn 8 抹掉，两条都重封校验和），
/// 带标志的 jsn 7 之后同一个 txg 5 里只剩紧挨着的 jsn 8 ⇒ 断在 jsn 7，C 一条都不施加、读回 B。
/// 只从「紧挨着的下一条之后」往后找的读者看不到 jsn 8，把 {jsn 6, jsn 7} 当一次发布施加（jsn 7 带着 C 的整个新根段），读回 C；
/// 上一条用例的取样点（标志在 jsn 6，后面还有 jsn 7、8 两条）拦不住这样的读者
/// （`research/prompts/m3-verus-land-code-r1-opus-output.md` C5-2）。
pub fn last_record_flag_followed_only_by_the_very_next_record_of_the_same_publish_breaks_the_chain_at_the_flag(
    images: &mut PoolCheckerImages,
) {
    let history = crash_before_the_third_root("last-flag-followed-only-by-the-very-next-record");
    let mut image = history.image.clone();
    rewrite_record(&mut image, history.third_counters[1], |record| {
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
        "C 的第二条带末条标志、紧挨着它还有同一次发布的一条：断在 C 的第二条，C 整体不施加",
    );
    // checker 的 I-8.9 判「末条之后还有记录」
    images.hand_out(
        "末条之后还有记录",
        || image.clone(),
        PoolCheckerExpectation::only_the_named_invariants()
            .and_violated_with_a_detail_containing("I-8.9", "末条之后还有记录"),
    );
}
