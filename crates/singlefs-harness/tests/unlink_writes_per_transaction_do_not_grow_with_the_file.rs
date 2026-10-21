//! unlink 单次发布的写入量不随文件长（`.claude/kb/checks-owed.md` C2（无界操作做成单事务），用户 2026-10-05 同意把 C2 的对象补上 unlink；
//! D8（核心索引结构） 已定项 16 第 2 条「从尾部分批」）：几份单元数不同的文件（快档 1、144、145、432、433 个数据单元，跨过一批 144 个的边界；
//! 慢档 2304、4608 个）各按产品批大小（`UnlinkBatchSize::OneExtentLowerLeaf`）发意图、走完全部批，量每一次发布（意图、每一批）交给设备的
//! 写调用与字节（`TransactionOutput::writes`，发布路径逐个写调用记的账），与每一次换下的数据单元个数。
//! 每一批换下的个数照条款算出来逐项比（意图那次 0 个，之后每批恰好 min(一批 144 个, 剩下的)），不只比「不多于一批」：
//! 「每批取 max(144, ⌈剩下 ÷ 3⌉)」那种批数封顶的写法在 432 个单元以内与产品逐项相同，433 个单元第一批就换下 145 个
//! （判决 `research/prompts/m3-batch19-code-r1-main-verification.md` 第二节 K1 N6 那一行，攻方 `research/prompts/m3-batch19-code-r1-opus-output.md`
//! 「N6」一节）。
//! 建池的几点罩不住批数封顶在 32 以上的那一族（4608 个单元以内与产品逐项相同），另一条用例不建池、直接比产品那一步
//! `unlink::next_unlink_step` 在剩下 145 到 2⁴⁰ 个单元上都只换下一批（判决 `research/prompts/m3-batch21-code-r1-main-verification.md`
//! 第二节「K2 N7 每批换下数随规模变」那一行，攻方 `research/prompts/m3-batch21-code-r1-opus-output.md` 第一节）。

mod common;

use common::{
    build_pool, parameters, sequential_write_in_process_as_instance_one_and_advance_the_pool,
};
use singlefs_core::address::{InodeNumber, InstanceGeneration};
use singlefs_core::admission::ring_window_on_one_device;
use singlefs_core::extent_tree::{
    lower_root_level_for, lower_segment_nodes_of_a_file_without_holes, lower_span_in_data_units,
    ExtentLowerNodePosition, ExtentTreeVersion, ExtentUpperNodePosition,
};
use singlefs_core::pointer::NodePointer;
use singlefs_core::transaction::{
    publish_unlink_intent, publish_unlink_step, ExtentTreeNodeOrigin, PoolWriter,
    TransactionOutput, TransactionUnit, FIRST_INODE_NUMBER,
};
use singlefs_core::unit::data_unit_payload_capacity;
use singlefs_core::unlink::{
    next_unlink_step, plan_the_extent_tree_after_a_deletion_batch, UnlinkBatchSize, UnlinkStep,
};
use singlefs_format::{
    EXTENT_TREE_LOWER_LEAF_DATA_UNITS, JOURNAL_NAMED_ENTRIES_PER_RECORD, ROOT_RING_REGIONS,
    SLOT_BYTES,
};

const FIRST_FILE: InodeNumber = InodeNumber(FIRST_INODE_NUMBER);

/// 新池新建文件那个实例：意图与各批都在这个进程里发。
const INSTANCE_OF_THE_PROCESS: InstanceGeneration = InstanceGeneration(1);

/// 测试池的盘数：每个单元每块盘各写一份（`admission::UnitsOfAPublishToLand` 的文档：第一版每个单元落池里每一块盘），
/// journal 记录这里也按每块盘一份算（往多里算）。上界那一处现核它等于建出来的池的盘数（镜像几块、分配器管几块盘）。
const DEVICES_OF_THE_TEST_POOL: u64 = 2;

/// 任一份文件里写调用最多的那次发布，至多比一个单元的文件里写调用最多的那次多几个写调用：从代码算，不随文件的单元数长。
/// 一次删除类发布每块盘新占的槽至多 W 个（`admission::slots_one_deletion_publish_newly_takes_on_one_device_at_most`，
/// D28（挂载期承诺量） 已定项 1「环窗口」那一条的 W = 24 + (6 + H_退役) + Σ_ℓ min(N_ℓ, 2·(K + 26))，准入按它给环窗口扣空间；
/// N_ℓ 只随盘的单元区定，K 是批大小，H_退役 是退役树的内部层数——这几条用例的池里退役树至多只有根，取 0）。这里不另抄那条式子，按这个池的分配器经 `admission::ring_window_on_one_device` 现算环窗口（= (根环槽数 + 1) × W 个槽）再除回 W。
/// 发布是写时复制，这次写出的每个单元都是新占的落点、至少一槽，每块盘一个写调用：单元的写调用至多「盘数 × W」；
/// 点名项一条 journal 记录装 `JOURNAL_NAMED_ENTRIES_PER_RECORD` 项，记录至多多 ⌈W ÷ 64⌉ 条、每块盘一份。
/// 一个单元的文件那次至少也写一条记录与一个根槽，所以差至多「盘数 × (W + ⌈W ÷ 64⌉)」。
/// 这是上界，不是钉的数：钉的数在各条用例末尾（432、433 个单元的文件最多那次 37、43 次，比一个单元的文件的 31 次多 6、12；
/// 慢档 2304、4608 个单元的文件最多那次 45、53 次，多 14、22。多出来的几乎都在分配记录树根之下那几个节点：
/// 按槽号按位置寻址，一批换下的记录与这次重写的单元的新旧落点散在几片叶上，片数随落点怎么排有起落，W 式子里分配记录树那一项罩着它）。
fn write_calls_the_largest_publish_of_any_file_may_add_over_a_one_unit_file() -> u64 {
    let pool = build_pool("unlink-writes-per-publish-bound-from-the-ring-window");
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
        "上界按测试池的盘数 {DEVICES_OF_THE_TEST_POOL} 乘：建出来的池（镜像块数、分配器管的盘数）就是这么多块盘"
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

/// unlink 的一次发布交给设备的写，与这次换下的数据单元个数。
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
            .expect("删除的每一步只减不增数据单元"),
    }
}

/// 新池新建文件之后把第一个文件顺序写成 `data_units_of_the_file` 个数据单元（最后一个装半个净荷），按产品批大小发意图、
/// 走完全部批；交回每一次发布的写，按先后（第一项是意图那次）。
fn writes_of_every_publish_of_an_unlink_in_product_batches(
    data_units_of_the_file: usize,
) -> Vec<WritesOfOnePublish> {
    let payload_capacity = data_unit_payload_capacity();
    let content: Vec<u8> = (0..(data_units_of_the_file - 1) * payload_capacity
        + payload_capacity / 2)
        .map(|index| u8::try_from((index * 31 + 7) % 251).expect("小于 256"))
        .collect();
    let mut pool = build_pool(&format!(
        "unlink-writes-per-publish-of-{data_units_of_the_file}-data-units"
    ));
    let grown =
        sequential_write_in_process_as_instance_one_and_advance_the_pool(&mut pool, &content);
    assert_eq!(grown.data_pointers.len(), data_units_of_the_file);
    let publish_parameters = parameters();
    let mut writes = Vec::new();
    {
        let previous = pool.output.clone();
        let devices = pool.devices.as_mut().expect("镜像还开着");
        let mut writer = PoolWriter::new(&publish_parameters, devices.as_mut_slice());
        pool.output = publish_unlink_intent(
            &mut writer,
            &mut pool.allocator,
            &previous,
            FIRST_FILE,
            previous
                .inode_record
                .expect("意图之前第一个文件那条记录在")
                .write_time_seconds,
            INSTANCE_OF_THE_PROCESS,
        )
        .expect("意图");
        writes.push(writes_of_one_publish(&previous, &pool.output));
    }
    // 迭代上界：每一步至少删一个单元，文件 `data_units_of_the_file` 个单元。
    while !pool.output.deleted_inodes.is_empty() {
        assert!(writes.len() <= data_units_of_the_file + 1, "删除步数有界");
        let previous = pool.output.clone();
        let devices = pool.devices.as_mut().expect("镜像还开着");
        let mut writer = PoolWriter::new(&publish_parameters, devices.as_mut_slice());
        pool.output = publish_unlink_step(
            &mut writer,
            &mut pool.allocator,
            &previous,
            FIRST_FILE,
            UnlinkBatchSize::OneExtentLowerLeaf,
            INSTANCE_OF_THE_PROCESS,
        )
        .expect("删除的下一步");
        writes.push(writes_of_one_publish(&previous, &pool.output));
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

/// 每一次发布换下的数据单元个数照条款该是多少（D8（核心索引结构） 已定项 16 第 2 条「从尾部分批」，产品每批 K 取一片下段叶的单元数
/// `EXTENT_TREE_LOWER_LEAF_DATA_UNITS`）：意图那次 0 个，之后每批恰好 min(K, 剩下的)，剩下 0 个为止。按先后，第一项是意图那次。
fn data_units_released_by_every_publish_as_the_clause_gives(
    data_units_of_the_file: usize,
) -> Vec<usize> {
    let data_units_per_batch =
        usize::try_from(EXTENT_TREE_LOWER_LEAF_DATA_UNITS).expect("144 装得进 usize");
    let mut released_by_every_publish = vec![0];
    let mut data_units_remaining = data_units_of_the_file;
    // 迭代上界：每一轮剩下的至少少 1（K ≥ 1），⌈单元数 ÷ K⌉ 轮走到 0。
    while data_units_remaining > 0 {
        let released_by_this_batch = data_units_remaining.min(data_units_per_batch);
        released_by_every_publish.push(released_by_this_batch);
        data_units_remaining -= released_by_this_batch;
    }
    released_by_every_publish
}

fn data_units_released_by_every_publish(writes: &[WritesOfOnePublish]) -> Vec<usize> {
    writes
        .iter()
        .map(|publish| publish.data_units_released)
        .collect()
}

/// 五份文件（1、144、145、432、433 个数据单元）：
/// ① 每一次发布换下的数据单元个数与条款算出来的逐项相同（意图 0 个，之后每批恰好 min(144, 剩下的)）；
/// ② 每份文件里写调用最多的那次发布，至多比一个单元的文件里的多 [`write_calls_the_largest_publish_of_any_file_may_add_over_a_one_unit_file`]
/// （从 W 式子算，不随文件长）；
/// ③ 每一次发布的写调用钉成绝对值（发布路径今天写出的形状：一个单元的文件意图 31、最后一批 25；432 个单元的文件意图 33、三批 37、35、33；
/// 433 个单元的文件四批里第二批 43）。
/// 判别力：批大小改成整个文件一批（`UnlinkBatchSize::data_units_per_batch` 的产品那一臂取 `u32::MAX`）时，145、432、433 个单元的文件
/// 意图之后只有一批、换下整个文件，①③ 红。这几个规模上整个文件一批的单次写调用不比分批的多（副本里量过：432 个单元的文件
/// 那一批 29 次，分批时最多 37 次），② 判不出这一刀（② 是 W 式子的上界，只拦超出准入预留的那一种）；2304、4608 个单元那两点在慢档
/// [`unlink_of_files_of_sixteen_and_thirty_two_batches_releases_exactly_one_batch_or_the_rest_per_publish_and_its_largest_publish_stays_within_a_constant_of_a_one_unit_file`]。
/// 每批取 max(144, ⌈剩下 ÷ 3⌉) 时 433 个单元的文件第一批换下 145 个，① 红（432 个单元以内与产品逐项相同）。
// harness-test-granularity:one-scenario 斜率要几份不同单元数的文件一起比，拆开就没有斜率
#[test]
fn unlink_of_files_of_one_to_four_batches_releases_exactly_one_batch_or_the_rest_per_publish_and_its_largest_publish_does_not_grow_with_the_file(
) {
    let one_data_unit = writes_of_every_publish_of_an_unlink_in_product_batches(1);
    let one_batch = writes_of_every_publish_of_an_unlink_in_product_batches(144);
    let one_batch_and_one_unit = writes_of_every_publish_of_an_unlink_in_product_batches(145);
    let three_batches = writes_of_every_publish_of_an_unlink_in_product_batches(432);
    let three_batches_and_one_unit = writes_of_every_publish_of_an_unlink_in_product_batches(433);
    let every_file = [
        (1, &one_data_unit),
        (144, &one_batch),
        (145, &one_batch_and_one_unit),
        (432, &three_batches),
        (433, &three_batches_and_one_unit),
    ];
    for (data_units_of_the_file, writes) in every_file {
        println!("{data_units_of_the_file} 个数据单元的文件，每一次发布（意图、各批）：{writes:?}");
    }
    for (data_units_of_the_file, writes) in every_file {
        assert_eq!(
            data_units_released_by_every_publish(writes),
            data_units_released_by_every_publish_as_the_clause_gives(data_units_of_the_file),
            "{data_units_of_the_file} 个数据单元的文件，意图那次换下 0 个，之后每批恰好换下 min({EXTENT_TREE_LOWER_LEAF_DATA_UNITS}, 剩下的) 个"
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
            vec![31, 29],
            vec![31, 33],
            vec![31, 31, 33],
            vec![33, 37, 35, 37],
            vec![33, 39, 43, 39, 33],
        ],
        "1、144、145、432、433 个数据单元的文件，每一次发布（意图、各批）的写调用：最后一批唯一那片叶容器身份消失，\
         退役树多写容器与根两个单元、两块盘各一份，比第 49 批多 4 次"
    );
}

/// 慢档：2304、4608 个数据单元的文件（16 批、32 批，判决 `research/prompts/m3-batch19-code-r1-main-verification.md` 第三节第 3 条「慢档 2304、4608
/// 两点量斜率」），与一个单元的文件比：
/// ① 每一次发布换下的数据单元个数与条款算出来的逐项相同（意图 0 个，之后每批恰好 144 个）；
/// ② 写调用最多的那次发布，至多比一个单元的文件里的多 [`write_calls_the_largest_publish_of_any_file_may_add_over_a_one_unit_file`]
/// （从 W 式子算，与文件的单元数无关）；
/// ③ 两份文件写调用最多的那次钉成绝对值：2304 个单元 45 次、4608 个单元 53 次（一个单元的文件 31 次）。
/// 判别力：每批取 max(144, ⌈剩下 ÷ 3⌉) 时两份文件意图之后只有三批、第一批换下 768 与 1536 个，① 红；批大小改成整个文件一批时同样 ① 红。
/// 一批重写的节点随文件长（例如每批把 extent 树下段或中央映射树整棵重写）而批数不变的写法，③ 红；长到超过 W 式子罩得住的，② 也红。
#[test]
#[ignore = "harness 慢档用例：两份几千个数据单元的文件各顺序写一次、再分十几到三十几批删完；判决点名放慢档，debug 下单跑量过 28、55、63 秒（随机器负载）"]
fn unlink_of_files_of_sixteen_and_thirty_two_batches_releases_exactly_one_batch_or_the_rest_per_publish_and_its_largest_publish_stays_within_a_constant_of_a_one_unit_file(
) {
    let one_data_unit = writes_of_every_publish_of_an_unlink_in_product_batches(1);
    let sixteen_batches = writes_of_every_publish_of_an_unlink_in_product_batches(2304);
    let thirty_two_batches = writes_of_every_publish_of_an_unlink_in_product_batches(4608);
    let every_file = [
        (1, &one_data_unit),
        (2304, &sixteen_batches),
        (4608, &thirty_two_batches),
    ];
    for (data_units_of_the_file, writes) in every_file {
        println!("{data_units_of_the_file} 个数据单元的文件，每一次发布（意图、各批）：{writes:?}");
    }
    for (data_units_of_the_file, writes) in every_file {
        assert_eq!(
            data_units_released_by_every_publish(writes),
            data_units_released_by_every_publish_as_the_clause_gives(data_units_of_the_file),
            "{data_units_of_the_file} 个数据单元的文件，意图那次换下 0 个，之后每批恰好换下 min({EXTENT_TREE_LOWER_LEAF_DATA_UNITS}, 剩下的) 个"
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
            largest_write_calls_of_one_publish(&one_data_unit),
            largest_write_calls_of_one_publish(&sixteen_batches),
            largest_write_calls_of_one_publish(&thirty_two_batches),
        ],
        [31, 45, 53],
        "1、2304、4608 个数据单元的文件，写调用最多的那次发布"
    );
}

/// 不建池时取的剩下单元数：建池那两条取到的 145、433 两点，慢档最大那份文件多一个单元的 4609，再往上到 2⁴⁰。
const DATA_UNITS_REMAINING_JUDGED_WITHOUT_A_POOL: [u64; 7] =
    [145, 433, 4_609, 9_216, 1_000_000, 1 << 32, 1 << 40];

/// 产品批大小在多大的文件上都是一片下段叶（D8（核心索引结构） 已定项 16 第 2 条「每批释放尾部 K 个单元」、K 产品取
/// `EXTENT_TREE_LOWER_LEAF_DATA_UNITS`）：不建池，直接比发布路径取下一步的那一个纯函数 `next_unlink_step(剩下, OneExtentLowerLeaf)`，
/// 剩下 145 到 2⁴⁰ 个单元的每一点都是「这一批换下 144 个、留下剩下 − 144 个、size 改成留下的单元数 × 净荷容量」。
/// 判别力：每批取 max(144, ⌈剩下 ÷ 32⌉)（批数封顶 32，攻方 `research/prompts/m3-batch21-code-r1-opus-model/patches/c2cap32.patch`）时
/// 剩下 4609 个单元那一点这一批换下 145 个，这一条红；那一刀在建池的 1 到 4608 个单元上与产品逐项相同，另两条照绿。
/// 批数封顶在 N 而 N 小于 2⁴⁰ ÷ 144 的同一族都在 2⁴⁰ 那一点红。批大小不经这个函数、在发布路径另算的写法这一条碰不到，靠建池的那两条。
/// 不建池时量下段第二层内部节点那一段的文件（D8（核心索引结构） 已定项 16 第 2 条；欠账 C2（无界操作做成单事务） 那一行
/// 「建池取样点最大 4 608 个单元，到不了 extent 下段长出第二层内部节点（> 21 168 个单元）那一段」）：
/// 21 313 个单元（第一批之后留 21 169 个，下段根仍在第二层）与 42 481 个单元（留 42 337 个，第二层根下三个第一层节点）。
/// 21 168 = 一片下段叶 144 个单元 × 扇出 147。
const DATA_UNITS_OF_FILES_PAST_THE_SECOND_LOWER_INTERNAL_LEVEL: [u64; 2] = [21_313, 42_481];

/// 一个 `data_units` 个单元、没有洞的第一个文件那一版 extent 树：下段每个节点、上段只有根兼叶（第一版只有第一个文件有条目）。
/// 删除一批的规划只看位置与上一版有没有这个节点（指针不读盘），指针一律取空根。
fn extent_tree_of_the_first_file_without_holes(data_units: u64) -> ExtentTreeVersion {
    ExtentTreeVersion {
        upper_nodes: vec![(
            ExtentUpperNodePosition { level: 0, index: 0 },
            NodePointer::empty_root(),
        )],
        lower_nodes: lower_segment_nodes_of_a_file_without_holes(data_units)
            .into_iter()
            .map(|position| (position, NodePointer::empty_root()))
            .collect(),
    }
}

/// 下段长出第二层内部节点之后（> 21 168 个单元）删除的一批仍只重写被截掉那一片下段叶与它的祖先（K-R3-1，`unlink::plan_the_extent_tree_after_a_deletion_batch`
/// 的文档）：不建池，照产品批大小取第一批（`next_unlink_step`），交给发布路径用的那一份规划，数这一批的 extent 树写。
/// 每份文件：① 这一版重写的下段节点恰好每层一个、都是留下那一段每层最末那一个（3 个：叶、第一层、第二层根）；
/// ② 上一版被换下的下段节点 4 个（被截的两片叶、第一层最末那一个、第二层根），加上段根兼叶；③ 上段只重写根兼叶。
/// 建池那几条的文件最大 4 608 个单元，下段至多一层内部节点，这一段它们走不到（欠账 C2 那一行的材料）。
/// 判别力：下段根在第二层时把第一层整层（连全在留下那一段里的）都重写，建池那几条逐项照绿（它们的下段没有第二层），这一条 ① ② 红。
#[test]
fn the_product_unlink_batch_rewrites_one_extent_lower_node_per_level_on_files_past_the_second_lower_internal_level(
) {
    for data_units_of_the_file in DATA_UNITS_OF_FILES_PAST_THE_SECOND_LOWER_INTERNAL_LEVEL {
        let UnlinkStep::Batch {
            data_units_kept, ..
        } = next_unlink_step(data_units_of_the_file, UnlinkBatchSize::OneExtentLowerLeaf)
        else {
            panic!("{data_units_of_the_file} 个单元的文件第一步是一批，不是最后一批")
        };
        assert_eq!(
            (
                lower_root_level_for(data_units_of_the_file),
                lower_root_level_for(data_units_kept)
            ),
            (2, 2),
            "{data_units_of_the_file} 个单元的文件与这一批之后留下的 {data_units_kept} 个，下段根都在第二层"
        );
        let plan = plan_the_extent_tree_after_a_deletion_batch(
            &extent_tree_of_the_first_file_without_holes(data_units_of_the_file),
            &std::collections::BTreeSet::from([FIRST_INODE_NUMBER]),
            FIRST_INODE_NUMBER,
            data_units_of_the_file,
            data_units_kept,
        );
        let rewritten_lower_nodes: Vec<ExtentLowerNodePosition> = plan
            .lower_nodes
            .iter()
            .filter(|(_, origin)| *origin == ExtentTreeNodeOrigin::RewrittenThisPublish)
            .map(|(position, _)| *position)
            .collect();
        let last_node_of_each_level_of_the_kept_file: Vec<ExtentLowerNodePosition> = (0..=2u8)
            .map(|level| ExtentLowerNodePosition {
                level,
                index: (data_units_kept - 1) / lower_span_in_data_units(level),
            })
            .collect();
        assert_eq!(
            rewritten_lower_nodes, last_node_of_each_level_of_the_kept_file,
            "{data_units_of_the_file} 个单元的文件第一批：下段只重写留下那一段每层最末那一个（被截掉那一片叶与它的祖先）"
        );
        let replaced_lower_nodes = plan
            .replaced_previous_roles
            .iter()
            .filter(|role| matches!(role, TransactionUnit::ExtentLowerNode(_)))
            .count();
        assert_eq!(
            (
                replaced_lower_nodes,
                plan.replaced_previous_roles.len() - replaced_lower_nodes
            ),
            (4, 1),
            "{data_units_of_the_file} 个单元的文件第一批：换下上一版下段 4 个节点（被截的两片叶、第一层最末那一个、第二层根）与上段根兼叶：{:?}",
            plan.replaced_previous_roles
        );
        assert_eq!(
            plan.upper_nodes,
            vec![(
                ExtentUpperNodePosition { level: 0, index: 0 },
                ExtentTreeNodeOrigin::RewrittenThisPublish
            )],
            "{data_units_of_the_file} 个单元的文件第一批：上段只重写根兼叶"
        );
    }
}

#[test]
fn the_product_unlink_batch_releases_one_extent_lower_leaf_from_a_file_of_any_size_up_to_two_to_the_fortieth_data_units(
) {
    let payload_capacity = u64::try_from(data_unit_payload_capacity()).expect("净荷容量装得进 u64");
    for data_units_remaining in DATA_UNITS_REMAINING_JUDGED_WITHOUT_A_POOL {
        let data_units_kept = data_units_remaining - EXTENT_TREE_LOWER_LEAF_DATA_UNITS;
        assert_eq!(
            next_unlink_step(data_units_remaining, UnlinkBatchSize::OneExtentLowerLeaf),
            UnlinkStep::Batch {
                data_units_kept,
                size_after_the_batch_in_bytes: data_units_kept * payload_capacity,
            },
            "剩下 {data_units_remaining} 个数据单元时这一批恰好换下 {EXTENT_TREE_LOWER_LEAF_DATA_UNITS} 个、留下 {data_units_kept} 个"
        );
    }
}
