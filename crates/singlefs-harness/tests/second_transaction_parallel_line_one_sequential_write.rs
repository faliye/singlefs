//! 里程碑「第二个事务」并行线一（一个文件跨多个单元）的写路径：`publish_sequential_write` 按切分纪律
//! 把一次写请求切成若干一单元事务（D16（发布语义） 已定项 5 末段 + D23（journal 的角色与格式） 已定项 7），
//! 切出一个事务的那一档与 `publish_overwrite` 走同一条发布路径、写出的字节一个不变。
//!
//! 切出多于一个事务的那一档要的两条条款 2026-09-23 都定了，这里各钉一条会红的用例：
//! - C490（extent 叶 key 的 offset 段没定单位）→ D8（核心索引结构） 已定项 3：offset 段是**文件字节偏移**，
//!   第二个数据单元的叶记录 key 逐字节等于 `(0, 1, 32634)`（按单元序号写就是 `(0, 1, 1)`，那条断言红）；
//! - C491（多条记录时共享内生块在哪条点名没定）→ D23（journal 的角色与格式） 已定项 17：一次发布切成 N 条记录时，
//!   共享的提交内生块**只在最后一条点名**，前面那条只点名它自己那个数据单元（把点名挪到前面那条，断言红）。
//!
//! 净荷容量 32634 字节（32768 − 头 105 − 预留 29，字节表二）是一个单元与两个单元的分界：32634 一个、32635 两个。
//! 那个数本身由 `singlefs-core` 的 `write_request_split` 单测钉着。

mod common;

use common::{build_pool, parameters, BuiltPool, FIXED_WRITE_TIME_SECONDS};
use singlefs_core::address::{DataUnitIndexInFile, InstanceGeneration};
use singlefs_core::journal::back_chain_of;
use singlefs_core::records::{data_key_tail, parse_extent_record};
use singlefs_core::transaction::{
    publish_sequential_write, FirstFile, PoolWriter, PublishError, TransactionOutput,
    TransactionUnit, FIRST_INODE_NUMBER,
};
use singlefs_core::unit::{parse_data_unit, parse_index_node, UNIT_CLASS_DATA};
use singlefs_core::write_request_split::data_unit_count_of_a_sequential_write;

/// 净荷容量：一个数据单元装得下的文件字节数。两档的分界就在它上面。
const PAYLOAD_CAPACITY_IN_BYTES: usize = 32_634;

fn content_of(length_in_bytes: usize) -> Vec<u8> {
    (0..length_in_bytes)
        .map(|index| u8::try_from((index * 13 + 5) % 251).expect("小于 256"))
        .collect()
}

fn sequential_write(
    pool: &mut BuiltPool,
    content: &[u8],
) -> Result<TransactionOutput, PublishError> {
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
    )?;
    pool.output = output.clone();
    Ok(output)
}

/// 一个单元装得下的顺序写走得通：切出一个事务、一条记录，接在 `publish_overwrite` 那一路上。
#[test]
fn a_sequential_write_that_fits_one_data_unit_publishes_through_the_single_transaction_path() {
    let mut pool = build_pool("parallel-line-one-one-unit");
    assert_eq!(
        data_unit_count_of_a_sequential_write(PAYLOAD_CAPACITY_IN_BYTES as u64),
        1,
        "净荷容量整整一个单元：切分纪律切出一个事务"
    );

    let txg_before = pool.output.root.checkpoint_txg;
    let output = sequential_write(&mut pool, &vec![7u8; PAYLOAD_CAPACITY_IN_BYTES])
        .expect("一个单元的顺序写");

    assert_eq!(
        output.root.checkpoint_txg.0,
        txg_before.0 + 1,
        "一个事务一次发布：txg 加一"
    );
    assert!(
        output.earlier_records_of_this_publish.is_empty(),
        "一个数据单元一条记录"
    );
}

/// C490 那条会红的用例：写两个数据单元，第二条 extent 叶记录的 key 逐字节等于 `(0, 1, 32634)`——offset 段是
/// 这个单元第一个字节在文件里的偏移（D8（核心索引结构） 已定项 3），不是单元序号 1；数据单元头里的锚点偏移是同一个数
/// （D9（加密） 已定项 6：锚点偏移 = `key.offset − ptr.extent_off`，第一版 extent 距起点偏移恒 0）。
#[test]
fn the_second_extent_leaf_record_key_is_the_file_byte_offset_of_the_second_data_unit() {
    let mut pool = build_pool("parallel-line-one-second-key");
    let content = content_of(PAYLOAD_CAPACITY_IN_BYTES + 1);
    let output = sequential_write(&mut pool, &content).expect("两个单元的顺序写");

    let extent_root = parse_index_node(&output.unit(TransactionUnit::ExtentRoot).bytes)
        .expect("刚写出的 extent 根兼叶解得开");
    assert_eq!(extent_root.level, 0, "144 条之内一棵 extent 树只有根兼叶");
    assert_eq!(extent_root.entries.len(), 2, "两个数据单元两条叶记录");
    let mut second_key_expected = [0u8; 24];
    second_key_expected[8..16].copy_from_slice(&FIRST_INODE_NUMBER.to_le_bytes());
    second_key_expected[16..24].copy_from_slice(&32_634u64.to_le_bytes());
    let (second_key, second_pointer) =
        parse_extent_record(&extent_root.entries[1]).expect("第二条叶记录解得开");
    assert_eq!(
        second_key, second_key_expected,
        "第二条叶记录的 key = (locality 0, inode 1, 文件字节偏移 32634)"
    );
    assert_eq!(
        extent_root.largest_key, second_key_expected,
        "节点头的 key 区间上界就是这条（I-1.1（key 区间罩住条目））"
    );
    assert_eq!(second_pointer, output.data_pointers[1]);

    let second_unit = &output
        .unit(TransactionUnit::Data(DataUnitIndexInFile(1)))
        .bytes;
    let header = parse_data_unit(second_unit).expect("第二个数据单元解得开");
    assert_eq!(
        header.identity.anchor_offset, 32_634,
        "锚点偏移与 key 的 offset 段是同一个数"
    );
    assert_eq!(header.declared_length, 1, "第二个单元只装最后那一个字节");
}

/// C491 那条会红的用例：两个数据单元 ⇒ 两个事务、两条记录（C310（事务切分纪律与记录数口径打架） 2026-09-16 用户定案）；
/// 前一条只点名它自己那个数据单元，这次发布共享的提交内生块（extent 根、inode 叶容器与根、分配记录树、记账树、映射树、树表）
/// 只在最后一条点名（D23（journal 的角色与格式） 已定项 17）。两条连号、同一个 checkpoint_txg、各带提交标记、反向链接成一串。
#[test]
fn a_publish_of_two_data_units_names_the_shared_commit_generated_units_only_in_its_last_record() {
    let mut pool = build_pool("parallel-line-one-two-records");
    let previous = pool.output.clone();
    let content = content_of(PAYLOAD_CAPACITY_IN_BYTES + 4000);
    let output = sequential_write(&mut pool, &content).expect("两个单元的顺序写");

    assert_eq!(
        output.earlier_records_of_this_publish.len(),
        1,
        "两个数据单元两条记录：末条之前只有一条"
    );
    let first = &output.earlier_records_of_this_publish[0];
    let last = &output.record;

    // 前一条：只点名文件第 0 个数据单元，点名项的 key 尾段是它自己那个事务的写序。
    assert_eq!(first.record.named.len(), 1, "前一条只点名一个单元");
    assert_eq!(first.record.named[0].unit_class, UNIT_CLASS_DATA);
    assert_eq!(
        first.record.named[0].key_tail,
        data_key_tail(output.data_pointers[0].write_order)
    );
    // 末条：第 1 个数据单元加全部共享的提交内生块。
    let shared_roles: Vec<TransactionUnit> = output
        .rewritten
        .iter()
        .copied()
        .filter(|identity| !matches!(identity, TransactionUnit::Data(_)))
        .collect();
    assert_eq!(
        shared_roles,
        vec![
            TransactionUnit::ExtentRoot,
            TransactionUnit::InodeLeafContainer(
                singlefs_core::inode_tree::InodeLeafContainerIndexInTree::LEFTMOST
            ),
            TransactionUnit::InodeRoot,
            TransactionUnit::AllocationTree,
            TransactionUnit::AccountingTree,
            TransactionUnit::MappingTree,
            TransactionUnit::TreeTable,
        ],
        "这次发布重写的共享提交内生块"
    );
    assert_eq!(
        last.named.len(),
        1 + shared_roles.len(),
        "末条点名自己那个数据单元加全部共享内生块"
    );
    assert_eq!(last.named[0].unit_class, UNIT_CLASS_DATA);
    assert_eq!(
        last.named[0].key_tail,
        data_key_tail(output.data_pointers[1].write_order)
    );
    let shared_named_in_the_first_record = first
        .record
        .named
        .iter()
        .filter(|named| named.unit_class != UNIT_CLASS_DATA)
        .count();
    assert_eq!(
        shared_named_in_the_first_record, 0,
        "前一条一个共享内生块都不点名（只在最后一条点名）"
    );
    for shared_role in &shared_roles {
        let unit = output.unit(*shared_role);
        let locations = pool
            .devices
            .as_ref()
            .map(|devices| {
                singlefs_core::make_filesystem::location_entries(
                    &devices
                        .iter()
                        .map(|(identity, _)| *identity)
                        .collect::<Vec<_>>(),
                    unit.slot,
                    &unit.bytes,
                )
            })
            .expect("镜像还开着");
        assert!(
            last.named.iter().any(|named| named.locations == locations),
            "{} 在末条里点名",
            shared_role.tag()
        );
    }

    // 两条记录：连号、同一个 txg、各一个事务号（连号）、各带提交标记，反向链接成一串。
    assert_eq!(first.record.counter, previous.record.counter + 1);
    assert_eq!(last.counter, first.record.counter + 1);
    assert_eq!(first.record.checkpoint_txg, last.checkpoint_txg);
    assert_eq!(
        (first.record.transaction, last.transaction),
        (
            previous.highest_transaction_number_in_this_instance + 1,
            previous.highest_transaction_number_in_this_instance + 2
        ),
        "一条记录一个事务，事务号连号"
    );
    assert!(first.record.is_commit && last.is_commit, "两条各带提交标记");
    assert_eq!(
        first.record.back_chain,
        back_chain_of(&previous.record_bytes)
    );
    assert_eq!(last.back_chain, back_chain_of(&first.bytes));
    assert_eq!(
        output.highest_transaction_number_in_this_instance,
        last.transaction
    );
    assert_eq!(
        (first.record.new_tree_table, first.record.new_mapping_root),
        (last.new_tree_table, last.new_mapping_root),
        "每条记录都带整次发布的新根段（D23（journal 的角色与格式） 已定项 15）"
    );
}
