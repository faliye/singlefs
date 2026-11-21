//! 实例表行的高水位与首个代号、重新发出的 inode 号、转出根环的根发出的树号，以及整张坏镜像表在池级 checker 上判红。

use super::*;

// ─── 根轮出环之后压低的树 ID 水位（C147）、读得出孩子时子指针出生树与身份引用不同（C297）、旧版转出回退候选集之后的重发号（C479） ───

/// C147（树 ID 水位的不变量还没有） 判别力第 ② 格「根轮出环」（判决 `research/prompts/m3-inv-a-r1-main-verification.md` 第二节第 10 行，
/// 采攻方的造法）：新池新建文件那一次发布（txg 3）发出十三棵树、把水位推到发第一个文件之后那一档；同一个进程接着覆盖写一圈根环那么多次
/// （3 个区域各 8 槽，txg 4 到 27），发树那条根与它之前的根全被轮出根环，环里每条根都是覆盖写发的、一棵树都没新发。
/// 再把环里每条根的水位压到 mkfs 那一档（[`set_the_tree_identifier_watermark_of_every_root`]）：只红 I-7.8，红在盘上出现过的最大树 ID 上
/// （十三棵树的条目还在每一版的树表里）。
/// 这一格钉住「累计高水位」那个读法（D8（核心索引结构） 已定项 8 ②「累计这两个字是承重的」）：判法改成「本 checkpoint 发出的最高号」
/// ——只数最新那一次发布才头一回出现的树 ID——时，环里最新那次发布一个号都没发，I-7.8 照绿，这一条红。
#[test]
fn a_tree_identifier_watermark_pressed_below_the_trees_issued_by_a_root_rotated_out_of_the_ring_reddens_only_the_watermark_invariant(
) {
    let mut pool = build_pool("known-bad-watermark-after-the-tree-issuing-root-rotated-out");
    let tree_issuing_root = pool.output.root;
    assert_eq!(
        (
            tree_issuing_root.checkpoint_txg,
            tree_issuing_root.tree_identifier_watermark
        ),
        (
            CheckpointTxg(3),
            singlefs_format::TREE_IDENTIFIER_WATERMARK_AFTER_FIRST_PUBLISH
        ),
        "发树那次发布是 txg 3、水位推到发第一个文件之后那一档"
    );
    let roots_in_the_ring = singlefs_format::ROOT_RING_REGIONS
        * singlefs_format::ROOT_RING_SLOTS_PER_REGION_AT_MAKE_FILESYSTEM;
    // 迭代上界：根环的槽数；每次覆盖写换一份内容，没有提前出口。
    for overwrite_index in 0..roots_in_the_ring {
        let content: Vec<u8> = (0..3000_u64)
            .map(|index| {
                u8::try_from((index * 11 + overwrite_index * 7 + 5) % 251).expect("小于 256")
            })
            .collect();
        overwrite_in_process_as_instance_one_and_advance_the_pool(&mut pool, &content);
    }
    let clean = pool.memory_pool();
    let system_configuration =
        singlefs_core::recovery::choose_system_configuration(&clean).expect("系统配置");
    let roots_left_in_the_ring = singlefs_core::recovery::readable_roots(
        &clean,
        &system_configuration.immutable.region_devices,
        &system_configuration.immutable.sizes,
        &system_configuration.immutable.filesystem_identifier,
    );
    assert_eq!(
        u64::try_from(roots_left_in_the_ring.len()).expect("根数装得进 u64"),
        roots_in_the_ring,
        "根环每一槽都住着一条覆盖写发的根"
    );
    assert!(
        roots_left_in_the_ring.iter().all(|root| {
            root.checkpoint_txg > tree_issuing_root.checkpoint_txg
                && root.tree_identifier_watermark
                    == singlefs_format::TREE_IDENTIFIER_WATERMARK_AFTER_FIRST_PUBLISH
        }),
        "发树那条根与它之前的根都轮出了根环；环里每条根照累计高水位带着发第一个文件之后那一档：{:?}",
        roots_left_in_the_ring
            .iter()
            .map(|root| (root.checkpoint_txg, root.tree_identifier_watermark))
            .collect::<Vec<_>>()
    );
    assert_lowering_the_watermark_reddens_only_the_watermark_invariant(
        &clean,
        singlefs_format::TREE_IDENTIFIER_WATERMARK_AT_MKFS,
        singlefs_format::TREE_IDENTIFIER_WATERMARK_AFTER_FIRST_PUBLISH - 1,
        "发树那条根轮出根环之后，环里每条根的水位压到 mkfs 那一档",
    );
}

/// C479（inode 号复用已发过的号今天造不出来） 「活记录被原地换」那一形在被换之前那一版转出回退候选集之后（欠账那一行造过的盘面）：
/// 新池新建文件之后建 inode 2、3、4（水位 5，那次发布留台阶 (txg 4, 5)），只供测试的开关把水位压回 4 再建一个——4 号那条活记录被原地换成新对象；
/// 再覆盖写第一个文件 3 次、把 F 抬到准入上限（第 4 新的非空有效根，就是重发那一版）：还留着 4 号旧对象的那几版都低于 F、转出回退候选集。
/// I-9.29（同号只许一个出生代） 看不见两份出生代、成立；水位 5 仍大于树里最大的 key 4、I-9.6 成立；
/// 只有 I-9.30 ② 判红——台阶 (4, 5) 说 4 号在 txg 4 之前就发过，4 号的新对象晚于它出生，单凭最新根就判得出。
/// 判别力：关掉 ② 里按 inode 树记录比台阶那一判（`walk.rs` 的 `judge_inode_number_watermark_steps_of_the_newest_root`），这一条红。
#[test]
fn a_reissued_inode_number_whose_old_object_left_the_rollback_candidate_set_reddens_only_the_watermark_step_invariant(
) {
    let mut pool = common_deletion_in_progress::pool_with_new_inodes(
        "known-bad-reissued-inode-number-after-the-old-version-left-the-candidates",
        3,
    );
    let (reissued, _) =
        scenarios_of_inode_number_watermark_pressed_back_reissues_a_number::create_one_inode_with_the_watermark_from(
            &mut pool,
            scenarios_of_inode_number_watermark_pressed_back_reissues_a_number::pressed_back_to(4),
        );
    let txg_of_the_reissue = reissued.root.checkpoint_txg;
    assert_eq!(
        (
            reissued.inode_number_watermark(),
            scenarios_of_inode_number_watermark_pressed_back_reissues_a_number::object_birth_txg_of(
                &reissued, 4
            ),
        ),
        (5, Some(txg_of_the_reissue.0)),
        "从压回去的 4 起发一个：4 号那条活记录换成这次发布建的新对象，水位写成 5"
    );
    scenarios_of_inode_number_watermark_pressed_back_reissues_a_number::overwrite_the_first_file_in_process(
        &mut pool, 3,
    );
    let raised = {
        let publish_parameters = parameters();
        let devices = pool.devices.as_mut().expect("镜像还开着");
        singlefs_core::mount::raise_rollback_floor_to_the_admission_ceiling(
            &publish_parameters,
            devices,
            &mut pool.allocator,
            &mut pool.output,
            ShadowLedger::Off,
        )
        .unwrap_or_else(|error| panic!("抬 F 到准入上限：{error:?}"))
    };
    assert!(
        matches!(
            raised,
            singlefs_core::mount::RaiseToTheAdmissionCeiling::Raised(_)
        ),
        "重发之后覆盖写三次，上限高于 F：{raised:?}"
    );
    assert!(
        pool.output.root.rollback_floor >= txg_of_the_reissue,
        "F 抬到 {:?}，不低于重发那一版的 txg {txg_of_the_reissue:?}：还留着 4 号旧对象的那几版都转出了回退候选集",
        pool.output.root.rollback_floor
    );
    let image = pool.memory_pool();
    let verdicts = check_pool_image(&image);
    assert_eq!(
        violated_invariants(&verdicts),
        [INODE_NUMBER_WATERMARK_STEPS_AGREE_WITH_THE_WATERMARK_THE_RECORDS_AND_THE_LINEAGE],
        "旧版转出候选集之后只红 I-9.30：{verdicts:?}"
    );
    for invariant_judged_and_holding in ["I-9.6", ONE_OBJECT_BIRTH_PER_INODE_NUMBER] {
        assert_eq!(
            verdict(&image, invariant_judged_and_holding),
            InvariantVerdict::Holds,
            "{invariant_judged_and_holding} 判过且成立"
        );
    }
    let found = verdict(
        &image,
        INODE_NUMBER_WATERMARK_STEPS_AGREE_WITH_THE_WATERMARK_THE_RECORDS_AND_THE_LINEAGE,
    );
    assert!(
        matches!(
            &found,
            InvariantVerdict::Violated(detail)
                if detail.contains("inode 树里 4 号的对象出生代") && detail.contains("(4, 5)")
        ),
        "I-9.30 红在 ② 那一判上：说明点名重发的 4 号与它晚于的那一级台阶 (4, 5)：{found:?}"
    );
}

/// 实例表行的 W 那一条（I-3.19，C740（实例表行的 W 与恢复施加的最大事务号没有不变量判））、新实例第一个 txg 那一条（I-8.14，
/// C764（新实例第一个 txg 被改坏没有检查会红））与由记录施加出来的树表 0 条那一版的分配记录树根那一条（I-5.7，C706（记录单落那一版的分配记录树根
/// 没有判据））：坏镜像都建在真写者搭出来、恢复由记录施加过一版的盘上（判决 `research/prompts/m3-close-b19-design-r2-main-verification.md`
/// 第二节 Q2、Q4、Q8），不在写完新池新建文件那一份上——那一份上只有实例 1、实例表一行都没有，三条都没有判的对象。
pub(super) const ROW_HIGH_WATER_FIRST_TXG_AND_REPLAYED_ALLOCATION_RECORD_TREE_ROOT_INVARIANTS_AND_WHERE_THEIR_BAD_IMAGES_LIVE: [(
    &str,
    &str,
); 3] = [
    (
        INSTANCE_TABLE_ROW_HIGH_WATER_IS_ONE_A_RECOVERY_COULD_HAVE_APPLIED,
        "checker_known_bad_images.rs: row_whose_high_water_is_written_as_zero_after_applying_a_record_whose_root_never_landed_reddens_the_row_high_water_invariant",
    ),
    (
        FIRST_TXG_OF_EACH_INSTANCE_IS_ABOVE_EVERY_ROOT_AND_RECORD_OF_THE_OLDER_INSTANCES,
        "checker_known_bad_images.rs: first_record_of_the_new_instance_numbered_with_the_txg_of_the_record_it_applied_reddens_the_first_txg_invariant",
    ),
    (
        ALLOCATION_RECORD_TREE_ROOT_OF_A_VERSION_APPLIED_ONLY_BY_RECORDS_IS_BORN_WITH_THE_VERSION,
        "checker_known_bad_images.rs: row_publish_record_carrying_the_allocation_record_tree_root_of_the_previous_row_publish_reddens_the_allocation_record_tree_root_birth_invariant",
    ),
];

/// I-3.19 在真写者上（集合形）：发布 (1, 5) 只落了单元与记录、根槽从没落盘，可写挂载择 B 的根 (1, 4)、
/// 施加那条记录、给实例 1 写行（[`writable_mount_over_a_version_applied_only_by_its_journal_record`]）。T 5 的根槽全零：落在 T 上那一格不在集合里；
/// 从环里实例 1 每条更早的根（txg 1–4）起重放到 5，那一段最大的非 0 事务号都是 3 ⇒ 集合 {3}，W 要是 3。
/// 判别力（`crates/mutations.tsv` 里点名这条的行）：可写挂载写行时 W 少记一，这条红在 I-3.19 那一句上（行的钉值在判定之后才核）。
#[test]
fn row_written_after_applying_a_record_whose_root_never_landed_holds_the_row_high_water_invariant()
{
    let built = writable_mount_over_a_version_applied_only_by_its_journal_record(
        "row-high-water-after-applying-a-record",
    );
    assert_eq!(
        verdict(
            &built.image,
            INSTANCE_TABLE_ROW_HIGH_WATER_IS_ONE_A_RECOVERY_COULD_HAVE_APPLIED
        ),
        InvariantVerdict::Holds,
        "行 (1, 5, W)：T 5 的根槽全零、集合不含 0，从实例 1 每条更早的根起重放到 5 都只施加到事务号 3，W 要是 3"
    );
    assert_eq!(
        built.rows_written,
        vec![InstanceRow {
            instance: InstanceGeneration(1),
            selected_root_txg: CheckpointTxg(5),
            applied_transaction_high_water: 3,
        }],
        "这次挂载写的行是 (1, 5, 3)"
    );
}

/// 实例表行里 W（属于该实例的最大已施加事务号）的偏移：kind 1、实例代号 4、所选根的 txg 8 之后（`crates/singlefs-core/src/instance_table.rs` 的行记录）。
const INSTANCE_TABLE_ROW_HIGH_WATER_OFFSET: usize = 13;

/// 可写挂载施加过一条记录那一份上（[`image_after_a_writable_mount_over_a_version_applied_only_by_its_journal_record`]）：最新根（暖机 txg 7）的实例表里
/// 实例 1 那一行 (1, 5, 3) 的 W 改成 `high_water`，重封、补到引用它的根槽。
fn set_the_high_water_of_the_row_of_instance_one_after_the_mount_over_the_applied_version(
    image: &mut MemoryPool,
    high_water: u64,
) {
    const INSTANCE_TABLE_ROW_INSTANCE_OFFSET: usize = 1;
    const INSTANCE_TABLE_ROW_BYTES: usize = 88;
    let (device, offset) = root_slot_of(CheckpointTxg(7));
    let newest_root = read(image, device, offset, 512);
    assert_eq!(get_u64(&newest_root, 28), 7, "最新的根是暖机 txg 7");
    let location = ROOT_RECORD_INSTANCE_TABLE_POINTER_OFFSET + 50;
    let mut slot_bytes = [0u8; 8];
    slot_bytes[..6].copy_from_slice(&newest_root[location + 4..location + 10]);
    let instance_table_slot = u64::from_le_bytes(slot_bytes);
    mutate_unit_of(
        image,
        &[(instance_table_slot, 32768)],
        instance_table_slot,
        |bytes| {
            let row_count = usize::from(u16::from_le_bytes([bytes[69], bytes[70]]));
            let row = (0..row_count)
                .map(|index| INSTANCE_TABLE_FIRST_ROW_START + INSTANCE_TABLE_ROW_BYTES * index)
                .find(|row| {
                    bytes[*row] == 0
                        && get_u32(bytes, row + INSTANCE_TABLE_ROW_INSTANCE_OFFSET) == 1
                })
                .expect("实例表里有实例 1 那一行");
            assert_eq!(
                get_u64(bytes, row + INSTANCE_TABLE_ROW_HIGH_WATER_OFFSET),
                3,
                "那一行是 (1, 5, 3)"
            );
            set_u64(
                bytes,
                row + INSTANCE_TABLE_ROW_HIGH_WATER_OFFSET,
                high_water,
            );
        },
        true,
    );
}

/// I-3.19 的坏镜像（C740 的判别力自证：W 写成 0 的那一张必须红）：同一份镜像上实例 1 那一行的 W 3 → 0。集合 {3} 不含 0，
/// 根环与那一段记录都读得全、认不出的格一格都没有 ⇒ I-3.19 红在这一行上。
#[test]
fn row_whose_high_water_is_written_as_zero_after_applying_a_record_whose_root_never_landed_reddens_the_row_high_water_invariant(
) {
    let mut image = image_after_a_writable_mount_over_a_version_applied_only_by_its_journal_record(
        "known-bad-row-high-water-zero",
    )
    .image;
    set_the_high_water_of_the_row_of_instance_one_after_the_mount_over_the_applied_version(
        &mut image, 0,
    );
    let found = verdict(
        &image,
        INSTANCE_TABLE_ROW_HIGH_WATER_IS_ONE_A_RECOVERY_COULD_HAVE_APPLIED,
    );
    let InvariantVerdict::Violated(detail) = &found else {
        panic!("行 (1, 5, 0)：W 0 不在集合 {{3}} 里，I-3.19 要红，实际 {found:?}");
    };
    assert!(
        detail.contains("(实例 1, T 5, W 0)") && detail.contains("{3}"),
        "红在实例 1 那一行、集合 {{3}} 上：{detail}"
    );
}

/// I-8.14 在真写者上：同一段历史，实例 1 读得出的根与记录里 txg 最大的是只由记录施加出来的那一版
/// (1, 5) 那条记录（根槽从没落盘、全零，不算读不出）；新实例 2 的第一次发布（写行）要高过它：
/// max(根环里最大的 txg 4, 环里记录最大的 txg 5) + 1 = 6（D23（journal 的角色与格式） 已定项 14 注 3）。判成立。
/// 判别力（`crates/mutations.tsv` 里点名这条的行）：挂载取新实例第一个 txg 只看根环（4 + 1 = 5，与那条记录同 txg），这条红在 I-8.14 那一句上
/// （写行发布的 txg 在判定之后才核）。
#[test]
fn writable_mount_after_applying_a_record_whose_root_never_landed_takes_a_first_txg_above_that_record_and_holds_the_first_txg_invariant(
) {
    let built = writable_mount_over_a_version_applied_only_by_its_journal_record(
        "first-txg-above-the-applied-record",
    );
    assert_eq!(
        verdict(
            &built.image,
            FIRST_TXG_OF_EACH_INSTANCE_IS_ABOVE_EVERY_ROOT_AND_RECORD_OF_THE_OLDER_INSTANCES
        ),
        InvariantVerdict::Holds,
        "实例 2 的第一个 txg 要高过实例 1 那条只由记录施加出来的 txg 5 的记录"
    );
    assert_eq!(
        built.row_publish_txg,
        CheckpointTxg(6),
        "实例 2 写行那次发布是 txg 6"
    );
}

/// I-8.14 的坏镜像（C764「新实例第一个 txg 被改坏」那一形）：同一份镜像上实例 2 写行那次发布的那条记录（两盘各一份）checkpoint_txg 6 → 5，
/// 重封、同一实例的下一条记录反向链跟着重算：实例 2 的第一个 txg 落到 5，不高于实例 1 那条 txg 5 的记录 ⇒ I-8.14 红。
#[test]
fn first_record_of_the_new_instance_numbered_with_the_txg_of_the_record_it_applied_reddens_the_first_txg_invariant(
) {
    let built = image_after_a_writable_mount_over_a_version_applied_only_by_its_journal_record(
        "known-bad-first-txg-of-the-new-instance",
    );
    let mut image = built.image.clone();
    for device in DEVICES {
        rewrite_journal_record_on_one_device_keeping_the_back_chain(
            &mut image,
            device,
            built.row_publish_counter,
            |record| {
                assert_eq!(
                    (record.instance, record.checkpoint_txg),
                    (InstanceGeneration(2), CheckpointTxg(6)),
                    "改的是实例 2 写行那次发布的记录"
                );
                record.checkpoint_txg = CheckpointTxg(5);
            },
        );
    }
    let found = verdict(
        &image,
        FIRST_TXG_OF_EACH_INSTANCE_IS_ABOVE_EVERY_ROOT_AND_RECORD_OF_THE_OLDER_INSTANCES,
    );
    let InvariantVerdict::Violated(detail) = &found else {
        panic!("实例 2 的第一个 txg 5 不高于实例 1 那条 txg 5 的记录：I-8.14 要红，实际 {found:?}");
    };
    assert!(
        detail.contains("实例 2 的第一个 txg 是 5") && detail.contains("txg 5）"),
        "红在实例 2 的第一个 txg 5 与实例 1 那条 txg 5 的记录上：{detail}"
    );
}

/// 四次可写挂载、一个文件都不写的池（树表一直 0 条；C706 那一形）：实例 1 不写行（零单元发布），
/// 实例 2、3、4 各发一次写行发布，每次重建分配记录树、根指针住根记录与记录新根段、点名实例表与分配记录树节点；
/// 第四次挂载给实例 3 写的行发布到不早于实例 3 那次写行发布的 txg。
struct FourWritableMountsOfAPoolWithoutAFile {
    image: MemoryPool,
    /// 实例 3 写行那次发布的 txg 与它那条记录的计数器。
    row_publish_txg_of_instance_three: CheckpointTxg,
    row_publish_counter_of_instance_three: u64,
    /// 实例 2 写行那次发布的 txg 与它写进根记录的分配记录树根。
    row_publish_txg_of_instance_two: CheckpointTxg,
    allocation_record_tree_root_of_instance_two: singlefs_core::pointer::NodePointer,
}

fn four_writable_mounts_of_a_pool_without_a_file(
    tag: &str,
) -> FourWritableMountsOfAPoolWithoutAFile {
    let mut formatted = format_pool(tag);
    let mut row_publishes = Vec::new();
    let mut rows_written_by_the_last_mount = Vec::new();
    // 迭代 4 次：同一个池上重开镜像挂四次，每次取号加一，不新建池。
    for instance in 1..=4u32 {
        let mut devices = formatted.reopen_recorded();
        let mounted =
            mount_writable(&parameters(), &mut devices).expect("只挂载、不写文件的池照常可写挂载");
        formatted.devices = Some(devices);
        assert_eq!(
            mounted.output.instance,
            InstanceGeneration(instance),
            "第 {instance} 次可写挂载取号 {instance}"
        );
        let PoolVersion::WithoutFile(row_publish) = &mounted.output.row_publish else {
            panic!("一个文件都不写：第 {instance} 次挂载写行那次发布出来的仍是树表 0 条的一版");
        };
        row_publishes.push((
            row_publish.root.checkpoint_txg,
            row_publish.record.counter,
            row_publish.root.allocation_record_tree_root,
            row_publish.record.named.len(),
        ));
        rows_written_by_the_last_mount = mounted.output.rows_written.clone();
    }
    let (
        row_publish_txg_of_instance_two,
        _,
        allocation_record_tree_root_of_instance_two,
        named_by_instance_two,
    ) = row_publishes[1];
    let (
        row_publish_txg_of_instance_three,
        row_publish_counter_of_instance_three,
        allocation_record_tree_root_of_instance_three,
        named_by_instance_three,
    ) = row_publishes[2];
    assert!(
        named_by_instance_two > 0 && named_by_instance_three > 0,
        "实例 2、3 的写行发布点名实例表与分配记录树节点"
    );
    assert_eq!(
        (
            allocation_record_tree_root_of_instance_two.head.birth_txg,
            allocation_record_tree_root_of_instance_three.head.birth_txg
        ),
        (
            row_publish_txg_of_instance_two,
            row_publish_txg_of_instance_three
        ),
        "写行发布每次重建分配记录树：根指针出生在那一版"
    );
    assert!(
        rows_written_by_the_last_mount.iter().any(|row| {
            row.instance == InstanceGeneration(3)
                && row.selected_root_txg >= row_publish_txg_of_instance_three
        }),
        "第四次挂载给实例 3 写的行发布到实例 3 写行那次发布之后：{rows_written_by_the_last_mount:?}"
    );
    FourWritableMountsOfAPoolWithoutAFile {
        image: formatted.memory_pool(),
        row_publish_txg_of_instance_three,
        row_publish_counter_of_instance_three,
        row_publish_txg_of_instance_two,
        allocation_record_tree_root_of_instance_two,
    }
}

/// I-5.7 在真写者上：四次可写挂载、一个文件都不写的池，根槽都读得出时没有「由记录施加出来」的那一版，I-5.7 报不适用；
/// 把实例 3 写行那次发布的根槽改坏（读坏一个根槽的镜像算合法）：实例表里实例 3 那一行发布到它之后、它的根槽读不出，
/// 它就是由记录施加出来的那一版，记录点名项非空、新根段里的分配记录树根出生在它自己的 txg ⇒ 判成立。
/// 判别力（`crates/mutations.tsv` 里点名这条的行）：I-5.7 的前件把「点名项非空」读反，这条红在成立那一句上。
#[test]
fn row_publish_on_a_version_without_file_whose_root_slot_tore_after_the_next_mount_holds_the_allocation_record_tree_root_birth_invariant(
) {
    let pool = four_writable_mounts_of_a_pool_without_a_file(
        "allocation-record-tree-root-born-with-the-row-publish",
    );
    assert_eq!(
        verdict(
            &pool.image,
            ALLOCATION_RECORD_TREE_ROOT_OF_A_VERSION_APPLIED_ONLY_BY_RECORDS_IS_BORN_WITH_THE_VERSION
        ),
        InvariantVerdict::NotApplicable(
            NO_VERSION_APPLIED_ONLY_BY_RECORDS_REWRITING_ITS_ALLOCATION_RECORD_TREE
        ),
        "根槽都读得出：没有由记录施加出来的那一版"
    );
    let mut torn = pool.image.clone();
    tear_the_root_slot_written_for_txg(&mut torn, pool.row_publish_txg_of_instance_three.0);
    assert_eq!(
        verdict(
            &torn,
            ALLOCATION_RECORD_TREE_ROOT_OF_A_VERSION_APPLIED_ONLY_BY_RECORDS_IS_BORN_WITH_THE_VERSION
        ),
        InvariantVerdict::Holds,
        "实例 3 写行那次发布的根槽读不出：那一版由记录施加出来，分配记录树根出生在它自己的 txg"
    );
}

/// 两盘上计数器 `counter` 那条记录按 `change` 改、重封；它后面同一实例的每一条记录反向链按改后的前一条重算、重封。
/// 下一个实例的第一条记录反向链恒 0（D23（journal 的角色与格式） 已定项 19 ②），不跟着改。
fn rewrite_journal_record_and_rechain_the_later_records_of_its_instance(
    image: &mut MemoryPool,
    counter: u64,
    change: impl Fn(&mut JournalRecord),
) {
    let ring_bytes = singlefs_format::JOURNAL_RING_DEFAULT_BYTES;
    for device in DEVICES {
        let offset = record_offset(counter, ring_bytes).0;
        let bytes = read(image, device, offset, 4096);
        let filesystem_identifier = get_u64(&bytes, JOURNAL_RECORD_FILESYSTEM_IDENTIFIER_OFFSET);
        let mut record =
            JournalRecord::parse(&bytes, filesystem_identifier).expect("改之前那条记录自证过");
        change(&mut record);
        let mut previous_bytes = record.to_bytes();
        write(image, device, offset, &previous_bytes);
        // 迭代次数的上界是这个实例在 `counter` 之后写的记录条数：下一格解不出、计数器对不上或换了实例就是出口。
        let mut later_counter = counter + 1;
        loop {
            let later_offset = record_offset(later_counter, ring_bytes).0;
            let later_bytes = read(image, device, later_offset, 4096);
            let Some(mut later) = JournalRecord::parse(&later_bytes, filesystem_identifier) else {
                break;
            };
            if later.instance != record.instance || later.counter != later_counter {
                break;
            }
            later.back_chain = back_chain_of(&previous_bytes);
            previous_bytes = later.to_bytes();
            write(image, device, later_offset, &previous_bytes);
            later_counter += 1;
        }
    }
}

/// I-5.7 的坏镜像（C706 的必红世界）：同一份改坏了根槽的镜像上，实例 3 写行那次发布的记录（两盘各一份）新根段里的分配记录树根
/// 换成实例 2 写行那次发布的（一棵合法的、树表 0 条那一版的旧树），重封、同一实例后面那几条记录的反向链跟着重算。点名项没变、前件照样成立，
/// 出生 txg 是实例 2 那一版的 ⇒ I-5.7 红；两棵都是合法的树，I-5.4 照旧成立。
/// 这一份上另有两条跟着红，都是换过的那棵树与相邻的那一版对不上：实例 3 的暖机根照抄的是实例 3 自己那棵树，(3, 5) → (3, 6) 后一版引用的
/// 分配记录树节点前一版不引用、诞生代号又不大于前一版（I-3.5）；实例 2 那棵树里罩着实例 2 那片实例表的未释放记录，在这一版里没有映射指着它
/// （I-3.15）。别的判定与换之前逐条相同。
#[test]
fn row_publish_record_carrying_the_allocation_record_tree_root_of_the_previous_row_publish_reddens_the_allocation_record_tree_root_birth_invariant(
) {
    let pool = four_writable_mounts_of_a_pool_without_a_file(
        "known-bad-allocation-record-tree-root-of-the-previous-row-publish",
    );
    let mut image = pool.image.clone();
    tear_the_root_slot_written_for_txg(&mut image, pool.row_publish_txg_of_instance_three.0);
    let verdicts_before_the_swap = check_pool_image(&image);
    rewrite_journal_record_and_rechain_the_later_records_of_its_instance(
        &mut image,
        pool.row_publish_counter_of_instance_three,
        |record| {
            assert_eq!(
                (record.instance, record.checkpoint_txg),
                (
                    InstanceGeneration(3),
                    pool.row_publish_txg_of_instance_three
                ),
                "改的是实例 3 写行那次发布的记录"
            );
            record.new_allocation_record_tree_root =
                pool.allocation_record_tree_root_of_instance_two;
        },
    );
    let verdicts = check_pool_image(&image);
    let found = verdict_among(
        &verdicts,
        ALLOCATION_RECORD_TREE_ROOT_OF_A_VERSION_APPLIED_ONLY_BY_RECORDS_IS_BORN_WITH_THE_VERSION,
    );
    let InvariantVerdict::Violated(detail) = &found else {
        panic!("记录里的分配记录树根出生在实例 2 那一版：I-5.7 要红，实际 {found:?}");
    };
    assert!(
        detail.contains(&format!(
            "实例 3、txg {}",
            pool.row_publish_txg_of_instance_three.0
        )) && detail.contains(&format!(
            "出生 txg 是 {}",
            pool.row_publish_txg_of_instance_two.0
        )),
        "红在实例 3 写行那一版、出生 txg 是实例 2 那一版的：{detail}"
    );
    assert_eq!(
        verdict_among(&verdicts, "I-5.4"),
        InvariantVerdict::Holds,
        "两棵都是合法的树：I-5.4 照旧成立"
    );
    let changed_with_the_neighbouring_version = [
        ALLOCATION_RECORD_TREE_ROOT_OF_A_VERSION_APPLIED_ONLY_BY_RECORDS_IS_BORN_WITH_THE_VERSION,
        REFERENCE_INTERVALS_ARE_EXACT,
        UNRELEASED_ALLOCATION_RECORD_COVERS_A_UNIT_THE_CENTRAL_MAPPING_PLACES_THERE,
    ];
    for red_with_the_neighbouring_version in [
        REFERENCE_INTERVALS_ARE_EXACT,
        UNRELEASED_ALLOCATION_RECORD_COVERS_A_UNIT_THE_CENTRAL_MAPPING_PLACES_THERE,
    ] {
        assert!(
            matches!(
                verdict_among(&verdicts, red_with_the_neighbouring_version),
                InvariantVerdict::Violated(_)
            ),
            "{red_with_the_neighbouring_version} 跟着红（换过的树与相邻那一版对不上）：{verdicts:?}"
        );
    }
    assert_eq!(
        verdicts_that_differ_outside(
            &verdicts,
            &verdicts_before_the_swap,
            &changed_with_the_neighbouring_version
        ),
        Vec::new(),
        "换之前与换之后，I-5.7、I-3.5、I-3.15 之外的判定逐条相同"
    );
}

/// 表里每一条坏镜像都红在自己的目标不变量上（`known-bad-images` 模式 S4 的 target_is_red，这里直接判 `MemoryPool`，不起给定状态引擎）。
/// 一张固定的表判一个结论：整张表还成立；每一条都从干净镜像的拷贝改起，互不依赖。
#[test]
fn every_known_bad_image_of_the_table_reddens_its_target_on_the_pool_checker() {
    let clean = build_pool("known-bad-targets").memory_pool();
    let entries = known_bad_images(&clean);
    let targets_not_red: Vec<(usize, &str)> = entries
        .iter()
        .enumerate()
        .filter_map(|(entry_index, (target, apply))| {
            let mut bad = clean.clone();
            apply(&mut bad);
            let target_is_red = check_pool_image(&bad).iter().any(|(name, verdict)| {
                *name == *target && matches!(verdict, InvariantVerdict::Violated(_))
            });
            (!target_is_red).then_some((entry_index, *target))
        })
        .collect();
    assert!(
        targets_not_red.is_empty(),
        "这几条没红在目标不变量上：{targets_not_red:?}"
    );
}
