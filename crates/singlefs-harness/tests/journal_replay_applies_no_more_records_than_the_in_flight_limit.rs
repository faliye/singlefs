//! 一次恢复重放的前缀最多施加「在飞记录数上限」条记录（D23（journal 的角色与格式） 已定项 18：上限 = 环槽数 ÷ F，
//! 「这个数的含义是一次恢复重放的前缀最多 N 条」）。
//!
//! 来历：Verus 核接回 `crates/` 的代码轮第一轮，攻方 C5-3（`research/prompts/m3-verus-land-code-r1-opus-output.md`）：
//! 取前缀核里 `taken >= in_flight_limit` 改成 `>`（差一）在全部 harness 目标上不红——默认环的上限是 65536，没有一条用例
//! 在水位之上摆得出比上限还多的记录。这里把环取 3 槽，上限 = 3 ÷ 3 = 1，所选根之后摆两次单条发布，钉「只施加 1 条」。
//!
//! 3 槽的环过不了 mkfs 与读者择系统配置（单元区起点要落在聚簇段边界上，3 槽的环之后那一槽不在边界上：
//! `recovery::system_configuration_values_this_reader_accepts` 报 `UnitAreaStartOffTheClusterSegmentBoundaryUnsupported`），
//! 所以走不了 `recover`：池照默认环建，崩溃镜像上把环里的记录按 3 槽环的落点（已定项 18「记录 n 落在环内偏移 (计数器 − 1) mod 槽数 × 4096」）
//! 搬一份，再拿环长改成 3 槽的那份系统配置扫环、照 `recover` 同一串调 `choose_root`、`scan_journal`、`replay_journal`。
//! 阳性对照：同一段历史在默认环的原镜像上走 `recover`，两条都施加。

mod common;

use common::{
    build_pool, overwrite_in_process_as_instance_one_and_advance_the_pool,
    tear_the_root_slot_written_for_txg, CONTENT_BYTES_OF_PUBLISH_B, CONTENT_BYTES_OF_PUBLISH_C,
};
use singlefs_core::address::{CheckpointTxg, DeviceIdentity, InstanceGeneration};
use singlefs_core::journal::record_offset;
use singlefs_core::recovery::{
    choose_root, choose_system_configuration, recover, replay_journal, scan_journal, JournalPolicy,
    PoolReader, RecoveryOutcome,
};
use singlefs_core::system_configuration::journal_in_flight_record_limit;
use singlefs_format::{JOURNAL_RECORD_BYTES, JOURNAL_RING_DEFAULT_BYTES};
use singlefs_harness::memory_pool::MemoryPool;

const BOTH_DEVICES: [DeviceIdentity; 2] = [DeviceIdentity(0), DeviceIdentity(1)];

/// 3 个记录槽的环：在飞上限 = 3 ÷ F（3）= 1。
const THREE_RECORD_RING_BYTES: u64 = 3 * JOURNAL_RECORD_BYTES;

/// 文件内容第 `index` 个字节取 (`index` × 步长 + 种子) mod 模数：模数小于 256，字节装得进 u8；
/// 步长与模数（11 × 23）互素，内容的周期是整个模数，换个种子就逐字节分得开。
const CONTENT_BYTE_STRIDE: usize = 7;
const CONTENT_BYTE_MODULUS_BELOW_256: usize = 253;
/// B、C 两版文件的内容种子（字节数在 `common`）：两版长度与种子各不相同，读回哪一版看得出。
const CONTENT_SEED_OF_PUBLISH_B: usize = 3;
const CONTENT_SEED_OF_PUBLISH_C: usize = 11;

fn content_of(length: usize, seed: usize) -> Vec<u8> {
    (0..length)
        .map(|index| {
            u8::try_from((index * CONTENT_BYTE_STRIDE + seed) % CONTENT_BYTE_MODULUS_BELOW_256)
                .expect("小于 256")
        })
        .collect()
}

/// 把计数器 `counters` 那几条记录从默认环的落点搬到 3 槽环的落点（两盘各一份；先全部读出再写，搬过去的会盖掉更早的记录）。
fn image_with_the_records_on_a_three_record_ring(
    image: &MemoryPool,
    counters: &[u64],
) -> MemoryPool {
    let record_bytes = usize::try_from(JOURNAL_RECORD_BYTES).expect("4096");
    let mut relocated = image.clone();
    for device in BOTH_DEVICES {
        let records: Vec<(u64, Vec<u8>)> = counters
            .iter()
            .map(|counter| {
                let bytes = image
                    .read(
                        device,
                        record_offset(*counter, JOURNAL_RING_DEFAULT_BYTES),
                        record_bytes,
                    )
                    .expect("默认环里那一格读得到");
                (*counter, bytes)
            })
            .collect();
        for (counter, bytes) in records {
            relocated
                .devices
                .get_mut(&device)
                .expect("有这块盘")
                .write(record_offset(counter, THREE_RECORD_RING_BYTES), &bytes);
        }
    }
    relocated
}

/// A（txg 3、jsn 3）的根落了；之后 B（txg 4、jsn 4）、C（txg 5、jsn 5）两次单条发布的记录与单元都在、根槽都坏了 ⇒ 所选根是 A 的，
/// 水位之上是两次整的发布。在飞上限 1 ⇒ 只施加 jsn 4、重建到 B；上限按 `>` 判（多施加一条）的读者把 jsn 5 也施加、重建到 C。
#[test]
fn with_an_in_flight_limit_of_one_only_the_first_of_two_single_record_publishes_above_the_chosen_root_is_applied(
) {
    let mut pool = build_pool("in-flight-limit-of-one");
    let first_version = (pool.output.root.instance, pool.output.root.checkpoint_txg);
    let second = overwrite_in_process_as_instance_one_and_advance_the_pool(
        &mut pool,
        &content_of(CONTENT_BYTES_OF_PUBLISH_B, CONTENT_SEED_OF_PUBLISH_B),
    );
    let third_content = content_of(CONTENT_BYTES_OF_PUBLISH_C, CONTENT_SEED_OF_PUBLISH_C);
    let third =
        overwrite_in_process_as_instance_one_and_advance_the_pool(&mut pool, &third_content);
    assert_eq!(
        (
            first_version,
            (second.root.checkpoint_txg, second.record.counter),
            (third.root.checkpoint_txg, third.record.counter)
        ),
        (
            (InstanceGeneration(1), CheckpointTxg(3)),
            (CheckpointTxg(4), 4),
            (CheckpointTxg(5), 5)
        ),
        "A 的根在 txg 3、jsn 3；B、C 各一条记录"
    );
    let mut image = pool.memory_pool();
    for txg in [4u64, 5] {
        tear_the_root_slot_written_for_txg(&mut image, txg);
    }

    // 阳性对照：默认环（上限 65536）上同一段历史，B、C 两次发布都施加、读回 C。
    let report_on_the_default_ring = recover(&image, JournalPolicy::Consult);
    assert_eq!(
        (
            report_on_the_default_ring.outcome,
            report_on_the_default_ring.effective_root,
            report_on_the_default_ring.journal.above_water,
            report_on_the_default_ring.journal.prefix_applied
        ),
        (
            RecoveryOutcome::FileRead {
                root: first_version,
                content: third_content
            },
            Some((InstanceGeneration(1), CheckpointTxg(5))),
            2,
            2
        ),
        "默认环：所选根是 A 的，水位之上两条都施加"
    );

    // 3 槽的环：上限 1。
    assert_eq!(
        journal_in_flight_record_limit(THREE_RECORD_RING_BYTES),
        1,
        "3 槽 ÷ F（3）= 1"
    );
    let relocated = image_with_the_records_on_a_three_record_ring(&image, &[3, 4, 5]);
    let system_configuration =
        choose_system_configuration(&relocated).expect("系统配置择得出（它记着的是默认环）");
    let mut three_record_ring_configuration = system_configuration;
    three_record_ring_configuration
        .immutable
        .sizes
        .journal_ring_bytes = THREE_RECORD_RING_BYTES;
    let chosen_root = choose_root(&relocated, &system_configuration).expect("择得出根");
    assert_eq!(
        (chosen_root.instance, chosen_root.checkpoint_txg),
        first_version,
        "B、C 的根槽都坏了：所选根是 A 的"
    );
    let records = scan_journal(&relocated, &three_record_ring_configuration);
    assert_eq!(
        records
            .keys()
            .map(|(_, counter)| *counter)
            .collect::<Vec<_>>(),
        [3, 4, 5],
        "3 槽的环里扫到 jsn 3、4、5"
    );
    let (report, rebuilt) = replay_journal(
        &relocated,
        &chosen_root,
        THREE_RECORD_RING_BYTES,
        &records,
        true,
    )
    .expect("所选根那次发布只有 jsn 3 带末条标志：锚点认得出");
    assert_eq!(
        (
            report.valid_records,
            report.above_water,
            report.prefix_applied,
            report.verification_passed
        ),
        (3, 2, 1, 1),
        "水位之上两条，上限 1：只施加 jsn 4，它的点名单元验过"
    );
    assert_eq!(
        (
            rebuilt.instance,
            rebuilt.checkpoint_txg,
            rebuilt.mapping_root
        ),
        (
            second.root.instance,
            second.root.checkpoint_txg,
            second.root.mapping_root
        ),
        "重建到 B，不是 C"
    );
}
