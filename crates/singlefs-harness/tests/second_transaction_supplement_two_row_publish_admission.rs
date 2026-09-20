//! 里程碑「第二个事务」增补 2 第 20a 行（代码三方第一轮打中，判决 `research/prompts/m2-wave1-code-r1-main-verification.md` 第二节第 1 行）：
//! 写行那次发布要用的准入（这次之后的分配记录条数、这次要写的记账行数）在**取号之前**算，算不过在任何写之前返回。
//! 2026-09-18 用户定案再把**暖机那几次空发布**的准入一起挪到取号之前（收口表第 20a 行）：写行之后要推 1–3 次空发布
//! （D16（发布语义） 已定项 8 甲′），每次重写四个固定点单元、每盘各加一条分配记录，改之前它们没进取号之前那一遍 ⇒
//! 「写行装得下、暖机第 N 次装不下」的池仍是取号写完、写行也发完，暖机才报错，实例代号照烧。
//! 本文件三条用例：写行那次装不下（`..._cannot_publish_the_rows_...`）、暖机第 1 次装不下、暖机第 2 次装不下。
//!
//! 改之前这两条只在发布路径里算：取号（两次系统配置槽写 + 一道屏障）已经写完才报出来，而取号的回卷只管取号自己那几次写报错、
//! 管不到取号之后的发布失败（`write_acquired_instance`）⇒ 分配记录树满了的池此后每试一次可写挂载就再烧一个实例代号、
//! 录制流多 3 步，而这个池本来就再也发布不出东西。
//!
//! 形态今天可达、不靠坏盘：分配记录树第一版只有一个节点（10 + 20 的条目，812 条），释放只改写记录不删 ⇒ 一路覆盖写就能填满它。
//! 覆盖写每次每盘加 8 条（16 条），空发布每次每盘加 4 条（8 条）——先覆盖写到装不下，再用空发布把余量填到「写行那次发布
//! （实例表 + 四个固定点单元 = 5 个角色 × 2 盘 = 10 条）也装不下」。
//! 回卷已经写出的代号、以及「烧代号算不算合法」不在这一轮里（收口表第 20a 行：交用户）。

mod common;

use common::{build_pool, disk_snapshot, parameters, BuiltPool, FIXED_WRITE_TIME_SECONDS};
use singlefs_core::address::{CheckpointTxg, DeviceIdentity, InstanceGeneration};
use singlefs_core::journal::back_chain_of;
use singlefs_core::mount::{mount_writable, MountError};
use singlefs_core::recovery::verified_superblock_slots;
use singlefs_core::transaction::{
    publish_overwrite, publish_version, FirstFile, InstanceTablePlan, PoolWriter, PublishError,
    PublishPlan, PublishShape, TransactionOutput,
};
use singlefs_core::unit::index_node_entry_capacity;
use singlefs_format::{ALLOCATION_RECORD_BYTES, ALLOCATION_RECORD_KEY_BYTES};

const DISKS: [DeviceIdentity; 2] = [DeviceIdentity(0), DeviceIdentity(1)];

fn content_of(length: usize, seed: usize) -> Vec<u8> {
    (0..length)
        .map(|index| u8::try_from((index * 7 + seed) % 253).expect("小于 256"))
        .collect()
}

/// 分配记录树第一版那一个节点装得下几条。
fn allocation_node_capacity() -> usize {
    index_node_entry_capacity(
        usize::try_from(ALLOCATION_RECORD_KEY_BYTES).expect("10"),
        usize::try_from(ALLOCATION_RECORD_BYTES).expect("20"),
    )
}

/// 同一个进程里接着现行版本覆盖写一次（八个角色），错误原样交回。
fn try_overwrite_in_process(
    pool: &mut BuiltPool,
    content: &[u8],
) -> Result<TransactionOutput, PublishError> {
    let publish_parameters = parameters();
    let devices = pool.devices.as_mut().expect("镜像还开着");
    let mut writer = PoolWriter::new(&publish_parameters, devices.as_mut_slice());
    let previous = pool.output.clone();
    let output = publish_overwrite(
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

/// 同一个进程里接着现行版本推一次空发布（不写文件、照抄实例表 ⇒ 四个固定点单元，每盘各四条新记录），错误原样交回。
fn try_empty_publish_in_process(pool: &mut BuiltPool) -> Result<TransactionOutput, PublishError> {
    let publish_parameters = parameters();
    let previous = pool.output.clone();
    let devices = pool.devices.as_mut().expect("镜像还开着");
    let mut writer = PoolWriter::new(&publish_parameters, devices.as_mut_slice());
    let output = publish_version(
        &mut writer,
        &mut pool.allocator,
        PublishPlan {
            txg: CheckpointTxg(previous.root.checkpoint_txg.0 + 1),
            counter: previous.record.counter + 1,
            transaction: 0,
            instance: InstanceGeneration(1),
            back_chain: back_chain_of(&previous.record_bytes),
            file: None,
            instance_table: InstanceTablePlan::Carry(previous.root.instance_table),
            tree_birth_txg: previous.tree_birth_txg(),
            tree_identifier_watermark: previous.root.tree_identifier_watermark,
            rollback_floor: previous.root.rollback_floor,
        },
        Some(&previous),
    )?;
    pool.output = output.clone();
    Ok(output)
}

/// 两块盘四个系统配置槽里自证过的那些槽写着的实例代号，按盘排。
fn superblock_instances(pool: &BuiltPool) -> Vec<(DeviceIdentity, Vec<InstanceGeneration>)> {
    let image = pool.memory_pool();
    let spacing = u64::from(parameters().geometry.fixed_structure_slot_spacing);
    DISKS
        .iter()
        .map(|device| {
            let mut instances: Vec<InstanceGeneration> = verified_superblock_slots(
                &image,
                *device,
                spacing,
                &parameters().filesystem_identifier,
            )
            .iter()
            .map(|superblock| superblock.journal_instance)
            .collect();
            instances.sort();
            (*device, instances)
        })
        .collect()
}

/// 把分配记录树填到「写行那次发布也装不下」：先覆盖写到覆盖写自己装不下，再推空发布到空发布自己装不下。
/// 之后 records + 10 > 812（空发布装不下 ⇒ records ≥ 805 > 802），写行那次发布的准入必不过。
fn fill_the_allocation_record_node(pool: &mut BuiltPool) -> (usize, usize) {
    let mut overwrites = 0usize;
    loop {
        match try_overwrite_in_process(pool, &content_of(3000 + overwrites % 97, overwrites)) {
            Ok(_) => overwrites += 1,
            Err(PublishError::AllocationRecordsExceedOneNode { .. }) => break,
            Err(other) => panic!("第 {overwrites} 次覆盖写报了别的错：{other:?}"),
        }
    }
    let mut empty_publishes = 0usize;
    loop {
        match try_empty_publish_in_process(pool) {
            Ok(_) => empty_publishes += 1,
            Err(PublishError::AllocationRecordsExceedOneNode { .. }) => break,
            Err(other) => panic!("第 {empty_publishes} 次空发布报了别的错：{other:?}"),
        }
    }
    (overwrites, empty_publishes)
}

/// 覆盖写 `overwrites` 次、再推 `empty_publishes` 次空发布，每一次都必须成立；交回这之后分配记录树里的条数。
/// 第一个事务留下 20 条，覆盖写每次加 16 条（八个角色 × 2 盘），空发布每次加 8 条（四个固定点单元 × 2 盘）。
fn publish_until_the_allocation_record_node_holds(
    pool: &mut BuiltPool,
    overwrites: usize,
    empty_publishes: usize,
) -> usize {
    for index in 0..overwrites {
        try_overwrite_in_process(pool, &content_of(3000 + index % 97, index))
            .unwrap_or_else(|error| panic!("第 {index} 次覆盖写要成立：{error:?}"));
    }
    for index in 0..empty_publishes {
        try_empty_publish_in_process(pool)
            .unwrap_or_else(|error| panic!("第 {index} 次空发布要成立：{error:?}"));
    }
    pool.allocator.records().len()
}

/// 验收：分配记录树满了的池上可写挂载——在取号之前返回 `RowPublishAdmissionRefusedBeforeAcquisition`，
/// 盘上逐字节不变（`DiskSnapshot`：两盘四个系统配置槽的原样字节、根环里全部自证过的根、录制流步数）、
/// 两块盘系统配置里的实例代号没动。
#[test]
fn writable_mount_that_cannot_publish_the_rows_is_refused_before_the_instance_is_acquired() {
    assert_eq!(allocation_node_capacity(), 812);
    let mut pool = build_pool("supplement-two-row-publish-admission");
    let (overwrites, empty_publishes) = fill_the_allocation_record_node(&mut pool);
    assert_eq!(
        (overwrites, empty_publishes),
        (49, 1),
        "第一个事务留下 20 条；49 次覆盖写每次 16 条填到 804（第 50 次要 820 > 812，被拒），1 次空发布 8 条填到 812（第 2 次要 820，被拒）"
    );
    let records_before_the_mount = pool.allocator.records().len();
    assert_eq!(records_before_the_mount, 812, "节点正好装满");
    let rows_publish_records =
        records_before_the_mount + PublishShape::ROW_PUBLISH.rewritten_roles().len() * DISKS.len();
    assert!(
        rows_publish_records > allocation_node_capacity(),
        "写行那次发布装不下：{records_before_the_mount} + 10"
    );

    let before = disk_snapshot(&pool.memory_pool(), &pool.stream);
    let instances_before = superblock_instances(&pool);
    assert_eq!(
        instances_before,
        vec![
            (DeviceIdentity(0), vec![InstanceGeneration(1); 2]),
            (DeviceIdentity(1), vec![InstanceGeneration(1); 2]),
        ],
        "挂载之前两块盘四个槽写的都是实例 1"
    );

    let mut devices = pool.reopen_recorded();
    let refused = mount_writable(&parameters(), &mut devices);
    pool.devices = Some(devices);

    match refused {
        Err(MountError::RowPublishAdmissionRefusedBeforeAcquisition {
            instance_to_acquire,
            cause:
                PublishError::AllocationRecordsExceedOneNode {
                    records,
                    capacity: 812,
                },
        }) => {
            assert_eq!(instance_to_acquire, InstanceGeneration(2));
            assert_eq!(
                records, rows_publish_records,
                "从盘上重建出来的记录数与进程里的相同，写行那次发布每盘各加 5 条"
            );
        }
        other => panic!("要在取号之前按写行那次发布的准入拒绝：{other:?}"),
    }

    assert_eq!(
        disk_snapshot(&pool.memory_pool(), &pool.stream),
        before,
        "盘上逐字节不变：系统配置槽、根环里的根、录制流步数都没动"
    );
    assert_eq!(
        superblock_instances(&pool),
        instances_before,
        "两块盘系统配置里的实例代号没动：取号一次都没发生"
    );
}

/// 「写行装得下、暖机第 `refused_warm_up_index` 次装不下」的池上可写挂载：在取号之前返回
/// `WarmUpAdmissionRefusedBeforeAcquisition`，盘上逐字节不变、四个系统配置槽里的实例代号不变、录制流一步没多。
///
/// 两块盘、区域归属 0 / 1 / 0：这两个池最后一次发布都落在 txg 52，新实例的第一次发布（写行）是 txg 53 = 区域 2 = 盘 0，
/// 暖机 txg 54 = 区域 0 = 盘 0（还没覆盖盘 1）、txg 55 = 区域 1 = 盘 1 ⇒ 这次挂载计划推 2 次暖机空发布。
///
/// ⚠️ 「装得下 / 装不下」按准入那套算术说：一次发布这次之后的条数 = 现在的条数 + 重写的角色数 × 盘数（`publish_admission`
/// 从第一天起就这么算，取号之前这一串把前面几次的也累上）。真跑起来加得比这少——重开之后分配器把上一版释放掉的落点再发出去时
/// 记录是改写不是追加，这两个池实测写行只加 2 条、两次暖机加 0 与 2 条（副本里插 `eprintln` 量的）。
/// 于是这两个池是「按准入算装不下」而不是「真发起来装不下」：删掉暖机那一半的准入，它们能一路发完（变异 M1 实测 `Ok(Mounted ..)`）。
/// 差在哪、要不要把可复用的已释放记录抵扣掉，是留给用户的一问，不在这一轮里定。
fn writable_mount_is_refused_before_acquisition_at_warm_up_publish(
    tag: &str,
    overwrites: usize,
    empty_publishes: usize,
    refused_warm_up_index: usize,
) {
    assert_eq!(allocation_node_capacity(), 812);
    let mut pool = build_pool(tag);
    let records_before_the_mount =
        publish_until_the_allocation_record_node_holds(&mut pool, overwrites, empty_publishes);
    assert_eq!(
        records_before_the_mount,
        20 + 16 * overwrites + 8 * empty_publishes,
        "第一个事务留下 20 条，覆盖写每次 16 条、空发布每次 8 条"
    );
    let records_per_warm_up_publish =
        PublishShape::EMPTY_PUBLISH.rewritten_roles().len() * DISKS.len();
    let records_after_the_row_publish =
        records_before_the_mount + PublishShape::ROW_PUBLISH.rewritten_roles().len() * DISKS.len();
    assert!(
        records_after_the_row_publish <= allocation_node_capacity(),
        "写行那次发布装得下：{records_before_the_mount} + 10"
    );
    let records_after_the_refused_warm_up =
        records_after_the_row_publish + records_per_warm_up_publish * refused_warm_up_index;
    assert!(
        records_after_the_refused_warm_up - records_per_warm_up_publish
            <= allocation_node_capacity(),
        "暖机第 {} 次还装得下",
        refused_warm_up_index - 1
    );
    assert!(
        records_after_the_refused_warm_up > allocation_node_capacity(),
        "暖机第 {refused_warm_up_index} 次装不下：{records_after_the_refused_warm_up} > 812"
    );

    let before = disk_snapshot(&pool.memory_pool(), &pool.stream);
    let instances_before = superblock_instances(&pool);
    assert_eq!(
        instances_before,
        vec![
            (DeviceIdentity(0), vec![InstanceGeneration(1); 2]),
            (DeviceIdentity(1), vec![InstanceGeneration(1); 2]),
        ],
        "挂载之前两块盘四个槽写的都是实例 1"
    );
    let steps_before = pool.stream.operations().len();

    let mut devices = pool.reopen_recorded();
    let refused = mount_writable(&parameters(), &mut devices);
    pool.devices = Some(devices);

    match refused {
        Err(MountError::WarmUpAdmissionRefusedBeforeAcquisition {
            instance_to_acquire,
            warm_up_publish_index,
            warm_up_publishes_planned,
            cause:
                PublishError::AllocationRecordsExceedOneNode {
                    records,
                    capacity: 812,
                },
        }) => {
            assert_eq!(instance_to_acquire, InstanceGeneration(2));
            assert_eq!(warm_up_publish_index, refused_warm_up_index);
            assert_eq!(
                warm_up_publishes_planned, 2,
                "写行 txg 53 落盘 0、暖机 txg 54 落盘 0、txg 55 落盘 1：要推两次才覆盖两块盘"
            );
            assert_eq!(
                records, records_after_the_refused_warm_up,
                "从盘上重建出来的记录数加上写行那次的 10 条、暖机每次 8 条"
            );
        }
        other => panic!("要在取号之前按暖机那几次空发布的准入拒绝：{other:?}"),
    }

    assert_eq!(
        disk_snapshot(&pool.memory_pool(), &pool.stream),
        before,
        "盘上逐字节不变：系统配置槽、根环里的根、录制流步数都没动"
    );
    assert_eq!(
        pool.stream.operations().len() - steps_before,
        0,
        "录制流 0 步：一个写、一道屏障都没发"
    );
    assert_eq!(
        superblock_instances(&pool),
        instances_before,
        "两块盘系统配置里的实例代号没动：取号一次都没发生"
    );
}

/// 验收（增补 2 第 20a 行，2026-09-18 用户定案的那一半）：写行那次装得下（796 + 10 = 806 ≤ 812）、暖机第 1 次装不下
/// （806 + 8 = 814 > 812）的池——在取号之前拒绝。48 次覆盖写 + 1 次空发布 ⇒ 20 + 768 + 8 = 796 条。
#[test]
fn writable_mount_whose_first_warm_up_publish_does_not_fit_is_refused_before_the_instance_is_acquired(
) {
    writable_mount_is_refused_before_acquisition_at_warm_up_publish(
        "supplement-two-warm-up-admission-first",
        48,
        1,
        1,
    );
}

/// 验收（同上）：写行那次与暖机第 1 次都装得下（788 + 10 = 798、798 + 8 = 806 ≤ 812）、暖机第 2 次装不下（806 + 8 = 814 > 812）
/// 的池——在取号之前拒绝。47 次覆盖写 + 2 次空发布 ⇒ 20 + 752 + 16 = 788 条。
/// 这一条才分辨得出「只把第一次暖机算进去」的半截改法：那种改法在这个池上仍然放行，取号与写行都发出去。
#[test]
fn writable_mount_whose_second_warm_up_publish_does_not_fit_is_refused_before_the_instance_is_acquired(
) {
    writable_mount_is_refused_before_acquisition_at_warm_up_publish(
        "supplement-two-warm-up-admission-second",
        47,
        2,
        2,
    );
}
