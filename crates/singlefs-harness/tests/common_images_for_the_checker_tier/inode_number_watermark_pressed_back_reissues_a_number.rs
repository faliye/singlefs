//! harness 测试文件 `crates/singlefs-harness/tests/inode_number_watermark_pressed_back_reissues_a_number.rs` 的场景函数：搭场景、核实现的行为，池级 checker 要判的镜像经 [`PoolCheckerImages`] 交出去
//! （做法见 `common_images_for_the_checker_tier/mod.rs`）。harness 档那份测试文件与 checker 档的
//! `crates/singlefs-checker-tier/tests/crash_given_states_of_the_images_harness_scenarios_hand_out.rs` 各经 `#[path]` 引这一份。
#![allow(dead_code, reason = "两档各只调其中一部分")]

use crate::common::{parameters, BuiltPool};
use crate::common_deletion_in_progress::{
    pool_with_new_inodes, INSTANCE_OF_THE_PUBLISHES, WRITE_TIME_SECONDS_OF_THE_NEW_INODES,
};
use crate::common_images_for_the_checker_tier::{PoolCheckerExpectation, PoolCheckerImages};
use singlefs_checker::image::{
    INODE_NUMBER_WATERMARK_STEPS_AGREE_WITH_THE_WATERMARK_THE_RECORDS_AND_THE_LINEAGE,
    ONE_OBJECT_BIRTH_PER_INODE_NUMBER,
};
use singlefs_core::address::{CheckpointTxg, InodeNumber};
use singlefs_core::mount::{
    raise_rollback_floor_to_the_admission_ceiling, RaiseToTheAdmissionCeiling, ShadowLedger,
};
use singlefs_core::transaction::{
    publish_new_inodes, publish_overwrite, publish_unlink_intent, publish_unlink_step, FirstFile,
    InodeNumberWatermarkPressedBackForTests, InodeNumberWatermarkSource, PoolWriter,
    TransactionOutput,
};
use singlefs_core::unlink::UnlinkBatchSize;
use singlefs_harness::memory_pool::MemoryPool;

/// 接着现行那一版建一个 inode，写入口上装着 `source` 这个水位来源；交回这一版与它的镜像。
pub fn create_one_inode_with_the_watermark_from(
    pool: &mut BuiltPool,
    source: InodeNumberWatermarkSource,
) -> (TransactionOutput, MemoryPool) {
    let publish_parameters = parameters();
    let previous = pool.output.clone();
    let devices = pool.devices.as_mut().expect("镜像还开着");
    let mut writer = PoolWriter::new(&publish_parameters, devices.as_mut_slice());
    writer.set_inode_number_watermark_source(source);
    assert_eq!(
        writer.inode_number_watermark_source().branch_name(),
        source.branch_name(),
        "运行时看得出走的是哪一臂"
    );
    let output = publish_new_inodes(
        &mut writer,
        &mut pool.allocator,
        &previous,
        1,
        WRITE_TIME_SECONDS_OF_THE_NEW_INODES,
        INSTANCE_OF_THE_PUBLISHES,
    )
    .expect("建一个 inode");
    pool.output = output.clone();
    (output, pool.memory_pool())
}

pub fn pressed_back_to(next_inode_number: u64) -> InodeNumberWatermarkSource {
    InodeNumberWatermarkSource::PressedBackForTests(
        InodeNumberWatermarkPressedBackForTests::new(next_inode_number)
            .expect("不小于第一个 inode 号"),
    )
}

/// 这一版 inode 树里 `inode` 那条记录的对象出生代；没有这条记录交回 `None`。
pub fn object_birth_of(output: &TransactionOutput, inode: u64) -> Option<u64> {
    output
        .inode_leaf_container_contents()
        .iter()
        .flat_map(|container| container.records.iter())
        .find(|record| record.inode == inode)
        .map(|record| record.object_birth.0)
}

/// 新池新建文件之后建 inode 2、3、4（水位 5），再建一个 inode：开关把水位压回 3 时新建的那个拿到 3 号——
/// inode 树里原来那条 3 号记录被原地换成新对象（对象出生代换成这次发布的 txg），记账里的水位写成 4，不大于树里最大的 key 4，
/// I-9.6 判红；回退候选集里更早那一版还留着 3 号的旧对象，I-9.29（同号只许一个出生代） 也红；建 2、3、4 那次留的台阶 (4, 5) 高过水位 4（①）、
/// 3 号的新对象晚于它出生（②），I-9.30 也红。
/// 同一段流开关关着时新建的拿 5 号、水位 6，I-9.6 成立、一条违例都没有。
pub fn creating_an_inode_after_the_watermark_is_pressed_back_below_the_largest_key_reddens_the_watermark_invariant(
    images: &mut PoolCheckerImages,
) {
    let mut pressed_pool = pool_with_new_inodes("watermark-pressed-back-below-the-largest-key", 3);
    assert_eq!(
        pressed_pool.output.inode_number_watermark(),
        5,
        "inode 1 到 4 发过，水位 5"
    );
    let (pressed, pressed_image) =
        create_one_inode_with_the_watermark_from(&mut pressed_pool, pressed_back_to(3));
    assert_eq!(
        pressed.inode_number_watermark(),
        4,
        "从压回去的 3 起发一个，水位写成 4"
    );
    assert_eq!(
        object_birth_of(&pressed, 3),
        Some(pressed.root.checkpoint_txg.0),
        "3 号被重发：树里那条记录换成这次发布建的新对象"
    );
    // 红 I-9.6、I-9.29 与 I-9.30；I-9.6 的违例说明写出树内最大 key 那一半，I-9.29 的说明点名被重发的那个号
    images.hand_out(
        "水位压回 3 再建一个 inode 之后",
        || pressed_image,
        PoolCheckerExpectation::exactly_these_violated(&[
            "I-9.6",
            ONE_OBJECT_BIRTH_PER_INODE_NUMBER,
            INODE_NUMBER_WATERMARK_STEPS_AGREE_WITH_THE_WATERMARK_THE_RECORDS_AND_THE_LINEAGE,
        ])
        .and_violated_with_a_detail_containing("I-9.6", "水位 4 不大于 inode 树内最大 key 4")
        .and_violated_with_a_detail_containing(
            ONE_OBJECT_BIRTH_PER_INODE_NUMBER,
            "inode 3 在回退候选集里对应不止一个对象出生代",
        ),
    );

    let mut control_pool = pool_with_new_inodes("watermark-not-pressed-back-control", 3);
    let (control, control_image) = create_one_inode_with_the_watermark_from(
        &mut control_pool,
        InodeNumberWatermarkSource::FromThePreviousVersionsAccounting,
    );
    assert_eq!(
        control.inode_number_watermark(),
        6,
        "开关关着：发 5 号，水位 6"
    );
    // 开关关着的同一段流 I-9.6 成立、一条违例都没有
    images.hand_out(
        "开关关着、同一段流再建一个 inode 之后",
        || control_image,
        PoolCheckerExpectation::no_invariant_violated().and_holds("I-9.6"),
    );
}

/// 在这个进程里把 `inode` unlink 到底：发意图，再按产品批大小一批一批发到删除队列空（它的记录删掉，墓碑留在墓碑树里）。
pub fn unlink_to_the_end_in_process(pool: &mut BuiltPool, inode: InodeNumber) {
    let publish_parameters = parameters();
    {
        let previous = pool.output.clone();
        let devices = pool.devices.as_mut().expect("镜像还开着");
        let mut writer = PoolWriter::new(&publish_parameters, devices.as_mut_slice());
        pool.output = publish_unlink_intent(
            &mut writer,
            &mut pool.allocator,
            &previous,
            inode,
            WRITE_TIME_SECONDS_OF_THE_NEW_INODES,
            INSTANCE_OF_THE_PUBLISHES,
        )
        .expect("意图");
    }
    let mut steps_published = 0;
    // 迭代上界：建出来的空 inode 没有数据单元，意图之后一批就删完；多走一步就是删不完。
    while !pool.output.deleted_inodes.is_empty() {
        assert!(steps_published < 1, "空 inode 意图之后一批就删完");
        let previous = pool.output.clone();
        let devices = pool.devices.as_mut().expect("镜像还开着");
        let mut writer = PoolWriter::new(&publish_parameters, devices.as_mut_slice());
        pool.output = publish_unlink_step(
            &mut writer,
            &mut pool.allocator,
            &previous,
            inode,
            UnlinkBatchSize::OneExtentLowerLeaf,
            INSTANCE_OF_THE_PUBLISHES,
        )
        .expect("删除的下一步");
        steps_published += 1;
    }
}

/// 在这个进程里把第一个文件覆盖写 `times` 次（每次一份不同的 3000 字节、写入时间各加一秒）：每次换一版 inode 记录与 extent 树，
/// 根都是「非空」的（抬 F 的上限按第 4 新的非空有效根取）。
pub fn overwrite_the_first_file_in_process(pool: &mut BuiltPool, times: u64) {
    let publish_parameters = parameters();
    // 迭代上界是 `times`。
    for overwrite_index in 0..times {
        let content: Vec<u8> = (0..3000_u64)
            .map(|index| {
                u8::try_from((index * 17 + overwrite_index * 5 + 3) % 251).expect("小于 256")
            })
            .collect();
        let previous = pool.output.clone();
        let devices = pool.devices.as_mut().expect("镜像还开着");
        let mut writer = PoolWriter::new(&publish_parameters, devices.as_mut_slice());
        pool.output = publish_overwrite(
            &mut writer,
            &mut pool.allocator,
            &previous,
            FirstFile {
                content: &content,
                write_time_seconds: WRITE_TIME_SECONDS_OF_THE_NEW_INODES + 1 + overwrite_index,
            },
            INSTANCE_OF_THE_PUBLISHES,
        )
        .unwrap_or_else(|refusal| panic!("第 {overwrite_index} 次覆盖写：{refusal:?}"));
    }
}

/// 删完那一版之后覆盖写几次：删除最后一批那条根之后再有三条非空的根，抬 F 的上限（第 4 新的非空有效根）才走到删除最后一批那条根。
pub const OVERWRITES_AFTER_THE_DELETION_SO_THE_CEILING_REACHES_ITS_LAST_ROOT: u64 = 3;

/// 新池新建文件之后建 inode 2、3（水位 4），把 3 号 unlink 到底（记录删掉、墓碑的对象 ID 是 3），覆盖写第一个文件 3 次，再把 F 抬到准入上限：
/// 上限走到删除最后一批那条根，还带着 3 号记录的那几条根（意图那一条与更早的）都低于 F，池级 checker 不再走它们。
/// 交回删除最后一批那条根的 txg。
pub fn pool_whose_inode_three_is_unlinked_and_whose_floor_is_raised_past_its_record(
    tag: &str,
) -> (BuiltPool, CheckpointTxg) {
    let mut pool = pool_with_new_inodes(tag, 2);
    assert_eq!(
        pool.output.inode_number_watermark(),
        4,
        "inode 1 到 3 发过，水位 4"
    );
    unlink_to_the_end_in_process(&mut pool, InodeNumber(3));
    assert_eq!(object_birth_of(&pool.output, 3), None, "3 号那条记录删掉了");
    let txg_of_the_last_deletion_step = pool.output.root.checkpoint_txg;
    overwrite_the_first_file_in_process(
        &mut pool,
        OVERWRITES_AFTER_THE_DELETION_SO_THE_CEILING_REACHES_ITS_LAST_ROOT,
    );
    let publish_parameters = parameters();
    let devices = pool.devices.as_mut().expect("镜像还开着");
    let raised = raise_rollback_floor_to_the_admission_ceiling(
        &publish_parameters,
        devices,
        &mut pool.allocator,
        &mut pool.output,
        ShadowLedger::Off,
    )
    .unwrap_or_else(|error| panic!("抬 F 到准入上限：{error:?}"));
    match raised {
        RaiseToTheAdmissionCeiling::Raised(_) => {}
        RaiseToTheAdmissionCeiling::FloorAlreadyAtTheCeiling { floor, ceiling } => {
            panic!("删完之后覆盖写三次，上限高于 F：F {floor:?}、上限 {ceiling:?}")
        }
    }
    assert!(
        pool.output.root.rollback_floor >= txg_of_the_last_deletion_step,
        "F 抬到 {:?}，不低于删除最后一批那条根的 txg {txg_of_the_last_deletion_step:?}：还带着 3 号记录的根都低于 F",
        pool.output.root.rollback_floor
    );
    (pool, txg_of_the_last_deletion_step)
}

/// C479 变体 c（第 19 批 S 的探针那一形，判决 `research/prompts/m3-batch19-code-r1-main-verification.md` 第二节「N7 附带」那一行）：
/// 建 inode 2、3，把 3 号 unlink 到底（墓碑的对象 ID 是 3），覆盖写第一个文件 3 次、把 F 抬到删除最后一批那条根，再把水位压回 2（低于墓碑最大对象号 3）
/// 建一个 inode：2 号被重发（树里那条记录原地换成新对象），记账的水位写成 3。池级 checker 走到的根里 inode 树最大 key 是 2，水位 3 大于它，
/// 那一半成立；所选根墓碑树里已发布墓碑的最大对象 ID 是 3，水位不大于它，I-9.6 判红，说明写的是墓碑那一半。同一段流开关关着时
/// 新建的拿 4 号、水位 5，I-9.6 成立、一条违例都没有。
/// 不抬 F 时池级 checker 还走着意图那一条根（3 号记录 nlink = 0 还在树里），树内最大 key 那一半先判出 3，说明写的是树那一半（S 的探针量过），
/// 判不出墓碑那一半拦不拦得住。
/// 判别力：墓碑那一半不比水位（`walk.rs` 里那一判恒真）时 I-9.6 成立，这一条红；关开关那一半由红转绿是判别力自证。
/// 2 号被重发时回退候选集里抬 F 之后那几版（覆盖写三次的）还留着 2 号的旧对象，I-9.29（同号只许一个出生代） 也红；
/// 建 2、3 号留的台阶高过压回去的水位、2 号的新对象晚于它出生，I-9.30 也红。
pub fn creating_an_inode_after_the_watermark_is_pressed_back_below_the_largest_tombstone_reddens_the_tombstone_half_of_the_watermark_invariant(
    images: &mut PoolCheckerImages,
) {
    let (mut pressed_pool, _) =
        pool_whose_inode_three_is_unlinked_and_whose_floor_is_raised_past_its_record(
            "watermark-pressed-back-below-the-largest-tombstone",
        );
    let (pressed, pressed_image) =
        create_one_inode_with_the_watermark_from(&mut pressed_pool, pressed_back_to(2));
    assert_eq!(
        pressed.inode_number_watermark(),
        3,
        "从压回去的 2 起发一个，水位写成 3"
    );
    assert_eq!(
        object_birth_of(&pressed, 2),
        Some(pressed.root.checkpoint_txg.0),
        "2 号被重发：树里那条记录换成这次发布建的新对象"
    );
    // 红 I-9.6、I-9.29 与 I-9.30；I-9.6 的违例说明写的是墓碑那一半（删除最后一批那条根之后走到的根里 inode 树最大 key 是 2，树那一半成立）
    images.hand_out(
        "水位压回 2（低于墓碑最大对象号 3）再建一个 inode 之后",
        || pressed_image,
        PoolCheckerExpectation::exactly_these_violated(&[
            "I-9.6",
            ONE_OBJECT_BIRTH_PER_INODE_NUMBER,
            INODE_NUMBER_WATERMARK_STEPS_AGREE_WITH_THE_WATERMARK_THE_RECORDS_AND_THE_LINEAGE,
        ])
        .and_if_violated_the_detail_satisfies(
            "I-9.6",
            "说明写的是墓碑那一半：以「记账里的 inode 号水位 3 不大于所选根」起头、以「的墓碑树里已发布的墓碑记录的最大对象 ID 3」收尾",
            |detail| {
                detail.starts_with("记账里的 inode 号水位 3 不大于所选根")
                    && detail.ends_with("的墓碑树里已发布的墓碑记录的最大对象 ID 3")
            },
        ),
    );

    let (mut control_pool, _) =
        pool_whose_inode_three_is_unlinked_and_whose_floor_is_raised_past_its_record(
            "watermark-not-pressed-back-below-the-largest-tombstone-control",
        );
    let (control, control_image) = create_one_inode_with_the_watermark_from(
        &mut control_pool,
        InodeNumberWatermarkSource::FromThePreviousVersionsAccounting,
    );
    assert_eq!(
        control.inode_number_watermark(),
        5,
        "开关关着：发 4 号，水位 5"
    );
    // 开关关着的同一段流 I-9.6 成立、一条违例都没有
    images.hand_out(
        "开关关着、同一段流再建一个 inode 之后",
        || control_image,
        PoolCheckerExpectation::no_invariant_violated().and_holds("I-9.6"),
    );
}

/// C479 变体 a（第 19 批 S 的探针：`/tmp/claude-1000/m3-batch19/s/probe/run2-probe.log` 那一形，I-9.6 拦不住的「压回的正是发过的最大号」）：
/// 新池新建文件之后建 inode 2、3、4（水位 5），开关把水位压回 4 再建一个——4 号那条活记录被原地换成新对象（对象出生代换成这次发布的 txg），
/// 记账的水位写成 5，仍大于树里最大的 key 4 与墓碑（没有）：I-9.6 成立。回退候选集里建 2、3、4 那一版还留着 4 号的旧对象，
/// I-9.29（同号只许一个出生代） 判红；建 2、3、4 那次留的台阶 (4, 5) 说 4 号在 txg 4 之前就发过，4 号的新对象晚于它出生，
/// I-9.30 ② 也判红（水位 5 不低于任何一级台阶，① 成立）——I-9.6 拦不住的这一形 I-9.30 单凭最新根就判得出。别的一条不红。
pub fn pressing_the_watermark_back_to_the_largest_issued_number_reissues_it_and_reddens_the_one_birth_per_number_and_the_watermark_step_invariants(
    images: &mut PoolCheckerImages,
) {
    let mut pressed_pool =
        pool_with_new_inodes("watermark-pressed-back-to-the-largest-issued-number", 3);
    let object_birth_before = object_birth_of(&pressed_pool.output, 4).expect("4 号活着");
    let (pressed, pressed_image) =
        create_one_inode_with_the_watermark_from(&mut pressed_pool, pressed_back_to(4));
    assert_eq!(
        pressed.inode_number_watermark(),
        5,
        "从压回去的 4 起发一个，水位写成 5"
    );
    assert_eq!(
        object_birth_of(&pressed, 4),
        Some(pressed.root.checkpoint_txg.0),
        "4 号被重发：树里那条记录换成这次发布建的新对象"
    );
    assert_ne!(
        Some(object_birth_before),
        object_birth_of(&pressed, 4),
        "重发前后 4 号的对象出生代不同"
    );
    // 水位 5 仍大于树里最大的 key 4：I-9.6 拦不住这一形，成立；红 I-9.29 与 I-9.30。I-9.30 的说明点名重发的 4 号与它晚于的那一级台阶 (4, 5)，
    // I-9.29 的说明点名被重发的那个号与它的旧出生代
    images.hand_out(
        "水位压回 4（发过的最大号）再建一个 inode 之后",
        || pressed_image,
        PoolCheckerExpectation::exactly_these_violated(&[
            ONE_OBJECT_BIRTH_PER_INODE_NUMBER,
            INODE_NUMBER_WATERMARK_STEPS_AGREE_WITH_THE_WATERMARK_THE_RECORDS_AND_THE_LINEAGE,
        ])
        .and_holds("I-9.6")
        .and_if_violated_the_detail_satisfies(
            INODE_NUMBER_WATERMARK_STEPS_AGREE_WITH_THE_WATERMARK_THE_RECORDS_AND_THE_LINEAGE,
            "说明点名重发的 4 号与它晚于的那一级台阶 (4, 5)",
            |detail| detail.contains("inode 树里 4 号的对象出生代") && detail.contains("(4, 5)"),
        )
        .and_if_violated_the_detail_satisfies_given_a_value(
            ONE_OBJECT_BIRTH_PER_INODE_NUMBER,
            "说明点名被重发的 4 号与它的旧出生代",
            object_birth_before,
            |detail, birth_before_the_reissue| {
                detail.contains("inode 4 在回退候选集里对应不止一个对象出生代")
                    && detail.contains(&format!("对象出生代 {birth_before_the_reissue}"))
            },
        ),
    );
}
