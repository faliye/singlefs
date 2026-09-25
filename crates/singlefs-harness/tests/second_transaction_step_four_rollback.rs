//! 里程碑「第二个事务」步 4 的验收：发布 C 之后进程退出、重开走管理员回退到 A 的根 (1, 3)——不施加 A 之后的任何记录、
//! 取实例代号 3、在 A 那一版实例表上写回退行 (1, 3, 0) 与中间实例行 (2, 0, 0)、发布 D（txg 9，jsn 接在环里最大的 jsn 8 之后 = 9，被抛弃的记录一条不盖）、
//! 暖机一次（txg 10 落盘 1）——冷启动择实例 3 的根读回第一次的内容；只被被抛弃根引用的槽由影子账隔离；池级 checker 全绿。

mod common;

use common::{
    build_pool, file_content, parameters, BuiltPool, FIXED_WRITE_TIME_SECONDS, IMAGE_BYTES,
};
use singlefs_checker::image::InvariantVerdict;
use singlefs_checker::index_node_view;
use singlefs_checker::walk::check_pool_image;
use singlefs_core::address::{
    CheckpointTxg, DeviceIdentity, DeviceOffsetInBytes, FileOffsetInBytes, InodeNumber,
    InstanceGeneration, TreeIdentifier,
};
use singlefs_core::allocation_record_tree::{
    AllocationRecordTreeNode, AllocationRecordTreeNodePosition,
};
use singlefs_core::allocator::{unit_area_slots_of_device, PoolAllocator};
use singlefs_core::block_device::{BlockDevice, WriteDurability};
use singlefs_core::journal::back_chain_of;
use singlefs_core::mount::{
    mount_rollback, mount_writable, InstanceRow, MountError, Mounted, RollbackCandidateExclusion,
    RollbackTarget, ShadowLedger,
};
use singlefs_core::mounted_read::mount_read_only;
use singlefs_core::pointer::LocationEntry;
use singlefs_core::records::TREE_KIND_ALLOCATION;
use singlefs_core::recovery::{
    allocation_records_under_root, choose_root, choose_system_configuration,
    highest_tree_identifier_watermark_in_the_ring, readable_roots, recover, replay_journal,
    scan_journal, tree_table_has_no_entries, walk_to_file, JournalPolicy, RecoveryFailure,
    RecoveryOutcome,
};
use singlefs_core::root_ring::{slot_offset, target_for_publish};
use singlefs_core::transaction::{
    publish_first_file, publish_overwrite, publish_without_units, FirstFile, PoolVersion,
    PoolWriter, TransactionOutput, TransactionUnit, VersionWithoutFilePublishOutput,
    ZeroUnitPublishPlan, FIRST_INODE_NUMBER,
};
use singlefs_format::{ROOT_RING_REGIONS, UNIT_AREA_START_SLOT};
use singlefs_harness::bad_disk_input::move_the_first_allocation_record_past_the_end_of_the_unit_area_and_reseal_the_chain;
use singlefs_harness::crash::MemoryPool;
use singlefs_harness::fault_injection::{
    NamedRootRingSlots, PoolReaderWithUnreadableRootRingSlots, RootRingSlotTarget,
};
use std::collections::BTreeSet;

const SECOND_FILE_BYTES: usize = 4100;
const THIRD_FILE_BYTES: usize = 2500;

fn content_of(length: usize, seed: usize) -> Vec<u8> {
    (0..length)
        .map(|index| u8::try_from((index * 7 + seed) % 253).expect("小于 256"))
        .collect()
}

fn second_content() -> Vec<u8> {
    content_of(SECOND_FILE_BYTES, 3)
}

fn third_content() -> Vec<u8> {
    content_of(THIRD_FILE_BYTES, 11)
}

fn overwrite_in_process(
    pool: &mut BuiltPool,
    content: &[u8],
    instance: InstanceGeneration,
) -> TransactionOutput {
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
        instance,
    )
    .expect("覆盖写");
    pool.output = output.clone();
    output
}

/// 固定脚本到 C：A、B（实例 1）、重开取号 2、写行、暖机两次、C（实例 2）。
fn build_through_third_publish(tag: &str) -> BuiltPool {
    let mut pool = build_pool(tag);
    overwrite_in_process(&mut pool, &second_content(), InstanceGeneration(1));
    let mut devices = pool.reopen_recorded();
    let mounted = mount_writable(&parameters(), &mut devices).expect("可写挂载");
    pool.devices = Some(devices);
    pool.allocator = mounted.allocator;
    pool.output = mounted
        .current
        .into_file_version()
        .expect("B 之后重开，现行那一版带文件");
    overwrite_in_process(&mut pool, &third_content(), InstanceGeneration(2));
    pool
}

/// 根环全部自证过的根带的树 ID 水位取 max（D8（核心索引结构） 已定项 8 ② 的那个量，也是 I-7.8（根记录树 ID 水位不低于全池最大树 ID）
/// 取 max 的范围）：用例拿它核「回退之前环里的最大水位」。
fn highest_watermark_among_ring_roots(image: &MemoryPool) -> u64 {
    let parameters = parameters();
    readable_roots(
        image,
        &parameters.region_devices,
        &parameters.geometry,
        &parameters.filesystem_identifier,
    )
    .iter()
    .map(|root| root.tree_identifier_watermark)
    .max()
    .expect("环里至少有一条自证过的根")
}

/// 池级 checker 一条违例都没有；交回全部判定，调用方再点名要真被判过（不是「不适用」）的那几条。
fn verdicts_without_any_violation(
    image: &MemoryPool,
    step: &str,
) -> Vec<(&'static str, InvariantVerdict)> {
    let verdicts = check_pool_image(image);
    let violated: Vec<(&str, &InvariantVerdict)> = verdicts
        .iter()
        .filter(|(_, verdict)| matches!(verdict, InvariantVerdict::Violated(_)))
        .map(|(invariant, verdict)| (*invariant, verdict))
        .collect();
    assert!(
        violated.is_empty(),
        "{step}：池级 checker 一条违例都没有：{violated:?}"
    );
    verdicts
}

fn assert_judged_and_holding(
    verdicts: &[(&'static str, InvariantVerdict)],
    invariants: &[&str],
    step: &str,
) {
    for invariant in invariants {
        assert!(
            verdicts
                .iter()
                .any(|(name, verdict)| name == invariant && *verdict == InvariantVerdict::Holds),
            "{step}：{invariant} 真被判过且成立：{:?}",
            verdicts.iter().find(|(name, _)| name == invariant)
        );
    }
}

fn warm_up_root() -> RollbackTarget {
    RollbackTarget {
        instance: InstanceGeneration(1),
        checkpoint_txg: CheckpointTxg(2),
    }
}

/// 回退之后那个会话的现行那一版（树表 0 条）：根、记录、记录的字节。
fn current_version_without_file(current: &PoolVersion) -> VersionWithoutFilePublishOutput {
    let PoolVersion::WithoutFile(version) = current else {
        panic!("回退到树表 0 条的暖机根：现行那一版仍是「没有文件版本」的一版")
    };
    version.clone()
}

/// 在回退之后那个会话里接着现行那一版（树表 0 条）发第一个文件版本，回来的那一版装回 pool。
fn publish_first_file_after_the_rollback(
    pool: &mut BuiltPool,
    allocator: &mut PoolAllocator,
    current: &PoolVersion,
    instance: InstanceGeneration,
    content: &[u8],
) -> TransactionOutput {
    let version = current_version_without_file(current);
    let publish_parameters = parameters();
    let devices = pool.devices.as_mut().expect("镜像还开着");
    let mut writer = PoolWriter::new(&publish_parameters, devices.as_mut_slice());
    let output = publish_first_file(
        &mut writer,
        allocator,
        &version.root,
        FirstFile {
            content,
            write_time_seconds: FIXED_WRITE_TIME_SECONDS + 120,
        },
        instance,
        &version.record_bytes,
    )
    .expect("回退到树表 0 条的一版之后再发第一个文件版本");
    pool.output = output.clone();
    pool.allocator = allocator.clone();
    output
}

/// 新发出来的八个号都不低于回退那一版带过来的水位、都高于回退之前发过的每一个号；八个号互不相同；
/// 树表七条与中央映射树根的头里写的就是这几个号；新水位 = 最大那个号 + 1（D8（核心索引结构） 已定项 8 ②）。
fn assert_fresh_tree_identifiers(
    output: &TransactionOutput,
    watermark_carried_by_the_rollback: u64,
    highest_tree_identifier_before_the_rollback: TreeIdentifier,
) {
    let issued = output.tree_identifiers.in_issue_order();
    for tree in issued {
        assert!(
            tree.0 >= watermark_carried_by_the_rollback
                && tree > highest_tree_identifier_before_the_rollback,
            "新发的号 {tree:?} 不低于回退带过来的水位 {watermark_carried_by_the_rollback}、高于此前最大的 {highest_tree_identifier_before_the_rollback:?}"
        );
    }
    assert_eq!(
        issued.iter().collect::<BTreeSet<_>>().len(),
        8,
        "八个号互不相同"
    );
    assert_eq!(
        output.root.tree_identifier_watermark,
        output.tree_identifiers.highest().0 + 1,
        "新水位 = 这次发出的最高号 + 1（它高于环里任何一条根带的）"
    );
    let tree_table_trees: BTreeSet<TreeIdentifier> = output
        .tree_table_entries
        .iter()
        .map(|entry| entry.tree)
        .collect();
    let issued_without_the_central_mapping: BTreeSet<TreeIdentifier> = issued
        .into_iter()
        .filter(|tree| *tree != output.tree_identifiers.central_mapping)
        .collect();
    assert_eq!(
        tree_table_trees, issued_without_the_central_mapping,
        "树表七条就是这次发的号（中央映射树不进树表）"
    );
    let mapping_node = index_node_view(&output.unit(TransactionUnit::MappingTree).bytes)
        .expect("中央映射树根是码 2 节点");
    assert_eq!(
        (
            mapping_node.tree_identifier,
            output.root.mapping_root.head.birth_tree
        ),
        (
            output.tree_identifiers.central_mapping.0,
            output.tree_identifiers.central_mapping
        ),
        "中央映射树根的头与根记录里它那条指针的出生树都是这次发的号"
    );
}

/// C511（回退到无文件那一版之后诞生代怎么接） 第 3 步：环里还留着带文件版本的根 (1, 3)（树 11..18、水位 19）时回退到树表 0 条的
/// 暖机根 (1, 2)——回退照常做，不再在写之前拒绝；回退行那次发布的根带回退之前根环里的水位 max 19（D8（核心索引结构） 已定项 8 ②），
/// 不带暖机根自己的 11。接着在同一个会话里再发第一个文件版本：八棵树从 19 起连号发、新水位 27，高于此前发过的每一个号；
/// 再覆盖写一版，让 I-9.14（树表条目的诞生 txg 跨根不变） 在同一条时间线上有两个树表可比。每一步池级 checker 一条违例都没有，
/// 最后 I-7.8（根记录树 ID 水位不低于全池最大树 ID）、I-9.14、I-9.10（对象出生代三处一致） 都真被判过且成立；冷启动读回覆盖写的内容。
#[test]
fn rolling_back_to_a_warm_up_root_while_the_ring_still_holds_a_file_version_carries_the_ring_watermark_and_the_next_first_file_issues_fresh_tree_identifiers(
) {
    let mut pool = build_pool("step-four-rollback-to-warm-up-root-then-first-file");
    let highest_tree_identifier_before_the_rollback = pool.output.tree_identifiers.highest();
    let ring_watermark_before_the_rollback =
        highest_watermark_among_ring_roots(&pool.memory_pool());
    assert_eq!(
        (
            highest_tree_identifier_before_the_rollback,
            ring_watermark_before_the_rollback
        ),
        (TreeIdentifier(18), 19),
        "回退之前：第一个事务发了 11..18，(1, 3) 带水位 19"
    );
    let parameters_of_the_pool = parameters();
    let warm_up_root_record = readable_roots(
        &pool.memory_pool(),
        &parameters_of_the_pool.region_devices,
        &parameters_of_the_pool.geometry,
        &parameters_of_the_pool.filesystem_identifier,
    )
    .into_iter()
    .find(|root| {
        (root.instance, root.checkpoint_txg)
            == (warm_up_root().instance, warm_up_root().checkpoint_txg)
    })
    .expect("暖机根 (1, 2) 还在环里");
    assert_eq!(
        warm_up_root_record.tree_identifier_watermark, 11,
        "回退到的那一版自己带的是 mkfs 种下的 11：沿它带就会低于环里的 max"
    );

    let mut devices = pool.reopen_recorded();
    let rolled_back = mount_rollback(
        &parameters(),
        &mut devices,
        warm_up_root(),
        ShadowLedger::On,
    )
    .expect("环里还留着带文件版本的根时回退照常做");
    pool.devices = Some(devices);
    let PoolVersion::WithoutFile(row) = &rolled_back.output.row_publish else {
        panic!("回退到树表 0 条的暖机根：回退行那次发布仍是「没有文件版本」的一版")
    };
    assert_eq!(
        (
            row.root.tree_identifier_watermark,
            row.record.new_tree_identifier_watermark
        ),
        (
            ring_watermark_before_the_rollback,
            ring_watermark_before_the_rollback
        ),
        "回退行那次发布的根与记录新根段都带回退之前根环里的水位 max，不带暖机根自己的 11"
    );
    let rolled_back_verdicts =
        verdicts_without_any_violation(&pool.memory_pool(), "回退之后（环里还有 (1, 3)）");
    assert_judged_and_holding(&rolled_back_verdicts, &["I-7.8"], "回退之后");

    let instance = rolled_back.output.instance;
    let mut allocator = rolled_back.allocator.clone();
    let first_content = content_of(3300, 17);
    let first_after_rollback = publish_first_file_after_the_rollback(
        &mut pool,
        &mut allocator,
        &rolled_back.current,
        instance,
        &first_content,
    );
    assert_fresh_tree_identifiers(
        &first_after_rollback,
        ring_watermark_before_the_rollback,
        highest_tree_identifier_before_the_rollback,
    );
    assert_eq!(
        first_after_rollback.tree_identifiers.extent,
        TreeIdentifier(ring_watermark_before_the_rollback),
        "从回退带过来的水位起连号发：extent 树拿到的就是 19"
    );
    verdicts_without_any_violation(&pool.memory_pool(), "回退之后再发第一个文件版本");

    let overwrite_content = content_of(2900, 23);
    let overwritten = overwrite_in_process(&mut pool, &overwrite_content, instance);
    assert_eq!(
        overwritten.tree_identifiers, first_after_rollback.tree_identifiers,
        "覆盖写照抄这八个号，不另发"
    );
    let final_verdicts =
        verdicts_without_any_violation(&pool.memory_pool(), "回退之后第一个文件版本再覆盖写一版");
    assert_judged_and_holding(
        &final_verdicts,
        &["I-7.8", "I-9.14", "I-9.10"],
        "回退之后第一个文件版本再覆盖写一版",
    );
    assert_eq!(
        recover(&pool.memory_pool(), JournalPolicy::Consult).outcome,
        RecoveryOutcome::FileRead {
            root: (instance, overwritten.root.checkpoint_txg),
            content: overwrite_content,
        },
        "冷启动择覆盖写那一版的根，读回它的内容"
    );
}

/// 回退到暖机根 (1, 2) 之后再发的第一个文件版本从带过来的水位 19 起发号，中央映射树拿到 23（mkfs 那条流上是 15）：
/// 只读挂载按根记录里映射根指针头部的出生树认映射树的根（`mounted_read::open_pool_for_read`，D19（块指针的结构与宽度预算） 已定项 7），
/// 择到这一版、读回那个文件逐字节相同。号写死成 15 时这里打不开（映射树根头里是 23）；mkfs 那条流上 15 恰好对，
/// 那条流上的用例判不出（代码三方 `research/prompts/m2-wave3-code-r1-main-verification.md` 第三节 Y6 表里 `mounted_read.rs` 那一格）。
#[test]
fn the_read_only_mount_after_rolling_back_to_a_warm_up_root_finds_the_central_mapping_under_the_tree_its_root_pointer_names(
) {
    let mut pool = build_pool("step-four-rollback-to-warm-up-root-read-only-mount");
    let mut devices = pool.reopen_recorded();
    let rolled_back = mount_rollback(
        &parameters(),
        &mut devices,
        warm_up_root(),
        ShadowLedger::On,
    )
    .expect("环里还留着带文件版本的根时回退照常做");
    pool.devices = Some(devices);
    let mut allocator = rolled_back.allocator.clone();
    let content = content_of(3300, 17);
    let first_after_rollback = publish_first_file_after_the_rollback(
        &mut pool,
        &mut allocator,
        &rolled_back.current,
        rolled_back.output.instance,
        &content,
    );
    assert_eq!(
        (
            first_after_rollback.tree_identifiers.central_mapping,
            first_after_rollback.root.mapping_root.head.birth_tree
        ),
        (TreeIdentifier(23), TreeIdentifier(23)),
        "从 19 起连号发：中央映射树拿到 23，根记录里映射根指针的出生树也是 23"
    );
    let image = pool.memory_pool();
    let mounted = mount_read_only(&image).expect("只读挂载打得开");
    assert_eq!(
        (
            mounted.effective_root.instance,
            mounted.effective_root.checkpoint_txg
        ),
        (
            rolled_back.output.instance,
            first_after_rollback.root.checkpoint_txg
        ),
        "只读挂载择到回退之后那个第一个文件版本"
    );
    let file = mounted
        .mounted
        .open_file(&image, InodeNumber(FIRST_INODE_NUMBER))
        .expect("打开文件");
    let output = file
        .read_at(
            &image,
            FileOffsetInBytes(0),
            u64::try_from(content.len()).expect("文件长度"),
        )
        .expect("挂载态读");
    assert_eq!(output.bytes, content, "挂载态读回回退之后写的内容");
}

/// C511（回退到无文件那一版之后诞生代怎么接） 判别力那一格：回退到暖机根 (1, 2) 之后在同一个会话里连推零单元发布，直到根环里一条
/// 带文件版本的根都不剩（(1, 3) 的根槽被轮转盖掉），盘上仍留着 (1, 3) 那一版写出的 11..18 号码 2 单元（被抛弃、影子账隔离着，
/// 零单元发布一个槽都不取）。这时 I-7.8（根记录树 ID 水位不低于全池最大树 ID） 只剩新线上的根可取 max：回退那一版带环里的 max 19
/// 并一路照抄下来就成立；沿回退到的那一版带（暖机根的 11，今天之前的取法）就是 11 ≤ 18，当场红。
/// 之后退出、重开可写挂载（写行那次发布按环算水位：环里只剩新线上带 19 的根），再发第一个文件版本、覆盖写一版：
/// 八个号从 19 起发，池级 checker 一条违例都没有，I-7.8、I-9.14（树表条目的诞生 txg 跨根不变）、I-9.10（对象出生代三处一致） 都真被判过。
/// 重开这一步不省：同一个会话里转满一圈根环之后，被换下的 mkfs 实例表还在 defer 队列里、已经没有一条环里的根引用它，
/// I-3.1（已分配统计对得上） 在那里判红——那是里程碑「第二个事务」增补 2 收口表第 ② 行记着的「一次挂载转过一整圈根环」那一族，
/// 与水位无关；重开时的回收把它还回去。
#[test]
fn after_rolling_back_to_a_warm_up_root_the_ring_watermark_outlives_the_file_version_roots_leaving_the_ring(
) {
    let mut pool = build_pool("step-four-rollback-to-warm-up-root-ring-turns");
    let highest_tree_identifier_before_the_rollback = pool.output.tree_identifiers.highest();
    let ring_watermark_before_the_rollback =
        highest_watermark_among_ring_roots(&pool.memory_pool());
    let mut devices_for_the_rollback = pool.reopen_recorded();
    let rolled_back = mount_rollback(
        &parameters(),
        &mut devices_for_the_rollback,
        warm_up_root(),
        ShadowLedger::On,
    )
    .expect("环里还留着带文件版本的根时回退照常做");
    pool.devices = Some(devices_for_the_rollback);
    let instance = rolled_back.output.instance;
    let mut current = rolled_back.current.clone();
    let parameters_of_the_pool = parameters();
    let ring_slots = parameters_of_the_pool
        .geometry
        .root_ring_slots_per_region
        .count()
        * ROOT_RING_REGIONS;
    let ring_still_holds_a_file_version = |image: &MemoryPool| {
        readable_roots(
            image,
            &parameters_of_the_pool.region_devices,
            &parameters_of_the_pool.geometry,
            &parameters_of_the_pool.filesystem_identifier,
        )
        .iter()
        .any(|root| matches!(tree_table_has_no_entries(image, root), Ok(false)))
    };
    assert!(
        ring_still_holds_a_file_version(&pool.memory_pool()),
        "回退刚做完：(1, 3) 还在环里"
    );
    let mut empty_publishes = 0u64;
    while ring_still_holds_a_file_version(&pool.memory_pool()) {
        assert!(
            empty_publishes < ring_slots,
            "推满一圈根环 {ring_slots} 次之前 (1, 3) 的根槽一定被盖掉"
        );
        let version = current_version_without_file(&current);
        let open_devices = pool.devices.as_mut().expect("镜像还开着");
        let mut writer = PoolWriter::new(&parameters_of_the_pool, open_devices.as_mut_slice());
        let next = publish_without_units(
            &mut writer,
            &version.root,
            ZeroUnitPublishPlan {
                txg: CheckpointTxg(version.root.checkpoint_txg.0 + 1),
                counter: version.record.counter + 1,
                instance,
                back_chain: back_chain_of(&version.record_bytes),
                rollback_floor: version.root.rollback_floor,
                // 会话里接着现行那一版：它的水位就是根环里的 max（回退行那次发布取过，之后照抄）。
                tree_identifier_watermark: version.root.tree_identifier_watermark,
            },
        )
        .expect("零单元发布");
        current = PoolVersion::WithoutFile(next);
        empty_publishes += 1;
    }
    // 先判 checker 再核水位：沿回退到的那一版带水位时，这一步要红在 I-7.8 上（判别力自证就看这一格）。
    let turned_verdicts = verdicts_without_any_violation(
        &pool.memory_pool(),
        "回退之后推零单元发布到 (1, 3) 离开根环",
    );
    assert_judged_and_holding(&turned_verdicts, &["I-7.8"], "(1, 3) 离开根环之后");
    assert_eq!(
        highest_watermark_among_ring_roots(&pool.memory_pool()),
        ring_watermark_before_the_rollback,
        "(1, 3) 离开根环之后，环里新线上的根仍带 19"
    );

    let mut devices_for_the_remount = pool.reopen_recorded();
    let remounted =
        mount_writable(&parameters(), &mut devices_for_the_remount).expect("重开可写挂载");
    pool.devices = Some(devices_for_the_remount);
    assert_eq!(
        remounted.current.root().tree_identifier_watermark,
        ring_watermark_before_the_rollback,
        "重开时写行那次发布按环算水位：环里只剩新线上的根，都带 19"
    );
    let remounted_instance = remounted.output.instance;
    let mut allocator = remounted.allocator.clone();
    let first_after_rollback = publish_first_file_after_the_rollback(
        &mut pool,
        &mut allocator,
        &remounted.current,
        remounted_instance,
        &content_of(3100, 29),
    );
    assert_fresh_tree_identifiers(
        &first_after_rollback,
        ring_watermark_before_the_rollback,
        highest_tree_identifier_before_the_rollback,
    );
    verdicts_without_any_violation(
        &pool.memory_pool(),
        "(1, 3) 离开根环、重开之后再发第一个文件版本",
    );
    overwrite_in_process(&mut pool, &content_of(2700, 31), remounted_instance);
    let final_verdicts = verdicts_without_any_violation(
        &pool.memory_pool(),
        "(1, 3) 离开根环、重开之后第一个文件版本再覆盖写一版",
    );
    assert_judged_and_holding(
        &final_verdicts,
        &["I-7.8", "I-9.14", "I-9.10"],
        "(1, 3) 离开根环、重开之后第一个文件版本再覆盖写一版",
    );
}

/// C511（回退到无文件那一版之后诞生代怎么接） 第 3 步里条款没写的那一格（根环里读不出的根怎么算）按最保守的读法做：
/// 带文件版本的根 (1, 3) 那一槽读不出（介质错、持续），而它那次发布的记录还在环里——本实例第一次发布要带的水位
/// 仍是 19，取自那条记录的新根段（D23（journal 的角色与格式） 已定项 15）。只看读得出的根就只剩带 mkfs 种下的 11 的那几条，
/// 回退到暖机根之后再发第一个文件版本就会重发 (1, 3) 用过的 11..18（D8（核心索引结构） 已定项 8 ②）。
#[test]
fn with_the_file_version_root_slot_unreadable_the_ring_watermark_still_comes_from_its_journal_record(
) {
    let pool = build_pool("step-four-unreadable-file-version-root-watermark");
    let image = pool.memory_pool();
    let system_configuration = choose_system_configuration(&image).expect("系统配置");
    let immutable = &system_configuration.immutable;
    let file_version_root_slot = target_for_publish(
        CheckpointTxg(3),
        parameters().geometry.root_ring_slots_per_region,
    );
    let unreadable = PoolReaderWithUnreadableRootRingSlots::new(
        &image,
        RootRingSlotTarget {
            named_slots: NamedRootRingSlots::naming(&[file_version_root_slot]),
            region_devices: immutable.region_devices,
            fixed_structure_slot_spacing: immutable.sizes.fixed_structure_slot_spacing,
        },
    );
    let readable = readable_roots(
        &unreadable,
        &immutable.region_devices,
        &immutable.sizes,
        &immutable.filesystem_identifier,
    );
    assert!(
        !readable.is_empty()
            && readable
                .iter()
                .all(|root| root.checkpoint_txg != CheckpointTxg(3)),
        "(1, 3) 那一槽读不出、别的根读得出：{:?}",
        readable
            .iter()
            .map(|root| (root.instance, root.checkpoint_txg))
            .collect::<Vec<_>>()
    );
    assert_eq!(
        readable
            .iter()
            .map(|root| root.tree_identifier_watermark)
            .max(),
        Some(11),
        "读得出的根都带 mkfs 种下的 11"
    );
    let records = scan_journal(&unreadable, &system_configuration);
    assert!(
        records
            .values()
            .any(|record| record.new_tree_identifier_watermark == 19),
        "(1, 3) 那次发布的记录还在环里，新根段带 19"
    );
    assert_eq!(
        highest_tree_identifier_watermark_in_the_ring(
            &unreadable,
            &immutable.region_devices,
            &immutable.sizes,
            &immutable.filesystem_identifier,
            &records,
        ),
        Some(19),
        "根读不出、记录还在：水位按记录带的 19 算，不退回读得出的根带的 11"
    );
}

fn first_root() -> RollbackTarget {
    RollbackTarget {
        instance: InstanceGeneration(1),
        checkpoint_txg: CheckpointTxg(3),
    }
}

/// 进程退出、重开走回退到 A 的根；回来的可写态装回 pool。
fn rollback_to_first_root(pool: &mut BuiltPool, shadow_ledger: ShadowLedger) -> Mounted {
    let mut devices = pool.reopen_recorded();
    let rolled_back =
        mount_rollback(&parameters(), &mut devices, first_root(), shadow_ledger).expect("回退");
    pool.devices = Some(devices);
    pool.allocator = rolled_back.allocator.clone();
    pool.output = rolled_back
        .current
        .file_version()
        .expect("回退到 A，现行那一版带文件")
        .clone();
    rolled_back
}

fn region_device(txg: u64) -> DeviceIdentity {
    let target = target_for_publish(
        CheckpointTxg(txg),
        parameters().geometry.root_ring_slots_per_region,
    );
    parameters().region_devices[usize::try_from(target.region).expect("区域号")]
}

/// 一版账里这块盘上占着的每个槽（记录按跨度展开）。
fn slots_of(output: &TransactionOutput, device: DeviceIdentity) -> BTreeSet<u64> {
    output
        .allocation_records
        .iter()
        .filter(|record| record.device == device)
        .flat_map(|record| record.slot.0..record.slot.0 + u64::from(record.span_slots))
        .collect()
}

/// 验收第一条：回退行 (1, 3, 0)、中间实例行 (2, 0, 0)；D 的根 (3, 9)、jsn 9（接在环里最大的 jsn 8 之后，C340 取 P2）、事务号 0、反向链 0，
/// 重写实例表 + 四个固定点单元；暖机一次落到另一块盘；一条记录都不施加；冷启动读回第一次的内容；被抛弃根独占的槽逐盘 54 个、
/// D 与暖机一个都不落在上面；checker 全绿（I-3.1 的并集按实例表把被抛弃的根排除，I-3.8 看见回退行）。
#[test]
fn rolling_back_to_the_first_root_writes_the_rollback_row_and_the_intermediate_row_and_cold_start_reads_the_first_content(
) {
    let mut pool = build_through_third_publish("step-four-rollback");
    let third = pool.output.clone();
    let rolled_back = rollback_to_first_root(&mut pool, ShadowLedger::On);
    let output = &rolled_back.output;
    assert_eq!(output.instance, InstanceGeneration(3));
    assert_eq!(
        output.rows_written,
        vec![
            InstanceRow {
                instance: InstanceGeneration(1),
                selected_root_txg: CheckpointTxg(3),
                applied_transaction_high_water: 0,
                is_rollback: true,
            },
            InstanceRow {
                instance: InstanceGeneration(2),
                selected_root_txg: CheckpointTxg(0),
                applied_transaction_high_water: 0,
                is_rollback: false,
            },
        ],
        "回退行与中间实例行"
    );
    assert_eq!(output.journal.prefix_applied, 0, "A 之后的记录一条都不施加");
    assert_eq!(
        (
            output.effective_root.instance,
            output.effective_root.checkpoint_txg
        ),
        (InstanceGeneration(1), CheckpointTxg(3))
    );
    let rollback_publish = output
        .row_publish
        .file_version()
        .expect("回退到带文件的 A：D 是带文件的一版");
    assert_eq!(
        (
            rollback_publish.root.instance,
            rollback_publish.root.checkpoint_txg,
            rollback_publish.record.counter,
            rollback_publish.record.transaction,
            rollback_publish.record.back_chain
        ),
        (InstanceGeneration(3), CheckpointTxg(9), 9, 0, 0),
        "D：txg = max(根环 8, 记录 8) + 1；jsn 接在环里最大的 8 之后（C340 取 P2，被抛弃的记录一条不盖）；本实例第一条反向链 0"
    );
    // 分配记录树按位置寻址（D8（核心索引结构） 已定项 14）：D 这一版的账里有被抛弃的 B、C 那几次发布之后才用到的槽（叶 62），
    // 写行那次两块盘各重写叶 61、叶 62、第 1 层那一个与根，先叶后根。
    let allocation_record_tree_node = |level: u8, device: u32, index_in_device: u64| {
        TransactionUnit::AllocationTreeNodeBelowTheRoot(AllocationRecordTreeNodePosition {
            level,
            device: DeviceIdentity(device),
            index_in_device,
        })
    };
    assert_eq!(
        rollback_publish.rewritten,
        vec![
            TransactionUnit::InstanceTable,
            allocation_record_tree_node(0, 0, 61),
            allocation_record_tree_node(0, 0, 62),
            allocation_record_tree_node(0, 1, 61),
            allocation_record_tree_node(0, 1, 62),
            allocation_record_tree_node(1, 0, 0),
            allocation_record_tree_node(1, 1, 0),
            TransactionUnit::AllocationTree,
            TransactionUnit::AccountingTree,
            TransactionUnit::MappingTree,
            TransactionUnit::TreeTable,
        ]
    );
    assert_eq!(
        output.warm_up_publishes.len(),
        1,
        "D 落盘 0、txg 10 落盘 1，一次就够"
    );
    let warm_up = output.warm_up_publishes[0]
        .file_version()
        .expect("带文件的一版上的暖机");
    assert_eq!(warm_up.root.checkpoint_txg, CheckpointTxg(10));
    assert_ne!(region_device(9), region_device(10));
    assert_eq!(warm_up.record.counter, 10);

    // 影子账：被抛弃的根 B、(2, 5)、(2, 6)、(2, 7)、C 引用而 A 不引用的槽——B 14、写行 10、暖机 8 + 8、C 14 = 54 个槽，逐盘
    // （每次发布连分配记录树五个节点，D8（核心索引结构） 已定项 14）。
    for device in [DeviceIdentity(0), DeviceIdentity(1)] {
        let abandoned: BTreeSet<u64> = slots_of(&third, device)
            .difference(&slots_of(rollback_publish, device))
            .copied()
            .collect();
        assert_eq!(abandoned.len(), 54, "盘 {device:?} 上只被被抛弃根引用的槽");
        // 影子账按窄读法只隔离这 54 个：mkfs 实例表那 2 个槽 A（候选）与 B 都引用，不在其内。
        assert!(
            output.isolated_slots_per_device.contains(&(device, 54)),
            "隔离的槽数 = 只被被抛弃根引用的 54 个 {:?}",
            output.isolated_slots_per_device
        );
        for publish in std::iter::once(rollback_publish).chain(
            output
                .warm_up_publishes
                .iter()
                .map(|publish| publish.file_version().expect("带文件的一版上的暖机")),
        ) {
            for placement in publish.placements() {
                for slot in placement.slot.0..placement.slot.0 + placement.span {
                    assert!(
                        !abandoned.contains(&slot),
                        "txg {} 的落点 {slot} 落在被抛弃根引用的槽上",
                        publish.root.checkpoint_txg.0
                    );
                }
            }
        }
    }

    let image = pool.memory_pool();
    let report = recover(&image, JournalPolicy::Consult);
    assert_eq!(
        report.outcome,
        RecoveryOutcome::FileRead {
            root: (InstanceGeneration(3), CheckpointTxg(10)),
            content: file_content()
        },
        "{:?}",
        report.journal
    );
    assert_eq!(
        report.journal.valid_records, 10,
        "jsn 1–8 原样在（P2 一条不盖）、9 是 D、10 是暖机"
    );
    assert_eq!(
        report.journal.prefix_applied, 0,
        "(3, 10) 之后 jsn 11 一条都没有"
    );
    let verdicts = check_pool_image(&image);
    for (invariant, verdict) in &verdicts {
        if *invariant == "I-8.8" || *invariant == "I-7.9" {
            // 一事务一条、每条都带提交标记：I-8.8（前缀里的事务不被切开） 的 ③ ④ 没有对象，报不适用
            // （判别力在 `checker_known_bad_images.rs`）。这段历史没抬过 F：I-7.9（回退下界 F 不高于抬 F 的上限） 没有抬 F 的根可判，
            // 报不适用（回退那条根带的是恢复算出的 F_生效，不算抬；阳性对照与坏镜像同在 `checker_known_bad_images.rs`）。
            assert!(
                matches!(verdict, InvariantVerdict::NotApplicable(_)),
                "{invariant} 在回退之后的镜像上报不适用：{verdict:?}"
            );
            continue;
        }
        assert_eq!(
            *verdict,
            InvariantVerdict::Holds,
            "{invariant} 在回退之后的镜像上要成立"
        );
    }
    for must_hold in ["I-3.1", "I-3.8", "I-5.2", "I-7.2"] {
        assert!(
            verdicts.iter().any(|(name, _)| *name == must_hold),
            "{must_hold} 真被判过"
        );
    }
}

/// 回退候选集（D23 已定项 14）：实例表里有行 (i, Ti, Wi) 的实例，只有 T ≤ Ti 的根可选——B 的根 (1, 4) 与 C 的根 (2, 8) 都是被抛弃时间线的
/// （F 还是 0，不低于 F），报 `OnAbandonedTimeline`；根环里没有的 (1, 42) 报 `NotInRing`。调用方按这个字段分流，不看文字。
#[test]
fn rolling_back_onto_an_abandoned_timeline_or_a_missing_root_is_refused() {
    let mut pool = build_through_third_publish("step-four-refused");
    rollback_to_first_root(&mut pool, ShadowLedger::On);
    let missing = RollbackTarget {
        instance: InstanceGeneration(1),
        checkpoint_txg: CheckpointTxg(42),
    };
    for (target, expected) in [
        (
            RollbackTarget {
                instance: InstanceGeneration(1),
                checkpoint_txg: CheckpointTxg(4),
            },
            RollbackCandidateExclusion::OnAbandonedTimeline,
        ),
        (
            RollbackTarget {
                instance: InstanceGeneration(2),
                checkpoint_txg: CheckpointTxg(8),
            },
            RollbackCandidateExclusion::OnAbandonedTimeline,
        ),
        (missing, RollbackCandidateExclusion::NotInRing),
    ] {
        let mut devices = pool.reopen_recorded();
        let refused = mount_rollback(&parameters(), &mut devices, target, ShadowLedger::On);
        pool.devices = Some(devices);
        match refused {
            Err(MountError::RollbackTargetNotACandidate {
                target: reported,
                exclusion,
            }) => assert_eq!((reported, exclusion), (target, expected)),
            other => panic!(
                "{target:?} 该被拒：{:?}",
                other.map(|mounted| mounted.output.instance)
            ),
        }
    }
}

/// 被抛弃的 C 那条根 (2, 8) 从盘上读出来，沿它走到文件（`recovery::walk_to_file`：恢复择到一条根之后走的同一段；
/// C 之上没有实例 2 的记录要施加）。回退见证（D23（journal 的角色与格式） 已定项 14「回退见证」）让恢复不再择它；
/// 它仍是影子账要护的对象：回退那一次挂载崩在写行的根落了、轮换还没落的那一格（post 写序认下的窗口），回退实例的根又全读不出时，
/// 恢复择的就是它（`second_transaction_supplement_two_rollback_witness.rs` 的 Z9-B 那条用例 m = 0 那一格）。
fn walk_to_the_file_under_the_abandoned_root_c(
    reader: &dyn singlefs_core::recovery::PoolReader,
) -> Result<Option<Vec<u8>>, RecoveryFailure> {
    let system_configuration = choose_system_configuration(reader).expect("系统配置");
    let root_c = readable_roots(
        reader,
        &system_configuration.immutable.region_devices,
        &system_configuration.immutable.sizes,
        &system_configuration.immutable.filesystem_identifier,
    )
    .into_iter()
    .find(|root| (root.instance, root.checkpoint_txg) == (InstanceGeneration(2), CheckpointTxg(8)))
    .expect("C 的根 (2, 8) 还在根环里、读得出");
    walk_to_file(reader, &root_c, &mut 0)
}

/// C314（回退可以复用被抛弃的根引用的单元） 那一格的必红，影子账开关强制进入：关掉影子账，回退之后再发两版文件，
/// 数据单元落回 B 与 C 的数据槽（50182、50184）；沿 C 的根 (2, 8) 走到文件，它的数据单元已被盖掉，读不出第三次的内容。
/// 影子账开着：两版数据落 50186、50188，沿 C 的根走到文件、第三次的内容原样读回。
/// 把实例 3 的四个根槽都改坏之后，恢复不再挂上 C：回退见证 (3, 1, 3) 抛弃 B、C（D23（journal 的角色与格式） 已定项 14「回退见证」），
/// 两臂都落到 R_old (1, 3)、读回第一次的内容——实二二三之前恢复挂上的是 C，这条用例在那里判影子账（那时它红在这一处）。
#[test]
fn without_the_shadow_ledger_publishes_after_the_rollback_reuse_the_abandoned_data_slots_and_the_abandoned_root_reads_a_torn_unit_while_the_witness_keeps_recovery_off_it(
) {
    for (shadow_ledger, expected_slots, expect_third_content_readable) in [
        (ShadowLedger::Off, [50182, 50184], false),
        (ShadowLedger::On, [50186, 50188], true),
    ] {
        let mut pool = build_through_third_publish("step-four-shadow");
        let rolled_back = rollback_to_first_root(&mut pool, shadow_ledger);
        // 五条硬要求第 4 条：分支必须可观测。⚠️ 光看 `isolated_slots_per_device` 分不开两臂——
        // `Off` 恒 0，而 `On` 在「没有被抛弃根、或它们引用的槽都还被候选集引用着」时也是 0；
        // 这条脚本恰好隔离了 54 个，换一段历史就不一定。分支名两臂永远不同。
        assert_eq!(
            rolled_back.output.shadow_ledger_branch,
            shadow_ledger.branch_name(),
            "挂载报出的分支名要与传进去的那一臂相符"
        );
        assert_ne!(
            rolled_back.output.shadow_ledger_branch,
            (if shadow_ledger == ShadowLedger::On {
                ShadowLedger::Off
            } else {
                ShadowLedger::On
            })
            .branch_name(),
            "两臂报的分支名不许相同：相同就等于运行时看不出走了哪一条"
        );
        let expected_isolated = if shadow_ledger == ShadowLedger::On {
            54
        } else {
            0
        };
        assert!(
            rolled_back
                .output
                .isolated_slots_per_device
                .iter()
                .all(|(_, isolated)| *isolated == expected_isolated),
            "{shadow_ledger:?}：{:?}",
            rolled_back.output.isolated_slots_per_device
        );
        let fourth = overwrite_in_process(&mut pool, &content_of(3000, 5), InstanceGeneration(3));
        let fifth = overwrite_in_process(&mut pool, &content_of(3100, 9), InstanceGeneration(3));
        assert_eq!(
            [
                fourth.data_pointers[0].locations[0].slot.0,
                fifth.data_pointers[0].locations[0].slot.0
            ],
            expected_slots,
            "{shadow_ledger:?} 下回退之后两版的数据落点"
        );
        let mut image = pool.memory_pool();
        for txg in [9u64, 10, 11, 12] {
            let target = target_for_publish(
                CheckpointTxg(txg),
                parameters().geometry.root_ring_slots_per_region,
            );
            image.flip_byte(region_device(txg), slot_offset(target, 4096), 100);
        }
        let report = recover(&image, JournalPolicy::Consult);
        assert_eq!(
            report.outcome,
            RecoveryOutcome::FileRead {
                root: (InstanceGeneration(1), CheckpointTxg(3)),
                content: file_content()
            },
            "{shadow_ledger:?}：回退见证抛弃 B、C，实例 3 的根全读不出时恢复落到 R_old (1, 3)"
        );
        let walked_under_c = walk_to_the_file_under_the_abandoned_root_c(&image);
        if expect_third_content_readable {
            assert_eq!(
                walked_under_c,
                Ok(Some(third_content())),
                "影子账开着：C 引用的单元一个没被盖，沿 C 的根读回第三次的内容"
            );
        } else {
            assert_ne!(
                walked_under_c,
                Ok(Some(third_content())),
                "影子账关着：C 的数据单元已被第五版盖掉，沿 C 的根读不回第三次的内容"
            );
        }
    }
}

/// 回退之后，回退实例（实例 3）发布过的四条根落在两块盘上（txg 9、11 在盘 0，txg 10、12 在盘 1）：按 (区域, 槽) 点名它们。
fn root_ring_slots_of_the_rollback_instance() -> NamedRootRingSlots {
    NamedRootRingSlots::naming(&[9u64, 10, 11, 12].map(|txg| {
        target_for_publish(
            CheckpointTxg(txg),
            parameters().geometry.root_ring_slots_per_region,
        )
    }))
}

/// 必红八条第八条（影子账关掉）的 ② 格：「让回退实例的根全读不出」。
///
/// 这一格**不是层 0 的崩溃状态**（前缀里的根按定义已持久，枚举不出「已持久的根不见了」），是层 0 之外的一次故障注入，
/// 靠步 0 的第五个开关强制进入（里程碑「第二个事务」第 228 行逐字：不许以「跑不到」静默通过）。
///
/// 与同一个文件里那条用 `flip_byte` 改坏根槽的用例分工不同：那条造的是「读得出、自证不过」（字节坏了），
/// 这条造的是「读返回失败」（介质错），而且**持续**——第一版没有根环槽的重定位
/// （C335（根槽持续读不出时实例表只增不减）），读不出的槽永远读不通，重试一遍还是读不出。
/// C332（回退实例两个根都读不出时回退被撤销） 要的正是这一形：回退实例的根在两块盘上都读不出。回退见证（D23（journal 的角色与格式）
/// 已定项 14「回退见证」，C332 的修法）之后恢复不再退到被抛弃时间线上的根 (2, 8)：见证 (3, 1, 3) 抛弃 B、C，两臂都落到 R_old (1, 3)、
/// 读回第一次的内容，两遍结论逐字相同（实二二三之前退到的是 C，这条用例在那里红）。
///
/// 影子账那一半改成沿 C 的根直接走到文件（`walk_to_the_file_under_the_abandoned_root_c`；恢复择 C 的那一格见那个函数的注释）：
/// 影子账关着：回退之后两版数据落回 B 与 C 的数据槽，C 的数据单元已被盖掉 ⇒ 读不回第三次的内容；
/// 影子账开着：C 引用的单元一个没被盖，第三次的内容原样读回。判别力自证就是这两遍的差。
#[test]
fn with_the_shadow_ledger_off_and_every_root_of_the_rollback_instance_unreadable_the_witness_keeps_recovery_on_the_rollback_target_and_the_abandoned_root_reads_a_torn_unit(
) {
    for (shadow_ledger, expect_third_content_readable) in
        [(ShadowLedger::Off, false), (ShadowLedger::On, true)]
    {
        let mut pool = build_through_third_publish("step-four-unreadable-rollback-roots");
        rollback_to_first_root(&mut pool, shadow_ledger);
        overwrite_in_process(&mut pool, &content_of(3000, 5), InstanceGeneration(3));
        overwrite_in_process(&mut pool, &content_of(3100, 9), InstanceGeneration(3));

        let image = pool.memory_pool();
        let system_configuration = choose_system_configuration(&image).expect("系统配置");
        let unreadable = PoolReaderWithUnreadableRootRingSlots::new(
            &image,
            RootRingSlotTarget {
                named_slots: root_ring_slots_of_the_rollback_instance(),
                region_devices: system_configuration.immutable.region_devices,
                fixed_structure_slot_spacing: system_configuration
                    .immutable
                    .sizes
                    .fixed_structure_slot_spacing,
            },
        );
        let readable = readable_roots(
            &unreadable,
            &system_configuration.immutable.region_devices,
            &system_configuration.immutable.sizes,
            &system_configuration.immutable.filesystem_identifier,
        );
        assert!(
            readable
                .iter()
                .all(|root| root.instance != InstanceGeneration(3)),
            "{shadow_ledger:?}：实例 3 的根一条都读不出了：{:?}",
            readable
                .iter()
                .map(|root| (root.instance, root.checkpoint_txg))
                .collect::<Vec<_>>()
        );
        assert!(
            unreadable.reads_refused() >= 4,
            "{shadow_ledger:?}：四个点名的槽都被拦过：{}",
            unreadable.reads_refused()
        );

        let outcome = recover(&unreadable, JournalPolicy::Consult).outcome;
        let refused_after_the_first_recovery = unreadable.reads_refused();
        assert_eq!(
            outcome,
            RecoveryOutcome::FileRead {
                root: (InstanceGeneration(1), CheckpointTxg(3)),
                content: file_content(),
            },
            "{shadow_ledger:?}：回退见证抛弃 B、C，四条根读不出之后恢复落到 R_old (1, 3)、读回第一次的内容"
        );
        let walked_under_c = walk_to_the_file_under_the_abandoned_root_c(&unreadable);
        if expect_third_content_readable {
            assert_eq!(
                walked_under_c,
                Ok(Some(third_content())),
                "影子账开着：C 引用的单元一个没被盖，沿 C 的根读回第三次的内容"
            );
        } else {
            assert_ne!(
                walked_under_c,
                Ok(Some(third_content())),
                "影子账关着：C 的数据单元已被回退之后那两版盖掉，沿 C 的根读不回第三次的内容"
            );
        }
        // 持续：同一份镜像再恢复一遍，拦下的读数接着涨、结论逐字相同（一次瞬时错顶不上 C335 的论证）。
        let outcome_again = recover(&unreadable, JournalPolicy::Consult).outcome;
        assert_eq!(
            outcome_again, outcome,
            "{shadow_ledger:?}：第二遍恢复的结局与第一遍逐字相同"
        );
        assert!(
            unreadable.reads_refused() > refused_after_the_first_recovery,
            "{shadow_ledger:?}：第二遍又拦下了读（读不出的槽永远读不通）：{} → {}",
            refused_after_the_first_recovery,
            unreadable.reads_refused()
        );
    }
}

/// 前缀第五条（D23 已定项 14）：所选根的实例有回退行时，该实例的记录只施加到回退行的 W 为止——直接喂 `replay_journal`：
/// 到 B 为止的镜像上选 A 的根 (1, 3)，不带回退行施加 B 那条（事务号 2）；W = 0 一条都不施加；W = 2 施加到 B；W = 1 停在 B 之前。
#[test]
fn the_rollback_row_caps_the_prefix_of_the_chosen_roots_instance_at_its_high_water() {
    let mut pool = build_pool("step-four-cap");
    overwrite_in_process(&mut pool, &second_content(), InstanceGeneration(1));
    let image = pool.memory_pool();
    let system_configuration = choose_system_configuration(&image).expect("系统配置");
    let newest = choose_root(&image, &system_configuration).expect("B 的根");
    assert_eq!(newest.checkpoint_txg, CheckpointTxg(4));
    let records = scan_journal(&image, &system_configuration);
    let roots = singlefs_core::recovery::readable_roots(
        &image,
        &system_configuration.immutable.region_devices,
        &system_configuration.immutable.sizes,
        &system_configuration.immutable.filesystem_identifier,
    );
    let first = roots
        .iter()
        .find(|root| root.checkpoint_txg == CheckpointTxg(3))
        .copied()
        .expect("A 的根在环里");
    for (high_water, expected_applied, expected_txg) in [
        (None, 1, 4u64),
        (Some(0), 0, 3),
        (Some(1), 0, 3),
        (Some(2), 1, 4),
    ] {
        let (report, effective) = replay_journal(
            &image,
            &first,
            system_configuration.immutable.sizes.journal_ring_bytes,
            &records,
            true,
            high_water,
        )
        .expect("所选根那次发布只有一条记录带末条标志：锚点认得出");
        assert_eq!(
            (report.prefix_applied, effective.checkpoint_txg.0),
            (expected_applied, expected_txg),
            "回退行 W = {high_water:?}"
        );
    }
}

/// C340 取 P2：回退之后新实例的第一条 jsn 接在环里最大的 jsn 之后，被抛弃发布的记录一条不盖——B 的记录已提交、根还没落盘时发起回退，
/// 崩在回退生效之前那次恢复才仍「与没发起回退时相同」（步 4 / 步 5 代码三方第一轮云端攻方腿打中 P1 盖掉 B 那条）。
/// 影子账只住内存，所以每次挂载都重算：回退之后普通重开一次，被抛弃根引用的槽照样隔离（同一轮攻方腿打中重开后隔离归零）。
#[test]
fn rolling_back_keeps_the_abandoned_records_in_the_ring_and_a_plain_remount_keeps_the_isolation() {
    let mut pool = build_through_third_publish("step-four-p2-remount");
    let rolled_back = rollback_to_first_root(&mut pool, ShadowLedger::On);
    assert_eq!(
        rolled_back.output.row_publish.record().counter,
        9,
        "D 的 jsn 接在 C 的 8 之后"
    );
    let image = pool.memory_pool();
    let system_configuration = choose_system_configuration(&image).expect("系统配置");
    let records = scan_journal(&image, &system_configuration);
    for (instance, counter) in [(1u32, 4u64), (2, 5), (2, 8), (3, 9), (3, 10)] {
        assert!(
            records.contains_key(&(InstanceGeneration(instance), counter)),
            "记录 ({instance}, jsn {counter}) 该原样在环里：{:?}",
            records.keys().collect::<Vec<_>>()
        );
    }
    let mut devices = pool.reopen_recorded();
    let remounted = mount_writable(&parameters(), &mut devices).expect("回退之后普通重开");
    pool.devices = Some(devices);
    pool.allocator = remounted.allocator.clone();
    pool.output = remounted
        .current
        .file_version()
        .expect("回退之后重开，现行那一版带文件")
        .clone();
    assert_eq!(remounted.output.instance, InstanceGeneration(4));
    assert_eq!(
        remounted.output.isolated_slots_per_device,
        vec![(DeviceIdentity(0), 54), (DeviceIdentity(1), 54)],
        "按 D 那一版实例表判被抛弃的根（B、实例 2 的四条）引用的槽，普通重开照样隔离：B 14、写行 10、两次暖机各 8、C 14 \
         （每次发布连分配记录树五个节点，D8（核心索引结构） 已定项 14）"
    );
    assert_eq!(remounted.output.abandoned_roots_unreadable, 0);
    // 被抛弃根引用的槽：B 的数据 50182–50183、C 的数据 50184–50185 与它们的节点（隔离），以及 mkfs 实例表 50176–50177
    // （A 也引用、不隔离，但 D 释放它之后还在 defer 里）——重开后的发布一个都不该落上去。
    let abandoned: BTreeSet<u64> = (50176..50178).chain(50182..50186).collect();
    let next = overwrite_in_process(&mut pool, &content_of(2100, 41), InstanceGeneration(4));
    for placement in next.placements() {
        for slot in placement.slot.0..placement.slot.0 + placement.span {
            assert!(
                !abandoned.contains(&slot),
                "重开后的发布落到了被抛弃根引用的槽 {slot}"
            );
        }
    }
}

/// 被抛弃根 C 的树表单元在两块盘上都改坏：影子账罩不到 C（读不出就没法知道它引用谁），挂载照样成功、只计数一条读不出的被抛弃根，
/// 别的被抛弃根引用的槽照旧隔离（步 4 / 步 5 代码三方第二轮云端攻方腿打中：一条被抛弃根的树表撕裂不能让每次挂载都失败）。
#[test]
fn torn_tree_table_of_an_abandoned_root_is_counted_and_does_not_fail_the_mount() {
    let mut pool = build_through_third_publish("step-four-torn-abandoned");
    let third = pool.output.clone();
    rollback_to_first_root(&mut pool, ShadowLedger::On);
    let mut devices = pool.reopen_recorded();
    for location in &third.root.tree_table.locations {
        let (_, recorded) = devices
            .iter_mut()
            .find(|(identity, _)| *identity == location.device)
            .expect("C 的树表所在的盘");
        let offset = location.slot.to_device_offset();
        let mut bytes = vec![0u8; usize::try_from(singlefs_format::NODE_BYTES).expect("16384")];
        recorded.read_at(offset, &mut bytes).expect("读 C 的树表");
        bytes[200] ^= 0xff;
        recorded
            .write_at(offset, &bytes, WriteDurability::Plain)
            .expect("改坏 C 的树表");
    }
    let remounted =
        mount_writable(&parameters(), &mut devices).expect("被抛弃根的树表撕裂不拒绝挂载");
    assert_eq!(remounted.output.abandoned_roots_unreadable, 1, "C 读不出");
    // C 独占的槽（它的数据 2 + 它的节点）罩不到；B、写行与暖机那些照旧隔离。
    assert_eq!(
        remounted.output.isolated_slots_per_device,
        vec![(DeviceIdentity(0), 40), (DeviceIdentity(1), 40)],
        "少了只被 C 引用的 14 个槽"
    );
}

/// 一条根槽在盘上的位置：哪块盘、哪个偏移。**两个量不是一个**（`recovery::visit_valid_roots` 同一条口径）：
/// 槽与槽之间的间距是 `fixed_structure_slot_spacing`（4096），槽自己的判定宽度是 `physical_block_size`（512）。
fn root_slot_position_of(txg: u64) -> (DeviceIdentity, DeviceOffsetInBytes) {
    (
        region_device(txg),
        slot_offset(
            target_for_publish(
                CheckpointTxg(txg),
                parameters().geometry.root_ring_slots_per_region,
            ),
            parameters().geometry.fixed_structure_slot_spacing,
        ),
    )
}

fn read_root_slot(
    devices: &mut [(DeviceIdentity, impl BlockDevice)],
    (device, offset): (DeviceIdentity, DeviceOffsetInBytes),
) -> Vec<u8> {
    let (_, recorded) = devices
        .iter_mut()
        .find(|(identity, _)| *identity == device)
        .expect("这条根的根槽所在的盘");
    let mut slot_bytes =
        vec![0u8; usize::try_from(parameters().geometry.physical_block_size).expect("4096")];
    recorded.read_at(offset, &mut slot_bytes).expect("读根槽");
    slot_bytes
}

fn write_root_slot(
    devices: &mut [(DeviceIdentity, impl BlockDevice)],
    (device, offset): (DeviceIdentity, DeviceOffsetInBytes),
    slot_bytes: &[u8],
) {
    let (_, recorded) = devices
        .iter_mut()
        .find(|(identity, _)| *identity == device)
        .expect("这条根的根槽所在的盘");
    recorded
        .write_at(offset, slot_bytes, WriteDurability::Plain)
        .expect("写回根槽");
}

/// 两块盘的同一个槽上读同一个单元：读第一块盘那一份（两盘同槽、内容相同，D2 已定项 10）。
fn read_unit_at(
    devices: &mut [(DeviceIdentity, impl BlockDevice)],
    location: LocationEntry,
    unit_bytes: usize,
) -> Vec<u8> {
    let (_, recorded) = devices
        .iter_mut()
        .find(|(identity, _)| *identity == location.device)
        .expect("这个单元所在的盘");
    let mut bytes = vec![0u8; unit_bytes];
    recorded
        .read_at(location.slot.to_device_offset(), &mut bytes)
        .expect("读单元");
    bytes
}

fn write_unit_to_every_location(
    devices: &mut [(DeviceIdentity, impl BlockDevice)],
    locations: [LocationEntry; 2],
    bytes: &[u8],
) {
    for location in locations {
        let (_, recorded) = devices
            .iter_mut()
            .find(|(identity, _)| *identity == location.device)
            .expect("这个单元所在的盘");
        recorded
            .write_at(
                location.slot.to_device_offset(),
                bytes,
                WriteDurability::Plain,
            )
            .expect("写单元");
    }
}

/// panic 面普查 R7（被抛弃根那棵账里的槽号与跨度 ⇒ `DeviceFreeMap::isolate` 的跨度断言）：
/// 影子账把**被抛弃根**那棵账里的每条记录原样喂进 `PoolAllocator::isolate_abandoned`，而那棵账是盘上读来的。
/// 这里把被抛弃根 C 那棵账最左那片叶的第一条分配记录改成「起点贴着单元区末尾、跨度 32767 槽」，链上校验和逐道重算——
/// 少重算一道，`allocation_records_under_root` 在读单元那一步就先拒了，坏法打不到要打的那一处。
///
/// 钉三样：**一、挂载不 panic**（这样的记录在进分配器之前判掉，返回 `AllocationRecordOutsideThePoolGeometry`）；
/// 二、C 因此被计成一条「账读不出的被抛弃根」，挂载照样成功（与树表撕裂那一条同一条口径：一条被抛弃根的账坏了不能让每次挂载都失败）；
/// 三、只被 C 引用的槽因此罩不到，隔离数与树表撕裂那一条相同。
///
/// 分配记录树按绝对槽号按位置寻址之后（D8（核心索引结构） 已定项 14），接走这条记录的是读树那一步的叶判：起点不在它所在叶按位置罩的
/// 那一段里（`allocation_record_tree::read_allocation_record_tree`）。此前钉的是 `recovery::allocation_records_fit_the_pool_geometry`
/// 里「跨度越过单元区末尾」那一判与这条调用链接上了（删掉它挂载就在 `isolate` 的跨度断言上 panic）；那一判如今只有罩着盘末尾的那片叶
/// 上的记录够得着，这条坏法打不到它（见 `bad_disk_input::move_the_first_allocation_record_past_the_end_of_the_unit_area_and_reseal_the_chain`
/// 的注），交回里写明。
#[test]
fn an_abandoned_roots_allocation_record_whose_span_runs_past_the_unit_area_is_counted_and_does_not_panic(
) {
    let mut pool = build_through_third_publish("step-four-abandoned-span");
    let third = pool.output.clone();
    rollback_to_first_root(&mut pool, ShadowLedger::On);
    let mut devices = pool.reopen_recorded();

    let node_bytes = usize::try_from(singlefs_format::NODE_BYTES).expect("16384");
    let allocation_tree_pointer = third
        .tree_table_entries
        .iter()
        .find(|entry| entry.kind == TREE_KIND_ALLOCATION)
        .expect("C 的树表里有分配记录树")
        .root;
    assert_eq!(
        Some(allocation_tree_pointer),
        third
            .allocation_record_tree
            .pointer_of(AllocationRecordTreeNode::Root),
        "树表里那条根指针就是 C 那棵分配记录树的根"
    );
    // 从根沿每层第 0 条条目往下的那一路（D8（核心索引结构） 已定项 14：内部条目按孩子那一段的起点升序，第 0 条是盘 0 最低的那一段）：
    // 根、盘 0 第 1 层第 0 个、盘 0 最低那片叶。
    let lowest_leaf_of_device_zero = third
        .allocation_record_tree
        .nodes
        .iter()
        .filter_map(|(node, _)| match node {
            AllocationRecordTreeNode::BelowTheRoot(position)
                if position.level == 0 && position.device == DeviceIdentity(0) =>
            {
                Some(*node)
            }
            AllocationRecordTreeNode::BelowTheRoot(_) | AllocationRecordTreeNode::Root => None,
        })
        .min()
        .expect("盘 0 上至少一片装着记录的叶");
    let path_pointers: Vec<_> = [
        AllocationRecordTreeNode::Root,
        AllocationRecordTreeNode::BelowTheRoot(AllocationRecordTreeNodePosition {
            level: 1,
            device: DeviceIdentity(0),
            index_in_device: 0,
        }),
        lowest_leaf_of_device_zero,
    ]
    .into_iter()
    .map(|node| {
        third
            .allocation_record_tree
            .pointer_of(node)
            .expect("C 那棵树里有这个节点")
    })
    .collect();
    let root_slot_position = root_slot_position_of(third.root.checkpoint_txg.0);
    let mut root_slot = read_root_slot(&mut devices, root_slot_position);
    let mut tree_table_node =
        read_unit_at(&mut devices, third.root.tree_table.locations[0], node_bytes);
    let mut allocation_tree_nodes_from_the_root: Vec<Vec<u8>> = path_pointers
        .iter()
        .map(|pointer| read_unit_at(&mut devices, pointer.locations[0], node_bytes))
        .collect();

    let unit_area_end_slot = UNIT_AREA_START_SLOT + unit_area_slots_of_device(IMAGE_BYTES);
    let what = move_the_first_allocation_record_past_the_end_of_the_unit_area_and_reseal_the_chain(
        &mut root_slot,
        &mut tree_table_node,
        &mut allocation_tree_nodes_from_the_root,
        unit_area_end_slot,
    )
    .expect("C 那棵账最左那片叶装着记录");
    for (pointer, bytes) in path_pointers
        .iter()
        .zip(&allocation_tree_nodes_from_the_root)
    {
        write_unit_to_every_location(&mut devices, pointer.locations, bytes);
    }
    write_unit_to_every_location(
        &mut devices,
        third.root.tree_table.locations,
        &tree_table_node,
    );
    write_root_slot(&mut devices, root_slot_position, &root_slot);

    // 先直接钉住那道判交回的**成员**：链上校验和都重算过，读得出、解得开，拦着这条记录的只有位置那一判
    // （它不在所在叶按位置罩的那一段里）。少钉这一格，用例在「链没重算全、单元根本读不出来」
    // 那种情形下也照样绿——影子账对「读不出」与「判红」做的是同一件事（只计数）。
    // 根要从盘上重新读回来：`third.root` 是改坏之前那一份，它那条树表指针里的整单元校验和还是旧的。
    let system_configuration = choose_system_configuration(&devices).expect("系统配置");
    let roots_on_disk = readable_roots(
        &devices,
        &system_configuration.immutable.region_devices,
        &system_configuration.immutable.sizes,
        &system_configuration.immutable.filesystem_identifier,
    );
    let abandoned_root_on_disk = roots_on_disk
        .iter()
        .find(|root| {
            root.instance == third.root.instance && root.checkpoint_txg == third.root.checkpoint_txg
        })
        .copied()
        .expect("C 的根槽自证校验和重算过，仍然读得出");
    let ledger = allocation_records_under_root(&devices, &abandoned_root_on_disk);
    assert!(
        matches!(
            ledger,
            Err(RecoveryFailure::AllocationRecordOutsideThePoolGeometry {
                what: "分配记录不在它所在叶按位置罩的那一段里，或末槽越过叶的末槽"
            })
        ),
        "C 那棵账该报「不在它所在叶按位置罩的那一段里」，实际交回的是 {ledger:?}（{what}；C 的树表在槽 {:?}、分配记录树根在槽 {:?}）",
        third.root.tree_table.locations.map(|location| location.slot),
        allocation_tree_pointer.locations.map(|location| location.slot)
    );

    let remounted = mount_writable(&parameters(), &mut devices)
        .unwrap_or_else(|error| panic!("被抛弃根那棵账里的坏记录不拒绝挂载：{error:?}（{what}）"));
    assert_eq!(
        remounted.output.abandoned_roots_unreadable, 1,
        "C 那棵账判红、计成一条读不出的被抛弃根（{what}）"
    );
    assert_eq!(
        remounted.output.isolated_slots_per_device,
        vec![(DeviceIdentity(0), 40), (DeviceIdentity(1), 40)],
        "只被 C 引用的 14 个槽罩不到，与树表撕裂那一条同一个数"
    );
}
