//! 第一个文件之外还有文件带数据单元时的删除各步（判决 `research/prompts/m3-multi-file-unlink-r1-main-verification.md` 第三节 M1、M2、M3，
//! 用户 2026-10-19 定的 K-U5 β 时序级读法）：删任一号、删第一个文件而别的文件还在、删到有条目的号集合空，与 extent 上段的规范形
//! （形状只由有条目的 inode 号集合定，写与删共用一个计划）。
//!
//! 每条用例一个场景：攻方量过的两形（只撤拒绝的孤儿 `research/prompts/m3-multi-file-unlink-r1-opus-model/out/opus-lift.txt`、批截第一个文件
//! `opus-lift_count.txt`）改完之后都做成；上段长到层 1 再删第一个文件（攻方 E 段，此前断言 panic）做成、降层；正推模型规则 A 的三种终态
//! （`research/prompts/m3-multi-file-unlink-r1-sonnet-model/out/run.out` 第 11–16 行）在实现上各一条；同一个有条目的号集合经两段历史得到
//! 同一个上段；删之前就有的别的文件删完第一个文件之后照常能写（β）。每一版都交池级 checker 判（连同新立的 I-9.40），一条违例都不许有。
mod common;
mod common_pointer_head_corruption;

use std::collections::BTreeSet;

use common::{build_pool, file_content, parameters, BuiltPool, Recorded, FIXED_WRITE_TIME_SECONDS};
use common_pointer_head_corruption::{replace_the_unit_and_propagate_its_checksum, the_slot_of};
use singlefs_checker::image::{
    InvariantVerdict, EXTENT_UPPER_SEGMENT_IS_THE_CANONICAL_SHAPE_OF_THE_INODES_WITH_AN_ENTRY,
};
use singlefs_checker::walk::check_pool_image;
use singlefs_core::address::{DeviceIdentity, FileOffsetInBytes, InodeNumber, InstanceGeneration};
use singlefs_core::extent_tree::{
    build_upper_internal_node, ExtentTreeNodeIdentity, ExtentUpperNodePosition,
};
use singlefs_core::mount::mount_writable;
use singlefs_core::mounted_read::mount_read_only;
use singlefs_core::mounted_session::MountedSession;
use singlefs_core::transaction::{
    publish_new_inodes, publish_sequential_write_of_an_inode, publish_unlink_intent,
    publish_unlink_step, PoolVersion, PoolWriter, TransactionOutput, FIRST_INODE_NUMBER,
};
use singlefs_core::unit::data_unit_payload_capacity;
use singlefs_core::unlink::{UnlinkBatchDataUnitsCappedForTests, UnlinkBatchSize};
use singlefs_harness::memory_pool::MemoryPool;

/// 这个进程（新池新建文件与它之后在同一个进程里的发布）的实例代号。
const INSTANCE_OF_THE_PROCESS: InstanceGeneration = InstanceGeneration(1);
/// 建 inode、按 inode 号写、unlink 那几版的写入时间：比池里第一个文件晚。
const WRITE_TIME_SECONDS_AFTER_THE_FIRST_FILE: u64 = FIXED_WRITE_TIME_SECONDS + 600;
/// 一次建 inode 至多建这么多个（一片 inode 叶容器装 233 条，一次跨不过一次末尾分裂）。
const INODES_CREATED_PER_PUBLISH_AT_MOST: u64 = 200;
/// 正推模型那组终态里第一个文件之外那个号（产品几何：上段叶罩 143 个号，500 落在第 3 片叶、要层级 1）。
const INODE_ON_THE_FOURTH_UPPER_LEAF: u64 = 500;
/// 攻方 E 段那个号：150 落在上段第 1 片叶，写它上段长到层 1。
const INODE_ON_THE_SECOND_UPPER_LEAF: u64 = 150;
/// 删除的各步至多走这么多步（三个单元、批大小 1 的文件意图一次加三批）：走不到最后一批说明计划出错，不无限循环。
const DELETION_STEPS_AT_MOST: usize = 16;

fn upper(level: u8, index: u64) -> ExtentUpperNodePosition {
    ExtentUpperNodePosition { level, index }
}

/// 定长的伪随机字节流（xorshift64，固定种子）：错位一个字节就对不上。
fn content_of(length: usize, seed: u64) -> Vec<u8> {
    let mut state = seed;
    (0..length)
        .map(|_| {
            state ^= state << 13;
            state ^= state >> 7;
            state ^= state << 17;
            u8::try_from(state & 0xff).expect("低 8 位")
        })
        .collect()
}

/// `data_units` 个数据单元的内容（最后一个单元差几个字节不满）。
fn content_of_data_units(data_units: usize, seed: u64) -> Vec<u8> {
    content_of(data_units * data_unit_payload_capacity() - 7, seed)
}

fn batches_of_one_data_unit() -> UnlinkBatchSize {
    UnlinkBatchSize::CappedForTests(
        UnlinkBatchDataUnitsCappedForTests::new(1).expect("1 在 1..=144 里"),
    )
}

/// 同一个进程里建 `count` 个 size 0 的 inode（号从水位起连着发），每次至多 [`INODES_CREATED_PER_PUBLISH_AT_MOST`] 个。
fn create_inodes_in_process(pool: &mut BuiltPool, count: u64) {
    let publish_parameters = parameters();
    let mut left = count;
    // 迭代上界：每一轮至少建一个，⌈count ÷ 200⌉ 轮建完。
    while left > 0 {
        let this_time = left.min(INODES_CREATED_PER_PUBLISH_AT_MOST);
        let previous = pool.output.clone();
        let devices = pool.devices.as_mut().expect("镜像还开着");
        let mut writer = PoolWriter::new(&publish_parameters, devices.as_mut_slice());
        pool.output = publish_new_inodes(
            &mut writer,
            &mut pool.allocator,
            &previous,
            this_time,
            WRITE_TIME_SECONDS_AFTER_THE_FIRST_FILE,
            INSTANCE_OF_THE_PROCESS,
        )
        .expect("建 inode");
        left -= this_time;
    }
}

/// 同一个进程里按 inode 号顺序写 `inode` 整份成 `content`（`inode` 是 1 时就是写第一个文件）。
fn write_in_process(pool: &mut BuiltPool, inode: u64, content: &[u8]) {
    let publish_parameters = parameters();
    let previous = pool.output.clone();
    let devices = pool.devices.as_mut().expect("镜像还开着");
    let mut writer = PoolWriter::new(&publish_parameters, devices.as_mut_slice());
    pool.output = publish_sequential_write_of_an_inode(
        &mut writer,
        &mut pool.allocator,
        &previous,
        InodeNumber(inode),
        content,
        WRITE_TIME_SECONDS_AFTER_THE_FIRST_FILE,
        INSTANCE_OF_THE_PROCESS,
    )
    .unwrap_or_else(|refusal| panic!("按 inode 号写 inode {inode}：{refusal:?}"));
}

/// 同一个进程里删 `inode`：意图那次，再按 `batch_size` 一批批走到最后一批。交回每一版（意图那一版在最前），每发一版都交池级 checker 判、
/// 一条违例都不许有（判决第三节 M1「每条删除路径都过池级 checker」）。
fn unlink_in_process(
    pool: &mut BuiltPool,
    inode: u64,
    batch_size: UnlinkBatchSize,
) -> Vec<TransactionOutput> {
    let publish_parameters = parameters();
    let mut versions = Vec::new();
    {
        let previous = pool.output.clone();
        let devices = pool.devices.as_mut().expect("镜像还开着");
        let mut writer = PoolWriter::new(&publish_parameters, devices.as_mut_slice());
        pool.output = publish_unlink_intent(
            &mut writer,
            &mut pool.allocator,
            &previous,
            InodeNumber(inode),
            WRITE_TIME_SECONDS_AFTER_THE_FIRST_FILE,
            INSTANCE_OF_THE_PROCESS,
        )
        .unwrap_or_else(|refusal| panic!("unlink inode {inode} 的意图：{refusal:?}"));
    }
    assert_no_violation(
        &pool.memory_pool(),
        &format!("unlink inode {inode} 的意图那一版"),
    );
    versions.push(pool.output.clone());
    // 迭代上界：每一批至少释放一个单元，[`DELETION_STEPS_AT_MOST`] 步之内走到最后一批（下面断言）。
    while pool.output.deleted_inodes.contains(&InodeNumber(inode)) {
        assert!(
            versions.len() <= DELETION_STEPS_AT_MOST,
            "unlink inode {inode} 走了 {} 步还没到最后一批",
            versions.len()
        );
        let previous = pool.output.clone();
        let devices = pool.devices.as_mut().expect("镜像还开着");
        let mut writer = PoolWriter::new(&publish_parameters, devices.as_mut_slice());
        pool.output = publish_unlink_step(
            &mut writer,
            &mut pool.allocator,
            &previous,
            InodeNumber(inode),
            batch_size,
            INSTANCE_OF_THE_PROCESS,
        )
        .unwrap_or_else(|refusal| panic!("unlink inode {inode} 的下一步：{refusal:?}"));
        assert_no_violation(
            &pool.memory_pool(),
            &format!("unlink inode {inode} 的第 {} 步之后", versions.len()),
        );
        versions.push(pool.output.clone());
    }
    versions
}

/// 池级 checker 对这张镜像一条违例都没有，I-9.40（上段规范形）真被判过且成立。
fn assert_no_violation(image: &MemoryPool, what: &str) {
    let verdicts = check_pool_image(image);
    let violations: Vec<(&str, String)> = verdicts
        .iter()
        .filter_map(|(invariant, verdict)| match verdict {
            InvariantVerdict::Violated(detail) => Some((*invariant, detail.clone())),
            InvariantVerdict::Holds
            | InvariantVerdict::NotApplicable(_)
            | InvariantVerdict::PartiallyJudged(_) => None,
        })
        .collect();
    assert!(
        violations.is_empty(),
        "{what}：池级 checker 判红 {violations:?}"
    );
    let canonical_shape = verdicts
        .iter()
        .find(|(invariant, _)| {
            *invariant == EXTENT_UPPER_SEGMENT_IS_THE_CANONICAL_SHAPE_OF_THE_INODES_WITH_AN_ENTRY
        })
        .map(|(_, verdict)| verdict.clone());
    assert_eq!(
        canonical_shape,
        Some(InvariantVerdict::Holds),
        "{what}：I-9.40（上段规范形）真被判过且成立"
    );
}

/// 只读挂载这张镜像、打开 `inode`、从偏移 0 读整个文件。
fn content_read_back(image: &MemoryPool, inode: u64) -> Vec<u8> {
    let mounted = mount_read_only(image).expect("只读挂载");
    let file = mounted
        .mounted
        .open_file(image, InodeNumber(inode))
        .unwrap_or_else(|failure| panic!("打开 inode {inode}：{failure:?}"));
    let size = file.inode_record().size;
    file.read_at(image, FileOffsetInBytes(0), size)
        .unwrap_or_else(|failure| panic!("读 inode {inode} 的 {size} 字节：{failure:?}"))
        .bytes
}

/// 一版 extent 树上段的节点位置，bump 次序。
fn upper_positions(version: &TransactionOutput) -> Vec<ExtentUpperNodePosition> {
    version
        .extent_tree
        .upper_nodes
        .iter()
        .map(|(position, _)| *position)
        .collect()
}

/// 关掉镜像（进程退出）、重开可写挂载，交回接着这次挂载的会话与它的盘。
fn session_after_a_writable_remount(
    pool: &mut BuiltPool,
) -> (MountedSession, Vec<(DeviceIdentity, Recorded)>) {
    let mut devices = pool.reopen_recorded();
    let mounted = mount_writable(&parameters(), &mut devices).expect("重开可写挂载");
    let (_, session) = MountedSession::of_the_mount(mounted);
    (session, devices)
}

/// 第一个文件之外 inode 2 带三个单元（第一个文件是新池新建文件那一份）。
fn pool_with_inode_two_holding_three_data_units(tag: &str) -> (BuiltPool, Vec<u8>) {
    let mut pool = build_pool(tag);
    create_inodes_in_process(&mut pool, 1);
    let content_of_inode_two = content_of_data_units(3, 0x2201);
    write_in_process(&mut pool, 2, &content_of_inode_two);
    (pool, content_of_inode_two)
}

/// 第一个文件与 inode 500 各带数据：有条目的号集合 {1, 500}，上段 (0, 0)、(0, 3)、(1, 0)，根层级 1。
fn pool_with_the_first_file_and_inode_five_hundred(tag: &str) -> (BuiltPool, Vec<u8>) {
    let mut pool = build_pool(tag);
    create_inodes_in_process(&mut pool, INODE_ON_THE_FOURTH_UPPER_LEAF - 1);
    let content_of_inode_five_hundred = content_of_data_units(2, 0x5001);
    write_in_process(
        &mut pool,
        INODE_ON_THE_FOURTH_UPPER_LEAF,
        &content_of_inode_five_hundred,
    );
    assert_eq!(
        upper_positions(&pool.output),
        vec![upper(0, 0), upper(0, 3), upper(1, 0)],
        "起点：S = {{1, 500}}"
    );
    (pool, content_of_inode_five_hundred)
}

/// 攻方 U1-甲 那一形（`opus-lift.txt`：只撤拒绝时最后一批删了记录、单元与上段条目留着，I-4.7、I-1.13 红、下次可写挂载拒）：
/// 第一个文件之外 inode 2 带三个单元时删它，按产品批大小意图之后就是最后一批——它的三个单元与下段整份换下、它的上段条目去掉
/// （不写标签 0），记录与队列 key 删掉；第一个文件逐字节不变，每一版池级 checker 全绿，重开可写挂载照常。
#[test]
fn unlinking_another_file_holding_three_data_units_releases_them_and_drops_its_upper_entry() {
    let (mut pool, _) =
        pool_with_inode_two_holding_three_data_units("unlink-of-inode-two-three-units");
    let versions = unlink_in_process(&mut pool, 2, UnlinkBatchSize::OneExtentLowerLeaf);
    let emptied = versions.last().expect("至少意图与最后一批");
    assert_eq!(
        versions.len(),
        2,
        "三个单元不多于一批：意图之后就是最后一批"
    );
    assert!(
        emptied.files_other_than_the_first.is_empty()
            && emptied.inode_record_of(InodeNumber(2)).is_none(),
        "inode 2 的单元、下段与记录都没了"
    );
    assert_eq!(
        upper_positions(emptied),
        vec![upper(0, 0)],
        "上段只剩第一个文件那片根兼叶"
    );
    assert_eq!(
        content_read_back(&pool.memory_pool(), FIRST_INODE_NUMBER),
        file_content()
    );
    let (session, _devices) = session_after_a_writable_remount(&mut pool);
    let remounted = session
        .current
        .file_version()
        .expect("重开之后是带文件的一版");
    assert!(
        remounted.files_other_than_the_first.is_empty() && remounted.deleted_inodes.is_empty(),
        "重开之后没有 inode 2 的单元、队列空"
    );
}

/// 攻方 U1-乙 那一形（`opus-lift_count.txt`：剩余单元数取 N 自己的而批仍截第一个文件，丢的单元被尾部标签洗成洞）：第一个文件三个单元、
/// inode 2 三个单元、批大小 1 删 inode 2——每一批只截 inode 2（剩 2、1 个），第一个文件每一批之后读回都与删之前逐字节相同，最后一批删掉 inode 2；
/// 每一版池级 checker 全绿（判决第三节 M4 取样点「删 N 的每一批之后第一个文件读回逐字节相同」）。
#[test]
fn each_deletion_batch_of_another_file_cuts_only_that_file_and_the_first_file_reads_back_unchanged()
{
    let mut pool = build_pool("unlink-of-inode-two-in-batches-of-one");
    let content_of_the_first_file = content_of_data_units(3, 0x0101);
    write_in_process(&mut pool, FIRST_INODE_NUMBER, &content_of_the_first_file);
    create_inodes_in_process(&mut pool, 1);
    write_in_process(&mut pool, 2, &content_of_data_units(3, 0x2202));
    let publish_parameters = parameters();
    let version_before_the_intent = pool.output.clone();
    {
        let devices = pool.devices.as_mut().expect("镜像还开着");
        let mut writer = PoolWriter::new(&publish_parameters, devices.as_mut_slice());
        pool.output = publish_unlink_intent(
            &mut writer,
            &mut pool.allocator,
            &version_before_the_intent,
            InodeNumber(2),
            WRITE_TIME_SECONDS_AFTER_THE_FIRST_FILE,
            INSTANCE_OF_THE_PROCESS,
        )
        .expect("unlink inode 2 的意图");
    }
    let mut data_units_of_inode_two_after_each_batch = Vec::new();
    // 迭代上界：三个单元、批大小 1，两批加最后一批。
    while pool.output.deleted_inodes.contains(&InodeNumber(2)) {
        assert!(data_units_of_inode_two_after_each_batch.len() < DELETION_STEPS_AT_MOST);
        let version_before_the_batch = pool.output.clone();
        {
            let devices = pool.devices.as_mut().expect("镜像还开着");
            let mut writer = PoolWriter::new(&publish_parameters, devices.as_mut_slice());
            pool.output = publish_unlink_step(
                &mut writer,
                &mut pool.allocator,
                &version_before_the_batch,
                InodeNumber(2),
                batches_of_one_data_unit(),
                INSTANCE_OF_THE_PROCESS,
            )
            .expect("unlink inode 2 的下一批");
        }
        let image = pool.memory_pool();
        assert_eq!(
            content_read_back(&image, FIRST_INODE_NUMBER),
            content_of_the_first_file,
            "删 inode 2 的第 {} 批之后第一个文件读回逐字节相同",
            data_units_of_inode_two_after_each_batch.len() + 1
        );
        assert_eq!(
            pool.output.data_pointers.len(),
            3,
            "第一个文件三个单元一个不少"
        );
        assert_no_violation(&image, "删 inode 2 的一批之后");
        data_units_of_inode_two_after_each_batch.push(
            pool.output
                .file_other_than_the_first(InodeNumber(2))
                .map_or(0, |file| file.data_pointers.len()),
        );
    }
    assert_eq!(
        data_units_of_inode_two_after_each_batch,
        vec![2, 1, 0],
        "每一批只截 inode 2 尾部一个单元，最后一批放掉剩下那一个"
    );
}

/// 攻方 E 段那一形（`opus-lift.txt` 第 31、35 行：上段长到层 1 再删第一个文件，`unlink.rs` 那条「上段只有根兼叶」断言 panic，意图落盘之后
/// 每次可写挂载续做都 panic）：写 inode 150 两个单元、上段长到 (0, 0)、(0, 1)、(1, 0)，再删第一个文件——做成，上段去掉 inode 1、
/// 第 0 片叶换下，剩 150 那片叶与根；这一版是带文件的一版（没有第一个文件那条记录，判决第三节 M3），inode 150 读得回，重开可写挂载照常。
#[test]
fn unlinking_the_first_file_after_the_upper_segment_grew_to_level_one_keeps_the_leaf_of_the_other_file(
) {
    let mut pool = build_pool("unlink-of-the-first-file-beside-inode-150");
    create_inodes_in_process(&mut pool, INODE_ON_THE_SECOND_UPPER_LEAF - 1);
    let content_of_inode_150 = content_of_data_units(2, 0x1501);
    write_in_process(
        &mut pool,
        INODE_ON_THE_SECOND_UPPER_LEAF,
        &content_of_inode_150,
    );
    assert_eq!(
        upper_positions(&pool.output),
        vec![upper(0, 0), upper(0, 1), upper(1, 0)],
        "写 inode 150 之后上段长到层 1"
    );
    let versions = unlink_in_process(
        &mut pool,
        FIRST_INODE_NUMBER,
        UnlinkBatchSize::OneExtentLowerLeaf,
    );
    let after = versions.last().expect("至少意图与最后一批");
    assert_eq!(
        upper_positions(after),
        vec![upper(0, 1), upper(1, 0)],
        "删完第一个文件：第 0 片叶换下，剩 150 那片叶与层级 1 的根"
    );
    assert!(
        after.inode_record.is_none() && after.data_pointers.is_empty(),
        "没有第一个文件那条记录、没有它的单元"
    );
    assert!(
        matches!(
            PoolVersion::of_a_version_of_trees(after.clone()),
            PoolVersion::WithFile(_)
        ),
        "删完第一个文件而别的文件还在：带文件的一版"
    );
    assert_eq!(
        content_read_back(&pool.memory_pool(), INODE_ON_THE_SECOND_UPPER_LEAF),
        content_of_inode_150
    );
    let (session, _devices) = session_after_a_writable_remount(&mut pool);
    assert!(
        matches!(session.current, PoolVersion::WithFile(_)),
        "重开可写挂载照常，是带文件的一版"
    );
}

/// 正推模型规则 A 的终态「只剩第一个文件」（`run.out` 第 15 行：删 500 → S = {1}，(0, 0) 照抄成根、层级 0，(0, 3) 与 (1, 0) 换下）：
/// 上段降层，第 0 片叶这一版是根；第一个文件逐字节不变，每一版池级 checker 全绿。
#[test]
fn deleting_inode_five_hundred_beside_the_first_file_lowers_the_upper_segment_to_the_first_leaf() {
    let (mut pool, _) = pool_with_the_first_file_and_inode_five_hundred(
        "unlink-of-inode-500-beside-the-first-file",
    );
    let leaf_of_the_first_file_before = pool
        .output
        .extent_tree
        .upper_pointer_of(upper(0, 0))
        .expect("第 0 片叶在");
    let versions = unlink_in_process(
        &mut pool,
        INODE_ON_THE_FOURTH_UPPER_LEAF,
        UnlinkBatchSize::OneExtentLowerLeaf,
    );
    let after = versions.last().expect("至少意图与最后一批");
    assert_eq!(
        upper_positions(after),
        vec![upper(0, 0)],
        "S = {{1}}：层级 0 的根兼叶"
    );
    assert_eq!(
        after.extent_tree.upper_pointer_of(upper(0, 0)),
        Some(leaf_of_the_first_file_before),
        "第 0 片叶不罩 500，照抄成根（指针不变）"
    );
    assert_eq!(
        content_read_back(&pool.memory_pool(), FIRST_INODE_NUMBER),
        file_content()
    );
}

/// 正推模型规则 A 的终态「只剩别的那一个文件」（`run.out` 第 12 行：删 1 → S = {500}，(1, 0) 重写、(0, 3) 照抄，(0, 0) 换下）：
/// 这一版是带文件的一版、没有第一个文件那条记录（判决第三节 M3），inode 500 逐字节读得回。
#[test]
fn deleting_the_first_file_beside_inode_five_hundred_leaves_its_leaf_under_a_level_one_root() {
    let (mut pool, content_of_inode_five_hundred) = pool_with_the_first_file_and_inode_five_hundred(
        "unlink-of-the-first-file-beside-inode-500",
    );
    let leaf_of_inode_five_hundred_before = pool
        .output
        .extent_tree
        .upper_pointer_of(upper(0, 3))
        .expect("第 3 片叶在");
    let versions = unlink_in_process(
        &mut pool,
        FIRST_INODE_NUMBER,
        UnlinkBatchSize::OneExtentLowerLeaf,
    );
    let after = versions.last().expect("至少意图与最后一批");
    assert_eq!(
        upper_positions(after),
        vec![upper(0, 3), upper(1, 0)],
        "S = {{500}}"
    );
    assert_eq!(
        after.extent_tree.upper_pointer_of(upper(0, 3)),
        Some(leaf_of_inode_five_hundred_before),
        "500 那片叶照抄（指针不变）"
    );
    assert!(
        matches!(
            PoolVersion::of_a_version_of_trees(after.clone()),
            PoolVersion::WithFile(_)
        ) && after.inode_record.is_none(),
        "删完第一个文件而别的文件还在：带文件的一版，没有第一个文件那条记录"
    );
    assert_eq!(
        content_read_back(&pool.memory_pool(), INODE_ON_THE_FOURTH_UPPER_LEAF),
        content_of_inode_five_hundred
    );
}

/// 正推模型规则 A 的终态「删空」与两种删序（`run.out` 第 13、16、17 行；判决第三节 M3「两种删序各一条用例，终态上段位置集合相同」）：
/// 先删 1 再删 500、先删 500 再删 1，两段各自走完、每一版池级 checker 全绿，终态都是层级 0 第 0 格一个零条目根兼叶（S 空），位置集合相同。
// harness-test-granularity:one-scenario 两种删序走到同一个终态是一个场景：比的是两段历史的终态，一段建不出这一比
#[test]
fn deleting_both_files_in_either_order_ends_in_one_zero_entry_upper_root() {
    let mut final_shapes = Vec::new();
    for (tag, order) in [
        (
            "unlink-of-both-files-the-first-file-first",
            [FIRST_INODE_NUMBER, INODE_ON_THE_FOURTH_UPPER_LEAF],
        ),
        (
            "unlink-of-both-files-inode-500-first",
            [INODE_ON_THE_FOURTH_UPPER_LEAF, FIRST_INODE_NUMBER],
        ),
    ] {
        let (mut pool, _) = pool_with_the_first_file_and_inode_five_hundred(tag);
        for inode in order {
            unlink_in_process(&mut pool, inode, UnlinkBatchSize::OneExtentLowerLeaf);
        }
        assert!(
            pool.output
                .inodes_with_an_extent_upper_leaf_entry()
                .is_empty()
                && pool.output.files_other_than_the_first.is_empty()
                && pool.output.data_pointers.is_empty(),
            "删序 {order:?}：有条目的号集合空"
        );
        final_shapes.push(upper_positions(&pool.output));
    }
    assert_eq!(
        final_shapes[0],
        vec![upper(0, 0)],
        "删空：层级 0 第 0 格一个零条目根兼叶"
    );
    assert_eq!(
        final_shapes[0], final_shapes[1],
        "两种删序终态上段位置集合相同"
    );
}

/// 同一个有条目的号集合经两段不同的历史得到同一个上段（判决第三节 M2：形状只由有条目的号集合定）：{1, 2} 直接写成，与先写 150（上段长到层 1）
/// 再删 150（降回层 0）——只升不降的读法下后一段留着层级 1 的根与第 0 片叶两个节点。
// harness-test-granularity:one-scenario 比的是两段历史的终态，一段建不出这一比
#[test]
fn one_set_of_files_with_data_reaches_one_upper_segment_through_two_histories() {
    let mut direct = build_pool("upper-shape-of-inodes-one-and-two-direct");
    create_inodes_in_process(&mut direct, INODE_ON_THE_SECOND_UPPER_LEAF - 1);
    write_in_process(&mut direct, 2, &content_of_data_units(1, 0x0201));
    let mut through_level_one = build_pool("upper-shape-of-inodes-one-and-two-through-level-one");
    create_inodes_in_process(&mut through_level_one, INODE_ON_THE_SECOND_UPPER_LEAF - 1);
    write_in_process(&mut through_level_one, 2, &content_of_data_units(1, 0x0202));
    write_in_process(
        &mut through_level_one,
        INODE_ON_THE_SECOND_UPPER_LEAF,
        &content_of_data_units(2, 0x1502),
    );
    assert_eq!(through_level_one.output.extent_tree.upper_root().level, 1);
    unlink_in_process(
        &mut through_level_one,
        INODE_ON_THE_SECOND_UPPER_LEAF,
        UnlinkBatchSize::OneExtentLowerLeaf,
    );
    assert_eq!(
        through_level_one
            .output
            .inodes_with_an_extent_upper_leaf_entry(),
        BTreeSet::from([FIRST_INODE_NUMBER, 2])
    );
    assert_eq!(upper_positions(&direct.output), vec![upper(0, 0)]);
    assert_eq!(
        upper_positions(&through_level_one.output),
        upper_positions(&direct.output),
        "同一个 S = {{1, 2}}，两段历史同一个上段"
    );
}

/// K-U5 的 β 时序级读法（用户 2026-10-19 定）照写的那一边（攻方 `opus-today.txt` 第 12 行那段历史）：先建 inode 2（size 0）、再把第一个文件
/// 删到底、再按 inode 号给 inode 2 写三个单元——inode 2 的对象出生代早于 inode 1 墓碑的死亡 txg，照写；读回逐字节相同、池级 checker 全绿。
/// 拒的那一边（删空再建 inode 2 再写）在 `writes_by_inode_number_of_several_files.rs`。
#[test]
fn a_file_created_before_the_first_file_was_deleted_is_still_written_after_it() {
    let mut pool = build_pool("write-of-an-inode-created-before-the-first-file-was-deleted");
    create_inodes_in_process(&mut pool, 1);
    let unlinked = unlink_in_process(
        &mut pool,
        FIRST_INODE_NUMBER,
        UnlinkBatchSize::OneExtentLowerLeaf,
    );
    let death_txg_of_the_first_file = unlinked[0].root.checkpoint_txg;
    assert_eq!(
        pool.output.death_txg_of_the_deleted_first_file(),
        Some(death_txg_of_the_first_file),
        "第一个文件删完：墓碑的死亡 txg 是意图那次"
    );
    assert!(
        pool.output
            .inode_record_of(InodeNumber(2))
            .is_some_and(|record| record.object_birth < death_txg_of_the_first_file),
        "inode 2 在删第一个文件之前建"
    );
    let content_of_inode_two = content_of_data_units(3, 0x2203);
    write_in_process(&mut pool, 2, &content_of_inode_two);
    let image = pool.memory_pool();
    assert_eq!(content_read_back(&image, 2), content_of_inode_two);
    assert_no_violation(&image, "删完第一个文件之后写原有的 inode 2");
}

/// 挂载续做对任一号同一条（判决第三节 M1、D8（核心索引结构） 已定项 16 第 6 条）：只发了 inode 2 的意图就关掉镜像，重开可写挂载把队列走到空——
/// inode 2 的单元、上段条目与记录都没了，第一个文件逐字节不变，续做走完的镜像池级 checker 全绿。
#[test]
fn a_writable_remount_after_the_intent_of_another_file_continues_its_deletion_to_the_end() {
    let (mut pool, _) =
        pool_with_inode_two_holding_three_data_units("unlink-of-inode-two-continued-by-the-mount");
    let publish_parameters = parameters();
    let previous = pool.output.clone();
    {
        let devices = pool.devices.as_mut().expect("镜像还开着");
        let mut writer = PoolWriter::new(&publish_parameters, devices.as_mut_slice());
        pool.output = publish_unlink_intent(
            &mut writer,
            &mut pool.allocator,
            &previous,
            InodeNumber(2),
            WRITE_TIME_SECONDS_AFTER_THE_FIRST_FILE,
            INSTANCE_OF_THE_PROCESS,
        )
        .expect("unlink inode 2 的意图");
    }
    let (session, devices) = session_after_a_writable_remount(&mut pool);
    let continued = session
        .current
        .file_version()
        .expect("续做走完之后是带文件的一版");
    assert!(
        continued.deleted_inodes.is_empty()
            && continued.files_other_than_the_first.is_empty()
            && continued.inode_record_of(InodeNumber(2)).is_none(),
        "续做把 inode 2 删到底"
    );
    assert_eq!(upper_positions(continued), vec![upper(0, 0)]);
    pool.devices = Some(devices);
    let image = pool.memory_pool();
    assert_eq!(
        content_read_back(&image, FIRST_INODE_NUMBER),
        file_content()
    );
    assert_no_violation(&image, "续做走完 inode 2 的删除之后");
}

/// I-9.40 的坏镜像（判决第三节 M2 必红世界「根层级高一层」那一形落在盘上）：{1, 150} 那一版上段是 (0, 0)、(0, 1)、(1, 0)；把层级 1 的根
/// 换成只指第 0 片叶的那一份（150 那片叶摘掉、重封、新的整单元校验和沿引用链补上去）。读到的有条目的号集合是 {1}，规范形是层级 0 的
/// 第 0 片叶一个，读到的根却在层级 1：I-9.40 红，说明里写出根层级与多出的 (1, 0)。150 那条记录与它的单元另红几条（I-4.6 这类），这里不钉它们。
#[test]
fn a_root_left_one_level_above_the_only_leaf_with_an_entry_reddens_the_upper_shape_invariant() {
    let mut pool = build_pool("upper-shape-root-one-level-too-high");
    create_inodes_in_process(&mut pool, INODE_ON_THE_SECOND_UPPER_LEAF - 1);
    write_in_process(
        &mut pool,
        INODE_ON_THE_SECOND_UPPER_LEAF,
        &content_of_data_units(1, 0x1503),
    );
    let root_pointer = pool
        .output
        .extent_tree
        .upper_pointer_of(upper(1, 0))
        .expect("上段根在层级 1");
    let leaf_of_the_first_file = pool
        .output
        .extent_tree
        .upper_pointer_of(upper(0, 0))
        .expect("第 0 片叶在");
    let filesystem_identifier = parameters().filesystem_identifier;
    let root_with_only_the_first_leaf = build_upper_internal_node(
        &ExtentTreeNodeIdentity {
            tree: pool.output.tree_identifiers.extent,
            birth_txg: root_pointer.head.birth_txg,
            filesystem_identifier: &filesystem_identifier,
            instance: root_pointer.instance,
        },
        upper(1, 0),
        &[(upper(0, 0), leaf_of_the_first_file)],
        root_pointer.birth_sequence,
    );
    let mut image = pool.memory_pool();
    replace_the_unit_and_propagate_its_checksum(
        &mut image,
        the_slot_of(&root_pointer),
        &root_with_only_the_first_leaf,
    );
    let verdict = check_pool_image(&image)
        .into_iter()
        .find(|(invariant, _)| {
            *invariant == EXTENT_UPPER_SEGMENT_IS_THE_CANONICAL_SHAPE_OF_THE_INODES_WITH_AN_ENTRY
        })
        .map(|(_, verdict)| verdict);
    let Some(InvariantVerdict::Violated(detail)) = verdict else {
        panic!("根留在层级 1、只挂第 0 片叶：I-9.40 该红，实际 {verdict:?}");
    };
    assert!(
        detail.contains("上段根层级 1") && detail.contains("(1, 0)"),
        "说明里写出读到的根层级与多出的节点：{detail}"
    );
}
