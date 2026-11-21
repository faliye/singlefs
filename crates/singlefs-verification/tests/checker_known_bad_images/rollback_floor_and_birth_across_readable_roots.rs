//! 回退下限与每个读得出的根上的出生代号：抬过上限、系统配置下限、卸载标记抬的下限、回退之后另记的出生代号。

use super::*;

/// 一片树表单元里 extent 树（树 ID 65536，按树 ID 升序排第一条）那条的诞生 txg 从 `from` 改成 `to`，重封、沿引用链补到根槽。
fn rewrite_the_extent_tree_birth_txg_in_tree_table(
    image: &mut MemoryPool,
    tree_table_slot: u64,
    from: u64,
    to: u64,
) {
    let node_bytes = usize::try_from(NODE_BYTES).expect("16384");
    mutate_unit_of(
        image,
        &[(tree_table_slot, node_bytes)],
        tree_table_slot,
        |bytes| {
            assert_eq!(
                get_u64(bytes, tree_table_entry_offset(0)),
                TREE_IDENTIFIER_EXTENT,
                "树表第一条是 extent 树"
            );
            let birth_txg = tree_table_entry_offset(0) + TREE_TABLE_ENTRY_BIRTH_TXG_OFFSET;
            assert_eq!(
                get_u64(bytes, birth_txg),
                from,
                "槽 {tree_table_slot} 那片树表里 extent 树的诞生 txg 本来是 {from}"
            );
            set_u64(bytes, birth_txg, to);
        },
        true,
    );
}

/// 反向那一头（C511 收窄之后仍要红的那一格）：挂着的时候回退到 A 之后，D 那片树表把 extent 树的诞生 txg 记成 D 那次发布的 txg
/// （在现行线上重新建树的样子），而回退目标 A 那一版仍记 3。向前回退不切掉任何一条线，A 与 D 在**同一条时间线上**
/// ⇒ I-9.14（树表条目的诞生 txg 跨根不变） 必须红；同一条线上记得不同必然也是全部可读根里记得不同，I-9.28（全部可读根里同一树 ID 的
/// 诞生 txg 相同） 跟着红，别的一条不红。把 I-9.14 的收窄做宽到把回退目标判出局的，这条红。
#[test]
fn birth_txg_that_the_line_after_a_rollback_records_differently_from_the_rollback_target_reddens_only_the_same_line_and_the_every_readable_root_birth_invariants(
) {
    let rolled_back =
        image_after_rolling_back_to_the_first_root("known-bad-after-rollback-same-line");
    for (invariant, found) in check_pool_image(&rolled_back.image) {
        assert_eq!(
            found,
            if invariant == LINEAGE_TREE_ENTRY_MATCHES_ITS_CHILD_HEADER {
                // 回退那次发布写出谱系树（一片容器、一条谱系记录）：它的根条目真被评估过且成立。
                InvariantVerdict::Holds
            } else if invariant == SCAN_REBUILD_AGREES_WITH_THE_WALKED_INDEX {
                // 谱系树里有记录：扫描重建按「不是任何现行版本的成员」认领回退抛弃的那一段，
                // 择回退目标那一版，与遍历方向真比过、成立。
                InvariantVerdict::Holds
            } else {
                expected_verdict_with_one_record_per_transaction(invariant)
            },
            "回退到 A 之后的干净镜像上 {invariant} 的判定"
        );
    }
    let mut image = rolled_back.image.clone();
    rewrite_the_extent_tree_birth_txg_in_tree_table(
        &mut image,
        rolled_back.tree_table_slot_of_the_rollback_publish,
        rolled_back.rollback_target_txg,
        rolled_back.rollback_publish_txg,
    );
    let verdicts = check_pool_image(&image);
    let violated: Vec<&str> = verdicts
        .iter()
        .filter(|(_, verdict)| matches!(verdict, InvariantVerdict::Violated(_)))
        .map(|(name, _)| *name)
        .collect();
    assert_eq!(
        violated,
        [
            "I-9.14",
            TREE_IDENTIFIER_BIRTH_TXG_AGREES_ACROSS_EVERY_READABLE_ROOT
        ],
        "回退目标 A 与回退那次发布 D 同在现行线上，诞生 txg 不一致只该红 I-9.14 与 I-9.28：{verdicts:?}"
    );
}

/// I-9.28（全部可读根里同一树 ID 的诞生 txg 相同） 的坏镜像，从「崩溃恢复抛弃了第三条根 C」那一份干净镜像出发：只改被抛弃的 C 那片树表里
/// extent 树（树 65536）的诞生 txg（`birth_txg` → C 那次发布的 txg `abandoned_publish_txg`），重封、沿引用链补到 C 的根槽。
/// C 不在回退候选集里（I-9.14 不拿它比）；I-9.28 不分时间线、比根环里全部读得出的根。
pub(super) fn known_bad_images_of_the_birth_txg_across_every_readable_root(
    tree_table_slot_of_the_abandoned_root: u64,
    birth_txg: u64,
    abandoned_publish_txg: u64,
) -> Vec<(&'static str, Mutation)> {
    vec![(
        TREE_IDENTIFIER_BIRTH_TXG_AGREES_ACROSS_EVERY_READABLE_ROOT,
        Box::new(move |image: &mut MemoryPool| {
            rewrite_the_extent_tree_birth_txg_in_tree_table(
                image,
                tree_table_slot_of_the_abandoned_root,
                birth_txg,
                abandoned_publish_txg,
            );
        }),
    )]
}

/// C511 的收窄那一头加 I-9.28 那一头：只有被崩溃恢复切掉的 C 把 extent 树的诞生 txg 记成它自己那次发布的 txg，
/// 现行线上 A、B、实例 2 与实例 3 的根仍一致记 3 ⇒ I-9.14（树表条目的诞生 txg 跨根不变） 不拿 C 比，判成立；
/// I-9.28（全部可读根里同一树 ID 的诞生 txg 相同） 比根环里全部读得出的根，C 也在里面，判红；别的每一条与干净镜像上相同。
/// 管理员回退改成向前发布之后，被切掉的线只由崩溃恢复造出（D23（journal 的角色与格式） 已定项 14）。
/// 把收窄去掉（I-9.14 拿根环里每一条根比，不按实例表判出局）这条红在 I-9.14 上；把 I-9.28 收成只比回退候选集，这条红在 I-9.28 上。
#[test]
fn birth_txg_recorded_differently_only_on_the_timeline_cut_off_by_a_recovery_holds_the_same_line_invariant_and_reddens_only_the_every_readable_root_invariant(
) {
    let (clean, birth_txg, tree_table_slot_of_the_abandoned_root, abandoned_publish_txg) =
        image_after_a_recovery_abandoned_the_third_root("known-good-after-recovery-abandoned-line");
    for (invariant, found) in check_pool_image(&clean) {
        // I-3.16 的全空段那一半在这一张上真判、成立：崩溃恢复之后盘 0 有一段只剩 defer 槽，按记录口径全空、按占用口径不空（两个口径分得开），
        // 核心层记账行写的与 checker 从最新根的记录现扫的都是记录口径（C372（全空聚簇段数两份定义相反） 代码那一半收口），两边相等。
        // 核心层改回占用口径（释放不让槽离开未释放的记录）时这里判红。
        if invariant == FRAGMENTATION_AND_EMPTY_SEGMENT_STATISTICS_EQUAL_A_FULL_SCAN {
            assert_eq!(
                found,
                InvariantVerdict::Holds,
                "崩溃恢复抛弃了 C 之后的干净镜像上 I-3.16 连全空段那一半都真判、成立（核心层计数与 checker 重算相等）"
            );
            continue;
        }
        assert_eq!(
            found,
            expected_verdict_with_one_record_per_transaction(invariant),
            "崩溃恢复抛弃了 C 之后的干净镜像上 {invariant} 的判定（I-9.28 在这里真被评估过且成立）"
        );
    }
    for (invariant, mutation) in known_bad_images_of_the_birth_txg_across_every_readable_root(
        tree_table_slot_of_the_abandoned_root,
        birth_txg,
        abandoned_publish_txg,
    ) {
        let mut image = clean.clone();
        mutation(&mut image);
        let verdicts = check_pool_image(&image);
        let birth_invariant = verdicts
            .iter()
            .find(|(name, _)| *name == "I-9.14")
            .expect("清单里有 I-9.14");
        assert_eq!(
            birth_invariant.1,
            InvariantVerdict::Holds,
            "被崩溃恢复切掉的 C 不与现行线比；现行线上的树表一致，I-9.14 要真被评估过且成立"
        );
        for (name, found) in &verdicts {
            if *name == invariant {
                assert!(
                    matches!(found, InvariantVerdict::Violated(detail)
                        if detail.contains("树 65536") && detail.contains(&format!("txg {abandoned_publish_txg}）记 {abandoned_publish_txg}"))),
                    "C 那片树表把树 65536 的诞生 txg 记成 {abandoned_publish_txg}，{invariant} 要红在它上面：{found:?}"
                );
            } else {
                assert_eq!(
                    *found,
                    expected_verdict_with_one_record_per_transaction(name),
                    "只改了被抛弃根 C 那片树表里的一个诞生 txg，{name} 的判定"
                );
            }
        }
    }
}

/// 这一格用到的内容：长度与字节都随 `seed` 变（具体哪些字节不承重）。
pub(super) fn content_seeded(length: usize, seed: usize) -> Vec<u8> {
    (0..length)
        .map(|index| u8::try_from((index * 7 + seed) % 253).expect("小于 256"))
        .collect()
}

/// 在同一个进程里接着现行版本覆盖写一次，现行版本跟着换成这一版。
pub(super) fn overwrite_and_advance(
    pool: &mut BuiltPool,
    content: &[u8],
    instance: InstanceGeneration,
) {
    let previous = pool.output.clone();
    pool.output = publish_overwrite_in_process(
        pool,
        &previous,
        content,
        FIXED_WRITE_TIME_SECONDS + 60,
        instance,
    )
    .expect("覆盖写");
}

/// 步 5 那段历史走到抬 F 之前：A、B、重开写行、C（实例 2）、挂着的时候回退到 (1, 3)（一次向前发布 D，txg 9，实例仍是 2）、
/// 实例 2 覆盖写五次（txg 10–14）。没有哪条根被实例表判出局；非空的不同状态按新到旧是 14、13、12、11、……（D 与 A 是同一个状态，
/// 只算一次），抬 F 的上限 = min(每块盘上最新的 13, 第 4 个不同状态 11) = 11
/// （`reuse_after_raising_the_floor.rs` 的 `raising_the_floor_above_the_fourth_newest_non_empty_root_is_refused` 钉着同一个上限）。
pub(super) fn pool_before_raising_the_floor(tag: &str) -> BuiltPool {
    let mut pool = build_pool(tag);
    overwrite_and_advance(&mut pool, &content_seeded(4100, 3), InstanceGeneration(1));
    let mut devices = pool.reopen_recorded();
    let mounted = mount_writable(&parameters(), &mut devices).expect("可写挂载");
    pool.devices = Some(devices);
    pool.allocator = mounted.allocator;
    pool.output = mounted
        .current
        .into_file_version()
        .expect("B 之后重开，现行那一版带文件");
    overwrite_and_advance(&mut pool, &content_seeded(2500, 11), InstanceGeneration(2));
    let rollback_parameters = parameters();
    let open_devices = pool.devices.as_mut().expect("镜像还开着");
    roll_back_by_a_forward_publish(
        &rollback_parameters,
        open_devices,
        &mut pool.allocator,
        &mut pool.output,
        RollbackTarget {
            instance: InstanceGeneration(1),
            checkpoint_txg: CheckpointTxg(3),
        },
    )
    .expect("挂着的时候回退到 A");
    assert_eq!(
        pool.output.root.checkpoint_txg,
        CheckpointTxg(9),
        "回退那次发布是 txg 9"
    );
    for seed in [17usize, 19, 23, 29, 31] {
        overwrite_and_advance(
            &mut pool,
            &content_seeded(3000 + seed, seed),
            InstanceGeneration(2),
        );
    }
    assert_eq!(
        pool.output.root.checkpoint_txg,
        CheckpointTxg(14),
        "实例 2 覆盖写到 txg 14"
    );
    pool
}

/// 抬 F 到 `new_floor`，走产品入口 `mount::raise_rollback_floor`：回收、影子账重算、带新 F 的空发布直到每块盘上都有一条，都是真的。
/// `floor_the_entry_is_told_is_current` 是交给入口的「现行版本」里记的 F——入口按「txg ≥ 今天的 F」取有效根算上限，
/// 把它先改成要抬到的那个值，入口算出来的上限就恒不低于它，超过真上限的目标也放得过去。盘上留下的就是步 5 验收那条变异
/// （F 抬过上限）的样子：记账、回收、分配记录与一次合法的抬 F 一样自洽，只有 F 本身抬过了头；盘上 txg 14 那条根带的还是 0，
/// checker 按它认「抬之前的 F」。只改根记录里那 8 个字节造不出它：候选集跟着 F 缩小，txg 11 那条根独占的单元掉出遍历，I-3.1 跟着红。
pub(super) fn raise_the_floor(
    pool: &mut BuiltPool,
    new_floor: CheckpointTxg,
    floor_the_entry_is_told_is_current: CheckpointTxg,
) {
    let mut current = pool.output.clone();
    current.root.rollback_floor = floor_the_entry_is_told_is_current;
    let devices = pool.devices.as_mut().expect("镜像还开着");
    raise_rollback_floor(
        &parameters(),
        devices,
        &mut pool.allocator,
        &mut current,
        new_floor,
        ShadowLedger::On,
    )
    .expect("抬 F");
    pool.output = current;
}

/// 抬 F 到 11 之后再发两版（txg 17、18）：E0 的数据单元落回 mkfs 种的实例表那两槽，E 的数据单元落回 mkfs 种的第 0 版树表那一槽
/// （都只被 txg 11 之下的根引用，抬 F 生效之后合法回收）——第 0 代根那几条的实例表与树表从此读不出。
fn overwrite_and_advance_onto_the_genesis_tree_table_slot(pool: &mut BuiltPool) {
    let data_slot = |current: &BuiltPool| {
        current
            .output
            .unit(TransactionUnit::Data(DataUnitIndexInFile::FIRST))
            .slot
            .0
    };
    overwrite_and_advance(pool, &content_seeded(1200, 3), InstanceGeneration(2));
    assert_eq!(
        data_slot(pool),
        INSTANCE_TABLE_SLOT.0,
        "E0 的数据单元落回 mkfs 种的实例表那两槽"
    );
    overwrite_and_advance(pool, &content_seeded(1234, 5), InstanceGeneration(2));
    assert_eq!(
        data_slot(pool),
        GENESIS_TREE_TABLE,
        "E 的数据单元落回 mkfs 种的第 0 版树表那一槽"
    );
}

/// I-7.9（回退下界 F 不高于抬 F 的上限） 的坏镜像：步 5 那段历史上把 F 抬到 12，而上限是 11。
/// 不是改一处字节改得出来的（见 `raise_the_floor`），所以这一格的「改法」是整份换成那段历史的镜像，入参那份不用。
pub(super) fn known_bad_images_of_the_rollback_floor() -> Vec<(&'static str, Mutation)> {
    vec![(
        "I-7.9",
        Box::new(|image: &mut MemoryPool| {
            let mut pool = pool_before_raising_the_floor("known-bad-floor-above-its-ceiling");
            raise_the_floor(&mut pool, CheckpointTxg(12), CheckpointTxg(12));
            *image = pool.memory_pool();
        }),
    )]
}

/// I-7.9（回退下界 F 不高于抬 F 的上限）：同一段历史（步 5 的脚本，上限 11）上三份镜像——
/// ① **抬到上限 11**（推 txg 15、16 两条带 F 11 的空发布；抬 F 的根是 15，它同实例的前一条 14 带 F 0）：I-7.9 真被评估过且成立，别的一条不红；
/// ② **① 之后再发两版 E0、E**（`overwrite_and_advance_onto_the_genesis_tree_table_slot`）：E 的数据单元落回 mkfs 种的第 0 版树表那一槽
///    （抬 F 生效之后它被合法回收），第 0 代根那几条的树表从此读不出——
///    它们在「抬之前的 F」（0）之上，是算上限的有效根，空不空判不了；把它们都算非空，第 4 新的非空根还是 11，上限的两沿都是 11，
///    I-7.9 照样真被评估过且成立（「不在后来每张镜像上重算」那一半，读不出的根不按空或非空猜）；
/// ③ **抬到 12**（上限 11，入口被骗过，见 `raise_the_floor`）⇒ **只**红 I-7.9，红在「从 0 抬到 12、上限 11」那一格。
#[test]
fn raising_the_floor_above_its_ceiling_reddens_only_the_rollback_floor_invariant() {
    let mut at_the_ceiling = pool_before_raising_the_floor("known-good-floor-at-its-ceiling");
    let floor_before_the_raise = at_the_ceiling.output.root.rollback_floor;
    assert_eq!(floor_before_the_raise, CheckpointTxg(0), "抬之前 F 是 0");
    raise_the_floor(
        &mut at_the_ceiling,
        CheckpointTxg(11),
        floor_before_the_raise,
    );
    assert_eq!(
        at_the_ceiling.output.root.rollback_floor,
        CheckpointTxg(11),
        "最新根带 F = 11"
    );
    let image_at_the_ceiling = at_the_ceiling.memory_pool();
    let verdicts_at_the_ceiling = check_pool_image(&image_at_the_ceiling);
    assert_eq!(
        violated_invariants(&verdicts_at_the_ceiling),
        Vec::<&str>::new(),
        "抬到上限 11：一条都不许红：{verdicts_at_the_ceiling:?}"
    );
    assert_eq!(
        verdict(&image_at_the_ceiling, "I-7.9"),
        InvariantVerdict::Holds,
        "抬到上限 11：I-7.9 要真被评估过且成立"
    );
    overwrite_and_advance_onto_the_genesis_tree_table_slot(&mut at_the_ceiling);
    let image_after_reuse = at_the_ceiling.memory_pool();
    let verdicts_after_reuse = check_pool_image(&image_after_reuse);
    assert_eq!(
        violated_invariants(&verdicts_after_reuse),
        Vec::<&str>::new(),
        "抬到 11 之后再发 E：一条都不许红：{verdicts_after_reuse:?}"
    );
    assert_eq!(
        verdict(&image_after_reuse, "I-7.9"),
        InvariantVerdict::Holds,
        "抬之前的 F 之上有树表读不出的根，上限的两沿仍都是 11：I-7.9 要真被评估过且成立"
    );
    for (invariant, mutation) in known_bad_images_of_the_rollback_floor() {
        let mut image = image_at_the_ceiling.clone();
        mutation(&mut image);
        let verdicts = check_pool_image(&image);
        assert_eq!(
            violated_invariants(&verdicts),
            [invariant],
            "F 抬到 12（上限 11）：判红的该只有 {invariant}：{verdicts:?}"
        );
        let InvariantVerdict::Violated(detail) = verdict(&image, invariant) else {
            panic!("上面判过 {invariant} 红");
        };
        assert!(
            detail.contains("实例 2 txg 15 那条根把回退下界 F 从 0 抬到 12，高于它之前的根算出的抬 F 上限 11")
                && detail.contains("非空有效根 [3, 4, 8, 9, 10, 11, 12, 13, 14]"),
            "红在「抬 F 的根 txg 15、从 0 抬到 12、非空有效根 3、4、8–14、上限 11」那一格：{detail}"
        );
    }
}

/// 系统配置槽里回退下界 F 那 8 字节的偏移：字段表最后一行（D22（单元原子性怎么合成） 已定项 9）。
fn system_configuration_rollback_floor_offset() -> usize {
    usize::try_from(singlefs_format::SYSTEM_CONFIGURATION_BYTES - 8).expect("字段表里的偏移")
}

/// 把 `device` 那块盘两个系统配置槽里的 F 都改成 `rollback_floor`（整槽重封，别的字节不动）。
fn set_the_system_configuration_floor_on_one_device(
    image: &mut MemoryPool,
    device: u32,
    rollback_floor: u64,
) {
    let spacing = u64::from(parameters().geometry.fixed_structure_slot_spacing);
    for slot_offset in [0, spacing] {
        mutate_system_configuration_slot(image, device, slot_offset, |bytes| {
            set_u64(
                bytes,
                system_configuration_rollback_floor_offset(),
                rollback_floor,
            );
        });
    }
}

/// 把 checkpoint_txg 为 `txg` 的那条根带的 F（根记录偏移 130）改成 `rollback_floor`，自证校验和重封。
fn set_the_root_floor(image: &mut MemoryPool, txg: CheckpointTxg, rollback_floor: u64) {
    let (device, offset) = root_slot_of(txg);
    let root_slot_bytes = usize::try_from(parameters().geometry.physical_block_size).expect("512");
    let mut bytes = read(image, device, offset, root_slot_bytes);
    set_u64(&mut bytes, 130, rollback_floor);
    let checksum_offset = singlefs_core::root_record::ROOT_CHECKSUM_OFFSET;
    let digest = wide_checksum_with_field_zeroed(&bytes, root_slot_bytes, checksum_offset);
    bytes[checksum_offset..checksum_offset + 32].copy_from_slice(&digest);
    write(image, device, offset, &bytes);
    copy_each_root_into_the_new_root_segment_of_its_journal_records(image);
}

/// 把 checkpoint_txg 为 `txg` 的那条根所在的根槽改坏一个字节（自证校验和罩整槽）：那条根读不出。
pub(super) fn make_the_root_unreadable(image: &mut MemoryPool, txg: CheckpointTxg) {
    let (device, offset) = root_slot_of(txg);
    let root_slot_bytes = usize::try_from(parameters().geometry.physical_block_size).expect("512");
    let mut bytes = read(image, device, offset, root_slot_bytes);
    bytes[100] ^= 0xff;
    write(image, device, offset, &bytes);
}

/// I-7.12（系统配置 F 不低于同盘根上的 F） 的坏镜像：步 5 那段历史上抬到上限 11（先把 11 写进两盘的系统配置，再推 txg 15 落盘 0、
/// 16 落盘 1），然后把盘 1 两个系统配置槽里的 F 都改回 0——就是「先写系统配置」挪到第一条根之后、崩在两者之间的那一格：
/// 盘 1 上 txg 16 那条根带 11，盘 1 的系统配置里读得出的最大 F 是 0。F 生效值照旧是 11（最新根与盘 0 的系统配置都带 11）。
/// 不是改一处字节就走得到抬 F 之后的历史，所以这一格的「改法」是整份换成那段历史的镜像再改，入参那份不用。
pub(super) fn known_bad_images_of_the_system_configuration_floor() -> Vec<(&'static str, Mutation)>
{
    vec![(
        "I-7.12",
        Box::new(|image: &mut MemoryPool| {
            let mut pool = pool_before_raising_the_floor("known-bad-system-configuration-floor");
            raise_the_floor(&mut pool, CheckpointTxg(11), CheckpointTxg(0));
            let mut raised = pool.memory_pool();
            set_the_system_configuration_floor_on_one_device(&mut raised, 1, 0);
            *image = raised;
        }),
    )]
}

/// I-7.12（系统配置 F 不低于同盘根上的 F）：步 5 那段历史上抬到上限 11 的镜像一条都不红、I-7.12 真被评估过且成立（每块盘的系统配置里
/// 都有先写进去的 11）；[`known_bad_images_of_the_system_configuration_floor`] 那份（盘 1 两槽的 F 改回 0）**只**红 I-7.12，红在盘 1、txg 16 那一格。
#[test]
fn one_device_whose_system_configuration_floor_is_below_a_root_on_it_reddens_only_the_system_configuration_floor_invariant(
) {
    let mut at_the_ceiling = pool_before_raising_the_floor("known-good-system-configuration-floor");
    raise_the_floor(&mut at_the_ceiling, CheckpointTxg(11), CheckpointTxg(0));
    let clean = at_the_ceiling.memory_pool();
    let clean_verdicts = check_pool_image(&clean);
    assert_eq!(
        violated_invariants(&clean_verdicts),
        Vec::<&str>::new(),
        "抬到上限 11：一条都不许红：{clean_verdicts:?}"
    );
    assert_eq!(
        verdict(&clean, "I-7.12"),
        InvariantVerdict::Holds,
        "抬到上限 11：I-7.12 要真被评估过且成立"
    );
    for (invariant, mutation) in known_bad_images_of_the_system_configuration_floor() {
        let mut image = clean.clone();
        mutation(&mut image);
        let verdicts = check_pool_image(&image);
        assert_eq!(
            violated_invariants(&verdicts),
            [invariant],
            "盘 1 两槽的 F 改回 0：判红的该只有 {invariant}：{verdicts:?}"
        );
        let InvariantVerdict::Violated(detail) = verdict(&image, invariant) else {
            panic!("上面判过 {invariant} 红");
        };
        assert!(
            detail.contains("盘 1：两槽里自证过的系统配置带的 F 最大 0")
                && detail.contains("txg 16 那条根带的 F 11"),
            "红在盘 1、txg 16 那条带 11 的根那一格：{detail}"
        );
    }
}

/// 正常卸载那段历史：新池新建文件之后同一个实例里覆盖写三次（txg 4–6），卸载把 F 抬到 6，推 txg 7（落盘 1）、8（落盘 0）两条带卸载记号的根。
/// 这段历史上准入抬 F 的上限是 3（第 4 新的非空根是 A 的 3）。
fn image_after_the_normal_unmount(tag: &str) -> MemoryPool {
    let mut pool = build_pool(tag);
    for seed in [3usize, 5, 7] {
        overwrite_and_advance(
            &mut pool,
            &content_seeded(2600 + seed, seed),
            InstanceGeneration(1),
        );
    }
    let mut current = PoolVersion::WithFile(Arc::new(pool.output.clone()));
    let devices = pool.devices.as_mut().expect("镜像还开着");
    let Unmounted::FloorRaisedToTheCurrentVersion(unmounted) = unmount(
        &parameters(),
        devices,
        &mut pool.allocator,
        &mut current,
        ShadowLedger::On,
    )
    .expect("正常卸载") else {
        panic!("现行那一版带文件：卸载抬 F");
    };
    assert_eq!(unmounted.rollback_floor, CheckpointTxg(6));
    pool.memory_pool()
}

/// I-7.9（回退下界 F 不高于抬 F 的上限） 带卸载记号那一支的坏镜像：卸载之后把 txg 7 那条带记号的根（抬 F 的那一条，盘 1）带的 F 改成 7，
/// 高于根环里 txg 比它小的最新那条根的 txg 6；盘 1 两个系统配置槽的 F 一起改成 7，I-7.12 不先红。
pub(super) fn known_bad_images_of_the_unmount_floor() -> Vec<(&'static str, Mutation)> {
    vec![(
        "I-7.9",
        Box::new(|image: &mut MemoryPool| {
            *image =
                image_after_the_normal_unmount("known-bad-unmount-floor-above-the-previous-root");
            set_the_root_floor(image, CheckpointTxg(7), 7);
            set_the_system_configuration_floor_on_one_device(image, 1, 7);
        }),
    )]
}

/// I-7.9（回退下界 F 不高于抬 F 的上限） 按卸载记号分两支（D22（单元原子性怎么合成） 已定项 7）：
/// ① 正常卸载之后的镜像（txg 7 那条带记号的根把 F 从 0 抬到 6）一条都不红，I-7.9 真被评估过且成立——按带记号那一支判 6 ≤ 6；
///    按准入那一支判它就红了（这段历史上准入上限是 3）；
/// ② [`known_bad_images_of_the_unmount_floor`] 那份（F 改成 7）**只**红 I-7.9，红在「带卸载记号的根从 0 抬到 7、前一条根 txg 6」那一格。
/// 不带记号那一支的坏镜像是 [`known_bad_images_of_the_rollback_floor`]（准入入口抬到 12、上限 11）。
#[test]
fn marked_root_raising_the_floor_above_the_previous_root_reddens_only_the_rollback_floor_invariant()
{
    let clean = image_after_the_normal_unmount("known-good-unmount-floor");
    let clean_verdicts = check_pool_image(&clean);
    assert_eq!(
        violated_invariants(&clean_verdicts),
        Vec::<&str>::new(),
        "正常卸载之后：一条都不许红：{clean_verdicts:?}"
    );
    assert_eq!(
        verdict(&clean, "I-7.9"),
        InvariantVerdict::Holds,
        "正常卸载之后：I-7.9 按带记号那一支真被评估过且成立"
    );
    for (invariant, mutation) in known_bad_images_of_the_unmount_floor() {
        let mut image = clean.clone();
        mutation(&mut image);
        let verdicts = check_pool_image(&image);
        assert_eq!(
            violated_invariants(&verdicts),
            [invariant],
            "带记号的根把 F 抬到 7（前一条根 txg 6）：判红的该只有 {invariant}：{verdicts:?}"
        );
        let InvariantVerdict::Violated(detail) = verdict(&image, invariant) else {
            panic!("上面判过 {invariant} 红");
        };
        assert!(
            detail.contains(
                "实例 1 txg 7 那条带卸载记号的根把回退下界 F 从 0 抬到 7，高于根环里 txg 比它小的最新那条根的 txg 6"
            ),
            "红在带记号那一支：{detail}"
        );
    }
}

/// C556（checker 与崩溃点重放不读系统配置里的 F）：步 5 那段历史上抬到 11、再发两版 E0、E（txg 17、18，E 的数据单元落回 mkfs 种的第 0 版树表那一槽），
/// 然后把四条带 F 11 的根（txg 15–18）都改坏——根环里读得出的根带的 F 全是 0，两块盘的系统配置里还是 11（抬 F 先写进去的）。
/// checker 的回退候选集下界取 F 生效值 max(最新根带的, 系统配置里的) = 11：被复用的那一槽只有 txg 11 之下的根引用，它们不在候选集里，
/// 一条都不红；I-3.1 照 C697（已分配统计在短环加大文件上误红） 的三判成立：最新根 txg 14 带 F 0、F 生效值 11，三判不按 F 取遍历集合的
/// 并集来比，「最新根带的 F 低于 F 生效值时不判」那条例外随三判拿掉（改三判之前这里报不适用）。
/// 判别力自证：同一张镜像再把四个系统配置槽的 F 都改回 0（等于只读根上的 F），txg 11 之下的根回到候选集、走到被复用的那一槽，
/// 按回退候选集判的那几条红。
#[test]
fn every_root_carrying_the_raised_floor_unreadable_keeps_the_candidate_invariants_green_because_the_checker_reads_the_system_configuration_floor(
) {
    let mut pool = pool_before_raising_the_floor("c556-roots-carrying-the-floor-unreadable");
    raise_the_floor(&mut pool, CheckpointTxg(11), CheckpointTxg(0));
    overwrite_and_advance_onto_the_genesis_tree_table_slot(&mut pool);
    assert_eq!(pool.output.root.checkpoint_txg, CheckpointTxg(18));
    let mut image = pool.memory_pool();
    for txg in [15, 16, 17, 18] {
        make_the_root_unreadable(&mut image, CheckpointTxg(txg));
    }
    let verdicts = check_pool_image(&image);
    assert_eq!(
        violated_invariants(&verdicts),
        Vec::<&str>::new(),
        "带 F 11 的根全读不出、系统配置里的 11 还在：一条都不许红：{verdicts:?}"
    );
    assert_eq!(
        verdict(&image, "I-3.1"),
        InvariantVerdict::Holds,
        "最新根 txg 14 带 F 0、F 生效值 11：I-3.1 照三判真被评估过且成立，不退回不判"
    );
    let mut floor_only_on_the_roots = image.clone();
    for device in DEVICES {
        set_the_system_configuration_floor_on_one_device(&mut floor_only_on_the_roots, device, 0);
    }
    let verdicts_reading_only_the_roots = check_pool_image(&floor_only_on_the_roots);
    assert_eq!(
        violated_invariants(&verdicts_reading_only_the_roots),
        ["I-2.1", "I-3.10", "I-4.8", "I-7.4"],
        "系统配置里的 F 也改回 0（等于只读根上的 F）：txg 11 之下的根回到候选集、走到被复用的那两处（实例表那两槽的分配记录分配代 0、\
         单元头诞生代号 17，I-3.10 也红）：{verdicts_reading_only_the_roots:?}"
    );
}
