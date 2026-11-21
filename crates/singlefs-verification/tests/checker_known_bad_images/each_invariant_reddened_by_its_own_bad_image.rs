//! 每条不变量配的坏镜像只染红它自己：总表与干净镜像判全绿，以及系统配置、自己的设备号、写者身份、跨度、出生身份、合并与回链各条。

use super::*;

/// C701（别的判法静默跳过读不出的部分），一律报部分没判：发布 B 之后的干净镜像上，A 那一版的树表（`TREE_TABLE`，
/// 两盘同槽）两份都把最后一个字节翻一位、不补引用链：A 的根指着的树表两份都对不上整单元校验和，A 那一版走读记失败、引用没收全，树表条目也读不到。
/// I-3.5（引用区间的精确性） 的 (A, B) 那一对与 I-9.14（树表条目的诞生 txg 跨根不变） 里 A 那一份证据都读不出：两条都报部分没判、点名没判的是什么，
/// 不静默跳过。
/// 判别力（`crates/mutations.tsv` 里点名这条的行）：I-3.5 那一对走读失败照旧静默跳过时不报部分没判，这条红；I-9.14 那一行同。
#[test]
fn candidate_tree_table_reading_back_from_neither_copy_leaves_the_reference_interval_and_tree_table_birth_invariants_partially_judged(
) {
    let mut image =
        image_after_the_overwrite("known-bad-candidate-tree-table-reads-back-from-neither-copy");
    for device in DEVICES {
        let mut bytes = read(&image, device, TREE_TABLE * SLOT, 16384);
        let last_byte = bytes.len() - 1;
        bytes[last_byte] ^= 1;
        write(&mut image, device, TREE_TABLE * SLOT, &bytes);
    }
    match verdict(&image, REFERENCE_INTERVALS_ARE_EXACT) {
        InvariantVerdict::PartiallyJudged(unjudged) => assert!(
            unjudged.contains("有一版走读记过失败、引用没收全，这一对不判"),
            "I-3.5 的部分没判点名走读失败的那一对：{unjudged}"
        ),
        other_verdict @ (InvariantVerdict::Holds
        | InvariantVerdict::Violated(_)
        | InvariantVerdict::NotApplicable(_)) => {
            panic!("A 那一版的树表读不出：I-3.5 要报部分没判，实际 {other_verdict:?}")
        }
    }
    match verdict(&image, "I-9.14") {
        InvariantVerdict::PartiallyJudged(unjudged) => assert!(
            unjudged.contains("候选根 txg 3：树表读不出"),
            "I-9.14 的部分没判点名 A 那条候选根的树表读不出：{unjudged}"
        ),
        other_verdict @ (InvariantVerdict::Holds
        | InvariantVerdict::Violated(_)
        | InvariantVerdict::NotApplicable(_)) => {
            panic!("A 那一版的树表读不出：I-9.14 要报部分没判，实际 {other_verdict:?}")
        }
    }
}

/// 根记录里树表指针的起点（根记录偏移 36 起 86 字节，D22（单元原子性怎么合成） 已定项 7 的字段表）。
const ROOT_RECORD_TREE_TABLE_POINTER_OFFSET: usize = 36;

/// C748 那一份坏镜像里另放一份第 0 版树表的槽（盘 1）：写完新池新建文件之后单元区只用到 `UNITS` 那几个槽，这一槽空着。
const SLOT_OF_ANOTHER_COPY_OF_THE_GENESIS_TREE_TABLE: u64 = 50_300;

/// C748（最终折叠表漏判引用区间的落点）：照攻方例子（`research/prompts/m3-batch35-36-code-r1-main-verification.md` 第二节「K1 / K2 V2 H-S1-fold」
/// 那一行与第三节第 3 条，攻方第六节）造的真盘面。写完新池新建文件的镜像上，A 之前那几版都指着 mkfs 种的第 0 版树表（两盘同槽：落点 a 在盘 0、
/// b 在盘 1）。取其中 txg 最大的两版 R⁻、R：R⁻ 那条根的树表指针第 0 条位置条目的校验和坏（读回 b 那一份，I-2.1 红，折叠表把 b 记成它自己），
/// R 那条根的树表指针改指 (b, c)，c 是盘 1 上另一槽里同一张树表的一份（位置条目同盘，I-2.5 也红）。
/// 按落点比：c 是 R 引用、R⁻ 不引用的落点，取诞生代号时折成 b 那个单元（第 0 版树表，诞生代号 0），不大于 R⁻.txg、也不在用户可见树里，I-3.5 红、点名 c。
/// 按全部版本走完之后的折叠表折成单元再比的话，c 折进 b、R⁻ 也引用 b，c 不判，I-3.5 成立——漏判。
/// 判别力（`crates/mutations.tsv` 里点名这条的行）：比的时候改回按折叠表折成的单元认，这条红。
#[test]
fn placement_the_final_fold_table_folds_into_a_unit_of_the_previous_version_still_reddens_the_appearance_rule(
) {
    let mut image = build_pool("known-bad-c748-final-fold-table-placement").memory_pool();
    let first_location_slot_offset =
        ROOT_RECORD_TREE_TABLE_POINTER_OFFSET + NODE_POINTER_FIRST_LOCATION_OFFSET + 4;
    let mut versions_on_the_genesis_tree_table: Vec<u64> = root_slot_offsets_of_both_devices()
        .into_iter()
        .map(|(device, offset)| read(&image, device, offset, 512))
        .filter(|root| &root[..4] == b"SFSR")
        .filter(|root| {
            let mut slot_bytes = [0u8; 8];
            slot_bytes[..6]
                .copy_from_slice(&root[first_location_slot_offset..first_location_slot_offset + 6]);
            u64::from_le_bytes(slot_bytes) == GENESIS_TREE_TABLE
        })
        .map(|root| get_u64(&root, ROOT_RECORD_CHECKPOINT_TXG_OFFSET))
        .collect();
    versions_on_the_genesis_tree_table.sort_unstable();
    let [.., previous_txg, version_txg] = versions_on_the_genesis_tree_table.as_slice() else {
        panic!("A 之前至少两版指着第 0 版树表：{versions_on_the_genesis_tree_table:?}")
    };
    let (previous_txg, version_txg) = (*previous_txg, *version_txg);
    assert_eq!(
        version_txg,
        previous_txg + 1,
        "R⁻、R 在回退候选集里相邻、中间没有洞"
    );
    let genesis_tree_table = read(&image, 1, GENESIS_TREE_TABLE * SLOT, 16384);
    write(
        &mut image,
        1,
        SLOT_OF_ANOTHER_COPY_OF_THE_GENESIS_TREE_TABLE * SLOT,
        &genesis_tree_table,
    );
    let first_location = ROOT_RECORD_TREE_TABLE_POINTER_OFFSET + NODE_POINTER_FIRST_LOCATION_OFFSET;
    let second_location = first_location + NODE_POINTER_LOCATION_BYTES;
    rewrite_the_root_slot_of(&mut image, CheckpointTxg(previous_txg), |root| {
        let checksum = first_location + NODE_POINTER_LOCATION_CHECKSUM_OFFSET;
        for byte in &mut root[checksum..checksum + 4] {
            *byte ^= 0x5a;
        }
    });
    // R 那条根直接改根槽、重封自证校验和，不抄进记录的新根段：两条位置条目同盘，实现的指针解析按 I-2.5 拒它（抄新根段那一步经实现解析），
    // 池级 checker 只判 I-2.5 红、照样跟着走。
    let (device_of_the_version, offset_of_the_version) = root_slot_of(CheckpointTxg(version_txg));
    let mut root = read(&image, device_of_the_version, offset_of_the_version, 512);
    assert_eq!(get_u64(&root, 28), version_txg, "那一槽是 R 的根");
    let location_on_device_one =
        root[second_location..second_location + NODE_POINTER_LOCATION_BYTES].to_vec();
    root[first_location..first_location + NODE_POINTER_LOCATION_BYTES]
        .copy_from_slice(&location_on_device_one);
    root[second_location + 4..second_location + 10]
        .copy_from_slice(&SLOT_OF_ANOTHER_COPY_OF_THE_GENESIS_TREE_TABLE.to_le_bytes()[..6]);
    let digest = wide_checksum_with_field_zeroed(&root, 512, 138);
    root[138..170].copy_from_slice(&digest);
    write(
        &mut image,
        device_of_the_version,
        offset_of_the_version,
        &root,
    );
    let verdicts = check_pool_image(&image);
    let set = violated_invariant_set(&verdicts);
    assert!(
        set.contains("I-2.1"),
        "R⁻ 那条根读树表时第 0 条那一份对不上：{verdicts:?}"
    );
    match verdict(&image, REFERENCE_INTERVALS_ARE_EXACT) {
        InvariantVerdict::Violated(detail) => assert!(
            detail.contains(&format!("txg {previous_txg}) → (实例 "))
                && detail.contains(&format!("、txg {version_txg})（"))
                && detail.contains(&format!(
                "落点盘 1 槽 {SLOT_OF_ANOTHER_COPY_OF_THE_GENESIS_TREE_TABLE}"
            )),
            "I-3.5 红在 (R⁻, R) 那一对、点名 c：{detail}"
        ),
        other_verdict @ (InvariantVerdict::Holds
        | InvariantVerdict::NotApplicable(_)
        | InvariantVerdict::PartiallyJudged(_)) => panic!(
            "c 是 R 引用、R⁻ 不引用的落点，诞生代号 0 不大于 R⁻.txg：I-3.5 要红，实际 {other_verdict:?}"
        ),
    }
}

/// C374 那两条：发布 B 之后的干净镜像上每条不变量都成立，两份坏镜像各自**只**红在自己那一条上
/// （别的不变量跟着红就说明镜像改宽了，判别力算不到这一条头上）。
#[test]
fn each_release_generation_and_tree_table_birth_bad_image_reddens_only_its_own_invariant_after_the_overwrite(
) {
    let clean = image_after_the_overwrite("known-bad-after-overwrite");
    for (invariant, found) in check_pool_image(&clean) {
        assert_eq!(
            found,
            if invariant == INSTANCE_TABLE_ROWS_PUBLISHED_BEFORE_THE_ROOT_REFERENCING_THEM {
                // 只有实例 1，实例表一行都没有（回退那几份有行，那里 I-3.12 成立）。
                InvariantVerdict::NotApplicable(INSTANCE_TABLE_ROWS_NOT_APPLICABLE)
            } else if invariant
                == INSTANCE_TABLE_ROW_HIGH_WATER_IS_ONE_A_RECOVERY_COULD_HAVE_APPLIED
            {
                // 同一个理由（回退那几份有行，那里 I-3.19 成立）。
                InvariantVerdict::NotApplicable(NO_INSTANCE_TABLE_ROW_UNDER_THE_NEWEST_ROOT)
            } else {
                expected_verdict_with_one_record_per_transaction(invariant)
            },
            "发布 B 之后的干净镜像上 {invariant} 的判定"
        );
    }
    for (invariant, mutation) in known_bad_images_after_the_overwrite() {
        let mut image = clean.clone();
        mutation(&mut image);
        let verdicts = check_pool_image(&image);
        let violated: Vec<&str> = verdicts
            .iter()
            .filter(|(_, verdict)| matches!(verdict, InvariantVerdict::Violated(_)))
            .map(|(name, _)| *name)
            .collect();
        // 同一条线上两条根记得不同，必然也是全部可读根里记得不同：I-9.14 的坏镜像连带 I-9.28 红，别的一条不红。
        // I-8.12 的坏镜像把实例 1 一条记录的 txg 改成 0：实例 1 的第一个 txg 落到 0，不高于 mkfs 种的实例 0 第 0 代根，I-8.14 跟着红。
        let expected: Vec<&str> = if invariant == "I-9.14" {
            vec![
                "I-9.14",
                TREE_IDENTIFIER_BIRTH_TXG_AGREES_ACROSS_EVERY_READABLE_ROOT,
            ]
        } else if invariant == JOURNAL_RECORD_TXG_NOT_DECREASING_WITHIN_AN_INSTANCE {
            vec![
                JOURNAL_RECORD_TXG_NOT_DECREASING_WITHIN_AN_INSTANCE,
                FIRST_TXG_OF_EACH_INSTANCE_IS_ABOVE_EVERY_ROOT_AND_RECORD_OF_THE_OLDER_INSTANCES,
            ]
        } else {
            vec![invariant]
        };
        assert_eq!(
            violated, expected,
            "判红的该只有 {expected:?}：{verdicts:?}"
        );
    }
}

/// I-7.3（环健康性） 的两支各跑一份，都建在 **mkfs 刚写完**那份镜像上——两份镜像只差三条根的 `checkpoint_txg`
/// （0 → 1），别的字节逐字相同：
/// ① 例外那一支的阳性对照：S 全部是第 0 代 ⇒ 判绿，而且要真被评估过（不是「不适用」）；
/// ② 代号一起改成 1 之后 S 里没有更早代号的记录、例外又够不着 ⇒ 判红，而且**除 I-7.3 之外每一条的判定与 ① 那一份逐项相同**。
/// 两份的唯一差别就是那个代号 ⇒ 这一条分得出差别，判别力算得到它头上（`test-discipline.md`「检查本身也可能是错的」）。
///
/// 第二段是写完新池新建文件那份镜像上的形态：S 只剩代号最大的那一条（除 txg 3 之外的根槽自证校验和都改坏）。
/// 它同时红 I-3.1（已分配统计对得上）——mkfs 种的树表单元（50178）只有那条第 0 代根还引用着，去掉它遍历就少算 16384，
/// 两条一起红是这份镜像的结构决定的，照实钉下来。
#[test]
fn the_root_ring_health_invariant_reddens_when_the_previous_generation_is_gone_but_not_right_after_mkfs(
) {
    let built = build_pool("known-bad-root-ring-health");
    let freshly_formatted = built.memory_pool_after_mkfs();
    let generations_right_after_mkfs: Vec<u64> = root_slot_offsets_of_both_devices()
        .into_iter()
        .map(|(device, offset)| read(&freshly_formatted, device, offset, 512))
        .filter(|bytes| bytes[..4] == *b"SFSR")
        .map(|bytes| get_u64(&bytes, 28))
        .collect();
    assert_eq!(
        generations_right_after_mkfs,
        [0, 0, 0],
        "mkfs 刚写完：三个区域的槽 0 各一条第 0 代根，例外那一支罩的就是这个态"
    );
    let verdicts_right_after_mkfs = check_pool_image(&freshly_formatted);
    assert_eq!(
        verdict(&freshly_formatted, "I-7.3"),
        InvariantVerdict::Holds,
        "S 全部是第 0 代 ⇒ 例外那一支判绿，而且要真被评估过"
    );
    for (invariant, mutation) in known_bad_images_of_the_root_ring_health() {
        let mut image = freshly_formatted.clone();
        mutation(&mut image);
        let verdicts = check_pool_image(&image);
        let violated: Vec<&str> = verdicts
            .iter()
            .filter(|(_, verdict)| matches!(verdict, InvariantVerdict::Violated(_)))
            .map(|(name, _)| *name)
            .collect();
        // 三个区域槽 0 都坐着 txg 1 的根，而 txg 1 只许住区域 1 槽 0：区域 0、2 那两条照 I-7.17（根槽位置等于 txg 算出的位置）
        // 必然一起红，它是这份镜像造法自带的，不是 I-7.3 搭车；I-7.3 的判别力仍由「两份只差那个代号」钉住（I-7.17 之外逐项相同）。
        let red_with_it = [invariant, ROOT_RECORDS_LIVE_IN_THE_SLOT_THEIR_TXG_NAMES];
        assert_eq!(
            violated, red_with_it,
            "判红的该只有 {invariant} 与 txg 1 住错区域的 I-7.17：{verdicts:?}"
        );
        let differences: Vec<(&str, &InvariantVerdict, &InvariantVerdict)> = verdicts
            .iter()
            .zip(&verdicts_right_after_mkfs)
            .filter(|((name, found), (_, before))| !red_with_it.contains(name) && found != before)
            .map(|((name, found), (_, before))| (*name, before, found))
            .collect();
        assert!(
            differences.is_empty(),
            "除 {invariant} 之外每一条的判定都该与 mkfs 刚写完那一份相同：{differences:?}"
        );
    }
    // 写完新池新建文件那份镜像上的同一个形态：S 只剩 txg 3 那一条。
    let after_the_new_pool_file_creation = built.memory_pool();
    assert_eq!(
        verdict(&after_the_new_pool_file_creation, "I-7.3"),
        InvariantVerdict::Holds,
        "写完新池新建文件的干净镜像上 I-7.3 要真被评估过且成立"
    );
    let mut only_the_newest_root_left = after_the_new_pool_file_creation.clone();
    let mut corrupted_root_slots = 0;
    for (device, offset) in root_slot_offsets_of_both_devices() {
        if (device, offset) == NEW_POOL_FILE_CREATION_ROOT_SLOT {
            continue;
        }
        let mut bytes = read(&only_the_newest_root_left, device, offset, 512);
        if &bytes[..4] != b"SFSR" {
            continue;
        }
        bytes[138] ^= 0xFF;
        write(&mut only_the_newest_root_left, device, offset, &bytes);
        corrupted_root_slots += 1;
    }
    assert_eq!(
        corrupted_root_slots, 3,
        "根环里除 txg 3 那条之外自证过的根：mkfs 第 0 代 1 条 + 暖机 2 条"
    );
    let verdicts = check_pool_image(&only_the_newest_root_left);
    let violated: Vec<&str> = verdicts
        .iter()
        .filter(|(_, verdict)| matches!(verdict, InvariantVerdict::Violated(_)))
        .map(|(name, _)| *name)
        .collect();
    // 根记录带回收门槛 t 之后（C773（读坏与全零根槽的回收门槛钉只在内存里）），txg 3 那条根写的 t = 0（那一刻第 0 代根还在环里）：
    // checker 照镜像现算 max(F = 0, 环里最旧有效根 3) = 3，I-7.18 红；I-3.1 照盘上的 t 把释放代落在 (0, 3] 的已释放槽（mkfs 种的树表单元）
    // 当钉带出来的那一截并进来，不再红——「上一代被直接覆盖」从记账那一条挪到门槛那一条上看出来。
    assert_eq!(
        violated,
        ["I-7.3", "I-7.18"],
        "S 只剩 txg 3 那一条：I-7.3 红，I-7.18 跟着红（第 0 代根一走，现算的门槛高过 txg 3 那条根写的 t）：{verdicts:?}"
    );
}

/// I-1.2（块头写序已发布） 与 I-4.2（无被引用未提交块） 那两份坏镜像**只**红在自己那一条上——两条判的是同一批
/// 被引用单元、同一份输入（头里的诞生代号、写序、出生序号），分得开它们的就是这两份镜像：
/// ① I-1.2 那一份只动头里的出生序号（不是谓词的输入）⇒ 指针对不上而「已发布」照旧成立；
/// ② I-4.2 那一份把头与指针的诞生代号一起抬到最新根的 txg 之上 ⇒ 指针仍然对得上而「已发布」不成立。
/// 一份镜像同时红两条，就说明两条里有一条是搭车判的，判别力算不到它头上。
#[test]
fn the_birth_identity_bad_images_redden_only_their_own_invariant() {
    let clean = build_pool("known-bad-birth-identity").memory_pool();
    let judged_here = ["I-1.2", "I-4.2"];
    let cases: Vec<(&'static str, Mutation)> = known_bad_images(&clean)
        .into_iter()
        .filter(|(invariant, _)| judged_here.contains(invariant))
        .collect();
    assert_eq!(cases.len(), judged_here.len(), "两条各有一份坏镜像");
    for (invariant, mutation) in cases {
        let mut image = clean.clone();
        mutation(&mut image);
        let verdicts = check_pool_image(&image);
        let violated: Vec<&str> = verdicts
            .iter()
            .filter(|(_, verdict)| matches!(verdict, InvariantVerdict::Violated(_)))
            .map(|(name, _)| *name)
            .collect();
        assert_eq!(
            violated,
            [invariant],
            "判红的该只有 {invariant}：{verdicts:?}"
        );
    }
}

/// I-1.8（归并后版本全序） 与 I-8.6（反向链算法） 那两份坏镜像**只**红在自己那一条上：
/// 别的不变量跟着红就说明镜像改宽了，判别力算不到这一条头上。
/// I-1.8 那一份尤其要看 I-2.1（校验和与内容匹配）：盘 1 那条位置条目跟着补过，两份各自与自己那条位置条目对得上，
/// 分得开「两个副本载荷不同」的只有 I-1.8 这一条。
#[test]
fn the_merge_and_the_back_chain_bad_images_redden_only_their_own_invariant() {
    let clean = build_pool("known-bad-merge-and-back-chain").memory_pool();
    let judged_here = ["I-1.8", "I-8.6"];
    let cases: Vec<(&'static str, Mutation)> = known_bad_images(&clean)
        .into_iter()
        .filter(|(invariant, _)| judged_here.contains(invariant))
        .collect();
    assert_eq!(cases.len(), judged_here.len(), "两条各有一份坏镜像");
    for (invariant, mutation) in cases {
        let mut image = clean.clone();
        mutation(&mut image);
        let verdicts = check_pool_image(&image);
        let violated: Vec<&str> = verdicts
            .iter()
            .filter(|(_, verdict)| matches!(verdict, InvariantVerdict::Violated(_)))
            .map(|(name, _)| *name)
            .collect();
        assert_eq!(
            violated,
            [invariant],
            "判红的该只有 {invariant}：{verdicts:?}"
        );
    }
}

/// I-7.13（系统配置池级字段在读者收的范围里） 那份坏镜像（盘 1 槽 0 加密类型 1）**只**红 I-7.13，红在盘 1 偏移 0 那一槽；
/// 别的每一条都报不适用（实现整池拒绝挂载，checker 不拿别的三槽照判、不替这个池作保）。
#[test]
fn a_system_configuration_slot_whose_encryption_type_is_on_reddens_only_its_own_invariant_and_every_other_is_not_applicable(
) {
    let clean = build_pool("known-bad-refused-system-configuration-value").memory_pool();
    let cases: Vec<(&'static str, Mutation)> = known_bad_images(&clean)
        .into_iter()
        .filter(|(invariant, _)| {
            *invariant == SYSTEM_CONFIGURATION_CARRIES_ONLY_VALUES_THE_READER_ACCEPTS
        })
        .collect();
    assert_eq!(
        cases.len(),
        1,
        "写完新池新建文件那份镜像上 I-7.13 有一份坏镜像"
    );
    for (invariant, mutation) in cases {
        let mut image = clean.clone();
        mutation(&mut image);
        let verdicts = check_pool_image(&image);
        assert!(
            verdicts.iter().any(|(judged, _)| *judged == invariant),
            "{invariant} 在判定清单里：{verdicts:?}"
        );
        for (judged, found) in &verdicts {
            if *judged == invariant {
                match found {
                    InvariantVerdict::Violated(detail) => assert!(
                        detail.contains(&format!(
                            "盘 {DEVICE_OF_THE_REFUSED_SYSTEM_CONFIGURATION_SLOT} 偏移 {OFFSET_OF_THE_REFUSED_SYSTEM_CONFIGURATION_SLOT}"
                        )),
                        "违例要点名被改的那一槽：{detail}"
                    ),
                    other @ (InvariantVerdict::Holds | InvariantVerdict::NotApplicable(_) | InvariantVerdict::PartiallyJudged(_)) => {
                        panic!("加密类型 1 的槽自证过：实现整池拒绝挂载，{invariant} 要判违例，得到 {other:?}")
                    }
                }
            } else {
                assert!(
                    matches!(found, InvariantVerdict::NotApplicable(_)),
                    "这个池挂不上，{judged} 要报不适用，得到 {found:?}"
                );
            }
        }
    }
}

/// I-7.13（系统配置池级字段在读者收的范围里） 的几何那五格（[`known_bad_images_of_the_system_configuration_geometry`]）各**只**红 I-7.13，
/// 红在盘 1 偏移 0 那一槽、说明里带该报的那个 `Verdict` 成员；别的每一条都报不适用（实现整池拒绝挂载，checker 不替这个池作保）。
/// checker 不判 417 = 环长现算、不判段边界、按字段读 325 与 371 时：A、B、C、D 四格 I-7.13 不红（A、D 红的是 I-5.2，
/// C 红的是 I-7.1），E 格红的是槽距那个成员。
#[test]
fn each_system_configuration_geometry_field_the_reader_refuses_reddens_only_its_own_invariant_naming_the_refused_field(
) {
    let clean = build_pool("known-bad-refused-system-configuration-geometry").memory_pool();
    let cases = known_bad_images_of_the_system_configuration_geometry();
    assert_eq!(cases.len(), 5, "实审 Y4 的 A、B、C、D、E 五格");
    for (refused_member, mutation) in cases {
        let mut image = clean.clone();
        mutation(&mut image);
        let verdicts = check_pool_image(&image);
        assert!(
            verdicts.iter().any(|(judged, _)| *judged
                == SYSTEM_CONFIGURATION_CARRIES_ONLY_VALUES_THE_READER_ACCEPTS),
            "I-7.13 在判定清单里：{verdicts:?}"
        );
        for (judged, found) in &verdicts {
            if *judged == SYSTEM_CONFIGURATION_CARRIES_ONLY_VALUES_THE_READER_ACCEPTS {
                match found {
                    InvariantVerdict::Violated(detail) => {
                        assert!(
                            detail.contains(&format!(
                                "盘 {DEVICE_OF_THE_REFUSED_SYSTEM_CONFIGURATION_SLOT} 偏移 {OFFSET_OF_THE_REFUSED_SYSTEM_CONFIGURATION_SLOT}"
                            )),
                            "{refused_member}：违例要点名被改的那一槽：{detail}"
                        );
                        assert!(
                            detail.contains(refused_member),
                            "{refused_member}：违例说明里要是这个成员：{detail}"
                        );
                    }
                    other @ (InvariantVerdict::Holds
                    | InvariantVerdict::NotApplicable(_)
                    | InvariantVerdict::PartiallyJudged(_)) => {
                        panic!("{refused_member}：这一槽自证过而带这一版读者不收的值，I-7.13 要判违例，得到 {other:?}")
                    }
                }
            } else {
                assert!(
                    matches!(found, InvariantVerdict::NotApplicable(_)),
                    "{refused_member}：这个池挂不上，{judged} 要报不适用，得到 {found:?}"
                );
            }
        }
    }
}

/// I-7.14（本盘设备号等于盘在池里的身份，暂立） 那份坏镜像（两块盘四个系统配置槽的本盘设备号互换）**只**红 I-7.14，
/// 说明里点名盘 0 上自述 1 的那一槽；别的每一条与干净镜像上的判定逐条相同。checker 若读本盘设备号而不拿它比盘的身份，
/// 这份镜像上 0 条违例（实审 Y2-b 越格线索）。
#[test]
fn devices_whose_own_device_numbers_are_swapped_redden_only_the_own_device_number_invariant() {
    let clean = build_pool("known-bad-swapped-own-device-numbers").memory_pool();
    let clean_verdicts = check_pool_image(&clean);
    let mut image = clean.clone();
    swap_the_own_device_numbers_of_the_two_devices(&mut image);
    let verdicts = check_pool_image(&image);
    assert_eq!(
        violated_invariants(&verdicts),
        vec![SYSTEM_CONFIGURATION_OWN_DEVICE_NUMBER_IS_THE_DEVICE_IDENTITY],
        "只红 I-7.14：{verdicts:?}"
    );
    let detail = match verdict(
        &image,
        SYSTEM_CONFIGURATION_OWN_DEVICE_NUMBER_IS_THE_DEVICE_IDENTITY,
    ) {
        InvariantVerdict::Violated(detail) => detail,
        other @ (InvariantVerdict::Holds
        | InvariantVerdict::NotApplicable(_)
        | InvariantVerdict::PartiallyJudged(_)) => {
            panic!("本盘设备号互换，I-7.14 要判违例，得到 {other:?}")
        }
    };
    assert!(
        detail.contains("盘 0 上") && detail.contains("本盘设备号 1"),
        "第一处违例是盘 0 上自述 1 的那一槽：{detail}"
    );
    assert_eq!(
        verdicts_that_differ_outside(
            &verdicts,
            &clean_verdicts,
            &[SYSTEM_CONFIGURATION_OWN_DEVICE_NUMBER_IS_THE_DEVICE_IDENTITY]
        ),
        Vec::new(),
        "除 I-7.14 之外每一条与干净镜像上的判定逐条相同"
    );
}

/// I-1.5（系统配置记管道身份，C868（系统配置管道身份没有 checker 判定））的坏镜像：盘 1 被择的那一槽实现标识清零、重封，**只**红 I-1.5，
/// 说明点名盘 1；别的每一条与干净镜像上的判定逐条相同。checker 若不读写入者身份，这份镜像上 0 条违例。
#[test]
fn a_chosen_system_configuration_whose_writer_implementation_identifier_is_all_zero_reddens_only_the_writer_identity_invariant(
) {
    let clean = build_pool("known-bad-writer-identity-cleared").memory_pool();
    let clean_verdicts = check_pool_image(&clean);
    assert_eq!(
        verdict(&clean, SYSTEM_CONFIGURATION_RECORDS_THE_WRITER_IDENTITY),
        InvariantVerdict::Holds,
        "干净镜像上 I-1.5 真被评估过且成立"
    );
    let mut image = clean.clone();
    clear_the_writer_implementation_identifier_of_the_chosen_slot_of_device_one(&mut image);
    let verdicts = check_pool_image(&image);
    assert_eq!(
        violated_invariants(&verdicts),
        vec![SYSTEM_CONFIGURATION_RECORDS_THE_WRITER_IDENTITY],
        "只红 I-1.5：{verdicts:?}"
    );
    let InvariantVerdict::Violated(detail) =
        verdict(&image, SYSTEM_CONFIGURATION_RECORDS_THE_WRITER_IDENTITY)
    else {
        panic!("上面判过 I-1.5 红");
    };
    assert!(
        detail.starts_with("盘 1 择到的系统配置") && detail.contains("实现标识 16 字节全零"),
        "违例点名盘 1 那一槽的实现标识全零：{detail}"
    );
    assert_eq!(
        verdicts_that_differ_outside(
            &verdicts,
            &clean_verdicts,
            &[SYSTEM_CONFIGURATION_RECORDS_THE_WRITER_IDENTITY]
        ),
        Vec::new(),
        "除 I-1.5 之外每一条与干净镜像上的判定逐条相同"
    );
}

/// I-3.18（分配记录的跨度等于它罩住的单元那一类的槽数，C411（分配记录的跨度段与单元类别不符没检查））的坏镜像：码 1 单元那条未释放记录
/// 只记一槽，**只**红 I-3.18，说明点名码 1 与跨 1 槽。单元的第二槽（50181）在盘上没有归属，只看重叠的 I-5.4 看不见它。
/// I-3.1（已分配统计对得上） 的逐槽那一判（遍历走到的槽都在记录里）见得到 50181，上沿也跟着少一槽，但这块畸形 I-3.18 已判红：
/// 配不上的全落在跨度段不符的那条记录罩着的单元里，I-3.1 报部分没判、点名 I-3.18，不判红。
/// 除这两条之外每一条与干净镜像上的判定逐条相同。
#[test]
fn an_unreleased_allocation_record_spanning_one_slot_of_a_two_slot_data_unit_reddens_only_the_span_invariant(
) {
    let clean = build_pool("known-bad-data-unit-record-narrowed").memory_pool();
    let clean_verdicts = check_pool_image(&clean);
    assert_eq!(
        verdict(
            &clean,
            ALLOCATION_RECORD_SPAN_EQUALS_THE_SLOTS_OF_THE_CLASS_OF_THE_UNIT_IT_COVERS
        ),
        InvariantVerdict::Holds,
        "干净镜像上 I-3.18 真被评估过且成立"
    );
    let mut image = clean.clone();
    narrow_the_record_of_the_data_unit_to_one_slot(&mut image);
    let verdicts = check_pool_image(&image);
    assert_eq!(
        violated_invariants(&verdicts),
        vec![ALLOCATION_RECORD_SPAN_EQUALS_THE_SLOTS_OF_THE_CLASS_OF_THE_UNIT_IT_COVERS],
        "只红 I-3.18：{verdicts:?}"
    );
    let InvariantVerdict::Violated(detail) = verdict(
        &image,
        ALLOCATION_RECORD_SPAN_EQUALS_THE_SLOTS_OF_THE_CLASS_OF_THE_UNIT_IT_COVERS,
    ) else {
        panic!("上面判过 I-3.18 红");
    };
    assert!(
        detail.contains(&format!("槽 {DATA_UNIT} 的分配记录（未释放）跨 1 槽"))
            && detail.contains("码 1，那一类占 2 槽"),
        "违例点名数据单元那条记录跨 1 槽、而它是码 1 两槽的单元：{detail}"
    );
    match verdict(&image, "I-3.1") {
        InvariantVerdict::PartiallyJudged(unjudged) => assert!(
            unjudged.contains(&format!("遍历走到而不在记录里的槽 [{}]", DATA_UNIT + 1))
                && unjudged.contains("（I-3.18 判它）"),
            "I-3.1 部分没判的说明点名第二槽 50181 与 I-3.18：{unjudged}"
        ),
        other @ (InvariantVerdict::Holds
        | InvariantVerdict::Violated(_)
        | InvariantVerdict::NotApplicable(_)) => {
            panic!("配不上的全落在 I-3.18 判红的单元里：I-3.1 该报部分没判，实际 {other:?}")
        }
    }
    assert_eq!(
        verdicts_that_differ_outside(
            &verdicts,
            &clean_verdicts,
            &[
                ALLOCATION_RECORD_SPAN_EQUALS_THE_SLOTS_OF_THE_CLASS_OF_THE_UNIT_IT_COVERS,
                "I-3.1"
            ]
        ),
        Vec::new(),
        "除 I-3.18、I-3.1 之外每一条与干净镜像上的判定逐条相同"
    );
}

/// 墓碑与谱系那九条的坏镜像各要一份自己的基底（意图之后、根环转过一圈之后、回退之后、旋钮压小写出的），不进上面那张
/// 只改干净镜像一处的表；这里登记每一条的坏镜像在哪个用例里（那几个用例各自断言只红它自己）。
const TOMBSTONE_AND_LINEAGE_INVARIANTS_AND_WHERE_THEIR_BAD_IMAGES_LIVE: [(&str, &str); 9] = [
    (
        LINEAGE_TREE_ENTRY_MATCHES_ITS_CHILD_HEADER,
        "checker_known_bad_images.rs: a_lineage_tree_entry_whose_identity_names_another_container_reddens_only_the_lineage_entry_invariant",
    ),
    (
        QUEUED_INODE_HAS_A_TOMBSTONE_ON_THE_LINEAGE,
        "pool_checker_judges_the_deletion_state_of_every_walked_version.rs: a_queued_inode_whose_tombstone_died_on_a_branch_a_rollback_left_reddens_only_the_queued_tombstone_invariant",
    ),
    (
        TOMBSTONE_ON_THE_LINEAGE_NAMES_NO_LIVE_OBJECT,
        "pool_checker_judges_the_deletion_state_of_every_walked_version.rs: a_tombstone_on_the_lineage_naming_a_live_inode_reddens_only_the_live_object_invariant",
    ),
    (
        TOMBSTONES_ON_THE_LINEAGE_ARE_DISTINCT_AND_EVERY_TOMBSTONE_IS_WELL_FORMED,
        "pool_checker_judges_the_deletion_state_of_every_walked_version.rs: a_tombstone_that_does_not_cover_the_whole_object_reddens_only_the_tombstone_record_invariant",
    ),
    (
        NON_RIGHTMOST_TOMBSTONE_CONTAINERS_ARE_FULL_ON_A_PRODUCT_IMAGE,
        "container_trees_grow_a_level_when_the_root_is_full.rs: a_capped_tombstone_tree_judged_as_a_product_image_reddens_only_the_full_container_invariant",
    ),
    (
        TOMBSTONE_CONTAINERS_STAY_WITHIN_THE_CAPACITY_KNOB_ON_A_TEST_IMAGE,
        "container_trees_grow_a_level_when_the_root_is_full.rs: a_tombstone_container_holding_more_records_than_the_knob_reddens_only_the_knob_capacity_invariant",
    ),
    (
        TOMBSTONE_TREE_ENTRY_MATCHES_ITS_CHILD_HEADER,
        "checker_known_bad_images.rs: a_tombstone_tree_entry_whose_identity_names_another_container_reddens_only_the_tombstone_entry_invariant",
    ),
    (
        DELETION_QUEUE_HOLDS_AT_MOST_ONE_KEY,
        "pool_checker_judges_the_deletion_state_of_every_walked_version.rs: a_deletion_queue_of_two_inodes_being_deleted_with_their_tombstones_reddens_only_the_at_most_one_key_invariant",
    ),
    (
        ONE_OBJECT_BIRTH_PER_INODE_NUMBER,
        "inode_number_watermark_pressed_back_reissues_a_number.rs: pressing_the_watermark_back_to_the_largest_issued_number_reissues_it_and_reddens_the_one_birth_per_number_and_the_watermark_step_invariants",
    ),
];

/// 退役树那五条（I-9.31–I-9.35）的坏镜像各建在
/// 「三个单元的文件删到底」那一份上（唯一那片叶容器身份消失、退役树一条记录），或它再建 31 次 inode、删的那几版转出回退候选集的那一份上，
/// 不进只改干净镜像一处的表。
const RETIREMENT_INVARIANTS_AND_WHERE_THEIR_BAD_IMAGES_LIVE: [(&str, &str); 5] = [
    (
        RETIREMENT_TREE_ENTRY_MATCHES_ITS_CHILD_HEADER,
        "checker_known_bad_images.rs: a_retirement_tree_entry_whose_identity_names_another_container_reddens_only_the_retirement_entry_invariant",
    ),
    (
        EVERY_RETIREMENT_RECORD_IS_WELL_FORMED,
        "checker_known_bad_images.rs: a_retirement_record_naming_a_container_of_record_type_one_reddens_only_the_well_formed_retirement_record_invariant",
    ),
    (
        RETIREMENT_RECORDS_ON_THE_LINEAGE_ARE_DISTINCT,
        "checker_known_bad_images.rs: a_second_copy_of_the_retirement_record_reddens_only_the_distinct_retirement_records_invariant",
    ),
    (
        RETIREMENT_RECORD_ON_THE_LINEAGE_NAMES_NO_LIVE_LEAF,
        "checker_known_bad_images.rs: a_retirement_record_naming_the_live_leaf_reddens_only_the_no_live_leaf_invariant",
    ),
    (
        RETIREMENT_RECORDS_ACCOUNT_FOR_EVERY_LEAF_GONE_SINCE_THE_PARENT_VERSION,
        "checker_known_bad_images.rs: a_retirement_record_moved_off_the_txg_the_leaf_died_at_reddens_only_the_retirement_completeness_invariant",
    ),
];

/// 谱系树与退役树的容器容量那两条（I-9.36、I-9.37）的坏镜像
/// 各要一份旋钮压小写出的基底（攻方 `opus_probe_container_closed_early` 那一形：两次叶身份消失、两次回退），不进只改干净镜像一处的表。
const LINEAGE_AND_RETIREMENT_CONTAINER_CAPACITY_INVARIANTS_AND_WHERE_THEIR_BAD_IMAGES_LIVE: [(
    &str,
    &str,
); 3] = [
    (
        NON_RIGHTMOST_LINEAGE_AND_RETIREMENT_CONTAINERS_ARE_FULL_ON_A_PRODUCT_IMAGE,
        "checker_known_bad_images.rs: two_leaves_retired_one_record_per_retirement_container_judged_as_a_product_image_reddens_only_the_full_container_invariant",
    ),
    (
        NON_RIGHTMOST_LINEAGE_AND_RETIREMENT_CONTAINERS_ARE_FULL_ON_A_PRODUCT_IMAGE,
        "checker_known_bad_images.rs: two_rollbacks_one_record_per_lineage_container_judged_as_a_product_image_reddens_only_the_full_container_invariant",
    ),
    (
        LINEAGE_AND_RETIREMENT_CONTAINERS_STAY_WITHIN_THE_CAPACITY_KNOBS_ON_A_TEST_IMAGE,
        "checker_known_bad_images.rs: a_lineage_container_holding_more_records_than_the_knob_reddens_only_the_knob_capacity_invariant",
    ),
];

/// I-3.14（实例表每片记录形态合法） 的三张坏镜像各建在攻方 N5 那一份基底上（崩溃恢复抛弃 C、C 与现行线同号不同出生代，
/// [`image_with_an_abandoned_root_naming_another_object_birth_and_the_newest_instance_table_changed`]），不进只改干净镜像一处的表；
/// 这里登记它们在哪几个用例里（各自断言判红的是 I-3.14 与这一片解不开带出来的那几条）。最后一条不是合法的链指针记录那一格的坏镜像在
/// `instance_table_chain_read.rs` 的 `contradictory_chain_record_reddens_only_the_instance_table_invariant`。
const INSTANCE_TABLE_RECORD_FORM_INVARIANT_AND_WHERE_ITS_BAD_IMAGES_LIVE: [(&str, &str); 3] = [
    (
        INSTANCE_TABLE_PAGE_RECORDS_ARE_WELL_FORMED,
        "checker_known_bad_images.rs: a_record_of_an_unregistered_kind_before_the_chain_record_of_the_newest_instance_table_does_not_decode_and_leaves_the_abandoned_root_in_the_candidate_set",
    ),
    (
        INSTANCE_TABLE_PAGE_RECORDS_ARE_WELL_FORMED,
        "checker_known_bad_images.rs: a_second_chain_record_in_the_middle_of_the_newest_instance_table_page_does_not_decode_and_leaves_the_abandoned_root_in_the_candidate_set",
    ),
    (
        INSTANCE_TABLE_PAGE_RECORDS_ARE_WELL_FORMED,
        "checker_known_bad_images.rs: a_row_whose_flags_are_not_zero_in_the_newest_instance_table_does_not_decode_and_leaves_the_abandoned_root_in_the_candidate_set",
    ),
];

/// I-9.30（inode 号水位的台阶行与水位、记录、谱系对得上） 的坏镜像各要一份自己的基底（崩溃恢复抛弃建过 inode 的根之后再建、再回退；
/// 建过 inode 之后覆盖写），不进只改干净镜像一处的表；这里登记它们在哪几个用例里（各自断言只红它）。② 那一判（记录晚于台阶出生）的坏镜像是
/// 压回水位再建的那几份（`inode_number_watermark_pressed_back_reissues_a_number.rs`）。
const INODE_NUMBER_WATERMARK_STEP_INVARIANT_AND_WHERE_ITS_BAD_IMAGES_LIVE: [(&str, &str); 4] = [
    (
        INODE_NUMBER_WATERMARK_STEPS_AGREE_WITH_THE_WATERMARK_THE_RECORDS_AND_THE_LINEAGE,
        "checker_known_bad_images.rs: a_lineage_record_whose_watermark_rolled_back_from_is_below_the_last_step_reddens_only_the_watermark_step_invariant",
    ),
    (
        INODE_NUMBER_WATERMARK_STEPS_AGREE_WITH_THE_WATERMARK_THE_RECORDS_AND_THE_LINEAGE,
        "checker_known_bad_images.rs: a_lineage_record_whose_watermark_rolled_back_from_is_zero_reddens_only_the_watermark_step_invariant",
    ),
    (
        INODE_NUMBER_WATERMARK_STEPS_AGREE_WITH_THE_WATERMARK_THE_RECORDS_AND_THE_LINEAGE,
        "checker_known_bad_images.rs: an_inode_number_watermark_step_dropped_from_the_accounting_tree_reddens_only_the_watermark_step_invariant",
    ),
    (
        INODE_NUMBER_WATERMARK_STEPS_AGREE_WITH_THE_WATERMARK_THE_RECORDS_AND_THE_LINEAGE,
        "checker_known_bad_images.rs: an_inode_number_watermark_step_raised_above_the_watermark_reddens_only_the_watermark_step_invariant",
    ),
];

/// 只读快照那一批的 I-3.6（deadlist 紧界） 与 I-3.7（活快照可枚举）（判决 `research/prompts/m4-snapshot-r1-main-verification.md` 第三节 S5）：
/// 坏镜像要写路径建出快照的基底（I-3.7）或一个有 deadlist 条目的世界（I-3.6，deadlist 写者落地之前只有池级 checker 单测里的合成世界），
/// 不进只改干净镜像一处的表；这里登记它们在哪几个用例里。
const SNAPSHOT_AND_DEADLIST_INVARIANTS_AND_WHERE_THEIR_BAD_IMAGES_LIVE: [(&str, &str); 3] = [
    (
        LIVE_SNAPSHOTS_ARE_ENUMERABLE,
        "checker_snapshot_space_accounting_must_red_worlds.rs: removing_the_second_snapshot_from_the_newest_tree_table_reddens_the_enumerable_invariant",
    ),
    (
        LIVE_SNAPSHOTS_ARE_ENUMERABLE,
        "checker_snapshot_space_accounting_must_red_worlds.rs: adding_a_snapshot_pair_with_an_older_txg_to_the_newest_tree_table_reddens_the_enumerable_invariant",
    ),
    (
        DEADLIST_ENTRIES_ARE_BORN_NO_LATER_THAN_THE_PREVIOUS_SNAPSHOT_OF_THEIR_SIDE,
        "crates/singlefs-checker/src/walk.rs 的测试模块: deadlist_entry_born_after_the_previous_snapshot_of_its_side_violates_the_tight_bound",
    ),
];

/// 里程碑四自建 GC 的 checker 新判（判决 `research/prompts/m4-gc-r1-main-verification.md` 第三节 G1、G3 的 N2–N6）：I-3.15、I-5.6、I-4.4、I-3.16、I-1.14
/// 的坏镜像在 `checker_compaction_intent_and_relocation_must_red_worlds.rs`（logged_ops 树与整理意图、搬迁那几份都要另搭，不从这里的干净镜像改一处）。
const COMPACTION_INVARIANTS_AND_WHERE_THEIR_BAD_IMAGES_LIVE: [(&str, &str); 5] = [
    (
        UNRELEASED_ALLOCATION_RECORD_COVERS_A_UNIT_THE_CENTRAL_MAPPING_PLACES_THERE,
        "checker_compaction_intent_and_relocation_must_red_worlds.rs: a_mapping_entry_moved_without_a_copy_reddens_the_reverse_lookup_at_the_source",
    ),
    (
        COMPACTION_REGION_HOLDS_NO_RECORD_ALLOCATED_AFTER_THE_INTENT_WAS_ISSUED,
        "checker_compaction_intent_and_relocation_must_red_worlds.rs: records_allocated_into_the_region_after_the_intent_was_issued_redden_the_region_exclusion",
    ),
    (
        INTENT_DESCRIBES_ONLY_THE_TARGET_STATE,
        "checker_compaction_intent_and_relocation_must_red_worlds.rs: a_compaction_intent_carrying_an_increment_in_its_padding_reddens_only_the_target_state_invariant",
    ),
    (
        FRAGMENTATION_AND_EMPTY_SEGMENT_STATISTICS_EQUAL_A_FULL_SCAN,
        "checker_compaction_intent_and_relocation_must_red_worlds.rs: a_runs_row_one_short_reddens_only_the_full_scan_statistics",
    ),
    (
        UNIT_BYTES_OF_ONE_MAPPING_KEY_AGREE_ACROSS_THE_VERSIONS,
        "checker_compaction_intent_and_relocation_must_red_worlds.rs: a_byte_changed_in_the_relocated_copy_reddens_the_same_bytes_invariant",
    ),
];

/// 截断那一批新立的两条（判决 `research/prompts/m4-trunc-r1-main-verification.md` 第三节 K7 T-2、T-4）：I-4.10、I-9.38 的坏镜像在
/// `checker_truncation_holes_and_intent_must_red_worlds.rs`（截断条目、size 之外挂着的单元、尾单元的声明长度都要另搭，不从这里的干净镜像改一处）。
/// 截断字段那一批（判决 `research/prompts/m4-trunc-r2-main-verification.md` 第三节 R6）新立的 I-9.39 的坏镜像在
/// `checker_truncation_field_must_red_worlds.rs`（要另置 incompat 位 2、另写记录的截断字段，不从这里的干净镜像改一处）。
const TRUNCATION_INVARIANTS_AND_WHERE_THEIR_BAD_IMAGES_LIVE: [(&str, &str); 3] = [
    (
        DATA_UNITS_BEYOND_THE_SIZE_ARE_NAMED_BY_A_TRUNCATION_INTENT,
        "checker_truncation_holes_and_intent_must_red_worlds.rs: deleting_the_truncation_entry_while_units_beyond_the_size_remain_reddens_the_intermediate_state_account",
    ),
    (
        DECLARED_LENGTH_OF_EACH_DATA_UNIT_WITHIN_THE_SIZE_MATCHES_THE_SIZE,
        "checker_truncation_holes_and_intent_must_red_worlds.rs: shrinking_the_size_into_the_unit_without_rewriting_it_reddens_only_the_declared_length",
    ),
    (
        LIVE_DATA_UNITS_BEYOND_THE_TRUNCATION_FIELD_ARE_BORN_NO_EARLIER_THAN_IT,
        "checker_truncation_field_must_red_worlds.rs: a_live_data_unit_born_before_the_truncation_field_reddens_only_the_birth_txg_invariant",
    ),
];

/// extent 上段规范形那一条（判决 `research/prompts/m3-multi-file-unlink-r1-main-verification.md` 第三节 M2）：I-9.40 的坏镜像要上段长到层级 1
/// 再改根（这里的干净镜像上段只有根兼叶），在 `unlink_of_files_other_than_the_first.rs`；必红世界三形的单测在 `crates/singlefs-checker/src/walk.rs`。
const EXTENT_UPPER_SHAPE_INVARIANT_AND_WHERE_ITS_BAD_IMAGES_LIVE: [(&str, &str); 1] = [(
    singlefs_checker::image::EXTENT_UPPER_SEGMENT_IS_THE_CANONICAL_SHAPE_OF_THE_INODES_WITH_AN_ENTRY,
    "unlink_of_files_other_than_the_first.rs: a_root_left_one_level_above_the_only_leaf_with_an_entry_reddens_the_upper_shape_invariant",
)];

/// 隔离记号那一条（D19（块指针的结构与宽度预算） 已定项 5「新加判据『记号与引用相符』」）：I-3.17 的坏镜像要先隔离一个单元、
/// 再改写记号（这里的干净镜像上没有隔离），在 `checker_quarantine_mark_agrees_with_the_references.rs`（一块盘带一块盘不带红在第三句、带记号而被
/// 最新根引用红在第二句）。
const RECLAIM_THRESHOLD_AND_PINNED_RING_SLOT_INVARIANTS_AND_WHERE_THEIR_BAD_IMAGES_LIVE: [(
    &str,
    &str,
); 2] = [
    (
        singlefs_checker::image::ROOT_RECORD_RECLAIM_THRESHOLD_EQUALS_THE_RECOMPUTED_THRESHOLD,
        "root_record_carries_the_reclaim_threshold_and_the_pinned_ring_slot.rs: a_reclaim_threshold_written_as_the_floor_while_a_slot_stays_pinned_reddens_the_reclaim_threshold_invariant、a_pinned_ring_slot_written_as_none_while_a_slot_stays_pinned_reddens_the_reclaim_threshold_invariant",
    ),
    (
        singlefs_checker::image::PINNED_RING_SLOT_HOLDS_NO_ROOT_OF_THE_SAME_INSTANCE,
        "root_record_carries_the_reclaim_threshold_and_the_pinned_ring_slot.rs: a_pinned_ring_slot_left_on_a_slot_this_instance_wrote_reddens_the_pinned_slot_invariant",
    ),
];

const QUARANTINE_MARK_INVARIANT_AND_WHERE_ITS_BAD_IMAGES_LIVE: [(&str, &str); 1] = [(
    QUARANTINE_MARK_AGREES_WITH_THE_REFERENCES,
    "checker_quarantine_mark_agrees_with_the_references.rs: a_quarantine_carrying_the_mark_on_one_device_only_is_violated_by_the_third_sentence、a_record_carrying_the_mark_under_a_unit_the_newest_root_references_is_violated_by_the_second_sentence",
)];

/// 记录落在它的计数器该落的环槽那一条（`.claude/kb/checks-owed.md` C897，D23（journal 的角色与格式） 已定项 18 的可执行形式）：I-8.13 的坏镜像要
/// 同一块盘上两条自证记录撞同一个计数器（这里的干净镜像上每条记录都坐在自己该落的槽上），审计 C02 那四种排列在
/// `checker_cached_reference_and_journal_conflicts.rs`。
const JOURNAL_RING_SLOT_INVARIANT_AND_WHERE_ITS_BAD_IMAGES_LIVE: [(&str, &str); 1] = [(
    singlefs_checker::image::JOURNAL_RECORDS_LIVE_IN_THE_RING_SLOT_THEIR_COUNTER_NAMES,
    "checker_cached_reference_and_journal_conflicts.rs: two_records_of_one_instance_and_counter_on_one_device_redden_the_ring_slot_invariant、two_records_of_one_instance_and_counter_on_one_device_in_reverse_order_redden_the_ring_slot_invariant、records_of_two_instances_at_one_counter_on_one_device_redden_the_ring_slot_invariant、records_of_two_instances_at_one_counter_on_one_device_in_reverse_order_redden_the_ring_slot_invariant",
)];

/// 走到的引用起点那一条（C944（走到的单元起点没有自己的记录），判决 `research/prompts/m3-close-b19-reason-r2-main-verification.md` 第二节 Q2「归属」）：
/// 坏镜像要一条宽记录恰好罩住一个走到、没有自己记录的邻居（这里的干净镜像上每个走到的起点都有自己的记录）。
const WALKED_REFERENCE_START_INVARIANT_AND_WHERE_ITS_BAD_IMAGES_LIVE: [(&str, &str); 1] = [(
    WALKED_REFERENCE_STARTS_ARE_STARTS_OF_ALLOCATION_RECORDS_THE_NEWEST_ROOT_COUNTS,
    "checker_known_bad_images.rs: record_widened_exactly_over_a_walked_one_slot_neighbor_without_its_own_record_reddens_only_the_span_and_the_walked_reference_start_invariants",
)];

/// 清单里不要求坏镜像的几条：现在一条都没有。I-4.4（意图只描述目标态） 判 logged_ops 里的整理意图，坏镜像登记在
/// [`COMPACTION_INVARIANTS_AND_WHERE_THEIR_BAD_IMAGES_LIVE`]。
const INVARIANTS_WITHOUT_A_BAD_IMAGE: [&str; 0] = [];

/// 干净镜像上 I-4.7（删除中 inode 有账） 没有对象时报的理由（`walk` 的删除态扫描）。
pub(super) const NO_INODE_BEING_DELETED_IN_ANY_WALKED_VERSION: &str = "走读到的每一版里都没有处于删除中的 inode（nlink = 0 的记录、有数据单元而没有 nlink ≥ 1 记录的号）";

#[test]
fn the_clean_image_holds_every_invariant_and_each_mutation_violates_its_target() {
    let clean = build_pool("known-bad").memory_pool();
    for (invariant, found) in check_pool_image(&clean) {
        let expected = match invariant {
            // 写完新池新建文件的镜像上只有 A 的根有树表条目：没有一棵树的条目出现在两个树表单元里，跨根比不出来。
            // 这一条的坏镜像在发布 B 之后那一份上（`each_release_generation_and_tree_table_birth_bad_image_reddens_only_its_own_invariant_after_the_overwrite`）。
            "I-9.14" => InvariantVerdict::NotApplicable(
                "没有一棵树的树表条目出现在两个不同的树表单元里：跨根比不出来",
            ),
            // 同一个理由，比的是根环里全部读得出的根（暖机那几代都指着 mkfs 种的第 0 版树表，空的）。这一条的坏镜像在「崩溃恢复抛弃了第三条根 C」
            // 那一份上（`birth_txg_recorded_differently_only_on_the_timeline_cut_off_by_a_recovery_holds_the_same_line_invariant_and_reddens_only_the_every_readable_root_invariant`）。
            "I-9.28" => InvariantVerdict::NotApplicable(
                "根环里读得出的根中，没有一棵树的树表条目出现在两个不同的树表单元里：跨根比不出来",
            ),
            // 这一份上实例 1 只有 A 一条非 0 事务号的记录（写行与两次暖机空发布都写 0）：一对可比的号凑不出来。
            // 「从 1 起」那半句判过且成立（A 的事务号是 1，计数器链从 1 读得全），理由句照实说（C469 ②，判决
            // `research/prompts/m3-close-b19-design-r2-main-verification.md` 第二节 Q3 采甲加理由句）。
            // 这一条的坏镜像与阳性对照都在发布 B 之后那一份上
            // （`repeated_transaction_number_in_one_instance_reddens_the_uncut_transaction_invariant_and_only_when_separated_the_transaction_number_invariant` 与
            // `each_transaction_boundary_bad_image_reddens_only_its_own_invariant_on_its_registered_criteria`）。
            "I-8.7" => InvariantVerdict::NotApplicable(
                "从 1 起判过且成立，凑不出一对可比的号：环里没有哪个实例写出过两条非 0 事务号的记录",
            ),
            // 这一份上只有实例 1，最新根的实例表一行都没有（第一次可写挂载不写行）。阳性对照与坏镜像在可写挂载施加过一条记录的那一份上
            // （`ROW_HIGH_WATER_FIRST_TXG_AND_REPLAYED_ALLOCATION_RECORD_TREE_ROOT_INVARIANTS_AND_WHERE_THEIR_BAD_IMAGES_LIVE`）。
            INSTANCE_TABLE_ROW_HIGH_WATER_IS_ONE_A_RECOVERY_COULD_HAVE_APPLIED => {
                InvariantVerdict::NotApplicable(NO_INSTANCE_TABLE_ROW_UNDER_THE_NEWEST_ROOT)
            }
            // 每条根槽都读得出、最新根之上没有记录：没有由记录施加出来的那一版。阳性对照与坏镜像在四次可写挂载、一个文件都不写的那一份上。
            ALLOCATION_RECORD_TREE_ROOT_OF_A_VERSION_APPLIED_ONLY_BY_RECORDS_IS_BORN_WITH_THE_VERSION => {
                InvariantVerdict::NotApplicable(
                    NO_VERSION_APPLIED_ONLY_BY_RECORDS_REWRITING_ITS_ALLOCATION_RECORD_TREE,
                )
            }
            // 一事务一条、每条都带提交标记：③ ④ 没有对象。写行与两次暖机那两条事务号 0 的记录各带提交标记 1，
            // 拿事务号 0 成组的写法在这一份上就红（③：一组里两条带提交标记）——这一格是「事务号 0 不成组」那条射程的阳性对照。
            // 阳性对照与坏镜像在发布 B 之后那一份上
            // （`each_transaction_boundary_bad_image_reddens_only_its_own_invariant_on_its_registered_criteria`）。
            "I-8.8" => InvariantVerdict::NotApplicable(COMMIT_MARKERS_NOT_APPLICABLE),
            // 这一份上没抬过 F：每条根带的 F 都是 0。阳性对照与坏镜像在步 5 那段历史上
            // （`raising_the_floor_above_its_ceiling_reddens_only_the_rollback_floor_invariant`）。
            "I-7.9" => InvariantVerdict::NotApplicable(NO_ROOT_RAISED_THE_FLOOR),
            // 这一份上只有实例 1，实例表一行都没有。阳性对照与坏镜像在可写挂载施加过一条记录的那一份上
            // （`an_instance_table_row_whose_published_txg_reaches_the_root_referencing_it_reddens_only_the_instance_table_row_invariant`）。
            "I-3.12" => InvariantVerdict::NotApplicable(INSTANCE_TABLE_ROWS_NOT_APPLICABLE),
            // 这一份上最新根（txg 3）之上没有记录：重放之后那一版就是它。阳性对照与坏镜像在「发布 B 的记录落了、根槽没落」那一份崩溃镜像上
            // （`a_replayed_version_whose_tree_table_does_not_read_back_reddens_only_the_replayed_version_invariant`）。
            "I-4.1" => InvariantVerdict::NotApplicable(NOTHING_APPLIED_ABOVE_THE_CHOSEN_ROOT),
            // 这一份上没有删除：inode 1 的记录 nlink 1、队列空，没有处于删除中的号。阳性对照与坏镜像在删除中间那几版上
            // （`pool_checker_judges_the_deletion_state_of_every_walked_version.rs`，坏镜像登记在下面那一串 `.chain` 里）。
            "I-4.7" => {
                InvariantVerdict::NotApplicable(NO_INODE_BEING_DELETED_IN_ANY_WALKED_VERSION)
            }
            // I-4.6 自截断起每条记录都判（单元连号、单元数对标签，m4-trunc-r1 判决 K7 T-3）：inode 1 的记录 size 与单元数对得上，成立。
            "I-4.6" => InvariantVerdict::Holds,
            // logged_ops 树是零根、没有意图条目：报不适用并写理由；坏镜像在 `COMPACTION_INVARIANTS_AND_WHERE_THEIR_BAD_IMAGES_LIVE`。
            "I-4.4" => InvariantVerdict::NotApplicable(
                NO_INTENT_ENTRY_IN_THE_LOGGED_OPERATIONS_OF_ANY_WALKED_VERSION,
            ),
            // 墓碑与谱系那九条：这一份上没有删除、没有回退，墓碑树与谱系树都是零根。坏镜像见
            // `TOMBSTONE_AND_LINEAGE_INVARIANTS_AND_WHERE_THEIR_BAD_IMAGES_LIVE`。
            other_invariant => {
                expected_verdict_of_the_tombstone_and_lineage_invariants_without_deletion(
                    other_invariant,
                )
            }
        };
        assert_eq!(found, expected, "干净镜像上 {invariant} 的判定");
    }
    let cases = known_bad_images(&clean);
    let (_, birth_txg, tree_table_slot_of_the_abandoned_root, abandoned_publish_txg) =
        image_after_a_recovery_abandoned_the_third_root(
            "known-bad-every-readable-root-birth-listed",
        );
    let targets: std::collections::BTreeSet<&str> = cases
        .iter()
        .chain(known_bad_images_after_the_overwrite().iter())
        .chain(
            known_bad_images_of_the_birth_txg_across_every_readable_root(
                tree_table_slot_of_the_abandoned_root,
                birth_txg,
                abandoned_publish_txg,
            )
            .iter(),
        )
        .chain(known_bad_images_of_overlapping_allocation_records().iter())
        .chain(known_bad_images_of_the_root_ring_health().iter())
        .chain(known_bad_images_of_the_transaction_boundaries().iter())
        .chain(known_bad_images_of_the_publish_ordinals().iter())
        .chain(known_bad_images_of_the_allocation_generation().iter())
        .chain(known_bad_images_of_a_pure_leak().iter())
        .chain(known_bad_images_of_the_deferred_statistic().iter())
        .chain(known_bad_images_of_the_rollback_floor().iter())
        .chain(known_bad_images_of_the_unmount_floor().iter())
        .chain(known_bad_images_of_the_system_configuration_floor().iter())
        .chain(known_bad_images_of_pointer_journal_and_generation_fields().iter())
        .chain(known_bad_images_of_the_system_configuration_slot_generations().iter())
        .chain(known_bad_images_of_the_instance_table_rows().iter())
        .chain(known_bad_images_of_the_ring_geometry_and_the_root_slot_alignment().iter())
        .chain(known_bad_images_of_the_tail_slot_witnesses().iter())
        .chain(known_bad_images_of_the_replayed_version().iter())
        .chain(common_deletion_in_progress::known_bad_images_after_the_intent_of_a_three_unit_file().iter())
        .chain(common_deletion_in_progress::known_bad_images_after_the_intent_of_an_empty_inode().iter())
        .chain(
            common_deletion_in_progress::known_bad_images_after_the_first_batch_of_a_three_unit_file()
                .iter(),
        )
        .chain(common_deletion_in_progress::known_bad_images_after_one_new_inode().iter())
        .chain(common_deletion_in_progress::known_bad_images_of_the_pending_deletion_occupancy().iter())
        .map(|(invariant, _)| *invariant)
        .chain(
            TOMBSTONE_AND_LINEAGE_INVARIANTS_AND_WHERE_THEIR_BAD_IMAGES_LIVE
                .iter()
                .map(|(invariant, _)| *invariant),
        )
        .chain(
            INSTANCE_TABLE_RECORD_FORM_INVARIANT_AND_WHERE_ITS_BAD_IMAGES_LIVE
                .iter()
                .map(|(invariant, _)| *invariant),
        )
        .chain(
            INODE_NUMBER_WATERMARK_STEP_INVARIANT_AND_WHERE_ITS_BAD_IMAGES_LIVE
                .iter()
                .map(|(invariant, _)| *invariant),
        )
        .chain(
            RETIREMENT_INVARIANTS_AND_WHERE_THEIR_BAD_IMAGES_LIVE
                .iter()
                .map(|(invariant, _)| *invariant),
        )
        .chain(
            LINEAGE_AND_RETIREMENT_CONTAINER_CAPACITY_INVARIANTS_AND_WHERE_THEIR_BAD_IMAGES_LIVE
                .iter()
                .map(|(invariant, _)| *invariant),
        )
        .chain(
            SNAPSHOT_AND_DEADLIST_INVARIANTS_AND_WHERE_THEIR_BAD_IMAGES_LIVE
                .iter()
                .map(|(invariant, _)| *invariant),
        )
        .chain(
            COMPACTION_INVARIANTS_AND_WHERE_THEIR_BAD_IMAGES_LIVE
                .iter()
                .map(|(invariant, _)| *invariant),
        )
        .chain(
            TRUNCATION_INVARIANTS_AND_WHERE_THEIR_BAD_IMAGES_LIVE
                .iter()
                .map(|(invariant, _)| *invariant),
        )
        .chain(
            EXTENT_UPPER_SHAPE_INVARIANT_AND_WHERE_ITS_BAD_IMAGES_LIVE
                .iter()
                .map(|(invariant, _)| *invariant),
        )
        .chain(
            QUARANTINE_MARK_INVARIANT_AND_WHERE_ITS_BAD_IMAGES_LIVE
                .iter()
                .map(|(invariant, _)| *invariant),
        )
        .chain(
            RECLAIM_THRESHOLD_AND_PINNED_RING_SLOT_INVARIANTS_AND_WHERE_THEIR_BAD_IMAGES_LIVE
                .iter()
                .map(|(invariant, _)| *invariant),
        )
        .chain(
            JOURNAL_RING_SLOT_INVARIANT_AND_WHERE_ITS_BAD_IMAGES_LIVE
                .iter()
                .map(|(invariant, _)| *invariant),
        )
        .chain(
            ROW_HIGH_WATER_FIRST_TXG_AND_REPLAYED_ALLOCATION_RECORD_TREE_ROOT_INVARIANTS_AND_WHERE_THEIR_BAD_IMAGES_LIVE
                .iter()
                .map(|(invariant, _)| *invariant),
        )
        .chain(
            WALKED_REFERENCE_START_INVARIANT_AND_WHERE_ITS_BAD_IMAGES_LIVE
                .iter()
                .map(|(invariant, _)| *invariant),
        )
        .collect();
    let listed: std::collections::BTreeSet<&str> = singlefs_checker::image::IMPLEMENTED_INVARIANTS
        .iter()
        .copied()
        .filter(|invariant| !INVARIANTS_WITHOUT_A_BAD_IMAGE.contains(invariant))
        .collect();
    assert_eq!(
        targets, listed,
        "第一版每条不变量（`INVARIANTS_WITHOUT_A_BAD_IMAGE` 那几条除外）都至少有一份坏镜像，也没有清单之外的"
    );
    for (invariant, mutation) in cases {
        let mut image = clean.clone();
        mutation(&mut image);
        let found = verdict(&image, invariant);
        assert!(
            matches!(found, InvariantVerdict::Violated(_)),
            "{invariant} 那份坏镜像应当判违例，实际 {found:?}"
        );
    }
}
