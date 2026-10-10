
// ───────────── b30 正推腿 M3 探针：池级 checker 读不读系统配置偏移 [433, 437)（T_time）─────────────
// 每块盘两个槽一起把这 4 字节改成几个值、整槽重封，整份判定向量（Debug 串）与没改的逐字相同。
#[test]
fn b30_m3_the_pool_checker_does_not_read_the_time_threshold_bytes() {
    let spacing = u64::from(parameters().geometry.fixed_structure_slot_spacing);
    let images: Vec<(&str, MemoryPool)> = vec![
        ("overwrite", image_after_the_overwrite("b30-433-overwrite")),
        ("rollback", image_after_rolling_back_to_the_first_root("b30-433-rollback").image),
        ("deletion-intent", common_deletion_in_progress::image_after_the_intent_of_a_three_unit_file("b30-433-intent")),
        ("normal-unmount", image_after_the_normal_unmount("b30-433-unmount")),
    ];
    for (name, image) in images {
        let clean = format!("{:?}", check_pool_image(&image));
        for value in [0u32, 1, 5, 4000, 5000, 200, u32::MAX] {
            let mut changed = image.clone();
            for device in [0u32, 1] {
                for slot_offset in [0, spacing] {
                    mutate_system_configuration_slot(&mut changed, device, slot_offset, |bytes| {
                        bytes[433..437].copy_from_slice(&value.to_le_bytes());
                    });
                }
            }
            let same = format!("{:?}", check_pool_image(&changed)) == clean;
            println!("B30433 image={name} t_time_written={value} verdict_vector_identical_to_clean={same}");
            assert!(same, "{name} 改 T_time 为 {value} 后池级 checker 的判定变了");
        }
    }
}
