//! 两档共用场景；池级 checker 镜像交给 checker 档给定状态引擎两段判。
//! K-U5 的 β 时序级读法只认这一版父链上的 inode 1 墓碑（D8（核心索引结构） 已定项 16 射程；代码轮判决
//! `research/prompts/m3-close-w5-code-r1-main-verification.md` 第三节第 1 条）：墓碑随现行那一版、管理员回退不取回退目标的，
//! 删第一个文件、回退到删之前、再删一次、再回退之后墓碑树里 inode 1 有两条，管着这一版的是父链上的那一条，不是最晚的那一条。
//! 历史取自攻方腿（`research/prompts/m3-close-w5-code-r1-opus-model/attack_histories_of_multi_file_writes_and_unlinks.rs` 的
//! `beta_after_two_rollbacks_lets_a_write_to_an_inode_born_after_the_deletion_through` 与
//! `a_write_after_rolling_back_past_the_deletion_of_the_first_file_is_not_judged_by_beta`），这里照条款写期望。
//! 另一条钉住「inode 1 的记录还在就不判」那一句：第一个文件删除中（意图之后、队列没走空）时 β 不判，写入口落到「承诺类意图队列已走空」那道断言。
//! 都在同一个进程里直调发布入口（不经会话：会话先走空，删除中那一形走不到写入口）。

#![allow(dead_code, reason = "两档各只调用其中一部分场景与帮手")]

use crate::common_images_for_the_checker_tier::{PoolCheckerExpectation, PoolCheckerImages};
use std::panic::{catch_unwind, AssertUnwindSafe};

use crate::common::{build_pool, disk_snapshot, parameters, BuiltPool};
use crate::common_deletion_in_progress::{
    publish_new_inodes_and_advance_the_pool, publish_the_intent_of, roll_back_to,
    unlink_to_the_end, INSTANCE_OF_THE_PUBLISHES, WRITE_TIME_SECONDS_OF_THE_NEW_INODES,
};
use singlefs_core::address::InodeNumber;
use singlefs_core::transaction::{
    publish_sequential_write_of_an_inode, PoolWriter, PublishError, TransactionOutput,
    FIRST_INODE_NUMBER,
};
use singlefs_core::unit::data_unit_payload_capacity;
use singlefs_harness::memory_pool::MemoryPool;

/// 按 inode 号写那几次的写入时间：晚于建 inode 那几次。
const WRITE_TIME_SECONDS_OF_THE_WRITES_BY_INODE_NUMBER: u64 =
    WRITE_TIME_SECONDS_OF_THE_NEW_INODES + 600;

/// 定长的伪随机字节流（xorshift64，种子取奇数）：`data_units` 个数据单元差 5 个字节写满。
fn content_of_data_units(data_units: usize, seed: u64) -> Vec<u8> {
    let mut state = seed | 1;
    (0..data_units * data_unit_payload_capacity() - 5)
        .map(|_| {
            state ^= state << 13;
            state ^= state >> 7;
            state ^= state << 17;
            u8::try_from(state & 0xff).expect("低 8 位")
        })
        .collect()
}

/// 同一个进程里按 inode 号顺序写一次：做成就把池接到写出的那一版上，被拒原样交回（池不动）。
fn try_sequential_write_of(
    pool: &mut BuiltPool,
    inode: u64,
    content: &[u8],
) -> Result<TransactionOutput, PublishError> {
    let publish_parameters = parameters();
    let previous = pool.output.clone();
    let devices = pool.devices.as_mut().expect("镜像还开着");
    let mut writer = PoolWriter::new(&publish_parameters, devices.as_mut_slice());
    let written = publish_sequential_write_of_an_inode(
        &mut writer,
        &mut pool.allocator,
        &previous,
        InodeNumber(inode),
        content,
        WRITE_TIME_SECONDS_OF_THE_WRITES_BY_INODE_NUMBER,
        INSTANCE_OF_THE_PUBLISHES,
    )?;
    pool.output = written.clone();
    Ok(written)
}

/// 一版墓碑树里 inode 1 的墓碑条数。
fn tombstones_of_the_first_file(version: &TransactionOutput) -> usize {
    version
        .tombstone_containers
        .iter()
        .flat_map(|container| container.contents.records.iter())
        .filter(|tombstone| tombstone.object.0 == FIRST_INODE_NUMBER)
        .count()
}

/// 删 inode 1（第一次死亡 txg）→ 建 inode 3（出生代晚于它，写 3 被拒）→ 回退到删之前 → 再删 inode 1（第二次死亡 txg，晚于 3 的出生代）
/// → 回退到建 3 那一版：这一版里 1 不在、3 在，这一版的父链是「建 3 ← 第一次删 1 ← 删之前」，3 生在第一次删完之后，β 照这条父链拒写 3；
/// 第二次删 1 那一段被这一次回退抛下（落在谱系记录 (T_rb, T_old) 的开区间里），它的墓碑留在墓碑树里却不管这一版。
/// 拒在任何写之前：系统配置槽、根环、录制流步数都不变。
/// 判别力：β 改回「墓碑树里 inode 1 的墓碑取最大的死亡 txg」（`crates/mutations.tsv` 以「K-U5 β 父链：」起头那一行）时读到第二次删的死亡 txg，
/// 写 3 做成，这一条红。
pub fn after_two_rollbacks_beta_refuses_a_write_to_an_inode_born_after_the_deletion_on_the_parent_chain(
    images: &mut PoolCheckerImages,
) {
    let mut pool = build_pool("beta-after-two-rollbacks-refuses");
    publish_new_inodes_and_advance_the_pool(&mut pool, 1);
    try_sequential_write_of(&mut pool, 2, &content_of_data_units(2, 0x22))
        .expect("第一个文件活着时写 inode 2");
    let before_deleting_the_first_file = pool.output.clone();
    let first_death = unlink_to_the_end(&mut pool, FIRST_INODE_NUMBER)
        .root
        .checkpoint_txg;
    let version_with_inode_three = publish_new_inodes_and_advance_the_pool(&mut pool, 1);
    let birth_of_inode_three = version_with_inode_three
        .inode_record_of(InodeNumber(3))
        .expect("删完第一个文件之后建的 inode 3")
        .object_birth;
    assert!(
        birth_of_inode_three > first_death,
        "3 生在第一次删完 1 之后：出生 {birth_of_inode_three:?}，死亡 {first_death:?}"
    );
    let refused_before_the_rollbacks =
        try_sequential_write_of(&mut pool, 3, &content_of_data_units(1, 0x33))
            .expect_err("回退之前写 3：β 拒");
    assert!(
        matches!(
            refused_before_the_rollbacks,
            PublishError::DataWriteToAnInodeBornAfterTheFirstFileWasDeletedIsOutsideTheFirstVersion {
                inode: 3,
                death_txg_of_the_first_file,
                ..
            } if death_txg_of_the_first_file == first_death
        ),
        "回退之前写 3 拒成 β，死亡 txg 是第一次删的：{refused_before_the_rollbacks:?}"
    );
    roll_back_to(&mut pool, &before_deleting_the_first_file);
    let second_death = unlink_to_the_end(&mut pool, FIRST_INODE_NUMBER)
        .root
        .checkpoint_txg;
    assert!(
        second_death > birth_of_inode_three,
        "第二次删 1 晚于 3 的出生：死亡 {second_death:?}，出生 {birth_of_inode_three:?}"
    );
    roll_back_to(&mut pool, &version_with_inode_three);
    assert!(
        pool.output.inode_record.is_none()
            && pool
                .output
                .inode_record_of(InodeNumber(3))
                .is_some_and(|record| record.object_birth == birth_of_inode_three),
        "回退到建 3 那一版：1 不在、3 在"
    );
    assert_eq!(
        tombstones_of_the_first_file(&pool.output),
        2,
        "墓碑随现行那一版：两次删 1 各留一条"
    );
    assert_eq!(
        pool.output.death_txg_of_the_deleted_first_file(),
        Some(first_death),
        "父链上的是第一次删的那一条；第二次删的 {second_death:?} 落在回退抛下的那一段里"
    );
    let before_the_write = disk_snapshot(&pool.memory_pool(), &pool.stream);
    let refusal = try_sequential_write_of(&mut pool, 3, &content_of_data_units(1, 0x34))
        .expect_err("回退到建 3 那一版之后写 3：β 照父链拒");
    assert!(
        matches!(
            refusal,
            PublishError::DataWriteToAnInodeBornAfterTheFirstFileWasDeletedIsOutsideTheFirstVersion {
                inode: 3,
                object_birth,
                death_txg_of_the_first_file,
            } if object_birth == birth_of_inode_three && death_txg_of_the_first_file == first_death
        ),
        "拒成 β，点名 inode 3、它的出生代与父链上那一次死亡 txg：{refusal:?}"
    );
    assert_eq!(
        disk_snapshot(&pool.memory_pool(), &pool.stream),
        before_the_write,
        "拒在任何写之前：系统配置槽、根环、录制流步数都不变"
    );
    hand_out_a_clean_image(images, &pool.memory_pool(), "两次回退之后的镜像");
}

/// 删 inode 1 → 回退到删之前（1 活着，那条墓碑随现行那一版留在墓碑树里）→ 建 inode（出生代晚于那条墓碑的死亡 txg）→ 写它：
/// 1 活着，这一版上没有删完第一个文件这件事，β 不判，照写；写出的镜像池级 checker 一条违例都没有。
pub fn a_write_after_rolling_back_past_the_deletion_of_the_first_file_is_not_judged_by_beta(
    images: &mut PoolCheckerImages,
) {
    let mut pool = build_pool("beta-not-judged-after-rolling-back-past-the-deletion");
    publish_new_inodes_and_advance_the_pool(&mut pool, 1);
    let before_deleting_the_first_file = pool.output.clone();
    let death = unlink_to_the_end(&mut pool, FIRST_INODE_NUMBER)
        .root
        .checkpoint_txg;
    roll_back_to(&mut pool, &before_deleting_the_first_file);
    assert!(
        pool.output.inode_record.is_some() && tombstones_of_the_first_file(&pool.output) == 1,
        "回退之后 1 活着，它那条墓碑还在墓碑树里"
    );
    let created = publish_new_inodes_and_advance_the_pool(&mut pool, 1);
    let birth = created
        .inode_record_of(InodeNumber(3))
        .expect("回退之后建的 inode 3")
        .object_birth;
    assert!(
        birth > death,
        "3 的出生代 {birth:?} 晚于墓碑的死亡 txg {death:?}"
    );
    let content = content_of_data_units(2, 0x3c);
    let written = try_sequential_write_of(&mut pool, 3, &content)
        .unwrap_or_else(|refusal| panic!("1 活着时写 3 照写：{refusal:?}"));
    assert_eq!(
        written
            .file_other_than_the_first(InodeNumber(3))
            .map(|file| file.data_pointers.len()),
        Some(2),
        "inode 3 写成两个数据单元"
    );
    hand_out_a_clean_image(images, &pool.memory_pool(), "写 3 之后");
}

/// 「inode 1 的记录还在就不判」那一句（K-U5 β，代码轮判决第三节第 1 条保留它）：第一个文件的意图发完（inode 1 nlink 0、队列 {1}、
/// 墓碑树一条，死亡 txg 在这一版父链上）之后建 inode 2（出生代晚于死亡 txg），不先走空就直调按 inode 号写 inode 2——1 那条记录还在（删除中），
/// β 不判，写入口落到「承诺类意图队列已走空」那道断言（`publish_sequential_write_of_an_inode` 文档的 Panics 一段：调用方先走空）。
/// 判别力：去掉那一句（`crates/mutations.tsv` 以「K-U5 β 父链：」起头、说删除中那一行）时 β 读到父链上那条墓碑、拒成 β，不落到那道断言，这一条红。
pub fn while_the_first_file_is_being_deleted_beta_does_not_judge_and_the_write_reaches_the_empty_queue_assertion(
) {
    let mut pool = build_pool("beta-not-judged-while-the-first-file-is-being-deleted");
    let intent = publish_the_intent_of(&mut pool, FIRST_INODE_NUMBER);
    assert!(
        intent.inode_record.is_some() && intent.deleted_inodes == vec![InodeNumber(1)],
        "意图之后 inode 1 那条记录还在（删除中）、队列 {{1}}"
    );
    let created = publish_new_inodes_and_advance_the_pool(&mut pool, 1);
    let birth = created
        .inode_record_of(InodeNumber(2))
        .expect("意图之后建的 inode 2")
        .object_birth;
    assert!(
        birth > intent.root.checkpoint_txg,
        "2 的出生代晚于 1 的死亡 txg"
    );
    let content = content_of_data_units(1, 0x2b);
    let outcome = catch_unwind(AssertUnwindSafe(|| {
        try_sequential_write_of(&mut pool, 2, &content)
    }));
    let panic_payload = match outcome {
        Err(payload) => payload,
        Ok(returned) => {
            panic!("删除中时 β 不判、写入口该落到队列已走空那道断言，实际交回 {returned:?}")
        }
    };
    let message = panic_payload
        .downcast_ref::<String>()
        .cloned()
        .or_else(|| {
            panic_payload
                .downcast_ref::<&str>()
                .map(|text| (*text).to_string())
        })
        .unwrap_or_default();
    assert!(
        message.contains("按 inode 号写之前承诺类意图队列已走空"),
        "落到的是队列已走空那道断言：{message}"
    );
}

fn hand_out_a_clean_image(images: &mut PoolCheckerImages, image: &MemoryPool, what: &str) {
    let label = format!("{}-{what}", images.handed_out());
    images.hand_out(
        &label,
        || image.clone(),
        PoolCheckerExpectation::no_invariant_violated(),
    );
}
