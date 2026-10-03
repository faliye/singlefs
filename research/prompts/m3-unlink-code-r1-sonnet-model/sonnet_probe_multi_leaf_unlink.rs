//! 正推腿的探针（m3-unlink-code-r1，Sonnet）：真发布路径上、多于一片下段叶的文件、产品批大小 144 的 unlink。
//! 背景：`unlink.rs` 的单测只在纯计划上判 300 → 200 / 300 → 144 / 3 → 1；入库的 harness 用例只删 3 个单元的文件（批大小 1）。
//! 这里把 145 / 288 / 289 / 300 个单元的文件经 `publish_unlink_intent` + `publish_unlink_step`（产品批大小）真发布到底，逐版判：
//! 池级 checker 无红、从盘上重建的版本与发布的版本相同（记录、队列、墓碑、数据指针、extent 树、映射 key）、冷走读答「没有文件」。
mod common;

use common::{
    build_pool, memory_pool_of_sparse_devices, parameters,
    sequential_write_in_process_as_instance_one_and_advance_the_pool, BuiltPool,
};
use singlefs_checker::image::InvariantVerdict;
use singlefs_checker::walk::check_pool_image;
use singlefs_core::address::{InodeNumber, InstanceGeneration};
use singlefs_core::admission::SpaceAdmission;
use singlefs_core::mount::mount_writable_with_space_admission;
use singlefs_core::recovery::{rebuild_version, recover, JournalPolicy, RebuiltVersion, RecoveryOutcome};
use singlefs_core::transaction::{
    publish_unlink_intent, publish_unlink_step, PoolWriter, TransactionOutput, FIRST_INODE_NUMBER,
};
use singlefs_core::unit::data_unit_payload_capacity;
use singlefs_core::unlink::UnlinkBatchSize;
use singlefs_harness::memory_pool::MemoryPool;
use singlefs_harness::SharedStream;

const FIRST_FILE: InodeNumber = InodeNumber(FIRST_INODE_NUMBER);
const INSTANCE: InstanceGeneration = InstanceGeneration(1);

fn violations(image: &MemoryPool) -> Vec<String> {
    check_pool_image(image)
        .into_iter()
        .filter_map(|(invariant, verdict)| match verdict {
            InvariantVerdict::Violated(detail) => Some(format!("{invariant}: {detail}")),
            InvariantVerdict::Holds | InvariantVerdict::NotApplicable(_) => None,
        })
        .collect()
}

fn pool_with_a_file_of(units: usize, tag: &str) -> BuiltPool {
    let payload = data_unit_payload_capacity();
    let content: Vec<u8> = (0..(units - 1) * payload + payload / 2)
        .map(|index| u8::try_from((index * 31 + 7) % 251).expect("小于 256"))
        .collect();
    let mut pool = build_pool(tag);
    let grown = sequential_write_in_process_as_instance_one_and_advance_the_pool(&mut pool, &content);
    assert_eq!(grown.data_pointers.len(), units);
    pool
}

fn intent(pool: &mut BuiltPool) -> TransactionOutput {
    let parameters = parameters();
    let previous = pool.output.clone();
    let devices = pool.devices.as_mut().expect("镜像还开着");
    let mut writer = PoolWriter::new(&parameters, devices.as_mut_slice());
    let output = publish_unlink_intent(&mut writer, &mut pool.allocator, &previous, FIRST_FILE, INSTANCE)
        .expect("意图");
    pool.output = output.clone();
    output
}

fn next_step(pool: &mut BuiltPool) -> TransactionOutput {
    let parameters = parameters();
    let previous = pool.output.clone();
    let devices = pool.devices.as_mut().expect("镜像还开着");
    let mut writer = PoolWriter::new(&parameters, devices.as_mut_slice());
    let output = publish_unlink_step(
        &mut writer,
        &mut pool.allocator,
        &previous,
        FIRST_FILE,
        UnlinkBatchSize::OneExtentLowerLeaf,
        INSTANCE,
    )
    .expect("下一步");
    pool.output = output.clone();
    output
}

fn judge_every_version(units: usize) {
    let mut pool = pool_with_a_file_of(units, &format!("sonnet-probe-multi-leaf-{units}"));
    let before = pool.output.clone();
    let mut published = vec![intent(&mut pool)];
    while !pool.output.deleted_inodes.is_empty() {
        published.push(next_step(&mut pool));
    }
    let image = pool.memory_pool();
    let mut summary = Vec::new();
    for version in &published {
        let (RebuiltVersion::WithFile(rebuilt) | RebuiltVersion::WithTreesAndNoFile(rebuilt)) =
            rebuild_version(&image, &version.root, Some(version.record.clone()))
                .unwrap_or_else(|failure| panic!("N={units} txg {:?} 重建：{failure:?}", version.root.checkpoint_txg))
        else {
            panic!("有树");
        };
        assert_eq!(
            (&rebuilt.inode_record, &rebuilt.deleted_inodes, &rebuilt.data_pointers, &rebuilt.extent_tree, &rebuilt.mapped_units),
            (&version.inode_record, &version.deleted_inodes, &version.data_pointers, &version.extent_tree, &version.mapped_units),
            "N={units} txg {:?}：重建 = 发布",
            version.root.checkpoint_txg
        );
        summary.push(format!(
            "txg {} 单元 {} 下段 {:?} 重写 {}",
            version.root.checkpoint_txg.0,
            version.data_pointers.len(),
            version.extent_tree.lower_nodes.iter().map(|(p, _)| (p.level, p.index)).collect::<Vec<_>>(),
            version.rewritten.iter().map(|role| role.tag()).collect::<Vec<_>>().join(","),
        ));
    }
    println!("N={units} 起点单元 {}：{}", before.data_pointers.len(), summary.join("\n   "));
    assert_eq!(violations(&image), Vec::<String>::new(), "N={units} 删完之后池级 checker 无红");
    let report = recover(&image, JournalPolicy::Consult);
    assert!(matches!(report.outcome, RecoveryOutcome::NoFile { .. }), "N={units} 冷走读：{:?}", report.outcome);
}

#[test]
fn product_batches_over_145_units() {
    judge_every_version(145);
}
#[test]
fn product_batches_over_288_units() {
    judge_every_version(288);
}
#[test]
fn product_batches_over_289_units() {
    judge_every_version(289);
}
#[test]
fn product_batches_over_300_units() {
    judge_every_version(300);
}

/// 300 个单元：意图 + 第一批之后崩（取这一刻的镜像），按产品批大小重开可写挂载续做到底：队列空、checker 无红。
#[test]
fn mount_continuation_over_300_units_after_one_batch() {
    let mut pool = pool_with_a_file_of(300, "sonnet-probe-multi-leaf-continuation");
    intent(&mut pool);
    next_step(&mut pool);
    let image = pool.memory_pool();
    let stream = SharedStream::new();
    let mut devices = common::crash_state_devices(&image, &[], &[], &stream);
    let mounted = mount_writable_with_space_admission(&parameters(), &mut devices, SpaceAdmission::JudgedByTheFormula)
        .expect("续做");
    let version = mounted.current.version_of_trees().expect("有树");
    println!("续做步数 {} 队列 {:?} 单元 {}", mounted.output.deletion_queue_continuation.len(), version.deleted_inodes, version.data_pointers.len());
    assert!(version.deleted_inodes.is_empty());
    let after = memory_pool_of_sparse_devices(&devices);
    assert_eq!(violations(&after), Vec::<String>::new());
}

/// 多片下段叶文件的第一批：每一个写点截断（录制流的每个前缀）、重开按产品批大小可写挂载续做到底；每个崩溃态与终态池级 checker 无红、
/// 终态逻辑内容（队列空、没有 inode 记录、没有数据单元、墓碑一条）相同。只切前缀、不排列没落盘的写（全枚举归 checker 档）。
fn cut_at_every_write_of_the_first_batch(units: usize) {
    let mut pool = pool_with_a_file_of(units, &format!("sonnet-probe-cut-{units}"));
    intent(&mut pool);
    let before_the_batch = pool.retained_operations().len();
    next_step(&mut pool);
    let after_the_batch = pool.retained_operations().len();
    while !pool.output.deleted_inodes.is_empty() {
        next_step(&mut pool);
    }
    let operations = pool.retained_operations();
    let mut cuts_judged = 0;
    let mut base_image = MemoryPool::with_devices(
        &[singlefs_core::address::DeviceIdentity(0), singlefs_core::address::DeviceIdentity(1)],
        common::IMAGE_BYTES,
    );
    base_image.apply(&operations[..before_the_batch]);
    for cut in before_the_batch..=after_the_batch {
        let mut crash_image = base_image.clone();
        crash_image.apply(&operations[before_the_batch..cut]);
        assert_eq!(violations(&crash_image), Vec::<String>::new(), "N={units} 切在 {cut}：崩溃态 checker 无红");
        let stream = SharedStream::new();
        let mut devices = common::crash_state_devices(&crash_image, &[], &[], &stream);
        let mounted = mount_writable_with_space_admission(&parameters(), &mut devices, SpaceAdmission::JudgedByTheFormula)
            .unwrap_or_else(|error| panic!("N={units} 切在 {cut}：重开可写挂载：{error:?}"));
        let version = mounted.current.version_of_trees().expect("有树");
        assert_eq!(
            (version.deleted_inodes.len(), version.inode_record.is_none(), version.data_pointers.len(), version.tombstone_containers.iter().map(|c| c.contents.records.len()).sum::<usize>()),
            (0, true, 0, 1),
            "N={units} 切在 {cut}：终态"
        );
        let after = memory_pool_of_sparse_devices(&devices);
        assert_eq!(violations(&after), Vec::<String>::new(), "N={units} 切在 {cut}：续做之后 checker 无红");
        cuts_judged += 1;
        if cuts_judged % 5 == 0 { println!("N={units} 已判 {cuts_judged} 个写点（切在 {cut}）"); }
    }
    println!("N={units} 第一批的写点 {} 个（前缀 {before_the_batch}..={after_the_batch}），逐个截断、续做、判过 {cuts_judged} 个", after_the_batch - before_the_batch);
}

#[test]
fn every_prefix_cut_of_the_first_batch_over_289_units_continues_to_the_same_end() {
    cut_at_every_write_of_the_first_batch(289);
}
#[test]
fn every_prefix_cut_of_the_first_batch_over_288_units_continues_to_the_same_end() {
    cut_at_every_write_of_the_first_batch(288);
}

/// 落点那一判（`admit_the_units_landing_on_every_device`）对一次删除类发布要的槽数 vs 这次发布真写的槽数：
/// 要的 = 这次普通分配那一半角色的槽 + ckpt_cost（整棵分配记录树的最坏数，D28 已定项 4）；真写的 = 这次重写的全部角色的槽。
/// 同时打出两块盘此刻没挡的槽数与环窗口（D28 已定项 1 末项）每盘扣的槽数。
#[test]
fn landing_demand_of_each_deletion_publish_against_the_slots_it_really_writes() {
    use singlefs_core::admission::{
        checkpoint_cost_of_the_version_to_build_on, ring_window_on_one_device, space_budget_of_role,
        PlacementRoomOnOneDevice, SpaceBudgetOfARole,
    };
    let mut pool = pool_with_a_file_of(300, "sonnet-probe-landing-demand");
    let mut steps = Vec::new();
    let mut previous = pool.output.clone();
    steps.push(intent(&mut pool));
    while !pool.output.deleted_inodes.is_empty() {
        steps.push(next_step(&mut pool));
    }
    // 分配器此刻是删完之后的；各步的 ckpt_cost 按各步的上一版现算（树高随版本变，分配器几何不变）。
    let room = PlacementRoomOnOneDevice::of_every_device_of_the_allocator(&pool.allocator);
    let ring_window = ring_window_on_one_device(
        &pool.allocator,
        parameters().geometry.root_ring_slots_per_region,
        singlefs_core::unlink::UnlinkBatchSize::OneExtentLowerLeaf.data_units_per_batch(),
    );
    for version in &steps {
        let ckpt_cost = checkpoint_cost_of_the_version_to_build_on(Some(&previous), &pool.allocator);
        let really_written: u64 = version.rewritten.iter().map(|role| role.span_slots()).sum();
        let ordinary: u64 = version
            .rewritten
            .iter()
            .filter(|role| space_budget_of_role(**role) == SpaceBudgetOfARole::OrdinaryAllocation)
            .map(|role| role.span_slots())
            .sum();
        println!(
            "txg {} 真写 {} 槽（普通分配 {}），落点那一判要 普通 {} + ckpt_cost {} = {}；此刻没挡的槽 {:?}；环窗口每盘 {:?}",
            version.root.checkpoint_txg.0,
            really_written,
            ordinary,
            ordinary,
            ckpt_cost.0,
            ordinary + ckpt_cost.0,
            room.iter().map(|r| r.slots_for_commit_generated_units).collect::<Vec<_>>(),
            ring_window,
        );
        previous = version.clone();
    }
}

/// 截到只差一个单元就到叶边界（kept = 143 / 287）：留下的最后一片叶的末个单元（143 / 287）正是被截掉的第一个单元，这片叶必须重写。
#[test]
fn product_batches_over_287_units() {
    judge_every_version(287);
}
#[test]
fn product_batches_over_431_units() {
    judge_every_version(431);
}
