//! 里程碑「第二个事务」并行线一（一个文件跨多个单元）的**写路径**验收，只做写这一半，读那一半不在这一轮。
//!
//! 三件事：
//! ① 一次写请求按切分纪律切成若干一单元事务（D16（发布语义） 已定项 5 末段、D23（journal 的角色与格式） 已定项 7、
//!    C310（事务切分纪律与记录数口径打架） 2026-09-16 用户定案），切出一个事务的那一档与覆盖写走同一条发布路径、写出的字节一个不变；
//! ② 切出不止一个事务时一次发布写出一个事务一条记录，跨过 67 那个点名项上限照样走得通，一片 extent 叶装满的 144 也走得通，
//!    多于 144 个单元时 extent 树下段长成两层（D8（核心索引结构） 已定项 14，用户 2026-09-24 定 K2），同样走得通。

mod common;

use common::{build_pool, parameters, BuiltPool, FIXED_WRITE_TIME_SECONDS};
use singlefs_core::address::InstanceGeneration;
use singlefs_core::transaction::{
    publish_overwrite, publish_sequential_write, FirstFile, PoolWriter, PublishError,
    TransactionOutput,
};
use singlefs_core::unit::data_unit_payload_capacity;
use singlefs_core::write_request_split::{
    data_unit_count_of_a_sequential_write, split_sequential_write_into_one_unit_transactions,
};
use singlefs_harness::RetainedOperation;

/// 一份可复现的内容：长度由调用点给，字节只随下标走。
fn content_of(length_in_bytes: usize) -> Vec<u8> {
    (0..length_in_bytes)
        .map(|index| u8::try_from((index * 7 + 3) % 253).expect("小于 256"))
        .collect()
}

/// 这份内容要几个数据单元：长度按净荷容量向上取整。
fn data_units_for(length_in_bytes: usize) -> u64 {
    data_unit_count_of_a_sequential_write(u64::try_from(length_in_bytes).expect("内容长度"))
}

/// 第一个事务之后在同一个实例里发一次顺序写，错误原样交回。
fn try_sequential_write(
    pool: &mut BuiltPool,
    content: &[u8],
) -> Result<TransactionOutput, PublishError> {
    let previous = pool.output.clone();
    let parameters = parameters();
    let devices = pool.devices.as_mut().expect("镜像还开着");
    let mut writer = PoolWriter::new(&parameters, devices.as_mut_slice());
    publish_sequential_write(
        &mut writer,
        &mut pool.allocator,
        &previous,
        FirstFile {
            content,
            write_time_seconds: FIXED_WRITE_TIME_SECONDS + 60,
        },
        InstanceGeneration(1),
    )
}

/// 第一个事务之后在同一个实例里覆盖写一次（对照臂）。
fn try_overwrite(pool: &mut BuiltPool, content: &[u8]) -> Result<TransactionOutput, PublishError> {
    let previous = pool.output.clone();
    let parameters = parameters();
    let devices = pool.devices.as_mut().expect("镜像还开着");
    let mut writer = PoolWriter::new(&parameters, devices.as_mut_slice());
    publish_overwrite(
        &mut writer,
        &mut pool.allocator,
        &previous,
        FirstFile {
            content,
            write_time_seconds: FIXED_WRITE_TIME_SECONDS + 60,
        },
        InstanceGeneration(1),
    )
}

/// 这次发布在录制流里发出的那几步（带内容）。
fn operations_since(pool: &BuiltPool, first_operation: usize) -> Vec<RetainedOperation> {
    pool.retained_operations()[first_operation..].to_vec()
}

/// 只要一个数据单元的顺序写，与同内容的覆盖写写出的字节一个不差：同一条发布路径
/// （`.claude/rules/fs-design.md`「一个事务层，所有结构共用」——并行线一不许另开一条发布路径）。
#[test]
fn a_sequential_write_of_one_data_unit_writes_the_same_bytes_as_an_overwrite_of_the_same_content() {
    let content = content_of(4100);
    assert_eq!(data_units_for(content.len()), 1);

    let mut sequential_pool = build_pool("parallel-line-one-sequential");
    let sequential_first_operation = sequential_pool.stream.operations().len();
    let sequential = try_sequential_write(&mut sequential_pool, &content).expect("顺序写");

    let mut overwrite_pool = build_pool("parallel-line-one-overwrite");
    let overwrite_first_operation = overwrite_pool.stream.operations().len();
    let overwritten = try_overwrite(&mut overwrite_pool, &content).expect("覆盖写");

    assert_eq!(
        sequential.units, overwritten.units,
        "九个角色的单元字节相同"
    );
    assert_eq!(
        sequential.record_bytes, overwritten.record_bytes,
        "journal 记录逐字节相同：一个数据单元一个事务一条记录"
    );
    assert_eq!(sequential.root, overwritten.root, "根记录相同");
    assert_eq!(
        sequential.allocation_records, overwritten.allocation_records,
        "分配记录相同：落点按 D3（空间分配） 已定项 8 逐单元取最低空槽对"
    );
    assert_eq!(sequential.mapping_keys, overwritten.mapping_keys);
    assert_eq!(
        sequential.key_order_mismatches, 0,
        "一个单元的请求没有相邻 key 可比（C319（请求内单元按 key 升序发出没有条款也没有检查））"
    );
    assert_eq!(
        operations_since(&sequential_pool, sequential_first_operation),
        operations_since(&overwrite_pool, overwrite_first_operation),
        "录制流这一段逐步相同：写的落点、字节、屏障次序都一样"
    );
}

/// 单元数跨过 67（一条 journal 记录的点名项上限）走得通：一次发布的记录条数 = 数据单元数
/// （C310（事务切分纪律与记录数口径打架） 2026-09-16 用户定案），67 那个点名项上限够不着——每条记录只点名一个数据单元，
/// 末条再加共享的提交内生块。144（一片 extent 叶装满）也走得通；145 起 extent 树下段长成两层（D8（核心索引结构） 已定项 14），
/// 同样走得通（下段的形状钉在 `second_transaction_parallel_line_one_multi_unit_file.rs`）。
#[test]
fn a_sequential_write_publishes_one_record_per_data_unit_across_the_sixty_seven_threshold_up_to_a_full_extent_leaf(
) {
    let payload_capacity = data_unit_payload_capacity();
    for expected_data_units in [2usize, 67, 68, 144, 145] {
        let content = content_of((expected_data_units - 1) * payload_capacity + 1);
        let expected_data_units_u64 = u64::try_from(expected_data_units).expect("单元数");
        assert_eq!(
            u64::try_from(
                split_sequential_write_into_one_unit_transactions(
                    u64::try_from(content.len()).expect("内容长度"),
                    1
                )
                .len()
            )
            .expect("事务数"),
            expected_data_units_u64,
            "切分算出的事务数 = 单元数"
        );

        let mut pool = build_pool("parallel-line-one-thresholds");
        let output = try_sequential_write(&mut pool, &content)
            .unwrap_or_else(|error| panic!("{expected_data_units} 个单元：{error:?}"));
        assert_eq!(
            output.earlier_records_of_this_publish.len() + 1,
            expected_data_units,
            "{expected_data_units} 个数据单元 {expected_data_units} 条记录"
        );
        assert_eq!(output.data_pointers.len(), expected_data_units);
        assert!(
            output
                .earlier_records_of_this_publish
                .iter()
                .all(|earlier| earlier.record.named.len() == 1),
            "末条之前每条只点名一个数据单元"
        );
        assert_eq!(output.key_order_mismatches, 0, "单元按 key 升序取号");
    }
}
