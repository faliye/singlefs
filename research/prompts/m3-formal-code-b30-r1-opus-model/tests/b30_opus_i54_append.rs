

// ───── b30 攻方腿 M2 探针（接在 checker_known_bad_images.rs 的副本后面）：覆盖写之后（发布 B）逐个把一个单元的两份副本清零，
// 看第 30 批按版本的 I-5.4 报什么；另把 B 的树表（50264）单独清零，看 B 那一版的分配记录走没走到、I-5.4 报不报部分没判 ─────
fn b30_opus_i54_label(verdict: &InvariantVerdict) -> String {
    match verdict {
        InvariantVerdict::Holds => "Holds".to_string(),
        InvariantVerdict::Violated(t) => format!("Violated({})", t.chars().take(80).collect::<String>()),
        InvariantVerdict::NotApplicable(t) => format!("NotApplicable({})", t.chars().take(40).collect::<String>()),
        InvariantVerdict::PartiallyJudged(t) => format!("PartiallyJudged({})", t.chars().take(120).collect::<String>()),
    }
}

#[test]
fn b30_opus_i54_after_zeroing_each_unit_of_the_second_version() {
    let clean = image_after_the_overwrite("b30-opus-i54-each-unit");
    println!("B30UNIT clean I-5.4={}", b30_opus_i54_label(&verdict(&clean, "I-5.4")));
    for (slot, length) in UNITS_AFTER_OVERWRITE {
        let mut image = clean.clone();
        zero_both_copies_of_the_unit(&mut image, slot, length);
        let verdicts = check_pool_image(&image);
        let i54 = verdicts.iter().find(|(n, _)| *n == "I-5.4").map(|(_, v)| b30_opus_i54_label(v)).unwrap();
        let tag = if slot == TREE_TABLE_AFTER_OVERWRITE { " (B 的树表)" } else if ALLOCATION_LEAVES_AFTER_OVERWRITE.contains(&slot) { " (B 的分配记录树叶)" } else { "" };
        println!("B30UNIT slot={slot}{tag} len={length} violated={:?} I-5.4={i54}", violated_invariants(&verdicts));
    }
}

// ───── b30 攻方腿 M3 探针：池级 checker 与 core 恢复读不读系统配置偏移 433 那 4 字节。覆盖写之后的镜像上，
// 每块盘两个系统配置槽（偏移 0 与 4096）的 [433, 437) 一起换成几个取值、整槽校验和重封；check_pool_image 的整份判定与
// recover 两遍（看 / 不看 journal）的结局要与没换时逐字相同 ─────
#[test]
fn b30_opus_time_threshold_bytes_do_not_move_any_verdict() {
    let clean = image_after_the_overwrite("b30-opus-time-threshold-bytes");
    let base_verdicts = format!("{:?}", check_pool_image(&clean));
    let base_consulted = format!("{:?}", recover(&clean, JournalPolicy::Consult).outcome);
    let base_ignored = format!("{:?}", recover(&clean, JournalPolicy::Ignore).outcome);
    for value in [5u32, 0, 200, 4_999, u32::MAX] {
        let mut image = clean.clone();
        let mut slots_changed = 0;
        for device in DEVICES {
            for slot_offset in [0u64, 4096] {
                let mut slot = read(&image, device, slot_offset, 4096);
                assert_eq!(&slot[433..437], &5000u32.to_le_bytes(), "盘 {device} 偏移 {slot_offset} 的槽今天写 5000");
                slot[433..437].copy_from_slice(&value.to_le_bytes());
                let digest = singlefs_core::checksum::wide_checksum_with_field_zeroed(
                    &slot,
                    4096,
                    singlefs_core::system_configuration::SYSTEM_CONFIGURATION_CHECKSUM_OFFSET,
                );
                slot[155..187].copy_from_slice(&digest);
                write(&mut image, device, slot_offset, &slot);
                slots_changed += 1;
            }
        }
        let verdicts = format!("{:?}", check_pool_image(&image));
        let consulted = format!("{:?}", recover(&image, JournalPolicy::Consult).outcome);
        let ignored = format!("{:?}", recover(&image, JournalPolicy::Ignore).outcome);
        println!(
            "B30TIME value={value} slots_changed={slots_changed} checker_same={} consulted_same={} ignored_same={}",
            verdicts == base_verdicts,
            consulted == base_consulted,
            ignored == base_ignored
        );
        assert_eq!(verdicts, base_verdicts, "T_time = {value}：池级 checker 判定变了");
        assert_eq!(consulted, base_consulted, "T_time = {value}：看 journal 的恢复结局变了");
        assert_eq!(ignored, base_ignored, "T_time = {value}：不看 journal 的恢复结局变了");
    }
}
