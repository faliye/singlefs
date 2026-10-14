//! harness 测试文件 `crates/singlefs-harness/tests/many_inodes_write_and_read.rs` 的场景函数：搭场景、核实现的行为，池级 checker 要判的镜像经 [`PoolCheckerImages`] 交出去
//! （做法见 `common_images_for_the_checker_tier/mod.rs`）。harness 档那份测试文件与 checker 档的
//! `crates/singlefs-checker-tier/tests/crash_given_states_of_the_images_harness_scenarios_hand_out.rs` 各经 `#[path]` 引这一份。
#![allow(dead_code, reason = "两档各只调其中一部分")]

use crate::common::{build_pool, parameters, BuiltPool, FIXED_WRITE_TIME_SECONDS, IMAGE_BYTES};
use crate::common_images_for_the_checker_tier::{PoolCheckerExpectation, PoolCheckerImages};
use singlefs_checker::check_journal_record;
use singlefs_core::address::{
    CheckpointTxg, DataUnitIndexInFile, DeviceIdentity, InstanceGeneration,
};
use singlefs_core::allocation_record_tree::AllocationRecordTreeNodePosition;
use singlefs_core::inode_tree::InodeLeafContainerIndexInTree;
use singlefs_core::journal::record_offset;
use singlefs_core::records::InodeRecord;
use singlefs_core::recovery::{
    rebuild_version, recover, JournalPolicy, PoolReader, RebuiltVersion,
};
use singlefs_core::transaction::{
    publish_new_inodes, PoolWriter, PublishError, TransactionOutput, TransactionUnit,
};
use singlefs_core::unit::unit_filesystem_identifier;
use singlefs_format::{
    INODE_LEAF_RECORDS, JOURNAL_NAMED_ENTRIES_PER_RECORD, JOURNAL_RECORD_BYTES,
    JOURNAL_RING_DEFAULT_BYTES,
};
use singlefs_harness::memory_pool::MemoryPool;
use singlefs_harness::RecordedOperationKind;

/// 分配记录树根之下 (层级, 盘, 同盘同层序号) 那个节点的角色（D8（核心索引结构） 已定项 14：按绝对槽号按位置寻址）。
pub fn allocation_record_tree_node(
    level: u8,
    device: u32,
    index_in_device: u64,
) -> TransactionUnit {
    TransactionUnit::AllocationTreeNodeBelowTheRoot(AllocationRecordTreeNodePosition {
        level,
        device: DeviceIdentity(device),
        index_in_device,
    })
}

/// 新池新建文件之后树里已有的那一条（inode 1）。建 N 个之后树里共 N + 1 条、水位 N + 2。
pub const INODES_BEFORE: u64 = 1;

/// 这一轮建几个：正好让树里的记录数跨过一容器 233 条（1 + 233 = 234 ⇒ 第 234 个 inode 触发分裂）。
pub const NEW_INODES_CROSSING_THE_CONTAINER: u64 = INODE_LEAF_RECORDS;

pub fn create_inodes(
    pool: &mut BuiltPool,
    new_inode_count: u64,
    write_time_seconds: u64,
) -> Result<TransactionOutput, PublishError> {
    let publish_parameters = parameters();
    let previous = pool.output.clone();
    let devices = pool.devices.as_mut().expect("镜像还开着");
    let mut writer = PoolWriter::new(&publish_parameters, devices.as_mut_slice());
    publish_new_inodes(
        &mut writer,
        &mut pool.allocator,
        &previous,
        new_inode_count,
        write_time_seconds,
        InstanceGeneration(1),
    )
}

/// 建 N 个 inode 的那次发布之后，盘上这一版的 inode 树：两片叶容器、左半身份与记录不变、
/// 水位 = N + 2；冷启动逐个读回每条记录，字段等于写入；池级 checker 一条违例都没有，
/// I-9.6（水位大于两处最大号） 与 I-9.12（分隔 key 落在孩子区间之外） 真被评估过且成立。
pub fn creating_inodes_across_the_two_hundred_thirty_three_threshold_splits_at_the_end_and_reads_back_cold(
    images: &mut PoolCheckerImages,
) {
    let mut pool = build_pool("parallel-line-three-split");
    let before_the_split = pool.output.clone();
    let leftmost = TransactionUnit::InodeLeafContainer(InodeLeafContainerIndexInTree::LEFTMOST);
    let second_container =
        TransactionUnit::InodeLeafContainer(InodeLeafContainerIndexInTree::of_position(1));
    assert_eq!(
        before_the_split.inode_number_watermark(),
        INODES_BEFORE + 1,
        "新池新建文件之后水位是 2（字节表六那一行）"
    );
    let identity_before_the_split = before_the_split.inode_leaf_containers[0].contents.identity;
    let records_before_the_split = before_the_split.inode_leaf_containers[0]
        .contents
        .records
        .clone();

    let write_time_seconds = FIXED_WRITE_TIME_SECONDS + 120;
    let after = create_inodes(
        &mut pool,
        NEW_INODES_CROSSING_THE_CONTAINER,
        write_time_seconds,
    )
    .expect("建 233 个 inode");
    pool.output = after.clone();
    let split_txg = after.root.checkpoint_txg;
    assert_eq!(
        split_txg,
        CheckpointTxg(before_the_split.root.checkpoint_txg.0 + 1),
        "一次发布：txg 加一"
    );

    // ① 树的形状：两片叶容器，左半满 233 条、右半从第 234 个 inode 起。
    assert_eq!(after.inode_leaf_containers.len(), 2, "跨过 233 ⇒ 分裂一次");
    assert_eq!(
        after.inode_leaf_containers[0].contents.records.len(),
        usize::try_from(INODE_LEAF_RECORDS).expect("233"),
        "左半装满 233 条"
    );
    assert_eq!(
        after.inode_leaf_containers[1].contents.records.len(),
        1,
        "右半从触发分裂的那条新记录起"
    );

    // ② C116 的三条纪律：左半保留全部原有记录与身份，右半是新容器、号 = 触发分裂那条记录的 inode 号、出生代 = 本次发布的 txg。
    assert_eq!(
        after.inode_leaf_containers[0].contents.identity, identity_before_the_split,
        "左半身份不变：出生树、打包记录类型、容器号、容器出生代四段逐项等于分裂之前那一版"
    );
    assert_eq!(
        after.inode_leaf_containers[0].contents.records[..records_before_the_split.len()],
        records_before_the_split[..],
        "左半保留全部原有记录（分裂点取末尾，一条都不搬到右半）"
    );
    assert_eq!(
        after.inode_leaf_containers[1].contents.identity.container,
        INODES_BEFORE + NEW_INODES_CROSSING_THE_CONTAINER,
        "右半容器号 = 触发分裂那条新记录的 inode 号（第 234 个）"
    );
    assert_eq!(
        after.inode_leaf_containers[1]
            .contents
            .identity
            .container_birth,
        split_txg,
        "右半出生代 = 本次发布的 checkpoint_txg"
    );
    assert_eq!(
        after.inode_leaf_containers[1].contents.identity.birth_tree,
        after.inode_leaf_containers[0].contents.identity.birth_tree,
        "右半出生树 = 执行这次分裂的那棵树"
    );

    // ③ 这次重写了哪些角色：两片叶容器 + inode 根 + 四个固定点角色的根（分配记录树、记账、映射、树表），加分配记录树按位置寻址（D8（核心索引结构） 已定项 14）
    // 重写的根之下那几个节点（4 GiB 两块盘上根在第 2 层，改的记录都在两块盘各自的叶 61 里，先叶后根）；数据单元与 extent 树根照抄。
    assert_eq!(
        after.rewritten,
        vec![
            leftmost,
            second_container,
            TransactionUnit::InodeRoot,
            allocation_record_tree_node(0, 0, 61),
            allocation_record_tree_node(0, 1, 61),
            allocation_record_tree_node(1, 0, 0),
            allocation_record_tree_node(1, 1, 0),
            TransactionUnit::AllocationTree,
            TransactionUnit::AccountingTree,
            TransactionUnit::MappingTree,
            TransactionUnit::TreeTable,
        ],
        "建 inode 不写用户数据：数据单元与 extent 树根这次不重写"
    );
    assert_eq!(after.record.named.len(), 11, "点名项 = 这次重写的单元数");
    for carried in [
        TransactionUnit::Data(DataUnitIndexInFile::FIRST),
        TransactionUnit::ExtentRoot,
    ] {
        assert_eq!(
            after.unit(carried).slot,
            before_the_split.unit(carried).slot,
            "{}：照抄、不换落点",
            carried.tag()
        );
        assert_eq!(
            after.unit(carried).bytes,
            before_the_split.unit(carried).bytes,
            "{}：照抄、一个字节都不变",
            carried.tag()
        );
    }
    assert_eq!(
        after.released,
        [
            leftmost,
            TransactionUnit::InodeRoot,
            allocation_record_tree_node(0, 0, 61),
            allocation_record_tree_node(0, 1, 61),
            allocation_record_tree_node(1, 0, 0),
            allocation_record_tree_node(1, 1, 0),
            TransactionUnit::AllocationTree,
            TransactionUnit::AccountingTree,
            TransactionUnit::MappingTree,
            TransactionUnit::TreeTable,
        ]
        .iter()
        .map(|identity| singlefs_core::allocator::Placement {
            slot: before_the_split.unit(*identity).slot,
            span: identity.span_slots(),
        })
        .collect::<Vec<_>>(),
        "释放的是这次重写的角色在上一版的落点；新建的那片右半容器上一版里没有 ⇒ 不释放"
    );

    // ④ 记账里的 inode 号水位 = N + 2。
    assert_eq!(
        after.inode_number_watermark(),
        NEW_INODES_CROSSING_THE_CONTAINER + 2,
        "建 N 个之后水位 = N + 2（D5 已定项 4 第 12 项：下一个可用号）"
    );

    // ⑤ 冷启动逐个读回记录，字段等于写入。
    let image = pool.memory_pool();
    let reopened = pool.reopen_cold();
    let rebuilt = rebuild_version(&reopened, &after.root, Some(after.record.clone()))
        .expect("冷启动重建这一版");
    let RebuiltVersion::WithFile(cold) = rebuilt else {
        panic!("这一版有文件");
    };
    let cold_records: Vec<InodeRecord> = cold
        .inode_leaf_containers
        .iter()
        .flat_map(|container| container.contents.records.iter().copied())
        .collect();
    assert_eq!(
        u64::try_from(cold_records.len()).expect("记录数"),
        INODES_BEFORE + NEW_INODES_CROSSING_THE_CONTAINER,
        "树里共 1 + N 条记录"
    );
    for (position, record) in cold_records.iter().enumerate() {
        let inode = u64::try_from(position).expect("序号") + 1;
        assert_eq!(record.inode, inode, "记录按 inode 号升序、号连着发");
        if inode == 1 {
            continue;
        }
        assert_eq!(
            *record,
            InodeRecord {
                inode,
                object_birth: split_txg,
                nlink: singlefs_core::records::NLINK_OF_A_LIVE_INODE_WITHOUT_DIRECTORIES,
                size: 0,
                change_count: split_txg.0,
                write_time_seconds,
            },
            "inode {inode} 的记录逐字段等于写入"
        );
    }
    assert_eq!(
        cold.inode_leaf_containers[0].contents.identity, identity_before_the_split,
        "冷启动读回的左半身份仍是分裂之前那一份（期望态带着分裂前的身份）"
    );
    assert_eq!(
        cold.inode_leaf_containers
            .iter()
            .map(|container| container.contents.identity)
            .collect::<Vec<_>>(),
        after
            .inode_leaf_containers
            .iter()
            .map(|container| container.contents.identity)
            .collect::<Vec<_>>(),
        "两片容器的身份与写者算出来的逐项相同"
    );
    assert_eq!(
        cold.inode_number_watermark(),
        NEW_INODES_CROSSING_THE_CONTAINER + 2,
        "冷启动从盘上读回的水位"
    );
    // 整条恢复路径（走读同款：inode 树 → extent 树 → 数据单元）照样读回第一个文件的内容。
    let report = recover(&reopened, JournalPolicy::Consult);
    assert_eq!(
        report.outcome,
        singlefs_core::recovery::RecoveryOutcome::FileRead {
            root: (InstanceGeneration(1), split_txg),
            content: crate::common::file_content(),
        },
        "第一个文件的内容不受建 inode 影响"
    );

    // ⑥ 池级 checker：一条违例都没有，I-9.6 与 I-9.12 真被评估过且成立。
    // 这几条要真被评估过、且成立
    images.hand_out(
        "分裂之后",
        || image.clone(),
        PoolCheckerExpectation::no_invariant_violated()
            .and_holds("I-9.6")
            .and_holds("I-9.12")
            .and_holds("I-9.4")
            .and_holds("I-9.2")
            .and_holds("I-9.13")
            .and_holds("I-3.1")
            .and_holds("I-5.1"),
    );
}

/// 一次只建一个 inode、离 233 还远：树里仍只有一片叶容器，重写的也只有它与根 + 四个固定点角色的根（分配记录树、记账、映射、树表；加分配记录树根之下那几个节点）。
/// 这一档钉住「没跨过门槛就不分裂」——分裂条件写成「≥ 233」或「> 0」时这条红。
pub fn creating_one_inode_below_the_threshold_keeps_one_container_and_bumps_the_watermark_by_one(
    images: &mut PoolCheckerImages,
) {
    let mut pool = build_pool("parallel-line-three-one");
    let before = pool.output.clone();
    let after = create_inodes(&mut pool, 1, FIXED_WRITE_TIME_SECONDS + 120).expect("建一个 inode");
    pool.output = after.clone();

    assert_eq!(after.inode_leaf_containers.len(), 1, "没跨过 233：不分裂");
    assert_eq!(
        after.inode_leaf_containers[0].contents.records.len(),
        2,
        "一片容器里两条记录"
    );
    assert_eq!(
        after.inode_leaf_containers[0].contents.identity,
        before.inode_leaf_containers[0].contents.identity,
        "COW 重写身份不变（容器号仍是 1、出生代仍是建树那次的 3）"
    );
    assert_eq!(
        after.inode_leaf_containers[0].contents.records[1].inode,
        before.inode_number_watermark(),
        "新 inode 的号 = 上一版记账里的水位"
    );
    assert_eq!(after.inode_number_watermark(), 3, "水位加一");
    assert_eq!(
        after.rewritten,
        vec![
            TransactionUnit::InodeLeafContainer(InodeLeafContainerIndexInTree::LEFTMOST),
            TransactionUnit::InodeRoot,
            allocation_record_tree_node(0, 0, 61),
            allocation_record_tree_node(0, 1, 61),
            allocation_record_tree_node(1, 0, 0),
            allocation_record_tree_node(1, 1, 0),
            TransactionUnit::AllocationTree,
            TransactionUnit::AccountingTree,
            TransactionUnit::MappingTree,
            TransactionUnit::TreeTable,
        ]
    );
    images.hand_out(
        "建一个 inode 之后",
        || pool.memory_pool(),
        PoolCheckerExpectation::no_invariant_violated(),
    );
}

/// 录制流前 `operation_count` 步施加到两块空内存盘上：崩在那一步之前的镜像。
pub fn memory_pool_of_the_first(pool: &BuiltPool, operation_count: usize) -> MemoryPool {
    let mut image = MemoryPool::with_devices(&[DeviceIdentity(0), DeviceIdentity(1)], IMAGE_BYTES);
    image.apply(&pool.retained_operations()[..operation_count]);
    image
}

/// 录制流里第一次写到 `counter` 那条 journal 记录的槽（两盘同偏移）的那一步：崩在它之前，那条记录一份都没落。
pub fn first_write_of_the_record(pool: &BuiltPool, counter: u64) -> usize {
    let offset = record_offset(counter, JOURNAL_RING_DEFAULT_BYTES);
    pool.retained_operations()
        .iter()
        .position(|retained| {
            retained.operation.kind == RecordedOperationKind::Write
                && retained.operation.offset == offset
        })
        .expect("那条记录写过")
}

/// 录制流里最后一次根槽 FUA 写的下标：崩在它之前，这次发布的单元与记录都落了、根没落。
pub fn index_of_the_last_root_slot_write(pool: &BuiltPool) -> usize {
    pool.retained_operations()
        .iter()
        .rposition(|retained| {
            retained.operation.kind == RecordedOperationKind::WriteForceUnitAccess
        })
        .expect("至少一次发布写过根槽")
}

/// 一次建的 inode 多到这次要点名的单元超过一条 journal 记录装得下的 64 项：建 inode 是一个事务、一个数据单元都不写，
/// 它点名的全是共享的提交内生块。D23（journal 的角色与格式） 已定项 17（用户 2026-09-24 定）：**末条再跨记录**——
/// 按 bump 次序装满 64 项再开下一条；两条都属于这一个事务（已定项 7：同一事务的记录共享事务号，提交标记只在它的最后一条），
/// 本次发布内序号 1、2（已定项 4），只有第二条带「本次发布末条」标志（记录标志位 0，已定项 17）。
/// 盘上的字段用 checker 的独立解析从盘上读（不信写者交回的）。之后：
/// - 发布做完：冷启动读回 1 + N 条 inode 记录；池级 checker 一条违例都没有，I-8.8（前缀里的事务不被切开） 这一回有了对象
///   （一个事务两条记录）、I-8.9（一次发布的记录序号连续且只有末条带标志） 都真被评估过且成立；
/// - 崩在根槽 FUA 之前（两条记录都落了）：恢复由记录施加这一版——第一条不带提交标记，恢复不在它那里停（一事务一条时的
///   「不带提交标记就停」在这一格少施加一整次发布）；
/// - 崩在第二条记录之前（第一条两份都落了）：末条没到，这次发布整体不施加，走的还是上一版的根。
pub fn publish_naming_more_units_than_one_journal_record_holds_spills_its_one_transaction_over_two_records_and_only_the_second_ends_the_publish(
    images: &mut PoolCheckerImages,
) {
    let mut pool = build_pool("parallel-line-three-spill-over-two-records");
    let before = pool.output.clone();
    // 非叶容器的角色有十一个：inode 根、记账树、映射树、树表，加分配记录树按位置寻址（D8（核心索引结构） 已定项 14）重写的七个节点——
    // 这次新写的五十几片容器从 50253 往后排、越过 50344，改的记录落在两块盘各自的叶 61 与叶 62 里，
    // 加两块盘各自的第 1 层节点 0 与根 ⇒ 容器数超过 64 − 11 = 53 就点不下一条记录。
    let containers_that_fit = JOURNAL_NAMED_ENTRIES_PER_RECORD - 11;
    let new_inode_count = (containers_that_fit + 1) * INODE_LEAF_RECORDS - INODES_BEFORE;
    let named_unit_capacity = usize::try_from(JOURNAL_NAMED_ENTRIES_PER_RECORD).expect("64");

    let after = create_inodes(&mut pool, new_inode_count, FIXED_WRITE_TIME_SECONDS + 120)
        .expect("点名项装不下一条记录时末条再跨记录，不拒");
    pool.output = after.clone();
    assert_eq!(
        after.rewritten.len(),
        named_unit_capacity + 1,
        "这次重写 54 片叶容器 + 11 个非叶容器角色 = 65 个单元"
    );
    assert_eq!(
        after.earlier_records_of_this_publish.len(),
        1,
        "65 项 = 装满的一条 64 项 + 跨出去的一条 1 项"
    );
    let first_counter = after.earlier_records_of_this_publish[0].record.counter;
    assert_eq!(
        (first_counter, after.record.counter),
        (before.record.counter + 1, before.record.counter + 2),
        "两条记录紧接在上一版那条之后"
    );

    // ① 盘上两条记录的字段（checker 的独立解析）。
    let image = pool.memory_pool();
    let filesystem_identifier = unit_filesystem_identifier(&parameters().filesystem_identifier);
    let record_bytes = usize::try_from(JOURNAL_RECORD_BYTES).expect("4096");
    let on_disk: Vec<(u64, u64, u8, u32, u8, usize)> = [first_counter, after.record.counter]
        .into_iter()
        .map(|counter| {
            let bytes = PoolReader::read(
                &image,
                DeviceIdentity(0),
                record_offset(counter, JOURNAL_RING_DEFAULT_BYTES),
                record_bytes,
            )
            .expect("记录落了");
            let view = check_journal_record(&bytes, filesystem_identifier).expect("记录自证过");
            (
                view.checkpoint_txg,
                view.transaction,
                view.commit_marker_byte,
                view.ordinal_within_publish,
                view.record_flags_byte,
                view.named.len(),
            )
        })
        .collect();
    let txg = after.root.checkpoint_txg.0;
    let transaction = before.highest_transaction_number_in_this_instance + 1;
    assert_eq!(
        on_disk,
        vec![
            (txg, transaction, 0, 1, 0, named_unit_capacity),
            (txg, transaction, 1, 2, 1, 1),
        ],
        "(txg, 事务号, 提交标记, 本次发布内序号, 记录标志, 点名项数)：同一事务、提交标记与末条标志只在第二条，第一条装满 64 项"
    );
    let named_in_bump_order: Vec<_> = after.earlier_records_of_this_publish[0]
        .record
        .named
        .iter()
        .chain(after.record.named.iter())
        .map(|named| named.mapping_key())
        .collect();
    assert_eq!(
        named_in_bump_order.len(),
        after.rewritten.len(),
        "两条合起来恰好点名这次重写的每个单元一次"
    );

    // ② 发布做完：冷启动读回、池级 checker。
    let reopened = pool.reopen_cold();
    let rebuilt = rebuild_version(&reopened, &after.root, Some(after.record.clone()))
        .expect("冷启动重建这一版");
    let RebuiltVersion::WithFile(cold) = rebuilt else {
        panic!("这一版有文件");
    };
    assert_eq!(
        cold.inode_leaf_containers
            .iter()
            .map(|container| container.contents.records.len())
            .sum::<usize>(),
        usize::try_from(INODES_BEFORE + new_inode_count).expect("记录数"),
        "树里共 1 + N 条记录"
    );
    // 发布做完：I-8.8、I-8.9 真被评估过且成立
    images.hand_out(
        "发布做完",
        || image.clone(),
        PoolCheckerExpectation::no_invariant_violated()
            .and_holds("I-8.8")
            .and_holds("I-8.9"),
    );

    // ③ 崩在根槽 FUA 之前：两条记录都落了，恢复由记录施加这一版。
    let crash_before_the_root =
        memory_pool_of_the_first(&pool, index_of_the_last_root_slot_write(&pool));
    let report_before_the_root = recover(&crash_before_the_root, JournalPolicy::Consult);
    assert_eq!(
        (
            report_before_the_root.effective_root,
            report_before_the_root.journal.prefix_applied
        ),
        (Some((InstanceGeneration(1), after.root.checkpoint_txg)), 2),
        "崩在根之前：两条记录整次施加，走的是这一版（{:?}）",
        report_before_the_root.journal
    );
    images.hand_out(
        "崩在根之前",
        || crash_before_the_root.clone(),
        PoolCheckerExpectation::no_invariant_violated(),
    );

    // ④ 崩在第二条记录之前：末条没到，整次不施加。
    let crash_before_the_second_record = memory_pool_of_the_first(
        &pool,
        first_write_of_the_record(&pool, after.record.counter),
    );
    let report_before_the_second_record =
        recover(&crash_before_the_second_record, JournalPolicy::Consult);
    assert_eq!(
        (
            report_before_the_second_record.effective_root,
            report_before_the_second_record.journal.prefix_applied
        ),
        (Some((InstanceGeneration(1), before.root.checkpoint_txg)), 0),
        "崩在第二条之前：这次发布整体不施加，走的还是上一版的根（{:?}）",
        report_before_the_second_record.journal
    );
    images.hand_out(
        "崩在第二条之前",
        || crash_before_the_second_record.clone(),
        PoolCheckerExpectation::no_invariant_violated(),
    );
}
