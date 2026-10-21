//! 截断单次发布的写入量不随文件长（判决 `research/prompts/m4-trunc-r1-main-verification.md` 第三节 K9「截断斜率用例（照
//! `unlink_writes_per_transaction_do_not_grow_with_the_file.rs`）」，K2「每一批只从尾部放 K 个单元（与 unlink 共用 `UnlinkBatchSize`）」）：
//! 几份单元数不同的文件（快档 1、144、145、432、433 个数据单元，跨过一批 144 个的边界；慢档 2304、4608 个）各截到 0，按产品批大小
//! （`UnlinkBatchSize::OneExtentLowerLeaf`）发意图、走完全部批，量每一次发布（意图、每一批、最后一批）交给设备的写调用与字节
//! （`TransactionOutput::writes`），与每一次换下的数据单元个数。截到 0 留一个声明长度 0 的单元（K3），size 之外挂着的是单元数 − 1 个：
//! 每一批换下的个数照条款算出来逐项比（意图那次 0 个，之后 size 之外还挂着的多于一批就恰好放一批 144 个，不多于一批的最后一批放完剩下的）。
//! 另一条用例不建池、直接比发布路径取下一步的那一个纯函数 `truncate::next_truncation_step` 在挂着 145 到 2⁴⁰ 个单元上都只放一批。

mod common;
mod common_truncation;

use common::{
    build_pool, parameters, sequential_write_in_process_as_instance_one_and_advance_the_pool,
};
use common_truncation::{
    change_the_size_of_the_first_file_and_advance, publish_the_next_truncation_step_and_advance,
    truncation_entry_of,
};
use singlefs_core::admission::ring_window_on_one_device;
use singlefs_core::transaction::TransactionOutput;
use singlefs_core::truncate::{next_truncation_step, TruncationStep};
use singlefs_core::unit::data_unit_payload_capacity;
use singlefs_core::unlink::UnlinkBatchSize;
use singlefs_format::{
    EXTENT_TREE_LOWER_LEAF_DATA_UNITS, JOURNAL_NAMED_ENTRIES_PER_RECORD, ROOT_RING_REGIONS,
    SLOT_BYTES,
};

/// 测试池的盘数：每个单元每块盘各写一份，journal 记录也按每块盘一份算（往多里算）；上界那一处现核它等于建出来的池的盘数。
const DEVICES_OF_THE_TEST_POOL: u64 = 2;

/// 任一份文件里写调用最多的那次发布，至多比一个单元的文件里写调用最多的那次多几个写调用（照 unlink 那一份的算法，判决第三节 K10：
/// 截断的意图、各批、最后一批当删除类，准入按一次删除类发布每盘新占至多 W 个槽给环窗口扣空间）：按这个池的分配器经
/// `admission::ring_window_on_one_device` 现算环窗口（= (根环槽数 + 1) × W 个槽）再除回 W；每个新占的槽每块盘一个写调用、记录至多多
/// ⌈W ÷ 64⌉ 条，差至多「盘数 × (W + ⌈W ÷ 64⌉)」。这是上界，钉的数在各条用例末尾。
fn write_calls_the_largest_publish_of_any_file_may_add_over_a_one_unit_file() -> u64 {
    let pool = build_pool("truncation-writes-per-publish-bound-from-the-ring-window");
    let devices_of_the_built_pool = (
        pool.devices.as_ref().expect("镜像还开着").len(),
        pool.allocator.devices.len(),
    );
    assert_eq!(
        (
            u64::try_from(devices_of_the_built_pool.0).expect("盘数装得进 u64"),
            u64::try_from(devices_of_the_built_pool.1).expect("盘数装得进 u64"),
        ),
        (DEVICES_OF_THE_TEST_POOL, DEVICES_OF_THE_TEST_POOL),
        "上界按测试池的盘数 {DEVICES_OF_THE_TEST_POOL} 乘：建出来的池就是这么多块盘"
    );
    let slots_per_region = parameters().geometry.root_ring_slots_per_region;
    let ring_window_in_slots = ring_window_on_one_device(
        &pool.allocator,
        slots_per_region,
        EXTENT_TREE_LOWER_LEAF_DATA_UNITS,
        0,
    )
    .0 / SLOT_BYTES;
    let roots_in_the_ring_plus_one = ROOT_RING_REGIONS * slots_per_region.count() + 1;
    assert_eq!(
        ring_window_in_slots % roots_in_the_ring_plus_one,
        0,
        "环窗口是 (根环槽数 + 1) × W 个槽"
    );
    let slots_one_deletion_publish_newly_takes_on_one_device_at_most =
        ring_window_in_slots / roots_in_the_ring_plus_one;
    DEVICES_OF_THE_TEST_POOL
        * (slots_one_deletion_publish_newly_takes_on_one_device_at_most
            + slots_one_deletion_publish_newly_takes_on_one_device_at_most
                .div_ceil(JOURNAL_NAMED_ENTRIES_PER_RECORD))
}

/// 截断的一次发布交给设备的写，与这次换下（放掉）的数据单元个数。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct WritesOfOnePublish {
    write_calls: u64,
    written_bytes: u64,
    data_units_released: usize,
}

fn writes_of_one_publish(
    previous: &TransactionOutput,
    published: &TransactionOutput,
) -> WritesOfOnePublish {
    let total = published.writes.total();
    WritesOfOnePublish {
        write_calls: total.write_calls,
        written_bytes: total.written_bytes,
        data_units_released: previous
            .data_pointers
            .len()
            .checked_sub(published.data_pointers.len())
            .expect("截断的每一步只减不增数据单元"),
    }
}

/// 新池新建文件之后把第一个文件顺序写成 `data_units_of_the_file` 个数据单元（最后一个装半个净荷），截到 0：发意图、按产品批大小走完全部批；
/// 交回每一次发布的写，按先后（第一项是意图那次）。
fn writes_of_every_publish_of_a_truncation_to_zero_in_product_batches(
    data_units_of_the_file: usize,
) -> Vec<WritesOfOnePublish> {
    let payload_capacity = data_unit_payload_capacity();
    let content: Vec<u8> = (0..(data_units_of_the_file - 1) * payload_capacity
        + payload_capacity / 2)
        .map(|index| u8::try_from((index * 31 + 7) % 251).expect("小于 256"))
        .collect();
    let mut pool = build_pool(&format!(
        "truncation-writes-per-publish-of-{data_units_of_the_file}-data-units"
    ));
    let grown =
        sequential_write_in_process_as_instance_one_and_advance_the_pool(&mut pool, &content);
    assert_eq!(grown.data_pointers.len(), data_units_of_the_file);
    let mut writes = Vec::new();
    let before_the_intent = pool.output.clone();
    let intent = change_the_size_of_the_first_file_and_advance(&mut pool, 0);
    writes.push(writes_of_one_publish(&before_the_intent, &intent));
    // 迭代上界：每一批至少放一个单元（不是最后一批的），文件 `data_units_of_the_file` 个单元；最后一批删截断条目、循环停下。
    while truncation_entry_of(&pool.output).is_some() {
        assert!(writes.len() <= data_units_of_the_file + 1, "截断步数有界");
        let previous = pool.output.clone();
        let published = publish_the_next_truncation_step_and_advance(
            &mut pool,
            UnlinkBatchSize::OneExtentLowerLeaf,
        );
        writes.push(writes_of_one_publish(&previous, &published));
    }
    writes
}

fn write_calls_of_every_publish(writes: &[WritesOfOnePublish]) -> Vec<u64> {
    writes.iter().map(|publish| publish.write_calls).collect()
}

fn largest_write_calls_of_one_publish(writes: &[WritesOfOnePublish]) -> u64 {
    writes
        .iter()
        .map(|publish| publish.write_calls)
        .max()
        .expect("意图那次至少一次发布")
}

/// 截到 0 时每一次发布换下的数据单元个数照条款该是多少（K2、K11，产品每批 K 取一片下段叶的单元数 `EXTENT_TREE_LOWER_LEAF_DATA_UNITS`）：
/// 意图那次 0 个；size 之外挂着单元数 − 1 个（截到 0 留一个声明长度 0 的单元，K3），多于 K 时这一批恰好放 K 个，不多于 K 时最后一批放完
/// （可以 0 个）。按先后，第一项是意图那次。
fn data_units_released_by_every_publish_as_the_clause_gives(
    data_units_of_the_file: usize,
) -> Vec<usize> {
    let data_units_per_batch =
        usize::try_from(EXTENT_TREE_LOWER_LEAF_DATA_UNITS).expect("144 装得进 usize");
    let mut released_by_every_publish = vec![0];
    let mut data_units_beyond_the_size = data_units_of_the_file - 1;
    // 迭代上界：不是最后一批的每一轮挂着的少 K 个（K ≥ 1），⌈挂着的 ÷ K⌉ 轮之内走到最后一批。
    while data_units_beyond_the_size > data_units_per_batch {
        released_by_every_publish.push(data_units_per_batch);
        data_units_beyond_the_size -= data_units_per_batch;
    }
    released_by_every_publish.push(data_units_beyond_the_size);
    released_by_every_publish
}

fn data_units_released_by_every_publish(writes: &[WritesOfOnePublish]) -> Vec<usize> {
    writes
        .iter()
        .map(|publish| publish.data_units_released)
        .collect()
}

/// 五份文件（1、144、145、432、433 个数据单元）截到 0：
/// ① 每一次发布换下的数据单元个数与条款算出来的逐项相同（意图 0 个，之后每批恰好 144 个、最后一批放完剩下的）；
/// ② 每份文件里写调用最多的那次发布，至多比一个单元的文件里的多 [`write_calls_the_largest_publish_of_any_file_may_add_over_a_one_unit_file`]
/// （从 W 式子算，不随文件长）；
/// ③ 每一次发布的写调用钉成绝对值（发布路径今天写出的形状：一个单元的文件意图 31、最后一批 21；432 个单元的文件意图 43、三批 33、31、31；
/// 433 个单元的文件第二批 39）。
/// 判别力：批大小改成整个文件一批（`UnlinkBatchSize::data_units_per_batch` 的产品那一臂取 `u32::MAX`，`crates/mutations.tsv` 一行）时
/// 432、433 个单元的文件意图之后只有最后一批、放掉整个文件，①③ 红。
// harness-test-granularity:one-scenario 斜率要几份不同单元数的文件一起比，拆开就没有斜率
#[test]
fn truncating_files_of_one_to_four_batches_to_zero_releases_exactly_one_batch_or_the_rest_per_publish_and_its_largest_publish_does_not_grow_with_the_file(
) {
    let one_data_unit = writes_of_every_publish_of_a_truncation_to_zero_in_product_batches(1);
    let one_batch = writes_of_every_publish_of_a_truncation_to_zero_in_product_batches(144);
    let one_batch_and_one_unit =
        writes_of_every_publish_of_a_truncation_to_zero_in_product_batches(145);
    let three_batches = writes_of_every_publish_of_a_truncation_to_zero_in_product_batches(432);
    let three_batches_and_one_unit =
        writes_of_every_publish_of_a_truncation_to_zero_in_product_batches(433);
    let every_file = [
        (1, &one_data_unit),
        (144, &one_batch),
        (145, &one_batch_and_one_unit),
        (432, &three_batches),
        (433, &three_batches_and_one_unit),
    ];
    for (data_units_of_the_file, writes) in every_file {
        println!(
            "{data_units_of_the_file} 个数据单元的文件截到 0，每一次发布（意图、各批）：{writes:?}"
        );
    }
    for (data_units_of_the_file, writes) in every_file {
        assert_eq!(
            data_units_released_by_every_publish(writes),
            data_units_released_by_every_publish_as_the_clause_gives(data_units_of_the_file),
            "{data_units_of_the_file} 个数据单元的文件截到 0，意图那次换下 0 个，之后每批恰好放 {EXTENT_TREE_LOWER_LEAF_DATA_UNITS} 个、最后一批放完剩下的"
        );
    }
    let largest_of_the_one_unit_file = largest_write_calls_of_one_publish(&one_data_unit);
    let write_calls_it_may_add =
        write_calls_the_largest_publish_of_any_file_may_add_over_a_one_unit_file();
    println!("W 式子算的上界：写调用最多的那次至多比一个单元的文件多 {write_calls_it_may_add} 次");
    for (data_units_of_the_file, writes) in every_file {
        assert!(
            largest_write_calls_of_one_publish(writes)
                <= largest_of_the_one_unit_file + write_calls_it_may_add,
            "{data_units_of_the_file} 个数据单元的文件，写调用最多的那次发布 {} 次，至多比一个单元的文件的 {largest_of_the_one_unit_file} 次多 \
             {write_calls_it_may_add} 次（W 式子算的上界）",
            largest_write_calls_of_one_publish(writes)
        );
    }
    assert_eq!(
        [
            write_calls_of_every_publish(&one_data_unit),
            write_calls_of_every_publish(&one_batch),
            write_calls_of_every_publish(&one_batch_and_one_unit),
            write_calls_of_every_publish(&three_batches),
            write_calls_of_every_publish(&three_batches_and_one_unit),
        ],
        [
            vec![31, 21],
            vec![37, 27],
            vec![39, 27],
            vec![43, 33, 31, 31],
            vec![43, 35, 39, 31],
        ],
        "1、144、145、432、433 个数据单元的文件截到 0，每一次发布（意图、各批、最后一批）的写调用：意图那次重写单元 0（声明长度 0）连同罩着它的那片下段叶到根，\
         下段层数随文件长一层就多两次（两块盘各一份）"
    );
}

/// 慢档：2304、4608 个数据单元的文件（16 批、32 批）截到 0，与一个单元的文件比：
/// ① 每一次发布换下的数据单元个数与条款算出来的逐项相同；
/// ② 写调用最多的那次发布，至多比一个单元的文件里的多 [`write_calls_the_largest_publish_of_any_file_may_add_over_a_one_unit_file`]；
/// ③ 写调用最多的那次钉成绝对值：2304 个单元 43 次、4608 个单元 45 次（一个单元的文件 31 次）。
/// 判别力：批大小改成整个文件一批时两份文件意图之后只有最后一批、放掉整个文件，① 红；一批重写的节点随文件长而批数不变的写法，③ 红。
#[test]
#[ignore = "harness 慢档用例：两份几千个数据单元的文件各顺序写一次、再截到 0 分十几到三十几批放完，照 unlink 那一份的慢档两点；debug 下单跑量过 87 秒"]
fn truncating_files_of_sixteen_and_thirty_two_batches_to_zero_releases_exactly_one_batch_or_the_rest_per_publish_and_its_largest_publish_stays_within_a_constant_of_a_one_unit_file(
) {
    let one_data_unit = writes_of_every_publish_of_a_truncation_to_zero_in_product_batches(1);
    let sixteen_batches = writes_of_every_publish_of_a_truncation_to_zero_in_product_batches(2304);
    let thirty_two_batches =
        writes_of_every_publish_of_a_truncation_to_zero_in_product_batches(4608);
    let every_file = [
        (1, &one_data_unit),
        (2304, &sixteen_batches),
        (4608, &thirty_two_batches),
    ];
    for (data_units_of_the_file, writes) in every_file {
        println!(
            "{data_units_of_the_file} 个数据单元的文件截到 0，每一次发布（意图、各批）：{writes:?}"
        );
    }
    for (data_units_of_the_file, writes) in every_file {
        assert_eq!(
            data_units_released_by_every_publish(writes),
            data_units_released_by_every_publish_as_the_clause_gives(data_units_of_the_file),
            "{data_units_of_the_file} 个数据单元的文件截到 0，意图那次换下 0 个，之后每批恰好放 {EXTENT_TREE_LOWER_LEAF_DATA_UNITS} 个、最后一批放完剩下的"
        );
    }
    let largest_of_the_one_unit_file = largest_write_calls_of_one_publish(&one_data_unit);
    let write_calls_it_may_add =
        write_calls_the_largest_publish_of_any_file_may_add_over_a_one_unit_file();
    for (data_units_of_the_file, writes) in every_file {
        assert!(
            largest_write_calls_of_one_publish(writes)
                <= largest_of_the_one_unit_file + write_calls_it_may_add,
            "{data_units_of_the_file} 个数据单元的文件，写调用最多的那次发布 {} 次，至多比一个单元的文件的 {largest_of_the_one_unit_file} 次多 \
             {write_calls_it_may_add} 次（W 式子算的上界）",
            largest_write_calls_of_one_publish(writes)
        );
    }
    assert_eq!(
        [
            largest_write_calls_of_one_publish(&one_data_unit),
            largest_write_calls_of_one_publish(&sixteen_batches),
            largest_write_calls_of_one_publish(&thirty_two_batches),
        ],
        [31, 43, 45],
        "1、2304、4608 个数据单元的文件截到 0，写调用最多的那次发布"
    );
}

/// 不建池时取的挂着的单元数：截到 0 时 size 之外挂着的多于一批才有不是最后一批的批，从挂着 146 个（size 之外 145 个）起；建池那一份的 433 个
/// 多一个单元的 434，慢档最大那份文件多两个单元的 4610，再往上到 2⁴⁰。
const DATA_UNITS_EXISTING_JUDGED_WITHOUT_A_POOL: [u64; 7] =
    [146, 434, 4_610, 9_216, 1_000_000, 1 << 32, 1 << 40];

/// 产品批大小在多大的文件上都是一片下段叶（K2「每一批只从尾部放 K 个单元」、K 与 unlink 共用 `UnlinkBatchSize`）：不建池，直接比发布路径取
/// 下一步的那一个纯函数 `next_truncation_step(挂着的, 0, OneExtentLowerLeaf)`，挂着 146 到 2⁴⁰ 个单元（size 之外 145 到 2⁴⁰ − 1 个）的每一点都是
/// 「这一批放 144 个、留下挂着的 − 144 个」。判别力：批大小改成整个文件一批时每一点都成了放完的最后一批，这一条红。
#[test]
fn the_product_truncation_batch_releases_one_extent_lower_leaf_from_a_file_of_any_size_up_to_two_to_the_fortieth_data_units(
) {
    for data_units_existing in DATA_UNITS_EXISTING_JUDGED_WITHOUT_A_POOL {
        assert_eq!(
            next_truncation_step(data_units_existing, 0, UnlinkBatchSize::OneExtentLowerLeaf),
            TruncationStep::Batch {
                data_units_kept: data_units_existing - EXTENT_TREE_LOWER_LEAF_DATA_UNITS,
            },
            "挂着 {data_units_existing} 个数据单元、截到 0 时这一批恰好放 {EXTENT_TREE_LOWER_LEAF_DATA_UNITS} 个"
        );
    }
}
